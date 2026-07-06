//! Checker-local overload site, template, and viability data layers for phase 8.

use crate::typed_ast::{CoercionId, NormalizedTypeId, TypeFactId, TypedSiteRef};
use mizar_resolve::resolved_ast::{ModuleId, SymbolId};
use mizar_session::SourceRange;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::{self, Write as _},
};

macro_rules! dense_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(usize);

        impl $name {
            pub const fn new(index: usize) -> Self {
                Self(index)
            }

            pub const fn index(self) -> usize {
                self.0
            }
        }
    };
}

macro_rules! string_key {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self::new(value)
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self::new(value)
            }
        }
    };
}

dense_id!(OverloadSiteId);
dense_id!(OverloadCandidateId);
dense_id!(OverloadDiagnosticId);
dense_id!(TemplateExpansionId);
dense_id!(CandidateViabilityId);
dense_id!(SpecificityGraphId);
dense_id!(SpecificityComparisonId);
dense_id!(SpecificityEdgeId);
dense_id!(OverloadResultId);
dense_id!(InsertedViewId);

string_key!(OverloadSiteKey);
string_key!(OverloadNameKey);
string_key!(OverloadDiagnosticMessageKey);
string_key!(CandidateProvenanceKey);
string_key!(TemplateInstantiationKey);
string_key!(TemplateParameterKey);
string_key!(QuaPathKey);
string_key!(ViabilityEvidenceKey);
string_key!(SpecificityReasonKey);
string_key!(SelectionReasonKey);
string_key!(InsertedViewReasonKey);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverloadCollectionOutput {
    sites: OverloadSiteTable,
    candidates: OverloadCandidateTable,
    diagnostics: OverloadDiagnosticTable,
}

impl OverloadCollectionOutput {
    pub fn collect(
        sites: impl IntoIterator<Item = OverloadSiteInput>,
        candidates: impl IntoIterator<Item = OverloadCandidateInput>,
    ) -> Self {
        OverloadCollectionBuilder::new(sites, candidates).finish()
    }

    pub const fn sites(&self) -> &OverloadSiteTable {
        &self.sites
    }

    pub const fn candidates(&self) -> &OverloadCandidateTable {
        &self.candidates
    }

    pub const fn diagnostics(&self) -> &OverloadDiagnosticTable {
        &self.diagnostics
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("overload-collection-debug-v1\n");
        write_sites(&mut output, &self.sites);
        write_candidates(&mut output, &self.candidates);
        write_diagnostics(&mut output, &self.diagnostics);
        output
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateExpansionOutput {
    candidates: OverloadCandidateTable,
    expansions: TemplateExpansionTable,
    diagnostics: OverloadDiagnosticTable,
}

impl TemplateExpansionOutput {
    pub fn expand(collection: &OverloadCollectionOutput) -> Self {
        TemplateExpansionBuilder::new(collection).finish()
    }

    pub const fn candidates(&self) -> &OverloadCandidateTable {
        &self.candidates
    }

    pub const fn expansions(&self) -> &TemplateExpansionTable {
        &self.expansions
    }

    pub const fn diagnostics(&self) -> &OverloadDiagnosticTable {
        &self.diagnostics
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("template-expansion-debug-v1\n");
        write_candidates(&mut output, &self.candidates);
        write_template_expansions(&mut output, &self.expansions);
        write_diagnostics(&mut output, &self.diagnostics);
        output
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateViabilityOutput {
    candidates: OverloadCandidateTable,
    decisions: CandidateViabilityTable,
    diagnostics: OverloadDiagnosticTable,
}

impl CandidateViabilityOutput {
    pub fn filter(
        expansion: &TemplateExpansionOutput,
        inputs: impl IntoIterator<Item = CandidateViabilityInput>,
    ) -> Self {
        CandidateViabilityBuilder::new(expansion, inputs).finish()
    }

    pub const fn candidates(&self) -> &OverloadCandidateTable {
        &self.candidates
    }

    pub const fn decisions(&self) -> &CandidateViabilityTable {
        &self.decisions
    }

    pub const fn diagnostics(&self) -> &OverloadDiagnosticTable {
        &self.diagnostics
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("candidate-viability-debug-v1\n");
        write_candidates(&mut output, &self.candidates);
        write_candidate_viability(&mut output, &self.decisions);
        write_diagnostics(&mut output, &self.diagnostics);
        output
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecificityGraphOutput {
    candidates: OverloadCandidateTable,
    graphs: SpecificityGraphTable,
    diagnostics: OverloadDiagnosticTable,
}

impl SpecificityGraphOutput {
    pub fn build(
        viability: &CandidateViabilityOutput,
        comparisons: impl IntoIterator<Item = SpecificityComparisonInput>,
    ) -> Self {
        SpecificityGraphBuilder::new(viability, comparisons).finish()
    }

    pub const fn candidates(&self) -> &OverloadCandidateTable {
        &self.candidates
    }

    pub const fn graphs(&self) -> &SpecificityGraphTable {
        &self.graphs
    }

    pub const fn diagnostics(&self) -> &OverloadDiagnosticTable {
        &self.diagnostics
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("specificity-graph-debug-v1\n");
        write_candidates(&mut output, &self.candidates);
        write_specificity_graphs(&mut output, &self.graphs);
        write_diagnostics(&mut output, &self.diagnostics);
        output
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverloadSelectionOutput {
    results: OverloadResultTable,
    inserted_views: InsertedViewTable,
    diagnostics: OverloadDiagnosticTable,
}

impl OverloadSelectionOutput {
    pub fn resolve(
        graphs: &SpecificityGraphOutput,
        inputs: impl IntoIterator<Item = OverloadSiteResolutionInput>,
    ) -> Self {
        OverloadSelectionBuilder::new(graphs, inputs).finish()
    }

    pub const fn results(&self) -> &OverloadResultTable {
        &self.results
    }

    pub const fn inserted_views(&self) -> &InsertedViewTable {
        &self.inserted_views
    }

    pub const fn diagnostics(&self) -> &OverloadDiagnosticTable {
        &self.diagnostics
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("overload-selection-debug-v1\n");
        write_overload_results(&mut output, &self.results);
        write_inserted_views(&mut output, &self.inserted_views);
        write_diagnostics(&mut output, &self.diagnostics);
        output
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverloadSiteInput {
    pub key: OverloadSiteKey,
    pub owner: TypedSiteRef,
    pub source_range: SourceRange,
    pub kind: OverloadSiteKind,
    pub name: OverloadNameKey,
    pub arguments: Vec<TypedSiteRef>,
    pub expected: Option<NormalizedTypeId>,
    pub source_qua: Vec<SourceQuaView>,
    pub recovery: OverloadSiteRecovery,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceQuaView {
    pub argument: TypedSiteRef,
    pub target: NormalizedTypeId,
    pub source_range: SourceRange,
    pub path: Option<QuaPathKey>,
    pub evidence_facts: Vec<TypeFactId>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum OverloadSiteKind {
    FunctorApplication,
    PredicateApplication,
    AttributeApplication,
    ModeApplication,
    SelectorApplication,
    StructureFieldApplication,
    TemplateName,
    Unsupported(UnsupportedOverloadRole),
}

impl OverloadSiteKind {
    pub const fn is_supported(&self) -> bool {
        !matches!(self, Self::Unsupported(_))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum UnsupportedOverloadRole {
    SchemeApplication,
    TheoremApplication,
    AlgorithmTemplate,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum OverloadSiteRecovery {
    Normal,
    Degraded {
        message_key: OverloadDiagnosticMessageKey,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverloadCandidateInput {
    pub site: OverloadSiteKey,
    pub symbol: SymbolId,
    pub ordinary_root: SymbolId,
    pub declaration_kind: CandidateDeclarationKind,
    pub parameters: Vec<NormalizedTypeId>,
    pub result: Option<NormalizedTypeId>,
    pub origin: CandidateOrigin,
    pub template: Option<TemplateCandidatePayload>,
    pub coherence: Option<CoherenceStatus>,
    pub provenance: CandidateProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CandidateDeclarationKind {
    Functor,
    Predicate,
    Attribute,
    Mode,
    Selector,
    StructureField,
    Template,
    Redefinition,
    Unsupported(UnsupportedOverloadRole),
}

impl CandidateDeclarationKind {
    pub const fn is_supported(&self) -> bool {
        !matches!(self, Self::Unsupported(_))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CandidateOrigin {
    Ordinary,
    Redefinition {
        refined: SymbolId,
    },
    TemplateDerived {
        template: SymbolId,
        instantiation: TemplateInstantiationKey,
    },
    Recovery(CandidateProvenanceKey),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CoherenceStatus {
    Accepted,
    Pending,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateCandidatePayload {
    pub template: SymbolId,
    pub instantiation_key: TemplateInstantiationKey,
    pub parameters: Vec<TemplateParameterKey>,
    pub arguments: Vec<TemplateArgument>,
    pub inferred_arguments: Vec<TemplateArgumentInference>,
    pub constraints: Vec<TemplateConstraintEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TemplateArgument {
    Explicit(NormalizedTypeId),
    Omitted(TemplateParameterKey),
    SourceQua {
        source: NormalizedTypeId,
        target: NormalizedTypeId,
        path: QuaPathKey,
        status: TemplateQuaStatus,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TemplateQuaStatus {
    AcceptedWidening,
    RejectedNarrowing,
    DeferredExternalDependency,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateArgumentInference {
    pub parameter: TemplateParameterKey,
    pub inferred: NormalizedTypeId,
    pub evidence_key: CandidateProvenanceKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateConstraintEvidence {
    pub parameter: TemplateParameterKey,
    pub evidence_key: CandidateProvenanceKey,
    pub facts: Vec<TypeFactId>,
    pub status: TemplateConstraintEvidenceStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TemplateConstraintEvidenceStatus {
    Accepted,
    Missing,
    DeferredExternalDependency,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateProvenance {
    pub stable_key: CandidateProvenanceKey,
    pub source_range: Option<SourceRange>,
    pub scope: CandidateScope,
    pub declaration_order: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CandidateScope {
    Local,
    Imported {
        module: ModuleId,
        import_key: CandidateProvenanceKey,
    },
    DependencySummary {
        module: ModuleId,
        summary_key: CandidateProvenanceKey,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverloadSite {
    pub id: OverloadSiteId,
    pub key: OverloadSiteKey,
    pub owner: TypedSiteRef,
    pub source_range: SourceRange,
    pub kind: OverloadSiteKind,
    pub name: OverloadNameKey,
    pub arguments: Vec<TypedSiteRef>,
    pub expected: Option<NormalizedTypeId>,
    pub source_qua: Vec<SourceQuaView>,
    pub recovery: OverloadSiteRecovery,
    pub status: OverloadSiteStatus,
    pub diagnostics: Vec<OverloadDiagnosticId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum OverloadSiteStatus {
    Collected,
    Degraded,
    Deferred,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OverloadSiteTable {
    entries: Vec<OverloadSite>,
}

impl OverloadSiteTable {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn get(&self, id: OverloadSiteId) -> Option<&OverloadSite> {
        self.entries.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (OverloadSiteId, &OverloadSite)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub fn canonical_iter(&self) -> impl Iterator<Item = (OverloadSiteId, &OverloadSite)> {
        let mut entries = self.entries.iter().collect::<Vec<_>>();
        entries.sort_by_key(|entry| site_output_key(entry));
        entries.into_iter().map(|entry| (entry.id, entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn push(&mut self, site: OverloadSite) {
        self.entries.push(site);
    }

    fn get_mut(&mut self, id: OverloadSiteId) -> Option<&mut OverloadSite> {
        self.entries.get_mut(id.index())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverloadCandidate {
    pub id: OverloadCandidateId,
    pub site: OverloadSiteId,
    pub site_key: OverloadSiteKey,
    pub symbol: SymbolId,
    pub ordinary_root: SymbolId,
    pub declaration_kind: CandidateDeclarationKind,
    pub parameters: Vec<NormalizedTypeId>,
    pub result: Option<NormalizedTypeId>,
    pub origin: CandidateOrigin,
    pub template: Option<TemplateCandidatePayload>,
    pub coherence: Option<CoherenceStatus>,
    pub provenance: CandidateProvenance,
    pub status: OverloadCandidateStatus,
    pub diagnostics: Vec<OverloadDiagnosticId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum OverloadCandidateStatus {
    Collected,
    Deferred,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OverloadCandidateTable {
    entries: Vec<OverloadCandidate>,
}

impl OverloadCandidateTable {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn get(&self, id: OverloadCandidateId) -> Option<&OverloadCandidate> {
        self.entries.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (OverloadCandidateId, &OverloadCandidate)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub fn canonical_iter(
        &self,
    ) -> impl Iterator<Item = (OverloadCandidateId, &OverloadCandidate)> {
        let mut entries = self.entries.iter().collect::<Vec<_>>();
        entries.sort_by_key(|entry| candidate_output_key(entry));
        entries.into_iter().map(|entry| (entry.id, entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn push(&mut self, candidate: OverloadCandidate) {
        self.entries.push(candidate);
    }

    fn get_mut(&mut self, id: OverloadCandidateId) -> Option<&mut OverloadCandidate> {
        self.entries.get_mut(id.index())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateExpansion {
    pub id: TemplateExpansionId,
    pub source_candidate: OverloadCandidateId,
    pub site: OverloadSiteId,
    pub template: SymbolId,
    pub instantiation_key: TemplateInstantiationKey,
    pub substitutions: Vec<TemplateSubstitution>,
    pub instantiated_candidate: Option<OverloadCandidateId>,
    pub status: TemplateExpansionStatus,
    pub diagnostics: Vec<OverloadDiagnosticId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TemplateExpansionTable {
    entries: Vec<TemplateExpansion>,
}

impl TemplateExpansionTable {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn get(&self, id: TemplateExpansionId) -> Option<&TemplateExpansion> {
        self.entries.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (TemplateExpansionId, &TemplateExpansion)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub fn canonical_iter(
        &self,
    ) -> impl Iterator<Item = (TemplateExpansionId, &TemplateExpansion)> {
        let mut entries = self.entries.iter().collect::<Vec<_>>();
        entries.sort_by_key(|entry| template_expansion_order_key(entry));
        entries.into_iter().map(|entry| (entry.id, entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn push(&mut self, expansion: TemplateExpansion) {
        self.entries.push(expansion);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateSubstitution {
    pub parameter: TemplateParameterKey,
    pub value: NormalizedTypeId,
    pub source: TemplateSubstitutionSource,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TemplateSubstitutionSource {
    Explicit,
    OmittedInference {
        evidence_key: CandidateProvenanceKey,
    },
    SourceQua {
        source: NormalizedTypeId,
        path: QuaPathKey,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TemplateExpansionStatus {
    Instantiated,
    Rejected(TemplateExpansionFailure),
    Deferred(TemplateExpansionFailure),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TemplateExpansionFailure {
    ArityMismatch,
    DuplicateParameter,
    ParameterMismatch,
    MissingInference,
    AmbiguousInference,
    UnknownConstraintParameter,
    MissingConstraintEvidence,
    DeferredConstraintEvidence,
    RejectedSourceQua,
    DeferredSourceQua,
    DeferredCandidate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateViabilityInput {
    pub candidate: OverloadCandidateId,
    pub arguments: Vec<ArgumentViabilityEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ArgumentViabilityEvidence {
    Exact {
        actual: NormalizedTypeId,
    },
    FactWidening {
        actual: NormalizedTypeId,
        target: NormalizedTypeId,
        facts: Vec<TypeFactId>,
        status: ViabilityFactStatus,
    },
    Coercion {
        actual: NormalizedTypeId,
        target: NormalizedTypeId,
        coercion: CoercionId,
        kind: ViabilityCoercionKind,
        status: ViabilityCoercionStatus,
        facts: Vec<TypeFactId>,
        path: Option<QuaPathKey>,
    },
    Missing {
        actual: Option<NormalizedTypeId>,
        target: NormalizedTypeId,
    },
    AmbiguousInheritance {
        actual: NormalizedTypeId,
        target: NormalizedTypeId,
        paths: Vec<QuaPathKey>,
    },
    DeferredExternalDependency {
        actual: Option<NormalizedTypeId>,
        target: NormalizedTypeId,
        reason: ViabilityEvidenceKey,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ViabilityFactStatus {
    Consumable,
    PendingObligation,
    Degraded,
    Rejected,
    OutOfScopeAssumption,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ViabilityCoercionKind {
    Widening,
    SourceQua,
    Narrowing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ViabilityCoercionStatus {
    Accepted,
    PendingObligation,
    Blocked,
    Rejected,
    MissingEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateViability {
    pub id: CandidateViabilityId,
    pub source_candidate: OverloadCandidateId,
    pub site: OverloadSiteId,
    pub output_candidate: Option<OverloadCandidateId>,
    pub status: CandidateViabilityStatus,
    pub diagnostics: Vec<OverloadDiagnosticId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CandidateViabilityTable {
    entries: Vec<CandidateViability>,
}

impl CandidateViabilityTable {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn get(&self, id: CandidateViabilityId) -> Option<&CandidateViability> {
        self.entries.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (CandidateViabilityId, &CandidateViability)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub fn canonical_iter(
        &self,
    ) -> impl Iterator<Item = (CandidateViabilityId, &CandidateViability)> {
        let mut entries = self.entries.iter().collect::<Vec<_>>();
        entries.sort_by_key(|entry| candidate_viability_order_key(entry));
        entries.into_iter().map(|entry| (entry.id, entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn push(&mut self, viability: CandidateViability) {
        self.entries.push(viability);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CandidateViabilityStatus {
    Viable { views: Vec<ArgumentViewPlan> },
    Rejected { reasons: Vec<CandidateRejection> },
    Blocked { reason: CandidateBlockedReason },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgumentViewPlan {
    pub argument_index: usize,
    pub actual: NormalizedTypeId,
    pub target: NormalizedTypeId,
    pub kind: ArgumentViewKind,
    pub facts: Vec<TypeFactId>,
    pub coercion: Option<CoercionId>,
    pub path: Option<QuaPathKey>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ArgumentViewKind {
    Exact,
    FactWidening,
    CoercionWidening,
    SourceQua,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateRejection {
    pub argument_index: usize,
    pub reason: CandidateRejectionReason,
    pub actual: Option<NormalizedTypeId>,
    pub target: Option<NormalizedTypeId>,
    pub facts: Vec<TypeFactId>,
    pub coercion: Option<CoercionId>,
    pub path: Option<QuaPathKey>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CandidateRejectionReason {
    ArityMismatch,
    ParameterMismatch,
    MissingEvidence,
    PendingEvidence,
    DegradedEvidence,
    RejectedEvidence,
    OutOfScopeAssumption,
    InvalidNarrowing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateBlockedReason {
    pub argument_index: Option<usize>,
    pub reason: CandidateBlockedReasonKind,
    pub actual: Option<NormalizedTypeId>,
    pub target: Option<NormalizedTypeId>,
    pub paths: Vec<QuaPathKey>,
    pub detail: Option<ViabilityEvidenceKey>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CandidateBlockedReasonKind {
    CandidateDeferred,
    DuplicateViabilityPayload,
    MissingViabilityPayload,
    AmbiguousInheritance,
    BlockedCoercion,
    DeferredExternalDependency,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecificityComparisonInput {
    pub left: OverloadCandidateId,
    pub right: OverloadCandidateId,
    pub status: SpecificityComparisonStatus,
    pub reasons: Vec<SpecificityReasonKey>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SpecificityComparisonStatus {
    LeftAtLeastRight,
    RightAtLeastLeft,
    Equivalent,
    Incomparable,
    Blocked(SpecificityBlockedReasonKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SpecificityBlockedReasonKind {
    DeferredExternalDependency,
    MissingRecordedFacts,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecificityGraph {
    pub id: SpecificityGraphId,
    pub site: OverloadSiteId,
    pub nodes: Vec<SpecificityNode>,
    pub comparisons: Vec<SpecificityComparison>,
    pub edges: Vec<SpecificityEdge>,
    pub diagnostics: Vec<OverloadDiagnosticId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpecificityGraphTable {
    entries: Vec<SpecificityGraph>,
}

impl SpecificityGraphTable {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn get(&self, id: SpecificityGraphId) -> Option<&SpecificityGraph> {
        self.entries.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (SpecificityGraphId, &SpecificityGraph)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub fn canonical_iter(&self) -> impl Iterator<Item = (SpecificityGraphId, &SpecificityGraph)> {
        let mut entries = self.entries.iter().collect::<Vec<_>>();
        entries.sort_by_key(|entry| specificity_graph_order_key(entry));
        entries.into_iter().map(|entry| (entry.id, entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn push(&mut self, graph: SpecificityGraph) {
        self.entries.push(graph);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecificityNode {
    pub candidate: OverloadCandidateId,
    pub ordinary_root: SymbolId,
    pub parameters: Vec<NormalizedTypeId>,
    pub origin: CandidateOrigin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecificityComparison {
    pub id: SpecificityComparisonId,
    pub left: OverloadCandidateId,
    pub right: OverloadCandidateId,
    pub status: SpecificityComparisonOutcome,
    pub reasons: Vec<SpecificityReasonKey>,
    pub diagnostics: Vec<OverloadDiagnosticId>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SpecificityComparisonOutcome {
    LeftAtLeastRight,
    RightAtLeastLeft,
    Equivalent,
    Incomparable,
    Blocked(SpecificityFailureReason),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecificityEdge {
    pub id: SpecificityEdgeId,
    pub from: OverloadCandidateId,
    pub to: OverloadCandidateId,
    pub reasons: Vec<SpecificityReasonKey>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SpecificityFailureReason {
    MissingComparisonPayload,
    DuplicateComparisonPayload,
    CrossSiteComparison,
    UnknownCandidate,
    DeferredExternalDependency,
    MissingRecordedFacts,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverloadSiteResolutionInput {
    pub site: OverloadSiteId,
    pub refinements: Vec<OverloadCandidateId>,
    pub refinement_join: RefinementJoinPayload,
    pub inserted_views: Vec<InsertedViewInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefinementJoinPayload {
    pub status: RefinementJoinStatus,
    pub exposed_result: Option<ExposedResultPayload>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum RefinementJoinStatus {
    Compatible,
    Incompatible(RefinementJoinFailure),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum RefinementJoinFailure {
    IncompatibleResultRadix,
    ContradictoryAttributes,
    NoUniqueJoinedType,
    MissingJoinPayload,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExposedResultPayload {
    pub result: Option<NormalizedTypeId>,
    pub source: ExposedResultSource,
    pub evidence: Vec<TypeFactId>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ExposedResultSource {
    SelectedRoot,
    StrongestRefinement,
    AttributeUnion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InsertedViewInput {
    pub argument: TypedSiteRef,
    pub target: NormalizedTypeId,
    pub selected_candidate: OverloadCandidateId,
    pub kind: InsertedViewKind,
    pub status: InsertedViewStatus,
    pub reason: InsertedViewReasonKey,
    pub evidence_facts: Vec<TypeFactId>,
    pub path: Option<QuaPathKey>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum InsertedViewKind {
    Widening,
    SourceQua,
    Narrowing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum InsertedViewStatus {
    Accepted,
    MissingEvidence,
    AmbiguousPath,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverloadResult {
    pub id: OverloadResultId,
    pub site: OverloadSiteId,
    pub graph: SpecificityGraphId,
    pub status: OverloadResultStatus,
    pub diagnostics: Vec<OverloadDiagnosticId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OverloadResultTable {
    entries: Vec<OverloadResult>,
}

impl OverloadResultTable {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn get(&self, id: OverloadResultId) -> Option<&OverloadResult> {
        self.entries.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (OverloadResultId, &OverloadResult)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub fn canonical_iter(&self) -> impl Iterator<Item = (OverloadResultId, &OverloadResult)> {
        let mut entries = self.entries.iter().collect::<Vec<_>>();
        entries.sort_by_key(|entry| overload_result_order_key(entry));
        entries.into_iter().map(|entry| (entry.id, entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn push(&mut self, result: OverloadResult) {
        self.entries.push(result);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum OverloadResultStatus {
    Resolved {
        root: OverloadCandidateId,
        refinements: Vec<OverloadCandidateId>,
        exposed_result: Option<ExposedResultPayload>,
        inserted_views: Vec<InsertedViewId>,
    },
    NoMatch {
        rejected: Vec<OverloadCandidateId>,
    },
    Ambiguous {
        candidates: Vec<OverloadCandidateId>,
    },
    IncompatibleRefinementJoin {
        root: OverloadCandidateId,
        refinements: Vec<OverloadCandidateId>,
        reason: RefinementJoinFailure,
    },
    Blocked {
        reason: OverloadBlockedReason,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum OverloadBlockedReason {
    BlockedSpecificityComparison,
    AmbiguousSelection,
    MissingSelectionPayload,
    DuplicateSelectionPayload,
    UnknownSelectionSite,
    MissingOrdinaryRootCandidate,
    AmbiguousOrdinaryRootCandidate,
    NonSelectedRootPayload,
    InvalidInsertedView,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InsertedView {
    pub id: InsertedViewId,
    pub site: OverloadSiteId,
    pub argument: TypedSiteRef,
    pub target: NormalizedTypeId,
    pub selected_candidate: OverloadCandidateId,
    pub kind: InsertedViewKind,
    pub reason: InsertedViewReasonKey,
    pub evidence_facts: Vec<TypeFactId>,
    pub path: Option<QuaPathKey>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InsertedViewTable {
    entries: Vec<InsertedView>,
}

impl InsertedViewTable {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn get(&self, id: InsertedViewId) -> Option<&InsertedView> {
        self.entries.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (InsertedViewId, &InsertedView)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub fn canonical_iter(&self) -> impl Iterator<Item = (InsertedViewId, &InsertedView)> {
        let mut entries = self.entries.iter().collect::<Vec<_>>();
        entries.sort_by_key(|entry| inserted_view_order_key(entry));
        entries.into_iter().map(|entry| (entry.id, entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn insert(&mut self, mut view: InsertedView) -> InsertedViewId {
        let id = InsertedViewId::new(self.entries.len());
        view.id = id;
        self.entries.push(view);
        id
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverloadDiagnostic {
    pub id: OverloadDiagnosticId,
    pub site: Option<OverloadSiteId>,
    pub site_key: Option<OverloadSiteKey>,
    pub candidate: Option<OverloadCandidateId>,
    pub provenance: Option<OverloadDiagnosticProvenance>,
    pub class: OverloadDiagnosticClass,
    pub severity: OverloadDiagnosticSeverity,
    pub message_key: OverloadDiagnosticMessageKey,
    pub recovery: OverloadDiagnosticRecovery,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverloadDiagnosticDraft {
    pub site: Option<OverloadSiteId>,
    pub site_key: Option<OverloadSiteKey>,
    pub candidate: Option<OverloadCandidateId>,
    pub provenance: Option<OverloadDiagnosticProvenance>,
    pub class: OverloadDiagnosticClass,
    pub severity: OverloadDiagnosticSeverity,
    pub message_key: OverloadDiagnosticMessageKey,
    pub recovery: OverloadDiagnosticRecovery,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum OverloadDiagnosticProvenance {
    SiteInput {
        owner: TypedSiteRef,
        source_range: SourceRange,
        kind: OverloadSiteKind,
        name: OverloadNameKey,
        arguments: Vec<TypedSiteRef>,
        source_qua: Vec<SourceQuaView>,
        recovery: OverloadSiteRecovery,
    },
    CandidateInput {
        symbol: SymbolId,
        ordinary_root: SymbolId,
        declaration_kind: CandidateDeclarationKind,
        template: Option<Box<TemplateCandidatePayload>>,
        coherence: Option<CoherenceStatus>,
        provenance: CandidateProvenance,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OverloadDiagnosticTable {
    entries: Vec<OverloadDiagnostic>,
}

impl OverloadDiagnosticTable {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn insert(&mut self, draft: OverloadDiagnosticDraft) -> OverloadDiagnosticId {
        let id = OverloadDiagnosticId::new(self.entries.len());
        self.entries.push(OverloadDiagnostic {
            id,
            site: draft.site,
            site_key: draft.site_key,
            candidate: draft.candidate,
            provenance: draft.provenance,
            class: draft.class,
            severity: draft.severity,
            message_key: draft.message_key,
            recovery: draft.recovery,
        });
        id
    }

    pub fn get(&self, id: OverloadDiagnosticId) -> Option<&OverloadDiagnostic> {
        self.entries.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (OverloadDiagnosticId, &OverloadDiagnostic)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub fn canonical_iter(
        &self,
    ) -> impl Iterator<Item = (OverloadDiagnosticId, &OverloadDiagnostic)> {
        let mut entries = self.entries.iter().collect::<Vec<_>>();
        entries.sort_by_key(|entry| diagnostic_output_key(entry));
        entries.into_iter().map(|entry| (entry.id, entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum OverloadDiagnosticClass {
    DuplicateSiteKey,
    MissingSite,
    UnsupportedSiteRole,
    UnsupportedCandidateRole,
    TemplateExpansion,
    Viability,
    Specificity,
    Selection,
    Recovery,
    ExternalDependencyGap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum OverloadDiagnosticSeverity {
    Error,
    Warning,
    Note,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum OverloadDiagnosticRecovery {
    Normal,
    Degraded,
}

type SiteRefOrderKey = (usize, u8, String);
type RangeOrderKey = (String, usize, usize);
type OptionalRangeOrderKey = (u8, String, usize, usize);
type SiteInputOrderKey = (
    RangeOrderKey,
    SiteRefOrderKey,
    OverloadSiteKind,
    String,
    String,
    usize,
);
type SiteOutputOrderKey = (
    RangeOrderKey,
    SiteRefOrderKey,
    OverloadSiteKind,
    String,
    String,
    usize,
);
type CandidateInputOrderKey = (
    usize,
    String,
    CandidateDeclarationKind,
    SymbolId,
    SymbolId,
    String,
    CandidateProvenanceOrderKey,
    usize,
    usize,
);
type CandidateOutputOrderKey = (
    usize,
    CandidateDeclarationKind,
    SymbolId,
    SymbolId,
    String,
    CandidateProvenanceOrderKey,
    usize,
);
type CandidateProvenanceOrderKey = (u8, String, String, OptionalRangeOrderKey, usize, String);
type DiagnosticOutputOrderKey = (
    u8,
    usize,
    String,
    u8,
    usize,
    OverloadDiagnosticClass,
    String,
    usize,
);
type TemplateExpansionOrderKey = (usize, String, usize);
type CandidateViabilityOrderKey = (usize, usize);
type SpecificityGraphOrderKey = (usize, usize);
type SpecificityComparisonKey = (usize, usize);
type SpecificityEdgeOrderKey = (usize, usize, usize);
type OverloadResultOrderKey = (usize, usize);
type InsertedViewOrderKey = (usize, usize, usize);
type InsertedViewInputOrderKey = (
    SiteRefOrderKey,
    usize,
    usize,
    InsertedViewKind,
    InsertedViewStatus,
    String,
    u8,
    String,
);

struct SiteInputWithOrder {
    input: OverloadSiteInput,
    ordinal: usize,
}

struct CandidateInputWithOrder {
    input: OverloadCandidateInput,
    ordinal: usize,
}

struct OverloadCollectionBuilder {
    site_inputs: Vec<SiteInputWithOrder>,
    candidate_inputs: Vec<CandidateInputWithOrder>,
    sites: OverloadSiteTable,
    candidates: OverloadCandidateTable,
    diagnostics: OverloadDiagnosticTable,
    site_ids: BTreeMap<OverloadSiteKey, OverloadSiteId>,
}

impl OverloadCollectionBuilder {
    fn new(
        sites: impl IntoIterator<Item = OverloadSiteInput>,
        candidates: impl IntoIterator<Item = OverloadCandidateInput>,
    ) -> Self {
        Self {
            site_inputs: sites
                .into_iter()
                .enumerate()
                .map(|(ordinal, input)| SiteInputWithOrder { input, ordinal })
                .collect(),
            candidate_inputs: candidates
                .into_iter()
                .enumerate()
                .map(|(ordinal, input)| CandidateInputWithOrder { input, ordinal })
                .collect(),
            sites: OverloadSiteTable::new(),
            candidates: OverloadCandidateTable::new(),
            diagnostics: OverloadDiagnosticTable::new(),
            site_ids: BTreeMap::new(),
        }
    }

    fn finish(mut self) -> OverloadCollectionOutput {
        self.collect_sites();
        self.collect_candidates();
        OverloadCollectionOutput {
            sites: self.sites,
            candidates: self.candidates,
            diagnostics: self.diagnostics,
        }
    }

    fn collect_sites(&mut self) {
        self.site_inputs
            .sort_by_key(|input| site_input_order_key(&input.input, input.ordinal));
        let inputs = std::mem::take(&mut self.site_inputs);

        for input in inputs {
            if self.site_ids.contains_key(&input.input.key) {
                self.insert_diagnostic(OverloadDiagnosticDraft {
                    site: None,
                    site_key: Some(input.input.key.clone()),
                    candidate: None,
                    provenance: Some(site_diagnostic_provenance(&input.input)),
                    class: OverloadDiagnosticClass::DuplicateSiteKey,
                    severity: OverloadDiagnosticSeverity::Error,
                    message_key: OverloadDiagnosticMessageKey::new("overload.site.duplicate_key"),
                    recovery: OverloadDiagnosticRecovery::Degraded,
                });
                continue;
            }

            let site_id = OverloadSiteId::new(self.sites.len());
            let status = site_status(&input.input);
            let site_key = input.input.key.clone();
            let diagnostic_provenance = site_diagnostic_provenance(&input.input);
            self.site_ids.insert(site_key.clone(), site_id);
            self.sites.push(OverloadSite {
                id: site_id,
                key: site_key.clone(),
                owner: input.input.owner,
                source_range: input.input.source_range,
                kind: input.input.kind.clone(),
                name: input.input.name,
                arguments: input.input.arguments,
                expected: input.input.expected,
                source_qua: input.input.source_qua,
                recovery: input.input.recovery.clone(),
                status,
                diagnostics: Vec::new(),
            });

            if let Some(message_key) = unsupported_site_message_key(&input.input.kind) {
                let diagnostic = self.insert_diagnostic(OverloadDiagnosticDraft {
                    site: Some(site_id),
                    site_key: Some(site_key.clone()),
                    candidate: None,
                    provenance: Some(diagnostic_provenance.clone()),
                    class: OverloadDiagnosticClass::UnsupportedSiteRole,
                    severity: OverloadDiagnosticSeverity::Note,
                    message_key,
                    recovery: OverloadDiagnosticRecovery::Degraded,
                });
                self.attach_site_diagnostic(site_id, diagnostic);
            }

            if let OverloadSiteRecovery::Degraded { message_key } = input.input.recovery {
                let diagnostic = self.insert_diagnostic(OverloadDiagnosticDraft {
                    site: Some(site_id),
                    site_key: Some(site_key),
                    candidate: None,
                    provenance: Some(diagnostic_provenance),
                    class: OverloadDiagnosticClass::Recovery,
                    severity: OverloadDiagnosticSeverity::Note,
                    message_key,
                    recovery: OverloadDiagnosticRecovery::Degraded,
                });
                self.attach_site_diagnostic(site_id, diagnostic);
            }
        }
    }

    fn collect_candidates(&mut self) {
        let site_ids = self.site_ids.clone();
        self.candidate_inputs
            .sort_by_key(|input| candidate_input_order_key(&input.input, input.ordinal, &site_ids));
        let inputs = std::mem::take(&mut self.candidate_inputs);

        for input in inputs {
            let Some(site_id) = self.site_ids.get(&input.input.site).copied() else {
                self.insert_diagnostic(OverloadDiagnosticDraft {
                    site: None,
                    site_key: Some(input.input.site.clone()),
                    candidate: None,
                    provenance: Some(candidate_diagnostic_provenance(&input.input)),
                    class: OverloadDiagnosticClass::MissingSite,
                    severity: OverloadDiagnosticSeverity::Error,
                    message_key: OverloadDiagnosticMessageKey::new(
                        "overload.candidate.missing_site",
                    ),
                    recovery: OverloadDiagnosticRecovery::Degraded,
                });
                continue;
            };

            let diagnostic_provenance = candidate_diagnostic_provenance(&input.input);
            let site_status = self
                .sites
                .get(site_id)
                .map(|site| site.status)
                .unwrap_or(OverloadSiteStatus::Deferred);
            let candidate_id = OverloadCandidateId::new(self.candidates.len());
            let status = if site_status == OverloadSiteStatus::Deferred
                || !input.input.declaration_kind.is_supported()
            {
                OverloadCandidateStatus::Deferred
            } else {
                OverloadCandidateStatus::Collected
            };
            self.candidates.push(OverloadCandidate {
                id: candidate_id,
                site: site_id,
                site_key: input.input.site.clone(),
                symbol: input.input.symbol,
                ordinary_root: input.input.ordinary_root,
                declaration_kind: input.input.declaration_kind.clone(),
                parameters: input.input.parameters,
                result: input.input.result,
                origin: input.input.origin,
                template: input.input.template,
                coherence: input.input.coherence,
                provenance: input.input.provenance,
                status,
                diagnostics: Vec::new(),
            });

            if let Some(message_key) =
                unsupported_candidate_message_key(&input.input.declaration_kind)
            {
                let diagnostic = self.insert_diagnostic(OverloadDiagnosticDraft {
                    site: Some(site_id),
                    site_key: Some(input.input.site),
                    candidate: Some(candidate_id),
                    provenance: Some(diagnostic_provenance),
                    class: OverloadDiagnosticClass::UnsupportedCandidateRole,
                    severity: OverloadDiagnosticSeverity::Note,
                    message_key,
                    recovery: OverloadDiagnosticRecovery::Degraded,
                });
                self.attach_candidate_diagnostic(candidate_id, diagnostic);
            }
        }
    }

    fn insert_diagnostic(&mut self, draft: OverloadDiagnosticDraft) -> OverloadDiagnosticId {
        self.diagnostics.insert(draft)
    }

    fn attach_site_diagnostic(
        &mut self,
        site_id: OverloadSiteId,
        diagnostic: OverloadDiagnosticId,
    ) {
        if let Some(site) = self.sites.get_mut(site_id) {
            site.diagnostics.push(diagnostic);
        }
    }

    fn attach_candidate_diagnostic(
        &mut self,
        candidate_id: OverloadCandidateId,
        diagnostic: OverloadDiagnosticId,
    ) {
        if let Some(candidate) = self.candidates.get_mut(candidate_id) {
            candidate.diagnostics.push(diagnostic);
        }
    }
}

struct TemplateExpansionBuilder<'a> {
    collection: &'a OverloadCollectionOutput,
    candidates: OverloadCandidateTable,
    expansions: TemplateExpansionTable,
    diagnostics: OverloadDiagnosticTable,
}

impl<'a> TemplateExpansionBuilder<'a> {
    fn new(collection: &'a OverloadCollectionOutput) -> Self {
        Self {
            collection,
            candidates: OverloadCandidateTable::new(),
            expansions: TemplateExpansionTable::new(),
            diagnostics: OverloadDiagnosticTable::new(),
        }
    }

    fn finish(mut self) -> TemplateExpansionOutput {
        for (_, candidate) in self.collection.candidates().canonical_iter() {
            if let Some(payload) = &candidate.template {
                self.expand_template_candidate(candidate, payload);
            } else {
                self.copy_concrete_candidate(candidate);
            }
        }
        TemplateExpansionOutput {
            candidates: self.candidates,
            expansions: self.expansions,
            diagnostics: self.diagnostics,
        }
    }

    fn expand_template_candidate(
        &mut self,
        candidate: &OverloadCandidate,
        payload: &TemplateCandidatePayload,
    ) {
        let expansion_id = TemplateExpansionId::new(self.expansions.len());
        let evaluation = evaluate_template_candidate(candidate, payload);
        let (status, substitutions, instantiated_candidate, diagnostics) = match evaluation {
            TemplateEvaluation::Instantiated { substitutions } => {
                let concrete = self.instantiate_candidate(candidate, payload);
                (
                    TemplateExpansionStatus::Instantiated,
                    substitutions,
                    Some(concrete),
                    Vec::new(),
                )
            }
            TemplateEvaluation::Rejected { failure } => {
                let diagnostic = self.insert_template_diagnostic(candidate, &failure, false);
                (
                    TemplateExpansionStatus::Rejected(failure),
                    Vec::new(),
                    None,
                    vec![diagnostic],
                )
            }
            TemplateEvaluation::Deferred { failure } => {
                let diagnostic = self.insert_template_diagnostic(candidate, &failure, true);
                (
                    TemplateExpansionStatus::Deferred(failure),
                    Vec::new(),
                    None,
                    vec![diagnostic],
                )
            }
        };

        self.expansions.push(TemplateExpansion {
            id: expansion_id,
            source_candidate: candidate.id,
            site: candidate.site,
            template: payload.template.clone(),
            instantiation_key: payload.instantiation_key.clone(),
            substitutions,
            instantiated_candidate,
            status,
            diagnostics,
        });
    }

    fn instantiate_candidate(
        &mut self,
        candidate: &OverloadCandidate,
        payload: &TemplateCandidatePayload,
    ) -> OverloadCandidateId {
        let id = OverloadCandidateId::new(self.candidates.len());
        let mut concrete = candidate.clone();
        concrete.id = id;
        concrete.origin = CandidateOrigin::TemplateDerived {
            template: payload.template.clone(),
            instantiation: payload.instantiation_key.clone(),
        };
        concrete.template = None;
        concrete.diagnostics = self.remap_candidate_diagnostics(candidate, id);
        self.candidates.push(concrete);
        id
    }

    fn copy_concrete_candidate(&mut self, candidate: &OverloadCandidate) -> OverloadCandidateId {
        let id = OverloadCandidateId::new(self.candidates.len());
        let mut concrete = candidate.clone();
        concrete.id = id;
        concrete.diagnostics = self.remap_candidate_diagnostics(candidate, id);
        self.candidates.push(concrete);
        id
    }

    fn remap_candidate_diagnostics(
        &mut self,
        candidate: &OverloadCandidate,
        output_candidate: OverloadCandidateId,
    ) -> Vec<OverloadDiagnosticId> {
        candidate
            .diagnostics
            .iter()
            .filter_map(|diagnostic| {
                self.collection
                    .diagnostics()
                    .get(*diagnostic)
                    .map(|diagnostic| {
                        self.diagnostics.insert(OverloadDiagnosticDraft {
                            site: diagnostic.site,
                            site_key: diagnostic.site_key.clone(),
                            candidate: Some(output_candidate),
                            provenance: diagnostic.provenance.clone(),
                            class: diagnostic.class,
                            severity: diagnostic.severity,
                            message_key: diagnostic.message_key.clone(),
                            recovery: diagnostic.recovery,
                        })
                    })
            })
            .collect()
    }

    fn insert_template_diagnostic(
        &mut self,
        candidate: &OverloadCandidate,
        failure: &TemplateExpansionFailure,
        deferred: bool,
    ) -> OverloadDiagnosticId {
        self.diagnostics.insert(OverloadDiagnosticDraft {
            site: Some(candidate.site),
            site_key: Some(candidate.site_key.clone()),
            candidate: None,
            provenance: Some(candidate_diagnostic_provenance_from_candidate(candidate)),
            class: OverloadDiagnosticClass::TemplateExpansion,
            severity: if deferred {
                OverloadDiagnosticSeverity::Note
            } else {
                OverloadDiagnosticSeverity::Error
            },
            message_key: OverloadDiagnosticMessageKey::new(format!(
                "overload.template.{}",
                template_failure_name(failure)
            )),
            recovery: if deferred {
                OverloadDiagnosticRecovery::Degraded
            } else {
                OverloadDiagnosticRecovery::Normal
            },
        })
    }
}

enum TemplateEvaluation {
    Instantiated {
        substitutions: Vec<TemplateSubstitution>,
    },
    Rejected {
        failure: TemplateExpansionFailure,
    },
    Deferred {
        failure: TemplateExpansionFailure,
    },
}

fn evaluate_template_candidate(
    candidate: &OverloadCandidate,
    payload: &TemplateCandidatePayload,
) -> TemplateEvaluation {
    if candidate.status != OverloadCandidateStatus::Collected {
        return TemplateEvaluation::Deferred {
            failure: TemplateExpansionFailure::DeferredCandidate,
        };
    }
    if payload.parameters.len() != payload.arguments.len() {
        return TemplateEvaluation::Rejected {
            failure: TemplateExpansionFailure::ArityMismatch,
        };
    }

    let mut seen_parameters = BTreeMap::new();
    for parameter in &payload.parameters {
        if seen_parameters.insert(parameter.clone(), ()).is_some() {
            return TemplateEvaluation::Rejected {
                failure: TemplateExpansionFailure::DuplicateParameter,
            };
        }
    }

    let mut inferences = BTreeMap::new();
    for inference in &payload.inferred_arguments {
        if inferences
            .insert(inference.parameter.clone(), inference)
            .is_some()
        {
            return TemplateEvaluation::Rejected {
                failure: TemplateExpansionFailure::AmbiguousInference,
            };
        }
    }

    let mut substitutions = Vec::new();
    for (parameter, argument) in payload.parameters.iter().zip(&payload.arguments) {
        match argument {
            TemplateArgument::Explicit(value) => {
                substitutions.push(TemplateSubstitution {
                    parameter: parameter.clone(),
                    value: *value,
                    source: TemplateSubstitutionSource::Explicit,
                });
            }
            TemplateArgument::Omitted(omitted) => {
                if omitted != parameter {
                    return TemplateEvaluation::Rejected {
                        failure: TemplateExpansionFailure::ParameterMismatch,
                    };
                }
                let Some(inference) = inferences.get(parameter) else {
                    return TemplateEvaluation::Rejected {
                        failure: TemplateExpansionFailure::MissingInference,
                    };
                };
                substitutions.push(TemplateSubstitution {
                    parameter: parameter.clone(),
                    value: inference.inferred,
                    source: TemplateSubstitutionSource::OmittedInference {
                        evidence_key: inference.evidence_key.clone(),
                    },
                });
            }
            TemplateArgument::SourceQua {
                source,
                target,
                path,
                status,
            } => match status {
                TemplateQuaStatus::AcceptedWidening => {
                    substitutions.push(TemplateSubstitution {
                        parameter: parameter.clone(),
                        value: *target,
                        source: TemplateSubstitutionSource::SourceQua {
                            source: *source,
                            path: path.clone(),
                        },
                    });
                }
                TemplateQuaStatus::RejectedNarrowing => {
                    return TemplateEvaluation::Rejected {
                        failure: TemplateExpansionFailure::RejectedSourceQua,
                    };
                }
                TemplateQuaStatus::DeferredExternalDependency => {
                    return TemplateEvaluation::Deferred {
                        failure: TemplateExpansionFailure::DeferredSourceQua,
                    };
                }
            },
        }
    }

    for constraint in &payload.constraints {
        if !seen_parameters.contains_key(&constraint.parameter) {
            return TemplateEvaluation::Rejected {
                failure: TemplateExpansionFailure::UnknownConstraintParameter,
            };
        }
        match constraint.status {
            TemplateConstraintEvidenceStatus::Accepted if !constraint.facts.is_empty() => {}
            TemplateConstraintEvidenceStatus::Accepted
            | TemplateConstraintEvidenceStatus::Missing => {
                return TemplateEvaluation::Rejected {
                    failure: TemplateExpansionFailure::MissingConstraintEvidence,
                };
            }
            TemplateConstraintEvidenceStatus::DeferredExternalDependency => {
                return TemplateEvaluation::Deferred {
                    failure: TemplateExpansionFailure::DeferredConstraintEvidence,
                };
            }
        }
    }

    TemplateEvaluation::Instantiated { substitutions }
}

struct CandidateViabilityBuilder<'a> {
    expansion: &'a TemplateExpansionOutput,
    inputs: BTreeMap<OverloadCandidateId, CandidateViabilityInput>,
    duplicate_inputs: BTreeSet<OverloadCandidateId>,
    candidates: OverloadCandidateTable,
    decisions: CandidateViabilityTable,
    diagnostics: OverloadDiagnosticTable,
}

impl<'a> CandidateViabilityBuilder<'a> {
    fn new(
        expansion: &'a TemplateExpansionOutput,
        inputs: impl IntoIterator<Item = CandidateViabilityInput>,
    ) -> Self {
        let mut input_map = BTreeMap::new();
        let mut duplicate_inputs = BTreeSet::new();
        for input in inputs {
            let candidate = input.candidate;
            if input_map.insert(candidate, input).is_some() {
                duplicate_inputs.insert(candidate);
            }
        }
        Self {
            expansion,
            inputs: input_map,
            duplicate_inputs,
            candidates: OverloadCandidateTable::new(),
            decisions: CandidateViabilityTable::new(),
            diagnostics: OverloadDiagnosticTable::new(),
        }
    }

    fn finish(mut self) -> CandidateViabilityOutput {
        self.insert_unknown_input_diagnostics();
        for (_, candidate) in self.expansion.candidates().canonical_iter() {
            self.evaluate_candidate(candidate);
        }
        CandidateViabilityOutput {
            candidates: self.candidates,
            decisions: self.decisions,
            diagnostics: self.diagnostics,
        }
    }

    fn insert_unknown_input_diagnostics(&mut self) {
        let candidate_ids = self
            .expansion
            .candidates()
            .iter()
            .map(|(id, _)| id)
            .collect::<BTreeSet<_>>();
        let unknown_inputs = self
            .inputs
            .keys()
            .filter(|id| !candidate_ids.contains(id))
            .copied()
            .collect::<Vec<_>>();
        for candidate in unknown_inputs {
            self.diagnostics.insert(OverloadDiagnosticDraft {
                site: None,
                site_key: None,
                candidate: None,
                provenance: None,
                class: OverloadDiagnosticClass::Viability,
                severity: OverloadDiagnosticSeverity::Note,
                message_key: OverloadDiagnosticMessageKey::new(format!(
                    "overload.viability.unknown_candidate_input.{}",
                    candidate.index()
                )),
                recovery: OverloadDiagnosticRecovery::Degraded,
            });
        }
    }

    fn evaluate_candidate(&mut self, candidate: &OverloadCandidate) {
        let decision_id = CandidateViabilityId::new(self.decisions.len());
        let evaluation = self.evaluate_candidate_status(candidate);
        let (status, output_candidate, diagnostics) = match evaluation {
            CandidateViabilityStatus::Viable { views } => {
                let output_candidate = self.copy_viable_candidate(candidate);
                (
                    CandidateViabilityStatus::Viable { views },
                    Some(output_candidate),
                    Vec::new(),
                )
            }
            CandidateViabilityStatus::Rejected { reasons } => {
                let diagnostic = self.insert_viability_diagnostic(
                    candidate,
                    Some(viability_rejection_name(
                        reasons
                            .first()
                            .map(|reason| reason.reason)
                            .unwrap_or(CandidateRejectionReason::MissingEvidence),
                    )),
                    false,
                );
                (
                    CandidateViabilityStatus::Rejected { reasons },
                    None,
                    vec![diagnostic],
                )
            }
            CandidateViabilityStatus::Blocked { reason } => {
                let diagnostic = self.insert_viability_diagnostic(
                    candidate,
                    Some(viability_blocked_reason_name(reason.reason)),
                    true,
                );
                (
                    CandidateViabilityStatus::Blocked { reason },
                    None,
                    vec![diagnostic],
                )
            }
        };

        self.decisions.push(CandidateViability {
            id: decision_id,
            source_candidate: candidate.id,
            site: candidate.site,
            output_candidate,
            status,
            diagnostics,
        });
    }

    fn evaluate_candidate_status(&self, candidate: &OverloadCandidate) -> CandidateViabilityStatus {
        if candidate.status != OverloadCandidateStatus::Collected {
            return CandidateViabilityStatus::Blocked {
                reason: CandidateBlockedReason {
                    argument_index: None,
                    reason: CandidateBlockedReasonKind::CandidateDeferred,
                    actual: None,
                    target: None,
                    paths: Vec::new(),
                    detail: None,
                },
            };
        }

        if self.duplicate_inputs.contains(&candidate.id) {
            return CandidateViabilityStatus::Blocked {
                reason: CandidateBlockedReason {
                    argument_index: None,
                    reason: CandidateBlockedReasonKind::DuplicateViabilityPayload,
                    actual: None,
                    target: None,
                    paths: Vec::new(),
                    detail: None,
                },
            };
        }

        let Some(input) = self.inputs.get(&candidate.id) else {
            return CandidateViabilityStatus::Blocked {
                reason: CandidateBlockedReason {
                    argument_index: None,
                    reason: CandidateBlockedReasonKind::MissingViabilityPayload,
                    actual: None,
                    target: None,
                    paths: Vec::new(),
                    detail: None,
                },
            };
        };

        if input.arguments.len() != candidate.parameters.len() {
            return CandidateViabilityStatus::Rejected {
                reasons: vec![CandidateRejection {
                    argument_index: input.arguments.len().min(candidate.parameters.len()),
                    reason: CandidateRejectionReason::ArityMismatch,
                    actual: None,
                    target: candidate
                        .parameters
                        .get(input.arguments.len().min(candidate.parameters.len()))
                        .copied(),
                    facts: Vec::new(),
                    coercion: None,
                    path: None,
                }],
            };
        }

        let mut views = Vec::new();
        let mut rejections = Vec::new();
        for (index, (target, evidence)) in candidate
            .parameters
            .iter()
            .copied()
            .zip(&input.arguments)
            .enumerate()
        {
            match evaluate_argument_viability(index, target, evidence) {
                ArgumentViabilityEvaluation::Accepted(view) => views.push(view),
                ArgumentViabilityEvaluation::Rejected(rejection) => rejections.push(rejection),
                ArgumentViabilityEvaluation::Blocked(reason) => {
                    return CandidateViabilityStatus::Blocked { reason };
                }
            }
        }

        if rejections.is_empty() {
            CandidateViabilityStatus::Viable { views }
        } else {
            CandidateViabilityStatus::Rejected {
                reasons: rejections,
            }
        }
    }

    fn copy_viable_candidate(&mut self, candidate: &OverloadCandidate) -> OverloadCandidateId {
        let id = OverloadCandidateId::new(self.candidates.len());
        let mut viable = candidate.clone();
        viable.id = id;
        viable.diagnostics = self.remap_candidate_diagnostics(candidate, id);
        self.candidates.push(viable);
        id
    }

    fn remap_candidate_diagnostics(
        &mut self,
        candidate: &OverloadCandidate,
        output_candidate: OverloadCandidateId,
    ) -> Vec<OverloadDiagnosticId> {
        candidate
            .diagnostics
            .iter()
            .filter_map(|diagnostic| {
                self.expansion
                    .diagnostics()
                    .get(*diagnostic)
                    .map(|diagnostic| {
                        self.diagnostics.insert(OverloadDiagnosticDraft {
                            site: diagnostic.site,
                            site_key: diagnostic.site_key.clone(),
                            candidate: Some(output_candidate),
                            provenance: diagnostic.provenance.clone(),
                            class: diagnostic.class,
                            severity: diagnostic.severity,
                            message_key: diagnostic.message_key.clone(),
                            recovery: diagnostic.recovery,
                        })
                    })
            })
            .collect()
    }

    fn insert_viability_diagnostic(
        &mut self,
        candidate: &OverloadCandidate,
        reason: Option<&str>,
        blocked: bool,
    ) -> OverloadDiagnosticId {
        let reason = reason.unwrap_or("unknown");
        self.diagnostics.insert(OverloadDiagnosticDraft {
            site: Some(candidate.site),
            site_key: Some(candidate.site_key.clone()),
            candidate: None,
            provenance: Some(candidate_diagnostic_provenance_from_candidate(candidate)),
            class: OverloadDiagnosticClass::Viability,
            severity: if blocked {
                OverloadDiagnosticSeverity::Note
            } else {
                OverloadDiagnosticSeverity::Error
            },
            message_key: OverloadDiagnosticMessageKey::new(format!("overload.viability.{reason}")),
            recovery: if blocked {
                OverloadDiagnosticRecovery::Degraded
            } else {
                OverloadDiagnosticRecovery::Normal
            },
        })
    }
}

enum ArgumentViabilityEvaluation {
    Accepted(ArgumentViewPlan),
    Rejected(CandidateRejection),
    Blocked(CandidateBlockedReason),
}

fn evaluate_argument_viability(
    argument_index: usize,
    parameter: NormalizedTypeId,
    evidence: &ArgumentViabilityEvidence,
) -> ArgumentViabilityEvaluation {
    match evidence {
        ArgumentViabilityEvidence::Exact { actual } if *actual == parameter => {
            ArgumentViabilityEvaluation::Accepted(ArgumentViewPlan {
                argument_index,
                actual: *actual,
                target: parameter,
                kind: ArgumentViewKind::Exact,
                facts: Vec::new(),
                coercion: None,
                path: None,
            })
        }
        ArgumentViabilityEvidence::Exact { actual } => {
            ArgumentViabilityEvaluation::Rejected(CandidateRejection {
                argument_index,
                reason: CandidateRejectionReason::MissingEvidence,
                actual: Some(*actual),
                target: Some(parameter),
                facts: Vec::new(),
                coercion: None,
                path: None,
            })
        }
        ArgumentViabilityEvidence::FactWidening {
            actual,
            target,
            facts,
            status,
        } => evaluate_fact_viability(argument_index, parameter, *actual, *target, facts, *status),
        ArgumentViabilityEvidence::Coercion {
            actual,
            target,
            coercion,
            kind,
            status,
            facts,
            path,
        } => evaluate_coercion_viability(
            argument_index,
            parameter,
            CoercionViabilityEvidence {
                actual: *actual,
                target: *target,
                coercion: *coercion,
                kind: *kind,
                status: *status,
                facts,
                path: path.clone(),
            },
        ),
        ArgumentViabilityEvidence::Missing { actual, target } => {
            if *target != parameter {
                return parameter_mismatch(argument_index, *actual, parameter, *target);
            }
            ArgumentViabilityEvaluation::Rejected(CandidateRejection {
                argument_index,
                reason: CandidateRejectionReason::MissingEvidence,
                actual: *actual,
                target: Some(parameter),
                facts: Vec::new(),
                coercion: None,
                path: None,
            })
        }
        ArgumentViabilityEvidence::AmbiguousInheritance {
            actual,
            target,
            paths,
        } => {
            if *target != parameter {
                return parameter_mismatch(argument_index, Some(*actual), parameter, *target);
            }
            ArgumentViabilityEvaluation::Blocked(CandidateBlockedReason {
                argument_index: Some(argument_index),
                reason: CandidateBlockedReasonKind::AmbiguousInheritance,
                actual: Some(*actual),
                target: Some(parameter),
                paths: paths.clone(),
                detail: None,
            })
        }
        ArgumentViabilityEvidence::DeferredExternalDependency {
            actual,
            target,
            reason,
        } => {
            if *target != parameter {
                return parameter_mismatch(argument_index, *actual, parameter, *target);
            }
            ArgumentViabilityEvaluation::Blocked(CandidateBlockedReason {
                argument_index: Some(argument_index),
                reason: CandidateBlockedReasonKind::DeferredExternalDependency,
                actual: *actual,
                target: Some(parameter),
                paths: Vec::new(),
                detail: Some(reason.clone()),
            })
        }
    }
}

fn evaluate_fact_viability(
    argument_index: usize,
    parameter: NormalizedTypeId,
    actual: NormalizedTypeId,
    target: NormalizedTypeId,
    facts: &[TypeFactId],
    status: ViabilityFactStatus,
) -> ArgumentViabilityEvaluation {
    if target != parameter {
        return parameter_mismatch(argument_index, Some(actual), parameter, target);
    }
    match status {
        ViabilityFactStatus::Consumable if !facts.is_empty() => {
            ArgumentViabilityEvaluation::Accepted(ArgumentViewPlan {
                argument_index,
                actual,
                target: parameter,
                kind: ArgumentViewKind::FactWidening,
                facts: facts.to_vec(),
                coercion: None,
                path: None,
            })
        }
        ViabilityFactStatus::Consumable => {
            ArgumentViabilityEvaluation::Rejected(CandidateRejection {
                argument_index,
                reason: CandidateRejectionReason::MissingEvidence,
                actual: Some(actual),
                target: Some(parameter),
                facts: facts.to_vec(),
                coercion: None,
                path: None,
            })
        }
        ViabilityFactStatus::PendingObligation => {
            ArgumentViabilityEvaluation::Rejected(CandidateRejection {
                argument_index,
                reason: CandidateRejectionReason::PendingEvidence,
                actual: Some(actual),
                target: Some(parameter),
                facts: facts.to_vec(),
                coercion: None,
                path: None,
            })
        }
        ViabilityFactStatus::Degraded => {
            ArgumentViabilityEvaluation::Rejected(CandidateRejection {
                argument_index,
                reason: CandidateRejectionReason::DegradedEvidence,
                actual: Some(actual),
                target: Some(parameter),
                facts: facts.to_vec(),
                coercion: None,
                path: None,
            })
        }
        ViabilityFactStatus::Rejected => {
            ArgumentViabilityEvaluation::Rejected(CandidateRejection {
                argument_index,
                reason: CandidateRejectionReason::RejectedEvidence,
                actual: Some(actual),
                target: Some(parameter),
                facts: facts.to_vec(),
                coercion: None,
                path: None,
            })
        }
        ViabilityFactStatus::OutOfScopeAssumption => {
            ArgumentViabilityEvaluation::Rejected(CandidateRejection {
                argument_index,
                reason: CandidateRejectionReason::OutOfScopeAssumption,
                actual: Some(actual),
                target: Some(parameter),
                facts: facts.to_vec(),
                coercion: None,
                path: None,
            })
        }
    }
}

struct CoercionViabilityEvidence<'a> {
    actual: NormalizedTypeId,
    target: NormalizedTypeId,
    coercion: CoercionId,
    kind: ViabilityCoercionKind,
    status: ViabilityCoercionStatus,
    facts: &'a [TypeFactId],
    path: Option<QuaPathKey>,
}

fn evaluate_coercion_viability(
    argument_index: usize,
    parameter: NormalizedTypeId,
    evidence: CoercionViabilityEvidence<'_>,
) -> ArgumentViabilityEvaluation {
    let CoercionViabilityEvidence {
        actual,
        target,
        coercion,
        kind,
        status,
        facts,
        path,
    } = evidence;

    if target != parameter {
        return parameter_mismatch(argument_index, Some(actual), parameter, target);
    }
    if kind == ViabilityCoercionKind::Narrowing {
        return ArgumentViabilityEvaluation::Rejected(CandidateRejection {
            argument_index,
            reason: CandidateRejectionReason::InvalidNarrowing,
            actual: Some(actual),
            target: Some(parameter),
            facts: facts.to_vec(),
            coercion: Some(coercion),
            path,
        });
    }
    match status {
        ViabilityCoercionStatus::Accepted => {
            let view_kind = match kind {
                ViabilityCoercionKind::Widening => ArgumentViewKind::CoercionWidening,
                ViabilityCoercionKind::SourceQua => ArgumentViewKind::SourceQua,
                ViabilityCoercionKind::Narrowing => unreachable!("handled above"),
            };
            ArgumentViabilityEvaluation::Accepted(ArgumentViewPlan {
                argument_index,
                actual,
                target: parameter,
                kind: view_kind,
                facts: facts.to_vec(),
                coercion: Some(coercion),
                path,
            })
        }
        ViabilityCoercionStatus::PendingObligation => {
            ArgumentViabilityEvaluation::Rejected(CandidateRejection {
                argument_index,
                reason: CandidateRejectionReason::PendingEvidence,
                actual: Some(actual),
                target: Some(parameter),
                facts: facts.to_vec(),
                coercion: Some(coercion),
                path,
            })
        }
        ViabilityCoercionStatus::Blocked => {
            ArgumentViabilityEvaluation::Blocked(CandidateBlockedReason {
                argument_index: Some(argument_index),
                reason: CandidateBlockedReasonKind::BlockedCoercion,
                actual: Some(actual),
                target: Some(parameter),
                paths: path.into_iter().collect(),
                detail: None,
            })
        }
        ViabilityCoercionStatus::Rejected => {
            ArgumentViabilityEvaluation::Rejected(CandidateRejection {
                argument_index,
                reason: CandidateRejectionReason::RejectedEvidence,
                actual: Some(actual),
                target: Some(parameter),
                facts: facts.to_vec(),
                coercion: Some(coercion),
                path,
            })
        }
        ViabilityCoercionStatus::MissingEvidence => {
            ArgumentViabilityEvaluation::Rejected(CandidateRejection {
                argument_index,
                reason: CandidateRejectionReason::MissingEvidence,
                actual: Some(actual),
                target: Some(parameter),
                facts: facts.to_vec(),
                coercion: Some(coercion),
                path,
            })
        }
    }
}

fn parameter_mismatch(
    argument_index: usize,
    actual: Option<NormalizedTypeId>,
    parameter: NormalizedTypeId,
    supplied_target: NormalizedTypeId,
) -> ArgumentViabilityEvaluation {
    ArgumentViabilityEvaluation::Rejected(CandidateRejection {
        argument_index,
        reason: CandidateRejectionReason::ParameterMismatch,
        actual,
        target: Some(parameter),
        facts: Vec::new(),
        coercion: None,
        path: Some(QuaPathKey::new(format!(
            "payload-target:{}",
            supplied_target.index()
        ))),
    })
}

struct SpecificityGraphBuilder {
    comparisons: BTreeMap<SpecificityComparisonKey, SpecificityComparisonInput>,
    duplicate_comparisons: BTreeSet<SpecificityComparisonKey>,
    candidates: OverloadCandidateTable,
    site_candidates: BTreeMap<OverloadSiteId, Vec<OverloadCandidateId>>,
    graphs: SpecificityGraphTable,
    diagnostics: OverloadDiagnosticTable,
}

impl SpecificityGraphBuilder {
    fn new(
        viability: &CandidateViabilityOutput,
        comparisons: impl IntoIterator<Item = SpecificityComparisonInput>,
    ) -> Self {
        let mut comparison_map = BTreeMap::new();
        let mut duplicate_comparisons = BTreeSet::new();
        for comparison in comparisons {
            let key = specificity_comparison_key(comparison.left, comparison.right);
            if comparison_map.insert(key, comparison).is_some() {
                duplicate_comparisons.insert(key);
            }
        }
        let mut site_candidates = BTreeMap::<OverloadSiteId, Vec<OverloadCandidateId>>::new();
        for (_, decision) in viability.decisions().canonical_iter() {
            let candidates = site_candidates.entry(decision.site).or_default();
            if let Some(candidate) = decision.output_candidate {
                candidates.push(candidate);
            }
        }
        Self {
            comparisons: comparison_map,
            duplicate_comparisons,
            candidates: viability.candidates().clone(),
            site_candidates,
            graphs: SpecificityGraphTable::new(),
            diagnostics: viability.diagnostics().clone(),
        }
    }

    fn finish(mut self) -> SpecificityGraphOutput {
        self.insert_global_input_diagnostics();
        for (site, candidates) in std::mem::take(&mut self.site_candidates) {
            self.build_site_graph(site, &candidates);
        }
        SpecificityGraphOutput {
            candidates: self.candidates,
            graphs: self.graphs,
            diagnostics: self.diagnostics,
        }
    }

    fn insert_global_input_diagnostics(&mut self) {
        let candidate_ids = self
            .candidates
            .iter()
            .map(|(id, _)| id)
            .collect::<BTreeSet<_>>();
        let duplicate_inputs = self
            .duplicate_comparisons
            .iter()
            .copied()
            .collect::<Vec<_>>();
        for (left, right) in duplicate_inputs {
            let left = self.candidates.get(OverloadCandidateId::new(left));
            let right = self.candidates.get(OverloadCandidateId::new(right));
            if !matches!((left, right), (Some(left), Some(right)) if left.site == right.site) {
                self.insert_specificity_diagnostic(
                    None,
                    SpecificityFailureReason::DuplicateComparisonPayload,
                );
            }
        }
        let inputs = self.comparisons.values().cloned().collect::<Vec<_>>();
        for input in inputs {
            let left = self.candidates.get(input.left);
            let right = self.candidates.get(input.right);
            match (left, right) {
                (Some(left), Some(right)) if left.site != right.site => {
                    self.insert_specificity_diagnostic(
                        None,
                        SpecificityFailureReason::CrossSiteComparison,
                    );
                }
                _ if !candidate_ids.contains(&input.left)
                    || !candidate_ids.contains(&input.right) =>
                {
                    self.insert_specificity_diagnostic(
                        None,
                        SpecificityFailureReason::UnknownCandidate,
                    );
                }
                _ => {}
            }
        }
    }

    fn build_site_graph(&mut self, site: OverloadSiteId, candidates: &[OverloadCandidateId]) {
        let graph_id = SpecificityGraphId::new(self.graphs.len());
        let nodes = candidates
            .iter()
            .filter_map(|candidate| {
                self.candidates
                    .get(*candidate)
                    .map(|entry| (candidate, entry))
            })
            .map(|(candidate, entry)| SpecificityNode {
                candidate: *candidate,
                ordinary_root: entry.ordinary_root.clone(),
                parameters: entry.parameters.clone(),
                origin: entry.origin.clone(),
            })
            .collect::<Vec<_>>();
        let mut comparisons = Vec::new();
        let mut edges = Vec::new();
        let mut diagnostics = Vec::new();

        for left_index in 0..candidates.len() {
            for right_index in (left_index + 1)..candidates.len() {
                let left = candidates[left_index];
                let right = candidates[right_index];
                let comparison_id = SpecificityComparisonId::new(comparisons.len());
                let (comparison, new_edges, new_diagnostics) =
                    self.evaluate_pair(site, comparison_id, left, right);
                for mut edge in new_edges {
                    edge.id = SpecificityEdgeId::new(edges.len());
                    edges.push(edge);
                }
                diagnostics.extend(new_diagnostics.iter().copied());
                comparisons.push(comparison);
            }
        }

        self.graphs.push(SpecificityGraph {
            id: graph_id,
            site,
            nodes,
            comparisons,
            edges,
            diagnostics,
        });
    }

    fn evaluate_pair(
        &mut self,
        site: OverloadSiteId,
        comparison_id: SpecificityComparisonId,
        left: OverloadCandidateId,
        right: OverloadCandidateId,
    ) -> (
        SpecificityComparison,
        Vec<SpecificityEdge>,
        Vec<OverloadDiagnosticId>,
    ) {
        let key = specificity_comparison_key(left, right);
        if self.duplicate_comparisons.contains(&key) {
            let diagnostic = self.insert_specificity_diagnostic(
                Some(site),
                SpecificityFailureReason::DuplicateComparisonPayload,
            );
            return (
                blocked_specificity_comparison(
                    comparison_id,
                    left,
                    right,
                    SpecificityFailureReason::DuplicateComparisonPayload,
                    vec![diagnostic],
                ),
                Vec::new(),
                vec![diagnostic],
            );
        }

        let Some(input) = self.comparisons.get(&key) else {
            let diagnostic = self.insert_specificity_diagnostic(
                Some(site),
                SpecificityFailureReason::MissingComparisonPayload,
            );
            return (
                blocked_specificity_comparison(
                    comparison_id,
                    left,
                    right,
                    SpecificityFailureReason::MissingComparisonPayload,
                    vec![diagnostic],
                ),
                Vec::new(),
                vec![diagnostic],
            );
        };

        let (status, edges) = orient_specificity_status(input, left, right);
        (
            SpecificityComparison {
                id: comparison_id,
                left,
                right,
                status,
                reasons: input.reasons.clone(),
                diagnostics: Vec::new(),
            },
            edges,
            Vec::new(),
        )
    }

    fn insert_specificity_diagnostic(
        &mut self,
        site: Option<OverloadSiteId>,
        reason: SpecificityFailureReason,
    ) -> OverloadDiagnosticId {
        self.diagnostics.insert(OverloadDiagnosticDraft {
            site,
            site_key: None,
            candidate: None,
            provenance: None,
            class: OverloadDiagnosticClass::Specificity,
            severity: OverloadDiagnosticSeverity::Note,
            message_key: OverloadDiagnosticMessageKey::new(format!(
                "overload.specificity.{}",
                specificity_failure_name(&reason)
            )),
            recovery: OverloadDiagnosticRecovery::Degraded,
        })
    }
}

fn orient_specificity_status(
    input: &SpecificityComparisonInput,
    left: OverloadCandidateId,
    right: OverloadCandidateId,
) -> (SpecificityComparisonOutcome, Vec<SpecificityEdge>) {
    let input_reversed = input.left == right && input.right == left;
    let reasons = input.reasons.clone();
    match &input.status {
        SpecificityComparisonStatus::LeftAtLeastRight if input_reversed => (
            SpecificityComparisonOutcome::RightAtLeastLeft,
            vec![specificity_edge(right, left, reasons)],
        ),
        SpecificityComparisonStatus::LeftAtLeastRight => (
            SpecificityComparisonOutcome::LeftAtLeastRight,
            vec![specificity_edge(left, right, reasons)],
        ),
        SpecificityComparisonStatus::RightAtLeastLeft if input_reversed => (
            SpecificityComparisonOutcome::LeftAtLeastRight,
            vec![specificity_edge(left, right, reasons)],
        ),
        SpecificityComparisonStatus::RightAtLeastLeft => (
            SpecificityComparisonOutcome::RightAtLeastLeft,
            vec![specificity_edge(right, left, reasons)],
        ),
        SpecificityComparisonStatus::Equivalent => (
            SpecificityComparisonOutcome::Equivalent,
            vec![
                specificity_edge(left, right, reasons.clone()),
                specificity_edge(right, left, reasons),
            ],
        ),
        SpecificityComparisonStatus::Incomparable => {
            (SpecificityComparisonOutcome::Incomparable, Vec::new())
        }
        SpecificityComparisonStatus::Blocked(reason) => (
            SpecificityComparisonOutcome::Blocked(match reason {
                SpecificityBlockedReasonKind::DeferredExternalDependency => {
                    SpecificityFailureReason::DeferredExternalDependency
                }
                SpecificityBlockedReasonKind::MissingRecordedFacts => {
                    SpecificityFailureReason::MissingRecordedFacts
                }
            }),
            Vec::new(),
        ),
    }
}

fn blocked_specificity_comparison(
    id: SpecificityComparisonId,
    left: OverloadCandidateId,
    right: OverloadCandidateId,
    reason: SpecificityFailureReason,
    diagnostics: Vec<OverloadDiagnosticId>,
) -> SpecificityComparison {
    SpecificityComparison {
        id,
        left,
        right,
        status: SpecificityComparisonOutcome::Blocked(reason),
        reasons: Vec::new(),
        diagnostics,
    }
}

fn specificity_edge(
    from: OverloadCandidateId,
    to: OverloadCandidateId,
    reasons: Vec<SpecificityReasonKey>,
) -> SpecificityEdge {
    SpecificityEdge {
        id: SpecificityEdgeId::new(0),
        from,
        to,
        reasons,
    }
}

fn specificity_comparison_key(
    left: OverloadCandidateId,
    right: OverloadCandidateId,
) -> SpecificityComparisonKey {
    let left = left.index();
    let right = right.index();
    if left <= right {
        (left, right)
    } else {
        (right, left)
    }
}

struct OverloadSelectionBuilder {
    graphs: SpecificityGraphOutput,
    inputs: BTreeMap<OverloadSiteId, OverloadSiteResolutionInput>,
    duplicate_inputs: BTreeSet<OverloadSiteId>,
    results: OverloadResultTable,
    inserted_views: InsertedViewTable,
    diagnostics: OverloadDiagnosticTable,
}

impl OverloadSelectionBuilder {
    fn new(
        graphs: &SpecificityGraphOutput,
        inputs: impl IntoIterator<Item = OverloadSiteResolutionInput>,
    ) -> Self {
        let mut input_map = BTreeMap::new();
        let mut duplicate_inputs = BTreeSet::new();
        for input in inputs {
            let site = input.site;
            if input_map.insert(site, input).is_some() {
                duplicate_inputs.insert(site);
            }
        }
        Self {
            graphs: graphs.clone(),
            inputs: input_map,
            duplicate_inputs,
            results: OverloadResultTable::new(),
            inserted_views: InsertedViewTable::new(),
            diagnostics: graphs.diagnostics().clone(),
        }
    }

    fn finish(mut self) -> OverloadSelectionOutput {
        self.insert_unknown_input_diagnostics();
        let graphs = self
            .graphs
            .graphs()
            .canonical_iter()
            .map(|(_, graph)| graph.clone())
            .collect::<Vec<_>>();
        for graph in graphs {
            self.resolve_graph(&graph);
        }
        OverloadSelectionOutput {
            results: self.results,
            inserted_views: self.inserted_views,
            diagnostics: self.diagnostics,
        }
    }

    fn insert_unknown_input_diagnostics(&mut self) {
        let graph_sites = self
            .graphs
            .graphs()
            .iter()
            .map(|(_, graph)| graph.site)
            .collect::<BTreeSet<_>>();
        let unknown_sites = self
            .inputs
            .keys()
            .filter(|site| !graph_sites.contains(site))
            .copied()
            .collect::<Vec<_>>();
        for _site in unknown_sites {
            self.insert_selection_diagnostic(None, OverloadBlockedReason::UnknownSelectionSite);
        }
    }

    fn resolve_graph(&mut self, graph: &SpecificityGraph) {
        let result_id = OverloadResultId::new(self.results.len());
        let (status, diagnostics) = self.resolve_graph_status(graph);
        self.results.push(OverloadResult {
            id: result_id,
            site: graph.site,
            graph: graph.id,
            status,
            diagnostics,
        });
    }

    fn resolve_graph_status(
        &mut self,
        graph: &SpecificityGraph,
    ) -> (OverloadResultStatus, Vec<OverloadDiagnosticId>) {
        if self.duplicate_inputs.contains(&graph.site) {
            let diagnostic = self.insert_selection_diagnostic(
                Some(graph.site),
                OverloadBlockedReason::DuplicateSelectionPayload,
            );
            return (
                OverloadResultStatus::Blocked {
                    reason: OverloadBlockedReason::DuplicateSelectionPayload,
                },
                vec![diagnostic],
            );
        }

        if graph.nodes.is_empty() {
            return (
                OverloadResultStatus::NoMatch {
                    rejected: Vec::new(),
                },
                Vec::new(),
            );
        }

        if graph
            .comparisons
            .iter()
            .any(|comparison| matches!(comparison.status, SpecificityComparisonOutcome::Blocked(_)))
        {
            let diagnostic = self.insert_selection_diagnostic(
                Some(graph.site),
                OverloadBlockedReason::BlockedSpecificityComparison,
            );
            return (
                OverloadResultStatus::Blocked {
                    reason: OverloadBlockedReason::BlockedSpecificityComparison,
                },
                vec![diagnostic],
            );
        }

        let maximal = maximal_candidates(graph);
        let maximal_non_redefinition_roots = maximal
            .iter()
            .filter_map(|candidate| {
                graph
                    .nodes
                    .iter()
                    .find(|node| node.candidate == *candidate)
                    .filter(|node| !matches!(node.origin, CandidateOrigin::Redefinition { .. }))
                    .map(|node| node.ordinary_root.clone())
            })
            .collect::<BTreeSet<_>>();
        if maximal.iter().any(|candidate| {
            graph.nodes.iter().any(|node| {
                node.candidate == *candidate
                    && matches!(node.origin, CandidateOrigin::Redefinition { .. })
                    && !maximal_non_redefinition_roots.contains(&node.ordinary_root)
            })
        }) {
            let diagnostic = self.insert_selection_diagnostic(
                Some(graph.site),
                OverloadBlockedReason::MissingOrdinaryRootCandidate,
            );
            return (
                OverloadResultStatus::Blocked {
                    reason: OverloadBlockedReason::MissingOrdinaryRootCandidate,
                },
                vec![diagnostic],
            );
        }
        if maximal_non_redefinition_roots.len() > 1 {
            let diagnostic = self.insert_selection_diagnostic(
                Some(graph.site),
                OverloadBlockedReason::AmbiguousSelection,
            );
            return (
                OverloadResultStatus::Ambiguous {
                    candidates: maximal,
                },
                vec![diagnostic],
            );
        }

        let root = match choose_selected_root(graph, &maximal) {
            Ok(root) => root,
            Err(reason) => {
                let diagnostic = self.insert_selection_diagnostic(Some(graph.site), reason.clone());
                return (OverloadResultStatus::Blocked { reason }, vec![diagnostic]);
            }
        };

        let Some(mut input) = self.inputs.get(&graph.site).cloned() else {
            let diagnostic = self.insert_selection_diagnostic(
                Some(graph.site),
                OverloadBlockedReason::MissingSelectionPayload,
            );
            return (
                OverloadResultStatus::Blocked {
                    reason: OverloadBlockedReason::MissingSelectionPayload,
                },
                vec![diagnostic],
            );
        };
        input.refinements.sort_by_key(|candidate| candidate.index());
        input
            .inserted_views
            .sort_by_key(inserted_view_input_order_key);

        let selected_root = graph
            .nodes
            .iter()
            .find(|node| node.candidate == root)
            .map(|node| node.ordinary_root.clone())
            .expect("selected root node");
        if !input.refinements.iter().all(|candidate| {
            self.is_active_refinement_candidate(graph, *candidate, &selected_root, root)
        }) {
            let diagnostic = self.insert_selection_diagnostic(
                Some(graph.site),
                OverloadBlockedReason::NonSelectedRootPayload,
            );
            return (
                OverloadResultStatus::Blocked {
                    reason: OverloadBlockedReason::NonSelectedRootPayload,
                },
                vec![diagnostic],
            );
        }

        if let RefinementJoinStatus::Incompatible(reason) = input.refinement_join.status {
            let diagnostic = self.insert_refinement_diagnostic(Some(graph.site), &reason);
            return (
                OverloadResultStatus::IncompatibleRefinementJoin {
                    root,
                    refinements: input.refinements,
                    reason,
                },
                vec![diagnostic],
            );
        }

        let Some(inserted_views) = self.insert_views(graph.site, root, &input) else {
            let diagnostic = self.insert_selection_diagnostic(
                Some(graph.site),
                OverloadBlockedReason::InvalidInsertedView,
            );
            return (
                OverloadResultStatus::Blocked {
                    reason: OverloadBlockedReason::InvalidInsertedView,
                },
                vec![diagnostic],
            );
        };

        (
            OverloadResultStatus::Resolved {
                root,
                refinements: input.refinements,
                exposed_result: input.refinement_join.exposed_result,
                inserted_views,
            },
            Vec::new(),
        )
    }

    fn insert_views(
        &mut self,
        site: OverloadSiteId,
        root: OverloadCandidateId,
        input: &OverloadSiteResolutionInput,
    ) -> Option<Vec<InsertedViewId>> {
        let allowed_candidates = std::iter::once(root)
            .chain(input.refinements.iter().copied())
            .collect::<BTreeSet<_>>();
        if input.inserted_views.iter().any(|view| {
            !allowed_candidates.contains(&view.selected_candidate)
                || view.kind == InsertedViewKind::Narrowing
                || view.status != InsertedViewStatus::Accepted
        }) {
            return None;
        }

        let mut inserted = Vec::new();
        for view in &input.inserted_views {
            let id = self.inserted_views.insert(InsertedView {
                id: InsertedViewId::new(0),
                site,
                argument: view.argument.clone(),
                target: view.target,
                selected_candidate: view.selected_candidate,
                kind: view.kind,
                reason: view.reason.clone(),
                evidence_facts: view.evidence_facts.clone(),
                path: view.path.clone(),
            });
            inserted.push(id);
        }
        Some(inserted)
    }

    fn is_active_refinement_candidate(
        &self,
        graph: &SpecificityGraph,
        candidate: OverloadCandidateId,
        selected_root: &SymbolId,
        root: OverloadCandidateId,
    ) -> bool {
        graph.nodes.iter().any(|node| {
            node.candidate == candidate
                && node.candidate != root
                && &node.ordinary_root == selected_root
                && matches!(node.origin, CandidateOrigin::Redefinition { .. })
        }) && self
            .graphs
            .candidates()
            .get(candidate)
            .is_some_and(|candidate| candidate.coherence == Some(CoherenceStatus::Accepted))
    }

    fn insert_selection_diagnostic(
        &mut self,
        site: Option<OverloadSiteId>,
        reason: OverloadBlockedReason,
    ) -> OverloadDiagnosticId {
        self.diagnostics.insert(OverloadDiagnosticDraft {
            site,
            site_key: None,
            candidate: None,
            provenance: None,
            class: OverloadDiagnosticClass::Selection,
            severity: OverloadDiagnosticSeverity::Error,
            message_key: OverloadDiagnosticMessageKey::new(format!(
                "overload.selection.{}",
                overload_blocked_reason_name(&reason)
            )),
            recovery: OverloadDiagnosticRecovery::Degraded,
        })
    }

    fn insert_refinement_diagnostic(
        &mut self,
        site: Option<OverloadSiteId>,
        reason: &RefinementJoinFailure,
    ) -> OverloadDiagnosticId {
        self.diagnostics.insert(OverloadDiagnosticDraft {
            site,
            site_key: None,
            candidate: None,
            provenance: None,
            class: OverloadDiagnosticClass::Selection,
            severity: OverloadDiagnosticSeverity::Error,
            message_key: OverloadDiagnosticMessageKey::new(format!(
                "overload.selection.refinement.{}",
                refinement_join_failure_name(reason)
            )),
            recovery: OverloadDiagnosticRecovery::Degraded,
        })
    }
}

fn maximal_candidates(graph: &SpecificityGraph) -> Vec<OverloadCandidateId> {
    let edges = graph
        .edges
        .iter()
        .map(|edge| (edge.from, edge.to))
        .collect::<BTreeSet<_>>();
    graph
        .nodes
        .iter()
        .filter(|node| {
            !graph.nodes.iter().any(|other| {
                other.candidate != node.candidate
                    && edges.contains(&(other.candidate, node.candidate))
                    && !edges.contains(&(node.candidate, other.candidate))
            })
        })
        .map(|node| node.candidate)
        .collect()
}

fn choose_selected_root(
    graph: &SpecificityGraph,
    maximal: &[OverloadCandidateId],
) -> Result<OverloadCandidateId, OverloadBlockedReason> {
    let roots = maximal
        .iter()
        .copied()
        .filter(|candidate| {
            graph.nodes.iter().any(|node| {
                node.candidate == *candidate
                    && !matches!(node.origin, CandidateOrigin::Redefinition { .. })
            })
        })
        .collect::<Vec<_>>();
    match roots.as_slice() {
        [root] => Ok(*root),
        [] => Err(OverloadBlockedReason::MissingOrdinaryRootCandidate),
        _ => Err(OverloadBlockedReason::AmbiguousOrdinaryRootCandidate),
    }
}

fn site_status(input: &OverloadSiteInput) -> OverloadSiteStatus {
    if !input.kind.is_supported() {
        OverloadSiteStatus::Deferred
    } else if matches!(input.recovery, OverloadSiteRecovery::Degraded { .. }) {
        OverloadSiteStatus::Degraded
    } else {
        OverloadSiteStatus::Collected
    }
}

fn unsupported_site_message_key(kind: &OverloadSiteKind) -> Option<OverloadDiagnosticMessageKey> {
    let OverloadSiteKind::Unsupported(role) = kind else {
        return None;
    };
    Some(OverloadDiagnosticMessageKey::new(format!(
        "overload.site.unsupported.{}",
        unsupported_role_name(role)
    )))
}

fn unsupported_candidate_message_key(
    kind: &CandidateDeclarationKind,
) -> Option<OverloadDiagnosticMessageKey> {
    let CandidateDeclarationKind::Unsupported(role) = kind else {
        return None;
    };
    Some(OverloadDiagnosticMessageKey::new(format!(
        "overload.candidate.unsupported.{}",
        unsupported_role_name(role)
    )))
}

fn site_diagnostic_provenance(input: &OverloadSiteInput) -> OverloadDiagnosticProvenance {
    OverloadDiagnosticProvenance::SiteInput {
        owner: input.owner.clone(),
        source_range: input.source_range,
        kind: input.kind.clone(),
        name: input.name.clone(),
        arguments: input.arguments.clone(),
        source_qua: input.source_qua.clone(),
        recovery: input.recovery.clone(),
    }
}

fn candidate_diagnostic_provenance(input: &OverloadCandidateInput) -> OverloadDiagnosticProvenance {
    OverloadDiagnosticProvenance::CandidateInput {
        symbol: input.symbol.clone(),
        ordinary_root: input.ordinary_root.clone(),
        declaration_kind: input.declaration_kind.clone(),
        template: input.template.clone().map(Box::new),
        coherence: input.coherence,
        provenance: input.provenance.clone(),
    }
}

fn candidate_diagnostic_provenance_from_candidate(
    candidate: &OverloadCandidate,
) -> OverloadDiagnosticProvenance {
    OverloadDiagnosticProvenance::CandidateInput {
        symbol: candidate.symbol.clone(),
        ordinary_root: candidate.ordinary_root.clone(),
        declaration_kind: candidate.declaration_kind.clone(),
        template: candidate.template.clone().map(Box::new),
        coherence: candidate.coherence,
        provenance: candidate.provenance.clone(),
    }
}

fn site_input_order_key(input: &OverloadSiteInput, ordinal: usize) -> SiteInputOrderKey {
    (
        range_order_key(input.source_range),
        site_ref_order_key(&input.owner),
        input.kind.clone(),
        input.name.as_str().to_owned(),
        input.key.as_str().to_owned(),
        ordinal,
    )
}

fn site_output_key(site: &OverloadSite) -> SiteOutputOrderKey {
    (
        range_order_key(site.source_range),
        site_ref_order_key(&site.owner),
        site.kind.clone(),
        site.name.as_str().to_owned(),
        site.key.as_str().to_owned(),
        site.id.index(),
    )
}

fn candidate_input_order_key(
    input: &OverloadCandidateInput,
    ordinal: usize,
    site_ids: &BTreeMap<OverloadSiteKey, OverloadSiteId>,
) -> CandidateInputOrderKey {
    let site_order = site_ids
        .get(&input.site)
        .map(|id| id.index())
        .unwrap_or(usize::MAX);
    (
        site_order,
        input.site.as_str().to_owned(),
        input.declaration_kind.clone(),
        input.ordinary_root.clone(),
        input.symbol.clone(),
        template_order_key(input.template.as_ref(), &input.origin),
        provenance_order_key(&input.provenance),
        input.provenance.declaration_order,
        ordinal,
    )
}

fn candidate_output_key(candidate: &OverloadCandidate) -> CandidateOutputOrderKey {
    (
        candidate.site.index(),
        candidate.declaration_kind.clone(),
        candidate.ordinary_root.clone(),
        candidate.symbol.clone(),
        template_order_key(candidate.template.as_ref(), &candidate.origin),
        provenance_order_key(&candidate.provenance),
        candidate.id.index(),
    )
}

fn template_expansion_order_key(expansion: &TemplateExpansion) -> TemplateExpansionOrderKey {
    (
        expansion.source_candidate.index(),
        expansion.instantiation_key.as_str().to_owned(),
        expansion.id.index(),
    )
}

fn candidate_viability_order_key(viability: &CandidateViability) -> CandidateViabilityOrderKey {
    (viability.source_candidate.index(), viability.id.index())
}

fn specificity_graph_order_key(graph: &SpecificityGraph) -> SpecificityGraphOrderKey {
    (graph.site.index(), graph.id.index())
}

fn specificity_edge_order_key(edge: &SpecificityEdge) -> SpecificityEdgeOrderKey {
    (edge.from.index(), edge.to.index(), edge.id.index())
}

fn overload_result_order_key(result: &OverloadResult) -> OverloadResultOrderKey {
    (result.site.index(), result.id.index())
}

fn inserted_view_order_key(view: &InsertedView) -> InsertedViewOrderKey {
    (
        view.site.index(),
        view.argument.node().index(),
        view.id.index(),
    )
}

fn inserted_view_input_order_key(view: &InsertedViewInput) -> InsertedViewInputOrderKey {
    let (path_missing, path) = view
        .path
        .as_ref()
        .map_or_else(|| (1, String::new()), |path| (0, path.as_str().to_owned()));
    (
        site_ref_order_key(&view.argument),
        view.selected_candidate.index(),
        view.target.index(),
        view.kind,
        view.status,
        view.reason.as_str().to_owned(),
        path_missing,
        path,
    )
}

fn diagnostic_output_key(diagnostic: &OverloadDiagnostic) -> DiagnosticOutputOrderKey {
    let (site_missing, site_index, site_key) = match (diagnostic.site, &diagnostic.site_key) {
        (Some(site), key) => (
            0,
            site.index(),
            key.as_ref()
                .map(|key| key.as_str().to_owned())
                .unwrap_or_default(),
        ),
        (None, Some(key)) => (1, usize::MAX, key.as_str().to_owned()),
        (None, None) => (2, usize::MAX, String::new()),
    };
    let (candidate_missing, candidate_index) = match diagnostic.candidate {
        Some(candidate) => (0, candidate.index()),
        None => (1, usize::MAX),
    };
    (
        site_missing,
        site_index,
        site_key,
        candidate_missing,
        candidate_index,
        diagnostic.class,
        diagnostic.message_key.as_str().to_owned(),
        diagnostic.id.index(),
    )
}

fn template_order_key(
    payload: Option<&TemplateCandidatePayload>,
    origin: &CandidateOrigin,
) -> String {
    payload
        .map(|payload| payload.instantiation_key.as_str().to_owned())
        .or_else(|| {
            if let CandidateOrigin::TemplateDerived { instantiation, .. } = origin {
                Some(instantiation.as_str().to_owned())
            } else {
                None
            }
        })
        .unwrap_or_default()
}

fn provenance_order_key(provenance: &CandidateProvenance) -> CandidateProvenanceOrderKey {
    match &provenance.scope {
        CandidateScope::Local => (
            0,
            String::new(),
            String::new(),
            optional_range_order_key(provenance.source_range),
            provenance.declaration_order,
            provenance.stable_key.as_str().to_owned(),
        ),
        CandidateScope::Imported { module, import_key } => (
            1,
            module_order_key(module),
            import_key.as_str().to_owned(),
            optional_range_order_key(provenance.source_range),
            provenance.declaration_order,
            provenance.stable_key.as_str().to_owned(),
        ),
        CandidateScope::DependencySummary {
            module,
            summary_key,
        } => (
            2,
            module_order_key(module),
            summary_key.as_str().to_owned(),
            optional_range_order_key(provenance.source_range),
            provenance.declaration_order,
            provenance.stable_key.as_str().to_owned(),
        ),
    }
}

fn optional_range_order_key(range: Option<SourceRange>) -> OptionalRangeOrderKey {
    range.map_or_else(
        || (1, String::new(), usize::MAX, usize::MAX),
        |range| {
            let (source, start, end) = range_order_key(range);
            (0, source, start, end)
        },
    )
}

fn range_order_key(range: SourceRange) -> RangeOrderKey {
    (source_order_key(range), range.start, range.end)
}

fn source_order_key(range: SourceRange) -> String {
    format!("{:?}", range.source_id)
}

fn site_ref_order_key(site: &TypedSiteRef) -> SiteRefOrderKey {
    match site {
        TypedSiteRef::Node(node) => (node.index(), 0, String::new()),
        TypedSiteRef::Role { node, role } => (node.index(), 1, role.as_str().to_owned()),
    }
}

fn site_ref_key(site: &TypedSiteRef) -> String {
    match site {
        TypedSiteRef::Node(node) => format!("node#{}", node.index()),
        TypedSiteRef::Role { node, role } => format!("node#{}:{}", node.index(), role.as_str()),
    }
}

fn module_order_key(module: &ModuleId) -> String {
    format!("{}::{}", module.package().as_str(), module.path().as_str())
}

fn write_sites(output: &mut String, sites: &OverloadSiteTable) {
    output.push_str("sites:\n");
    if sites.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for (id, site) in sites.canonical_iter() {
        let _ = write!(
            output,
            "  site#{} key=\"{}\" kind={} status={} owner={} range=",
            id.index(),
            escaped_display(site.key.as_str()),
            site_kind_name(&site.kind),
            site_status_name(site.status),
            site_ref_key(&site.owner)
        );
        write_range(output, site.source_range);
        let _ = write!(
            output,
            " name=\"{}\" args=",
            escaped_display(site.name.as_str())
        );
        write_site_refs(output, &site.arguments);
        output.push_str(" expected=");
        write_optional_type(output, site.expected);
        output.push_str(" source_qua=");
        write_qua_views(output, &site.source_qua);
        output.push_str(" recovery=");
        write_site_recovery(output, &site.recovery);
        output.push_str(" diagnostics=");
        write_diagnostic_ids(output, &site.diagnostics);
        output.push('\n');
    }
}

fn write_candidates(output: &mut String, candidates: &OverloadCandidateTable) {
    output.push_str("candidates:\n");
    if candidates.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for (id, candidate) in candidates.canonical_iter() {
        let _ = write!(
            output,
            "  candidate#{} site=site#{} site_key=\"{}\" status={} declaration={} symbol=",
            id.index(),
            candidate.site.index(),
            escaped_display(candidate.site_key.as_str()),
            candidate_status_name(candidate.status),
            candidate_declaration_kind_name(&candidate.declaration_kind)
        );
        write_symbol_id(output, &candidate.symbol);
        output.push_str(" root=");
        write_symbol_id(output, &candidate.ordinary_root);
        output.push_str(" parameters=");
        write_type_ids(output, &candidate.parameters);
        output.push_str(" result=");
        write_optional_type(output, candidate.result);
        output.push_str(" origin=");
        write_origin(output, &candidate.origin);
        output.push_str(" template=");
        write_template_payload(output, candidate.template.as_ref());
        output.push_str(" coherence=");
        write_optional_coherence(output, candidate.coherence);
        output.push_str(" provenance=");
        write_candidate_provenance(output, &candidate.provenance);
        output.push_str(" diagnostics=");
        write_diagnostic_ids(output, &candidate.diagnostics);
        output.push('\n');
    }
}

fn write_template_expansions(output: &mut String, expansions: &TemplateExpansionTable) {
    output.push_str("template_expansions:\n");
    if expansions.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for (id, expansion) in expansions.canonical_iter() {
        let _ = write!(
            output,
            "  expansion#{} source=candidate#{} site=site#{} template=",
            id.index(),
            expansion.source_candidate.index(),
            expansion.site.index()
        );
        write_symbol_id(output, &expansion.template);
        let _ = write!(
            output,
            " instantiation=\"{}\" status=",
            escaped_display(expansion.instantiation_key.as_str())
        );
        write_template_expansion_status(output, &expansion.status);
        output.push_str(" substitutions=");
        write_template_substitutions(output, &expansion.substitutions);
        output.push_str(" instantiated=");
        write_optional_candidate_id(output, expansion.instantiated_candidate);
        output.push_str(" diagnostics=");
        write_diagnostic_ids(output, &expansion.diagnostics);
        output.push('\n');
    }
}

fn write_candidate_viability(output: &mut String, decisions: &CandidateViabilityTable) {
    output.push_str("candidate_viability:\n");
    if decisions.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for (id, decision) in decisions.canonical_iter() {
        let _ = write!(
            output,
            "  viability#{} source=candidate#{} site=site#{} output=",
            id.index(),
            decision.source_candidate.index(),
            decision.site.index()
        );
        write_optional_candidate_id(output, decision.output_candidate);
        output.push_str(" status=");
        write_candidate_viability_status(output, &decision.status);
        output.push_str(" diagnostics=");
        write_diagnostic_ids(output, &decision.diagnostics);
        output.push('\n');
    }
}

fn write_candidate_viability_status(output: &mut String, status: &CandidateViabilityStatus) {
    match status {
        CandidateViabilityStatus::Viable { views } => {
            output.push_str("viable(views=");
            write_argument_view_plans(output, views);
            output.push(')');
        }
        CandidateViabilityStatus::Rejected { reasons } => {
            output.push_str("rejected(reasons=");
            write_candidate_rejections(output, reasons);
            output.push(')');
        }
        CandidateViabilityStatus::Blocked { reason } => {
            output.push_str("blocked(reason=");
            write_candidate_blocked_reason(output, reason);
            output.push(')');
        }
    }
}

fn write_argument_view_plans(output: &mut String, views: &[ArgumentViewPlan]) {
    output.push('[');
    for (index, view) in views.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "{{arg={} actual=", view.argument_index);
        write_type_id(output, view.actual);
        output.push_str(" target=");
        write_type_id(output, view.target);
        let _ = write!(output, " kind={}", argument_view_kind_name(view.kind));
        output.push_str(" facts=");
        write_fact_ids(output, &view.facts);
        output.push_str(" coercion=");
        write_optional_coercion_id(output, view.coercion);
        output.push_str(" path=");
        write_optional_qua_path(output, view.path.as_ref());
        output.push('}');
    }
    output.push(']');
}

fn write_candidate_rejections(output: &mut String, rejections: &[CandidateRejection]) {
    output.push('[');
    for (index, rejection) in rejections.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(
            output,
            "{{arg={} reason={} actual=",
            rejection.argument_index,
            viability_rejection_name(rejection.reason)
        );
        write_optional_type(output, rejection.actual);
        output.push_str(" target=");
        write_optional_type(output, rejection.target);
        output.push_str(" facts=");
        write_fact_ids(output, &rejection.facts);
        output.push_str(" coercion=");
        write_optional_coercion_id(output, rejection.coercion);
        output.push_str(" path=");
        write_optional_qua_path(output, rejection.path.as_ref());
        output.push('}');
    }
    output.push(']');
}

fn write_candidate_blocked_reason(output: &mut String, reason: &CandidateBlockedReason) {
    let _ = write!(
        output,
        "{{arg={} reason={} actual=",
        reason
            .argument_index
            .map(|index| index.to_string())
            .unwrap_or_else(|| "<none>".to_owned()),
        viability_blocked_reason_name(reason.reason)
    );
    write_optional_type(output, reason.actual);
    output.push_str(" target=");
    write_optional_type(output, reason.target);
    output.push_str(" paths=");
    write_qua_paths(output, &reason.paths);
    output.push_str(" detail=");
    match &reason.detail {
        Some(detail) => {
            let _ = write!(output, "\"{}\"", escaped_display(detail.as_str()));
        }
        None => output.push_str("<none>"),
    }
    output.push('}');
}

fn write_specificity_graphs(output: &mut String, graphs: &SpecificityGraphTable) {
    output.push_str("specificity_graphs:\n");
    if graphs.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for (id, graph) in graphs.canonical_iter() {
        let _ = write!(
            output,
            "  graph#{} site=site#{} nodes=",
            id.index(),
            graph.site.index()
        );
        write_specificity_nodes(output, &graph.nodes);
        output.push_str(" comparisons=");
        write_specificity_comparisons(output, &graph.comparisons);
        output.push_str(" edges=");
        write_specificity_edges(output, &graph.edges);
        output.push_str(" diagnostics=");
        write_diagnostic_ids(output, &graph.diagnostics);
        output.push('\n');
    }
}

fn write_specificity_nodes(output: &mut String, nodes: &[SpecificityNode]) {
    output.push('[');
    for (index, node) in nodes.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(
            output,
            "{{candidate=candidate#{} root=",
            node.candidate.index()
        );
        write_symbol_id(output, &node.ordinary_root);
        output.push_str(" parameters=");
        write_type_ids(output, &node.parameters);
        output.push_str(" origin=");
        write_origin(output, &node.origin);
        output.push('}');
    }
    output.push(']');
}

fn write_specificity_comparisons(output: &mut String, comparisons: &[SpecificityComparison]) {
    output.push('[');
    for (index, comparison) in comparisons.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(
            output,
            "{{comparison#{} left=candidate#{} right=candidate#{} status=",
            comparison.id.index(),
            comparison.left.index(),
            comparison.right.index()
        );
        write_specificity_outcome(output, &comparison.status);
        output.push_str(" reasons=");
        write_specificity_reasons(output, &comparison.reasons);
        output.push_str(" diagnostics=");
        write_diagnostic_ids(output, &comparison.diagnostics);
        output.push('}');
    }
    output.push(']');
}

fn write_specificity_edges(output: &mut String, edges: &[SpecificityEdge]) {
    let mut edges = edges.iter().collect::<Vec<_>>();
    edges.sort_by_key(|edge| specificity_edge_order_key(edge));
    output.push('[');
    for (index, edge) in edges.into_iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(
            output,
            "{{edge#{} from=candidate#{} to=candidate#{} reasons=",
            edge.id.index(),
            edge.from.index(),
            edge.to.index()
        );
        write_specificity_reasons(output, &edge.reasons);
        output.push('}');
    }
    output.push(']');
}

fn write_specificity_outcome(output: &mut String, outcome: &SpecificityComparisonOutcome) {
    match outcome {
        SpecificityComparisonOutcome::LeftAtLeastRight => output.push_str("left_at_least_right"),
        SpecificityComparisonOutcome::RightAtLeastLeft => output.push_str("right_at_least_left"),
        SpecificityComparisonOutcome::Equivalent => output.push_str("equivalent"),
        SpecificityComparisonOutcome::Incomparable => output.push_str("incomparable"),
        SpecificityComparisonOutcome::Blocked(reason) => {
            let _ = write!(output, "blocked({})", specificity_failure_name(reason));
        }
    }
}

fn write_overload_results(output: &mut String, results: &OverloadResultTable) {
    output.push_str("overload_results:\n");
    if results.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for (id, result) in results.canonical_iter() {
        let _ = write!(
            output,
            "  result#{} site=site#{} graph=graph#{} status=",
            id.index(),
            result.site.index(),
            result.graph.index()
        );
        write_overload_result_status(output, &result.status);
        output.push_str(" diagnostics=");
        write_diagnostic_ids(output, &result.diagnostics);
        output.push('\n');
    }
}

fn write_overload_result_status(output: &mut String, status: &OverloadResultStatus) {
    match status {
        OverloadResultStatus::Resolved {
            root,
            refinements,
            exposed_result,
            inserted_views,
        } => {
            let _ = write!(output, "resolved(root=candidate#{}", root.index());
            output.push_str(" refinements=");
            write_candidate_ids(output, refinements);
            output.push_str(" exposed_result=");
            write_exposed_result_payload(output, exposed_result.as_ref());
            output.push_str(" inserted_views=");
            write_inserted_view_ids(output, inserted_views);
            output.push(')');
        }
        OverloadResultStatus::NoMatch { rejected } => {
            output.push_str("no_match(rejected=");
            write_candidate_ids(output, rejected);
            output.push(')');
        }
        OverloadResultStatus::Ambiguous { candidates } => {
            output.push_str("ambiguous(candidates=");
            write_candidate_ids(output, candidates);
            output.push(')');
        }
        OverloadResultStatus::IncompatibleRefinementJoin {
            root,
            refinements,
            reason,
        } => {
            let _ = write!(
                output,
                "incompatible_refinement_join(root=candidate#{} refinements=",
                root.index()
            );
            write_candidate_ids(output, refinements);
            let _ = write!(output, " reason={})", refinement_join_failure_name(reason));
        }
        OverloadResultStatus::Blocked { reason } => {
            let _ = write!(
                output,
                "blocked(reason={})",
                overload_blocked_reason_name(reason)
            );
        }
    }
}

fn write_inserted_views(output: &mut String, views: &InsertedViewTable) {
    output.push_str("inserted_views:\n");
    if views.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for (id, view) in views.canonical_iter() {
        let _ = write!(
            output,
            "  view#{} site=site#{} argument={} target=",
            id.index(),
            view.site.index(),
            site_ref_key(&view.argument)
        );
        write_type_id(output, view.target);
        let _ = write!(
            output,
            " selected=candidate#{} kind={} reason=\"{}\" facts=",
            view.selected_candidate.index(),
            inserted_view_kind_name(view.kind),
            escaped_display(view.reason.as_str())
        );
        write_fact_ids(output, &view.evidence_facts);
        output.push_str(" path=");
        write_optional_qua_path(output, view.path.as_ref());
        output.push('\n');
    }
}

fn write_specificity_reasons(output: &mut String, reasons: &[SpecificityReasonKey]) {
    output.push('[');
    for (index, reason) in reasons.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "\"{}\"", escaped_display(reason.as_str()));
    }
    output.push(']');
}

fn write_diagnostics(output: &mut String, diagnostics: &OverloadDiagnosticTable) {
    output.push_str("diagnostics:\n");
    if diagnostics.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for (id, diagnostic) in diagnostics.canonical_iter() {
        let _ = write!(output, "  diagnostic#{} site=", id.index());
        write_optional_site_id(output, diagnostic.site);
        output.push_str(" site_key=");
        write_optional_site_key(output, diagnostic.site_key.as_ref());
        output.push_str(" candidate=");
        write_optional_candidate_id(output, diagnostic.candidate);
        output.push_str(" provenance=");
        write_diagnostic_provenance(output, diagnostic.provenance.as_ref());
        let _ = writeln!(
            output,
            " class={} severity={} message_key=\"{}\" recovery={}",
            diagnostic_class_name(diagnostic.class),
            diagnostic_severity_name(diagnostic.severity),
            escaped_display(diagnostic.message_key.as_str()),
            diagnostic_recovery_name(diagnostic.recovery)
        );
    }
}

fn write_diagnostic_provenance(
    output: &mut String,
    provenance: Option<&OverloadDiagnosticProvenance>,
) {
    let Some(provenance) = provenance else {
        output.push_str("<none>");
        return;
    };
    match provenance {
        OverloadDiagnosticProvenance::SiteInput {
            owner,
            source_range,
            kind,
            name,
            arguments,
            source_qua,
            recovery,
        } => {
            let _ = write!(output, "site_input(owner={}, range=", site_ref_key(owner));
            write_range(output, *source_range);
            let _ = write!(
                output,
                ", kind={}, name=\"{}\", args=",
                site_kind_name(kind),
                escaped_display(name.as_str())
            );
            write_site_refs(output, arguments);
            output.push_str(", source_qua=");
            write_qua_views(output, source_qua);
            output.push_str(", recovery=");
            write_site_recovery(output, recovery);
            output.push(')');
        }
        OverloadDiagnosticProvenance::CandidateInput {
            symbol,
            ordinary_root,
            declaration_kind,
            template,
            coherence,
            provenance,
        } => {
            output.push_str("candidate_input(symbol=");
            write_symbol_id(output, symbol);
            output.push_str(", root=");
            write_symbol_id(output, ordinary_root);
            let _ = write!(
                output,
                ", declaration={}, template=",
                candidate_declaration_kind_name(declaration_kind)
            );
            write_template_payload(output, template.as_deref());
            output.push_str(", coherence=");
            write_optional_coherence(output, *coherence);
            output.push_str(", provenance=");
            write_candidate_provenance(output, provenance);
            output.push(')');
        }
    }
}

fn write_qua_views(output: &mut String, views: &[SourceQuaView]) {
    output.push('[');
    for (index, view) in views.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        output.push('{');
        output.push_str("argument=");
        output.push_str(&site_ref_key(&view.argument));
        output.push_str(", target=");
        write_type_id(output, view.target);
        output.push_str(", range=");
        write_range(output, view.source_range);
        output.push_str(", path=");
        match &view.path {
            Some(path) => {
                let _ = write!(output, "\"{}\"", escaped_display(path.as_str()));
            }
            None => output.push_str("<none>"),
        }
        output.push_str(", facts=");
        write_fact_ids(output, &view.evidence_facts);
        output.push('}');
    }
    output.push(']');
}

fn write_template_expansion_status(output: &mut String, status: &TemplateExpansionStatus) {
    match status {
        TemplateExpansionStatus::Instantiated => output.push_str("instantiated"),
        TemplateExpansionStatus::Rejected(failure) => {
            let _ = write!(output, "rejected({})", template_failure_name(failure));
        }
        TemplateExpansionStatus::Deferred(failure) => {
            let _ = write!(output, "deferred({})", template_failure_name(failure));
        }
    }
}

fn write_template_substitutions(output: &mut String, substitutions: &[TemplateSubstitution]) {
    output.push('[');
    for (index, substitution) in substitutions.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(
            output,
            "{{parameter=\"{}\", value=",
            escaped_display(substitution.parameter.as_str())
        );
        write_type_id(output, substitution.value);
        output.push_str(", source=");
        write_template_substitution_source(output, &substitution.source);
        output.push('}');
    }
    output.push(']');
}

fn write_template_substitution_source(output: &mut String, source: &TemplateSubstitutionSource) {
    match source {
        TemplateSubstitutionSource::Explicit => output.push_str("explicit"),
        TemplateSubstitutionSource::OmittedInference { evidence_key } => {
            let _ = write!(
                output,
                "omitted_inference(\"{}\")",
                escaped_display(evidence_key.as_str())
            );
        }
        TemplateSubstitutionSource::SourceQua { source, path } => {
            output.push_str("source_qua(source=");
            write_type_id(output, *source);
            let _ = write!(output, ", path=\"{}\")", escaped_display(path.as_str()));
        }
    }
}

fn write_template_payload(output: &mut String, payload: Option<&TemplateCandidatePayload>) {
    let Some(payload) = payload else {
        output.push_str("<none>");
        return;
    };
    output.push('{');
    output.push_str("template=");
    write_symbol_id(output, &payload.template);
    let _ = write!(
        output,
        ", instantiation=\"{}\", parameters=",
        escaped_display(payload.instantiation_key.as_str())
    );
    write_template_parameters(output, &payload.parameters);
    output.push_str(", arguments=");
    write_template_arguments(output, &payload.arguments);
    output.push_str(", inferred=");
    write_template_inferences(output, &payload.inferred_arguments);
    output.push_str(", constraints=");
    write_template_constraints(output, &payload.constraints);
    output.push('}');
}

fn write_template_parameters(output: &mut String, parameters: &[TemplateParameterKey]) {
    output.push('[');
    for (index, parameter) in parameters.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "\"{}\"", escaped_display(parameter.as_str()));
    }
    output.push(']');
}

fn write_template_arguments(output: &mut String, arguments: &[TemplateArgument]) {
    output.push('[');
    for (index, argument) in arguments.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        match argument {
            TemplateArgument::Explicit(id) => {
                output.push_str("explicit=");
                write_type_id(output, *id);
            }
            TemplateArgument::Omitted(parameter) => {
                let _ = write!(
                    output,
                    "omitted=\"{}\"",
                    escaped_display(parameter.as_str())
                );
            }
            TemplateArgument::SourceQua {
                source,
                target,
                path,
                status,
            } => {
                output.push_str("source_qua(source=");
                write_type_id(output, *source);
                output.push_str(", target=");
                write_type_id(output, *target);
                let _ = write!(
                    output,
                    ", path=\"{}\", status={})",
                    escaped_display(path.as_str()),
                    template_qua_status_name(*status)
                );
            }
        }
    }
    output.push(']');
}

fn write_template_inferences(output: &mut String, inferences: &[TemplateArgumentInference]) {
    output.push('[');
    for (index, inference) in inferences.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(
            output,
            "{{parameter=\"{}\", inferred=",
            escaped_display(inference.parameter.as_str())
        );
        write_type_id(output, inference.inferred);
        let _ = write!(
            output,
            ", evidence=\"{}\"}}",
            escaped_display(inference.evidence_key.as_str())
        );
    }
    output.push(']');
}

fn write_template_constraints(output: &mut String, constraints: &[TemplateConstraintEvidence]) {
    output.push('[');
    for (index, constraint) in constraints.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(
            output,
            "{{parameter=\"{}\", evidence=\"{}\", status={}, facts=",
            escaped_display(constraint.parameter.as_str()),
            escaped_display(constraint.evidence_key.as_str()),
            template_constraint_status_name(constraint.status)
        );
        write_fact_ids(output, &constraint.facts);
        output.push('}');
    }
    output.push(']');
}

fn write_candidate_provenance(output: &mut String, provenance: &CandidateProvenance) {
    output.push('{');
    let _ = write!(
        output,
        "key=\"{}\", scope=",
        escaped_display(provenance.stable_key.as_str())
    );
    write_candidate_scope(output, &provenance.scope);
    output.push_str(", range=");
    match provenance.source_range {
        Some(range) => write_range(output, range),
        None => output.push_str("<none>"),
    }
    let _ = write!(
        output,
        ", declaration_order={}",
        provenance.declaration_order
    );
    output.push('}');
}

fn write_candidate_scope(output: &mut String, scope: &CandidateScope) {
    match scope {
        CandidateScope::Local => output.push_str("local"),
        CandidateScope::Imported { module, import_key } => {
            output.push_str("imported(module=");
            write_module_id(output, module);
            let _ = write!(
                output,
                ", import=\"{}\")",
                escaped_display(import_key.as_str())
            );
        }
        CandidateScope::DependencySummary {
            module,
            summary_key,
        } => {
            output.push_str("dependency_summary(module=");
            write_module_id(output, module);
            let _ = write!(
                output,
                ", summary=\"{}\")",
                escaped_display(summary_key.as_str())
            );
        }
    }
}

fn write_origin(output: &mut String, origin: &CandidateOrigin) {
    match origin {
        CandidateOrigin::Ordinary => output.push_str("ordinary"),
        CandidateOrigin::Redefinition { refined } => {
            output.push_str("redefinition(refined=");
            write_symbol_id(output, refined);
            output.push(')');
        }
        CandidateOrigin::TemplateDerived {
            template,
            instantiation,
        } => {
            output.push_str("template_derived(template=");
            write_symbol_id(output, template);
            let _ = write!(
                output,
                ", instantiation=\"{}\")",
                escaped_display(instantiation.as_str())
            );
        }
        CandidateOrigin::Recovery(key) => {
            let _ = write!(output, "recovery(\"{}\")", escaped_display(key.as_str()));
        }
    }
}

fn write_site_recovery(output: &mut String, recovery: &OverloadSiteRecovery) {
    match recovery {
        OverloadSiteRecovery::Normal => output.push_str("normal"),
        OverloadSiteRecovery::Degraded { message_key } => {
            let _ = write!(
                output,
                "degraded(\"{}\")",
                escaped_display(message_key.as_str())
            );
        }
    }
}

fn write_optional_coherence(output: &mut String, coherence: Option<CoherenceStatus>) {
    match coherence {
        Some(status) => output.push_str(coherence_status_name(status)),
        None => output.push_str("<none>"),
    }
}

fn write_site_refs(output: &mut String, sites: &[TypedSiteRef]) {
    output.push('[');
    for (index, site) in sites.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        output.push_str(&site_ref_key(site));
    }
    output.push(']');
}

fn write_type_ids(output: &mut String, ids: &[NormalizedTypeId]) {
    output.push('[');
    for (index, id) in ids.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        write_type_id(output, *id);
    }
    output.push(']');
}

fn write_fact_ids(output: &mut String, ids: &[TypeFactId]) {
    output.push('[');
    for (index, id) in ids.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "fact#{}", id.index());
    }
    output.push(']');
}

fn write_candidate_ids(output: &mut String, ids: &[OverloadCandidateId]) {
    output.push('[');
    for (index, id) in ids.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "candidate#{}", id.index());
    }
    output.push(']');
}

fn write_inserted_view_ids(output: &mut String, ids: &[InsertedViewId]) {
    output.push('[');
    for (index, id) in ids.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "view#{}", id.index());
    }
    output.push(']');
}

fn write_exposed_result_payload(output: &mut String, payload: Option<&ExposedResultPayload>) {
    let Some(payload) = payload else {
        output.push_str("<none>");
        return;
    };
    output.push_str("{result=");
    write_optional_type(output, payload.result);
    let _ = write!(
        output,
        " source={} evidence=",
        exposed_result_source_name(&payload.source)
    );
    write_fact_ids(output, &payload.evidence);
    output.push('}');
}

fn write_qua_paths(output: &mut String, paths: &[QuaPathKey]) {
    output.push('[');
    for (index, path) in paths.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "\"{}\"", escaped_display(path.as_str()));
    }
    output.push(']');
}

fn write_diagnostic_ids(output: &mut String, ids: &[OverloadDiagnosticId]) {
    output.push('[');
    for (index, id) in ids.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "diagnostic#{}", id.index());
    }
    output.push(']');
}

fn write_optional_type(output: &mut String, id: Option<NormalizedTypeId>) {
    match id {
        Some(id) => write_type_id(output, id),
        None => output.push_str("<none>"),
    }
}

fn write_type_id(output: &mut String, id: NormalizedTypeId) {
    let _ = write!(output, "normalized_type#{}", id.index());
}

fn write_optional_site_id(output: &mut String, id: Option<OverloadSiteId>) {
    match id {
        Some(id) => {
            let _ = write!(output, "site#{}", id.index());
        }
        None => output.push_str("<none>"),
    }
}

fn write_optional_site_key(output: &mut String, key: Option<&OverloadSiteKey>) {
    match key {
        Some(key) => {
            let _ = write!(output, "\"{}\"", escaped_display(key.as_str()));
        }
        None => output.push_str("<none>"),
    }
}

fn write_optional_candidate_id(output: &mut String, id: Option<OverloadCandidateId>) {
    match id {
        Some(id) => {
            let _ = write!(output, "candidate#{}", id.index());
        }
        None => output.push_str("<none>"),
    }
}

fn write_optional_coercion_id(output: &mut String, id: Option<CoercionId>) {
    match id {
        Some(id) => {
            let _ = write!(output, "coercion#{}", id.index());
        }
        None => output.push_str("<none>"),
    }
}

fn write_optional_qua_path(output: &mut String, path: Option<&QuaPathKey>) {
    match path {
        Some(path) => {
            let _ = write!(output, "\"{}\"", escaped_display(path.as_str()));
        }
        None => output.push_str("<none>"),
    }
}

fn write_range(output: &mut String, range: SourceRange) {
    let _ = write!(
        output,
        "source=\"{}\":{}..{}",
        escaped_display(&source_order_key(range)),
        range.start,
        range.end
    );
}

fn write_module_id(output: &mut String, module: &ModuleId) {
    write_escaped(output, module.package().as_str());
    output.push_str("::");
    write_escaped(output, module.path().as_str());
}

fn write_symbol_id(output: &mut String, symbol: &SymbolId) {
    output.push_str("{fqn=\"");
    write_escaped(output, symbol.fqn().as_str());
    output.push_str("\" module=");
    write_module_id(output, symbol.module());
    output.push_str(" local=\"");
    write_escaped(output, symbol.local().as_str());
    output.push_str("\"}");
}

fn site_kind_name(kind: &OverloadSiteKind) -> &'static str {
    match kind {
        OverloadSiteKind::FunctorApplication => "functor_application",
        OverloadSiteKind::PredicateApplication => "predicate_application",
        OverloadSiteKind::AttributeApplication => "attribute_application",
        OverloadSiteKind::ModeApplication => "mode_application",
        OverloadSiteKind::SelectorApplication => "selector_application",
        OverloadSiteKind::StructureFieldApplication => "structure_field_application",
        OverloadSiteKind::TemplateName => "template_name",
        OverloadSiteKind::Unsupported(_) => "unsupported",
    }
}

fn unsupported_role_name(role: &UnsupportedOverloadRole) -> &'static str {
    match role {
        UnsupportedOverloadRole::SchemeApplication => "scheme_application",
        UnsupportedOverloadRole::TheoremApplication => "theorem_application",
        UnsupportedOverloadRole::AlgorithmTemplate => "algorithm_template",
        UnsupportedOverloadRole::Unknown => "unknown",
    }
}

fn candidate_declaration_kind_name(kind: &CandidateDeclarationKind) -> &'static str {
    match kind {
        CandidateDeclarationKind::Functor => "functor",
        CandidateDeclarationKind::Predicate => "predicate",
        CandidateDeclarationKind::Attribute => "attribute",
        CandidateDeclarationKind::Mode => "mode",
        CandidateDeclarationKind::Selector => "selector",
        CandidateDeclarationKind::StructureField => "structure_field",
        CandidateDeclarationKind::Template => "template",
        CandidateDeclarationKind::Redefinition => "redefinition",
        CandidateDeclarationKind::Unsupported(_) => "unsupported",
    }
}

fn site_status_name(status: OverloadSiteStatus) -> &'static str {
    match status {
        OverloadSiteStatus::Collected => "collected",
        OverloadSiteStatus::Degraded => "degraded",
        OverloadSiteStatus::Deferred => "deferred",
    }
}

fn candidate_status_name(status: OverloadCandidateStatus) -> &'static str {
    match status {
        OverloadCandidateStatus::Collected => "collected",
        OverloadCandidateStatus::Deferred => "deferred",
    }
}

fn coherence_status_name(status: CoherenceStatus) -> &'static str {
    match status {
        CoherenceStatus::Accepted => "accepted",
        CoherenceStatus::Pending => "pending",
        CoherenceStatus::Rejected => "rejected",
    }
}

fn diagnostic_class_name(class: OverloadDiagnosticClass) -> &'static str {
    match class {
        OverloadDiagnosticClass::DuplicateSiteKey => "duplicate_site_key",
        OverloadDiagnosticClass::MissingSite => "missing_site",
        OverloadDiagnosticClass::UnsupportedSiteRole => "unsupported_site_role",
        OverloadDiagnosticClass::UnsupportedCandidateRole => "unsupported_candidate_role",
        OverloadDiagnosticClass::TemplateExpansion => "template_expansion",
        OverloadDiagnosticClass::Viability => "viability",
        OverloadDiagnosticClass::Specificity => "specificity",
        OverloadDiagnosticClass::Selection => "selection",
        OverloadDiagnosticClass::Recovery => "recovery",
        OverloadDiagnosticClass::ExternalDependencyGap => "external_dependency_gap",
    }
}

fn template_failure_name(failure: &TemplateExpansionFailure) -> &'static str {
    match failure {
        TemplateExpansionFailure::ArityMismatch => "arity_mismatch",
        TemplateExpansionFailure::DuplicateParameter => "duplicate_parameter",
        TemplateExpansionFailure::ParameterMismatch => "parameter_mismatch",
        TemplateExpansionFailure::MissingInference => "missing_inference",
        TemplateExpansionFailure::AmbiguousInference => "ambiguous_inference",
        TemplateExpansionFailure::UnknownConstraintParameter => "unknown_constraint_parameter",
        TemplateExpansionFailure::MissingConstraintEvidence => "missing_constraint_evidence",
        TemplateExpansionFailure::DeferredConstraintEvidence => "deferred_constraint_evidence",
        TemplateExpansionFailure::RejectedSourceQua => "rejected_source_qua",
        TemplateExpansionFailure::DeferredSourceQua => "deferred_source_qua",
        TemplateExpansionFailure::DeferredCandidate => "deferred_candidate",
    }
}

fn template_qua_status_name(status: TemplateQuaStatus) -> &'static str {
    match status {
        TemplateQuaStatus::AcceptedWidening => "accepted_widening",
        TemplateQuaStatus::RejectedNarrowing => "rejected_narrowing",
        TemplateQuaStatus::DeferredExternalDependency => "deferred_external_dependency",
    }
}

fn template_constraint_status_name(status: TemplateConstraintEvidenceStatus) -> &'static str {
    match status {
        TemplateConstraintEvidenceStatus::Accepted => "accepted",
        TemplateConstraintEvidenceStatus::Missing => "missing",
        TemplateConstraintEvidenceStatus::DeferredExternalDependency => {
            "deferred_external_dependency"
        }
    }
}

fn argument_view_kind_name(kind: ArgumentViewKind) -> &'static str {
    match kind {
        ArgumentViewKind::Exact => "exact",
        ArgumentViewKind::FactWidening => "fact_widening",
        ArgumentViewKind::CoercionWidening => "coercion_widening",
        ArgumentViewKind::SourceQua => "source_qua",
    }
}

fn viability_rejection_name(reason: CandidateRejectionReason) -> &'static str {
    match reason {
        CandidateRejectionReason::ArityMismatch => "arity_mismatch",
        CandidateRejectionReason::ParameterMismatch => "parameter_mismatch",
        CandidateRejectionReason::MissingEvidence => "missing_evidence",
        CandidateRejectionReason::PendingEvidence => "pending_evidence",
        CandidateRejectionReason::DegradedEvidence => "degraded_evidence",
        CandidateRejectionReason::RejectedEvidence => "rejected_evidence",
        CandidateRejectionReason::OutOfScopeAssumption => "out_of_scope_assumption",
        CandidateRejectionReason::InvalidNarrowing => "invalid_narrowing",
    }
}

fn viability_blocked_reason_name(reason: CandidateBlockedReasonKind) -> &'static str {
    match reason {
        CandidateBlockedReasonKind::CandidateDeferred => "candidate_deferred",
        CandidateBlockedReasonKind::DuplicateViabilityPayload => "duplicate_viability_payload",
        CandidateBlockedReasonKind::MissingViabilityPayload => "missing_viability_payload",
        CandidateBlockedReasonKind::AmbiguousInheritance => "ambiguous_inheritance",
        CandidateBlockedReasonKind::BlockedCoercion => "blocked_coercion",
        CandidateBlockedReasonKind::DeferredExternalDependency => "deferred_external_dependency",
    }
}

fn specificity_failure_name(reason: &SpecificityFailureReason) -> &'static str {
    match reason {
        SpecificityFailureReason::MissingComparisonPayload => "missing_comparison_payload",
        SpecificityFailureReason::DuplicateComparisonPayload => "duplicate_comparison_payload",
        SpecificityFailureReason::CrossSiteComparison => "cross_site_comparison",
        SpecificityFailureReason::UnknownCandidate => "unknown_candidate",
        SpecificityFailureReason::DeferredExternalDependency => "deferred_external_dependency",
        SpecificityFailureReason::MissingRecordedFacts => "missing_recorded_facts",
    }
}

fn overload_blocked_reason_name(reason: &OverloadBlockedReason) -> &'static str {
    match reason {
        OverloadBlockedReason::BlockedSpecificityComparison => "blocked_specificity_comparison",
        OverloadBlockedReason::AmbiguousSelection => "ambiguous_selection",
        OverloadBlockedReason::MissingSelectionPayload => "missing_selection_payload",
        OverloadBlockedReason::DuplicateSelectionPayload => "duplicate_selection_payload",
        OverloadBlockedReason::UnknownSelectionSite => "unknown_selection_site",
        OverloadBlockedReason::MissingOrdinaryRootCandidate => "missing_ordinary_root_candidate",
        OverloadBlockedReason::AmbiguousOrdinaryRootCandidate => {
            "ambiguous_ordinary_root_candidate"
        }
        OverloadBlockedReason::NonSelectedRootPayload => "non_selected_root_payload",
        OverloadBlockedReason::InvalidInsertedView => "invalid_inserted_view",
    }
}

fn refinement_join_failure_name(reason: &RefinementJoinFailure) -> &'static str {
    match reason {
        RefinementJoinFailure::IncompatibleResultRadix => "incompatible_result_radix",
        RefinementJoinFailure::ContradictoryAttributes => "contradictory_attributes",
        RefinementJoinFailure::NoUniqueJoinedType => "no_unique_joined_type",
        RefinementJoinFailure::MissingJoinPayload => "missing_join_payload",
    }
}

fn inserted_view_kind_name(kind: InsertedViewKind) -> &'static str {
    match kind {
        InsertedViewKind::Widening => "widening",
        InsertedViewKind::SourceQua => "source_qua",
        InsertedViewKind::Narrowing => "narrowing",
    }
}

fn exposed_result_source_name(source: &ExposedResultSource) -> &'static str {
    match source {
        ExposedResultSource::SelectedRoot => "selected_root",
        ExposedResultSource::StrongestRefinement => "strongest_refinement",
        ExposedResultSource::AttributeUnion => "attribute_union",
    }
}

fn diagnostic_severity_name(severity: OverloadDiagnosticSeverity) -> &'static str {
    match severity {
        OverloadDiagnosticSeverity::Error => "error",
        OverloadDiagnosticSeverity::Warning => "warning",
        OverloadDiagnosticSeverity::Note => "note",
    }
}

fn diagnostic_recovery_name(recovery: OverloadDiagnosticRecovery) -> &'static str {
    match recovery {
        OverloadDiagnosticRecovery::Normal => "normal",
        OverloadDiagnosticRecovery::Degraded => "degraded",
    }
}

fn escaped_display(value: &str) -> String {
    let mut escaped = String::new();
    write_escaped(&mut escaped, value);
    escaped
}

fn write_escaped(output: &mut String, value: &str) {
    for character in value.chars() {
        match character {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character if character.is_control() => {
                let _ = write!(output, "\\u{{{:x}}}", character as u32);
            }
            character => output.push(character),
        }
    }
}

impl fmt::Display for OverloadSiteKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typed_ast::{NormalizedTypeId, TypeRole, TypedNodeId, TypedSiteRef};
    use mizar_resolve::resolved_ast::{FullyQualifiedName, LocalSymbolId, ModuleId};
    use mizar_session::{
        BuildSnapshotId, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator,
        SourceId,
    };

    #[test]
    fn collects_supported_site_kinds_with_source_provenance() {
        let source_id = source_id(1);
        let sites = vec![
            site(
                "functor",
                OverloadSiteKind::FunctorApplication,
                source_id,
                10,
            ),
            site(
                "predicate",
                OverloadSiteKind::PredicateApplication,
                source_id,
                20,
            ),
            site(
                "attribute",
                OverloadSiteKind::AttributeApplication,
                source_id,
                30,
            ),
            site("mode", OverloadSiteKind::ModeApplication, source_id, 40),
            site(
                "selector",
                OverloadSiteKind::SelectorApplication,
                source_id,
                50,
            ),
            site(
                "structure",
                OverloadSiteKind::StructureFieldApplication,
                source_id,
                60,
            ),
            site("template", OverloadSiteKind::TemplateName, source_id, 70),
        ];
        let candidates = vec![
            candidate(
                "template",
                "template-root",
                "template-ref",
                CandidateScope::Local,
                0,
            ),
            candidate(
                "functor",
                "functor-root",
                "functor-ref",
                CandidateScope::Local,
                1,
            ),
        ];

        let output = OverloadCollectionOutput::collect(sites, candidates);

        assert_eq!(output.sites().len(), 7);
        assert_eq!(output.candidates().len(), 2);
        assert!(output.diagnostics().is_empty());
        assert_eq!(
            output
                .sites()
                .iter()
                .map(|(_, site)| site.key.as_str())
                .collect::<Vec<_>>(),
            [
                "functor",
                "predicate",
                "attribute",
                "mode",
                "selector",
                "structure",
                "template"
            ]
        );
        let debug = output.debug_text();
        assert!(debug.contains("key=\"functor\" kind=functor_application"));
        assert!(debug.contains("key=\"structure\" kind=structure_field_application"));
        assert!(debug.contains("range=source=\"SourceId"));
        assert!(debug.contains("provenance={key=\"template-ref-provenance\""));
    }

    #[test]
    fn preserves_already_filtered_candidate_sets_and_provenance() {
        let source_id = source_id(2);
        let imported_module = module("mathlib", "hidden/import");
        let output = OverloadCollectionOutput::collect(
            vec![site(
                "call",
                OverloadSiteKind::FunctorApplication,
                source_id,
                10,
            )],
            vec![
                candidate("call", "root-a", "local-a", CandidateScope::Local, 0),
                candidate(
                    "call",
                    "root-b",
                    "imported-b",
                    CandidateScope::Imported {
                        module: imported_module,
                        import_key: CandidateProvenanceKey::new("visible-import"),
                    },
                    1,
                ),
            ],
        );

        assert_eq!(output.candidates().len(), 2);
        let scopes = output
            .candidates()
            .iter()
            .map(|(_, candidate)| &candidate.provenance.scope)
            .collect::<Vec<_>>();
        assert!(matches!(scopes[0], CandidateScope::Local));
        assert!(matches!(scopes[1], CandidateScope::Imported { .. }));
        let (_, local) = output.candidates().iter().next().expect("local candidate");
        assert_eq!(local.provenance.stable_key.as_str(), "local-a-provenance");
        assert_range(local.provenance.source_range.expect("local range"), 0, 1);
        assert_eq!(local.provenance.declaration_order, 0);
        let (_, imported) = output
            .candidates()
            .iter()
            .nth(1)
            .expect("imported candidate");
        assert_eq!(
            imported.provenance.stable_key.as_str(),
            "imported-b-provenance"
        );
        assert_range(
            imported.provenance.source_range.expect("import range"),
            1,
            2,
        );
        assert_eq!(imported.provenance.declaration_order, 1);
        let debug = output.debug_text();
        assert!(debug.contains("scope=local"));
        assert!(debug.contains("scope=imported(module=mathlib::hidden/import"));
    }

    #[test]
    fn candidate_order_is_deterministic_by_site_and_declaration_keys() {
        let source_id = source_id(3);
        let sites = vec![
            site("later", OverloadSiteKind::FunctorApplication, source_id, 50),
            site(
                "earlier",
                OverloadSiteKind::FunctorApplication,
                source_id,
                10,
            ),
        ];
        let candidates = vec![
            candidate("later", "root-z", "symbol-z", CandidateScope::Local, 9),
            candidate("earlier", "root-b", "symbol-b", CandidateScope::Local, 2),
            candidate("earlier", "root-a", "symbol-c", CandidateScope::Local, 3),
            candidate("earlier", "root-a", "symbol-a", CandidateScope::Local, 1),
        ];

        let output = OverloadCollectionOutput::collect(sites.clone(), candidates.clone());
        let permuted_output = OverloadCollectionOutput::collect(
            sites.into_iter().rev().collect::<Vec<_>>(),
            candidates.into_iter().rev().collect::<Vec<_>>(),
        );

        assert_eq!(
            output
                .candidates()
                .iter()
                .map(|(_, candidate)| candidate.symbol.local().as_str())
                .collect::<Vec<_>>(),
            ["symbol-a", "symbol-c", "symbol-b", "symbol-z"]
        );
        assert_eq!(output.debug_text(), permuted_output.debug_text());
    }

    #[test]
    fn candidate_order_uses_provenance_and_declaration_order_tie_breakers() {
        let source_id = source_id(31);
        let mut later = candidate("call", "root-a", "symbol-a", CandidateScope::Local, 30);
        later.provenance.stable_key = CandidateProvenanceKey::new("later");
        later.provenance.source_range = Some(range(source_id, 80, 81));
        let mut earlier = candidate("call", "root-a", "symbol-a", CandidateScope::Local, 10);
        earlier.provenance.stable_key = CandidateProvenanceKey::new("earlier");
        earlier.provenance.source_range = Some(range(source_id, 40, 41));

        let output = OverloadCollectionOutput::collect(
            vec![site(
                "call",
                OverloadSiteKind::FunctorApplication,
                source_id,
                10,
            )],
            vec![later, earlier],
        );

        assert_eq!(
            output
                .candidates()
                .iter()
                .map(|(_, candidate)| candidate.provenance.stable_key.as_str())
                .collect::<Vec<_>>(),
            ["earlier", "later"]
        );
    }

    #[test]
    fn candidate_template_payload_and_coherence_are_retained() {
        let source_id = source_id(32);
        let template = symbol_id("template-source");
        let mut input = candidate(
            "call",
            "root",
            "template-candidate",
            CandidateScope::Local,
            0,
        );
        input.origin = CandidateOrigin::TemplateDerived {
            template: template.clone(),
            instantiation: TemplateInstantiationKey::new("T=mode"),
        };
        input.template = Some(TemplateCandidatePayload {
            template,
            instantiation_key: TemplateInstantiationKey::new("T=mode"),
            parameters: vec![
                TemplateParameterKey::new("T"),
                TemplateParameterKey::new("U"),
            ],
            arguments: vec![
                TemplateArgument::Explicit(NormalizedTypeId::new(7)),
                TemplateArgument::Omitted(TemplateParameterKey::new("U")),
            ],
            inferred_arguments: vec![TemplateArgumentInference {
                parameter: TemplateParameterKey::new("U"),
                inferred: NormalizedTypeId::new(10),
                evidence_key: CandidateProvenanceKey::new("inferred-U"),
            }],
            constraints: vec![TemplateConstraintEvidence {
                parameter: TemplateParameterKey::new("U"),
                evidence_key: CandidateProvenanceKey::new("constraint-visible"),
                facts: vec![TypeFactId::new(8)],
                status: TemplateConstraintEvidenceStatus::Accepted,
            }],
        });
        input.coherence = Some(CoherenceStatus::Accepted);

        let output = OverloadCollectionOutput::collect(
            vec![site("call", OverloadSiteKind::TemplateName, source_id, 10)],
            vec![input],
        );

        let (_, candidate) = output.candidates().iter().next().expect("candidate");
        assert_eq!(candidate.coherence, Some(CoherenceStatus::Accepted));
        let payload = candidate.template.as_ref().expect("template payload");
        assert_eq!(payload.instantiation_key.as_str(), "T=mode");
        assert_eq!(
            payload.arguments,
            [
                TemplateArgument::Explicit(NormalizedTypeId::new(7)),
                TemplateArgument::Omitted(TemplateParameterKey::new("U"))
            ]
        );
        assert_eq!(payload.constraints[0].parameter.as_str(), "U");
        assert_eq!(
            payload.constraints[0].evidence_key.as_str(),
            "constraint-visible"
        );
        assert_eq!(payload.constraints[0].facts, [TypeFactId::new(8)]);
        let debug = output.debug_text();
        assert!(debug.contains("coherence=accepted"));
        assert!(debug.contains("instantiation=\"T=mode\""));
    }

    #[test]
    fn explicit_template_candidates_expand_into_concrete_candidates_deterministically() {
        let source_id = source_id(33);
        let sites = vec![site(
            "call",
            OverloadSiteKind::FunctorApplication,
            source_id,
            10,
        )];
        let plain = candidate("call", "root", "same", CandidateScope::Local, 0);
        let mut templated = candidate("call", "root", "same", CandidateScope::Local, 1);
        templated.parameters = vec![NormalizedTypeId::new(7), NormalizedTypeId::new(8)];
        templated.result = Some(NormalizedTypeId::new(9));
        templated.template = Some(template_payload(
            "T=explicit",
            vec![TemplateParameterKey::new("T")],
            vec![TemplateArgument::Explicit(NormalizedTypeId::new(7))],
        ));
        let collection = OverloadCollectionOutput::collect(sites.clone(), vec![templated, plain]);
        let mut reversed_candidates = collection
            .candidates()
            .iter()
            .map(|(_, candidate)| OverloadCandidateInput {
                site: candidate.site_key.clone(),
                symbol: candidate.symbol.clone(),
                ordinary_root: candidate.ordinary_root.clone(),
                declaration_kind: candidate.declaration_kind.clone(),
                parameters: candidate.parameters.clone(),
                result: candidate.result,
                origin: candidate.origin.clone(),
                template: candidate.template.clone(),
                coherence: candidate.coherence,
                provenance: candidate.provenance.clone(),
            })
            .collect::<Vec<_>>();
        reversed_candidates.reverse();
        let permuted_collection = OverloadCollectionOutput::collect(sites, reversed_candidates);

        let output = TemplateExpansionOutput::expand(&collection);
        let permuted_output = TemplateExpansionOutput::expand(&permuted_collection);

        assert_eq!(output.debug_text(), permuted_output.debug_text());
        assert_eq!(output.expansions().len(), 1);
        let (_, expansion) = output.expansions().iter().next().expect("expansion");
        assert_eq!(expansion.status, TemplateExpansionStatus::Instantiated);
        assert_eq!(
            expansion.substitutions,
            [TemplateSubstitution {
                parameter: TemplateParameterKey::new("T"),
                value: NormalizedTypeId::new(7),
                source: TemplateSubstitutionSource::Explicit
            }]
        );
        let candidate_origins = output
            .candidates()
            .canonical_iter()
            .map(|(_, candidate)| &candidate.origin)
            .collect::<Vec<_>>();
        assert!(matches!(candidate_origins[0], CandidateOrigin::Ordinary));
        assert!(matches!(
            candidate_origins[1],
            CandidateOrigin::TemplateDerived { .. }
        ));
        let plain_candidate = output
            .candidates()
            .canonical_iter()
            .find_map(|(_, candidate)| {
                matches!(candidate.origin, CandidateOrigin::Ordinary).then_some(candidate)
            })
            .expect("plain candidate");
        assert_eq!(
            plain_candidate.parameters,
            [NormalizedTypeId::new(1), NormalizedTypeId::new(2)]
        );
        assert_eq!(plain_candidate.result, Some(NormalizedTypeId::new(3)));
        assert_eq!(plain_candidate.status, OverloadCandidateStatus::Collected);
        let expanded = output
            .candidates()
            .get(
                expansion
                    .instantiated_candidate
                    .expect("instantiated candidate"),
            )
            .expect("expanded candidate");
        assert!(expanded.template.is_none());
        assert_eq!(
            expanded.parameters,
            [NormalizedTypeId::new(7), NormalizedTypeId::new(8)]
        );
        assert_eq!(expanded.result, Some(NormalizedTypeId::new(9)));
        assert_eq!(expanded.status, OverloadCandidateStatus::Collected);
    }

    #[test]
    fn omitted_template_arguments_require_explicit_inference_payload() {
        let source_id = source_id(34);
        let mut inferred = candidate("call", "root", "inferred", CandidateScope::Local, 0);
        let mut inferred_payload = template_payload(
            "U=inferred",
            vec![TemplateParameterKey::new("U")],
            vec![TemplateArgument::Omitted(TemplateParameterKey::new("U"))],
        );
        inferred_payload.inferred_arguments = vec![TemplateArgumentInference {
            parameter: TemplateParameterKey::new("U"),
            inferred: NormalizedTypeId::new(11),
            evidence_key: CandidateProvenanceKey::new("exact-argument-type"),
        }];
        inferred.template = Some(inferred_payload);
        let mut missing = candidate("call", "root", "missing", CandidateScope::Local, 1);
        missing.template = Some(template_payload(
            "U=missing",
            vec![TemplateParameterKey::new("U")],
            vec![TemplateArgument::Omitted(TemplateParameterKey::new("U"))],
        ));
        let collection = OverloadCollectionOutput::collect(
            vec![site(
                "call",
                OverloadSiteKind::FunctorApplication,
                source_id,
                10,
            )],
            vec![missing, inferred],
        );

        let output = TemplateExpansionOutput::expand(&collection);

        assert_eq!(output.candidates().len(), 1);
        assert_eq!(
            output
                .expansions()
                .canonical_iter()
                .map(|(_, expansion)| &expansion.status)
                .collect::<Vec<_>>(),
            [
                &TemplateExpansionStatus::Instantiated,
                &TemplateExpansionStatus::Rejected(TemplateExpansionFailure::MissingInference)
            ]
        );
        assert!(
            output
                .debug_text()
                .contains("omitted_inference(\"exact-argument-type\")")
        );
        assert!(
            output
                .debug_text()
                .contains("message_key=\"overload.template.missing_inference\"")
        );
    }

    #[test]
    fn constraint_evidence_accepts_rejects_and_defers_templates() {
        let source_id = source_id(35);
        let accepted = constrained_template_candidate(
            "accepted",
            TemplateConstraintEvidenceStatus::Accepted,
            vec![TypeFactId::new(1)],
            0,
        );
        let missing = constrained_template_candidate(
            "missing",
            TemplateConstraintEvidenceStatus::Missing,
            Vec::new(),
            1,
        );
        let deferred = constrained_template_candidate(
            "deferred",
            TemplateConstraintEvidenceStatus::DeferredExternalDependency,
            Vec::new(),
            2,
        );
        let collection = OverloadCollectionOutput::collect(
            vec![site(
                "call",
                OverloadSiteKind::FunctorApplication,
                source_id,
                10,
            )],
            vec![deferred, accepted, missing],
        );

        let output = TemplateExpansionOutput::expand(&collection);

        assert_eq!(output.candidates().len(), 1);
        assert_eq!(
            output
                .expansions()
                .canonical_iter()
                .map(|(_, expansion)| &expansion.status)
                .collect::<Vec<_>>(),
            [
                &TemplateExpansionStatus::Instantiated,
                &TemplateExpansionStatus::Deferred(
                    TemplateExpansionFailure::DeferredConstraintEvidence
                ),
                &TemplateExpansionStatus::Rejected(
                    TemplateExpansionFailure::MissingConstraintEvidence
                )
            ]
        );
        let debug = output.debug_text();
        assert!(debug.contains("message_key=\"overload.template.missing_constraint_evidence\""));
        assert!(debug.contains("message_key=\"overload.template.deferred_constraint_evidence\""));
    }

    #[test]
    fn source_qua_template_arguments_must_be_accepted_widenings() {
        let source_id = source_id(36);
        let mut accepted = candidate("call", "root", "qua-ok", CandidateScope::Local, 0);
        accepted.template = Some(template_payload(
            "T=qua-ok",
            vec![TemplateParameterKey::new("T")],
            vec![TemplateArgument::SourceQua {
                source: NormalizedTypeId::new(3),
                target: NormalizedTypeId::new(4),
                path: QuaPathKey::new("widening-path"),
                status: TemplateQuaStatus::AcceptedWidening,
            }],
        ));
        let mut rejected = candidate("call", "root", "qua-bad", CandidateScope::Local, 1);
        rejected.template = Some(template_payload(
            "T=qua-bad",
            vec![TemplateParameterKey::new("T")],
            vec![TemplateArgument::SourceQua {
                source: NormalizedTypeId::new(4),
                target: NormalizedTypeId::new(3),
                path: QuaPathKey::new("narrowing-path"),
                status: TemplateQuaStatus::RejectedNarrowing,
            }],
        ));
        let mut deferred = candidate("call", "root", "qua-deferred", CandidateScope::Local, 2);
        deferred.template = Some(template_payload(
            "T=qua-deferred",
            vec![TemplateParameterKey::new("T")],
            vec![TemplateArgument::SourceQua {
                source: NormalizedTypeId::new(5),
                target: NormalizedTypeId::new(6),
                path: QuaPathKey::new("external-qua-path"),
                status: TemplateQuaStatus::DeferredExternalDependency,
            }],
        ));
        let collection = OverloadCollectionOutput::collect(
            vec![site(
                "call",
                OverloadSiteKind::FunctorApplication,
                source_id,
                10,
            )],
            vec![rejected, deferred, accepted],
        );

        let output = TemplateExpansionOutput::expand(&collection);

        assert_eq!(output.candidates().len(), 1);
        assert_eq!(
            output
                .expansions()
                .canonical_iter()
                .map(|(_, expansion)| &expansion.status)
                .collect::<Vec<_>>(),
            [
                &TemplateExpansionStatus::Rejected(TemplateExpansionFailure::RejectedSourceQua),
                &TemplateExpansionStatus::Deferred(TemplateExpansionFailure::DeferredSourceQua),
                &TemplateExpansionStatus::Instantiated
            ]
        );
        let debug = output.debug_text();
        assert!(debug.contains("source_qua(source=normalized_type#3, path=\"widening-path\")"));
        assert!(debug.contains("message_key=\"overload.template.rejected_source_qua\""));
        assert!(debug.contains("message_key=\"overload.template.deferred_source_qua\""));
    }

    #[test]
    fn template_expansion_diagnostics_do_not_reference_output_candidate_ids() {
        let source_id = source_id(37);
        let mut rejected = candidate("call", "root", "aaa-template", CandidateScope::Local, 0);
        rejected.template = Some(template_payload(
            "T=arity-mismatch",
            vec![TemplateParameterKey::new("T")],
            Vec::new(),
        ));
        let plain = candidate("call", "root", "zzz-plain", CandidateScope::Local, 1);
        let collection = OverloadCollectionOutput::collect(
            vec![site(
                "call",
                OverloadSiteKind::FunctorApplication,
                source_id,
                10,
            )],
            vec![rejected, plain],
        );

        let output = TemplateExpansionOutput::expand(&collection);

        assert_eq!(output.candidates().len(), 1);
        let (_, output_candidate) = output.candidates().iter().next().expect("candidate");
        assert_eq!(output_candidate.id, OverloadCandidateId::new(0));
        let (_, diagnostic) = output.diagnostics().iter().next().expect("diagnostic");
        assert_eq!(diagnostic.class, OverloadDiagnosticClass::TemplateExpansion);
        assert!(diagnostic.candidate.is_none());
        let (_, expansion) = output.expansions().iter().next().expect("expansion");
        assert_eq!(expansion.source_candidate, OverloadCandidateId::new(0));
    }

    #[test]
    fn non_template_candidate_diagnostics_are_remapped_into_expansion_output() {
        let source_id = source_id(38);
        let mut input = candidate("call", "root", "unsupported", CandidateScope::Local, 0);
        input.declaration_kind =
            CandidateDeclarationKind::Unsupported(UnsupportedOverloadRole::TheoremApplication);
        let collection = OverloadCollectionOutput::collect(
            vec![site(
                "call",
                OverloadSiteKind::FunctorApplication,
                source_id,
                10,
            )],
            vec![input],
        );

        let output = TemplateExpansionOutput::expand(&collection);

        let (_, candidate) = output.candidates().iter().next().expect("candidate");
        assert_eq!(candidate.id, OverloadCandidateId::new(0));
        assert_eq!(candidate.diagnostics.len(), 1);
        let diagnostic = output
            .diagnostics()
            .get(candidate.diagnostics[0])
            .expect("remapped diagnostic");
        assert_eq!(
            diagnostic.class,
            OverloadDiagnosticClass::UnsupportedCandidateRole
        );
        assert_eq!(diagnostic.candidate, Some(candidate.id));
        assert!(
            output
                .debug_text()
                .contains("class=unsupported_candidate_role")
        );
    }

    #[test]
    fn deferred_template_candidates_preserve_exclusion_reasons() {
        let source_id = source_id(39);
        let mut input = candidate("scheme", "root", "symbol", CandidateScope::Local, 0);
        input.template = Some(template_payload(
            "T=deferred",
            vec![TemplateParameterKey::new("T")],
            vec![TemplateArgument::Explicit(NormalizedTypeId::new(5))],
        ));
        let collection = OverloadCollectionOutput::collect(
            vec![site(
                "scheme",
                OverloadSiteKind::Unsupported(UnsupportedOverloadRole::SchemeApplication),
                source_id,
                10,
            )],
            vec![input],
        );

        let output = TemplateExpansionOutput::expand(&collection);

        assert!(output.candidates().is_empty());
        let (_, expansion) = output.expansions().iter().next().expect("expansion");
        assert_eq!(
            expansion.status,
            TemplateExpansionStatus::Deferred(TemplateExpansionFailure::DeferredCandidate)
        );
        assert_eq!(output.diagnostics().len(), 1);
        assert!(
            output
                .debug_text()
                .contains("message_key=\"overload.template.deferred_candidate\"")
        );
    }

    #[test]
    fn viability_accepts_exact_fact_and_source_qua_evidence_deterministically() {
        let source_id = source_id(40);
        let candidates = vec![
            candidate("call", "root", "exact", CandidateScope::Local, 0),
            candidate("call", "root", "fact", CandidateScope::Local, 1),
            candidate("call", "root", "qua", CandidateScope::Local, 2),
            candidate("call", "root", "widening", CandidateScope::Local, 3),
        ];
        let expansion = expanded_candidates(candidates, source_id);
        let inputs = viability_inputs_by_symbol(
            &expansion,
            [
                (
                    "exact",
                    vec![
                        ArgumentViabilityEvidence::Exact {
                            actual: NormalizedTypeId::new(1),
                        },
                        ArgumentViabilityEvidence::Exact {
                            actual: NormalizedTypeId::new(2),
                        },
                    ],
                ),
                (
                    "fact",
                    vec![
                        ArgumentViabilityEvidence::Exact {
                            actual: NormalizedTypeId::new(1),
                        },
                        ArgumentViabilityEvidence::FactWidening {
                            actual: NormalizedTypeId::new(20),
                            target: NormalizedTypeId::new(2),
                            facts: vec![TypeFactId::new(7)],
                            status: ViabilityFactStatus::Consumable,
                        },
                    ],
                ),
                (
                    "qua",
                    vec![
                        ArgumentViabilityEvidence::Exact {
                            actual: NormalizedTypeId::new(1),
                        },
                        ArgumentViabilityEvidence::Coercion {
                            actual: NormalizedTypeId::new(21),
                            target: NormalizedTypeId::new(2),
                            coercion: CoercionId::new(3),
                            kind: ViabilityCoercionKind::SourceQua,
                            status: ViabilityCoercionStatus::Accepted,
                            facts: vec![TypeFactId::new(8)],
                            path: Some(QuaPathKey::new("source-qua-path")),
                        },
                    ],
                ),
                (
                    "widening",
                    vec![
                        ArgumentViabilityEvidence::Exact {
                            actual: NormalizedTypeId::new(1),
                        },
                        ArgumentViabilityEvidence::Coercion {
                            actual: NormalizedTypeId::new(22),
                            target: NormalizedTypeId::new(2),
                            coercion: CoercionId::new(6),
                            kind: ViabilityCoercionKind::Widening,
                            status: ViabilityCoercionStatus::Accepted,
                            facts: vec![TypeFactId::new(9)],
                            path: Some(QuaPathKey::new("widening-path")),
                        },
                    ],
                ),
            ],
        );
        let mut reversed_inputs = inputs.clone();
        reversed_inputs.reverse();

        let output = CandidateViabilityOutput::filter(&expansion, inputs);
        let permuted_output = CandidateViabilityOutput::filter(&expansion, reversed_inputs);

        assert_eq!(output.debug_text(), permuted_output.debug_text());
        assert_eq!(output.candidates().len(), 4);
        assert!(output.diagnostics().is_empty());
        let debug = output.debug_text();
        assert!(debug.contains("kind=exact"));
        assert!(debug.contains("kind=fact_widening facts=[fact#7]"));
        assert!(debug.contains("kind=source_qua facts=[fact#8] coercion=coercion#3"));
        assert!(debug.contains("kind=coercion_widening facts=[fact#9] coercion=coercion#6"));
    }

    #[test]
    fn non_consumable_fact_evidence_rejects_with_stable_reasons() {
        let source_id = source_id(41);
        let expansion = expanded_candidates(
            vec![
                candidate("call", "root", "pending", CandidateScope::Local, 0),
                candidate("call", "root", "degraded", CandidateScope::Local, 1),
                candidate("call", "root", "rejected", CandidateScope::Local, 2),
                candidate("call", "root", "out-of-scope", CandidateScope::Local, 3),
            ],
            source_id,
        );
        let input_for = |status| {
            vec![
                ArgumentViabilityEvidence::Exact {
                    actual: NormalizedTypeId::new(1),
                },
                ArgumentViabilityEvidence::FactWidening {
                    actual: NormalizedTypeId::new(9),
                    target: NormalizedTypeId::new(2),
                    facts: vec![TypeFactId::new(1)],
                    status,
                },
            ]
        };
        let inputs = viability_inputs_by_symbol(
            &expansion,
            [
                ("pending", input_for(ViabilityFactStatus::PendingObligation)),
                ("degraded", input_for(ViabilityFactStatus::Degraded)),
                ("rejected", input_for(ViabilityFactStatus::Rejected)),
                (
                    "out-of-scope",
                    input_for(ViabilityFactStatus::OutOfScopeAssumption),
                ),
            ],
        );

        let output = CandidateViabilityOutput::filter(&expansion, inputs);

        assert!(output.candidates().is_empty());
        let reasons = output
            .decisions()
            .canonical_iter()
            .map(|(_, decision)| match &decision.status {
                CandidateViabilityStatus::Rejected { reasons } => reasons[0].reason,
                status => panic!("expected rejection, got {status:?}"),
            })
            .collect::<Vec<_>>();
        assert_eq!(
            reasons,
            [
                CandidateRejectionReason::DegradedEvidence,
                CandidateRejectionReason::OutOfScopeAssumption,
                CandidateRejectionReason::PendingEvidence,
                CandidateRejectionReason::RejectedEvidence,
            ]
        );
        let debug = output.debug_text();
        assert!(debug.contains("message_key=\"overload.viability.pending_evidence\""));
        assert!(debug.contains("message_key=\"overload.viability.degraded_evidence\""));
        assert!(debug.contains("message_key=\"overload.viability.rejected_evidence\""));
        assert!(debug.contains("message_key=\"overload.viability.out_of_scope_assumption\""));
    }

    #[test]
    fn narrowing_missing_and_ambiguous_evidence_do_not_become_viable() {
        let source_id = source_id(42);
        let expansion = expanded_candidates(
            vec![
                candidate("call", "root", "ambiguous", CandidateScope::Local, 0),
                candidate("call", "root", "missing", CandidateScope::Local, 1),
                candidate("call", "root", "narrowing", CandidateScope::Local, 2),
                candidate("call", "root", "blocked", CandidateScope::Local, 3),
            ],
            source_id,
        );
        let inputs = viability_inputs_by_symbol(
            &expansion,
            [
                (
                    "ambiguous",
                    vec![
                        ArgumentViabilityEvidence::Exact {
                            actual: NormalizedTypeId::new(1),
                        },
                        ArgumentViabilityEvidence::AmbiguousInheritance {
                            actual: NormalizedTypeId::new(9),
                            target: NormalizedTypeId::new(2),
                            paths: vec![QuaPathKey::new("left"), QuaPathKey::new("right")],
                        },
                    ],
                ),
                (
                    "missing",
                    vec![
                        ArgumentViabilityEvidence::Exact {
                            actual: NormalizedTypeId::new(1),
                        },
                        ArgumentViabilityEvidence::Missing {
                            actual: Some(NormalizedTypeId::new(9)),
                            target: NormalizedTypeId::new(2),
                        },
                    ],
                ),
                (
                    "narrowing",
                    vec![
                        ArgumentViabilityEvidence::Exact {
                            actual: NormalizedTypeId::new(1),
                        },
                        ArgumentViabilityEvidence::Coercion {
                            actual: NormalizedTypeId::new(2),
                            target: NormalizedTypeId::new(2),
                            coercion: CoercionId::new(4),
                            kind: ViabilityCoercionKind::Narrowing,
                            status: ViabilityCoercionStatus::Accepted,
                            facts: Vec::new(),
                            path: Some(QuaPathKey::new("narrowing")),
                        },
                    ],
                ),
                (
                    "blocked",
                    vec![
                        ArgumentViabilityEvidence::Exact {
                            actual: NormalizedTypeId::new(1),
                        },
                        ArgumentViabilityEvidence::Coercion {
                            actual: NormalizedTypeId::new(9),
                            target: NormalizedTypeId::new(2),
                            coercion: CoercionId::new(5),
                            kind: ViabilityCoercionKind::Widening,
                            status: ViabilityCoercionStatus::Blocked,
                            facts: Vec::new(),
                            path: Some(QuaPathKey::new("blocked-widening")),
                        },
                    ],
                ),
            ],
        );

        let output = CandidateViabilityOutput::filter(&expansion, inputs);

        assert!(output.candidates().is_empty());
        let debug = output.debug_text();
        assert!(debug.contains("blocked(reason={arg=1 reason=ambiguous_inheritance"));
        assert!(debug.contains("message_key=\"overload.viability.missing_evidence\""));
        assert!(debug.contains("message_key=\"overload.viability.invalid_narrowing\""));
        assert!(debug.contains("message_key=\"overload.viability.blocked_coercion\""));
    }

    #[test]
    fn missing_and_deferred_viability_payloads_block_without_output_candidate_ids() {
        let source_id = source_id(43);
        let expansion = expanded_candidates(
            vec![
                candidate("call", "root", "deferred", CandidateScope::Local, 0),
                candidate("call", "root", "missing-payload", CandidateScope::Local, 1),
            ],
            source_id,
        );
        let inputs = viability_inputs_by_symbol(
            &expansion,
            [(
                "deferred",
                vec![
                    ArgumentViabilityEvidence::Exact {
                        actual: NormalizedTypeId::new(1),
                    },
                    ArgumentViabilityEvidence::DeferredExternalDependency {
                        actual: Some(NormalizedTypeId::new(9)),
                        target: NormalizedTypeId::new(2),
                        reason: ViabilityEvidenceKey::new("missing-trace-fact"),
                    },
                ],
            )],
        );

        let output = CandidateViabilityOutput::filter(&expansion, inputs);

        assert!(output.candidates().is_empty());
        assert_eq!(output.diagnostics().len(), 2);
        for (_, diagnostic) in output.diagnostics().iter() {
            assert!(diagnostic.candidate.is_none());
        }
        let debug = output.debug_text();
        assert!(debug.contains("reason=deferred_external_dependency"));
        assert!(debug.contains("detail=\"missing-trace-fact\""));
        assert!(debug.contains("message_key=\"overload.viability.missing_viability_payload\""));
    }

    #[test]
    fn viability_remaps_existing_diagnostics_only_for_retained_candidates() {
        let source_id = source_id(44);
        let mut expansion = expanded_candidates(
            vec![
                candidate(
                    "call",
                    "root",
                    "rejected-with-diag",
                    CandidateScope::Local,
                    0,
                ),
                candidate("call", "root", "viable-with-diag", CandidateScope::Local, 1),
            ],
            source_id,
        );
        attach_existing_candidate_diagnostic(
            &mut expansion,
            "rejected-with-diag",
            "overload.collection.preexisting_rejected",
        );
        attach_existing_candidate_diagnostic(
            &mut expansion,
            "viable-with-diag",
            "overload.collection.preexisting_viable",
        );
        let inputs = viability_inputs_by_symbol(
            &expansion,
            [
                (
                    "rejected-with-diag",
                    vec![
                        ArgumentViabilityEvidence::Exact {
                            actual: NormalizedTypeId::new(1),
                        },
                        ArgumentViabilityEvidence::Missing {
                            actual: Some(NormalizedTypeId::new(9)),
                            target: NormalizedTypeId::new(2),
                        },
                    ],
                ),
                (
                    "viable-with-diag",
                    vec![
                        ArgumentViabilityEvidence::Exact {
                            actual: NormalizedTypeId::new(1),
                        },
                        ArgumentViabilityEvidence::Exact {
                            actual: NormalizedTypeId::new(2),
                        },
                    ],
                ),
            ],
        );

        let output = CandidateViabilityOutput::filter(&expansion, inputs);

        assert_eq!(output.candidates().len(), 1);
        let (_, viable) = output.candidates().iter().next().expect("viable");
        assert_eq!(viable.symbol.local().as_str(), "viable-with-diag");
        assert_eq!(viable.diagnostics.len(), 1);
        let remapped = output
            .diagnostics()
            .get(viable.diagnostics[0])
            .expect("remapped diagnostic");
        assert_eq!(remapped.candidate, Some(viable.id));
        assert_eq!(
            remapped.message_key.as_str(),
            "overload.collection.preexisting_viable"
        );
        let debug = output.debug_text();
        assert!(debug.contains("message_key=\"overload.collection.preexisting_viable\""));
        assert!(!debug.contains("message_key=\"overload.collection.preexisting_rejected\""));
        assert!(debug.contains("message_key=\"overload.viability.missing_evidence\""));
    }

    #[test]
    fn duplicate_and_unknown_viability_inputs_are_reported_deterministically() {
        let source_id = source_id(45);
        let expansion = expanded_candidates(
            vec![candidate("call", "root", "dup", CandidateScope::Local, 0)],
            source_id,
        );
        let duplicate_candidate = candidate_id_by_symbol(&expansion, "dup");
        let viable_input = CandidateViabilityInput {
            candidate: duplicate_candidate,
            arguments: vec![
                ArgumentViabilityEvidence::Exact {
                    actual: NormalizedTypeId::new(1),
                },
                ArgumentViabilityEvidence::Exact {
                    actual: NormalizedTypeId::new(2),
                },
            ],
        };
        let rejecting_input = CandidateViabilityInput {
            candidate: duplicate_candidate,
            arguments: vec![
                ArgumentViabilityEvidence::Exact {
                    actual: NormalizedTypeId::new(1),
                },
                ArgumentViabilityEvidence::Missing {
                    actual: Some(NormalizedTypeId::new(9)),
                    target: NormalizedTypeId::new(2),
                },
            ],
        };
        let unknown_input = CandidateViabilityInput {
            candidate: OverloadCandidateId::new(99),
            arguments: Vec::new(),
        };

        let output = CandidateViabilityOutput::filter(
            &expansion,
            vec![
                viable_input.clone(),
                unknown_input.clone(),
                rejecting_input.clone(),
            ],
        );
        let reversed_output = CandidateViabilityOutput::filter(
            &expansion,
            vec![rejecting_input, unknown_input, viable_input],
        );

        assert_eq!(output.debug_text(), reversed_output.debug_text());
        assert!(output.candidates().is_empty());
        let (_, decision) = output.decisions().iter().next().expect("decision");
        assert!(matches!(
            decision.status,
            CandidateViabilityStatus::Blocked {
                reason: CandidateBlockedReason {
                    reason: CandidateBlockedReasonKind::DuplicateViabilityPayload,
                    ..
                }
            }
        ));
        let debug = output.debug_text();
        assert!(debug.contains("message_key=\"overload.viability.duplicate_viability_payload\""));
        assert!(debug.contains("message_key=\"overload.viability.unknown_candidate_input.99\""));
    }

    #[test]
    fn specificity_graph_records_edges_equivalence_and_incomparability_deterministically() {
        let source_id = source_id(46);
        let viability = viable_output(
            vec![site(
                "call",
                OverloadSiteKind::FunctorApplication,
                source_id,
                10,
            )],
            vec![
                candidate("call", "root-a", "a", CandidateScope::Local, 0),
                candidate("call", "root-b", "b", CandidateScope::Local, 1),
                candidate("call", "root-c", "c", CandidateScope::Local, 2),
            ],
        );
        let a = candidate_id_by_symbol_in_table(viability.candidates(), "a");
        let b = candidate_id_by_symbol_in_table(viability.candidates(), "b");
        let c = candidate_id_by_symbol_in_table(viability.candidates(), "c");
        let comparisons = vec![
            SpecificityComparisonInput {
                left: a,
                right: b,
                status: SpecificityComparisonStatus::LeftAtLeastRight,
                reasons: vec![SpecificityReasonKey::new("a-subsumes-b")],
            },
            SpecificityComparisonInput {
                left: c,
                right: a,
                status: SpecificityComparisonStatus::Equivalent,
                reasons: vec![SpecificityReasonKey::new("same-parameters")],
            },
            SpecificityComparisonInput {
                left: b,
                right: c,
                status: SpecificityComparisonStatus::Incomparable,
                reasons: vec![SpecificityReasonKey::new("distinct-attributes")],
            },
        ];
        let mut reversed = comparisons.clone();
        reversed.reverse();

        let output = SpecificityGraphOutput::build(&viability, comparisons);
        let reversed_output = SpecificityGraphOutput::build(&viability, reversed);

        assert_eq!(output.debug_text(), reversed_output.debug_text());
        assert_eq!(output.candidates().len(), 3);
        assert_eq!(output.graphs().len(), 1);
        assert!(output.diagnostics().is_empty());
        let (_, graph) = output.graphs().iter().next().expect("graph");
        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(graph.comparisons.len(), 3);
        assert_eq!(graph.edges.len(), 3);
        let debug = output.debug_text();
        assert!(debug.contains("status=left_at_least_right"));
        assert!(debug.contains("status=equivalent"));
        assert!(debug.contains("status=incomparable"));
        assert!(debug.contains("edge#0 from=candidate#0 to=candidate#1"));
        assert!(debug.contains("edge#1 from=candidate#0 to=candidate#2"));
        assert!(debug.contains("edge#2 from=candidate#2 to=candidate#0"));
    }

    #[test]
    fn specificity_graphs_are_per_site_and_ignore_result_types() {
        let source_id = source_id(47);
        let mut left = candidate("call-a", "root-a", "left", CandidateScope::Local, 0);
        left.result = Some(NormalizedTypeId::new(100));
        let mut right = candidate("call-a", "root-b", "right", CandidateScope::Local, 1);
        right.result = Some(NormalizedTypeId::new(200));
        let other = candidate("call-b", "root-c", "other", CandidateScope::Local, 2);
        let viability = viable_output(
            vec![
                site(
                    "call-a",
                    OverloadSiteKind::FunctorApplication,
                    source_id,
                    10,
                ),
                site(
                    "call-b",
                    OverloadSiteKind::PredicateApplication,
                    source_id,
                    30,
                ),
            ],
            vec![left, right, other],
        );
        let left = candidate_id_by_symbol_in_table(viability.candidates(), "left");
        let right = candidate_id_by_symbol_in_table(viability.candidates(), "right");

        let output = SpecificityGraphOutput::build(
            &viability,
            [SpecificityComparisonInput {
                left,
                right,
                status: SpecificityComparisonStatus::RightAtLeastLeft,
                reasons: vec![SpecificityReasonKey::new("parameter-only-order")],
            }],
        );

        assert_eq!(output.graphs().len(), 2);
        let debug = output.debug_text();
        assert!(debug.contains("graph#0 site=site#0"));
        assert!(debug.contains("graph#1 site=site#1"));
        assert!(debug.contains("parameter-only-order"));
        assert!(debug.contains("edge#0 from=candidate#1 to=candidate#0"));
    }

    #[test]
    fn specificity_keeps_empty_graph_for_sites_without_viable_candidates() {
        let source_id = source_id(48);
        let expansion = expanded_candidates(
            vec![candidate(
                "call",
                "root",
                "rejected",
                CandidateScope::Local,
                0,
            )],
            source_id,
        );
        let rejected = candidate_id_by_symbol(&expansion, "rejected");
        let viability = CandidateViabilityOutput::filter(
            &expansion,
            [CandidateViabilityInput {
                candidate: rejected,
                arguments: vec![
                    ArgumentViabilityEvidence::Exact {
                        actual: NormalizedTypeId::new(1),
                    },
                    ArgumentViabilityEvidence::Missing {
                        actual: Some(NormalizedTypeId::new(9)),
                        target: NormalizedTypeId::new(2),
                    },
                ],
            }],
        );

        let output = SpecificityGraphOutput::build(&viability, []);

        assert!(output.candidates().is_empty());
        assert_eq!(output.graphs().len(), 1);
        let (_, graph) = output.graphs().iter().next().expect("graph");
        assert!(graph.nodes.is_empty());
        assert!(graph.comparisons.is_empty());
        assert!(graph.edges.is_empty());
        assert!(output.debug_text().contains("graph#0 site=site#0 nodes=[]"));
    }

    #[test]
    fn specificity_blocked_comparison_rows_do_not_create_edges() {
        let source_id = source_id(49);
        let viability = viable_output(
            vec![site(
                "call",
                OverloadSiteKind::FunctorApplication,
                source_id,
                10,
            )],
            vec![
                candidate("call", "root-a", "a", CandidateScope::Local, 0),
                candidate("call", "root-b", "b", CandidateScope::Local, 1),
                candidate("call", "root-c", "c", CandidateScope::Local, 2),
            ],
        );
        let a = candidate_id_by_symbol_in_table(viability.candidates(), "a");
        let b = candidate_id_by_symbol_in_table(viability.candidates(), "b");
        let c = candidate_id_by_symbol_in_table(viability.candidates(), "c");

        let output = SpecificityGraphOutput::build(
            &viability,
            [
                SpecificityComparisonInput {
                    left: a,
                    right: b,
                    status: SpecificityComparisonStatus::Blocked(
                        SpecificityBlockedReasonKind::DeferredExternalDependency,
                    ),
                    reasons: vec![SpecificityReasonKey::new("missing-comparison-summary")],
                },
                SpecificityComparisonInput {
                    left: a,
                    right: c,
                    status: SpecificityComparisonStatus::Blocked(
                        SpecificityBlockedReasonKind::MissingRecordedFacts,
                    ),
                    reasons: vec![SpecificityReasonKey::new("missing-fact")],
                },
                SpecificityComparisonInput {
                    left: b,
                    right: c,
                    status: SpecificityComparisonStatus::Incomparable,
                    reasons: vec![SpecificityReasonKey::new("incomparable")],
                },
            ],
        );

        let (_, graph) = output.graphs().iter().next().expect("graph");
        assert!(graph.edges.is_empty());
        let debug = output.debug_text();
        assert!(debug.contains("blocked(deferred_external_dependency)"));
        assert!(debug.contains("blocked(missing_recorded_facts)"));
        assert!(debug.contains("missing-comparison-summary"));
        assert!(debug.contains("missing-fact"));
        assert!(output.diagnostics().is_empty());
    }

    #[test]
    fn specificity_reports_missing_duplicate_unknown_and_cross_site_payloads() {
        let source_id = source_id(50);
        let viability = viable_output(
            vec![
                site(
                    "call-a",
                    OverloadSiteKind::FunctorApplication,
                    source_id,
                    10,
                ),
                site(
                    "call-b",
                    OverloadSiteKind::PredicateApplication,
                    source_id,
                    30,
                ),
            ],
            vec![
                candidate("call-a", "root-a", "a", CandidateScope::Local, 0),
                candidate("call-a", "root-b", "b", CandidateScope::Local, 1),
                candidate("call-a", "root-d", "d", CandidateScope::Local, 2),
                candidate("call-b", "root-c", "c", CandidateScope::Local, 3),
            ],
        );
        let a = candidate_id_by_symbol_in_table(viability.candidates(), "a");
        let b = candidate_id_by_symbol_in_table(viability.candidates(), "b");
        let c = candidate_id_by_symbol_in_table(viability.candidates(), "c");

        let duplicate = SpecificityComparisonInput {
            left: a,
            right: b,
            status: SpecificityComparisonStatus::LeftAtLeastRight,
            reasons: vec![SpecificityReasonKey::new("first")],
        };
        let cross_site = SpecificityComparisonInput {
            left: a,
            right: c,
            status: SpecificityComparisonStatus::LeftAtLeastRight,
            reasons: vec![SpecificityReasonKey::new("cross-site")],
        };
        let cross_site_reversed = SpecificityComparisonInput {
            left: c,
            right: a,
            status: SpecificityComparisonStatus::RightAtLeastLeft,
            reasons: vec![SpecificityReasonKey::new("cross-site-reversed")],
        };
        let unknown = SpecificityComparisonInput {
            left: OverloadCandidateId::new(99),
            right: c,
            status: SpecificityComparisonStatus::Incomparable,
            reasons: vec![SpecificityReasonKey::new("unknown")],
        };
        let comparisons = vec![
            duplicate.clone(),
            duplicate,
            cross_site,
            cross_site_reversed,
            unknown,
        ];
        let mut reversed = comparisons.clone();
        reversed.reverse();

        let output = SpecificityGraphOutput::build(&viability, comparisons);
        let reversed_output = SpecificityGraphOutput::build(&viability, reversed);

        assert_eq!(output.debug_text(), reversed_output.debug_text());
        let debug = output.debug_text();
        assert!(debug.contains("blocked(duplicate_comparison_payload)"));
        assert!(debug.contains("blocked(missing_comparison_payload)"));
        assert!(
            debug.contains("message_key=\"overload.specificity.duplicate_comparison_payload\"")
        );
        assert!(debug.contains("message_key=\"overload.specificity.missing_comparison_payload\""));
        assert!(debug.contains("message_key=\"overload.specificity.cross_site_comparison\""));
        assert!(debug.contains("message_key=\"overload.specificity.unknown_candidate\""));
    }

    #[test]
    fn selection_resolves_unique_maximal_root_and_inserted_widening_view() {
        let source_id = source_id(51);
        let ordinary = candidate("call", "root", "ordinary", CandidateScope::Local, 0);
        let mut refinement = candidate("call", "root", "refinement", CandidateScope::Local, 1);
        refinement.origin = CandidateOrigin::Redefinition {
            refined: symbol_id("ordinary"),
        };
        refinement.coherence = Some(CoherenceStatus::Accepted);
        refinement.result = Some(NormalizedTypeId::new(20));
        let other = candidate("call", "other-root", "other", CandidateScope::Local, 2);
        let viability = viable_output(
            vec![site(
                "call",
                OverloadSiteKind::FunctorApplication,
                source_id,
                10,
            )],
            vec![ordinary, refinement, other],
        );
        let ordinary = candidate_id_by_symbol_in_table(viability.candidates(), "ordinary");
        let refinement = candidate_id_by_symbol_in_table(viability.candidates(), "refinement");
        let other = candidate_id_by_symbol_in_table(viability.candidates(), "other");
        let graphs = SpecificityGraphOutput::build(
            &viability,
            [
                SpecificityComparisonInput {
                    left: refinement,
                    right: ordinary,
                    status: SpecificityComparisonStatus::Equivalent,
                    reasons: vec![SpecificityReasonKey::new("same-root-refinement")],
                },
                SpecificityComparisonInput {
                    left: refinement,
                    right: other,
                    status: SpecificityComparisonStatus::LeftAtLeastRight,
                    reasons: vec![SpecificityReasonKey::new("refinement-beats-other")],
                },
                SpecificityComparisonInput {
                    left: ordinary,
                    right: other,
                    status: SpecificityComparisonStatus::LeftAtLeastRight,
                    reasons: vec![SpecificityReasonKey::new("ordinary-beats-other")],
                },
            ],
        );
        let site = site_id_by_candidate_symbol(graphs.candidates(), "ordinary");

        let output = OverloadSelectionOutput::resolve(
            &graphs,
            [compatible_resolution_input(
                site,
                vec![refinement],
                Some(ExposedResultSource::StrongestRefinement),
                vec![
                    accepted_inserted_view(
                        TypedSiteRef::Node(TypedNodeId::new(101)),
                        NormalizedTypeId::new(2),
                        ordinary,
                        InsertedViewKind::Widening,
                        "argument-widening",
                    ),
                    accepted_inserted_view(
                        TypedSiteRef::Role {
                            node: TypedNodeId::new(102),
                            role: TypeRole::new("source-qua"),
                        },
                        NormalizedTypeId::new(22),
                        refinement,
                        InsertedViewKind::SourceQua,
                        "source-qua-view",
                    ),
                ],
            )],
        );

        assert_eq!(output.results().len(), 1);
        assert_eq!(output.inserted_views().len(), 2);
        assert!(output.diagnostics().is_empty());
        let (_, result) = output.results().iter().next().expect("selection result");
        let OverloadResultStatus::Resolved {
            root,
            refinements,
            exposed_result,
            inserted_views,
        } = &result.status
        else {
            panic!("expected resolved selection, got {:?}", result.status);
        };
        assert_eq!(*root, ordinary);
        assert_eq!(refinements, &[refinement]);
        assert_eq!(
            exposed_result.as_ref().map(|payload| &payload.source),
            Some(&ExposedResultSource::StrongestRefinement)
        );
        assert_eq!(inserted_views.len(), 2);
        let debug = output.debug_text();
        assert!(debug.contains("status=resolved(root=candidate#"));
        assert!(debug.contains("source=strongest_refinement"));
        assert!(debug.contains("kind=widening reason=\"argument-widening\""));
        assert!(debug.contains("kind=source_qua reason=\"source-qua-view\""));
    }

    #[test]
    fn selection_reports_no_match_and_ambiguity_as_failed_sites() {
        let source_id = source_id(52);
        let mut template_derived = candidate("ambiguous", "root-b", "b", CandidateScope::Local, 2);
        template_derived.origin = CandidateOrigin::TemplateDerived {
            template: symbol_id("template-b"),
            instantiation: TemplateInstantiationKey::new("T=ambiguous"),
        };
        template_derived.result = Some(NormalizedTypeId::new(200));
        let collection = OverloadCollectionOutput::collect(
            vec![
                site("empty", OverloadSiteKind::FunctorApplication, source_id, 10),
                site(
                    "ambiguous",
                    OverloadSiteKind::FunctorApplication,
                    source_id,
                    30,
                ),
            ],
            vec![
                candidate("empty", "empty-root", "rejected", CandidateScope::Local, 0),
                candidate("ambiguous", "root-a", "a", CandidateScope::Local, 1),
                template_derived,
            ],
        );
        let expansion = TemplateExpansionOutput::expand(&collection);
        let rejected = candidate_id_by_symbol(&expansion, "rejected");
        let a = candidate_id_by_symbol(&expansion, "a");
        let b = candidate_id_by_symbol(&expansion, "b");
        let viability = CandidateViabilityOutput::filter(
            &expansion,
            [
                CandidateViabilityInput {
                    candidate: rejected,
                    arguments: vec![
                        ArgumentViabilityEvidence::Exact {
                            actual: NormalizedTypeId::new(1),
                        },
                        ArgumentViabilityEvidence::Missing {
                            actual: Some(NormalizedTypeId::new(9)),
                            target: NormalizedTypeId::new(2),
                        },
                    ],
                },
                CandidateViabilityInput {
                    candidate: a,
                    arguments: vec![
                        ArgumentViabilityEvidence::Exact {
                            actual: NormalizedTypeId::new(1),
                        },
                        ArgumentViabilityEvidence::Exact {
                            actual: NormalizedTypeId::new(2),
                        },
                    ],
                },
                CandidateViabilityInput {
                    candidate: b,
                    arguments: vec![
                        ArgumentViabilityEvidence::Exact {
                            actual: NormalizedTypeId::new(1),
                        },
                        ArgumentViabilityEvidence::Exact {
                            actual: NormalizedTypeId::new(2),
                        },
                    ],
                },
            ],
        );
        let graphs = SpecificityGraphOutput::build(
            &viability,
            [SpecificityComparisonInput {
                left: candidate_id_by_symbol_in_table(viability.candidates(), "a"),
                right: candidate_id_by_symbol_in_table(viability.candidates(), "b"),
                status: SpecificityComparisonStatus::Incomparable,
                reasons: vec![SpecificityReasonKey::new("distinct-roots")],
            }],
        );

        let output = OverloadSelectionOutput::resolve(&graphs, []);

        assert_eq!(output.results().len(), 2);
        assert!(output.inserted_views().is_empty());
        let statuses = output
            .results()
            .iter()
            .map(|(_, result)| &result.status)
            .collect::<Vec<_>>();
        assert!(
            statuses
                .iter()
                .any(|status| matches!(status, OverloadResultStatus::NoMatch { .. }))
        );
        assert!(statuses
            .iter()
            .any(|status| matches!(status, OverloadResultStatus::Ambiguous { candidates } if candidates.len() == 2)));
        assert!(
            !statuses
                .iter()
                .any(|status| matches!(status, OverloadResultStatus::Resolved { .. }))
        );
        let debug = output.debug_text();
        assert!(debug.contains("status=no_match(rejected=[])"));
        assert!(debug.contains("status=ambiguous(candidates=[candidate#"));
        assert!(debug.contains("message_key=\"overload.selection.ambiguous_selection\""));
    }

    #[test]
    fn selection_keeps_equivalent_template_derived_roots_ambiguous() {
        let source_id = source_id(82);
        let mut template_a = candidate(
            "equivalent-templates",
            "template-root-a",
            "template-a",
            CandidateScope::Imported {
                module: module("pkg", "tmpl_a"),
                import_key: CandidateProvenanceKey::new("tmpl-a-import"),
            },
            0,
        );
        template_a.origin = CandidateOrigin::TemplateDerived {
            template: symbol_id("template-source-a"),
            instantiation: TemplateInstantiationKey::new("T=Concrete"),
        };
        template_a.result = Some(NormalizedTypeId::new(501));
        let mut template_b = candidate(
            "equivalent-templates",
            "template-root-b",
            "template-b",
            CandidateScope::Local,
            1,
        );
        template_b.origin = CandidateOrigin::TemplateDerived {
            template: symbol_id("template-source-b"),
            instantiation: TemplateInstantiationKey::new("U=Concrete"),
        };
        template_b.result = Some(NormalizedTypeId::new(777));
        let viability = viable_output(
            vec![site(
                "equivalent-templates",
                OverloadSiteKind::FunctorApplication,
                source_id,
                10,
            )],
            vec![template_b, template_a],
        );
        let template_a = candidate_id_by_symbol_in_table(viability.candidates(), "template-a");
        let template_b = candidate_id_by_symbol_in_table(viability.candidates(), "template-b");
        let graphs = SpecificityGraphOutput::build(
            &viability,
            [SpecificityComparisonInput {
                left: template_a,
                right: template_b,
                status: SpecificityComparisonStatus::Equivalent,
                reasons: vec![SpecificityReasonKey::new("equivalent-concrete-vectors")],
            }],
        );

        let output = OverloadSelectionOutput::resolve(&graphs, []);

        let (_, result) = output.results().iter().next().expect("selection result");
        let OverloadResultStatus::Ambiguous { candidates } = &result.status else {
            panic!(
                "expected ambiguous equivalent templates, got {:?}",
                result.status
            );
        };
        assert_eq!(candidates.len(), 2);
        assert!(candidates.contains(&template_a));
        assert!(candidates.contains(&template_b));
        let debug = output.debug_text();
        assert!(debug.contains("message_key=\"overload.selection.ambiguous_selection\""));
        assert!(!debug.contains("status=resolved("));
    }

    #[test]
    fn selection_uses_only_encoded_template_priority() {
        let source_id = source_id(83);
        let mut priority_template = candidate(
            "encoded-priority",
            "priority-template-root",
            "priority-template",
            CandidateScope::Imported {
                module: module("pkg", "priority_template"),
                import_key: CandidateProvenanceKey::new("priority-template-import"),
            },
            1,
        );
        priority_template.origin = CandidateOrigin::TemplateDerived {
            template: symbol_id("priority-template-source"),
            instantiation: TemplateInstantiationKey::new("T=Special"),
        };
        let mut strict_template = candidate(
            "template-strict",
            "strict-template-root",
            "strict-template",
            CandidateScope::Imported {
                module: module("pkg", "strict_template"),
                import_key: CandidateProvenanceKey::new("strict-template-import"),
            },
            3,
        );
        strict_template.origin = CandidateOrigin::TemplateDerived {
            template: symbol_id("strict-template-source"),
            instantiation: TemplateInstantiationKey::new("T=Narrow"),
        };
        let mut tied_template = candidate(
            "unencoded-tie",
            "tied-template-root",
            "tied-template",
            CandidateScope::Imported {
                module: module("pkg", "tied_template"),
                import_key: CandidateProvenanceKey::new("tied-template-import"),
            },
            5,
        );
        tied_template.origin = CandidateOrigin::TemplateDerived {
            template: symbol_id("tied-template-source"),
            instantiation: TemplateInstantiationKey::new("T=Equivalent"),
        };
        let viability = viable_output(
            vec![
                site(
                    "encoded-priority",
                    OverloadSiteKind::FunctorApplication,
                    source_id,
                    10,
                ),
                site(
                    "template-strict",
                    OverloadSiteKind::FunctorApplication,
                    source_id,
                    30,
                ),
                site(
                    "unencoded-tie",
                    OverloadSiteKind::FunctorApplication,
                    source_id,
                    50,
                ),
            ],
            vec![
                candidate(
                    "encoded-priority",
                    "priority-ordinary-root",
                    "priority-ordinary",
                    CandidateScope::Local,
                    0,
                ),
                priority_template,
                candidate(
                    "template-strict",
                    "strict-ordinary-root",
                    "strict-ordinary",
                    CandidateScope::Local,
                    2,
                ),
                strict_template,
                candidate(
                    "unencoded-tie",
                    "tied-ordinary-root",
                    "tied-ordinary",
                    CandidateScope::Local,
                    4,
                ),
                tied_template,
            ],
        );
        let priority_ordinary =
            candidate_id_by_symbol_in_table(viability.candidates(), "priority-ordinary");
        let priority_template =
            candidate_id_by_symbol_in_table(viability.candidates(), "priority-template");
        let strict_ordinary =
            candidate_id_by_symbol_in_table(viability.candidates(), "strict-ordinary");
        let strict_template =
            candidate_id_by_symbol_in_table(viability.candidates(), "strict-template");
        let tied_ordinary =
            candidate_id_by_symbol_in_table(viability.candidates(), "tied-ordinary");
        let tied_template =
            candidate_id_by_symbol_in_table(viability.candidates(), "tied-template");
        let graphs = SpecificityGraphOutput::build(
            &viability,
            [
                SpecificityComparisonInput {
                    left: priority_ordinary,
                    right: priority_template,
                    status: SpecificityComparisonStatus::LeftAtLeastRight,
                    reasons: vec![SpecificityReasonKey::new("encoded-non-template-priority")],
                },
                SpecificityComparisonInput {
                    left: strict_ordinary,
                    right: strict_template,
                    status: SpecificityComparisonStatus::RightAtLeastLeft,
                    reasons: vec![SpecificityReasonKey::new("template-strictly-narrower")],
                },
                SpecificityComparisonInput {
                    left: tied_ordinary,
                    right: tied_template,
                    status: SpecificityComparisonStatus::Equivalent,
                    reasons: vec![SpecificityReasonKey::new("priority-not-encoded")],
                },
            ],
        );
        let priority_site = site_id_by_candidate_symbol(graphs.candidates(), "priority-ordinary");
        let strict_site = site_id_by_candidate_symbol(graphs.candidates(), "strict-ordinary");
        let tied_site = site_id_by_candidate_symbol(graphs.candidates(), "tied-ordinary");

        let output = OverloadSelectionOutput::resolve(
            &graphs,
            [
                compatible_resolution_input(priority_site, Vec::new(), None, Vec::new()),
                compatible_resolution_input(strict_site, Vec::new(), None, Vec::new()),
            ],
        );
        let statuses = output
            .results()
            .iter()
            .map(|(_, result)| (result.site, &result.status))
            .collect::<BTreeMap<_, _>>();
        assert!(matches!(
            statuses.get(&priority_site),
            Some(OverloadResultStatus::Resolved { root, .. }) if *root == priority_ordinary
        ));
        assert!(matches!(
            statuses.get(&strict_site),
            Some(OverloadResultStatus::Resolved { root, .. }) if *root == strict_template
        ));
        assert!(matches!(
            statuses.get(&tied_site),
            Some(OverloadResultStatus::Ambiguous { candidates })
                if candidates.contains(&tied_ordinary) && candidates.contains(&tied_template)
        ));
        let graph_debug = graphs.debug_text();
        assert!(graph_debug.contains("encoded-non-template-priority"));
        assert!(graph_debug.contains("template-strictly-narrower"));
        assert!(graph_debug.contains("priority-not-encoded"));
        let debug = output.debug_text();
        assert!(debug.contains("message_key=\"overload.selection.ambiguous_selection\""));
    }

    #[test]
    fn selection_does_not_use_redefinition_metadata_to_break_root_ties() {
        let source_id = source_id(84);
        let root_a = candidate(
            "redefinition-tie",
            "root-a",
            "root-a-symbol",
            CandidateScope::Local,
            0,
        );
        let mut root_a_refinement = candidate(
            "redefinition-tie",
            "root-a",
            "root-a-refinement",
            CandidateScope::Local,
            1,
        );
        root_a_refinement.origin = CandidateOrigin::Redefinition {
            refined: symbol_id("root-a-symbol"),
        };
        root_a_refinement.coherence = Some(CoherenceStatus::Accepted);
        root_a_refinement.result = Some(NormalizedTypeId::new(901));
        let mut root_b = candidate(
            "redefinition-tie",
            "root-b",
            "root-b-symbol",
            CandidateScope::Imported {
                module: module("pkg", "root_b"),
                import_key: CandidateProvenanceKey::new("root-b-import"),
            },
            2,
        );
        root_b.result = Some(NormalizedTypeId::new(902));
        let viability = viable_output(
            vec![site(
                "redefinition-tie",
                OverloadSiteKind::FunctorApplication,
                source_id,
                10,
            )],
            vec![root_b, root_a_refinement, root_a],
        );
        let root_a = candidate_id_by_symbol_in_table(viability.candidates(), "root-a-symbol");
        let root_a_refinement =
            candidate_id_by_symbol_in_table(viability.candidates(), "root-a-refinement");
        let root_b = candidate_id_by_symbol_in_table(viability.candidates(), "root-b-symbol");
        let graphs = SpecificityGraphOutput::build(
            &viability,
            [
                SpecificityComparisonInput {
                    left: root_a,
                    right: root_a_refinement,
                    status: SpecificityComparisonStatus::Equivalent,
                    reasons: vec![SpecificityReasonKey::new("same-root-refinement")],
                },
                SpecificityComparisonInput {
                    left: root_a,
                    right: root_b,
                    status: SpecificityComparisonStatus::Equivalent,
                    reasons: vec![SpecificityReasonKey::new("root-tie")],
                },
                SpecificityComparisonInput {
                    left: root_a_refinement,
                    right: root_b,
                    status: SpecificityComparisonStatus::Equivalent,
                    reasons: vec![SpecificityReasonKey::new("refinement-result-not-tiebreak")],
                },
            ],
        );

        let output = OverloadSelectionOutput::resolve(&graphs, []);

        let (_, result) = output.results().iter().next().expect("selection result");
        let OverloadResultStatus::Ambiguous { candidates } = &result.status else {
            panic!(
                "expected redefinition metadata to preserve ambiguity, got {:?}",
                result.status
            );
        };
        assert!(candidates.contains(&root_a));
        assert!(candidates.contains(&root_a_refinement));
        assert!(candidates.contains(&root_b));
        assert!(
            graphs
                .debug_text()
                .contains("refinement-result-not-tiebreak")
        );
        let debug = output.debug_text();
        assert!(debug.contains("message_key=\"overload.selection.ambiguous_selection\""));
        assert!(!debug.contains("status=resolved("));
    }

    #[test]
    fn selection_reports_missing_duplicate_unknown_and_blocked_payloads() {
        let source_id = source_id(56);
        let mut malformed_refinement = candidate(
            "malformed",
            "malformed-root",
            "malformed-refinement",
            CandidateScope::Local,
            5,
        );
        malformed_refinement.origin = CandidateOrigin::Redefinition {
            refined: symbol_id("malformed-root-symbol"),
        };
        let mut ambiguous_template = candidate(
            "ambiguous-root",
            "ambiguous-root",
            "ambiguous-template",
            CandidateScope::Local,
            9,
        );
        ambiguous_template.origin = CandidateOrigin::TemplateDerived {
            template: symbol_id("ambiguous-template-source"),
            instantiation: TemplateInstantiationKey::new("T=ambiguous-root"),
        };
        let viability = viable_output(
            vec![
                site(
                    "missing",
                    OverloadSiteKind::FunctorApplication,
                    source_id,
                    10,
                ),
                site(
                    "duplicate",
                    OverloadSiteKind::FunctorApplication,
                    source_id,
                    30,
                ),
                site(
                    "blocked",
                    OverloadSiteKind::FunctorApplication,
                    source_id,
                    50,
                ),
                site(
                    "malformed",
                    OverloadSiteKind::FunctorApplication,
                    source_id,
                    70,
                ),
                site(
                    "ambiguous-root",
                    OverloadSiteKind::FunctorApplication,
                    source_id,
                    90,
                ),
            ],
            vec![
                candidate(
                    "missing",
                    "missing-root",
                    "missing-root-symbol",
                    CandidateScope::Local,
                    0,
                ),
                candidate(
                    "duplicate",
                    "duplicate-root",
                    "duplicate-root-symbol",
                    CandidateScope::Local,
                    1,
                ),
                candidate(
                    "blocked",
                    "blocked-root-a",
                    "blocked-a",
                    CandidateScope::Local,
                    2,
                ),
                candidate(
                    "blocked",
                    "blocked-root-b",
                    "blocked-b",
                    CandidateScope::Local,
                    3,
                ),
                candidate(
                    "malformed",
                    "malformed-root",
                    "malformed-root-symbol",
                    CandidateScope::Local,
                    4,
                ),
                malformed_refinement,
                candidate(
                    "ambiguous-root",
                    "ambiguous-root",
                    "ambiguous-ordinary",
                    CandidateScope::Local,
                    8,
                ),
                ambiguous_template,
            ],
        );
        let blocked_a = candidate_id_by_symbol_in_table(viability.candidates(), "blocked-a");
        let blocked_b = candidate_id_by_symbol_in_table(viability.candidates(), "blocked-b");
        let malformed_root =
            candidate_id_by_symbol_in_table(viability.candidates(), "malformed-root-symbol");
        let malformed_refinement =
            candidate_id_by_symbol_in_table(viability.candidates(), "malformed-refinement");
        let ambiguous_ordinary =
            candidate_id_by_symbol_in_table(viability.candidates(), "ambiguous-ordinary");
        let ambiguous_template =
            candidate_id_by_symbol_in_table(viability.candidates(), "ambiguous-template");
        let graphs = SpecificityGraphOutput::build(
            &viability,
            [
                SpecificityComparisonInput {
                    left: blocked_a,
                    right: blocked_b,
                    status: SpecificityComparisonStatus::Blocked(
                        SpecificityBlockedReasonKind::DeferredExternalDependency,
                    ),
                    reasons: vec![SpecificityReasonKey::new("external-specificity")],
                },
                SpecificityComparisonInput {
                    left: malformed_refinement,
                    right: malformed_root,
                    status: SpecificityComparisonStatus::LeftAtLeastRight,
                    reasons: vec![SpecificityReasonKey::new("refinement-only-maximal")],
                },
                SpecificityComparisonInput {
                    left: ambiguous_ordinary,
                    right: ambiguous_template,
                    status: SpecificityComparisonStatus::Equivalent,
                    reasons: vec![SpecificityReasonKey::new(
                        "missing-explicit-template-tie-breaker",
                    )],
                },
            ],
        );
        let duplicate_site =
            site_id_by_candidate_symbol(graphs.candidates(), "duplicate-root-symbol");
        let missing_site = site_id_by_candidate_symbol(graphs.candidates(), "missing-root-symbol");
        let blocked_site = site_id_by_candidate_symbol(graphs.candidates(), "blocked-a");
        let malformed_site =
            site_id_by_candidate_symbol(graphs.candidates(), "malformed-root-symbol");
        let ambiguous_root_site =
            site_id_by_candidate_symbol(graphs.candidates(), "ambiguous-ordinary");
        let duplicate_input =
            compatible_resolution_input(duplicate_site, Vec::new(), None, Vec::new());

        let output = OverloadSelectionOutput::resolve(
            &graphs,
            [
                duplicate_input.clone(),
                compatible_resolution_input(OverloadSiteId::new(99), Vec::new(), None, Vec::new()),
                compatible_resolution_input(malformed_site, Vec::new(), None, Vec::new()),
                compatible_resolution_input(ambiguous_root_site, Vec::new(), None, Vec::new()),
                duplicate_input,
            ],
        );

        let statuses = output
            .results()
            .iter()
            .map(|(_, result)| (result.site, &result.status))
            .collect::<BTreeMap<_, _>>();
        assert!(matches!(
            statuses.get(&missing_site),
            Some(OverloadResultStatus::Blocked {
                reason: OverloadBlockedReason::MissingSelectionPayload
            })
        ));
        assert!(matches!(
            statuses.get(&duplicate_site),
            Some(OverloadResultStatus::Blocked {
                reason: OverloadBlockedReason::DuplicateSelectionPayload
            })
        ));
        assert!(matches!(
            statuses.get(&blocked_site),
            Some(OverloadResultStatus::Blocked {
                reason: OverloadBlockedReason::BlockedSpecificityComparison
            })
        ));
        assert!(matches!(
            statuses.get(&malformed_site),
            Some(OverloadResultStatus::Blocked {
                reason: OverloadBlockedReason::MissingOrdinaryRootCandidate
            })
        ));
        assert!(matches!(
            statuses.get(&ambiguous_root_site),
            Some(OverloadResultStatus::Blocked {
                reason: OverloadBlockedReason::AmbiguousOrdinaryRootCandidate
            })
        ));
        let debug = output.debug_text();
        assert!(debug.contains("message_key=\"overload.selection.missing_selection_payload\""));
        assert!(debug.contains("message_key=\"overload.selection.duplicate_selection_payload\""));
        assert!(debug.contains("message_key=\"overload.selection.unknown_selection_site\""));
        assert!(
            debug.contains("message_key=\"overload.selection.blocked_specificity_comparison\"")
        );
        assert!(
            debug.contains("message_key=\"overload.selection.missing_ordinary_root_candidate\"")
        );
        assert!(
            debug.contains("message_key=\"overload.selection.ambiguous_ordinary_root_candidate\"")
        );
    }

    #[test]
    fn selection_blocks_redefinition_only_maximal_roots_before_ambiguity() {
        let source_id = source_id(57);
        let mut ref_a = candidate(
            "malformed",
            "root-a",
            "root-a-refinement",
            CandidateScope::Local,
            1,
        );
        ref_a.origin = CandidateOrigin::Redefinition {
            refined: symbol_id("root-a-symbol"),
        };
        ref_a.coherence = Some(CoherenceStatus::Accepted);
        let mut ref_b = candidate(
            "malformed",
            "root-b",
            "root-b-refinement",
            CandidateScope::Local,
            3,
        );
        ref_b.origin = CandidateOrigin::Redefinition {
            refined: symbol_id("root-b-symbol"),
        };
        ref_b.coherence = Some(CoherenceStatus::Accepted);
        let viability = viable_output(
            vec![site(
                "malformed",
                OverloadSiteKind::FunctorApplication,
                source_id,
                10,
            )],
            vec![
                candidate(
                    "malformed",
                    "root-a",
                    "root-a-symbol",
                    CandidateScope::Local,
                    0,
                ),
                ref_a,
                candidate(
                    "malformed",
                    "root-b",
                    "root-b-symbol",
                    CandidateScope::Local,
                    2,
                ),
                ref_b,
            ],
        );
        let root_a = candidate_id_by_symbol_in_table(viability.candidates(), "root-a-symbol");
        let ref_a = candidate_id_by_symbol_in_table(viability.candidates(), "root-a-refinement");
        let root_b = candidate_id_by_symbol_in_table(viability.candidates(), "root-b-symbol");
        let ref_b = candidate_id_by_symbol_in_table(viability.candidates(), "root-b-refinement");
        let graphs = SpecificityGraphOutput::build(
            &viability,
            [
                SpecificityComparisonInput {
                    left: ref_a,
                    right: root_a,
                    status: SpecificityComparisonStatus::LeftAtLeastRight,
                    reasons: vec![SpecificityReasonKey::new("ref-a-only-maximal")],
                },
                SpecificityComparisonInput {
                    left: ref_b,
                    right: root_b,
                    status: SpecificityComparisonStatus::LeftAtLeastRight,
                    reasons: vec![SpecificityReasonKey::new("ref-b-only-maximal")],
                },
                SpecificityComparisonInput {
                    left: root_a,
                    right: root_b,
                    status: SpecificityComparisonStatus::Incomparable,
                    reasons: vec![SpecificityReasonKey::new("root-cross")],
                },
                SpecificityComparisonInput {
                    left: root_a,
                    right: ref_b,
                    status: SpecificityComparisonStatus::Incomparable,
                    reasons: vec![SpecificityReasonKey::new("root-a-ref-b-cross")],
                },
                SpecificityComparisonInput {
                    left: ref_a,
                    right: root_b,
                    status: SpecificityComparisonStatus::Incomparable,
                    reasons: vec![SpecificityReasonKey::new("ref-a-root-b-cross")],
                },
                SpecificityComparisonInput {
                    left: ref_a,
                    right: ref_b,
                    status: SpecificityComparisonStatus::Incomparable,
                    reasons: vec![SpecificityReasonKey::new("ref-cross")],
                },
            ],
        );

        let output = OverloadSelectionOutput::resolve(&graphs, []);

        let (_, result) = output.results().iter().next().expect("selection result");
        assert!(matches!(
            result.status,
            OverloadResultStatus::Blocked {
                reason: OverloadBlockedReason::MissingOrdinaryRootCandidate
            }
        ));
        let debug = output.debug_text();
        assert!(
            debug.contains("message_key=\"overload.selection.missing_ordinary_root_candidate\"")
        );
        assert!(!debug.contains("status=ambiguous("));
    }

    #[test]
    fn selection_handles_refinement_join_payloads() {
        let source_id = source_id(53);
        let strongest_root = candidate(
            "strongest",
            "strongest-root",
            "strongest-root-symbol",
            CandidateScope::Local,
            0,
        );
        let mut strongest_refinement = candidate(
            "strongest",
            "strongest-root",
            "strongest-refinement",
            CandidateScope::Local,
            1,
        );
        strongest_refinement.origin = CandidateOrigin::Redefinition {
            refined: symbol_id("strongest-root-symbol"),
        };
        strongest_refinement.coherence = Some(CoherenceStatus::Accepted);
        let attribute_root = candidate(
            "attribute",
            "attribute-root",
            "attribute-root-symbol",
            CandidateScope::Local,
            2,
        );
        let mut attribute_refinement = candidate(
            "attribute",
            "attribute-root",
            "attribute-refinement",
            CandidateScope::Local,
            3,
        );
        attribute_refinement.origin = CandidateOrigin::Redefinition {
            refined: symbol_id("attribute-root-symbol"),
        };
        attribute_refinement.coherence = Some(CoherenceStatus::Accepted);
        let bad_root = candidate(
            "bad",
            "bad-root",
            "bad-root-symbol",
            CandidateScope::Local,
            4,
        );
        let mut bad_refinement = candidate(
            "bad",
            "bad-root",
            "bad-refinement",
            CandidateScope::Local,
            5,
        );
        bad_refinement.origin = CandidateOrigin::Redefinition {
            refined: symbol_id("bad-root-symbol"),
        };
        bad_refinement.coherence = Some(CoherenceStatus::Accepted);
        let viability = viable_output(
            vec![
                site(
                    "strongest",
                    OverloadSiteKind::FunctorApplication,
                    source_id,
                    10,
                ),
                site(
                    "attribute",
                    OverloadSiteKind::AttributeApplication,
                    source_id,
                    30,
                ),
                site("bad", OverloadSiteKind::FunctorApplication, source_id, 50),
            ],
            vec![
                strongest_root,
                strongest_refinement,
                attribute_root,
                attribute_refinement,
                bad_root,
                bad_refinement,
            ],
        );
        let strongest_root =
            candidate_id_by_symbol_in_table(viability.candidates(), "strongest-root-symbol");
        let strongest_refinement =
            candidate_id_by_symbol_in_table(viability.candidates(), "strongest-refinement");
        let attribute_root =
            candidate_id_by_symbol_in_table(viability.candidates(), "attribute-root-symbol");
        let attribute_refinement =
            candidate_id_by_symbol_in_table(viability.candidates(), "attribute-refinement");
        let bad_root = candidate_id_by_symbol_in_table(viability.candidates(), "bad-root-symbol");
        let bad_refinement =
            candidate_id_by_symbol_in_table(viability.candidates(), "bad-refinement");
        let graphs = SpecificityGraphOutput::build(
            &viability,
            [
                SpecificityComparisonInput {
                    left: strongest_refinement,
                    right: strongest_root,
                    status: SpecificityComparisonStatus::Equivalent,
                    reasons: vec![SpecificityReasonKey::new("strongest-result")],
                },
                SpecificityComparisonInput {
                    left: attribute_refinement,
                    right: attribute_root,
                    status: SpecificityComparisonStatus::Equivalent,
                    reasons: vec![SpecificityReasonKey::new("attribute-union")],
                },
                SpecificityComparisonInput {
                    left: bad_refinement,
                    right: bad_root,
                    status: SpecificityComparisonStatus::Equivalent,
                    reasons: vec![SpecificityReasonKey::new("bad-refinement")],
                },
            ],
        );
        let strongest_site =
            site_id_by_candidate_symbol(graphs.candidates(), "strongest-root-symbol");
        let attribute_site =
            site_id_by_candidate_symbol(graphs.candidates(), "attribute-root-symbol");
        let bad_site = site_id_by_candidate_symbol(graphs.candidates(), "bad-root-symbol");

        let output = OverloadSelectionOutput::resolve(
            &graphs,
            [
                compatible_resolution_input(
                    strongest_site,
                    vec![strongest_refinement],
                    Some(ExposedResultSource::StrongestRefinement),
                    Vec::new(),
                ),
                compatible_resolution_input(
                    attribute_site,
                    vec![attribute_refinement],
                    Some(ExposedResultSource::AttributeUnion),
                    Vec::new(),
                ),
                OverloadSiteResolutionInput {
                    site: bad_site,
                    refinements: vec![bad_refinement],
                    refinement_join: RefinementJoinPayload {
                        status: RefinementJoinStatus::Incompatible(
                            RefinementJoinFailure::ContradictoryAttributes,
                        ),
                        exposed_result: None,
                    },
                    inserted_views: Vec::new(),
                },
            ],
        );

        let statuses = output
            .results()
            .iter()
            .map(|(_, result)| (result.site, &result.status))
            .collect::<BTreeMap<_, _>>();
        assert!(matches!(
            statuses.get(&strongest_site),
            Some(OverloadResultStatus::Resolved {
                exposed_result: Some(ExposedResultPayload {
                    source: ExposedResultSource::StrongestRefinement,
                    ..
                }),
                ..
            })
        ));
        assert!(matches!(
            statuses.get(&attribute_site),
            Some(OverloadResultStatus::Resolved {
                exposed_result: Some(ExposedResultPayload {
                    source: ExposedResultSource::AttributeUnion,
                    ..
                }),
                ..
            })
        ));
        assert!(matches!(
            statuses.get(&bad_site),
            Some(OverloadResultStatus::IncompatibleRefinementJoin {
                reason: RefinementJoinFailure::ContradictoryAttributes,
                ..
            })
        ));
        let debug = output.debug_text();
        assert!(debug.contains("source=strongest_refinement"));
        assert!(debug.contains("source=attribute_union"));
        assert!(debug.contains("reason=contradictory_attributes"));
        assert!(
            debug
                .contains("message_key=\"overload.selection.refinement.contradictory_attributes\"")
        );
    }

    #[test]
    fn selection_rejects_non_selected_refinements_and_invalid_views() {
        let source_id = source_id(54);
        let viability = viable_output(
            vec![
                site(
                    "nonselected",
                    OverloadSiteKind::FunctorApplication,
                    source_id,
                    10,
                ),
                site(
                    "invalid-view",
                    OverloadSiteKind::FunctorApplication,
                    source_id,
                    30,
                ),
                site(
                    "narrow-view",
                    OverloadSiteKind::FunctorApplication,
                    source_id,
                    50,
                ),
                site(
                    "foreign-view",
                    OverloadSiteKind::FunctorApplication,
                    source_id,
                    70,
                ),
                site(
                    "ordinary-refinement",
                    OverloadSiteKind::FunctorApplication,
                    source_id,
                    90,
                ),
                site(
                    "rejected-refinement",
                    OverloadSiteKind::FunctorApplication,
                    source_id,
                    110,
                ),
            ],
            vec![
                candidate(
                    "nonselected",
                    "root-a",
                    "selected",
                    CandidateScope::Local,
                    0,
                ),
                candidate(
                    "nonselected",
                    "root-b",
                    "foreign-refinement",
                    CandidateScope::Local,
                    1,
                ),
                candidate(
                    "invalid-view",
                    "view-root",
                    "view-root-symbol",
                    CandidateScope::Local,
                    2,
                ),
                candidate(
                    "narrow-view",
                    "narrow-root",
                    "narrow-root-symbol",
                    CandidateScope::Local,
                    3,
                ),
                candidate(
                    "foreign-view",
                    "foreign-root-a",
                    "foreign-selected",
                    CandidateScope::Local,
                    4,
                ),
                candidate(
                    "foreign-view",
                    "foreign-root-b",
                    "foreign-view-candidate",
                    CandidateScope::Local,
                    5,
                ),
                candidate(
                    "ordinary-refinement",
                    "ordinary-root",
                    "ordinary-root-symbol",
                    CandidateScope::Local,
                    6,
                ),
                candidate(
                    "ordinary-refinement",
                    "ordinary-root",
                    "zz-ordinary-peer",
                    CandidateScope::Local,
                    7,
                ),
                candidate(
                    "rejected-refinement",
                    "rejected-root",
                    "rejected-root-symbol",
                    CandidateScope::Local,
                    8,
                ),
                {
                    let mut rejected = candidate(
                        "rejected-refinement",
                        "rejected-root",
                        "rejected-refinement-symbol",
                        CandidateScope::Local,
                        9,
                    );
                    rejected.origin = CandidateOrigin::Redefinition {
                        refined: symbol_id("rejected-root-symbol"),
                    };
                    rejected.coherence = Some(CoherenceStatus::Rejected);
                    rejected
                },
            ],
        );
        let selected = candidate_id_by_symbol_in_table(viability.candidates(), "selected");
        let foreign_refinement =
            candidate_id_by_symbol_in_table(viability.candidates(), "foreign-refinement");
        let view_root = candidate_id_by_symbol_in_table(viability.candidates(), "view-root-symbol");
        let narrow_root =
            candidate_id_by_symbol_in_table(viability.candidates(), "narrow-root-symbol");
        let foreign_selected =
            candidate_id_by_symbol_in_table(viability.candidates(), "foreign-selected");
        let foreign_view_candidate =
            candidate_id_by_symbol_in_table(viability.candidates(), "foreign-view-candidate");
        let ordinary_root =
            candidate_id_by_symbol_in_table(viability.candidates(), "ordinary-root-symbol");
        let ordinary_peer =
            candidate_id_by_symbol_in_table(viability.candidates(), "zz-ordinary-peer");
        let rejected_root =
            candidate_id_by_symbol_in_table(viability.candidates(), "rejected-root-symbol");
        let rejected_refinement =
            candidate_id_by_symbol_in_table(viability.candidates(), "rejected-refinement-symbol");
        let graphs = SpecificityGraphOutput::build(
            &viability,
            [
                SpecificityComparisonInput {
                    left: selected,
                    right: foreign_refinement,
                    status: SpecificityComparisonStatus::LeftAtLeastRight,
                    reasons: vec![SpecificityReasonKey::new("selected-root")],
                },
                SpecificityComparisonInput {
                    left: foreign_selected,
                    right: foreign_view_candidate,
                    status: SpecificityComparisonStatus::LeftAtLeastRight,
                    reasons: vec![SpecificityReasonKey::new("foreign-view-root")],
                },
                SpecificityComparisonInput {
                    left: ordinary_peer,
                    right: ordinary_root,
                    status: SpecificityComparisonStatus::RightAtLeastLeft,
                    reasons: vec![SpecificityReasonKey::new("same-root-not-refinement")],
                },
                SpecificityComparisonInput {
                    left: rejected_refinement,
                    right: rejected_root,
                    status: SpecificityComparisonStatus::Equivalent,
                    reasons: vec![SpecificityReasonKey::new("rejected-coherence")],
                },
            ],
        );
        let nonselected_site = site_id_by_candidate_symbol(graphs.candidates(), "selected");
        let invalid_view_site =
            site_id_by_candidate_symbol(graphs.candidates(), "view-root-symbol");
        let narrow_view_site =
            site_id_by_candidate_symbol(graphs.candidates(), "narrow-root-symbol");
        let foreign_view_site =
            site_id_by_candidate_symbol(graphs.candidates(), "foreign-selected");
        let ordinary_refinement_site =
            site_id_by_candidate_symbol(graphs.candidates(), "ordinary-root-symbol");
        let rejected_refinement_site =
            site_id_by_candidate_symbol(graphs.candidates(), "rejected-root-symbol");

        let output = OverloadSelectionOutput::resolve(
            &graphs,
            [
                compatible_resolution_input(
                    nonselected_site,
                    vec![foreign_refinement],
                    None,
                    Vec::new(),
                ),
                compatible_resolution_input(
                    invalid_view_site,
                    Vec::new(),
                    None,
                    vec![
                        accepted_inserted_view(
                            TypedSiteRef::Node(TypedNodeId::new(300)),
                            NormalizedTypeId::new(2),
                            view_root,
                            InsertedViewKind::Widening,
                            "would-leak-before-invalid",
                        ),
                        InsertedViewInput {
                            argument: TypedSiteRef::Node(TypedNodeId::new(301)),
                            target: NormalizedTypeId::new(2),
                            selected_candidate: view_root,
                            kind: InsertedViewKind::Widening,
                            status: InsertedViewStatus::MissingEvidence,
                            reason: InsertedViewReasonKey::new("missing-view-evidence"),
                            evidence_facts: Vec::new(),
                            path: Some(QuaPathKey::new("missing-view-path")),
                        },
                    ],
                ),
                compatible_resolution_input(
                    narrow_view_site,
                    Vec::new(),
                    None,
                    vec![InsertedViewInput {
                        argument: TypedSiteRef::Node(TypedNodeId::new(302)),
                        target: NormalizedTypeId::new(2),
                        selected_candidate: narrow_root,
                        kind: InsertedViewKind::Narrowing,
                        status: InsertedViewStatus::Accepted,
                        reason: InsertedViewReasonKey::new("narrowing-view"),
                        evidence_facts: vec![TypeFactId::new(2)],
                        path: Some(QuaPathKey::new("narrowing-view-path")),
                    }],
                ),
                compatible_resolution_input(
                    foreign_view_site,
                    Vec::new(),
                    None,
                    vec![accepted_inserted_view(
                        TypedSiteRef::Node(TypedNodeId::new(303)),
                        NormalizedTypeId::new(2),
                        foreign_view_candidate,
                        InsertedViewKind::Widening,
                        "foreign-view-candidate",
                    )],
                ),
                compatible_resolution_input(
                    ordinary_refinement_site,
                    vec![ordinary_peer],
                    None,
                    Vec::new(),
                ),
                compatible_resolution_input(
                    rejected_refinement_site,
                    vec![rejected_refinement],
                    None,
                    Vec::new(),
                ),
            ],
        );

        assert!(output.inserted_views().is_empty());
        let statuses = output
            .results()
            .iter()
            .map(|(_, result)| (result.site, &result.status))
            .collect::<BTreeMap<_, _>>();
        assert!(matches!(
            statuses.get(&nonselected_site),
            Some(OverloadResultStatus::Blocked {
                reason: OverloadBlockedReason::NonSelectedRootPayload
            })
        ));
        assert!(matches!(
            statuses.get(&invalid_view_site),
            Some(OverloadResultStatus::Blocked {
                reason: OverloadBlockedReason::InvalidInsertedView
            })
        ));
        assert!(matches!(
            statuses.get(&narrow_view_site),
            Some(OverloadResultStatus::Blocked {
                reason: OverloadBlockedReason::InvalidInsertedView
            })
        ));
        assert!(matches!(
            statuses.get(&foreign_view_site),
            Some(OverloadResultStatus::Blocked {
                reason: OverloadBlockedReason::InvalidInsertedView
            })
        ));
        assert!(matches!(
            statuses.get(&ordinary_refinement_site),
            Some(OverloadResultStatus::Blocked {
                reason: OverloadBlockedReason::NonSelectedRootPayload
            })
        ));
        assert!(matches!(
            statuses.get(&rejected_refinement_site),
            Some(OverloadResultStatus::Blocked {
                reason: OverloadBlockedReason::NonSelectedRootPayload
            })
        ));
        let debug = output.debug_text();
        assert!(debug.contains("message_key=\"overload.selection.non_selected_root_payload\""));
        assert!(debug.contains("message_key=\"overload.selection.invalid_inserted_view\""));
        assert!(!debug.contains("status=resolved("));
    }

    #[test]
    fn selection_rendering_is_deterministic_for_equivalent_payload_order() {
        let source_id = source_id(55);
        let ordinary = candidate(
            "stable",
            "stable-root",
            "stable-root-symbol",
            CandidateScope::Local,
            0,
        );
        let mut ref_a = candidate(
            "stable",
            "stable-root",
            "stable-ref-a",
            CandidateScope::Local,
            1,
        );
        ref_a.origin = CandidateOrigin::Redefinition {
            refined: symbol_id("stable-root-symbol"),
        };
        ref_a.coherence = Some(CoherenceStatus::Accepted);
        let mut ref_b = candidate(
            "stable",
            "stable-root",
            "stable-ref-b",
            CandidateScope::Local,
            2,
        );
        ref_b.origin = CandidateOrigin::Redefinition {
            refined: symbol_id("stable-root-symbol"),
        };
        ref_b.coherence = Some(CoherenceStatus::Accepted);
        let viability = viable_output(
            vec![site(
                "stable",
                OverloadSiteKind::FunctorApplication,
                source_id,
                10,
            )],
            vec![ordinary, ref_b, ref_a],
        );
        let ordinary =
            candidate_id_by_symbol_in_table(viability.candidates(), "stable-root-symbol");
        let ref_a = candidate_id_by_symbol_in_table(viability.candidates(), "stable-ref-a");
        let ref_b = candidate_id_by_symbol_in_table(viability.candidates(), "stable-ref-b");
        let graphs = SpecificityGraphOutput::build(
            &viability,
            [
                SpecificityComparisonInput {
                    left: ref_a,
                    right: ordinary,
                    status: SpecificityComparisonStatus::Equivalent,
                    reasons: vec![SpecificityReasonKey::new("ref-a-root")],
                },
                SpecificityComparisonInput {
                    left: ref_b,
                    right: ordinary,
                    status: SpecificityComparisonStatus::Equivalent,
                    reasons: vec![SpecificityReasonKey::new("ref-b-root")],
                },
                SpecificityComparisonInput {
                    left: ref_a,
                    right: ref_b,
                    status: SpecificityComparisonStatus::Equivalent,
                    reasons: vec![SpecificityReasonKey::new("same-root-equivalent")],
                },
            ],
        );
        let site = site_id_by_candidate_symbol(graphs.candidates(), "stable-root-symbol");
        let view_a = accepted_inserted_view(
            TypedSiteRef::Node(TypedNodeId::new(401)),
            NormalizedTypeId::new(2),
            ordinary,
            InsertedViewKind::Widening,
            "stable-view-a",
        );
        let view_b = accepted_inserted_view(
            TypedSiteRef::Node(TypedNodeId::new(402)),
            NormalizedTypeId::new(2),
            ref_a,
            InsertedViewKind::SourceQua,
            "stable-view-b",
        );

        let output = OverloadSelectionOutput::resolve(
            &graphs,
            [compatible_resolution_input(
                site,
                vec![ref_b, ref_a],
                Some(ExposedResultSource::AttributeUnion),
                vec![view_b.clone(), view_a.clone()],
            )],
        );
        let reversed_output = OverloadSelectionOutput::resolve(
            &graphs,
            [compatible_resolution_input(
                site,
                vec![ref_a, ref_b],
                Some(ExposedResultSource::AttributeUnion),
                vec![view_a, view_b],
            )],
        );

        assert_eq!(output.debug_text(), reversed_output.debug_text());
        assert_eq!(output.inserted_views().len(), 2);
        let debug = output.debug_text();
        assert!(debug.contains("refinements=[candidate#"));
        assert!(debug.contains("inserted_views=[view#0, view#1]"));
    }

    #[test]
    fn source_qua_and_recovery_site_provenance_are_retained() {
        let source_id = source_id(4);
        let mut input = site(
            "qua-site",
            OverloadSiteKind::FunctorApplication,
            source_id,
            10,
        );
        input.source_qua = vec![SourceQuaView {
            argument: TypedSiteRef::Role {
                node: TypedNodeId::new(2),
                role: TypeRole::new("argument"),
            },
            target: NormalizedTypeId::new(9),
            source_range: range(source_id, 12, 15),
            path: Some(QuaPathKey::new("mode-upcast")),
            evidence_facts: vec![TypeFactId::new(3)],
        }];
        input.recovery = OverloadSiteRecovery::Degraded {
            message_key: OverloadDiagnosticMessageKey::new("overload.site.recovered"),
        };

        let output = OverloadCollectionOutput::collect(vec![input], []);

        let (_, site) = output.sites().iter().next().expect("site collected");
        assert_eq!(site.status, OverloadSiteStatus::Degraded);
        assert_eq!(
            site.owner,
            TypedSiteRef::Node(TypedNodeId::new(1)),
            "owner is part of site provenance"
        );
        assert_range(site.source_range, 10, 15);
        assert_eq!(site.name.as_str(), "qua-site-name");
        assert_eq!(site.arguments, [TypedSiteRef::Node(TypedNodeId::new(101))]);
        assert_eq!(site.expected, Some(NormalizedTypeId::new(1)));
        assert_eq!(site.source_qua[0].target, NormalizedTypeId::new(9));
        assert_eq!(
            site.recovery,
            OverloadSiteRecovery::Degraded {
                message_key: OverloadDiagnosticMessageKey::new("overload.site.recovered")
            }
        );
        assert_eq!(output.diagnostics().len(), 1);
        let debug = output.debug_text();
        assert!(debug.contains("path=\"mode-upcast\""));
        assert!(debug.contains("facts=[fact#3]"));
        assert!(debug.contains("message_key=\"overload.site.recovered\""));
    }

    #[test]
    fn unknown_candidate_site_is_diagnosed_without_insertion() {
        let mut missing = candidate("missing", "root", "symbol", CandidateScope::Local, 0);
        missing.template = Some(TemplateCandidatePayload {
            template: symbol_id("missing-template"),
            instantiation_key: TemplateInstantiationKey::new("T=missing"),
            parameters: vec![TemplateParameterKey::new("T")],
            arguments: vec![TemplateArgument::Explicit(NormalizedTypeId::new(4))],
            inferred_arguments: Vec::new(),
            constraints: Vec::new(),
        });
        missing.coherence = Some(CoherenceStatus::Pending);
        let output = OverloadCollectionOutput::collect(Vec::new(), vec![missing]);

        assert!(output.sites().is_empty());
        assert!(output.candidates().is_empty());
        assert_eq!(output.diagnostics().len(), 1);
        let (_, diagnostic) = output
            .diagnostics()
            .iter()
            .next()
            .expect("missing site diagnostic");
        assert_eq!(diagnostic.class, OverloadDiagnosticClass::MissingSite);
        assert_eq!(
            diagnostic.site_key.as_ref().map(OverloadSiteKey::as_str),
            Some("missing")
        );
        let Some(OverloadDiagnosticProvenance::CandidateInput {
            declaration_kind,
            template,
            coherence,
            provenance,
            ..
        }) = &diagnostic.provenance
        else {
            panic!("missing candidate diagnostic keeps rejected candidate provenance");
        };
        assert_eq!(*declaration_kind, CandidateDeclarationKind::Functor);
        assert_eq!(*coherence, Some(CoherenceStatus::Pending));
        assert_eq!(
            template
                .as_ref()
                .map(|payload| payload.instantiation_key.as_str()),
            Some("T=missing")
        );
        assert_eq!(provenance.stable_key.as_str(), "symbol-provenance");
        assert_range(provenance.source_range.expect("candidate range"), 0, 1);
        assert_eq!(provenance.declaration_order, 0);
        assert!(output.debug_text().contains("coherence=pending"));
        assert!(output.debug_text().contains("instantiation=\"T=missing\""));
    }

    #[test]
    fn unsupported_roles_are_deferred_with_stable_diagnostics() {
        let source_id = source_id(5);
        let output = OverloadCollectionOutput::collect(
            vec![site(
                "scheme",
                OverloadSiteKind::Unsupported(UnsupportedOverloadRole::SchemeApplication),
                source_id,
                10,
            )],
            vec![OverloadCandidateInput {
                declaration_kind: CandidateDeclarationKind::Unsupported(
                    UnsupportedOverloadRole::TheoremApplication,
                ),
                ..candidate("scheme", "root", "symbol", CandidateScope::Local, 0)
            }],
        );

        let (_, site) = output.sites().iter().next().expect("site retained");
        assert_eq!(site.status, OverloadSiteStatus::Deferred);
        let (_, candidate) = output
            .candidates()
            .iter()
            .next()
            .expect("candidate retained");
        assert_eq!(candidate.status, OverloadCandidateStatus::Deferred);
        assert_eq!(output.diagnostics().len(), 2);
        let debug = output.debug_text();
        assert!(debug.contains("message_key=\"overload.site.unsupported.scheme_application\""));
        assert!(
            debug.contains("message_key=\"overload.candidate.unsupported.theorem_application\"")
        );
    }

    #[test]
    fn duplicate_site_keys_are_diagnosed_deterministically() {
        let source_id = source_id(6);
        let output = OverloadCollectionOutput::collect(
            vec![
                site("dup", OverloadSiteKind::FunctorApplication, source_id, 20),
                site("dup", OverloadSiteKind::PredicateApplication, source_id, 10),
            ],
            Vec::new(),
        );

        assert_eq!(output.sites().len(), 1);
        assert_eq!(output.diagnostics().len(), 1);
        let (_, diagnostic) = output.diagnostics().iter().next().expect("diagnostic");
        let Some(OverloadDiagnosticProvenance::SiteInput {
            source_range,
            kind,
            name,
            ..
        }) = &diagnostic.provenance
        else {
            panic!("duplicate site diagnostic keeps skipped site provenance");
        };
        assert_range(*source_range, 20, 25);
        assert_eq!(*kind, OverloadSiteKind::FunctorApplication);
        assert_eq!(name.as_str(), "dup-name");
        let debug = output.debug_text();
        assert!(debug.contains("key=\"dup\" kind=predicate_application"));
        assert!(debug.contains("class=duplicate_site_key"));
        assert!(debug.contains("provenance=site_input"));
    }

    fn site(
        key: &str,
        kind: OverloadSiteKind,
        source_id: SourceId,
        start: usize,
    ) -> OverloadSiteInput {
        OverloadSiteInput {
            key: OverloadSiteKey::new(key),
            owner: TypedSiteRef::Node(TypedNodeId::new(start / 10)),
            source_range: range(source_id, start, start + 5),
            kind,
            name: OverloadNameKey::new(format!("{key}-name")),
            arguments: vec![TypedSiteRef::Node(TypedNodeId::new((start / 10) + 100))],
            expected: Some(NormalizedTypeId::new(start / 10)),
            source_qua: Vec::new(),
            recovery: OverloadSiteRecovery::Normal,
        }
    }

    fn candidate(
        site: &str,
        root: &str,
        symbol: &str,
        scope: CandidateScope,
        declaration_order: usize,
    ) -> OverloadCandidateInput {
        OverloadCandidateInput {
            site: OverloadSiteKey::new(site),
            symbol: symbol_id(symbol),
            ordinary_root: symbol_id(root),
            declaration_kind: CandidateDeclarationKind::Functor,
            parameters: vec![NormalizedTypeId::new(1), NormalizedTypeId::new(2)],
            result: Some(NormalizedTypeId::new(3)),
            origin: CandidateOrigin::Ordinary,
            template: None,
            coherence: None,
            provenance: CandidateProvenance {
                stable_key: CandidateProvenanceKey::new(format!("{symbol}-provenance")),
                source_range: Some(range(
                    source_id(99),
                    declaration_order,
                    declaration_order + 1,
                )),
                scope,
                declaration_order,
            },
        }
    }

    fn template_payload(
        instantiation: &str,
        parameters: Vec<TemplateParameterKey>,
        arguments: Vec<TemplateArgument>,
    ) -> TemplateCandidatePayload {
        TemplateCandidatePayload {
            template: symbol_id("template-source"),
            instantiation_key: TemplateInstantiationKey::new(instantiation),
            parameters,
            arguments,
            inferred_arguments: Vec::new(),
            constraints: Vec::new(),
        }
    }

    fn constrained_template_candidate(
        symbol: &str,
        status: TemplateConstraintEvidenceStatus,
        facts: Vec<TypeFactId>,
        declaration_order: usize,
    ) -> OverloadCandidateInput {
        let mut input = candidate(
            "call",
            "root",
            symbol,
            CandidateScope::Local,
            declaration_order,
        );
        let mut payload = template_payload(
            &format!("T={symbol}"),
            vec![TemplateParameterKey::new("T")],
            vec![TemplateArgument::Explicit(NormalizedTypeId::new(7))],
        );
        payload.constraints = vec![TemplateConstraintEvidence {
            parameter: TemplateParameterKey::new("T"),
            evidence_key: CandidateProvenanceKey::new(format!("{symbol}-constraint")),
            facts,
            status,
        }];
        input.template = Some(payload);
        input
    }

    fn expanded_candidates(
        candidates: Vec<OverloadCandidateInput>,
        source_id: SourceId,
    ) -> TemplateExpansionOutput {
        let collection = OverloadCollectionOutput::collect(
            vec![site(
                "call",
                OverloadSiteKind::FunctorApplication,
                source_id,
                10,
            )],
            candidates,
        );
        TemplateExpansionOutput::expand(&collection)
    }

    fn viability_inputs_by_symbol<const N: usize>(
        expansion: &TemplateExpansionOutput,
        entries: [(&str, Vec<ArgumentViabilityEvidence>); N],
    ) -> Vec<CandidateViabilityInput> {
        entries
            .into_iter()
            .map(|(symbol, arguments)| {
                let candidate = candidate_id_by_symbol(expansion, symbol);
                CandidateViabilityInput {
                    candidate,
                    arguments,
                }
            })
            .collect()
    }

    fn viable_output(
        sites: Vec<OverloadSiteInput>,
        candidates: Vec<OverloadCandidateInput>,
    ) -> CandidateViabilityOutput {
        let collection = OverloadCollectionOutput::collect(sites, candidates);
        let expansion = TemplateExpansionOutput::expand(&collection);
        let inputs = expansion
            .candidates()
            .iter()
            .map(|(candidate, _)| CandidateViabilityInput {
                candidate,
                arguments: vec![
                    ArgumentViabilityEvidence::Exact {
                        actual: NormalizedTypeId::new(1),
                    },
                    ArgumentViabilityEvidence::Exact {
                        actual: NormalizedTypeId::new(2),
                    },
                ],
            })
            .collect::<Vec<_>>();
        CandidateViabilityOutput::filter(&expansion, inputs)
    }

    fn candidate_id_by_symbol(
        expansion: &TemplateExpansionOutput,
        symbol: &str,
    ) -> OverloadCandidateId {
        candidate_id_by_symbol_in_table(expansion.candidates(), symbol)
    }

    fn candidate_id_by_symbol_in_table(
        candidates: &OverloadCandidateTable,
        symbol: &str,
    ) -> OverloadCandidateId {
        candidates
            .iter()
            .find_map(|(id, candidate)| (candidate.symbol.local().as_str() == symbol).then_some(id))
            .expect("candidate symbol")
    }

    fn site_id_by_candidate_symbol(
        candidates: &OverloadCandidateTable,
        symbol: &str,
    ) -> OverloadSiteId {
        candidates
            .iter()
            .find_map(|(_, candidate)| {
                (candidate.symbol.local().as_str() == symbol).then_some(candidate.site)
            })
            .expect("candidate site")
    }

    fn compatible_resolution_input(
        site: OverloadSiteId,
        refinements: Vec<OverloadCandidateId>,
        source: Option<ExposedResultSource>,
        inserted_views: Vec<InsertedViewInput>,
    ) -> OverloadSiteResolutionInput {
        OverloadSiteResolutionInput {
            site,
            refinements,
            refinement_join: RefinementJoinPayload {
                status: RefinementJoinStatus::Compatible,
                exposed_result: source.map(|source| ExposedResultPayload {
                    result: Some(NormalizedTypeId::new(700 + site.index())),
                    source,
                    evidence: vec![TypeFactId::new(700 + site.index())],
                }),
            },
            inserted_views,
        }
    }

    fn accepted_inserted_view(
        argument: TypedSiteRef,
        target: NormalizedTypeId,
        selected_candidate: OverloadCandidateId,
        kind: InsertedViewKind,
        reason: &str,
    ) -> InsertedViewInput {
        InsertedViewInput {
            argument,
            target,
            selected_candidate,
            kind,
            status: InsertedViewStatus::Accepted,
            reason: InsertedViewReasonKey::new(reason),
            evidence_facts: vec![TypeFactId::new(9)],
            path: Some(QuaPathKey::new(format!("{reason}-path"))),
        }
    }

    fn attach_existing_candidate_diagnostic(
        expansion: &mut TemplateExpansionOutput,
        symbol: &str,
        message_key: &str,
    ) {
        let candidate_id = candidate_id_by_symbol(expansion, symbol);
        let candidate = expansion
            .candidates()
            .get(candidate_id)
            .expect("candidate")
            .clone();
        let diagnostic = expansion.diagnostics.insert(OverloadDiagnosticDraft {
            site: Some(candidate.site),
            site_key: Some(candidate.site_key.clone()),
            candidate: Some(candidate.id),
            provenance: Some(candidate_diagnostic_provenance_from_candidate(&candidate)),
            class: OverloadDiagnosticClass::Recovery,
            severity: OverloadDiagnosticSeverity::Note,
            message_key: OverloadDiagnosticMessageKey::new(message_key),
            recovery: OverloadDiagnosticRecovery::Degraded,
        });
        expansion
            .candidates
            .entries
            .get_mut(candidate_id.index())
            .expect("candidate")
            .diagnostics
            .push(diagnostic);
    }

    fn symbol_id(name: &str) -> SymbolId {
        SymbolId::new(
            module("work", "article"),
            LocalSymbolId::new(name),
            FullyQualifiedName::new(format!("work::article::{name}")),
        )
    }

    fn module(package: &str, path: &str) -> ModuleId {
        ModuleId::new(PackageId::new(package), ModulePath::new(path))
    }

    fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
        SourceRange {
            source_id,
            start,
            end,
        }
    }

    fn assert_range(range: SourceRange, start: usize, end: usize) {
        assert_eq!(range.start, start);
        assert_eq!(range.end, end);
    }

    fn source_id(seed: u8) -> SourceId {
        let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{seed:064x}"
        ))
        .expect("valid build snapshot id");
        InMemorySessionIdAllocator::new()
            .next_source_id(snapshot)
            .expect("source id allocation succeeds")
    }
}
