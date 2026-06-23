//! Final source-shaped resolved typed AST assembly for checker phase 8.

use crate::{
    cluster_trace::{ClusterFactId, ClusterFactTable},
    overload_resolution::{
        CandidateDeclarationKind, CandidateOrigin, CandidateProvenance, CandidateViabilityId,
        CandidateViabilityOutput, CandidateViabilityStatus, CoherenceStatus, ExposedResultPayload,
        InsertedViewId, InsertedViewKind, InsertedViewReasonKey, OverloadBlockedReason,
        OverloadCandidateId, OverloadCandidateStatus, OverloadCollectionOutput, OverloadDiagnostic,
        OverloadDiagnosticClass, OverloadDiagnosticId, OverloadDiagnosticSeverity,
        OverloadDiagnosticTable, OverloadResultStatus, OverloadSelectionOutput, OverloadSiteId,
        QuaPathKey, RefinementJoinFailure, SpecificityComparisonId, SpecificityComparisonOutcome,
        SpecificityEdge, SpecificityGraphId, SpecificityGraphOutput, SpecificityNode,
        SpecificityReasonKey, TemplateCandidatePayload, TemplateExpansionId,
        TemplateExpansionOutput, TemplateExpansionStatus, TemplateInstantiationKey,
        TemplateSubstitution,
    },
    typed_ast::{
        LocalTypeContextId, NodeRecoveryState, NormalizedTypeId, TypeDiagnosticId,
        TypeDiagnosticSeverity, TypeDiagnosticTable, TypeEntryActual, TypeFactId, TypedAst,
        TypedNodeId, TypedSiteRef, TypingState,
    },
};
use mizar_resolve::resolved_ast::{ModuleId, SymbolId};
use mizar_session::{GeneratedSpanAnchor, SourceAnchor, SourceId, SourceRange};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
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

dense_id!(ResolvedTypedNodeId);
dense_id!(ExpressionMetadataId);
dense_id!(OverloadResolutionId);
dense_id!(CoercionInsertionId);
dense_id!(ResolvedTypedDiagnosticId);

string_key!(ExprId);
string_key!(SourceNodeRole);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedTypedAst {
    source_id: SourceId,
    module_id: ModuleId,
    nodes: ResolvedTypedArena,
    expr_metadata: ExpressionMetadataTable,
    collection_candidates: OverloadCandidateSummaryTable,
    expanded_candidates: OverloadCandidateSummaryTable,
    template_expansions: TemplateExpansionSummaryTable,
    viable_candidates: OverloadCandidateSummaryTable,
    viability_decisions: CandidateViabilitySummaryTable,
    specificity_graphs: ResolvedSpecificityGraphTable,
    resolved_overloads: OverloadResolutionTable,
    inserted_coercions: CoercionInsertionTable,
    cluster_facts: ClusterFactTable,
    diagnostics: ResolvedTypedDiagnosticTable,
}

impl ResolvedTypedAst {
    pub fn assemble(inputs: ResolvedTypedAstInputs<'_>) -> Result<Self, ResolvedTypedAstError> {
        ResolvedTypedAstAssembler::new(inputs).assemble()
    }

    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub const fn nodes(&self) -> &ResolvedTypedArena {
        &self.nodes
    }

    pub const fn expr_metadata(&self) -> &ExpressionMetadataTable {
        &self.expr_metadata
    }

    pub const fn collection_candidates(&self) -> &OverloadCandidateSummaryTable {
        &self.collection_candidates
    }

    pub const fn expanded_candidates(&self) -> &OverloadCandidateSummaryTable {
        &self.expanded_candidates
    }

    pub const fn template_expansions(&self) -> &TemplateExpansionSummaryTable {
        &self.template_expansions
    }

    pub const fn viable_candidates(&self) -> &OverloadCandidateSummaryTable {
        &self.viable_candidates
    }

    pub const fn viability_decisions(&self) -> &CandidateViabilitySummaryTable {
        &self.viability_decisions
    }

    pub const fn specificity_graphs(&self) -> &ResolvedSpecificityGraphTable {
        &self.specificity_graphs
    }

    pub const fn resolved_overloads(&self) -> &OverloadResolutionTable {
        &self.resolved_overloads
    }

    pub const fn inserted_coercions(&self) -> &CoercionInsertionTable {
        &self.inserted_coercions
    }

    pub const fn cluster_facts(&self) -> &ClusterFactTable {
        &self.cluster_facts
    }

    pub const fn diagnostics(&self) -> &ResolvedTypedDiagnosticTable {
        &self.diagnostics
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("resolved-typed-ast-debug-v1\n");
        output.push_str("module: ");
        write_module_id(&mut output, &self.module_id);
        output.push('\n');
        output.push_str("root: ");
        write_optional_resolved_node_id(&mut output, self.nodes.root());
        output.push('\n');
        write_resolved_nodes(&mut output, &self.nodes);
        write_expression_metadata(&mut output, &self.expr_metadata);
        write_candidate_summaries(
            &mut output,
            "collection-candidates",
            &self.collection_candidates,
        );
        write_candidate_summaries(
            &mut output,
            "expanded-candidates",
            &self.expanded_candidates,
        );
        write_template_expansions(&mut output, &self.template_expansions);
        write_candidate_summaries(&mut output, "viable-candidates", &self.viable_candidates);
        write_viability_summaries(&mut output, &self.viability_decisions);
        write_specificity_graphs(&mut output, &self.specificity_graphs);
        write_overload_records(&mut output, &self.resolved_overloads);
        write_coercion_insertions(&mut output, &self.inserted_coercions);
        write_cluster_facts(&mut output, &self.cluster_facts);
        write_resolved_diagnostics(&mut output, &self.diagnostics);
        output
    }
}

#[derive(Debug)]
pub struct ResolvedTypedAstInputs<'a> {
    pub typed_ast: &'a TypedAst,
    pub cluster_facts: &'a ClusterFactTable,
    pub overload_collection: &'a OverloadCollectionOutput,
    pub template_expansion: &'a TemplateExpansionOutput,
    pub viability: &'a CandidateViabilityOutput,
    pub specificity: &'a SpecificityGraphOutput,
    pub overload_selection: &'a OverloadSelectionOutput,
    pub expressions: Vec<ExpressionMetadataInput>,
    pub node_hints: Vec<ResolvedNodeKindHint>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpressionMetadataInput {
    pub expr: ExprId,
    pub typed_site: TypedSiteRef,
    pub local_context: Option<LocalTypeContextId>,
    pub cluster_facts: Vec<ClusterFactId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedNodeKindHint {
    pub typed_node: TypedNodeId,
    pub kind: ResolvedNodeKindHintKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ResolvedNodeKindHintKind {
    SourcePreserved { role: SourceNodeRole },
    ResolvedUse { symbol: SymbolId },
    Degraded { reason: ResolvedNodeRecoveryReason },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedTypedArena {
    root: Option<ResolvedTypedNodeId>,
    nodes: Vec<ResolvedTypedNode>,
}

impl ResolvedTypedArena {
    pub const fn root(&self) -> Option<ResolvedTypedNodeId> {
        self.root
    }

    pub fn node(&self, id: ResolvedTypedNodeId) -> Option<&ResolvedTypedNode> {
        self.nodes.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (ResolvedTypedNodeId, &ResolvedTypedNode)> {
        self.nodes.iter().map(|entry| (entry.id, entry))
    }

    pub const fn len(&self) -> usize {
        self.nodes.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedTypedNode {
    pub id: ResolvedTypedNodeId,
    pub typed_node: TypedNodeId,
    pub source_range: SourceRange,
    pub children: Vec<ResolvedTypedNodeId>,
    pub kind: ResolvedTypedNodeKind,
    pub final_type: Option<NormalizedTypeId>,
    pub metadata: Option<ExpressionMetadataId>,
    pub diagnostics: Vec<ResolvedTypedDiagnosticId>,
    pub recovery: ResolvedNodeRecovery,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ResolvedTypedNodeKind {
    SourcePreserved { role: SourceNodeRole },
    ResolvedUse { symbol: SymbolId },
    FailedOverload { result: OverloadResolutionId },
    Degraded { reason: ResolvedNodeRecoveryReason },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ResolvedNodeRecovery {
    Normal,
    Recovered,
    Degraded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ResolvedNodeRecoveryReason {
    TypedRecovery(NodeRecoveryState),
    TypingState(TypingState),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpressionMetadata {
    pub id: ExpressionMetadataId,
    pub expr: ExprId,
    pub typed_site: TypedSiteRef,
    pub source_range: SourceRange,
    pub final_type: Option<NormalizedTypeId>,
    pub visible_facts: Vec<TypeFactId>,
    pub cluster_facts: Vec<ClusterFactId>,
    pub overload: Option<OverloadResolutionId>,
    pub inserted_views: Vec<CoercionInsertionId>,
    pub local_context: Option<LocalTypeContextId>,
    pub diagnostics: Vec<ResolvedTypedDiagnosticId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExpressionMetadataTable {
    entries: Vec<ExpressionMetadata>,
    by_expr: BTreeMap<ExprId, ExpressionMetadataId>,
    by_site: BTreeMap<TypedSiteRef, ExpressionMetadataId>,
}

impl ExpressionMetadataTable {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
            by_expr: BTreeMap::new(),
            by_site: BTreeMap::new(),
        }
    }

    pub fn get(&self, id: ExpressionMetadataId) -> Option<&ExpressionMetadata> {
        self.entries.get(id.index())
    }

    pub fn get_by_expr(&self, expr: &ExprId) -> Option<&ExpressionMetadata> {
        self.by_expr.get(expr).and_then(|id| self.get(*id))
    }

    pub fn id_by_expr(&self, expr: &ExprId) -> Option<ExpressionMetadataId> {
        self.by_expr.get(expr).copied()
    }

    pub fn get_by_site(&self, site: &TypedSiteRef) -> Option<&ExpressionMetadata> {
        self.by_site.get(site).and_then(|id| self.get(*id))
    }

    pub fn id_by_site(&self, site: &TypedSiteRef) -> Option<ExpressionMetadataId> {
        self.by_site.get(site).copied()
    }

    pub fn iter(&self) -> impl Iterator<Item = (ExpressionMetadataId, &ExpressionMetadata)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub fn canonical_iter(
        &self,
    ) -> impl Iterator<Item = (ExpressionMetadataId, &ExpressionMetadata)> {
        self.by_expr
            .values()
            .copied()
            .map(|id| (id, &self.entries[id.index()]))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn insert(
        &mut self,
        draft: ExpressionMetadataDraft,
    ) -> Result<ExpressionMetadataId, ResolvedTypedAstError> {
        if self.by_expr.contains_key(&draft.expr) {
            return Err(ResolvedTypedAstError::DuplicateExpression { expr: draft.expr });
        }
        if self.by_site.contains_key(&draft.typed_site) {
            return Err(ResolvedTypedAstError::DuplicateExpressionSite {
                site: draft.typed_site,
            });
        }
        let id = ExpressionMetadataId::new(self.entries.len());
        self.by_expr.insert(draft.expr.clone(), id);
        self.by_site.insert(draft.typed_site.clone(), id);
        self.entries.push(ExpressionMetadata {
            id,
            expr: draft.expr,
            typed_site: draft.typed_site,
            source_range: draft.source_range,
            final_type: draft.final_type,
            visible_facts: draft.visible_facts,
            cluster_facts: draft.cluster_facts,
            overload: draft.overload,
            inserted_views: draft.inserted_views,
            local_context: draft.local_context,
            diagnostics: draft.diagnostics,
        });
        Ok(id)
    }
}

struct ExpressionMetadataDraft {
    expr: ExprId,
    typed_site: TypedSiteRef,
    source_range: SourceRange,
    final_type: Option<NormalizedTypeId>,
    visible_facts: Vec<TypeFactId>,
    cluster_facts: Vec<ClusterFactId>,
    overload: Option<OverloadResolutionId>,
    inserted_views: Vec<CoercionInsertionId>,
    local_context: Option<LocalTypeContextId>,
    diagnostics: Vec<ResolvedTypedDiagnosticId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverloadCandidateSummary {
    pub candidate: OverloadCandidateId,
    pub site: OverloadSiteId,
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
    pub diagnostics: Vec<ResolvedTypedDiagnosticId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OverloadCandidateSummaryTable {
    entries: Vec<OverloadCandidateSummary>,
}

impl OverloadCandidateSummaryTable {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn get(&self, id: OverloadCandidateId) -> Option<&OverloadCandidateSummary> {
        self.entries.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (OverloadCandidateId, &OverloadCandidateSummary)> {
        self.entries.iter().map(|entry| (entry.candidate, entry))
    }

    pub fn canonical_iter(
        &self,
    ) -> impl Iterator<Item = (OverloadCandidateId, &OverloadCandidateSummary)> {
        let mut entries = self.entries.iter().collect::<Vec<_>>();
        entries.sort_by_key(|entry| candidate_summary_order_key(entry));
        entries.into_iter().map(|entry| (entry.candidate, entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn push(&mut self, summary: OverloadCandidateSummary) {
        self.entries.push(summary);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateExpansionSummary {
    pub id: TemplateExpansionId,
    pub source_candidate: OverloadCandidateId,
    pub site: OverloadSiteId,
    pub template: SymbolId,
    pub instantiation_key: TemplateInstantiationKey,
    pub substitutions: Vec<TemplateSubstitution>,
    pub instantiated_candidate: Option<OverloadCandidateId>,
    pub status: TemplateExpansionStatus,
    pub diagnostics: Vec<ResolvedTypedDiagnosticId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TemplateExpansionSummaryTable {
    entries: Vec<TemplateExpansionSummary>,
}

impl TemplateExpansionSummaryTable {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn get(&self, id: TemplateExpansionId) -> Option<&TemplateExpansionSummary> {
        self.entries.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (TemplateExpansionId, &TemplateExpansionSummary)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub fn canonical_iter(
        &self,
    ) -> impl Iterator<Item = (TemplateExpansionId, &TemplateExpansionSummary)> {
        let mut entries = self.entries.iter().collect::<Vec<_>>();
        entries.sort_by_key(|entry| template_expansion_summary_order_key(entry));
        entries.into_iter().map(|entry| (entry.id, entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn push(&mut self, summary: TemplateExpansionSummary) {
        self.entries.push(summary);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateViabilitySummary {
    pub id: CandidateViabilityId,
    pub source_candidate: OverloadCandidateId,
    pub site: OverloadSiteId,
    pub output_candidate: Option<OverloadCandidateId>,
    pub status: CandidateViabilityStatus,
    pub diagnostics: Vec<ResolvedTypedDiagnosticId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CandidateViabilitySummaryTable {
    entries: Vec<CandidateViabilitySummary>,
}

impl CandidateViabilitySummaryTable {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn get(&self, id: CandidateViabilityId) -> Option<&CandidateViabilitySummary> {
        self.entries.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (CandidateViabilityId, &CandidateViabilitySummary)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub fn canonical_iter(
        &self,
    ) -> impl Iterator<Item = (CandidateViabilityId, &CandidateViabilitySummary)> {
        let mut entries = self.entries.iter().collect::<Vec<_>>();
        entries.sort_by_key(|entry| viability_summary_order_key(entry));
        entries.into_iter().map(|entry| (entry.id, entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn push(&mut self, summary: CandidateViabilitySummary) {
        self.entries.push(summary);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedSpecificityComparison {
    pub id: SpecificityComparisonId,
    pub left: OverloadCandidateId,
    pub right: OverloadCandidateId,
    pub status: SpecificityComparisonOutcome,
    pub reasons: Vec<SpecificityReasonKey>,
    pub diagnostics: Vec<ResolvedTypedDiagnosticId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedSpecificityGraph {
    pub graph: SpecificityGraphId,
    pub site: OverloadSiteId,
    pub nodes: Vec<SpecificityNode>,
    pub comparisons: Vec<ResolvedSpecificityComparison>,
    pub edges: Vec<SpecificityEdge>,
    pub diagnostics: Vec<ResolvedTypedDiagnosticId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResolvedSpecificityGraphTable {
    entries: Vec<ResolvedSpecificityGraph>,
}

impl ResolvedSpecificityGraphTable {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn get(&self, id: SpecificityGraphId) -> Option<&ResolvedSpecificityGraph> {
        self.entries.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (SpecificityGraphId, &ResolvedSpecificityGraph)> {
        self.entries.iter().map(|entry| (entry.graph, entry))
    }

    pub fn canonical_iter(
        &self,
    ) -> impl Iterator<Item = (SpecificityGraphId, &ResolvedSpecificityGraph)> {
        let mut entries = self.entries.iter().collect::<Vec<_>>();
        entries.sort_by_key(|entry| specificity_graph_summary_order_key(entry));
        entries.into_iter().map(|entry| (entry.graph, entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn push(&mut self, graph: ResolvedSpecificityGraph) {
        self.entries.push(graph);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverloadResolutionRecord {
    pub id: OverloadResolutionId,
    pub site: OverloadSiteId,
    pub typed_site: TypedSiteRef,
    pub source_range: SourceRange,
    pub status: OverloadResolutionStatus,
    pub candidates: Vec<OverloadCandidateId>,
    pub specificity_graph: Option<SpecificityGraphId>,
    pub diagnostics: Vec<ResolvedTypedDiagnosticId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum OverloadResolutionStatus {
    Resolved {
        root: OverloadCandidateId,
        active_refinements: Vec<OverloadCandidateId>,
        exposed_result: Option<ExposedResultPayload>,
        inserted_views: Vec<CoercionInsertionId>,
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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OverloadResolutionTable {
    entries: Vec<OverloadResolutionRecord>,
}

impl OverloadResolutionTable {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn get(&self, id: OverloadResolutionId) -> Option<&OverloadResolutionRecord> {
        self.entries.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (OverloadResolutionId, &OverloadResolutionRecord)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub fn canonical_iter(
        &self,
    ) -> impl Iterator<Item = (OverloadResolutionId, &OverloadResolutionRecord)> {
        let mut entries = self.entries.iter().collect::<Vec<_>>();
        entries.sort_by_key(|entry| overload_resolution_order_key(entry));
        entries.into_iter().map(|entry| (entry.id, entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn push(&mut self, record: OverloadResolutionRecord) {
        self.entries.push(record);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoercionInsertion {
    pub id: CoercionInsertionId,
    pub typed_site: TypedSiteRef,
    pub source_range: SourceRange,
    pub target: NormalizedTypeId,
    pub selected_candidate: Option<OverloadCandidateId>,
    pub source: CoercionInsertionSource,
    pub reason: InsertedViewReasonKey,
    pub evidence_facts: Vec<TypeFactId>,
    pub path: Option<QuaPathKey>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CoercionInsertionSource {
    SourceQua,
    InsertedWidening,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CoercionInsertionTable {
    entries: Vec<CoercionInsertion>,
}

impl CoercionInsertionTable {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn get(&self, id: CoercionInsertionId) -> Option<&CoercionInsertion> {
        self.entries.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (CoercionInsertionId, &CoercionInsertion)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub fn canonical_iter(
        &self,
    ) -> impl Iterator<Item = (CoercionInsertionId, &CoercionInsertion)> {
        let mut entries = self.entries.iter().collect::<Vec<_>>();
        entries.sort_by_key(|entry| coercion_insertion_order_key(entry));
        entries.into_iter().map(|entry| (entry.id, entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn push(&mut self, insertion: CoercionInsertion) {
        self.entries.push(insertion);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedTypedDiagnostic {
    pub id: ResolvedTypedDiagnosticId,
    pub source_range: SourceRange,
    pub owner: Option<TypedSiteRef>,
    pub source: ResolvedTypedDiagnosticSource,
    pub severity: ResolvedTypedDiagnosticSeverity,
    pub message_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ResolvedTypedDiagnosticSource {
    Type(TypeDiagnosticId),
    OverloadCollection(OverloadDiagnosticId),
    TemplateExpansion(OverloadDiagnosticId),
    Viability(OverloadDiagnosticId),
    Specificity(OverloadDiagnosticId),
    Selection(OverloadDiagnosticId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ResolvedTypedDiagnosticSeverity {
    Error,
    Warning,
    Note,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResolvedTypedDiagnosticTable {
    entries: Vec<ResolvedTypedDiagnostic>,
}

impl ResolvedTypedDiagnosticTable {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn get(&self, id: ResolvedTypedDiagnosticId) -> Option<&ResolvedTypedDiagnostic> {
        self.entries.get(id.index())
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (ResolvedTypedDiagnosticId, &ResolvedTypedDiagnostic)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub fn canonical_iter(
        &self,
    ) -> impl Iterator<Item = (ResolvedTypedDiagnosticId, &ResolvedTypedDiagnostic)> {
        let mut entries = self.entries.iter().collect::<Vec<_>>();
        entries.sort_by_key(|entry| diagnostic_order_key(entry));
        entries.into_iter().map(|entry| (entry.id, entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn push(&mut self, diagnostic: ResolvedTypedDiagnostic) {
        self.entries.push(diagnostic);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CandidateSummaryNamespace {
    Collection,
    Expanded,
    Viable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ResolvedTypedAstError {
    DuplicateExpression {
        expr: ExprId,
    },
    DuplicateExpressionSite {
        site: TypedSiteRef,
    },
    DuplicateNodeHint {
        typed_node: TypedNodeId,
    },
    InvalidTypedNode {
        typed_node: TypedNodeId,
    },
    InvalidTypedChild {
        typed_node: TypedNodeId,
        child: TypedNodeId,
    },
    InvalidTypedSite {
        site: TypedSiteRef,
    },
    InvalidLocalContext {
        context: LocalTypeContextId,
    },
    InvalidClusterFact {
        fact: ClusterFactId,
    },
    InvalidOverloadSite {
        site: OverloadSiteId,
    },
    InvalidOverloadCandidate {
        namespace: CandidateSummaryNamespace,
        candidate: OverloadCandidateId,
    },
    InvalidSpecificityGraph {
        graph: SpecificityGraphId,
    },
    InvalidInsertedView {
        view: InsertedViewId,
    },
    UnsupportedInsertedViewKind {
        view: InsertedViewId,
        kind: InsertedViewKind,
    },
}

impl fmt::Display for ResolvedTypedAstError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateExpression { expr } => {
                write!(
                    formatter,
                    "duplicate expression metadata `{}`",
                    expr.as_str()
                )
            }
            Self::DuplicateExpressionSite { site } => {
                write!(formatter, "duplicate expression metadata site `{site:?}`")
            }
            Self::DuplicateNodeHint { typed_node } => {
                write!(
                    formatter,
                    "duplicate resolved node hint for typed node {}",
                    typed_node.index()
                )
            }
            Self::InvalidTypedNode { typed_node } => {
                write!(
                    formatter,
                    "typed node {} does not exist",
                    typed_node.index()
                )
            }
            Self::InvalidTypedChild { typed_node, child } => write!(
                formatter,
                "typed node {} references missing child {}",
                typed_node.index(),
                child.index()
            ),
            Self::InvalidTypedSite { site } => {
                write!(formatter, "typed site {:?} does not exist", site)
            }
            Self::InvalidLocalContext { context } => {
                write!(
                    formatter,
                    "local context {} does not exist",
                    context.index()
                )
            }
            Self::InvalidClusterFact { fact } => {
                write!(formatter, "cluster fact {} does not exist", fact.index())
            }
            Self::InvalidOverloadSite { site } => {
                write!(formatter, "overload site {} does not exist", site.index())
            }
            Self::InvalidOverloadCandidate {
                namespace,
                candidate,
            } => write!(
                formatter,
                "{namespace:?} overload candidate {} does not exist",
                candidate.index()
            ),
            Self::InvalidSpecificityGraph { graph } => {
                write!(
                    formatter,
                    "specificity graph {} does not exist",
                    graph.index()
                )
            }
            Self::InvalidInsertedView { view } => {
                write!(formatter, "inserted view {} does not exist", view.index())
            }
            Self::UnsupportedInsertedViewKind { view, kind } => write!(
                formatter,
                "inserted view {} has unsupported kind {:?}",
                view.index(),
                kind
            ),
        }
    }
}

impl Error for ResolvedTypedAstError {}

struct ResolvedTypedAstAssembler<'a> {
    inputs: ResolvedTypedAstInputs<'a>,
}

impl<'a> ResolvedTypedAstAssembler<'a> {
    fn new(inputs: ResolvedTypedAstInputs<'a>) -> Self {
        Self { inputs }
    }

    fn assemble(self) -> Result<ResolvedTypedAst, ResolvedTypedAstError> {
        let source_id = self.inputs.typed_ast.source_id();
        let module_id = self.inputs.typed_ast.module_id().clone();
        let root_range = root_range(self.inputs.typed_ast);
        let diagnostics = build_diagnostics(&self.inputs, root_range);
        let collection_candidates = copy_candidate_summaries(
            self.inputs.overload_collection.candidates(),
            &diagnostics.collection,
        );
        let expanded_candidates = copy_candidate_summaries(
            self.inputs.template_expansion.candidates(),
            &diagnostics.template_expansion,
        );
        let viable_candidates =
            copy_candidate_summaries(self.inputs.viability.candidates(), &diagnostics.viability);
        let template_expansions = copy_template_expansions(
            self.inputs.template_expansion.expansions(),
            &diagnostics.template_expansion,
        );
        let viability_decisions =
            copy_viability_decisions(self.inputs.viability, &diagnostics.viability);
        let specificity_graphs =
            copy_specificity_graphs(self.inputs.specificity, &diagnostics.specificity);
        let overload_projection = build_overload_projection(
            &self.inputs,
            &diagnostics.specificity,
            &diagnostics.selection,
        )?;
        let expr_metadata = build_expression_metadata(
            &self.inputs,
            &overload_projection,
            &viable_candidates,
            &diagnostics,
        )?;
        let nodes = build_resolved_nodes(
            &self.inputs,
            &expr_metadata,
            &overload_projection,
            &viable_candidates,
            &diagnostics.type_diagnostics,
        )?;

        Ok(ResolvedTypedAst {
            source_id,
            module_id,
            nodes,
            expr_metadata,
            collection_candidates,
            expanded_candidates,
            template_expansions,
            viable_candidates,
            viability_decisions,
            specificity_graphs,
            resolved_overloads: overload_projection.records,
            inserted_coercions: overload_projection.insertions,
            cluster_facts: self.inputs.cluster_facts.clone(),
            diagnostics: diagnostics.table,
        })
    }
}

struct ResolvedDiagnosticMaps {
    table: ResolvedTypedDiagnosticTable,
    type_diagnostics: BTreeMap<TypeDiagnosticId, ResolvedTypedDiagnosticId>,
    collection: BTreeMap<OverloadDiagnosticId, ResolvedTypedDiagnosticId>,
    template_expansion: BTreeMap<OverloadDiagnosticId, ResolvedTypedDiagnosticId>,
    viability: BTreeMap<OverloadDiagnosticId, ResolvedTypedDiagnosticId>,
    specificity: BTreeMap<OverloadDiagnosticId, ResolvedTypedDiagnosticId>,
    selection: BTreeMap<OverloadDiagnosticId, ResolvedTypedDiagnosticId>,
}

struct OverloadProjection {
    records: OverloadResolutionTable,
    insertions: CoercionInsertionTable,
    by_typed_site: BTreeMap<TypedSiteRef, OverloadResolutionId>,
    inserted_by_typed_site: BTreeMap<TypedSiteRef, Vec<CoercionInsertionId>>,
}

fn build_diagnostics(
    inputs: &ResolvedTypedAstInputs<'_>,
    root_range: SourceRange,
) -> ResolvedDiagnosticMaps {
    let mut table = ResolvedTypedDiagnosticTable::new();
    let type_diagnostics = copy_type_diagnostics(&mut table, inputs.typed_ast.diagnostics());
    let collection = copy_overload_diagnostics(
        &mut table,
        inputs.overload_collection.diagnostics(),
        ResolvedTypedDiagnosticSource::OverloadCollection,
        inputs.overload_collection,
        Some(inputs.overload_collection.candidates()),
        root_range,
        |_| true,
    );
    let template_expansion = copy_overload_diagnostics(
        &mut table,
        inputs.template_expansion.diagnostics(),
        ResolvedTypedDiagnosticSource::TemplateExpansion,
        inputs.overload_collection,
        Some(inputs.template_expansion.candidates()),
        root_range,
        |_| true,
    );
    let viability = copy_overload_diagnostics(
        &mut table,
        inputs.viability.diagnostics(),
        ResolvedTypedDiagnosticSource::Viability,
        inputs.overload_collection,
        Some(inputs.viability.candidates()),
        root_range,
        |_| true,
    );
    let specificity = copy_overload_diagnostics(
        &mut table,
        inputs.specificity.diagnostics(),
        ResolvedTypedDiagnosticSource::Specificity,
        inputs.overload_collection,
        Some(inputs.specificity.candidates()),
        root_range,
        |diagnostic| diagnostic.class == OverloadDiagnosticClass::Specificity,
    );
    let selection = copy_overload_diagnostics(
        &mut table,
        inputs.overload_selection.diagnostics(),
        ResolvedTypedDiagnosticSource::Selection,
        inputs.overload_collection,
        Some(inputs.specificity.candidates()),
        root_range,
        |diagnostic| diagnostic.class == OverloadDiagnosticClass::Selection,
    );

    ResolvedDiagnosticMaps {
        table,
        type_diagnostics,
        collection,
        template_expansion,
        viability,
        specificity,
        selection,
    }
}

fn copy_type_diagnostics(
    output: &mut ResolvedTypedDiagnosticTable,
    input: &TypeDiagnosticTable,
) -> BTreeMap<TypeDiagnosticId, ResolvedTypedDiagnosticId> {
    let mut map = BTreeMap::new();
    for (source_id, diagnostic) in input.canonical_iter() {
        let id = ResolvedTypedDiagnosticId::new(output.len());
        output.push(ResolvedTypedDiagnostic {
            id,
            source_range: diagnostic.source_range,
            owner: diagnostic.owner.clone(),
            source: ResolvedTypedDiagnosticSource::Type(source_id),
            severity: type_severity(diagnostic.severity),
            message_key: diagnostic.message_key.clone(),
        });
        map.insert(source_id, id);
    }
    map
}

fn copy_overload_diagnostics(
    output: &mut ResolvedTypedDiagnosticTable,
    input: &OverloadDiagnosticTable,
    source: fn(OverloadDiagnosticId) -> ResolvedTypedDiagnosticSource,
    collection: &OverloadCollectionOutput,
    candidates: Option<&crate::overload_resolution::OverloadCandidateTable>,
    fallback_range: SourceRange,
    include: fn(&OverloadDiagnostic) -> bool,
) -> BTreeMap<OverloadDiagnosticId, ResolvedTypedDiagnosticId> {
    let mut map = BTreeMap::new();
    for (source_id, diagnostic) in input.canonical_iter() {
        if !include(diagnostic) {
            continue;
        }
        let id = ResolvedTypedDiagnosticId::new(output.len());
        output.push(ResolvedTypedDiagnostic {
            id,
            source_range: overload_diagnostic_range(
                diagnostic,
                collection,
                candidates,
                fallback_range,
            ),
            owner: overload_diagnostic_owner(diagnostic, collection, candidates),
            source: source(source_id),
            severity: overload_severity(diagnostic.severity),
            message_key: diagnostic.message_key.as_str().to_owned(),
        });
        map.insert(source_id, id);
    }
    map
}

fn copy_candidate_summaries(
    input: &crate::overload_resolution::OverloadCandidateTable,
    diagnostics: &BTreeMap<OverloadDiagnosticId, ResolvedTypedDiagnosticId>,
) -> OverloadCandidateSummaryTable {
    let mut output = OverloadCandidateSummaryTable::new();
    for (_, candidate) in input.iter() {
        output.push(OverloadCandidateSummary {
            candidate: candidate.id,
            site: candidate.site,
            symbol: candidate.symbol.clone(),
            ordinary_root: candidate.ordinary_root.clone(),
            declaration_kind: candidate.declaration_kind.clone(),
            parameters: candidate.parameters.clone(),
            result: candidate.result,
            origin: candidate.origin.clone(),
            template: candidate.template.clone(),
            coherence: candidate.coherence,
            provenance: candidate.provenance.clone(),
            status: candidate.status,
            diagnostics: map_diagnostics(&candidate.diagnostics, diagnostics),
        });
    }
    output
}

fn copy_template_expansions(
    input: &crate::overload_resolution::TemplateExpansionTable,
    diagnostics: &BTreeMap<OverloadDiagnosticId, ResolvedTypedDiagnosticId>,
) -> TemplateExpansionSummaryTable {
    let mut output = TemplateExpansionSummaryTable::new();
    for (_, expansion) in input.iter() {
        output.push(TemplateExpansionSummary {
            id: expansion.id,
            source_candidate: expansion.source_candidate,
            site: expansion.site,
            template: expansion.template.clone(),
            instantiation_key: expansion.instantiation_key.clone(),
            substitutions: expansion.substitutions.clone(),
            instantiated_candidate: expansion.instantiated_candidate,
            status: expansion.status.clone(),
            diagnostics: map_diagnostics(&expansion.diagnostics, diagnostics),
        });
    }
    output
}

fn copy_viability_decisions(
    input: &CandidateViabilityOutput,
    diagnostics: &BTreeMap<OverloadDiagnosticId, ResolvedTypedDiagnosticId>,
) -> CandidateViabilitySummaryTable {
    let mut output = CandidateViabilitySummaryTable::new();
    for (_, decision) in input.decisions().iter() {
        output.push(CandidateViabilitySummary {
            id: decision.id,
            source_candidate: decision.source_candidate,
            site: decision.site,
            output_candidate: decision.output_candidate,
            status: decision.status.clone(),
            diagnostics: map_diagnostics(&decision.diagnostics, diagnostics),
        });
    }
    output
}

fn copy_specificity_graphs(
    input: &SpecificityGraphOutput,
    diagnostics: &BTreeMap<OverloadDiagnosticId, ResolvedTypedDiagnosticId>,
) -> ResolvedSpecificityGraphTable {
    let mut output = ResolvedSpecificityGraphTable::new();
    for (_, graph) in input.graphs().iter() {
        let comparisons = graph
            .comparisons
            .iter()
            .map(|comparison| ResolvedSpecificityComparison {
                id: comparison.id,
                left: comparison.left,
                right: comparison.right,
                status: comparison.status.clone(),
                reasons: comparison.reasons.clone(),
                diagnostics: map_diagnostics(&comparison.diagnostics, diagnostics),
            })
            .collect();
        output.push(ResolvedSpecificityGraph {
            graph: graph.id,
            site: graph.site,
            nodes: graph.nodes.clone(),
            comparisons,
            edges: graph.edges.clone(),
            diagnostics: map_diagnostics(&graph.diagnostics, diagnostics),
        });
    }
    output
}

fn build_overload_projection(
    inputs: &ResolvedTypedAstInputs<'_>,
    specificity_diagnostics: &BTreeMap<OverloadDiagnosticId, ResolvedTypedDiagnosticId>,
    selection_diagnostics: &BTreeMap<OverloadDiagnosticId, ResolvedTypedDiagnosticId>,
) -> Result<OverloadProjection, ResolvedTypedAstError> {
    let mut records = OverloadResolutionTable::new();
    let mut insertions = CoercionInsertionTable::new();
    let mut by_typed_site = BTreeMap::new();
    let mut inserted_by_view = BTreeMap::new();
    let mut inserted_by_typed_site = BTreeMap::<TypedSiteRef, Vec<CoercionInsertionId>>::new();

    for (_, result) in inputs.overload_selection.results().canonical_iter() {
        let site = inputs
            .overload_collection
            .sites()
            .get(result.site)
            .ok_or(ResolvedTypedAstError::InvalidOverloadSite { site: result.site })?;
        let graph = inputs.specificity.graphs().get(result.graph).ok_or(
            ResolvedTypedAstError::InvalidSpecificityGraph {
                graph: result.graph,
            },
        )?;
        let mut diagnostics = map_diagnostics(&graph.diagnostics, specificity_diagnostics);
        diagnostics.extend(map_diagnostics(&result.diagnostics, selection_diagnostics));
        diagnostics.sort();
        diagnostics.dedup();

        let mut inserted_views = Vec::new();
        if let OverloadResultStatus::Resolved {
            inserted_views: view_ids,
            ..
        } = &result.status
        {
            for view_id in view_ids {
                let insertion_id = project_inserted_view(
                    *view_id,
                    inputs,
                    &mut insertions,
                    &mut inserted_by_view,
                    &mut inserted_by_typed_site,
                )?;
                inserted_views.push(insertion_id);
            }
        }

        let status = match &result.status {
            OverloadResultStatus::Resolved {
                root,
                refinements,
                exposed_result,
                ..
            } => OverloadResolutionStatus::Resolved {
                root: *root,
                active_refinements: refinements.clone(),
                exposed_result: exposed_result.clone(),
                inserted_views,
            },
            OverloadResultStatus::NoMatch { rejected } => OverloadResolutionStatus::NoMatch {
                rejected: rejected.clone(),
            },
            OverloadResultStatus::Ambiguous { candidates } => OverloadResolutionStatus::Ambiguous {
                candidates: candidates.clone(),
            },
            OverloadResultStatus::IncompatibleRefinementJoin {
                root,
                refinements,
                reason,
            } => OverloadResolutionStatus::IncompatibleRefinementJoin {
                root: *root,
                refinements: refinements.clone(),
                reason: reason.clone(),
            },
            OverloadResultStatus::Blocked { reason } => OverloadResolutionStatus::Blocked {
                reason: reason.clone(),
            },
        };

        let id = OverloadResolutionId::new(records.len());
        let candidates = graph
            .nodes
            .iter()
            .map(|node| node.candidate)
            .collect::<Vec<_>>();
        records.push(OverloadResolutionRecord {
            id,
            site: result.site,
            typed_site: site.owner.clone(),
            source_range: site.source_range,
            status,
            candidates,
            specificity_graph: Some(result.graph),
            diagnostics,
        });
        by_typed_site.insert(site.owner.clone(), id);
    }

    Ok(OverloadProjection {
        records,
        insertions,
        by_typed_site,
        inserted_by_typed_site,
    })
}

fn project_inserted_view(
    view_id: InsertedViewId,
    inputs: &ResolvedTypedAstInputs<'_>,
    insertions: &mut CoercionInsertionTable,
    inserted_by_view: &mut BTreeMap<InsertedViewId, CoercionInsertionId>,
    inserted_by_typed_site: &mut BTreeMap<TypedSiteRef, Vec<CoercionInsertionId>>,
) -> Result<CoercionInsertionId, ResolvedTypedAstError> {
    if let Some(id) = inserted_by_view.get(&view_id) {
        return Ok(*id);
    }
    let view = inputs
        .overload_selection
        .inserted_views()
        .get(view_id)
        .ok_or(ResolvedTypedAstError::InvalidInsertedView { view: view_id })?;
    if inputs
        .specificity
        .candidates()
        .get(view.selected_candidate)
        .is_none()
    {
        return Err(ResolvedTypedAstError::InvalidOverloadCandidate {
            namespace: CandidateSummaryNamespace::Viable,
            candidate: view.selected_candidate,
        });
    }
    let source = match view.kind {
        InsertedViewKind::Widening => CoercionInsertionSource::InsertedWidening,
        InsertedViewKind::SourceQua => CoercionInsertionSource::SourceQua,
        InsertedViewKind::Narrowing => {
            return Err(ResolvedTypedAstError::UnsupportedInsertedViewKind {
                view: view_id,
                kind: view.kind,
            });
        }
    };
    let id = CoercionInsertionId::new(insertions.len());
    let source_range = source_range_for_site(inputs.typed_ast, &view.argument)?;
    insertions.push(CoercionInsertion {
        id,
        typed_site: view.argument.clone(),
        source_range,
        target: view.target,
        selected_candidate: Some(view.selected_candidate),
        source,
        reason: view.reason.clone(),
        evidence_facts: view.evidence_facts.clone(),
        path: view.path.clone(),
    });
    inserted_by_view.insert(view_id, id);
    inserted_by_typed_site
        .entry(view.argument.clone())
        .or_default()
        .push(id);
    Ok(id)
}

fn build_expression_metadata(
    inputs: &ResolvedTypedAstInputs<'_>,
    overloads: &OverloadProjection,
    viable_candidates: &OverloadCandidateSummaryTable,
    diagnostics: &ResolvedDiagnosticMaps,
) -> Result<ExpressionMetadataTable, ResolvedTypedAstError> {
    let mut table = ExpressionMetadataTable::new();
    let mut expression_inputs = inputs.expressions.iter().collect::<Vec<_>>();
    expression_inputs.sort_by_key(|input| {
        (
            input.expr.as_str().to_owned(),
            site_ref_order_key(&input.typed_site),
        )
    });
    for input in expression_inputs {
        validate_site(inputs.typed_ast, &input.typed_site)?;
        let source_range = source_range_for_site(inputs.typed_ast, &input.typed_site)?;
        let local_context = expression_context(inputs.typed_ast, input)?;
        let visible_facts =
            visible_facts_for_site(inputs.typed_ast, &input.typed_site, local_context);
        for fact in &input.cluster_facts {
            if inputs.cluster_facts.get(*fact).is_none() {
                return Err(ResolvedTypedAstError::InvalidClusterFact { fact: *fact });
            }
        }
        let overload = overloads.by_typed_site.get(&input.typed_site).copied();
        let inserted_views = overloads
            .inserted_by_typed_site
            .get(&input.typed_site)
            .cloned()
            .unwrap_or_default();
        let final_type =
            final_type_for_site(inputs, overloads, viable_candidates, &input.typed_site);
        let mut metadata_diagnostics = diagnostics_for_site(inputs, diagnostics, &input.typed_site);
        if let Some(overload) = overload.and_then(|id| overloads.records.get(id)) {
            metadata_diagnostics.extend(overload.diagnostics.iter().copied());
        }
        metadata_diagnostics.sort();
        metadata_diagnostics.dedup();
        table.insert(ExpressionMetadataDraft {
            expr: input.expr.clone(),
            typed_site: input.typed_site.clone(),
            source_range,
            final_type,
            visible_facts,
            cluster_facts: sorted_cluster_facts(&input.cluster_facts, inputs.cluster_facts),
            overload,
            inserted_views,
            local_context,
            diagnostics: metadata_diagnostics,
        })?;
    }
    Ok(table)
}

fn build_resolved_nodes(
    inputs: &ResolvedTypedAstInputs<'_>,
    metadata: &ExpressionMetadataTable,
    overloads: &OverloadProjection,
    viable_candidates: &OverloadCandidateSummaryTable,
    type_diagnostics: &BTreeMap<TypeDiagnosticId, ResolvedTypedDiagnosticId>,
) -> Result<ResolvedTypedArena, ResolvedTypedAstError> {
    let mut hints = BTreeMap::new();
    for hint in &inputs.node_hints {
        validate_typed_node(inputs.typed_ast, hint.typed_node)?;
        if hints.insert(hint.typed_node, hint.kind.clone()).is_some() {
            return Err(ResolvedTypedAstError::DuplicateNodeHint {
                typed_node: hint.typed_node,
            });
        }
    }

    let mut nodes = Vec::new();
    for (typed_id, typed_node) in inputs.typed_ast.nodes().iter() {
        let id = ResolvedTypedNodeId::new(typed_id.index());
        let children = typed_node
            .children
            .iter()
            .map(|child| {
                validate_typed_node(inputs.typed_ast, *child)?;
                Ok(ResolvedTypedNodeId::new(child.index()))
            })
            .collect::<Result<Vec<_>, ResolvedTypedAstError>>()?;
        let site = TypedSiteRef::Node(typed_id);
        let metadata_id = metadata.id_by_site(&site);
        let overload_id = overloads.by_typed_site.get(&site).copied();
        let failed_overload_id = overload_id.filter(|id| {
            overloads
                .records
                .get(*id)
                .is_some_and(overload_record_is_failed)
        });
        let final_type = final_type_for_site(inputs, overloads, viable_candidates, &site);
        let diagnostics = typed_node
            .links
            .diagnostics
            .iter()
            .filter_map(|diagnostic| type_diagnostics.get(diagnostic).copied())
            .collect::<Vec<_>>();
        let kind = resolved_node_kind(
            typed_id,
            typed_node,
            failed_overload_id,
            hints.get(&typed_id),
        );
        nodes.push(ResolvedTypedNode {
            id,
            typed_node: typed_id,
            source_range: anchor_to_range(&typed_node.anchor, inputs.typed_ast.source_id()),
            children,
            kind,
            final_type,
            metadata: metadata_id,
            diagnostics,
            recovery: resolved_recovery(typed_node.recovery),
        });
    }

    Ok(ResolvedTypedArena {
        root: inputs
            .typed_ast
            .nodes()
            .root()
            .map(|root| ResolvedTypedNodeId::new(root.index())),
        nodes,
    })
}

fn resolved_node_kind(
    typed_id: TypedNodeId,
    typed_node: &crate::typed_ast::TypedNode,
    overload_id: Option<OverloadResolutionId>,
    hint: Option<&ResolvedNodeKindHintKind>,
) -> ResolvedTypedNodeKind {
    if let Some(result) = overload_id {
        return ResolvedTypedNodeKind::FailedOverload { result };
    }
    match (typed_node.recovery, typed_node.typing) {
        (NodeRecoveryState::Degraded, _) => ResolvedTypedNodeKind::Degraded {
            reason: ResolvedNodeRecoveryReason::TypedRecovery(NodeRecoveryState::Degraded),
        },
        (_, TypingState::Error | TypingState::Skipped | TypingState::Unknown) => {
            ResolvedTypedNodeKind::Degraded {
                reason: ResolvedNodeRecoveryReason::TypingState(typed_node.typing),
            }
        }
        _ => match hint {
            Some(ResolvedNodeKindHintKind::SourcePreserved { role }) => {
                ResolvedTypedNodeKind::SourcePreserved { role: role.clone() }
            }
            Some(ResolvedNodeKindHintKind::ResolvedUse { symbol }) => {
                ResolvedTypedNodeKind::ResolvedUse {
                    symbol: symbol.clone(),
                }
            }
            Some(ResolvedNodeKindHintKind::Degraded { reason }) => {
                ResolvedTypedNodeKind::Degraded { reason: *reason }
            }
            None => ResolvedTypedNodeKind::SourcePreserved {
                role: SourceNodeRole::new(format!(
                    "{}#{}",
                    typed_node.kind.as_str(),
                    typed_id.index()
                )),
            },
        },
    }
}

fn overload_record_is_failed(record: &OverloadResolutionRecord) -> bool {
    !matches!(record.status, OverloadResolutionStatus::Resolved { .. })
}

fn resolved_recovery(recovery: NodeRecoveryState) -> ResolvedNodeRecovery {
    match recovery {
        NodeRecoveryState::Normal => ResolvedNodeRecovery::Normal,
        NodeRecoveryState::Recovered => ResolvedNodeRecovery::Recovered,
        NodeRecoveryState::Degraded => ResolvedNodeRecovery::Degraded,
    }
}

fn final_type_for_site(
    inputs: &ResolvedTypedAstInputs<'_>,
    overloads: &OverloadProjection,
    viable_candidates: &OverloadCandidateSummaryTable,
    site: &TypedSiteRef,
) -> Option<NormalizedTypeId> {
    if let Some(overload_id) = overloads.by_typed_site.get(site)
        && let Some(record) = overloads.records.get(*overload_id)
        && let OverloadResolutionStatus::Resolved {
            root,
            exposed_result,
            ..
        } = &record.status
    {
        if let Some(result) = exposed_result.as_ref().and_then(|payload| payload.result) {
            return Some(result);
        }
        if let Some(candidate) = viable_candidates.get(*root) {
            return candidate.result;
        }
    }
    type_entry_final_type(inputs.typed_ast, site)
}

fn type_entry_final_type(typed_ast: &TypedAst, site: &TypedSiteRef) -> Option<NormalizedTypeId> {
    typed_ast
        .types()
        .canonical_iter()
        .filter(|(_, entry)| &entry.owner == site && entry.status.is_available_for_handoff())
        .find_map(|(_, entry)| match entry.actual {
            TypeEntryActual::Known(id) => Some(id),
            TypeEntryActual::CandidateSet(_) | TypeEntryActual::Absent => None,
        })
}

fn expression_context(
    typed_ast: &TypedAst,
    input: &ExpressionMetadataInput,
) -> Result<Option<LocalTypeContextId>, ResolvedTypedAstError> {
    let context = input.local_context.or_else(|| {
        typed_ast
            .nodes()
            .node(input.typed_site.node())
            .and_then(|node| node.links.context)
    });
    if let Some(context) = context
        && typed_ast.contexts().get(context).is_none()
    {
        return Err(ResolvedTypedAstError::InvalidLocalContext { context });
    }
    Ok(context)
}

fn visible_facts_for_site(
    typed_ast: &TypedAst,
    site: &TypedSiteRef,
    context: Option<LocalTypeContextId>,
) -> Vec<TypeFactId> {
    let mut facts = BTreeSet::new();
    if let Some(context) = context.and_then(|id| typed_ast.contexts().get(id)) {
        facts.extend(context.visible_facts.iter().copied());
    }
    if let Some(node) = typed_ast.nodes().node(site.node()) {
        facts.extend(node.links.facts.iter().copied());
    }
    facts.into_iter().collect()
}

fn diagnostics_for_site(
    inputs: &ResolvedTypedAstInputs<'_>,
    maps: &ResolvedDiagnosticMaps,
    site: &TypedSiteRef,
) -> Vec<ResolvedTypedDiagnosticId> {
    inputs
        .typed_ast
        .nodes()
        .node(site.node())
        .map(|node| {
            node.links
                .diagnostics
                .iter()
                .filter_map(|diagnostic| maps.type_diagnostics.get(diagnostic).copied())
                .collect()
        })
        .unwrap_or_default()
}

fn sorted_cluster_facts(facts: &[ClusterFactId], table: &ClusterFactTable) -> Vec<ClusterFactId> {
    let mut facts = facts.to_vec();
    facts.sort_by_key(|id| {
        table.get(*id).map_or_else(
            || (usize::MAX, String::new()),
            |fact| (id.index(), format!("{:?}", fact.provenance())),
        )
    });
    facts.dedup();
    facts
}

fn validate_site(typed_ast: &TypedAst, site: &TypedSiteRef) -> Result<(), ResolvedTypedAstError> {
    validate_typed_node(typed_ast, site.node())
        .map_err(|_| ResolvedTypedAstError::InvalidTypedSite { site: site.clone() })
}

fn validate_typed_node(
    typed_ast: &TypedAst,
    typed_node: TypedNodeId,
) -> Result<(), ResolvedTypedAstError> {
    typed_ast
        .nodes()
        .node(typed_node)
        .map(|_| ())
        .ok_or(ResolvedTypedAstError::InvalidTypedNode { typed_node })
}

fn source_range_for_site(
    typed_ast: &TypedAst,
    site: &TypedSiteRef,
) -> Result<SourceRange, ResolvedTypedAstError> {
    typed_ast
        .nodes()
        .node(site.node())
        .map(|node| anchor_to_range(&node.anchor, typed_ast.source_id()))
        .ok_or_else(|| ResolvedTypedAstError::InvalidTypedSite { site: site.clone() })
}

fn root_range(typed_ast: &TypedAst) -> SourceRange {
    typed_ast
        .nodes()
        .root()
        .and_then(|root| typed_ast.nodes().node(root))
        .or_else(|| typed_ast.nodes().iter().next().map(|(_, node)| node))
        .map(|node| anchor_to_range(&node.anchor, typed_ast.source_id()))
        .unwrap_or(SourceRange {
            source_id: typed_ast.source_id(),
            start: 0,
            end: 0,
        })
}

fn anchor_to_range(anchor: &SourceAnchor, fallback_source_id: SourceId) -> SourceRange {
    match anchor {
        SourceAnchor::Range(range) => *range,
        SourceAnchor::Point { source_id, offset } => SourceRange {
            source_id: *source_id,
            start: *offset,
            end: *offset,
        },
        SourceAnchor::Generated(origin) => {
            generated_anchor_to_range(origin.anchor(), fallback_source_id)
        }
        _ => SourceRange {
            source_id: fallback_source_id,
            start: 0,
            end: 0,
        },
    }
}

fn generated_anchor_to_range(
    anchor: GeneratedSpanAnchor,
    fallback_source_id: SourceId,
) -> SourceRange {
    match anchor {
        GeneratedSpanAnchor::Range(range) => range,
        GeneratedSpanAnchor::Point { source_id, offset } => SourceRange {
            source_id,
            start: offset,
            end: offset,
        },
        _ => SourceRange {
            source_id: fallback_source_id,
            start: 0,
            end: 0,
        },
    }
}

fn overload_diagnostic_range(
    diagnostic: &OverloadDiagnostic,
    collection: &OverloadCollectionOutput,
    candidates: Option<&crate::overload_resolution::OverloadCandidateTable>,
    fallback: SourceRange,
) -> SourceRange {
    if let Some(crate::overload_resolution::OverloadDiagnosticProvenance::SiteInput {
        source_range,
        ..
    }) = &diagnostic.provenance
    {
        return *source_range;
    }
    if let Some(site) = diagnostic.site.and_then(|id| collection.sites().get(id)) {
        return site.source_range;
    }
    if let Some(candidate) = diagnostic
        .candidate
        .and_then(|id| candidates.and_then(|table| table.get(id)))
        && let Some(site) = collection.sites().get(candidate.site)
    {
        return site.source_range;
    }
    fallback
}

fn overload_diagnostic_owner(
    diagnostic: &OverloadDiagnostic,
    collection: &OverloadCollectionOutput,
    candidates: Option<&crate::overload_resolution::OverloadCandidateTable>,
) -> Option<TypedSiteRef> {
    if let Some(crate::overload_resolution::OverloadDiagnosticProvenance::SiteInput {
        owner, ..
    }) = &diagnostic.provenance
    {
        return Some(owner.clone());
    }
    if let Some(site) = diagnostic.site.and_then(|id| collection.sites().get(id)) {
        return Some(site.owner.clone());
    }
    diagnostic
        .candidate
        .and_then(|id| candidates.and_then(|table| table.get(id)))
        .and_then(|candidate| collection.sites().get(candidate.site))
        .map(|site| site.owner.clone())
}

fn map_diagnostics(
    ids: &[OverloadDiagnosticId],
    map: &BTreeMap<OverloadDiagnosticId, ResolvedTypedDiagnosticId>,
) -> Vec<ResolvedTypedDiagnosticId> {
    let mut diagnostics = ids
        .iter()
        .filter_map(|id| map.get(id).copied())
        .collect::<Vec<_>>();
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn type_severity(severity: TypeDiagnosticSeverity) -> ResolvedTypedDiagnosticSeverity {
    match severity {
        TypeDiagnosticSeverity::Error => ResolvedTypedDiagnosticSeverity::Error,
        TypeDiagnosticSeverity::Warning => ResolvedTypedDiagnosticSeverity::Warning,
        TypeDiagnosticSeverity::Note => ResolvedTypedDiagnosticSeverity::Note,
    }
}

fn overload_severity(severity: OverloadDiagnosticSeverity) -> ResolvedTypedDiagnosticSeverity {
    match severity {
        OverloadDiagnosticSeverity::Error => ResolvedTypedDiagnosticSeverity::Error,
        OverloadDiagnosticSeverity::Warning => ResolvedTypedDiagnosticSeverity::Warning,
        OverloadDiagnosticSeverity::Note => ResolvedTypedDiagnosticSeverity::Note,
    }
}

type SiteRefOrderKey = (usize, u8, String);
type CandidateSummaryOrderKey = (usize, String, String);
type TemplateExpansionSummaryOrderKey = (usize, usize, String);
type ViabilitySummaryOrderKey = (usize, usize);
type SpecificityGraphSummaryOrderKey = (usize, usize);
type OverloadResolutionOrderKey = (String, usize, usize, usize);
type CoercionInsertionOrderKey = (SiteRefOrderKey, usize, u8, String);
type DiagnosticOrderKey = (String, usize, usize, String, usize);

fn site_ref_order_key(site: &TypedSiteRef) -> SiteRefOrderKey {
    match site {
        TypedSiteRef::Node(node) => (node.index(), 0, String::new()),
        TypedSiteRef::Role { node, role } => (node.index(), 1, role.as_str().to_owned()),
    }
}

fn candidate_summary_order_key(candidate: &OverloadCandidateSummary) -> CandidateSummaryOrderKey {
    (
        candidate.site.index(),
        candidate.provenance.stable_key.as_str().to_owned(),
        candidate.symbol.fqn().as_str().to_owned(),
    )
}

fn template_expansion_summary_order_key(
    expansion: &TemplateExpansionSummary,
) -> TemplateExpansionSummaryOrderKey {
    (
        expansion.site.index(),
        expansion.id.index(),
        expansion.instantiation_key.as_str().to_owned(),
    )
}

fn viability_summary_order_key(summary: &CandidateViabilitySummary) -> ViabilitySummaryOrderKey {
    (summary.site.index(), summary.id.index())
}

fn specificity_graph_summary_order_key(
    graph: &ResolvedSpecificityGraph,
) -> SpecificityGraphSummaryOrderKey {
    (graph.site.index(), graph.graph.index())
}

fn overload_resolution_order_key(record: &OverloadResolutionRecord) -> OverloadResolutionOrderKey {
    (
        source_order_key(record.source_range),
        record.source_range.start,
        record.source_range.end,
        record.site.index(),
    )
}

fn coercion_insertion_order_key(insertion: &CoercionInsertion) -> CoercionInsertionOrderKey {
    (
        site_ref_order_key(&insertion.typed_site),
        insertion.target.index(),
        match insertion.source {
            CoercionInsertionSource::SourceQua => 0,
            CoercionInsertionSource::InsertedWidening => 1,
        },
        insertion.reason.as_str().to_owned(),
    )
}

fn diagnostic_order_key(diagnostic: &ResolvedTypedDiagnostic) -> DiagnosticOrderKey {
    (
        source_order_key(diagnostic.source_range),
        diagnostic.source_range.start,
        diagnostic.source_range.end,
        diagnostic.message_key.clone(),
        diagnostic.id.index(),
    )
}

fn source_order_key(range: SourceRange) -> String {
    format!("{:?}", range.source_id)
}

fn write_resolved_nodes(output: &mut String, arena: &ResolvedTypedArena) {
    output.push_str("nodes:\n");
    for (id, node) in arena.iter() {
        let _ = write!(
            output,
            "  node#{} typed=node#{} range=",
            id.index(),
            node.typed_node.index()
        );
        write_range(output, node.source_range);
        output.push_str(" kind=");
        write_node_kind(output, &node.kind);
        output.push_str(" final_type=");
        write_optional_type(output, node.final_type);
        output.push_str(" metadata=");
        write_optional_metadata_id(output, node.metadata);
        output.push_str(" children=");
        write_resolved_node_ids(output, &node.children);
        output.push('\n');
    }
}

fn write_expression_metadata(output: &mut String, table: &ExpressionMetadataTable) {
    output.push_str("expression-metadata:\n");
    for (id, metadata) in table.canonical_iter() {
        let _ = write!(
            output,
            "  metadata#{} expr=\"{}\" site=",
            id.index(),
            escaped_display(metadata.expr.as_str())
        );
        write_site_ref(output, &metadata.typed_site);
        output.push_str(" final_type=");
        write_optional_type(output, metadata.final_type);
        output.push_str(" overload=");
        write_optional_overload_id(output, metadata.overload);
        output.push_str(" inserted_views=");
        write_coercion_ids(output, &metadata.inserted_views);
        output.push('\n');
    }
}

fn write_candidate_summaries(
    output: &mut String,
    label: &str,
    table: &OverloadCandidateSummaryTable,
) {
    let _ = writeln!(output, "{label}:");
    for (id, candidate) in table.canonical_iter() {
        let _ = write!(
            output,
            "  candidate#{} site=site#{} symbol=\"{}\" root=\"{}\" status={:?} result=",
            id.index(),
            candidate.site.index(),
            escaped_display(candidate.symbol.fqn().as_str()),
            escaped_display(candidate.ordinary_root.fqn().as_str()),
            candidate.status
        );
        write_optional_type(output, candidate.result);
        output.push('\n');
    }
}

fn write_template_expansions(output: &mut String, table: &TemplateExpansionSummaryTable) {
    output.push_str("template-expansions:\n");
    for (id, expansion) in table.canonical_iter() {
        let _ = write!(
            output,
            "  expansion#{} source=candidate#{} instantiated=",
            id.index(),
            expansion.source_candidate.index()
        );
        write_optional_candidate_id(output, expansion.instantiated_candidate);
        let _ = writeln!(output, " status={:?}", expansion.status);
    }
}

fn write_viability_summaries(output: &mut String, table: &CandidateViabilitySummaryTable) {
    output.push_str("viability-decisions:\n");
    for (id, summary) in table.canonical_iter() {
        let _ = write!(
            output,
            "  viability#{} source=candidate#{} output=",
            id.index(),
            summary.source_candidate.index()
        );
        write_optional_candidate_id(output, summary.output_candidate);
        let _ = writeln!(output, " status={:?}", summary.status);
    }
}

fn write_specificity_graphs(output: &mut String, table: &ResolvedSpecificityGraphTable) {
    output.push_str("specificity-graphs:\n");
    for (id, graph) in table.canonical_iter() {
        let _ = writeln!(
            output,
            "  graph#{} site=site#{} nodes={} comparisons={} edges={}",
            id.index(),
            graph.site.index(),
            graph.nodes.len(),
            graph.comparisons.len(),
            graph.edges.len()
        );
    }
}

fn write_overload_records(output: &mut String, table: &OverloadResolutionTable) {
    output.push_str("overload-records:\n");
    for (id, record) in table.canonical_iter() {
        let _ = write!(
            output,
            "  overload#{} site=site#{} status=",
            id.index(),
            record.site.index()
        );
        write_overload_status(output, &record.status);
        output.push_str(" candidates=");
        write_candidate_ids(output, &record.candidates);
        output.push('\n');
    }
}

fn write_coercion_insertions(output: &mut String, table: &CoercionInsertionTable) {
    output.push_str("inserted-coercions:\n");
    for (id, insertion) in table.canonical_iter() {
        let _ = write!(output, "  coercion#{} site=", id.index());
        write_site_ref(output, &insertion.typed_site);
        let _ = write!(
            output,
            " target=type#{} source={:?} reason=\"{}\" path=",
            insertion.target.index(),
            insertion.source,
            escaped_display(insertion.reason.as_str())
        );
        write_optional_path(output, insertion.path.as_ref());
        output.push('\n');
    }
}

fn write_cluster_facts(output: &mut String, table: &ClusterFactTable) {
    output.push_str("cluster-facts:\n");
    for (id, fact) in table.canonical_iter() {
        let _ = writeln!(
            output,
            "  cluster_fact#{} fingerprint=\"{}\" provenance={:?}",
            id.index(),
            escaped_display(fact.fingerprint().as_str()),
            fact.provenance()
        );
    }
}

fn write_resolved_diagnostics(output: &mut String, table: &ResolvedTypedDiagnosticTable) {
    output.push_str("diagnostics:\n");
    for (id, diagnostic) in table.canonical_iter() {
        let _ = write!(
            output,
            "  diagnostic#{} severity={:?} key=\"{}\" range=",
            id.index(),
            diagnostic.severity,
            escaped_display(&diagnostic.message_key)
        );
        write_range(output, diagnostic.source_range);
        output.push('\n');
    }
}

fn write_node_kind(output: &mut String, kind: &ResolvedTypedNodeKind) {
    match kind {
        ResolvedTypedNodeKind::SourcePreserved { role } => {
            let _ = write!(
                output,
                "source_preserved(\"{}\")",
                escaped_display(role.as_str())
            );
        }
        ResolvedTypedNodeKind::ResolvedUse { symbol } => {
            let _ = write!(
                output,
                "resolved_use(\"{}\")",
                escaped_display(symbol.fqn().as_str())
            );
        }
        ResolvedTypedNodeKind::FailedOverload { result } => {
            let _ = write!(output, "failed_overload(overload#{})", result.index());
        }
        ResolvedTypedNodeKind::Degraded { reason } => {
            let _ = write!(output, "degraded({reason:?})");
        }
    }
}

fn write_overload_status(output: &mut String, status: &OverloadResolutionStatus) {
    match status {
        OverloadResolutionStatus::Resolved {
            root,
            active_refinements,
            inserted_views,
            ..
        } => {
            let _ = write!(output, "resolved(root=candidate#{}", root.index());
            output.push_str(", refinements=");
            write_candidate_ids(output, active_refinements);
            output.push_str(", views=");
            write_coercion_ids(output, inserted_views);
            output.push(')');
        }
        OverloadResolutionStatus::NoMatch { rejected } => {
            output.push_str("no_match(rejected=");
            write_candidate_ids(output, rejected);
            output.push(')');
        }
        OverloadResolutionStatus::Ambiguous { candidates } => {
            output.push_str("ambiguous(candidates=");
            write_candidate_ids(output, candidates);
            output.push(')');
        }
        OverloadResolutionStatus::IncompatibleRefinementJoin {
            root,
            refinements,
            reason,
        } => {
            let _ = write!(output, "incompatible_join(root=candidate#{}", root.index());
            output.push_str(", refinements=");
            write_candidate_ids(output, refinements);
            let _ = write!(output, ", reason={reason:?})");
        }
        OverloadResolutionStatus::Blocked { reason } => {
            let _ = write!(output, "blocked({reason:?})");
        }
    }
}

fn write_site_ref(output: &mut String, site: &TypedSiteRef) {
    match site {
        TypedSiteRef::Node(node) => {
            let _ = write!(output, "node#{}", node.index());
        }
        TypedSiteRef::Role { node, role } => {
            let _ = write!(
                output,
                "node#{}::{}",
                node.index(),
                escaped_display(role.as_str())
            );
        }
    }
}

fn write_optional_resolved_node_id(output: &mut String, id: Option<ResolvedTypedNodeId>) {
    match id {
        Some(id) => {
            let _ = write!(output, "resolved_node#{}", id.index());
        }
        None => output.push_str("<none>"),
    }
}

fn write_resolved_node_ids(output: &mut String, ids: &[ResolvedTypedNodeId]) {
    output.push('[');
    for (index, id) in ids.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "resolved_node#{}", id.index());
    }
    output.push(']');
}

fn write_optional_metadata_id(output: &mut String, id: Option<ExpressionMetadataId>) {
    match id {
        Some(id) => {
            let _ = write!(output, "metadata#{}", id.index());
        }
        None => output.push_str("<none>"),
    }
}

fn write_optional_overload_id(output: &mut String, id: Option<OverloadResolutionId>) {
    match id {
        Some(id) => {
            let _ = write!(output, "overload#{}", id.index());
        }
        None => output.push_str("<none>"),
    }
}

fn write_optional_type(output: &mut String, id: Option<NormalizedTypeId>) {
    match id {
        Some(id) => {
            let _ = write!(output, "type#{}", id.index());
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

fn write_coercion_ids(output: &mut String, ids: &[CoercionInsertionId]) {
    output.push('[');
    for (index, id) in ids.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "coercion#{}", id.index());
    }
    output.push(']');
}

fn write_optional_path(output: &mut String, path: Option<&QuaPathKey>) {
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

fn escaped_display(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn write_escaped(output: &mut String, value: &str) {
    output.push('"');
    output.push_str(&escaped_display(value));
    output.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cluster_trace::{
            ClusterAttributeFingerprint, ClusterFactDraft, ClusterFactFingerprint,
            ClusterFactProvenance, ClusterStepId, ClusterTypeFingerprint,
        },
        overload_resolution::{
            ArgumentViabilityEvidence, CandidateScope, CandidateViabilityInput,
            ExposedResultSource, InsertedViewInput, InsertedViewStatus, OverloadCandidateInput,
            OverloadNameKey, OverloadSiteInput, OverloadSiteKey, OverloadSiteKind,
            OverloadSiteRecovery, OverloadSiteResolutionInput, RefinementJoinPayload,
            RefinementJoinStatus, SourceQuaView, SpecificityComparisonInput,
            SpecificityComparisonStatus, TemplateArgument, TemplateConstraintEvidence,
            TemplateInstantiationKey, TemplateParameterKey, TemplateQuaStatus,
            UnsupportedOverloadRole,
        },
        typed_ast::{
            BuiltinRuleId, CoercionTable, ContextRecoveryState, FactProvenance, FactStatus,
            InitialObligationTable, LocalTypeContextDraft, LocalTypeContextTable,
            OpenCandidateSetId, Polarity, TypeContextLayer, TypeEntryDraft, TypeEntryId,
            TypeFactDraft, TypeFactTable, TypePredicateRef, TypeProvenance, TypeRole, TypeStatus,
            TypeTable, TypedArenaBuilder, TypedAstParts, TypedNode, TypedNodeLinks,
        },
    };
    use mizar_resolve::resolved_ast::{FullyQualifiedName, LocalSymbolId};
    use mizar_session::{
        BuildSnapshotId, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator,
    };

    #[test]
    fn assembly_preserves_source_shape_metadata_and_successful_overload_type() {
        let source = source_id(1);
        let typed = typed_ast_fixture(
            source,
            TypeEntryActual::CandidateSet(OpenCandidateSetId::new(0)),
        );
        let expr_site = TypedSiteRef::Node(TypedNodeId::new(0));
        let mut cluster_facts = ClusterFactTable::new();
        let cluster_fact = cluster_facts.insert(cluster_fact(source, 8));
        let mut refinement = candidate(
            "call",
            "root",
            "refinement",
            1,
            Some(NormalizedTypeId::new(6)),
        );
        refinement.declaration_kind = CandidateDeclarationKind::Redefinition;
        refinement.origin = CandidateOrigin::Redefinition {
            refined: symbol("root"),
        };
        refinement.coherence = Some(CoherenceStatus::Accepted);
        let collection = OverloadCollectionOutput::collect(
            [site("call", expr_site.clone(), source, 10)],
            [
                candidate("call", "root", "root", 0, Some(NormalizedTypeId::new(4))),
                refinement,
            ],
        );
        let expansion = TemplateExpansionOutput::expand(&collection);
        let expanded_root = candidate_id_by_symbol(expansion.candidates(), "root");
        let expanded_refinement = candidate_id_by_symbol(expansion.candidates(), "refinement");
        let viability = CandidateViabilityOutput::filter(
            &expansion,
            [
                viable_input(expanded_root),
                viable_input(expanded_refinement),
            ],
        );
        let viable_root = candidate_id_by_symbol(viability.candidates(), "root");
        let viable_refinement = candidate_id_by_symbol(viability.candidates(), "refinement");
        let graphs = SpecificityGraphOutput::build(
            &viability,
            [SpecificityComparisonInput {
                left: viable_root,
                right: viable_refinement,
                status: SpecificityComparisonStatus::Equivalent,
                reasons: vec![SpecificityReasonKey::new("same-root-refinement")],
            }],
        );
        let selected_site = viability
            .candidates()
            .get(viable_root)
            .expect("viable candidate")
            .site;
        let selection = OverloadSelectionOutput::resolve(
            &graphs,
            [OverloadSiteResolutionInput {
                site: selected_site,
                refinements: vec![viable_refinement],
                refinement_join: RefinementJoinPayload {
                    status: RefinementJoinStatus::Compatible,
                    exposed_result: Some(ExposedResultPayload {
                        result: Some(NormalizedTypeId::new(9)),
                        source: ExposedResultSource::SelectedRoot,
                        evidence: Vec::new(),
                    }),
                },
                inserted_views: vec![InsertedViewInput {
                    argument: expr_site.clone(),
                    target: NormalizedTypeId::new(2),
                    selected_candidate: viable_root,
                    kind: InsertedViewKind::Widening,
                    status: InsertedViewStatus::Accepted,
                    reason: InsertedViewReasonKey::new("argument-widening"),
                    evidence_facts: Vec::new(),
                    path: Some(QuaPathKey::new("A<B")),
                }],
            }],
        );

        let resolved = assemble(
            AssemblyRefs {
                typed_ast: &typed,
                cluster_facts: &cluster_facts,
                collection: &collection,
                expansion: &expansion,
                viability: &viability,
                graphs: &graphs,
                selection: &selection,
            },
            vec![ExpressionMetadataInput {
                expr: ExprId::new("expr.call"),
                typed_site: expr_site.clone(),
                local_context: None,
                cluster_facts: vec![cluster_fact],
            }],
        )
        .expect("assembly succeeds");

        let metadata = resolved
            .expr_metadata()
            .get_by_expr(&ExprId::new("expr.call"))
            .expect("metadata by expression id");
        assert_eq!(metadata.final_type, Some(NormalizedTypeId::new(9)));
        assert_eq!(metadata.cluster_facts, vec![cluster_fact]);
        assert_eq!(metadata.inserted_views.len(), 1);
        let record = resolved
            .resolved_overloads()
            .get(metadata.overload.expect("overload"))
            .expect("overload record");
        let OverloadResolutionStatus::Resolved {
            root,
            active_refinements,
            inserted_views,
            ..
        } = &record.status
        else {
            panic!("expected resolved overload, got {:?}", record.status);
        };
        assert_eq!(*root, viable_root);
        assert_eq!(active_refinements, &[viable_refinement]);
        assert_eq!(inserted_views, &metadata.inserted_views);
        assert!(matches!(
            resolved
                .nodes()
                .node(ResolvedTypedNodeId::new(0))
                .expect("expression node")
                .kind,
            ResolvedTypedNodeKind::SourcePreserved { .. }
        ));
        let coercion = resolved
            .inserted_coercions()
            .get(metadata.inserted_views[0])
            .expect("inserted coercion");
        assert_eq!(coercion.source, CoercionInsertionSource::InsertedWidening);
        assert_eq!(coercion.reason.as_str(), "argument-widening");
        assert_eq!(coercion.path.as_ref().map(QuaPathKey::as_str), Some("A<B"));
        assert_eq!(
            resolved
                .cluster_facts()
                .get(cluster_fact)
                .unwrap()
                .provenance(),
            &ClusterFactProvenance::Input
        );
        assert!(
            resolved
                .debug_text()
                .contains("resolved-typed-ast-debug-v1")
        );
    }

    #[test]
    fn failed_sites_do_not_insert_views_and_candidate_namespaces_are_distinct() {
        let source = source_id(2);
        let typed = typed_ast_fixture(
            source,
            TypeEntryActual::CandidateSet(OpenCandidateSetId::new(0)),
        );
        let expr_site = TypedSiteRef::Node(TypedNodeId::new(0));
        let cluster_facts = ClusterFactTable::new();
        let collection = OverloadCollectionOutput::collect(
            [site("call", expr_site.clone(), source, 10)],
            [
                candidate(
                    "call",
                    "a-rejected-root",
                    "a-rejected",
                    0,
                    Some(NormalizedTypeId::new(4)),
                ),
                candidate(
                    "call",
                    "z-accepted-root",
                    "z-accepted",
                    1,
                    Some(NormalizedTypeId::new(5)),
                ),
            ],
        );
        let expansion = TemplateExpansionOutput::expand(&collection);
        let rejected = candidate_id_by_symbol(expansion.candidates(), "a-rejected");
        let accepted = candidate_id_by_symbol(expansion.candidates(), "z-accepted");
        let viability = CandidateViabilityOutput::filter(
            &expansion,
            [CandidateViabilityInput {
                candidate: accepted,
                arguments: vec![ArgumentViabilityEvidence::Exact {
                    actual: NormalizedTypeId::new(1),
                }],
            }],
        );
        let graphs = SpecificityGraphOutput::build(&viability, []);
        let selection = OverloadSelectionOutput::resolve(&graphs, []);

        let resolved = assemble(
            AssemblyRefs {
                typed_ast: &typed,
                cluster_facts: &cluster_facts,
                collection: &collection,
                expansion: &expansion,
                viability: &viability,
                graphs: &graphs,
                selection: &selection,
            },
            vec![ExpressionMetadataInput {
                expr: ExprId::new("expr.failed"),
                typed_site: expr_site,
                local_context: None,
                cluster_facts: Vec::new(),
            }],
        )
        .expect("assembly succeeds");

        assert_eq!(resolved.expanded_candidates().len(), 2);
        assert_eq!(resolved.viable_candidates().len(), 1);
        let rejected_decision = resolved
            .viability_decisions()
            .canonical_iter()
            .find(|(_, decision)| decision.source_candidate == rejected)
            .map(|(_, decision)| decision)
            .expect("rejected decision");
        assert!(matches!(
            rejected_decision.status,
            CandidateViabilityStatus::Rejected { .. } | CandidateViabilityStatus::Blocked { .. }
        ));
        assert_eq!(rejected_decision.output_candidate, None);
        assert!(!rejected_decision.diagnostics.is_empty());
        let accepted_decision = resolved
            .viability_decisions()
            .canonical_iter()
            .find(|(_, decision)| decision.source_candidate == accepted)
            .map(|(_, decision)| decision)
            .expect("accepted decision");
        assert_eq!(
            accepted_decision.output_candidate,
            Some(OverloadCandidateId::new(0))
        );
        assert_eq!(accepted, OverloadCandidateId::new(1));
        assert!(resolved.inserted_coercions().is_empty());
        assert!(matches!(
            resolved
                .nodes()
                .node(ResolvedTypedNodeId::new(0))
                .expect("expression node")
                .kind,
            ResolvedTypedNodeKind::FailedOverload { .. }
        ));
    }

    #[test]
    fn template_expansion_summaries_preserve_instantiated_rejected_and_deferred_rows() {
        let source = source_id(3);
        let typed = typed_ast_fixture(source, TypeEntryActual::Known(NormalizedTypeId::new(1)));
        let expr_site = TypedSiteRef::Node(TypedNodeId::new(0));
        let cluster_facts = ClusterFactTable::new();
        let mut accepted_template = candidate(
            "call",
            "template-root-0",
            "template-accepted",
            0,
            Some(NormalizedTypeId::new(3)),
        );
        accepted_template.template = Some(TemplateCandidatePayload {
            template: symbol("template-symbol-0"),
            instantiation_key: TemplateInstantiationKey::new("S=accepted"),
            parameters: vec![TemplateParameterKey::new("S")],
            arguments: vec![TemplateArgument::SourceQua {
                source: NormalizedTypeId::new(1),
                target: NormalizedTypeId::new(3),
                path: QuaPathKey::new("accepted"),
                status: TemplateQuaStatus::AcceptedWidening,
            }],
            inferred_arguments: Vec::new(),
            constraints: Vec::<TemplateConstraintEvidence>::new(),
        });
        let mut rejected_template = candidate(
            "call",
            "template-root",
            "template-rejected",
            1,
            Some(NormalizedTypeId::new(4)),
        );
        rejected_template.template = Some(TemplateCandidatePayload {
            template: symbol("template-symbol"),
            instantiation_key: TemplateInstantiationKey::new("T=?"),
            parameters: vec![TemplateParameterKey::new("T")],
            arguments: vec![TemplateArgument::SourceQua {
                source: NormalizedTypeId::new(1),
                target: NormalizedTypeId::new(2),
                path: QuaPathKey::new("bad"),
                status: TemplateQuaStatus::RejectedNarrowing,
            }],
            inferred_arguments: Vec::new(),
            constraints: Vec::<TemplateConstraintEvidence>::new(),
        });
        let mut deferred_template = candidate(
            "call",
            "template-root-2",
            "template-deferred",
            2,
            Some(NormalizedTypeId::new(5)),
        );
        deferred_template.template = Some(TemplateCandidatePayload {
            template: symbol("template-symbol-2"),
            instantiation_key: TemplateInstantiationKey::new("U=?"),
            parameters: vec![TemplateParameterKey::new("U")],
            arguments: vec![TemplateArgument::SourceQua {
                source: NormalizedTypeId::new(1),
                target: NormalizedTypeId::new(3),
                path: QuaPathKey::new("deferred"),
                status: TemplateQuaStatus::DeferredExternalDependency,
            }],
            inferred_arguments: Vec::new(),
            constraints: Vec::new(),
        });
        let collection = OverloadCollectionOutput::collect(
            [site("call", expr_site.clone(), source, 10)],
            [rejected_template, deferred_template, accepted_template],
        );
        let expansion = TemplateExpansionOutput::expand(&collection);
        let viability = CandidateViabilityOutput::filter(&expansion, []);
        let graphs = SpecificityGraphOutput::build(&viability, []);
        let selection = OverloadSelectionOutput::resolve(&graphs, []);

        let resolved = assemble(
            AssemblyRefs {
                typed_ast: &typed,
                cluster_facts: &cluster_facts,
                collection: &collection,
                expansion: &expansion,
                viability: &viability,
                graphs: &graphs,
                selection: &selection,
            },
            vec![ExpressionMetadataInput {
                expr: ExprId::new("expr.template"),
                typed_site: expr_site,
                local_context: None,
                cluster_facts: Vec::new(),
            }],
        )
        .expect("assembly succeeds");

        assert_eq!(resolved.template_expansions().len(), 3);
        let statuses = resolved
            .template_expansions()
            .canonical_iter()
            .map(|(_, expansion)| &expansion.status)
            .collect::<Vec<_>>();
        assert!(
            statuses
                .iter()
                .any(|status| matches!(status, TemplateExpansionStatus::Instantiated))
        );
        assert!(
            statuses
                .iter()
                .any(|status| matches!(status, TemplateExpansionStatus::Rejected(_)))
        );
        assert!(
            statuses
                .iter()
                .any(|status| matches!(status, TemplateExpansionStatus::Deferred(_)))
        );
        let instantiated = resolved
            .template_expansions()
            .canonical_iter()
            .find(|(_, expansion)| {
                matches!(expansion.status, TemplateExpansionStatus::Instantiated)
            })
            .map(|(_, expansion)| expansion)
            .expect("instantiated template expansion");
        let instantiated_candidate = instantiated
            .instantiated_candidate
            .expect("instantiated candidate id");
        assert!(
            resolved
                .expanded_candidates()
                .get(instantiated_candidate)
                .is_some()
        );
        assert!(resolved.debug_text().contains("template-expansions:"));
    }

    #[test]
    fn expanded_candidate_summaries_keep_remapped_candidate_diagnostics() {
        let source = source_id(8);
        let typed = typed_ast_fixture(source, TypeEntryActual::Known(NormalizedTypeId::new(1)));
        let expr_site = TypedSiteRef::Node(TypedNodeId::new(0));
        let cluster_facts = ClusterFactTable::new();
        let mut unsupported = candidate(
            "call",
            "unsupported-root",
            "unsupported",
            0,
            Some(NormalizedTypeId::new(4)),
        );
        unsupported.declaration_kind =
            CandidateDeclarationKind::Unsupported(UnsupportedOverloadRole::TheoremApplication);
        let collection = OverloadCollectionOutput::collect(
            [site("call", expr_site.clone(), source, 10)],
            [unsupported],
        );
        let expansion = TemplateExpansionOutput::expand(&collection);
        let viability = CandidateViabilityOutput::filter(&expansion, []);
        let graphs = SpecificityGraphOutput::build(&viability, []);
        let selection = OverloadSelectionOutput::resolve(&graphs, []);

        let resolved = assemble(
            AssemblyRefs {
                typed_ast: &typed,
                cluster_facts: &cluster_facts,
                collection: &collection,
                expansion: &expansion,
                viability: &viability,
                graphs: &graphs,
                selection: &selection,
            },
            vec![expression("expr.remapped_diagnostic", expr_site)],
        )
        .expect("remapped diagnostic assembly succeeds");

        let (_, summary) = resolved
            .expanded_candidates()
            .canonical_iter()
            .find(|(_, candidate)| candidate.symbol.local().as_str() == "unsupported")
            .expect("expanded unsupported candidate");
        assert_eq!(summary.diagnostics.len(), 1);
        let diagnostic = resolved
            .diagnostics()
            .get(summary.diagnostics[0])
            .expect("resolved remapped diagnostic");
        assert!(matches!(
            diagnostic.source,
            ResolvedTypedDiagnosticSource::TemplateExpansion(_)
        ));
        assert_eq!(
            diagnostic.message_key,
            "overload.candidate.unsupported.theorem_application"
        );
    }

    #[test]
    fn deterministic_debug_text_canonicalizes_equivalent_input_orderings() {
        let forward = deterministic_order_fixture(false);
        let reversed = deterministic_order_fixture(true);

        assert_eq!(forward.debug_text(), reversed.debug_text());

        let metadata = forward
            .expr_metadata()
            .get_by_expr(&ExprId::new("expr.a"))
            .expect("node expression metadata");
        assert_eq!(
            metadata.cluster_facts,
            vec![ClusterFactId::new(0), ClusterFactId::new(1)]
        );
        assert_eq!(
            forward
                .cluster_facts()
                .get(ClusterFactId::new(1))
                .expect("trace cluster fact")
                .provenance(),
            &ClusterFactProvenance::TraceStep(ClusterStepId::new(0))
        );

        let insertion_sources = forward
            .inserted_coercions()
            .canonical_iter()
            .map(|(_, insertion)| insertion.source)
            .collect::<Vec<_>>();
        assert!(insertion_sources.contains(&CoercionInsertionSource::SourceQua));
        assert!(insertion_sources.contains(&CoercionInsertionSource::InsertedWidening));
    }

    #[test]
    fn failed_selection_statuses_preserve_failed_nodes_without_insertions() {
        let source = source_id(5);
        let expr_site = TypedSiteRef::Node(TypedNodeId::new(0));
        let empty_cluster_facts = ClusterFactTable::new();

        let typed = typed_ast_fixture(
            source,
            TypeEntryActual::CandidateSet(OpenCandidateSetId::new(0)),
        );
        let collection = OverloadCollectionOutput::collect(
            [site("call", expr_site.clone(), source, 10)],
            [candidate(
                "call",
                "no-match-root",
                "no-match",
                0,
                Some(NormalizedTypeId::new(4)),
            )],
        );
        let expansion = TemplateExpansionOutput::expand(&collection);
        let viability =
            CandidateViabilityOutput::filter(&expansion, Vec::<CandidateViabilityInput>::new());
        let graphs =
            SpecificityGraphOutput::build(&viability, Vec::<SpecificityComparisonInput>::new());
        let selection =
            OverloadSelectionOutput::resolve(&graphs, Vec::<OverloadSiteResolutionInput>::new());
        let resolved = assemble(
            AssemblyRefs {
                typed_ast: &typed,
                cluster_facts: &empty_cluster_facts,
                collection: &collection,
                expansion: &expansion,
                viability: &viability,
                graphs: &graphs,
                selection: &selection,
            },
            vec![expression("expr.no_match", expr_site.clone())],
        )
        .expect("no-match assembly succeeds");
        assert!(matches!(
            overload_record(&resolved, "expr.no_match").status,
            OverloadResolutionStatus::NoMatch { .. }
        ));
        assert_failed_node_without_insertions(&resolved);

        let typed = typed_ast_fixture(
            source,
            TypeEntryActual::CandidateSet(OpenCandidateSetId::new(0)),
        );
        let collection = OverloadCollectionOutput::collect(
            [site("call", expr_site.clone(), source, 20)],
            [
                candidate(
                    "call",
                    "ambiguous-a-root",
                    "ambiguous-a",
                    0,
                    Some(NormalizedTypeId::new(4)),
                ),
                candidate(
                    "call",
                    "ambiguous-b-root",
                    "ambiguous-b",
                    1,
                    Some(NormalizedTypeId::new(5)),
                ),
            ],
        );
        let expansion = TemplateExpansionOutput::expand(&collection);
        let left = candidate_id_by_symbol(expansion.candidates(), "ambiguous-a");
        let right = candidate_id_by_symbol(expansion.candidates(), "ambiguous-b");
        let viability =
            CandidateViabilityOutput::filter(&expansion, [viable_input(left), viable_input(right)]);
        let left = candidate_id_by_symbol(viability.candidates(), "ambiguous-a");
        let right = candidate_id_by_symbol(viability.candidates(), "ambiguous-b");
        let graphs = SpecificityGraphOutput::build(
            &viability,
            [SpecificityComparisonInput {
                left,
                right,
                status: SpecificityComparisonStatus::Incomparable,
                reasons: vec![SpecificityReasonKey::new("distinct-roots")],
            }],
        );
        let selection =
            OverloadSelectionOutput::resolve(&graphs, Vec::<OverloadSiteResolutionInput>::new());
        let resolved = assemble(
            AssemblyRefs {
                typed_ast: &typed,
                cluster_facts: &empty_cluster_facts,
                collection: &collection,
                expansion: &expansion,
                viability: &viability,
                graphs: &graphs,
                selection: &selection,
            },
            vec![expression("expr.ambiguous", expr_site.clone())],
        )
        .expect("ambiguous assembly succeeds");
        assert!(matches!(
            overload_record(&resolved, "expr.ambiguous").status,
            OverloadResolutionStatus::Ambiguous { .. }
        ));
        assert_failed_node_without_insertions(&resolved);

        let typed = typed_ast_fixture(
            source,
            TypeEntryActual::CandidateSet(OpenCandidateSetId::new(0)),
        );
        let collection = OverloadCollectionOutput::collect(
            [site("call", expr_site.clone(), source, 30)],
            [candidate(
                "call",
                "blocked-root",
                "blocked",
                0,
                Some(NormalizedTypeId::new(4)),
            )],
        );
        let expansion = TemplateExpansionOutput::expand(&collection);
        let candidate_id = candidate_id_by_symbol(expansion.candidates(), "blocked");
        let viability = CandidateViabilityOutput::filter(&expansion, [viable_input(candidate_id)]);
        let graphs =
            SpecificityGraphOutput::build(&viability, Vec::<SpecificityComparisonInput>::new());
        let selection =
            OverloadSelectionOutput::resolve(&graphs, Vec::<OverloadSiteResolutionInput>::new());
        let resolved = assemble(
            AssemblyRefs {
                typed_ast: &typed,
                cluster_facts: &empty_cluster_facts,
                collection: &collection,
                expansion: &expansion,
                viability: &viability,
                graphs: &graphs,
                selection: &selection,
            },
            vec![expression("expr.blocked", expr_site.clone())],
        )
        .expect("blocked assembly succeeds");
        assert!(matches!(
            overload_record(&resolved, "expr.blocked").status,
            OverloadResolutionStatus::Blocked {
                reason: OverloadBlockedReason::MissingSelectionPayload
            }
        ));
        assert_failed_node_without_insertions(&resolved);

        let typed = typed_ast_fixture(
            source,
            TypeEntryActual::CandidateSet(OpenCandidateSetId::new(0)),
        );
        let mut refinement = candidate(
            "call",
            "join-root",
            "join-refinement",
            1,
            Some(NormalizedTypeId::new(5)),
        );
        refinement.declaration_kind = CandidateDeclarationKind::Redefinition;
        refinement.origin = CandidateOrigin::Redefinition {
            refined: symbol("join-root"),
        };
        refinement.coherence = Some(CoherenceStatus::Accepted);
        let collection = OverloadCollectionOutput::collect(
            [site("call", expr_site.clone(), source, 40)],
            [
                candidate(
                    "call",
                    "join-root",
                    "join-root",
                    0,
                    Some(NormalizedTypeId::new(4)),
                ),
                refinement,
            ],
        );
        let expansion = TemplateExpansionOutput::expand(&collection);
        let root = candidate_id_by_symbol(expansion.candidates(), "join-root");
        let refinement = candidate_id_by_symbol(expansion.candidates(), "join-refinement");
        let viability = CandidateViabilityOutput::filter(
            &expansion,
            [viable_input(root), viable_input(refinement)],
        );
        let root = candidate_id_by_symbol(viability.candidates(), "join-root");
        let refinement = candidate_id_by_symbol(viability.candidates(), "join-refinement");
        let graphs = SpecificityGraphOutput::build(
            &viability,
            [SpecificityComparisonInput {
                left: root,
                right: refinement,
                status: SpecificityComparisonStatus::LeftAtLeastRight,
                reasons: vec![SpecificityReasonKey::new("root-covers-refinement")],
            }],
        );
        let selection = OverloadSelectionOutput::resolve(
            &graphs,
            [OverloadSiteResolutionInput {
                site: OverloadSiteId::new(0),
                refinements: vec![refinement],
                refinement_join: RefinementJoinPayload {
                    status: RefinementJoinStatus::Incompatible(
                        RefinementJoinFailure::MissingJoinPayload,
                    ),
                    exposed_result: None,
                },
                inserted_views: Vec::new(),
            }],
        );
        let resolved = assemble(
            AssemblyRefs {
                typed_ast: &typed,
                cluster_facts: &empty_cluster_facts,
                collection: &collection,
                expansion: &expansion,
                viability: &viability,
                graphs: &graphs,
                selection: &selection,
            },
            vec![expression("expr.incompatible", expr_site)],
        )
        .expect("incompatible refinement assembly succeeds");
        assert!(matches!(
            overload_record(&resolved, "expr.incompatible").status,
            OverloadResolutionStatus::IncompatibleRefinementJoin {
                reason: RefinementJoinFailure::MissingJoinPayload,
                ..
            }
        ));
        assert_failed_node_without_insertions(&resolved);
    }

    #[test]
    fn specificity_and_selection_diagnostics_are_remapped_to_resolved_ids() {
        let source = source_id(6);
        let typed = typed_ast_fixture(
            source,
            TypeEntryActual::CandidateSet(OpenCandidateSetId::new(0)),
        );
        let expr_site = TypedSiteRef::Node(TypedNodeId::new(0));
        let cluster_facts = ClusterFactTable::new();
        let collection = OverloadCollectionOutput::collect(
            [site("call", expr_site.clone(), source, 10)],
            [
                candidate(
                    "call",
                    "left-root",
                    "left",
                    0,
                    Some(NormalizedTypeId::new(4)),
                ),
                candidate(
                    "call",
                    "right-root",
                    "right",
                    1,
                    Some(NormalizedTypeId::new(5)),
                ),
            ],
        );
        let expansion = TemplateExpansionOutput::expand(&collection);
        let left = candidate_id_by_symbol(expansion.candidates(), "left");
        let right = candidate_id_by_symbol(expansion.candidates(), "right");
        let viability =
            CandidateViabilityOutput::filter(&expansion, [viable_input(left), viable_input(right)]);
        let graphs =
            SpecificityGraphOutput::build(&viability, Vec::<SpecificityComparisonInput>::new());
        let selection =
            OverloadSelectionOutput::resolve(&graphs, Vec::<OverloadSiteResolutionInput>::new());

        let resolved = assemble(
            AssemblyRefs {
                typed_ast: &typed,
                cluster_facts: &cluster_facts,
                collection: &collection,
                expansion: &expansion,
                viability: &viability,
                graphs: &graphs,
                selection: &selection,
            },
            vec![expression("expr.diagnostics", expr_site)],
        )
        .expect("diagnostic remap assembly succeeds");

        let graph = resolved
            .specificity_graphs()
            .get(SpecificityGraphId::new(0))
            .expect("specificity graph");
        assert_eq!(graph.diagnostics.len(), 1);
        assert_eq!(graph.comparisons.len(), 1);
        assert_eq!(graph.comparisons[0].diagnostics, graph.diagnostics);
        assert!(matches!(
            graph.comparisons[0].status,
            SpecificityComparisonOutcome::Blocked(_)
        ));

        let specificity_diagnostic = resolved
            .diagnostics()
            .get(graph.diagnostics[0])
            .expect("specificity diagnostic");
        assert!(matches!(
            specificity_diagnostic.source,
            ResolvedTypedDiagnosticSource::Specificity(_)
        ));
        assert_eq!(
            specificity_diagnostic.message_key,
            "overload.specificity.missing_comparison_payload"
        );

        let record = overload_record(&resolved, "expr.diagnostics");
        assert!(record.diagnostics.contains(&graph.diagnostics[0]));
        let selection_diagnostic = record
            .diagnostics
            .iter()
            .copied()
            .find(|id| {
                resolved.diagnostics().get(*id).is_some_and(|diagnostic| {
                    matches!(
                        diagnostic.source,
                        ResolvedTypedDiagnosticSource::Selection(_)
                    )
                })
            })
            .expect("selection diagnostic on overload record");
        assert_eq!(
            resolved
                .diagnostics()
                .get(selection_diagnostic)
                .expect("selection diagnostic")
                .message_key,
            "overload.selection.blocked_specificity_comparison"
        );
        assert_eq!(
            resolved
                .expr_metadata()
                .get_by_expr(&ExprId::new("expr.diagnostics"))
                .expect("expression metadata")
                .diagnostics,
            record.diagnostics
        );
    }

    #[test]
    fn validation_rejects_unknown_cluster_fact_and_unsupported_insertions() {
        let source = source_id(7);
        let typed = typed_ast_fixture(
            source,
            TypeEntryActual::CandidateSet(OpenCandidateSetId::new(0)),
        );
        let expr_site = TypedSiteRef::Node(TypedNodeId::new(0));
        let cluster_facts = ClusterFactTable::new();
        let collection = OverloadCollectionOutput::collect(
            [site("call", expr_site.clone(), source, 10)],
            [candidate(
                "call",
                "root",
                "root",
                0,
                Some(NormalizedTypeId::new(4)),
            )],
        );
        let expansion = TemplateExpansionOutput::expand(&collection);
        let root = candidate_id_by_symbol(expansion.candidates(), "root");
        let viability = CandidateViabilityOutput::filter(&expansion, [viable_input(root)]);
        let graphs =
            SpecificityGraphOutput::build(&viability, Vec::<SpecificityComparisonInput>::new());
        let selection =
            OverloadSelectionOutput::resolve(&graphs, Vec::<OverloadSiteResolutionInput>::new());

        let invalid_fact = ClusterFactId::new(99);
        let error = assemble(
            AssemblyRefs {
                typed_ast: &typed,
                cluster_facts: &cluster_facts,
                collection: &collection,
                expansion: &expansion,
                viability: &viability,
                graphs: &graphs,
                selection: &selection,
            },
            vec![ExpressionMetadataInput {
                expr: ExprId::new("expr.bad_fact"),
                typed_site: expr_site.clone(),
                local_context: None,
                cluster_facts: vec![invalid_fact],
            }],
        )
        .expect_err("unknown cluster fact is rejected");
        assert!(matches!(
            error,
            ResolvedTypedAstError::InvalidClusterFact { fact } if fact == invalid_fact
        ));

        let error = assemble(
            AssemblyRefs {
                typed_ast: &typed,
                cluster_facts: &cluster_facts,
                collection: &collection,
                expansion: &expansion,
                viability: &viability,
                graphs: &graphs,
                selection: &selection,
            },
            vec![
                expression("expr.site_a", expr_site.clone()),
                expression("expr.site_b", expr_site.clone()),
            ],
        )
        .expect_err("duplicate expression metadata site is rejected");
        assert!(matches!(
            error,
            ResolvedTypedAstError::DuplicateExpressionSite { site } if site == expr_site
        ));

        let inputs = ResolvedTypedAstInputs {
            typed_ast: &typed,
            cluster_facts: &cluster_facts,
            overload_collection: &collection,
            template_expansion: &expansion,
            viability: &viability,
            specificity: &graphs,
            overload_selection: &selection,
            expressions: Vec::new(),
            node_hints: Vec::new(),
        };
        let mut insertions = CoercionInsertionTable::new();
        let mut inserted_by_view = BTreeMap::new();
        let mut inserted_by_typed_site = BTreeMap::new();
        let error = project_inserted_view(
            InsertedViewId::new(99),
            &inputs,
            &mut insertions,
            &mut inserted_by_view,
            &mut inserted_by_typed_site,
        )
        .expect_err("unknown inserted view id is rejected");
        assert!(matches!(
            error,
            ResolvedTypedAstError::InvalidInsertedView { view } if view == InsertedViewId::new(99)
        ));

        let root = candidate_id_by_symbol(viability.candidates(), "root");
        let narrowing_selection = OverloadSelectionOutput::resolve(
            &graphs,
            [OverloadSiteResolutionInput {
                site: OverloadSiteId::new(0),
                refinements: Vec::new(),
                refinement_join: RefinementJoinPayload {
                    status: RefinementJoinStatus::Compatible,
                    exposed_result: None,
                },
                inserted_views: vec![InsertedViewInput {
                    argument: expr_site.clone(),
                    target: NormalizedTypeId::new(2),
                    selected_candidate: root,
                    kind: InsertedViewKind::Narrowing,
                    status: InsertedViewStatus::Accepted,
                    reason: InsertedViewReasonKey::new("unsupported-narrowing"),
                    evidence_facts: Vec::new(),
                    path: Some(QuaPathKey::new("B<A")),
                }],
            }],
        );
        let resolved = assemble(
            AssemblyRefs {
                typed_ast: &typed,
                cluster_facts: &cluster_facts,
                collection: &collection,
                expansion: &expansion,
                viability: &viability,
                graphs: &graphs,
                selection: &narrowing_selection,
            },
            vec![expression("expr.narrowing", expr_site.clone())],
        )
        .expect("selection-level narrowing rejection assembles as failed overload");
        assert!(matches!(
            overload_record(&resolved, "expr.narrowing").status,
            OverloadResolutionStatus::Blocked {
                reason: OverloadBlockedReason::InvalidInsertedView
            }
        ));
        assert_failed_node_without_insertions(&resolved);

        let invalid_candidate_selection = OverloadSelectionOutput::resolve(
            &graphs,
            [OverloadSiteResolutionInput {
                site: OverloadSiteId::new(0),
                refinements: Vec::new(),
                refinement_join: RefinementJoinPayload {
                    status: RefinementJoinStatus::Compatible,
                    exposed_result: None,
                },
                inserted_views: vec![InsertedViewInput {
                    argument: expr_site.clone(),
                    target: NormalizedTypeId::new(2),
                    selected_candidate: OverloadCandidateId::new(99),
                    kind: InsertedViewKind::Widening,
                    status: InsertedViewStatus::Accepted,
                    reason: InsertedViewReasonKey::new("foreign-candidate"),
                    evidence_facts: Vec::new(),
                    path: None,
                }],
            }],
        );
        let resolved = assemble(
            AssemblyRefs {
                typed_ast: &typed,
                cluster_facts: &cluster_facts,
                collection: &collection,
                expansion: &expansion,
                viability: &viability,
                graphs: &graphs,
                selection: &invalid_candidate_selection,
            },
            vec![expression("expr.invalid_candidate", expr_site)],
        )
        .expect("selection-level invalid candidate rejection assembles as failed overload");
        assert!(matches!(
            overload_record(&resolved, "expr.invalid_candidate").status,
            OverloadResolutionStatus::Blocked {
                reason: OverloadBlockedReason::InvalidInsertedView
            }
        ));
        assert_failed_node_without_insertions(&resolved);
    }

    struct AssemblyRefs<'a> {
        typed_ast: &'a TypedAst,
        cluster_facts: &'a ClusterFactTable,
        collection: &'a OverloadCollectionOutput,
        expansion: &'a TemplateExpansionOutput,
        viability: &'a CandidateViabilityOutput,
        graphs: &'a SpecificityGraphOutput,
        selection: &'a OverloadSelectionOutput,
    }

    fn assemble(
        refs: AssemblyRefs<'_>,
        expressions: Vec<ExpressionMetadataInput>,
    ) -> Result<ResolvedTypedAst, ResolvedTypedAstError> {
        ResolvedTypedAst::assemble(ResolvedTypedAstInputs {
            typed_ast: refs.typed_ast,
            cluster_facts: refs.cluster_facts,
            overload_collection: refs.collection,
            template_expansion: refs.expansion,
            viability: refs.viability,
            specificity: refs.graphs,
            overload_selection: refs.selection,
            expressions,
            node_hints: Vec::new(),
        })
    }

    fn deterministic_order_fixture(reverse: bool) -> ResolvedTypedAst {
        let source = source_id(4);
        let typed = typed_ast_fixture(
            source,
            TypeEntryActual::CandidateSet(OpenCandidateSetId::new(0)),
        );
        let expr_site = TypedSiteRef::Node(TypedNodeId::new(0));
        let role_site = TypedSiteRef::Role {
            node: TypedNodeId::new(0),
            role: TypeRole::new("arg0"),
        };
        let mut cluster_facts = ClusterFactTable::new();
        let input_fact = cluster_facts.insert(cluster_fact(source, 3));
        let mut trace_draft = cluster_fact(source, 4);
        trace_draft.provenance = ClusterFactProvenance::TraceStep(ClusterStepId::new(0));
        let trace_fact = cluster_facts.insert(trace_draft);

        let mut candidates = vec![
            candidate("call", "root", "root", 0, Some(NormalizedTypeId::new(4))),
            candidate(
                "call",
                "other-root",
                "other",
                1,
                Some(NormalizedTypeId::new(5)),
            ),
        ];
        if reverse {
            candidates.reverse();
        }
        let collection = OverloadCollectionOutput::collect(
            [site("call", expr_site.clone(), source, 10)],
            candidates,
        );
        let expansion = TemplateExpansionOutput::expand(&collection);
        let root = candidate_id_by_symbol(expansion.candidates(), "root");
        let other = candidate_id_by_symbol(expansion.candidates(), "other");
        let viability_inputs = if reverse {
            vec![viable_input(other), viable_input(root)]
        } else {
            vec![viable_input(root), viable_input(other)]
        };
        let viability = CandidateViabilityOutput::filter(&expansion, viability_inputs);
        let root = candidate_id_by_symbol(viability.candidates(), "root");
        let other = candidate_id_by_symbol(viability.candidates(), "other");
        let graphs = SpecificityGraphOutput::build(
            &viability,
            [SpecificityComparisonInput {
                left: root,
                right: other,
                status: SpecificityComparisonStatus::LeftAtLeastRight,
                reasons: vec![SpecificityReasonKey::new("root-preferred")],
            }],
        );
        let mut inserted_views = vec![
            InsertedViewInput {
                argument: role_site.clone(),
                target: NormalizedTypeId::new(2),
                selected_candidate: root,
                kind: InsertedViewKind::Widening,
                status: InsertedViewStatus::Accepted,
                reason: InsertedViewReasonKey::new("role-widening"),
                evidence_facts: vec![TypeFactId::new(0)],
                path: Some(QuaPathKey::new("A<B")),
            },
            InsertedViewInput {
                argument: expr_site.clone(),
                target: NormalizedTypeId::new(3),
                selected_candidate: root,
                kind: InsertedViewKind::SourceQua,
                status: InsertedViewStatus::Accepted,
                reason: InsertedViewReasonKey::new("source-qua"),
                evidence_facts: Vec::new(),
                path: Some(QuaPathKey::new("qua")),
            },
        ];
        if reverse {
            inserted_views.reverse();
        }
        let selection = OverloadSelectionOutput::resolve(
            &graphs,
            [OverloadSiteResolutionInput {
                site: OverloadSiteId::new(0),
                refinements: Vec::new(),
                refinement_join: RefinementJoinPayload {
                    status: RefinementJoinStatus::Compatible,
                    exposed_result: Some(ExposedResultPayload {
                        result: Some(NormalizedTypeId::new(9)),
                        source: ExposedResultSource::SelectedRoot,
                        evidence: Vec::new(),
                    }),
                },
                inserted_views,
            }],
        );
        let mut expressions = vec![
            ExpressionMetadataInput {
                expr: ExprId::new("expr.a"),
                typed_site: expr_site,
                local_context: None,
                cluster_facts: if reverse {
                    vec![trace_fact, input_fact, trace_fact]
                } else {
                    vec![input_fact, trace_fact, input_fact]
                },
            },
            ExpressionMetadataInput {
                expr: ExprId::new("expr.z"),
                typed_site: role_site,
                local_context: None,
                cluster_facts: Vec::new(),
            },
        ];
        if reverse {
            expressions.reverse();
        }

        assemble(
            AssemblyRefs {
                typed_ast: &typed,
                cluster_facts: &cluster_facts,
                collection: &collection,
                expansion: &expansion,
                viability: &viability,
                graphs: &graphs,
                selection: &selection,
            },
            expressions,
        )
        .expect("deterministic order assembly succeeds")
    }

    fn expression(expr: &str, typed_site: TypedSiteRef) -> ExpressionMetadataInput {
        ExpressionMetadataInput {
            expr: ExprId::new(expr),
            typed_site,
            local_context: None,
            cluster_facts: Vec::new(),
        }
    }

    fn viable_input(candidate: OverloadCandidateId) -> CandidateViabilityInput {
        CandidateViabilityInput {
            candidate,
            arguments: vec![ArgumentViabilityEvidence::Exact {
                actual: NormalizedTypeId::new(1),
            }],
        }
    }

    fn overload_record<'a>(
        resolved: &'a ResolvedTypedAst,
        expr: &str,
    ) -> &'a OverloadResolutionRecord {
        let metadata = resolved
            .expr_metadata()
            .get_by_expr(&ExprId::new(expr))
            .unwrap_or_else(|| panic!("missing metadata {expr}"));
        resolved
            .resolved_overloads()
            .get(metadata.overload.expect("metadata has overload"))
            .unwrap_or_else(|| panic!("missing overload record {expr}"))
    }

    fn assert_failed_node_without_insertions(resolved: &ResolvedTypedAst) {
        assert!(resolved.inserted_coercions().is_empty());
        assert!(matches!(
            resolved
                .nodes()
                .node(ResolvedTypedNodeId::new(0))
                .expect("expression node")
                .kind,
            ResolvedTypedNodeKind::FailedOverload { .. }
        ));
    }

    fn typed_ast_fixture(source: SourceId, actual: TypeEntryActual) -> TypedAst {
        let expr_node = TypedNodeId::new(0);
        let root_node = TypedNodeId::new(1);
        let context_id = LocalTypeContextId::new(0);
        let type_entry = TypeEntryId::new(0);
        let fact = TypeFactId::new(0);
        let mut builder = TypedArenaBuilder::new();
        let mut expr_links = TypedNodeLinks {
            context: Some(context_id),
            type_entry: Some(type_entry),
            facts: vec![fact],
            ..TypedNodeLinks::default()
        };
        expr_links.diagnostics = Vec::new();
        builder
            .push(
                TypedNode::new(
                    "FunctorApplication",
                    SourceAnchor::Range(range(source, 10, 20)),
                )
                .with_typing(TypingState::Successful)
                .with_links(expr_links),
            )
            .expect("expr node");
        builder
            .push(
                TypedNode::new("CompilationUnit", SourceAnchor::Range(range(source, 0, 30)))
                    .with_children(vec![expr_node])
                    .with_typing(TypingState::Successful),
            )
            .expect("root node");
        let arena = builder.finish(Some(root_node)).expect("typed arena");

        let mut facts = TypeFactTable::new();
        facts.insert(TypeFactDraft {
            subject: TypedSiteRef::Node(expr_node),
            predicate: TypePredicateRef::new("registered"),
            polarity: Polarity::Positive,
            provenance: FactProvenance::Builtin(BuiltinRuleId::new("fixture")),
            status: FactStatus::Known,
        });
        let mut contexts = LocalTypeContextTable::new();
        contexts.insert(LocalTypeContextDraft {
            owner: TypedSiteRef::Node(expr_node),
            parent: None,
            layer: TypeContextLayer::Expression,
            bindings: Vec::new(),
            introduced_assumptions: Vec::new(),
            visible_facts: vec![fact],
            recovery: ContextRecoveryState::Normal,
        });
        let mut types = TypeTable::new();
        types.insert(TypeEntryDraft {
            owner: TypedSiteRef::Node(expr_node),
            expected: None,
            actual,
            status: TypeStatus::Known,
            provenance: TypeProvenance::Builtin(BuiltinRuleId::new("fixture")),
        });

        TypedAst::try_new(TypedAstParts {
            source_id: source,
            module_id: module(),
            resolved_root: None,
            nodes: arena,
            contexts,
            types,
            facts,
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("typed ast")
    }

    fn site(key: &str, owner: TypedSiteRef, source: SourceId, start: usize) -> OverloadSiteInput {
        OverloadSiteInput {
            key: OverloadSiteKey::new(key),
            owner,
            source_range: range(source, start, start + 5),
            kind: OverloadSiteKind::FunctorApplication,
            name: OverloadNameKey::new(key),
            arguments: vec![TypedSiteRef::Role {
                node: TypedNodeId::new(0),
                role: TypeRole::new("arg0"),
            }],
            expected: None,
            source_qua: Vec::<SourceQuaView>::new(),
            recovery: OverloadSiteRecovery::Normal,
        }
    }

    fn candidate(
        site: &str,
        root: &str,
        symbol_name: &str,
        declaration_order: usize,
        result: Option<NormalizedTypeId>,
    ) -> OverloadCandidateInput {
        OverloadCandidateInput {
            site: OverloadSiteKey::new(site),
            symbol: symbol(symbol_name),
            ordinary_root: symbol(root),
            declaration_kind: CandidateDeclarationKind::Functor,
            parameters: vec![NormalizedTypeId::new(1)],
            result,
            origin: CandidateOrigin::Ordinary,
            template: None,
            coherence: None,
            provenance: CandidateProvenance {
                stable_key: crate::overload_resolution::CandidateProvenanceKey::new(symbol_name),
                source_range: None,
                scope: CandidateScope::Local,
                declaration_order,
            },
        }
    }

    fn candidate_id_by_symbol(
        candidates: &crate::overload_resolution::OverloadCandidateTable,
        name: &str,
    ) -> OverloadCandidateId {
        candidates
            .iter()
            .find(|(_, candidate)| candidate.symbol.local().as_str() == name)
            .map(|(id, _)| id)
            .unwrap_or_else(|| panic!("missing candidate {name}"))
    }

    fn cluster_fact(source: SourceId, offset: usize) -> ClusterFactDraft {
        ClusterFactDraft {
            fingerprint: ClusterFactFingerprint::new(format!("fact-{offset}")),
            source_type: ClusterTypeFingerprint::new("A"),
            attribute: ClusterAttributeFingerprint::new("attr"),
            generated_type: ClusterTypeFingerprint::new("B"),
            provenance: ClusterFactProvenance::Input,
            source_range: range(source, offset, offset + 1),
        }
    }

    fn symbol(name: &str) -> SymbolId {
        SymbolId::new(
            module(),
            LocalSymbolId::new(name),
            FullyQualifiedName::new(format!("pkg::main::{name}")),
        )
    }

    fn module() -> ModuleId {
        ModuleId::new(PackageId::new("pkg"), ModulePath::new("main"))
    }

    fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
        SourceRange {
            source_id,
            start,
            end,
        }
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
