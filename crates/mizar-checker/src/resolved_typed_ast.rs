//! Final source-shaped resolved typed AST assembly for checker phase 8.

use crate::{
    binding_env::{BindingContextLayer, BindingContextOwner, BindingContextRecovery, BindingEnv},
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
    source_attribute::SourceAttributeHandoff,
    source_context::SourceBindingContextHandoff,
    source_type::SourceTypeApplicationHandoff,
    type_checker::{
        CheckedFormulaId, CheckedFormulaTable, CheckedStatementOwner, ExportStatus, FormulaKind,
        FormulaStatus, TermFormulaInferenceOutput, Visibility,
    },
    typed_ast::{
        LocalTypeContextId, NodeRecoveryState, NormalizedTypeId, TypeDiagnosticId,
        TypeDiagnosticSeverity, TypeDiagnosticTable, TypeEntryActual, TypeFactId, TypedAst,
        TypedNodeId, TypedSiteRef, TypingState,
    },
};
use mizar_resolve::resolved_ast::{ModuleId, SemanticOrigin, SymbolId};
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
dense_id!(StatementSemanticId);
dense_id!(StatementProofIntentId);
dense_id!(CheckedProofId);
dense_id!(CheckedProofNodeId);
dense_id!(CheckedTerminalGoalId);

string_key!(ExprId);
string_key!(SourceNodeRole);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedTypedAst {
    source_id: SourceId,
    module_id: ModuleId,
    source_context: Option<SourceBindingContextHandoff>,
    source_type: Option<SourceTypeApplicationHandoff>,
    source_attribute: Option<SourceAttributeHandoff>,
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
    checked_formulas: CheckedFormulaTable,
    statement_semantics: StatementSemanticTable,
    checked_proofs: CheckedProofTable,
    checked_proof_nodes: CheckedProofNodeTable,
    checked_terminal_goals: CheckedTerminalGoalTable,
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

    pub const fn source_context(&self) -> Option<&SourceBindingContextHandoff> {
        self.source_context.as_ref()
    }

    pub const fn source_type(&self) -> Option<&SourceTypeApplicationHandoff> {
        self.source_type.as_ref()
    }

    pub const fn source_attribute(&self) -> Option<&SourceAttributeHandoff> {
        self.source_attribute.as_ref()
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

    pub const fn checked_formulas(&self) -> &CheckedFormulaTable {
        &self.checked_formulas
    }

    pub const fn statement_semantics(&self) -> &StatementSemanticTable {
        &self.statement_semantics
    }

    pub const fn checked_proofs(&self) -> &CheckedProofTable {
        &self.checked_proofs
    }

    pub const fn checked_proof_nodes(&self) -> &CheckedProofNodeTable {
        &self.checked_proof_nodes
    }

    pub const fn checked_terminal_goals(&self) -> &CheckedTerminalGoalTable {
        &self.checked_terminal_goals
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("resolved-typed-ast-debug-v1\n");
        output.push_str("module: ");
        write_module_id(&mut output, &self.module_id);
        output.push('\n');
        output.push_str("root: ");
        write_optional_resolved_node_id(&mut output, self.nodes.root());
        output.push('\n');
        if let Some(source_context) = &self.source_context {
            output.push_str(&source_context.debug_text());
        }
        if let Some(source_type) = &self.source_type {
            output.push_str(&source_type.debug_text());
        }
        if let Some(source_attribute) = &self.source_attribute {
            output.push_str(&source_attribute.debug_text());
        }
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
        if !self.statement_semantics.is_empty() {
            write_statement_semantics(&mut output, &self.statement_semantics);
        }
        if !self.checked_proofs.is_empty() {
            write_checked_proofs(&mut output, &self.checked_proofs);
            write_checked_proof_nodes(&mut output, &self.checked_proof_nodes);
            write_checked_terminal_goals(&mut output, &self.checked_terminal_goals);
        }
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
    pub statement_semantics: Option<StatementSemanticInputs<'a>>,
    pub statement_proofs: Option<StatementProofInputs<'a>>,
}

#[derive(Debug)]
pub struct StatementSemanticInputs<'a> {
    pub owner: &'a CheckedStatementOwner,
    pub binding_env: &'a BindingEnv,
    pub term_formula: &'a TermFormulaInferenceOutput,
    pub rows: Vec<StatementSemanticInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatementSemanticInput {
    pub owner: SymbolId,
    pub owner_node: TypedNodeId,
    pub formula: CheckedFormulaId,
    pub formula_node: TypedNodeId,
}

#[derive(Debug)]
pub struct StatementProofInputs<'a> {
    pub owner: &'a CheckedStatementOwner,
    pub rows: Vec<StatementProofIntentInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatementProofIntentInput {
    pub id: StatementProofIntentId,
    pub source_order: usize,
    pub statement: StatementSemanticId,
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub owner: SymbolId,
    pub owner_node: TypedNodeId,
    pub owner_range: SourceRange,
    pub owner_origin: SemanticOrigin,
    pub owner_visibility: Visibility,
    pub owner_export_status: ExportStatus,
    pub formula: CheckedFormulaId,
    pub formula_site: TypedSiteRef,
    pub formula_node: TypedNodeId,
    pub formula_range: SourceRange,
    pub recovery: NodeRecoveryState,
    pub policy: TheoremPolicyIntent,
    pub justification: TheoremJustificationIntent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TheoremPolicyIntent {
    Unmodified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TheoremJustificationIntent {
    Omitted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CheckedProofStatus {
    PendingAutomaticProof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CheckedProofNodeKind {
    TerminalGoal(CheckedTerminalGoalId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CheckedCitation {}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CheckedProofLabel {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedProof {
    pub id: CheckedProofId,
    pub source_order: usize,
    pub statement: StatementSemanticId,
    pub owner: SymbolId,
    pub owner_node: TypedNodeId,
    pub owner_visibility: Visibility,
    pub owner_export_status: ExportStatus,
    pub proposition: CheckedFormulaId,
    pub policy: TheoremPolicyIntent,
    pub justification: TheoremJustificationIntent,
    pub root: CheckedProofNodeId,
    pub status: CheckedProofStatus,
    pub source_range: SourceRange,
    pub owner_origin: SemanticOrigin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedProofNode {
    pub id: CheckedProofNodeId,
    pub proof: CheckedProofId,
    pub kind: CheckedProofNodeKind,
    pub source_range: SourceRange,
    pub recovery: NodeRecoveryState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedTerminalGoal {
    pub id: CheckedTerminalGoalId,
    pub proof: CheckedProofId,
    pub node: CheckedProofNodeId,
    pub statement: StatementSemanticId,
    pub owner: SymbolId,
    pub formula: CheckedFormulaId,
    pub formula_site: TypedSiteRef,
    pub formula_node: TypedNodeId,
    pub source_range: SourceRange,
    pub recovery: NodeRecoveryState,
    pub citations: Vec<CheckedCitation>,
    pub active_context: Vec<CheckedFormulaId>,
    pub local_path: String,
    pub label: Option<CheckedProofLabel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatementSemantic {
    pub id: StatementSemanticId,
    pub owner: SymbolId,
    pub owner_node: TypedNodeId,
    pub owner_range: SourceRange,
    pub owner_origin: SemanticOrigin,
    pub formula: CheckedFormulaId,
    pub formula_node: TypedNodeId,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StatementSemanticTable {
    entries: Vec<StatementSemantic>,
}

impl StatementSemanticTable {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn get(&self, id: StatementSemanticId) -> Option<&StatementSemantic> {
        self.entries.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (StatementSemanticId, &StatementSemantic)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CheckedProofTable {
    entries: Vec<CheckedProof>,
}

impl CheckedProofTable {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn get(&self, id: CheckedProofId) -> Option<&CheckedProof> {
        self.entries.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (CheckedProofId, &CheckedProof)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CheckedProofNodeTable {
    entries: Vec<CheckedProofNode>,
}

impl CheckedProofNodeTable {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn get(&self, id: CheckedProofNodeId) -> Option<&CheckedProofNode> {
        self.entries.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (CheckedProofNodeId, &CheckedProofNode)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CheckedTerminalGoalTable {
    entries: Vec<CheckedTerminalGoal>,
}

impl CheckedTerminalGoalTable {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn get(&self, id: CheckedTerminalGoalId) -> Option<&CheckedTerminalGoal> {
        self.entries.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (CheckedTerminalGoalId, &CheckedTerminalGoal)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
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
    StatementProofBundleMismatch,
    MissingStatementSemantic,
    NonSingletonStatementSemantic {
        count: usize,
    },
    DuplicateStatementOwner {
        owner: SymbolId,
    },
    DuplicateStatementFormula {
        formula: CheckedFormulaId,
    },
    StatementEnvironmentMismatch,
    InvalidStatementOwner {
        owner: SymbolId,
    },
    InvalidStatementFormula {
        formula: CheckedFormulaId,
    },
    InvalidStatementFormulaPayload {
        formula: CheckedFormulaId,
    },
    InvalidStatementTree,
    StatementSourceOrderMismatch,
    MissingStatementProofIntent,
    NonSingletonStatementProofIntent {
        count: usize,
    },
    DuplicateStatementProofIntent {
        id: StatementProofIntentId,
    },
    InvalidStatementProofIntent,
    InvalidStatementProofOutput,
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
            Self::StatementProofBundleMismatch => formatter.write_str(
                "statement semantic and proof-intent bundles must be supplied together",
            ),
            Self::MissingStatementSemantic => {
                formatter.write_str("statement semantic contract is missing its required row")
            }
            Self::NonSingletonStatementSemantic { count } => write!(
                formatter,
                "statement semantic contract requires exactly one row, got {count}"
            ),
            Self::DuplicateStatementOwner { owner } => write!(
                formatter,
                "duplicate statement semantic owner `{}`",
                owner.fqn().as_str()
            ),
            Self::DuplicateStatementFormula { formula } => write!(
                formatter,
                "duplicate statement semantic formula {}",
                formula.index()
            ),
            Self::StatementEnvironmentMismatch => formatter.write_str(
                "statement semantic symbol, binding, formula, and typed AST environments do not match",
            ),
            Self::InvalidStatementOwner { owner } => write!(
                formatter,
                "invalid statement semantic theorem owner `{}`",
                owner.fqn().as_str()
            ),
            Self::InvalidStatementFormula { formula } => write!(
                formatter,
                "invalid statement semantic checked formula {}",
                formula.index()
            ),
            Self::InvalidStatementFormulaPayload { formula } => write!(
                formatter,
                "statement semantic checked formula {} has forbidden payload",
                formula.index()
            ),
            Self::InvalidStatementTree => {
                formatter.write_str("invalid exact statement semantic typed tree")
            }
            Self::StatementSourceOrderMismatch => formatter
                .write_str("statement semantic theorem/formula source order does not match"),
            Self::MissingStatementProofIntent => {
                formatter.write_str("statement proof-intent contract is missing its required row")
            }
            Self::NonSingletonStatementProofIntent { count } => write!(
                formatter,
                "statement proof-intent contract requires exactly one row, got {count}"
            ),
            Self::DuplicateStatementProofIntent { id } => write!(
                formatter,
                "duplicate statement proof-intent id {}",
                id.index()
            ),
            Self::InvalidStatementProofIntent => {
                formatter.write_str("invalid exact statement proof-intent input")
            }
            Self::InvalidStatementProofOutput => {
                formatter.write_str("invalid exact checked proof output")
            }
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
        validate_statement_bundle_presence(&self.inputs)?;
        let (checked_formulas, statement_semantics) =
            build_statement_semantics(&self.inputs, source_id, &module_id)?;
        let (checked_proofs, checked_proof_nodes, checked_terminal_goals) = build_statement_proofs(
            &self.inputs,
            source_id,
            &module_id,
            &checked_formulas,
            &statement_semantics,
        )?;
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
            source_context: self.inputs.typed_ast.source_context().cloned(),
            source_type: self.inputs.typed_ast.source_type().cloned(),
            source_attribute: self.inputs.typed_ast.source_attribute().cloned(),
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
            checked_formulas,
            statement_semantics,
            checked_proofs,
            checked_proof_nodes,
            checked_terminal_goals,
        })
    }
}

fn validate_statement_bundle_presence(
    inputs: &ResolvedTypedAstInputs<'_>,
) -> Result<(), ResolvedTypedAstError> {
    match (&inputs.statement_semantics, &inputs.statement_proofs) {
        (None, None) | (Some(_), Some(_)) => Ok(()),
        _ => Err(ResolvedTypedAstError::StatementProofBundleMismatch),
    }
}

fn build_statement_semantics(
    inputs: &ResolvedTypedAstInputs<'_>,
    source_id: SourceId,
    module_id: &ModuleId,
) -> Result<(CheckedFormulaTable, StatementSemanticTable), ResolvedTypedAstError> {
    let Some(statement_inputs) = &inputs.statement_semantics else {
        return Ok((CheckedFormulaTable::new(), StatementSemanticTable::new()));
    };

    if statement_inputs.rows.is_empty() {
        return Err(ResolvedTypedAstError::MissingStatementSemantic);
    }
    let mut owners = BTreeSet::new();
    let mut formulas = BTreeSet::new();
    for row in &statement_inputs.rows {
        if !owners.insert(row.owner.clone()) {
            return Err(ResolvedTypedAstError::DuplicateStatementOwner {
                owner: row.owner.clone(),
            });
        }
        if !formulas.insert(row.formula) {
            return Err(ResolvedTypedAstError::DuplicateStatementFormula {
                formula: row.formula,
            });
        }
    }
    if statement_inputs.rows.len() != 1 {
        return Err(ResolvedTypedAstError::NonSingletonStatementSemantic {
            count: statement_inputs.rows.len(),
        });
    }

    if statement_inputs.binding_env.module_id() != module_id
        || statement_inputs.binding_env.source_id() != source_id
        || statement_inputs.term_formula.module_id() != module_id
        || statement_inputs.term_formula.source_id() != source_id
    {
        return Err(ResolvedTypedAstError::StatementEnvironmentMismatch);
    }

    let row = &statement_inputs.rows[0];
    let owner = statement_inputs.owner;
    if owner.symbol() != &row.owner
        || owner.symbol().module() != module_id
        || owner.origin().source_id() != source_id
        || owner.origin().module_id() != module_id
    {
        return Err(ResolvedTypedAstError::InvalidStatementOwner {
            owner: row.owner.clone(),
        });
    }
    let owner_range = owner.source_range();

    let term_formula = statement_inputs.term_formula;
    if term_formula.formulas().len() != 1
        || !term_formula.normalized_types().is_empty()
        || !term_formula.terms().is_empty()
        || !term_formula.candidate_sets().is_empty()
        || !term_formula.type_entries().is_empty()
        || !term_formula.facts().is_empty()
        || !term_formula.diagnostics().is_empty()
    {
        return Err(ResolvedTypedAstError::InvalidStatementFormulaPayload {
            formula: row.formula,
        });
    }
    let formula = term_formula.formulas().get(row.formula).ok_or(
        ResolvedTypedAstError::InvalidStatementFormula {
            formula: row.formula,
        },
    )?;
    if formula.kind != FormulaKind::Contradiction
        || formula.status != FormulaStatus::Checked
        || formula.recovery != NodeRecoveryState::Normal
    {
        return Err(ResolvedTypedAstError::InvalidStatementFormula {
            formula: row.formula,
        });
    }
    if !formula.terms.is_empty()
        || formula.asserted_type.is_some()
        || !formula.expected_types.is_empty()
        || formula.candidate_set.is_some()
        || !formula.facts.is_empty()
        || !formula.deferred.is_empty()
    {
        return Err(ResolvedTypedAstError::InvalidStatementFormulaPayload {
            formula: row.formula,
        });
    }

    let contexts = statement_inputs.binding_env.contexts();
    let Some(context) = contexts.get(formula.context) else {
        return Err(ResolvedTypedAstError::InvalidStatementFormula {
            formula: row.formula,
        });
    };
    if !exact_statement_binding_shape(StatementBindingShape {
        context_count: contexts.len(),
        owner_is_module: context.owner == BindingContextOwner::Module,
        parent_is_none: context.parent.is_none(),
        layer_is_module: context.layer == BindingContextLayer::Module,
        recovery_is_normal: context.recovery == BindingContextRecovery::Normal,
        context_bindings_empty: context.bindings.is_empty(),
        visible_bindings_empty: context.visible_bindings.is_empty(),
        binding_table_empty: statement_inputs.binding_env.bindings().is_empty(),
        diagnostics_empty: statement_inputs.binding_env.diagnostics().is_empty(),
    }) {
        return Err(ResolvedTypedAstError::InvalidStatementFormulaPayload {
            formula: row.formula,
        });
    }

    let TypedSiteRef::Node(_) = &formula.site else {
        return Err(ResolvedTypedAstError::InvalidStatementTree);
    };
    let formula_node_id = row.formula_node;
    let nodes = inputs.typed_ast.nodes();
    let Some(root_id) = nodes.root() else {
        return Err(ResolvedTypedAstError::InvalidStatementTree);
    };
    let Some(root) = nodes.node(root_id) else {
        return Err(ResolvedTypedAstError::InvalidStatementTree);
    };
    let Some(owner_node) = nodes.node(row.owner_node) else {
        return Err(ResolvedTypedAstError::InvalidStatementTree);
    };
    let Some(formula_node) = nodes.node(formula_node_id) else {
        return Err(ResolvedTypedAstError::InvalidStatementTree);
    };
    if !exact_statement_tree_shape(StatementTreeShape {
        node_count: nodes.len(),
        root_kind_matches: root.kind.as_str() == "source.module",
        owner_kind_matches: owner_node.kind.as_str() == "source.statement.theorem",
        formula_kind_matches: formula_node.kind.as_str() == "source.formula.contradiction",
        root_child_matches: root.children == [row.owner_node],
        owner_child_matches: owner_node.children == [formula_node_id],
        formula_children_empty: formula_node.children.is_empty(),
        root_recovery_normal: root.recovery == NodeRecoveryState::Normal,
        owner_recovery_normal: owner_node.recovery == NodeRecoveryState::Normal,
        formula_recovery_normal: formula_node.recovery == NodeRecoveryState::Normal,
        root_typing_successful: root.typing == TypingState::Successful,
        owner_typing_successful: owner_node.typing == TypingState::Successful,
        formula_typing_successful: formula_node.typing == TypingState::Successful,
    }) {
        return Err(ResolvedTypedAstError::InvalidStatementTree);
    }
    let mut statement_roles = BTreeMap::new();
    for hint in &inputs.node_hints {
        let ResolvedNodeKindHintKind::SourcePreserved { role } = &hint.kind else {
            return Err(ResolvedTypedAstError::InvalidStatementTree);
        };
        if statement_roles
            .insert(hint.typed_node, role.as_str())
            .is_some()
        {
            return Err(ResolvedTypedAstError::InvalidStatementTree);
        }
    }
    if statement_roles.len() != 3
        || statement_roles.get(&root_id) != Some(&"source.module")
        || statement_roles.get(&row.owner_node) != Some(&"source.statement.theorem")
        || statement_roles.get(&formula_node_id) != Some(&"source.formula.contradiction")
    {
        return Err(ResolvedTypedAstError::InvalidStatementTree);
    }
    let SourceAnchor::Range(root_range) = &root.anchor else {
        return Err(ResolvedTypedAstError::InvalidStatementTree);
    };
    let SourceAnchor::Range(typed_owner_range) = &owner_node.anchor else {
        return Err(ResolvedTypedAstError::InvalidStatementTree);
    };
    let SourceAnchor::Range(typed_formula_range) = &formula_node.anchor else {
        return Err(ResolvedTypedAstError::InvalidStatementTree);
    };
    if !exact_statement_range_shape(StatementRangeShape {
        root_ordered: root_range.start <= root_range.end,
        owner_ordered: owner_range.start <= owner_range.end,
        formula_ordered: formula.source_range.start <= formula.source_range.end,
        owner_range_matches: *typed_owner_range == owner_range,
        formula_range_matches: *typed_formula_range == formula.source_range,
        formula_recovery_matches: formula_node.recovery == formula.recovery,
        root_source_matches: root_range.source_id == source_id,
        owner_source_matches: typed_owner_range.source_id == source_id,
        formula_source_matches: typed_formula_range.source_id == source_id,
        root_contains_owner: root_range.start <= typed_owner_range.start
            && typed_owner_range.end <= root_range.end,
    }) {
        return Err(ResolvedTypedAstError::InvalidStatementTree);
    }
    if !(owner_range.start < formula.source_range.start
        && formula.source_range.end < owner_range.end)
    {
        return Err(ResolvedTypedAstError::StatementSourceOrderMismatch);
    }

    Ok((
        term_formula.formulas().clone(),
        StatementSemanticTable {
            entries: vec![StatementSemantic {
                id: StatementSemanticId::new(0),
                owner: row.owner.clone(),
                owner_node: row.owner_node,
                owner_range,
                owner_origin: owner.origin().clone(),
                formula: row.formula,
                formula_node: row.formula_node,
            }],
        },
    ))
}

fn build_statement_proofs(
    inputs: &ResolvedTypedAstInputs<'_>,
    source_id: SourceId,
    module_id: &ModuleId,
    checked_formulas: &CheckedFormulaTable,
    statements: &StatementSemanticTable,
) -> Result<
    (
        CheckedProofTable,
        CheckedProofNodeTable,
        CheckedTerminalGoalTable,
    ),
    ResolvedTypedAstError,
> {
    let Some(proof_inputs) = &inputs.statement_proofs else {
        return Ok((
            CheckedProofTable::new(),
            CheckedProofNodeTable::new(),
            CheckedTerminalGoalTable::new(),
        ));
    };
    if proof_inputs.rows.is_empty() {
        return Err(ResolvedTypedAstError::MissingStatementProofIntent);
    }
    let mut ids = BTreeSet::new();
    for row in &proof_inputs.rows {
        if !ids.insert(row.id) {
            return Err(ResolvedTypedAstError::DuplicateStatementProofIntent { id: row.id });
        }
    }
    if proof_inputs.rows.len() != 1 {
        return Err(ResolvedTypedAstError::NonSingletonStatementProofIntent {
            count: proof_inputs.rows.len(),
        });
    }

    let row = &proof_inputs.rows[0];
    let Some(statement) = statements.get(StatementSemanticId::new(0)) else {
        return Err(ResolvedTypedAstError::InvalidStatementProofIntent);
    };
    let Some(formula) = checked_formulas.get(statement.formula) else {
        return Err(ResolvedTypedAstError::InvalidStatementProofIntent);
    };
    let Some(statement_inputs) = &inputs.statement_semantics else {
        return Err(ResolvedTypedAstError::StatementProofBundleMismatch);
    };
    let owner = proof_inputs.owner;
    if owner != statement_inputs.owner
        || row.id != StatementProofIntentId::new(0)
        || row.source_order != 0
        || row.statement != StatementSemanticId::new(0)
        || row.source_id != source_id
        || row.module_id != *module_id
        || row.owner != statement.owner
        || row.owner != *owner.symbol()
        || row.owner_node != statement.owner_node
        || row.owner_range != statement.owner_range
        || row.owner_range != owner.source_range()
        || row.owner_origin != statement.owner_origin
        || row.owner_origin != *owner.origin()
        || row.owner_visibility != Visibility::Public
        || row.owner_visibility != owner.visibility()
        || row.owner_export_status != ExportStatus::Exported
        || row.owner_export_status != owner.export_status()
        || row.formula != statement.formula
        || row.formula != formula.id
        || row.formula_site != formula.site
        || row.formula_node != statement.formula_node
        || row.formula_range != formula.source_range
        || row.recovery != NodeRecoveryState::Normal
        || row.recovery != formula.recovery
        || row.policy != TheoremPolicyIntent::Unmodified
        || row.justification != TheoremJustificationIntent::Omitted
        || !matches!(row.formula_site, TypedSiteRef::Node(_))
    {
        return Err(ResolvedTypedAstError::InvalidStatementProofIntent);
    }

    let proofs = CheckedProofTable {
        entries: vec![CheckedProof {
            id: CheckedProofId::new(0),
            source_order: 0,
            statement: StatementSemanticId::new(0),
            owner: row.owner.clone(),
            owner_node: row.owner_node,
            owner_visibility: row.owner_visibility,
            owner_export_status: row.owner_export_status,
            proposition: row.formula,
            policy: row.policy,
            justification: row.justification,
            root: CheckedProofNodeId::new(0),
            status: CheckedProofStatus::PendingAutomaticProof,
            source_range: row.owner_range,
            owner_origin: row.owner_origin.clone(),
        }],
    };
    let nodes = CheckedProofNodeTable {
        entries: vec![CheckedProofNode {
            id: CheckedProofNodeId::new(0),
            proof: CheckedProofId::new(0),
            kind: CheckedProofNodeKind::TerminalGoal(CheckedTerminalGoalId::new(0)),
            source_range: row.formula_range,
            recovery: row.recovery,
        }],
    };
    let goals = CheckedTerminalGoalTable {
        entries: vec![CheckedTerminalGoal {
            id: CheckedTerminalGoalId::new(0),
            proof: CheckedProofId::new(0),
            node: CheckedProofNodeId::new(0),
            statement: StatementSemanticId::new(0),
            owner: row.owner.clone(),
            formula: row.formula,
            formula_site: row.formula_site.clone(),
            formula_node: row.formula_node,
            source_range: row.formula_range,
            recovery: row.recovery,
            citations: Vec::new(),
            active_context: Vec::new(),
            local_path: "proof/0".to_owned(),
            label: None,
        }],
    };
    validate_checked_proof_tables(checked_formulas, statements, &proofs, &nodes, &goals)?;
    Ok((proofs, nodes, goals))
}

fn validate_checked_proof_tables(
    checked_formulas: &CheckedFormulaTable,
    statements: &StatementSemanticTable,
    proofs: &CheckedProofTable,
    nodes: &CheckedProofNodeTable,
    goals: &CheckedTerminalGoalTable,
) -> Result<(), ResolvedTypedAstError> {
    let status_matches = proofs
        .get(CheckedProofId::new(0))
        .is_some_and(|proof| proof.status == CheckedProofStatus::PendingAutomaticProof);
    validate_checked_proof_tables_with_status_match(
        checked_formulas,
        statements,
        proofs,
        nodes,
        goals,
        status_matches,
    )
}

fn validate_checked_proof_tables_with_status_match(
    checked_formulas: &CheckedFormulaTable,
    statements: &StatementSemanticTable,
    proofs: &CheckedProofTable,
    nodes: &CheckedProofNodeTable,
    goals: &CheckedTerminalGoalTable,
    status_matches: bool,
) -> Result<(), ResolvedTypedAstError> {
    let Some(statement) = statements.get(StatementSemanticId::new(0)) else {
        return Err(ResolvedTypedAstError::InvalidStatementProofOutput);
    };
    let Some(formula) = checked_formulas.get(statement.formula) else {
        return Err(ResolvedTypedAstError::InvalidStatementProofOutput);
    };
    let Some(proof) = proofs.get(CheckedProofId::new(0)) else {
        return Err(ResolvedTypedAstError::InvalidStatementProofOutput);
    };
    let Some(node) = nodes.get(CheckedProofNodeId::new(0)) else {
        return Err(ResolvedTypedAstError::InvalidStatementProofOutput);
    };
    let Some(goal) = goals.get(CheckedTerminalGoalId::new(0)) else {
        return Err(ResolvedTypedAstError::InvalidStatementProofOutput);
    };
    if !exact_checked_proof_output_shape(CheckedProofOutputShape {
        proof_count: proofs.len(),
        node_count: nodes.len(),
        goal_count: goals.len(),
        proof_id_matches: proof.id == CheckedProofId::new(0),
        proof_root_matches: proof.root == node.id,
        proof_status_matches: status_matches,
        node_id_matches: node.id == CheckedProofNodeId::new(0),
        node_proof_matches: node.proof == proof.id,
        node_kind_matches: node.kind == CheckedProofNodeKind::TerminalGoal(goal.id),
        goal_id_matches: goal.id == CheckedTerminalGoalId::new(0),
        goal_proof_matches: goal.proof == proof.id,
        goal_node_matches: goal.node == node.id,
        citations_empty: goal.citations.is_empty(),
        active_context_empty: goal.active_context.is_empty(),
        local_path_matches: goal.local_path == "proof/0",
        label_is_none: goal.label.is_none(),
    }) || proof.source_order != 0
        || proof.statement != statement.id
        || proof.owner != statement.owner
        || proof.owner_node != statement.owner_node
        || proof.owner_visibility != Visibility::Public
        || proof.owner_export_status != ExportStatus::Exported
        || proof.proposition != statement.formula
        || proof.policy != TheoremPolicyIntent::Unmodified
        || proof.justification != TheoremJustificationIntent::Omitted
        || proof.source_range != statement.owner_range
        || proof.owner_origin != statement.owner_origin
        || node.source_range != formula.source_range
        || node.recovery != NodeRecoveryState::Normal
        || goal.statement != statement.id
        || goal.owner != statement.owner
        || goal.formula != statement.formula
        || goal.formula_site != formula.site
        || !matches!(goal.formula_site, TypedSiteRef::Node(_))
        || goal.formula_node != statement.formula_node
        || goal.source_range != formula.source_range
        || goal.recovery != NodeRecoveryState::Normal
    {
        return Err(ResolvedTypedAstError::InvalidStatementProofOutput);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CheckedProofOutputShape {
    proof_count: usize,
    node_count: usize,
    goal_count: usize,
    proof_id_matches: bool,
    proof_root_matches: bool,
    proof_status_matches: bool,
    node_id_matches: bool,
    node_proof_matches: bool,
    node_kind_matches: bool,
    goal_id_matches: bool,
    goal_proof_matches: bool,
    goal_node_matches: bool,
    citations_empty: bool,
    active_context_empty: bool,
    local_path_matches: bool,
    label_is_none: bool,
}

const fn exact_checked_proof_output_shape(shape: CheckedProofOutputShape) -> bool {
    shape.proof_count == 1
        && shape.node_count == 1
        && shape.goal_count == 1
        && shape.proof_id_matches
        && shape.proof_root_matches
        && shape.proof_status_matches
        && shape.node_id_matches
        && shape.node_proof_matches
        && shape.node_kind_matches
        && shape.goal_id_matches
        && shape.goal_proof_matches
        && shape.goal_node_matches
        && shape.citations_empty
        && shape.active_context_empty
        && shape.local_path_matches
        && shape.label_is_none
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StatementBindingShape {
    context_count: usize,
    owner_is_module: bool,
    parent_is_none: bool,
    layer_is_module: bool,
    recovery_is_normal: bool,
    context_bindings_empty: bool,
    visible_bindings_empty: bool,
    binding_table_empty: bool,
    diagnostics_empty: bool,
}

const fn exact_statement_binding_shape(shape: StatementBindingShape) -> bool {
    shape.context_count == 1
        && shape.owner_is_module
        && shape.parent_is_none
        && shape.layer_is_module
        && shape.recovery_is_normal
        && shape.context_bindings_empty
        && shape.visible_bindings_empty
        && shape.binding_table_empty
        && shape.diagnostics_empty
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StatementTreeShape {
    node_count: usize,
    root_kind_matches: bool,
    owner_kind_matches: bool,
    formula_kind_matches: bool,
    root_child_matches: bool,
    owner_child_matches: bool,
    formula_children_empty: bool,
    root_recovery_normal: bool,
    owner_recovery_normal: bool,
    formula_recovery_normal: bool,
    root_typing_successful: bool,
    owner_typing_successful: bool,
    formula_typing_successful: bool,
}

const fn exact_statement_tree_shape(shape: StatementTreeShape) -> bool {
    shape.node_count == 3
        && shape.root_kind_matches
        && shape.owner_kind_matches
        && shape.formula_kind_matches
        && shape.root_child_matches
        && shape.owner_child_matches
        && shape.formula_children_empty
        && shape.root_recovery_normal
        && shape.owner_recovery_normal
        && shape.formula_recovery_normal
        && shape.root_typing_successful
        && shape.owner_typing_successful
        && shape.formula_typing_successful
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StatementRangeShape {
    root_ordered: bool,
    owner_ordered: bool,
    formula_ordered: bool,
    owner_range_matches: bool,
    formula_range_matches: bool,
    formula_recovery_matches: bool,
    root_source_matches: bool,
    owner_source_matches: bool,
    formula_source_matches: bool,
    root_contains_owner: bool,
}

const fn exact_statement_range_shape(shape: StatementRangeShape) -> bool {
    shape.root_ordered
        && shape.owner_ordered
        && shape.formula_ordered
        && shape.owner_range_matches
        && shape.formula_range_matches
        && shape.formula_recovery_matches
        && shape.root_source_matches
        && shape.owner_source_matches
        && shape.formula_source_matches
        && shape.root_contains_owner
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

fn write_statement_semantics(output: &mut String, table: &StatementSemanticTable) {
    output.push_str("statement-semantics:\n");
    for (id, statement) in table.iter() {
        let _ = write!(
            output,
            "  statement#{} owner=\"{}\" owner_node=node#{} formula=formula#{} range=",
            id.index(),
            escaped_display(statement.owner.fqn().as_str()),
            statement.owner_node.index(),
            statement.formula.index(),
        );
        write_range(output, statement.owner_range);
        output.push('\n');
    }
}

fn write_checked_proofs(output: &mut String, table: &CheckedProofTable) {
    output.push_str("checked-proofs:\n");
    for (id, proof) in table.iter() {
        let _ = write!(
            output,
            "  proof#{} source_order={} statement=statement#{} owner=\"{}\" owner_node=node#{} visibility={:?} export={:?} proposition=formula#{} policy={:?} justification={:?} root=proof_node#{} status={:?} range=",
            id.index(),
            proof.source_order,
            proof.statement.index(),
            escaped_display(proof.owner.fqn().as_str()),
            proof.owner_node.index(),
            proof.owner_visibility,
            proof.owner_export_status,
            proof.proposition.index(),
            proof.policy,
            proof.justification,
            proof.root.index(),
            proof.status,
        );
        write_range(output, proof.source_range);
        let _ = writeln!(output, " origin={:?}", proof.owner_origin);
    }
}

fn write_checked_proof_nodes(output: &mut String, table: &CheckedProofNodeTable) {
    output.push_str("checked-proof-nodes:\n");
    for (id, node) in table.iter() {
        let _ = write!(
            output,
            "  proof_node#{} proof=proof#{} kind={:?} range=",
            id.index(),
            node.proof.index(),
            node.kind,
        );
        write_range(output, node.source_range);
        let _ = writeln!(output, " recovery={:?}", node.recovery);
    }
}

fn write_checked_terminal_goals(output: &mut String, table: &CheckedTerminalGoalTable) {
    output.push_str("checked-terminal-goals:\n");
    for (id, goal) in table.iter() {
        let _ = write!(
            output,
            "  terminal_goal#{} proof=proof#{} node=proof_node#{} statement=statement#{} owner=\"{}\" formula=formula#{} formula_site=",
            id.index(),
            goal.proof.index(),
            goal.node.index(),
            goal.statement.index(),
            escaped_display(goal.owner.fqn().as_str()),
            goal.formula.index(),
        );
        write_site_ref(output, &goal.formula_site);
        let _ = write!(
            output,
            " formula_node=node#{} range=",
            goal.formula_node.index()
        );
        write_range(output, goal.source_range);
        let _ = write!(
            output,
            " recovery={:?} citations={:?} active_context=[",
            goal.recovery, goal.citations
        );
        for (index, formula) in goal.active_context.iter().enumerate() {
            if index > 0 {
                output.push_str(", ");
            }
            let _ = write!(output, "formula#{}", formula.index());
        }
        let _ = writeln!(
            output,
            "] local_path=\"{}\" label={:?}",
            escaped_display(&goal.local_path),
            goal.label
        );
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
        binding_env::{
            BindingContextDraft, BindingContextId, BindingContextTable, BindingDiagnosticTable,
            BindingEnvParts, BindingTable,
        },
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
        type_checker::{
            CandidateIdentity, ExpectedTypeInput, FormulaDeferredReason, FormulaFactInput,
            FormulaInput, OpenCandidateInput, StatementPayloadTableForTest, TermFormulaChecker,
            TermInput, TermKind, TypeExpressionInput, TypeHeadInput,
        },
        typed_ast::{
            BuiltinRuleId, CoercionTable, ContextRecoveryState, FactProvenance, FactStatus,
            InitialObligationTable, LocalTypeContextDraft, LocalTypeContextTable,
            OpenCandidateSetId, Polarity, TypeContextLayer, TypeEntryDraft, TypeEntryId,
            TypeFactDraft, TypeFactTable, TypePredicateRef, TypeProvenance, TypeRole, TypeStatus,
            TypeTable, TypedArenaBuilder, TypedAstParts, TypedNode, TypedNodeLinks,
        },
    };
    use mizar_resolve::resolved_ast::{FullyQualifiedName, LocalSymbolId, SemanticOrigin};
    use mizar_session::{
        BuildSnapshotId, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator,
    };

    #[test]
    fn exact_statement_semantic_projection_is_deterministic_and_fail_closed() {
        let fixture = statement_fixture(
            FormulaKind::Contradiction,
            NodeRecoveryState::Normal,
            50,
            60,
            false,
        );
        let row = fixture.row();
        let legacy = assemble_fixture_with_proof_rows(
            &fixture,
            &fixture.checked_owner,
            Vec::new(),
            false,
            false,
        )
        .expect("legacy assembly without statement/proof bundles");
        assert_eq!(
            legacy.debug_text(),
            concat!(
                "resolved-typed-ast-debug-v1\n",
                "module: \"pkg\"::\"main\"\n",
                "root: resolved_node#2\n",
                "nodes:\n",
                "  node#0 typed=node#0 range=source=\"SourceId(OpaqueId(1))\":50..60 ",
                "kind=source_preserved(\"source.formula.contradiction\") final_type=<none> ",
                "metadata=<none> children=[]\n",
                "  node#1 typed=node#1 range=source=\"SourceId(OpaqueId(1))\":10..90 ",
                "kind=source_preserved(\"source.statement.theorem\") final_type=<none> ",
                "metadata=<none> children=[resolved_node#0]\n",
                "  node#2 typed=node#2 range=source=\"SourceId(OpaqueId(1))\":0..100 ",
                "kind=source_preserved(\"source.module\") final_type=<none> ",
                "metadata=<none> children=[resolved_node#1]\n",
                "expression-metadata:\n",
                "collection-candidates:\n",
                "expanded-candidates:\n",
                "template-expansions:\n",
                "viable-candidates:\n",
                "viability-decisions:\n",
                "specificity-graphs:\n",
                "overload-records:\n",
                "inserted-coercions:\n",
                "cluster-facts:\n",
                "diagnostics:\n",
            )
        );
        let first = assemble_statement(&fixture, vec![row.clone()])
            .expect("exact statement semantic projection should assemble");
        let second = assemble_statement(&fixture, vec![row.clone()])
            .expect("equivalent statement semantic projection should assemble");
        assert_eq!(first, second);
        assert_eq!(first.debug_text(), second.debug_text());
        assert_eq!(first.statement_semantics().len(), 1);
        assert_eq!(first.checked_formulas(), fixture.term_formula.formulas());
        let statement = first
            .statement_semantics()
            .get(StatementSemanticId::new(0))
            .expect("statement row");
        assert_eq!(statement.owner, fixture.owner);
        assert_eq!(statement.owner_node, TypedNodeId::new(1));
        assert_eq!(statement.formula, CheckedFormulaId::new(0));
        assert_eq!(statement.formula_node, TypedNodeId::new(0));
        assert_eq!(
            first
                .checked_formulas()
                .get(statement.formula)
                .expect("checked formula")
                .site,
            TypedSiteRef::Node(TypedNodeId::new(27))
        );
        assert!(first.debug_text().contains("statement-semantics:"));

        let proof = first
            .checked_proofs()
            .get(CheckedProofId::new(0))
            .expect("checked proof row");
        let proof_node = first
            .checked_proof_nodes()
            .get(CheckedProofNodeId::new(0))
            .expect("checked proof node");
        let terminal = first
            .checked_terminal_goals()
            .get(CheckedTerminalGoalId::new(0))
            .expect("checked terminal goal");
        assert_eq!(proof.source_order, 0);
        assert_eq!(proof.statement, statement.id);
        assert_eq!(proof.owner, statement.owner);
        assert_eq!(proof.owner_node, statement.owner_node);
        assert_eq!(proof.owner_visibility, Visibility::Public);
        assert_eq!(proof.owner_export_status, ExportStatus::Exported);
        assert_eq!(proof.proposition, statement.formula);
        assert_eq!(proof.policy, TheoremPolicyIntent::Unmodified);
        assert_eq!(proof.justification, TheoremJustificationIntent::Omitted);
        assert_eq!(proof.status, CheckedProofStatus::PendingAutomaticProof);
        assert_eq!(proof.root, proof_node.id);
        assert_eq!(proof_node.proof, proof.id);
        assert_eq!(
            proof_node.kind,
            CheckedProofNodeKind::TerminalGoal(terminal.id)
        );
        assert_eq!(terminal.proof, proof.id);
        assert_eq!(terminal.node, proof_node.id);
        assert_eq!(
            terminal.formula_site,
            TypedSiteRef::Node(TypedNodeId::new(27))
        );
        assert!(terminal.citations.is_empty());
        assert!(terminal.active_context.is_empty());
        assert_eq!(terminal.local_path, "proof/0");
        assert!(terminal.label.is_none());
        assert_eq!(
            first.debug_text(),
            concat!(
                "resolved-typed-ast-debug-v1\n",
                "module: \"pkg\"::\"main\"\n",
                "root: resolved_node#2\n",
                "nodes:\n",
                "  node#0 typed=node#0 range=source=\"SourceId(OpaqueId(1))\":50..60 ",
                "kind=source_preserved(\"source.formula.contradiction\") final_type=<none> ",
                "metadata=<none> children=[]\n",
                "  node#1 typed=node#1 range=source=\"SourceId(OpaqueId(1))\":10..90 ",
                "kind=source_preserved(\"source.statement.theorem\") final_type=<none> ",
                "metadata=<none> children=[resolved_node#0]\n",
                "  node#2 typed=node#2 range=source=\"SourceId(OpaqueId(1))\":0..100 ",
                "kind=source_preserved(\"source.module\") final_type=<none> ",
                "metadata=<none> children=[resolved_node#1]\n",
                "expression-metadata:\n",
                "collection-candidates:\n",
                "expanded-candidates:\n",
                "template-expansions:\n",
                "viable-candidates:\n",
                "viability-decisions:\n",
                "specificity-graphs:\n",
                "overload-records:\n",
                "inserted-coercions:\n",
                "cluster-facts:\n",
                "diagnostics:\n",
                "statement-semantics:\n",
                "  statement#0 owner=\"pkg::main::theorem::Task180\" owner_node=node#1 ",
                "formula=formula#0 range=source=\"SourceId(OpaqueId(1))\":10..90\n",
                "checked-proofs:\n",
                "  proof#0 source_order=0 statement=statement#0 ",
                "owner=\"pkg::main::theorem::Task180\" owner_node=node#1 ",
                "visibility=Public export=Exported proposition=formula#0 ",
                "policy=Unmodified justification=Omitted root=proof_node#0 ",
                "status=PendingAutomaticProof ",
                "range=source=\"SourceId(OpaqueId(1))\":10..90 ",
                "origin=SemanticOrigin { source_id: SourceId(OpaqueId(1)), ",
                "module_id: ModuleId { package: PackageId(\"pkg\"), ",
                "path: ModulePath(\"main\") }, anchor: Range(SourceRange { ",
                "source_id: SourceId(OpaqueId(1)), start: 10, end: 90 }), ",
                "structural_path: [0], import_edge: None, recovered: false }\n",
                "checked-proof-nodes:\n",
                "  proof_node#0 proof=proof#0 ",
                "kind=TerminalGoal(CheckedTerminalGoalId(0)) ",
                "range=source=\"SourceId(OpaqueId(1))\":50..60 recovery=Normal\n",
                "checked-terminal-goals:\n",
                "  terminal_goal#0 proof=proof#0 node=proof_node#0 ",
                "statement=statement#0 owner=\"pkg::main::theorem::Task180\" ",
                "formula=formula#0 formula_site=node#27 formula_node=node#0 ",
                "range=source=\"SourceId(OpaqueId(1))\":50..60 recovery=Normal ",
                "citations=[] active_context=[] local_path=\"proof/0\" label=None\n",
            )
        );

        let missing = assemble_statement(&fixture, Vec::new())
            .expect_err("missing statement row should fail closed");
        assert!(matches!(
            missing,
            ResolvedTypedAstError::MissingStatementSemantic
        ));

        let duplicate = assemble_statement(&fixture, vec![row.clone(), row.clone()])
            .expect_err("duplicate statement row should fail closed");
        assert!(matches!(
            duplicate,
            ResolvedTypedAstError::DuplicateStatementOwner { .. }
        ));

        let invalid_formula = assemble_statement(
            &fixture,
            vec![StatementSemanticInput {
                formula: CheckedFormulaId::new(1),
                ..row.clone()
            }],
        )
        .expect_err("invalid checked formula id should fail closed");
        assert!(matches!(
            invalid_formula,
            ResolvedTypedAstError::InvalidStatementFormula { .. }
        ));

        let wrong_owner_node = assemble_statement(
            &fixture,
            vec![StatementSemanticInput {
                owner_node: TypedNodeId::new(0),
                ..row
            }],
        )
        .expect_err("wrong theorem typed node should fail closed");
        assert!(matches!(
            wrong_owner_node,
            ResolvedTypedAstError::InvalidStatementTree
        ));

        for invalid in [
            statement_fixture(
                FormulaKind::Thesis,
                NodeRecoveryState::Normal,
                50,
                60,
                false,
            ),
            statement_fixture(
                FormulaKind::Contradiction,
                NodeRecoveryState::Recovered,
                50,
                60,
                false,
            ),
            statement_fixture(
                FormulaKind::Contradiction,
                NodeRecoveryState::Normal,
                50,
                60,
                true,
            ),
            statement_fixture(
                FormulaKind::Contradiction,
                NodeRecoveryState::Normal,
                5,
                6,
                false,
            ),
        ] {
            assert!(
                assemble_statement(&invalid, vec![invalid.row()]).is_err(),
                "invalid kind/recovery/status/order/provenance must fail closed"
            );
        }
    }

    #[test]
    fn exact_statement_proof_intent_and_output_fail_closed_atomically() {
        let fixture = statement_fixture(
            FormulaKind::Contradiction,
            NodeRecoveryState::Normal,
            50,
            60,
            false,
        );
        let valid = fixture.proof_row();

        for (include_statement, include_proof) in [(true, false), (false, true)] {
            let error = assemble_fixture_with_proof_rows(
                &fixture,
                &fixture.checked_owner,
                vec![valid.clone()],
                include_statement,
                include_proof,
            )
            .expect_err("asymmetric statement/proof bundles must fail closed");
            assert!(matches!(
                error,
                ResolvedTypedAstError::StatementProofBundleMismatch
            ));
        }

        let missing = assemble_fixture_with_proof_rows(
            &fixture,
            &fixture.checked_owner,
            Vec::new(),
            true,
            true,
        )
        .expect_err("missing proof intent must fail closed");
        assert!(matches!(
            missing,
            ResolvedTypedAstError::MissingStatementProofIntent
        ));
        let duplicate = assemble_fixture_with_proof_rows(
            &fixture,
            &fixture.checked_owner,
            vec![valid.clone(), valid.clone()],
            true,
            true,
        )
        .expect_err("duplicate proof intent must fail closed");
        assert!(matches!(
            duplicate,
            ResolvedTypedAstError::DuplicateStatementProofIntent { .. }
        ));
        let mut second = valid.clone();
        second.id = StatementProofIntentId::new(1);
        let non_singleton = assemble_fixture_with_proof_rows(
            &fixture,
            &fixture.checked_owner,
            vec![valid.clone(), second],
            true,
            true,
        )
        .expect_err("multiple proof intents must fail closed");
        assert!(matches!(
            non_singleton,
            ResolvedTypedAstError::NonSingletonStatementProofIntent { count: 2 }
        ));

        let other_owner = SymbolId::new(
            fixture.module.clone(),
            LocalSymbolId::new("OtherTask268"),
            FullyQualifiedName::new("pkg::main::theorem::OtherTask268"),
        );
        let other_origin = SemanticOrigin::new(
            fixture.source,
            fixture.module.clone(),
            SourceAnchor::Range(fixture.owner_range),
            vec![1],
        );
        let other_module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("other"));
        let other_source = secondary_source_id(93);
        let invalid_rows = vec![
            StatementProofIntentInput {
                id: StatementProofIntentId::new(1),
                ..valid.clone()
            },
            StatementProofIntentInput {
                source_order: 1,
                ..valid.clone()
            },
            StatementProofIntentInput {
                statement: StatementSemanticId::new(1),
                ..valid.clone()
            },
            StatementProofIntentInput {
                source_id: other_source,
                ..valid.clone()
            },
            StatementProofIntentInput {
                module_id: other_module,
                ..valid.clone()
            },
            StatementProofIntentInput {
                owner: other_owner,
                ..valid.clone()
            },
            StatementProofIntentInput {
                owner_node: TypedNodeId::new(0),
                ..valid.clone()
            },
            StatementProofIntentInput {
                owner_range: range(fixture.source, 11, 90),
                ..valid.clone()
            },
            StatementProofIntentInput {
                owner_origin: other_origin,
                ..valid.clone()
            },
            StatementProofIntentInput {
                owner_visibility: Visibility::Private,
                ..valid.clone()
            },
            StatementProofIntentInput {
                owner_export_status: ExportStatus::LocalOnly,
                ..valid.clone()
            },
            StatementProofIntentInput {
                formula: CheckedFormulaId::new(1),
                ..valid.clone()
            },
            StatementProofIntentInput {
                formula_site: TypedSiteRef::Role {
                    node: TypedNodeId::new(27),
                    role: TypeRole::new("task268.corrupt"),
                },
                ..valid.clone()
            },
            StatementProofIntentInput {
                formula_node: TypedNodeId::new(1),
                ..valid.clone()
            },
            StatementProofIntentInput {
                formula_range: range(fixture.source, 51, 60),
                ..valid.clone()
            },
            StatementProofIntentInput {
                recovery: NodeRecoveryState::Recovered,
                ..valid.clone()
            },
        ];
        for invalid in invalid_rows {
            let error = assemble_fixture_with_proof_rows(
                &fixture,
                &fixture.checked_owner,
                vec![invalid],
                true,
                true,
            )
            .expect_err("corrupt proof intent must fail closed");
            assert!(matches!(
                error,
                ResolvedTypedAstError::InvalidStatementProofIntent
            ));
        }

        for (visibility, export_status) in [
            (Visibility::Private, ExportStatus::Exported),
            (Visibility::Public, ExportStatus::LocalOnly),
        ] {
            let corrupt_owner = CheckedStatementOwner::from_validated_parts_and_visibility_for_test(
                fixture.owner.clone(),
                fixture.owner_range,
                fixture.checked_owner.origin().clone(),
                visibility,
                export_status,
            );
            let error = assemble_fixture_with_proof_rows(
                &fixture,
                &corrupt_owner,
                vec![valid.clone()],
                true,
                true,
            )
            .expect_err("authenticated-owner visibility corruption must fail closed");
            assert!(matches!(
                error,
                ResolvedTypedAstError::InvalidStatementProofIntent
            ));
        }

        let output_shape = CheckedProofOutputShape {
            proof_count: 1,
            node_count: 1,
            goal_count: 1,
            proof_id_matches: true,
            proof_root_matches: true,
            proof_status_matches: true,
            node_id_matches: true,
            node_proof_matches: true,
            node_kind_matches: true,
            goal_id_matches: true,
            goal_proof_matches: true,
            goal_node_matches: true,
            citations_empty: true,
            active_context_empty: true,
            local_path_matches: true,
            label_is_none: true,
        };
        assert!(exact_checked_proof_output_shape(output_shape));
        for invalid in [
            CheckedProofOutputShape {
                proof_count: 2,
                ..output_shape
            },
            CheckedProofOutputShape {
                node_count: 2,
                ..output_shape
            },
            CheckedProofOutputShape {
                goal_count: 2,
                ..output_shape
            },
            CheckedProofOutputShape {
                proof_id_matches: false,
                ..output_shape
            },
            CheckedProofOutputShape {
                proof_root_matches: false,
                ..output_shape
            },
            CheckedProofOutputShape {
                proof_status_matches: false,
                ..output_shape
            },
            CheckedProofOutputShape {
                node_id_matches: false,
                ..output_shape
            },
            CheckedProofOutputShape {
                node_proof_matches: false,
                ..output_shape
            },
            CheckedProofOutputShape {
                node_kind_matches: false,
                ..output_shape
            },
            CheckedProofOutputShape {
                goal_id_matches: false,
                ..output_shape
            },
            CheckedProofOutputShape {
                goal_proof_matches: false,
                ..output_shape
            },
            CheckedProofOutputShape {
                goal_node_matches: false,
                ..output_shape
            },
            CheckedProofOutputShape {
                citations_empty: false,
                ..output_shape
            },
            CheckedProofOutputShape {
                active_context_empty: false,
                ..output_shape
            },
            CheckedProofOutputShape {
                local_path_matches: false,
                ..output_shape
            },
            CheckedProofOutputShape {
                label_is_none: false,
                ..output_shape
            },
        ] {
            assert!(
                !exact_checked_proof_output_shape(invalid),
                "every checked-proof output-shape predicate must fail closed independently"
            );
        }

        let resolved = assemble_statement(&fixture, vec![fixture.row()])
            .expect("valid proof output for postvalidation corruption");
        let reject = |proofs: CheckedProofTable,
                      nodes: CheckedProofNodeTable,
                      goals: CheckedTerminalGoalTable| {
            assert!(matches!(
                validate_checked_proof_tables(
                    resolved.checked_formulas(),
                    resolved.statement_semantics(),
                    &proofs,
                    &nodes,
                    &goals,
                ),
                Err(ResolvedTypedAstError::InvalidStatementProofOutput)
            ));
        };
        macro_rules! reject_proof_mutation {
            ($mutation:expr) => {{
                let mut proofs = resolved.checked_proofs().clone();
                ($mutation)(&mut proofs.entries[0]);
                reject(
                    proofs,
                    resolved.checked_proof_nodes().clone(),
                    resolved.checked_terminal_goals().clone(),
                );
            }};
        }
        macro_rules! reject_node_mutation {
            ($mutation:expr) => {{
                let mut nodes = resolved.checked_proof_nodes().clone();
                ($mutation)(&mut nodes.entries[0]);
                reject(
                    resolved.checked_proofs().clone(),
                    nodes,
                    resolved.checked_terminal_goals().clone(),
                );
            }};
        }
        macro_rules! reject_goal_mutation {
            ($mutation:expr) => {{
                let mut goals = resolved.checked_terminal_goals().clone();
                ($mutation)(&mut goals.entries[0]);
                reject(
                    resolved.checked_proofs().clone(),
                    resolved.checked_proof_nodes().clone(),
                    goals,
                );
            }};
        }

        reject_proof_mutation!(|proof: &mut CheckedProof| proof.id = CheckedProofId::new(1));
        reject_proof_mutation!(|proof: &mut CheckedProof| proof.source_order = 1);
        reject_proof_mutation!(
            |proof: &mut CheckedProof| proof.statement = StatementSemanticId::new(1)
        );
        reject_proof_mutation!(
            |proof: &mut CheckedProof| proof.owner = symbol("task268-output-corrupt")
        );
        reject_proof_mutation!(|proof: &mut CheckedProof| proof.owner_node = TypedNodeId::new(0));
        reject_proof_mutation!(
            |proof: &mut CheckedProof| proof.owner_visibility = Visibility::Private
        );
        reject_proof_mutation!(
            |proof: &mut CheckedProof| proof.owner_export_status = ExportStatus::LocalOnly
        );
        reject_proof_mutation!(
            |proof: &mut CheckedProof| proof.proposition = CheckedFormulaId::new(1)
        );
        reject_proof_mutation!(|proof: &mut CheckedProof| proof.root = CheckedProofNodeId::new(1));
        reject_proof_mutation!(
            |proof: &mut CheckedProof| proof.source_range = range(fixture.source, 11, 90)
        );
        reject_proof_mutation!(
            |proof: &mut CheckedProof| proof.owner_origin = proof.owner_origin.clone().recovered()
        );

        reject_node_mutation!(|node: &mut CheckedProofNode| node.id = CheckedProofNodeId::new(1));
        reject_node_mutation!(|node: &mut CheckedProofNode| node.proof = CheckedProofId::new(1));
        reject_node_mutation!(|node: &mut CheckedProofNode| node.kind =
            CheckedProofNodeKind::TerminalGoal(CheckedTerminalGoalId::new(1)));
        reject_node_mutation!(
            |node: &mut CheckedProofNode| node.source_range = range(fixture.source, 51, 60)
        );
        reject_node_mutation!(
            |node: &mut CheckedProofNode| node.recovery = NodeRecoveryState::Recovered
        );

        reject_goal_mutation!(
            |goal: &mut CheckedTerminalGoal| goal.id = CheckedTerminalGoalId::new(1)
        );
        reject_goal_mutation!(|goal: &mut CheckedTerminalGoal| goal.proof = CheckedProofId::new(1));
        reject_goal_mutation!(
            |goal: &mut CheckedTerminalGoal| goal.node = CheckedProofNodeId::new(1)
        );
        reject_goal_mutation!(
            |goal: &mut CheckedTerminalGoal| goal.statement = StatementSemanticId::new(1)
        );
        reject_goal_mutation!(
            |goal: &mut CheckedTerminalGoal| goal.owner = symbol("task268-goal-corrupt")
        );
        reject_goal_mutation!(
            |goal: &mut CheckedTerminalGoal| goal.formula = CheckedFormulaId::new(1)
        );
        reject_goal_mutation!(|goal: &mut CheckedTerminalGoal| goal.formula_site =
            TypedSiteRef::Role {
                node: TypedNodeId::new(27),
                role: TypeRole::new("task268.output-corrupt"),
            });
        reject_goal_mutation!(
            |goal: &mut CheckedTerminalGoal| goal.formula_node = TypedNodeId::new(1)
        );
        reject_goal_mutation!(
            |goal: &mut CheckedTerminalGoal| goal.source_range = range(fixture.source, 51, 60)
        );
        reject_goal_mutation!(
            |goal: &mut CheckedTerminalGoal| goal.recovery = NodeRecoveryState::Recovered
        );
        reject_goal_mutation!(|goal: &mut CheckedTerminalGoal| goal
            .active_context
            .push(CheckedFormulaId::new(0)));
        reject_goal_mutation!(
            |goal: &mut CheckedTerminalGoal| goal.local_path = "proof/1".to_owned()
        );

        let mut proofs = resolved.checked_proofs().clone();
        proofs.entries.push(proofs.entries[0].clone());
        reject(
            proofs,
            resolved.checked_proof_nodes().clone(),
            resolved.checked_terminal_goals().clone(),
        );
        let mut nodes = resolved.checked_proof_nodes().clone();
        nodes.entries.push(nodes.entries[0].clone());
        reject(
            resolved.checked_proofs().clone(),
            nodes,
            resolved.checked_terminal_goals().clone(),
        );
        let mut goals = resolved.checked_terminal_goals().clone();
        goals.entries.push(goals.entries[0].clone());
        reject(
            resolved.checked_proofs().clone(),
            resolved.checked_proof_nodes().clone(),
            goals,
        );
        reject(
            CheckedProofTable::default(),
            resolved.checked_proof_nodes().clone(),
            resolved.checked_terminal_goals().clone(),
        );
        reject(
            resolved.checked_proofs().clone(),
            CheckedProofNodeTable::default(),
            resolved.checked_terminal_goals().clone(),
        );
        reject(
            resolved.checked_proofs().clone(),
            resolved.checked_proof_nodes().clone(),
            CheckedTerminalGoalTable::default(),
        );
        assert!(matches!(
            validate_checked_proof_tables_with_status_match(
                resolved.checked_formulas(),
                resolved.statement_semantics(),
                resolved.checked_proofs(),
                resolved.checked_proof_nodes(),
                resolved.checked_terminal_goals(),
                false,
            ),
            Err(ResolvedTypedAstError::InvalidStatementProofOutput)
        ));
    }

    #[test]
    fn statement_semantic_projection_rejects_every_validation_family_and_owns_output() {
        let fixture = statement_fixture(
            FormulaKind::Contradiction,
            NodeRecoveryState::Normal,
            50,
            60,
            false,
        );
        let row = fixture.row();
        let other_owner = SymbolId::new(
            fixture.module.clone(),
            LocalSymbolId::new("OtherTask180"),
            FullyQualifiedName::new("pkg::main::theorem::OtherTask180"),
        );

        let duplicate_formula = assemble_statement(
            &fixture,
            vec![
                row.clone(),
                StatementSemanticInput {
                    owner: other_owner.clone(),
                    ..row.clone()
                },
            ],
        )
        .expect_err("duplicate formula identity should fail closed");
        assert!(matches!(
            duplicate_formula,
            ResolvedTypedAstError::DuplicateStatementFormula { .. }
        ));

        let non_singleton = assemble_statement(
            &fixture,
            vec![
                row.clone(),
                StatementSemanticInput {
                    owner: other_owner.clone(),
                    formula: CheckedFormulaId::new(1),
                    ..row.clone()
                },
            ],
        )
        .expect_err("two unique rows should fail the singleton contract");
        assert!(matches!(
            non_singleton,
            ResolvedTypedAstError::NonSingletonStatementSemantic { count: 2 }
        ));

        let other_module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("other"));
        let mismatched_bindings = statement_binding_env(
            fixture.source,
            other_module.clone(),
            BindingContextRecovery::Normal,
        );
        let environment = assemble_statement_with(
            &fixture.typed_ast,
            &fixture.checked_owner,
            &mismatched_bindings,
            &fixture.term_formula,
            vec![row.clone()],
            statement_node_hints(),
        )
        .expect_err("binding module mismatch should fail closed");
        assert!(matches!(
            environment,
            ResolvedTypedAstError::StatementEnvironmentMismatch
        ));
        let mismatched_source_bindings = statement_binding_env(
            secondary_source_id(91),
            fixture.module.clone(),
            BindingContextRecovery::Normal,
        );
        let source_environment = assemble_statement_with(
            &fixture.typed_ast,
            &fixture.checked_owner,
            &mismatched_source_bindings,
            &fixture.term_formula,
            vec![row.clone()],
            statement_node_hints(),
        )
        .expect_err("binding source mismatch should fail closed");
        assert!(matches!(
            source_environment,
            ResolvedTypedAstError::StatementEnvironmentMismatch
        ));

        let other_module_formula = TermFormulaChecker::default().infer_without_symbols_for_test(
            &statement_binding_env(
                fixture.source,
                other_module.clone(),
                BindingContextRecovery::Normal,
            ),
            [FormulaInput::new(
                TypedSiteRef::Node(TypedNodeId::new(27)),
                fixture.context,
                fixture.formula_range,
                FormulaKind::Contradiction,
            )],
        );
        let term_module_environment = assemble_statement_with(
            &fixture.typed_ast,
            &fixture.checked_owner,
            &fixture.binding_env,
            &other_module_formula,
            vec![row.clone()],
            statement_node_hints(),
        )
        .expect_err("term/formula module mismatch should fail closed");
        assert!(matches!(
            term_module_environment,
            ResolvedTypedAstError::StatementEnvironmentMismatch
        ));
        let other_source = secondary_source_id(92);
        let other_source_formula = TermFormulaChecker::default().infer_without_symbols_for_test(
            &statement_binding_env(
                other_source,
                fixture.module.clone(),
                BindingContextRecovery::Normal,
            ),
            [FormulaInput::new(
                TypedSiteRef::Node(TypedNodeId::new(27)),
                fixture.context,
                range(other_source, 50, 60),
                FormulaKind::Contradiction,
            )],
        );
        let term_source_environment = assemble_statement_with(
            &fixture.typed_ast,
            &fixture.checked_owner,
            &fixture.binding_env,
            &other_source_formula,
            vec![row.clone()],
            statement_node_hints(),
        )
        .expect_err("term/formula source mismatch should fail closed");
        assert!(matches!(
            term_source_environment,
            ResolvedTypedAstError::StatementEnvironmentMismatch
        ));

        let wrong_owner = CheckedStatementOwner::from_validated_parts_for_test(
            other_owner,
            fixture.owner_range,
            SemanticOrigin::new(
                fixture.source,
                fixture.module.clone(),
                SourceAnchor::Range(fixture.owner_range),
                vec![0],
            ),
        );
        let owner_error = assemble_statement_with(
            &fixture.typed_ast,
            &wrong_owner,
            &fixture.binding_env,
            &fixture.term_formula,
            vec![row.clone()],
            statement_node_hints(),
        )
        .expect_err("checked owner mismatch should fail closed");
        assert!(matches!(
            owner_error,
            ResolvedTypedAstError::InvalidStatementOwner { .. }
        ));

        let formula_input = FormulaInput::new(
            TypedSiteRef::Node(TypedNodeId::new(27)),
            fixture.context,
            fixture.formula_range,
            FormulaKind::Contradiction,
        );
        let duplicate_formula_payload = fixture
            .term_formula
            .clone()
            .duplicate_formula_for_statement_test();
        let formula_count_error = assemble_statement_with(
            &fixture.typed_ast,
            &fixture.checked_owner,
            &fixture.binding_env,
            &duplicate_formula_payload,
            vec![row.clone()],
            statement_node_hints(),
        )
        .expect_err("non-singleton formula table should fail closed");
        assert!(matches!(
            formula_count_error,
            ResolvedTypedAstError::InvalidStatementFormulaPayload { .. }
        ));

        let term_payload = TermFormulaChecker::default().infer_inputs_without_symbols_for_test(
            &fixture.binding_env,
            [TermInput::new(
                TypedSiteRef::Node(TypedNodeId::new(28)),
                fixture.context,
                range(fixture.source, 40, 41),
                TermKind::Unsupported,
            )],
            [formula_input.clone()],
        );
        let top_level_payload = assemble_statement_with(
            &fixture.typed_ast,
            &fixture.checked_owner,
            &fixture.binding_env,
            &term_payload,
            vec![row.clone()],
            statement_node_hints(),
        )
        .expect_err("forbidden top-level term payload should fail closed");
        assert!(matches!(
            top_level_payload,
            ResolvedTypedAstError::InvalidStatementFormulaPayload { .. }
        ));

        let type_expression = TypeExpressionInput::new(
            TypedSiteRef::Node(TypedNodeId::new(29)),
            range(fixture.source, 61, 64),
            "set",
            TypeHeadInput::BuiltinSet,
        );
        let typed_formula = TermFormulaChecker::default().infer_without_symbols_for_test(
            &fixture.binding_env,
            [formula_input
                .clone()
                .with_asserted_type(type_expression.clone())],
        );
        let candidate_formula = TermFormulaChecker::default().infer_without_symbols_for_test(
            &fixture.binding_env,
            [formula_input
                .clone()
                .with_candidates(vec![OpenCandidateInput::new(
                    CandidateIdentity::Builtin("statement-test-predicate".to_owned()),
                    fixture.formula_range,
                )])],
        );
        let fact_formula = TermFormulaChecker::default().infer_without_symbols_for_test(
            &fixture.binding_env,
            [formula_input.clone().with_facts(vec![FormulaFactInput::new(
                TypedSiteRef::Node(TypedNodeId::new(27)),
                TypePredicateRef::new("statement-test-fact"),
                Polarity::Positive,
                fixture.formula_range,
            )])],
        );
        let diagnostic_formula = TermFormulaChecker::default().infer_without_symbols_for_test(
            &fixture.binding_env,
            [formula_input
                .clone()
                .with_deferred(vec![FormulaDeferredReason::MissingFormulaPayload])],
        );
        for (table, payload) in [
            (
                StatementPayloadTableForTest::NormalizedTypes,
                typed_formula.clone(),
            ),
            (StatementPayloadTableForTest::Terms, term_payload.clone()),
            (
                StatementPayloadTableForTest::CandidateSets,
                candidate_formula.clone(),
            ),
            (
                StatementPayloadTableForTest::TypeEntries,
                typed_formula.clone(),
            ),
            (StatementPayloadTableForTest::Facts, fact_formula.clone()),
            (
                StatementPayloadTableForTest::Diagnostics,
                diagnostic_formula.clone(),
            ),
        ] {
            let isolated = payload.retain_statement_payload_table_for_test(table);
            let error = assemble_statement_with(
                &fixture.typed_ast,
                &fixture.checked_owner,
                &fixture.binding_env,
                &isolated,
                vec![row.clone()],
                statement_node_hints(),
            )
            .expect_err("each forbidden top-level table should fail closed independently");
            assert!(matches!(
                error,
                ResolvedTypedAstError::InvalidStatementFormulaPayload { .. }
            ));
        }

        let formula_payload = TermFormulaChecker::default().infer_without_symbols_for_test(
            &fixture.binding_env,
            [formula_input
                .clone()
                .with_terms(vec![TypedSiteRef::Node(TypedNodeId::new(28))])],
        );
        let formula_payload_error = assemble_statement_with(
            &fixture.typed_ast,
            &fixture.checked_owner,
            &fixture.binding_env,
            &formula_payload,
            vec![row.clone()],
            statement_node_hints(),
        )
        .expect_err("forbidden formula child payload should fail closed");
        assert!(matches!(
            formula_payload_error,
            ResolvedTypedAstError::InvalidStatementFormulaPayload { .. }
        ));

        let expected_formula = TermFormulaChecker::default().infer_without_symbols_for_test(
            &fixture.binding_env,
            [formula_input
                .clone()
                .with_expected_types(vec![ExpectedTypeInput::new(
                    TypedSiteRef::Node(TypedNodeId::new(28)),
                    type_expression,
                    fixture.formula_range,
                )])],
        );
        for payload in [
            typed_formula.clear_top_level_statement_payload_for_test(),
            expected_formula.clear_top_level_statement_payload_for_test(),
            candidate_formula.clear_top_level_statement_payload_for_test(),
            fact_formula.clear_top_level_statement_payload_for_test(),
            diagnostic_formula.clear_top_level_statement_payload_for_test(),
        ] {
            let error = assemble_statement_with(
                &fixture.typed_ast,
                &fixture.checked_owner,
                &fixture.binding_env,
                &payload,
                vec![row.clone()],
                statement_node_hints(),
            )
            .expect_err("each forbidden checked-formula child should fail closed independently");
            assert!(matches!(
                error,
                ResolvedTypedAstError::InvalidStatementFormulaPayload { .. }
            ));
        }

        let invalid_formula_context = fixture
            .term_formula
            .clone()
            .set_formula_context_for_statement_test(BindingContextId::new(99));
        let invalid_context_id = assemble_statement_with(
            &fixture.typed_ast,
            &fixture.checked_owner,
            &fixture.binding_env,
            &invalid_formula_context,
            vec![row.clone()],
            statement_node_hints(),
        )
        .expect_err("missing formula binding context should fail closed");
        assert!(matches!(
            invalid_context_id,
            ResolvedTypedAstError::InvalidStatementFormula { .. }
        ));

        let role_formula = TermFormulaChecker::default().infer_without_symbols_for_test(
            &fixture.binding_env,
            [FormulaInput::new(
                TypedSiteRef::Role {
                    node: TypedNodeId::new(27),
                    role: TypeRole::new("source-formula"),
                },
                fixture.context,
                fixture.formula_range,
                FormulaKind::Contradiction,
            )],
        );
        let role_site = assemble_statement_with(
            &fixture.typed_ast,
            &fixture.checked_owner,
            &fixture.binding_env,
            &role_formula,
            vec![row.clone()],
            statement_node_hints(),
        )
        .expect_err("non-node checked formula site should fail closed");
        assert!(matches!(
            role_site,
            ResolvedTypedAstError::InvalidStatementTree
        ));

        let recovered_bindings = statement_binding_env(
            fixture.source,
            fixture.module.clone(),
            BindingContextRecovery::Recovered,
        );
        let context_error = assemble_statement_with(
            &fixture.typed_ast,
            &fixture.checked_owner,
            &recovered_bindings,
            &fixture.term_formula,
            vec![row.clone()],
            statement_node_hints(),
        )
        .expect_err("recovered binding context should fail closed");
        assert!(matches!(
            context_error,
            ResolvedTypedAstError::InvalidStatementFormulaPayload { .. }
        ));

        let binding_shape = StatementBindingShape {
            context_count: 1,
            owner_is_module: true,
            parent_is_none: true,
            layer_is_module: true,
            recovery_is_normal: true,
            context_bindings_empty: true,
            visible_bindings_empty: true,
            binding_table_empty: true,
            diagnostics_empty: true,
        };
        assert!(exact_statement_binding_shape(binding_shape));
        for invalid in [
            StatementBindingShape {
                context_count: 2,
                ..binding_shape
            },
            StatementBindingShape {
                owner_is_module: false,
                ..binding_shape
            },
            StatementBindingShape {
                parent_is_none: false,
                ..binding_shape
            },
            StatementBindingShape {
                layer_is_module: false,
                ..binding_shape
            },
            StatementBindingShape {
                recovery_is_normal: false,
                ..binding_shape
            },
            StatementBindingShape {
                context_bindings_empty: false,
                ..binding_shape
            },
            StatementBindingShape {
                visible_bindings_empty: false,
                ..binding_shape
            },
            StatementBindingShape {
                binding_table_empty: false,
                ..binding_shape
            },
            StatementBindingShape {
                diagnostics_empty: false,
                ..binding_shape
            },
        ] {
            assert!(
                !exact_statement_binding_shape(invalid),
                "every exact binding-context predicate must fail closed independently"
            );
        }

        let tree_shape = StatementTreeShape {
            node_count: 3,
            root_kind_matches: true,
            owner_kind_matches: true,
            formula_kind_matches: true,
            root_child_matches: true,
            owner_child_matches: true,
            formula_children_empty: true,
            root_recovery_normal: true,
            owner_recovery_normal: true,
            formula_recovery_normal: true,
            root_typing_successful: true,
            owner_typing_successful: true,
            formula_typing_successful: true,
        };
        assert!(exact_statement_tree_shape(tree_shape));
        for invalid in [
            StatementTreeShape {
                node_count: 4,
                ..tree_shape
            },
            StatementTreeShape {
                root_kind_matches: false,
                ..tree_shape
            },
            StatementTreeShape {
                owner_kind_matches: false,
                ..tree_shape
            },
            StatementTreeShape {
                formula_kind_matches: false,
                ..tree_shape
            },
            StatementTreeShape {
                root_child_matches: false,
                ..tree_shape
            },
            StatementTreeShape {
                owner_child_matches: false,
                ..tree_shape
            },
            StatementTreeShape {
                formula_children_empty: false,
                ..tree_shape
            },
            StatementTreeShape {
                root_recovery_normal: false,
                ..tree_shape
            },
            StatementTreeShape {
                owner_recovery_normal: false,
                ..tree_shape
            },
            StatementTreeShape {
                formula_recovery_normal: false,
                ..tree_shape
            },
            StatementTreeShape {
                root_typing_successful: false,
                ..tree_shape
            },
            StatementTreeShape {
                owner_typing_successful: false,
                ..tree_shape
            },
            StatementTreeShape {
                formula_typing_successful: false,
                ..tree_shape
            },
        ] {
            assert!(
                !exact_statement_tree_shape(invalid),
                "every exact compact-tree predicate must fail closed independently"
            );
        }

        let range_shape = StatementRangeShape {
            root_ordered: true,
            owner_ordered: true,
            formula_ordered: true,
            owner_range_matches: true,
            formula_range_matches: true,
            formula_recovery_matches: true,
            root_source_matches: true,
            owner_source_matches: true,
            formula_source_matches: true,
            root_contains_owner: true,
        };
        assert!(exact_statement_range_shape(range_shape));
        for invalid in [
            StatementRangeShape {
                root_ordered: false,
                ..range_shape
            },
            StatementRangeShape {
                owner_ordered: false,
                ..range_shape
            },
            StatementRangeShape {
                formula_ordered: false,
                ..range_shape
            },
            StatementRangeShape {
                owner_range_matches: false,
                ..range_shape
            },
            StatementRangeShape {
                formula_range_matches: false,
                ..range_shape
            },
            StatementRangeShape {
                formula_recovery_matches: false,
                ..range_shape
            },
            StatementRangeShape {
                root_source_matches: false,
                ..range_shape
            },
            StatementRangeShape {
                owner_source_matches: false,
                ..range_shape
            },
            StatementRangeShape {
                formula_source_matches: false,
                ..range_shape
            },
            StatementRangeShape {
                root_contains_owner: false,
                ..range_shape
            },
        ] {
            assert!(
                !exact_statement_range_shape(invalid),
                "every exact compact-tree range predicate must fail closed independently"
            );
        }

        let wrong_formula_node = assemble_statement(
            &fixture,
            vec![StatementSemanticInput {
                formula_node: TypedNodeId::new(1),
                ..row.clone()
            }],
        )
        .expect_err("wrong formula typed node should fail closed");
        assert!(matches!(
            wrong_formula_node,
            ResolvedTypedAstError::InvalidStatementTree
        ));

        for corruption in [
            StatementTreeCorruption::BrokenEdge,
            StatementTreeCorruption::RecoveredOwner,
            StatementTreeCorruption::PartialOwner,
            StatementTreeCorruption::PointOwnerAnchor,
            StatementTreeCorruption::PointFormulaAnchor,
            StatementTreeCorruption::PointRootAnchor,
            StatementTreeCorruption::FormulaRangeMismatch,
            StatementTreeCorruption::RootContainment,
            StatementTreeCorruption::SwappedKinds,
        ] {
            let typed_ast = corrupted_statement_typed_ast(&fixture, corruption);
            let error = assemble_statement_with(
                &typed_ast,
                &fixture.checked_owner,
                &fixture.binding_env,
                &fixture.term_formula,
                vec![row.clone()],
                statement_node_hints(),
            )
            .expect_err("typed tree corruption should fail closed");
            assert!(matches!(error, ResolvedTypedAstError::InvalidStatementTree));
        }

        let mut wrong_hints = statement_node_hints();
        wrong_hints.swap(0, 2);
        wrong_hints[0].typed_node = TypedNodeId::new(0);
        wrong_hints[2].typed_node = TypedNodeId::new(2);
        let hint_error = assemble_statement_with(
            &fixture.typed_ast,
            &fixture.checked_owner,
            &fixture.binding_env,
            &fixture.term_formula,
            vec![row.clone()],
            wrong_hints,
        )
        .expect_err("swapped final-tree roles should fail closed");
        assert!(matches!(
            hint_error,
            ResolvedTypedAstError::InvalidStatementTree
        ));

        let mut duplicate_hints = statement_node_hints();
        duplicate_hints.push(duplicate_hints[0].clone());
        let mut non_source_hints = statement_node_hints();
        non_source_hints[0].kind = ResolvedNodeKindHintKind::Degraded {
            reason: ResolvedNodeRecoveryReason::TypingState(TypingState::Error),
        };
        let hints = statement_node_hints();
        for invalid_hints in [
            Vec::new(),
            hints[..2].to_vec(),
            duplicate_hints,
            non_source_hints,
        ] {
            let error = assemble_statement_with(
                &fixture.typed_ast,
                &fixture.checked_owner,
                &fixture.binding_env,
                &fixture.term_formula,
                vec![row.clone()],
                invalid_hints,
            )
            .expect_err("missing, duplicate, or non-source hints should fail closed");
            assert!(matches!(error, ResolvedTypedAstError::InvalidStatementTree));
        }

        for invalid_range in [(60, 50), (10, 60), (50, 90)] {
            let invalid = statement_fixture(
                FormulaKind::Contradiction,
                NodeRecoveryState::Normal,
                invalid_range.0,
                invalid_range.1,
                false,
            );
            assert!(
                assemble_statement(&invalid, vec![invalid.row()]).is_err(),
                "inverted or boundary-equal formula range must fail closed"
            );
        }

        let mut mutable_fixture = statement_fixture(
            FormulaKind::Contradiction,
            NodeRecoveryState::Normal,
            50,
            60,
            false,
        );
        let owned = assemble_statement(&mutable_fixture, vec![mutable_fixture.row()])
            .expect("owned statement projection");
        mutable_fixture.term_formula = TermFormulaChecker::default()
            .infer_without_symbols_for_test(
                &mutable_fixture.binding_env,
                [FormulaInput::new(
                    TypedSiteRef::Node(TypedNodeId::new(27)),
                    mutable_fixture.context,
                    mutable_fixture.formula_range,
                    FormulaKind::Thesis,
                )],
            );
        assert_eq!(
            owned
                .checked_formulas()
                .get(CheckedFormulaId::new(0))
                .expect("owned checked formula")
                .kind,
            FormulaKind::Contradiction
        );
        assert_eq!(owned.statement_semantics().len(), 1);
    }

    struct StatementFixture {
        source: SourceId,
        module: ModuleId,
        context: BindingContextId,
        owner_range: SourceRange,
        formula_range: SourceRange,
        typed_ast: TypedAst,
        binding_env: BindingEnv,
        term_formula: TermFormulaInferenceOutput,
        owner: SymbolId,
        checked_owner: CheckedStatementOwner,
    }

    impl StatementFixture {
        fn row(&self) -> StatementSemanticInput {
            StatementSemanticInput {
                owner: self.owner.clone(),
                owner_node: TypedNodeId::new(1),
                formula: CheckedFormulaId::new(0),
                formula_node: TypedNodeId::new(0),
            }
        }

        fn proof_row(&self) -> StatementProofIntentInput {
            let formula = self
                .term_formula
                .formulas()
                .get(CheckedFormulaId::new(0))
                .expect("statement fixture checked formula");
            StatementProofIntentInput {
                id: StatementProofIntentId::new(0),
                source_order: 0,
                statement: StatementSemanticId::new(0),
                source_id: self.source,
                module_id: self.module.clone(),
                owner: self.owner.clone(),
                owner_node: TypedNodeId::new(1),
                owner_range: self.owner_range,
                owner_origin: self.checked_owner.origin().clone(),
                owner_visibility: self.checked_owner.visibility(),
                owner_export_status: self.checked_owner.export_status(),
                formula: CheckedFormulaId::new(0),
                formula_site: formula.site.clone(),
                formula_node: TypedNodeId::new(0),
                formula_range: self.formula_range,
                recovery: formula.recovery,
                policy: TheoremPolicyIntent::Unmodified,
                justification: TheoremJustificationIntent::Omitted,
            }
        }
    }

    fn statement_fixture(
        formula_kind: FormulaKind,
        formula_recovery: NodeRecoveryState,
        formula_start: usize,
        formula_end: usize,
        deferred: bool,
    ) -> StatementFixture {
        let source = source_id(90);
        let module = module();
        let owner_range = range(source, 10, 90);
        let formula_range = range(source, formula_start, formula_end);
        let owner = SymbolId::new(
            module.clone(),
            LocalSymbolId::new("Task180"),
            FullyQualifiedName::new("pkg::main::theorem::Task180"),
        );
        let owner_origin = SemanticOrigin::new(
            source,
            module.clone(),
            SourceAnchor::Range(owner_range),
            vec![0],
        );
        let checked_owner = CheckedStatementOwner::from_validated_parts_for_test(
            owner.clone(),
            owner_range,
            owner_origin,
        );

        let mut contexts = BindingContextTable::new();
        let context = contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Module,
            parent: None,
            layer: BindingContextLayer::Module,
            lexical_scope: None,
            bindings: Vec::new(),
            visible_bindings: Vec::new(),
            recovery: BindingContextRecovery::Normal,
        });
        let binding_env = BindingEnv::try_new(BindingEnvParts {
            source_id: source,
            module_id: module.clone(),
            contexts,
            bindings: BindingTable::new(),
            diagnostics: BindingDiagnosticTable::new(),
        })
        .expect("statement binding environment");

        let mut formula_input = FormulaInput::new(
            TypedSiteRef::Node(TypedNodeId::new(27)),
            context,
            formula_range,
            formula_kind,
        )
        .with_recovery(formula_recovery);
        if deferred {
            formula_input =
                formula_input.with_deferred(vec![FormulaDeferredReason::MissingFormulaPayload]);
        }
        let term_formula = TermFormulaChecker::default()
            .infer_without_symbols_for_test(&binding_env, [formula_input]);

        let mut builder = TypedArenaBuilder::new();
        builder
            .push(
                TypedNode::new(
                    "source.formula.contradiction",
                    SourceAnchor::Range(formula_range),
                )
                .with_recovery(formula_recovery)
                .with_typing(TypingState::Successful),
            )
            .expect("formula node");
        builder
            .push(
                TypedNode::new("source.statement.theorem", SourceAnchor::Range(owner_range))
                    .with_children(vec![TypedNodeId::new(0)])
                    .with_recovery(NodeRecoveryState::Normal)
                    .with_typing(TypingState::Successful),
            )
            .expect("theorem node");
        builder
            .push(
                TypedNode::new("source.module", SourceAnchor::Range(range(source, 0, 100)))
                    .with_children(vec![TypedNodeId::new(1)])
                    .with_recovery(NodeRecoveryState::Normal)
                    .with_typing(TypingState::Successful),
            )
            .expect("module node");
        let typed_ast = TypedAst::try_new(TypedAstParts {
            source_id: source,
            module_id: module,
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: builder
                .finish(Some(TypedNodeId::new(2)))
                .expect("statement typed arena"),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("statement typed AST");

        StatementFixture {
            source,
            module: typed_ast.module_id().clone(),
            context,
            owner_range,
            formula_range,
            typed_ast,
            binding_env,
            term_formula,
            owner,
            checked_owner,
        }
    }

    #[derive(Clone, Copy)]
    enum StatementTreeCorruption {
        BrokenEdge,
        RecoveredOwner,
        PartialOwner,
        PointOwnerAnchor,
        PointFormulaAnchor,
        PointRootAnchor,
        FormulaRangeMismatch,
        RootContainment,
        SwappedKinds,
    }

    fn corrupted_statement_typed_ast(
        fixture: &StatementFixture,
        corruption: StatementTreeCorruption,
    ) -> TypedAst {
        let formula_kind = if matches!(corruption, StatementTreeCorruption::SwappedKinds) {
            "source.module"
        } else {
            "source.formula.contradiction"
        };
        let formula_range = if matches!(corruption, StatementTreeCorruption::FormulaRangeMismatch) {
            range(
                fixture.source,
                fixture.formula_range.start + 1,
                fixture.formula_range.end,
            )
        } else {
            fixture.formula_range
        };
        let formula_anchor = if matches!(corruption, StatementTreeCorruption::PointFormulaAnchor) {
            SourceAnchor::Point {
                source_id: fixture.source,
                offset: formula_range.start,
            }
        } else {
            SourceAnchor::Range(formula_range)
        };
        let mut builder = TypedArenaBuilder::new();
        builder
            .push(
                TypedNode::new(formula_kind, formula_anchor)
                    .with_recovery(NodeRecoveryState::Normal)
                    .with_typing(TypingState::Successful),
            )
            .expect("corrupt formula node");
        let owner_kind = if matches!(corruption, StatementTreeCorruption::SwappedKinds) {
            "source.formula.contradiction"
        } else {
            "source.statement.theorem"
        };
        let owner_anchor = if matches!(corruption, StatementTreeCorruption::PointOwnerAnchor) {
            SourceAnchor::Point {
                source_id: fixture.source,
                offset: fixture.owner_range.start,
            }
        } else {
            SourceAnchor::Range(fixture.owner_range)
        };
        let mut owner = TypedNode::new(owner_kind, owner_anchor)
            .with_recovery(
                if matches!(corruption, StatementTreeCorruption::RecoveredOwner) {
                    NodeRecoveryState::Recovered
                } else {
                    NodeRecoveryState::Normal
                },
            )
            .with_typing(
                if matches!(corruption, StatementTreeCorruption::PartialOwner) {
                    TypingState::Error
                } else {
                    TypingState::Successful
                },
            );
        if !matches!(corruption, StatementTreeCorruption::BrokenEdge) {
            owner = owner.with_children(vec![TypedNodeId::new(0)]);
        }
        builder.push(owner).expect("corrupt owner node");
        let root_range = if matches!(corruption, StatementTreeCorruption::RootContainment) {
            range(
                fixture.source,
                fixture.owner_range.start + 1,
                fixture.owner_range.end - 1,
            )
        } else {
            range(fixture.source, 0, 100)
        };
        let root_kind = if matches!(corruption, StatementTreeCorruption::SwappedKinds) {
            "source.statement.theorem"
        } else {
            "source.module"
        };
        let root_anchor = if matches!(corruption, StatementTreeCorruption::PointRootAnchor) {
            SourceAnchor::Point {
                source_id: fixture.source,
                offset: root_range.start,
            }
        } else {
            SourceAnchor::Range(root_range)
        };
        builder
            .push(
                TypedNode::new(root_kind, root_anchor)
                    .with_children(vec![TypedNodeId::new(1)])
                    .with_recovery(NodeRecoveryState::Normal)
                    .with_typing(TypingState::Successful),
            )
            .expect("corrupt root node");
        TypedAst::try_new(TypedAstParts {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: builder
                .finish(Some(TypedNodeId::new(2)))
                .expect("corrupt statement arena"),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("structurally valid corrupt typed AST")
    }

    fn statement_binding_env(
        source: SourceId,
        module: ModuleId,
        recovery: BindingContextRecovery,
    ) -> BindingEnv {
        let mut contexts = BindingContextTable::new();
        contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Module,
            parent: None,
            layer: BindingContextLayer::Module,
            lexical_scope: None,
            bindings: Vec::new(),
            visible_bindings: Vec::new(),
            recovery,
        });
        BindingEnv::try_new(BindingEnvParts {
            source_id: source,
            module_id: module,
            contexts,
            bindings: BindingTable::new(),
            diagnostics: BindingDiagnosticTable::new(),
        })
        .expect("statement binding environment variant")
    }

    fn assemble_statement(
        fixture: &StatementFixture,
        rows: Vec<StatementSemanticInput>,
    ) -> Result<ResolvedTypedAst, ResolvedTypedAstError> {
        assemble_statement_with(
            &fixture.typed_ast,
            &fixture.checked_owner,
            &fixture.binding_env,
            &fixture.term_formula,
            rows,
            statement_node_hints(),
        )
    }

    fn assemble_fixture_with_proof_rows(
        fixture: &StatementFixture,
        proof_owner: &CheckedStatementOwner,
        proof_rows: Vec<StatementProofIntentInput>,
        include_statement: bool,
        include_proof: bool,
    ) -> Result<ResolvedTypedAst, ResolvedTypedAstError> {
        assemble_statement_with_bundles(
            &fixture.typed_ast,
            &fixture.binding_env,
            &fixture.term_formula,
            StatementAssemblyBundles {
                statement_owner: &fixture.checked_owner,
                proof_owner,
                rows: vec![fixture.row()],
                proof_rows,
                include_statement,
                include_proof,
            },
            statement_node_hints(),
        )
    }

    fn assemble_statement_with(
        typed_ast: &TypedAst,
        owner: &CheckedStatementOwner,
        binding_env: &BindingEnv,
        term_formula: &TermFormulaInferenceOutput,
        rows: Vec<StatementSemanticInput>,
        node_hints: Vec<ResolvedNodeKindHint>,
    ) -> Result<ResolvedTypedAst, ResolvedTypedAstError> {
        let proof_rows =
            rows.first()
                .and_then(|row| {
                    term_formula.formulas().get(row.formula).map(|formula| {
                        StatementProofIntentInput {
                            id: StatementProofIntentId::new(0),
                            source_order: 0,
                            statement: StatementSemanticId::new(0),
                            source_id: typed_ast.source_id(),
                            module_id: typed_ast.module_id().clone(),
                            owner: row.owner.clone(),
                            owner_node: row.owner_node,
                            owner_range: owner.source_range(),
                            owner_origin: owner.origin().clone(),
                            owner_visibility: owner.visibility(),
                            owner_export_status: owner.export_status(),
                            formula: row.formula,
                            formula_site: formula.site.clone(),
                            formula_node: row.formula_node,
                            formula_range: formula.source_range,
                            recovery: formula.recovery,
                            policy: TheoremPolicyIntent::Unmodified,
                            justification: TheoremJustificationIntent::Omitted,
                        }
                    })
                })
                .into_iter()
                .collect();
        assemble_statement_with_bundles(
            typed_ast,
            binding_env,
            term_formula,
            StatementAssemblyBundles {
                statement_owner: owner,
                proof_owner: owner,
                rows,
                proof_rows,
                include_statement: true,
                include_proof: true,
            },
            node_hints,
        )
    }

    struct StatementAssemblyBundles<'a> {
        statement_owner: &'a CheckedStatementOwner,
        proof_owner: &'a CheckedStatementOwner,
        rows: Vec<StatementSemanticInput>,
        proof_rows: Vec<StatementProofIntentInput>,
        include_statement: bool,
        include_proof: bool,
    }

    fn assemble_statement_with_bundles(
        typed_ast: &TypedAst,
        binding_env: &BindingEnv,
        term_formula: &TermFormulaInferenceOutput,
        bundles: StatementAssemblyBundles<'_>,
        node_hints: Vec<ResolvedNodeKindHint>,
    ) -> Result<ResolvedTypedAst, ResolvedTypedAstError> {
        let cluster_facts = ClusterFactTable::new();
        let collection = OverloadCollectionOutput::collect(
            Vec::<OverloadSiteInput>::new(),
            Vec::<OverloadCandidateInput>::new(),
        );
        let expansion = TemplateExpansionOutput::expand(&collection);
        let viability =
            CandidateViabilityOutput::filter(&expansion, Vec::<CandidateViabilityInput>::new());
        let graphs =
            SpecificityGraphOutput::build(&viability, Vec::<SpecificityComparisonInput>::new());
        let selection =
            OverloadSelectionOutput::resolve(&graphs, Vec::<OverloadSiteResolutionInput>::new());
        ResolvedTypedAst::assemble(ResolvedTypedAstInputs {
            typed_ast,
            cluster_facts: &cluster_facts,
            overload_collection: &collection,
            template_expansion: &expansion,
            viability: &viability,
            specificity: &graphs,
            overload_selection: &selection,
            expressions: Vec::new(),
            node_hints,
            statement_semantics: bundles
                .include_statement
                .then_some(StatementSemanticInputs {
                    owner: bundles.statement_owner,
                    binding_env,
                    term_formula,
                    rows: bundles.rows,
                }),
            statement_proofs: bundles.include_proof.then_some(StatementProofInputs {
                owner: bundles.proof_owner,
                rows: bundles.proof_rows,
            }),
        })
    }

    fn statement_node_hints() -> Vec<ResolvedNodeKindHint> {
        vec![
            ResolvedNodeKindHint {
                typed_node: TypedNodeId::new(0),
                kind: ResolvedNodeKindHintKind::SourcePreserved {
                    role: SourceNodeRole::new("source.formula.contradiction"),
                },
            },
            ResolvedNodeKindHint {
                typed_node: TypedNodeId::new(1),
                kind: ResolvedNodeKindHintKind::SourcePreserved {
                    role: SourceNodeRole::new("source.statement.theorem"),
                },
            },
            ResolvedNodeKindHint {
                typed_node: TypedNodeId::new(2),
                kind: ResolvedNodeKindHintKind::SourcePreserved {
                    role: SourceNodeRole::new("source.module"),
                },
            },
        ]
    }

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
            statement_semantics: None,
            statement_proofs: None,
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
            statement_semantics: None,
            statement_proofs: None,
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
            source_context: None,
            source_type: None,
            source_attribute: None,
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

    fn secondary_source_id(seed: u8) -> SourceId {
        let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{seed:064x}"
        ))
        .expect("valid build snapshot id");
        let allocator = InMemorySessionIdAllocator::new();
        allocator
            .next_source_id(snapshot)
            .expect("first source id allocation succeeds");
        allocator
            .next_source_id(snapshot)
            .expect("second source id allocation succeeds")
    }
}
