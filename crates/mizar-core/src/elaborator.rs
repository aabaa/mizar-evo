//! Core elaboration context preparation.
//!
//! Implements the task-8 through task-13 elaboration slices specified in
//! [elaborator.md](../../../../doc/design/mizar-core/en/elaborator.md).

use crate::{
    binder_normalization::{BinderContext, NormalizedVarClass, NormalizedVarSort},
    core_ir::{
        CoreAlgorithm, CoreAlgorithmId, CoreAlgorithmMatchArm, CoreAlgorithmStmt,
        CoreAlgorithmStmtId, CoreAlgorithmStmtKind, CoreAlgorithmStmtTable, CoreAlgorithmTable,
        CoreBinder, CoreCitation, CoreContractSet, CoreDefinition, CoreDefinitionId,
        CoreDefinitionOwner, CoreDefinitionTable, CoreDiagnostic, CoreDiagnosticClass,
        CoreDiagnosticId, CoreDiagnosticMessageKey, CoreDiagnosticRecovery, CoreDiagnosticSeverity,
        CoreDiagnosticTable, CoreFormula, CoreFormulaId, CoreFormulaKind, CoreFormulaTable, CoreIr,
        CoreIrParts, CoreItem, CoreItemId, CoreItemKind, CoreItemStatus, CoreItemTable,
        CoreJustification, CoreLabelRef, CoreNodeRef, CorePlace, CoreProof, CoreProofId,
        CoreProofNode, CoreProofNodeId, CoreProofNodeKind, CoreProofNodeTable, CoreProofStatus,
        CoreProofTable, CoreProvenance, CoreProvenanceKey, CoreProvenancePhase, CoreSourceAnchor,
        CoreSourceMap, CoreSourceRef, CoreTerm, CoreTermId, CoreTermKind, CoreTermTable,
        CoreTypePredicate, CoreVarId, CoreVarRole, CoreVisibility, DefinitionBody,
        DefinitionBranchBody, ExpansionPolicy, GeneratedOrigin, GeneratedOriginId,
        GeneratedOriginKey, GeneratedOriginKind, GeneratedOriginTable, GhostEffectKey,
        GuardedDefinitionBranch, LocalProofOrProgramPath, NormalizedSemanticOrigin, ObligationSeed,
        ObligationSeedId, ObligationSeedKind, ObligationSeedStatus, ObligationSeedTable,
        ProofBranchKind,
    },
};
use mizar_checker::{
    binding_env::{
        BinderIdentity, BindingContextId, BindingContextLayer, BindingContextOwner,
        BindingContextRecovery, BindingEnv, BindingId, BindingKind, BindingRecoveryState,
        BindingStatus, BindingTypeSite,
    },
    cluster_trace::ClusterFactId,
    overload_resolution::{QuaPathKey, TemplateInstantiationKey, TemplateParameterKey},
    registration_resolution::{
        CheckerRegistrationId, ExistentialGateBaseEvidenceCoverage,
        ExistentialGateBaseEvidenceKind, ExistentialGateId, ExistentialGateStatus,
        RegistrationDiagnosticId,
    },
    resolved_typed_ast::{
        CheckedProofId, CheckedProofNodeId, CheckedProofNodeKind, CheckedProofStatus,
        CheckedTerminalGoalId, CoercionInsertionId, OverloadResolutionId, ResolvedNodeRecovery,
        ResolvedTypedAst, ResolvedTypedDiagnosticId, ResolvedTypedDiagnosticSeverity,
        ResolvedTypedNodeId, ResolvedTypedNodeKind, StatementSemanticId,
        TheoremJustificationIntent, TheoremPolicyIntent,
    },
    source_attribute_definition::{
        SourceAttributeDefinitionHandoff, SourceAttributeDefinitionId,
        SourceAttributeDefinitionRecovery,
    },
    source_context::{
        SourceBindingContextHandoff, SourceBindingSiteRole, SourceDeclarationId, SourceItemId,
        SourceItemRecovery, SourceItemRole, SourceItemVisibility,
    },
    source_formula_composition::{
        SourceNestedFraenkelCaptureGraphCaptureId, SourceNestedFraenkelCaptureGraphOwnerHandoff,
    },
    source_functor_definition::{
        SourceFunctorCorrectnessKind, SourceFunctorDefiniensTarget, SourceFunctorDefinitionHandoff,
        SourceFunctorDefinitionId, SourceFunctorDefinitionRecovery, SourceFunctorDefinitionStyle,
    },
    source_predicate_definition::{
        SourcePredicateDefinitionHandoff, SourcePredicateDefinitionId,
        SourcePredicateDefinitionRecovery,
    },
    source_property_implementation::{
        SourcePropertyCarrierIdentity, SourcePropertyEqualsSelectorIdentityHandoff,
        SourcePropertyImplementationHandoff, SourcePropertyImplementationStyle,
        SourcePropertyParameterId,
    },
    source_structure::{
        SourceStructureEdgeId, SourceStructureEdgeRole, SourceStructureMemberId,
        SourceStructureMemberRole, SourceStructureRecovery, SourceStructureRequestId,
        SourceStructureRequestKind, SourceStructureTarget, SourceStructureTermId,
        SourceStructureTermKind,
    },
    source_structure_definition::{
        SourceStructureDefinitionHandoff, SourceStructureDefinitionId,
        SourceStructureDefinitionRecovery, SourceStructureMemberKind,
    },
    source_term::{
        SourcePrimaryTermId, SourcePrimaryTermKind, SourcePrimaryTermRecovery,
        SourcePrimaryTermReferenceId, SourcePrimaryTermReferenceRole, SourcePrimaryTermRole,
    },
    source_type::{
        SourceTypeApplicationForm, SourceTypeApplicationHandoff, SourceTypeApplicationId,
        SourceTypeExpressionId, SourceTypeHead, SourceTypeStructureMemberId,
    },
    type_checker::{CheckedFormulaId, FormulaKind, FormulaStatus},
    typed_ast::{
        BindingTypeRef, InitialObligationId, InitialObligationKind, NodeRecoveryState,
        NormalizedTypeId, Polarity, TypeDiagnosticId, TypeFactId, TypedNodeId, TypedSiteRef,
    },
};
use mizar_resolve::env::{ExportStatus, Visibility};
use mizar_resolve::names::FraenkelGeneratorVariableBindingId;
use mizar_resolve::resolved_ast::{ModuleId, SemanticOrigin, SymbolId};
use mizar_session::{SourceAnchor, SourceId, SourceRange};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
};

pub type CoreContextResult<T> = Result<T, CoreContextError>;

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CoreContextError {
    MissingProvenance {
        input: &'static str,
    },
    UnsupportedProvenancePhase {
        input: &'static str,
        phase: CoreProvenancePhase,
    },
    ForeignItemSeed {
        symbol: Box<SymbolId>,
        expected_module: Box<ModuleId>,
    },
    CurrentModuleDependencySummary {
        symbol: Box<SymbolId>,
    },
    DuplicateItemSymbol {
        symbol: Box<SymbolId>,
    },
    DuplicateDependencySummary {
        symbol: Box<SymbolId>,
    },
    DuplicateVariable {
        var: CoreVarId,
    },
    UndeclaredBinderVariable {
        var: CoreVarId,
    },
    DuplicateGeneratedOriginSeed {
        owner: Box<SymbolId>,
        kind: GeneratedOriginKind,
        key: GeneratedOriginKey,
    },
}

impl fmt::Display for CoreContextError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingProvenance { input } => {
                write!(formatter, "{input} is missing checker/resolver provenance")
            }
            Self::UnsupportedProvenancePhase { input, phase } => {
                write!(
                    formatter,
                    "{input} has unsupported provenance phase {phase:?}; expected resolver or checker"
                )
            }
            Self::ForeignItemSeed {
                symbol,
                expected_module,
            } => {
                write!(
                    formatter,
                    "current-module item seed {symbol:?} does not belong to module {expected_module:?}"
                )
            }
            Self::CurrentModuleDependencySummary { symbol } => {
                write!(
                    formatter,
                    "dependency summary {symbol:?} belongs to the current module"
                )
            }
            Self::DuplicateItemSymbol { symbol } => {
                write!(formatter, "duplicate current-module item symbol {symbol:?}")
            }
            Self::DuplicateDependencySummary { symbol } => {
                write!(
                    formatter,
                    "duplicate dependency summary for symbol {symbol:?}"
                )
            }
            Self::DuplicateVariable { var } => {
                write!(formatter, "duplicate binder variable seed {}", var.index())
            }
            Self::UndeclaredBinderVariable { var } => {
                write!(
                    formatter,
                    "binder source seed references undeclared variable {}",
                    var.index()
                )
            }
            Self::DuplicateGeneratedOriginSeed { owner, kind, key } => {
                write!(
                    formatter,
                    "duplicate generated origin seed for owner {owner:?}, kind {kind:?}, key {}",
                    key.as_str()
                )
            }
        }
    }
}

impl Error for CoreContextError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckerOwnedProvenance {
    entries: Vec<CoreProvenance>,
}

impl CheckerOwnedProvenance {
    pub fn resolver(key: impl Into<CoreProvenanceKey>) -> Self {
        Self {
            entries: vec![CoreProvenance::new(CoreProvenancePhase::Resolver, key)],
        }
    }

    pub fn checker(key: impl Into<CoreProvenanceKey>) -> Self {
        Self {
            entries: vec![CoreProvenance::new(CoreProvenancePhase::Checker, key)],
        }
    }

    pub fn try_new(entries: Vec<CoreProvenance>) -> CoreContextResult<Self> {
        validate_checker_owned_provenance("checker-owned provenance", &entries)?;
        Ok(Self { entries })
    }

    pub fn as_slice(&self) -> &[CoreProvenance] {
        &self.entries
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreItemSeed {
    symbol: SymbolId,
    kind: CoreItemKind,
    visibility: CoreVisibility,
    source: CoreSourceRef,
    dependencies: Vec<SymbolId>,
    definition_boundary: Option<DefinitionBoundaryKind>,
    provenance: CheckerOwnedProvenance,
}

impl CoreItemSeed {
    pub fn new(
        symbol: SymbolId,
        kind: CoreItemKind,
        visibility: impl Into<CoreVisibility>,
        source: CoreSourceRef,
        provenance: CheckerOwnedProvenance,
    ) -> Self {
        Self {
            symbol,
            kind,
            visibility: visibility.into(),
            source,
            dependencies: Vec::new(),
            definition_boundary: None,
            provenance,
        }
    }

    pub fn with_dependencies(mut self, dependencies: Vec<SymbolId>) -> Self {
        self.dependencies = dependencies;
        self
    }

    pub fn with_definition_boundary(mut self, kind: DefinitionBoundaryKind) -> Self {
        self.definition_boundary = Some(kind);
        self
    }

    pub const fn symbol(&self) -> &SymbolId {
        &self.symbol
    }

    pub fn dependencies(&self) -> &[SymbolId] {
        &self.dependencies
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreDependencySummary {
    symbol: SymbolId,
    kind: CoreItemKind,
    visibility: CoreVisibility,
    provenance: CheckerOwnedProvenance,
}

impl CoreDependencySummary {
    pub fn new(
        symbol: SymbolId,
        kind: CoreItemKind,
        visibility: impl Into<CoreVisibility>,
        provenance: CheckerOwnedProvenance,
    ) -> Self {
        Self {
            symbol,
            kind,
            visibility: visibility.into(),
            provenance,
        }
    }

    pub const fn symbol(&self) -> &SymbolId {
        &self.symbol
    }

    pub const fn kind(&self) -> &CoreItemKind {
        &self.kind
    }

    pub const fn visibility(&self) -> &CoreVisibility {
        &self.visibility
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreVariableSeed {
    var: CoreVarId,
    class: NormalizedVarClass,
    role: CoreVarRole,
    sort: NormalizedVarSort,
    type_facts: Vec<TypeFactId>,
    provenance: CheckerOwnedProvenance,
}

impl CoreVariableSeed {
    pub fn new(
        var: CoreVarId,
        class: NormalizedVarClass,
        role: impl Into<CoreVarRole>,
        sort: NormalizedVarSort,
        provenance: CheckerOwnedProvenance,
    ) -> Self {
        Self {
            var,
            class,
            role: role.into(),
            sort,
            type_facts: Vec::new(),
            provenance,
        }
    }

    pub fn with_type_facts(mut self, type_facts: Vec<TypeFactId>) -> Self {
        self.type_facts = type_facts;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreBinderSeed {
    var: CoreVarId,
    source: CoreSourceRef,
    provenance: CheckerOwnedProvenance,
}

impl CoreBinderSeed {
    pub fn new(var: CoreVarId, source: CoreSourceRef, provenance: CheckerOwnedProvenance) -> Self {
        Self {
            var,
            source,
            provenance,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedOriginSeed {
    owner: SymbolId,
    kind: GeneratedOriginKind,
    key: GeneratedOriginKey,
    functor: Option<SymbolId>,
    params: Vec<CoreVarId>,
    evidence: Vec<CoreProvenance>,
    source: CoreSourceRef,
    provenance: CheckerOwnedProvenance,
}

impl GeneratedOriginSeed {
    pub fn new(
        owner: SymbolId,
        kind: GeneratedOriginKind,
        key: impl Into<GeneratedOriginKey>,
        source: CoreSourceRef,
        provenance: CheckerOwnedProvenance,
    ) -> Self {
        Self {
            owner,
            kind,
            key: key.into(),
            functor: None,
            params: Vec::new(),
            evidence: Vec::new(),
            source,
            provenance,
        }
    }

    pub fn with_params(mut self, params: Vec<CoreVarId>) -> Self {
        self.params = params;
        self
    }

    pub fn with_functor(mut self, functor: SymbolId) -> Self {
        self.functor = Some(functor);
        self
    }

    pub fn with_evidence(mut self, evidence: Vec<CoreProvenance>) -> Self {
        self.evidence = evidence;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum DefinitionBoundaryKind {
    DefinitionalItem,
    Theorem,
    Lemma,
    Scheme,
    Registration,
    Reduction,
    Algorithm,
    GeneratedDefinition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum DefinitionBoundaryStatus {
    PendingBody,
    Skipped,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinitionBoundary {
    pub item: CoreItemId,
    pub symbol: SymbolId,
    pub kind: DefinitionBoundaryKind,
    pub status: DefinitionBoundaryStatus,
    pub source: CoreSourceRef,
    pub provenance: CheckerOwnedProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DefinitionBoundaryRegistry {
    by_item: BTreeMap<CoreItemId, DefinitionBoundary>,
    by_symbol: BTreeMap<SymbolId, CoreItemId>,
}

impl DefinitionBoundaryRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_by_item(&self, item: CoreItemId) -> Option<&DefinitionBoundary> {
        self.by_item.get(&item)
    }

    pub fn get_by_symbol(&self, symbol: &SymbolId) -> Option<&DefinitionBoundary> {
        self.by_symbol
            .get(symbol)
            .and_then(|item| self.get_by_item(*item))
    }

    pub fn iter(&self) -> impl Iterator<Item = (CoreItemId, &DefinitionBoundary)> {
        self.by_item.iter().map(|(id, boundary)| (*id, boundary))
    }

    fn insert(&mut self, boundary: DefinitionBoundary) {
        self.by_symbol
            .insert(boundary.symbol.clone(), boundary.item);
        self.by_item.insert(boundary.item, boundary);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GeneratedOriginRegistry {
    table: GeneratedOriginTable,
    by_key: BTreeMap<(CoreItemId, GeneratedOriginKind, GeneratedOriginKey), GeneratedOriginId>,
}

impl GeneratedOriginRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub const fn table(&self) -> &GeneratedOriginTable {
        &self.table
    }

    pub fn get_by_key(
        &self,
        owner: CoreItemId,
        kind: GeneratedOriginKind,
        key: &GeneratedOriginKey,
    ) -> Option<GeneratedOriginId> {
        self.by_key.get(&(owner, kind, key.clone())).copied()
    }

    fn insert(&mut self, owner: CoreItemId, origin: GeneratedOrigin) -> GeneratedOriginId {
        let key = (owner, origin.kind, origin.key.clone());
        let id = self.table.insert(origin);
        self.by_key.insert(key, id);
        id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CoreItemRegistry {
    items: CoreItemTable,
    by_symbol: BTreeMap<SymbolId, CoreItemId>,
    dependencies: BTreeMap<CoreItemId, CoreDependencyResolution>,
}

impl CoreItemRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub const fn items(&self) -> &CoreItemTable {
        &self.items
    }

    pub fn id_for_symbol(&self, symbol: &SymbolId) -> Option<CoreItemId> {
        self.by_symbol.get(symbol).copied()
    }

    pub fn dependencies(&self, item: CoreItemId) -> Option<&CoreDependencyResolution> {
        self.dependencies.get(&item)
    }

    pub fn iter(&self) -> impl Iterator<Item = (CoreItemId, &CoreItem)> {
        self.items.iter()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CoreDependencyResolution {
    pub local: Vec<CoreItemId>,
    pub external: Vec<SymbolId>,
    pub missing: Vec<SymbolId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinderSourceRecord {
    pub var: CoreVarId,
    pub source: CoreSourceRef,
    pub provenance: CheckerOwnedProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BinderSourceRegistry {
    by_var: BTreeMap<CoreVarId, BinderSourceRecord>,
}

impl BinderSourceRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, var: CoreVarId) -> Option<&BinderSourceRecord> {
        self.by_var.get(&var)
    }

    pub fn iter(&self) -> impl Iterator<Item = (CoreVarId, &BinderSourceRecord)> {
        self.by_var.iter().map(|(var, record)| (*var, record))
    }

    fn insert(&mut self, record: BinderSourceRecord) -> CoreContextResult<()> {
        if self.by_var.contains_key(&record.var) {
            return Err(CoreContextError::DuplicateVariable { var: record.var });
        }
        self.by_var.insert(record.var, record);
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedTypedAstSummary {
    source_id: SourceId,
    module_id: ModuleId,
    checker_sites: Vec<CheckerSiteSummary>,
}

impl ResolvedTypedAstSummary {
    pub fn new(source_id: SourceId, module_id: ModuleId) -> Self {
        Self {
            source_id,
            module_id,
            checker_sites: Vec::new(),
        }
    }

    pub fn from_ast(ast: &ResolvedTypedAst) -> Self {
        let mut summary = Self::new(ast.source_id(), ast.module_id().clone());

        for (id, node) in ast.nodes().iter() {
            match &node.kind {
                ResolvedTypedNodeKind::FailedOverload { result } => {
                    summary.checker_sites.push(CheckerSiteSummary {
                        kind: CheckerSiteKind::FailedOverload { result: *result },
                        source: CoreSourceRef::direct(node.source_range),
                        diagnostics: node.diagnostics.clone(),
                        severity: CheckerSiteSeverity::Error,
                    });
                }
                ResolvedTypedNodeKind::Degraded { .. } => {
                    summary.checker_sites.push(CheckerSiteSummary {
                        kind: CheckerSiteKind::RecoveredNode {
                            node: id,
                            recovery: node.recovery,
                        },
                        source: CoreSourceRef::direct(node.source_range),
                        diagnostics: node.diagnostics.clone(),
                        severity: CheckerSiteSeverity::Warning,
                    });
                }
                ResolvedTypedNodeKind::SourcePreserved { .. }
                | ResolvedTypedNodeKind::ResolvedUse { .. }
                    if node.recovery != ResolvedNodeRecovery::Normal =>
                {
                    summary.checker_sites.push(CheckerSiteSummary {
                        kind: CheckerSiteKind::RecoveredNode {
                            node: id,
                            recovery: node.recovery,
                        },
                        source: CoreSourceRef::direct(node.source_range),
                        diagnostics: node.diagnostics.clone(),
                        severity: CheckerSiteSeverity::Warning,
                    });
                }
                _ => {}
            }
        }

        for (id, diagnostic) in ast.diagnostics().canonical_iter() {
            summary.checker_sites.push(CheckerSiteSummary {
                kind: CheckerSiteKind::CheckerDiagnostic { diagnostic: id },
                source: CoreSourceRef::direct(diagnostic.source_range),
                diagnostics: vec![id],
                severity: CheckerSiteSeverity::from(diagnostic.severity),
            });
        }

        summary.checker_sites.sort_by(checker_site_cmp);
        summary
    }

    pub fn with_checker_sites(mut self, checker_sites: Vec<CheckerSiteSummary>) -> Self {
        self.checker_sites = checker_sites;
        self.checker_sites.sort_by(checker_site_cmp);
        self
    }

    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub fn checker_sites(&self) -> &[CheckerSiteSummary] {
        &self.checker_sites
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckerSiteSummary {
    pub kind: CheckerSiteKind,
    pub source: CoreSourceRef,
    pub diagnostics: Vec<ResolvedTypedDiagnosticId>,
    pub severity: CheckerSiteSeverity,
}

impl CheckerSiteSummary {
    pub fn failed_overload(result: OverloadResolutionId, source: CoreSourceRef) -> Self {
        Self {
            kind: CheckerSiteKind::FailedOverload { result },
            source,
            diagnostics: Vec::new(),
            severity: CheckerSiteSeverity::Error,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CheckerSiteKind {
    FailedOverload {
        result: OverloadResolutionId,
    },
    RecoveredNode {
        node: ResolvedTypedNodeId,
        recovery: ResolvedNodeRecovery,
    },
    CheckerDiagnostic {
        diagnostic: ResolvedTypedDiagnosticId,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CheckerSiteSeverity {
    Error,
    Warning,
    Note,
}

impl From<ResolvedTypedDiagnosticSeverity> for CheckerSiteSeverity {
    fn from(value: ResolvedTypedDiagnosticSeverity) -> Self {
        match value {
            ResolvedTypedDiagnosticSeverity::Error => Self::Error,
            ResolvedTypedDiagnosticSeverity::Warning => Self::Warning,
            ResolvedTypedDiagnosticSeverity::Note => Self::Note,
            _ => Self::Error,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreContextInput {
    pub resolved: ResolvedTypedAstSummary,
    pub item_seeds: Vec<CoreItemSeed>,
    pub dependency_summaries: Vec<CoreDependencySummary>,
    pub variable_seeds: Vec<CoreVariableSeed>,
    pub binder_seeds: Vec<CoreBinderSeed>,
    pub generated_origin_seeds: Vec<GeneratedOriginSeed>,
}

impl CoreContextInput {
    pub fn new(resolved: ResolvedTypedAstSummary) -> Self {
        Self {
            resolved,
            item_seeds: Vec::new(),
            dependency_summaries: Vec::new(),
            variable_seeds: Vec::new(),
            binder_seeds: Vec::new(),
            generated_origin_seeds: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreContext {
    source_id: SourceId,
    module_id: ModuleId,
    item_registry: CoreItemRegistry,
    dependency_summaries: BTreeMap<SymbolId, CoreDependencySummary>,
    definition_boundaries: DefinitionBoundaryRegistry,
    generated_origins: GeneratedOriginRegistry,
    binder_context: BinderContext,
    binder_sources: BinderSourceRegistry,
    binder_type_facts: BTreeMap<CoreVarId, Vec<TypeFactId>>,
    source_map: CoreSourceMap,
    diagnostics: CoreDiagnosticTable,
    worklist: ElaborationWorklist,
}

impl CoreContext {
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub const fn item_registry(&self) -> &CoreItemRegistry {
        &self.item_registry
    }

    pub const fn dependency_summaries(&self) -> &BTreeMap<SymbolId, CoreDependencySummary> {
        &self.dependency_summaries
    }

    pub const fn definition_boundaries(&self) -> &DefinitionBoundaryRegistry {
        &self.definition_boundaries
    }

    pub const fn generated_origins(&self) -> &GeneratedOriginRegistry {
        &self.generated_origins
    }

    pub const fn binder_context(&self) -> &BinderContext {
        &self.binder_context
    }

    pub const fn binder_sources(&self) -> &BinderSourceRegistry {
        &self.binder_sources
    }

    pub const fn binder_type_facts(&self) -> &BTreeMap<CoreVarId, Vec<TypeFactId>> {
        &self.binder_type_facts
    }

    pub const fn source_map(&self) -> &CoreSourceMap {
        &self.source_map
    }

    pub const fn diagnostics(&self) -> &CoreDiagnosticTable {
        &self.diagnostics
    }

    pub const fn worklist(&self) -> &ElaborationWorklist {
        &self.worklist
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ElaborationWorklist {
    entries: Vec<ElaborationWorkItem>,
}

impl ElaborationWorklist {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn entries(&self) -> &[ElaborationWorkItem] {
        &self.entries
    }

    fn push(&mut self, entry: ElaborationWorkItem) {
        self.entries.push(entry);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElaborationWorkItem {
    pub kind: ElaborationWorkItemKind,
    pub status: ElaborationWorkStatus,
    pub source: CoreSourceRef,
    pub diagnostics: Vec<CoreDiagnosticId>,
    pub checker_diagnostics: Vec<ResolvedTypedDiagnosticId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ElaborationWorkItemKind {
    Item(CoreItemId),
    CheckerSite(CheckerSiteKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ElaborationWorkStatus {
    Pending,
    Skipped,
    Error,
}

pub fn prepare_core_context(input: CoreContextInput) -> CoreContextResult<CoreContext> {
    validate_input(&input)?;

    let mut item_seeds = input.item_seeds;
    item_seeds.sort_by(item_seed_cmp);

    let dependency_summaries = dependency_summary_map(input.dependency_summaries)?;
    let mut diagnostics = CoreDiagnosticTable::new();
    let mut source_map = CoreSourceMap::new();
    let mut item_registry = CoreItemRegistry::new();
    let mut definition_boundaries = DefinitionBoundaryRegistry::new();

    for seed in &item_seeds {
        if item_registry.by_symbol.contains_key(&seed.symbol) {
            return Err(CoreContextError::DuplicateItemSymbol {
                symbol: Box::new(seed.symbol.clone()),
            });
        }

        let source = normalized_source(seed.source.clone());
        let item = CoreItem::new(
            seed.symbol.clone(),
            seed.kind.clone(),
            seed.visibility.clone(),
            source.clone(),
        );
        let item_id = item_registry.items.insert(item);
        item_registry.by_symbol.insert(seed.symbol.clone(), item_id);
        source_map.item_sources.insert(item_id, source.clone());

        if let Some(kind) = seed.definition_boundary {
            definition_boundaries.insert(DefinitionBoundary {
                item: item_id,
                symbol: seed.symbol.clone(),
                kind,
                status: DefinitionBoundaryStatus::PendingBody,
                source,
                provenance: seed.provenance.clone(),
            });
        }
    }

    resolve_item_dependencies(
        &item_seeds,
        &dependency_summaries,
        &mut item_registry,
        &mut diagnostics,
    );

    let (binder_context, binder_sources, binder_type_facts) =
        prepare_binder_context(input.variable_seeds, input.binder_seeds)?;
    let mut generated_origins = GeneratedOriginRegistry::new();
    prepare_generated_origins(
        input.generated_origin_seeds,
        &item_registry,
        &mut generated_origins,
        &mut source_map,
        &mut diagnostics,
    )?;

    let mut worklist = ElaborationWorklist::new();
    push_item_worklist(&item_registry, &mut worklist);
    push_checker_site_worklist(&input.resolved, &mut diagnostics, &mut worklist);

    Ok(CoreContext {
        source_id: input.resolved.source_id,
        module_id: input.resolved.module_id,
        item_registry,
        dependency_summaries,
        definition_boundaries,
        generated_origins,
        binder_context,
        binder_sources,
        binder_type_facts,
        source_map,
        diagnostics,
        worklist,
    })
}

fn validate_input(input: &CoreContextInput) -> CoreContextResult<()> {
    for seed in &input.item_seeds {
        if seed.symbol.module() != input.resolved.module_id() {
            return Err(CoreContextError::ForeignItemSeed {
                symbol: Box::new(seed.symbol.clone()),
                expected_module: Box::new(input.resolved.module_id().clone()),
            });
        }
        validate_checker_owned_provenance("item seed", seed.provenance.as_slice())?;
    }
    for summary in &input.dependency_summaries {
        if summary.symbol.module() == input.resolved.module_id() {
            return Err(CoreContextError::CurrentModuleDependencySummary {
                symbol: Box::new(summary.symbol.clone()),
            });
        }
        validate_checker_owned_provenance("dependency summary", summary.provenance.as_slice())?;
    }
    for seed in &input.variable_seeds {
        validate_checker_owned_provenance("variable seed", seed.provenance.as_slice())?;
    }
    for seed in &input.binder_seeds {
        validate_checker_owned_provenance("binder seed", seed.provenance.as_slice())?;
    }
    for seed in &input.generated_origin_seeds {
        validate_checker_owned_provenance("generated origin seed", seed.provenance.as_slice())?;
        if !seed.evidence.is_empty() {
            validate_checker_owned_provenance("generated origin evidence", &seed.evidence)?;
        }
    }
    Ok(())
}

fn validate_checker_owned_provenance(
    input: &'static str,
    entries: &[CoreProvenance],
) -> CoreContextResult<()> {
    if entries.is_empty() {
        return Err(CoreContextError::MissingProvenance { input });
    }
    for entry in entries {
        if !matches!(
            entry.phase,
            CoreProvenancePhase::Resolver | CoreProvenancePhase::Checker
        ) {
            return Err(CoreContextError::UnsupportedProvenancePhase {
                input,
                phase: entry.phase,
            });
        }
    }
    Ok(())
}

fn dependency_summary_map(
    summaries: Vec<CoreDependencySummary>,
) -> CoreContextResult<BTreeMap<SymbolId, CoreDependencySummary>> {
    let mut map = BTreeMap::new();
    for summary in summaries {
        if map.contains_key(&summary.symbol) {
            return Err(CoreContextError::DuplicateDependencySummary {
                symbol: Box::new(summary.symbol),
            });
        }
        map.insert(summary.symbol.clone(), summary);
    }
    Ok(map)
}

fn resolve_item_dependencies(
    seeds: &[CoreItemSeed],
    dependency_summaries: &BTreeMap<SymbolId, CoreDependencySummary>,
    item_registry: &mut CoreItemRegistry,
    diagnostics: &mut CoreDiagnosticTable,
) {
    for seed in seeds {
        let item_id = item_registry
            .id_for_symbol(&seed.symbol)
            .expect("item seed inserted before dependency resolution");
        let mut resolution = CoreDependencyResolution::default();

        for dependency in &seed.dependencies {
            if let Some(local) = item_registry.id_for_symbol(dependency) {
                resolution.local.push(local);
            } else if dependency_summaries.contains_key(dependency) {
                resolution.external.push(dependency.clone());
            } else {
                resolution.missing.push(dependency.clone());
                let diagnostic = diagnostic(
                    CoreDiagnosticClass::UnresolvedSemanticInput,
                    CoreDiagnosticSeverity::Error,
                    CoreDiagnosticRecovery::Fatal,
                    "missing-dependency-summary",
                    seed.source.clone(),
                    Some(CoreNodeRef::Item(item_id)),
                );
                let diagnostic_id = diagnostics.insert(diagnostic);
                if let Some(item) = item_registry.items.get_mut(item_id) {
                    item.status = CoreItemStatus::Partial;
                    item.diagnostics.push(diagnostic_id);
                }
            }
        }

        resolution.local.sort();
        resolution.local.dedup();
        resolution.external.sort();
        resolution.external.dedup();
        resolution.missing.sort();
        resolution.missing.dedup();
        if let Some(item) = item_registry.items.get_mut(item_id) {
            item.dependencies = resolution.local.clone();
        }
        item_registry.dependencies.insert(item_id, resolution);
    }
}

fn prepare_binder_context(
    variable_seeds: Vec<CoreVariableSeed>,
    binder_seeds: Vec<CoreBinderSeed>,
) -> CoreContextResult<(
    BinderContext,
    BinderSourceRegistry,
    BTreeMap<CoreVarId, Vec<TypeFactId>>,
)> {
    let mut seen = BTreeSet::new();
    let mut context = BinderContext::new();
    let mut type_facts = BTreeMap::new();

    for seed in variable_seeds {
        if !seen.insert(seed.var) {
            return Err(CoreContextError::DuplicateVariable { var: seed.var });
        }
        context.declare_variable(seed.var, seed.class, seed.role, seed.sort);
        let mut seed_type_facts = seed.type_facts;
        seed_type_facts.sort();
        seed_type_facts.dedup();
        type_facts.insert(seed.var, seed_type_facts);
    }

    let mut sources = BinderSourceRegistry::new();
    for seed in binder_seeds {
        if !seen.contains(&seed.var) {
            return Err(CoreContextError::UndeclaredBinderVariable { var: seed.var });
        }
        sources.insert(BinderSourceRecord {
            var: seed.var,
            source: normalized_source(seed.source),
            provenance: seed.provenance,
        })?;
    }

    Ok((context, sources, type_facts))
}

fn prepare_generated_origins(
    seeds: Vec<GeneratedOriginSeed>,
    item_registry: &CoreItemRegistry,
    generated_origins: &mut GeneratedOriginRegistry,
    source_map: &mut CoreSourceMap,
    diagnostics: &mut CoreDiagnosticTable,
) -> CoreContextResult<()> {
    let mut seen = BTreeSet::new();
    for seed in seeds {
        let Some(owner) = item_registry.id_for_symbol(&seed.owner) else {
            diagnostics.insert(diagnostic(
                CoreDiagnosticClass::UnresolvedSemanticInput,
                CoreDiagnosticSeverity::Error,
                CoreDiagnosticRecovery::Fatal,
                "missing-generated-origin-owner",
                seed.source,
                None,
            ));
            continue;
        };
        let key = (owner, seed.kind, seed.key.clone());
        if !seen.insert(key) {
            return Err(CoreContextError::DuplicateGeneratedOriginSeed {
                owner: Box::new(seed.owner),
                kind: seed.kind,
                key: seed.key,
            });
        }
        let source = normalized_source(seed.source);
        let mut evidence = seed.evidence;
        evidence.extend(seed.provenance.as_slice().iter().cloned());
        evidence.sort();
        evidence.dedup();
        let origin = GeneratedOrigin {
            owner,
            kind: seed.kind,
            key: seed.key,
            functor: seed.functor,
            params: seed.params,
            evidence,
            source: source.clone(),
        };
        let origin_id = generated_origins.insert(owner, origin);
        source_map.generated_sources.insert(origin_id, source);
    }
    Ok(())
}

fn push_item_worklist(item_registry: &CoreItemRegistry, worklist: &mut ElaborationWorklist) {
    for (id, item) in item_registry.iter() {
        worklist.push(ElaborationWorkItem {
            kind: ElaborationWorkItemKind::Item(id),
            status: match item.status {
                CoreItemStatus::Valid => ElaborationWorkStatus::Pending,
                CoreItemStatus::Partial | CoreItemStatus::Skipped => ElaborationWorkStatus::Skipped,
                CoreItemStatus::Error => ElaborationWorkStatus::Error,
            },
            source: item.source.clone(),
            diagnostics: item.diagnostics.clone(),
            checker_diagnostics: Vec::new(),
        });
    }
}

fn push_checker_site_worklist(
    resolved: &ResolvedTypedAstSummary,
    diagnostics: &mut CoreDiagnosticTable,
    worklist: &mut ElaborationWorklist,
) {
    for site in resolved.checker_sites() {
        let (class, severity, recovery, status, message) = match site.severity {
            CheckerSiteSeverity::Error => (
                CoreDiagnosticClass::UnsupportedLowering,
                CoreDiagnosticSeverity::Error,
                CoreDiagnosticRecovery::Fatal,
                ElaborationWorkStatus::Error,
                "checker-error-site-preserved",
            ),
            CheckerSiteSeverity::Warning => (
                CoreDiagnosticClass::UnsupportedLowering,
                CoreDiagnosticSeverity::Warning,
                CoreDiagnosticRecovery::Partial,
                ElaborationWorkStatus::Skipped,
                "checker-recovered-site-preserved",
            ),
            CheckerSiteSeverity::Note => (
                CoreDiagnosticClass::UnsupportedLowering,
                CoreDiagnosticSeverity::Note,
                CoreDiagnosticRecovery::Recovered,
                ElaborationWorkStatus::Skipped,
                "checker-note-site-preserved",
            ),
        };
        let diagnostic_id = diagnostics.insert(diagnostic(
            class,
            severity,
            recovery,
            message,
            site.source.clone(),
            None,
        ));
        worklist.push(ElaborationWorkItem {
            kind: ElaborationWorkItemKind::CheckerSite(site.kind.clone()),
            status,
            source: normalized_source(site.source.clone()),
            diagnostics: vec![diagnostic_id],
            checker_diagnostics: site.diagnostics.clone(),
        });
    }
}

const NESTED_FRAENKEL_CAPTURE_CORE_ROLE: &str = "fraenkel-captured-parameter";
const NESTED_FRAENKEL_CAPTURE_CORE_PROVENANCE_PREFIX: &str =
    "source-nested-fraenkel-capture-core-variable-v1.capture";

const SOURCE_BINDING_CORE_RESERVED_ROLE: &str = "reserved-variable";
const SOURCE_BINDING_CORE_PARAMETER_ROLE: &str = "definition-parameter";
const SOURCE_BINDING_CORE_PROVENANCE_PREFIX: &str = "source-binding-core-variable-v1.binding";

/// One immutable Core variable associated with one checker-authenticated
/// source binding row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceBindingCoreVariable {
    binding: BindingId,
    core_var: CoreVarId,
}

impl SourceBindingCoreVariable {
    #[must_use]
    pub const fn binding(&self) -> BindingId {
        self.binding
    }

    #[must_use]
    pub const fn core_var(&self) -> CoreVarId {
        self.core_var
    }
}

/// Immutable Core variables in exact checker binding-table order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceBindingCoreVariableTable {
    rows: Vec<(BindingId, SourceBindingCoreVariable)>,
}

impl SourceBindingCoreVariableTable {
    fn empty() -> Self {
        Self { rows: Vec::new() }
    }

    #[must_use]
    pub fn get(&self, binding: BindingId) -> Option<&SourceBindingCoreVariable> {
        self.rows
            .iter()
            .find_map(|(id, row)| (*id == binding).then_some(row))
    }

    pub fn iter(&self) -> impl Iterator<Item = (BindingId, &SourceBindingCoreVariable)> {
        self.rows.iter().map(|(binding, row)| (*binding, row))
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// Errors raised while associating checker source bindings with Core.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceBindingCoreContextError {
    EnvironmentMismatch,
    InvalidCoreContext,
    InvalidBindingEnvironment,
    CoreVariableAllocationOverflow,
    CoreVariableCollision { var: CoreVarId },
    InvalidBindingAssociation,
}

impl fmt::Display for SourceBindingCoreContextError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EnvironmentMismatch => {
                formatter.write_str("source binding Core context environment is invalid")
            }
            Self::InvalidCoreContext => {
                formatter.write_str("source binding Core context is invalid")
            }
            Self::InvalidBindingEnvironment => formatter
                .write_str("source binding environment is invalid for Core context transport"),
            Self::CoreVariableAllocationOverflow => {
                formatter.write_str("source binding Core variable allocation overflowed")
            }
            Self::CoreVariableCollision { var } => {
                write!(
                    formatter,
                    "source binding Core variable {} collides",
                    var.index()
                )
            }
            Self::InvalidBindingAssociation => {
                formatter.write_str("source binding Core variable association is invalid")
            }
        }
    }
}

impl Error for SourceBindingCoreContextError {}

/// Immutable Core context handoff for checker-authenticated source bindings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceBindingCoreContextHandoff {
    context: CoreContext,
    binding_env: BindingEnv,
    variables: SourceBindingCoreVariableTable,
}

impl SourceBindingCoreContextHandoff {
    #[must_use]
    pub const fn source_id(&self) -> SourceId {
        self.context.source_id()
    }

    #[must_use]
    pub const fn module_id(&self) -> &ModuleId {
        self.context.module_id()
    }

    #[must_use]
    pub const fn context(&self) -> &CoreContext {
        &self.context
    }

    #[must_use]
    pub const fn binding_env(&self) -> &BindingEnv {
        &self.binding_env
    }

    #[must_use]
    pub const fn variables(&self) -> &SourceBindingCoreVariableTable {
        &self.variables
    }

    #[must_use]
    pub fn debug_text(&self) -> String {
        let variables = self
            .variables
            .iter()
            .map(|(binding, row)| format!("{}:{}", binding.index(), row.core_var().index()))
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "source-binding-core-context-v1|module={}.{}|bindings={}|variables={}",
            self.module_id().package().as_str(),
            self.module_id().path().as_str(),
            self.binding_env.bindings().len(),
            variables,
        )
    }

    fn validate(&self) -> Result<(), SourceBindingCoreContextError> {
        if self.source_id() != self.binding_env.source_id()
            || self.module_id() != self.binding_env.module_id()
        {
            return Err(SourceBindingCoreContextError::EnvironmentMismatch);
        }
        let allowed = source_binding_vars(&self.variables);
        let used = validate_source_binding_core_context_shape(&self.context, &allowed)?;
        validate_source_binding_environment(&self.binding_env)?;
        validate_source_binding_association(
            &self.context,
            &self.binding_env,
            &self.variables,
            &used,
        )
    }
}

/// Builds the standalone immutable Core source-binding handoff.
#[derive(Debug, Clone, Copy)]
pub struct SourceBindingCoreContextProducer;

impl SourceBindingCoreContextProducer {
    pub fn build(
        mut context: CoreContext,
        binding_env: BindingEnv,
    ) -> Result<SourceBindingCoreContextHandoff, SourceBindingCoreContextError> {
        if context.source_id() != binding_env.source_id()
            || context.module_id() != binding_env.module_id()
        {
            return Err(SourceBindingCoreContextError::EnvironmentMismatch);
        }
        let used = validate_source_binding_core_context_shape(&context, &BTreeSet::new())?;
        validate_source_binding_environment(&binding_env)?;
        let allocated = allocate_source_binding_core_vars(&used, binding_env.bindings().len())?;
        let mut variables = SourceBindingCoreVariableTable::empty();

        for ((binding, entry), core_var) in binding_env.bindings().iter().zip(allocated) {
            let key = source_binding_provenance_key(binding);
            let provenance = CoreProvenance::new(CoreProvenancePhase::Checker, key.clone());
            let source =
                CoreSourceRef::direct(entry.declaration_range).with_provenance(vec![provenance]);
            let role = match entry.kind {
                BindingKind::ReservedVariable => SOURCE_BINDING_CORE_RESERVED_ROLE,
                BindingKind::DefinitionParameter => SOURCE_BINDING_CORE_PARAMETER_ROLE,
                _ => return Err(SourceBindingCoreContextError::InvalidBindingEnvironment),
            };
            context.binder_context.declare_variable(
                core_var,
                NormalizedVarClass::Free,
                role,
                NormalizedVarSort::Term,
            );
            context.binder_type_facts.insert(core_var, Vec::new());
            context
                .binder_sources
                .insert(BinderSourceRecord {
                    var: core_var,
                    source,
                    provenance: CheckerOwnedProvenance::checker(key),
                })
                .map_err(|_| SourceBindingCoreContextError::CoreVariableCollision {
                    var: core_var,
                })?;
            variables
                .rows
                .push((binding, SourceBindingCoreVariable { binding, core_var }));
        }

        let handoff = SourceBindingCoreContextHandoff {
            context,
            binding_env,
            variables,
        };
        handoff.validate()?;
        Ok(handoff)
    }
}

fn source_binding_vars(table: &SourceBindingCoreVariableTable) -> BTreeSet<CoreVarId> {
    table.iter().map(|(_, row)| row.core_var()).collect()
}

fn source_binding_provenance_key(binding: BindingId) -> CoreProvenanceKey {
    CoreProvenanceKey::new(format!(
        "{SOURCE_BINDING_CORE_PROVENANCE_PREFIX}.{}",
        binding.index()
    ))
}

fn validate_source_binding_core_context_shape(
    context: &CoreContext,
    allowed_source_binding_vars: &BTreeSet<CoreVarId>,
) -> Result<BTreeSet<CoreVarId>, SourceBindingCoreContextError> {
    let used = validate_core_context_shape(context, &BTreeSet::new())
        .map_err(|_| SourceBindingCoreContextError::InvalidCoreContext)?;
    if context
        .binder_context
        .variable_roles
        .iter()
        .any(|(var, role)| {
            matches!(
                role.as_str(),
                SOURCE_BINDING_CORE_RESERVED_ROLE | SOURCE_BINDING_CORE_PARAMETER_ROLE
            ) && !allowed_source_binding_vars.contains(var)
        })
    {
        return Err(SourceBindingCoreContextError::InvalidCoreContext);
    }
    Ok(used)
}

fn validate_source_binding_environment(
    binding_env: &BindingEnv,
) -> Result<(), SourceBindingCoreContextError> {
    if binding_env.contexts().is_empty()
        || binding_env.bindings().is_empty()
        || !binding_env.diagnostics().is_empty()
    {
        return Err(SourceBindingCoreContextError::InvalidBindingEnvironment);
    }
    let Some(module_context) = binding_env.contexts().get(BindingContextId::new(0)) else {
        return Err(SourceBindingCoreContextError::InvalidBindingEnvironment);
    };
    if !is_normal_module_context(module_context) {
        return Err(SourceBindingCoreContextError::InvalidBindingEnvironment);
    }
    for (context_id, context) in binding_env.contexts().iter() {
        let invalid = if context_id == BindingContextId::new(0) {
            !is_normal_module_context(context)
        } else {
            !is_normal_declaration_context(context)
        };
        if invalid {
            return Err(SourceBindingCoreContextError::InvalidBindingEnvironment);
        }
    }

    for (_, binding) in binding_env.bindings().iter() {
        if binding.status == BindingStatus::Degraded
            || binding.status == BindingStatus::Omitted
            || binding.recovery != BindingRecoveryState::Normal
            || !binding.captured.identities().is_empty()
            || !binding.diagnostics.is_empty()
        {
            return Err(SourceBindingCoreContextError::InvalidBindingEnvironment);
        }
        let Some(owner_context) = binding_env.contexts().get(binding.owner_context) else {
            return Err(SourceBindingCoreContextError::InvalidBindingEnvironment);
        };
        match (&binding.kind, &binding.identity, binding.status) {
            (
                BindingKind::ReservedVariable,
                BinderIdentity::ReservedVariable {
                    spelling,
                    declaration_range,
                },
                BindingStatus::Reserved,
            ) if binding.owner_context == BindingContextId::new(0)
                && is_normal_module_context(owner_context)
                && spelling == &binding.spelling
                && *declaration_range == binding.declaration_range => {}
            (
                BindingKind::DefinitionParameter,
                BinderIdentity::ResolverLocal {
                    scope,
                    ordinal,
                    declaration_range,
                },
                BindingStatus::Active,
            ) if is_normal_declaration_context(owner_context)
                && owner_context.lexical_scope.as_ref() == Some(scope)
                && *ordinal == binding.visible_after_ordinal
                && *declaration_range == binding.declaration_range => {}
            _ => return Err(SourceBindingCoreContextError::InvalidBindingEnvironment),
        }
    }
    Ok(())
}

fn is_normal_module_context(context: &mizar_checker::binding_env::BindingContext) -> bool {
    context.owner == BindingContextOwner::Module
        && context.parent.is_none()
        && context.layer == BindingContextLayer::Module
        && context.lexical_scope.is_none()
        && context.recovery == BindingContextRecovery::Normal
}

fn is_normal_declaration_context(context: &mizar_checker::binding_env::BindingContext) -> bool {
    matches!(context.owner, BindingContextOwner::DeclarationShell(_))
        && context.parent.is_some()
        && context.layer == BindingContextLayer::Declaration
        && context.lexical_scope.is_some()
        && context.recovery == BindingContextRecovery::Normal
}

fn validate_source_binding_association(
    context: &CoreContext,
    binding_env: &BindingEnv,
    variables: &SourceBindingCoreVariableTable,
    used: &BTreeSet<CoreVarId>,
) -> Result<(), SourceBindingCoreContextError> {
    if variables.len() != binding_env.bindings().len() {
        return Err(SourceBindingCoreContextError::InvalidBindingAssociation);
    }
    let mut row_vars = BTreeSet::new();
    for (_, row) in variables.iter() {
        if !row_vars.insert(row.core_var()) {
            return Err(SourceBindingCoreContextError::CoreVariableCollision {
                var: row.core_var(),
            });
        }
    }
    let non_source_used = used
        .iter()
        .copied()
        .filter(|var| !row_vars.contains(var))
        .collect::<BTreeSet<_>>();
    let allocated = allocate_source_binding_core_vars(&non_source_used, variables.len())?;

    for (((binding, entry), row), expected_var) in binding_env
        .bindings()
        .iter()
        .zip(variables.iter())
        .zip(allocated)
    {
        if row.0 != binding || row.1.binding() != binding || row.1.core_var() != expected_var {
            return Err(SourceBindingCoreContextError::InvalidBindingAssociation);
        }
        let Some(record) = context.binder_sources.get(row.1.core_var()) else {
            return Err(SourceBindingCoreContextError::InvalidBindingAssociation);
        };
        let key = source_binding_provenance_key(binding);
        let expected_provenance = CoreProvenance::new(CoreProvenancePhase::Checker, key.clone());
        if record.source.anchor != CoreSourceAnchor::SourceRange(entry.declaration_range)
            || record.source.provenance.as_slice() != [expected_provenance.clone()]
            || record.provenance.as_slice()
                != [CoreProvenance::new(CoreProvenancePhase::Checker, key)]
            || context
                .binder_context
                .variable_classes
                .get(&row.1.core_var())
                != Some(&NormalizedVarClass::Free)
            || context.binder_context.variable_sorts.get(&row.1.core_var())
                != Some(&NormalizedVarSort::Term)
            || context
                .binder_context
                .variable_roles
                .get(&row.1.core_var())
                .map(CoreVarRole::as_str)
                != Some(match entry.kind {
                    BindingKind::ReservedVariable => SOURCE_BINDING_CORE_RESERVED_ROLE,
                    BindingKind::DefinitionParameter => SOURCE_BINDING_CORE_PARAMETER_ROLE,
                    _ => return Err(SourceBindingCoreContextError::InvalidBindingAssociation),
                })
            || context.binder_type_facts.get(&row.1.core_var()) != Some(&Vec::new())
        {
            return Err(SourceBindingCoreContextError::InvalidBindingAssociation);
        }
    }
    Ok(())
}

const SOURCE_PREDICATE_CORE_ITEM_PROVENANCE_KEY: &str =
    "source-predicate-core-item-v1.definition.0";

/// One immutable association between a checker source item, its predicate
/// definition, the definition's whole symbol, and the corresponding Core
/// item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePredicateCoreItemAssociation {
    source_item: SourceItemId,
    definition: SourcePredicateDefinitionId,
    symbol: SymbolId,
    core_item: CoreItemId,
}

impl SourcePredicateCoreItemAssociation {
    #[must_use]
    pub const fn source_item(&self) -> SourceItemId {
        self.source_item
    }

    #[must_use]
    pub const fn definition(&self) -> SourcePredicateDefinitionId {
        self.definition
    }

    #[must_use]
    pub const fn symbol(&self) -> &SymbolId {
        &self.symbol
    }

    #[must_use]
    pub const fn core_item(&self) -> CoreItemId {
        self.core_item
    }
}

/// Immutable source-ordered table of predicate/Core item associations.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SourcePredicateCoreItemAssociationTable {
    rows: Vec<SourcePredicateCoreItemAssociation>,
}

impl SourcePredicateCoreItemAssociationTable {
    fn empty() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn get(
        &self,
        definition: SourcePredicateDefinitionId,
    ) -> Option<&SourcePredicateCoreItemAssociation> {
        self.rows.iter().find(|row| row.definition() == definition)
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<
        Item = (
            SourcePredicateDefinitionId,
            &SourcePredicateCoreItemAssociation,
        ),
    > {
        self.rows.iter().map(|row| (row.definition(), row))
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// Errors raised while building the exact Task-259 predicate/Core context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourcePredicateCoreContextError {
    EnvironmentMismatch,
    InvalidSourceBindingContext,
    InvalidCheckerOwner,
    InvalidCoreContext,
    InvalidItemAssociation,
}

impl fmt::Display for SourcePredicateCoreContextError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EnvironmentMismatch => {
                formatter.write_str("predicate Core context environment is invalid")
            }
            Self::InvalidSourceBindingContext => {
                formatter.write_str("predicate Core source-binding context is invalid")
            }
            Self::InvalidCheckerOwner => {
                formatter.write_str("predicate Core checker owner is invalid")
            }
            Self::InvalidCoreContext => formatter.write_str("predicate Core context is invalid"),
            Self::InvalidItemAssociation => {
                formatter.write_str("predicate Core item association is invalid")
            }
        }
    }
}

impl Error for SourcePredicateCoreContextError {}

/// Immutable Core context handoff for the exact checker-authenticated
/// Task-259 predicate definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePredicateCoreContextHandoff {
    source_bindings: SourceBindingCoreContextHandoff,
    source_context: SourceBindingContextHandoff,
    checker_owner: SourcePredicateDefinitionHandoff,
    items: SourcePredicateCoreItemAssociationTable,
}

impl SourcePredicateCoreContextHandoff {
    #[must_use]
    pub const fn source_id(&self) -> SourceId {
        self.source_bindings.source_id()
    }

    #[must_use]
    pub const fn module_id(&self) -> &ModuleId {
        self.source_bindings.module_id()
    }

    #[must_use]
    pub const fn context(&self) -> &CoreContext {
        self.source_bindings.context()
    }

    #[must_use]
    pub const fn source_bindings(&self) -> &SourceBindingCoreContextHandoff {
        &self.source_bindings
    }

    #[must_use]
    pub const fn source_context(&self) -> &SourceBindingContextHandoff {
        &self.source_context
    }

    #[must_use]
    pub const fn checker_owner(&self) -> &SourcePredicateDefinitionHandoff {
        &self.checker_owner
    }

    #[must_use]
    pub const fn items(&self) -> &SourcePredicateCoreItemAssociationTable {
        &self.items
    }

    #[must_use]
    pub fn debug_text(&self) -> String {
        let associations = self
            .items
            .iter()
            .map(|(definition, association)| {
                format!(
                    "{}:{}:{}:{}",
                    definition.index(),
                    association.source_item().index(),
                    association.symbol().fqn().as_str(),
                    association.core_item().index(),
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "source-predicate-core-item-context-v1|module={}.{}|source-bindings={}|definitions={}|items={}",
            self.module_id().package().as_str(),
            self.module_id().path().as_str(),
            self.source_bindings.binding_env().bindings().len(),
            self.checker_owner.definitions().len(),
            associations,
        )
    }

    fn validate(&self) -> Result<(), SourcePredicateCoreContextError> {
        validate_predicate_core_environment(
            &self.source_bindings,
            &self.source_context,
            &self.checker_owner,
        )?;
        self.source_bindings
            .validate()
            .map_err(|_| SourcePredicateCoreContextError::InvalidSourceBindingContext)?;
        let definition =
            validate_predicate_checker_owner(&self.source_context, &self.checker_owner)?;
        let core_item = validate_predicate_core_shape(
            self.source_bindings.context(),
            definition.symbol(),
            definition.source_range(),
        )?;
        validate_predicate_item_association(
            &self.source_context,
            &self.checker_owner,
            &self.items,
            definition,
            core_item,
        )
    }
}

/// Builds the standalone immutable Task-259 predicate/Core context handoff.
#[derive(Debug, Clone, Copy)]
pub struct SourcePredicateCoreContextProducer;

impl SourcePredicateCoreContextProducer {
    pub fn build(
        source_bindings: SourceBindingCoreContextHandoff,
        source_context: SourceBindingContextHandoff,
        checker_owner: SourcePredicateDefinitionHandoff,
    ) -> Result<SourcePredicateCoreContextHandoff, SourcePredicateCoreContextError> {
        validate_predicate_core_environment(&source_bindings, &source_context, &checker_owner)?;
        source_bindings
            .validate()
            .map_err(|_| SourcePredicateCoreContextError::InvalidSourceBindingContext)?;
        let definition = validate_predicate_checker_owner(&source_context, &checker_owner)?;
        let core_item = validate_predicate_core_shape(
            source_bindings.context(),
            definition.symbol(),
            definition.source_range(),
        )?;
        let source_item = source_context
            .context_links()
            .get(definition.context())
            .and_then(|link| link.item)
            .ok_or(SourcePredicateCoreContextError::InvalidItemAssociation)?;
        let mut items = SourcePredicateCoreItemAssociationTable::empty();
        items.rows.push(SourcePredicateCoreItemAssociation {
            source_item,
            definition: definition.id(),
            symbol: definition.symbol().clone(),
            core_item,
        });
        let handoff = SourcePredicateCoreContextHandoff {
            source_bindings,
            source_context,
            checker_owner,
            items,
        };
        handoff.validate()?;
        Ok(handoff)
    }
}

fn validate_predicate_core_environment(
    source_bindings: &SourceBindingCoreContextHandoff,
    source_context: &SourceBindingContextHandoff,
    checker_owner: &SourcePredicateDefinitionHandoff,
) -> Result<(), SourcePredicateCoreContextError> {
    let source_id = source_bindings.source_id();
    let module_id = source_bindings.module_id();
    if source_context.source_id() != source_id
        || source_context.module_id() != module_id
        || checker_owner.source_id() != source_id
        || checker_owner.module_id() != module_id
        || source_bindings.binding_env().source_id() != source_id
        || source_bindings.binding_env().module_id() != module_id
        || source_context.binding_env().source_id() != source_id
        || source_context.binding_env().module_id() != module_id
        || source_context.binding_env() != source_bindings.binding_env()
        || source_bindings.context().source_id() != source_id
        || source_bindings.context().module_id() != module_id
    {
        return Err(SourcePredicateCoreContextError::EnvironmentMismatch);
    }
    Ok(())
}

fn range_contains(outer: SourceRange, inner: SourceRange) -> bool {
    outer.source_id == inner.source_id && outer.start <= inner.start && inner.end <= outer.end
}

fn validate_predicate_checker_owner<'a>(
    source_context: &SourceBindingContextHandoff,
    checker_owner: &'a SourcePredicateDefinitionHandoff,
) -> Result<
    &'a mizar_checker::source_predicate_definition::SourcePredicateDefinition,
    SourcePredicateCoreContextError,
> {
    if checker_owner.source_context_fingerprint() != source_context.debug_text()
        || source_context.items().len() != 1
        || source_context.declarations().len() != 2
        || source_context.context_links().len() != 2
        || source_context.local_contexts().len() != 2
    {
        return Err(SourcePredicateCoreContextError::InvalidCheckerOwner);
    }

    let source_item = source_context
        .items()
        .get(SourceItemId::new(0))
        .ok_or(SourcePredicateCoreContextError::InvalidCheckerOwner)?;
    if source_item.id != SourceItemId::new(0)
        || source_item.role != SourceItemRole::DefinitionBlock
        || source_item.shell_ordinal != 0
        || source_item.visibility != SourceItemVisibility::Unspecified
        || source_item.recovery != SourceItemRecovery::Normal
        || source_item.parent.is_some()
        || source_item.local_scope.is_none()
        || source_item.predecessor.is_some()
        || source_item.source_range.source_id != source_context.source_id()
    {
        return Err(SourcePredicateCoreContextError::InvalidCheckerOwner);
    }

    let module_context = source_context
        .binding_env()
        .contexts()
        .get(BindingContextId::new(0))
        .ok_or(SourcePredicateCoreContextError::InvalidCheckerOwner)?;
    let definition_context = source_context
        .binding_env()
        .contexts()
        .get(source_item.binding_context)
        .ok_or(SourcePredicateCoreContextError::InvalidCheckerOwner)?;
    if module_context.id != BindingContextId::new(0)
        || !is_normal_module_context(module_context)
        || definition_context.id != source_item.binding_context
        || !is_normal_declaration_context(definition_context)
        || definition_context.owner != BindingContextOwner::DeclarationShell(source_item.shell)
        || definition_context.parent != Some(BindingContextId::new(0))
        || definition_context.lexical_scope != source_item.local_scope
    {
        return Err(SourcePredicateCoreContextError::InvalidCheckerOwner);
    }

    let declaration_validity = source_context
        .declarations()
        .iter()
        .map(|(id, declaration)| {
            declaration.id == id
                && declaration.item == source_item.id
                && declaration.binding_context == source_item.binding_context
                && declaration.local_context == source_item.local_context
                && declaration.predecessor
                    == id.index().checked_sub(1).map(SourceDeclarationId::new)
                && declaration.declaration_range.source_id == source_context.source_id()
                && declaration.written_type_range.source_id == source_context.source_id()
                && matches!(
                    declaration.role,
                    SourceBindingSiteRole::DefinitionParameter { .. }
                )
        })
        .collect::<Vec<_>>();
    if declaration_validity.len() != 2 || declaration_validity.iter().any(|valid| !valid) {
        return Err(SourcePredicateCoreContextError::InvalidCheckerOwner);
    }
    let declaration_rows = source_context.declarations().iter().collect::<Vec<_>>();
    if declaration_rows[0].1.source_ordinal != 0
        || declaration_rows[1].1.source_ordinal != 1
        || declaration_rows[0].1.source_ordinal >= declaration_rows[1].1.source_ordinal
        || definition_context.bindings
            != declaration_rows
                .iter()
                .map(|(_, row)| row.binding)
                .collect::<Vec<_>>()
        || definition_context.visible_bindings != definition_context.bindings
    {
        return Err(SourcePredicateCoreContextError::InvalidCheckerOwner);
    }
    for (index, (_, declaration)) in declaration_rows.iter().enumerate() {
        let binding = source_context
            .binding_env()
            .bindings()
            .get(declaration.binding)
            .ok_or(SourcePredicateCoreContextError::InvalidCheckerOwner)?;
        if declaration.binding != binding.id
            || binding.owner_context != source_item.binding_context
            || binding.kind != BindingKind::DefinitionParameter
            || binding.status != BindingStatus::Active
            || binding.recovery != BindingRecoveryState::Normal
            || binding.declaration_range != declaration.declaration_range
            || binding.visible_after_ordinal != declaration.source_ordinal
            || binding.spelling != declaration.spelling
            || index != declaration.source_ordinal
        {
            return Err(SourcePredicateCoreContextError::InvalidCheckerOwner);
        }
    }

    let module_local_context = source_context
        .local_contexts()
        .get(mizar_checker::typed_ast::LocalTypeContextId::new(0))
        .ok_or(SourcePredicateCoreContextError::InvalidCheckerOwner)?;
    let definition_local_context = source_context
        .local_contexts()
        .get(source_item.local_context)
        .ok_or(SourcePredicateCoreContextError::InvalidCheckerOwner)?;
    let expected_local_bindings = declaration_rows
        .iter()
        .map(|(_, declaration)| BindingTypeRef::Site(declaration.site.clone()))
        .collect::<Vec<_>>();
    if module_local_context.id != mizar_checker::typed_ast::LocalTypeContextId::new(0)
        || module_local_context.parent.is_some()
        || module_local_context.layer != mizar_checker::typed_ast::TypeContextLayer::Module
        || module_local_context.recovery != mizar_checker::typed_ast::ContextRecoveryState::Normal
        || !module_local_context.bindings.is_empty()
        || !module_local_context.introduced_assumptions.is_empty()
        || !module_local_context.visible_facts.is_empty()
        || definition_local_context.id != source_item.local_context
        || definition_local_context.owner != source_item.site
        || definition_local_context.parent
            != Some(mizar_checker::typed_ast::LocalTypeContextId::new(0))
        || definition_local_context.layer != mizar_checker::typed_ast::TypeContextLayer::Declaration
        || definition_local_context.recovery
            != mizar_checker::typed_ast::ContextRecoveryState::Normal
        || definition_local_context.bindings != expected_local_bindings
        || !definition_local_context.introduced_assumptions.is_empty()
        || !definition_local_context.visible_facts.is_empty()
    {
        return Err(SourcePredicateCoreContextError::InvalidCheckerOwner);
    }

    for (index, (link_id, link)) in source_context.context_links().iter().enumerate() {
        let expected_context = BindingContextId::new(index);
        let expected_item = (index == 1).then_some(source_item.id);
        if link_id != index
            || link.binding_context != expected_context
            || link.local_context != mizar_checker::typed_ast::LocalTypeContextId::new(index)
            || link.item != expected_item
        {
            return Err(SourcePredicateCoreContextError::InvalidCheckerOwner);
        }
    }

    if checker_owner.definitions().len() != 1
        || checker_owner.parameters().len() != 2
        || checker_owner.guards().len() != 1
        || checker_owner.properties().len() != 1
        || checker_owner.correctness().len() != 1
    {
        return Err(SourcePredicateCoreContextError::InvalidCheckerOwner);
    }
    let definition = checker_owner
        .definitions()
        .get(SourcePredicateDefinitionId::new(0))
        .ok_or(SourcePredicateCoreContextError::InvalidCheckerOwner)?;
    if definition.id() != SourcePredicateDefinitionId::new(0)
        || definition.source_ordinal() != 0
        || definition.context() != source_item.binding_context
        || definition.recovery() != SourcePredicateDefinitionRecovery::Normal
        || definition.source_range().source_id != source_context.source_id()
        || !range_contains(source_item.source_range, definition.source_range())
    {
        return Err(SourcePredicateCoreContextError::InvalidCheckerOwner);
    }
    let origin = definition.origin();
    if origin.source_id() != source_context.source_id()
        || origin.module_id() != source_context.module_id()
        || origin.is_recovered()
        || origin.import_edge().is_some()
        || origin.anchor() != &SourceAnchor::Range(definition.source_range())
    {
        return Err(SourcePredicateCoreContextError::InvalidCheckerOwner);
    }
    for (index, (_, parameter)) in checker_owner.parameters().iter().enumerate() {
        let (_, declaration) = declaration_rows
            .get(index)
            .ok_or(SourcePredicateCoreContextError::InvalidCheckerOwner)?;
        if parameter.owner() != definition.id()
            || parameter.ordinal() != index
            || parameter.binding() != declaration.binding
            || parameter.site() != &declaration.site
            || parameter.declaration_range() != declaration.declaration_range
            || parameter.context() != definition.context()
            || parameter.recovery() != SourcePredicateDefinitionRecovery::Normal
            || parameter.source_range().source_id != source_context.source_id()
            || parameter.declaration_range().source_id != source_context.source_id()
            || !range_contains(parameter.source_range(), parameter.declaration_range())
        {
            return Err(SourcePredicateCoreContextError::InvalidCheckerOwner);
        }
    }
    let guard = checker_owner
        .guards()
        .get(mizar_checker::source_predicate_definition::SourcePredicateGuardId::new(0))
        .ok_or(SourcePredicateCoreContextError::InvalidCheckerOwner)?;
    if guard.id() != mizar_checker::source_predicate_definition::SourcePredicateGuardId::new(0)
        || guard.owner() != definition.id()
        || guard.ordinal() != 0
        || guard.context() != definition.context()
        || guard.recovery() != SourcePredicateDefinitionRecovery::Normal
        || guard.source_range().source_id != source_context.source_id()
    {
        return Err(SourcePredicateCoreContextError::InvalidCheckerOwner);
    }
    let property = checker_owner
        .properties()
        .get(mizar_checker::source_predicate_definition::SourcePredicatePropertyId::new(0))
        .ok_or(SourcePredicateCoreContextError::InvalidCheckerOwner)?;
    if property.id()
        != mizar_checker::source_predicate_definition::SourcePredicatePropertyId::new(0)
        || property.owner() != definition.id()
        || property.ordinal() != 0
        || property.recovery() != SourcePredicateDefinitionRecovery::Normal
        || property.source_range().source_id != source_context.source_id()
    {
        return Err(SourcePredicateCoreContextError::InvalidCheckerOwner);
    }
    let correctness = checker_owner
        .correctness()
        .get(mizar_checker::source_predicate_definition::SourcePredicateCorrectnessId::new(0))
        .ok_or(SourcePredicateCoreContextError::InvalidCheckerOwner)?;
    if correctness.id()
        != mizar_checker::source_predicate_definition::SourcePredicateCorrectnessId::new(0)
        || correctness.owner() != definition.id()
        || correctness.property() != property.id()
        || correctness.ordinal() != 0
    {
        return Err(SourcePredicateCoreContextError::InvalidCheckerOwner);
    }
    if !matches!(property.justification(), SourceAnchor::Range(range) if range.source_id == source_context.source_id())
        || !matches!(correctness.source_anchor(), SourceAnchor::Range(range) if range.source_id == source_context.source_id())
    {
        return Err(SourcePredicateCoreContextError::InvalidCheckerOwner);
    }
    Ok(definition)
}

fn validate_predicate_core_shape(
    context: &CoreContext,
    symbol: &SymbolId,
    source_range: SourceRange,
) -> Result<CoreItemId, SourcePredicateCoreContextError> {
    if validate_core_context_shape(context, &BTreeSet::new()).is_err()
        || !context.dependency_summaries.is_empty()
        || !context.generated_origins.table().is_empty()
        || !context.generated_origins.by_key.is_empty()
        || !context.diagnostics.is_empty()
        || context.source_map.item_sources.len() != 1
        || !context.source_map.term_sources.is_empty()
        || !context.source_map.formula_sources.is_empty()
        || !context.source_map.definition_sources.is_empty()
        || !context.source_map.proof_sources.is_empty()
        || !context.source_map.algorithm_sources.is_empty()
        || !context.source_map.generated_sources.is_empty()
        || !context.source_map.obligation_sources.is_empty()
        || context.item_registry.items.len() != 1
        || context.item_registry.by_symbol.len() != 1
        || context.item_registry.dependencies.len() != 1
        || context.definition_boundaries.by_item.len() != 1
        || context.definition_boundaries.by_symbol.len() != 1
        || context.worklist.entries.len() != 1
    {
        return Err(SourcePredicateCoreContextError::InvalidCoreContext);
    }
    let core_item = context
        .item_registry
        .id_for_symbol(symbol)
        .ok_or(SourcePredicateCoreContextError::InvalidCoreContext)?;
    let item = context
        .item_registry
        .items
        .get(core_item)
        .ok_or(SourcePredicateCoreContextError::InvalidCoreContext)?;
    let expected_provenance = CoreProvenance::new(
        CoreProvenancePhase::Checker,
        SOURCE_PREDICATE_CORE_ITEM_PROVENANCE_KEY,
    );
    let expected_source =
        CoreSourceRef::direct(source_range).with_provenance(vec![expected_provenance.clone()]);
    if item.symbol != *symbol
        || item.kind != CoreItemKind::Predicate
        || item.visibility.as_str() != "public"
        || item.status != CoreItemStatus::Valid
        || !item.dependencies.is_empty()
        || !item.diagnostics.is_empty()
        || item.source != expected_source
        || context.source_map.item_sources.get(&core_item) != Some(&item.source)
    {
        return Err(SourcePredicateCoreContextError::InvalidCoreContext);
    }
    let dependency = context
        .item_registry
        .dependencies
        .get(&core_item)
        .ok_or(SourcePredicateCoreContextError::InvalidCoreContext)?;
    if !dependency.local.is_empty()
        || !dependency.external.is_empty()
        || !dependency.missing.is_empty()
    {
        return Err(SourcePredicateCoreContextError::InvalidCoreContext);
    }
    let boundary = context
        .definition_boundaries
        .by_item
        .get(&core_item)
        .ok_or(SourcePredicateCoreContextError::InvalidCoreContext)?;
    if context.definition_boundaries.by_symbol.get(symbol) != Some(&core_item)
        || boundary.item != core_item
        || boundary.symbol != *symbol
        || boundary.kind != DefinitionBoundaryKind::DefinitionalItem
        || boundary.status != DefinitionBoundaryStatus::PendingBody
        || boundary.source != expected_source
        || boundary.provenance.as_slice() != [expected_provenance.clone()]
    {
        return Err(SourcePredicateCoreContextError::InvalidCoreContext);
    }
    let work_item = context
        .worklist
        .entries
        .first()
        .ok_or(SourcePredicateCoreContextError::InvalidCoreContext)?;
    if work_item.kind != ElaborationWorkItemKind::Item(core_item)
        || work_item.status != ElaborationWorkStatus::Pending
        || work_item.source != expected_source
        || !work_item.diagnostics.is_empty()
        || !work_item.checker_diagnostics.is_empty()
    {
        return Err(SourcePredicateCoreContextError::InvalidCoreContext);
    }
    Ok(core_item)
}

fn validate_predicate_item_association(
    source_context: &SourceBindingContextHandoff,
    checker_owner: &SourcePredicateDefinitionHandoff,
    items: &SourcePredicateCoreItemAssociationTable,
    definition: &mizar_checker::source_predicate_definition::SourcePredicateDefinition,
    core_item: CoreItemId,
) -> Result<(), SourcePredicateCoreContextError> {
    if items.len() != checker_owner.definitions().len() || items.len() != 1 {
        return Err(SourcePredicateCoreContextError::InvalidItemAssociation);
    }
    let link = source_context
        .context_links()
        .get(definition.context())
        .ok_or(SourcePredicateCoreContextError::InvalidItemAssociation)?;
    let source_item = link
        .item
        .ok_or(SourcePredicateCoreContextError::InvalidItemAssociation)?;
    let association = items
        .get(definition.id())
        .ok_or(SourcePredicateCoreContextError::InvalidItemAssociation)?;
    if association.definition() != definition.id()
        || association.source_item() != source_item
        || association.symbol() != definition.symbol()
        || association.core_item() != core_item
        || items
            .iter()
            .next()
            .is_none_or(|(id, row)| id != definition.id() || row != association)
    {
        return Err(SourcePredicateCoreContextError::InvalidItemAssociation);
    }
    Ok(())
}

const SOURCE_FUNCTOR_CORE_ITEM_PROVENANCE_KEYS: [&str; 2] = [
    "source-functor-core-item-v1.definition.0",
    "source-functor-core-item-v1.definition.1",
];

/// One immutable association between a checker source item, its functor
/// definition, the definition's whole symbol, and the corresponding Core
/// item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFunctorCoreItemAssociation {
    source_item: SourceItemId,
    definition: SourceFunctorDefinitionId,
    symbol: SymbolId,
    core_item: CoreItemId,
}

impl SourceFunctorCoreItemAssociation {
    #[must_use]
    pub const fn source_item(&self) -> SourceItemId {
        self.source_item
    }

    #[must_use]
    pub const fn definition(&self) -> SourceFunctorDefinitionId {
        self.definition
    }

    #[must_use]
    pub const fn symbol(&self) -> &SymbolId {
        &self.symbol
    }

    #[must_use]
    pub const fn core_item(&self) -> CoreItemId {
        self.core_item
    }
}

/// Immutable source-ordered table of functor/Core item associations.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SourceFunctorCoreItemAssociationTable {
    rows: Vec<SourceFunctorCoreItemAssociation>,
}

impl SourceFunctorCoreItemAssociationTable {
    fn empty() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn get(
        &self,
        definition: SourceFunctorDefinitionId,
    ) -> Option<&SourceFunctorCoreItemAssociation> {
        self.rows.iter().find(|row| row.definition() == definition)
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (SourceFunctorDefinitionId, &SourceFunctorCoreItemAssociation)> {
        self.rows.iter().map(|row| (row.definition(), row))
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// Errors raised while building the exact Task-260 functor/Core context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceFunctorCoreContextError {
    EnvironmentMismatch,
    InvalidSourceBindingContext,
    InvalidCheckerOwner,
    InvalidCoreContext,
    InvalidItemAssociation,
}

impl fmt::Display for SourceFunctorCoreContextError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EnvironmentMismatch => {
                formatter.write_str("functor Core context environment is invalid")
            }
            Self::InvalidSourceBindingContext => {
                formatter.write_str("functor Core source-binding context is invalid")
            }
            Self::InvalidCheckerOwner => {
                formatter.write_str("functor Core checker owner is invalid")
            }
            Self::InvalidCoreContext => formatter.write_str("functor Core context is invalid"),
            Self::InvalidItemAssociation => {
                formatter.write_str("functor Core item association is invalid")
            }
        }
    }
}

impl Error for SourceFunctorCoreContextError {}

/// Immutable Core context handoff for the exact checker-authenticated
/// Task-260 functor definitions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFunctorCoreContextHandoff {
    source_bindings: SourceBindingCoreContextHandoff,
    source_context: SourceBindingContextHandoff,
    checker_owner: SourceFunctorDefinitionHandoff,
    items: SourceFunctorCoreItemAssociationTable,
}

impl SourceFunctorCoreContextHandoff {
    #[must_use]
    pub const fn source_id(&self) -> SourceId {
        self.source_bindings.source_id()
    }

    #[must_use]
    pub const fn module_id(&self) -> &ModuleId {
        self.source_bindings.module_id()
    }

    #[must_use]
    pub const fn context(&self) -> &CoreContext {
        self.source_bindings.context()
    }

    #[must_use]
    pub const fn source_bindings(&self) -> &SourceBindingCoreContextHandoff {
        &self.source_bindings
    }

    #[must_use]
    pub const fn source_context(&self) -> &SourceBindingContextHandoff {
        &self.source_context
    }

    #[must_use]
    pub const fn checker_owner(&self) -> &SourceFunctorDefinitionHandoff {
        &self.checker_owner
    }

    #[must_use]
    pub const fn items(&self) -> &SourceFunctorCoreItemAssociationTable {
        &self.items
    }

    #[must_use]
    pub fn debug_text(&self) -> String {
        let associations = self
            .items
            .iter()
            .map(|(definition, association)| {
                format!(
                    "{}:{}:{}:{}",
                    definition.index(),
                    association.source_item().index(),
                    association.symbol().fqn().as_str(),
                    association.core_item().index(),
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "source-functor-core-item-context-v1|module={}.{}|source-bindings={}|definitions={}|items={}",
            self.module_id().package().as_str(),
            self.module_id().path().as_str(),
            self.source_bindings.binding_env().bindings().len(),
            self.checker_owner.definitions().len(),
            associations,
        )
    }

    fn validate(&self) -> Result<(), SourceFunctorCoreContextError> {
        validate_functor_core_environment(
            &self.source_bindings,
            &self.source_context,
            &self.checker_owner,
        )?;
        self.source_bindings
            .validate()
            .map_err(|_| SourceFunctorCoreContextError::InvalidSourceBindingContext)?;
        validate_functor_checker_owner(&self.source_context, &self.checker_owner)?;
        let core_items =
            validate_functor_core_shape(self.source_bindings.context(), &self.checker_owner)?;
        validate_functor_item_association(
            &self.source_context,
            &self.checker_owner,
            &self.items,
            &core_items,
        )
    }
}

/// Builds the standalone immutable Task-260 functor/Core context handoff.
#[derive(Debug, Clone, Copy)]
pub struct SourceFunctorCoreContextProducer;

impl SourceFunctorCoreContextProducer {
    pub fn build(
        source_bindings: SourceBindingCoreContextHandoff,
        source_context: SourceBindingContextHandoff,
        checker_owner: SourceFunctorDefinitionHandoff,
    ) -> Result<SourceFunctorCoreContextHandoff, SourceFunctorCoreContextError> {
        validate_functor_core_environment(&source_bindings, &source_context, &checker_owner)?;
        source_bindings
            .validate()
            .map_err(|_| SourceFunctorCoreContextError::InvalidSourceBindingContext)?;
        validate_functor_checker_owner(&source_context, &checker_owner)?;
        let core_items = validate_functor_core_shape(source_bindings.context(), &checker_owner)?;
        let mut items = SourceFunctorCoreItemAssociationTable::empty();
        for ((definition_id, definition), core_item) in
            checker_owner.definitions().iter().zip(core_items)
        {
            let source_item = source_context
                .context_links()
                .get(definition.context())
                .and_then(|link| link.item)
                .ok_or(SourceFunctorCoreContextError::InvalidItemAssociation)?;
            items.rows.push(SourceFunctorCoreItemAssociation {
                source_item,
                definition: definition_id,
                symbol: definition.symbol().clone(),
                core_item,
            });
        }
        let handoff = SourceFunctorCoreContextHandoff {
            source_bindings,
            source_context,
            checker_owner,
            items,
        };
        handoff.validate()?;
        Ok(handoff)
    }
}

fn validate_functor_core_environment(
    source_bindings: &SourceBindingCoreContextHandoff,
    source_context: &SourceBindingContextHandoff,
    checker_owner: &SourceFunctorDefinitionHandoff,
) -> Result<(), SourceFunctorCoreContextError> {
    let source_id = source_bindings.source_id();
    let module_id = source_bindings.module_id();
    if source_context.source_id() != source_id
        || source_context.module_id() != module_id
        || checker_owner.source_id() != source_id
        || checker_owner.module_id() != module_id
        || source_bindings.binding_env().source_id() != source_id
        || source_bindings.binding_env().module_id() != module_id
        || source_context.binding_env().source_id() != source_id
        || source_context.binding_env().module_id() != module_id
        || source_context.binding_env() != source_bindings.binding_env()
        || source_bindings.context().source_id() != source_id
        || source_bindings.context().module_id() != module_id
    {
        return Err(SourceFunctorCoreContextError::EnvironmentMismatch);
    }
    Ok(())
}

fn validate_functor_checker_owner(
    source_context: &SourceBindingContextHandoff,
    checker_owner: &SourceFunctorDefinitionHandoff,
) -> Result<(), SourceFunctorCoreContextError> {
    if checker_owner.source_context_fingerprint() != source_context.debug_text()
        || checker_owner.source_type_fingerprint().is_empty()
        || checker_owner.source_term_fingerprint().is_empty()
        || checker_owner.application_fingerprint().is_some()
        || checker_owner.structure_fingerprint().is_some()
        || checker_owner.set_term_fingerprint().is_some()
        || checker_owner
            .atomic_formula_fingerprint()
            .is_none_or(str::is_empty)
        || source_context.items().len() != 1
        || source_context.declarations().len() != 2
        || source_context.context_links().len() != 2
        || source_context.local_contexts().len() != 2
        || source_context.binding_env().contexts().len() != 2
        || source_context.binding_env().bindings().len() != 2
        || !source_context.binding_env().diagnostics().is_empty()
    {
        return Err(SourceFunctorCoreContextError::InvalidCheckerOwner);
    }

    let source_item = source_context
        .items()
        .get(SourceItemId::new(0))
        .ok_or(SourceFunctorCoreContextError::InvalidCheckerOwner)?;
    if source_item.id != SourceItemId::new(0)
        || source_item.role != SourceItemRole::DefinitionBlock
        || source_item.shell_ordinal != 0
        || source_item.visibility != SourceItemVisibility::Unspecified
        || source_item.recovery != SourceItemRecovery::Normal
        || source_item.parent.is_some()
        || source_item
            .local_scope
            .as_ref()
            .is_none_or(|scope| scope.path() != [0])
        || source_item.predecessor.is_some()
        || source_item.binding_context != BindingContextId::new(1)
        || source_item.local_context != mizar_checker::typed_ast::LocalTypeContextId::new(1)
        || source_item.source_range
            != (SourceRange {
                source_id: source_context.source_id(),
                start: 0,
                end: 261,
            })
        || source_item.site
            != mizar_checker::typed_ast::TypedSiteRef::Node(
                mizar_checker::typed_ast::TypedNodeId::new(104),
            )
        || source_item.source_range.source_id != source_context.source_id()
    {
        return Err(SourceFunctorCoreContextError::InvalidCheckerOwner);
    }

    let module_context = source_context
        .binding_env()
        .contexts()
        .get(BindingContextId::new(0))
        .ok_or(SourceFunctorCoreContextError::InvalidCheckerOwner)?;
    let definition_context = source_context
        .binding_env()
        .contexts()
        .get(BindingContextId::new(1))
        .ok_or(SourceFunctorCoreContextError::InvalidCheckerOwner)?;
    if module_context.id != BindingContextId::new(0)
        || !is_normal_module_context(module_context)
        || definition_context.id != BindingContextId::new(1)
        || !is_normal_declaration_context(definition_context)
        || definition_context.owner != BindingContextOwner::DeclarationShell(source_item.shell)
        || definition_context.parent != Some(BindingContextId::new(0))
        || definition_context.lexical_scope != source_item.local_scope
    {
        return Err(SourceFunctorCoreContextError::InvalidCheckerOwner);
    }

    let declaration_rows = source_context.declarations().iter().collect::<Vec<_>>();
    let declaration_validity = declaration_rows
        .iter()
        .map(|(id, declaration)| {
            declaration.id == *id
                && declaration.item == source_item.id
                && declaration.binding_context == BindingContextId::new(1)
                && declaration.local_context == mizar_checker::typed_ast::LocalTypeContextId::new(1)
                && declaration.predecessor
                    == id.index().checked_sub(1).map(SourceDeclarationId::new)
                && declaration.declaration_range.source_id == source_context.source_id()
                && declaration.written_type_range.source_id == source_context.source_id()
                && matches!(
                    declaration.role,
                    SourceBindingSiteRole::DefinitionParameter { .. }
                )
        })
        .collect::<Vec<_>>();
    if declaration_rows.len() != 2
        || declaration_validity.iter().any(|valid| !valid)
        || declaration_rows[0].1.source_ordinal != 0
        || declaration_rows[1].1.source_ordinal != 1
        || declaration_rows[0].1.source_ordinal >= declaration_rows[1].1.source_ordinal
        || definition_context.bindings
            != declaration_rows
                .iter()
                .map(|(_, row)| row.binding)
                .collect::<Vec<_>>()
        || definition_context.visible_bindings != definition_context.bindings
    {
        return Err(SourceFunctorCoreContextError::InvalidCheckerOwner);
    }
    for (index, (_, declaration)) in declaration_rows.iter().enumerate() {
        let binding = source_context
            .binding_env()
            .bindings()
            .get(declaration.binding)
            .ok_or(SourceFunctorCoreContextError::InvalidCheckerOwner)?;
        if declaration.binding != binding.id
            || binding.owner_context != BindingContextId::new(1)
            || binding.kind != BindingKind::DefinitionParameter
            || binding.status != BindingStatus::Active
            || binding.recovery != BindingRecoveryState::Normal
            || binding.declaration_range != declaration.declaration_range
            || binding.visible_after_ordinal != declaration.source_ordinal
            || binding.spelling != declaration.spelling
            || index != declaration.source_ordinal
        {
            return Err(SourceFunctorCoreContextError::InvalidCheckerOwner);
        }
    }

    let module_local_context = source_context
        .local_contexts()
        .get(mizar_checker::typed_ast::LocalTypeContextId::new(0))
        .ok_or(SourceFunctorCoreContextError::InvalidCheckerOwner)?;
    let definition_local_context = source_context
        .local_contexts()
        .get(mizar_checker::typed_ast::LocalTypeContextId::new(1))
        .ok_or(SourceFunctorCoreContextError::InvalidCheckerOwner)?;
    let expected_local_bindings = declaration_rows
        .iter()
        .map(|(_, declaration)| BindingTypeRef::Site(declaration.site.clone()))
        .collect::<Vec<_>>();
    if module_local_context.id != mizar_checker::typed_ast::LocalTypeContextId::new(0)
        || module_local_context.parent.is_some()
        || module_local_context.layer != mizar_checker::typed_ast::TypeContextLayer::Module
        || module_local_context.recovery != mizar_checker::typed_ast::ContextRecoveryState::Normal
        || !module_local_context.bindings.is_empty()
        || !module_local_context.introduced_assumptions.is_empty()
        || !module_local_context.visible_facts.is_empty()
        || definition_local_context.id != mizar_checker::typed_ast::LocalTypeContextId::new(1)
        || definition_local_context.owner != source_item.site
        || definition_local_context.parent
            != Some(mizar_checker::typed_ast::LocalTypeContextId::new(0))
        || definition_local_context.layer != mizar_checker::typed_ast::TypeContextLayer::Declaration
        || definition_local_context.recovery
            != mizar_checker::typed_ast::ContextRecoveryState::Normal
        || definition_local_context.bindings != expected_local_bindings
        || !definition_local_context.introduced_assumptions.is_empty()
        || !definition_local_context.visible_facts.is_empty()
    {
        return Err(SourceFunctorCoreContextError::InvalidCheckerOwner);
    }

    for (index, (link_id, link)) in source_context.context_links().iter().enumerate() {
        let expected_item = (index == 1).then_some(source_item.id);
        if link_id != index
            || link.binding_context != BindingContextId::new(index)
            || link.local_context != mizar_checker::typed_ast::LocalTypeContextId::new(index)
            || link.item != expected_item
        {
            return Err(SourceFunctorCoreContextError::InvalidCheckerOwner);
        }
    }

    if checker_owner.definitions().len() != 2
        || checker_owner.parameters().len() != 2
        || checker_owner.guards().len() != 1
        || checker_owner.definientia().len() != 2
        || checker_owner.correctness().len() != 2
    {
        return Err(SourceFunctorCoreContextError::InvalidCheckerOwner);
    }
    let definitions = [
        checker_owner
            .definitions()
            .get(SourceFunctorDefinitionId::new(0))
            .ok_or(SourceFunctorCoreContextError::InvalidCheckerOwner)?,
        checker_owner
            .definitions()
            .get(SourceFunctorDefinitionId::new(1))
            .ok_or(SourceFunctorCoreContextError::InvalidCheckerOwner)?,
    ];
    let expected_ranges = [(61, 118), (121, 179)];
    let expected_sites = [84, 95];
    let expected_styles = [
        SourceFunctorDefinitionStyle::Equals,
        SourceFunctorDefinitionStyle::Means,
    ];
    let expected_spelling = [
        "func Task260EqualsDef: task260_equals(x) -> set equals x;",
        "func Task260MeansDef: task260_means(y) -> set means x = y;",
    ];
    for (index, definition) in definitions.into_iter().enumerate() {
        let expected_range = SourceRange {
            source_id: source_context.source_id(),
            start: expected_ranges[index].0,
            end: expected_ranges[index].1,
        };
        let expected_origin_path = [4_u32, 0, 9, index as u32];
        if definition.id() != SourceFunctorDefinitionId::new(index)
            || definition.definition().index() != index
            || definition.contribution().index() != 0
            || definition.source_ordinal() != index
            || definition.context() != BindingContextId::new(1)
            || definition.recovery() != SourceFunctorDefinitionRecovery::Normal
            || definition.source_range() != expected_range
            || definition.site()
                != &mizar_checker::typed_ast::TypedSiteRef::Node(
                    mizar_checker::typed_ast::TypedNodeId::new(expected_sites[index]),
                )
            || definition.spelling() != expected_spelling[index]
            || definition.style() != expected_styles[index]
            || definition.return_type().index() != index
            || definition.definiens().index() != index
            || definition.origin().source_id() != source_context.source_id()
            || definition.origin().module_id() != source_context.module_id()
            || definition.origin().is_recovered()
            || definition.origin().import_edge().is_some()
            || definition.origin().anchor() != &SourceAnchor::Range(expected_range)
            || definition.origin().structural_path() != expected_origin_path
        {
            return Err(SourceFunctorCoreContextError::InvalidCheckerOwner);
        }
    }

    let parameter_ranges = [(13, 26, 17, 18), (29, 42, 33, 34)];
    let parameter_sites = [65, 69];
    let parameter_spelling = ["let x be set;", "let y be set;"];
    for (index, (_, parameter)) in checker_owner.parameters().iter().enumerate() {
        let (_, declaration) = declaration_rows
            .get(index)
            .ok_or(SourceFunctorCoreContextError::InvalidCheckerOwner)?;
        let source_range = SourceRange {
            source_id: source_context.source_id(),
            start: parameter_ranges[index].0,
            end: parameter_ranges[index].1,
        };
        let declaration_range = SourceRange {
            source_id: source_context.source_id(),
            start: parameter_ranges[index].2,
            end: parameter_ranges[index].3,
        };
        if parameter.id().index() != index
            || parameter.ordinal() != index
            || parameter.binding() != declaration.binding
            || parameter.written_type().index() != index
            || parameter.site()
                != &mizar_checker::typed_ast::TypedSiteRef::Node(
                    mizar_checker::typed_ast::TypedNodeId::new(parameter_sites[index]),
                )
            || parameter.site() != &declaration.site
            || parameter.source_range() != source_range
            || parameter.declaration_range() != declaration_range
            || parameter.declaration_range() != declaration.declaration_range
            || parameter.context() != BindingContextId::new(1)
            || parameter.recovery() != SourceFunctorDefinitionRecovery::Normal
            || parameter.spelling() != parameter_spelling[index]
        {
            return Err(SourceFunctorCoreContextError::InvalidCheckerOwner);
        }
    }

    let guard = checker_owner
        .guards()
        .get(mizar_checker::source_functor_definition::SourceFunctorGuardId::new(0))
        .ok_or(SourceFunctorCoreContextError::InvalidCheckerOwner)?;
    if guard.id() != mizar_checker::source_functor_definition::SourceFunctorGuardId::new(0)
        || guard.ordinal() != 0
        || guard.formula().index() != 0
        || guard.site()
            != &mizar_checker::typed_ast::TypedSiteRef::Node(
                mizar_checker::typed_ast::TypedNodeId::new(77),
            )
        || guard.source_range()
            != (SourceRange {
                source_id: source_context.source_id(),
                start: 45,
                end: 58,
            })
        || guard.context() != BindingContextId::new(1)
        || guard.recovery() != SourceFunctorDefinitionRecovery::Normal
        || guard.spelling() != "assume x = x;"
    {
        return Err(SourceFunctorCoreContextError::InvalidCheckerOwner);
    }

    let definiens_targets = [
        SourceFunctorDefiniensTarget::Primary(
            mizar_checker::source_term::SourcePrimaryTermId::new(2),
        ),
        SourceFunctorDefiniensTarget::AtomicFormula(
            mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(1),
        ),
    ];
    let definiens_sites = [83, 94];
    let definiens_ranges = [(116, 117), (173, 178)];
    let definiens_spelling = ["x", "x = y"];
    for (index, (_, definiens)) in checker_owner.definientia().iter().enumerate() {
        if definiens.id().index() != index
            || definiens.owner() != SourceFunctorDefinitionId::new(index)
            || definiens.ordinal() != index
            || definiens.target() != definiens_targets[index]
            || definiens.site()
                != &mizar_checker::typed_ast::TypedSiteRef::Node(
                    mizar_checker::typed_ast::TypedNodeId::new(definiens_sites[index]),
                )
            || definiens.source_range()
                != (SourceRange {
                    source_id: source_context.source_id(),
                    start: definiens_ranges[index].0,
                    end: definiens_ranges[index].1,
                })
            || definiens.context() != BindingContextId::new(1)
            || definiens.recovery() != SourceFunctorDefinitionRecovery::Normal
            || definiens.spelling() != definiens_spelling[index]
        {
            return Err(SourceFunctorCoreContextError::InvalidCheckerOwner);
        }
    }

    let correctness_kinds = [
        SourceFunctorCorrectnessKind::Existence,
        SourceFunctorCorrectnessKind::Uniqueness,
    ];
    let correctness_sites = [99, 103];
    let correctness_ranges = [(182, 217, 192, 216), (220, 256, 231, 255)];
    let correctness_spelling = [
        "existence by computation(steps: 1);",
        "uniqueness by computation(steps: 1);",
    ];
    for (index, (_, correctness)) in checker_owner.correctness().iter().enumerate() {
        let source_range = SourceRange {
            source_id: source_context.source_id(),
            start: correctness_ranges[index].0,
            end: correctness_ranges[index].1,
        };
        let justification = SourceRange {
            source_id: source_context.source_id(),
            start: correctness_ranges[index].2,
            end: correctness_ranges[index].3,
        };
        if correctness.id().index() != index
            || correctness.owner() != SourceFunctorDefinitionId::new(1)
            || correctness.ordinal() != index
            || correctness.kind() != correctness_kinds[index]
            || correctness.site()
                != &mizar_checker::typed_ast::TypedSiteRef::Node(
                    mizar_checker::typed_ast::TypedNodeId::new(correctness_sites[index]),
                )
            || correctness.source_range() != source_range
            || correctness.justification() != &SourceAnchor::Range(justification)
            || correctness.recovery() != SourceFunctorDefinitionRecovery::Normal
            || correctness.spelling() != correctness_spelling[index]
            || correctness.obligation().index() != index
        {
            return Err(SourceFunctorCoreContextError::InvalidCheckerOwner);
        }
    }
    Ok(())
}

fn functor_core_provenance_key(definition: SourceFunctorDefinitionId) -> Option<CoreProvenanceKey> {
    if definition == SourceFunctorDefinitionId::new(0) {
        Some(CoreProvenanceKey::new(
            SOURCE_FUNCTOR_CORE_ITEM_PROVENANCE_KEYS[0],
        ))
    } else if definition == SourceFunctorDefinitionId::new(1) {
        Some(CoreProvenanceKey::new(
            SOURCE_FUNCTOR_CORE_ITEM_PROVENANCE_KEYS[1],
        ))
    } else {
        None
    }
}

fn validate_functor_core_shape(
    context: &CoreContext,
    checker_owner: &SourceFunctorDefinitionHandoff,
) -> Result<[CoreItemId; 2], SourceFunctorCoreContextError> {
    if validate_core_context_shape(context, &BTreeSet::new()).is_err()
        || !context.dependency_summaries.is_empty()
        || !context.generated_origins.table().is_empty()
        || !context.generated_origins.by_key.is_empty()
        || !context.diagnostics.is_empty()
        || context.source_map.item_sources.len() != 2
        || !context.source_map.term_sources.is_empty()
        || !context.source_map.formula_sources.is_empty()
        || !context.source_map.definition_sources.is_empty()
        || !context.source_map.proof_sources.is_empty()
        || !context.source_map.algorithm_sources.is_empty()
        || !context.source_map.generated_sources.is_empty()
        || !context.source_map.obligation_sources.is_empty()
        || context.item_registry.items.len() != 2
        || context.item_registry.by_symbol.len() != 2
        || context.item_registry.dependencies.len() != 2
        || context.definition_boundaries.by_item.len() != 2
        || context.definition_boundaries.by_symbol.len() != 2
        || context.worklist.entries.len() != 2
    {
        return Err(SourceFunctorCoreContextError::InvalidCoreContext);
    }
    let definitions = [
        checker_owner
            .definitions()
            .get(SourceFunctorDefinitionId::new(0))
            .ok_or(SourceFunctorCoreContextError::InvalidCoreContext)?,
        checker_owner
            .definitions()
            .get(SourceFunctorDefinitionId::new(1))
            .ok_or(SourceFunctorCoreContextError::InvalidCoreContext)?,
    ];
    let mut core_items = [CoreItemId::new(0), CoreItemId::new(0)];
    for (index, definition) in definitions.into_iter().enumerate() {
        let core_item = context
            .item_registry
            .id_for_symbol(definition.symbol())
            .ok_or(SourceFunctorCoreContextError::InvalidCoreContext)?;
        if index == 1 && core_item == core_items[0] {
            return Err(SourceFunctorCoreContextError::InvalidCoreContext);
        }
        core_items[index] = core_item;
        let key = functor_core_provenance_key(definition.id())
            .ok_or(SourceFunctorCoreContextError::InvalidCoreContext)?;
        let provenance = CoreProvenance::new(CoreProvenancePhase::Checker, key.clone());
        let expected_source = CoreSourceRef::direct(definition.source_range())
            .with_provenance(vec![provenance.clone()]);
        let item = context
            .item_registry
            .items
            .get(core_item)
            .ok_or(SourceFunctorCoreContextError::InvalidCoreContext)?;
        if item.symbol != *definition.symbol()
            || item.kind != CoreItemKind::Functor
            || item.visibility.as_str() != "public"
            || item.status != CoreItemStatus::Valid
            || !item.dependencies.is_empty()
            || !item.diagnostics.is_empty()
            || item.source != expected_source
            || context.source_map.item_sources.get(&core_item) != Some(&item.source)
        {
            return Err(SourceFunctorCoreContextError::InvalidCoreContext);
        }
        let dependency = context
            .item_registry
            .dependencies
            .get(&core_item)
            .ok_or(SourceFunctorCoreContextError::InvalidCoreContext)?;
        if !dependency.local.is_empty()
            || !dependency.external.is_empty()
            || !dependency.missing.is_empty()
        {
            return Err(SourceFunctorCoreContextError::InvalidCoreContext);
        }
        let boundary = context
            .definition_boundaries
            .by_item
            .get(&core_item)
            .ok_or(SourceFunctorCoreContextError::InvalidCoreContext)?;
        if context
            .definition_boundaries
            .by_symbol
            .get(definition.symbol())
            != Some(&core_item)
            || boundary.item != core_item
            || boundary.symbol != *definition.symbol()
            || boundary.kind != DefinitionBoundaryKind::DefinitionalItem
            || boundary.status != DefinitionBoundaryStatus::PendingBody
            || boundary.source != expected_source
            || boundary.provenance.as_slice() != [provenance.clone()]
        {
            return Err(SourceFunctorCoreContextError::InvalidCoreContext);
        }
    }
    for (index, core_item) in core_items.into_iter().enumerate() {
        let definition = definitions[index];
        let key = functor_core_provenance_key(definition.id())
            .ok_or(SourceFunctorCoreContextError::InvalidCoreContext)?;
        let expected_source = CoreSourceRef::direct(definition.source_range())
            .with_provenance(vec![CoreProvenance::new(CoreProvenancePhase::Checker, key)]);
        let work_item = context
            .worklist
            .entries
            .get(index)
            .ok_or(SourceFunctorCoreContextError::InvalidCoreContext)?;
        if work_item.kind != ElaborationWorkItemKind::Item(core_item)
            || work_item.status != ElaborationWorkStatus::Pending
            || work_item.source != expected_source
            || !work_item.diagnostics.is_empty()
            || !work_item.checker_diagnostics.is_empty()
        {
            return Err(SourceFunctorCoreContextError::InvalidCoreContext);
        }
    }
    Ok(core_items)
}

fn validate_functor_item_association(
    source_context: &SourceBindingContextHandoff,
    checker_owner: &SourceFunctorDefinitionHandoff,
    items: &SourceFunctorCoreItemAssociationTable,
    core_items: &[CoreItemId; 2],
) -> Result<(), SourceFunctorCoreContextError> {
    if items.len() != 2 || items.is_empty() || checker_owner.definitions().len() != 2 {
        return Err(SourceFunctorCoreContextError::InvalidItemAssociation);
    }
    let source_item = SourceItemId::new(0);
    for (index, core_item) in core_items.iter().copied().enumerate() {
        let definition_id = SourceFunctorDefinitionId::new(index);
        let definition = checker_owner
            .definitions()
            .get(definition_id)
            .ok_or(SourceFunctorCoreContextError::InvalidItemAssociation)?;
        let link = source_context
            .context_links()
            .get(definition.context())
            .ok_or(SourceFunctorCoreContextError::InvalidItemAssociation)?;
        if link.item != Some(source_item) {
            return Err(SourceFunctorCoreContextError::InvalidItemAssociation);
        }
        let association = items
            .get(definition_id)
            .ok_or(SourceFunctorCoreContextError::InvalidItemAssociation)?;
        if association.definition() != definition_id
            || association.source_item() != source_item
            || association.symbol() != definition.symbol()
            || association.core_item() != core_item
        {
            return Err(SourceFunctorCoreContextError::InvalidItemAssociation);
        }
        let Some((row_id, row)) = items.iter().nth(index) else {
            return Err(SourceFunctorCoreContextError::InvalidItemAssociation);
        };
        if row_id != definition_id || row != association {
            return Err(SourceFunctorCoreContextError::InvalidItemAssociation);
        }
    }
    Ok(())
}

const SOURCE_ATTRIBUTE_CORE_ITEM_PROVENANCE_KEY: &str =
    "source-attribute-core-item-v1.definition.0";

/// One immutable association between a checker source item, its attribute
/// definition, the definition's whole symbol, and the corresponding Core item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceAttributeCoreItemAssociation {
    source_item: SourceItemId,
    definition: SourceAttributeDefinitionId,
    symbol: SymbolId,
    core_item: CoreItemId,
}

impl SourceAttributeCoreItemAssociation {
    #[must_use]
    pub const fn source_item(&self) -> SourceItemId {
        self.source_item
    }

    #[must_use]
    pub const fn definition(&self) -> SourceAttributeDefinitionId {
        self.definition
    }

    #[must_use]
    pub const fn symbol(&self) -> &SymbolId {
        &self.symbol
    }

    #[must_use]
    pub const fn core_item(&self) -> CoreItemId {
        self.core_item
    }
}

/// Immutable source-ordered table of attribute/Core item associations.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SourceAttributeCoreItemAssociationTable {
    rows: Vec<SourceAttributeCoreItemAssociation>,
}

impl SourceAttributeCoreItemAssociationTable {
    fn empty() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn get(
        &self,
        definition: SourceAttributeDefinitionId,
    ) -> Option<&SourceAttributeCoreItemAssociation> {
        self.rows.iter().find(|row| row.definition() == definition)
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<
        Item = (
            SourceAttributeDefinitionId,
            &SourceAttributeCoreItemAssociation,
        ),
    > {
        self.rows.iter().map(|row| (row.definition(), row))
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// Errors raised while building the exact Task-261 attribute/Core context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceAttributeCoreContextError {
    EnvironmentMismatch,
    InvalidSourceBindingContext,
    InvalidCheckerOwner,
    InvalidCoreContext,
    InvalidItemAssociation,
}

impl fmt::Display for SourceAttributeCoreContextError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EnvironmentMismatch => {
                formatter.write_str("attribute Core context environment is invalid")
            }
            Self::InvalidSourceBindingContext => {
                formatter.write_str("attribute Core source-binding context is invalid")
            }
            Self::InvalidCheckerOwner => {
                formatter.write_str("attribute Core checker owner is invalid")
            }
            Self::InvalidCoreContext => formatter.write_str("attribute Core context is invalid"),
            Self::InvalidItemAssociation => {
                formatter.write_str("attribute Core item association is invalid")
            }
        }
    }
}

impl Error for SourceAttributeCoreContextError {}

/// Immutable Core context handoff for the exact checker-authenticated
/// Task-261 attribute definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceAttributeCoreContextHandoff {
    source_bindings: SourceBindingCoreContextHandoff,
    source_context: SourceBindingContextHandoff,
    checker_owner: SourceAttributeDefinitionHandoff,
    items: SourceAttributeCoreItemAssociationTable,
}

impl SourceAttributeCoreContextHandoff {
    #[must_use]
    pub const fn source_id(&self) -> SourceId {
        self.source_bindings.source_id()
    }

    #[must_use]
    pub const fn module_id(&self) -> &ModuleId {
        self.source_bindings.module_id()
    }

    #[must_use]
    pub const fn context(&self) -> &CoreContext {
        self.source_bindings.context()
    }

    #[must_use]
    pub const fn source_bindings(&self) -> &SourceBindingCoreContextHandoff {
        &self.source_bindings
    }

    #[must_use]
    pub const fn source_context(&self) -> &SourceBindingContextHandoff {
        &self.source_context
    }

    #[must_use]
    pub const fn checker_owner(&self) -> &SourceAttributeDefinitionHandoff {
        &self.checker_owner
    }

    #[must_use]
    pub const fn items(&self) -> &SourceAttributeCoreItemAssociationTable {
        &self.items
    }

    #[must_use]
    pub fn debug_text(&self) -> String {
        let associations = self
            .items
            .iter()
            .map(|(definition, association)| {
                format!(
                    "{}:{}:{}:{}",
                    definition.index(),
                    association.source_item().index(),
                    association.symbol().fqn().as_str(),
                    association.core_item().index(),
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "source-attribute-core-item-context-v1|module={}.{}|source-bindings={}|definitions={}|items={}",
            self.module_id().package().as_str(),
            self.module_id().path().as_str(),
            self.source_bindings.binding_env().bindings().len(),
            self.checker_owner.definitions().len(),
            associations,
        )
    }

    fn validate(&self) -> Result<(), SourceAttributeCoreContextError> {
        validate_attribute_core_environment(
            &self.source_bindings,
            &self.source_context,
            &self.checker_owner,
        )?;
        self.source_bindings
            .validate()
            .map_err(|_| SourceAttributeCoreContextError::InvalidSourceBindingContext)?;
        let definition =
            validate_attribute_checker_owner(&self.source_context, &self.checker_owner)?;
        let core_item = validate_attribute_core_shape(
            self.source_bindings.context(),
            definition.symbol(),
            definition.source_range(),
        )?;
        validate_attribute_item_association(
            &self.source_context,
            &self.checker_owner,
            &self.items,
            definition,
            core_item,
        )
    }
}

/// Builds the standalone immutable Task-261 attribute/Core context handoff.
#[derive(Debug, Clone, Copy)]
pub struct SourceAttributeCoreContextProducer;

impl SourceAttributeCoreContextProducer {
    pub fn build(
        source_bindings: SourceBindingCoreContextHandoff,
        source_context: SourceBindingContextHandoff,
        checker_owner: SourceAttributeDefinitionHandoff,
    ) -> Result<SourceAttributeCoreContextHandoff, SourceAttributeCoreContextError> {
        validate_attribute_core_environment(&source_bindings, &source_context, &checker_owner)?;
        source_bindings
            .validate()
            .map_err(|_| SourceAttributeCoreContextError::InvalidSourceBindingContext)?;
        let definition = validate_attribute_checker_owner(&source_context, &checker_owner)?;
        let core_item = validate_attribute_core_shape(
            source_bindings.context(),
            definition.symbol(),
            definition.source_range(),
        )?;
        let source_item = source_context
            .context_links()
            .get(definition.context())
            .and_then(|link| link.item)
            .ok_or(SourceAttributeCoreContextError::InvalidItemAssociation)?;
        let mut items = SourceAttributeCoreItemAssociationTable::empty();
        items.rows.push(SourceAttributeCoreItemAssociation {
            source_item,
            definition: definition.id(),
            symbol: definition.symbol().clone(),
            core_item,
        });
        let handoff = SourceAttributeCoreContextHandoff {
            source_bindings,
            source_context,
            checker_owner,
            items,
        };
        handoff.validate()?;
        Ok(handoff)
    }
}

fn validate_attribute_core_environment(
    source_bindings: &SourceBindingCoreContextHandoff,
    source_context: &SourceBindingContextHandoff,
    checker_owner: &SourceAttributeDefinitionHandoff,
) -> Result<(), SourceAttributeCoreContextError> {
    let source_id = source_bindings.source_id();
    let module_id = source_bindings.module_id();
    if source_context.source_id() != source_id
        || source_context.module_id() != module_id
        || checker_owner.source_id() != source_id
        || checker_owner.module_id() != module_id
        || source_bindings.binding_env().source_id() != source_id
        || source_bindings.binding_env().module_id() != module_id
        || source_context.binding_env().source_id() != source_id
        || source_context.binding_env().module_id() != module_id
        || source_context.binding_env() != source_bindings.binding_env()
        || source_bindings.context().source_id() != source_id
        || source_bindings.context().module_id() != module_id
    {
        return Err(SourceAttributeCoreContextError::EnvironmentMismatch);
    }
    Ok(())
}

fn validate_attribute_checker_owner<'a>(
    source_context: &SourceBindingContextHandoff,
    checker_owner: &'a SourceAttributeDefinitionHandoff,
) -> Result<
    &'a mizar_checker::source_attribute_definition::SourceAttributeDefinition,
    SourceAttributeCoreContextError,
> {
    if checker_owner.source_context_fingerprint() != source_context.debug_text()
        || checker_owner.source_type_fingerprint().is_empty()
        || checker_owner.source_term_fingerprint().is_empty()
        || checker_owner.source_atomic_formula_fingerprint().is_empty()
        || source_context.items().len() != 1
        || source_context.declarations().len() != 2
        || source_context.context_links().len() != 2
        || source_context.local_contexts().len() != 2
        || source_context.binding_env().contexts().len() != 2
        || source_context.binding_env().bindings().len() != 2
        || !source_context.binding_env().diagnostics().is_empty()
    {
        return Err(SourceAttributeCoreContextError::InvalidCheckerOwner);
    }

    let source_item = source_context
        .items()
        .get(SourceItemId::new(0))
        .ok_or(SourceAttributeCoreContextError::InvalidCheckerOwner)?;
    if source_item.id != SourceItemId::new(0)
        || source_item.shell.index() != 0
        || source_item.role != SourceItemRole::DefinitionBlock
        || source_item.shell_ordinal != 0
        || source_item.visibility != SourceItemVisibility::Unspecified
        || source_item.recovery != SourceItemRecovery::Normal
        || source_item.parent.is_some()
        || source_item
            .local_scope
            .as_ref()
            .is_none_or(|scope| scope.path() != [0])
        || source_item.predecessor.is_some()
        || source_item.binding_context != BindingContextId::new(1)
        || source_item.local_context != mizar_checker::typed_ast::LocalTypeContextId::new(1)
        || source_item.source_range
            != (SourceRange {
                source_id: source_context.source_id(),
                start: 0,
                end: 115,
            })
        || source_item.site
            != mizar_checker::typed_ast::TypedSiteRef::Node(
                mizar_checker::typed_ast::TypedNodeId::new(41),
            )
    {
        return Err(SourceAttributeCoreContextError::InvalidCheckerOwner);
    }

    let module_context = source_context
        .binding_env()
        .contexts()
        .get(BindingContextId::new(0))
        .ok_or(SourceAttributeCoreContextError::InvalidCheckerOwner)?;
    let definition_context = source_context
        .binding_env()
        .contexts()
        .get(BindingContextId::new(1))
        .ok_or(SourceAttributeCoreContextError::InvalidCheckerOwner)?;
    if module_context.id != BindingContextId::new(0)
        || !is_normal_module_context(module_context)
        || definition_context.id != BindingContextId::new(1)
        || !is_normal_declaration_context(definition_context)
        || definition_context.owner != BindingContextOwner::DeclarationShell(source_item.shell)
        || definition_context.parent != Some(BindingContextId::new(0))
        || definition_context.lexical_scope != source_item.local_scope
        || !module_context.bindings.is_empty()
        || !module_context.visible_bindings.is_empty()
    {
        return Err(SourceAttributeCoreContextError::InvalidCheckerOwner);
    }

    let declaration_rows = source_context.declarations().iter().collect::<Vec<_>>();
    if declaration_rows.len() != 2 {
        return Err(SourceAttributeCoreContextError::InvalidCheckerOwner);
    }
    let expected_parameter_ranges = [(13, 26, 17, 18, 22, 25), (29, 42, 33, 34, 38, 41)];
    let expected_parameter_sites = [27, 31];
    let expected_parameter_spelling = ["x", "y"];
    for (index, (id, declaration)) in declaration_rows.iter().enumerate() {
        let expected_source_range = SourceRange {
            source_id: source_context.source_id(),
            start: expected_parameter_ranges[index].0,
            end: expected_parameter_ranges[index].1,
        };
        let expected_declaration_range = SourceRange {
            source_id: source_context.source_id(),
            start: expected_parameter_ranges[index].2,
            end: expected_parameter_ranges[index].3,
        };
        let expected_written_type_range = SourceRange {
            source_id: source_context.source_id(),
            start: expected_parameter_ranges[index].4,
            end: expected_parameter_ranges[index].5,
        };
        let SourceBindingSiteRole::DefinitionParameter { local } = &declaration.role else {
            return Err(SourceAttributeCoreContextError::InvalidCheckerOwner);
        };
        if *id != SourceDeclarationId::new(index)
            || declaration.id != *id
            || declaration.item != source_item.id
            || declaration.binding != BindingId::new(index)
            || declaration.source_ordinal != index
            || declaration.spelling != expected_parameter_spelling[index]
            || declaration.declaration_range != expected_declaration_range
            || declaration.written_type_range != expected_written_type_range
            || declaration.site
                != mizar_checker::typed_ast::TypedSiteRef::Node(
                    mizar_checker::typed_ast::TypedNodeId::new(expected_parameter_sites[index]),
                )
            || declaration.binding_context != BindingContextId::new(1)
            || declaration.local_context != mizar_checker::typed_ast::LocalTypeContextId::new(1)
            || declaration.shadowed_binding.is_some()
            || declaration.predecessor != index.checked_sub(1).map(SourceDeclarationId::new)
            || local.spelling() != expected_parameter_spelling[index]
            || local.scope().path() != [0]
            || local.declaration_range() != expected_declaration_range
            || local.visible_after_ordinal() != index
            || !range_contains(source_item.source_range, expected_source_range)
        {
            return Err(SourceAttributeCoreContextError::InvalidCheckerOwner);
        }

        let binding = source_context
            .binding_env()
            .bindings()
            .get(declaration.binding)
            .ok_or(SourceAttributeCoreContextError::InvalidCheckerOwner)?;
        let BinderIdentity::ResolverLocal {
            scope,
            ordinal,
            declaration_range,
        } = &binding.identity
        else {
            return Err(SourceAttributeCoreContextError::InvalidCheckerOwner);
        };
        if binding.id != BindingId::new(index)
            || binding.spelling != expected_parameter_spelling[index]
            || binding.kind != BindingKind::DefinitionParameter
            || binding.owner_context != BindingContextId::new(1)
            || binding.declaration_range != expected_declaration_range
            || binding.visible_after_ordinal != index
            || binding.type_site != BindingTypeSite::Source(expected_written_type_range)
            || binding.status != BindingStatus::Active
            || !binding.captured.identities().is_empty()
            || !binding.diagnostics.is_empty()
            || binding.recovery != BindingRecoveryState::Normal
            || scope.path() != [0]
            || *ordinal != index
            || *declaration_range != expected_declaration_range
        {
            return Err(SourceAttributeCoreContextError::InvalidCheckerOwner);
        }
    }
    let expected_bindings = vec![BindingId::new(0), BindingId::new(1)];
    if definition_context.bindings != expected_bindings
        || definition_context.visible_bindings != expected_bindings
    {
        return Err(SourceAttributeCoreContextError::InvalidCheckerOwner);
    }

    let module_local_context = source_context
        .local_contexts()
        .get(mizar_checker::typed_ast::LocalTypeContextId::new(0))
        .ok_or(SourceAttributeCoreContextError::InvalidCheckerOwner)?;
    let definition_local_context = source_context
        .local_contexts()
        .get(mizar_checker::typed_ast::LocalTypeContextId::new(1))
        .ok_or(SourceAttributeCoreContextError::InvalidCheckerOwner)?;
    let expected_local_bindings = declaration_rows
        .iter()
        .map(|(_, declaration)| BindingTypeRef::Site(declaration.site.clone()))
        .collect::<Vec<_>>();
    if module_local_context.id != mizar_checker::typed_ast::LocalTypeContextId::new(0)
        || module_local_context.parent.is_some()
        || module_local_context.owner
            != mizar_checker::typed_ast::TypedSiteRef::Node(
                mizar_checker::typed_ast::TypedNodeId::new(44),
            )
        || module_local_context.layer != mizar_checker::typed_ast::TypeContextLayer::Module
        || module_local_context.recovery != mizar_checker::typed_ast::ContextRecoveryState::Normal
        || !module_local_context.bindings.is_empty()
        || !module_local_context.introduced_assumptions.is_empty()
        || !module_local_context.visible_facts.is_empty()
        || definition_local_context.id != mizar_checker::typed_ast::LocalTypeContextId::new(1)
        || definition_local_context.owner != source_item.site
        || definition_local_context.parent
            != Some(mizar_checker::typed_ast::LocalTypeContextId::new(0))
        || definition_local_context.layer != mizar_checker::typed_ast::TypeContextLayer::Declaration
        || definition_local_context.recovery
            != mizar_checker::typed_ast::ContextRecoveryState::Normal
        || definition_local_context.bindings != expected_local_bindings
        || !definition_local_context.introduced_assumptions.is_empty()
        || !definition_local_context.visible_facts.is_empty()
    {
        return Err(SourceAttributeCoreContextError::InvalidCheckerOwner);
    }

    for (index, (link_id, link)) in source_context.context_links().iter().enumerate() {
        let expected_item = (index == 1).then_some(source_item.id);
        if link_id != index
            || link.binding_context != BindingContextId::new(index)
            || link.local_context != mizar_checker::typed_ast::LocalTypeContextId::new(index)
            || link.item != expected_item
        {
            return Err(SourceAttributeCoreContextError::InvalidCheckerOwner);
        }
    }

    if checker_owner.definitions().len() != 1
        || checker_owner.parameters().len() != 2
        || checker_owner.subjects().len() != 1
        || checker_owner.definientia().len() != 1
    {
        return Err(SourceAttributeCoreContextError::InvalidCheckerOwner);
    }
    let definition = checker_owner
        .definitions()
        .get(SourceAttributeDefinitionId::new(0))
        .ok_or(SourceAttributeCoreContextError::InvalidCheckerOwner)?;
    let definition_range = SourceRange {
        source_id: source_context.source_id(),
        start: 45,
        end: 110,
    };
    if definition.id() != SourceAttributeDefinitionId::new(0)
        || definition.definition().index() != 0
        || definition.contribution().index() != 0
        || definition.site()
            != &mizar_checker::typed_ast::TypedSiteRef::Node(
                mizar_checker::typed_ast::TypedNodeId::new(40),
            )
        || definition.source_range() != definition_range
        || definition.source_ordinal() != 0
        || definition.context() != BindingContextId::new(1)
        || definition.recovery() != SourceAttributeDefinitionRecovery::Normal
        || definition.spelling()
            != "attr Task261AttributeDefinition: x is task261_marked means x = y;"
        || definition.subject().index() != 0
        || definition.definiens().index() != 0
        || definition.symbol().module() != source_context.module_id()
        || definition.origin().source_id() != source_context.source_id()
        || definition.origin().module_id() != source_context.module_id()
        || definition.origin().is_recovered()
        || definition.origin().import_edge().is_some()
        || definition.origin().anchor() != &SourceAnchor::Range(definition_range)
        || definition.origin().structural_path() != [4, 0, 7, 0]
        || !range_contains(source_item.source_range, definition_range)
    {
        return Err(SourceAttributeCoreContextError::InvalidCheckerOwner);
    }

    let parameter_spelling = ["let x be set;", "let y be set;"];
    for (index, (parameter_id, parameter)) in checker_owner.parameters().iter().enumerate() {
        let declaration = declaration_rows[index].1;
        let expected_source_range = SourceRange {
            source_id: source_context.source_id(),
            start: expected_parameter_ranges[index].0,
            end: expected_parameter_ranges[index].1,
        };
        if parameter_id.index() != index
            || parameter.id().index() != index
            || parameter.owner() != definition.id()
            || parameter.ordinal() != index
            || parameter.binding() != BindingId::new(index)
            || parameter.written_type().index() != index
            || parameter.site() != &declaration.site
            || parameter.source_range() != expected_source_range
            || parameter.declaration_range() != declaration.declaration_range
            || parameter.context() != BindingContextId::new(1)
            || parameter.recovery() != SourceAttributeDefinitionRecovery::Normal
            || parameter.spelling() != parameter_spelling[index]
        {
            return Err(SourceAttributeCoreContextError::InvalidCheckerOwner);
        }
    }

    let subject = checker_owner
        .subjects()
        .get(mizar_checker::source_attribute_definition::SourceAttributeSubjectId::new(0))
        .ok_or(SourceAttributeCoreContextError::InvalidCheckerOwner)?;
    if subject.id().index() != 0
        || subject.owner() != definition.id()
        || subject.binding() != BindingId::new(0)
        || subject.site() != definition.site()
        || subject.source_range()
            != (SourceRange {
                source_id: source_context.source_id(),
                start: 78,
                end: 79,
            })
        || subject.context() != BindingContextId::new(1)
        || subject.recovery() != SourceAttributeDefinitionRecovery::Normal
        || subject.spelling() != "x"
    {
        return Err(SourceAttributeCoreContextError::InvalidCheckerOwner);
    }

    let definiens = checker_owner
        .definientia()
        .get(mizar_checker::source_attribute_definition::SourceAttributeDefiniensId::new(0))
        .ok_or(SourceAttributeCoreContextError::InvalidCheckerOwner)?;
    if definiens.id().index() != 0
        || definiens.owner() != definition.id()
        || definiens.ordinal() != 0
        || definiens.formula().index() != 0
        || definiens.site()
            != &mizar_checker::typed_ast::TypedSiteRef::Node(
                mizar_checker::typed_ast::TypedNodeId::new(39),
            )
        || definiens.source_range()
            != (SourceRange {
                source_id: source_context.source_id(),
                start: 104,
                end: 109,
            })
        || definiens.context() != BindingContextId::new(1)
        || definiens.recovery() != SourceAttributeDefinitionRecovery::Normal
        || definiens.spelling() != "x = y"
    {
        return Err(SourceAttributeCoreContextError::InvalidCheckerOwner);
    }
    Ok(definition)
}

fn validate_attribute_core_shape(
    context: &CoreContext,
    symbol: &SymbolId,
    source_range: SourceRange,
) -> Result<CoreItemId, SourceAttributeCoreContextError> {
    if validate_core_context_shape(context, &BTreeSet::new()).is_err()
        || !context.dependency_summaries.is_empty()
        || !context.generated_origins.table().is_empty()
        || !context.generated_origins.by_key.is_empty()
        || !context.diagnostics.is_empty()
        || context.source_map.item_sources.len() != 1
        || !context.source_map.term_sources.is_empty()
        || !context.source_map.formula_sources.is_empty()
        || !context.source_map.definition_sources.is_empty()
        || !context.source_map.proof_sources.is_empty()
        || !context.source_map.algorithm_sources.is_empty()
        || !context.source_map.generated_sources.is_empty()
        || !context.source_map.obligation_sources.is_empty()
        || context.item_registry.items.len() != 1
        || context.item_registry.by_symbol.len() != 1
        || context.item_registry.dependencies.len() != 1
        || context.definition_boundaries.by_item.len() != 1
        || context.definition_boundaries.by_symbol.len() != 1
        || context.worklist.entries.len() != 1
    {
        return Err(SourceAttributeCoreContextError::InvalidCoreContext);
    }
    let core_item = context
        .item_registry
        .id_for_symbol(symbol)
        .ok_or(SourceAttributeCoreContextError::InvalidCoreContext)?;
    let item = context
        .item_registry
        .items
        .get(core_item)
        .ok_or(SourceAttributeCoreContextError::InvalidCoreContext)?;
    let expected_provenance = CoreProvenance::new(
        CoreProvenancePhase::Checker,
        SOURCE_ATTRIBUTE_CORE_ITEM_PROVENANCE_KEY,
    );
    let expected_source =
        CoreSourceRef::direct(source_range).with_provenance(vec![expected_provenance.clone()]);
    if item.symbol != *symbol
        || item.kind != CoreItemKind::Attribute
        || item.visibility.as_str() != "public"
        || item.status != CoreItemStatus::Valid
        || !item.dependencies.is_empty()
        || !item.diagnostics.is_empty()
        || item.source != expected_source
        || context.source_map.item_sources.get(&core_item) != Some(&item.source)
    {
        return Err(SourceAttributeCoreContextError::InvalidCoreContext);
    }
    let dependency = context
        .item_registry
        .dependencies
        .get(&core_item)
        .ok_or(SourceAttributeCoreContextError::InvalidCoreContext)?;
    if !dependency.local.is_empty()
        || !dependency.external.is_empty()
        || !dependency.missing.is_empty()
    {
        return Err(SourceAttributeCoreContextError::InvalidCoreContext);
    }
    let boundary = context
        .definition_boundaries
        .by_item
        .get(&core_item)
        .ok_or(SourceAttributeCoreContextError::InvalidCoreContext)?;
    if context.definition_boundaries.by_symbol.get(symbol) != Some(&core_item)
        || boundary.item != core_item
        || boundary.symbol != *symbol
        || boundary.kind != DefinitionBoundaryKind::DefinitionalItem
        || boundary.status != DefinitionBoundaryStatus::PendingBody
        || boundary.source != expected_source
        || boundary.provenance.as_slice() != [expected_provenance.clone()]
    {
        return Err(SourceAttributeCoreContextError::InvalidCoreContext);
    }
    let work_item = context
        .worklist
        .entries
        .first()
        .ok_or(SourceAttributeCoreContextError::InvalidCoreContext)?;
    if work_item.kind != ElaborationWorkItemKind::Item(core_item)
        || work_item.status != ElaborationWorkStatus::Pending
        || work_item.source != expected_source
        || !work_item.diagnostics.is_empty()
        || !work_item.checker_diagnostics.is_empty()
    {
        return Err(SourceAttributeCoreContextError::InvalidCoreContext);
    }
    Ok(core_item)
}

fn validate_attribute_item_association(
    source_context: &SourceBindingContextHandoff,
    checker_owner: &SourceAttributeDefinitionHandoff,
    items: &SourceAttributeCoreItemAssociationTable,
    definition: &mizar_checker::source_attribute_definition::SourceAttributeDefinition,
    core_item: CoreItemId,
) -> Result<(), SourceAttributeCoreContextError> {
    if items.len() != checker_owner.definitions().len() || items.len() != 1 {
        return Err(SourceAttributeCoreContextError::InvalidItemAssociation);
    }
    let source_item = source_context
        .context_links()
        .get(definition.context())
        .and_then(|link| link.item)
        .ok_or(SourceAttributeCoreContextError::InvalidItemAssociation)?;
    let association = items
        .get(definition.id())
        .ok_or(SourceAttributeCoreContextError::InvalidItemAssociation)?;
    if association.definition() != definition.id()
        || association.source_item() != source_item
        || association.symbol() != definition.symbol()
        || association.core_item() != core_item
        || items
            .iter()
            .next()
            .is_none_or(|(id, row)| id != definition.id() || row != association)
    {
        return Err(SourceAttributeCoreContextError::InvalidItemAssociation);
    }
    Ok(())
}

const SOURCE_MODE_CORE_ITEM_PROVENANCE_KEY: &str = "source-mode-core-item-v1.definition.0";

/// One immutable association between a checker source item, its mode
/// definition, the definition's whole symbol, and the corresponding Core
/// item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceModeCoreItemAssociation {
    source_item: SourceItemId,
    definition: mizar_checker::source_mode_definition::SourceModeDefinitionId,
    symbol: SymbolId,
    core_item: CoreItemId,
}

impl SourceModeCoreItemAssociation {
    #[must_use]
    pub const fn source_item(&self) -> SourceItemId {
        self.source_item
    }

    #[must_use]
    pub const fn definition(
        &self,
    ) -> mizar_checker::source_mode_definition::SourceModeDefinitionId {
        self.definition
    }

    #[must_use]
    pub const fn symbol(&self) -> &SymbolId {
        &self.symbol
    }

    #[must_use]
    pub const fn core_item(&self) -> CoreItemId {
        self.core_item
    }
}

/// Immutable source-ordered table of mode/Core item associations.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SourceModeCoreItemAssociationTable {
    rows: Vec<SourceModeCoreItemAssociation>,
}

impl SourceModeCoreItemAssociationTable {
    fn empty() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn get(
        &self,
        definition: mizar_checker::source_mode_definition::SourceModeDefinitionId,
    ) -> Option<&SourceModeCoreItemAssociation> {
        self.rows.iter().find(|row| row.definition() == definition)
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<
        Item = (
            mizar_checker::source_mode_definition::SourceModeDefinitionId,
            &SourceModeCoreItemAssociation,
        ),
    > {
        self.rows.iter().map(|row| (row.definition(), row))
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// Errors raised while building the exact Task-262 mode/Core context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceModeCoreContextError {
    EnvironmentMismatch,
    InvalidSourceBindingContext,
    InvalidCheckerOwner,
    InvalidCoreContext,
    InvalidItemAssociation,
}

impl fmt::Display for SourceModeCoreContextError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EnvironmentMismatch => {
                formatter.write_str("mode Core context environment is invalid")
            }
            Self::InvalidSourceBindingContext => {
                formatter.write_str("mode Core source-binding context is invalid")
            }
            Self::InvalidCheckerOwner => formatter.write_str("mode Core checker owner is invalid"),
            Self::InvalidCoreContext => formatter.write_str("mode Core context is invalid"),
            Self::InvalidItemAssociation => {
                formatter.write_str("mode Core item association is invalid")
            }
        }
    }
}

impl Error for SourceModeCoreContextError {}

/// Immutable Core context handoff for the exact checker-authenticated
/// Task-262 mode definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceModeCoreContextHandoff {
    source_bindings: SourceBindingCoreContextHandoff,
    source_context: SourceBindingContextHandoff,
    checker_owner: mizar_checker::source_mode_definition::SourceModeDefinitionHandoff,
    items: SourceModeCoreItemAssociationTable,
}

impl SourceModeCoreContextHandoff {
    #[must_use]
    pub const fn source_id(&self) -> SourceId {
        self.source_bindings.source_id()
    }

    #[must_use]
    pub const fn module_id(&self) -> &ModuleId {
        self.source_bindings.module_id()
    }

    #[must_use]
    pub const fn context(&self) -> &CoreContext {
        self.source_bindings.context()
    }

    #[must_use]
    pub const fn source_bindings(&self) -> &SourceBindingCoreContextHandoff {
        &self.source_bindings
    }

    #[must_use]
    pub const fn source_context(&self) -> &SourceBindingContextHandoff {
        &self.source_context
    }

    #[must_use]
    pub const fn checker_owner(
        &self,
    ) -> &mizar_checker::source_mode_definition::SourceModeDefinitionHandoff {
        &self.checker_owner
    }

    #[must_use]
    pub const fn items(&self) -> &SourceModeCoreItemAssociationTable {
        &self.items
    }

    #[must_use]
    pub fn debug_text(&self) -> String {
        let associations = self
            .items
            .iter()
            .map(|(definition, association)| {
                format!(
                    "{}:{}:{}:{}",
                    definition.index(),
                    association.source_item().index(),
                    association.symbol().fqn().as_str(),
                    association.core_item().index(),
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "source-mode-core-item-context-v1|module={}.{}|source-bindings={}|definitions={}|items={}",
            self.module_id().package().as_str(),
            self.module_id().path().as_str(),
            self.source_bindings.binding_env().bindings().len(),
            self.checker_owner.definitions().len(),
            associations,
        )
    }

    fn validate(&self) -> Result<(), SourceModeCoreContextError> {
        validate_mode_core_environment(
            &self.source_bindings,
            &self.source_context,
            &self.checker_owner,
        )?;
        self.source_bindings
            .validate()
            .map_err(|_| SourceModeCoreContextError::InvalidSourceBindingContext)?;
        let definition = validate_mode_checker_owner(&self.source_context, &self.checker_owner)?;
        let core_item = validate_mode_core_shape(
            self.source_bindings.context(),
            definition.symbol(),
            definition.source_range(),
        )?;
        validate_mode_item_association(
            &self.source_context,
            &self.checker_owner,
            &self.items,
            definition,
            core_item,
        )
    }
}

/// Builds the standalone immutable Task-262 mode/Core context handoff.
#[derive(Debug, Clone, Copy)]
pub struct SourceModeCoreContextProducer;

impl SourceModeCoreContextProducer {
    pub fn build(
        source_bindings: SourceBindingCoreContextHandoff,
        source_context: SourceBindingContextHandoff,
        checker_owner: mizar_checker::source_mode_definition::SourceModeDefinitionHandoff,
    ) -> Result<SourceModeCoreContextHandoff, SourceModeCoreContextError> {
        validate_mode_core_environment(&source_bindings, &source_context, &checker_owner)?;
        source_bindings
            .validate()
            .map_err(|_| SourceModeCoreContextError::InvalidSourceBindingContext)?;
        let definition = validate_mode_checker_owner(&source_context, &checker_owner)?;
        let core_item = validate_mode_core_shape(
            source_bindings.context(),
            definition.symbol(),
            definition.source_range(),
        )?;
        let source_item = source_context
            .context_links()
            .get(definition.context())
            .and_then(|link| link.item)
            .ok_or(SourceModeCoreContextError::InvalidItemAssociation)?;
        let mut items = SourceModeCoreItemAssociationTable::empty();
        items.rows.push(SourceModeCoreItemAssociation {
            source_item,
            definition: definition.id(),
            symbol: definition.symbol().clone(),
            core_item,
        });
        let handoff = SourceModeCoreContextHandoff {
            source_bindings,
            source_context,
            checker_owner,
            items,
        };
        handoff.validate()?;
        Ok(handoff)
    }
}

fn validate_mode_core_environment(
    source_bindings: &SourceBindingCoreContextHandoff,
    source_context: &SourceBindingContextHandoff,
    checker_owner: &mizar_checker::source_mode_definition::SourceModeDefinitionHandoff,
) -> Result<(), SourceModeCoreContextError> {
    let source_id = source_bindings.source_id();
    let module_id = source_bindings.module_id();
    if source_context.source_id() != source_id
        || source_context.module_id() != module_id
        || checker_owner.source_id() != source_id
        || checker_owner.module_id() != module_id
        || source_bindings.binding_env().source_id() != source_id
        || source_bindings.binding_env().module_id() != module_id
        || source_context.binding_env().source_id() != source_id
        || source_context.binding_env().module_id() != module_id
        || source_context.binding_env() != source_bindings.binding_env()
        || source_bindings.context().source_id() != source_id
        || source_bindings.context().module_id() != module_id
    {
        return Err(SourceModeCoreContextError::EnvironmentMismatch);
    }
    Ok(())
}

fn validate_mode_checker_owner<'a>(
    source_context: &SourceBindingContextHandoff,
    checker_owner: &'a mizar_checker::source_mode_definition::SourceModeDefinitionHandoff,
) -> Result<
    &'a mizar_checker::source_mode_definition::SourceModeDefinition,
    SourceModeCoreContextError,
> {
    if checker_owner.source_context_fingerprint() != source_context.debug_text()
        || checker_owner.source_type_fingerprint().is_empty()
        || checker_owner.base_initial_obligation_count() != 0
        || source_context.items().len() != 1
        || source_context.declarations().len() != 2
        || source_context.context_links().len() != 2
        || source_context.local_contexts().len() != 2
        || source_context.binding_env().contexts().len() != 2
        || source_context.binding_env().bindings().len() != 2
        || !source_context.binding_env().diagnostics().is_empty()
    {
        return Err(SourceModeCoreContextError::InvalidCheckerOwner);
    }

    let source_item = source_context
        .items()
        .get(SourceItemId::new(0))
        .ok_or(SourceModeCoreContextError::InvalidCheckerOwner)?;
    if source_item.id != SourceItemId::new(0)
        || source_item.shell.index() != 0
        || source_item.role != SourceItemRole::DefinitionBlock
        || source_item.shell_ordinal != 0
        || source_item.visibility != SourceItemVisibility::Unspecified
        || source_item.recovery != SourceItemRecovery::Normal
        || source_item.parent.is_some()
        || source_item
            .local_scope
            .as_ref()
            .is_none_or(|scope| scope.path() != [0])
        || source_item.predecessor.is_some()
        || source_item.binding_context != BindingContextId::new(1)
        || source_item.local_context != mizar_checker::typed_ast::LocalTypeContextId::new(1)
        || source_item.source_range
            != (SourceRange {
                source_id: source_context.source_id(),
                start: 0,
                end: 140,
            })
        || source_item.site
            != mizar_checker::typed_ast::TypedSiteRef::Node(
                mizar_checker::typed_ast::TypedNodeId::new(50),
            )
    {
        return Err(SourceModeCoreContextError::InvalidCheckerOwner);
    }

    let module_context = source_context
        .binding_env()
        .contexts()
        .get(BindingContextId::new(0))
        .ok_or(SourceModeCoreContextError::InvalidCheckerOwner)?;
    let definition_context = source_context
        .binding_env()
        .contexts()
        .get(BindingContextId::new(1))
        .ok_or(SourceModeCoreContextError::InvalidCheckerOwner)?;
    if module_context.id != BindingContextId::new(0)
        || !is_normal_module_context(module_context)
        || definition_context.id != BindingContextId::new(1)
        || !is_normal_declaration_context(definition_context)
        || definition_context.owner != BindingContextOwner::DeclarationShell(source_item.shell)
        || definition_context.parent != Some(BindingContextId::new(0))
        || definition_context.lexical_scope != source_item.local_scope
        || !module_context.bindings.is_empty()
        || !module_context.visible_bindings.is_empty()
    {
        return Err(SourceModeCoreContextError::InvalidCheckerOwner);
    }

    let declaration_rows = source_context.declarations().iter().collect::<Vec<_>>();
    if declaration_rows.len() != 2 {
        return Err(SourceModeCoreContextError::InvalidCheckerOwner);
    }
    let expected_parameter_ranges = [(13, 26, 17, 18, 22, 25), (29, 42, 33, 34, 38, 41)];
    let expected_parameter_sites = [37, 41];
    let expected_parameter_spelling = ["x", "y"];
    for (index, (id, declaration)) in declaration_rows.iter().enumerate() {
        let expected_source_range = SourceRange {
            source_id: source_context.source_id(),
            start: expected_parameter_ranges[index].0,
            end: expected_parameter_ranges[index].1,
        };
        let expected_declaration_range = SourceRange {
            source_id: source_context.source_id(),
            start: expected_parameter_ranges[index].2,
            end: expected_parameter_ranges[index].3,
        };
        let expected_written_type_range = SourceRange {
            source_id: source_context.source_id(),
            start: expected_parameter_ranges[index].4,
            end: expected_parameter_ranges[index].5,
        };
        let SourceBindingSiteRole::DefinitionParameter { local } = &declaration.role else {
            return Err(SourceModeCoreContextError::InvalidCheckerOwner);
        };
        if *id != SourceDeclarationId::new(index)
            || declaration.id != *id
            || declaration.item != source_item.id
            || declaration.binding != BindingId::new(index)
            || declaration.source_ordinal != index
            || declaration.spelling != expected_parameter_spelling[index]
            || declaration.declaration_range != expected_declaration_range
            || declaration.written_type_range != expected_written_type_range
            || declaration.site
                != mizar_checker::typed_ast::TypedSiteRef::Node(
                    mizar_checker::typed_ast::TypedNodeId::new(expected_parameter_sites[index]),
                )
            || declaration.binding_context != BindingContextId::new(1)
            || declaration.local_context != mizar_checker::typed_ast::LocalTypeContextId::new(1)
            || declaration.shadowed_binding.is_some()
            || declaration.predecessor != index.checked_sub(1).map(SourceDeclarationId::new)
            || local.spelling() != expected_parameter_spelling[index]
            || local.scope().path() != [0]
            || local.declaration_range() != expected_declaration_range
            || local.visible_after_ordinal() != index
            || !range_contains(source_item.source_range, expected_source_range)
        {
            return Err(SourceModeCoreContextError::InvalidCheckerOwner);
        }

        let binding = source_context
            .binding_env()
            .bindings()
            .get(declaration.binding)
            .ok_or(SourceModeCoreContextError::InvalidCheckerOwner)?;
        let BinderIdentity::ResolverLocal {
            scope,
            ordinal,
            declaration_range,
        } = &binding.identity
        else {
            return Err(SourceModeCoreContextError::InvalidCheckerOwner);
        };
        if binding.id != BindingId::new(index)
            || binding.spelling != expected_parameter_spelling[index]
            || binding.kind != BindingKind::DefinitionParameter
            || binding.owner_context != BindingContextId::new(1)
            || binding.declaration_range != expected_declaration_range
            || binding.visible_after_ordinal != index
            || binding.type_site != BindingTypeSite::Source(expected_written_type_range)
            || binding.status != BindingStatus::Active
            || !binding.captured.identities().is_empty()
            || !binding.diagnostics.is_empty()
            || binding.recovery != BindingRecoveryState::Normal
            || scope.path() != [0]
            || *ordinal != index
            || *declaration_range != expected_declaration_range
        {
            return Err(SourceModeCoreContextError::InvalidCheckerOwner);
        }
    }
    let expected_bindings = vec![BindingId::new(0), BindingId::new(1)];
    if definition_context.bindings != expected_bindings
        || definition_context.visible_bindings != expected_bindings
    {
        return Err(SourceModeCoreContextError::InvalidCheckerOwner);
    }

    let module_local_context = source_context
        .local_contexts()
        .get(mizar_checker::typed_ast::LocalTypeContextId::new(0))
        .ok_or(SourceModeCoreContextError::InvalidCheckerOwner)?;
    let definition_local_context = source_context
        .local_contexts()
        .get(mizar_checker::typed_ast::LocalTypeContextId::new(1))
        .ok_or(SourceModeCoreContextError::InvalidCheckerOwner)?;
    let expected_local_bindings = declaration_rows
        .iter()
        .map(|(_, declaration)| BindingTypeRef::Site(declaration.site.clone()))
        .collect::<Vec<_>>();
    if module_local_context.id != mizar_checker::typed_ast::LocalTypeContextId::new(0)
        || module_local_context.parent.is_some()
        || module_local_context.owner
            != mizar_checker::typed_ast::TypedSiteRef::Node(
                mizar_checker::typed_ast::TypedNodeId::new(53),
            )
        || module_local_context.layer != mizar_checker::typed_ast::TypeContextLayer::Module
        || module_local_context.recovery != mizar_checker::typed_ast::ContextRecoveryState::Normal
        || !module_local_context.bindings.is_empty()
        || !module_local_context.introduced_assumptions.is_empty()
        || !module_local_context.visible_facts.is_empty()
        || definition_local_context.id != mizar_checker::typed_ast::LocalTypeContextId::new(1)
        || definition_local_context.owner != source_item.site
        || definition_local_context.parent
            != Some(mizar_checker::typed_ast::LocalTypeContextId::new(0))
        || definition_local_context.layer != mizar_checker::typed_ast::TypeContextLayer::Declaration
        || definition_local_context.recovery
            != mizar_checker::typed_ast::ContextRecoveryState::Normal
        || definition_local_context.bindings != expected_local_bindings
        || !definition_local_context.introduced_assumptions.is_empty()
        || !definition_local_context.visible_facts.is_empty()
    {
        return Err(SourceModeCoreContextError::InvalidCheckerOwner);
    }

    for (index, (link_id, link)) in source_context.context_links().iter().enumerate() {
        let expected_item = (index == 1).then_some(source_item.id);
        if link_id != index
            || link.binding_context != BindingContextId::new(index)
            || link.local_context != mizar_checker::typed_ast::LocalTypeContextId::new(index)
            || link.item != expected_item
        {
            return Err(SourceModeCoreContextError::InvalidCheckerOwner);
        }
    }

    if checker_owner.definitions().len() != 1
        || checker_owner.parameters().len() != 2
        || checker_owner.applications().len() != 1
        || checker_owner.expansions().len() != 1
        || checker_owner.inhabitation_requests().len() != 1
        || checker_owner.properties().len() != 1
    {
        return Err(SourceModeCoreContextError::InvalidCheckerOwner);
    }
    let definition = checker_owner
        .definitions()
        .get(mizar_checker::source_mode_definition::SourceModeDefinitionId::new(0))
        .ok_or(SourceModeCoreContextError::InvalidCheckerOwner)?;
    let definition_range = SourceRange {
        source_id: source_context.source_id(),
        start: 45,
        end: 135,
    };
    if definition.id() != mizar_checker::source_mode_definition::SourceModeDefinitionId::new(0)
        || definition.definition().index() != 0
        || definition.contribution().index() != 0
        || definition.site()
            != &mizar_checker::typed_ast::TypedSiteRef::Node(
                mizar_checker::typed_ast::TypedNodeId::new(49),
            )
        || definition.source_range() != definition_range
        || definition.source_ordinal() != 0
        || definition.context() != BindingContextId::new(1)
        || definition.recovery()
            != mizar_checker::source_mode_definition::SourceModeDefinitionRecovery::Normal
        || definition.spelling()
            != "mode Task262ModeDefinition: Task262Mode [x, y] is set;\n  sethood by computation(steps: 1);"
        || definition.application()
            != mizar_checker::source_mode_definition::SourceModeApplicationId::new(0)
        || definition.expansion()
            != mizar_checker::source_mode_definition::SourceModeExpansionId::new(0)
        || definition.inhabitation_request()
            != mizar_checker::source_mode_definition::SourceModeInhabitationRequestId::new(0)
        || definition.property()
            != Some(mizar_checker::source_mode_definition::SourceModePropertyId::new(0))
        || definition.symbol().module() != source_context.module_id()
        || definition.origin().source_id() != source_context.source_id()
        || definition.origin().module_id() != source_context.module_id()
        || definition.origin().is_recovered()
        || definition.origin().import_edge().is_some()
        || definition.origin().anchor() != &SourceAnchor::Range(definition_range)
        || definition.origin().structural_path() != [4, 0, 10, 0]
        || !range_contains(source_item.source_range, definition_range)
    {
        return Err(SourceModeCoreContextError::InvalidCheckerOwner);
    }

    let parameter_spelling = ["let x be set;", "let y be set;"];
    for (index, (parameter_id, parameter)) in checker_owner.parameters().iter().enumerate() {
        let declaration = declaration_rows[index].1;
        let expected_source_range = SourceRange {
            source_id: source_context.source_id(),
            start: expected_parameter_ranges[index].0,
            end: expected_parameter_ranges[index].1,
        };
        if parameter_id.index() != index
            || parameter.id().index() != index
            || parameter.owner() != definition.id()
            || parameter.ordinal() != index
            || parameter.binding() != BindingId::new(index)
            || parameter.written_type().index() != index
            || parameter.site() != &declaration.site
            || parameter.source_range() != expected_source_range
            || parameter.declaration_range() != declaration.declaration_range
            || parameter.pattern_range()
                != (SourceRange {
                    source_id: source_context.source_id(),
                    start: if index == 0 { 86 } else { 89 },
                    end: if index == 0 { 87 } else { 90 },
                })
            || parameter.context() != BindingContextId::new(1)
            || parameter.recovery()
                != mizar_checker::source_mode_definition::SourceModeDefinitionRecovery::Normal
            || parameter.spelling() != parameter_spelling[index]
        {
            return Err(SourceModeCoreContextError::InvalidCheckerOwner);
        }
    }

    let application = checker_owner
        .applications()
        .get(mizar_checker::source_mode_definition::SourceModeApplicationId::new(0))
        .ok_or(SourceModeCoreContextError::InvalidCheckerOwner)?;
    if application.id() != mizar_checker::source_mode_definition::SourceModeApplicationId::new(0)
        || application.owner() != definition.id()
        || application.ordinal() != 0
        || application.parameters()
            != [
                mizar_checker::source_mode_definition::SourceModeParameterId::new(0),
                mizar_checker::source_mode_definition::SourceModeParameterId::new(1),
            ]
        || application.site()
            != &mizar_checker::typed_ast::TypedSiteRef::Node(
                mizar_checker::typed_ast::TypedNodeId::new(42),
            )
        || application.source_range()
            != (SourceRange {
                source_id: source_context.source_id(),
                start: 73,
                end: 91,
            })
        || application.context() != BindingContextId::new(1)
        || application.recovery()
            != mizar_checker::source_mode_definition::SourceModeDefinitionRecovery::Normal
        || application.spelling() != "Task262Mode [ x , y ]"
    {
        return Err(SourceModeCoreContextError::InvalidCheckerOwner);
    }

    let expansion = checker_owner
        .expansions()
        .get(mizar_checker::source_mode_definition::SourceModeExpansionId::new(0))
        .ok_or(SourceModeCoreContextError::InvalidCheckerOwner)?;
    if expansion.id() != mizar_checker::source_mode_definition::SourceModeExpansionId::new(0)
        || expansion.owner() != definition.id()
        || expansion.ordinal() != 0
        || expansion.rhs() != mizar_checker::source_type::SourceTypeModeRhsId::new(0)
        || expansion.site()
            != &mizar_checker::typed_ast::TypedSiteRef::Node(
                mizar_checker::typed_ast::TypedNodeId::new(44),
            )
        || expansion.source_range()
            != (SourceRange {
                source_id: source_context.source_id(),
                start: 95,
                end: 98,
            })
        || expansion.context() != BindingContextId::new(1)
        || expansion.recovery()
            != mizar_checker::source_mode_definition::SourceModeDefinitionRecovery::Normal
        || expansion.spelling() != "set"
    {
        return Err(SourceModeCoreContextError::InvalidCheckerOwner);
    }

    let request = checker_owner
        .inhabitation_requests()
        .get(mizar_checker::source_mode_definition::SourceModeInhabitationRequestId::new(0))
        .ok_or(SourceModeCoreContextError::InvalidCheckerOwner)?;
    if request.id()
        != mizar_checker::source_mode_definition::SourceModeInhabitationRequestId::new(0)
        || request.owner() != definition.id()
        || request.ordinal() != 0
        || request.expansion() != expansion.id()
        || request.kind()
            != mizar_checker::source_mode_definition::SourceModeInhabitationRequestKind::Rhs
        || request.site() != expansion.site()
        || request.source_range() != expansion.source_range()
        || request.context() != BindingContextId::new(1)
        || request.recovery()
            != mizar_checker::source_mode_definition::SourceModeDefinitionRecovery::Normal
        || request.spelling() != "set"
    {
        return Err(SourceModeCoreContextError::InvalidCheckerOwner);
    }

    let property = checker_owner
        .properties()
        .get(mizar_checker::source_mode_definition::SourceModePropertyId::new(0))
        .ok_or(SourceModeCoreContextError::InvalidCheckerOwner)?;
    if property.id() != mizar_checker::source_mode_definition::SourceModePropertyId::new(0)
        || property.owner() != definition.id()
        || property.ordinal() != 0
        || property.kind() != mizar_checker::source_mode_definition::SourceModePropertyKind::Sethood
        || property.site()
            != &mizar_checker::typed_ast::TypedSiteRef::Node(
                mizar_checker::typed_ast::TypedNodeId::new(48),
            )
        || property.source_range()
            != (SourceRange {
                source_id: source_context.source_id(),
                start: 102,
                end: 135,
            })
        || property.justification()
            != &SourceAnchor::Range(SourceRange {
                source_id: source_context.source_id(),
                start: 113,
                end: 134,
            })
        || property.recovery()
            != mizar_checker::source_mode_definition::SourceModeDefinitionRecovery::Normal
        || property.spelling() != "sethood by computation(steps: 1);"
        || property.obligation().index() != 0
    {
        return Err(SourceModeCoreContextError::InvalidCheckerOwner);
    }
    Ok(definition)
}

fn validate_mode_core_shape(
    context: &CoreContext,
    symbol: &SymbolId,
    source_range: SourceRange,
) -> Result<CoreItemId, SourceModeCoreContextError> {
    if validate_core_context_shape(context, &BTreeSet::new()).is_err()
        || !context.dependency_summaries.is_empty()
        || !context.generated_origins.table().is_empty()
        || !context.generated_origins.by_key.is_empty()
        || !context.diagnostics.is_empty()
        || context.source_map.item_sources.len() != 1
        || !context.source_map.term_sources.is_empty()
        || !context.source_map.formula_sources.is_empty()
        || !context.source_map.definition_sources.is_empty()
        || !context.source_map.proof_sources.is_empty()
        || !context.source_map.algorithm_sources.is_empty()
        || !context.source_map.generated_sources.is_empty()
        || !context.source_map.obligation_sources.is_empty()
        || context.item_registry.items.len() != 1
        || context.item_registry.by_symbol.len() != 1
        || context.item_registry.dependencies.len() != 1
        || context.definition_boundaries.by_item.len() != 1
        || context.definition_boundaries.by_symbol.len() != 1
        || context.worklist.entries.len() != 1
    {
        return Err(SourceModeCoreContextError::InvalidCoreContext);
    }
    let core_item = context
        .item_registry
        .id_for_symbol(symbol)
        .ok_or(SourceModeCoreContextError::InvalidCoreContext)?;
    let item = context
        .item_registry
        .items
        .get(core_item)
        .ok_or(SourceModeCoreContextError::InvalidCoreContext)?;
    let expected_provenance = CoreProvenance::new(
        CoreProvenancePhase::Checker,
        SOURCE_MODE_CORE_ITEM_PROVENANCE_KEY,
    );
    let expected_source =
        CoreSourceRef::direct(source_range).with_provenance(vec![expected_provenance.clone()]);
    if item.symbol != *symbol
        || item.kind != CoreItemKind::Mode
        || item.visibility.as_str() != "public"
        || item.status != CoreItemStatus::Valid
        || !item.dependencies.is_empty()
        || !item.diagnostics.is_empty()
        || item.source != expected_source
        || context.source_map.item_sources.get(&core_item) != Some(&item.source)
    {
        return Err(SourceModeCoreContextError::InvalidCoreContext);
    }
    let dependency = context
        .item_registry
        .dependencies
        .get(&core_item)
        .ok_or(SourceModeCoreContextError::InvalidCoreContext)?;
    if !dependency.local.is_empty()
        || !dependency.external.is_empty()
        || !dependency.missing.is_empty()
    {
        return Err(SourceModeCoreContextError::InvalidCoreContext);
    }
    let boundary = context
        .definition_boundaries
        .by_item
        .get(&core_item)
        .ok_or(SourceModeCoreContextError::InvalidCoreContext)?;
    if context.definition_boundaries.by_symbol.get(symbol) != Some(&core_item)
        || boundary.item != core_item
        || boundary.symbol != *symbol
        || boundary.kind != DefinitionBoundaryKind::DefinitionalItem
        || boundary.status != DefinitionBoundaryStatus::PendingBody
        || boundary.source != expected_source
        || boundary.provenance.as_slice() != [expected_provenance.clone()]
    {
        return Err(SourceModeCoreContextError::InvalidCoreContext);
    }
    let work_item = context
        .worklist
        .entries
        .first()
        .ok_or(SourceModeCoreContextError::InvalidCoreContext)?;
    if work_item.kind != ElaborationWorkItemKind::Item(core_item)
        || work_item.status != ElaborationWorkStatus::Pending
        || work_item.source != expected_source
        || !work_item.diagnostics.is_empty()
        || !work_item.checker_diagnostics.is_empty()
    {
        return Err(SourceModeCoreContextError::InvalidCoreContext);
    }
    Ok(core_item)
}

fn validate_mode_item_association(
    source_context: &SourceBindingContextHandoff,
    checker_owner: &mizar_checker::source_mode_definition::SourceModeDefinitionHandoff,
    items: &SourceModeCoreItemAssociationTable,
    definition: &mizar_checker::source_mode_definition::SourceModeDefinition,
    core_item: CoreItemId,
) -> Result<(), SourceModeCoreContextError> {
    if items.len() != checker_owner.definitions().len() || items.len() != 1 {
        return Err(SourceModeCoreContextError::InvalidItemAssociation);
    }
    let source_item = source_context
        .context_links()
        .get(definition.context())
        .and_then(|link| link.item)
        .ok_or(SourceModeCoreContextError::InvalidItemAssociation)?;
    let association = items
        .get(definition.id())
        .ok_or(SourceModeCoreContextError::InvalidItemAssociation)?;
    if association.definition() != definition.id()
        || association.source_item() != source_item
        || association.symbol() != definition.symbol()
        || association.core_item() != core_item
        || items
            .iter()
            .next()
            .is_none_or(|(id, row)| id != definition.id() || row != association)
    {
        return Err(SourceModeCoreContextError::InvalidItemAssociation);
    }
    Ok(())
}

const SOURCE_STRUCTURE_CORE_ITEM_PROVENANCE_KEYS: [&str; 2] = [
    "source-structure-core-item-v1.definition.0",
    "source-structure-core-item-v1.definition.1",
];

/// One immutable association between a checker structure definition, its
/// whole symbol, and the corresponding Core item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStructureCoreItemAssociation {
    definition: SourceStructureDefinitionId,
    symbol: SymbolId,
    core_item: CoreItemId,
}

impl SourceStructureCoreItemAssociation {
    #[must_use]
    pub const fn definition(&self) -> SourceStructureDefinitionId {
        self.definition
    }

    #[must_use]
    pub const fn symbol(&self) -> &SymbolId {
        &self.symbol
    }

    #[must_use]
    pub const fn core_item(&self) -> CoreItemId {
        self.core_item
    }
}

/// Immutable source-ordered table of structure/Core item associations.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SourceStructureCoreItemAssociationTable {
    rows: Vec<SourceStructureCoreItemAssociation>,
}

impl SourceStructureCoreItemAssociationTable {
    fn empty() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn get(
        &self,
        definition: SourceStructureDefinitionId,
    ) -> Option<&SourceStructureCoreItemAssociation> {
        self.rows.iter().find(|row| row.definition() == definition)
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<
        Item = (
            SourceStructureDefinitionId,
            &SourceStructureCoreItemAssociation,
        ),
    > {
        self.rows.iter().map(|row| (row.definition(), row))
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// Errors raised while building the exact Task-263 structure/Core context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceStructureCoreContextError {
    EnvironmentMismatch,
    InvalidCheckerOwner,
    InvalidCoreContext,
    InvalidItemAssociation,
}

impl fmt::Display for SourceStructureCoreContextError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EnvironmentMismatch => {
                formatter.write_str("structure Core context environment is invalid")
            }
            Self::InvalidCheckerOwner => {
                formatter.write_str("structure Core checker owner is invalid")
            }
            Self::InvalidCoreContext => formatter.write_str("structure Core context is invalid"),
            Self::InvalidItemAssociation => {
                formatter.write_str("structure Core item association is invalid")
            }
        }
    }
}

impl Error for SourceStructureCoreContextError {}

/// Immutable Core context handoff for the exact checker-authenticated
/// Task-263 structure definitions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStructureCoreContextHandoff {
    context: CoreContext,
    checker_owner: SourceStructureDefinitionHandoff,
    items: SourceStructureCoreItemAssociationTable,
}

impl SourceStructureCoreContextHandoff {
    #[must_use]
    pub const fn source_id(&self) -> SourceId {
        self.context.source_id()
    }

    #[must_use]
    pub const fn module_id(&self) -> &ModuleId {
        self.context.module_id()
    }

    #[must_use]
    pub const fn context(&self) -> &CoreContext {
        &self.context
    }

    #[must_use]
    pub const fn checker_owner(&self) -> &SourceStructureDefinitionHandoff {
        &self.checker_owner
    }

    #[must_use]
    pub const fn items(&self) -> &SourceStructureCoreItemAssociationTable {
        &self.items
    }

    #[must_use]
    pub fn debug_text(&self) -> String {
        let associations = self
            .items
            .iter()
            .map(|(definition, association)| {
                format!(
                    "{}:{}:{}",
                    definition.index(),
                    association.symbol().fqn().as_str(),
                    association.core_item().index(),
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "source-structure-core-item-context-v1|module={}.{}|definitions={}|inheritances={}|items={}",
            self.module_id().package().as_str(),
            self.module_id().path().as_str(),
            self.checker_owner.definitions().len(),
            self.checker_owner.inheritances().len(),
            associations,
        )
    }

    fn validate(&self) -> Result<(), SourceStructureCoreContextError> {
        validate_structure_core_environment(&self.context, &self.checker_owner)?;
        let definitions = validate_structure_checker_owner(&self.checker_owner)?;
        let core_items = validate_structure_core_shape(&self.context, definitions)?;
        validate_structure_item_associations(
            &self.checker_owner,
            &self.items,
            definitions,
            core_items,
        )
    }
}

/// Builds the standalone immutable Task-263 structure/Core context handoff.
#[derive(Debug, Clone, Copy)]
pub struct SourceStructureCoreContextProducer;

impl SourceStructureCoreContextProducer {
    pub fn build(
        context: CoreContext,
        checker_owner: SourceStructureDefinitionHandoff,
    ) -> Result<SourceStructureCoreContextHandoff, SourceStructureCoreContextError> {
        validate_structure_core_environment(&context, &checker_owner)?;
        let definitions = validate_structure_checker_owner(&checker_owner)?;
        let core_items = validate_structure_core_shape(&context, definitions)?;
        let mut items = SourceStructureCoreItemAssociationTable::empty();
        for (definition, core_item) in definitions.into_iter().zip(core_items) {
            items.rows.push(SourceStructureCoreItemAssociation {
                definition: definition.id(),
                symbol: definition.symbol().clone(),
                core_item,
            });
        }
        let handoff = SourceStructureCoreContextHandoff {
            context,
            checker_owner,
            items,
        };
        handoff.validate()?;
        Ok(handoff)
    }
}

fn validate_structure_core_environment(
    context: &CoreContext,
    checker_owner: &SourceStructureDefinitionHandoff,
) -> Result<(), SourceStructureCoreContextError> {
    if context.source_id() != checker_owner.source_id()
        || context.module_id() != checker_owner.module_id()
    {
        return Err(SourceStructureCoreContextError::EnvironmentMismatch);
    }
    Ok(())
}

fn validate_structure_checker_owner(
    checker_owner: &SourceStructureDefinitionHandoff,
) -> Result<
    [&mizar_checker::source_structure_definition::SourceStructureDefinition; 2],
    SourceStructureCoreContextError,
> {
    if checker_owner.source_type_fingerprint().is_empty()
        || checker_owner.base_initial_obligation_count() != 0
        || checker_owner.definitions().len() != 2
        || checker_owner.members().len() != 4
        || checker_owner.inheritances().len() != 1
        || checker_owner.mappings().len() != 2
        || !checker_owner.coherence_requests().is_empty()
    {
        return Err(SourceStructureCoreContextError::InvalidCheckerOwner);
    }

    let definitions = [
        checker_owner
            .definitions()
            .get(SourceStructureDefinitionId::new(0))
            .ok_or(SourceStructureCoreContextError::InvalidCheckerOwner)?,
        checker_owner
            .definitions()
            .get(SourceStructureDefinitionId::new(1))
            .ok_or(SourceStructureCoreContextError::InvalidCheckerOwner)?,
    ];
    let definition_ids = [0, 3];
    let definition_sites = [57, 65];
    let definition_ranges = [(13, 98), (102, 190)];
    let definition_members = [[0, 1], [2, 3]];
    let definition_fields = [0, 2];
    let definition_paths: [&[u32]; 2] = [&[4, 0, 11, 0], &[4, 0, 11, 1]];
    let definition_spellings = [
        "struct Task263Base where\n    field carrier -> set;\n    property marker -> set;\n  end;",
        "struct Task263Derived where\n    field carrier -> set;\n    property marker -> set;\n  end;",
    ];
    for (index, definition) in definitions.iter().enumerate() {
        let source_range = SourceRange {
            source_id: checker_owner.source_id(),
            start: definition_ranges[index].0,
            end: definition_ranges[index].1,
        };
        let expected_members = definition_members[index].map(SourceStructureDefinitionId::new);
        let expected_members = expected_members.map(|id| {
            mizar_checker::source_structure_definition::SourceStructureMemberId::new(id.index())
        });
        let expected_field =
            mizar_checker::source_structure_definition::SourceStructureMemberId::new(
                definition_fields[index],
            );
        if definition.id() != SourceStructureDefinitionId::new(index)
            || definition.definition().index() != definition_ids[index]
            || definition.contribution().index() != 0
            || definition.site() != &TypedSiteRef::Node(TypedNodeId::new(definition_sites[index]))
            || definition.source_range() != source_range
            || definition.source_ordinal() != index
            || definition.recovery() != SourceStructureDefinitionRecovery::Normal
            || definition.spelling() != definition_spellings[index]
            || definition.members() != expected_members
            || definition.constructor_fields() != [expected_field]
            || definition.symbol().module() != checker_owner.module_id()
            || !is_exact_structure_origin(
                definition.origin(),
                checker_owner.source_id(),
                checker_owner.module_id(),
                source_range,
                definition_paths[index],
            )
        {
            return Err(SourceStructureCoreContextError::InvalidCheckerOwner);
        }
    }
    if definitions[0].symbol() == definitions[1].symbol() {
        return Err(SourceStructureCoreContextError::InvalidCheckerOwner);
    }

    let member_definition_ids = [1, 2, 4, 5];
    let member_owners = [0, 0, 1, 1];
    let member_ordinals = [0, 1, 0, 1];
    let member_kinds = [
        SourceStructureMemberKind::Field,
        SourceStructureMemberKind::Property,
        SourceStructureMemberKind::Field,
        SourceStructureMemberKind::Property,
    ];
    let member_sites = [53, 56, 61, 64];
    let member_ranges = [(42, 63), (68, 91), (134, 155), (160, 183)];
    let member_spellings = [
        "field carrier -> set;",
        "property marker -> set;",
        "field carrier -> set;",
        "property marker -> set;",
    ];
    let member_constructor_ordinals = [Some(0), None, Some(0), None];
    let member_paths: [&[u32]; 4] = [
        &[4, 0, 11, 0, 18, 0],
        &[4, 0, 11, 0, 19, 1],
        &[4, 0, 11, 1, 18, 0],
        &[4, 0, 11, 1, 19, 1],
    ];
    let mut member_symbols = BTreeSet::new();
    for index in 0..4 {
        let member = checker_owner
            .members()
            .get(mizar_checker::source_structure_definition::SourceStructureMemberId::new(index))
            .ok_or(SourceStructureCoreContextError::InvalidCheckerOwner)?;
        let source_range = SourceRange {
            source_id: checker_owner.source_id(),
            start: member_ranges[index].0,
            end: member_ranges[index].1,
        };
        if member.id().index() != index
            || member.definition().index() != member_definition_ids[index]
            || member.contribution().index() != 0
            || member.owner() != SourceStructureDefinitionId::new(member_owners[index])
            || member.ordinal() != member_ordinals[index]
            || member.kind() != member_kinds[index]
            || member.site() != &TypedSiteRef::Node(TypedNodeId::new(member_sites[index]))
            || member.source_range() != source_range
            || member.recovery() != SourceStructureDefinitionRecovery::Normal
            || member.spelling() != member_spellings[index]
            || member.written_type().index() != index
            || member.constructor_ordinal() != member_constructor_ordinals[index]
            || member.symbol().module() != checker_owner.module_id()
            || !member_symbols.insert(member.symbol())
            || !is_exact_structure_origin(
                member.origin(),
                checker_owner.source_id(),
                checker_owner.module_id(),
                source_range,
                member_paths[index],
            )
        {
            return Err(SourceStructureCoreContextError::InvalidCheckerOwner);
        }
    }

    let inheritance = checker_owner
        .inheritances()
        .get(mizar_checker::source_structure_definition::SourceStructureInheritanceId::new(0))
        .ok_or(SourceStructureCoreContextError::InvalidCheckerOwner)?;
    let inheritance_range = SourceRange {
        source_id: checker_owner.source_id(),
        start: 194,
        end: 314,
    };
    if inheritance.id().index() != 0
        || inheritance.child() != SourceStructureDefinitionId::new(1)
        || inheritance.parent() != SourceStructureDefinitionId::new(0)
        || inheritance.site() != &TypedSiteRef::Node(TypedNodeId::new(70))
        || inheritance.source_range() != inheritance_range
        || inheritance.source_ordinal() != 0
        || inheritance.recovery() != SourceStructureDefinitionRecovery::Normal
        || inheritance.spelling()
            != "inherit Task263Derived extends Task263Base where\n    field carrier from carrier;\n    property marker from marker;\n  end;"
        || inheritance.mappings()
            != [
                mizar_checker::source_structure_definition::SourceStructureMappingId::new(0),
                mizar_checker::source_structure_definition::SourceStructureMappingId::new(1),
            ]
    {
        return Err(SourceStructureCoreContextError::InvalidCheckerOwner);
    }

    let mapping_definition_ids = [6, 7];
    let mapping_kinds = [
        SourceStructureMemberKind::Field,
        SourceStructureMemberKind::Property,
    ];
    let mapping_view_members = [2, 3];
    let mapping_parent_members = [0, 1];
    let mapping_sites = [68, 69];
    let mapping_ranges = [(247, 274), (279, 307)];
    let mapping_spellings = [
        "field carrier from carrier;",
        "property marker from marker;",
    ];
    let mapping_paths: [&[u32]; 2] = [&[4, 0, 20, 2, 21, 0], &[4, 0, 20, 2, 22, 1]];
    for index in 0..2 {
        let mapping = checker_owner
            .mappings()
            .get(mizar_checker::source_structure_definition::SourceStructureMappingId::new(index))
            .ok_or(SourceStructureCoreContextError::InvalidCheckerOwner)?;
        let source_range = SourceRange {
            source_id: checker_owner.source_id(),
            start: mapping_ranges[index].0,
            end: mapping_ranges[index].1,
        };
        if mapping.id().index() != index
            || mapping.definition().index() != mapping_definition_ids[index]
            || mapping.contribution().index() != 0
            || mapping.inheritance() != inheritance.id()
            || mapping.ordinal() != index
            || mapping.kind() != mapping_kinds[index]
            || mapping.view_member().index() != mapping_view_members[index]
            || mapping.parent_member().index() != mapping_parent_members[index]
            || mapping.root_member().index() != mapping_parent_members[index]
            || mapping.path() != [inheritance.id()]
            || mapping.site() != &TypedSiteRef::Node(TypedNodeId::new(mapping_sites[index]))
            || mapping.source_range() != source_range
            || mapping.recovery() != SourceStructureDefinitionRecovery::Normal
            || mapping.spelling() != mapping_spellings[index]
            || mapping.symbol().module() != checker_owner.module_id()
            || !is_exact_structure_origin(
                mapping.origin(),
                checker_owner.source_id(),
                checker_owner.module_id(),
                source_range,
                mapping_paths[index],
            )
        {
            return Err(SourceStructureCoreContextError::InvalidCheckerOwner);
        }
    }
    Ok(definitions)
}

fn is_exact_structure_origin(
    origin: &SemanticOrigin,
    source_id: SourceId,
    module_id: &ModuleId,
    source_range: SourceRange,
    structural_path: &[u32],
) -> bool {
    origin.source_id() == source_id
        && origin.module_id() == module_id
        && !origin.is_recovered()
        && origin.import_edge().is_none()
        && origin.anchor() == &SourceAnchor::Range(source_range)
        && origin.structural_path() == structural_path
}

fn validate_structure_core_shape(
    context: &CoreContext,
    definitions: [&mizar_checker::source_structure_definition::SourceStructureDefinition; 2],
) -> Result<[CoreItemId; 2], SourceStructureCoreContextError> {
    let binder_context = context.binder_context();
    if validate_core_context_shape(context, &BTreeSet::new()).is_err()
        || !binder_context.frames.is_empty()
        || !binder_context.free_variables.is_empty()
        || !binder_context.variable_classes.is_empty()
        || !binder_context.variable_roles.is_empty()
        || !binder_context.variable_sorts.is_empty()
        || context.binder_sources.iter().next().is_some()
        || !context.binder_type_facts.is_empty()
        || !context.dependency_summaries.is_empty()
        || !context.generated_origins.table().is_empty()
        || !context.generated_origins.by_key.is_empty()
        || !context.diagnostics.is_empty()
        || context.source_map.item_sources.len() != 2
        || !context.source_map.term_sources.is_empty()
        || !context.source_map.formula_sources.is_empty()
        || !context.source_map.definition_sources.is_empty()
        || !context.source_map.proof_sources.is_empty()
        || !context.source_map.algorithm_sources.is_empty()
        || !context.source_map.generated_sources.is_empty()
        || !context.source_map.obligation_sources.is_empty()
        || context.item_registry.items.len() != 2
        || context.item_registry.by_symbol.len() != 2
        || context.item_registry.dependencies.len() != 2
        || context.definition_boundaries.by_item.len() != 2
        || context.definition_boundaries.by_symbol.len() != 2
        || context.worklist.entries.len() != 2
    {
        return Err(SourceStructureCoreContextError::InvalidCoreContext);
    }
    let core_items = [
        context
            .item_registry
            .id_for_symbol(definitions[0].symbol())
            .ok_or(SourceStructureCoreContextError::InvalidCoreContext)?,
        context
            .item_registry
            .id_for_symbol(definitions[1].symbol())
            .ok_or(SourceStructureCoreContextError::InvalidCoreContext)?,
    ];
    if core_items[0] == core_items[1] {
        return Err(SourceStructureCoreContextError::InvalidCoreContext);
    }
    for index in 0..2 {
        let core_item = core_items[index];
        let definition = definitions[index];
        let item = context
            .item_registry
            .items
            .get(core_item)
            .ok_or(SourceStructureCoreContextError::InvalidCoreContext)?;
        let expected_provenance = CoreProvenance::new(
            CoreProvenancePhase::Checker,
            SOURCE_STRUCTURE_CORE_ITEM_PROVENANCE_KEYS[index],
        );
        let expected_source = CoreSourceRef::direct(definition.source_range())
            .with_provenance(vec![expected_provenance.clone()]);
        let expected_dependencies: &[CoreItemId] = if index == 0 { &[] } else { &core_items[..1] };
        if item.symbol != *definition.symbol()
            || item.kind != CoreItemKind::Structure
            || item.visibility.as_str() != "public"
            || item.status != CoreItemStatus::Valid
            || item.dependencies != expected_dependencies
            || !item.diagnostics.is_empty()
            || item.source != expected_source
            || context.source_map.item_sources.get(&core_item) != Some(&item.source)
        {
            return Err(SourceStructureCoreContextError::InvalidCoreContext);
        }
        let dependency = context
            .item_registry
            .dependencies
            .get(&core_item)
            .ok_or(SourceStructureCoreContextError::InvalidCoreContext)?;
        if dependency.local != expected_dependencies
            || !dependency.external.is_empty()
            || !dependency.missing.is_empty()
        {
            return Err(SourceStructureCoreContextError::InvalidCoreContext);
        }
        let boundary = context
            .definition_boundaries
            .by_item
            .get(&core_item)
            .ok_or(SourceStructureCoreContextError::InvalidCoreContext)?;
        if context
            .definition_boundaries
            .by_symbol
            .get(definition.symbol())
            != Some(&core_item)
            || boundary.item != core_item
            || boundary.symbol != *definition.symbol()
            || boundary.kind != DefinitionBoundaryKind::DefinitionalItem
            || boundary.status != DefinitionBoundaryStatus::PendingBody
            || boundary.source != expected_source
            || boundary.provenance.as_slice() != [expected_provenance]
        {
            return Err(SourceStructureCoreContextError::InvalidCoreContext);
        }
        let work_item = &context.worklist.entries[index];
        if work_item.kind != ElaborationWorkItemKind::Item(core_item)
            || work_item.status != ElaborationWorkStatus::Pending
            || work_item.source != expected_source
            || !work_item.diagnostics.is_empty()
            || !work_item.checker_diagnostics.is_empty()
        {
            return Err(SourceStructureCoreContextError::InvalidCoreContext);
        }
    }
    Ok(core_items)
}

fn validate_structure_item_associations(
    checker_owner: &SourceStructureDefinitionHandoff,
    items: &SourceStructureCoreItemAssociationTable,
    definitions: [&mizar_checker::source_structure_definition::SourceStructureDefinition; 2],
    core_items: [CoreItemId; 2],
) -> Result<(), SourceStructureCoreContextError> {
    if items.len() != checker_owner.definitions().len() || items.len() != 2 {
        return Err(SourceStructureCoreContextError::InvalidItemAssociation);
    }
    for index in 0..2 {
        let definition = definitions[index];
        let association = items
            .get(definition.id())
            .ok_or(SourceStructureCoreContextError::InvalidItemAssociation)?;
        if association.definition() != definition.id()
            || association.symbol() != definition.symbol()
            || association.core_item() != core_items[index]
            || items
                .iter()
                .nth(index)
                .is_none_or(|(id, row)| id != definition.id() || row != association)
        {
            return Err(SourceStructureCoreContextError::InvalidItemAssociation);
        }
    }
    Ok(())
}

const SOURCE_PROPERTY_CARRIER_CORE_ITEM_PROVENANCE_KEY: &str =
    "source-property-carrier-core-item-v1.structure";

/// Immutable Core context for one checker-authenticated Task-264 carrier item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePropertyCarrierCoreContextHandoff {
    context: CoreContext,
    checker_owner: SourcePropertyImplementationHandoff,
    carrier_item: CoreItemId,
}

impl SourcePropertyCarrierCoreContextHandoff {
    #[must_use]
    pub const fn source_id(&self) -> SourceId {
        self.context.source_id()
    }

    #[must_use]
    pub const fn module_id(&self) -> &ModuleId {
        self.context.module_id()
    }

    #[must_use]
    pub const fn context(&self) -> &CoreContext {
        &self.context
    }

    #[must_use]
    pub const fn checker_owner(&self) -> &SourcePropertyImplementationHandoff {
        &self.checker_owner
    }

    #[must_use]
    pub const fn carrier_item(&self) -> CoreItemId {
        self.carrier_item
    }

    #[must_use]
    pub fn debug_text(&self) -> String {
        let identity = self.checker_owner.carrier_identity();
        format!(
            "source-property-carrier-core-item-context-v1|module={}.{}|carrier={}:{}:{}|item={}",
            self.module_id().package().as_str(),
            self.module_id().path().as_str(),
            identity.structure_symbol().fqn().as_str(),
            identity.structure_definition().index(),
            identity.structure_contribution().index(),
            self.carrier_item.index(),
        )
    }

    fn validate(&self) -> Result<(), SourcePropertyCarrierCoreContextError> {
        validate_property_carrier_environment(&self.context, &self.checker_owner)?;
        let identity = validate_property_carrier_checker_owner(&self.checker_owner)?;
        let carrier_item = validate_property_carrier_core_shape(&self.context, identity)?;
        validate_property_carrier_item_association(
            &self.context,
            identity,
            self.carrier_item,
            carrier_item,
        )
    }
}

/// Errors raised while building the exact Task-264 carrier/Core context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourcePropertyCarrierCoreContextError {
    EnvironmentMismatch,
    InvalidCheckerOwner,
    InvalidCoreContext,
    InvalidItemAssociation,
}

impl fmt::Display for SourcePropertyCarrierCoreContextError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EnvironmentMismatch => {
                formatter.write_str("property carrier Core context environment is invalid")
            }
            Self::InvalidCheckerOwner => {
                formatter.write_str("property carrier Core checker owner is invalid")
            }
            Self::InvalidCoreContext => {
                formatter.write_str("property carrier Core context is invalid")
            }
            Self::InvalidItemAssociation => {
                formatter.write_str("property carrier Core item association is invalid")
            }
        }
    }
}

impl Error for SourcePropertyCarrierCoreContextError {}

/// Builds the standalone immutable Task-264 carrier/Core context handoff.
#[derive(Debug, Clone, Copy)]
pub struct SourcePropertyCarrierCoreContextProducer;

impl SourcePropertyCarrierCoreContextProducer {
    pub fn build(
        context: CoreContext,
        checker_owner: SourcePropertyImplementationHandoff,
    ) -> Result<SourcePropertyCarrierCoreContextHandoff, SourcePropertyCarrierCoreContextError>
    {
        validate_property_carrier_environment(&context, &checker_owner)?;
        let identity = validate_property_carrier_checker_owner(&checker_owner)?;
        let carrier_item = validate_property_carrier_core_shape(&context, identity)?;
        let handoff = SourcePropertyCarrierCoreContextHandoff {
            context,
            checker_owner,
            carrier_item,
        };
        handoff.validate()?;
        Ok(handoff)
    }
}

fn validate_property_carrier_environment(
    context: &CoreContext,
    checker_owner: &SourcePropertyImplementationHandoff,
) -> Result<(), SourcePropertyCarrierCoreContextError> {
    if context.source_id() != checker_owner.source_id()
        || context.module_id() != checker_owner.module_id()
    {
        return Err(SourcePropertyCarrierCoreContextError::EnvironmentMismatch);
    }
    Ok(())
}

fn validate_property_carrier_checker_owner(
    checker_owner: &SourcePropertyImplementationHandoff,
) -> Result<&SourcePropertyCarrierIdentity, SourcePropertyCarrierCoreContextError> {
    let mut implementations = checker_owner.implementations().iter();
    let (implementation_id, implementation) = implementations
        .next()
        .ok_or(SourcePropertyCarrierCoreContextError::InvalidCheckerOwner)?;
    let is_means = implementation.style() == SourcePropertyImplementationStyle::Means;
    let is_equals = implementation.style() == SourcePropertyImplementationStyle::Equals;
    let optional_profile_matches = if is_means {
        checker_owner.source_structure_fingerprint().is_none()
            && checker_owner
                .source_atomic_formula_fingerprint()
                .is_some_and(|value| !value.is_empty())
            && checker_owner.correctness().len() == 2
    } else if is_equals {
        checker_owner
            .source_structure_fingerprint()
            .is_some_and(|value| !value.is_empty())
            && checker_owner.source_atomic_formula_fingerprint().is_none()
            && checker_owner.correctness().is_empty()
    } else {
        false
    };
    if implementation_id.index() != 0
        || implementation.id().index() != 0
        || implementation.target().index() != 0
        || implementations.next().is_some()
        || checker_owner.parameters().len() != 1
        || checker_owner.targets().len() != 1
        || checker_owner.definientia().len() != 1
        || checker_owner.source_context_fingerprint().is_empty()
        || checker_owner.source_type_fingerprint().is_empty()
        || checker_owner.source_term_fingerprint().is_empty()
        || checker_owner
            .source_functor_application_fingerprint()
            .is_some()
        || checker_owner.source_set_term_fingerprint().is_some()
        || !optional_profile_matches
    {
        return Err(SourcePropertyCarrierCoreContextError::InvalidCheckerOwner);
    }

    let identity = checker_owner.carrier_identity();
    let symbols = [
        identity.structure_symbol(),
        identity.field_symbol(),
        identity.property_symbol(),
    ];
    let definitions = [
        identity.structure_definition(),
        identity.field_definition(),
        identity.property_definition(),
    ];
    let contributions = [
        identity.structure_contribution(),
        identity.field_contribution(),
        identity.property_contribution(),
    ];
    let origins = [
        identity.structure_origin(),
        identity.field_origin(),
        identity.property_origin(),
    ];
    let ranges = [(13, 101), (45, 66), (71, 94)];
    let paths: [&[u32]; 3] = [&[4, 0, 11, 0], &[4, 0, 11, 0, 18, 0], &[4, 0, 11, 0, 19, 1]];
    for index in 0..3 {
        if definitions[index].index() != index
            || contributions[index].index() != 0
            || symbols[index].module() != checker_owner.module_id()
            || !is_exact_property_carrier_origin(
                origins[index],
                checker_owner.source_id(),
                checker_owner.module_id(),
                SourceRange {
                    source_id: checker_owner.source_id(),
                    start: ranges[index].0,
                    end: ranges[index].1,
                },
                paths[index],
            )
        {
            return Err(SourcePropertyCarrierCoreContextError::InvalidCheckerOwner);
        }
    }
    if symbols[0] == symbols[1] || symbols[0] == symbols[2] || symbols[1] == symbols[2] {
        return Err(SourcePropertyCarrierCoreContextError::InvalidCheckerOwner);
    }

    let mut targets = checker_owner.targets().iter();
    let (target_id, target) = targets
        .next()
        .ok_or(SourcePropertyCarrierCoreContextError::InvalidCheckerOwner)?;
    if target_id.index() != 0
        || target.id().index() != 0
        || target.owner().index() != 0
        || target.ordinal() != 0
        || target.symbol() != identity.property_symbol()
        || target.definition() != identity.property_definition()
        || target.contribution() != identity.property_contribution()
        || target.origin() != identity.property_origin()
        || targets.next().is_some()
    {
        return Err(SourcePropertyCarrierCoreContextError::InvalidCheckerOwner);
    }
    Ok(identity)
}

fn is_exact_property_carrier_origin(
    origin: &SemanticOrigin,
    source_id: SourceId,
    module_id: &ModuleId,
    source_range: SourceRange,
    structural_path: &[u32],
) -> bool {
    origin.source_id() == source_id
        && origin.module_id() == module_id
        && !origin.is_recovered()
        && origin.import_edge().is_none()
        && origin.anchor() == &SourceAnchor::Range(source_range)
        && origin.structural_path() == structural_path
}

fn validate_property_carrier_core_shape(
    context: &CoreContext,
    identity: &SourcePropertyCarrierIdentity,
) -> Result<CoreItemId, SourcePropertyCarrierCoreContextError> {
    let binder_context = context.binder_context();
    if validate_core_context_shape(context, &BTreeSet::new()).is_err()
        || !binder_context.frames.is_empty()
        || !binder_context.free_variables.is_empty()
        || !binder_context.variable_classes.is_empty()
        || !binder_context.variable_roles.is_empty()
        || !binder_context.variable_sorts.is_empty()
        || context.binder_sources.iter().next().is_some()
        || !context.binder_type_facts.is_empty()
        || !context.dependency_summaries.is_empty()
        || !context.generated_origins.table().is_empty()
        || !context.generated_origins.by_key.is_empty()
        || !context.diagnostics.is_empty()
        || context.source_map.item_sources.len() != 1
        || !context.source_map.term_sources.is_empty()
        || !context.source_map.formula_sources.is_empty()
        || !context.source_map.definition_sources.is_empty()
        || !context.source_map.proof_sources.is_empty()
        || !context.source_map.algorithm_sources.is_empty()
        || !context.source_map.generated_sources.is_empty()
        || !context.source_map.obligation_sources.is_empty()
        || context.item_registry.items.len() != 1
        || context.item_registry.by_symbol.len() != 1
        || context.item_registry.dependencies.len() != 1
        || context.definition_boundaries.by_item.len() != 1
        || context.definition_boundaries.by_symbol.len() != 1
        || context.worklist.entries.len() != 1
    {
        return Err(SourcePropertyCarrierCoreContextError::InvalidCoreContext);
    }

    let carrier_item = context
        .item_registry
        .id_for_symbol(identity.structure_symbol())
        .ok_or(SourcePropertyCarrierCoreContextError::InvalidCoreContext)?;
    let item = context
        .item_registry
        .items
        .get(carrier_item)
        .ok_or(SourcePropertyCarrierCoreContextError::InvalidCoreContext)?;
    let expected_provenance = CoreProvenance::new(
        CoreProvenancePhase::Checker,
        SOURCE_PROPERTY_CARRIER_CORE_ITEM_PROVENANCE_KEY,
    );
    let expected_source = CoreSourceRef::direct(SourceRange {
        source_id: context.source_id(),
        start: 13,
        end: 101,
    })
    .with_provenance(vec![expected_provenance.clone()]);
    if item.symbol != *identity.structure_symbol()
        || item.kind != CoreItemKind::Structure
        || item.visibility.as_str() != "public"
        || item.status != CoreItemStatus::Valid
        || !item.dependencies.is_empty()
        || !item.diagnostics.is_empty()
        || item.source != expected_source
        || context.source_map.item_sources.get(&carrier_item) != Some(&item.source)
    {
        return Err(SourcePropertyCarrierCoreContextError::InvalidCoreContext);
    }

    let dependency = context
        .item_registry
        .dependencies
        .get(&carrier_item)
        .ok_or(SourcePropertyCarrierCoreContextError::InvalidCoreContext)?;
    if !dependency.local.is_empty()
        || !dependency.external.is_empty()
        || !dependency.missing.is_empty()
    {
        return Err(SourcePropertyCarrierCoreContextError::InvalidCoreContext);
    }

    let boundary = context
        .definition_boundaries
        .by_item
        .get(&carrier_item)
        .ok_or(SourcePropertyCarrierCoreContextError::InvalidCoreContext)?;
    if context
        .definition_boundaries
        .by_symbol
        .get(identity.structure_symbol())
        != Some(&carrier_item)
        || boundary.item != carrier_item
        || boundary.symbol != *identity.structure_symbol()
        || boundary.kind != DefinitionBoundaryKind::DefinitionalItem
        || boundary.status != DefinitionBoundaryStatus::PendingBody
        || boundary.source != expected_source
        || boundary.provenance.as_slice() != [expected_provenance]
    {
        return Err(SourcePropertyCarrierCoreContextError::InvalidCoreContext);
    }

    let work_item = context
        .worklist
        .entries
        .first()
        .ok_or(SourcePropertyCarrierCoreContextError::InvalidCoreContext)?;
    if work_item.kind != ElaborationWorkItemKind::Item(carrier_item)
        || work_item.status != ElaborationWorkStatus::Pending
        || work_item.source != expected_source
        || !work_item.diagnostics.is_empty()
        || !work_item.checker_diagnostics.is_empty()
    {
        return Err(SourcePropertyCarrierCoreContextError::InvalidCoreContext);
    }
    Ok(carrier_item)
}

fn validate_property_carrier_item_association(
    context: &CoreContext,
    identity: &SourcePropertyCarrierIdentity,
    carrier_item: CoreItemId,
    expected_item: CoreItemId,
) -> Result<(), SourcePropertyCarrierCoreContextError> {
    if carrier_item != expected_item
        || context
            .item_registry
            .id_for_symbol(identity.structure_symbol())
            != Some(carrier_item)
    {
        return Err(SourcePropertyCarrierCoreContextError::InvalidItemAssociation);
    }
    Ok(())
}

/// One immutable Task-264 implementation domain associated with its Core carrier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePropertyDomainTypeAssociation {
    binding: BindingId,
    application: SourceTypeApplicationId,
    root: SourceTypeExpressionId,
    carrier_item: CoreItemId,
}

impl SourcePropertyDomainTypeAssociation {
    #[must_use]
    pub const fn binding(&self) -> BindingId {
        self.binding
    }

    #[must_use]
    pub const fn application(&self) -> SourceTypeApplicationId {
        self.application
    }

    #[must_use]
    pub const fn root(&self) -> SourceTypeExpressionId {
        self.root
    }

    #[must_use]
    pub const fn carrier_item(&self) -> CoreItemId {
        self.carrier_item
    }
}

/// One immutable Task-264 selector identity associated with its written type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePropertySelectorTypeAssociation {
    symbol: SymbolId,
    member_type: SourceTypeStructureMemberId,
    root: SourceTypeExpressionId,
}

impl SourcePropertySelectorTypeAssociation {
    #[must_use]
    pub const fn symbol(&self) -> &SymbolId {
        &self.symbol
    }

    #[must_use]
    pub const fn member_type(&self) -> SourceTypeStructureMemberId {
        self.member_type
    }

    #[must_use]
    pub const fn root(&self) -> SourceTypeExpressionId {
        self.root
    }
}

/// Immutable exact Task-264 carrier/selector/type association context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePropertySelectorTypeContextHandoff {
    carrier_context: SourcePropertyCarrierCoreContextHandoff,
    source_type: SourceTypeApplicationHandoff,
    domain: SourcePropertyDomainTypeAssociation,
    association: SourcePropertySelectorTypeAssociation,
}

impl SourcePropertySelectorTypeContextHandoff {
    #[must_use]
    pub const fn source_id(&self) -> SourceId {
        self.carrier_context.source_id()
    }

    #[must_use]
    pub const fn module_id(&self) -> &ModuleId {
        self.carrier_context.module_id()
    }

    #[must_use]
    pub const fn carrier_context(&self) -> &SourcePropertyCarrierCoreContextHandoff {
        &self.carrier_context
    }

    #[must_use]
    pub const fn source_type(&self) -> &SourceTypeApplicationHandoff {
        &self.source_type
    }

    #[must_use]
    pub const fn carrier_item(&self) -> CoreItemId {
        self.carrier_context.carrier_item()
    }

    #[must_use]
    pub const fn domain(&self) -> &SourcePropertyDomainTypeAssociation {
        &self.domain
    }

    #[must_use]
    pub const fn association(&self) -> &SourcePropertySelectorTypeAssociation {
        &self.association
    }

    #[must_use]
    pub fn debug_text(&self) -> String {
        let property = &self.association;
        let identity = self.carrier_context.checker_owner().carrier_identity();
        format!(
            "source-property-selector-type-context-v1|module={}.{}|carrier-item={}|property={}:{}:{}:{}:{}",
            self.module_id().package().as_str(),
            self.module_id().path().as_str(),
            self.carrier_item().index(),
            property.symbol().fqn().as_str(),
            identity.property_definition().index(),
            identity.property_contribution().index(),
            property.member_type().index(),
            property.root().index(),
        )
    }

    fn validate(&self) -> Result<(), SourcePropertySelectorTypeContextError> {
        validate_property_selector_type_environment(&self.carrier_context, &self.source_type)?;
        self.carrier_context
            .validate()
            .map_err(|_| SourcePropertySelectorTypeContextError::InvalidCarrierContext)?;
        validate_property_selector_source_type(&self.carrier_context, &self.source_type)?;
        validate_property_selector_type_associations(
            &self.carrier_context,
            &self.source_type,
            &self.domain,
            &self.association,
        )
    }
}

/// Errors raised while authenticating the exact Task-264 selector/type rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourcePropertySelectorTypeContextError {
    EnvironmentMismatch,
    InvalidCarrierContext,
    InvalidSourceType,
    InvalidAssociation,
}

impl fmt::Display for SourcePropertySelectorTypeContextError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EnvironmentMismatch => {
                formatter.write_str("property selector/type context environment is invalid")
            }
            Self::InvalidCarrierContext => {
                formatter.write_str("property selector/type carrier context is invalid")
            }
            Self::InvalidSourceType => {
                formatter.write_str("property selector/type source type is invalid")
            }
            Self::InvalidAssociation => {
                formatter.write_str("property selector/type association is invalid")
            }
        }
    }
}

impl Error for SourcePropertySelectorTypeContextError {}

/// Builds the standalone immutable Task-264 selector/type context handoff.
#[derive(Debug, Clone, Copy)]
pub struct SourcePropertySelectorTypeContextProducer;

impl SourcePropertySelectorTypeContextProducer {
    pub fn build(
        carrier_context: SourcePropertyCarrierCoreContextHandoff,
        source_type: SourceTypeApplicationHandoff,
    ) -> Result<SourcePropertySelectorTypeContextHandoff, SourcePropertySelectorTypeContextError>
    {
        validate_property_selector_type_environment(&carrier_context, &source_type)?;
        carrier_context
            .validate()
            .map_err(|_| SourcePropertySelectorTypeContextError::InvalidCarrierContext)?;
        validate_property_selector_source_type(&carrier_context, &source_type)?;

        let identity = carrier_context.checker_owner().carrier_identity();
        let parameter = carrier_context
            .checker_owner()
            .parameters()
            .iter()
            .next()
            .map(|(_, row)| row)
            .ok_or(SourcePropertySelectorTypeContextError::InvalidSourceType)?;
        let application = source_type
            .applications()
            .get(parameter.written_type())
            .ok_or(SourcePropertySelectorTypeContextError::InvalidSourceType)?;
        let domain = SourcePropertyDomainTypeAssociation {
            binding: parameter.binding(),
            application: application.id(),
            root: application.root(),
            carrier_item: carrier_context.carrier_item(),
        };
        let association = SourcePropertySelectorTypeAssociation {
            symbol: identity.property_symbol().clone(),
            member_type: SourceTypeStructureMemberId::new(1),
            root: SourceTypeExpressionId::new(2),
        };
        validate_property_selector_type_associations(
            &carrier_context,
            &source_type,
            &domain,
            &association,
        )?;
        let handoff = SourcePropertySelectorTypeContextHandoff {
            carrier_context,
            source_type,
            domain,
            association,
        };
        handoff.validate()?;
        Ok(handoff)
    }
}

fn validate_property_selector_type_environment(
    carrier_context: &SourcePropertyCarrierCoreContextHandoff,
    source_type: &SourceTypeApplicationHandoff,
) -> Result<(), SourcePropertySelectorTypeContextError> {
    if carrier_context.source_id() != source_type.source_id()
        || carrier_context.module_id() != source_type.module_id()
    {
        return Err(SourcePropertySelectorTypeContextError::EnvironmentMismatch);
    }
    Ok(())
}

fn validate_property_selector_source_type(
    carrier_context: &SourcePropertyCarrierCoreContextHandoff,
    source_type: &SourceTypeApplicationHandoff,
) -> Result<(), SourcePropertySelectorTypeContextError> {
    let checker_owner = carrier_context.checker_owner();
    if source_type.debug_text() != checker_owner.source_type_fingerprint()
        || source_type.applications().len() != 1
        || source_type.expressions().len() != 3
        || !source_type.arguments().is_empty()
        || !source_type.definition_returns().is_empty()
        || !source_type.mode_rhs().is_empty()
        || source_type.structure_members().len() != 2
    {
        return Err(SourcePropertySelectorTypeContextError::InvalidSourceType);
    }

    let application = source_type
        .applications()
        .get(SourceTypeApplicationId::new(0))
        .ok_or(SourcePropertySelectorTypeContextError::InvalidSourceType)?;
    if application.id().index() != 0
        || application.binding() != BindingId::new(0)
        || application.source_ordinal() != 0
        || application.root() != SourceTypeExpressionId::new(0)
    {
        return Err(SourcePropertySelectorTypeContextError::InvalidSourceType);
    }

    let implementation = checker_owner
        .implementations()
        .iter()
        .next()
        .map(|(_, row)| row)
        .ok_or(SourcePropertySelectorTypeContextError::InvalidSourceType)?;
    let (application_site, application_head_site, member_sites) = match implementation.style() {
        SourcePropertyImplementationStyle::Means => (63, 64, [56, 59]),
        SourcePropertyImplementationStyle::Equals => (45, 46, [38, 41]),
        _ => return Err(SourcePropertySelectorTypeContextError::InvalidSourceType),
    };
    let identity = checker_owner.carrier_identity();
    for (index, (site, head_site, start, end)) in [
        (application_site, application_head_site, 130, 144),
        (member_sites[0] - 1, member_sites[0] - 2, 62, 65),
        (member_sites[1] - 1, member_sites[1] - 2, 90, 93),
    ]
    .into_iter()
    .enumerate()
    {
        let expression = source_type
            .expressions()
            .get(SourceTypeExpressionId::new(index))
            .ok_or(SourcePropertySelectorTypeContextError::InvalidSourceType)?;
        let expected_range = SourceRange {
            source_id: source_type.source_id(),
            start,
            end,
        };
        let head_matches = match (index, expression.head()) {
            (
                0,
                SourceTypeHead::Symbol {
                    symbol,
                    contribution,
                },
            ) => {
                symbol == identity.structure_symbol()
                    && *contribution == identity.structure_contribution()
            }
            (1 | 2, SourceTypeHead::BuiltinSet) => true,
            _ => false,
        };
        let spelling = if index == 0 { "Task264Carrier" } else { "set" };
        if expression.id().index() != index
            || expression.source_id() != source_type.source_id()
            || expression.module_id() != source_type.module_id()
            || expression.site() != &TypedSiteRef::Node(TypedNodeId::new(site))
            || expression.source_range() != expected_range
            || expression.spelling() != spelling
            || expression.head_site() != &TypedSiteRef::Node(TypedNodeId::new(head_site))
            || expression.head_range() != expected_range
            || expression.head_spelling() != spelling
            || expression.form() != SourceTypeApplicationForm::Bare
            || expression.recovery() != NodeRecoveryState::Normal
            || !head_matches
        {
            return Err(SourcePropertySelectorTypeContextError::InvalidSourceType);
        }
    }

    for (index, (site, start, end)) in [(member_sites[0], 45, 66), (member_sites[1], 71, 94)]
        .into_iter()
        .enumerate()
    {
        let member = source_type
            .structure_members()
            .get(SourceTypeStructureMemberId::new(index))
            .ok_or(SourcePropertySelectorTypeContextError::InvalidSourceType)?;
        if member.id().index() != index
            || member.member_site() != &TypedSiteRef::Node(TypedNodeId::new(site))
            || member.member_range()
                != (SourceRange {
                    source_id: source_type.source_id(),
                    start,
                    end,
                })
            || member.source_ordinal() != index
            || member.root() != SourceTypeExpressionId::new(index + 1)
        {
            return Err(SourcePropertySelectorTypeContextError::InvalidSourceType);
        }
    }

    let parameter = checker_owner
        .parameters()
        .iter()
        .next()
        .map(|(_, row)| row)
        .ok_or(SourcePropertySelectorTypeContextError::InvalidSourceType)?;
    let target = checker_owner
        .targets()
        .iter()
        .next()
        .map(|(_, row)| row)
        .ok_or(SourcePropertySelectorTypeContextError::InvalidSourceType)?;
    if parameter.id().index() != 0
        || parameter.owner().index() != 0
        || parameter.ordinal() != 0
        || parameter.binding() != BindingId::new(0)
        || parameter.written_type() != SourceTypeApplicationId::new(0)
        || target.id().index() != 0
        || target.owner().index() != 0
        || target.ordinal() != 0
        || target.subject() != parameter.binding()
        || target.symbol() != identity.property_symbol()
        || target.definition() != identity.property_definition()
        || target.contribution() != identity.property_contribution()
        || target.return_type() != SourceTypeStructureMemberId::new(1)
    {
        return Err(SourcePropertySelectorTypeContextError::InvalidSourceType);
    }
    Ok(())
}

fn validate_property_selector_type_associations(
    carrier_context: &SourcePropertyCarrierCoreContextHandoff,
    source_type: &SourceTypeApplicationHandoff,
    domain: &SourcePropertyDomainTypeAssociation,
    association: &SourcePropertySelectorTypeAssociation,
) -> Result<(), SourcePropertySelectorTypeContextError> {
    let identity = carrier_context.checker_owner().carrier_identity();
    let parameter = carrier_context
        .checker_owner()
        .parameters()
        .iter()
        .next()
        .map(|(_, row)| row)
        .ok_or(SourcePropertySelectorTypeContextError::InvalidAssociation)?;
    let application = source_type
        .applications()
        .get(parameter.written_type())
        .ok_or(SourcePropertySelectorTypeContextError::InvalidAssociation)?;
    let domain_expression = source_type
        .expressions()
        .get(application.root())
        .ok_or(SourcePropertySelectorTypeContextError::InvalidAssociation)?;
    let target = carrier_context
        .checker_owner()
        .targets()
        .iter()
        .next()
        .map(|(_, row)| row)
        .ok_or(SourcePropertySelectorTypeContextError::InvalidAssociation)?;
    let member = source_type
        .structure_members()
        .get(target.return_type())
        .ok_or(SourcePropertySelectorTypeContextError::InvalidAssociation)?;
    let domain_head_matches = matches!(
        domain_expression.head(),
        SourceTypeHead::Symbol {
            symbol,
            contribution,
        } if symbol == identity.structure_symbol()
            && *contribution == identity.structure_contribution()
    );
    if domain.binding() != parameter.binding()
        || domain.application() != parameter.written_type()
        || domain.application() != application.id()
        || application.binding() != domain.binding()
        || domain.root() != application.root()
        || domain.root() != domain_expression.id()
        || !domain_head_matches
        || domain.carrier_item() != carrier_context.carrier_item()
        || carrier_context
            .context()
            .item_registry()
            .id_for_symbol(identity.structure_symbol())
            != Some(domain.carrier_item())
        || target.subject() != domain.binding()
        || association.symbol() != identity.property_symbol()
        || association.symbol() != target.symbol()
        || association.member_type() != target.return_type()
        || association.member_type() != SourceTypeStructureMemberId::new(1)
        || association.root() != member.root()
        || association.root() != SourceTypeExpressionId::new(2)
        || association.symbol().module() != carrier_context.module_id()
    {
        return Err(SourcePropertySelectorTypeContextError::InvalidAssociation);
    }
    Ok(())
}

/// One immutable Core variable associated with the exact Task-264 property
/// parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePropertyParameterCoreVariableAssociation {
    parameter: SourcePropertyParameterId,
    binding: BindingId,
    core_var: CoreVarId,
}

impl SourcePropertyParameterCoreVariableAssociation {
    #[must_use]
    pub const fn parameter(&self) -> SourcePropertyParameterId {
        self.parameter
    }

    #[must_use]
    pub const fn binding(&self) -> BindingId {
        self.binding
    }

    #[must_use]
    pub const fn core_var(&self) -> CoreVarId {
        self.core_var
    }
}

/// Immutable Task-264 parameter/Core context built from complete branded
/// checker and Core handoffs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePropertyParameterCoreContextHandoff {
    selector_context: SourcePropertySelectorTypeContextHandoff,
    source_context: SourceBindingContextHandoff,
    source_bindings: SourceBindingCoreContextHandoff,
    association: SourcePropertyParameterCoreVariableAssociation,
}

impl SourcePropertyParameterCoreContextHandoff {
    #[must_use]
    pub const fn source_id(&self) -> SourceId {
        self.selector_context.source_id()
    }

    #[must_use]
    pub const fn module_id(&self) -> &ModuleId {
        self.selector_context.module_id()
    }

    #[must_use]
    pub const fn context(&self) -> &CoreContext {
        self.source_bindings.context()
    }

    #[must_use]
    pub const fn selector_context(&self) -> &SourcePropertySelectorTypeContextHandoff {
        &self.selector_context
    }

    #[must_use]
    pub const fn source_context(&self) -> &SourceBindingContextHandoff {
        &self.source_context
    }

    #[must_use]
    pub const fn source_bindings(&self) -> &SourceBindingCoreContextHandoff {
        &self.source_bindings
    }

    #[must_use]
    pub const fn association(&self) -> &SourcePropertyParameterCoreVariableAssociation {
        &self.association
    }

    #[must_use]
    pub fn debug_text(&self) -> String {
        format!(
            "source-property-parameter-core-context-v1|module={}.{}|carrier-item={}|bindings={}|parameter={}:{}:{}",
            self.module_id().package().as_str(),
            self.module_id().path().as_str(),
            self.selector_context.carrier_item().index(),
            self.source_bindings.binding_env().bindings().len(),
            self.association.parameter().index(),
            self.association.binding().index(),
            self.association.core_var().index(),
        )
    }

    fn validate(&self) -> Result<(), SourcePropertyParameterCoreContextError> {
        validate_property_parameter_environment(&self.selector_context, &self.source_context)?;
        self.selector_context
            .validate()
            .map_err(|_| SourcePropertyParameterCoreContextError::InvalidSelectorContext)?;
        validate_property_parameter_source_context(&self.selector_context, &self.source_context)?;
        let expected =
            build_property_parameter_source_bindings(&self.selector_context, &self.source_context)?;
        if self.source_bindings != expected {
            return Err(SourcePropertyParameterCoreContextError::InvalidBindingContext);
        }
        validate_property_parameter_association(
            &self.selector_context,
            &self.source_context,
            &self.source_bindings,
            &self.association,
        )
    }
}

/// Errors raised while associating the exact Task-264 parameter with Core.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourcePropertyParameterCoreContextError {
    EnvironmentMismatch,
    InvalidSelectorContext,
    InvalidSourceContext,
    InvalidBindingContext,
    InvalidAssociation,
}

impl fmt::Display for SourcePropertyParameterCoreContextError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EnvironmentMismatch => {
                formatter.write_str("property parameter Core environment is invalid")
            }
            Self::InvalidSelectorContext => {
                formatter.write_str("property parameter selector context is invalid")
            }
            Self::InvalidSourceContext => {
                formatter.write_str("property parameter source context is invalid")
            }
            Self::InvalidBindingContext => {
                formatter.write_str("property parameter Core binding context is invalid")
            }
            Self::InvalidAssociation => {
                formatter.write_str("property parameter Core association is invalid")
            }
        }
    }
}

impl Error for SourcePropertyParameterCoreContextError {}

/// Builds the standalone immutable Task-264 parameter/Core context handoff.
#[derive(Debug, Clone, Copy)]
pub struct SourcePropertyParameterCoreContextProducer;

impl SourcePropertyParameterCoreContextProducer {
    pub fn build(
        selector_context: SourcePropertySelectorTypeContextHandoff,
        source_context: SourceBindingContextHandoff,
    ) -> Result<SourcePropertyParameterCoreContextHandoff, SourcePropertyParameterCoreContextError>
    {
        validate_property_parameter_environment(&selector_context, &source_context)?;
        selector_context
            .validate()
            .map_err(|_| SourcePropertyParameterCoreContextError::InvalidSelectorContext)?;
        validate_property_parameter_source_context(&selector_context, &source_context)?;
        let source_bindings =
            build_property_parameter_source_bindings(&selector_context, &source_context)?;
        let parameter = selector_context
            .carrier_context()
            .checker_owner()
            .parameters()
            .iter()
            .next()
            .map(|(_, row)| row)
            .ok_or(SourcePropertyParameterCoreContextError::InvalidAssociation)?;
        let variable = source_bindings
            .variables()
            .get(parameter.binding())
            .ok_or(SourcePropertyParameterCoreContextError::InvalidAssociation)?;
        let association = SourcePropertyParameterCoreVariableAssociation {
            parameter: parameter.id(),
            binding: parameter.binding(),
            core_var: variable.core_var(),
        };
        validate_property_parameter_association(
            &selector_context,
            &source_context,
            &source_bindings,
            &association,
        )?;
        let handoff = SourcePropertyParameterCoreContextHandoff {
            selector_context,
            source_context,
            source_bindings,
            association,
        };
        handoff.validate()?;
        Ok(handoff)
    }
}

fn validate_property_parameter_environment(
    selector_context: &SourcePropertySelectorTypeContextHandoff,
    source_context: &SourceBindingContextHandoff,
) -> Result<(), SourcePropertyParameterCoreContextError> {
    let source_id = selector_context.source_id();
    let module_id = selector_context.module_id();
    if source_context.source_id() != source_id
        || source_context.module_id() != module_id
        || source_context.binding_env().source_id() != source_id
        || source_context.binding_env().module_id() != module_id
        || selector_context.carrier_context().source_id() != source_id
        || selector_context.carrier_context().module_id() != module_id
        || selector_context.source_type().source_id() != source_id
        || selector_context.source_type().module_id() != module_id
    {
        return Err(SourcePropertyParameterCoreContextError::EnvironmentMismatch);
    }
    Ok(())
}

fn validate_property_parameter_source_context(
    selector_context: &SourcePropertySelectorTypeContextHandoff,
    source_context: &SourceBindingContextHandoff,
) -> Result<(), SourcePropertyParameterCoreContextError> {
    let checker_owner = selector_context.carrier_context().checker_owner();
    let implementation = checker_owner
        .implementations()
        .iter()
        .next()
        .map(|(_, row)| row)
        .ok_or(SourcePropertyParameterCoreContextError::InvalidSourceContext)?;
    let parameter = checker_owner
        .parameters()
        .iter()
        .next()
        .map(|(_, row)| row)
        .ok_or(SourcePropertyParameterCoreContextError::InvalidSourceContext)?;
    let target = checker_owner
        .targets()
        .iter()
        .next()
        .map(|(_, row)| row)
        .ok_or(SourcePropertyParameterCoreContextError::InvalidSourceContext)?;
    if source_context.debug_text() != checker_owner.source_context_fingerprint()
        || source_context.items().len() != 1
        || source_context.declarations().len() != 1
        || source_context.context_links().len() != 2
        || source_context.local_contexts().len() != 2
        || source_context.binding_env().contexts().len() != 2
        || source_context.binding_env().bindings().len() != 1
        || !source_context.binding_env().diagnostics().is_empty()
        || checker_owner.implementations().len() != 1
        || checker_owner.parameters().len() != 1
        || checker_owner.targets().len() != 1
        || parameter.id() != SourcePropertyParameterId::new(0)
        || parameter.owner().index() != 0
        || parameter.ordinal() != 0
        || parameter.binding() != BindingId::new(0)
        || parameter.written_type() != selector_context.domain().application()
        || parameter.binding() != selector_context.domain().binding()
        || target.subject() != parameter.binding()
        || implementation.parameter() != parameter.id()
    {
        return Err(SourcePropertyParameterCoreContextError::InvalidSourceContext);
    }

    let item = source_context
        .items()
        .get(SourceItemId::new(0))
        .ok_or(SourcePropertyParameterCoreContextError::InvalidSourceContext)?;
    let declaration = source_context
        .declarations()
        .get(SourceDeclarationId::new(0))
        .ok_or(SourcePropertyParameterCoreContextError::InvalidSourceContext)?;
    let SourceBindingSiteRole::DefinitionParameter { local } = &declaration.role else {
        return Err(SourcePropertyParameterCoreContextError::InvalidSourceContext);
    };
    if item.id != SourceItemId::new(0)
        || item.shell != implementation.shell()
        || item.shell_ordinal != implementation.shell().index()
        || item.role != SourceItemRole::PropertyImplementation
        || item.source_range != implementation.source_range()
        || item.parent.is_some()
        || item.visibility != SourceItemVisibility::Unspecified
        || item.site != *implementation.site()
        || item
            .local_scope
            .as_ref()
            .is_none_or(|scope| scope.path() != [4])
        || item.recovery != SourceItemRecovery::Normal
        || item.binding_context != BindingContextId::new(1)
        || item.local_context != mizar_checker::typed_ast::LocalTypeContextId::new(1)
        || item.predecessor.is_some()
        || declaration.id != SourceDeclarationId::new(0)
        || declaration.item != item.id
        || declaration.binding != parameter.binding()
        || declaration.source_ordinal != parameter.ordinal()
        || declaration.spelling != "M"
        || declaration.declaration_range != parameter.declaration_range()
        || declaration.written_type_range.start != 130
        || declaration.written_type_range.end != 144
        || declaration.site != *parameter.site()
        || declaration.binding_context != parameter.context()
        || declaration.local_context != mizar_checker::typed_ast::LocalTypeContextId::new(1)
        || declaration.shadowed_binding.is_some()
        || declaration.predecessor.is_some()
        || local.spelling() != "M"
        || local.scope().path() != [4]
        || local.declaration_range() != parameter.declaration_range()
        || local.visible_after_ordinal() != 0
    {
        return Err(SourcePropertyParameterCoreContextError::InvalidSourceContext);
    }

    let module_context = source_context
        .binding_env()
        .contexts()
        .get(BindingContextId::new(0))
        .ok_or(SourcePropertyParameterCoreContextError::InvalidSourceContext)?;
    let definition_context = source_context
        .binding_env()
        .contexts()
        .get(BindingContextId::new(1))
        .ok_or(SourcePropertyParameterCoreContextError::InvalidSourceContext)?;
    let binding = source_context
        .binding_env()
        .bindings()
        .get(parameter.binding())
        .ok_or(SourcePropertyParameterCoreContextError::InvalidSourceContext)?;
    let BinderIdentity::ResolverLocal {
        scope,
        ordinal,
        declaration_range,
    } = &binding.identity
    else {
        return Err(SourcePropertyParameterCoreContextError::InvalidSourceContext);
    };
    if module_context.id != BindingContextId::new(0)
        || !is_normal_module_context(module_context)
        || !module_context.bindings.is_empty()
        || !module_context.visible_bindings.is_empty()
        || definition_context.id != BindingContextId::new(1)
        || !is_normal_declaration_context(definition_context)
        || definition_context.owner != BindingContextOwner::DeclarationShell(item.shell)
        || definition_context.parent != Some(BindingContextId::new(0))
        || definition_context
            .lexical_scope
            .as_ref()
            .is_none_or(|scope| scope.path() != [4])
        || definition_context.bindings != [parameter.binding()]
        || definition_context.visible_bindings != [parameter.binding()]
        || binding.id != parameter.binding()
        || binding.spelling != "M"
        || binding.kind != BindingKind::DefinitionParameter
        || binding.owner_context != BindingContextId::new(1)
        || binding.declaration_range != parameter.declaration_range()
        || binding.visible_after_ordinal != 0
        || binding.type_site != BindingTypeSite::Source(declaration.written_type_range)
        || binding.status != BindingStatus::Active
        || !binding.captured.identities().is_empty()
        || !binding.diagnostics.is_empty()
        || binding.recovery != BindingRecoveryState::Normal
        || scope.path() != [4]
        || *ordinal != 0
        || *declaration_range != parameter.declaration_range()
    {
        return Err(SourcePropertyParameterCoreContextError::InvalidSourceContext);
    }

    for index in 0..2 {
        let link = source_context
            .context_links()
            .get(BindingContextId::new(index))
            .ok_or(SourcePropertyParameterCoreContextError::InvalidSourceContext)?;
        if link.binding_context != BindingContextId::new(index)
            || link.local_context != mizar_checker::typed_ast::LocalTypeContextId::new(index)
            || link.item != (index == 1).then_some(item.id)
        {
            return Err(SourcePropertyParameterCoreContextError::InvalidSourceContext);
        }
    }
    Ok(())
}

fn build_property_parameter_source_bindings(
    selector_context: &SourcePropertySelectorTypeContextHandoff,
    source_context: &SourceBindingContextHandoff,
) -> Result<SourceBindingCoreContextHandoff, SourcePropertyParameterCoreContextError> {
    SourceBindingCoreContextProducer::build(
        selector_context.carrier_context().context().clone(),
        source_context.binding_env().clone(),
    )
    .map_err(|_| SourcePropertyParameterCoreContextError::InvalidBindingContext)
}

fn validate_property_parameter_association(
    selector_context: &SourcePropertySelectorTypeContextHandoff,
    source_context: &SourceBindingContextHandoff,
    source_bindings: &SourceBindingCoreContextHandoff,
    association: &SourcePropertyParameterCoreVariableAssociation,
) -> Result<(), SourcePropertyParameterCoreContextError> {
    source_bindings
        .validate()
        .map_err(|_| SourcePropertyParameterCoreContextError::InvalidBindingContext)?;
    let parameter = selector_context
        .carrier_context()
        .checker_owner()
        .parameters()
        .get(association.parameter())
        .ok_or(SourcePropertyParameterCoreContextError::InvalidAssociation)?;
    let variable = source_bindings
        .variables()
        .get(association.binding())
        .ok_or(SourcePropertyParameterCoreContextError::InvalidAssociation)?;
    let source_record = source_bindings
        .context()
        .binder_sources()
        .get(association.core_var())
        .ok_or(SourcePropertyParameterCoreContextError::InvalidAssociation)?;
    let expected_key = source_binding_provenance_key(association.binding());
    let expected_provenance =
        CoreProvenance::new(CoreProvenancePhase::Checker, expected_key.clone());
    if association.parameter() != SourcePropertyParameterId::new(0)
        || association.parameter() != parameter.id()
        || association.binding() != BindingId::new(0)
        || association.binding() != parameter.binding()
        || association.binding() != selector_context.domain().binding()
        || association.core_var() != CoreVarId::new(0)
        || association.core_var() != variable.core_var()
        || source_bindings.variables().len() != 1
        || source_bindings.binding_env() != source_context.binding_env()
        || !source_bindings.context().binder_context().frames.is_empty()
        || source_bindings.context().binder_context().free_variables
            != BTreeSet::from([association.core_var()])
        || source_bindings
            .context()
            .binder_context()
            .variable_classes
            .get(&association.core_var())
            != Some(&NormalizedVarClass::Free)
        || source_bindings
            .context()
            .binder_context()
            .variable_roles
            .get(&association.core_var())
            .map(CoreVarRole::as_str)
            != Some(SOURCE_BINDING_CORE_PARAMETER_ROLE)
        || source_bindings
            .context()
            .binder_context()
            .variable_sorts
            .get(&association.core_var())
            != Some(&NormalizedVarSort::Term)
        || source_bindings
            .context()
            .binder_type_facts()
            .get(&association.core_var())
            != Some(&Vec::new())
        || source_record.source.anchor
            != CoreSourceAnchor::SourceRange(parameter.declaration_range())
        || source_record.source.provenance.as_slice() != [expected_provenance.clone()]
        || source_record.provenance.as_slice()
            != [CoreProvenance::new(
                CoreProvenancePhase::Checker,
                expected_key,
            )]
        || source_bindings.context().item_registry().id_for_symbol(
            selector_context
                .carrier_context()
                .checker_owner()
                .carrier_identity()
                .structure_symbol(),
        ) != Some(selector_context.carrier_item())
    {
        return Err(SourcePropertyParameterCoreContextError::InvalidAssociation);
    }
    Ok(())
}

const SOURCE_PROPERTY_EQUALS_SELECTOR_BASE_TERM_SEED_PROVENANCE_KEY: &str =
    "source-property-equals-selector-term-seed-v1.base";
const SOURCE_PROPERTY_EQUALS_SELECTOR_TERM_SEED_PROVENANCE_KEY: &str =
    "source-property-equals-selector-term-seed-v1.selector";

/// Exact Task-264 source-term rows associated with their local Core term seeds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePropertyEqualsSelectorTermSeedAssociation {
    parameter: SourcePropertyParameterId,
    binding: BindingId,
    core_var: CoreVarId,
    source_base: SourcePrimaryTermId,
    base_seed: CoreTermSeedId,
    source_selector: SourceStructureTermId,
    selector_seed: CoreTermSeedId,
}

impl SourcePropertyEqualsSelectorTermSeedAssociation {
    #[must_use]
    pub const fn parameter(&self) -> SourcePropertyParameterId {
        self.parameter
    }

    #[must_use]
    pub const fn binding(&self) -> BindingId {
        self.binding
    }

    #[must_use]
    pub const fn core_var(&self) -> CoreVarId {
        self.core_var
    }

    #[must_use]
    pub const fn source_base(&self) -> SourcePrimaryTermId {
        self.source_base
    }

    #[must_use]
    pub const fn base_seed(&self) -> CoreTermSeedId {
        self.base_seed
    }

    #[must_use]
    pub const fn source_selector(&self) -> SourceStructureTermId {
        self.source_selector
    }

    #[must_use]
    pub const fn selector_seed(&self) -> CoreTermSeedId {
        self.selector_seed
    }
}

/// Immutable, property-owner-aware input for later Task-264 equals lowering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePropertyEqualsSelectorTermSeedHandoff {
    definition_owner: CoreDefinitionOwner,
    parameter_context: SourcePropertyParameterCoreContextHandoff,
    selector_identity: SourcePropertyEqualsSelectorIdentityHandoff,
    terms: Vec<CoreTermSeed>,
    association: SourcePropertyEqualsSelectorTermSeedAssociation,
}

impl SourcePropertyEqualsSelectorTermSeedHandoff {
    #[must_use]
    pub const fn source_id(&self) -> SourceId {
        self.parameter_context.source_id()
    }

    #[must_use]
    pub const fn module_id(&self) -> &ModuleId {
        self.parameter_context.module_id()
    }

    #[must_use]
    pub const fn definition_owner(&self) -> &CoreDefinitionOwner {
        &self.definition_owner
    }

    #[must_use]
    pub const fn parameter_context(&self) -> &SourcePropertyParameterCoreContextHandoff {
        &self.parameter_context
    }

    #[must_use]
    pub const fn selector_identity(&self) -> &SourcePropertyEqualsSelectorIdentityHandoff {
        &self.selector_identity
    }

    #[must_use]
    pub fn terms(&self) -> &[CoreTermSeed] {
        &self.terms
    }

    #[must_use]
    pub const fn association(&self) -> &SourcePropertyEqualsSelectorTermSeedAssociation {
        &self.association
    }

    #[must_use]
    pub fn debug_text(&self) -> String {
        format!(
            concat!(
                "source-property-equals-selector-term-seeds-v1|module={}.{}|",
                "owner-anchor={}|property={}|selector={}|source={}:{}|seed={}:{}|",
                "parameter={}:{}:{}"
            ),
            self.module_id().package().as_str(),
            self.module_id().path().as_str(),
            self.definition_owner.anchor_item().index(),
            self.definition_owner
                .property_symbol()
                .expect("validated property owner")
                .fqn()
                .as_str(),
            self.selector_identity
                .association()
                .selector_symbol()
                .fqn()
                .as_str(),
            self.association.source_base().index(),
            self.association.source_selector().index(),
            self.association.base_seed().index(),
            self.association.selector_seed().index(),
            self.association.parameter().index(),
            self.association.binding().index(),
            self.association.core_var().index(),
        )
    }

    fn validate(&self) -> Result<(), SourcePropertyEqualsSelectorTermSeedError> {
        validate_property_equals_selector_term_seed_environment(
            &self.parameter_context,
            &self.selector_identity,
        )?;
        self.parameter_context
            .validate()
            .map_err(|_| SourcePropertyEqualsSelectorTermSeedError::InvalidParameterContext)?;
        validate_property_equals_selector_identity(
            &self.parameter_context,
            &self.selector_identity,
        )?;
        validate_property_equals_selector_definition_owner(
            &self.parameter_context,
            &self.definition_owner,
        )?;
        validate_property_equals_selector_term_seeds(
            &self.parameter_context,
            &self.selector_identity,
            &self.terms,
            &self.association,
        )
    }
}

/// Errors raised while retaining the exact Task-264 equals selector term seeds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourcePropertyEqualsSelectorTermSeedError {
    EnvironmentMismatch,
    InvalidParameterContext,
    InvalidSelectorIdentity,
    InvalidDefinitionOwner,
    InvalidTermSeeds,
}

impl fmt::Display for SourcePropertyEqualsSelectorTermSeedError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EnvironmentMismatch => {
                formatter.write_str("property equals selector term seed environment is invalid")
            }
            Self::InvalidParameterContext => {
                formatter.write_str("property equals selector parameter context is invalid")
            }
            Self::InvalidSelectorIdentity => {
                formatter.write_str("property equals selector identity is invalid")
            }
            Self::InvalidDefinitionOwner => {
                formatter.write_str("property equals selector definition owner is invalid")
            }
            Self::InvalidTermSeeds => {
                formatter.write_str("property equals selector term seeds are invalid")
            }
        }
    }
}

impl Error for SourcePropertyEqualsSelectorTermSeedError {}

/// Builds the standalone Task-264 equals selector term-seed handoff.
#[derive(Debug, Clone, Copy)]
pub struct SourcePropertyEqualsSelectorTermSeedProducer;

impl SourcePropertyEqualsSelectorTermSeedProducer {
    pub fn build(
        parameter_context: SourcePropertyParameterCoreContextHandoff,
        selector_identity: SourcePropertyEqualsSelectorIdentityHandoff,
    ) -> Result<
        SourcePropertyEqualsSelectorTermSeedHandoff,
        SourcePropertyEqualsSelectorTermSeedError,
    > {
        validate_property_equals_selector_term_seed_environment(
            &parameter_context,
            &selector_identity,
        )?;
        parameter_context
            .validate()
            .map_err(|_| SourcePropertyEqualsSelectorTermSeedError::InvalidParameterContext)?;
        validate_property_equals_selector_identity(&parameter_context, &selector_identity)?;

        let definition_owner = parameter_context.selector_context().definition_owner();
        validate_property_equals_selector_definition_owner(&parameter_context, &definition_owner)?;

        let association = SourcePropertyEqualsSelectorTermSeedAssociation {
            parameter: SourcePropertyParameterId::new(0),
            binding: BindingId::new(0),
            core_var: CoreVarId::new(0),
            source_base: SourcePrimaryTermId::new(0),
            base_seed: CoreTermSeedId::new(0),
            source_selector: SourceStructureTermId::new(0),
            selector_seed: CoreTermSeedId::new(1),
        };
        let base = selector_identity
            .terms()
            .terms()
            .get(association.source_base())
            .ok_or(SourcePropertyEqualsSelectorTermSeedError::InvalidTermSeeds)?;
        let selector = selector_identity
            .structures()
            .terms()
            .get(association.source_selector())
            .ok_or(SourcePropertyEqualsSelectorTermSeedError::InvalidTermSeeds)?;
        let terms = vec![
            CoreTermSeed::new(
                CoreTermSeedKind::Var(association.core_var()),
                CoreSourceRef::direct(base.source_range()),
                CheckerOwnedProvenance::checker(
                    SOURCE_PROPERTY_EQUALS_SELECTOR_BASE_TERM_SEED_PROVENANCE_KEY,
                ),
            ),
            CoreTermSeed::new(
                CoreTermSeedKind::Select {
                    selector: selector_identity.association().selector_symbol().clone(),
                    base: association.base_seed(),
                },
                CoreSourceRef::direct(selector.source_range()),
                CheckerOwnedProvenance::checker(
                    SOURCE_PROPERTY_EQUALS_SELECTOR_TERM_SEED_PROVENANCE_KEY,
                ),
            ),
        ];
        validate_property_equals_selector_term_seeds(
            &parameter_context,
            &selector_identity,
            &terms,
            &association,
        )?;

        let handoff = SourcePropertyEqualsSelectorTermSeedHandoff {
            definition_owner,
            parameter_context,
            selector_identity,
            terms,
            association,
        };
        handoff.validate()?;
        Ok(handoff)
    }
}

fn validate_property_equals_selector_term_seed_environment(
    parameter_context: &SourcePropertyParameterCoreContextHandoff,
    selector_identity: &SourcePropertyEqualsSelectorIdentityHandoff,
) -> Result<(), SourcePropertyEqualsSelectorTermSeedError> {
    if parameter_context.source_id() != selector_identity.source_id()
        || parameter_context.module_id() != selector_identity.module_id()
    {
        return Err(SourcePropertyEqualsSelectorTermSeedError::EnvironmentMismatch);
    }
    Ok(())
}

fn validate_property_equals_selector_identity(
    parameter_context: &SourcePropertyParameterCoreContextHandoff,
    selector_identity: &SourcePropertyEqualsSelectorIdentityHandoff,
) -> Result<(), SourcePropertyEqualsSelectorTermSeedError> {
    let property = selector_identity.property();
    let association = selector_identity.association();
    let implementation = property
        .implementations()
        .iter()
        .next()
        .map(|(_, row)| row)
        .ok_or(SourcePropertyEqualsSelectorTermSeedError::InvalidSelectorIdentity)?;
    let base = selector_identity
        .terms()
        .terms()
        .get(association.base_term())
        .ok_or(SourcePropertyEqualsSelectorTermSeedError::InvalidSelectorIdentity)?;
    let reference = selector_identity
        .terms()
        .references()
        .get(association.base_reference())
        .ok_or(SourcePropertyEqualsSelectorTermSeedError::InvalidSelectorIdentity)?;
    let selector = selector_identity
        .structures()
        .terms()
        .get(association.structure_term())
        .ok_or(SourcePropertyEqualsSelectorTermSeedError::InvalidSelectorIdentity)?;
    let member = selector_identity
        .structures()
        .members()
        .get(association.member())
        .ok_or(SourcePropertyEqualsSelectorTermSeedError::InvalidSelectorIdentity)?;
    let edge = selector_identity
        .structures()
        .edges()
        .get(association.base_edge())
        .ok_or(SourcePropertyEqualsSelectorTermSeedError::InvalidSelectorIdentity)?;
    let request = selector_identity
        .structures()
        .requests()
        .get(association.member_request())
        .ok_or(SourcePropertyEqualsSelectorTermSeedError::InvalidSelectorIdentity)?;
    let source_id = parameter_context.source_id();
    let parameter = parameter_context.association();

    if parameter_context
        .selector_context()
        .carrier_context()
        .checker_owner()
        != property
        || implementation.style() != SourcePropertyImplementationStyle::Equals
        || property.implementations().len() != 1
        || property.parameters().len() != 1
        || property.targets().len() != 1
        || property.definientia().len() != 1
        || !property.correctness().is_empty()
        || property.source_functor_application_fingerprint().is_some()
        || property.source_structure_fingerprint()
            != Some(selector_identity.structures().debug_text().as_str())
        || property.source_term_fingerprint() != selector_identity.terms().debug_text()
        || property.source_set_term_fingerprint().is_some()
        || property.source_atomic_formula_fingerprint().is_some()
        || selector_identity.structures().primary_term_fingerprint()
            != selector_identity.terms().debug_text()
        || selector_identity
            .structures()
            .application_fingerprint()
            .is_some()
        || association.implementation().index() != 0
        || association.definiens().index() != 0
        || association.structure_term() != SourceStructureTermId::new(0)
        || association.member() != SourceStructureMemberId::new(0)
        || association.member_request() != SourceStructureRequestId::new(0)
        || association.base_edge() != SourceStructureEdgeId::new(0)
        || association.base_term() != SourcePrimaryTermId::new(0)
        || association.base_reference() != SourcePrimaryTermReferenceId::new(0)
        || association.base_binding() != BindingId::new(0)
        || association.base_binding() != parameter.binding()
        || association.selector_symbol() != property.carrier_identity().field_symbol()
        || selector_identity.terms().terms().len() != 1
        || selector_identity.terms().references().len() != 1
        || !selector_identity.terms().numeric_type_requests().is_empty()
        || base.kind() != SourcePrimaryTermKind::VariableReference
        || base.role() != SourcePrimaryTermRole::Value
        || base.recovery() != SourcePrimaryTermRecovery::Normal
        || base.parent().is_some()
        || base.source_ordinal() != 0
        || base.context() != BindingContextId::new(1)
        || base.site() != &TypedSiteRef::Node(TypedNodeId::new(48))
        || base.source_range()
            != (SourceRange {
                source_id,
                start: 173,
                end: 174,
            })
        || base.spelling() != "M"
        || reference.term() != association.base_term()
        || reference.binding() != association.base_binding()
        || reference.role() != SourcePrimaryTermReferenceRole::Variable
        || reference
            .lexical_scope()
            .is_none_or(|scope| scope.path() != [4])
        || reference.use_ordinal() != 1
        || selector_identity.structures().terms().len() != 1
        || !selector_identity.structures().wrappers().is_empty()
        || !selector_identity.structures().roots().is_empty()
        || selector_identity.structures().members().len() != 1
        || !selector_identity.structures().field_updates().is_empty()
        || selector_identity.structures().edges().len() != 1
        || selector_identity.structures().requests().len() != 3
        || selector.kind() != SourceStructureTermKind::SelectorAccess
        || selector.recovery() != SourceStructureRecovery::Normal
        || selector.source_ordinal() != 0
        || selector.context() != BindingContextId::new(1)
        || selector.site() != &TypedSiteRef::Node(TypedNodeId::new(49))
        || selector.source_range()
            != (SourceRange {
                source_id,
                start: 173,
                end: 182,
            })
        || selector.spelling() != "M.carrier"
        || member.term() != association.structure_term()
        || member.ordinal() != 0
        || member.role() != SourceStructureMemberRole::Selector
        || member.parent().is_some()
        || member.site() != &TypedSiteRef::Node(TypedNodeId::new(31))
        || member.source_range()
            != (SourceRange {
                source_id,
                start: 175,
                end: 182,
            })
        || member.spelling() != "carrier"
        || edge.term() != association.structure_term()
        || edge.ordinal() != 0
        || edge.role() != SourceStructureEdgeRole::SelectorBase
        || edge.member().is_some()
        || edge.target() != SourceStructureTarget::Primary(association.base_term())
        || request.term() != association.structure_term()
        || request.member() != Some(association.member())
        || request.request_ordinal() != 0
        || request.kind() != SourceStructureRequestKind::MemberIdentity
    {
        return Err(SourcePropertyEqualsSelectorTermSeedError::InvalidSelectorIdentity);
    }
    Ok(())
}

fn validate_property_equals_selector_definition_owner(
    parameter_context: &SourcePropertyParameterCoreContextHandoff,
    owner: &CoreDefinitionOwner,
) -> Result<(), SourcePropertyEqualsSelectorTermSeedError> {
    let expected = parameter_context.selector_context().definition_owner();
    if owner != &expected
        || owner.anchor_item() != parameter_context.selector_context().carrier_item()
        || owner.anchor_item() != CoreItemId::new(0)
        || owner.item().is_some()
        || owner.property_symbol()
            != Some(parameter_context.selector_context().association().symbol())
    {
        return Err(SourcePropertyEqualsSelectorTermSeedError::InvalidDefinitionOwner);
    }
    Ok(())
}

fn validate_property_equals_selector_term_seeds(
    parameter_context: &SourcePropertyParameterCoreContextHandoff,
    selector_identity: &SourcePropertyEqualsSelectorIdentityHandoff,
    terms: &[CoreTermSeed],
    association: &SourcePropertyEqualsSelectorTermSeedAssociation,
) -> Result<(), SourcePropertyEqualsSelectorTermSeedError> {
    let parameter = parameter_context.association();
    let selector = selector_identity.association();
    let source_base = selector_identity
        .terms()
        .terms()
        .get(association.source_base())
        .ok_or(SourcePropertyEqualsSelectorTermSeedError::InvalidTermSeeds)?;
    let source_selector = selector_identity
        .structures()
        .terms()
        .get(association.source_selector())
        .ok_or(SourcePropertyEqualsSelectorTermSeedError::InvalidTermSeeds)?;
    let [base, selected] = terms else {
        return Err(SourcePropertyEqualsSelectorTermSeedError::InvalidTermSeeds);
    };
    let base_provenance = CoreProvenance::new(
        CoreProvenancePhase::Checker,
        SOURCE_PROPERTY_EQUALS_SELECTOR_BASE_TERM_SEED_PROVENANCE_KEY,
    );
    let selector_provenance = CoreProvenance::new(
        CoreProvenancePhase::Checker,
        SOURCE_PROPERTY_EQUALS_SELECTOR_TERM_SEED_PROVENANCE_KEY,
    );

    if association.parameter() != SourcePropertyParameterId::new(0)
        || association.parameter() != parameter.parameter()
        || association.binding() != BindingId::new(0)
        || association.binding() != parameter.binding()
        || association.binding() != selector.base_binding()
        || association.core_var() != CoreVarId::new(0)
        || association.core_var() != parameter.core_var()
        || association.source_base() != SourcePrimaryTermId::new(0)
        || association.source_base() != selector.base_term()
        || association.base_seed() != CoreTermSeedId::new(0)
        || association.source_selector() != SourceStructureTermId::new(0)
        || association.source_selector() != selector.structure_term()
        || association.selector_seed() != CoreTermSeedId::new(1)
        || base.kind != CoreTermSeedKind::Var(association.core_var())
        || base.source != CoreSourceRef::direct(source_base.source_range())
        || !base.source.provenance.is_empty()
        || base.provenance.as_slice() != [base_provenance]
        || selected.kind
            != (CoreTermSeedKind::Select {
                selector: selector.selector_symbol().clone(),
                base: association.base_seed(),
            })
        || selected.source != CoreSourceRef::direct(source_selector.source_range())
        || !selected.source.provenance.is_empty()
        || selected.provenance.as_slice() != [selector_provenance]
        || validate_checker_owned_provenance(
            "property equals base term seed",
            base.provenance.as_slice(),
        )
        .is_err()
        || validate_checker_owned_provenance(
            "property equals selector term seed",
            selected.provenance.as_slice(),
        )
        .is_err()
    {
        return Err(SourcePropertyEqualsSelectorTermSeedError::InvalidTermSeeds);
    }
    Ok(())
}

/// Local Task-264 seed-to-term and root association.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePropertyEqualsSelectorTermLoweringAssociation {
    base_seed: CoreTermSeedId,
    base_term: CoreTermId,
    selector_seed: CoreTermSeedId,
    selector_term: CoreTermId,
    root_term: CoreTermId,
}

impl SourcePropertyEqualsSelectorTermLoweringAssociation {
    #[must_use]
    pub const fn base_seed(&self) -> CoreTermSeedId {
        self.base_seed
    }

    #[must_use]
    pub const fn base_term(&self) -> CoreTermId {
        self.base_term
    }

    #[must_use]
    pub const fn selector_seed(&self) -> CoreTermSeedId {
        self.selector_seed
    }

    #[must_use]
    pub const fn selector_term(&self) -> CoreTermId {
        self.selector_term
    }

    #[must_use]
    pub const fn root_term(&self) -> CoreTermId {
        self.root_term
    }
}

/// Immutable unattached Task-264 equals selector term lowering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePropertyEqualsSelectorTermLoweringHandoff {
    seed_handoff: SourcePropertyEqualsSelectorTermSeedHandoff,
    terms: CoreTermTable,
    source_map: CoreSourceMap,
    association: SourcePropertyEqualsSelectorTermLoweringAssociation,
}

impl SourcePropertyEqualsSelectorTermLoweringHandoff {
    #[must_use]
    pub const fn source_id(&self) -> SourceId {
        self.seed_handoff.source_id()
    }

    #[must_use]
    pub const fn module_id(&self) -> &ModuleId {
        self.seed_handoff.module_id()
    }

    #[must_use]
    pub const fn definition_owner(&self) -> &CoreDefinitionOwner {
        self.seed_handoff.definition_owner()
    }

    #[must_use]
    pub const fn seed_handoff(&self) -> &SourcePropertyEqualsSelectorTermSeedHandoff {
        &self.seed_handoff
    }

    #[must_use]
    pub const fn terms(&self) -> &CoreTermTable {
        &self.terms
    }

    #[must_use]
    pub const fn source_map(&self) -> &CoreSourceMap {
        &self.source_map
    }

    #[must_use]
    pub const fn association(&self) -> &SourcePropertyEqualsSelectorTermLoweringAssociation {
        &self.association
    }

    #[must_use]
    pub fn debug_text(&self) -> String {
        format!(
            concat!(
                "source-property-equals-selector-term-lowering-v1|module={}.{}|",
                "owner-anchor={}|property={}|seed={}:{}|term={}:{}|root={}"
            ),
            self.module_id().package().as_str(),
            self.module_id().path().as_str(),
            self.definition_owner().anchor_item().index(),
            self.definition_owner()
                .property_symbol()
                .expect("validated property owner")
                .fqn()
                .as_str(),
            self.association.base_seed().index(),
            self.association.selector_seed().index(),
            self.association.base_term().index(),
            self.association.selector_term().index(),
            self.association.root_term().index(),
        )
    }

    fn validate(&self) -> Result<(), SourcePropertyEqualsSelectorTermLoweringError> {
        self.seed_handoff
            .validate()
            .map_err(|_| SourcePropertyEqualsSelectorTermLoweringError::InvalidSeedHandoff)?;
        validate_property_equals_selector_term_lowering_owner(&self.seed_handoff)?;
        validate_property_equals_selector_term_lowering_association(&self.association)?;
        validate_property_equals_selector_term_lowering_terms(
            &self.seed_handoff,
            &self.terms,
            &self.association,
        )?;
        validate_property_equals_selector_term_lowering_source_map(
            &self.terms,
            &self.source_map,
            &self.association,
        )
    }
}

/// Errors raised while lowering the exact Task-264 equals selector term graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourcePropertyEqualsSelectorTermLoweringError {
    InvalidSeedHandoff,
    InvalidTermLowering,
}

impl fmt::Display for SourcePropertyEqualsSelectorTermLoweringError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSeedHandoff => {
                formatter.write_str("property equals selector term seed handoff is invalid")
            }
            Self::InvalidTermLowering => {
                formatter.write_str("property equals selector term lowering is invalid")
            }
        }
    }
}

impl Error for SourcePropertyEqualsSelectorTermLoweringError {}

/// Builds the standalone Task-264 equals selector term lowering.
#[derive(Debug, Clone, Copy)]
pub struct SourcePropertyEqualsSelectorTermLoweringProducer;

impl SourcePropertyEqualsSelectorTermLoweringProducer {
    pub fn build(
        seed_handoff: SourcePropertyEqualsSelectorTermSeedHandoff,
    ) -> Result<
        SourcePropertyEqualsSelectorTermLoweringHandoff,
        SourcePropertyEqualsSelectorTermLoweringError,
    > {
        seed_handoff
            .validate()
            .map_err(|_| SourcePropertyEqualsSelectorTermLoweringError::InvalidSeedHandoff)?;
        validate_property_equals_selector_term_lowering_owner(&seed_handoff)?;

        let association = SourcePropertyEqualsSelectorTermLoweringAssociation {
            base_seed: CoreTermSeedId::new(0),
            base_term: CoreTermId::new(0),
            selector_seed: CoreTermSeedId::new(1),
            selector_term: CoreTermId::new(1),
            root_term: CoreTermId::new(1),
        };
        validate_property_equals_selector_term_lowering_association(&association)?;

        let [base_seed, selector_seed] = seed_handoff.terms() else {
            return Err(SourcePropertyEqualsSelectorTermLoweringError::InvalidTermLowering);
        };
        let base_source = normalized_source(source_with_provenance(
            base_seed.source.clone(),
            &base_seed.provenance,
        ));
        let selector_source = normalized_source(source_with_provenance(
            selector_seed.source.clone(),
            &selector_seed.provenance,
        ));
        let mut terms = CoreTermTable::new();
        let base_term = terms.insert(CoreTerm::new(
            CoreTermKind::Var(CoreVarId::new(0)),
            base_source.clone(),
        ));
        let selector_term = terms.insert(CoreTerm::new(
            CoreTermKind::Select {
                selector: seed_handoff
                    .selector_identity()
                    .association()
                    .selector_symbol()
                    .clone(),
                base: base_term,
            },
            selector_source.clone(),
        ));
        if base_term != association.base_term() || selector_term != association.selector_term() {
            return Err(SourcePropertyEqualsSelectorTermLoweringError::InvalidTermLowering);
        }

        let mut source_map = CoreSourceMap::new();
        source_map.term_sources.insert(base_term, base_source);
        source_map
            .term_sources
            .insert(selector_term, selector_source);
        validate_property_equals_selector_term_lowering_terms(&seed_handoff, &terms, &association)?;
        validate_property_equals_selector_term_lowering_source_map(
            &terms,
            &source_map,
            &association,
        )?;

        let handoff = SourcePropertyEqualsSelectorTermLoweringHandoff {
            seed_handoff,
            terms,
            source_map,
            association,
        };
        handoff.validate()?;
        Ok(handoff)
    }
}

fn validate_property_equals_selector_term_lowering_owner(
    seed_handoff: &SourcePropertyEqualsSelectorTermSeedHandoff,
) -> Result<(), SourcePropertyEqualsSelectorTermLoweringError> {
    validate_property_equals_selector_definition_owner(
        seed_handoff.parameter_context(),
        seed_handoff.definition_owner(),
    )
    .map_err(|_| SourcePropertyEqualsSelectorTermLoweringError::InvalidTermLowering)
}

fn validate_property_equals_selector_term_lowering_association(
    association: &SourcePropertyEqualsSelectorTermLoweringAssociation,
) -> Result<(), SourcePropertyEqualsSelectorTermLoweringError> {
    if association.base_seed() != CoreTermSeedId::new(0)
        || association.base_term() != CoreTermId::new(0)
        || association.selector_seed() != CoreTermSeedId::new(1)
        || association.selector_term() != CoreTermId::new(1)
        || association.root_term() != association.selector_term()
    {
        return Err(SourcePropertyEqualsSelectorTermLoweringError::InvalidTermLowering);
    }
    Ok(())
}

fn validate_property_equals_selector_term_lowering_terms(
    seed_handoff: &SourcePropertyEqualsSelectorTermSeedHandoff,
    terms: &CoreTermTable,
    association: &SourcePropertyEqualsSelectorTermLoweringAssociation,
) -> Result<(), SourcePropertyEqualsSelectorTermLoweringError> {
    let [base_seed, selector_seed] = seed_handoff.terms() else {
        return Err(SourcePropertyEqualsSelectorTermLoweringError::InvalidTermLowering);
    };
    let base = terms
        .get(association.base_term())
        .ok_or(SourcePropertyEqualsSelectorTermLoweringError::InvalidTermLowering)?;
    let selector = terms
        .get(association.selector_term())
        .ok_or(SourcePropertyEqualsSelectorTermLoweringError::InvalidTermLowering)?;
    let expected_base_source = normalized_source(source_with_provenance(
        base_seed.source.clone(),
        &base_seed.provenance,
    ));
    let expected_selector_source = normalized_source(source_with_provenance(
        selector_seed.source.clone(),
        &selector_seed.provenance,
    ));

    if terms.len() != 2
        || base.kind != CoreTermKind::Var(CoreVarId::new(0))
        || base.source != expected_base_source
        || selector.kind
            != (CoreTermKind::Select {
                selector: seed_handoff
                    .selector_identity()
                    .association()
                    .selector_symbol()
                    .clone(),
                base: association.base_term(),
            })
        || selector.source != expected_selector_source
    {
        return Err(SourcePropertyEqualsSelectorTermLoweringError::InvalidTermLowering);
    }
    Ok(())
}

fn validate_property_equals_selector_term_lowering_source_map(
    terms: &CoreTermTable,
    source_map: &CoreSourceMap,
    association: &SourcePropertyEqualsSelectorTermLoweringAssociation,
) -> Result<(), SourcePropertyEqualsSelectorTermLoweringError> {
    let base = terms
        .get(association.base_term())
        .ok_or(SourcePropertyEqualsSelectorTermLoweringError::InvalidTermLowering)?;
    let selector = terms
        .get(association.selector_term())
        .ok_or(SourcePropertyEqualsSelectorTermLoweringError::InvalidTermLowering)?;

    if source_map.term_sources.len() != 2
        || source_map.term_sources.get(&association.base_term()) != Some(&base.source)
        || source_map.term_sources.get(&association.selector_term()) != Some(&selector.source)
        || !source_map.item_sources.is_empty()
        || !source_map.formula_sources.is_empty()
        || !source_map.definition_sources.is_empty()
        || !source_map.proof_sources.is_empty()
        || !source_map.algorithm_sources.is_empty()
        || !source_map.generated_sources.is_empty()
        || !source_map.obligation_sources.is_empty()
    {
        return Err(SourcePropertyEqualsSelectorTermLoweringError::InvalidTermLowering);
    }
    Ok(())
}

/// One immutable Core variable associated with one checker-authenticated
/// nested-Fraenkel capture row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceNestedFraenkelCaptureCoreVariable {
    capture: SourceNestedFraenkelCaptureGraphCaptureId,
    generator:
        mizar_checker::source_formula_composition::SourceNestedFraenkelCaptureGraphGeneratorId,
    resolver_binding: FraenkelGeneratorVariableBindingId,
    core_var: CoreVarId,
}

impl SourceNestedFraenkelCaptureCoreVariable {
    #[must_use]
    pub const fn capture(&self) -> SourceNestedFraenkelCaptureGraphCaptureId {
        self.capture
    }

    #[must_use]
    pub const fn generator(
        &self,
    ) -> mizar_checker::source_formula_composition::SourceNestedFraenkelCaptureGraphGeneratorId
    {
        self.generator
    }

    #[must_use]
    pub const fn resolver_binding(&self) -> FraenkelGeneratorVariableBindingId {
        self.resolver_binding
    }

    #[must_use]
    pub const fn core_var(&self) -> CoreVarId {
        self.core_var
    }
}

/// Immutable table of Core variables keyed by checker capture identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceNestedFraenkelCaptureCoreVariableTable {
    rows: BTreeMap<
        SourceNestedFraenkelCaptureGraphCaptureId,
        SourceNestedFraenkelCaptureCoreVariable,
    >,
}

impl SourceNestedFraenkelCaptureCoreVariableTable {
    fn empty() -> Self {
        Self {
            rows: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn get(
        &self,
        id: SourceNestedFraenkelCaptureGraphCaptureId,
    ) -> Option<&SourceNestedFraenkelCaptureCoreVariable> {
        self.rows.get(&id)
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<
        Item = (
            SourceNestedFraenkelCaptureGraphCaptureId,
            &SourceNestedFraenkelCaptureCoreVariable,
        ),
    > {
        self.rows.iter().map(|(id, row)| (*id, row))
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// Errors raised while associating a checker capture receipt with Core.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceNestedFraenkelCaptureCoreContextError {
    EnvironmentMismatch,
    InvalidCoreContext,
    InvalidOwnerAssociation,
    CoreVariableAllocationOverflow,
    CoreVariableCollision { var: CoreVarId },
    InvalidCaptureAssociation,
}

impl fmt::Display for SourceNestedFraenkelCaptureCoreContextError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EnvironmentMismatch => {
                formatter.write_str("nested Fraenkel capture Core context environment is invalid")
            }
            Self::InvalidCoreContext => {
                formatter.write_str("nested Fraenkel capture Core context is invalid")
            }
            Self::InvalidOwnerAssociation => {
                formatter.write_str("nested Fraenkel capture Core owner association is invalid")
            }
            Self::CoreVariableAllocationOverflow => {
                formatter.write_str("nested Fraenkel capture Core variable allocation overflowed")
            }
            Self::CoreVariableCollision { var } => write!(
                formatter,
                "nested Fraenkel capture Core variable {} collides",
                var.index()
            ),
            Self::InvalidCaptureAssociation => {
                formatter.write_str("nested Fraenkel capture Core association is invalid")
            }
        }
    }
}

impl Error for SourceNestedFraenkelCaptureCoreContextError {}

/// Immutable Core handoff for the checker-authenticated nested capture.
#[derive(Clone, PartialEq, Eq)]
pub struct SourceNestedFraenkelCaptureCoreContextHandoff {
    context: CoreContext,
    checker_receipt: SourceNestedFraenkelCaptureGraphOwnerHandoff,
    owner_item: CoreItemId,
    captured_variables: SourceNestedFraenkelCaptureCoreVariableTable,
}

impl SourceNestedFraenkelCaptureCoreContextHandoff {
    #[must_use]
    pub const fn source_id(&self) -> SourceId {
        self.context.source_id()
    }

    #[must_use]
    pub const fn module_id(&self) -> &ModuleId {
        self.context.module_id()
    }

    #[must_use]
    pub const fn context(&self) -> &CoreContext {
        &self.context
    }

    #[must_use]
    pub const fn checker_receipt(&self) -> &SourceNestedFraenkelCaptureGraphOwnerHandoff {
        &self.checker_receipt
    }

    #[must_use]
    pub const fn owner_item(&self) -> CoreItemId {
        self.owner_item
    }

    #[must_use]
    pub const fn captured_variables(&self) -> &SourceNestedFraenkelCaptureCoreVariableTable {
        &self.captured_variables
    }

    #[must_use]
    pub fn debug_text(&self) -> String {
        let vars = self
            .captured_variables
            .iter()
            .map(|(_, row)| row.core_var().index().to_string())
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "source-nested-fraenkel-capture-core-context-v1|module={}.{}|captures={}|vars={}|owner={}",
            self.module_id().package().as_str(),
            self.module_id().path().as_str(),
            self.captured_variables.len(),
            vars,
            self.checker_receipt.owner().symbol().fqn().as_str(),
        )
    }

    fn validate(&self) -> Result<(), SourceNestedFraenkelCaptureCoreContextError> {
        if self.source_id() != self.checker_receipt.source_id()
            || self.module_id() != self.checker_receipt.module_id()
        {
            return Err(SourceNestedFraenkelCaptureCoreContextError::EnvironmentMismatch);
        }
        let allowed = capture_vars(&self.captured_variables);
        let used = validate_core_context_shape(&self.context, &allowed)?;
        let owner_item = validate_owner_association(&self.context, &self.checker_receipt)?;
        if owner_item != self.owner_item {
            return Err(SourceNestedFraenkelCaptureCoreContextError::InvalidOwnerAssociation);
        }
        validate_capture_association(
            &self.context,
            &self.checker_receipt,
            &self.captured_variables,
            &used,
        )
    }
}

/// Builds the standalone immutable Core capture-context handoff.
#[derive(Debug, Clone, Copy)]
pub struct SourceNestedFraenkelCaptureCoreContextProducer;

impl SourceNestedFraenkelCaptureCoreContextProducer {
    pub fn build(
        mut context: CoreContext,
        checker_receipt: SourceNestedFraenkelCaptureGraphOwnerHandoff,
    ) -> Result<
        SourceNestedFraenkelCaptureCoreContextHandoff,
        SourceNestedFraenkelCaptureCoreContextError,
    > {
        if context.source_id() != checker_receipt.source_id()
            || context.module_id() != checker_receipt.module_id()
        {
            return Err(SourceNestedFraenkelCaptureCoreContextError::EnvironmentMismatch);
        }
        let used = validate_core_context_shape(&context, &BTreeSet::new())?;
        let owner_item = validate_owner_association(&context, &checker_receipt)?;
        let allocated =
            allocate_capture_core_vars(&used, checker_receipt.graph().captures().len())?;
        let associations = capture_associations(&checker_receipt)?;
        if allocated.len() != associations.len() {
            return Err(SourceNestedFraenkelCaptureCoreContextError::InvalidCaptureAssociation);
        }

        let mut captured_variables = SourceNestedFraenkelCaptureCoreVariableTable::empty();
        for (association, core_var) in associations.into_iter().zip(allocated) {
            let key = capture_provenance_key(association.capture);
            let provenance = CoreProvenance::new(CoreProvenancePhase::Checker, key.clone());
            let source =
                CoreSourceRef::direct(association.binder_range).with_provenance(vec![provenance]);
            context.binder_context.declare_variable(
                core_var,
                NormalizedVarClass::Free,
                NESTED_FRAENKEL_CAPTURE_CORE_ROLE,
                NormalizedVarSort::Term,
            );
            context.binder_type_facts.insert(core_var, Vec::new());
            context
                .binder_sources
                .insert(BinderSourceRecord {
                    var: core_var,
                    source,
                    provenance: CheckerOwnedProvenance::checker(key),
                })
                .map_err(|_| {
                    SourceNestedFraenkelCaptureCoreContextError::CoreVariableCollision {
                        var: core_var,
                    }
                })?;
            if captured_variables
                .rows
                .insert(
                    association.capture,
                    SourceNestedFraenkelCaptureCoreVariable {
                        capture: association.capture,
                        generator: association.generator,
                        resolver_binding: association.resolver_binding,
                        core_var,
                    },
                )
                .is_some()
            {
                return Err(SourceNestedFraenkelCaptureCoreContextError::InvalidCaptureAssociation);
            }
        }

        let handoff = SourceNestedFraenkelCaptureCoreContextHandoff {
            context,
            checker_receipt,
            owner_item,
            captured_variables,
        };
        handoff.validate()?;
        Ok(handoff)
    }
}

#[derive(Clone, Copy)]
struct CaptureAssociation {
    capture: SourceNestedFraenkelCaptureGraphCaptureId,
    generator:
        mizar_checker::source_formula_composition::SourceNestedFraenkelCaptureGraphGeneratorId,
    resolver_binding: FraenkelGeneratorVariableBindingId,
    binder_range: SourceRange,
}

fn capture_provenance_key(capture: SourceNestedFraenkelCaptureGraphCaptureId) -> CoreProvenanceKey {
    CoreProvenanceKey::new(format!(
        "{NESTED_FRAENKEL_CAPTURE_CORE_PROVENANCE_PREFIX}.{}",
        capture.index()
    ))
}

fn capture_vars(table: &SourceNestedFraenkelCaptureCoreVariableTable) -> BTreeSet<CoreVarId> {
    table.iter().map(|(_, row)| row.core_var()).collect()
}

fn validate_core_context_shape(
    context: &CoreContext,
    allowed_capture_vars: &BTreeSet<CoreVarId>,
) -> Result<BTreeSet<CoreVarId>, SourceNestedFraenkelCaptureCoreContextError> {
    let binder_context = context.binder_context();
    let declared = &binder_context.free_variables;
    let class_keys = binder_context
        .variable_classes
        .keys()
        .copied()
        .collect::<BTreeSet<_>>();
    let role_keys = binder_context
        .variable_roles
        .keys()
        .copied()
        .collect::<BTreeSet<_>>();
    let sort_keys = binder_context
        .variable_sorts
        .keys()
        .copied()
        .collect::<BTreeSet<_>>();
    let type_fact_keys = context
        .binder_type_facts
        .keys()
        .copied()
        .collect::<BTreeSet<_>>();
    if class_keys != *declared
        || role_keys != *declared
        || sort_keys != *declared
        || type_fact_keys != *declared
    {
        return Err(SourceNestedFraenkelCaptureCoreContextError::InvalidCoreContext);
    }
    if binder_context.variable_roles.iter().any(|(var, role)| {
        role.as_str() == NESTED_FRAENKEL_CAPTURE_CORE_ROLE && !allowed_capture_vars.contains(var)
    }) {
        return Err(SourceNestedFraenkelCaptureCoreContextError::InvalidCoreContext);
    }

    let mut used = declared.clone();
    for (var, record) in context.binder_sources.iter() {
        if var != record.var || !declared.contains(&var) {
            return Err(SourceNestedFraenkelCaptureCoreContextError::InvalidCoreContext);
        }
        if validate_checker_owned_provenance("binder source", record.provenance.as_slice()).is_err()
        {
            return Err(SourceNestedFraenkelCaptureCoreContextError::InvalidCoreContext);
        }
        used.insert(var);
    }
    for frame in &binder_context.frames {
        if !declared.contains(&frame.original_var) {
            return Err(SourceNestedFraenkelCaptureCoreContextError::InvalidCoreContext);
        }
        used.insert(frame.original_var);
    }
    for (_, origin) in context.generated_origins.table().iter() {
        for var in &origin.params {
            if !declared.contains(var) {
                return Err(SourceNestedFraenkelCaptureCoreContextError::InvalidCoreContext);
            }
            used.insert(*var);
        }
    }
    Ok(used)
}

fn validate_owner_association(
    context: &CoreContext,
    checker_receipt: &SourceNestedFraenkelCaptureGraphOwnerHandoff,
) -> Result<CoreItemId, SourceNestedFraenkelCaptureCoreContextError> {
    let origin = checker_receipt.owner().origin();
    let owner_range = validate_owner_origin(context, origin)?;
    validate_owner_item(context, checker_receipt.owner().symbol(), owner_range)
}

fn validate_owner_origin(
    context: &CoreContext,
    origin: &SemanticOrigin,
) -> Result<SourceRange, SourceNestedFraenkelCaptureCoreContextError> {
    let Some(owner_range) = (match origin.anchor() {
        SourceAnchor::Range(range) => Some(*range),
        _ => None,
    }) else {
        return Err(SourceNestedFraenkelCaptureCoreContextError::InvalidOwnerAssociation);
    };
    if origin.source_id() != context.source_id()
        || origin.module_id() != context.module_id()
        || origin.is_recovered()
        || origin.import_edge().is_some()
    {
        return Err(SourceNestedFraenkelCaptureCoreContextError::InvalidOwnerAssociation);
    }
    Ok(owner_range)
}

fn validate_owner_item(
    context: &CoreContext,
    symbol: &SymbolId,
    owner_range: SourceRange,
) -> Result<CoreItemId, SourceNestedFraenkelCaptureCoreContextError> {
    let Some(item_id) = context.item_registry.id_for_symbol(symbol) else {
        return Err(SourceNestedFraenkelCaptureCoreContextError::InvalidOwnerAssociation);
    };
    let Some(item) = context.item_registry.items().get(item_id) else {
        return Err(SourceNestedFraenkelCaptureCoreContextError::InvalidOwnerAssociation);
    };
    if item.symbol != *symbol
        || item.kind != CoreItemKind::Functor
        || item.status != CoreItemStatus::Valid
        || context.source_map.item_sources.get(&item_id) != Some(&item.source)
    {
        return Err(SourceNestedFraenkelCaptureCoreContextError::InvalidOwnerAssociation);
    }
    if !matches!(
        &item.source.anchor,
        CoreSourceAnchor::SourceRange(range) if *range == owner_range
    ) {
        return Err(SourceNestedFraenkelCaptureCoreContextError::InvalidOwnerAssociation);
    }
    Ok(item_id)
}

fn allocate_capture_core_vars(
    used: &BTreeSet<CoreVarId>,
    count: usize,
) -> Result<Vec<CoreVarId>, SourceNestedFraenkelCaptureCoreContextError> {
    allocate_core_vars(
        used,
        count,
        SourceNestedFraenkelCaptureCoreContextError::CoreVariableAllocationOverflow,
        |var| SourceNestedFraenkelCaptureCoreContextError::CoreVariableCollision { var },
    )
}

fn allocate_source_binding_core_vars(
    used: &BTreeSet<CoreVarId>,
    count: usize,
) -> Result<Vec<CoreVarId>, SourceBindingCoreContextError> {
    allocate_core_vars(
        used,
        count,
        SourceBindingCoreContextError::CoreVariableAllocationOverflow,
        |var| SourceBindingCoreContextError::CoreVariableCollision { var },
    )
}

fn allocate_core_vars<E>(
    used: &BTreeSet<CoreVarId>,
    count: usize,
    overflow: E,
    collision: impl Fn(CoreVarId) -> E,
) -> Result<Vec<CoreVarId>, E>
where
    E: Clone,
{
    let next = match used.iter().next_back() {
        Some(var) => var.index().checked_add(1).ok_or_else(|| overflow.clone())?,
        None => 0,
    };
    let mut allocated = Vec::with_capacity(count);
    let mut reserved = used.clone();
    for offset in 0..count {
        let index = next.checked_add(offset).ok_or_else(|| overflow.clone())?;
        let var = CoreVarId::new(index);
        if !reserved.insert(var) {
            return Err(collision(var));
        }
        allocated.push(var);
    }
    Ok(allocated)
}

fn capture_associations(
    checker_receipt: &SourceNestedFraenkelCaptureGraphOwnerHandoff,
) -> Result<Vec<CaptureAssociation>, SourceNestedFraenkelCaptureCoreContextError> {
    let graph = checker_receipt.graph();
    if graph.generators().len() != 3 || graph.captures().len() != 2 {
        return Err(SourceNestedFraenkelCaptureCoreContextError::InvalidCaptureAssociation);
    }
    let Some((local_generator, _)) = graph.generators().iter().next() else {
        return Err(SourceNestedFraenkelCaptureCoreContextError::InvalidCaptureAssociation);
    };
    let mut associations = Vec::with_capacity(2);
    let mut seen_generators = BTreeSet::new();
    let mut seen_bindings = BTreeSet::new();
    for (capture_id, capture) in graph.captures().iter() {
        if capture.generator() == local_generator
            || !seen_generators.insert(capture.generator())
            || !seen_bindings.insert(capture.resolver_binding())
        {
            return Err(SourceNestedFraenkelCaptureCoreContextError::InvalidCaptureAssociation);
        }
        let Some(generator) = graph.generators().get(capture.generator()) else {
            return Err(SourceNestedFraenkelCaptureCoreContextError::InvalidCaptureAssociation);
        };
        if generator.resolver_binding() != capture.resolver_binding() {
            return Err(SourceNestedFraenkelCaptureCoreContextError::InvalidCaptureAssociation);
        }
        associations.push(CaptureAssociation {
            capture: capture_id,
            generator: capture.generator(),
            resolver_binding: capture.resolver_binding(),
            binder_range: generator.binder_range(),
        });
    }
    Ok(associations)
}

fn validate_capture_association(
    context: &CoreContext,
    checker_receipt: &SourceNestedFraenkelCaptureGraphOwnerHandoff,
    captured_variables: &SourceNestedFraenkelCaptureCoreVariableTable,
    used: &BTreeSet<CoreVarId>,
) -> Result<(), SourceNestedFraenkelCaptureCoreContextError> {
    let associations = capture_associations(checker_receipt)?;
    validate_capture_rows(context, captured_variables, used, &associations)
}

fn validate_capture_rows(
    context: &CoreContext,
    captured_variables: &SourceNestedFraenkelCaptureCoreVariableTable,
    used: &BTreeSet<CoreVarId>,
    associations: &[CaptureAssociation],
) -> Result<(), SourceNestedFraenkelCaptureCoreContextError> {
    if captured_variables.len() != associations.len() {
        return Err(SourceNestedFraenkelCaptureCoreContextError::InvalidCaptureAssociation);
    }
    let capture_vars = capture_vars(captured_variables);
    if capture_vars.len() != captured_variables.len() {
        return Err(
            SourceNestedFraenkelCaptureCoreContextError::CoreVariableCollision {
                var: captured_variables
                    .iter()
                    .find_map(|(_, row)| {
                        let var = row.core_var();
                        (captured_variables
                            .iter()
                            .filter(|(_, other)| other.core_var() == var)
                            .count()
                            > 1)
                        .then_some(var)
                    })
                    .unwrap_or_else(|| CoreVarId::new(0)),
            },
        );
    }
    let non_capture_used = used
        .iter()
        .copied()
        .filter(|var| !capture_vars.contains(var))
        .collect::<BTreeSet<_>>();
    let allocated = allocate_capture_core_vars(&non_capture_used, captured_variables.len())?;
    for (association, expected_var) in associations.iter().zip(allocated) {
        let Some(row) = captured_variables.get(association.capture) else {
            return Err(SourceNestedFraenkelCaptureCoreContextError::InvalidCaptureAssociation);
        };
        if row.capture() != association.capture
            || row.generator() != association.generator
            || row.resolver_binding() != association.resolver_binding
            || row.core_var() != expected_var
        {
            return Err(SourceNestedFraenkelCaptureCoreContextError::InvalidCaptureAssociation);
        }
        let Some(record) = context.binder_sources.get(row.core_var()) else {
            return Err(SourceNestedFraenkelCaptureCoreContextError::InvalidCaptureAssociation);
        };
        let key = capture_provenance_key(association.capture);
        if record.source.anchor != CoreSourceAnchor::SourceRange(association.binder_range)
            || record.source.provenance.as_slice()
                != [CoreProvenance::new(
                    CoreProvenancePhase::Checker,
                    key.clone(),
                )]
            || record.provenance.as_slice()
                != [CoreProvenance::new(CoreProvenancePhase::Checker, key)]
        {
            return Err(SourceNestedFraenkelCaptureCoreContextError::InvalidCaptureAssociation);
        }
        if context.binder_context.variable_classes.get(&row.core_var())
            != Some(&NormalizedVarClass::Free)
            || context.binder_context.variable_sorts.get(&row.core_var())
                != Some(&NormalizedVarSort::Term)
            || context
                .binder_context
                .variable_roles
                .get(&row.core_var())
                .map(CoreVarRole::as_str)
                != Some(NESTED_FRAENKEL_CAPTURE_CORE_ROLE)
            || context.binder_type_facts.get(&row.core_var()) != Some(&Vec::new())
        {
            return Err(SourceNestedFraenkelCaptureCoreContextError::InvalidCaptureAssociation);
        }
    }
    Ok(())
}

fn diagnostic(
    class: CoreDiagnosticClass,
    severity: CoreDiagnosticSeverity,
    recovery: CoreDiagnosticRecovery,
    message_key: impl Into<CoreDiagnosticMessageKey>,
    source: CoreSourceRef,
    owner: Option<CoreNodeRef>,
) -> CoreDiagnostic {
    CoreDiagnostic {
        class,
        severity,
        recovery,
        message_key: message_key.into(),
        primary_source: normalized_source(source),
        related: Vec::new(),
        owner,
    }
}

fn normalized_source(source: CoreSourceRef) -> CoreSourceRef {
    let provenance = source.provenance.clone();
    source.with_provenance(provenance)
}

fn item_seed_cmp(left: &CoreItemSeed, right: &CoreItemSeed) -> std::cmp::Ordering {
    source_order_key(&left.source)
        .cmp(&source_order_key(&right.source))
        .then_with(|| left.symbol.cmp(&right.symbol))
}

fn checker_site_cmp(left: &CheckerSiteSummary, right: &CheckerSiteSummary) -> std::cmp::Ordering {
    source_order_key(&left.source)
        .cmp(&source_order_key(&right.source))
        .then_with(|| format!("{:?}", left.kind).cmp(&format!("{:?}", right.kind)))
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SourceOrderKey {
    kind: u8,
    source_id: String,
    start: usize,
    end: usize,
    owner: String,
    origin_kind: String,
    generated_key: String,
    reason: String,
}

fn source_order_key(source: &CoreSourceRef) -> SourceOrderKey {
    match &source.anchor {
        CoreSourceAnchor::SourceRange(SourceRange {
            source_id,
            start,
            end,
        }) => SourceOrderKey {
            kind: 0,
            source_id: format!("{source_id:?}"),
            start: *start,
            end: *end,
            owner: String::new(),
            origin_kind: String::new(),
            generated_key: String::new(),
            reason: String::new(),
        },
        CoreSourceAnchor::GeneratedFrom(generated_from) => SourceOrderKey {
            kind: 1,
            source_id: String::new(),
            start: 0,
            end: 0,
            owner: format!("{:?}", generated_from.owner),
            origin_kind: format!("{:?}", generated_from.kind),
            generated_key: generated_from.key.as_str().to_owned(),
            reason: generated_from.reason.as_str().to_owned(),
        },
    }
}

pub type TypeAndFactResult<T> = Result<T, TypeAndFactLoweringError>;

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TypeAndFactLoweringError {
    MissingOwnerItem {
        owner: CoreItemId,
    },
    UndeclaredSubject {
        var: CoreVarId,
    },
    NonTermSubject {
        var: CoreVarId,
        sort: NormalizedVarSort,
    },
    ClusterFactMissingCheckerFact {
        cluster_fact: ClusterFactId,
    },
    MissingActiveObligationGoal {
        obligation: Option<InitialObligationId>,
    },
    InactiveObligationWithoutReason {
        obligation: Option<InitialObligationId>,
    },
    EmptyReductViewPayload {
        path: QuaPathKey,
    },
    DuplicateTemplateTypeParameter {
        parameter: TemplateParameterKey,
    },
    DuplicateTemplateTypeActualGate {
        instantiation: TemplateInstantiationKey,
        parameter: TemplateParameterKey,
    },
    DuplicateTemplateTypeParameterSethood {
        parameter: TemplateParameterKey,
        evidence_key: TemplateSethoodEvidenceKey,
    },
    DuplicateTemplateSchemeActual {
        instantiation: TemplateInstantiationKey,
        parameter: TemplateParameterKey,
    },
    PartialTemplateTypeActualBaseEvidence {
        instantiation: TemplateInstantiationKey,
        parameter: TemplateParameterKey,
    },
    SatisfiedTemplateTypeActualWithoutEvidence {
        instantiation: TemplateInstantiationKey,
        parameter: TemplateParameterKey,
    },
    UnsatisfiedTemplateTypeActualCarriesEvidence {
        instantiation: TemplateInstantiationKey,
        parameter: TemplateParameterKey,
        status: ExistentialGateStatus,
    },
    AcceptedTemplateTypeParameterSethoodWithoutEvidence {
        parameter: TemplateParameterKey,
        evidence_key: TemplateSethoodEvidenceKey,
    },
    BareTemplateTypeParameterSethoodAccepted {
        parameter: TemplateParameterKey,
        evidence_key: TemplateSethoodEvidenceKey,
    },
    MissingTemplateTypeParameterSethoodCarriesEvidence {
        parameter: TemplateParameterKey,
        evidence_key: TemplateSethoodEvidenceKey,
        status: TemplateTypeParameterSethoodStatus,
    },
    MissingTemplateTypeParameterSethoodWrongSource {
        parameter: TemplateParameterKey,
        evidence_key: TemplateSethoodEvidenceKey,
        source_kind: TemplateTypeParameterSethoodSource,
    },
    TemplateSchemeActualKindMismatch {
        instantiation: TemplateInstantiationKey,
        parameter: TemplateParameterKey,
        parameter_kind: TemplateSchemeParameterKind,
        actual_kind: TemplateSchemeActualKind,
    },
    TemplateSchemeActualArityMismatch {
        instantiation: TemplateInstantiationKey,
        parameter: TemplateParameterKey,
        expected: usize,
        actual: usize,
    },
    AcceptedTemplateSchemeActualMissingEvidence {
        instantiation: TemplateInstantiationKey,
        parameter: TemplateParameterKey,
    },
    TemplateSchemeActualPartialDomainEvidence {
        instantiation: TemplateInstantiationKey,
        parameter: TemplateParameterKey,
    },
    TemplateSchemeActualInvalidCodomainEvidence {
        instantiation: TemplateInstantiationKey,
        parameter: TemplateParameterKey,
    },
    TemplateSchemeFunctorMissingGuardSeed {
        instantiation: TemplateInstantiationKey,
        parameter: TemplateParameterKey,
    },
    TemplateSchemeFunctorInvalidGuardSeedStatus {
        instantiation: TemplateInstantiationKey,
        parameter: TemplateParameterKey,
        status: ObligationSeedStatus,
    },
    TemplateSchemeFunctorInvalidGuardSeedKind {
        instantiation: TemplateInstantiationKey,
        parameter: TemplateParameterKey,
        kind: InitialObligationKind,
    },
    RejectedTemplateSchemeActualCarriesEvidence {
        instantiation: TemplateInstantiationKey,
        parameter: TemplateParameterKey,
        status: TemplateSchemeActualStatus,
    },
    TemplateSchemeTypeActualCarriesCallableEvidence {
        instantiation: TemplateInstantiationKey,
        parameter: TemplateParameterKey,
    },
    TemplateSchemeActualMissingSubstitutionEvidence {
        instantiation: TemplateInstantiationKey,
        parameter: TemplateParameterKey,
    },
    UnsupportedPolarity,
    InvalidSeedProvenance(CoreContextError),
}

impl fmt::Display for TypeAndFactLoweringError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingOwnerItem { owner } => {
                write!(formatter, "missing core item owner {}", owner.index())
            }
            Self::UndeclaredSubject { var } => {
                write!(
                    formatter,
                    "undeclared type/fact subject variable {}",
                    var.index()
                )
            }
            Self::NonTermSubject { var, sort } => {
                write!(
                    formatter,
                    "type/fact subject variable {} has non-term sort {sort:?}",
                    var.index()
                )
            }
            Self::ClusterFactMissingCheckerFact { cluster_fact } => {
                write!(
                    formatter,
                    "cluster fact {} is missing its accepted checker type fact",
                    cluster_fact.index()
                )
            }
            Self::MissingActiveObligationGoal { obligation } => {
                write!(
                    formatter,
                    "active carried obligation {obligation:?} is missing an explicit core goal"
                )
            }
            Self::InactiveObligationWithoutReason { obligation } => {
                write!(
                    formatter,
                    "inactive carried obligation {obligation:?} needs a diagnostic or provenance reason"
                )
            }
            Self::EmptyReductViewPayload { path } => {
                write!(
                    formatter,
                    "reduct view path {} needs at least one explicit view functor",
                    path.as_str()
                )
            }
            Self::DuplicateTemplateTypeParameter { parameter } => {
                write!(
                    formatter,
                    "duplicate template type parameter inhabitation seed {}",
                    parameter.as_str()
                )
            }
            Self::DuplicateTemplateTypeActualGate {
                instantiation,
                parameter,
            } => {
                write!(
                    formatter,
                    "duplicate template type actual gate for instantiation {} parameter {}",
                    instantiation.as_str(),
                    parameter.as_str()
                )
            }
            Self::DuplicateTemplateTypeParameterSethood {
                parameter,
                evidence_key,
            } => {
                write!(
                    formatter,
                    "duplicate template type-parameter sethood record for parameter {} evidence {}",
                    parameter.as_str(),
                    evidence_key.as_str()
                )
            }
            Self::DuplicateTemplateSchemeActual {
                instantiation,
                parameter,
            } => {
                write!(
                    formatter,
                    "duplicate template scheme actual for instantiation {} parameter {}",
                    instantiation.as_str(),
                    parameter.as_str()
                )
            }
            Self::PartialTemplateTypeActualBaseEvidence {
                instantiation,
                parameter,
            } => {
                write!(
                    formatter,
                    "template type actual gate for instantiation {} parameter {} has partial base evidence",
                    instantiation.as_str(),
                    parameter.as_str()
                )
            }
            Self::SatisfiedTemplateTypeActualWithoutEvidence {
                instantiation,
                parameter,
            } => {
                write!(
                    formatter,
                    "satisfied template type actual gate for instantiation {} parameter {} has no registration, base evidence, or guard facts",
                    instantiation.as_str(),
                    parameter.as_str()
                )
            }
            Self::UnsatisfiedTemplateTypeActualCarriesEvidence {
                instantiation,
                parameter,
                status,
            } => {
                write!(
                    formatter,
                    "unsatisfied template type actual gate for instantiation {} parameter {} carries accepted evidence with status {status:?}",
                    instantiation.as_str(),
                    parameter.as_str()
                )
            }
            Self::AcceptedTemplateTypeParameterSethoodWithoutEvidence {
                parameter,
                evidence_key,
            } => {
                write!(
                    formatter,
                    "accepted template type-parameter sethood record for parameter {} evidence {} has no checker facts",
                    parameter.as_str(),
                    evidence_key.as_str()
                )
            }
            Self::BareTemplateTypeParameterSethoodAccepted {
                parameter,
                evidence_key,
            } => {
                write!(
                    formatter,
                    "bare template type parameter {} cannot accept sethood evidence {}",
                    parameter.as_str(),
                    evidence_key.as_str()
                )
            }
            Self::MissingTemplateTypeParameterSethoodCarriesEvidence {
                parameter,
                evidence_key,
                status,
            } => {
                write!(
                    formatter,
                    "non-accepted template type-parameter sethood record for parameter {} evidence {} carries checker facts with status {status:?}",
                    parameter.as_str(),
                    evidence_key.as_str()
                )
            }
            Self::MissingTemplateTypeParameterSethoodWrongSource {
                parameter,
                evidence_key,
                source_kind,
            } => {
                write!(
                    formatter,
                    "missing template type-parameter sethood record for parameter {} evidence {} has wrong source {source_kind:?}",
                    parameter.as_str(),
                    evidence_key.as_str()
                )
            }
            Self::TemplateSchemeActualKindMismatch {
                instantiation,
                parameter,
                parameter_kind,
                actual_kind,
            } => {
                write!(
                    formatter,
                    "template scheme actual for instantiation {} parameter {} has incompatible parameter/actual kinds {parameter_kind:?}/{actual_kind:?}",
                    instantiation.as_str(),
                    parameter.as_str()
                )
            }
            Self::TemplateSchemeActualArityMismatch {
                instantiation,
                parameter,
                expected,
                actual,
            } => {
                write!(
                    formatter,
                    "template scheme actual for instantiation {} parameter {} has arity mismatch: expected {expected}, got {actual}",
                    instantiation.as_str(),
                    parameter.as_str()
                )
            }
            Self::AcceptedTemplateSchemeActualMissingEvidence {
                instantiation,
                parameter,
            } => {
                write!(
                    formatter,
                    "accepted template scheme actual for instantiation {} parameter {} is missing complete compatibility evidence",
                    instantiation.as_str(),
                    parameter.as_str()
                )
            }
            Self::TemplateSchemeActualPartialDomainEvidence {
                instantiation,
                parameter,
            } => {
                write!(
                    formatter,
                    "template scheme actual for instantiation {} parameter {} has partial domain-widening evidence",
                    instantiation.as_str(),
                    parameter.as_str()
                )
            }
            Self::TemplateSchemeActualInvalidCodomainEvidence {
                instantiation,
                parameter,
            } => {
                write!(
                    formatter,
                    "template scheme actual for instantiation {} parameter {} has invalid codomain-widening evidence",
                    instantiation.as_str(),
                    parameter.as_str()
                )
            }
            Self::TemplateSchemeFunctorMissingGuardSeed {
                instantiation,
                parameter,
            } => {
                write!(
                    formatter,
                    "accepted functor actual for instantiation {} parameter {} is missing its skipped guard obligation seed",
                    instantiation.as_str(),
                    parameter.as_str()
                )
            }
            Self::TemplateSchemeFunctorInvalidGuardSeedStatus {
                instantiation,
                parameter,
                status,
            } => {
                write!(
                    formatter,
                    "functor actual for instantiation {} parameter {} has invalid guard seed status {status:?}",
                    instantiation.as_str(),
                    parameter.as_str()
                )
            }
            Self::TemplateSchemeFunctorInvalidGuardSeedKind {
                instantiation,
                parameter,
                kind,
            } => {
                write!(
                    formatter,
                    "functor actual for instantiation {} parameter {} has invalid guard seed kind {kind:?}",
                    instantiation.as_str(),
                    parameter.as_str()
                )
            }
            Self::RejectedTemplateSchemeActualCarriesEvidence {
                instantiation,
                parameter,
                status,
            } => {
                write!(
                    formatter,
                    "rejected template scheme actual for instantiation {} parameter {} carries accepted evidence with status {status:?}",
                    instantiation.as_str(),
                    parameter.as_str()
                )
            }
            Self::TemplateSchemeTypeActualCarriesCallableEvidence {
                instantiation,
                parameter,
            } => {
                write!(
                    formatter,
                    "type scheme actual for instantiation {} parameter {} carries callable compatibility evidence",
                    instantiation.as_str(),
                    parameter.as_str()
                )
            }
            Self::TemplateSchemeActualMissingSubstitutionEvidence {
                instantiation,
                parameter,
            } => {
                write!(
                    formatter,
                    "enclosing template parameter actual for instantiation {} parameter {} is missing substitution-composition evidence",
                    instantiation.as_str(),
                    parameter.as_str()
                )
            }
            Self::UnsupportedPolarity => write!(formatter, "unsupported checker polarity"),
            Self::InvalidSeedProvenance(error) => write!(formatter, "{error}"),
        }
    }
}

impl Error for TypeAndFactLoweringError {}

impl From<CoreContextError> for TypeAndFactLoweringError {
    fn from(value: CoreContextError) -> Self {
        Self::InvalidSeedProvenance(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeAndFactLoweringInput {
    pub owner: CoreItemId,
    pub declared_binders: Vec<DeclaredBinderTypeSeed>,
    pub formula_assertions: Vec<TypePredicateSeed>,
    pub attribute_chains: Vec<AttributeChainSeed>,
    pub mode_expansions: Vec<ModeExpansionSeed>,
    pub cluster_facts: Vec<ClusterFactSeed>,
    pub view_explanations: Vec<ViewExplanationSeed>,
    pub template_type_parameters: Vec<TemplateTypeParameterInhabitationSeed>,
    pub template_type_actual_gates: Vec<TemplateTypeActualGateSeed>,
    pub template_type_parameter_sethoods: Vec<TemplateTypeParameterSethoodSeed>,
    pub template_scheme_actuals: Vec<TemplateSchemeActualSeed>,
    pub reconsiderings: Vec<ReconsideringSeed>,
    pub carried_obligations: Vec<CarriedInitialObligationSeed>,
    pub missing_evidence: Vec<MissingEvidenceSeed>,
}

impl TypeAndFactLoweringInput {
    pub const fn new(owner: CoreItemId) -> Self {
        Self {
            owner,
            declared_binders: Vec::new(),
            formula_assertions: Vec::new(),
            attribute_chains: Vec::new(),
            mode_expansions: Vec::new(),
            cluster_facts: Vec::new(),
            view_explanations: Vec::new(),
            template_type_parameters: Vec::new(),
            template_type_actual_gates: Vec::new(),
            template_type_parameter_sethoods: Vec::new(),
            template_scheme_actuals: Vec::new(),
            reconsiderings: Vec::new(),
            carried_obligations: Vec::new(),
            missing_evidence: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypePredicateSeed {
    pub subject: CoreVarId,
    pub predicate: CoreTypePredicate,
    pub polarity: Polarity,
    pub checker_fact: Option<TypeFactId>,
    pub source: CoreSourceRef,
    pub provenance: CheckerOwnedProvenance,
}

impl TypePredicateSeed {
    pub fn positive(
        subject: CoreVarId,
        predicate: impl Into<CoreTypePredicate>,
        source: CoreSourceRef,
        provenance: CheckerOwnedProvenance,
    ) -> Self {
        Self {
            subject,
            predicate: predicate.into(),
            polarity: Polarity::Positive,
            checker_fact: None,
            source,
            provenance,
        }
    }

    pub fn with_checker_fact(mut self, fact: TypeFactId) -> Self {
        self.checker_fact = Some(fact);
        self
    }

    pub fn with_polarity(mut self, polarity: Polarity) -> Self {
        self.polarity = polarity;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclaredBinderTypeSeed {
    pub var: CoreVarId,
    pub role: CoreVarRole,
    pub predicate: CoreTypePredicate,
    pub source_name: Option<String>,
    pub source: CoreSourceRef,
    pub provenance: CheckerOwnedProvenance,
}

impl DeclaredBinderTypeSeed {
    pub fn new(
        var: CoreVarId,
        role: impl Into<CoreVarRole>,
        predicate: impl Into<CoreTypePredicate>,
        source: CoreSourceRef,
        provenance: CheckerOwnedProvenance,
    ) -> Self {
        Self {
            var,
            role: role.into(),
            predicate: predicate.into(),
            source_name: None,
            source,
            provenance,
        }
    }

    pub fn with_source_name(mut self, source_name: impl Into<String>) -> Self {
        self.source_name = Some(source_name.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeChainSeed {
    pub facts: Vec<TypePredicateSeed>,
    pub source: CoreSourceRef,
    pub provenance: CheckerOwnedProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModeExpansionSeed {
    pub subject: CoreVarId,
    pub normalized_type: NormalizedTypeId,
    pub predicate: CoreTypePredicate,
    pub source: CoreSourceRef,
    pub provenance: CheckerOwnedProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterFactSeed {
    pub cluster_fact: ClusterFactId,
    pub fact: TypePredicateSeed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateTypeParameterInhabitationSeed {
    pub parameter: TemplateParameterKey,
    pub witness: CoreVarId,
    pub witness_role: CoreVarRole,
    pub witness_source_name: Option<String>,
    pub predicate: CoreTypePredicate,
    pub source: CoreSourceRef,
    pub provenance: CheckerOwnedProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateTypeActualGateSeed {
    pub instantiation: TemplateInstantiationKey,
    pub parameter: TemplateParameterKey,
    pub actual_type: NormalizedTypeId,
    pub gate: Option<ExistentialGateId>,
    pub status: ExistentialGateStatus,
    pub registration: Option<CheckerRegistrationId>,
    pub base_evidence_kind: Option<ExistentialGateBaseEvidenceKind>,
    pub base_evidence_coverage: Option<ExistentialGateBaseEvidenceCoverage>,
    pub facts: Vec<TypeFactId>,
    pub diagnostics: Vec<RegistrationDiagnosticId>,
    pub source: CoreSourceRef,
    pub provenance: CheckerOwnedProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TemplateSethoodEvidenceKey(String);

impl TemplateSethoodEvidenceKey {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for TemplateSethoodEvidenceKey {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for TemplateSethoodEvidenceKey {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TemplateTypeParameterSethoodSource {
    BareParameter,
    BoundInherited,
    ConstraintSupplied,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TemplateTypeParameterSethoodStatus {
    Accepted,
    Missing,
    DegradedRecovery,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateTypeParameterSethoodSeed {
    pub parameter: TemplateParameterKey,
    pub evidence_key: TemplateSethoodEvidenceKey,
    pub normalized_type: NormalizedTypeId,
    pub source_kind: TemplateTypeParameterSethoodSource,
    pub status: TemplateTypeParameterSethoodStatus,
    pub facts: Vec<TypeFactId>,
    pub diagnostics: Vec<TypeDiagnosticId>,
    pub source: CoreSourceRef,
    pub provenance: CheckerOwnedProvenance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TemplateSchemeParameterKind {
    Type,
    Predicate,
    Functor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TemplateSchemeActualKind {
    TypeExpression,
    EnclosingTypeParameter,
    Defpred,
    Deffunc,
    TemplateFunctor,
    EnclosingPredicateParameter,
    EnclosingFunctorParameter,
    PromotedTerminatingAlgorithm,
    PartialAlgorithm,
    VoidAlgorithm,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TemplateSchemeActualStatus {
    Accepted,
    SignatureMismatch,
    RoleMismatch,
    ArityMismatch,
    PartialAlgorithm,
    VoidAlgorithm,
    Unsupported,
    MissingEvidence,
    DegradedRecovery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TemplateWideningEvidenceStatus {
    Accepted,
    Missing,
    DeferredExternalDependency,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateWideningEvidenceSeed {
    pub from_type: NormalizedTypeId,
    pub to_type: NormalizedTypeId,
    pub status: TemplateWideningEvidenceStatus,
    pub facts: Vec<TypeFactId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateSubstitutionCompositionSeed {
    pub enclosing_parameter: TemplateParameterKey,
    pub source: CoreSourceRef,
    pub provenance: CheckerOwnedProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateSchemeActualSeed {
    pub instantiation: TemplateInstantiationKey,
    pub parameter: TemplateParameterKey,
    pub parameter_kind: TemplateSchemeParameterKind,
    pub actual_kind: TemplateSchemeActualKind,
    pub status: TemplateSchemeActualStatus,
    pub expected_arity: usize,
    pub actual_arity: usize,
    /// Direction: schema-domain type widens to actual declared parameter type.
    pub domain_evidence: Vec<TemplateWideningEvidenceSeed>,
    /// Direction for functors: actual declared result type widens to schema codomain.
    pub codomain_evidence: Option<TemplateWideningEvidenceSeed>,
    pub guard_obligation: Option<CarriedInitialObligationSeed>,
    pub substitution: Option<TemplateSubstitutionCompositionSeed>,
    pub checker_diagnostics: Vec<TypeDiagnosticId>,
    pub source: CoreSourceRef,
    pub provenance: CheckerOwnedProvenance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ViewExplanationKind {
    SourceQua,
    InsertedView,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReductViewSeed {
    pub path: QuaPathKey,
    pub functors: Vec<SymbolId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewExplanationSeed {
    pub kind: ViewExplanationKind,
    pub inserted_view: Option<CoercionInsertionId>,
    pub target_type: Option<NormalizedTypeId>,
    pub reduct: Option<ReductViewSeed>,
    pub evidence_facts: Vec<TypeFactId>,
    pub source: CoreSourceRef,
    pub provenance: CheckerOwnedProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconsideringSeed {
    pub var: CoreVarId,
    pub role: CoreVarRole,
    pub predicate: Option<CoreTypePredicate>,
    pub obligation: Option<CarriedInitialObligationSeed>,
    pub source_name: Option<String>,
    pub source: CoreSourceRef,
    pub provenance: CheckerOwnedProvenance,
}

impl ReconsideringSeed {
    pub fn new(
        var: CoreVarId,
        role: impl Into<CoreVarRole>,
        source: CoreSourceRef,
        provenance: CheckerOwnedProvenance,
    ) -> Self {
        Self {
            var,
            role: role.into(),
            predicate: None,
            obligation: None,
            source_name: None,
            source,
            provenance,
        }
    }

    pub fn with_predicate(mut self, predicate: impl Into<CoreTypePredicate>) -> Self {
        self.predicate = Some(predicate.into());
        self
    }

    pub fn with_obligation(mut self, obligation: CarriedInitialObligationSeed) -> Self {
        self.obligation = Some(obligation);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObligationFormulaSeed {
    pub subject: CoreVarId,
    pub predicate: CoreTypePredicate,
    pub polarity: Polarity,
    pub source: CoreSourceRef,
}

impl ObligationFormulaSeed {
    pub fn positive(
        subject: CoreVarId,
        predicate: impl Into<CoreTypePredicate>,
        source: CoreSourceRef,
    ) -> Self {
        Self {
            subject,
            predicate: predicate.into(),
            polarity: Polarity::Positive,
            source,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CarriedInitialObligationSeed {
    pub checker_obligation: Option<InitialObligationId>,
    pub checker_kind: InitialObligationKind,
    pub status: ObligationSeedStatus,
    pub goal: Option<ObligationFormulaSeed>,
    pub context: Vec<ObligationFormulaSeed>,
    pub local_path: LocalProofOrProgramPath,
    pub semantic_origin: NormalizedSemanticOrigin,
    pub source: CoreSourceRef,
    pub provenance: CheckerOwnedProvenance,
}

impl CarriedInitialObligationSeed {
    pub fn active(
        checker_obligation: InitialObligationId,
        checker_kind: InitialObligationKind,
        goal: ObligationFormulaSeed,
        local_path: impl Into<LocalProofOrProgramPath>,
        semantic_origin: impl Into<NormalizedSemanticOrigin>,
        source: CoreSourceRef,
        provenance: CheckerOwnedProvenance,
    ) -> Self {
        Self {
            checker_obligation: Some(checker_obligation),
            checker_kind,
            status: ObligationSeedStatus::Active,
            goal: Some(goal),
            context: Vec::new(),
            local_path: local_path.into(),
            semantic_origin: semantic_origin.into(),
            source,
            provenance,
        }
    }

    pub fn with_context(mut self, context: Vec<ObligationFormulaSeed>) -> Self {
        self.context = context;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum MissingEvidenceKind {
    Sethood,
    NonEmptiness,
    Coercion,
    Cluster,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingEvidenceSeed {
    pub kind: MissingEvidenceKind,
    pub diagnostic: Option<TypeDiagnosticId>,
    pub deferred_obligation: Option<CarriedInitialObligationSeed>,
    pub source: CoreSourceRef,
    pub provenance: CheckerOwnedProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeAndFactLoweringOutput {
    pub terms: CoreTermTable,
    pub formulas: CoreFormulaTable,
    pub obligation_seeds: ObligationSeedTable,
    pub source_map: CoreSourceMap,
    pub diagnostics: CoreDiagnosticTable,
    pub binder_guards: Vec<LoweredBinderGuard>,
    pub assumptions: Vec<CoreFormulaId>,
    pub assertions: Vec<CoreFormulaId>,
    pub attribute_formulas: Vec<CoreFormulaId>,
    pub mode_expansions: Vec<LoweredModeExpansion>,
    pub cluster_facts: Vec<LoweredClusterFact>,
    pub view_explanations: Vec<ViewExplanation>,
    pub template_type_parameter_inhabitations: Vec<LoweredTemplateTypeParameterInhabitation>,
    pub template_type_actual_gates: Vec<TemplateTypeActualGate>,
    pub template_type_parameter_sethoods: Vec<TemplateTypeParameterSethood>,
    pub template_scheme_actuals: Vec<TemplateSchemeActual>,
    pub reconsidered_binders: Vec<ReconsideredBinding>,
    pub carried_obligations: Vec<ObligationSeedId>,
    pub missing_evidence: Vec<MissingEvidenceRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredBinderGuard {
    pub binder: CoreBinder,
    pub assumption: CoreFormulaId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredModeExpansion {
    pub normalized_type: NormalizedTypeId,
    pub formula: CoreFormulaId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredClusterFact {
    pub cluster_fact: ClusterFactId,
    pub formula: CoreFormulaId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredTemplateTypeParameterInhabitation {
    pub parameter: TemplateParameterKey,
    pub witness: CoreBinder,
    pub witness_term: CoreTermId,
    pub witness_fact: CoreFormulaId,
    pub assumption: CoreFormulaId,
    pub predicate: CoreTypePredicate,
    pub provenance: Vec<CoreProvenance>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateTypeActualGate {
    pub instantiation: TemplateInstantiationKey,
    pub parameter: TemplateParameterKey,
    pub actual_type: NormalizedTypeId,
    pub gate: Option<ExistentialGateId>,
    pub status: ExistentialGateStatus,
    pub registration: Option<CheckerRegistrationId>,
    pub base_evidence_kind: Option<ExistentialGateBaseEvidenceKind>,
    pub base_evidence_coverage: Option<ExistentialGateBaseEvidenceCoverage>,
    pub facts: Vec<TypeFactId>,
    pub checker_diagnostics: Vec<RegistrationDiagnosticId>,
    pub diagnostic: Option<CoreDiagnosticId>,
    pub source: CoreSourceRef,
    pub provenance: Vec<CoreProvenance>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateTypeParameterSethood {
    pub parameter: TemplateParameterKey,
    pub evidence_key: TemplateSethoodEvidenceKey,
    pub normalized_type: NormalizedTypeId,
    pub source_kind: TemplateTypeParameterSethoodSource,
    pub status: TemplateTypeParameterSethoodStatus,
    pub facts: Vec<TypeFactId>,
    pub checker_diagnostics: Vec<TypeDiagnosticId>,
    pub diagnostic: Option<CoreDiagnosticId>,
    pub source: CoreSourceRef,
    pub provenance: Vec<CoreProvenance>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateDirectionalWideningEvidence {
    pub from_type: NormalizedTypeId,
    pub to_type: NormalizedTypeId,
    pub status: TemplateWideningEvidenceStatus,
    pub facts: Vec<TypeFactId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateSubstitutionComposition {
    pub enclosing_parameter: TemplateParameterKey,
    pub source: CoreSourceRef,
    pub provenance: Vec<CoreProvenance>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateSchemeActual {
    pub instantiation: TemplateInstantiationKey,
    pub parameter: TemplateParameterKey,
    pub parameter_kind: TemplateSchemeParameterKind,
    pub actual_kind: TemplateSchemeActualKind,
    pub status: TemplateSchemeActualStatus,
    pub expected_arity: usize,
    pub actual_arity: usize,
    pub domain_evidence: Vec<TemplateDirectionalWideningEvidence>,
    pub codomain_evidence: Option<TemplateDirectionalWideningEvidence>,
    pub guard_obligation: Option<ObligationSeedId>,
    pub substitution: Option<TemplateSubstitutionComposition>,
    pub checker_diagnostics: Vec<TypeDiagnosticId>,
    pub diagnostic: Option<CoreDiagnosticId>,
    pub source: CoreSourceRef,
    pub provenance: Vec<CoreProvenance>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewExplanation {
    pub kind: ViewExplanationKind,
    pub inserted_view: Option<CoercionInsertionId>,
    pub target_type: Option<NormalizedTypeId>,
    pub reduct: Option<ReductView>,
    pub evidence_facts: Vec<TypeFactId>,
    pub source: CoreSourceRef,
    pub provenance: Vec<CoreProvenance>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReductView {
    pub path: QuaPathKey,
    pub functors: Vec<SymbolId>,
}

impl From<ReductViewSeed> for ReductView {
    fn from(seed: ReductViewSeed) -> Self {
        Self {
            path: seed.path,
            functors: seed.functors,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconsideredBinding {
    pub binder: CoreBinder,
    pub obligation: Option<ObligationSeedId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingEvidenceRecord {
    pub kind: MissingEvidenceKind,
    pub checker_diagnostic: Option<TypeDiagnosticId>,
    pub diagnostic: CoreDiagnosticId,
    pub obligation: Option<ObligationSeedId>,
    pub provenance: Vec<CoreProvenance>,
}

#[derive(Debug, Clone)]
struct TypeAndFactLoweringState {
    owner: CoreItemId,
    terms: CoreTermTable,
    formulas: CoreFormulaTable,
    obligation_seeds: ObligationSeedTable,
    source_map: CoreSourceMap,
    diagnostics: CoreDiagnosticTable,
}

impl TypeAndFactLoweringState {
    fn new(owner: CoreItemId) -> Self {
        Self {
            owner,
            terms: CoreTermTable::new(),
            formulas: CoreFormulaTable::new(),
            obligation_seeds: ObligationSeedTable::new(),
            source_map: CoreSourceMap::new(),
            diagnostics: CoreDiagnosticTable::new(),
        }
    }

    fn insert_var_term(&mut self, var: CoreVarId, source: CoreSourceRef) -> CoreTermId {
        let source = normalized_source(source);
        let id = self
            .terms
            .insert(CoreTerm::new(CoreTermKind::Var(var), source.clone()));
        self.source_map.term_sources.insert(id, source);
        id
    }

    fn insert_formula(&mut self, kind: CoreFormulaKind, source: CoreSourceRef) -> CoreFormulaId {
        let source = normalized_source(source);
        let id = self.formulas.insert(CoreFormula::new(kind, source.clone()));
        self.source_map.formula_sources.insert(id, source);
        id
    }

    fn insert_type_predicate(
        &mut self,
        seed: &TypePredicateSeed,
    ) -> TypeAndFactResult<CoreFormulaId> {
        let subject = self.insert_var_term(seed.subject, seed.source.clone());
        let positive = self.insert_formula(
            CoreFormulaKind::TypePred {
                subject,
                ty: seed.predicate.clone(),
            },
            seed.source.clone(),
        );
        Ok(match seed.polarity {
            Polarity::Positive => positive,
            Polarity::Negative => {
                self.insert_formula(CoreFormulaKind::Not(positive), seed.source.clone())
            }
            _ => return Err(TypeAndFactLoweringError::UnsupportedPolarity),
        })
    }

    fn insert_template_type_parameter_inhabitation(
        &mut self,
        seed: TemplateTypeParameterInhabitationSeed,
    ) -> LoweredTemplateTypeParameterInhabitation {
        let witness_term = self.insert_var_term(seed.witness, seed.source.clone());
        let witness_fact = self.insert_formula(
            CoreFormulaKind::TypePred {
                subject: witness_term,
                ty: seed.predicate.clone(),
            },
            seed.source.clone(),
        );
        let witness = CoreBinder {
            var: seed.witness,
            role: seed.witness_role,
            ty_guard: None,
            source_name: seed.witness_source_name,
            source: normalized_source(seed.source.clone()),
        };
        let assumption = self.insert_formula(
            CoreFormulaKind::Exists {
                binders: vec![witness.clone()],
                body: witness_fact,
            },
            seed.source,
        );
        LoweredTemplateTypeParameterInhabitation {
            parameter: seed.parameter,
            witness,
            witness_term,
            witness_fact,
            assumption,
            predicate: seed.predicate,
            provenance: seed.provenance.as_slice().to_vec(),
        }
    }

    fn lower_template_type_actual_gate(
        &mut self,
        mut seed: TemplateTypeActualGateSeed,
    ) -> TemplateTypeActualGate {
        seed.facts.sort();
        seed.facts.dedup();
        seed.diagnostics.sort();
        seed.diagnostics.dedup();
        let diagnostic = if template_actual_gate_is_satisfied(seed.status) {
            None
        } else {
            Some(self.insert_diagnostic(
                CoreDiagnosticClass::UnresolvedSemanticInput,
                CoreDiagnosticSeverity::Error,
                CoreDiagnosticRecovery::Partial,
                template_actual_gate_message_key(seed.status),
                seed.source.clone(),
            ))
        };
        TemplateTypeActualGate {
            instantiation: seed.instantiation,
            parameter: seed.parameter,
            actual_type: seed.actual_type,
            gate: seed.gate,
            status: seed.status,
            registration: seed.registration,
            base_evidence_kind: seed.base_evidence_kind,
            base_evidence_coverage: seed.base_evidence_coverage,
            facts: seed.facts,
            checker_diagnostics: seed.diagnostics,
            diagnostic,
            source: normalized_source(seed.source),
            provenance: seed.provenance.as_slice().to_vec(),
        }
    }

    fn lower_template_type_parameter_sethood(
        &mut self,
        mut seed: TemplateTypeParameterSethoodSeed,
    ) -> TemplateTypeParameterSethood {
        seed.facts.sort();
        seed.facts.dedup();
        seed.diagnostics.sort();
        seed.diagnostics.dedup();
        let diagnostic = if template_type_parameter_sethood_is_accepted(seed.status) {
            None
        } else {
            Some(self.insert_diagnostic(
                CoreDiagnosticClass::UnresolvedSemanticInput,
                CoreDiagnosticSeverity::Error,
                CoreDiagnosticRecovery::Partial,
                template_type_parameter_sethood_message_key(seed.status),
                seed.source.clone(),
            ))
        };
        TemplateTypeParameterSethood {
            parameter: seed.parameter,
            evidence_key: seed.evidence_key,
            normalized_type: seed.normalized_type,
            source_kind: seed.source_kind,
            status: seed.status,
            facts: seed.facts,
            checker_diagnostics: seed.diagnostics,
            diagnostic,
            source: normalized_source(seed.source),
            provenance: seed.provenance.as_slice().to_vec(),
        }
    }

    fn lower_template_scheme_actual(
        &mut self,
        mut seed: TemplateSchemeActualSeed,
    ) -> TypeAndFactResult<TemplateSchemeActual> {
        for evidence in &mut seed.domain_evidence {
            evidence.facts.sort();
            evidence.facts.dedup();
        }
        if let Some(evidence) = &mut seed.codomain_evidence {
            evidence.facts.sort();
            evidence.facts.dedup();
        }
        seed.checker_diagnostics.sort();
        seed.checker_diagnostics.dedup();

        let guard_obligation = seed
            .guard_obligation
            .map(|obligation| insert_carried_obligation(self, obligation))
            .transpose()?;
        let substitution = seed
            .substitution
            .map(|substitution| TemplateSubstitutionComposition {
                enclosing_parameter: substitution.enclosing_parameter,
                source: source_with_provenance(substitution.source, &substitution.provenance),
                provenance: substitution.provenance.as_slice().to_vec(),
            });
        let diagnostic = if template_scheme_actual_is_accepted(seed.status) {
            None
        } else {
            Some(self.insert_diagnostic(
                CoreDiagnosticClass::UnresolvedSemanticInput,
                CoreDiagnosticSeverity::Error,
                CoreDiagnosticRecovery::Partial,
                template_scheme_actual_message_key(seed.status),
                seed.source.clone(),
            ))
        };

        Ok(TemplateSchemeActual {
            instantiation: seed.instantiation,
            parameter: seed.parameter,
            parameter_kind: seed.parameter_kind,
            actual_kind: seed.actual_kind,
            status: seed.status,
            expected_arity: seed.expected_arity,
            actual_arity: seed.actual_arity,
            domain_evidence: seed
                .domain_evidence
                .into_iter()
                .map(TemplateDirectionalWideningEvidence::from)
                .collect(),
            codomain_evidence: seed
                .codomain_evidence
                .map(TemplateDirectionalWideningEvidence::from),
            guard_obligation,
            substitution,
            checker_diagnostics: seed.checker_diagnostics,
            diagnostic,
            source: normalized_source(seed.source),
            provenance: seed.provenance.as_slice().to_vec(),
        })
    }

    fn insert_obligation_formula(
        &mut self,
        seed: &ObligationFormulaSeed,
    ) -> TypeAndFactResult<CoreFormulaId> {
        let fact = TypePredicateSeed {
            subject: seed.subject,
            predicate: seed.predicate.clone(),
            polarity: seed.polarity,
            checker_fact: None,
            source: seed.source.clone(),
            provenance: CheckerOwnedProvenance::checker("obligation-formula"),
        };
        self.insert_type_predicate(&fact)
    }

    fn insert_diagnostic(
        &mut self,
        class: CoreDiagnosticClass,
        severity: CoreDiagnosticSeverity,
        recovery: CoreDiagnosticRecovery,
        message_key: impl Into<CoreDiagnosticMessageKey>,
        source: CoreSourceRef,
    ) -> CoreDiagnosticId {
        self.diagnostics.insert(diagnostic(
            class,
            severity,
            recovery,
            message_key,
            source,
            Some(CoreNodeRef::Item(self.owner)),
        ))
    }
}

pub fn lower_type_and_fact_inputs(
    context: &CoreContext,
    input: TypeAndFactLoweringInput,
) -> TypeAndFactResult<TypeAndFactLoweringOutput> {
    if context.item_registry().items().get(input.owner).is_none() {
        return Err(TypeAndFactLoweringError::MissingOwnerItem { owner: input.owner });
    }
    validate_type_and_fact_input(context, &input)?;

    let mut state = TypeAndFactLoweringState::new(input.owner);
    let mut assumptions = Vec::new();
    let mut template_type_parameter_inhabitations = Vec::new();
    for seed in input.template_type_parameters {
        let lowered = state.insert_template_type_parameter_inhabitation(seed);
        assumptions.push(lowered.assumption);
        template_type_parameter_inhabitations.push(lowered);
    }

    let mut binder_guards = Vec::new();
    for seed in input.declared_binders {
        let predicate = TypePredicateSeed::positive(
            seed.var,
            seed.predicate.clone(),
            seed.source.clone(),
            seed.provenance.clone(),
        );
        let guard = state.insert_type_predicate(&predicate)?;
        let binder = CoreBinder {
            var: seed.var,
            role: seed.role,
            ty_guard: Some(guard),
            source_name: seed.source_name,
            source: normalized_source(seed.source),
        };
        assumptions.push(guard);
        binder_guards.push(LoweredBinderGuard {
            binder,
            assumption: guard,
        });
    }

    let mut assertions = Vec::new();
    for seed in input.formula_assertions {
        assertions.push(state.insert_type_predicate(&seed)?);
    }

    let mut attribute_formulas = Vec::new();
    for mut chain in input.attribute_chains {
        chain.facts.sort_by(attribute_fact_cmp);
        let mut formulas = Vec::new();
        for fact in &chain.facts {
            formulas.push(state.insert_type_predicate(fact)?);
        }
        let lowered = match formulas.as_slice() {
            [] => state.insert_formula(CoreFormulaKind::True, chain.source.clone()),
            [single] => *single,
            _ => state.insert_formula(CoreFormulaKind::And(formulas), chain.source.clone()),
        };
        attribute_formulas.push(lowered);
    }

    let mut mode_expansions = Vec::new();
    for seed in input.mode_expansions {
        let fact =
            TypePredicateSeed::positive(seed.subject, seed.predicate, seed.source, seed.provenance);
        let formula = state.insert_type_predicate(&fact)?;
        mode_expansions.push(LoweredModeExpansion {
            normalized_type: seed.normalized_type,
            formula,
        });
    }

    let mut cluster_facts = Vec::new();
    for seed in input.cluster_facts {
        let formula = state.insert_type_predicate(&seed.fact)?;
        cluster_facts.push(LoweredClusterFact {
            cluster_fact: seed.cluster_fact,
            formula,
        });
    }

    let mut view_explanations = Vec::new();
    for mut seed in input.view_explanations {
        seed.evidence_facts.sort();
        seed.evidence_facts.dedup();
        view_explanations.push(ViewExplanation {
            kind: seed.kind,
            inserted_view: seed.inserted_view,
            target_type: seed.target_type,
            reduct: seed.reduct.map(ReductView::from),
            evidence_facts: seed.evidence_facts,
            source: normalized_source(seed.source),
            provenance: seed.provenance.as_slice().to_vec(),
        });
    }

    let mut template_type_actual_gates = Vec::new();
    for seed in input.template_type_actual_gates {
        template_type_actual_gates.push(state.lower_template_type_actual_gate(seed));
    }

    let mut template_type_parameter_sethoods = Vec::new();
    for seed in input.template_type_parameter_sethoods {
        template_type_parameter_sethoods.push(state.lower_template_type_parameter_sethood(seed));
    }

    let mut template_scheme_actuals = Vec::new();
    for seed in input.template_scheme_actuals {
        template_scheme_actuals.push(state.lower_template_scheme_actual(seed)?);
    }

    let mut reconsidered_binders = Vec::new();
    for seed in input.reconsiderings {
        let guard = if let Some(predicate) = seed.predicate {
            let fact = TypePredicateSeed::positive(
                seed.var,
                predicate,
                seed.source.clone(),
                seed.provenance.clone(),
            );
            Some(state.insert_type_predicate(&fact)?)
        } else {
            None
        };
        let obligation = if let Some(obligation) = seed.obligation {
            Some(insert_carried_obligation(&mut state, obligation)?)
        } else {
            None
        };
        reconsidered_binders.push(ReconsideredBinding {
            binder: CoreBinder {
                var: seed.var,
                role: seed.role,
                ty_guard: guard,
                source_name: seed.source_name,
                source: normalized_source(seed.source),
            },
            obligation,
        });
    }

    let mut carried_obligations = Vec::new();
    for seed in input.carried_obligations {
        carried_obligations.push(insert_carried_obligation(&mut state, seed)?);
    }

    let mut missing_evidence = Vec::new();
    for seed in input.missing_evidence {
        let diagnostic_id = state.insert_diagnostic(
            CoreDiagnosticClass::UnresolvedSemanticInput,
            CoreDiagnosticSeverity::Error,
            CoreDiagnosticRecovery::Partial,
            missing_evidence_message_key(seed.kind),
            seed.source.clone(),
        );
        let obligation = if let Some(mut obligation) = seed.deferred_obligation {
            obligation.status = match obligation.status {
                ObligationSeedStatus::Active => ObligationSeedStatus::Deferred,
                status => status,
            };
            Some(insert_carried_obligation_with_diagnostics(
                &mut state,
                obligation,
                vec![diagnostic_id],
            )?)
        } else {
            None
        };
        missing_evidence.push(MissingEvidenceRecord {
            kind: seed.kind,
            checker_diagnostic: seed.diagnostic,
            diagnostic: diagnostic_id,
            obligation,
            provenance: seed.provenance.as_slice().to_vec(),
        });
    }

    Ok(TypeAndFactLoweringOutput {
        terms: state.terms,
        formulas: state.formulas,
        obligation_seeds: state.obligation_seeds,
        source_map: state.source_map,
        diagnostics: state.diagnostics,
        binder_guards,
        assumptions,
        assertions,
        attribute_formulas,
        mode_expansions,
        cluster_facts,
        view_explanations,
        template_type_parameter_inhabitations,
        template_type_actual_gates,
        template_type_parameter_sethoods,
        template_scheme_actuals,
        reconsidered_binders,
        carried_obligations,
        missing_evidence,
    })
}

fn validate_type_and_fact_input(
    context: &CoreContext,
    input: &TypeAndFactLoweringInput,
) -> TypeAndFactResult<()> {
    let mut template_parameters = BTreeSet::new();
    for seed in &input.template_type_parameters {
        validate_checker_owned_provenance(
            "template type parameter inhabitation seed",
            seed.provenance.as_slice(),
        )?;
        if !template_parameters.insert(seed.parameter.clone()) {
            return Err(TypeAndFactLoweringError::DuplicateTemplateTypeParameter {
                parameter: seed.parameter.clone(),
            });
        }
    }
    let mut template_actual_gates = BTreeSet::new();
    for seed in &input.template_type_actual_gates {
        validate_checker_owned_provenance(
            "template type actual gate seed",
            seed.provenance.as_slice(),
        )?;
        let key = (seed.instantiation.clone(), seed.parameter.clone());
        if !template_actual_gates.insert(key) {
            return Err(TypeAndFactLoweringError::DuplicateTemplateTypeActualGate {
                instantiation: seed.instantiation.clone(),
                parameter: seed.parameter.clone(),
            });
        }
        validate_template_type_actual_gate_seed(seed)?;
    }
    let mut template_type_parameter_sethoods = BTreeSet::new();
    for seed in &input.template_type_parameter_sethoods {
        validate_checker_owned_provenance(
            "template type parameter sethood seed",
            seed.provenance.as_slice(),
        )?;
        let key = (seed.parameter.clone(), seed.evidence_key.clone());
        if !template_type_parameter_sethoods.insert(key) {
            return Err(
                TypeAndFactLoweringError::DuplicateTemplateTypeParameterSethood {
                    parameter: seed.parameter.clone(),
                    evidence_key: seed.evidence_key.clone(),
                },
            );
        }
        validate_template_type_parameter_sethood_seed(seed)?;
    }
    let mut template_scheme_actuals = BTreeSet::new();
    for seed in &input.template_scheme_actuals {
        validate_checker_owned_provenance(
            "template scheme actual seed",
            seed.provenance.as_slice(),
        )?;
        let key = (seed.instantiation.clone(), seed.parameter.clone());
        if !template_scheme_actuals.insert(key) {
            return Err(TypeAndFactLoweringError::DuplicateTemplateSchemeActual {
                instantiation: seed.instantiation.clone(),
                parameter: seed.parameter.clone(),
            });
        }
        validate_template_scheme_actual_seed(context, seed)?;
    }
    for seed in &input.declared_binders {
        ensure_declared_subject(context, seed.var)?;
        validate_checker_owned_provenance("declared binder type seed", seed.provenance.as_slice())?;
    }
    for seed in &input.formula_assertions {
        validate_predicate_seed(context, "formula assertion seed", seed)?;
    }
    for chain in &input.attribute_chains {
        validate_checker_owned_provenance("attribute chain seed", chain.provenance.as_slice())?;
        for fact in &chain.facts {
            validate_predicate_seed(context, "attribute fact seed", fact)?;
        }
    }
    for seed in &input.mode_expansions {
        ensure_declared_subject(context, seed.subject)?;
        validate_checker_owned_provenance("mode expansion seed", seed.provenance.as_slice())?;
    }
    for seed in &input.cluster_facts {
        validate_predicate_seed(context, "cluster fact seed", &seed.fact)?;
        if seed.fact.checker_fact.is_none() {
            return Err(TypeAndFactLoweringError::ClusterFactMissingCheckerFact {
                cluster_fact: seed.cluster_fact,
            });
        }
    }
    for seed in &input.view_explanations {
        validate_checker_owned_provenance("view explanation seed", seed.provenance.as_slice())?;
        if let Some(reduct) = &seed.reduct {
            validate_type_fact_reduct_view_seed(reduct)?;
        }
    }
    for seed in &input.reconsiderings {
        validate_checker_owned_provenance("reconsidering seed", seed.provenance.as_slice())?;
        ensure_declared_subject(context, seed.var)?;
        if let Some(obligation) = &seed.obligation {
            validate_carried_obligation_seed(context, obligation, true)?;
        }
    }
    for seed in &input.carried_obligations {
        validate_carried_obligation_seed(context, seed, true)?;
    }
    for seed in &input.missing_evidence {
        validate_checker_owned_provenance("missing evidence seed", seed.provenance.as_slice())?;
        if let Some(obligation) = &seed.deferred_obligation {
            validate_carried_obligation_seed(context, obligation, true)?;
        }
    }
    Ok(())
}

fn validate_type_fact_reduct_view_seed(reduct: &ReductViewSeed) -> TypeAndFactResult<()> {
    if reduct.functors.is_empty() {
        return Err(TypeAndFactLoweringError::EmptyReductViewPayload {
            path: reduct.path.clone(),
        });
    }
    Ok(())
}

fn validate_template_type_actual_gate_seed(
    seed: &TemplateTypeActualGateSeed,
) -> TypeAndFactResult<()> {
    if seed.base_evidence_kind.is_some() != seed.base_evidence_coverage.is_some() {
        return Err(
            TypeAndFactLoweringError::PartialTemplateTypeActualBaseEvidence {
                instantiation: seed.instantiation.clone(),
                parameter: seed.parameter.clone(),
            },
        );
    }
    let has_evidence =
        seed.registration.is_some() || seed.base_evidence_kind.is_some() || !seed.facts.is_empty();
    if template_actual_gate_is_satisfied(seed.status) {
        if !has_evidence {
            return Err(
                TypeAndFactLoweringError::SatisfiedTemplateTypeActualWithoutEvidence {
                    instantiation: seed.instantiation.clone(),
                    parameter: seed.parameter.clone(),
                },
            );
        }
    } else if has_evidence {
        return Err(
            TypeAndFactLoweringError::UnsatisfiedTemplateTypeActualCarriesEvidence {
                instantiation: seed.instantiation.clone(),
                parameter: seed.parameter.clone(),
                status: seed.status,
            },
        );
    }
    Ok(())
}

fn template_actual_gate_is_satisfied(status: ExistentialGateStatus) -> bool {
    matches!(status, ExistentialGateStatus::Satisfied)
}

fn template_actual_gate_message_key(status: ExistentialGateStatus) -> &'static str {
    match status {
        ExistentialGateStatus::MissingExistential => "missing-template-type-actual-inhabitation",
        ExistentialGateStatus::BlockedGuard => "blocked-template-type-actual-inhabitation-guard",
        ExistentialGateStatus::InvalidCandidate => {
            "invalid-template-type-actual-inhabitation-candidate"
        }
        ExistentialGateStatus::DegradedRecovery => "degraded-template-type-actual-inhabitation",
        ExistentialGateStatus::Satisfied => "satisfied-template-type-actual-inhabitation",
        _ => "unsupported-template-type-actual-inhabitation-status",
    }
}

fn validate_template_type_parameter_sethood_seed(
    seed: &TemplateTypeParameterSethoodSeed,
) -> TypeAndFactResult<()> {
    let has_evidence = !seed.facts.is_empty();
    if template_type_parameter_sethood_is_accepted(seed.status) {
        if seed.source_kind == TemplateTypeParameterSethoodSource::BareParameter {
            return Err(
                TypeAndFactLoweringError::BareTemplateTypeParameterSethoodAccepted {
                    parameter: seed.parameter.clone(),
                    evidence_key: seed.evidence_key.clone(),
                },
            );
        }
        if !has_evidence {
            return Err(
                TypeAndFactLoweringError::AcceptedTemplateTypeParameterSethoodWithoutEvidence {
                    parameter: seed.parameter.clone(),
                    evidence_key: seed.evidence_key.clone(),
                },
            );
        }
    } else {
        if seed.status == TemplateTypeParameterSethoodStatus::Missing
            && seed.source_kind != TemplateTypeParameterSethoodSource::BareParameter
        {
            return Err(
                TypeAndFactLoweringError::MissingTemplateTypeParameterSethoodWrongSource {
                    parameter: seed.parameter.clone(),
                    evidence_key: seed.evidence_key.clone(),
                    source_kind: seed.source_kind,
                },
            );
        }
        if has_evidence {
            return Err(
                TypeAndFactLoweringError::MissingTemplateTypeParameterSethoodCarriesEvidence {
                    parameter: seed.parameter.clone(),
                    evidence_key: seed.evidence_key.clone(),
                    status: seed.status,
                },
            );
        }
    }
    Ok(())
}

fn template_type_parameter_sethood_is_accepted(status: TemplateTypeParameterSethoodStatus) -> bool {
    matches!(status, TemplateTypeParameterSethoodStatus::Accepted)
}

fn template_type_parameter_sethood_message_key(
    status: TemplateTypeParameterSethoodStatus,
) -> &'static str {
    match status {
        TemplateTypeParameterSethoodStatus::Missing => "missing-template-type-parameter-sethood",
        TemplateTypeParameterSethoodStatus::DegradedRecovery => {
            "degraded-template-type-parameter-sethood"
        }
        TemplateTypeParameterSethoodStatus::Accepted => "accepted-template-type-parameter-sethood",
    }
}

fn validate_template_scheme_actual_seed(
    context: &CoreContext,
    seed: &TemplateSchemeActualSeed,
) -> TypeAndFactResult<()> {
    if !template_scheme_actual_kind_matches(seed.parameter_kind, seed.actual_kind) {
        return Err(TypeAndFactLoweringError::TemplateSchemeActualKindMismatch {
            instantiation: seed.instantiation.clone(),
            parameter: seed.parameter.clone(),
            parameter_kind: seed.parameter_kind,
            actual_kind: seed.actual_kind,
        });
    }
    if template_scheme_actual_is_accepted(seed.status)
        && !template_scheme_actual_kind_can_be_accepted(seed.actual_kind)
    {
        return Err(TypeAndFactLoweringError::TemplateSchemeActualKindMismatch {
            instantiation: seed.instantiation.clone(),
            parameter: seed.parameter.clone(),
            parameter_kind: seed.parameter_kind,
            actual_kind: seed.actual_kind,
        });
    }
    if seed.substitution.is_some()
        && !template_scheme_actual_uses_enclosing_parameter(seed.actual_kind)
    {
        return Err(TypeAndFactLoweringError::TemplateSchemeActualKindMismatch {
            instantiation: seed.instantiation.clone(),
            parameter: seed.parameter.clone(),
            parameter_kind: seed.parameter_kind,
            actual_kind: seed.actual_kind,
        });
    }
    if seed.expected_arity != seed.actual_arity {
        return Err(
            TypeAndFactLoweringError::TemplateSchemeActualArityMismatch {
                instantiation: seed.instantiation.clone(),
                parameter: seed.parameter.clone(),
                expected: seed.expected_arity,
                actual: seed.actual_arity,
            },
        );
    }
    for evidence in &seed.domain_evidence {
        if evidence.status != TemplateWideningEvidenceStatus::Accepted {
            return Err(
                TypeAndFactLoweringError::TemplateSchemeActualPartialDomainEvidence {
                    instantiation: seed.instantiation.clone(),
                    parameter: seed.parameter.clone(),
                },
            );
        }
    }
    if matches!(
        seed.codomain_evidence.as_ref(),
        Some(evidence) if evidence.status != TemplateWideningEvidenceStatus::Accepted
    ) {
        return Err(
            TypeAndFactLoweringError::TemplateSchemeActualInvalidCodomainEvidence {
                instantiation: seed.instantiation.clone(),
                parameter: seed.parameter.clone(),
            },
        );
    }

    match seed.parameter_kind {
        TemplateSchemeParameterKind::Type => validate_type_scheme_actual_seed(seed),
        TemplateSchemeParameterKind::Predicate => validate_predicate_scheme_actual_seed(seed),
        TemplateSchemeParameterKind::Functor => validate_functor_scheme_actual_seed(context, seed),
    }
}

fn validate_type_scheme_actual_seed(seed: &TemplateSchemeActualSeed) -> TypeAndFactResult<()> {
    if !seed.domain_evidence.is_empty()
        || seed.codomain_evidence.is_some()
        || seed.guard_obligation.is_some()
    {
        return Err(
            TypeAndFactLoweringError::TemplateSchemeTypeActualCarriesCallableEvidence {
                instantiation: seed.instantiation.clone(),
                parameter: seed.parameter.clone(),
            },
        );
    }
    if template_scheme_actual_is_accepted(seed.status)
        && seed.actual_kind == TemplateSchemeActualKind::EnclosingTypeParameter
        && seed.substitution.is_none()
    {
        return Err(
            TypeAndFactLoweringError::TemplateSchemeActualMissingSubstitutionEvidence {
                instantiation: seed.instantiation.clone(),
                parameter: seed.parameter.clone(),
            },
        );
    }
    validate_rejected_scheme_actual_payload(seed)
}

fn validate_predicate_scheme_actual_seed(seed: &TemplateSchemeActualSeed) -> TypeAndFactResult<()> {
    if seed.codomain_evidence.is_some() {
        return Err(
            TypeAndFactLoweringError::TemplateSchemeActualInvalidCodomainEvidence {
                instantiation: seed.instantiation.clone(),
                parameter: seed.parameter.clone(),
            },
        );
    }
    if seed.guard_obligation.is_some() {
        return Err(
            TypeAndFactLoweringError::TemplateSchemeTypeActualCarriesCallableEvidence {
                instantiation: seed.instantiation.clone(),
                parameter: seed.parameter.clone(),
            },
        );
    }
    if template_scheme_actual_is_accepted(seed.status) {
        if seed.domain_evidence.len() != seed.expected_arity {
            return Err(
                TypeAndFactLoweringError::AcceptedTemplateSchemeActualMissingEvidence {
                    instantiation: seed.instantiation.clone(),
                    parameter: seed.parameter.clone(),
                },
            );
        }
        if seed.actual_kind == TemplateSchemeActualKind::EnclosingPredicateParameter
            && seed.substitution.is_none()
        {
            return Err(
                TypeAndFactLoweringError::TemplateSchemeActualMissingSubstitutionEvidence {
                    instantiation: seed.instantiation.clone(),
                    parameter: seed.parameter.clone(),
                },
            );
        }
    }
    validate_rejected_scheme_actual_payload(seed)
}

fn validate_functor_scheme_actual_seed(
    context: &CoreContext,
    seed: &TemplateSchemeActualSeed,
) -> TypeAndFactResult<()> {
    if template_scheme_actual_is_accepted(seed.status) {
        if seed.domain_evidence.len() != seed.expected_arity || seed.codomain_evidence.is_none() {
            return Err(
                TypeAndFactLoweringError::AcceptedTemplateSchemeActualMissingEvidence {
                    instantiation: seed.instantiation.clone(),
                    parameter: seed.parameter.clone(),
                },
            );
        }
        let Some(guard) = &seed.guard_obligation else {
            return Err(
                TypeAndFactLoweringError::TemplateSchemeFunctorMissingGuardSeed {
                    instantiation: seed.instantiation.clone(),
                    parameter: seed.parameter.clone(),
                },
            );
        };
        if guard.status != ObligationSeedStatus::Skipped {
            return Err(
                TypeAndFactLoweringError::TemplateSchemeFunctorInvalidGuardSeedStatus {
                    instantiation: seed.instantiation.clone(),
                    parameter: seed.parameter.clone(),
                    status: guard.status,
                },
            );
        }
        if map_initial_obligation_kind(guard.checker_kind) != ObligationSeedKind::CheckerInitial {
            return Err(
                TypeAndFactLoweringError::TemplateSchemeFunctorInvalidGuardSeedKind {
                    instantiation: seed.instantiation.clone(),
                    parameter: seed.parameter.clone(),
                    kind: guard.checker_kind,
                },
            );
        }
        validate_carried_obligation_seed(context, guard, true)?;
        if seed.actual_kind == TemplateSchemeActualKind::EnclosingFunctorParameter
            && seed.substitution.is_none()
        {
            return Err(
                TypeAndFactLoweringError::TemplateSchemeActualMissingSubstitutionEvidence {
                    instantiation: seed.instantiation.clone(),
                    parameter: seed.parameter.clone(),
                },
            );
        }
    }
    validate_rejected_scheme_actual_payload(seed)
}

fn validate_rejected_scheme_actual_payload(
    seed: &TemplateSchemeActualSeed,
) -> TypeAndFactResult<()> {
    if template_scheme_actual_is_accepted(seed.status) {
        if let Some(substitution) = &seed.substitution {
            validate_checker_owned_provenance(
                "template scheme substitution composition seed",
                substitution.provenance.as_slice(),
            )?;
        }
        return Ok(());
    }

    if !seed.domain_evidence.is_empty()
        || seed.codomain_evidence.is_some()
        || seed.guard_obligation.is_some()
        || seed.substitution.is_some()
    {
        return Err(
            TypeAndFactLoweringError::RejectedTemplateSchemeActualCarriesEvidence {
                instantiation: seed.instantiation.clone(),
                parameter: seed.parameter.clone(),
                status: seed.status,
            },
        );
    }
    Ok(())
}

fn template_scheme_actual_kind_matches(
    parameter_kind: TemplateSchemeParameterKind,
    actual_kind: TemplateSchemeActualKind,
) -> bool {
    match parameter_kind {
        TemplateSchemeParameterKind::Type => matches!(
            actual_kind,
            TemplateSchemeActualKind::TypeExpression
                | TemplateSchemeActualKind::EnclosingTypeParameter
                | TemplateSchemeActualKind::Unsupported
        ),
        TemplateSchemeParameterKind::Predicate => matches!(
            actual_kind,
            TemplateSchemeActualKind::Defpred
                | TemplateSchemeActualKind::EnclosingPredicateParameter
                | TemplateSchemeActualKind::Unsupported
        ),
        TemplateSchemeParameterKind::Functor => matches!(
            actual_kind,
            TemplateSchemeActualKind::Deffunc
                | TemplateSchemeActualKind::TemplateFunctor
                | TemplateSchemeActualKind::EnclosingFunctorParameter
                | TemplateSchemeActualKind::PromotedTerminatingAlgorithm
                | TemplateSchemeActualKind::PartialAlgorithm
                | TemplateSchemeActualKind::VoidAlgorithm
                | TemplateSchemeActualKind::Unsupported
        ),
    }
}

fn template_scheme_actual_kind_can_be_accepted(actual_kind: TemplateSchemeActualKind) -> bool {
    matches!(
        actual_kind,
        TemplateSchemeActualKind::TypeExpression
            | TemplateSchemeActualKind::EnclosingTypeParameter
            | TemplateSchemeActualKind::Defpred
            | TemplateSchemeActualKind::Deffunc
            | TemplateSchemeActualKind::TemplateFunctor
            | TemplateSchemeActualKind::EnclosingPredicateParameter
            | TemplateSchemeActualKind::EnclosingFunctorParameter
            | TemplateSchemeActualKind::PromotedTerminatingAlgorithm
    )
}

fn template_scheme_actual_uses_enclosing_parameter(actual_kind: TemplateSchemeActualKind) -> bool {
    matches!(
        actual_kind,
        TemplateSchemeActualKind::EnclosingTypeParameter
            | TemplateSchemeActualKind::EnclosingPredicateParameter
            | TemplateSchemeActualKind::EnclosingFunctorParameter
    )
}

fn template_scheme_actual_is_accepted(status: TemplateSchemeActualStatus) -> bool {
    matches!(status, TemplateSchemeActualStatus::Accepted)
}

fn template_scheme_actual_message_key(status: TemplateSchemeActualStatus) -> &'static str {
    match status {
        TemplateSchemeActualStatus::SignatureMismatch => {
            "template-scheme-actual-signature-mismatch"
        }
        TemplateSchemeActualStatus::RoleMismatch => "template-scheme-actual-role-mismatch",
        TemplateSchemeActualStatus::ArityMismatch => "template-scheme-actual-arity-mismatch",
        TemplateSchemeActualStatus::PartialAlgorithm => "partial-algorithm-template-functor-actual",
        TemplateSchemeActualStatus::VoidAlgorithm => "void-algorithm-template-functor-actual",
        TemplateSchemeActualStatus::Unsupported => "unsupported-template-scheme-actual",
        TemplateSchemeActualStatus::MissingEvidence => "template-scheme-actual-missing-evidence",
        TemplateSchemeActualStatus::DegradedRecovery => "template-scheme-actual-degraded-recovery",
        TemplateSchemeActualStatus::Accepted => "accepted-template-scheme-actual",
    }
}

impl From<TemplateWideningEvidenceSeed> for TemplateDirectionalWideningEvidence {
    fn from(mut seed: TemplateWideningEvidenceSeed) -> Self {
        seed.facts.sort();
        seed.facts.dedup();
        Self {
            from_type: seed.from_type,
            to_type: seed.to_type,
            status: seed.status,
            facts: seed.facts,
        }
    }
}

fn validate_predicate_seed(
    context: &CoreContext,
    input: &'static str,
    seed: &TypePredicateSeed,
) -> TypeAndFactResult<()> {
    ensure_declared_subject(context, seed.subject)?;
    validate_checker_owned_provenance(input, seed.provenance.as_slice())?;
    Ok(())
}

fn validate_carried_obligation_seed(
    context: &CoreContext,
    seed: &CarriedInitialObligationSeed,
    allow_goal_subjects_from_context: bool,
) -> TypeAndFactResult<()> {
    validate_checker_owned_provenance("carried obligation seed", seed.provenance.as_slice())?;
    if seed.status == ObligationSeedStatus::Active && seed.goal.is_none() {
        return Err(TypeAndFactLoweringError::MissingActiveObligationGoal {
            obligation: seed.checker_obligation,
        });
    }
    if seed.status != ObligationSeedStatus::Active
        && seed.goal.is_none()
        && seed.provenance.as_slice().is_empty()
    {
        return Err(TypeAndFactLoweringError::InactiveObligationWithoutReason {
            obligation: seed.checker_obligation,
        });
    }
    if allow_goal_subjects_from_context {
        if let Some(goal) = &seed.goal {
            ensure_declared_subject(context, goal.subject)?;
        }
        for fact in &seed.context {
            ensure_declared_subject(context, fact.subject)?;
        }
    }
    Ok(())
}

fn ensure_declared_subject(context: &CoreContext, var: CoreVarId) -> TypeAndFactResult<()> {
    if !context.binder_context().free_variables.contains(&var) {
        return Err(TypeAndFactLoweringError::UndeclaredSubject { var });
    }
    match context.binder_context().variable_sorts.get(&var) {
        Some(NormalizedVarSort::Term) => Ok(()),
        Some(sort) => Err(TypeAndFactLoweringError::NonTermSubject { var, sort: *sort }),
        None => Err(TypeAndFactLoweringError::UndeclaredSubject { var }),
    }
}

fn insert_carried_obligation(
    state: &mut TypeAndFactLoweringState,
    seed: CarriedInitialObligationSeed,
) -> TypeAndFactResult<ObligationSeedId> {
    insert_carried_obligation_with_diagnostics(state, seed, Vec::new())
}

fn insert_carried_obligation_with_diagnostics(
    state: &mut TypeAndFactLoweringState,
    seed: CarriedInitialObligationSeed,
    diagnostics: Vec<CoreDiagnosticId>,
) -> TypeAndFactResult<ObligationSeedId> {
    if seed.status == ObligationSeedStatus::Active && seed.goal.is_none() {
        return Err(TypeAndFactLoweringError::MissingActiveObligationGoal {
            obligation: seed.checker_obligation,
        });
    }
    if seed.status != ObligationSeedStatus::Active
        && seed.goal.is_none()
        && diagnostics.is_empty()
        && seed.provenance.as_slice().is_empty()
    {
        return Err(TypeAndFactLoweringError::InactiveObligationWithoutReason {
            obligation: seed.checker_obligation,
        });
    }

    let goal = seed
        .goal
        .as_ref()
        .map(|goal| state.insert_obligation_formula(goal))
        .transpose()?;
    let mut context_formulas = Vec::new();
    for fact in &seed.context {
        context_formulas.push(state.insert_obligation_formula(fact)?);
    }
    let mut provenance = seed.provenance.as_slice().to_vec();
    if let Some(obligation) = seed.checker_obligation {
        provenance.push(CoreProvenance::new(
            CoreProvenancePhase::Checker,
            format!("initial-obligation#{}", obligation.index()),
        ));
    }
    provenance.sort();
    provenance.dedup();
    let mut core_refs = vec![CoreNodeRef::Item(state.owner)];
    if let Some(goal) = goal {
        core_refs.push(CoreNodeRef::Formula(goal));
    }
    for formula in &context_formulas {
        core_refs.push(CoreNodeRef::Formula(*formula));
    }

    let source = normalized_source(seed.source);
    let obligation = ObligationSeed {
        owner: state.owner,
        kind: map_initial_obligation_kind(seed.checker_kind),
        goal,
        context: context_formulas,
        local_path: seed.local_path,
        label: None,
        semantic_origin: seed.semantic_origin,
        provenance,
        source: source.clone(),
        core_refs,
        status: seed.status,
        diagnostics,
    };
    let id = state.obligation_seeds.insert(obligation);
    state.source_map.obligation_sources.insert(id, source);
    Ok(id)
}

fn attribute_fact_cmp(left: &TypePredicateSeed, right: &TypePredicateSeed) -> std::cmp::Ordering {
    left.predicate
        .cmp(&right.predicate)
        .then_with(|| left.polarity.cmp(&right.polarity))
        .then_with(|| source_order_key(&left.source).cmp(&source_order_key(&right.source)))
        .then_with(|| left.checker_fact.cmp(&right.checker_fact))
}

fn map_initial_obligation_kind(kind: InitialObligationKind) -> ObligationSeedKind {
    match kind {
        InitialObligationKind::Sethood => ObligationSeedKind::GeneratedSethood,
        InitialObligationKind::NonEmptiness => ObligationSeedKind::GeneratedNonEmptiness,
        InitialObligationKind::Narrowing | InitialObligationKind::RegistrationCorrectness => {
            ObligationSeedKind::CheckerInitial
        }
        _ => ObligationSeedKind::CheckerInitial,
    }
}

fn missing_evidence_message_key(kind: MissingEvidenceKind) -> &'static str {
    match kind {
        MissingEvidenceKind::Sethood => "missing-sethood-evidence",
        MissingEvidenceKind::NonEmptiness => "missing-non-emptiness-evidence",
        MissingEvidenceKind::Coercion => "missing-coercion-evidence",
        MissingEvidenceKind::Cluster => "missing-cluster-evidence",
    }
}

pub type TermAndFormulaResult<T> = Result<T, TermAndFormulaLoweringError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TemplateSethoodRecordErrorKind {
    Duplicate,
    Missing,
    NotAccepted,
    WrongSource,
    NormalizedTypeMismatch,
    MissingEvidence,
    UnexpectedBareEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TermAndFormulaLoweringError {
    MissingOwnerItem {
        owner: CoreItemId,
    },
    MissingTermSeed {
        seed: CoreTermSeedId,
    },
    MissingFormulaSeed {
        seed: CoreFormulaSeedId,
    },
    CyclicTermSeed {
        seed: CoreTermSeedId,
    },
    CyclicFormulaSeed {
        seed: CoreFormulaSeedId,
    },
    UndeclaredVariable {
        var: CoreVarId,
    },
    NonTermVariable {
        var: CoreVarId,
        sort: NormalizedVarSort,
    },
    FutureBinderInGuard {
        binder: CoreVarId,
        later: CoreVarId,
    },
    GeneratedOriginParameterMismatch {
        origin: GeneratedOriginId,
        key: GeneratedOriginKey,
    },
    MissingGeneratedOriginFunctor {
        origin: GeneratedOriginId,
        key: GeneratedOriginKey,
    },
    GeneratedFunctorMismatch {
        key: GeneratedOriginKey,
        expected: Box<SymbolId>,
        actual: Box<SymbolId>,
    },
    InvalidFraenkelMembershipObligation {
        kind: ObligationSeedKind,
        status: ObligationSeedStatus,
    },
    InvalidFraenkelMissingSethoodObligation {
        kind: ObligationSeedKind,
    },
    InvalidTemplateFraenkelSethoodEvidence {
        parameter: TemplateParameterKey,
        evidence_key: TemplateSethoodEvidenceKey,
        reason: TemplateSethoodRecordErrorKind,
    },
    MissingActiveObligationGoal {
        kind: ObligationSeedKind,
    },
    EmptyReductViewPayload {
        path: QuaPathKey,
    },
    InvalidSeedProvenance(CoreContextError),
}

impl fmt::Display for TermAndFormulaLoweringError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingOwnerItem { owner } => {
                write!(formatter, "missing core item owner {}", owner.index())
            }
            Self::MissingTermSeed { seed } => {
                write!(formatter, "missing term seed {}", seed.index())
            }
            Self::MissingFormulaSeed { seed } => {
                write!(formatter, "missing formula seed {}", seed.index())
            }
            Self::CyclicTermSeed { seed } => {
                write!(formatter, "cyclic term seed {}", seed.index())
            }
            Self::CyclicFormulaSeed { seed } => {
                write!(formatter, "cyclic formula seed {}", seed.index())
            }
            Self::UndeclaredVariable { var } => {
                write!(
                    formatter,
                    "undeclared term/formula variable {}",
                    var.index()
                )
            }
            Self::NonTermVariable { var, sort } => {
                write!(
                    formatter,
                    "term/formula variable {} has non-term sort {sort:?}",
                    var.index()
                )
            }
            Self::FutureBinderInGuard { binder, later } => {
                write!(
                    formatter,
                    "guard for binder {} references later binder {}",
                    binder.index(),
                    later.index()
                )
            }
            Self::GeneratedOriginParameterMismatch { origin, key } => {
                write!(
                    formatter,
                    "generated origin {} for key {} has different normalized params",
                    origin.index(),
                    key.as_str()
                )
            }
            Self::MissingGeneratedOriginFunctor { origin, key } => {
                write!(
                    formatter,
                    "generated origin {} for key {} is missing its generated functor",
                    origin.index(),
                    key.as_str()
                )
            }
            Self::GeneratedFunctorMismatch {
                key,
                expected,
                actual,
            } => {
                write!(
                    formatter,
                    "generated origin key {} expected functor {expected:?}, got {actual:?}",
                    key.as_str()
                )
            }
            Self::InvalidFraenkelMembershipObligation { kind, status } => {
                write!(
                    formatter,
                    "Fraenkel membership obligation must be active FraenkelMembershipAxiom, got {kind:?}/{status:?}"
                )
            }
            Self::InvalidFraenkelMissingSethoodObligation { kind } => {
                write!(
                    formatter,
                    "missing Fraenkel sethood obligation must be GeneratedSethood, got {kind:?}"
                )
            }
            Self::InvalidTemplateFraenkelSethoodEvidence {
                parameter,
                evidence_key,
                reason,
            } => {
                write!(
                    formatter,
                    "template Fraenkel sethood evidence for parameter {} evidence {} is invalid: {reason:?}",
                    parameter.as_str(),
                    evidence_key.as_str()
                )
            }
            Self::MissingActiveObligationGoal { kind } => {
                write!(formatter, "active {kind:?} obligation is missing a goal")
            }
            Self::EmptyReductViewPayload { path } => {
                write!(
                    formatter,
                    "reduct view path {} needs at least one explicit view functor",
                    path.as_str()
                )
            }
            Self::InvalidSeedProvenance(error) => write!(formatter, "{error}"),
        }
    }
}

impl Error for TermAndFormulaLoweringError {}

impl From<CoreContextError> for TermAndFormulaLoweringError {
    fn from(value: CoreContextError) -> Self {
        Self::InvalidSeedProvenance(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CoreTermSeedId(usize);

impl CoreTermSeedId {
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    pub const fn index(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CoreFormulaSeedId(usize);

impl CoreFormulaSeedId {
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    pub const fn index(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermAndFormulaLoweringInput {
    pub owner: CoreItemId,
    pub template_type_parameter_sethoods: Vec<TemplateTypeParameterSethood>,
    pub terms: Vec<CoreTermSeed>,
    pub formulas: Vec<CoreFormulaSeed>,
    pub failed_sites: Vec<FailedSemanticSiteSeed>,
}

impl TermAndFormulaLoweringInput {
    pub const fn new(owner: CoreItemId) -> Self {
        Self {
            owner,
            template_type_parameter_sethoods: Vec::new(),
            terms: Vec::new(),
            formulas: Vec::new(),
            failed_sites: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreTermSeed {
    pub kind: CoreTermSeedKind,
    pub source: CoreSourceRef,
    pub provenance: CheckerOwnedProvenance,
}

impl CoreTermSeed {
    pub fn new(
        kind: CoreTermSeedKind,
        source: CoreSourceRef,
        provenance: CheckerOwnedProvenance,
    ) -> Self {
        Self {
            kind,
            source,
            provenance,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateFraenkelSethoodEvidenceSeed {
    pub parameter: TemplateParameterKey,
    pub evidence_key: TemplateSethoodEvidenceKey,
    pub normalized_type: NormalizedTypeId,
    pub source: CoreSourceRef,
    pub provenance: CheckerOwnedProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CoreTermSeedKind {
    Var(CoreVarId),
    Const(SymbolId),
    Apply {
        functor: SymbolId,
        args: Vec<CoreTermSeedId>,
    },
    Select {
        selector: SymbolId,
        base: CoreTermSeedId,
    },
    Tuple(Vec<CoreTermSeedId>),
    SetEnum(Vec<CoreTermSeedId>),
    Qua {
        base: CoreTermSeedId,
        explanation: ViewExplanationSeed,
    },
    StableChoice {
        functor: SymbolId,
        origin_functor: SymbolId,
        key: GeneratedOriginKey,
        params: Vec<CoreVarId>,
        args: Vec<CoreTermSeedId>,
        evidence: Vec<CoreProvenance>,
    },
    Fraenkel {
        functor: SymbolId,
        origin_functor: SymbolId,
        key: GeneratedOriginKey,
        params: Vec<CoreVarId>,
        args: Vec<CoreTermSeedId>,
        sethood_evidence: Vec<CoreProvenance>,
        template_type_parameter_sethood: Option<TemplateFraenkelSethoodEvidenceSeed>,
        membership_obligation: Box<FraenkelMembershipObligationSeed>,
        missing_sethood_obligation: Option<Box<CoreObligationSeed>>,
    },
    Error(FailedSemanticSiteSeed),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreFormulaSeed {
    pub kind: CoreFormulaSeedKind,
    pub source: CoreSourceRef,
    pub provenance: CheckerOwnedProvenance,
}

impl CoreFormulaSeed {
    pub fn new(
        kind: CoreFormulaSeedKind,
        source: CoreSourceRef,
        provenance: CheckerOwnedProvenance,
    ) -> Self {
        Self {
            kind,
            source,
            provenance,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CoreFormulaSeedKind {
    True,
    False,
    Atom {
        predicate: SymbolId,
        args: Vec<CoreTermSeedId>,
    },
    Equals {
        left: CoreTermSeedId,
        right: CoreTermSeedId,
    },
    TypePred {
        subject: CoreTermSeedId,
        ty: CoreTypePredicate,
    },
    Not(CoreFormulaSeedId),
    And(Vec<CoreFormulaSeedId>),
    Or(Vec<CoreFormulaSeedId>),
    Implies {
        premise: CoreFormulaSeedId,
        conclusion: CoreFormulaSeedId,
    },
    Iff {
        left: CoreFormulaSeedId,
        right: CoreFormulaSeedId,
    },
    Forall {
        binders: Vec<QuantifierBinderSeed>,
        body: CoreFormulaSeedId,
    },
    Exists {
        binders: Vec<QuantifierBinderSeed>,
        body: CoreFormulaSeedId,
    },
    Error(FailedSemanticSiteSeed),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuantifierBinderSeed {
    pub var: CoreVarId,
    pub role: CoreVarRole,
    pub guard: Option<CoreFormulaSeedId>,
    pub guard_mentions: Vec<CoreVarId>,
    pub source_name: Option<String>,
    pub source: CoreSourceRef,
    pub provenance: CheckerOwnedProvenance,
}

impl QuantifierBinderSeed {
    pub fn new(
        var: CoreVarId,
        role: impl Into<CoreVarRole>,
        source: CoreSourceRef,
        provenance: CheckerOwnedProvenance,
    ) -> Self {
        Self {
            var,
            role: role.into(),
            guard: None,
            guard_mentions: Vec::new(),
            source_name: None,
            source,
            provenance,
        }
    }

    pub fn with_guard(mut self, guard: CoreFormulaSeedId, mentions: Vec<CoreVarId>) -> Self {
        self.guard = Some(guard);
        self.guard_mentions = mentions;
        self
    }

    pub fn with_source_name(mut self, source_name: impl Into<String>) -> Self {
        self.source_name = Some(source_name.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailedSemanticSiteSeed {
    pub class: CoreDiagnosticClass,
    pub severity: CoreDiagnosticSeverity,
    pub recovery: CoreDiagnosticRecovery,
    pub message_key: CoreDiagnosticMessageKey,
    pub source: CoreSourceRef,
    pub provenance: CheckerOwnedProvenance,
}

impl FailedSemanticSiteSeed {
    pub fn error(
        message_key: impl Into<CoreDiagnosticMessageKey>,
        source: CoreSourceRef,
        provenance: CheckerOwnedProvenance,
    ) -> Self {
        Self {
            class: CoreDiagnosticClass::UnsupportedLowering,
            severity: CoreDiagnosticSeverity::Error,
            recovery: CoreDiagnosticRecovery::Fatal,
            message_key: message_key.into(),
            source,
            provenance,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreObligationSeed {
    pub kind: ObligationSeedKind,
    pub status: ObligationSeedStatus,
    pub goal: Option<CoreFormulaSeedId>,
    pub context: Vec<CoreFormulaSeedId>,
    pub local_path: LocalProofOrProgramPath,
    pub label: Option<crate::core_ir::CoreLabelRef>,
    pub semantic_origin: NormalizedSemanticOrigin,
    pub source: CoreSourceRef,
    pub provenance: CheckerOwnedProvenance,
}

impl CoreObligationSeed {
    pub fn active(
        kind: ObligationSeedKind,
        goal: CoreFormulaSeedId,
        local_path: impl Into<LocalProofOrProgramPath>,
        semantic_origin: impl Into<NormalizedSemanticOrigin>,
        source: CoreSourceRef,
        provenance: CheckerOwnedProvenance,
    ) -> Self {
        Self {
            kind,
            status: ObligationSeedStatus::Active,
            goal: Some(goal),
            context: Vec::new(),
            local_path: local_path.into(),
            label: None,
            semantic_origin: semantic_origin.into(),
            source,
            provenance,
        }
    }

    pub fn deferred(
        kind: ObligationSeedKind,
        local_path: impl Into<LocalProofOrProgramPath>,
        semantic_origin: impl Into<NormalizedSemanticOrigin>,
        source: CoreSourceRef,
        provenance: CheckerOwnedProvenance,
    ) -> Self {
        Self {
            kind,
            status: ObligationSeedStatus::Deferred,
            goal: None,
            context: Vec::new(),
            local_path: local_path.into(),
            label: None,
            semantic_origin: semantic_origin.into(),
            source,
            provenance,
        }
    }

    pub fn with_context(mut self, context: Vec<CoreFormulaSeedId>) -> Self {
        self.context = context;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum FraenkelMembershipObligationSeed {
    New(CoreObligationSeed),
    AlreadyCarried(AlreadyCarriedFraenkelMembershipSeed),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlreadyCarriedFraenkelMembershipSeed {
    pub source: CoreSourceRef,
    pub provenance: CheckerOwnedProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermAndFormulaLoweringOutput {
    pub terms: CoreTermTable,
    pub formulas: CoreFormulaTable,
    /// Step-1 generated origins merged with Task-10 additions for `CoreIr` validation.
    pub generated: GeneratedOriginTable,
    /// Generated origins newly emitted by this lowering slice only.
    pub generated_delta: GeneratedOriginTable,
    pub obligation_seeds: ObligationSeedTable,
    pub source_map: CoreSourceMap,
    pub diagnostics: CoreDiagnosticTable,
    pub term_map: BTreeMap<CoreTermSeedId, CoreTermId>,
    pub formula_map: BTreeMap<CoreFormulaSeedId, CoreFormulaId>,
    pub new_generated_origins: Vec<GeneratedOriginId>,
    pub generated_origin_refs: Vec<GeneratedOriginUse>,
    pub view_explanations: Vec<ViewExplanation>,
    pub generated_obligations: Vec<LoweredGeneratedObligation>,
    pub already_carried_generated_obligations: Vec<AlreadyCarriedGeneratedObligation>,
    pub failed_sites: Vec<CoreDiagnosticId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedOriginUse {
    pub term: CoreTermId,
    pub origin: GeneratedOriginId,
    pub kind: GeneratedOriginKind,
    pub key: GeneratedOriginKey,
    pub functor: SymbolId,
    pub args: Vec<CoreTermId>,
    pub source: CoreSourceRef,
    pub reused_existing: bool,
    pub reuse_source: GeneratedOriginReuseSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum GeneratedOriginReuseSource {
    ExistingRegistry,
    NewDelta,
    CurrentDelta,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GeneratedOriginDraft {
    origin: GeneratedOriginId,
    kind: GeneratedOriginKind,
    key: GeneratedOriginKey,
    source: CoreSourceRef,
    reused_existing: bool,
    reuse_source: GeneratedOriginReuseSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GeneratedOriginRequest {
    kind: GeneratedOriginKind,
    key: GeneratedOriginKey,
    functor: SymbolId,
    params: Vec<CoreVarId>,
    evidence: Vec<CoreProvenance>,
    source: CoreSourceRef,
    provenance: CheckerOwnedProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredGeneratedObligation {
    pub obligation: ObligationSeedId,
    pub kind: ObligationSeedKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlreadyCarriedGeneratedObligation {
    pub origin: GeneratedOriginId,
    pub kind: ObligationSeedKind,
    pub source: CoreSourceRef,
    pub provenance: Vec<CoreProvenance>,
}

#[derive(Debug, Clone)]
struct TermAndFormulaLoweringState {
    owner: CoreItemId,
    terms: CoreTermTable,
    formulas: CoreFormulaTable,
    generated: GeneratedOriginTable,
    generated_delta: GeneratedOriginTable,
    initial_generated_keys: BTreeSet<(CoreItemId, GeneratedOriginKind, GeneratedOriginKey)>,
    generated_by_key:
        BTreeMap<(CoreItemId, GeneratedOriginKind, GeneratedOriginKey), GeneratedOriginId>,
    obligation_seeds: ObligationSeedTable,
    source_map: CoreSourceMap,
    diagnostics: CoreDiagnosticTable,
    term_map: BTreeMap<CoreTermSeedId, CoreTermId>,
    formula_map: BTreeMap<CoreFormulaSeedId, CoreFormulaId>,
    term_stack: BTreeSet<CoreTermSeedId>,
    formula_stack: BTreeSet<CoreFormulaSeedId>,
    new_generated_origins: Vec<GeneratedOriginId>,
    generated_origin_refs: Vec<GeneratedOriginUse>,
    view_explanations: Vec<ViewExplanation>,
    generated_obligations: Vec<LoweredGeneratedObligation>,
    already_carried_generated_obligations: Vec<AlreadyCarriedGeneratedObligation>,
    failed_sites: Vec<CoreDiagnosticId>,
}

impl TermAndFormulaLoweringState {
    fn new(context: &CoreContext, owner: CoreItemId) -> Self {
        let generated = context.generated_origins().table().clone();
        let mut generated_by_key = BTreeMap::new();
        let mut initial_generated_keys = BTreeSet::new();
        for (id, origin) in generated.iter() {
            let key = (origin.owner, origin.kind, origin.key.clone());
            initial_generated_keys.insert(key.clone());
            generated_by_key.insert(key, id);
        }

        let mut source_map = CoreSourceMap::new();
        source_map.item_sources = context.source_map().item_sources.clone();
        source_map.generated_sources = context.source_map().generated_sources.clone();

        Self {
            owner,
            terms: CoreTermTable::new(),
            formulas: CoreFormulaTable::new(),
            generated,
            generated_delta: GeneratedOriginTable::new(),
            initial_generated_keys,
            generated_by_key,
            obligation_seeds: ObligationSeedTable::new(),
            source_map,
            diagnostics: CoreDiagnosticTable::new(),
            term_map: BTreeMap::new(),
            formula_map: BTreeMap::new(),
            term_stack: BTreeSet::new(),
            formula_stack: BTreeSet::new(),
            new_generated_origins: Vec::new(),
            generated_origin_refs: Vec::new(),
            view_explanations: Vec::new(),
            generated_obligations: Vec::new(),
            already_carried_generated_obligations: Vec::new(),
            failed_sites: Vec::new(),
        }
    }

    fn insert_term(&mut self, kind: CoreTermKind, source: CoreSourceRef) -> CoreTermId {
        let source = normalized_source(source);
        let id = self.terms.insert(CoreTerm::new(kind, source.clone()));
        self.source_map.term_sources.insert(id, source);
        id
    }

    fn insert_formula(&mut self, kind: CoreFormulaKind, source: CoreSourceRef) -> CoreFormulaId {
        let source = normalized_source(source);
        let id = self.formulas.insert(CoreFormula::new(kind, source.clone()));
        self.source_map.formula_sources.insert(id, source);
        id
    }

    fn insert_failed_site(&mut self, site: FailedSemanticSiteSeed) -> CoreDiagnosticId {
        self.diagnostics.insert(diagnostic(
            site.class,
            site.severity,
            site.recovery,
            site.message_key,
            source_with_provenance(site.source, &site.provenance),
            Some(CoreNodeRef::Item(self.owner)),
        ))
    }

    fn set_diagnostic_owner(&mut self, diagnostic_id: CoreDiagnosticId, owner: CoreNodeRef) {
        if let Some(diagnostic) = self.diagnostics.get_mut(diagnostic_id) {
            diagnostic.owner = Some(owner);
        }
    }

    fn ensure_generated_origin(
        &mut self,
        request: GeneratedOriginRequest,
    ) -> TermAndFormulaResult<GeneratedOriginDraft> {
        let GeneratedOriginRequest {
            kind,
            key,
            functor,
            params,
            evidence,
            source,
            provenance,
        } = request;
        let map_key = (self.owner, kind, key.clone());
        let source = source_with_provenance(source, &provenance);
        if let Some(origin) = self.generated_by_key.get(&map_key).copied() {
            let existing = self
                .generated
                .get(origin)
                .expect("generated_by_key points into generated table");
            if existing.params != params {
                return Err(
                    TermAndFormulaLoweringError::GeneratedOriginParameterMismatch { origin, key },
                );
            }
            match &existing.functor {
                Some(existing_functor) if existing_functor == &functor => {}
                Some(existing_functor) => {
                    return Err(TermAndFormulaLoweringError::GeneratedFunctorMismatch {
                        key,
                        expected: Box::new(existing_functor.clone()),
                        actual: Box::new(functor),
                    });
                }
                None => {
                    return Err(TermAndFormulaLoweringError::MissingGeneratedOriginFunctor {
                        origin,
                        key,
                    });
                }
            }
            let reuse_source = if self.initial_generated_keys.contains(&map_key) {
                GeneratedOriginReuseSource::ExistingRegistry
            } else {
                GeneratedOriginReuseSource::CurrentDelta
            };
            return Ok(GeneratedOriginDraft {
                origin,
                kind,
                key,
                source: normalized_source(source),
                reused_existing: true,
                reuse_source,
            });
        }

        let mut origin_evidence = evidence;
        origin_evidence.extend(provenance.as_slice().iter().cloned());
        origin_evidence.sort();
        origin_evidence.dedup();
        let origin = GeneratedOrigin {
            owner: self.owner,
            kind,
            key: key.clone(),
            functor: Some(functor),
            params,
            evidence: origin_evidence,
            source: normalized_source(source.clone()),
        };
        let origin_id = self.generated.insert(origin.clone());
        self.generated_delta.insert(origin);
        self.generated_by_key.insert(map_key, origin_id);
        self.new_generated_origins.push(origin_id);
        self.source_map
            .generated_sources
            .insert(origin_id, normalized_source(source.clone()));
        Ok(GeneratedOriginDraft {
            origin: origin_id,
            kind,
            key,
            source: normalized_source(source),
            reused_existing: false,
            reuse_source: GeneratedOriginReuseSource::NewDelta,
        })
    }

    fn push_generated_origin_use(
        &mut self,
        draft: GeneratedOriginDraft,
        term: CoreTermId,
        functor: SymbolId,
        args: Vec<CoreTermId>,
    ) {
        self.generated_origin_refs.push(GeneratedOriginUse {
            term,
            origin: draft.origin,
            kind: draft.kind,
            key: draft.key,
            functor,
            args,
            source: draft.source,
            reused_existing: draft.reused_existing,
            reuse_source: draft.reuse_source,
        });
    }

    fn insert_core_obligation(
        &mut self,
        input: &TermAndFormulaLoweringInput,
        seed: CoreObligationSeed,
    ) -> TermAndFormulaResult<ObligationSeedId> {
        if seed.status == ObligationSeedStatus::Active && seed.goal.is_none() {
            return Err(TermAndFormulaLoweringError::MissingActiveObligationGoal {
                kind: seed.kind,
            });
        }

        let goal = seed
            .goal
            .map(|goal| self.lower_formula_seed(input, goal))
            .transpose()?;
        let mut context = Vec::new();
        for formula in seed.context {
            context.push(self.lower_formula_seed(input, formula)?);
        }

        let mut core_refs = vec![CoreNodeRef::Item(self.owner)];
        if let Some(goal) = goal {
            core_refs.push(CoreNodeRef::Formula(goal));
        }
        for formula in &context {
            core_refs.push(CoreNodeRef::Formula(*formula));
        }

        let kind = seed.kind;
        let source = source_with_provenance(seed.source, &seed.provenance);
        let obligation = ObligationSeed {
            owner: self.owner,
            kind: kind.clone(),
            goal,
            context,
            local_path: seed.local_path,
            label: seed.label,
            semantic_origin: seed.semantic_origin,
            provenance: seed.provenance.as_slice().to_vec(),
            source: normalized_source(source.clone()),
            core_refs,
            status: seed.status,
            diagnostics: Vec::new(),
        };
        let id = self.obligation_seeds.insert(obligation);
        self.source_map
            .obligation_sources
            .insert(id, normalized_source(source));
        self.generated_obligations.push(LoweredGeneratedObligation {
            obligation: id,
            kind,
        });
        Ok(id)
    }

    fn lower_term_seed(
        &mut self,
        input: &TermAndFormulaLoweringInput,
        seed_id: CoreTermSeedId,
    ) -> TermAndFormulaResult<CoreTermId> {
        if let Some(term) = self.term_map.get(&seed_id).copied() {
            return Ok(term);
        }
        if !self.term_stack.insert(seed_id) {
            return Err(TermAndFormulaLoweringError::CyclicTermSeed { seed: seed_id });
        }

        let result = self.lower_term_seed_inner(input, seed_id);
        self.term_stack.remove(&seed_id);
        let term = result?;
        self.term_map.insert(seed_id, term);
        Ok(term)
    }

    fn lower_term_seed_inner(
        &mut self,
        input: &TermAndFormulaLoweringInput,
        seed_id: CoreTermSeedId,
    ) -> TermAndFormulaResult<CoreTermId> {
        let seed = input
            .terms
            .get(seed_id.index())
            .cloned()
            .ok_or(TermAndFormulaLoweringError::MissingTermSeed { seed: seed_id })?;
        let source = source_with_provenance(seed.source.clone(), &seed.provenance);

        match seed.kind {
            CoreTermSeedKind::Var(var) => self.insert_declared_var_term(var, source),
            CoreTermSeedKind::Const(symbol) => {
                Ok(self.insert_term(CoreTermKind::Const(symbol), source))
            }
            CoreTermSeedKind::Apply { functor, args } => {
                let args = self.lower_term_refs(input, args)?;
                Ok(self.insert_term(CoreTermKind::Apply { functor, args }, source))
            }
            CoreTermSeedKind::Select { selector, base } => {
                let base = self.lower_term_seed(input, base)?;
                Ok(self.insert_term(CoreTermKind::Select { selector, base }, source))
            }
            CoreTermSeedKind::Tuple(args) => {
                let args = self.lower_term_refs(input, args)?;
                Ok(self.insert_term(CoreTermKind::Tuple(args), source))
            }
            CoreTermSeedKind::SetEnum(args) => {
                let args = self.lower_term_refs(input, args)?;
                Ok(self.insert_term(CoreTermKind::SetEnum(args), source))
            }
            CoreTermSeedKind::Qua { base, explanation } => {
                let lowered = self.lower_term_seed(input, base)?;
                let reduct = explanation.reduct.clone();
                self.push_view_explanation(explanation);
                Ok(match reduct {
                    Some(reduct) => self.lower_reduct_view(lowered, &reduct, source),
                    None => lowered,
                })
            }
            CoreTermSeedKind::StableChoice {
                functor,
                origin_functor,
                key,
                params,
                args,
                evidence,
            } => {
                validate_generated_functor(&key, &origin_functor, &functor)?;
                let draft = self.ensure_generated_origin(GeneratedOriginRequest {
                    kind: GeneratedOriginKind::StableChoice,
                    key,
                    functor: origin_functor,
                    params,
                    evidence,
                    source: seed.source,
                    provenance: seed.provenance.clone(),
                })?;
                let args = self.lower_term_refs(input, args)?;
                let term = self.insert_term(
                    CoreTermKind::Apply {
                        functor: functor.clone(),
                        args: args.clone(),
                    },
                    source,
                );
                self.push_generated_origin_use(draft, term, functor, args);
                Ok(term)
            }
            CoreTermSeedKind::Fraenkel {
                functor,
                origin_functor,
                key,
                params,
                args,
                mut sethood_evidence,
                template_type_parameter_sethood,
                membership_obligation,
                missing_sethood_obligation,
            } => {
                validate_generated_functor(&key, &origin_functor, &functor)?;
                if let Some(evidence) = template_type_parameter_sethood {
                    if let Some(record_evidence) =
                        resolve_template_fraenkel_sethood_evidence(input, &evidence)?
                    {
                        sethood_evidence.extend(record_evidence);
                    } else {
                        sethood_evidence.clear();
                    }
                }
                if sethood_evidence.is_empty() {
                    let diagnostic_id = self.diagnostics.insert(diagnostic(
                        CoreDiagnosticClass::UnresolvedSemanticInput,
                        CoreDiagnosticSeverity::Error,
                        CoreDiagnosticRecovery::Partial,
                        "missing-fraenkel-sethood-evidence",
                        source.clone(),
                        Some(CoreNodeRef::Item(self.owner)),
                    ));
                    if let Some(mut obligation) = missing_sethood_obligation {
                        obligation.status = ObligationSeedStatus::Deferred;
                        self.insert_core_obligation(input, *obligation)?;
                    }
                    let term = self.insert_term(CoreTermKind::Error(diagnostic_id), source);
                    self.set_diagnostic_owner(diagnostic_id, CoreNodeRef::Term(term));
                    return Ok(term);
                }

                let draft = self.ensure_generated_origin(GeneratedOriginRequest {
                    kind: GeneratedOriginKind::FraenkelComprehension,
                    key,
                    functor: origin_functor,
                    params,
                    evidence: sethood_evidence,
                    source: seed.source,
                    provenance: seed.provenance.clone(),
                })?;
                match *membership_obligation {
                    FraenkelMembershipObligationSeed::New(obligation) => {
                        self.insert_core_obligation(input, obligation)?;
                    }
                    FraenkelMembershipObligationSeed::AlreadyCarried(already_carried) => {
                        self.already_carried_generated_obligations.push(
                            AlreadyCarriedGeneratedObligation {
                                origin: draft.origin,
                                kind: ObligationSeedKind::FraenkelMembershipAxiom,
                                source: source_with_provenance(
                                    already_carried.source,
                                    &already_carried.provenance,
                                ),
                                provenance: already_carried.provenance.as_slice().to_vec(),
                            },
                        );
                    }
                }
                let args = self.lower_term_refs(input, args)?;
                let term = self.insert_term(
                    CoreTermKind::Apply {
                        functor: functor.clone(),
                        args: args.clone(),
                    },
                    source,
                );
                self.push_generated_origin_use(draft, term, functor, args);
                Ok(term)
            }
            CoreTermSeedKind::Error(site) => {
                let diagnostic_id = self.insert_failed_site(site);
                let term = self.insert_term(CoreTermKind::Error(diagnostic_id), source);
                self.set_diagnostic_owner(diagnostic_id, CoreNodeRef::Term(term));
                Ok(term)
            }
        }
    }

    fn lower_term_refs(
        &mut self,
        input: &TermAndFormulaLoweringInput,
        refs: Vec<CoreTermSeedId>,
    ) -> TermAndFormulaResult<Vec<CoreTermId>> {
        refs.into_iter()
            .map(|seed| self.lower_term_seed(input, seed))
            .collect()
    }

    fn insert_declared_var_term(
        &mut self,
        var: CoreVarId,
        source: CoreSourceRef,
    ) -> TermAndFormulaResult<CoreTermId> {
        Ok(self.insert_term(CoreTermKind::Var(var), source))
    }

    fn lower_formula_seed(
        &mut self,
        input: &TermAndFormulaLoweringInput,
        seed_id: CoreFormulaSeedId,
    ) -> TermAndFormulaResult<CoreFormulaId> {
        if let Some(formula) = self.formula_map.get(&seed_id).copied() {
            return Ok(formula);
        }
        if !self.formula_stack.insert(seed_id) {
            return Err(TermAndFormulaLoweringError::CyclicFormulaSeed { seed: seed_id });
        }

        let result = self.lower_formula_seed_inner(input, seed_id);
        self.formula_stack.remove(&seed_id);
        let formula = result?;
        self.formula_map.insert(seed_id, formula);
        Ok(formula)
    }

    fn lower_formula_seed_inner(
        &mut self,
        input: &TermAndFormulaLoweringInput,
        seed_id: CoreFormulaSeedId,
    ) -> TermAndFormulaResult<CoreFormulaId> {
        let seed = input
            .formulas
            .get(seed_id.index())
            .cloned()
            .ok_or(TermAndFormulaLoweringError::MissingFormulaSeed { seed: seed_id })?;
        let source = source_with_provenance(seed.source.clone(), &seed.provenance);

        let kind = match seed.kind {
            CoreFormulaSeedKind::True => CoreFormulaKind::True,
            CoreFormulaSeedKind::False => CoreFormulaKind::False,
            CoreFormulaSeedKind::Atom { predicate, args } => CoreFormulaKind::Atom {
                predicate,
                args: self.lower_term_refs(input, args)?,
            },
            CoreFormulaSeedKind::Equals { left, right } => CoreFormulaKind::Equals {
                left: self.lower_term_seed(input, left)?,
                right: self.lower_term_seed(input, right)?,
            },
            CoreFormulaSeedKind::TypePred { subject, ty } => CoreFormulaKind::TypePred {
                subject: self.lower_term_seed(input, subject)?,
                ty,
            },
            CoreFormulaSeedKind::Not(inner) => {
                CoreFormulaKind::Not(self.lower_formula_seed(input, inner)?)
            }
            CoreFormulaSeedKind::And(children) => CoreFormulaKind::And(
                children
                    .into_iter()
                    .map(|child| self.lower_formula_seed(input, child))
                    .collect::<TermAndFormulaResult<Vec<_>>>()?,
            ),
            CoreFormulaSeedKind::Or(children) => CoreFormulaKind::Or(
                children
                    .into_iter()
                    .map(|child| self.lower_formula_seed(input, child))
                    .collect::<TermAndFormulaResult<Vec<_>>>()?,
            ),
            CoreFormulaSeedKind::Implies {
                premise,
                conclusion,
            } => CoreFormulaKind::Implies {
                premise: self.lower_formula_seed(input, premise)?,
                conclusion: self.lower_formula_seed(input, conclusion)?,
            },
            CoreFormulaSeedKind::Iff { left, right } => CoreFormulaKind::Iff {
                left: self.lower_formula_seed(input, left)?,
                right: self.lower_formula_seed(input, right)?,
            },
            CoreFormulaSeedKind::Forall { binders, body } => CoreFormulaKind::Forall {
                binders: self.lower_quantifier_binders(input, binders)?,
                body: self.lower_formula_seed(input, body)?,
            },
            CoreFormulaSeedKind::Exists { binders, body } => CoreFormulaKind::Exists {
                binders: self.lower_quantifier_binders(input, binders)?,
                body: self.lower_formula_seed(input, body)?,
            },
            CoreFormulaSeedKind::Error(site) => {
                let diagnostic_id = self.insert_failed_site(site);
                let formula = self.insert_formula(CoreFormulaKind::Error(diagnostic_id), source);
                self.set_diagnostic_owner(diagnostic_id, CoreNodeRef::Formula(formula));
                return Ok(formula);
            }
        };

        Ok(self.insert_formula(kind, source))
    }

    fn lower_quantifier_binders(
        &mut self,
        input: &TermAndFormulaLoweringInput,
        binders: Vec<QuantifierBinderSeed>,
    ) -> TermAndFormulaResult<Vec<CoreBinder>> {
        let mut lowered = Vec::new();
        for binder in binders {
            let guard = binder
                .guard
                .map(|guard| self.lower_formula_seed(input, guard))
                .transpose()?;
            lowered.push(CoreBinder {
                var: binder.var,
                role: binder.role,
                ty_guard: guard,
                source_name: binder.source_name,
                source: source_with_provenance(binder.source, &binder.provenance),
            });
        }
        Ok(lowered)
    }

    fn push_view_explanation(&mut self, mut seed: ViewExplanationSeed) {
        seed.evidence_facts.sort();
        seed.evidence_facts.dedup();
        self.view_explanations.push(ViewExplanation {
            kind: seed.kind,
            inserted_view: seed.inserted_view,
            target_type: seed.target_type,
            reduct: seed.reduct.map(ReductView::from),
            evidence_facts: seed.evidence_facts,
            source: source_with_provenance(seed.source, &seed.provenance),
            provenance: seed.provenance.as_slice().to_vec(),
        });
    }

    fn lower_reduct_view(
        &mut self,
        mut current: CoreTermId,
        reduct: &ReductViewSeed,
        source: CoreSourceRef,
    ) -> CoreTermId {
        for functor in &reduct.functors {
            current = self.insert_term(
                CoreTermKind::Apply {
                    functor: functor.clone(),
                    args: vec![current],
                },
                source.clone(),
            );
        }
        current
    }
}

pub fn lower_term_and_formula_inputs(
    context: &CoreContext,
    input: TermAndFormulaLoweringInput,
) -> TermAndFormulaResult<TermAndFormulaLoweringOutput> {
    if context.item_registry().items().get(input.owner).is_none() {
        return Err(TermAndFormulaLoweringError::MissingOwnerItem { owner: input.owner });
    }
    validate_term_and_formula_input(context, &input)?;

    let mut state = TermAndFormulaLoweringState::new(context, input.owner);
    for site in input.failed_sites.iter().cloned() {
        let diagnostic_id = state.insert_failed_site(site);
        state.failed_sites.push(diagnostic_id);
    }
    for index in 0..input.terms.len() {
        state.lower_term_seed(&input, CoreTermSeedId::new(index))?;
    }
    for index in 0..input.formulas.len() {
        state.lower_formula_seed(&input, CoreFormulaSeedId::new(index))?;
    }

    Ok(TermAndFormulaLoweringOutput {
        terms: state.terms,
        formulas: state.formulas,
        generated: state.generated,
        generated_delta: state.generated_delta,
        obligation_seeds: state.obligation_seeds,
        source_map: state.source_map,
        diagnostics: state.diagnostics,
        term_map: state.term_map,
        formula_map: state.formula_map,
        new_generated_origins: state.new_generated_origins,
        generated_origin_refs: state.generated_origin_refs,
        view_explanations: state.view_explanations,
        generated_obligations: state.generated_obligations,
        already_carried_generated_obligations: state.already_carried_generated_obligations,
        failed_sites: state.failed_sites,
    })
}

fn validate_template_type_parameter_sethood_record(
    record: &TemplateTypeParameterSethood,
) -> TermAndFormulaResult<()> {
    validate_checker_owned_provenance("template type parameter sethood record", &record.provenance)
        .map_err(TermAndFormulaLoweringError::InvalidSeedProvenance)?;
    match record.status {
        TemplateTypeParameterSethoodStatus::Accepted => {
            if record.source_kind == TemplateTypeParameterSethoodSource::BareParameter {
                return Err(
                    TermAndFormulaLoweringError::InvalidTemplateFraenkelSethoodEvidence {
                        parameter: record.parameter.clone(),
                        evidence_key: record.evidence_key.clone(),
                        reason: TemplateSethoodRecordErrorKind::WrongSource,
                    },
                );
            }
            if record.facts.is_empty() {
                return Err(
                    TermAndFormulaLoweringError::InvalidTemplateFraenkelSethoodEvidence {
                        parameter: record.parameter.clone(),
                        evidence_key: record.evidence_key.clone(),
                        reason: TemplateSethoodRecordErrorKind::MissingEvidence,
                    },
                );
            }
        }
        TemplateTypeParameterSethoodStatus::Missing => {
            if record.source_kind != TemplateTypeParameterSethoodSource::BareParameter {
                return Err(
                    TermAndFormulaLoweringError::InvalidTemplateFraenkelSethoodEvidence {
                        parameter: record.parameter.clone(),
                        evidence_key: record.evidence_key.clone(),
                        reason: TemplateSethoodRecordErrorKind::WrongSource,
                    },
                );
            }
            if !record.facts.is_empty() {
                return Err(
                    TermAndFormulaLoweringError::InvalidTemplateFraenkelSethoodEvidence {
                        parameter: record.parameter.clone(),
                        evidence_key: record.evidence_key.clone(),
                        reason: TemplateSethoodRecordErrorKind::UnexpectedBareEvidence,
                    },
                );
            }
        }
        TemplateTypeParameterSethoodStatus::DegradedRecovery => {
            if !record.facts.is_empty() {
                return Err(
                    TermAndFormulaLoweringError::InvalidTemplateFraenkelSethoodEvidence {
                        parameter: record.parameter.clone(),
                        evidence_key: record.evidence_key.clone(),
                        reason: TemplateSethoodRecordErrorKind::UnexpectedBareEvidence,
                    },
                );
            }
        }
    }
    Ok(())
}

fn resolve_template_fraenkel_sethood_evidence(
    input: &TermAndFormulaLoweringInput,
    evidence: &TemplateFraenkelSethoodEvidenceSeed,
) -> TermAndFormulaResult<Option<Vec<CoreProvenance>>> {
    validate_checker_owned_provenance(
        "template Fraenkel sethood evidence seed",
        evidence.provenance.as_slice(),
    )
    .map_err(TermAndFormulaLoweringError::InvalidSeedProvenance)?;
    let record = find_template_type_parameter_sethood_record(input, evidence)?;
    if record.normalized_type != evidence.normalized_type {
        return Err(
            TermAndFormulaLoweringError::InvalidTemplateFraenkelSethoodEvidence {
                parameter: evidence.parameter.clone(),
                evidence_key: evidence.evidence_key.clone(),
                reason: TemplateSethoodRecordErrorKind::NormalizedTypeMismatch,
            },
        );
    }
    match record.status {
        TemplateTypeParameterSethoodStatus::Accepted => {
            if record.source_kind == TemplateTypeParameterSethoodSource::BareParameter {
                return Err(
                    TermAndFormulaLoweringError::InvalidTemplateFraenkelSethoodEvidence {
                        parameter: evidence.parameter.clone(),
                        evidence_key: evidence.evidence_key.clone(),
                        reason: TemplateSethoodRecordErrorKind::WrongSource,
                    },
                );
            }
            let mut provenance = record.provenance.clone();
            provenance.extend(evidence.provenance.as_slice().iter().cloned());
            provenance.sort();
            provenance.dedup();
            Ok(Some(provenance))
        }
        TemplateTypeParameterSethoodStatus::Missing
            if record.source_kind == TemplateTypeParameterSethoodSource::BareParameter =>
        {
            Ok(None)
        }
        _ => Err(
            TermAndFormulaLoweringError::InvalidTemplateFraenkelSethoodEvidence {
                parameter: evidence.parameter.clone(),
                evidence_key: evidence.evidence_key.clone(),
                reason: TemplateSethoodRecordErrorKind::NotAccepted,
            },
        ),
    }
}

fn validate_template_fraenkel_sethood_evidence(
    input: &TermAndFormulaLoweringInput,
    evidence: &TemplateFraenkelSethoodEvidenceSeed,
) -> TermAndFormulaResult<()> {
    resolve_template_fraenkel_sethood_evidence(input, evidence).map(|_| ())
}

fn find_template_type_parameter_sethood_record<'a>(
    input: &'a TermAndFormulaLoweringInput,
    evidence: &TemplateFraenkelSethoodEvidenceSeed,
) -> TermAndFormulaResult<&'a TemplateTypeParameterSethood> {
    input
        .template_type_parameter_sethoods
        .iter()
        .find(|record| {
            record.parameter == evidence.parameter && record.evidence_key == evidence.evidence_key
        })
        .ok_or_else(
            || TermAndFormulaLoweringError::InvalidTemplateFraenkelSethoodEvidence {
                parameter: evidence.parameter.clone(),
                evidence_key: evidence.evidence_key.clone(),
                reason: TemplateSethoodRecordErrorKind::Missing,
            },
        )
}

fn validate_term_and_formula_input(
    context: &CoreContext,
    input: &TermAndFormulaLoweringInput,
) -> TermAndFormulaResult<()> {
    let mut template_sethood_records = BTreeSet::new();
    for record in &input.template_type_parameter_sethoods {
        let key = (record.parameter.clone(), record.evidence_key.clone());
        if !template_sethood_records.insert(key) {
            return Err(
                TermAndFormulaLoweringError::InvalidTemplateFraenkelSethoodEvidence {
                    parameter: record.parameter.clone(),
                    evidence_key: record.evidence_key.clone(),
                    reason: TemplateSethoodRecordErrorKind::Duplicate,
                },
            );
        }
        validate_template_type_parameter_sethood_record(record)?;
    }
    for seed in &input.terms {
        validate_checker_owned_provenance("term seed", seed.provenance.as_slice())?;
        validate_term_seed_kind(context, input, &seed.kind)?;
    }
    for seed in &input.formulas {
        validate_checker_owned_provenance("formula seed", seed.provenance.as_slice())?;
        validate_formula_seed_kind(context, input, &seed.kind)?;
    }
    for site in &input.failed_sites {
        validate_checker_owned_provenance("failed semantic site", site.provenance.as_slice())?;
    }
    Ok(())
}

fn validate_term_seed_kind(
    context: &CoreContext,
    input: &TermAndFormulaLoweringInput,
    kind: &CoreTermSeedKind,
) -> TermAndFormulaResult<()> {
    match kind {
        CoreTermSeedKind::Var(var) => ensure_declared_term_variable(context, *var),
        CoreTermSeedKind::StableChoice {
            params, evidence, ..
        } => {
            validate_generated_params(context, params)?;
            if !evidence.is_empty() {
                validate_checker_owned_provenance("stable choice evidence", evidence)?;
            }
            Ok(())
        }
        CoreTermSeedKind::Fraenkel {
            params,
            sethood_evidence,
            template_type_parameter_sethood,
            membership_obligation,
            missing_sethood_obligation,
            ..
        } => {
            validate_generated_params(context, params)?;
            if !sethood_evidence.is_empty() {
                validate_checker_owned_provenance("fraenkel sethood evidence", sethood_evidence)?;
            }
            match membership_obligation.as_ref() {
                FraenkelMembershipObligationSeed::New(obligation) => {
                    validate_fraenkel_membership_obligation(obligation)?;
                }
                FraenkelMembershipObligationSeed::AlreadyCarried(already_carried) => {
                    validate_checker_owned_provenance(
                        "already carried fraenkel membership",
                        already_carried.provenance.as_slice(),
                    )?;
                }
            }
            if let Some(obligation) = missing_sethood_obligation.as_deref() {
                validate_fraenkel_missing_sethood_obligation(obligation)?;
            }
            if let Some(evidence) = template_type_parameter_sethood {
                validate_template_fraenkel_sethood_evidence(input, evidence)?;
            }
            Ok(())
        }
        CoreTermSeedKind::Qua { explanation, .. } => {
            validate_checker_owned_provenance(
                "qua view explanation",
                explanation.provenance.as_slice(),
            )?;
            if let Some(reduct) = &explanation.reduct {
                validate_term_reduct_view_seed(reduct)?;
            }
            Ok(())
        }
        CoreTermSeedKind::Error(site) => {
            validate_checker_owned_provenance("term error seed", site.provenance.as_slice())?;
            Ok(())
        }
        CoreTermSeedKind::Const(_)
        | CoreTermSeedKind::Apply { .. }
        | CoreTermSeedKind::Select { .. }
        | CoreTermSeedKind::Tuple(_)
        | CoreTermSeedKind::SetEnum(_) => Ok(()),
    }
}

fn validate_formula_seed_kind(
    context: &CoreContext,
    input: &TermAndFormulaLoweringInput,
    kind: &CoreFormulaSeedKind,
) -> TermAndFormulaResult<()> {
    match kind {
        CoreFormulaSeedKind::Forall { binders, .. }
        | CoreFormulaSeedKind::Exists { binders, .. } => {
            validate_quantifier_binder_seeds(context, input, binders)
        }
        CoreFormulaSeedKind::Error(site) => {
            validate_checker_owned_provenance("formula error seed", site.provenance.as_slice())?;
            Ok(())
        }
        CoreFormulaSeedKind::True
        | CoreFormulaSeedKind::False
        | CoreFormulaSeedKind::Atom { .. }
        | CoreFormulaSeedKind::Equals { .. }
        | CoreFormulaSeedKind::TypePred { .. }
        | CoreFormulaSeedKind::Not(_)
        | CoreFormulaSeedKind::And(_)
        | CoreFormulaSeedKind::Or(_)
        | CoreFormulaSeedKind::Implies { .. }
        | CoreFormulaSeedKind::Iff { .. } => Ok(()),
    }
}

fn validate_term_reduct_view_seed(reduct: &ReductViewSeed) -> TermAndFormulaResult<()> {
    if reduct.functors.is_empty() {
        return Err(TermAndFormulaLoweringError::EmptyReductViewPayload {
            path: reduct.path.clone(),
        });
    }
    Ok(())
}

fn validate_generated_params(
    context: &CoreContext,
    params: &[CoreVarId],
) -> TermAndFormulaResult<()> {
    for param in params {
        ensure_declared_term_variable(context, *param)?;
    }
    Ok(())
}

fn validate_generated_functor(
    key: &GeneratedOriginKey,
    expected: &SymbolId,
    actual: &SymbolId,
) -> TermAndFormulaResult<()> {
    if expected == actual {
        Ok(())
    } else {
        Err(TermAndFormulaLoweringError::GeneratedFunctorMismatch {
            key: key.clone(),
            expected: Box::new(expected.clone()),
            actual: Box::new(actual.clone()),
        })
    }
}

fn validate_quantifier_binder_seeds(
    context: &CoreContext,
    input: &TermAndFormulaLoweringInput,
    binders: &[QuantifierBinderSeed],
) -> TermAndFormulaResult<()> {
    for (index, binder) in binders.iter().enumerate() {
        validate_checker_owned_provenance("quantifier binder seed", binder.provenance.as_slice())?;
        ensure_declared_term_variable(context, binder.var)?;
        let later = binders
            .iter()
            .skip(index + 1)
            .map(|later| later.var)
            .collect::<BTreeSet<_>>();
        let mut mentions = binder
            .guard_mentions
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        if let Some(guard) = binder.guard {
            mentions.extend(seed_formula_free_variables(input, guard)?);
        }
        for mention in mentions {
            ensure_declared_term_variable(context, mention)?;
            if later.contains(&mention) {
                return Err(TermAndFormulaLoweringError::FutureBinderInGuard {
                    binder: binder.var,
                    later: mention,
                });
            }
        }
    }
    Ok(())
}

fn seed_term_free_variables(
    input: &TermAndFormulaLoweringInput,
    seed_id: CoreTermSeedId,
) -> TermAndFormulaResult<BTreeSet<CoreVarId>> {
    seed_term_free_variables_inner(input, seed_id, &mut BTreeSet::new())
}

fn seed_term_free_variables_inner(
    input: &TermAndFormulaLoweringInput,
    seed_id: CoreTermSeedId,
    stack: &mut BTreeSet<CoreTermSeedId>,
) -> TermAndFormulaResult<BTreeSet<CoreVarId>> {
    if !stack.insert(seed_id) {
        return Err(TermAndFormulaLoweringError::CyclicTermSeed { seed: seed_id });
    }
    let seed = input
        .terms
        .get(seed_id.index())
        .ok_or(TermAndFormulaLoweringError::MissingTermSeed { seed: seed_id })?;
    let mut vars = BTreeSet::new();
    match &seed.kind {
        CoreTermSeedKind::Var(var) => {
            vars.insert(*var);
        }
        CoreTermSeedKind::Const(_) | CoreTermSeedKind::Error(_) => {}
        CoreTermSeedKind::Apply { args, .. }
        | CoreTermSeedKind::Tuple(args)
        | CoreTermSeedKind::SetEnum(args) => {
            for arg in args {
                vars.extend(seed_term_free_variables_inner(input, *arg, stack)?);
            }
        }
        CoreTermSeedKind::Select { base, .. } | CoreTermSeedKind::Qua { base, .. } => {
            vars.extend(seed_term_free_variables_inner(input, *base, stack)?);
        }
        CoreTermSeedKind::StableChoice { params, args, .. }
        | CoreTermSeedKind::Fraenkel { params, args, .. } => {
            vars.extend(params.iter().copied());
            for arg in args {
                vars.extend(seed_term_free_variables_inner(input, *arg, stack)?);
            }
        }
    }
    stack.remove(&seed_id);
    Ok(vars)
}

fn seed_formula_free_variables(
    input: &TermAndFormulaLoweringInput,
    seed_id: CoreFormulaSeedId,
) -> TermAndFormulaResult<BTreeSet<CoreVarId>> {
    seed_formula_free_variables_inner(input, seed_id, &mut BTreeSet::new())
}

fn seed_formula_free_variables_inner(
    input: &TermAndFormulaLoweringInput,
    seed_id: CoreFormulaSeedId,
    stack: &mut BTreeSet<CoreFormulaSeedId>,
) -> TermAndFormulaResult<BTreeSet<CoreVarId>> {
    if !stack.insert(seed_id) {
        return Err(TermAndFormulaLoweringError::CyclicFormulaSeed { seed: seed_id });
    }
    let seed = input
        .formulas
        .get(seed_id.index())
        .ok_or(TermAndFormulaLoweringError::MissingFormulaSeed { seed: seed_id })?;
    let mut vars = BTreeSet::new();
    match &seed.kind {
        CoreFormulaSeedKind::True | CoreFormulaSeedKind::False | CoreFormulaSeedKind::Error(_) => {}
        CoreFormulaSeedKind::Atom { args, .. } => {
            for arg in args {
                vars.extend(seed_term_free_variables(input, *arg)?);
            }
        }
        CoreFormulaSeedKind::Equals { left, right } => {
            vars.extend(seed_term_free_variables(input, *left)?);
            vars.extend(seed_term_free_variables(input, *right)?);
        }
        CoreFormulaSeedKind::TypePred { subject, .. } => {
            vars.extend(seed_term_free_variables(input, *subject)?);
        }
        CoreFormulaSeedKind::Not(inner) => {
            vars.extend(seed_formula_free_variables_inner(input, *inner, stack)?);
        }
        CoreFormulaSeedKind::And(children) | CoreFormulaSeedKind::Or(children) => {
            for child in children {
                vars.extend(seed_formula_free_variables_inner(input, *child, stack)?);
            }
        }
        CoreFormulaSeedKind::Implies {
            premise,
            conclusion,
        } => {
            vars.extend(seed_formula_free_variables_inner(input, *premise, stack)?);
            vars.extend(seed_formula_free_variables_inner(
                input,
                *conclusion,
                stack,
            )?);
        }
        CoreFormulaSeedKind::Iff { left, right } => {
            vars.extend(seed_formula_free_variables_inner(input, *left, stack)?);
            vars.extend(seed_formula_free_variables_inner(input, *right, stack)?);
        }
        CoreFormulaSeedKind::Forall { binders, body }
        | CoreFormulaSeedKind::Exists { binders, body } => {
            for binder in binders {
                if let Some(guard) = binder.guard {
                    vars.extend(seed_formula_free_variables_inner(input, guard, stack)?);
                }
            }
            vars.extend(seed_formula_free_variables_inner(input, *body, stack)?);
            for binder in binders {
                vars.remove(&binder.var);
            }
        }
    }
    stack.remove(&seed_id);
    Ok(vars)
}

fn validate_core_obligation_seed(seed: &CoreObligationSeed) -> TermAndFormulaResult<()> {
    validate_checker_owned_provenance("core obligation seed", seed.provenance.as_slice())?;
    if seed.status == ObligationSeedStatus::Active && seed.goal.is_none() {
        return Err(TermAndFormulaLoweringError::MissingActiveObligationGoal {
            kind: seed.kind.clone(),
        });
    }
    Ok(())
}

fn validate_fraenkel_membership_obligation(seed: &CoreObligationSeed) -> TermAndFormulaResult<()> {
    validate_core_obligation_seed(seed)?;
    if seed.kind != ObligationSeedKind::FraenkelMembershipAxiom
        || seed.status != ObligationSeedStatus::Active
    {
        return Err(
            TermAndFormulaLoweringError::InvalidFraenkelMembershipObligation {
                kind: seed.kind.clone(),
                status: seed.status,
            },
        );
    }
    Ok(())
}

fn validate_fraenkel_missing_sethood_obligation(
    seed: &CoreObligationSeed,
) -> TermAndFormulaResult<()> {
    validate_core_obligation_seed(seed)?;
    if seed.kind != ObligationSeedKind::GeneratedSethood {
        return Err(
            TermAndFormulaLoweringError::InvalidFraenkelMissingSethoodObligation {
                kind: seed.kind.clone(),
            },
        );
    }
    Ok(())
}

fn ensure_declared_term_variable(
    context: &CoreContext,
    var: CoreVarId,
) -> TermAndFormulaResult<()> {
    match context.binder_context().variable_sorts.get(&var) {
        Some(NormalizedVarSort::Term) => Ok(()),
        Some(sort) => Err(TermAndFormulaLoweringError::NonTermVariable { var, sort: *sort }),
        None => Err(TermAndFormulaLoweringError::UndeclaredVariable { var }),
    }
}

fn source_with_provenance(
    source: CoreSourceRef,
    provenance: &CheckerOwnedProvenance,
) -> CoreSourceRef {
    let mut entries = source.provenance.clone();
    entries.extend(provenance.as_slice().iter().cloned());
    source.with_provenance(entries)
}

pub type DefinitionLoweringResult<T> = Result<T, DefinitionLoweringError>;

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DefinitionLoweringError {
    MissingOwnerItem {
        owner: CoreItemId,
    },
    DuplicateDefinitionOwner {
        owner: CoreItemId,
    },
    UndeclaredDefinitionParam {
        var: CoreVarId,
    },
    NonTermDefinitionParam {
        var: CoreVarId,
        sort: NormalizedVarSort,
    },
    MissingDefinitionBoundary {
        owner: CoreItemId,
    },
    DefinitionBoundaryNotPending {
        owner: CoreItemId,
        status: DefinitionBoundaryStatus,
    },
    DefinitionSymbolMismatch {
        owner: CoreItemId,
        expected: Box<SymbolId>,
        actual: Box<SymbolId>,
    },
    MissingTermBody {
        term: CoreTermId,
    },
    MissingFormulaBody {
        formula: CoreFormulaId,
    },
    MissingGeneratedDependency {
        origin: GeneratedOriginId,
    },
    SpuriousGeneratedDependency {
        origin: GeneratedOriginId,
    },
    MissingOtherwiseExcludes {
        branch: usize,
    },
    OtherwiseExcludesMismatch {
        branch: usize,
    },
    AlgorithmBodyDeferred,
    AlgorithmBoundaryRequiresDeferredBody {
        owner: CoreItemId,
    },
    InvalidCorrectnessObligation {
        kind: ObligationSeedKind,
        status: ObligationSeedStatus,
    },
    ExistingCorrectnessOwnerMismatch {
        obligation: ObligationSeedId,
        expected: CoreItemId,
        actual: CoreItemId,
    },
    MissingActiveCorrectnessGoal,
    InvalidSeedProvenance(CoreContextError),
}

impl fmt::Display for DefinitionLoweringError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingOwnerItem { owner } => {
                write!(formatter, "missing definition owner item {}", owner.index())
            }
            Self::DuplicateDefinitionOwner { owner } => {
                write!(
                    formatter,
                    "definition input contains duplicate owner item {}",
                    owner.index()
                )
            }
            Self::UndeclaredDefinitionParam { var } => {
                write!(formatter, "undeclared definition parameter {}", var.index())
            }
            Self::NonTermDefinitionParam { var, sort } => {
                write!(
                    formatter,
                    "definition parameter {} has non-term sort {sort:?}",
                    var.index()
                )
            }
            Self::MissingDefinitionBoundary { owner } => {
                write!(
                    formatter,
                    "missing pending definition boundary for item {}",
                    owner.index()
                )
            }
            Self::DefinitionBoundaryNotPending { owner, status } => {
                write!(
                    formatter,
                    "definition boundary for item {} has status {status:?}",
                    owner.index()
                )
            }
            Self::DefinitionSymbolMismatch {
                owner,
                expected,
                actual,
            } => {
                write!(
                    formatter,
                    "definition seed for item {} used symbol {actual:?}; expected {expected:?}",
                    owner.index()
                )
            }
            Self::MissingTermBody { term } => {
                write!(
                    formatter,
                    "definition references missing term {}",
                    term.index()
                )
            }
            Self::MissingFormulaBody { formula } => {
                write!(
                    formatter,
                    "definition references missing formula {}",
                    formula.index()
                )
            }
            Self::MissingGeneratedDependency { origin } => {
                write!(
                    formatter,
                    "definition references missing generated dependency {}",
                    origin.index()
                )
            }
            Self::SpuriousGeneratedDependency { origin } => {
                write!(
                    formatter,
                    "definition dependency {} is not reachable from generated term uses",
                    origin.index()
                )
            }
            Self::MissingOtherwiseExcludes { branch } => {
                write!(
                    formatter,
                    "otherwise branch {branch} has no excluded guards"
                )
            }
            Self::OtherwiseExcludesMismatch { branch } => {
                write!(
                    formatter,
                    "otherwise branch {branch} exclusions do not match prior guards"
                )
            }
            Self::AlgorithmBodyDeferred => {
                write!(
                    formatter,
                    "algorithm-backed definition body is deferred to Task 13"
                )
            }
            Self::AlgorithmBoundaryRequiresDeferredBody { owner } => {
                write!(
                    formatter,
                    "algorithm boundary item {} must use a deferred or unavailable body in Task 11",
                    owner.index()
                )
            }
            Self::InvalidCorrectnessObligation { kind, status } => {
                write!(
                    formatter,
                    "definition correctness obligation must be DefinitionCorrectness with a valid status, got {kind:?}/{status:?}"
                )
            }
            Self::ExistingCorrectnessOwnerMismatch {
                obligation,
                expected,
                actual,
            } => {
                write!(
                    formatter,
                    "existing definition correctness obligation {} is owned by item {}, expected item {}",
                    obligation.index(),
                    actual.index(),
                    expected.index()
                )
            }
            Self::MissingActiveCorrectnessGoal => {
                write!(
                    formatter,
                    "active definition correctness obligation needs a goal"
                )
            }
            Self::InvalidSeedProvenance(error) => write!(formatter, "{error}"),
        }
    }
}

impl Error for DefinitionLoweringError {}

impl From<CoreContextError> for DefinitionLoweringError {
    fn from(value: CoreContextError) -> Self {
        Self::InvalidSeedProvenance(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinitionLoweringInput {
    pub definitions: Vec<DefinitionSeed>,
}

impl DefinitionLoweringInput {
    pub const fn new() -> Self {
        Self {
            definitions: Vec::new(),
        }
    }
}

impl Default for DefinitionLoweringInput {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinitionSeed {
    pub owner: CoreItemId,
    pub symbol: SymbolId,
    pub params: Vec<CoreBinder>,
    pub body: DefinitionBodySeed,
    pub expansion: ExpansionPolicy,
    pub correctness: Vec<DefinitionCorrectnessSeed>,
    pub generated_dependencies: Vec<GeneratedOriginId>,
    pub source: CoreSourceRef,
    pub provenance: CheckerOwnedProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DefinitionBodySeed {
    Term(CoreTermId),
    Formula(CoreFormulaId),
    Guarded(Vec<GuardedDefinitionBranchSeed>),
    AlgorithmDeferred(FailedSemanticSiteSeed),
    Unavailable(FailedSemanticSiteSeed),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuardedDefinitionBranchSeed {
    pub guard: DefinitionGuardSeed,
    pub body: DefinitionBranchBody,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DefinitionGuardSeed {
    Explicit(CoreFormulaId),
    Otherwise {
        guard: CoreFormulaId,
        excludes: Vec<CoreFormulaId>,
        provenance: CheckerOwnedProvenance,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DefinitionCorrectnessSeed {
    New(Box<DefinitionObligationSeed>),
    Existing(ObligationSeedId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinitionObligationSeed {
    pub kind: ObligationSeedKind,
    pub status: ObligationSeedStatus,
    pub goal: Option<CoreFormulaId>,
    pub context: Vec<CoreFormulaId>,
    pub local_path: LocalProofOrProgramPath,
    pub label: Option<crate::core_ir::CoreLabelRef>,
    pub semantic_origin: NormalizedSemanticOrigin,
    pub source: CoreSourceRef,
    pub provenance: CheckerOwnedProvenance,
}

impl DefinitionObligationSeed {
    pub fn active(
        goal: CoreFormulaId,
        local_path: impl Into<LocalProofOrProgramPath>,
        semantic_origin: impl Into<NormalizedSemanticOrigin>,
        source: CoreSourceRef,
        provenance: CheckerOwnedProvenance,
    ) -> Self {
        Self {
            kind: ObligationSeedKind::DefinitionCorrectness,
            status: ObligationSeedStatus::Active,
            goal: Some(goal),
            context: Vec::new(),
            local_path: local_path.into(),
            label: None,
            semantic_origin: semantic_origin.into(),
            source,
            provenance,
        }
    }

    pub fn deferred(
        local_path: impl Into<LocalProofOrProgramPath>,
        semantic_origin: impl Into<NormalizedSemanticOrigin>,
        source: CoreSourceRef,
        provenance: CheckerOwnedProvenance,
    ) -> Self {
        Self {
            kind: ObligationSeedKind::DefinitionCorrectness,
            status: ObligationSeedStatus::Deferred,
            goal: None,
            context: Vec::new(),
            local_path: local_path.into(),
            label: None,
            semantic_origin: semantic_origin.into(),
            source,
            provenance,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinitionLoweringOutput {
    pub definitions: CoreDefinitionTable,
    pub obligation_seeds: ObligationSeedTable,
    pub source_map: CoreSourceMap,
    pub diagnostics: CoreDiagnosticTable,
    pub definition_map: BTreeMap<CoreItemId, CoreDefinitionId>,
    pub item_status_updates: Vec<DefinitionItemStatusUpdate>,
    pub correctness_obligations: Vec<DefinitionCorrectnessRecord>,
    pub generated_dependencies: Vec<DefinitionGeneratedDependencyRecord>,
    pub otherwise_guards: Vec<OtherwiseGuardRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinitionItemStatusUpdate {
    pub item: CoreItemId,
    pub status: CoreItemStatus,
    pub diagnostics: Vec<CoreDiagnosticId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinitionCorrectnessRecord {
    pub definition: CoreDefinitionId,
    pub obligation: ObligationSeedId,
    pub is_new: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinitionGeneratedDependencyRecord {
    pub definition: CoreDefinitionId,
    pub origin: GeneratedOriginId,
    pub use_terms: Vec<CoreTermId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OtherwiseGuardRecord {
    pub definition: CoreDefinitionId,
    pub branch_index: usize,
    pub guard: CoreFormulaId,
    pub excludes: Vec<CoreFormulaId>,
    pub provenance: Vec<CoreProvenance>,
}

#[derive(Debug, Clone)]
struct DefinitionLoweringState {
    definitions: CoreDefinitionTable,
    obligation_seeds: ObligationSeedTable,
    initial_obligation_len: usize,
    source_map: CoreSourceMap,
    diagnostics: CoreDiagnosticTable,
    definition_map: BTreeMap<CoreItemId, CoreDefinitionId>,
    item_status_updates: Vec<DefinitionItemStatusUpdate>,
    correctness_obligations: Vec<DefinitionCorrectnessRecord>,
    generated_dependencies: Vec<DefinitionGeneratedDependencyRecord>,
    otherwise_guards: Vec<OtherwiseGuardRecord>,
}

impl DefinitionLoweringState {
    fn new(context: &CoreContext, term_formula: &TermAndFormulaLoweringOutput) -> Self {
        let mut source_map = CoreSourceMap::new();
        source_map.item_sources = context.source_map().item_sources.clone();
        source_map.term_sources = term_formula.source_map.term_sources.clone();
        source_map.formula_sources = term_formula.source_map.formula_sources.clone();
        source_map.generated_sources = context.source_map().generated_sources.clone();
        source_map
            .generated_sources
            .extend(term_formula.source_map.generated_sources.clone());
        source_map.obligation_sources = term_formula.source_map.obligation_sources.clone();
        Self {
            definitions: CoreDefinitionTable::new(),
            obligation_seeds: term_formula.obligation_seeds.clone(),
            initial_obligation_len: term_formula.obligation_seeds.len(),
            source_map,
            diagnostics: term_formula.diagnostics.clone(),
            definition_map: BTreeMap::new(),
            item_status_updates: Vec::new(),
            correctness_obligations: Vec::new(),
            generated_dependencies: Vec::new(),
            otherwise_guards: Vec::new(),
        }
    }

    fn insert_definition(
        &mut self,
        seed: DefinitionSeed,
        body: DefinitionBody,
        correctness: Vec<ObligationSeedId>,
    ) -> CoreDefinitionId {
        let source = source_with_provenance(seed.source, &seed.provenance);
        let definition = CoreDefinition {
            owner: CoreDefinitionOwner::for_item(seed.owner),
            symbol: seed.symbol,
            params: seed.params,
            body,
            expansion: seed.expansion,
            correctness,
            generated_dependencies: seed.generated_dependencies,
            source: normalized_source(source.clone()),
        };
        let id = self.definitions.insert(definition);
        self.source_map
            .definition_sources
            .insert(id, normalized_source(source));
        self.definition_map.insert(seed.owner, id);
        id
    }

    fn insert_failed_site(
        &mut self,
        owner: CoreItemId,
        site: FailedSemanticSiteSeed,
    ) -> CoreDiagnosticId {
        self.diagnostics.insert(diagnostic(
            site.class,
            site.severity,
            site.recovery,
            site.message_key,
            source_with_provenance(site.source, &site.provenance),
            Some(CoreNodeRef::Item(owner)),
        ))
    }
}

pub fn lower_definition_inputs(
    context: &CoreContext,
    term_formula: &TermAndFormulaLoweringOutput,
    input: DefinitionLoweringInput,
) -> DefinitionLoweringResult<DefinitionLoweringOutput> {
    validate_definition_input(context, term_formula, &input)?;
    let mut state = DefinitionLoweringState::new(context, term_formula);

    for seed in input.definitions {
        let body_refs = definition_body_refs(&seed.body);
        let body = lower_definition_body(&mut state, seed.owner, &seed.body)?;
        let generated_records = validate_generated_dependencies(
            term_formula,
            &body_refs,
            &seed.generated_dependencies,
        )?;
        let correctness = insert_definition_correctness(
            &mut state,
            term_formula,
            seed.owner,
            &seed.correctness,
            &body_refs,
        )?;
        let definition_id = state.insert_definition(seed, body, correctness.clone());
        attach_definition_backrefs(
            &mut state,
            term_formula,
            definition_id,
            &correctness,
            &body_refs,
        );
        for record in generated_records {
            state
                .generated_dependencies
                .push(DefinitionGeneratedDependencyRecord {
                    definition: definition_id,
                    origin: record.origin,
                    use_terms: record.use_terms,
                });
        }
    }

    Ok(DefinitionLoweringOutput {
        definitions: state.definitions,
        obligation_seeds: state.obligation_seeds,
        source_map: state.source_map,
        diagnostics: state.diagnostics,
        definition_map: state.definition_map,
        item_status_updates: state.item_status_updates,
        correctness_obligations: state.correctness_obligations,
        generated_dependencies: state.generated_dependencies,
        otherwise_guards: state.otherwise_guards,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct DefinitionBodyRefs {
    terms: BTreeSet<CoreTermId>,
    formulas: BTreeSet<CoreFormulaId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DefinitionGeneratedDependencyUse {
    origin: GeneratedOriginId,
    use_terms: Vec<CoreTermId>,
}

fn validate_definition_input(
    context: &CoreContext,
    term_formula: &TermAndFormulaLoweringOutput,
    input: &DefinitionLoweringInput,
) -> DefinitionLoweringResult<()> {
    let mut seen_owners = BTreeSet::new();
    for seed in &input.definitions {
        if !seen_owners.insert(seed.owner) {
            return Err(DefinitionLoweringError::DuplicateDefinitionOwner { owner: seed.owner });
        }
        validate_checker_owned_provenance("definition seed", seed.provenance.as_slice())?;
        let item = context
            .item_registry()
            .items()
            .get(seed.owner)
            .ok_or(DefinitionLoweringError::MissingOwnerItem { owner: seed.owner })?;
        if item.symbol != seed.symbol {
            return Err(DefinitionLoweringError::DefinitionSymbolMismatch {
                owner: seed.owner,
                expected: Box::new(item.symbol.clone()),
                actual: Box::new(seed.symbol.clone()),
            });
        }
        let boundary = context
            .definition_boundaries()
            .get_by_item(seed.owner)
            .ok_or(DefinitionLoweringError::MissingDefinitionBoundary { owner: seed.owner })?;
        if boundary.status != DefinitionBoundaryStatus::PendingBody {
            return Err(DefinitionLoweringError::DefinitionBoundaryNotPending {
                owner: seed.owner,
                status: boundary.status,
            });
        }
        if boundary.kind == DefinitionBoundaryKind::Algorithm
            && !matches!(
                &seed.body,
                DefinitionBodySeed::AlgorithmDeferred(_) | DefinitionBodySeed::Unavailable(_)
            )
        {
            return Err(
                DefinitionLoweringError::AlgorithmBoundaryRequiresDeferredBody {
                    owner: seed.owner,
                },
            );
        }
        validate_definition_params(context, term_formula, &seed.params)?;
        validate_definition_body_seed(term_formula, &seed.body)?;
        validate_generated_dependencies(
            term_formula,
            &definition_body_refs(&seed.body),
            &seed.generated_dependencies,
        )?;
        for correctness in &seed.correctness {
            validate_definition_correctness_seed(term_formula, seed.owner, correctness)?;
        }
    }
    Ok(())
}

fn validate_definition_params(
    context: &CoreContext,
    term_formula: &TermAndFormulaLoweringOutput,
    params: &[CoreBinder],
) -> DefinitionLoweringResult<()> {
    for binder in params {
        match context.binder_context().variable_sorts.get(&binder.var) {
            Some(NormalizedVarSort::Term) => {}
            Some(sort) => {
                return Err(DefinitionLoweringError::NonTermDefinitionParam {
                    var: binder.var,
                    sort: *sort,
                });
            }
            None => {
                return Err(DefinitionLoweringError::UndeclaredDefinitionParam { var: binder.var });
            }
        }
        if let Some(guard) = binder.ty_guard {
            validate_definition_formula(term_formula, guard)?;
        }
    }
    Ok(())
}

fn validate_definition_body_seed(
    term_formula: &TermAndFormulaLoweringOutput,
    body: &DefinitionBodySeed,
) -> DefinitionLoweringResult<()> {
    match body {
        DefinitionBodySeed::Term(term) => validate_definition_term(term_formula, *term),
        DefinitionBodySeed::Formula(formula) => validate_definition_formula(term_formula, *formula),
        DefinitionBodySeed::Guarded(branches) => {
            let mut prior_guards = Vec::new();
            for (index, branch) in branches.iter().enumerate() {
                match &branch.guard {
                    DefinitionGuardSeed::Explicit(guard) => {
                        validate_definition_formula(term_formula, *guard)?;
                        prior_guards.push(*guard);
                    }
                    DefinitionGuardSeed::Otherwise {
                        guard,
                        excludes,
                        provenance,
                    } => {
                        validate_checker_owned_provenance(
                            "otherwise definition guard",
                            provenance.as_slice(),
                        )?;
                        validate_definition_formula(term_formula, *guard)?;
                        if excludes.is_empty() {
                            return Err(DefinitionLoweringError::MissingOtherwiseExcludes {
                                branch: index,
                            });
                        }
                        if excludes != &prior_guards {
                            return Err(DefinitionLoweringError::OtherwiseExcludesMismatch {
                                branch: index,
                            });
                        }
                        prior_guards.push(*guard);
                    }
                }
                validate_definition_branch_body(term_formula, &branch.body)?;
            }
            Ok(())
        }
        DefinitionBodySeed::AlgorithmDeferred(site) | DefinitionBodySeed::Unavailable(site) => {
            validate_checker_owned_provenance(
                "definition unavailable site",
                site.provenance.as_slice(),
            )?;
            Ok(())
        }
    }
}

fn validate_definition_branch_body(
    term_formula: &TermAndFormulaLoweringOutput,
    body: &DefinitionBranchBody,
) -> DefinitionLoweringResult<()> {
    match body {
        DefinitionBranchBody::Term(term) => validate_definition_term(term_formula, *term),
        DefinitionBranchBody::Formula(formula) => {
            validate_definition_formula(term_formula, *formula)
        }
    }
}

fn validate_definition_term(
    term_formula: &TermAndFormulaLoweringOutput,
    term: CoreTermId,
) -> DefinitionLoweringResult<()> {
    term_formula
        .terms
        .get(term)
        .map(|_| ())
        .ok_or(DefinitionLoweringError::MissingTermBody { term })
}

fn validate_definition_formula(
    term_formula: &TermAndFormulaLoweringOutput,
    formula: CoreFormulaId,
) -> DefinitionLoweringResult<()> {
    term_formula
        .formulas
        .get(formula)
        .map(|_| ())
        .ok_or(DefinitionLoweringError::MissingFormulaBody { formula })
}

fn validate_definition_correctness_seed(
    term_formula: &TermAndFormulaLoweringOutput,
    owner: CoreItemId,
    seed: &DefinitionCorrectnessSeed,
) -> DefinitionLoweringResult<()> {
    match seed {
        DefinitionCorrectnessSeed::New(seed) => {
            validate_checker_owned_provenance(
                "definition correctness seed",
                seed.provenance.as_slice(),
            )?;
            if seed.kind != ObligationSeedKind::DefinitionCorrectness {
                return Err(DefinitionLoweringError::InvalidCorrectnessObligation {
                    kind: seed.kind.clone(),
                    status: seed.status,
                });
            }
            match (seed.status, seed.goal) {
                (ObligationSeedStatus::Active, Some(goal)) => {
                    validate_definition_formula(term_formula, goal)?;
                }
                (ObligationSeedStatus::Active, None) => {
                    return Err(DefinitionLoweringError::MissingActiveCorrectnessGoal);
                }
                (_, Some(goal)) => {
                    validate_definition_formula(term_formula, goal)?;
                }
                (_, None) => {}
            }
            for formula in &seed.context {
                validate_definition_formula(term_formula, *formula)?;
            }
            Ok(())
        }
        DefinitionCorrectnessSeed::Existing(obligation) => {
            let seed = term_formula.obligation_seeds.get(*obligation).ok_or(
                DefinitionLoweringError::InvalidCorrectnessObligation {
                    kind: ObligationSeedKind::DefinitionCorrectness,
                    status: ObligationSeedStatus::Error,
                },
            )?;
            if seed.owner != owner {
                return Err(DefinitionLoweringError::ExistingCorrectnessOwnerMismatch {
                    obligation: *obligation,
                    expected: owner,
                    actual: seed.owner,
                });
            }
            if seed.kind != ObligationSeedKind::DefinitionCorrectness {
                return Err(DefinitionLoweringError::InvalidCorrectnessObligation {
                    kind: seed.kind.clone(),
                    status: seed.status,
                });
            }
            if seed.status == ObligationSeedStatus::Active && seed.goal.is_none() {
                return Err(DefinitionLoweringError::MissingActiveCorrectnessGoal);
            }
            if let Some(goal) = seed.goal {
                validate_definition_formula(term_formula, goal)?;
            }
            for formula in &seed.context {
                validate_definition_formula(term_formula, *formula)?;
            }
            Ok(())
        }
    }
}

fn collect_reachable_term_refs(
    term_formula: &TermAndFormulaLoweringOutput,
    term: CoreTermId,
    refs: &mut DefinitionBodyRefs,
) {
    if !refs.terms.insert(term) {
        return;
    }
    let Some(term_row) = term_formula.terms.get(term) else {
        return;
    };
    match &term_row.kind {
        CoreTermKind::Var(_) | CoreTermKind::Const(_) | CoreTermKind::Error(_) => {}
        CoreTermKind::Apply { args, .. }
        | CoreTermKind::Tuple(args)
        | CoreTermKind::SetEnum(args) => {
            for arg in args {
                collect_reachable_term_refs(term_formula, *arg, refs);
            }
        }
        CoreTermKind::Select { base, .. } => {
            collect_reachable_term_refs(term_formula, *base, refs);
        }
        CoreTermKind::Generated { args, .. } => {
            for arg in args {
                collect_reachable_term_refs(term_formula, *arg, refs);
            }
        }
    }
}

fn collect_reachable_formula_refs(
    term_formula: &TermAndFormulaLoweringOutput,
    formula: CoreFormulaId,
    refs: &mut DefinitionBodyRefs,
) {
    if !refs.formulas.insert(formula) {
        return;
    }
    let Some(formula_row) = term_formula.formulas.get(formula) else {
        return;
    };
    match &formula_row.kind {
        CoreFormulaKind::True | CoreFormulaKind::False | CoreFormulaKind::Error(_) => {}
        CoreFormulaKind::Atom { args, .. } => {
            for arg in args {
                collect_reachable_term_refs(term_formula, *arg, refs);
            }
        }
        CoreFormulaKind::Equals { left, right } => {
            collect_reachable_term_refs(term_formula, *left, refs);
            collect_reachable_term_refs(term_formula, *right, refs);
        }
        CoreFormulaKind::TypePred { subject, .. } => {
            collect_reachable_term_refs(term_formula, *subject, refs);
        }
        CoreFormulaKind::Not(child) => {
            collect_reachable_formula_refs(term_formula, *child, refs);
        }
        CoreFormulaKind::And(children) | CoreFormulaKind::Or(children) => {
            for child in children {
                collect_reachable_formula_refs(term_formula, *child, refs);
            }
        }
        CoreFormulaKind::Implies {
            premise,
            conclusion,
        } => {
            collect_reachable_formula_refs(term_formula, *premise, refs);
            collect_reachable_formula_refs(term_formula, *conclusion, refs);
        }
        CoreFormulaKind::Iff { left, right } => {
            collect_reachable_formula_refs(term_formula, *left, refs);
            collect_reachable_formula_refs(term_formula, *right, refs);
        }
        CoreFormulaKind::Forall { binders, body } | CoreFormulaKind::Exists { binders, body } => {
            for binder in binders {
                if let Some(guard) = binder.ty_guard {
                    collect_reachable_formula_refs(term_formula, guard, refs);
                }
            }
            collect_reachable_formula_refs(term_formula, *body, refs);
        }
    }
}

fn reachable_definition_body_refs(
    term_formula: &TermAndFormulaLoweringOutput,
    direct_refs: &DefinitionBodyRefs,
) -> DefinitionBodyRefs {
    let mut refs = DefinitionBodyRefs::default();
    for term in &direct_refs.terms {
        collect_reachable_term_refs(term_formula, *term, &mut refs);
    }
    for formula in &direct_refs.formulas {
        collect_reachable_formula_refs(term_formula, *formula, &mut refs);
    }
    refs
}

fn validate_generated_dependencies(
    term_formula: &TermAndFormulaLoweringOutput,
    body_refs: &DefinitionBodyRefs,
    dependencies: &[GeneratedOriginId],
) -> DefinitionLoweringResult<Vec<DefinitionGeneratedDependencyUse>> {
    let reachable_refs = reachable_definition_body_refs(term_formula, body_refs);
    let mut reachable: BTreeMap<GeneratedOriginId, Vec<CoreTermId>> = BTreeMap::new();
    for use_record in &term_formula.generated_origin_refs {
        if reachable_refs.terms.contains(&use_record.term) {
            reachable
                .entry(use_record.origin)
                .or_default()
                .push(use_record.term);
        }
    }
    for term in &reachable_refs.terms {
        if let Some(CoreTerm {
            kind: CoreTermKind::Generated { origin, .. },
            ..
        }) = term_formula.terms.get(*term)
        {
            reachable.entry(*origin).or_default().push(*term);
        }
    }

    let dependencies = dependencies.iter().copied().collect::<BTreeSet<_>>();
    for (origin, use_terms) in &reachable {
        term_formula
            .generated
            .get(*origin)
            .ok_or(DefinitionLoweringError::MissingGeneratedDependency { origin: *origin })?;
        if !dependencies.contains(origin) {
            return Err(DefinitionLoweringError::MissingGeneratedDependency { origin: *origin });
        }
        debug_assert!(!use_terms.is_empty());
    }

    let mut records = Vec::new();
    for dependency in dependencies {
        term_formula
            .generated
            .get(dependency)
            .ok_or(DefinitionLoweringError::MissingGeneratedDependency { origin: dependency })?;
        let Some(use_terms) = reachable.get(&dependency) else {
            return Err(DefinitionLoweringError::SpuriousGeneratedDependency {
                origin: dependency,
            });
        };
        let mut use_terms = use_terms.clone();
        use_terms.sort();
        use_terms.dedup();
        records.push(DefinitionGeneratedDependencyUse {
            origin: dependency,
            use_terms,
        });
    }
    Ok(records)
}

fn insert_definition_correctness(
    state: &mut DefinitionLoweringState,
    term_formula: &TermAndFormulaLoweringOutput,
    owner: CoreItemId,
    seeds: &[DefinitionCorrectnessSeed],
    body_refs: &DefinitionBodyRefs,
) -> DefinitionLoweringResult<Vec<ObligationSeedId>> {
    let mut lowered = Vec::new();
    let reachable_refs = reachable_definition_body_refs(term_formula, body_refs);
    for seed in seeds {
        match seed {
            DefinitionCorrectnessSeed::Existing(obligation) => {
                let existing = term_formula.obligation_seeds.get(*obligation).ok_or(
                    DefinitionLoweringError::InvalidCorrectnessObligation {
                        kind: ObligationSeedKind::DefinitionCorrectness,
                        status: ObligationSeedStatus::Error,
                    },
                )?;
                if existing.owner != owner {
                    return Err(DefinitionLoweringError::ExistingCorrectnessOwnerMismatch {
                        obligation: *obligation,
                        expected: owner,
                        actual: existing.owner,
                    });
                }
                if existing.kind != ObligationSeedKind::DefinitionCorrectness {
                    return Err(DefinitionLoweringError::InvalidCorrectnessObligation {
                        kind: existing.kind.clone(),
                        status: existing.status,
                    });
                }
                lowered.push(*obligation);
            }
            DefinitionCorrectnessSeed::New(seed) => {
                let source = source_with_provenance(seed.source.clone(), &seed.provenance);
                let mut provenance = seed.provenance.as_slice().to_vec();
                provenance.sort();
                provenance.dedup();
                let mut core_refs = vec![CoreNodeRef::Item(owner)];
                if let Some(goal) = seed.goal {
                    core_refs.push(CoreNodeRef::Formula(goal));
                }
                for formula in &seed.context {
                    core_refs.push(CoreNodeRef::Formula(*formula));
                }
                for term in &reachable_refs.terms {
                    core_refs.push(CoreNodeRef::Term(*term));
                }
                for formula in &reachable_refs.formulas {
                    core_refs.push(CoreNodeRef::Formula(*formula));
                }
                core_refs.sort();
                core_refs.dedup();
                let obligation = ObligationSeed {
                    owner,
                    kind: seed.kind.clone(),
                    goal: seed.goal,
                    context: seed.context.clone(),
                    local_path: seed.local_path.clone(),
                    label: seed.label.clone(),
                    semantic_origin: seed.semantic_origin.clone(),
                    provenance,
                    source: normalized_source(source.clone()),
                    core_refs,
                    status: seed.status,
                    diagnostics: Vec::new(),
                };
                let id = state.obligation_seeds.insert(obligation);
                state
                    .source_map
                    .obligation_sources
                    .insert(id, normalized_source(source));
                lowered.push(id);
            }
        }
    }
    Ok(lowered)
}

fn attach_definition_backrefs(
    state: &mut DefinitionLoweringState,
    term_formula: &TermAndFormulaLoweringOutput,
    definition: CoreDefinitionId,
    correctness: &[ObligationSeedId],
    body_refs: &DefinitionBodyRefs,
) {
    let reachable_refs = reachable_definition_body_refs(term_formula, body_refs);
    for obligation in correctness {
        if let Some(seed) = state.obligation_seeds.get_mut(*obligation) {
            seed.core_refs.push(CoreNodeRef::Definition(definition));
            for term in &reachable_refs.terms {
                seed.core_refs.push(CoreNodeRef::Term(*term));
            }
            for formula in &reachable_refs.formulas {
                seed.core_refs.push(CoreNodeRef::Formula(*formula));
            }
            seed.core_refs.sort();
            seed.core_refs.dedup();
        }
        state
            .correctness_obligations
            .push(DefinitionCorrectnessRecord {
                definition,
                obligation: *obligation,
                is_new: obligation.index() >= state.initial_obligation_len,
            });
    }
}

fn lower_definition_body(
    state: &mut DefinitionLoweringState,
    owner: CoreItemId,
    body: &DefinitionBodySeed,
) -> DefinitionLoweringResult<DefinitionBody> {
    match body {
        DefinitionBodySeed::Term(term) => Ok(DefinitionBody::Term(*term)),
        DefinitionBodySeed::Formula(formula) => Ok(DefinitionBody::Formula(*formula)),
        DefinitionBodySeed::Guarded(branches) => {
            let mut lowered = Vec::new();
            for (index, branch) in branches.iter().enumerate() {
                let guard = match &branch.guard {
                    DefinitionGuardSeed::Explicit(guard) => *guard,
                    DefinitionGuardSeed::Otherwise {
                        guard,
                        excludes,
                        provenance,
                    } => {
                        state.otherwise_guards.push(OtherwiseGuardRecord {
                            definition: CoreDefinitionId::new(state.definitions.len()),
                            branch_index: index,
                            guard: *guard,
                            excludes: excludes.clone(),
                            provenance: provenance.as_slice().to_vec(),
                        });
                        *guard
                    }
                };
                lowered.push(GuardedDefinitionBranch {
                    guard,
                    body: branch.body.clone(),
                });
            }
            Ok(DefinitionBody::Guarded(lowered))
        }
        DefinitionBodySeed::AlgorithmDeferred(site) | DefinitionBodySeed::Unavailable(site) => {
            let diagnostic_id = state.insert_failed_site(owner, site.clone());
            let status = match body {
                DefinitionBodySeed::AlgorithmDeferred(_) => CoreItemStatus::Skipped,
                DefinitionBodySeed::Unavailable(_) => CoreItemStatus::Error,
                _ => unreachable!("covered by outer match"),
            };
            state.item_status_updates.push(DefinitionItemStatusUpdate {
                item: owner,
                status,
                diagnostics: vec![diagnostic_id],
            });
            Ok(DefinitionBody::Unavailable(diagnostic_id))
        }
    }
}

fn definition_body_refs(body: &DefinitionBodySeed) -> DefinitionBodyRefs {
    let mut refs = DefinitionBodyRefs::default();
    match body {
        DefinitionBodySeed::Term(term) => {
            refs.terms.insert(*term);
        }
        DefinitionBodySeed::Formula(formula) => {
            refs.formulas.insert(*formula);
        }
        DefinitionBodySeed::Guarded(branches) => {
            for branch in branches {
                match &branch.guard {
                    DefinitionGuardSeed::Explicit(guard)
                    | DefinitionGuardSeed::Otherwise { guard, .. } => {
                        refs.formulas.insert(*guard);
                    }
                }
                match branch.body {
                    DefinitionBranchBody::Term(term) => {
                        refs.terms.insert(term);
                    }
                    DefinitionBranchBody::Formula(formula) => {
                        refs.formulas.insert(formula);
                    }
                }
            }
        }
        DefinitionBodySeed::AlgorithmDeferred(_) | DefinitionBodySeed::Unavailable(_) => {}
    }
    refs
}

pub type ProofLoweringResult<T> = Result<T, ProofLoweringError>;

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ProofLoweringError {
    MissingOwnerItem {
        owner: CoreItemId,
    },
    UnsupportedProofItemKind {
        owner: CoreItemId,
        kind: CoreItemKind,
    },
    ProofSymbolMismatch {
        owner: CoreItemId,
        expected: Box<SymbolId>,
        actual: Box<SymbolId>,
    },
    DuplicateProofOwner {
        owner: CoreItemId,
    },
    MissingProposition {
        proposition: CoreFormulaId,
    },
    MissingProofFormula {
        formula: CoreFormulaId,
    },
    UndeclaredIntroducedBinder {
        var: CoreVarId,
    },
    NonTermIntroducedBinder {
        var: CoreVarId,
        sort: NormalizedVarSort,
    },
    InvalidProofLabel {
        label: CoreLabelRef,
    },
    DuplicateProofLabel {
        label: CoreLabelRef,
    },
    UnknownProofLabel {
        label: CoreLabelRef,
    },
    InvalidSymbolCitation {
        symbol: Box<SymbolId>,
    },
    MissingGeneratedCitation {
        origin: GeneratedOriginId,
    },
    MalformedSkeletonRequiresErrorStatus {
        status: CoreProofStatus,
    },
    ErrorStatusRequiresMalformedSkeleton,
    AssumedProofCannotHaveTerminalGoals {
        owner: CoreItemId,
    },
    InvalidSeedProvenance(CoreContextError),
}

impl fmt::Display for ProofLoweringError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingOwnerItem { owner } => {
                write!(formatter, "missing proof owner item {}", owner.index())
            }
            Self::UnsupportedProofItemKind { owner, kind } => {
                write!(
                    formatter,
                    "proof owner item {} has unsupported kind {kind:?}",
                    owner.index()
                )
            }
            Self::ProofSymbolMismatch {
                owner,
                expected,
                actual,
            } => {
                write!(
                    formatter,
                    "proof seed for item {} used symbol {actual:?}; expected {expected:?}",
                    owner.index()
                )
            }
            Self::DuplicateProofOwner { owner } => {
                write!(
                    formatter,
                    "proof input contains duplicate owner item {}",
                    owner.index()
                )
            }
            Self::MissingProposition { proposition } => {
                write!(
                    formatter,
                    "proof proposition formula {} is missing",
                    proposition.index()
                )
            }
            Self::MissingProofFormula { formula } => {
                write!(
                    formatter,
                    "proof references missing formula {}",
                    formula.index()
                )
            }
            Self::UndeclaredIntroducedBinder { var } => {
                write!(
                    formatter,
                    "proof introduces undeclared binder {}",
                    var.index()
                )
            }
            Self::NonTermIntroducedBinder { var, sort } => {
                write!(
                    formatter,
                    "proof introduced binder {} has non-term sort {sort:?}",
                    var.index()
                )
            }
            Self::InvalidProofLabel { label } => {
                write!(formatter, "invalid empty proof label {}", label.as_str())
            }
            Self::DuplicateProofLabel { label } => {
                write!(formatter, "duplicate proof label {}", label.as_str())
            }
            Self::UnknownProofLabel { label } => {
                write!(formatter, "unknown proof label citation {}", label.as_str())
            }
            Self::InvalidSymbolCitation { symbol } => {
                write!(
                    formatter,
                    "proof citation references unknown symbol {symbol:?}"
                )
            }
            Self::MissingGeneratedCitation { origin } => {
                write!(
                    formatter,
                    "proof citation references missing generated origin {}",
                    origin.index()
                )
            }
            Self::MalformedSkeletonRequiresErrorStatus { status } => {
                write!(
                    formatter,
                    "malformed or missing proof skeleton requires Error status, got {status:?}"
                )
            }
            Self::ErrorStatusRequiresMalformedSkeleton => {
                write!(
                    formatter,
                    "Error proof status requires a malformed skeleton root"
                )
            }
            Self::AssumedProofCannotHaveTerminalGoals { owner } => {
                write!(
                    formatter,
                    "assumed proof for item {} cannot emit terminal proof goals",
                    owner.index()
                )
            }
            Self::InvalidSeedProvenance(error) => write!(formatter, "{error}"),
        }
    }
}

impl Error for ProofLoweringError {}

impl From<CoreContextError> for ProofLoweringError {
    fn from(value: CoreContextError) -> Self {
        Self::InvalidSeedProvenance(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofLoweringInput {
    pub proofs: Vec<ProofSeed>,
}

impl ProofLoweringInput {
    pub const fn new() -> Self {
        Self { proofs: Vec::new() }
    }
}

impl Default for ProofLoweringInput {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofSeed {
    pub owner: CoreItemId,
    pub symbol: SymbolId,
    pub proposition: CoreFormulaId,
    pub status: CoreProofStatus,
    pub skeleton: ProofSkeletonSeed,
    pub source: CoreSourceRef,
    pub provenance: CheckerOwnedProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ProofSkeletonSeed {
    Node(ProofNodeSeed),
    Missing(MalformedProofSkeletonSeed),
}

impl ProofSkeletonSeed {
    const fn is_malformed_root(&self) -> bool {
        matches!(self, Self::Missing(_) | Self::Node(ProofNodeSeed::Error(_)))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ProofNodeSeed {
    IntroduceBinder {
        binder: CoreBinder,
        child: Box<ProofNodeSeed>,
        source: CoreSourceRef,
        provenance: CheckerOwnedProvenance,
    },
    Assume {
        label: Option<CoreLabelRef>,
        formula: ProofFormulaRef,
        child: Box<ProofNodeSeed>,
        source: CoreSourceRef,
        provenance: CheckerOwnedProvenance,
    },
    Step {
        label: Option<CoreLabelRef>,
        formula: ProofFormulaRef,
        justification: ProofJustificationSeed,
        source: CoreSourceRef,
        provenance: CheckerOwnedProvenance,
    },
    CurrentGoal {
        thesis: ProofFormulaRef,
        child: Box<ProofNodeSeed>,
        source: CoreSourceRef,
        provenance: CheckerOwnedProvenance,
    },
    Sequence {
        children: Vec<ProofNodeSeed>,
        source: CoreSourceRef,
        provenance: CheckerOwnedProvenance,
    },
    Branch {
        kind: ProofBranchKind,
        children: Vec<ProofNodeSeed>,
        source: CoreSourceRef,
        provenance: CheckerOwnedProvenance,
    },
    TerminalGoal(ProofTerminalGoalSeed),
    Error(MalformedProofSkeletonSeed),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ProofFormulaRef {
    Formula(CoreFormulaId),
    Thesis,
}

impl From<CoreFormulaId> for ProofFormulaRef {
    fn from(value: CoreFormulaId) -> Self {
        Self::Formula(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofJustificationSeed {
    pub citations: Vec<CoreCitation>,
    pub source: CoreSourceRef,
    pub provenance: CheckerOwnedProvenance,
}

impl ProofJustificationSeed {
    pub fn new(
        citations: Vec<CoreCitation>,
        source: CoreSourceRef,
        provenance: CheckerOwnedProvenance,
    ) -> Self {
        Self {
            citations,
            source,
            provenance,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofTerminalGoalSeed {
    pub goal: ProofFormulaRef,
    pub context: Vec<CoreFormulaId>,
    pub citations: Vec<CoreCitation>,
    pub local_path: LocalProofOrProgramPath,
    pub label: Option<CoreLabelRef>,
    pub semantic_origin: NormalizedSemanticOrigin,
    pub source: CoreSourceRef,
    pub provenance: CheckerOwnedProvenance,
}

impl ProofTerminalGoalSeed {
    pub fn active(
        goal: impl Into<ProofFormulaRef>,
        local_path: impl Into<LocalProofOrProgramPath>,
        semantic_origin: impl Into<NormalizedSemanticOrigin>,
        source: CoreSourceRef,
        provenance: CheckerOwnedProvenance,
    ) -> Self {
        Self {
            goal: goal.into(),
            context: Vec::new(),
            citations: Vec::new(),
            local_path: local_path.into(),
            label: None,
            semantic_origin: semantic_origin.into(),
            source,
            provenance,
        }
    }

    pub fn with_context(mut self, context: Vec<CoreFormulaId>) -> Self {
        self.context = context;
        self
    }

    pub fn with_citations(mut self, citations: Vec<CoreCitation>) -> Self {
        self.citations = citations;
        self
    }

    pub fn with_label(mut self, label: CoreLabelRef) -> Self {
        self.label = Some(label);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MalformedProofSkeletonSeed {
    pub message_key: CoreDiagnosticMessageKey,
    pub source: CoreSourceRef,
    pub provenance: CheckerOwnedProvenance,
}

impl MalformedProofSkeletonSeed {
    pub fn error(
        message_key: impl Into<CoreDiagnosticMessageKey>,
        source: CoreSourceRef,
        provenance: CheckerOwnedProvenance,
    ) -> Self {
        Self {
            message_key: message_key.into(),
            source,
            provenance,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofLoweringOutput {
    pub proofs: CoreProofTable,
    pub proof_nodes: CoreProofNodeTable,
    pub obligation_seeds: ObligationSeedTable,
    pub source_map: CoreSourceMap,
    pub diagnostics: CoreDiagnosticTable,
    pub proof_map: BTreeMap<CoreItemId, CoreProofId>,
    pub proof_statuses: Vec<ProofStatusRecord>,
    pub terminal_obligations: Vec<ProofTerminalObligationRecord>,
    pub terminal_citations: Vec<ProofTerminalCitationRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofStatusRecord {
    pub proof: CoreProofId,
    pub item: CoreItemId,
    pub status: CoreProofStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofTerminalObligationRecord {
    pub proof: CoreProofId,
    pub node: CoreProofNodeId,
    pub obligation: ObligationSeedId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofTerminalCitationRecord {
    pub proof: CoreProofId,
    pub node: CoreProofNodeId,
    pub obligation: ObligationSeedId,
    pub citations: Vec<CoreCitation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingProofTerminalObligation {
    node: CoreProofNodeId,
    obligation: ObligationSeedId,
    citations: Vec<CoreCitation>,
}

#[derive(Debug, Clone)]
struct ProofLoweringState {
    proofs: CoreProofTable,
    proof_nodes: CoreProofNodeTable,
    obligation_seeds: ObligationSeedTable,
    initial_obligation_len: usize,
    source_map: CoreSourceMap,
    diagnostics: CoreDiagnosticTable,
    proof_map: BTreeMap<CoreItemId, CoreProofId>,
    proof_statuses: Vec<ProofStatusRecord>,
    terminal_obligations: Vec<ProofTerminalObligationRecord>,
    terminal_citations: Vec<ProofTerminalCitationRecord>,
}

impl ProofLoweringState {
    fn new(definitions: &DefinitionLoweringOutput) -> Self {
        Self {
            proofs: CoreProofTable::new(),
            proof_nodes: CoreProofNodeTable::new(),
            obligation_seeds: definitions.obligation_seeds.clone(),
            initial_obligation_len: definitions.obligation_seeds.len(),
            source_map: definitions.source_map.clone(),
            diagnostics: definitions.diagnostics.clone(),
            proof_map: BTreeMap::new(),
            proof_statuses: Vec::new(),
            terminal_obligations: Vec::new(),
            terminal_citations: Vec::new(),
        }
    }

    fn insert_proof_node(
        &mut self,
        kind: CoreProofNodeKind,
        source: CoreSourceRef,
        diagnostics: Vec<CoreDiagnosticId>,
    ) -> CoreProofNodeId {
        let source = normalized_source(source);
        let id = self.proof_nodes.insert(CoreProofNode {
            kind,
            source: source.clone(),
            diagnostics,
        });
        self.source_map.proof_sources.insert(id, source);
        id
    }

    fn insert_malformed_error_node(&mut self, site: MalformedProofSkeletonSeed) -> CoreProofNodeId {
        let source = source_with_provenance(site.source, &site.provenance);
        let diagnostic_id = self.diagnostics.insert(diagnostic(
            CoreDiagnosticClass::MalformedProofSkeleton,
            CoreDiagnosticSeverity::Error,
            CoreDiagnosticRecovery::Fatal,
            site.message_key,
            source.clone(),
            None,
        ));
        let node = self.insert_proof_node(
            CoreProofNodeKind::Error(diagnostic_id),
            source,
            vec![diagnostic_id],
        );
        if let Some(diagnostic) = self.diagnostics.get_mut(diagnostic_id) {
            diagnostic.owner = Some(CoreNodeRef::ProofNode(node));
        }
        node
    }
}

#[derive(Debug, Clone, Default)]
struct ProofLabelScope {
    all_labels: BTreeSet<CoreLabelRef>,
}

#[derive(Debug, Clone, Copy)]
struct ProofLoweringEnv<'a> {
    context: &'a CoreContext,
    term_formula: &'a TermAndFormulaLoweringOutput,
    owner: CoreItemId,
    thesis: CoreFormulaId,
}

pub fn lower_proof_inputs(
    context: &CoreContext,
    term_formula: &TermAndFormulaLoweringOutput,
    definitions: &DefinitionLoweringOutput,
    input: ProofLoweringInput,
) -> ProofLoweringResult<ProofLoweringOutput> {
    validate_proof_input(context, term_formula, &input)?;
    let mut state = ProofLoweringState::new(definitions);

    for seed in input.proofs {
        let mut terminal_obligations = Vec::new();
        let root = match seed.skeleton {
            ProofSkeletonSeed::Missing(site) => state.insert_malformed_error_node(site),
            ProofSkeletonSeed::Node(node) => {
                let mut label_scope = ProofLabelScope::default();
                let env = ProofLoweringEnv {
                    context,
                    term_formula,
                    owner: seed.owner,
                    thesis: seed.proposition,
                };
                let mut path_labels = BTreeSet::new();
                let mut path_formulas = BTreeSet::new();
                lower_proof_node(
                    &mut state,
                    &env,
                    &mut label_scope,
                    &mut path_labels,
                    &mut path_formulas,
                    &mut terminal_obligations,
                    node,
                )?
            }
        };
        if seed.status == CoreProofStatus::Assumed && !terminal_obligations.is_empty() {
            return Err(ProofLoweringError::AssumedProofCannotHaveTerminalGoals {
                owner: seed.owner,
            });
        }

        let source = normalized_source(source_with_provenance(seed.source, &seed.provenance));
        let proof = CoreProof {
            item: seed.owner,
            proposition: seed.proposition,
            root,
            status: seed.status,
            source,
        };
        let proof_id = state.proofs.insert(proof);
        state.proof_map.insert(seed.owner, proof_id);
        state.proof_statuses.push(ProofStatusRecord {
            proof: proof_id,
            item: seed.owner,
            status: seed.status,
        });
        attach_proof_backrefs(&mut state, proof_id, &terminal_obligations);
    }

    Ok(ProofLoweringOutput {
        proofs: state.proofs,
        proof_nodes: state.proof_nodes,
        obligation_seeds: state.obligation_seeds,
        source_map: state.source_map,
        diagnostics: state.diagnostics,
        proof_map: state.proof_map,
        proof_statuses: state.proof_statuses,
        terminal_obligations: state.terminal_obligations,
        terminal_citations: state.terminal_citations,
    })
}

fn validate_proof_input(
    context: &CoreContext,
    term_formula: &TermAndFormulaLoweringOutput,
    input: &ProofLoweringInput,
) -> ProofLoweringResult<()> {
    let mut seen_owners = BTreeSet::new();
    for seed in &input.proofs {
        if !seen_owners.insert(seed.owner) {
            return Err(ProofLoweringError::DuplicateProofOwner { owner: seed.owner });
        }
        validate_checker_owned_provenance("proof seed", seed.provenance.as_slice())?;
        let item = context
            .item_registry()
            .items()
            .get(seed.owner)
            .ok_or(ProofLoweringError::MissingOwnerItem { owner: seed.owner })?;
        if item.symbol != seed.symbol {
            return Err(ProofLoweringError::ProofSymbolMismatch {
                owner: seed.owner,
                expected: Box::new(item.symbol.clone()),
                actual: Box::new(seed.symbol.clone()),
            });
        }
        if !matches!(item.kind, CoreItemKind::Theorem | CoreItemKind::Lemma) {
            return Err(ProofLoweringError::UnsupportedProofItemKind {
                owner: seed.owner,
                kind: item.kind.clone(),
            });
        }
        term_formula.formulas.get(seed.proposition).ok_or(
            ProofLoweringError::MissingProposition {
                proposition: seed.proposition,
            },
        )?;
        match (seed.status, seed.skeleton.is_malformed_root()) {
            (CoreProofStatus::Error, false) => {
                return Err(ProofLoweringError::ErrorStatusRequiresMalformedSkeleton);
            }
            (status, true) if status != CoreProofStatus::Error => {
                return Err(ProofLoweringError::MalformedSkeletonRequiresErrorStatus { status });
            }
            _ => {}
        }
        let mut labels = ProofLabelScope::default();
        if let ProofSkeletonSeed::Node(node) = &seed.skeleton {
            let mut path_labels = BTreeSet::new();
            let mut path_formulas = BTreeSet::new();
            validate_proof_node_seed(
                context,
                term_formula,
                seed.proposition,
                &mut labels,
                &mut path_labels,
                &mut path_formulas,
                node,
            )?;
        } else if let ProofSkeletonSeed::Missing(site) = &seed.skeleton {
            validate_checker_owned_provenance(
                "malformed proof skeleton",
                site.provenance.as_slice(),
            )?;
        }
    }
    Ok(())
}

fn validate_proof_node_seed(
    context: &CoreContext,
    term_formula: &TermAndFormulaLoweringOutput,
    current_thesis: CoreFormulaId,
    labels: &mut ProofLabelScope,
    path_labels: &mut BTreeSet<CoreLabelRef>,
    path_formulas: &mut BTreeSet<CoreFormulaId>,
    node: &ProofNodeSeed,
) -> ProofLoweringResult<()> {
    match node {
        ProofNodeSeed::IntroduceBinder {
            binder,
            child,
            provenance,
            ..
        } => {
            validate_checker_owned_provenance("proof introduced binder", provenance.as_slice())?;
            validate_proof_binder(context, term_formula, binder)?;
            let mut child_labels = path_labels.clone();
            let mut child_path_formulas = path_formulas.clone();
            validate_proof_node_seed(
                context,
                term_formula,
                current_thesis,
                labels,
                &mut child_labels,
                &mut child_path_formulas,
                child,
            )
        }
        ProofNodeSeed::Assume {
            label,
            formula,
            child,
            provenance,
            ..
        } => {
            validate_checker_owned_provenance("proof assumption", provenance.as_slice())?;
            let resolved = resolve_proof_formula(term_formula, current_thesis, *formula)?;
            let mut child_labels = path_labels.clone();
            let mut child_path_formulas = path_formulas.clone();
            if let Some(label) = label {
                introduce_proof_label(labels, &mut child_labels, label)?;
            }
            child_path_formulas.insert(resolved);
            validate_proof_node_seed(
                context,
                term_formula,
                current_thesis,
                labels,
                &mut child_labels,
                &mut child_path_formulas,
                child,
            )
        }
        ProofNodeSeed::Step {
            label,
            formula,
            justification,
            provenance,
            ..
        } => {
            validate_checker_owned_provenance("proof step", provenance.as_slice())?;
            resolve_proof_formula(term_formula, current_thesis, *formula)?;
            validate_proof_justification(context, term_formula, path_labels, justification)?;
            if let Some(label) = label {
                introduce_proof_label(labels, path_labels, label)?;
            }
            Ok(())
        }
        ProofNodeSeed::CurrentGoal {
            thesis,
            child,
            provenance,
            ..
        } => {
            validate_checker_owned_provenance("proof current goal", provenance.as_slice())?;
            let thesis = resolve_proof_formula(term_formula, current_thesis, *thesis)?;
            let mut child_labels = path_labels.clone();
            let mut child_path_formulas = path_formulas.clone();
            validate_proof_node_seed(
                context,
                term_formula,
                thesis,
                labels,
                &mut child_labels,
                &mut child_path_formulas,
                child,
            )
        }
        ProofNodeSeed::Sequence {
            children,
            provenance,
            ..
        } => {
            validate_checker_owned_provenance("proof sequence", provenance.as_slice())?;
            for child in children {
                validate_proof_node_seed(
                    context,
                    term_formula,
                    current_thesis,
                    labels,
                    path_labels,
                    path_formulas,
                    child,
                )?;
            }
            Ok(())
        }
        ProofNodeSeed::Branch {
            children,
            provenance,
            ..
        } => {
            validate_checker_owned_provenance("proof branch", provenance.as_slice())?;
            for child in children {
                let mut child_labels = path_labels.clone();
                let mut child_path_formulas = path_formulas.clone();
                validate_proof_node_seed(
                    context,
                    term_formula,
                    current_thesis,
                    labels,
                    &mut child_labels,
                    &mut child_path_formulas,
                    child,
                )?;
            }
            Ok(())
        }
        ProofNodeSeed::TerminalGoal(seed) => {
            validate_terminal_goal_seed(context, term_formula, current_thesis, path_labels, seed)
        }
        ProofNodeSeed::Error(site) => validate_checker_owned_provenance(
            "malformed proof skeleton",
            site.provenance.as_slice(),
        )
        .map_err(Into::into),
    }
}

fn validate_proof_binder(
    context: &CoreContext,
    term_formula: &TermAndFormulaLoweringOutput,
    binder: &CoreBinder,
) -> ProofLoweringResult<()> {
    match context.binder_context().variable_sorts.get(&binder.var) {
        Some(NormalizedVarSort::Term) => {}
        Some(sort) => {
            return Err(ProofLoweringError::NonTermIntroducedBinder {
                var: binder.var,
                sort: *sort,
            });
        }
        None => {
            return Err(ProofLoweringError::UndeclaredIntroducedBinder { var: binder.var });
        }
    }
    if let Some(guard) = binder.ty_guard {
        validate_proof_formula(term_formula, guard)?;
    }
    Ok(())
}

fn validate_terminal_goal_seed(
    context: &CoreContext,
    term_formula: &TermAndFormulaLoweringOutput,
    current_thesis: CoreFormulaId,
    path_labels: &BTreeSet<CoreLabelRef>,
    seed: &ProofTerminalGoalSeed,
) -> ProofLoweringResult<()> {
    validate_checker_owned_provenance("terminal proof goal", seed.provenance.as_slice())?;
    resolve_proof_formula(term_formula, current_thesis, seed.goal)?;
    for formula in &seed.context {
        validate_proof_formula(term_formula, *formula)?;
    }
    validate_proof_citations(context, term_formula, path_labels, &seed.citations)?;
    if let Some(label) = &seed.label {
        validate_proof_label(label)?;
    }
    Ok(())
}

fn validate_proof_justification(
    context: &CoreContext,
    term_formula: &TermAndFormulaLoweringOutput,
    path_labels: &BTreeSet<CoreLabelRef>,
    justification: &ProofJustificationSeed,
) -> ProofLoweringResult<()> {
    validate_checker_owned_provenance("proof justification", justification.provenance.as_slice())?;
    validate_proof_citations(context, term_formula, path_labels, &justification.citations)
}

fn validate_proof_citations(
    context: &CoreContext,
    term_formula: &TermAndFormulaLoweringOutput,
    path_labels: &BTreeSet<CoreLabelRef>,
    citations: &[CoreCitation],
) -> ProofLoweringResult<()> {
    for citation in citations {
        citation_core_refs(context, term_formula, path_labels, citation)?;
    }
    Ok(())
}

fn validate_proof_formula(
    term_formula: &TermAndFormulaLoweringOutput,
    formula: CoreFormulaId,
) -> ProofLoweringResult<()> {
    term_formula
        .formulas
        .get(formula)
        .map(|_| ())
        .ok_or(ProofLoweringError::MissingProofFormula { formula })
}

fn resolve_proof_formula(
    term_formula: &TermAndFormulaLoweringOutput,
    current_thesis: CoreFormulaId,
    formula: ProofFormulaRef,
) -> ProofLoweringResult<CoreFormulaId> {
    match formula {
        ProofFormulaRef::Formula(formula) => {
            validate_proof_formula(term_formula, formula)?;
            Ok(formula)
        }
        ProofFormulaRef::Thesis => {
            validate_proof_formula(term_formula, current_thesis)?;
            Ok(current_thesis)
        }
    }
}

fn validate_proof_label(label: &CoreLabelRef) -> ProofLoweringResult<()> {
    if label.as_str().is_empty() {
        Err(ProofLoweringError::InvalidProofLabel {
            label: label.clone(),
        })
    } else {
        Ok(())
    }
}

fn introduce_proof_label(
    labels: &mut ProofLabelScope,
    path_labels: &mut BTreeSet<CoreLabelRef>,
    label: &CoreLabelRef,
) -> ProofLoweringResult<()> {
    validate_proof_label(label)?;
    if !labels.all_labels.insert(label.clone()) {
        return Err(ProofLoweringError::DuplicateProofLabel {
            label: label.clone(),
        });
    }
    path_labels.insert(label.clone());
    Ok(())
}

fn citation_core_refs(
    context: &CoreContext,
    term_formula: &TermAndFormulaLoweringOutput,
    path_labels: &BTreeSet<CoreLabelRef>,
    citation: &CoreCitation,
) -> ProofLoweringResult<Vec<CoreNodeRef>> {
    match citation {
        CoreCitation::Label(label) => {
            validate_proof_label(label)?;
            if !path_labels.contains(label) {
                return Err(ProofLoweringError::UnknownProofLabel {
                    label: label.clone(),
                });
            }
            Ok(Vec::new())
        }
        CoreCitation::Symbol(symbol) => {
            if let Some(item) = context.item_registry().id_for_symbol(symbol) {
                let item_kind = &context
                    .item_registry()
                    .items()
                    .get(item)
                    .expect("registered symbol id must resolve to an item")
                    .kind;
                if proof_citation_kind_allowed(item_kind) {
                    Ok(vec![CoreNodeRef::Item(item)])
                } else {
                    Err(ProofLoweringError::InvalidSymbolCitation {
                        symbol: Box::new(symbol.clone()),
                    })
                }
            } else if let Some(summary) = context.dependency_summaries().get(symbol) {
                if proof_citation_kind_allowed(summary.kind()) {
                    Ok(Vec::new())
                } else {
                    Err(ProofLoweringError::InvalidSymbolCitation {
                        symbol: Box::new(symbol.clone()),
                    })
                }
            } else {
                Err(ProofLoweringError::InvalidSymbolCitation {
                    symbol: Box::new(symbol.clone()),
                })
            }
        }
        CoreCitation::Generated(origin) => term_formula
            .generated
            .get(*origin)
            .map(|_| vec![CoreNodeRef::Generated(*origin)])
            .ok_or(ProofLoweringError::MissingGeneratedCitation { origin: *origin }),
    }
}

fn proof_citation_kind_allowed(kind: &CoreItemKind) -> bool {
    matches!(
        kind,
        CoreItemKind::Theorem | CoreItemKind::Lemma | CoreItemKind::Scheme
    )
}

fn lower_proof_node(
    state: &mut ProofLoweringState,
    env: &ProofLoweringEnv<'_>,
    labels: &mut ProofLabelScope,
    path_labels: &mut BTreeSet<CoreLabelRef>,
    path_formulas: &mut BTreeSet<CoreFormulaId>,
    terminal_obligations: &mut Vec<PendingProofTerminalObligation>,
    node: ProofNodeSeed,
) -> ProofLoweringResult<CoreProofNodeId> {
    match node {
        ProofNodeSeed::IntroduceBinder {
            binder,
            child,
            source,
            provenance,
        } => {
            validate_proof_binder(env.context, env.term_formula, &binder)?;
            let mut child_labels = path_labels.clone();
            let mut child_path_formulas = path_formulas.clone();
            let child = lower_proof_node(
                state,
                env,
                labels,
                &mut child_labels,
                &mut child_path_formulas,
                terminal_obligations,
                *child,
            )?;
            Ok(state.insert_proof_node(
                CoreProofNodeKind::IntroduceBinder { binder, child },
                source_with_provenance(source, &provenance),
                Vec::new(),
            ))
        }
        ProofNodeSeed::Assume {
            label,
            formula,
            child,
            source,
            provenance,
        } => {
            let formula = resolve_proof_formula(env.term_formula, env.thesis, formula)?;
            let mut child_labels = path_labels.clone();
            let mut child_path_formulas = path_formulas.clone();
            if let Some(label) = &label {
                introduce_proof_label(labels, &mut child_labels, label)?;
            }
            child_path_formulas.insert(formula);
            let child = lower_proof_node(
                state,
                env,
                labels,
                &mut child_labels,
                &mut child_path_formulas,
                terminal_obligations,
                *child,
            )?;
            Ok(state.insert_proof_node(
                CoreProofNodeKind::Assume {
                    label,
                    formula,
                    child,
                },
                source_with_provenance(source, &provenance),
                Vec::new(),
            ))
        }
        ProofNodeSeed::Step {
            label,
            formula,
            justification,
            source,
            provenance,
        } => {
            let formula = resolve_proof_formula(env.term_formula, env.thesis, formula)?;
            validate_proof_justification(
                env.context,
                env.term_formula,
                path_labels,
                &justification,
            )?;
            if let Some(label) = &label {
                introduce_proof_label(labels, path_labels, label)?;
            }
            Ok(state.insert_proof_node(
                CoreProofNodeKind::Step {
                    label,
                    formula,
                    justification: CoreJustification {
                        citations: justification.citations,
                        source: normalized_source(source_with_provenance(
                            justification.source,
                            &justification.provenance,
                        )),
                    },
                },
                source_with_provenance(source, &provenance),
                Vec::new(),
            ))
        }
        ProofNodeSeed::CurrentGoal {
            thesis,
            child,
            source,
            provenance,
        } => {
            let thesis = resolve_proof_formula(env.term_formula, env.thesis, thesis)?;
            let child_env = ProofLoweringEnv { thesis, ..*env };
            let mut child_labels = path_labels.clone();
            let mut child_path_formulas = path_formulas.clone();
            let child = lower_proof_node(
                state,
                &child_env,
                labels,
                &mut child_labels,
                &mut child_path_formulas,
                terminal_obligations,
                *child,
            )?;
            Ok(state.insert_proof_node(
                CoreProofNodeKind::CurrentGoal { thesis, child },
                source_with_provenance(source, &provenance),
                Vec::new(),
            ))
        }
        ProofNodeSeed::Sequence {
            children,
            source,
            provenance,
        } => {
            let mut lowered = Vec::new();
            for child in children {
                lowered.push(lower_proof_node(
                    state,
                    env,
                    labels,
                    path_labels,
                    path_formulas,
                    terminal_obligations,
                    child,
                )?);
            }
            Ok(state.insert_proof_node(
                CoreProofNodeKind::Sequence { children: lowered },
                source_with_provenance(source, &provenance),
                Vec::new(),
            ))
        }
        ProofNodeSeed::Branch {
            kind,
            children,
            source,
            provenance,
        } => {
            let mut lowered = Vec::new();
            for child in children {
                let mut child_labels = path_labels.clone();
                let mut child_path_formulas = path_formulas.clone();
                lowered.push(lower_proof_node(
                    state,
                    env,
                    labels,
                    &mut child_labels,
                    &mut child_path_formulas,
                    terminal_obligations,
                    child,
                )?);
            }
            Ok(state.insert_proof_node(
                CoreProofNodeKind::Branch {
                    kind,
                    children: lowered,
                },
                source_with_provenance(source, &provenance),
                Vec::new(),
            ))
        }
        ProofNodeSeed::TerminalGoal(seed) => insert_terminal_goal(
            state,
            env,
            path_labels,
            path_formulas,
            seed,
            terminal_obligations,
        ),
        ProofNodeSeed::Error(site) => Ok(state.insert_malformed_error_node(site)),
    }
}

fn insert_terminal_goal(
    state: &mut ProofLoweringState,
    env: &ProofLoweringEnv<'_>,
    path_labels: &BTreeSet<CoreLabelRef>,
    path_formulas: &BTreeSet<CoreFormulaId>,
    seed: ProofTerminalGoalSeed,
    terminal_obligations: &mut Vec<PendingProofTerminalObligation>,
) -> ProofLoweringResult<CoreProofNodeId> {
    let goal = resolve_proof_formula(env.term_formula, env.thesis, seed.goal)?;
    let mut context_formulas = path_formulas.iter().copied().collect::<Vec<_>>();
    context_formulas.extend(seed.context);
    context_formulas.sort();
    context_formulas.dedup();
    for formula in &context_formulas {
        validate_proof_formula(env.term_formula, *formula)?;
    }
    let mut citation_refs = Vec::new();
    let citations = seed.citations;
    for citation in &citations {
        citation_refs.extend(citation_core_refs(
            env.context,
            env.term_formula,
            path_labels,
            citation,
        )?);
    }
    let source = source_with_provenance(seed.source.clone(), &seed.provenance);
    let mut provenance = seed.provenance.as_slice().to_vec();
    provenance.sort();
    provenance.dedup();
    let mut core_refs = vec![CoreNodeRef::Item(env.owner), CoreNodeRef::Formula(goal)];
    for formula in &context_formulas {
        core_refs.push(CoreNodeRef::Formula(*formula));
    }
    core_refs.extend(citation_refs);
    core_refs.sort();
    core_refs.dedup();
    let obligation = ObligationSeed {
        owner: env.owner,
        kind: ObligationSeedKind::TheoremProof,
        goal: Some(goal),
        context: context_formulas,
        local_path: seed.local_path,
        label: seed.label,
        semantic_origin: seed.semantic_origin,
        provenance,
        source: normalized_source(source.clone()),
        core_refs,
        status: ObligationSeedStatus::Active,
        diagnostics: Vec::new(),
    };
    let obligation_id = state.obligation_seeds.insert(obligation);
    state
        .source_map
        .obligation_sources
        .insert(obligation_id, normalized_source(source.clone()));
    let node = state.insert_proof_node(
        CoreProofNodeKind::TerminalGoal {
            obligation: obligation_id,
            citations: citations.clone(),
        },
        source,
        Vec::new(),
    );
    if let Some(obligation) = state.obligation_seeds.get_mut(obligation_id) {
        obligation.core_refs.push(CoreNodeRef::ProofNode(node));
        obligation.core_refs.sort();
        obligation.core_refs.dedup();
    }
    terminal_obligations.push(PendingProofTerminalObligation {
        node,
        obligation: obligation_id,
        citations,
    });
    Ok(node)
}

fn attach_proof_backrefs(
    state: &mut ProofLoweringState,
    proof: CoreProofId,
    terminal_obligations: &[PendingProofTerminalObligation],
) {
    debug_assert!(
        terminal_obligations
            .iter()
            .all(|pending| pending.obligation.index() >= state.initial_obligation_len)
    );
    for pending in terminal_obligations {
        if let Some(seed) = state.obligation_seeds.get_mut(pending.obligation) {
            seed.core_refs.push(CoreNodeRef::Proof(proof));
            seed.core_refs.push(CoreNodeRef::ProofNode(pending.node));
            seed.core_refs.sort();
            seed.core_refs.dedup();
        }
        state
            .terminal_obligations
            .push(ProofTerminalObligationRecord {
                proof,
                node: pending.node,
                obligation: pending.obligation,
            });
        state.terminal_citations.push(ProofTerminalCitationRecord {
            proof,
            node: pending.node,
            obligation: pending.obligation,
            citations: pending.citations.clone(),
        });
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExactTask180LoweringError {
    InvalidCheckerBundle { reason: String },
    GenericLowering { stage: &'static str, reason: String },
    ProvenanceEnrichment { reason: String },
    InvalidProjection { reason: String },
}

impl fmt::Display for ExactTask180LoweringError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCheckerBundle { reason } => {
                write!(formatter, "invalid exact Task-180 checker bundle: {reason}")
            }
            Self::GenericLowering { stage, reason } => {
                write!(
                    formatter,
                    "exact Task-180 generic {stage} lowering failed: {reason}"
                )
            }
            Self::ProvenanceEnrichment { reason } => {
                write!(
                    formatter,
                    "exact Task-180 provenance enrichment failed: {reason}"
                )
            }
            Self::InvalidProjection { reason } => {
                write!(
                    formatter,
                    "invalid exact Task-180 CoreIr projection: {reason}"
                )
            }
        }
    }
}

impl Error for ExactTask180LoweringError {}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExactTask180Draft {
    source_id: SourceId,
    module_id: ModuleId,
    owner: SymbolId,
    owner_range: SourceRange,
    owner_origin: SemanticOrigin,
    owner_node: TypedNodeId,
    formula_range: SourceRange,
    formula_site_node: TypedNodeId,
    formula_node: TypedNodeId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExactTask180FailureInjection {
    None,
    #[cfg(test)]
    Preflight,
    #[cfg(test)]
    GenericLowering,
    #[cfg(test)]
    ProvenanceEnrichment,
    #[cfg(test)]
    InvalidProjection,
}

pub fn lower_exact_task180_handoff(
    resolved: &ResolvedTypedAst,
) -> Result<CoreIr, ExactTask180LoweringError> {
    let draft = extract_exact_task180_draft(resolved)?;
    lower_exact_task180_draft(draft, ExactTask180FailureInjection::None)
}

fn extract_exact_task180_draft(
    resolved: &ResolvedTypedAst,
) -> Result<ExactTask180Draft, ExactTask180LoweringError> {
    let invalid = |reason: &str| ExactTask180LoweringError::InvalidCheckerBundle {
        reason: reason.to_owned(),
    };
    if resolved.nodes().len() != 3
        || resolved.nodes().root() != Some(ResolvedTypedNodeId::new(2))
        || resolved.checked_formulas().len() != 1
        || resolved.statement_semantics().len() != 1
        || resolved.checked_proofs().len() != 1
        || resolved.checked_proof_nodes().len() != 1
        || resolved.checked_terminal_goals().len() != 1
    {
        return Err(invalid("exact table or compact-tree cardinality mismatch"));
    }
    if !resolved.expr_metadata().is_empty()
        || !resolved.collection_candidates().is_empty()
        || !resolved.expanded_candidates().is_empty()
        || !resolved.template_expansions().is_empty()
        || !resolved.viable_candidates().is_empty()
        || !resolved.viability_decisions().is_empty()
        || !resolved.specificity_graphs().is_empty()
        || !resolved.resolved_overloads().is_empty()
        || !resolved.inserted_coercions().is_empty()
        || !resolved.cluster_facts().is_empty()
        || !resolved.diagnostics().is_empty()
    {
        return Err(invalid("unrelated checker payload must be empty"));
    }

    let formula = resolved
        .checked_formulas()
        .get(CheckedFormulaId::new(0))
        .ok_or_else(|| invalid("missing checked formula zero"))?;
    let statement = resolved
        .statement_semantics()
        .get(StatementSemanticId::new(0))
        .ok_or_else(|| invalid("missing statement semantic zero"))?;
    let proof = resolved
        .checked_proofs()
        .get(CheckedProofId::new(0))
        .ok_or_else(|| invalid("missing checked proof zero"))?;
    let proof_node = resolved
        .checked_proof_nodes()
        .get(CheckedProofNodeId::new(0))
        .ok_or_else(|| invalid("missing checked proof node zero"))?;
    let terminal = resolved
        .checked_terminal_goals()
        .get(CheckedTerminalGoalId::new(0))
        .ok_or_else(|| invalid("missing checked terminal goal zero"))?;
    let formula_site_node = exact_task180_formula_site_node(&formula.site)?;

    let compact_formula = resolved
        .nodes()
        .node(ResolvedTypedNodeId::new(0))
        .ok_or_else(|| invalid("missing compact formula node"))?;
    let compact_theorem = resolved
        .nodes()
        .node(ResolvedTypedNodeId::new(1))
        .ok_or_else(|| invalid("missing compact theorem node"))?;
    let compact_root = resolved
        .nodes()
        .node(ResolvedTypedNodeId::new(2))
        .ok_or_else(|| invalid("missing compact module root"))?;
    let role_is = |node: &mizar_checker::resolved_typed_ast::ResolvedTypedNode, expected: &str| {
        matches!(
            &node.kind,
            ResolvedTypedNodeKind::SourcePreserved { role } if role.as_str() == expected
        )
    };
    if compact_formula.id != ResolvedTypedNodeId::new(0)
        || compact_formula.typed_node != TypedNodeId::new(0)
        || !compact_formula.children.is_empty()
        || !role_is(compact_formula, "source.formula.contradiction")
        || compact_formula.source_range != formula.source_range
        || compact_formula.final_type.is_some()
        || compact_formula.metadata.is_some()
        || !compact_formula.diagnostics.is_empty()
        || compact_formula.recovery != ResolvedNodeRecovery::Normal
        || compact_theorem.id != ResolvedTypedNodeId::new(1)
        || compact_theorem.typed_node != TypedNodeId::new(1)
        || compact_theorem.children != [ResolvedTypedNodeId::new(0)]
        || !role_is(compact_theorem, "source.statement.theorem")
        || compact_theorem.source_range != statement.owner_range
        || compact_theorem.final_type.is_some()
        || compact_theorem.metadata.is_some()
        || !compact_theorem.diagnostics.is_empty()
        || compact_theorem.recovery != ResolvedNodeRecovery::Normal
        || compact_root.id != ResolvedTypedNodeId::new(2)
        || compact_root.typed_node != TypedNodeId::new(2)
        || compact_root.children != [ResolvedTypedNodeId::new(1)]
        || !role_is(compact_root, "source.module")
        || compact_root.final_type.is_some()
        || compact_root.metadata.is_some()
        || !compact_root.diagnostics.is_empty()
        || compact_root.recovery != ResolvedNodeRecovery::Normal
        || compact_root.source_range.source_id != resolved.source_id()
        || compact_root.source_range.start > statement.owner_range.start
        || compact_root.source_range.end < statement.owner_range.end
    {
        return Err(invalid("compact Task-266 source tree mismatch"));
    }

    if formula.id != CheckedFormulaId::new(0)
        || formula.context != BindingContextId::new(0)
        || formula.source_range.source_id != resolved.source_id()
        || formula.recovery != NodeRecoveryState::Normal
        || formula.kind != FormulaKind::Contradiction
        || !formula.terms.is_empty()
        || formula.asserted_type.is_some()
        || !formula.expected_types.is_empty()
        || formula.candidate_set.is_some()
        || !formula.facts.is_empty()
        || formula.status != FormulaStatus::Checked
        || !formula.deferred.is_empty()
    {
        return Err(invalid("checked contradiction formula mismatch"));
    }
    if statement.id != StatementSemanticId::new(0)
        || statement.owner.module() != resolved.module_id()
        || statement.owner_node != TypedNodeId::new(1)
        || statement.owner_range.source_id != resolved.source_id()
        || statement.formula != formula.id
        || statement.formula_node != TypedNodeId::new(0)
        || statement.owner_origin.source_id() != resolved.source_id()
        || statement.owner_origin.module_id() != resolved.module_id()
        || statement.owner_origin.anchor() != &SourceAnchor::Range(statement.owner_range)
        || statement.owner_origin.import_edge().is_some()
        || statement.owner_origin.is_recovered()
    {
        return Err(invalid("statement owner/formula identity mismatch"));
    }
    if proof.id != CheckedProofId::new(0)
        || proof.source_order != 0
        || proof.statement != statement.id
        || proof.owner != statement.owner
        || proof.owner_node != statement.owner_node
        || proof.owner_visibility != Visibility::Public
        || proof.owner_export_status != ExportStatus::Exported
        || proof.proposition != statement.formula
        || proof.policy != TheoremPolicyIntent::Unmodified
        || proof.justification != TheoremJustificationIntent::Omitted
        || proof.root != CheckedProofNodeId::new(0)
        || proof.status != CheckedProofStatus::PendingAutomaticProof
        || proof.source_range != statement.owner_range
        || proof.owner_origin != statement.owner_origin
    {
        return Err(invalid("checked proof identity or policy mismatch"));
    }
    if proof_node.id != CheckedProofNodeId::new(0)
        || proof_node.proof != proof.id
        || proof_node.kind != CheckedProofNodeKind::TerminalGoal(terminal.id)
        || proof_node.source_range != formula.source_range
        || proof_node.recovery != NodeRecoveryState::Normal
        || terminal.id != CheckedTerminalGoalId::new(0)
        || terminal.proof != proof.id
        || terminal.node != proof_node.id
        || terminal.statement != statement.id
        || terminal.owner != statement.owner
        || terminal.formula != statement.formula
        || terminal.formula_site != formula.site
        || !matches!(terminal.formula_site, TypedSiteRef::Node(_))
        || terminal.formula_node != statement.formula_node
        || terminal.source_range != formula.source_range
        || terminal.recovery != NodeRecoveryState::Normal
        || !terminal.citations.is_empty()
        || !terminal.active_context.is_empty()
        || terminal.local_path != "proof/0"
        || terminal.label.is_some()
    {
        return Err(invalid("direct checked terminal-goal identity mismatch"));
    }

    Ok(ExactTask180Draft {
        source_id: resolved.source_id(),
        module_id: resolved.module_id().clone(),
        owner: statement.owner.clone(),
        owner_range: statement.owner_range,
        owner_origin: statement.owner_origin.clone(),
        owner_node: statement.owner_node,
        formula_range: formula.source_range,
        formula_site_node,
        formula_node: statement.formula_node,
    })
}

fn exact_task180_formula_site_node(
    site: &TypedSiteRef,
) -> Result<TypedNodeId, ExactTask180LoweringError> {
    match site {
        TypedSiteRef::Node(node) => Ok(*node),
        _ => Err(ExactTask180LoweringError::InvalidCheckerBundle {
            reason: "checked formula site must be a real node site".to_owned(),
        }),
    }
}

fn lower_exact_task180_draft(
    draft: ExactTask180Draft,
    _injection: ExactTask180FailureInjection,
) -> Result<CoreIr, ExactTask180LoweringError> {
    #[cfg(test)]
    if _injection == ExactTask180FailureInjection::Preflight {
        return Err(ExactTask180LoweringError::InvalidCheckerBundle {
            reason: "injected preflight failure".to_owned(),
        });
    }

    let resolver_key = exact_task267_resolver_key(&draft.owner, &draft.owner_origin);
    let statement_key = exact_task267_statement_key(&draft);
    let proof_key = "task267/v1;proof=0;statement=0;policy=unmodified;justification=omitted;status=pending-automatic-proof".to_owned();
    let terminal_key = exact_task267_terminal_key(&draft);
    let skeleton_key = "task267/v1;local-path=7:proof/0".to_owned();

    #[cfg(test)]
    if _injection == ExactTask180FailureInjection::GenericLowering {
        return Err(ExactTask180LoweringError::GenericLowering {
            stage: "context",
            reason: "injected generic lowering failure".to_owned(),
        });
    }

    let item_provenance = CheckerOwnedProvenance::try_new(vec![
        CoreProvenance::new(CoreProvenancePhase::Resolver, resolver_key.clone()),
        CoreProvenance::new(CoreProvenancePhase::Checker, statement_key.clone()),
    ])
    .map_err(|error| exact_generic_error("context provenance", error))?;
    let item_source = CoreSourceRef::direct(draft.owner_range).with_provenance(vec![
        CoreProvenance::new(CoreProvenancePhase::Resolver, resolver_key.clone()),
        CoreProvenance::new(CoreProvenancePhase::Checker, statement_key.clone()),
    ]);
    let mut context_input = CoreContextInput::new(ResolvedTypedAstSummary::new(
        draft.source_id,
        draft.module_id.clone(),
    ));
    context_input.item_seeds.push(CoreItemSeed::new(
        draft.owner.clone(),
        CoreItemKind::Theorem,
        "public",
        item_source,
        item_provenance,
    ));
    let context = prepare_core_context(context_input)
        .map_err(|error| exact_generic_error("context", error))?;

    let owner = context
        .item_registry()
        .id_for_symbol(&draft.owner)
        .ok_or_else(|| ExactTask180LoweringError::GenericLowering {
            stage: "context",
            reason: "exact owner item was not registered".to_owned(),
        })?;
    if owner != CoreItemId::new(0) {
        return Err(ExactTask180LoweringError::GenericLowering {
            stage: "context",
            reason: "exact owner item did not receive dense id zero".to_owned(),
        });
    }

    let mut term_formula_input = TermAndFormulaLoweringInput::new(owner);
    term_formula_input.formulas.push(CoreFormulaSeed::new(
        CoreFormulaSeedKind::False,
        CoreSourceRef::direct(draft.formula_range),
        CheckerOwnedProvenance::checker(statement_key.clone()),
    ));
    let term_formula = lower_term_and_formula_inputs(&context, term_formula_input)
        .map_err(|error| exact_generic_error("term/formula", error))?;
    let definitions =
        lower_definition_inputs(&context, &term_formula, DefinitionLoweringInput::new())
            .map_err(|error| exact_generic_error("definition", error))?;

    let terminal_source =
        CoreSourceRef::direct(draft.formula_range).with_provenance(vec![CoreProvenance::new(
            CoreProvenancePhase::ProofSkeleton,
            skeleton_key.clone(),
        )]);
    let terminal = ProofTerminalGoalSeed::active(
        CoreFormulaId::new(0),
        "proof/0",
        draft.owner.fqn().as_str(),
        terminal_source,
        CheckerOwnedProvenance::checker(terminal_key.clone()),
    );
    let proof_provenance = CheckerOwnedProvenance::try_new(vec![
        CoreProvenance::new(CoreProvenancePhase::Resolver, resolver_key.clone()),
        CoreProvenance::new(CoreProvenancePhase::Checker, proof_key.clone()),
    ])
    .map_err(|error| exact_generic_error("proof provenance", error))?;
    let proof_input = ProofLoweringInput {
        proofs: vec![ProofSeed {
            owner,
            symbol: draft.owner.clone(),
            proposition: CoreFormulaId::new(0),
            status: CoreProofStatus::PendingAutomaticProof,
            skeleton: ProofSkeletonSeed::Node(ProofNodeSeed::TerminalGoal(terminal)),
            source: CoreSourceRef::direct(draft.owner_range),
            provenance: proof_provenance,
        }],
    };
    let mut proofs = lower_proof_inputs(&context, &term_formula, &definitions, proof_input)
        .map_err(|error| exact_generic_error("proof", error))?;

    #[cfg(test)]
    if _injection == ExactTask180FailureInjection::ProvenanceEnrichment {
        return Err(ExactTask180LoweringError::ProvenanceEnrichment {
            reason: "injected provenance enrichment failure".to_owned(),
        });
    }
    let obligation = proofs
        .obligation_seeds
        .get_mut(ObligationSeedId::new(0))
        .ok_or_else(|| ExactTask180LoweringError::ProvenanceEnrichment {
            reason: "missing sole theorem-proof obligation".to_owned(),
        })?;
    obligation.provenance.push(CoreProvenance::new(
        CoreProvenancePhase::ProofSkeleton,
        skeleton_key.clone(),
    ));
    obligation.provenance.sort();
    obligation.provenance.dedup();

    let parts = CoreIrParts {
        source_id: context.source_id(),
        module_id: context.module_id().clone(),
        items: context.item_registry().items().clone(),
        terms: term_formula.terms.clone(),
        formulas: term_formula.formulas.clone(),
        definitions: definitions.definitions.clone(),
        proofs: proofs.proofs.clone(),
        proof_nodes: proofs.proof_nodes.clone(),
        algorithms: CoreAlgorithmTable::new(),
        algorithm_statements: CoreAlgorithmStmtTable::new(),
        generated: term_formula.generated.clone(),
        obligation_seeds: proofs.obligation_seeds.clone(),
        source_map: proofs.source_map.clone(),
        diagnostics: proofs.diagnostics.clone(),
    };
    let core =
        CoreIr::try_new(parts).map_err(|error| ExactTask180LoweringError::InvalidProjection {
            reason: error.to_string(),
        })?;

    #[cfg(test)]
    if _injection == ExactTask180FailureInjection::InvalidProjection {
        return Err(ExactTask180LoweringError::InvalidProjection {
            reason: "injected final projection failure".to_owned(),
        });
    }
    validate_exact_task180_projection(
        &core,
        &draft,
        ExactTask267Keys {
            resolver: &resolver_key,
            statement: &statement_key,
            proof: &proof_key,
            terminal: &terminal_key,
            skeleton: &skeleton_key,
        },
    )?;
    Ok(core)
}

fn exact_generic_error(stage: &'static str, error: impl fmt::Display) -> ExactTask180LoweringError {
    ExactTask180LoweringError::GenericLowering {
        stage,
        reason: error.to_string(),
    }
}

fn exact_task267_resolver_key(owner: &SymbolId, origin: &SemanticOrigin) -> String {
    let fqn = owner.fqn().as_str();
    let path = origin
        .structural_path()
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "task267/v1;owner-fqn={}:{};origin-path={}:{}",
        fqn.len(),
        fqn,
        origin.structural_path().len(),
        path
    )
}

fn exact_task267_statement_key(draft: &ExactTask180Draft) -> String {
    format!(
        "task267/v1;statement=0;owner-node={};formula=0;formula-site-node={};formula-node={}",
        draft.owner_node.index(),
        draft.formula_site_node.index(),
        draft.formula_node.index()
    )
}

fn exact_task267_terminal_key(draft: &ExactTask180Draft) -> String {
    format!(
        "task267/v1;proof-node=0;terminal-goal=0;formula=0;formula-site-node={};formula-node={}",
        draft.formula_site_node.index(),
        draft.formula_node.index()
    )
}

struct ExactTask267Keys<'a> {
    resolver: &'a str,
    statement: &'a str,
    proof: &'a str,
    terminal: &'a str,
    skeleton: &'a str,
}

fn validate_exact_task180_projection(
    core: &CoreIr,
    draft: &ExactTask180Draft,
    keys: ExactTask267Keys<'_>,
) -> Result<(), ExactTask180LoweringError> {
    let invalid = |reason: &str| ExactTask180LoweringError::InvalidProjection {
        reason: reason.to_owned(),
    };
    if core.source_id() != draft.source_id
        || core.module_id() != &draft.module_id
        || core.items().len() != 1
        || !core.terms().is_empty()
        || core.formulas().len() != 1
        || !core.definitions().is_empty()
        || core.proofs().len() != 1
        || core.proof_nodes().len() != 1
        || !core.algorithms().is_empty()
        || !core.algorithm_statements().is_empty()
        || !core.generated().is_empty()
        || core.obligation_seeds().len() != 1
        || !core.diagnostics().is_empty()
    {
        return Err(invalid("table cardinality mismatch"));
    }

    let item = core
        .items()
        .get(CoreItemId::new(0))
        .ok_or_else(|| invalid("missing item zero"))?;
    let formula = core
        .formulas()
        .get(CoreFormulaId::new(0))
        .ok_or_else(|| invalid("missing formula zero"))?;
    let proof = core
        .proofs()
        .get(CoreProofId::new(0))
        .ok_or_else(|| invalid("missing proof zero"))?;
    let proof_node = core
        .proof_nodes()
        .get(CoreProofNodeId::new(0))
        .ok_or_else(|| invalid("missing proof node zero"))?;
    let obligation = core
        .obligation_seeds()
        .get(ObligationSeedId::new(0))
        .ok_or_else(|| invalid("missing obligation zero"))?;

    let item_source = CoreSourceRef::direct(draft.owner_range).with_provenance(vec![
        CoreProvenance::new(CoreProvenancePhase::Resolver, keys.resolver),
        CoreProvenance::new(CoreProvenancePhase::Checker, keys.statement),
    ]);
    let formula_source =
        CoreSourceRef::direct(draft.formula_range).with_provenance(vec![CoreProvenance::new(
            CoreProvenancePhase::Checker,
            keys.statement,
        )]);
    let proof_source = CoreSourceRef::direct(draft.owner_range).with_provenance(vec![
        CoreProvenance::new(CoreProvenancePhase::Resolver, keys.resolver),
        CoreProvenance::new(CoreProvenancePhase::Checker, keys.proof),
    ]);
    let terminal_source = CoreSourceRef::direct(draft.formula_range).with_provenance(vec![
        CoreProvenance::new(CoreProvenancePhase::Checker, keys.terminal),
        CoreProvenance::new(CoreProvenancePhase::ProofSkeleton, keys.skeleton),
    ]);
    if item.symbol != draft.owner
        || item.kind != CoreItemKind::Theorem
        || item.visibility.as_str() != "public"
        || item.status != CoreItemStatus::Valid
        || !item.dependencies.is_empty()
        || item.source != item_source
        || !item.diagnostics.is_empty()
        || formula.kind != CoreFormulaKind::False
        || formula.source != formula_source
        || proof.item != CoreItemId::new(0)
        || proof.proposition != CoreFormulaId::new(0)
        || proof.root != CoreProofNodeId::new(0)
        || proof.status != CoreProofStatus::PendingAutomaticProof
        || proof.source != proof_source
        || proof_node.source != terminal_source
        || !proof_node.diagnostics.is_empty()
    {
        return Err(invalid("item/formula/proof identity or source mismatch"));
    }
    let CoreProofNodeKind::TerminalGoal {
        obligation: terminal_obligation,
        citations,
    } = &proof_node.kind
    else {
        return Err(invalid("proof root is not a direct terminal goal"));
    };
    if *terminal_obligation != ObligationSeedId::new(0) || !citations.is_empty() {
        return Err(invalid("terminal goal payload mismatch"));
    }
    let expected_refs = vec![
        CoreNodeRef::Item(CoreItemId::new(0)),
        CoreNodeRef::Formula(CoreFormulaId::new(0)),
        CoreNodeRef::Proof(CoreProofId::new(0)),
        CoreNodeRef::ProofNode(CoreProofNodeId::new(0)),
    ];
    let expected_provenance = vec![
        CoreProvenance::new(CoreProvenancePhase::Checker, keys.terminal),
        CoreProvenance::new(CoreProvenancePhase::ProofSkeleton, keys.skeleton),
    ];
    if obligation.owner != CoreItemId::new(0)
        || obligation.kind != ObligationSeedKind::TheoremProof
        || obligation.goal != Some(CoreFormulaId::new(0))
        || !obligation.context.is_empty()
        || obligation.local_path.as_str() != "proof/0"
        || obligation.label.is_some()
        || obligation.semantic_origin.as_str() != draft.owner.fqn().as_str()
        || obligation.provenance != expected_provenance
        || obligation.source != terminal_source
        || obligation.core_refs != expected_refs
        || obligation.status != ObligationSeedStatus::Active
        || !obligation.diagnostics.is_empty()
    {
        return Err(invalid("theorem-proof obligation mismatch"));
    }

    let source_map = core.source_map();
    if source_map.item_sources.len() != 1
        || source_map.item_sources.get(&CoreItemId::new(0)) != Some(&item_source)
        || !source_map.term_sources.is_empty()
        || source_map.formula_sources.len() != 1
        || source_map.formula_sources.get(&CoreFormulaId::new(0)) != Some(&formula_source)
        || !source_map.definition_sources.is_empty()
        || source_map.proof_sources.len() != 1
        || source_map.proof_sources.get(&CoreProofNodeId::new(0)) != Some(&terminal_source)
        || !source_map.algorithm_sources.is_empty()
        || !source_map.generated_sources.is_empty()
        || source_map.obligation_sources.len() != 1
        || source_map.obligation_sources.get(&ObligationSeedId::new(0)) != Some(&terminal_source)
    {
        return Err(invalid("exact source-map mismatch"));
    }
    Ok(())
}

pub type AlgorithmLoweringResult<T> = Result<T, AlgorithmLoweringError>;

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AlgorithmLoweringError {
    MissingOwnerItem {
        owner: CoreItemId,
    },
    DuplicateAlgorithmOwner {
        owner: CoreItemId,
    },
    UnsupportedAlgorithmItemKind {
        owner: CoreItemId,
        kind: CoreItemKind,
    },
    MissingAlgorithmBoundary {
        owner: CoreItemId,
    },
    AlgorithmBoundaryMismatch {
        owner: CoreItemId,
        kind: DefinitionBoundaryKind,
    },
    AlgorithmBoundaryNotPending {
        owner: CoreItemId,
        status: DefinitionBoundaryStatus,
    },
    AlgorithmSymbolMismatch {
        owner: CoreItemId,
        expected: Box<SymbolId>,
        actual: Box<SymbolId>,
    },
    MissingAlgorithmTerm {
        term: CoreTermId,
    },
    MissingAlgorithmFormula {
        formula: CoreFormulaId,
    },
    UndeclaredAlgorithmBinder {
        var: CoreVarId,
    },
    NonTermAlgorithmBinder {
        var: CoreVarId,
        sort: NormalizedVarSort,
    },
    InvalidAlgorithmTarget {
        target: CorePlace,
    },
    InvalidSeedProvenance(CoreContextError),
}

impl fmt::Display for AlgorithmLoweringError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingOwnerItem { owner } => {
                write!(formatter, "missing algorithm owner item {}", owner.index())
            }
            Self::DuplicateAlgorithmOwner { owner } => {
                write!(
                    formatter,
                    "algorithm input contains duplicate owner item {}",
                    owner.index()
                )
            }
            Self::UnsupportedAlgorithmItemKind { owner, kind } => {
                write!(
                    formatter,
                    "algorithm owner item {} has unsupported kind {kind:?}",
                    owner.index()
                )
            }
            Self::MissingAlgorithmBoundary { owner } => {
                write!(
                    formatter,
                    "missing algorithm boundary for item {}",
                    owner.index()
                )
            }
            Self::AlgorithmBoundaryMismatch { owner, kind } => {
                write!(
                    formatter,
                    "algorithm boundary for item {} has non-algorithm kind {kind:?}",
                    owner.index()
                )
            }
            Self::AlgorithmBoundaryNotPending { owner, status } => {
                write!(
                    formatter,
                    "algorithm boundary for item {} has status {status:?}",
                    owner.index()
                )
            }
            Self::AlgorithmSymbolMismatch {
                owner,
                expected,
                actual,
            } => {
                write!(
                    formatter,
                    "algorithm seed for item {} used symbol {actual:?}; expected {expected:?}",
                    owner.index()
                )
            }
            Self::MissingAlgorithmTerm { term } => {
                write!(
                    formatter,
                    "algorithm references missing term {}",
                    term.index()
                )
            }
            Self::MissingAlgorithmFormula { formula } => {
                write!(
                    formatter,
                    "algorithm references missing formula {}",
                    formula.index()
                )
            }
            Self::UndeclaredAlgorithmBinder { var } => {
                write!(
                    formatter,
                    "algorithm uses undeclared binder {}",
                    var.index()
                )
            }
            Self::NonTermAlgorithmBinder { var, sort } => {
                write!(
                    formatter,
                    "algorithm binder {} has non-term sort {sort:?}",
                    var.index()
                )
            }
            Self::InvalidAlgorithmTarget { target } => {
                write!(
                    formatter,
                    "algorithm assignment has invalid target {}",
                    target.as_str()
                )
            }
            Self::InvalidSeedProvenance(error) => write!(formatter, "{error}"),
        }
    }
}

impl Error for AlgorithmLoweringError {}

impl From<CoreContextError> for AlgorithmLoweringError {
    fn from(value: CoreContextError) -> Self {
        Self::InvalidSeedProvenance(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlgorithmLoweringInput {
    pub algorithms: Vec<AlgorithmSeed>,
}

impl AlgorithmLoweringInput {
    pub const fn new() -> Self {
        Self {
            algorithms: Vec::new(),
        }
    }
}

impl Default for AlgorithmLoweringInput {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlgorithmSeed {
    pub owner: CoreItemId,
    pub symbol: SymbolId,
    pub params: Vec<CoreBinder>,
    pub result: Option<CoreBinder>,
    pub contracts: CoreContractSet,
    pub payload: AlgorithmPayloadSeed,
    pub ghost_effects: Vec<GhostEffectKey>,
    pub source: CoreSourceRef,
    pub provenance: CheckerOwnedProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AlgorithmPayloadSeed {
    Statements(Vec<AlgorithmStmtSeed>),
    Missing(FailedSemanticSiteSeed),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AlgorithmStmtSeed {
    Let {
        binder: CoreBinder,
        value: Option<CoreTermId>,
        ghost: bool,
        source: CoreSourceRef,
        provenance: CheckerOwnedProvenance,
    },
    Assign {
        target: CorePlace,
        value: CoreTermId,
        source: CoreSourceRef,
        provenance: CheckerOwnedProvenance,
    },
    Assert {
        formula: CoreFormulaId,
        source: CoreSourceRef,
        provenance: CheckerOwnedProvenance,
    },
    If {
        condition: CoreFormulaId,
        then_body: Vec<AlgorithmStmtSeed>,
        else_body: Vec<AlgorithmStmtSeed>,
        source: CoreSourceRef,
        provenance: CheckerOwnedProvenance,
    },
    While {
        condition: CoreFormulaId,
        invariants: Vec<CoreFormulaId>,
        decreasing: Vec<CoreTermId>,
        body: Vec<AlgorithmStmtSeed>,
        source: CoreSourceRef,
        provenance: CheckerOwnedProvenance,
    },
    Match {
        scrutinee: CoreTermId,
        arms: Vec<AlgorithmMatchArmSeed>,
        source: CoreSourceRef,
        provenance: CheckerOwnedProvenance,
    },
    Return {
        value: Option<CoreTermId>,
        source: CoreSourceRef,
        provenance: CheckerOwnedProvenance,
    },
    Break {
        source: CoreSourceRef,
        provenance: CheckerOwnedProvenance,
    },
    Continue {
        source: CoreSourceRef,
        provenance: CheckerOwnedProvenance,
    },
    Pick {
        binder: CoreBinder,
        witness_ty: Option<CoreFormulaId>,
        ghost: bool,
        source: CoreSourceRef,
        provenance: CheckerOwnedProvenance,
    },
    Error(FailedSemanticSiteSeed),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlgorithmMatchArmSeed {
    pub pattern: CoreProvenanceKey,
    pub body: Vec<AlgorithmStmtSeed>,
    pub provenance: CheckerOwnedProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlgorithmLoweringOutput {
    pub algorithms: CoreAlgorithmTable,
    pub algorithm_statements: CoreAlgorithmStmtTable,
    pub source_map: CoreSourceMap,
    pub diagnostics: CoreDiagnosticTable,
    pub algorithm_map: BTreeMap<CoreItemId, CoreAlgorithmId>,
}

#[derive(Debug, Clone)]
struct AlgorithmLoweringState {
    algorithms: CoreAlgorithmTable,
    algorithm_statements: CoreAlgorithmStmtTable,
    source_map: CoreSourceMap,
    diagnostics: CoreDiagnosticTable,
    algorithm_map: BTreeMap<CoreItemId, CoreAlgorithmId>,
}

impl AlgorithmLoweringState {
    fn new(proofs: &ProofLoweringOutput) -> Self {
        Self {
            algorithms: CoreAlgorithmTable::new(),
            algorithm_statements: CoreAlgorithmStmtTable::new(),
            source_map: proofs.source_map.clone(),
            diagnostics: proofs.diagnostics.clone(),
            algorithm_map: BTreeMap::new(),
        }
    }

    fn insert_statement(
        &mut self,
        owner: CoreAlgorithmId,
        kind: CoreAlgorithmStmtKind,
        source: CoreSourceRef,
        diagnostics: Vec<CoreDiagnosticId>,
    ) -> CoreAlgorithmStmtId {
        let source = normalized_source(source);
        let id = self.algorithm_statements.insert(CoreAlgorithmStmt {
            owner,
            kind,
            source: source.clone(),
            diagnostics,
        });
        self.source_map.algorithm_sources.insert(id, source);
        id
    }
}

pub fn lower_algorithm_inputs(
    context: &CoreContext,
    term_formula: &TermAndFormulaLoweringOutput,
    proofs: &ProofLoweringOutput,
    input: AlgorithmLoweringInput,
) -> AlgorithmLoweringResult<AlgorithmLoweringOutput> {
    validate_algorithm_input(context, term_formula, &input)?;
    let mut state = AlgorithmLoweringState::new(proofs);

    for seed in input.algorithms {
        let algorithm_id = CoreAlgorithmId::new(state.algorithms.len());
        let (statements, diagnostics) =
            lower_algorithm_payload(&mut state, algorithm_id, &seed.payload)?;
        let source = normalized_source(source_with_provenance(seed.source, &seed.provenance));
        let algorithm = CoreAlgorithm {
            item: seed.owner,
            symbol: seed.symbol,
            params: seed.params,
            result: seed.result,
            contracts: seed.contracts,
            statements,
            ghost_effects: seed.ghost_effects,
            source,
            diagnostics,
        };
        let inserted = state.algorithms.insert(algorithm);
        debug_assert_eq!(inserted, algorithm_id);
        state.algorithm_map.insert(seed.owner, inserted);
    }

    Ok(AlgorithmLoweringOutput {
        algorithms: state.algorithms,
        algorithm_statements: state.algorithm_statements,
        source_map: state.source_map,
        diagnostics: state.diagnostics,
        algorithm_map: state.algorithm_map,
    })
}

fn validate_algorithm_input(
    context: &CoreContext,
    term_formula: &TermAndFormulaLoweringOutput,
    input: &AlgorithmLoweringInput,
) -> AlgorithmLoweringResult<()> {
    let mut seen_owners = BTreeSet::new();
    for seed in &input.algorithms {
        if !seen_owners.insert(seed.owner) {
            return Err(AlgorithmLoweringError::DuplicateAlgorithmOwner { owner: seed.owner });
        }
        validate_checker_owned_provenance("algorithm seed", seed.provenance.as_slice())?;
        let item = context
            .item_registry()
            .items()
            .get(seed.owner)
            .ok_or(AlgorithmLoweringError::MissingOwnerItem { owner: seed.owner })?;
        if item.symbol != seed.symbol {
            return Err(AlgorithmLoweringError::AlgorithmSymbolMismatch {
                owner: seed.owner,
                expected: Box::new(item.symbol.clone()),
                actual: Box::new(seed.symbol.clone()),
            });
        }
        if item.kind != CoreItemKind::Algorithm {
            return Err(AlgorithmLoweringError::UnsupportedAlgorithmItemKind {
                owner: seed.owner,
                kind: item.kind.clone(),
            });
        }
        let boundary = context
            .definition_boundaries()
            .get_by_item(seed.owner)
            .ok_or(AlgorithmLoweringError::MissingAlgorithmBoundary { owner: seed.owner })?;
        if boundary.kind != DefinitionBoundaryKind::Algorithm {
            return Err(AlgorithmLoweringError::AlgorithmBoundaryMismatch {
                owner: seed.owner,
                kind: boundary.kind,
            });
        }
        if boundary.status != DefinitionBoundaryStatus::PendingBody {
            return Err(AlgorithmLoweringError::AlgorithmBoundaryNotPending {
                owner: seed.owner,
                status: boundary.status,
            });
        }
        validate_algorithm_binders(context, term_formula, &seed.params)?;
        if let Some(result) = &seed.result {
            validate_algorithm_binder(context, term_formula, result)?;
        }
        validate_algorithm_contracts(term_formula, &seed.contracts)?;
        match &seed.payload {
            AlgorithmPayloadSeed::Statements(statements) => {
                validate_algorithm_statements(context, term_formula, statements)?;
            }
            AlgorithmPayloadSeed::Missing(site) => {
                validate_checker_owned_provenance(
                    "missing algorithm payload",
                    site.provenance.as_slice(),
                )?;
            }
        }
    }
    Ok(())
}

fn validate_algorithm_binders(
    context: &CoreContext,
    term_formula: &TermAndFormulaLoweringOutput,
    binders: &[CoreBinder],
) -> AlgorithmLoweringResult<()> {
    for binder in binders {
        validate_algorithm_binder(context, term_formula, binder)?;
    }
    Ok(())
}

fn validate_algorithm_binder(
    context: &CoreContext,
    term_formula: &TermAndFormulaLoweringOutput,
    binder: &CoreBinder,
) -> AlgorithmLoweringResult<()> {
    match context.binder_context().variable_sorts.get(&binder.var) {
        Some(NormalizedVarSort::Term) => {}
        Some(sort) => {
            return Err(AlgorithmLoweringError::NonTermAlgorithmBinder {
                var: binder.var,
                sort: *sort,
            });
        }
        None => {
            return Err(AlgorithmLoweringError::UndeclaredAlgorithmBinder { var: binder.var });
        }
    }
    if let Some(guard) = binder.ty_guard {
        validate_algorithm_formula(term_formula, guard)?;
    }
    Ok(())
}

fn validate_algorithm_contracts(
    term_formula: &TermAndFormulaLoweringOutput,
    contracts: &CoreContractSet,
) -> AlgorithmLoweringResult<()> {
    for formula in contracts
        .requires
        .iter()
        .chain(contracts.ensures.iter())
        .chain(contracts.invariants.iter())
        .chain(contracts.assertions.iter())
    {
        validate_algorithm_formula(term_formula, *formula)?;
    }
    for term in &contracts.decreasing {
        validate_algorithm_term(term_formula, *term)?;
    }
    Ok(())
}

fn validate_algorithm_statements(
    context: &CoreContext,
    term_formula: &TermAndFormulaLoweringOutput,
    statements: &[AlgorithmStmtSeed],
) -> AlgorithmLoweringResult<()> {
    for statement in statements {
        validate_algorithm_statement_seed(context, term_formula, statement)?;
    }
    Ok(())
}

fn validate_algorithm_statement_seed(
    context: &CoreContext,
    term_formula: &TermAndFormulaLoweringOutput,
    statement: &AlgorithmStmtSeed,
) -> AlgorithmLoweringResult<()> {
    match statement {
        AlgorithmStmtSeed::Let {
            binder,
            value,
            provenance,
            ..
        } => {
            validate_checker_owned_provenance("algorithm let", provenance.as_slice())?;
            validate_algorithm_binder(context, term_formula, binder)?;
            if let Some(value) = value {
                validate_algorithm_term(term_formula, *value)?;
            }
        }
        AlgorithmStmtSeed::Assign {
            target,
            value,
            provenance,
            ..
        } => {
            validate_checker_owned_provenance("algorithm assignment", provenance.as_slice())?;
            validate_algorithm_target(target)?;
            validate_algorithm_term(term_formula, *value)?;
        }
        AlgorithmStmtSeed::Assert {
            formula,
            provenance,
            ..
        } => {
            validate_checker_owned_provenance("algorithm assertion", provenance.as_slice())?;
            validate_algorithm_formula(term_formula, *formula)?;
        }
        AlgorithmStmtSeed::If {
            condition,
            then_body,
            else_body,
            provenance,
            ..
        } => {
            validate_checker_owned_provenance("algorithm if", provenance.as_slice())?;
            validate_algorithm_formula(term_formula, *condition)?;
            validate_algorithm_statements(context, term_formula, then_body)?;
            validate_algorithm_statements(context, term_formula, else_body)?;
        }
        AlgorithmStmtSeed::While {
            condition,
            invariants,
            decreasing,
            body,
            provenance,
            ..
        } => {
            validate_checker_owned_provenance("algorithm while", provenance.as_slice())?;
            validate_algorithm_formula(term_formula, *condition)?;
            for invariant in invariants {
                validate_algorithm_formula(term_formula, *invariant)?;
            }
            for term in decreasing {
                validate_algorithm_term(term_formula, *term)?;
            }
            validate_algorithm_statements(context, term_formula, body)?;
        }
        AlgorithmStmtSeed::Match {
            scrutinee,
            arms,
            provenance,
            ..
        } => {
            validate_checker_owned_provenance("algorithm match", provenance.as_slice())?;
            validate_algorithm_term(term_formula, *scrutinee)?;
            for arm in arms {
                validate_checker_owned_provenance(
                    "algorithm match arm",
                    arm.provenance.as_slice(),
                )?;
                validate_algorithm_statements(context, term_formula, &arm.body)?;
            }
        }
        AlgorithmStmtSeed::Return {
            value, provenance, ..
        } => {
            validate_checker_owned_provenance("algorithm return", provenance.as_slice())?;
            if let Some(value) = value {
                validate_algorithm_term(term_formula, *value)?;
            }
        }
        AlgorithmStmtSeed::Break { provenance, .. } => {
            validate_checker_owned_provenance("algorithm break", provenance.as_slice())?;
        }
        AlgorithmStmtSeed::Continue { provenance, .. } => {
            validate_checker_owned_provenance("algorithm continue", provenance.as_slice())?;
        }
        AlgorithmStmtSeed::Pick {
            binder,
            witness_ty,
            provenance,
            ..
        } => {
            validate_checker_owned_provenance("algorithm pick", provenance.as_slice())?;
            validate_algorithm_binder(context, term_formula, binder)?;
            if let Some(witness_ty) = witness_ty {
                validate_algorithm_formula(term_formula, *witness_ty)?;
            }
        }
        AlgorithmStmtSeed::Error(site) => {
            validate_checker_owned_provenance(
                "malformed algorithm statement",
                site.provenance.as_slice(),
            )?;
        }
    }
    Ok(())
}

fn validate_algorithm_target(target: &CorePlace) -> AlgorithmLoweringResult<()> {
    if target.as_str().is_empty() {
        Err(AlgorithmLoweringError::InvalidAlgorithmTarget {
            target: target.clone(),
        })
    } else {
        Ok(())
    }
}

fn validate_algorithm_term(
    term_formula: &TermAndFormulaLoweringOutput,
    term: CoreTermId,
) -> AlgorithmLoweringResult<()> {
    term_formula
        .terms
        .get(term)
        .map(|_| ())
        .ok_or(AlgorithmLoweringError::MissingAlgorithmTerm { term })
}

fn validate_algorithm_formula(
    term_formula: &TermAndFormulaLoweringOutput,
    formula: CoreFormulaId,
) -> AlgorithmLoweringResult<()> {
    term_formula
        .formulas
        .get(formula)
        .map(|_| ())
        .ok_or(AlgorithmLoweringError::MissingAlgorithmFormula { formula })
}

fn lower_algorithm_payload(
    state: &mut AlgorithmLoweringState,
    owner: CoreAlgorithmId,
    payload: &AlgorithmPayloadSeed,
) -> AlgorithmLoweringResult<(Vec<CoreAlgorithmStmtId>, Vec<CoreDiagnosticId>)> {
    match payload {
        AlgorithmPayloadSeed::Statements(statements) => {
            lower_algorithm_statement_block(state, owner, statements).map(|statements| {
                let diagnostics = collect_algorithm_statement_diagnostics(
                    &state.algorithm_statements,
                    &statements,
                );
                (statements, diagnostics)
            })
        }
        AlgorithmPayloadSeed::Missing(site) => {
            let (statement, diagnostic) = insert_algorithm_error_statement(state, owner, site);
            Ok((vec![statement], vec![diagnostic]))
        }
    }
}

fn lower_algorithm_statement_block(
    state: &mut AlgorithmLoweringState,
    owner: CoreAlgorithmId,
    statements: &[AlgorithmStmtSeed],
) -> AlgorithmLoweringResult<Vec<CoreAlgorithmStmtId>> {
    statements
        .iter()
        .map(|statement| lower_algorithm_statement(state, owner, statement))
        .collect()
}

fn lower_algorithm_statement(
    state: &mut AlgorithmLoweringState,
    owner: CoreAlgorithmId,
    statement: &AlgorithmStmtSeed,
) -> AlgorithmLoweringResult<CoreAlgorithmStmtId> {
    match statement {
        AlgorithmStmtSeed::Let {
            binder,
            value,
            ghost,
            source,
            provenance,
        } => Ok(state.insert_statement(
            owner,
            CoreAlgorithmStmtKind::Let {
                binder: binder.clone(),
                value: *value,
                ghost: *ghost,
            },
            source_with_provenance(source.clone(), provenance),
            Vec::new(),
        )),
        AlgorithmStmtSeed::Assign {
            target,
            value,
            source,
            provenance,
        } => Ok(state.insert_statement(
            owner,
            CoreAlgorithmStmtKind::Assign {
                target: target.clone(),
                value: *value,
            },
            source_with_provenance(source.clone(), provenance),
            Vec::new(),
        )),
        AlgorithmStmtSeed::Assert {
            formula,
            source,
            provenance,
        } => Ok(state.insert_statement(
            owner,
            CoreAlgorithmStmtKind::Assert { formula: *formula },
            source_with_provenance(source.clone(), provenance),
            Vec::new(),
        )),
        AlgorithmStmtSeed::If {
            condition,
            then_body,
            else_body,
            source,
            provenance,
        } => {
            let then_body = lower_algorithm_statement_block(state, owner, then_body)?;
            let else_body = lower_algorithm_statement_block(state, owner, else_body)?;
            Ok(state.insert_statement(
                owner,
                CoreAlgorithmStmtKind::If {
                    condition: *condition,
                    then_body,
                    else_body,
                },
                source_with_provenance(source.clone(), provenance),
                Vec::new(),
            ))
        }
        AlgorithmStmtSeed::While {
            condition,
            invariants,
            decreasing,
            body,
            source,
            provenance,
        } => {
            let body = lower_algorithm_statement_block(state, owner, body)?;
            Ok(state.insert_statement(
                owner,
                CoreAlgorithmStmtKind::While {
                    condition: *condition,
                    invariants: invariants.clone(),
                    decreasing: decreasing.clone(),
                    body,
                },
                source_with_provenance(source.clone(), provenance),
                Vec::new(),
            ))
        }
        AlgorithmStmtSeed::Match {
            scrutinee,
            arms,
            source,
            provenance,
        } => {
            let mut lowered_arms = Vec::new();
            for arm in arms {
                lowered_arms.push(CoreAlgorithmMatchArm {
                    pattern: arm.pattern.clone(),
                    body: lower_algorithm_statement_block(state, owner, &arm.body)?,
                });
            }
            Ok(state.insert_statement(
                owner,
                CoreAlgorithmStmtKind::Match {
                    scrutinee: *scrutinee,
                    arms: lowered_arms,
                },
                source_with_provenance(source.clone(), provenance),
                Vec::new(),
            ))
        }
        AlgorithmStmtSeed::Return {
            value,
            source,
            provenance,
        } => Ok(state.insert_statement(
            owner,
            CoreAlgorithmStmtKind::Return(*value),
            source_with_provenance(source.clone(), provenance),
            Vec::new(),
        )),
        AlgorithmStmtSeed::Break { source, provenance } => Ok(state.insert_statement(
            owner,
            CoreAlgorithmStmtKind::Break,
            source_with_provenance(source.clone(), provenance),
            Vec::new(),
        )),
        AlgorithmStmtSeed::Continue { source, provenance } => Ok(state.insert_statement(
            owner,
            CoreAlgorithmStmtKind::Continue,
            source_with_provenance(source.clone(), provenance),
            Vec::new(),
        )),
        AlgorithmStmtSeed::Pick {
            binder,
            witness_ty,
            ghost,
            source,
            provenance,
        } => Ok(state.insert_statement(
            owner,
            CoreAlgorithmStmtKind::Pick {
                binder: binder.clone(),
                witness_ty: *witness_ty,
                ghost: *ghost,
            },
            source_with_provenance(source.clone(), provenance),
            Vec::new(),
        )),
        AlgorithmStmtSeed::Error(site) => {
            let (statement, _) = insert_algorithm_error_statement(state, owner, site);
            Ok(statement)
        }
    }
}

fn collect_algorithm_statement_diagnostics(
    table: &CoreAlgorithmStmtTable,
    statements: &[CoreAlgorithmStmtId],
) -> Vec<CoreDiagnosticId> {
    let mut diagnostics = Vec::new();
    let mut seen = BTreeSet::new();
    collect_algorithm_statement_diagnostics_into(table, statements, &mut diagnostics, &mut seen);
    diagnostics
}

fn collect_algorithm_statement_diagnostics_into(
    table: &CoreAlgorithmStmtTable,
    statements: &[CoreAlgorithmStmtId],
    diagnostics: &mut Vec<CoreDiagnosticId>,
    seen: &mut BTreeSet<CoreDiagnosticId>,
) {
    for statement_id in statements {
        let Some(statement) = table.get(*statement_id) else {
            continue;
        };
        for diagnostic in &statement.diagnostics {
            if seen.insert(*diagnostic) {
                diagnostics.push(*diagnostic);
            }
        }
        match &statement.kind {
            CoreAlgorithmStmtKind::If {
                then_body,
                else_body,
                ..
            } => {
                collect_algorithm_statement_diagnostics_into(table, then_body, diagnostics, seen);
                collect_algorithm_statement_diagnostics_into(table, else_body, diagnostics, seen);
            }
            CoreAlgorithmStmtKind::While { body, .. } => {
                collect_algorithm_statement_diagnostics_into(table, body, diagnostics, seen);
            }
            CoreAlgorithmStmtKind::Match { arms, .. } => {
                for arm in arms {
                    collect_algorithm_statement_diagnostics_into(
                        table,
                        &arm.body,
                        diagnostics,
                        seen,
                    );
                }
            }
            CoreAlgorithmStmtKind::Let { .. }
            | CoreAlgorithmStmtKind::Assign { .. }
            | CoreAlgorithmStmtKind::Assert { .. }
            | CoreAlgorithmStmtKind::Return(_)
            | CoreAlgorithmStmtKind::Break
            | CoreAlgorithmStmtKind::Continue
            | CoreAlgorithmStmtKind::Pick { .. }
            | CoreAlgorithmStmtKind::Error(_) => {}
        }
    }
}

fn insert_algorithm_error_statement(
    state: &mut AlgorithmLoweringState,
    owner: CoreAlgorithmId,
    site: &FailedSemanticSiteSeed,
) -> (CoreAlgorithmStmtId, CoreDiagnosticId) {
    let source = source_with_provenance(site.source.clone(), &site.provenance);
    let diagnostic_id = state.diagnostics.insert(diagnostic(
        CoreDiagnosticClass::AlgorithmShell,
        CoreDiagnosticSeverity::Error,
        CoreDiagnosticRecovery::Fatal,
        site.message_key.clone(),
        source.clone(),
        None,
    ));
    let statement = state.insert_statement(
        owner,
        CoreAlgorithmStmtKind::Error(diagnostic_id),
        source,
        vec![diagnostic_id],
    );
    if let Some(diagnostic) = state.diagnostics.get_mut(diagnostic_id) {
        diagnostic.owner = Some(CoreNodeRef::AlgorithmStmt(statement));
    }
    (statement, diagnostic_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core_ir::{
        CoreAlgorithmStmtTable, CoreAlgorithmTable, CoreDefinitionTable, CoreIr, CoreIrParts,
        CoreProofNodeTable, CoreProofTable,
    };
    use mizar_checker::{
        binding_env::{
            BindingContextDraft, BindingContextTable, BindingDiagnosticClass,
            BindingDiagnosticDraft, BindingDiagnosticRecovery, BindingDiagnosticSeverity,
            BindingDiagnosticTable, BindingDraft, BindingEnvParts, BindingTable, BindingTypeSite,
            CapturedFreeVariables,
        },
        typed_ast::TypeRole,
    };
    use mizar_resolve::names::LocalTermScope;
    use mizar_resolve::resolved_ast::{FullyQualifiedName, LocalSymbolId};
    use mizar_session::{
        BuildSnapshotId, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator,
    };

    fn source_id_for(hex_pair: &str) -> SourceId {
        let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            hex_pair.repeat(32)
        ))
        .expect("valid snapshot id");
        InMemorySessionIdAllocator::new()
            .next_source_id(snapshot)
            .expect("source id")
    }

    fn source_id() -> SourceId {
        source_id_for("08")
    }

    fn alternate_source_id() -> SourceId {
        let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            "09".repeat(32)
        ))
        .expect("valid alternate snapshot id");
        let allocator = InMemorySessionIdAllocator::new();
        let _discarded = allocator
            .next_source_id(snapshot)
            .expect("first alternate source id");
        allocator
            .next_source_id(snapshot)
            .expect("second alternate source id")
    }

    fn range(start: usize, end: usize) -> SourceRange {
        SourceRange {
            source_id: source_id(),
            start,
            end,
        }
    }

    fn direct(start: usize, end: usize) -> CoreSourceRef {
        CoreSourceRef::direct(range(start, end))
    }

    fn module_id() -> ModuleId {
        ModuleId::new(PackageId::new("pkg"), ModulePath::new("main"))
    }

    fn external_module_id() -> ModuleId {
        ModuleId::new(PackageId::new("pkg"), ModulePath::new("dep"))
    }

    fn symbol(name: &str) -> SymbolId {
        SymbolId::new(
            module_id(),
            LocalSymbolId::new(name),
            FullyQualifiedName::new(format!("pkg::main::{name}")),
        )
    }

    fn external_symbol(name: &str) -> SymbolId {
        SymbolId::new(
            external_module_id(),
            LocalSymbolId::new(name),
            FullyQualifiedName::new(format!("pkg::dep::{name}")),
        )
    }

    fn provenance(key: &str) -> CheckerOwnedProvenance {
        CheckerOwnedProvenance::checker(key)
    }

    fn summary() -> ResolvedTypedAstSummary {
        ResolvedTypedAstSummary::new(source_id(), module_id())
    }

    fn source_binding_env_with_one(
        source_id: SourceId,
        module_id: ModuleId,
        kind: BindingKind,
        identity: BinderIdentity,
        status: BindingStatus,
    ) -> BindingEnv {
        let declaration_range = SourceRange {
            source_id,
            start: 10,
            end: 11,
        };
        let mut bindings = BindingTable::new();
        let binding = bindings.insert(BindingDraft {
            spelling: "x".to_owned(),
            kind,
            identity,
            owner_context: BindingContextId::new(0),
            declaration_range,
            visible_after_ordinal: 0,
            type_site: BindingTypeSite::Source(SourceRange {
                source_id,
                start: 12,
                end: 15,
            }),
            status,
            captured: CapturedFreeVariables::default(),
            diagnostics: Vec::new(),
            recovery: BindingRecoveryState::Normal,
        });
        let mut contexts = BindingContextTable::new();
        contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Module,
            parent: None,
            layer: BindingContextLayer::Module,
            lexical_scope: None,
            bindings: vec![binding],
            visible_bindings: vec![binding],
            recovery: BindingContextRecovery::Normal,
        });
        BindingEnv::try_new(BindingEnvParts {
            source_id,
            module_id,
            contexts,
            bindings,
            diagnostics: mizar_checker::binding_env::BindingDiagnosticTable::new(),
        })
        .expect("valid test binding environment")
    }

    fn source_binding_reserved_draft() -> BindingDraft {
        let declaration_range = range(10, 11);
        BindingDraft {
            spelling: "x".to_owned(),
            kind: BindingKind::ReservedVariable,
            identity: BinderIdentity::ReservedVariable {
                spelling: "x".to_owned(),
                declaration_range,
            },
            owner_context: BindingContextId::new(0),
            declaration_range,
            visible_after_ordinal: 0,
            type_site: BindingTypeSite::Source(range(12, 15)),
            status: BindingStatus::Reserved,
            captured: CapturedFreeVariables::default(),
            diagnostics: Vec::new(),
            recovery: BindingRecoveryState::Normal,
        }
    }

    fn source_binding_second_reserved_draft() -> BindingDraft {
        let declaration_range = range(20, 21);
        BindingDraft {
            spelling: "y".to_owned(),
            kind: BindingKind::ReservedVariable,
            identity: BinderIdentity::ReservedVariable {
                spelling: "y".to_owned(),
                declaration_range,
            },
            owner_context: BindingContextId::new(0),
            declaration_range,
            visible_after_ordinal: 1,
            type_site: BindingTypeSite::Source(range(22, 25)),
            status: BindingStatus::Reserved,
            captured: CapturedFreeVariables::default(),
            diagnostics: Vec::new(),
            recovery: BindingRecoveryState::Normal,
        }
    }

    fn source_binding_env_with_options(
        module_recovery: BindingContextRecovery,
        first: BindingDraft,
        second: BindingDraft,
        diagnostics: BindingDiagnosticTable,
    ) -> BindingEnv {
        let mut contexts = BindingContextTable::new();
        contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Module,
            parent: None,
            layer: BindingContextLayer::Module,
            lexical_scope: None,
            bindings: vec![BindingId::new(0), BindingId::new(1)],
            visible_bindings: vec![BindingId::new(0), BindingId::new(1)],
            recovery: module_recovery,
        });
        let mut bindings = BindingTable::new();
        bindings.insert(first);
        bindings.insert(second);
        BindingEnv::try_new(BindingEnvParts {
            source_id: source_id(),
            module_id: module_id(),
            contexts,
            bindings,
            diagnostics,
        })
        .expect("valid test binding environment")
    }

    fn source_binding_env() -> BindingEnv {
        source_binding_env_with_options(
            BindingContextRecovery::Normal,
            source_binding_reserved_draft(),
            source_binding_second_reserved_draft(),
            BindingDiagnosticTable::new(),
        )
    }

    fn empty_source_binding_env() -> BindingEnv {
        let mut contexts = BindingContextTable::new();
        contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Module,
            parent: None,
            layer: BindingContextLayer::Module,
            lexical_scope: None,
            bindings: Vec::new(),
            visible_bindings: Vec::new(),
            recovery: BindingContextRecovery::Normal,
        });
        BindingEnv::try_new(BindingEnvParts {
            source_id: source_id(),
            module_id: module_id(),
            contexts,
            bindings: BindingTable::new(),
            diagnostics: BindingDiagnosticTable::new(),
        })
        .expect("valid empty binding environment")
    }

    fn input_with_items(item_seeds: Vec<CoreItemSeed>) -> CoreContextInput {
        let mut input = CoreContextInput::new(summary());
        input.item_seeds = item_seeds;
        input
    }

    fn item_seed(name: &str, start: usize) -> CoreItemSeed {
        CoreItemSeed::new(
            symbol(name),
            CoreItemKind::Theorem,
            "public",
            direct(start, start + 3),
            provenance(format!("checker:item:{name}").as_str()),
        )
        .with_definition_boundary(DefinitionBoundaryKind::Theorem)
    }

    fn exact_task180_draft() -> ExactTask180Draft {
        let owner_range = range(10, 90);
        ExactTask180Draft {
            source_id: source_id(),
            module_id: module_id(),
            owner: symbol("SourceDerivedContradictionConstantBoundary"),
            owner_range,
            owner_origin: SemanticOrigin::new(
                source_id(),
                module_id(),
                SourceAnchor::Range(owner_range),
                Vec::new(),
            ),
            owner_node: TypedNodeId::new(1),
            formula_range: range(50, 63),
            formula_site_node: TypedNodeId::new(42),
            formula_node: TypedNodeId::new(0),
        }
    }

    #[test]
    fn exact_task267_encoders_are_byte_canonical_and_keep_node_roles_distinct() {
        let mut draft = exact_task180_draft();
        draft.owner = SymbolId::new(
            module_id(),
            LocalSymbolId::new("unicode"),
            FullyQualifiedName::new("pkg::main::証明"),
        );
        assert_eq!(
            exact_task267_resolver_key(&draft.owner, &draft.owner_origin),
            "task267/v1;owner-fqn=17:pkg::main::証明;origin-path=0:"
        );

        draft.owner_origin = SemanticOrigin::new(
            source_id(),
            module_id(),
            SourceAnchor::Range(draft.owner_range),
            vec![0, 12],
        );
        assert_eq!(
            exact_task267_resolver_key(&draft.owner, &draft.owner_origin),
            "task267/v1;owner-fqn=17:pkg::main::証明;origin-path=2:0,12"
        );

        draft.owner_node = TypedNodeId::new(12);
        draft.formula_site_node = TypedNodeId::new(34);
        draft.formula_node = TypedNodeId::new(5);
        assert_eq!(
            exact_task267_statement_key(&draft),
            "task267/v1;statement=0;owner-node=12;formula=0;formula-site-node=34;formula-node=5"
        );
        assert_eq!(
            exact_task267_terminal_key(&draft),
            "task267/v1;proof-node=0;terminal-goal=0;formula=0;formula-site-node=34;formula-node=5"
        );
        assert_eq!(
            exact_task180_formula_site_node(&TypedSiteRef::Node(TypedNodeId::new(34)))
                .expect("node sites are exact inputs"),
            TypedNodeId::new(34)
        );
        assert!(matches!(
            exact_task180_formula_site_node(&TypedSiteRef::Role {
                node: TypedNodeId::new(34),
                role: TypeRole::new("result"),
            }),
            Err(ExactTask180LoweringError::InvalidCheckerBundle { .. })
        ));
    }

    #[test]
    fn exact_task180_draft_lowers_atomically_and_deterministically() {
        let draft = exact_task180_draft();
        let first = lower_exact_task180_draft(draft.clone(), ExactTask180FailureInjection::None)
            .expect("exact draft lowers");
        let second = lower_exact_task180_draft(draft, ExactTask180FailureInjection::None)
            .expect("equivalent exact draft lowers");

        assert_eq!(first, second);
        assert_eq!(first.debug_text(), second.debug_text());
        assert!(!first.has_error_nodes());
        assert_eq!(
            first
                .proofs()
                .get(CoreProofId::new(0))
                .expect("proof zero")
                .status,
            CoreProofStatus::PendingAutomaticProof
        );
    }

    #[test]
    fn exact_task180_failure_injections_return_only_typed_errors() {
        let draft = exact_task180_draft();
        assert!(matches!(
            lower_exact_task180_draft(draft.clone(), ExactTask180FailureInjection::Preflight),
            Err(ExactTask180LoweringError::InvalidCheckerBundle { .. })
        ));
        assert!(matches!(
            lower_exact_task180_draft(draft.clone(), ExactTask180FailureInjection::GenericLowering),
            Err(ExactTask180LoweringError::GenericLowering { .. })
        ));
        assert!(matches!(
            lower_exact_task180_draft(
                draft.clone(),
                ExactTask180FailureInjection::ProvenanceEnrichment
            ),
            Err(ExactTask180LoweringError::ProvenanceEnrichment { .. })
        ));
        assert!(matches!(
            lower_exact_task180_draft(draft, ExactTask180FailureInjection::InvalidProjection),
            Err(ExactTask180LoweringError::InvalidProjection { .. })
        ));
    }

    fn exact_task180_core_parts(core: &CoreIr) -> CoreIrParts {
        CoreIrParts {
            source_id: core.source_id(),
            module_id: core.module_id().clone(),
            items: core.items().clone(),
            terms: core.terms().clone(),
            formulas: core.formulas().clone(),
            definitions: core.definitions().clone(),
            proofs: core.proofs().clone(),
            proof_nodes: core.proof_nodes().clone(),
            algorithms: core.algorithms().clone(),
            algorithm_statements: core.algorithm_statements().clone(),
            generated: core.generated().clone(),
            obligation_seeds: core.obligation_seeds().clone(),
            source_map: core.source_map().clone(),
            diagnostics: core.diagnostics().clone(),
        }
    }

    type ExactTask180ProjectionMutation = fn(&mut CoreIrParts);

    fn assert_exact_task180_projection_rejects(
        case: &str,
        core: &CoreIr,
        draft: &ExactTask180Draft,
        mutate: ExactTask180ProjectionMutation,
    ) {
        let mut parts = exact_task180_core_parts(core);
        mutate(&mut parts);
        let mutated = CoreIr::try_new(parts).unwrap_or_else(|error| {
            panic!("{case}: near-miss must remain generic CoreIr: {error}")
        });
        let resolver_key = exact_task267_resolver_key(&draft.owner, &draft.owner_origin);
        let statement_key = exact_task267_statement_key(draft);
        let proof_key = "task267/v1;proof=0;statement=0;policy=unmodified;justification=omitted;status=pending-automatic-proof";
        let terminal_key = exact_task267_terminal_key(draft);
        assert!(
            matches!(
                validate_exact_task180_projection(
                    &mutated,
                    draft,
                    ExactTask267Keys {
                        resolver: &resolver_key,
                        statement: &statement_key,
                        proof: proof_key,
                        terminal: &terminal_key,
                        skeleton: "task267/v1;local-path=7:proof/0",
                    },
                ),
                Err(ExactTask180LoweringError::InvalidProjection { .. })
            ),
            "{case}: exact projection validation must fail closed",
        );
    }

    #[test]
    fn exact_task180_postvalidator_rejects_structurally_valid_near_misses() {
        let draft = exact_task180_draft();
        let core = lower_exact_task180_draft(draft.clone(), ExactTask180FailureInjection::None)
            .expect("exact draft lowers");
        let cases: &[(&str, ExactTask180ProjectionMutation)] = &[
            ("item visibility", |parts| {
                parts
                    .items
                    .get_mut(CoreItemId::new(0))
                    .expect("item zero")
                    .visibility = CoreVisibility::new("private");
            }),
            ("formula kind", |parts| {
                parts
                    .formulas
                    .get_mut(CoreFormulaId::new(0))
                    .expect("formula zero")
                    .kind = CoreFormulaKind::True;
            }),
            ("formula source", |parts| {
                let source = CoreSourceRef::direct(SourceRange {
                    source_id: parts.source_id,
                    start: 51,
                    end: 62,
                });
                parts
                    .formulas
                    .get_mut(CoreFormulaId::new(0))
                    .expect("formula zero")
                    .source = source.clone();
                parts
                    .source_map
                    .formula_sources
                    .insert(CoreFormulaId::new(0), source);
            }),
            ("proof source", |parts| {
                parts
                    .proofs
                    .get_mut(CoreProofId::new(0))
                    .expect("proof zero")
                    .source = CoreSourceRef::direct(SourceRange {
                    source_id: parts.source_id,
                    start: 11,
                    end: 89,
                });
            }),
            ("proof status", |parts| {
                parts
                    .proofs
                    .get_mut(CoreProofId::new(0))
                    .expect("proof zero")
                    .status = CoreProofStatus::Open;
            }),
            ("proof-node kind", |parts| {
                parts
                    .proof_nodes
                    .get_mut(CoreProofNodeId::new(0))
                    .expect("proof node zero")
                    .kind = CoreProofNodeKind::Sequence {
                    children: Vec::new(),
                };
            }),
            ("terminal citations", |parts| {
                let CoreProofNodeKind::TerminalGoal { citations, .. } = &mut parts
                    .proof_nodes
                    .get_mut(CoreProofNodeId::new(0))
                    .expect("proof node zero")
                    .kind
                else {
                    panic!("exact proof node must be terminal");
                };
                citations.push(CoreCitation::Symbol(external_symbol("ExternalCitation")));
            }),
            ("terminal source", |parts| {
                let source = CoreSourceRef::direct(SourceRange {
                    source_id: parts.source_id,
                    start: 52,
                    end: 61,
                });
                parts
                    .proof_nodes
                    .get_mut(CoreProofNodeId::new(0))
                    .expect("proof node zero")
                    .source = source.clone();
                parts
                    .source_map
                    .proof_sources
                    .insert(CoreProofNodeId::new(0), source);
            }),
            ("obligation status", |parts| {
                parts
                    .obligation_seeds
                    .get_mut(ObligationSeedId::new(0))
                    .expect("obligation zero")
                    .status = ObligationSeedStatus::Deferred;
            }),
            ("obligation path", |parts| {
                parts
                    .obligation_seeds
                    .get_mut(ObligationSeedId::new(0))
                    .expect("obligation zero")
                    .local_path = LocalProofOrProgramPath::new("proof/1");
            }),
            ("obligation provenance", |parts| {
                parts
                    .obligation_seeds
                    .get_mut(ObligationSeedId::new(0))
                    .expect("obligation zero")
                    .provenance = vec![CoreProvenance::new(
                    CoreProvenancePhase::Checker,
                    "task267/v1;unexpected-obligation-provenance",
                )];
            }),
            ("obligation core refs", |parts| {
                parts
                    .obligation_seeds
                    .get_mut(ObligationSeedId::new(0))
                    .expect("obligation zero")
                    .core_refs
                    .retain(|core_ref| !matches!(core_ref, CoreNodeRef::Formula(_)));
            }),
            ("obligation source-map ownership", |parts| {
                let source = CoreSourceRef::direct(SourceRange {
                    source_id: parts.source_id,
                    start: 53,
                    end: 60,
                });
                parts
                    .obligation_seeds
                    .get_mut(ObligationSeedId::new(0))
                    .expect("obligation zero")
                    .source = source.clone();
                parts
                    .source_map
                    .obligation_sources
                    .insert(ObligationSeedId::new(0), source);
            }),
            ("item source-map ownership", |parts| {
                let source = parts
                    .formulas
                    .get(CoreFormulaId::new(0))
                    .expect("formula zero")
                    .source
                    .clone();
                parts
                    .items
                    .get_mut(CoreItemId::new(0))
                    .expect("item zero")
                    .source = source.clone();
                parts
                    .source_map
                    .item_sources
                    .insert(CoreItemId::new(0), source);
            }),
        ];

        for (case, mutate) in cases {
            assert_exact_task180_projection_rejects(case, &core, &draft, *mutate);
        }
    }

    fn algorithm_item_seed(name: &str, start: usize) -> CoreItemSeed {
        CoreItemSeed::new(
            symbol(name),
            CoreItemKind::Algorithm,
            "public",
            direct(start, start + 3),
            provenance(format!("checker:item:{name}").as_str()),
        )
        .with_definition_boundary(DefinitionBoundaryKind::Algorithm)
    }

    fn context_with_algorithm_var_sorts(
        vars: Vec<(CoreVarId, NormalizedVarSort)>,
    ) -> (CoreContext, CoreItemId) {
        let mut input = input_with_items(vec![algorithm_item_seed("Owner", 0)]);
        input.variable_seeds = vars
            .iter()
            .map(|(var, sort)| {
                CoreVariableSeed::new(
                    *var,
                    NormalizedVarClass::Free,
                    "term-binder",
                    *sort,
                    provenance(format!("checker:algorithm-var:{}", var.index()).as_str()),
                )
            })
            .collect();
        input.binder_seeds = vars
            .iter()
            .map(|(var, _)| {
                CoreBinderSeed::new(
                    *var,
                    direct(var.index() + 1, var.index() + 2),
                    provenance(format!("checker:algorithm-binder:{}", var.index()).as_str()),
                )
            })
            .collect();
        let context = prepare_core_context(input).expect("context");
        let owner = context
            .item_registry()
            .id_for_symbol(&symbol("Owner"))
            .expect("owner id");
        (context, owner)
    }

    fn context_with_var(var: CoreVarId) -> (CoreContext, CoreItemId) {
        context_with_var_sort(var, NormalizedVarSort::Term)
    }

    fn context_with_var_sort(var: CoreVarId, sort: NormalizedVarSort) -> (CoreContext, CoreItemId) {
        let mut input = input_with_items(vec![item_seed("Owner", 0)]);
        input.variable_seeds = vec![CoreVariableSeed::new(
            var,
            NormalizedVarClass::Free,
            "term-binder",
            sort,
            provenance("checker:var"),
        )];
        input.binder_seeds = vec![CoreBinderSeed::new(
            var,
            direct(1, 2),
            provenance("checker:binder"),
        )];
        let context = prepare_core_context(input).expect("context");
        let owner = context
            .item_registry()
            .id_for_symbol(&symbol("Owner"))
            .expect("owner id");
        (context, owner)
    }

    fn context_with_var_sorts(
        vars: Vec<(CoreVarId, NormalizedVarSort)>,
    ) -> (CoreContext, CoreItemId) {
        let mut input = input_with_items(vec![item_seed("Owner", 0)]);
        input.variable_seeds = vars
            .iter()
            .map(|(var, sort)| {
                CoreVariableSeed::new(
                    *var,
                    NormalizedVarClass::Free,
                    "term-binder",
                    *sort,
                    provenance(format!("checker:var:{}", var.index()).as_str()),
                )
            })
            .collect();
        input.binder_seeds = vars
            .iter()
            .map(|(var, _)| {
                CoreBinderSeed::new(
                    *var,
                    direct(var.index() + 1, var.index() + 2),
                    provenance(format!("checker:binder:{}", var.index()).as_str()),
                )
            })
            .collect();
        let context = prepare_core_context(input).expect("context");
        let owner = context
            .item_registry()
            .id_for_symbol(&symbol("Owner"))
            .expect("owner id");
        (context, owner)
    }

    fn context_with_existing_choice_origin(
        var: CoreVarId,
        key: GeneratedOriginKey,
    ) -> (CoreContext, CoreItemId, GeneratedOriginId) {
        let mut input = input_with_items(vec![item_seed("Owner", 0)]);
        input.variable_seeds = vec![CoreVariableSeed::new(
            var,
            NormalizedVarClass::Free,
            "term-binder",
            NormalizedVarSort::Term,
            provenance("checker:choice-var"),
        )];
        input.binder_seeds = vec![CoreBinderSeed::new(
            var,
            direct(1, 2),
            provenance("checker:choice-binder"),
        )];
        input.generated_origin_seeds = vec![
            GeneratedOriginSeed::new(
                symbol("Owner"),
                GeneratedOriginKind::StableChoice,
                key.clone(),
                direct(90, 91),
                provenance("checker:existing-choice"),
            )
            .with_functor(symbol("choice_existing"))
            .with_params(vec![var])
            .with_evidence(vec![CoreProvenance::new(
                CoreProvenancePhase::Checker,
                "checker:existing-choice:evidence",
            )]),
        ];
        let context = prepare_core_context(input).expect("context");
        let owner = context
            .item_registry()
            .id_for_symbol(&symbol("Owner"))
            .expect("owner id");
        let origin = context
            .generated_origins()
            .get_by_key(owner, GeneratedOriginKind::StableChoice, &key)
            .expect("existing generated origin");
        (context, owner, origin)
    }

    fn type_fact(
        subject: CoreVarId,
        predicate: &str,
        start: usize,
        polarity: Polarity,
    ) -> TypePredicateSeed {
        TypePredicateSeed::positive(
            subject,
            CoreTypePredicate::new(predicate),
            direct(start, start + 1),
            provenance(format!("checker:fact:{predicate}").as_str()),
        )
        .with_polarity(polarity)
    }

    fn active_obligation(
        subject: CoreVarId,
        predicate: &str,
        start: usize,
    ) -> CarriedInitialObligationSeed {
        CarriedInitialObligationSeed::active(
            InitialObligationId::new(start),
            InitialObligationKind::Narrowing,
            ObligationFormulaSeed::positive(subject, predicate, direct(start, start + 1)),
            format!("type-obligation/{start}"),
            format!("pkg::main::Owner.type-obligation.{start}"),
            direct(start, start + 1),
            provenance(format!("checker:obligation:{start}").as_str()),
        )
    }

    fn term_seed(kind: CoreTermSeedKind, start: usize) -> CoreTermSeed {
        CoreTermSeed::new(
            kind,
            direct(start, start + 1),
            provenance(format!("checker:term:{start}").as_str()),
        )
    }

    fn reduct_view(path: &str, functors: &[&str]) -> ReductViewSeed {
        ReductViewSeed {
            path: QuaPathKey::new(path),
            functors: functors.iter().copied().map(symbol).collect(),
        }
    }

    fn template_type_parameter(
        parameter: &str,
        witness: CoreVarId,
        predicate: &str,
        start: usize,
    ) -> TemplateTypeParameterInhabitationSeed {
        TemplateTypeParameterInhabitationSeed {
            parameter: TemplateParameterKey::new(parameter),
            witness,
            witness_role: CoreVarRole::new("template-type-witness"),
            witness_source_name: Some(format!("{parameter}$inhabitant")),
            predicate: CoreTypePredicate::new(predicate),
            source: direct(start, start + 1),
            provenance: provenance(format!("checker:template-param:{parameter}").as_str()),
        }
    }

    fn template_type_actual_gate(
        instantiation: &str,
        parameter: &str,
        status: ExistentialGateStatus,
        start: usize,
    ) -> TemplateTypeActualGateSeed {
        TemplateTypeActualGateSeed {
            instantiation: TemplateInstantiationKey::new(instantiation),
            parameter: TemplateParameterKey::new(parameter),
            actual_type: NormalizedTypeId::new(start),
            gate: Some(ExistentialGateId::new(start)),
            status,
            registration: None,
            base_evidence_kind: None,
            base_evidence_coverage: None,
            facts: Vec::new(),
            diagnostics: Vec::new(),
            source: direct(start, start + 1),
            provenance: provenance(
                format!("checker:template-gate:{instantiation}:{parameter}").as_str(),
            ),
        }
    }

    fn template_sethood_seed(
        parameter: &str,
        evidence_key: &str,
        source_kind: TemplateTypeParameterSethoodSource,
        status: TemplateTypeParameterSethoodStatus,
        start: usize,
    ) -> TemplateTypeParameterSethoodSeed {
        TemplateTypeParameterSethoodSeed {
            parameter: TemplateParameterKey::new(parameter),
            evidence_key: TemplateSethoodEvidenceKey::new(evidence_key),
            normalized_type: NormalizedTypeId::new(start),
            source_kind,
            status,
            facts: Vec::new(),
            diagnostics: Vec::new(),
            source: direct(start, start + 1),
            provenance: provenance(format!("checker:template-sethood:{parameter}").as_str()),
        }
    }

    fn template_fraenkel_sethood(
        parameter: &str,
        evidence_key: &str,
        normalized_type: usize,
        start: usize,
    ) -> TemplateFraenkelSethoodEvidenceSeed {
        TemplateFraenkelSethoodEvidenceSeed {
            parameter: TemplateParameterKey::new(parameter),
            evidence_key: TemplateSethoodEvidenceKey::new(evidence_key),
            normalized_type: NormalizedTypeId::new(normalized_type),
            source: direct(start, start + 1),
            provenance: provenance(format!("checker:fraenkel-template-sethood:{start}").as_str()),
        }
    }

    fn template_scheme_actual(
        instantiation: &str,
        parameter: &str,
        parameter_kind: TemplateSchemeParameterKind,
        actual_kind: TemplateSchemeActualKind,
        status: TemplateSchemeActualStatus,
        arity: usize,
        start: usize,
    ) -> TemplateSchemeActualSeed {
        TemplateSchemeActualSeed {
            instantiation: TemplateInstantiationKey::new(instantiation),
            parameter: TemplateParameterKey::new(parameter),
            parameter_kind,
            actual_kind,
            status,
            expected_arity: arity,
            actual_arity: arity,
            domain_evidence: Vec::new(),
            codomain_evidence: None,
            guard_obligation: None,
            substitution: None,
            checker_diagnostics: Vec::new(),
            source: direct(start, start + 1),
            provenance: provenance(
                format!("checker:scheme-actual:{instantiation}:{parameter}").as_str(),
            ),
        }
    }

    fn widening_evidence(from: usize, to: usize, facts: &[usize]) -> TemplateWideningEvidenceSeed {
        TemplateWideningEvidenceSeed {
            from_type: NormalizedTypeId::new(from),
            to_type: NormalizedTypeId::new(to),
            status: TemplateWideningEvidenceStatus::Accepted,
            facts: facts.iter().copied().map(TypeFactId::new).collect(),
        }
    }

    fn skipped_guard_obligation(start: usize) -> CarriedInitialObligationSeed {
        CarriedInitialObligationSeed {
            checker_obligation: Some(InitialObligationId::new(start)),
            checker_kind: InitialObligationKind::Narrowing,
            status: ObligationSeedStatus::Skipped,
            goal: None,
            context: Vec::new(),
            local_path: format!("scheme-actual-guard/{start}").into(),
            semantic_origin: format!("pkg::main::Owner.scheme-actual-guard.{start}").into(),
            source: direct(start, start + 1),
            provenance: provenance(format!("checker:scheme-actual:guard:{start}").as_str()),
        }
    }

    fn substitution_composition(
        enclosing_parameter: &str,
        start: usize,
    ) -> TemplateSubstitutionCompositionSeed {
        TemplateSubstitutionCompositionSeed {
            enclosing_parameter: TemplateParameterKey::new(enclosing_parameter),
            source: direct(start, start + 1),
            provenance: provenance(
                format!("checker:scheme-actual:substitution:{enclosing_parameter}:{start}")
                    .as_str(),
            ),
        }
    }

    fn source_qua_explanation(
        path: &str,
        functors: &[&str],
        target_type: usize,
        start: usize,
    ) -> ViewExplanationSeed {
        ViewExplanationSeed {
            kind: ViewExplanationKind::SourceQua,
            inserted_view: None,
            target_type: Some(NormalizedTypeId::new(target_type)),
            reduct: Some(reduct_view(path, functors)),
            evidence_facts: Vec::new(),
            source: direct(start, start + 1),
            provenance: provenance(format!("checker:reduct-view:{path}").as_str()),
        }
    }

    fn formula_seed(kind: CoreFormulaSeedKind, start: usize) -> CoreFormulaSeed {
        CoreFormulaSeed::new(
            kind,
            direct(start, start + 1),
            provenance(format!("checker:formula:{start}").as_str()),
        )
    }

    fn failed_site(message: &str, start: usize) -> FailedSemanticSiteSeed {
        FailedSemanticSiteSeed::error(
            message,
            direct(start, start + 1),
            provenance(format!("checker:failed:{message}").as_str()),
        )
    }

    fn expected_checker_source(start: usize, end: usize, key: &str) -> CoreSourceRef {
        source_with_provenance(direct(start, end), &provenance(key))
    }

    fn assert_step2_delta_valid(context: &CoreContext, output: &TypeAndFactLoweringOutput) {
        let mut source_map = output.source_map.clone();
        source_map.item_sources = context.source_map().item_sources.clone();
        let parts = CoreIrParts {
            source_id: context.source_id(),
            module_id: context.module_id().clone(),
            items: context.item_registry().items().clone(),
            terms: output.terms.clone(),
            formulas: output.formulas.clone(),
            definitions: CoreDefinitionTable::new(),
            proofs: CoreProofTable::new(),
            proof_nodes: CoreProofNodeTable::new(),
            algorithms: CoreAlgorithmTable::new(),
            algorithm_statements: CoreAlgorithmStmtTable::new(),
            generated: GeneratedOriginTable::new(),
            obligation_seeds: output.obligation_seeds.clone(),
            source_map,
            diagnostics: output.diagnostics.clone(),
        };
        CoreIr::try_new(parts).expect("step 2 delta validates when merged with context items");
    }

    fn assert_step3_delta_valid(context: &CoreContext, output: &TermAndFormulaLoweringOutput) {
        let parts = CoreIrParts {
            source_id: context.source_id(),
            module_id: context.module_id().clone(),
            items: context.item_registry().items().clone(),
            terms: output.terms.clone(),
            formulas: output.formulas.clone(),
            definitions: CoreDefinitionTable::new(),
            proofs: CoreProofTable::new(),
            proof_nodes: CoreProofNodeTable::new(),
            algorithms: CoreAlgorithmTable::new(),
            algorithm_statements: CoreAlgorithmStmtTable::new(),
            generated: output.generated.clone(),
            obligation_seeds: output.obligation_seeds.clone(),
            source_map: output.source_map.clone(),
            diagnostics: output.diagnostics.clone(),
        };
        CoreIr::try_new(parts).expect("step 3 delta validates when merged with context items");
    }

    fn assert_step4_delta_valid(
        context: &CoreContext,
        term_formula: &TermAndFormulaLoweringOutput,
        output: &DefinitionLoweringOutput,
    ) {
        let parts = CoreIrParts {
            source_id: context.source_id(),
            module_id: context.module_id().clone(),
            items: context.item_registry().items().clone(),
            terms: term_formula.terms.clone(),
            formulas: term_formula.formulas.clone(),
            definitions: output.definitions.clone(),
            proofs: CoreProofTable::new(),
            proof_nodes: CoreProofNodeTable::new(),
            algorithms: CoreAlgorithmTable::new(),
            algorithm_statements: CoreAlgorithmStmtTable::new(),
            generated: term_formula.generated.clone(),
            obligation_seeds: output.obligation_seeds.clone(),
            source_map: output.source_map.clone(),
            diagnostics: output.diagnostics.clone(),
        };
        CoreIr::try_new(parts).expect("step 4 delta validates when merged with prior lowering");
    }

    fn assert_step5_delta_valid(
        context: &CoreContext,
        term_formula: &TermAndFormulaLoweringOutput,
        definitions: &DefinitionLoweringOutput,
        output: &ProofLoweringOutput,
    ) {
        let parts = CoreIrParts {
            source_id: context.source_id(),
            module_id: context.module_id().clone(),
            items: context.item_registry().items().clone(),
            terms: term_formula.terms.clone(),
            formulas: term_formula.formulas.clone(),
            definitions: definitions.definitions.clone(),
            proofs: output.proofs.clone(),
            proof_nodes: output.proof_nodes.clone(),
            algorithms: CoreAlgorithmTable::new(),
            algorithm_statements: CoreAlgorithmStmtTable::new(),
            generated: term_formula.generated.clone(),
            obligation_seeds: output.obligation_seeds.clone(),
            source_map: output.source_map.clone(),
            diagnostics: output.diagnostics.clone(),
        };
        CoreIr::try_new(parts).expect("step 5 delta validates when merged with prior lowering");
    }

    fn assert_step6_delta_valid(
        context: &CoreContext,
        term_formula: &TermAndFormulaLoweringOutput,
        definitions: &DefinitionLoweringOutput,
        proofs: &ProofLoweringOutput,
        output: &AlgorithmLoweringOutput,
    ) {
        let parts = CoreIrParts {
            source_id: context.source_id(),
            module_id: context.module_id().clone(),
            items: context.item_registry().items().clone(),
            terms: term_formula.terms.clone(),
            formulas: term_formula.formulas.clone(),
            definitions: definitions.definitions.clone(),
            proofs: proofs.proofs.clone(),
            proof_nodes: proofs.proof_nodes.clone(),
            algorithms: output.algorithms.clone(),
            algorithm_statements: output.algorithm_statements.clone(),
            generated: term_formula.generated.clone(),
            obligation_seeds: proofs.obligation_seeds.clone(),
            source_map: output.source_map.clone(),
            diagnostics: output.diagnostics.clone(),
        };
        CoreIr::try_new(parts).expect("step 6 delta validates when merged with prior lowering");
    }

    fn test_binder(var: CoreVarId, ty_guard: Option<CoreFormulaId>, start: usize) -> CoreBinder {
        CoreBinder {
            var,
            role: CoreVarRole::new("term-binder"),
            ty_guard,
            source_name: Some(format!("v{}", var.index())),
            source: direct(start, start + 1),
        }
    }

    fn lower_test_terms_and_formulas(
        context: &CoreContext,
        owner: CoreItemId,
        terms: Vec<CoreTermSeed>,
        formulas: Vec<CoreFormulaSeed>,
    ) -> TermAndFormulaLoweringOutput {
        let mut input = TermAndFormulaLoweringInput::new(owner);
        input.terms = terms;
        input.formulas = formulas;
        lower_term_and_formula_inputs(context, input).expect("term/formula lowering")
    }

    fn empty_definition_output(
        context: &CoreContext,
        term_formula: &TermAndFormulaLoweringOutput,
    ) -> DefinitionLoweringOutput {
        lower_definition_inputs(context, term_formula, DefinitionLoweringInput::new())
            .expect("empty definition lowering")
    }

    fn empty_proof_output(
        context: &CoreContext,
        term_formula: &TermAndFormulaLoweringOutput,
        definitions: &DefinitionLoweringOutput,
    ) -> ProofLoweringOutput {
        lower_proof_inputs(
            context,
            term_formula,
            definitions,
            ProofLoweringInput::new(),
        )
        .expect("empty proof lowering")
    }

    fn definition_seed(
        owner: CoreItemId,
        symbol: SymbolId,
        body: DefinitionBodySeed,
        start: usize,
    ) -> DefinitionSeed {
        DefinitionSeed {
            owner,
            symbol,
            params: Vec::new(),
            body,
            expansion: ExpansionPolicy::Opaque,
            correctness: Vec::new(),
            generated_dependencies: Vec::new(),
            source: direct(start, start + 1),
            provenance: provenance(format!("checker:definition:{start}").as_str()),
        }
    }

    fn algorithm_seed(
        owner: CoreItemId,
        symbol: SymbolId,
        payload: AlgorithmPayloadSeed,
        start: usize,
    ) -> AlgorithmSeed {
        AlgorithmSeed {
            owner,
            symbol,
            params: Vec::new(),
            result: None,
            contracts: CoreContractSet::default(),
            payload,
            ghost_effects: Vec::new(),
            source: direct(start, start + 1),
            provenance: provenance(format!("checker:algorithm:{start}").as_str()),
        }
    }

    fn proof_seed(
        owner: CoreItemId,
        symbol: SymbolId,
        proposition: CoreFormulaId,
        status: CoreProofStatus,
        skeleton: ProofSkeletonSeed,
        start: usize,
    ) -> ProofSeed {
        ProofSeed {
            owner,
            symbol,
            proposition,
            status,
            skeleton,
            source: direct(start, start + 1),
            provenance: provenance(format!("checker:proof:{start}").as_str()),
        }
    }

    fn malformed_proof(message: &str, start: usize) -> MalformedProofSkeletonSeed {
        MalformedProofSkeletonSeed::error(
            message,
            direct(start, start + 1),
            provenance(format!("checker:proof:malformed:{message}").as_str()),
        )
    }

    fn proof_step_node(start: usize) -> ProofNodeSeed {
        ProofNodeSeed::Step {
            label: None,
            formula: ProofFormulaRef::Thesis,
            justification: ProofJustificationSeed::new(
                Vec::new(),
                direct(start, start + 1),
                provenance(format!("checker:proof:step:{start}:justification").as_str()),
            ),
            source: direct(start + 1, start + 2),
            provenance: provenance(format!("checker:proof:step:{start}").as_str()),
        }
    }

    fn proof_step_skeleton(start: usize) -> ProofSkeletonSeed {
        ProofSkeletonSeed::Node(proof_step_node(start))
    }

    fn assert_type_predicate(
        output: &TypeAndFactLoweringOutput,
        formula: CoreFormulaId,
        expected_var: CoreVarId,
        expected_predicate: &str,
    ) {
        let CoreFormulaKind::TypePred { subject, ty } =
            &output.formulas.get(formula).expect("type predicate").kind
        else {
            panic!("expected TypePred");
        };
        assert_eq!(ty.as_str(), expected_predicate);
        assert!(matches!(
            output.terms.get(*subject).expect("subject term").kind,
            CoreTermKind::Var(var) if var == expected_var
        ));
    }

    #[test]
    fn declared_binder_type_lowers_to_guard_and_assumption() {
        let var = CoreVarId::new(0);
        let (context, owner) = context_with_var(var);
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.declared_binders = vec![
            DeclaredBinderTypeSeed::new(
                var,
                "term-binder",
                "Nat",
                direct(2, 5),
                provenance("checker:declared-type"),
            )
            .with_source_name("x"),
        ];

        let output = lower_type_and_fact_inputs(&context, input).expect("lowering");
        let guard = output.binder_guards[0].binder.ty_guard.expect("guard");

        assert_eq!(output.assumptions, vec![guard]);
        assert_eq!(output.binder_guards[0].assumption, guard);
        assert_eq!(
            output.binder_guards[0].binder.source_name.as_deref(),
            Some("x")
        );
        assert_type_predicate(&output, guard, var, "Nat");
        assert_step2_delta_valid(&context, &output);
    }

    #[test]
    fn formula_assertion_lowers_to_type_predicate_formula() {
        let var = CoreVarId::new(0);
        let (context, owner) = context_with_var(var);
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.formula_assertions = vec![type_fact(var, "set", 6, Polarity::Positive)];

        let output = lower_type_and_fact_inputs(&context, input).expect("lowering");
        let assertion = output.assertions[0];

        assert_type_predicate(&output, assertion, var, "set");
        assert_step2_delta_valid(&context, &output);
    }

    #[test]
    fn attribute_chains_lower_polarity_and_deterministic_conjunction_order() {
        let var = CoreVarId::new(0);
        let (context, owner) = context_with_var(var);
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.attribute_chains = vec![AttributeChainSeed {
            facts: vec![
                type_fact(var, "Z", 10, Polarity::Positive),
                type_fact(var, "A", 8, Polarity::Negative),
            ],
            source: direct(8, 12),
            provenance: provenance("checker:attribute-chain"),
        }];

        let output = lower_type_and_fact_inputs(&context, input).expect("lowering");
        let conjunction = output.attribute_formulas[0];
        let CoreFormulaKind::And(children) = &output.formulas.get(conjunction).expect("and").kind
        else {
            panic!("expected conjunction");
        };
        assert_eq!(children.len(), 2);
        let CoreFormulaKind::Not(negative_atom) =
            output.formulas.get(children[0]).expect("negative").kind
        else {
            panic!("expected negative attribute");
        };
        assert_type_predicate(&output, negative_atom, var, "A");
        assert_type_predicate(&output, children[1], var, "Z");
        assert_step2_delta_valid(&context, &output);
    }

    #[test]
    fn mode_expansion_uses_checker_normalized_type_id() {
        let var = CoreVarId::new(0);
        let (context, owner) = context_with_var(var);
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.mode_expansions = vec![ModeExpansionSeed {
            subject: var,
            normalized_type: NormalizedTypeId::new(42),
            predicate: CoreTypePredicate::new("mode:Element"),
            source: direct(12, 15),
            provenance: provenance("checker:mode-expansion"),
        }];

        let output = lower_type_and_fact_inputs(&context, input).expect("lowering");
        let lowered = &output.mode_expansions[0];

        assert_eq!(lowered.normalized_type, NormalizedTypeId::new(42));
        assert_type_predicate(&output, lowered.formula, var, "mode:Element");
        assert_step2_delta_valid(&context, &output);
    }

    #[test]
    fn cluster_facts_lower_without_registration_activation() {
        let var = CoreVarId::new(0);
        let (context, owner) = context_with_var(var);
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.cluster_facts = vec![ClusterFactSeed {
            cluster_fact: ClusterFactId::new(3),
            fact: type_fact(var, "cluster:inhabited", 16, Polarity::Positive)
                .with_checker_fact(TypeFactId::new(5)),
        }];

        let output = lower_type_and_fact_inputs(&context, input).expect("lowering");

        assert_eq!(output.cluster_facts.len(), 1);
        assert_eq!(output.cluster_facts[0].cluster_fact, ClusterFactId::new(3));
        assert_type_predicate(
            &output,
            output.cluster_facts[0].formula,
            var,
            "cluster:inhabited",
        );
        assert!(output.obligation_seeds.is_empty());
        assert!(output.diagnostics.is_empty());
        assert_step2_delta_valid(&context, &output);
    }

    #[test]
    fn qua_and_inserted_views_record_provenance_without_cast_or_proof_steps() {
        let var = CoreVarId::new(0);
        let (context, owner) = context_with_var(var);
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.view_explanations = vec![
            ViewExplanationSeed {
                kind: ViewExplanationKind::SourceQua,
                inserted_view: None,
                target_type: Some(NormalizedTypeId::new(1)),
                reduct: None,
                evidence_facts: vec![TypeFactId::new(2), TypeFactId::new(1), TypeFactId::new(1)],
                source: direct(20, 23),
                provenance: provenance("checker:view:source-qua"),
            },
            ViewExplanationSeed {
                kind: ViewExplanationKind::InsertedView,
                inserted_view: Some(CoercionInsertionId::new(0)),
                target_type: Some(NormalizedTypeId::new(2)),
                reduct: None,
                evidence_facts: vec![TypeFactId::new(4)],
                source: direct(24, 25),
                provenance: provenance("checker:view:inserted"),
            },
        ];

        let output = lower_type_and_fact_inputs(&context, input).expect("lowering");

        assert!(output.terms.is_empty());
        assert!(output.formulas.is_empty());
        assert!(output.obligation_seeds.is_empty());
        assert_eq!(
            output.view_explanations[0].kind,
            ViewExplanationKind::SourceQua
        );
        assert_eq!(
            output.view_explanations[0].target_type,
            Some(NormalizedTypeId::new(1))
        );
        assert_eq!(output.view_explanations[0].source, direct(20, 23));
        assert_eq!(
            output.view_explanations[0].provenance,
            vec![CoreProvenance::new(
                CoreProvenancePhase::Checker,
                "checker:view:source-qua"
            )]
        );
        assert_eq!(
            output.view_explanations[0].evidence_facts,
            vec![TypeFactId::new(1), TypeFactId::new(2)]
        );
        assert_eq!(
            output.view_explanations[1].kind,
            ViewExplanationKind::InsertedView
        );
        assert_eq!(
            output.view_explanations[1].inserted_view,
            Some(CoercionInsertionId::new(0))
        );
        assert_eq!(
            output.view_explanations[1].target_type,
            Some(NormalizedTypeId::new(2))
        );
        assert_eq!(
            output.view_explanations[1].evidence_facts,
            vec![TypeFactId::new(4)]
        );
        assert_eq!(output.view_explanations[1].source, direct(24, 25));
        assert_eq!(
            output.view_explanations[1].provenance,
            vec![CoreProvenance::new(
                CoreProvenancePhase::Checker,
                "checker:view:inserted"
            )]
        );
    }

    #[test]
    fn type_fact_lowering_preserves_valid_reduct_view_metadata_without_terms() {
        let var = CoreVarId::new(0);
        let (context, owner) = context_with_var(var);
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.view_explanations = vec![ViewExplanationSeed {
            kind: ViewExplanationKind::SourceQua,
            inserted_view: None,
            target_type: Some(NormalizedTypeId::new(3)),
            reduct: Some(reduct_view("Ring>AddGroup>Magma", &["z_step", "a_step"])),
            evidence_facts: vec![TypeFactId::new(8), TypeFactId::new(8), TypeFactId::new(7)],
            source: direct(26, 27),
            provenance: provenance("checker:view:reduct-metadata"),
        }];

        let output = lower_type_and_fact_inputs(&context, input).expect("lowering");

        assert!(output.terms.is_empty());
        assert!(output.formulas.is_empty());
        assert_eq!(output.view_explanations.len(), 1);
        let reduct = output.view_explanations[0]
            .reduct
            .as_ref()
            .expect("reduct metadata");
        assert_eq!(reduct.path.as_str(), "Ring>AddGroup>Magma");
        assert_eq!(reduct.functors, vec![symbol("z_step"), symbol("a_step")]);
        assert_eq!(
            output.view_explanations[0].evidence_facts,
            vec![TypeFactId::new(7), TypeFactId::new(8)]
        );
        assert_step2_delta_valid(&context, &output);
    }

    #[test]
    fn reconsidering_carries_checker_obligation_seed() {
        let var = CoreVarId::new(0);
        let (context, owner) = context_with_var(var);
        let obligation = active_obligation(var, "narrowed:Nat", 30);
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.reconsiderings = vec![
            ReconsideringSeed::new(
                var,
                "term-binder",
                direct(28, 31),
                provenance("checker:reconsider"),
            )
            .with_predicate("narrowed:Nat")
            .with_obligation(obligation),
        ];

        let output = lower_type_and_fact_inputs(&context, input).expect("lowering");
        let reconsidered = &output.reconsidered_binders[0];
        let obligation = reconsidered.obligation.expect("obligation");
        let seed = output
            .obligation_seeds
            .get(obligation)
            .expect("obligation seed");

        assert_eq!(reconsidered.binder.var, var);
        assert_eq!(reconsidered.binder.role, CoreVarRole::new("term-binder"));
        assert_eq!(reconsidered.binder.source, direct(28, 31));
        assert!(reconsidered.binder.ty_guard.is_some());
        assert_type_predicate(
            &output,
            reconsidered.binder.ty_guard.expect("guard"),
            var,
            "narrowed:Nat",
        );
        assert_eq!(seed.status, ObligationSeedStatus::Active);
        assert_eq!(seed.kind, ObligationSeedKind::CheckerInitial);
        assert!(seed.goal.is_some());
        assert_step2_delta_valid(&context, &output);
    }

    #[test]
    fn standalone_carried_obligations_populate_output_vector() {
        let var = CoreVarId::new(0);
        let (context, owner) = context_with_var(var);
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.carried_obligations = vec![active_obligation(var, "standalone:goal", 34)];

        let output = lower_type_and_fact_inputs(&context, input).expect("lowering");
        let obligation = output.carried_obligations[0];
        let seed = output
            .obligation_seeds
            .get(obligation)
            .expect("obligation seed");

        assert_eq!(output.carried_obligations, vec![obligation]);
        assert_eq!(seed.status, ObligationSeedStatus::Active);
        assert!(seed.goal.is_some());
        assert_step2_delta_valid(&context, &output);
    }

    #[test]
    fn missing_evidence_emits_diagnostic_and_deferred_seed_without_proving() {
        let var = CoreVarId::new(0);
        let (context, owner) = context_with_var(var);
        let deferred = CarriedInitialObligationSeed {
            checker_obligation: Some(InitialObligationId::new(99)),
            checker_kind: InitialObligationKind::Sethood,
            status: ObligationSeedStatus::Deferred,
            goal: None,
            context: vec![ObligationFormulaSeed::positive(var, "set", direct(35, 36))],
            local_path: LocalProofOrProgramPath::new("type/missing/sethood"),
            semantic_origin: NormalizedSemanticOrigin::new("pkg::main::Owner.missing-sethood"),
            source: direct(35, 36),
            provenance: provenance("checker:missing:sethood"),
        };
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.missing_evidence = vec![MissingEvidenceSeed {
            kind: MissingEvidenceKind::Sethood,
            diagnostic: Some(TypeDiagnosticId::new(7)),
            deferred_obligation: Some(deferred),
            source: direct(35, 36),
            provenance: provenance("checker:missing-evidence"),
        }];

        let output = lower_type_and_fact_inputs(&context, input).expect("lowering");
        let missing = &output.missing_evidence[0];
        let diagnostic = output
            .diagnostics
            .get(missing.diagnostic)
            .expect("diagnostic");
        let obligation = missing.obligation.expect("deferred seed");
        let seed = output
            .obligation_seeds
            .get(obligation)
            .expect("obligation seed");

        assert_eq!(missing.checker_diagnostic, Some(TypeDiagnosticId::new(7)));
        assert!(!missing.provenance.is_empty());
        assert_eq!(diagnostic.message_key.as_str(), "missing-sethood-evidence");
        assert_eq!(seed.status, ObligationSeedStatus::Deferred);
        assert_eq!(seed.kind, ObligationSeedKind::GeneratedSethood);
        assert!(seed.goal.is_none());
        assert_eq!(seed.diagnostics, vec![missing.diagnostic]);
        assert_step2_delta_valid(&context, &output);
    }

    #[test]
    fn missing_evidence_matrix_preserves_each_required_category() {
        let var = CoreVarId::new(0);
        let (context, owner) = context_with_var(var);
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.missing_evidence = vec![
            MissingEvidenceSeed {
                kind: MissingEvidenceKind::NonEmptiness,
                diagnostic: Some(TypeDiagnosticId::new(11)),
                deferred_obligation: None,
                source: direct(40, 41),
                provenance: provenance("checker:missing:non-empty"),
            },
            MissingEvidenceSeed {
                kind: MissingEvidenceKind::Coercion,
                diagnostic: Some(TypeDiagnosticId::new(12)),
                deferred_obligation: None,
                source: direct(42, 43),
                provenance: provenance("checker:missing:coercion"),
            },
            MissingEvidenceSeed {
                kind: MissingEvidenceKind::Cluster,
                diagnostic: Some(TypeDiagnosticId::new(13)),
                deferred_obligation: None,
                source: direct(44, 45),
                provenance: provenance("checker:missing:cluster"),
            },
        ];

        let output = lower_type_and_fact_inputs(&context, input).expect("lowering");
        let messages = output
            .missing_evidence
            .iter()
            .map(|missing| {
                output
                    .diagnostics
                    .get(missing.diagnostic)
                    .expect("diagnostic")
                    .message_key
                    .as_str()
                    .to_owned()
            })
            .collect::<Vec<_>>();

        assert_eq!(
            messages,
            vec![
                "missing-non-emptiness-evidence",
                "missing-coercion-evidence",
                "missing-cluster-evidence"
            ]
        );
        assert_eq!(
            output
                .missing_evidence
                .iter()
                .map(|missing| missing.checker_diagnostic)
                .collect::<Vec<_>>(),
            vec![
                Some(TypeDiagnosticId::new(11)),
                Some(TypeDiagnosticId::new(12)),
                Some(TypeDiagnosticId::new(13))
            ]
        );
        assert!(output.obligation_seeds.is_empty());
        assert_step2_delta_valid(&context, &output);
    }

    #[test]
    fn template_type_parameter_inhabitation_emits_schema_exists_assumption() {
        let (context, owner) = context_with_var(CoreVarId::new(0));
        let witness = CoreVarId::new(42);
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.template_type_parameters = vec![template_type_parameter("T", witness, "is_T", 46)];

        let output = lower_type_and_fact_inputs(&context, input).expect("lowering");
        assert_eq!(output.template_type_parameter_inhabitations.len(), 1);
        assert_eq!(output.assumptions.len(), 1);
        let lowered = &output.template_type_parameter_inhabitations[0];
        assert_eq!(lowered.parameter, TemplateParameterKey::new("T"));
        assert_eq!(lowered.assumption, output.assumptions[0]);
        assert_eq!(lowered.witness.var, witness);
        assert_eq!(
            lowered.witness.role,
            CoreVarRole::new("template-type-witness")
        );
        assert_eq!(lowered.witness.source_name.as_deref(), Some("T$inhabitant"));

        let CoreFormulaKind::Exists { binders, body } = &output
            .formulas
            .get(lowered.assumption)
            .expect("exists formula")
            .kind
        else {
            panic!("schema parameter assumption must be an existential formula");
        };
        assert_eq!(binders, &vec![lowered.witness.clone()]);
        assert_eq!(*body, lowered.witness_fact);
        let CoreFormulaKind::TypePred { subject, ty } = &output
            .formulas
            .get(lowered.witness_fact)
            .expect("witness fact")
            .kind
        else {
            panic!("schema witness body must be a type predicate");
        };
        assert_eq!(*subject, lowered.witness_term);
        assert_eq!(ty, &CoreTypePredicate::new("is_T"));
        assert!(matches!(
            output.terms.get(lowered.witness_term).expect("witness term").kind,
            CoreTermKind::Var(actual) if actual == witness
        ));
        assert!(output.template_type_actual_gates.is_empty());
        assert!(output.diagnostics.is_empty());
        assert!(output.obligation_seeds.is_empty());
        assert_step2_delta_valid(&context, &output);
    }

    #[test]
    fn template_type_actual_gates_preserve_accepted_evidence_without_axioms() {
        let (context, owner) = context_with_var(CoreVarId::new(0));
        let mut registration_gate = template_type_actual_gate(
            "Inhab[prime Nat]",
            "T",
            ExistentialGateStatus::Satisfied,
            47,
        );
        registration_gate.registration = Some(CheckerRegistrationId::new(5));
        let mut base_gate =
            template_type_actual_gate("Inhab[set]", "U", ExistentialGateStatus::Satisfied, 48);
        base_gate.base_evidence_kind = Some(ExistentialGateBaseEvidenceKind::BuiltinSet);
        base_gate.base_evidence_coverage = Some(ExistentialGateBaseEvidenceCoverage::Builtin);
        let mut guard_gate =
            template_type_actual_gate("Inhab[Struct]", "V", ExistentialGateStatus::Satisfied, 49);
        guard_gate.facts = vec![TypeFactId::new(9), TypeFactId::new(3), TypeFactId::new(3)];
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.template_type_actual_gates = vec![registration_gate, base_gate, guard_gate];

        let output = lower_type_and_fact_inputs(&context, input).expect("lowering");
        assert_eq!(output.template_type_actual_gates.len(), 3);
        assert!(output.assumptions.is_empty());
        assert!(output.terms.is_empty());
        assert!(output.formulas.is_empty());
        assert!(output.diagnostics.is_empty());
        assert!(output.obligation_seeds.is_empty());

        assert_eq!(
            output.template_type_actual_gates[0].registration,
            Some(CheckerRegistrationId::new(5))
        );
        assert_eq!(
            output.template_type_actual_gates[1].base_evidence_kind,
            Some(ExistentialGateBaseEvidenceKind::BuiltinSet)
        );
        assert_eq!(
            output.template_type_actual_gates[1].base_evidence_coverage,
            Some(ExistentialGateBaseEvidenceCoverage::Builtin)
        );
        assert_eq!(
            output.template_type_actual_gates[2].facts,
            vec![TypeFactId::new(3), TypeFactId::new(9)]
        );
        assert!(
            output
                .template_type_actual_gates
                .iter()
                .all(|gate| gate.status == ExistentialGateStatus::Satisfied
                    && gate.diagnostic.is_none()
                    && !gate.provenance.is_empty())
        );
        assert_step2_delta_valid(&context, &output);
    }

    #[test]
    fn missing_template_type_actual_gate_rejects_without_existential_axiom() {
        let (context, owner) = context_with_var(CoreVarId::new(0));
        let mut missing = template_type_actual_gate(
            "Inhab[hollow set]",
            "T",
            ExistentialGateStatus::MissingExistential,
            50,
        );
        missing.diagnostics = vec![RegistrationDiagnosticId::new(12)];
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.template_type_actual_gates = vec![missing];

        let output = lower_type_and_fact_inputs(&context, input).expect("lowering");
        assert_eq!(output.template_type_actual_gates.len(), 1);
        let gate = &output.template_type_actual_gates[0];
        let diagnostic = gate.diagnostic.expect("core diagnostic");
        assert_eq!(
            gate.checker_diagnostics,
            vec![RegistrationDiagnosticId::new(12)]
        );
        assert_eq!(
            output
                .diagnostics
                .get(diagnostic)
                .expect("diagnostic")
                .message_key
                .as_str(),
            "missing-template-type-actual-inhabitation"
        );
        assert!(output.assumptions.is_empty());
        assert!(output.terms.is_empty());
        assert!(output.formulas.is_empty());
        assert!(output.obligation_seeds.is_empty());
        assert_step2_delta_valid(&context, &output);
    }

    #[test]
    fn unsatisfied_template_type_actual_gate_status_matrix_is_diagnostic_only() {
        let (context, owner) = context_with_var(CoreVarId::new(0));
        let cases = [
            (
                ExistentialGateStatus::MissingExistential,
                "missing-template-type-actual-inhabitation",
            ),
            (
                ExistentialGateStatus::BlockedGuard,
                "blocked-template-type-actual-inhabitation-guard",
            ),
            (
                ExistentialGateStatus::InvalidCandidate,
                "invalid-template-type-actual-inhabitation-candidate",
            ),
            (
                ExistentialGateStatus::DegradedRecovery,
                "degraded-template-type-actual-inhabitation",
            ),
        ];
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.template_type_actual_gates = cases
            .iter()
            .enumerate()
            .map(|(offset, (status, _))| {
                let mut seed = template_type_actual_gate(
                    format!("Inhab[case{offset}]").as_str(),
                    format!("T{offset}").as_str(),
                    *status,
                    56 + offset,
                );
                seed.diagnostics = vec![RegistrationDiagnosticId::new(100 + offset)];
                seed
            })
            .collect();

        let output = lower_type_and_fact_inputs(&context, input).expect("lowering");
        assert_eq!(output.template_type_actual_gates.len(), cases.len());
        for (gate, (status, message)) in output.template_type_actual_gates.iter().zip(cases) {
            assert_eq!(gate.status, status);
            assert!(gate.registration.is_none());
            assert!(gate.base_evidence_kind.is_none());
            assert!(gate.base_evidence_coverage.is_none());
            assert!(gate.facts.is_empty());
            let diagnostic = gate.diagnostic.expect("diagnostic-only rejected gate");
            assert_eq!(
                output
                    .diagnostics
                    .get(diagnostic)
                    .expect("diagnostic")
                    .message_key
                    .as_str(),
                message
            );
        }
        assert!(output.assumptions.is_empty());
        assert!(output.terms.is_empty());
        assert!(output.formulas.is_empty());
        assert!(output.obligation_seeds.is_empty());
        assert_step2_delta_valid(&context, &output);
    }

    #[test]
    fn template_type_actual_gate_payloads_fail_closed() {
        let (context, owner) = context_with_var(CoreVarId::new(0));
        let no_evidence = template_type_actual_gate(
            "Inhab[NoEvidence]",
            "T",
            ExistentialGateStatus::Satisfied,
            51,
        );
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.template_type_actual_gates = vec![no_evidence];
        assert!(matches!(
            lower_type_and_fact_inputs(&context, input),
            Err(TypeAndFactLoweringError::SatisfiedTemplateTypeActualWithoutEvidence { .. })
        ));

        let mut partial_base =
            template_type_actual_gate("Inhab[partial]", "T", ExistentialGateStatus::Satisfied, 52);
        partial_base.base_evidence_kind = Some(ExistentialGateBaseEvidenceKind::BuiltinObject);
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.template_type_actual_gates = vec![partial_base];
        assert!(matches!(
            lower_type_and_fact_inputs(&context, input),
            Err(TypeAndFactLoweringError::PartialTemplateTypeActualBaseEvidence { .. })
        ));

        let mut unsatisfied_with_registration = template_type_actual_gate(
            "Inhab[hollow set]",
            "T",
            ExistentialGateStatus::MissingExistential,
            53,
        );
        unsatisfied_with_registration.registration = Some(CheckerRegistrationId::new(9));
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.template_type_actual_gates = vec![unsatisfied_with_registration];
        assert!(matches!(
            lower_type_and_fact_inputs(&context, input),
            Err(
                TypeAndFactLoweringError::UnsatisfiedTemplateTypeActualCarriesEvidence {
                    status: ExistentialGateStatus::MissingExistential,
                    ..
                }
            )
        ));

        let mut unsatisfied_with_base = template_type_actual_gate(
            "Inhab[blocked-base]",
            "T",
            ExistentialGateStatus::BlockedGuard,
            60,
        );
        unsatisfied_with_base.base_evidence_kind =
            Some(ExistentialGateBaseEvidenceKind::BuiltinObject);
        unsatisfied_with_base.base_evidence_coverage =
            Some(ExistentialGateBaseEvidenceCoverage::Builtin);
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.template_type_actual_gates = vec![unsatisfied_with_base];
        assert!(matches!(
            lower_type_and_fact_inputs(&context, input),
            Err(
                TypeAndFactLoweringError::UnsatisfiedTemplateTypeActualCarriesEvidence {
                    status: ExistentialGateStatus::BlockedGuard,
                    ..
                }
            )
        ));

        let mut unsatisfied_with_facts = template_type_actual_gate(
            "Inhab[invalid-fact]",
            "T",
            ExistentialGateStatus::InvalidCandidate,
            61,
        );
        unsatisfied_with_facts.facts = vec![TypeFactId::new(10)];
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.template_type_actual_gates = vec![unsatisfied_with_facts];
        assert!(matches!(
            lower_type_and_fact_inputs(&context, input),
            Err(
                TypeAndFactLoweringError::UnsatisfiedTemplateTypeActualCarriesEvidence {
                    status: ExistentialGateStatus::InvalidCandidate,
                    ..
                }
            )
        ));

        let mut first =
            template_type_actual_gate("Inhab[set]", "T", ExistentialGateStatus::Satisfied, 62);
        first.registration = Some(CheckerRegistrationId::new(1));
        let mut second =
            template_type_actual_gate("Inhab[set]", "T", ExistentialGateStatus::Satisfied, 63);
        second.registration = Some(CheckerRegistrationId::new(2));
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.template_type_actual_gates = vec![first, second];
        assert!(matches!(
            lower_type_and_fact_inputs(&context, input),
            Err(TypeAndFactLoweringError::DuplicateTemplateTypeActualGate { .. })
        ));
    }

    #[test]
    fn template_type_parameter_sethood_records_bound_constraint_and_bare_missing() {
        let (context, owner) = context_with_var(CoreVarId::new(0));
        let mut bound = template_sethood_seed(
            "T",
            "T:bound:Nat",
            TemplateTypeParameterSethoodSource::BoundInherited,
            TemplateTypeParameterSethoodStatus::Accepted,
            64,
        );
        bound.facts = vec![TypeFactId::new(9), TypeFactId::new(8), TypeFactId::new(9)];
        bound.diagnostics = vec![TypeDiagnosticId::new(4), TypeDiagnosticId::new(4)];
        let mut constraint = template_sethood_seed(
            "U",
            "U:constraint:sethood",
            TemplateTypeParameterSethoodSource::ConstraintSupplied,
            TemplateTypeParameterSethoodStatus::Accepted,
            65,
        );
        constraint.facts = vec![TypeFactId::new(10)];
        let bare = template_sethood_seed(
            "Bare",
            "Bare:bare",
            TemplateTypeParameterSethoodSource::BareParameter,
            TemplateTypeParameterSethoodStatus::Missing,
            66,
        );

        let mut input = TypeAndFactLoweringInput::new(owner);
        input.template_type_parameter_sethoods = vec![bound, constraint, bare];

        let output = lower_type_and_fact_inputs(&context, input).expect("lowering");

        assert_eq!(output.template_type_parameter_sethoods.len(), 3);
        let bound = &output.template_type_parameter_sethoods[0];
        assert_eq!(bound.parameter.as_str(), "T");
        assert_eq!(bound.evidence_key.as_str(), "T:bound:Nat");
        assert_eq!(
            bound.source_kind,
            TemplateTypeParameterSethoodSource::BoundInherited
        );
        assert_eq!(bound.status, TemplateTypeParameterSethoodStatus::Accepted);
        assert_eq!(bound.facts, vec![TypeFactId::new(8), TypeFactId::new(9)]);
        assert_eq!(bound.checker_diagnostics, vec![TypeDiagnosticId::new(4)]);
        assert!(bound.diagnostic.is_none());

        let constraint = &output.template_type_parameter_sethoods[1];
        assert_eq!(constraint.parameter.as_str(), "U");
        assert_eq!(constraint.evidence_key.as_str(), "U:constraint:sethood");
        assert_eq!(
            constraint.source_kind,
            TemplateTypeParameterSethoodSource::ConstraintSupplied
        );
        assert_eq!(
            constraint.status,
            TemplateTypeParameterSethoodStatus::Accepted
        );
        assert_eq!(constraint.facts, vec![TypeFactId::new(10)]);
        assert!(constraint.diagnostic.is_none());

        let bare = &output.template_type_parameter_sethoods[2];
        assert_eq!(bare.parameter.as_str(), "Bare");
        assert_eq!(
            bare.source_kind,
            TemplateTypeParameterSethoodSource::BareParameter
        );
        assert_eq!(bare.status, TemplateTypeParameterSethoodStatus::Missing);
        assert!(bare.facts.is_empty());
        let diagnostic = output
            .diagnostics
            .get(bare.diagnostic.expect("bare diagnostic"))
            .expect("diagnostic");
        assert_eq!(
            diagnostic.message_key.as_str(),
            "missing-template-type-parameter-sethood"
        );
        assert!(output.assumptions.is_empty());
        assert!(output.terms.is_empty());
        assert!(output.formulas.is_empty());
        assert!(output.obligation_seeds.is_empty());
        assert_step2_delta_valid(&context, &output);
    }

    #[test]
    fn template_type_parameter_sethood_payloads_fail_closed() {
        let (context, owner) = context_with_var(CoreVarId::new(0));

        let mut accepted_bare = template_sethood_seed(
            "Bare",
            "Bare:accepted",
            TemplateTypeParameterSethoodSource::BareParameter,
            TemplateTypeParameterSethoodStatus::Accepted,
            67,
        );
        accepted_bare.facts = vec![TypeFactId::new(11)];
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.template_type_parameter_sethoods = vec![accepted_bare];
        assert!(matches!(
            lower_type_and_fact_inputs(&context, input),
            Err(TypeAndFactLoweringError::BareTemplateTypeParameterSethoodAccepted { .. })
        ));

        let no_facts = template_sethood_seed(
            "T",
            "T:no-facts",
            TemplateTypeParameterSethoodSource::BoundInherited,
            TemplateTypeParameterSethoodStatus::Accepted,
            68,
        );
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.template_type_parameter_sethoods = vec![no_facts];
        assert!(matches!(
            lower_type_and_fact_inputs(&context, input),
            Err(
                TypeAndFactLoweringError::AcceptedTemplateTypeParameterSethoodWithoutEvidence { .. }
            )
        ));

        let mut missing_with_facts = template_sethood_seed(
            "Bare",
            "Bare:missing-with-facts",
            TemplateTypeParameterSethoodSource::BareParameter,
            TemplateTypeParameterSethoodStatus::Missing,
            69,
        );
        missing_with_facts.facts = vec![TypeFactId::new(12)];
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.template_type_parameter_sethoods = vec![missing_with_facts];
        assert!(matches!(
            lower_type_and_fact_inputs(&context, input),
            Err(
                TypeAndFactLoweringError::MissingTemplateTypeParameterSethoodCarriesEvidence {
                    status: TemplateTypeParameterSethoodStatus::Missing,
                    ..
                }
            )
        ));

        let missing_bound = template_sethood_seed(
            "T",
            "T:missing-bound",
            TemplateTypeParameterSethoodSource::BoundInherited,
            TemplateTypeParameterSethoodStatus::Missing,
            70,
        );
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.template_type_parameter_sethoods = vec![missing_bound];
        assert!(matches!(
            lower_type_and_fact_inputs(&context, input),
            Err(
                TypeAndFactLoweringError::MissingTemplateTypeParameterSethoodWrongSource {
                    source_kind: TemplateTypeParameterSethoodSource::BoundInherited,
                    ..
                }
            )
        ));

        let mut degraded_with_facts = template_sethood_seed(
            "T",
            "T:degraded",
            TemplateTypeParameterSethoodSource::ConstraintSupplied,
            TemplateTypeParameterSethoodStatus::DegradedRecovery,
            71,
        );
        degraded_with_facts.facts = vec![TypeFactId::new(13)];
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.template_type_parameter_sethoods = vec![degraded_with_facts];
        assert!(matches!(
            lower_type_and_fact_inputs(&context, input),
            Err(
                TypeAndFactLoweringError::MissingTemplateTypeParameterSethoodCarriesEvidence {
                    status: TemplateTypeParameterSethoodStatus::DegradedRecovery,
                    ..
                }
            )
        ));

        let mut first = template_sethood_seed(
            "T",
            "T:dup",
            TemplateTypeParameterSethoodSource::BoundInherited,
            TemplateTypeParameterSethoodStatus::Accepted,
            72,
        );
        first.facts = vec![TypeFactId::new(14)];
        let mut second = first.clone();
        second.source = direct(73, 74);
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.template_type_parameter_sethoods = vec![first, second];
        assert!(matches!(
            lower_type_and_fact_inputs(&context, input),
            Err(TypeAndFactLoweringError::DuplicateTemplateTypeParameterSethood { .. })
        ));

        let mut accepted = template_sethood_seed(
            "T",
            "T:conflict",
            TemplateTypeParameterSethoodSource::BoundInherited,
            TemplateTypeParameterSethoodStatus::Accepted,
            74,
        );
        accepted.facts = vec![TypeFactId::new(15)];
        let conflicting_missing = template_sethood_seed(
            "T",
            "T:conflict",
            TemplateTypeParameterSethoodSource::BareParameter,
            TemplateTypeParameterSethoodStatus::Missing,
            75,
        );
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.template_type_parameter_sethoods = vec![accepted, conflicting_missing];
        assert!(matches!(
            lower_type_and_fact_inputs(&context, input),
            Err(TypeAndFactLoweringError::DuplicateTemplateTypeParameterSethood { .. })
        ));
    }

    #[test]
    fn template_scheme_deffunc_actual_preserves_signature_evidence_and_guard_seed() {
        let (context, owner) = context_with_var(CoreVarId::new(0));
        let mut actual = template_scheme_actual(
            "Iter[double]",
            "F",
            TemplateSchemeParameterKind::Functor,
            TemplateSchemeActualKind::Deffunc,
            TemplateSchemeActualStatus::Accepted,
            1,
            70,
        );
        actual.domain_evidence = vec![widening_evidence(10, 11, &[9, 3, 3])];
        actual.codomain_evidence = Some(widening_evidence(12, 13, &[7]));
        actual.guard_obligation = Some(skipped_guard_obligation(71));
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.template_scheme_actuals = vec![actual];

        let output = lower_type_and_fact_inputs(&context, input).expect("lowering");
        assert_eq!(output.template_scheme_actuals.len(), 1);
        let lowered = &output.template_scheme_actuals[0];
        assert_eq!(lowered.actual_kind, TemplateSchemeActualKind::Deffunc);
        assert_eq!(lowered.status, TemplateSchemeActualStatus::Accepted);
        assert_eq!(
            lowered.domain_evidence,
            vec![TemplateDirectionalWideningEvidence {
                from_type: NormalizedTypeId::new(10),
                to_type: NormalizedTypeId::new(11),
                status: TemplateWideningEvidenceStatus::Accepted,
                facts: vec![TypeFactId::new(3), TypeFactId::new(9)],
            }]
        );
        assert_eq!(
            lowered.codomain_evidence,
            Some(TemplateDirectionalWideningEvidence {
                from_type: NormalizedTypeId::new(12),
                to_type: NormalizedTypeId::new(13),
                status: TemplateWideningEvidenceStatus::Accepted,
                facts: vec![TypeFactId::new(7)],
            })
        );

        let guard = lowered.guard_obligation.expect("skipped guard seed");
        let guard_seed = output
            .obligation_seeds
            .get(guard)
            .expect("guard obligation");
        assert_eq!(guard_seed.kind, ObligationSeedKind::CheckerInitial);
        assert_eq!(guard_seed.status, ObligationSeedStatus::Skipped);
        assert!(guard_seed.goal.is_none());
        assert!(output.assumptions.is_empty());
        assert!(output.terms.is_empty());
        assert!(output.formulas.is_empty());
        assert!(output.diagnostics.is_empty());
        assert_step2_delta_valid(&context, &output);
    }

    #[test]
    fn rejected_template_scheme_actuals_are_diagnostic_only() {
        let (context, owner) = context_with_var(CoreVarId::new(0));
        let result_mismatch = template_scheme_actual(
            "Iter[shrink]",
            "F",
            TemplateSchemeParameterKind::Functor,
            TemplateSchemeActualKind::Deffunc,
            TemplateSchemeActualStatus::SignatureMismatch,
            1,
            72,
        );
        let defpred_mismatch = template_scheme_actual(
            "Induction[narrow]",
            "P",
            TemplateSchemeParameterKind::Predicate,
            TemplateSchemeActualKind::Defpred,
            TemplateSchemeActualStatus::SignatureMismatch,
            1,
            73,
        );
        let mut partial_algorithm = template_scheme_actual(
            "Sigma[partial]",
            "F",
            TemplateSchemeParameterKind::Functor,
            TemplateSchemeActualKind::PartialAlgorithm,
            TemplateSchemeActualStatus::PartialAlgorithm,
            1,
            74,
        );
        partial_algorithm.checker_diagnostics = vec![TypeDiagnosticId::new(8)];
        let void_algorithm = template_scheme_actual(
            "Sigma[void]",
            "F3",
            TemplateSchemeParameterKind::Functor,
            TemplateSchemeActualKind::VoidAlgorithm,
            TemplateSchemeActualStatus::VoidAlgorithm,
            1,
            75,
        );
        let unsupported_actual = template_scheme_actual(
            "Scheme[template-predicate]",
            "P2",
            TemplateSchemeParameterKind::Predicate,
            TemplateSchemeActualKind::Unsupported,
            TemplateSchemeActualStatus::Unsupported,
            1,
            76,
        );
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.template_scheme_actuals = vec![
            result_mismatch,
            defpred_mismatch,
            partial_algorithm,
            void_algorithm,
            unsupported_actual,
        ];

        let output = lower_type_and_fact_inputs(&context, input).expect("lowering");
        assert_eq!(output.template_scheme_actuals.len(), 5);
        let messages: Vec<_> = output
            .template_scheme_actuals
            .iter()
            .map(|actual| {
                let diagnostic = actual.diagnostic.expect("diagnostic-only rejection");
                output
                    .diagnostics
                    .get(diagnostic)
                    .expect("diagnostic")
                    .message_key
                    .as_str()
                    .to_owned()
            })
            .collect();
        assert_eq!(
            messages,
            vec![
                "template-scheme-actual-signature-mismatch",
                "template-scheme-actual-signature-mismatch",
                "partial-algorithm-template-functor-actual",
                "void-algorithm-template-functor-actual",
                "unsupported-template-scheme-actual",
            ]
        );
        assert_eq!(
            output.template_scheme_actuals[2].checker_diagnostics,
            vec![TypeDiagnosticId::new(8)]
        );
        assert!(output.template_scheme_actuals.iter().all(
            |actual| actual.domain_evidence.is_empty()
                && actual.codomain_evidence.is_none()
                && actual.guard_obligation.is_none()
                && actual.substitution.is_none()
        ));
        assert!(output.assumptions.is_empty());
        assert!(output.terms.is_empty());
        assert!(output.formulas.is_empty());
        assert!(output.obligation_seeds.is_empty());
        assert_step2_delta_valid(&context, &output);
    }

    #[test]
    fn accepted_plain_template_scheme_type_actual_preserves_type_row_without_callable_evidence() {
        let (context, owner) = context_with_var(CoreVarId::new(0));
        let type_actual = template_scheme_actual(
            "Choice[Nat]",
            "T",
            TemplateSchemeParameterKind::Type,
            TemplateSchemeActualKind::TypeExpression,
            TemplateSchemeActualStatus::Accepted,
            0,
            74,
        );

        let mut input = TypeAndFactLoweringInput::new(owner);
        input.template_scheme_actuals = vec![type_actual];

        let output = lower_type_and_fact_inputs(&context, input).expect("lowering");
        assert_eq!(output.template_scheme_actuals.len(), 1);
        let lowered = &output.template_scheme_actuals[0];
        assert_eq!(lowered.parameter_kind, TemplateSchemeParameterKind::Type);
        assert_eq!(
            lowered.actual_kind,
            TemplateSchemeActualKind::TypeExpression
        );
        assert_eq!(lowered.status, TemplateSchemeActualStatus::Accepted);
        assert!(lowered.domain_evidence.is_empty());
        assert!(lowered.codomain_evidence.is_none());
        assert!(lowered.guard_obligation.is_none());
        assert!(lowered.substitution.is_none());
        assert!(output.assumptions.is_empty());
        assert!(output.terms.is_empty());
        assert!(output.formulas.is_empty());
        assert!(output.obligation_seeds.is_empty());
        assert!(output.diagnostics.is_empty());
        assert_step2_delta_valid(&context, &output);
    }

    #[test]
    fn accepted_defpred_template_functor_and_promoted_algorithm_actuals() {
        let (context, owner) = context_with_var(CoreVarId::new(0));
        let mut pred = template_scheme_actual(
            "Induction[IsOdd]",
            "P",
            TemplateSchemeParameterKind::Predicate,
            TemplateSchemeActualKind::Defpred,
            TemplateSchemeActualStatus::Accepted,
            1,
            75,
        );
        pred.domain_evidence = vec![widening_evidence(20, 21, &[11])];

        let mut template_functor = template_scheme_actual(
            "Sigma[Square]",
            "F",
            TemplateSchemeParameterKind::Functor,
            TemplateSchemeActualKind::TemplateFunctor,
            TemplateSchemeActualStatus::Accepted,
            1,
            76,
        );
        template_functor.domain_evidence = vec![widening_evidence(22, 23, &[12])];
        template_functor.codomain_evidence = Some(widening_evidence(24, 25, &[13]));
        template_functor.guard_obligation = Some(skipped_guard_obligation(77));

        let mut promoted_algorithm = template_scheme_actual(
            "Sigma[factorial]",
            "F2",
            TemplateSchemeParameterKind::Functor,
            TemplateSchemeActualKind::PromotedTerminatingAlgorithm,
            TemplateSchemeActualStatus::Accepted,
            1,
            78,
        );
        promoted_algorithm.domain_evidence = vec![widening_evidence(26, 27, &[14])];
        promoted_algorithm.codomain_evidence = Some(widening_evidence(28, 29, &[15]));
        promoted_algorithm.guard_obligation = Some(skipped_guard_obligation(79));

        let mut input = TypeAndFactLoweringInput::new(owner);
        input.template_scheme_actuals = vec![pred, template_functor, promoted_algorithm];

        let output = lower_type_and_fact_inputs(&context, input).expect("lowering");
        assert_eq!(output.template_scheme_actuals.len(), 3);
        assert_eq!(
            output.template_scheme_actuals[0].actual_kind,
            TemplateSchemeActualKind::Defpred
        );
        assert!(output.template_scheme_actuals[0].guard_obligation.is_none());
        assert!(
            output.template_scheme_actuals[0]
                .codomain_evidence
                .is_none()
        );
        assert_eq!(
            output.template_scheme_actuals[1].actual_kind,
            TemplateSchemeActualKind::TemplateFunctor
        );
        assert_eq!(
            output.template_scheme_actuals[2].actual_kind,
            TemplateSchemeActualKind::PromotedTerminatingAlgorithm
        );
        assert_eq!(output.obligation_seeds.iter().count(), 2);
        assert!(
            output
                .obligation_seeds
                .iter()
                .all(|(_, seed)| seed.status == ObligationSeedStatus::Skipped
                    && seed.kind == ObligationSeedKind::CheckerInitial)
        );
        assert!(output.diagnostics.is_empty());
        assert_step2_delta_valid(&context, &output);
    }

    #[test]
    fn enclosing_scheme_parameters_preserve_substitution_metadata_without_symbols() {
        let (context, owner) = context_with_var(CoreVarId::new(0));
        let mut type_actual = template_scheme_actual(
            "Inner[T]",
            "T",
            TemplateSchemeParameterKind::Type,
            TemplateSchemeActualKind::EnclosingTypeParameter,
            TemplateSchemeActualStatus::Accepted,
            0,
            80,
        );
        type_actual.substitution = Some(substitution_composition("OuterT", 81));

        let mut pred_actual = template_scheme_actual(
            "Inner[P]",
            "P",
            TemplateSchemeParameterKind::Predicate,
            TemplateSchemeActualKind::EnclosingPredicateParameter,
            TemplateSchemeActualStatus::Accepted,
            1,
            82,
        );
        pred_actual.domain_evidence = vec![widening_evidence(30, 31, &[16])];
        pred_actual.substitution = Some(substitution_composition("OuterP", 83));

        let mut functor_actual = template_scheme_actual(
            "Inner[F]",
            "F",
            TemplateSchemeParameterKind::Functor,
            TemplateSchemeActualKind::EnclosingFunctorParameter,
            TemplateSchemeActualStatus::Accepted,
            1,
            84,
        );
        functor_actual.domain_evidence = vec![widening_evidence(32, 33, &[17])];
        functor_actual.codomain_evidence = Some(widening_evidence(34, 35, &[18]));
        functor_actual.guard_obligation = Some(skipped_guard_obligation(85));
        functor_actual.substitution = Some(substitution_composition("OuterF", 86));

        let mut input = TypeAndFactLoweringInput::new(owner);
        input.template_scheme_actuals = vec![type_actual, pred_actual, functor_actual];

        let output = lower_type_and_fact_inputs(&context, input).expect("lowering");
        assert_eq!(output.template_scheme_actuals.len(), 3);
        assert_eq!(
            output
                .template_scheme_actuals
                .iter()
                .map(|actual| actual
                    .substitution
                    .as_ref()
                    .expect("substitution")
                    .enclosing_parameter
                    .as_str()
                    .to_owned())
                .collect::<Vec<_>>(),
            vec!["OuterT", "OuterP", "OuterF"]
        );
        assert!(output.assumptions.is_empty());
        assert!(output.terms.is_empty());
        assert!(output.formulas.is_empty());
        assert!(output.diagnostics.is_empty());
        assert_eq!(output.obligation_seeds.iter().count(), 1);
        assert_step2_delta_valid(&context, &output);
    }

    #[test]
    fn template_scheme_actual_payloads_fail_closed() {
        let (context, owner) = context_with_var(CoreVarId::new(0));

        let mut missing_guard = template_scheme_actual(
            "Sigma[no-guard]",
            "F",
            TemplateSchemeParameterKind::Functor,
            TemplateSchemeActualKind::Deffunc,
            TemplateSchemeActualStatus::Accepted,
            1,
            87,
        );
        missing_guard.domain_evidence = vec![widening_evidence(40, 41, &[19])];
        missing_guard.codomain_evidence = Some(widening_evidence(42, 43, &[20]));
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.template_scheme_actuals = vec![missing_guard];
        assert!(matches!(
            lower_type_and_fact_inputs(&context, input),
            Err(TypeAndFactLoweringError::TemplateSchemeFunctorMissingGuardSeed { .. })
        ));

        let mut active_guard = template_scheme_actual(
            "Sigma[active-guard]",
            "F",
            TemplateSchemeParameterKind::Functor,
            TemplateSchemeActualKind::Deffunc,
            TemplateSchemeActualStatus::Accepted,
            1,
            88,
        );
        active_guard.domain_evidence = vec![widening_evidence(44, 45, &[21])];
        active_guard.codomain_evidence = Some(widening_evidence(46, 47, &[22]));
        active_guard.guard_obligation = Some(active_obligation(CoreVarId::new(0), "guard", 89));
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.template_scheme_actuals = vec![active_guard];
        assert!(matches!(
            lower_type_and_fact_inputs(&context, input),
            Err(
                TypeAndFactLoweringError::TemplateSchemeFunctorInvalidGuardSeedStatus {
                    status: ObligationSeedStatus::Active,
                    ..
                }
            )
        ));

        for (kind, start) in [
            (InitialObligationKind::Sethood, 112),
            (InitialObligationKind::NonEmptiness, 114),
        ] {
            let mut generated_kind_guard = template_scheme_actual(
                "Sigma[generated-guard]",
                "F",
                TemplateSchemeParameterKind::Functor,
                TemplateSchemeActualKind::Deffunc,
                TemplateSchemeActualStatus::Accepted,
                1,
                start,
            );
            generated_kind_guard.domain_evidence = vec![widening_evidence(80, 81, &[37])];
            generated_kind_guard.codomain_evidence = Some(widening_evidence(82, 83, &[38]));
            let mut guard = skipped_guard_obligation(start + 1);
            guard.checker_kind = kind;
            generated_kind_guard.guard_obligation = Some(guard);
            let mut input = TypeAndFactLoweringInput::new(owner);
            input.template_scheme_actuals = vec![generated_kind_guard];
            assert!(matches!(
                lower_type_and_fact_inputs(&context, input),
                Err(
                    TypeAndFactLoweringError::TemplateSchemeFunctorInvalidGuardSeedKind {
                        kind: rejected_kind,
                        ..
                    }
                ) if rejected_kind == kind
            ));
        }

        let mut predicate_with_codomain = template_scheme_actual(
            "Induction[codomain]",
            "P",
            TemplateSchemeParameterKind::Predicate,
            TemplateSchemeActualKind::Defpred,
            TemplateSchemeActualStatus::Accepted,
            1,
            90,
        );
        predicate_with_codomain.domain_evidence = vec![widening_evidence(48, 49, &[23])];
        predicate_with_codomain.codomain_evidence = Some(widening_evidence(50, 51, &[24]));
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.template_scheme_actuals = vec![predicate_with_codomain];
        assert!(matches!(
            lower_type_and_fact_inputs(&context, input),
            Err(TypeAndFactLoweringError::TemplateSchemeActualInvalidCodomainEvidence { .. })
        ));

        let mut rejected_with_evidence = template_scheme_actual(
            "Sigma[bad]",
            "F",
            TemplateSchemeParameterKind::Functor,
            TemplateSchemeActualKind::PartialAlgorithm,
            TemplateSchemeActualStatus::PartialAlgorithm,
            1,
            91,
        );
        rejected_with_evidence.domain_evidence = vec![widening_evidence(52, 53, &[25])];
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.template_scheme_actuals = vec![rejected_with_evidence];
        assert!(matches!(
            lower_type_and_fact_inputs(&context, input),
            Err(
                TypeAndFactLoweringError::RejectedTemplateSchemeActualCarriesEvidence {
                    status: TemplateSchemeActualStatus::PartialAlgorithm,
                    ..
                }
            )
        ));

        let mut type_with_callable_evidence = template_scheme_actual(
            "Inner[type-evidence]",
            "T",
            TemplateSchemeParameterKind::Type,
            TemplateSchemeActualKind::TypeExpression,
            TemplateSchemeActualStatus::Accepted,
            0,
            92,
        );
        type_with_callable_evidence.domain_evidence = vec![widening_evidence(54, 55, &[26])];
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.template_scheme_actuals = vec![type_with_callable_evidence];
        assert!(matches!(
            lower_type_and_fact_inputs(&context, input),
            Err(TypeAndFactLoweringError::TemplateSchemeTypeActualCarriesCallableEvidence { .. })
        ));

        let missing_predicate_domain = template_scheme_actual(
            "Induction[missing-domain]",
            "P",
            TemplateSchemeParameterKind::Predicate,
            TemplateSchemeActualKind::Defpred,
            TemplateSchemeActualStatus::Accepted,
            1,
            93,
        );
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.template_scheme_actuals = vec![missing_predicate_domain];
        assert!(matches!(
            lower_type_and_fact_inputs(&context, input),
            Err(TypeAndFactLoweringError::AcceptedTemplateSchemeActualMissingEvidence { .. })
        ));

        let mut missing_functor_domain = template_scheme_actual(
            "Sigma[missing-domain]",
            "F",
            TemplateSchemeParameterKind::Functor,
            TemplateSchemeActualKind::Deffunc,
            TemplateSchemeActualStatus::Accepted,
            1,
            94,
        );
        missing_functor_domain.codomain_evidence = Some(widening_evidence(64, 65, &[31]));
        missing_functor_domain.guard_obligation = Some(skipped_guard_obligation(95));
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.template_scheme_actuals = vec![missing_functor_domain];
        assert!(matches!(
            lower_type_and_fact_inputs(&context, input),
            Err(TypeAndFactLoweringError::AcceptedTemplateSchemeActualMissingEvidence { .. })
        ));

        let mut missing_functor_codomain = template_scheme_actual(
            "Sigma[missing-codomain]",
            "F",
            TemplateSchemeParameterKind::Functor,
            TemplateSchemeActualKind::Deffunc,
            TemplateSchemeActualStatus::Accepted,
            1,
            96,
        );
        missing_functor_codomain.domain_evidence = vec![widening_evidence(66, 67, &[32])];
        missing_functor_codomain.guard_obligation = Some(skipped_guard_obligation(97));
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.template_scheme_actuals = vec![missing_functor_codomain];
        assert!(matches!(
            lower_type_and_fact_inputs(&context, input),
            Err(TypeAndFactLoweringError::AcceptedTemplateSchemeActualMissingEvidence { .. })
        ));

        let mut partial_domain = template_scheme_actual(
            "Induction[partial-domain]",
            "P",
            TemplateSchemeParameterKind::Predicate,
            TemplateSchemeActualKind::Defpred,
            TemplateSchemeActualStatus::Accepted,
            1,
            98,
        );
        partial_domain.domain_evidence = vec![TemplateWideningEvidenceSeed {
            from_type: NormalizedTypeId::new(68),
            to_type: NormalizedTypeId::new(69),
            status: TemplateWideningEvidenceStatus::DeferredExternalDependency,
            facts: Vec::new(),
        }];
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.template_scheme_actuals = vec![partial_domain];
        assert!(matches!(
            lower_type_and_fact_inputs(&context, input),
            Err(TypeAndFactLoweringError::TemplateSchemeActualPartialDomainEvidence { .. })
        ));

        let mut partial_codomain = template_scheme_actual(
            "Sigma[partial-codomain]",
            "F",
            TemplateSchemeParameterKind::Functor,
            TemplateSchemeActualKind::Deffunc,
            TemplateSchemeActualStatus::Accepted,
            1,
            99,
        );
        partial_codomain.domain_evidence = vec![widening_evidence(70, 71, &[33])];
        partial_codomain.codomain_evidence = Some(TemplateWideningEvidenceSeed {
            from_type: NormalizedTypeId::new(72),
            to_type: NormalizedTypeId::new(73),
            status: TemplateWideningEvidenceStatus::Missing,
            facts: Vec::new(),
        });
        partial_codomain.guard_obligation = Some(skipped_guard_obligation(100));
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.template_scheme_actuals = vec![partial_codomain];
        assert!(matches!(
            lower_type_and_fact_inputs(&context, input),
            Err(TypeAndFactLoweringError::TemplateSchemeActualInvalidCodomainEvidence { .. })
        ));

        let missing_type_substitution = template_scheme_actual(
            "Inner[missing-type-subst]",
            "T",
            TemplateSchemeParameterKind::Type,
            TemplateSchemeActualKind::EnclosingTypeParameter,
            TemplateSchemeActualStatus::Accepted,
            0,
            101,
        );
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.template_scheme_actuals = vec![missing_type_substitution];
        assert!(matches!(
            lower_type_and_fact_inputs(&context, input),
            Err(TypeAndFactLoweringError::TemplateSchemeActualMissingSubstitutionEvidence { .. })
        ));

        let mut missing_predicate_substitution = template_scheme_actual(
            "Inner[missing-predicate-subst]",
            "P",
            TemplateSchemeParameterKind::Predicate,
            TemplateSchemeActualKind::EnclosingPredicateParameter,
            TemplateSchemeActualStatus::Accepted,
            1,
            102,
        );
        missing_predicate_substitution.domain_evidence = vec![widening_evidence(74, 75, &[34])];
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.template_scheme_actuals = vec![missing_predicate_substitution];
        assert!(matches!(
            lower_type_and_fact_inputs(&context, input),
            Err(TypeAndFactLoweringError::TemplateSchemeActualMissingSubstitutionEvidence { .. })
        ));

        let mut missing_functor_substitution = template_scheme_actual(
            "Inner[missing-functor-subst]",
            "F",
            TemplateSchemeParameterKind::Functor,
            TemplateSchemeActualKind::EnclosingFunctorParameter,
            TemplateSchemeActualStatus::Accepted,
            1,
            103,
        );
        missing_functor_substitution.domain_evidence = vec![widening_evidence(76, 77, &[35])];
        missing_functor_substitution.codomain_evidence = Some(widening_evidence(78, 79, &[36]));
        missing_functor_substitution.guard_obligation = Some(skipped_guard_obligation(104));
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.template_scheme_actuals = vec![missing_functor_substitution];
        assert!(matches!(
            lower_type_and_fact_inputs(&context, input),
            Err(TypeAndFactLoweringError::TemplateSchemeActualMissingSubstitutionEvidence { .. })
        ));

        let role_mismatch = template_scheme_actual(
            "Sigma[predicate-as-functor]",
            "F",
            TemplateSchemeParameterKind::Functor,
            TemplateSchemeActualKind::Defpred,
            TemplateSchemeActualStatus::RoleMismatch,
            1,
            105,
        );
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.template_scheme_actuals = vec![role_mismatch];
        assert!(matches!(
            lower_type_and_fact_inputs(&context, input),
            Err(TypeAndFactLoweringError::TemplateSchemeActualKindMismatch { .. })
        ));

        let mut arity_mismatch = template_scheme_actual(
            "Induction[arity]",
            "P",
            TemplateSchemeParameterKind::Predicate,
            TemplateSchemeActualKind::Defpred,
            TemplateSchemeActualStatus::ArityMismatch,
            2,
            106,
        );
        arity_mismatch.actual_arity = 1;
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.template_scheme_actuals = vec![arity_mismatch];
        assert!(matches!(
            lower_type_and_fact_inputs(&context, input),
            Err(TypeAndFactLoweringError::TemplateSchemeActualArityMismatch { .. })
        ));

        let mut unsupported_accepted = template_scheme_actual(
            "Sigma[unsupported]",
            "F",
            TemplateSchemeParameterKind::Functor,
            TemplateSchemeActualKind::Unsupported,
            TemplateSchemeActualStatus::Accepted,
            1,
            107,
        );
        unsupported_accepted.domain_evidence = vec![widening_evidence(56, 57, &[27])];
        unsupported_accepted.codomain_evidence = Some(widening_evidence(58, 59, &[28]));
        unsupported_accepted.guard_obligation = Some(skipped_guard_obligation(108));
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.template_scheme_actuals = vec![unsupported_accepted];
        assert!(matches!(
            lower_type_and_fact_inputs(&context, input),
            Err(TypeAndFactLoweringError::TemplateSchemeActualKindMismatch { .. })
        ));

        let mut first = template_scheme_actual(
            "Sigma[dup]",
            "F",
            TemplateSchemeParameterKind::Functor,
            TemplateSchemeActualKind::Deffunc,
            TemplateSchemeActualStatus::Accepted,
            1,
            109,
        );
        first.domain_evidence = vec![widening_evidence(60, 61, &[29])];
        first.codomain_evidence = Some(widening_evidence(62, 63, &[30]));
        first.guard_obligation = Some(skipped_guard_obligation(110));
        let mut second = first.clone();
        second.source = direct(111, 112);
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.template_scheme_actuals = vec![first, second];
        assert!(matches!(
            lower_type_and_fact_inputs(&context, input),
            Err(TypeAndFactLoweringError::DuplicateTemplateSchemeActual { .. })
        ));
    }

    #[test]
    fn type_fact_subject_must_be_declared_term_variable() {
        let var = CoreVarId::new(0);
        let (formula_context, owner) = context_with_var_sort(var, NormalizedVarSort::Formula);
        let mut formula_input = TypeAndFactLoweringInput::new(owner);
        formula_input.formula_assertions = vec![type_fact(var, "set", 50, Polarity::Positive)];

        assert!(matches!(
            lower_type_and_fact_inputs(&formula_context, formula_input),
            Err(TypeAndFactLoweringError::NonTermSubject { var: actual, sort: NormalizedVarSort::Formula })
                if actual == var
        ));

        let (context, owner) = context_with_var(CoreVarId::new(1));
        let mut undeclared_input = TypeAndFactLoweringInput::new(owner);
        undeclared_input.formula_assertions =
            vec![type_fact(CoreVarId::new(99), "set", 51, Polarity::Positive)];

        assert!(matches!(
            lower_type_and_fact_inputs(&context, undeclared_input),
            Err(TypeAndFactLoweringError::UndeclaredSubject { var }) if var == CoreVarId::new(99)
        ));
    }

    #[test]
    fn reconsidering_and_cluster_facts_enforce_checker_boundaries() {
        let var = CoreVarId::new(0);
        let (context, owner) = context_with_var(var);

        let mut reconsidering_input = TypeAndFactLoweringInput::new(owner);
        reconsidering_input.reconsiderings = vec![ReconsideringSeed::new(
            CoreVarId::new(77),
            "term-binder",
            direct(52, 53),
            provenance("checker:bad-reconsider"),
        )];
        assert!(matches!(
            lower_type_and_fact_inputs(&context, reconsidering_input),
            Err(TypeAndFactLoweringError::UndeclaredSubject { var }) if var == CoreVarId::new(77)
        ));

        let mut cluster_input = TypeAndFactLoweringInput::new(owner);
        cluster_input.cluster_facts = vec![ClusterFactSeed {
            cluster_fact: ClusterFactId::new(9),
            fact: type_fact(var, "cluster:accepted-only", 54, Polarity::Positive),
        }];
        assert!(matches!(
            lower_type_and_fact_inputs(&context, cluster_input),
            Err(TypeAndFactLoweringError::ClusterFactMissingCheckerFact { cluster_fact })
                if cluster_fact == ClusterFactId::new(9)
        ));
    }

    #[test]
    fn term_lowering_covers_core_term_shapes() {
        let x = CoreVarId::new(0);
        let (context, owner) = context_with_var(x);
        let mut input = TermAndFormulaLoweringInput::new(owner);
        input.terms = vec![
            term_seed(CoreTermSeedKind::Var(x), 60),
            term_seed(CoreTermSeedKind::Const(symbol("Const")), 61),
            term_seed(
                CoreTermSeedKind::Apply {
                    functor: symbol("Functor"),
                    args: vec![CoreTermSeedId::new(0), CoreTermSeedId::new(1)],
                },
                62,
            ),
            term_seed(
                CoreTermSeedKind::Select {
                    selector: symbol("selector"),
                    base: CoreTermSeedId::new(2),
                },
                63,
            ),
            term_seed(
                CoreTermSeedKind::Tuple(vec![CoreTermSeedId::new(0), CoreTermSeedId::new(3)]),
                64,
            ),
            term_seed(
                CoreTermSeedKind::SetEnum(vec![CoreTermSeedId::new(1), CoreTermSeedId::new(4)]),
                65,
            ),
        ];

        let output = lower_term_and_formula_inputs(&context, input).expect("lowering");
        let term_id = |seed| output.term_map[&CoreTermSeedId::new(seed)];

        assert!(matches!(
            output.terms.get(term_id(0)).expect("var").kind,
            CoreTermKind::Var(var) if var == x
        ));
        assert!(matches!(
            &output.terms.get(term_id(1)).expect("const").kind,
            CoreTermKind::Const(symbol_id) if symbol_id == &symbol("Const")
        ));
        assert!(matches!(
            &output.terms.get(term_id(2)).expect("apply").kind,
            CoreTermKind::Apply { functor, args }
                if functor == &symbol("Functor") && args == &vec![term_id(0), term_id(1)]
        ));
        assert!(matches!(
            &output.terms.get(term_id(3)).expect("select").kind,
            CoreTermKind::Select { selector, base }
                if selector == &symbol("selector") && *base == term_id(2)
        ));
        assert!(matches!(
            &output.terms.get(term_id(4)).expect("tuple").kind,
            CoreTermKind::Tuple(args) if args == &vec![term_id(0), term_id(3)]
        ));
        assert!(matches!(
            &output.terms.get(term_id(5)).expect("set enum").kind,
            CoreTermKind::SetEnum(args) if args == &vec![term_id(1), term_id(4)]
        ));
        assert_step3_delta_valid(&context, &output);
    }

    #[test]
    fn formula_lowering_covers_constants_atoms_connectives_and_type_predicates() {
        let x = CoreVarId::new(0);
        let y = CoreVarId::new(1);
        let (context, owner) = context_with_var_sorts(vec![
            (x, NormalizedVarSort::Term),
            (y, NormalizedVarSort::Term),
        ]);
        let mut input = TermAndFormulaLoweringInput::new(owner);
        input.terms = vec![
            term_seed(CoreTermSeedKind::Var(x), 70),
            term_seed(CoreTermSeedKind::Var(y), 71),
        ];
        input.formulas = vec![
            formula_seed(CoreFormulaSeedKind::True, 72),
            formula_seed(CoreFormulaSeedKind::False, 73),
            formula_seed(
                CoreFormulaSeedKind::Atom {
                    predicate: symbol("Predicate"),
                    args: vec![CoreTermSeedId::new(0), CoreTermSeedId::new(1)],
                },
                74,
            ),
            formula_seed(
                CoreFormulaSeedKind::Equals {
                    left: CoreTermSeedId::new(0),
                    right: CoreTermSeedId::new(1),
                },
                75,
            ),
            formula_seed(
                CoreFormulaSeedKind::TypePred {
                    subject: CoreTermSeedId::new(0),
                    ty: CoreTypePredicate::new("set"),
                },
                76,
            ),
            formula_seed(CoreFormulaSeedKind::Not(CoreFormulaSeedId::new(1)), 77),
            formula_seed(
                CoreFormulaSeedKind::And(vec![
                    CoreFormulaSeedId::new(2),
                    CoreFormulaSeedId::new(3),
                    CoreFormulaSeedId::new(4),
                ]),
                78,
            ),
            formula_seed(
                CoreFormulaSeedKind::Or(vec![CoreFormulaSeedId::new(0), CoreFormulaSeedId::new(1)]),
                79,
            ),
            formula_seed(
                CoreFormulaSeedKind::Implies {
                    premise: CoreFormulaSeedId::new(2),
                    conclusion: CoreFormulaSeedId::new(3),
                },
                80,
            ),
            formula_seed(
                CoreFormulaSeedKind::Iff {
                    left: CoreFormulaSeedId::new(8),
                    right: CoreFormulaSeedId::new(7),
                },
                81,
            ),
        ];

        let output = lower_term_and_formula_inputs(&context, input).expect("lowering");
        let term_id = |seed| output.term_map[&CoreTermSeedId::new(seed)];
        let formula_id = |seed| output.formula_map[&CoreFormulaSeedId::new(seed)];

        assert!(matches!(
            output.formulas.get(formula_id(0)).expect("true").kind,
            CoreFormulaKind::True
        ));
        assert!(matches!(
            output.formulas.get(formula_id(1)).expect("false").kind,
            CoreFormulaKind::False
        ));
        assert!(matches!(
            &output.formulas.get(formula_id(2)).expect("atom").kind,
            CoreFormulaKind::Atom { predicate, args }
                if predicate == &symbol("Predicate") && args == &vec![term_id(0), term_id(1)]
        ));
        assert!(matches!(
            output.formulas.get(formula_id(3)).expect("equals").kind,
            CoreFormulaKind::Equals { left, right } if left == term_id(0) && right == term_id(1)
        ));
        assert!(matches!(
            &output.formulas.get(formula_id(4)).expect("type pred").kind,
            CoreFormulaKind::TypePred { subject, ty } if *subject == term_id(0) && ty.as_str() == "set"
        ));
        assert!(matches!(
            output.formulas.get(formula_id(5)).expect("not").kind,
            CoreFormulaKind::Not(inner) if inner == formula_id(1)
        ));
        assert!(matches!(
            &output.formulas.get(formula_id(6)).expect("and").kind,
            CoreFormulaKind::And(children)
                if children == &vec![formula_id(2), formula_id(3), formula_id(4)]
        ));
        assert!(matches!(
            &output.formulas.get(formula_id(7)).expect("or").kind,
            CoreFormulaKind::Or(children) if children == &vec![formula_id(0), formula_id(1)]
        ));
        assert!(matches!(
            output.formulas.get(formula_id(8)).expect("implies").kind,
            CoreFormulaKind::Implies { premise, conclusion }
                if premise == formula_id(2) && conclusion == formula_id(3)
        ));
        assert!(matches!(
            output.formulas.get(formula_id(9)).expect("iff").kind,
            CoreFormulaKind::Iff { left, right } if left == formula_id(8) && right == formula_id(7)
        ));
        assert_step3_delta_valid(&context, &output);
    }

    #[test]
    fn quantifier_guards_allow_self_and_prior_binders_but_reject_later_binders() {
        let x = CoreVarId::new(0);
        let y = CoreVarId::new(1);
        let (context, owner) = context_with_var_sorts(vec![
            (x, NormalizedVarSort::Term),
            (y, NormalizedVarSort::Term),
        ]);
        let mut input = TermAndFormulaLoweringInput::new(owner);
        input.formulas = vec![
            formula_seed(CoreFormulaSeedKind::True, 82),
            formula_seed(
                CoreFormulaSeedKind::Forall {
                    binders: vec![
                        QuantifierBinderSeed::new(
                            x,
                            "term-binder",
                            direct(83, 84),
                            provenance("checker:forall:x"),
                        )
                        .with_guard(CoreFormulaSeedId::new(0), vec![x])
                        .with_source_name("x"),
                        QuantifierBinderSeed::new(
                            y,
                            "term-binder",
                            direct(85, 86),
                            provenance("checker:forall:y"),
                        )
                        .with_guard(CoreFormulaSeedId::new(0), vec![x, y])
                        .with_source_name("y"),
                    ],
                    body: CoreFormulaSeedId::new(0),
                },
                87,
            ),
            formula_seed(
                CoreFormulaSeedKind::Exists {
                    binders: vec![
                        QuantifierBinderSeed::new(
                            x,
                            "term-binder",
                            direct(87, 88),
                            provenance("checker:exists:x"),
                        )
                        .with_source_name("x"),
                    ],
                    body: CoreFormulaSeedId::new(0),
                },
                88,
            ),
        ];

        let output = lower_term_and_formula_inputs(&context, input).expect("lowering");
        let forall = output.formula_map[&CoreFormulaSeedId::new(1)];
        let CoreFormulaKind::Forall { binders, body } =
            &output.formulas.get(forall).expect("forall").kind
        else {
            panic!("expected forall");
        };
        assert_eq!(binders.len(), 2);
        assert_eq!(binders[0].source_name.as_deref(), Some("x"));
        assert_eq!(binders[1].source_name.as_deref(), Some("y"));
        assert!(binders.iter().all(|binder| binder.ty_guard.is_some()));
        assert_eq!(*body, output.formula_map[&CoreFormulaSeedId::new(0)]);
        let exists = output.formula_map[&CoreFormulaSeedId::new(2)];
        let CoreFormulaKind::Exists {
            binders: exists_binders,
            body: exists_body,
        } = &output.formulas.get(exists).expect("exists").kind
        else {
            panic!("expected exists");
        };
        assert_eq!(exists_binders.len(), 1);
        assert_eq!(exists_binders[0].var, x);
        assert_eq!(*exists_body, output.formula_map[&CoreFormulaSeedId::new(0)]);
        assert_step3_delta_valid(&context, &output);

        let mut bad_input = TermAndFormulaLoweringInput::new(owner);
        bad_input.terms = vec![term_seed(CoreTermSeedKind::Var(y), 88)];
        bad_input.formulas = vec![
            formula_seed(
                CoreFormulaSeedKind::TypePred {
                    subject: CoreTermSeedId::new(0),
                    ty: CoreTypePredicate::new("Nat"),
                },
                88,
            ),
            formula_seed(
                CoreFormulaSeedKind::Forall {
                    binders: vec![
                        QuantifierBinderSeed::new(
                            x,
                            "term-binder",
                            direct(89, 90),
                            provenance("checker:forall:bad-x"),
                        )
                        .with_guard(CoreFormulaSeedId::new(0), Vec::new()),
                        QuantifierBinderSeed::new(
                            y,
                            "term-binder",
                            direct(91, 92),
                            provenance("checker:forall:bad-y"),
                        ),
                    ],
                    body: CoreFormulaSeedId::new(0),
                },
                93,
            ),
        ];
        assert!(matches!(
            lower_term_and_formula_inputs(&context, bad_input),
            Err(TermAndFormulaLoweringError::FutureBinderInGuard { binder, later })
                if binder == x && later == y
        ));
    }

    #[test]
    fn qua_terms_reuse_underlying_term_and_record_view_explanation() {
        let x = CoreVarId::new(0);
        let (context, owner) = context_with_var(x);
        let mut input = TermAndFormulaLoweringInput::new(owner);
        input.terms = vec![
            term_seed(CoreTermSeedKind::Var(x), 94),
            term_seed(
                CoreTermSeedKind::Qua {
                    base: CoreTermSeedId::new(0),
                    explanation: ViewExplanationSeed {
                        kind: ViewExplanationKind::SourceQua,
                        inserted_view: None,
                        target_type: Some(NormalizedTypeId::new(12)),
                        reduct: None,
                        evidence_facts: vec![
                            TypeFactId::new(3),
                            TypeFactId::new(2),
                            TypeFactId::new(2),
                        ],
                        source: direct(95, 96),
                        provenance: provenance("checker:term-qua"),
                    },
                },
                95,
            ),
            term_seed(
                CoreTermSeedKind::Qua {
                    base: CoreTermSeedId::new(0),
                    explanation: ViewExplanationSeed {
                        kind: ViewExplanationKind::InsertedView,
                        inserted_view: Some(CoercionInsertionId::new(7)),
                        target_type: Some(NormalizedTypeId::new(13)),
                        reduct: None,
                        evidence_facts: vec![TypeFactId::new(5)],
                        source: direct(96, 97),
                        provenance: provenance("checker:term-inserted-view"),
                    },
                },
                96,
            ),
        ];

        let output = lower_term_and_formula_inputs(&context, input).expect("lowering");

        assert_eq!(output.terms.len(), 1);
        assert_eq!(
            output.term_map[&CoreTermSeedId::new(1)],
            output.term_map[&CoreTermSeedId::new(0)]
        );
        assert_eq!(
            output.term_map[&CoreTermSeedId::new(2)],
            output.term_map[&CoreTermSeedId::new(0)]
        );
        assert_eq!(output.view_explanations.len(), 2);
        assert_eq!(
            output.view_explanations[0].evidence_facts,
            vec![TypeFactId::new(2), TypeFactId::new(3)]
        );
        assert_eq!(
            output.view_explanations[1].kind,
            ViewExplanationKind::InsertedView
        );
        assert_eq!(
            output.view_explanations[1].inserted_view,
            Some(CoercionInsertionId::new(7))
        );
        assert!(output.view_explanations[0].reduct.is_none());
        assert!(output.view_explanations[1].reduct.is_none());
        assert_eq!(
            output.view_explanations[1].evidence_facts,
            vec![TypeFactId::new(5)]
        );
        assert_step3_delta_valid(&context, &output);
    }

    #[test]
    fn reduct_qua_lowers_renamed_views_to_distinct_terms() {
        let r = CoreVarId::new(0);
        let (context, owner) = context_with_var(r);
        let mut input = TermAndFormulaLoweringInput::new(owner);
        input.terms = vec![
            term_seed(CoreTermSeedKind::Var(r), 94),
            term_seed(
                CoreTermSeedKind::Qua {
                    base: CoreTermSeedId::new(0),
                    explanation: source_qua_explanation(
                        "Ring>AddMagma",
                        &["view_add_magma"],
                        20,
                        95,
                    ),
                },
                95,
            ),
            term_seed(
                CoreTermSeedKind::Qua {
                    base: CoreTermSeedId::new(0),
                    explanation: source_qua_explanation(
                        "Ring>MulMagma",
                        &["view_mul_magma"],
                        21,
                        96,
                    ),
                },
                96,
            ),
            term_seed(
                CoreTermSeedKind::Select {
                    selector: symbol("binop"),
                    base: CoreTermSeedId::new(1),
                },
                97,
            ),
            term_seed(
                CoreTermSeedKind::Select {
                    selector: symbol("binop"),
                    base: CoreTermSeedId::new(2),
                },
                98,
            ),
        ];
        input.formulas = vec![
            formula_seed(
                CoreFormulaSeedKind::Atom {
                    predicate: symbol("is_commutative"),
                    args: vec![CoreTermSeedId::new(1)],
                },
                99,
            ),
            formula_seed(
                CoreFormulaSeedKind::TypePred {
                    subject: CoreTermSeedId::new(2),
                    ty: CoreTypePredicate::new("commutative_Magma"),
                },
                100,
            ),
        ];

        let output = lower_term_and_formula_inputs(&context, input).expect("lowering");
        let term_id = |seed| output.term_map[&CoreTermSeedId::new(seed)];
        let formula_id = |seed| output.formula_map[&CoreFormulaSeedId::new(seed)];
        let base = term_id(0);
        let add_view = term_id(1);
        let mul_view = term_id(2);

        assert_ne!(add_view, base);
        assert_ne!(mul_view, base);
        assert_ne!(add_view, mul_view);
        assert!(matches!(
            &output.terms.get(add_view).expect("add view").kind,
            CoreTermKind::Apply { functor, args }
                if functor == &symbol("view_add_magma") && args == &vec![base]
        ));
        assert!(matches!(
            &output.terms.get(mul_view).expect("mul view").kind,
            CoreTermKind::Apply { functor, args }
                if functor == &symbol("view_mul_magma") && args == &vec![base]
        ));
        assert!(matches!(
            &output.terms.get(term_id(3)).expect("add binop").kind,
            CoreTermKind::Select { selector, base }
                if selector == &symbol("binop") && *base == add_view
        ));
        assert!(matches!(
            &output.terms.get(term_id(4)).expect("mul binop").kind,
            CoreTermKind::Select { selector, base }
                if selector == &symbol("binop") && *base == mul_view
        ));
        assert!(matches!(
            &output.formulas.get(formula_id(0)).expect("attribute").kind,
            CoreFormulaKind::Atom { predicate, args }
                if predicate == &symbol("is_commutative") && args == &vec![add_view]
        ));
        assert!(matches!(
            &output.formulas.get(formula_id(1)).expect("type pred").kind,
            CoreFormulaKind::TypePred { subject, ty }
                if *subject == mul_view && ty == &CoreTypePredicate::new("commutative_Magma")
        ));
        assert!(
            output
                .formulas
                .iter()
                .all(|(_, formula)| !matches!(formula.kind, CoreFormulaKind::Equals { .. }))
        );
        assert_eq!(
            output.view_explanations[0]
                .reduct
                .as_ref()
                .expect("add reduct")
                .path
                .as_str(),
            "Ring>AddMagma"
        );
        assert_eq!(
            output.view_explanations[0]
                .reduct
                .as_ref()
                .expect("add reduct")
                .functors,
            vec![symbol("view_add_magma")]
        );
        assert_eq!(
            output.view_explanations[1]
                .reduct
                .as_ref()
                .expect("mul reduct")
                .path
                .as_str(),
            "Ring>MulMagma"
        );
        assert_eq!(
            output.view_explanations[1]
                .reduct
                .as_ref()
                .expect("mul reduct")
                .functors,
            vec![symbol("view_mul_magma")]
        );
        assert_step3_delta_valid(&context, &output);
    }

    #[test]
    fn composed_reduct_view_lowers_nested_and_template_bounds_use_final_view() {
        let r = CoreVarId::new(0);
        let (context, owner) = context_with_var(r);
        let mut input = TermAndFormulaLoweringInput::new(owner);
        input.terms = vec![
            term_seed(CoreTermSeedKind::Var(r), 101),
            term_seed(
                CoreTermSeedKind::Qua {
                    base: CoreTermSeedId::new(0),
                    explanation: source_qua_explanation(
                        "Ring>AddGroup>Magma",
                        &["z_view_add_group", "a_view_group_magma"],
                        22,
                        102,
                    ),
                },
                102,
            ),
            term_seed(
                CoreTermSeedKind::Select {
                    selector: symbol("binop"),
                    base: CoreTermSeedId::new(1),
                },
                103,
            ),
        ];
        input.formulas = vec![formula_seed(
            CoreFormulaSeedKind::TypePred {
                subject: CoreTermSeedId::new(1),
                ty: CoreTypePredicate::new("template_bound_commutative"),
            },
            104,
        )];

        let output = lower_term_and_formula_inputs(&context, input).expect("lowering");
        let term_id = |seed| output.term_map[&CoreTermSeedId::new(seed)];
        let formula_id = |seed| output.formula_map[&CoreFormulaSeedId::new(seed)];
        let base = term_id(0);
        let final_view = term_id(1);
        let CoreTermKind::Apply {
            functor: final_functor,
            args: final_args,
        } = &output.terms.get(final_view).expect("final view").kind
        else {
            panic!("expected final view apply");
        };
        assert_eq!(final_functor, &symbol("a_view_group_magma"));
        let [intermediate] = final_args.as_slice() else {
            panic!("expected unary final view");
        };
        assert!(matches!(
            &output.terms.get(*intermediate).expect("intermediate view").kind,
            CoreTermKind::Apply { functor, args }
                if functor == &symbol("z_view_add_group") && args == &vec![base]
        ));
        assert!(matches!(
            &output.terms.get(term_id(2)).expect("selected field").kind,
            CoreTermKind::Select { selector, base }
                if selector == &symbol("binop") && *base == final_view
        ));
        assert!(matches!(
            &output.formulas.get(formula_id(0)).expect("template bound").kind,
            CoreFormulaKind::TypePred { subject, ty }
                if *subject == final_view
                    && ty == &CoreTypePredicate::new("template_bound_commutative")
        ));
        assert_step3_delta_valid(&context, &output);
    }

    #[test]
    fn exact_instance_extensionality_guard_is_preserved_on_reduct_term() {
        let r = CoreVarId::new(0);
        let (context, owner) = context_with_var(r);
        let mut input = TermAndFormulaLoweringInput::new(owner);
        input.terms = vec![
            term_seed(CoreTermSeedKind::Var(r), 105),
            term_seed(
                CoreTermSeedKind::Qua {
                    base: CoreTermSeedId::new(0),
                    explanation: source_qua_explanation(
                        "Ring>AddMagma",
                        &["view_add_magma"],
                        23,
                        106,
                    ),
                },
                106,
            ),
        ];
        input.formulas = vec![
            formula_seed(
                CoreFormulaSeedKind::TypePred {
                    subject: CoreTermSeedId::new(1),
                    ty: CoreTypePredicate::new("exact_Magma"),
                },
                107,
            ),
            formula_seed(
                CoreFormulaSeedKind::Atom {
                    predicate: symbol("magma_field_extensionality"),
                    args: vec![CoreTermSeedId::new(1)],
                },
                108,
            ),
            formula_seed(
                CoreFormulaSeedKind::Implies {
                    premise: CoreFormulaSeedId::new(0),
                    conclusion: CoreFormulaSeedId::new(1),
                },
                109,
            ),
        ];

        let output = lower_term_and_formula_inputs(&context, input).expect("lowering");
        let term_id = |seed| output.term_map[&CoreTermSeedId::new(seed)];
        let formula_id = |seed| output.formula_map[&CoreFormulaSeedId::new(seed)];
        let view = term_id(1);

        assert_eq!(output.formulas.len(), 3);
        assert!(matches!(
            &output.formulas.get(formula_id(0)).expect("exact guard").kind,
            CoreFormulaKind::TypePred { subject, ty }
                if *subject == view && ty == &CoreTypePredicate::new("exact_Magma")
        ));
        assert!(matches!(
            &output.formulas.get(formula_id(1)).expect("extensionality atom").kind,
            CoreFormulaKind::Atom { predicate, args }
                if predicate == &symbol("magma_field_extensionality") && args == &vec![view]
        ));
        assert!(matches!(
            &output.formulas.get(formula_id(2)).expect("guarded formula").kind,
            CoreFormulaKind::Implies { premise, conclusion }
                if *premise == formula_id(0) && *conclusion == formula_id(1)
        ));
        assert!(output.formulas.iter().all(|(_, formula)| {
            !matches!(
                &formula.kind,
                CoreFormulaKind::TypePred { ty, .. } if ty == &CoreTypePredicate::new("is_Magma")
            )
        }));
        assert_step3_delta_valid(&context, &output);
    }

    #[test]
    fn reduct_view_payload_requires_explicit_functors() {
        let x = CoreVarId::new(0);
        let (context, owner) = context_with_var(x);
        let empty_reduct = reduct_view("Ring>Magma", &[]);
        let mut type_input = TypeAndFactLoweringInput::new(owner);
        type_input.view_explanations = vec![ViewExplanationSeed {
            kind: ViewExplanationKind::SourceQua,
            inserted_view: None,
            target_type: Some(NormalizedTypeId::new(24)),
            reduct: Some(empty_reduct.clone()),
            evidence_facts: Vec::new(),
            source: direct(110, 111),
            provenance: provenance("checker:empty-reduct-step2"),
        }];
        assert!(matches!(
            lower_type_and_fact_inputs(&context, type_input),
            Err(TypeAndFactLoweringError::EmptyReductViewPayload { path })
                if path.as_str() == "Ring>Magma"
        ));

        let mut term_input = TermAndFormulaLoweringInput::new(owner);
        term_input.terms = vec![
            term_seed(CoreTermSeedKind::Var(x), 112),
            term_seed(
                CoreTermSeedKind::Qua {
                    base: CoreTermSeedId::new(0),
                    explanation: ViewExplanationSeed {
                        kind: ViewExplanationKind::SourceQua,
                        inserted_view: None,
                        target_type: Some(NormalizedTypeId::new(25)),
                        reduct: Some(empty_reduct),
                        evidence_facts: Vec::new(),
                        source: direct(113, 114),
                        provenance: provenance("checker:empty-reduct-step3"),
                    },
                },
                113,
            ),
        ];
        assert!(matches!(
            lower_term_and_formula_inputs(&context, term_input),
            Err(TermAndFormulaLoweringError::EmptyReductViewPayload { path })
                if path.as_str() == "Ring>Magma"
        ));
    }

    #[test]
    fn stable_choice_reuses_existing_and_delta_generated_origin_keys() {
        let x = CoreVarId::new(0);
        let existing_key = GeneratedOriginKey::new("choice:existing");
        let new_key = GeneratedOriginKey::new("choice:new");
        let (context, owner, existing_origin) =
            context_with_existing_choice_origin(x, existing_key.clone());
        let existing_generated_len = context.generated_origins().table().len();
        let mut input = TermAndFormulaLoweringInput::new(owner);
        input.terms = vec![
            term_seed(CoreTermSeedKind::Var(x), 96),
            term_seed(
                CoreTermSeedKind::StableChoice {
                    functor: symbol("choice_existing"),
                    origin_functor: symbol("choice_existing"),
                    key: existing_key.clone(),
                    params: vec![x],
                    args: vec![CoreTermSeedId::new(0)],
                    evidence: vec![CoreProvenance::new(
                        CoreProvenancePhase::Checker,
                        "checker:choice:use-existing-1",
                    )],
                },
                97,
            ),
            term_seed(
                CoreTermSeedKind::StableChoice {
                    functor: symbol("choice_existing"),
                    origin_functor: symbol("choice_existing"),
                    key: existing_key,
                    params: vec![x],
                    args: vec![CoreTermSeedId::new(0)],
                    evidence: vec![CoreProvenance::new(
                        CoreProvenancePhase::Checker,
                        "checker:choice:use-existing-2",
                    )],
                },
                98,
            ),
            term_seed(
                CoreTermSeedKind::StableChoice {
                    functor: symbol("choice_new"),
                    origin_functor: symbol("choice_new"),
                    key: new_key.clone(),
                    params: vec![x],
                    args: vec![CoreTermSeedId::new(0)],
                    evidence: vec![CoreProvenance::new(
                        CoreProvenancePhase::Checker,
                        "checker:choice:new-1",
                    )],
                },
                99,
            ),
            term_seed(
                CoreTermSeedKind::StableChoice {
                    functor: symbol("choice_new"),
                    origin_functor: symbol("choice_new"),
                    key: new_key.clone(),
                    params: vec![x],
                    args: vec![CoreTermSeedId::new(0)],
                    evidence: vec![CoreProvenance::new(
                        CoreProvenancePhase::Checker,
                        "checker:choice:new-2",
                    )],
                },
                100,
            ),
        ];

        let output = lower_term_and_formula_inputs(&context, input).expect("lowering");

        assert_eq!(output.generated.len(), existing_generated_len + 1);
        assert_eq!(output.generated_delta.len(), 1);
        assert_eq!(output.new_generated_origins.len(), 1);
        let (_, delta_origin) = output
            .generated_delta
            .iter()
            .next()
            .expect("single generated delta");
        assert_eq!(delta_origin.kind, GeneratedOriginKind::StableChoice);
        assert_eq!(delta_origin.key, new_key);
        assert_eq!(delta_origin.params, vec![x]);
        assert_eq!(output.generated_origin_refs.len(), 4);
        assert_eq!(output.generated_origin_refs[0].origin, existing_origin);
        assert!(output.generated_origin_refs[0].reused_existing);
        assert_eq!(
            output.generated_origin_refs[0].reuse_source,
            GeneratedOriginReuseSource::ExistingRegistry
        );
        assert_eq!(output.generated_origin_refs[1].origin, existing_origin);
        assert!(output.generated_origin_refs[1].reused_existing);
        assert_eq!(
            output.generated_origin_refs[1].reuse_source,
            GeneratedOriginReuseSource::ExistingRegistry
        );
        let new_origin = output.generated_origin_refs[2].origin;
        assert_ne!(new_origin, existing_origin);
        assert!(!output.generated_origin_refs[2].reused_existing);
        assert_eq!(
            output.generated_origin_refs[2].reuse_source,
            GeneratedOriginReuseSource::NewDelta
        );
        assert_eq!(output.new_generated_origins, vec![new_origin]);
        assert_eq!(output.generated_origin_refs[3].origin, new_origin);
        assert!(output.generated_origin_refs[3].reused_existing);
        assert_eq!(
            output.generated_origin_refs[3].reuse_source,
            GeneratedOriginReuseSource::CurrentDelta
        );
        for (seed, expected_functor, expected_key) in [
            (1, symbol("choice_existing"), "choice:existing"),
            (2, symbol("choice_existing"), "choice:existing"),
            (3, symbol("choice_new"), "choice:new"),
            (4, symbol("choice_new"), "choice:new"),
        ] {
            let CoreTermKind::Apply { functor, args } = &output
                .terms
                .get(output.term_map[&CoreTermSeedId::new(seed)])
                .expect("choice term")
                .kind
            else {
                panic!("expected stable choice apply");
            };
            assert_eq!(functor, &expected_functor);
            assert_eq!(args, &vec![output.term_map[&CoreTermSeedId::new(0)]]);
            let use_record = &output.generated_origin_refs[seed - 1];
            assert_eq!(use_record.term, output.term_map[&CoreTermSeedId::new(seed)]);
            assert_eq!(use_record.key.as_str(), expected_key);
            assert_eq!(use_record.functor, expected_functor);
            assert_eq!(
                use_record.args,
                vec![output.term_map[&CoreTermSeedId::new(0)]]
            );
        }
        assert_step3_delta_valid(&context, &output);

        let mut mismatch_input = TermAndFormulaLoweringInput::new(owner);
        mismatch_input.terms = vec![term_seed(
            CoreTermSeedKind::StableChoice {
                functor: symbol("choice_existing"),
                origin_functor: symbol("choice_existing"),
                key: GeneratedOriginKey::new("choice:existing"),
                params: Vec::new(),
                args: Vec::new(),
                evidence: vec![CoreProvenance::new(
                    CoreProvenancePhase::Checker,
                    "checker:choice:param-mismatch",
                )],
            },
            100,
        )];
        assert!(matches!(
            lower_term_and_formula_inputs(&context, mismatch_input),
            Err(TermAndFormulaLoweringError::GeneratedOriginParameterMismatch { origin, key })
                if origin == existing_origin && key.as_str() == "choice:existing"
        ));

        let mut functor_mismatch = TermAndFormulaLoweringInput::new(owner);
        functor_mismatch.terms = vec![term_seed(
            CoreTermSeedKind::StableChoice {
                functor: symbol("ordinary_functor"),
                origin_functor: symbol("choice_existing"),
                key: GeneratedOriginKey::new("choice:existing"),
                params: vec![x],
                args: Vec::new(),
                evidence: vec![CoreProvenance::new(
                    CoreProvenancePhase::Checker,
                    "checker:choice:functor-mismatch",
                )],
            },
            100,
        )];
        assert!(matches!(
            lower_term_and_formula_inputs(&context, functor_mismatch),
            Err(TermAndFormulaLoweringError::GeneratedFunctorMismatch { key, .. })
                if key.as_str() == "choice:existing"
        ));

        let mut registry_functor_mismatch = TermAndFormulaLoweringInput::new(owner);
        registry_functor_mismatch.terms = vec![term_seed(
            CoreTermSeedKind::StableChoice {
                functor: symbol("ordinary_functor"),
                origin_functor: symbol("ordinary_functor"),
                key: GeneratedOriginKey::new("choice:existing"),
                params: vec![x],
                args: Vec::new(),
                evidence: vec![CoreProvenance::new(
                    CoreProvenancePhase::Checker,
                    "checker:choice:registry-functor-mismatch",
                )],
            },
            101,
        )];
        assert!(matches!(
            lower_term_and_formula_inputs(&context, registry_functor_mismatch),
            Err(TermAndFormulaLoweringError::GeneratedFunctorMismatch { key, expected, actual })
                if key.as_str() == "choice:existing"
                    && expected.as_ref() == &symbol("choice_existing")
                    && actual.as_ref() == &symbol("ordinary_functor")
        ));
    }

    #[test]
    fn fraenkel_lowering_preserves_evidence_and_membership_obligation() {
        let x = CoreVarId::new(0);
        let (context, owner) = context_with_var(x);
        let mut input = TermAndFormulaLoweringInput::new(owner);
        input.terms = vec![
            term_seed(CoreTermSeedKind::Var(x), 101),
            term_seed(
                CoreTermSeedKind::Fraenkel {
                    functor: symbol("fraenkel_set"),
                    origin_functor: symbol("fraenkel_set"),
                    key: GeneratedOriginKey::new("fraenkel:mapper:predicate"),
                    params: vec![x],
                    args: vec![CoreTermSeedId::new(0)],
                    sethood_evidence: vec![CoreProvenance::new(
                        CoreProvenancePhase::Checker,
                        "checker:fraenkel:sethood",
                    )],
                    template_type_parameter_sethood: None,
                    membership_obligation: Box::new(FraenkelMembershipObligationSeed::New(
                        CoreObligationSeed::active(
                            ObligationSeedKind::FraenkelMembershipAxiom,
                            CoreFormulaSeedId::new(0),
                            "fraenkel/membership",
                            "pkg::main::Owner.fraenkel.membership",
                            direct(103, 104),
                            provenance("checker:fraenkel:obligation"),
                        ),
                    )),
                    missing_sethood_obligation: None,
                },
                102,
            ),
            term_seed(
                CoreTermSeedKind::Fraenkel {
                    functor: symbol("fraenkel_set"),
                    origin_functor: symbol("fraenkel_set"),
                    key: GeneratedOriginKey::new("fraenkel:mapper:predicate"),
                    params: vec![x],
                    args: vec![CoreTermSeedId::new(0)],
                    sethood_evidence: vec![CoreProvenance::new(
                        CoreProvenancePhase::Checker,
                        "checker:fraenkel:sethood:reuse",
                    )],
                    template_type_parameter_sethood: None,
                    membership_obligation: Box::new(
                        FraenkelMembershipObligationSeed::AlreadyCarried(
                            AlreadyCarriedFraenkelMembershipSeed {
                                source: direct(104, 105),
                                provenance: provenance("checker:fraenkel:already-carried"),
                            },
                        ),
                    ),
                    missing_sethood_obligation: None,
                },
                104,
            ),
        ];
        input.formulas = vec![formula_seed(CoreFormulaSeedKind::True, 103)];

        let output = lower_term_and_formula_inputs(&context, input).expect("lowering");
        let fraenkel_term = output.term_map[&CoreTermSeedId::new(1)];
        let generated_ref = &output.generated_origin_refs[0];
        let generated = output
            .generated
            .get(generated_ref.origin)
            .expect("fraenkel origin");
        let reused_generated_ref = &output.generated_origin_refs[1];
        let obligation = output
            .obligation_seeds
            .get(output.generated_obligations[0].obligation)
            .expect("fraenkel obligation");

        assert!(matches!(
            output.terms.get(fraenkel_term).expect("fraenkel").kind,
            CoreTermKind::Apply { .. }
        ));
        assert_eq!(output.generated_delta.len(), 1);
        assert_eq!(output.generated_origin_refs.len(), 2);
        assert_eq!(
            generated_ref.reuse_source,
            GeneratedOriginReuseSource::NewDelta
        );
        assert_eq!(
            reused_generated_ref.reuse_source,
            GeneratedOriginReuseSource::CurrentDelta
        );
        assert_eq!(reused_generated_ref.origin, generated_ref.origin);
        assert_eq!(
            reused_generated_ref.term,
            output.term_map[&CoreTermSeedId::new(2)]
        );
        assert_eq!(reused_generated_ref.functor, symbol("fraenkel_set"));
        assert_eq!(
            reused_generated_ref.args,
            vec![output.term_map[&CoreTermSeedId::new(0)]]
        );
        assert_eq!(generated.kind, GeneratedOriginKind::FraenkelComprehension);
        assert!(generated.evidence.iter().any(|entry| {
            entry.phase == CoreProvenancePhase::Checker
                && entry.key.as_str() == "checker:fraenkel:sethood"
        }));
        assert_eq!(
            output.generated_obligations[0].kind,
            ObligationSeedKind::FraenkelMembershipAxiom
        );
        assert_eq!(obligation.status, ObligationSeedStatus::Active);
        assert!(obligation.goal.is_some());
        assert_eq!(output.already_carried_generated_obligations.len(), 1);
        assert_eq!(
            output.already_carried_generated_obligations[0].origin,
            generated_ref.origin
        );
        assert_step3_delta_valid(&context, &output);

        let mut bad_membership = TermAndFormulaLoweringInput::new(owner);
        bad_membership.terms = vec![
            term_seed(CoreTermSeedKind::Var(x), 107),
            term_seed(
                CoreTermSeedKind::Fraenkel {
                    functor: symbol("fraenkel_set"),
                    origin_functor: symbol("fraenkel_set"),
                    key: GeneratedOriginKey::new("fraenkel:bad-membership"),
                    params: vec![x],
                    args: vec![CoreTermSeedId::new(0)],
                    sethood_evidence: vec![CoreProvenance::new(
                        CoreProvenancePhase::Checker,
                        "checker:fraenkel:bad:sethood",
                    )],
                    template_type_parameter_sethood: None,
                    membership_obligation: Box::new(FraenkelMembershipObligationSeed::New(
                        CoreObligationSeed::active(
                            ObligationSeedKind::GeneratedSethood,
                            CoreFormulaSeedId::new(0),
                            "fraenkel/bad-membership",
                            "pkg::main::Owner.fraenkel.bad-membership",
                            direct(108, 109),
                            provenance("checker:fraenkel:bad-membership"),
                        ),
                    )),
                    missing_sethood_obligation: None,
                },
                108,
            ),
        ];
        bad_membership.formulas = vec![formula_seed(CoreFormulaSeedKind::True, 109)];
        assert!(matches!(
            lower_term_and_formula_inputs(&context, bad_membership),
            Err(TermAndFormulaLoweringError::InvalidFraenkelMembershipObligation {
                kind,
                status: ObligationSeedStatus::Active,
            }) if kind == ObligationSeedKind::GeneratedSethood
        ));

        let mut deferred_membership = TermAndFormulaLoweringInput::new(owner);
        deferred_membership.terms = vec![
            term_seed(CoreTermSeedKind::Var(x), 110),
            term_seed(
                CoreTermSeedKind::Fraenkel {
                    functor: symbol("fraenkel_set"),
                    origin_functor: symbol("fraenkel_set"),
                    key: GeneratedOriginKey::new("fraenkel:deferred-membership"),
                    params: vec![x],
                    args: vec![CoreTermSeedId::new(0)],
                    sethood_evidence: vec![CoreProvenance::new(
                        CoreProvenancePhase::Checker,
                        "checker:fraenkel:deferred:sethood",
                    )],
                    template_type_parameter_sethood: None,
                    membership_obligation: Box::new(FraenkelMembershipObligationSeed::New(
                        CoreObligationSeed::deferred(
                            ObligationSeedKind::FraenkelMembershipAxiom,
                            "fraenkel/deferred-membership",
                            "pkg::main::Owner.fraenkel.deferred-membership",
                            direct(111, 112),
                            provenance("checker:fraenkel:deferred-membership"),
                        ),
                    )),
                    missing_sethood_obligation: None,
                },
                111,
            ),
        ];
        assert!(matches!(
            lower_term_and_formula_inputs(&context, deferred_membership),
            Err(TermAndFormulaLoweringError::InvalidFraenkelMembershipObligation {
                kind,
                status: ObligationSeedStatus::Deferred,
            }) if kind == ObligationSeedKind::FraenkelMembershipAxiom
        ));
    }

    #[test]
    fn ordinary_fraenkel_sethood_evidence_does_not_need_template_record() {
        let x = CoreVarId::new(0);
        let (context, owner) = context_with_var(x);
        let degraded = template_sethood_seed(
            "T",
            "T:degraded-unused",
            TemplateTypeParameterSethoodSource::BoundInherited,
            TemplateTypeParameterSethoodStatus::DegradedRecovery,
            199,
        );
        let mut type_input = TypeAndFactLoweringInput::new(owner);
        type_input.template_type_parameter_sethoods = vec![degraded];
        let type_output = lower_type_and_fact_inputs(&context, type_input).expect("type lowering");

        let mut input = TermAndFormulaLoweringInput::new(owner);
        input.template_type_parameter_sethoods =
            type_output.template_type_parameter_sethoods.clone();
        input.terms = vec![
            term_seed(CoreTermSeedKind::Var(x), 200),
            term_seed(
                CoreTermSeedKind::Fraenkel {
                    functor: symbol("ordinary_fraenkel"),
                    origin_functor: symbol("ordinary_fraenkel"),
                    key: GeneratedOriginKey::new("fraenkel:ordinary"),
                    params: vec![x],
                    args: vec![CoreTermSeedId::new(0)],
                    sethood_evidence: vec![CoreProvenance::new(
                        CoreProvenancePhase::Checker,
                        "checker:fraenkel:ordinary-sethood",
                    )],
                    template_type_parameter_sethood: None,
                    membership_obligation: Box::new(
                        FraenkelMembershipObligationSeed::AlreadyCarried(
                            AlreadyCarriedFraenkelMembershipSeed {
                                source: direct(201, 202),
                                provenance: provenance("checker:fraenkel:ordinary-carried"),
                            },
                        ),
                    ),
                    missing_sethood_obligation: None,
                },
                201,
            ),
        ];

        let output = lower_term_and_formula_inputs(&context, input).expect("lowering");

        assert_eq!(output.generated_origin_refs.len(), 1);
        assert!(matches!(
            output
                .terms
                .get(output.term_map[&CoreTermSeedId::new(1)])
                .expect("fraenkel")
                .kind,
            CoreTermKind::Apply { .. }
        ));
        let generated = output
            .generated
            .get(output.generated_origin_refs[0].origin)
            .expect("ordinary fraenkel origin");
        assert!(generated.evidence.iter().any(|entry| {
            entry.phase == CoreProvenancePhase::Checker
                && entry.key.as_str() == "checker:fraenkel:ordinary-sethood"
        }));
        assert!(!generated.evidence.iter().any(|entry| {
            entry.phase == CoreProvenancePhase::Checker
                && entry.key.as_str() == "checker:template-sethood:T"
        }));
        assert!(output.diagnostics.is_empty());
        assert!(output.generated_obligations.is_empty());
        assert!(output.obligation_seeds.is_empty());
        assert_step2_delta_valid(&context, &type_output);
        assert_step3_delta_valid(&context, &output);
    }

    #[test]
    fn fraenkel_template_type_parameter_sethood_records_gate_generated_origins() {
        let x = CoreVarId::new(0);
        let (context, owner) = context_with_var(x);
        let mut bound = template_sethood_seed(
            "T",
            "T:bound:Nat",
            TemplateTypeParameterSethoodSource::BoundInherited,
            TemplateTypeParameterSethoodStatus::Accepted,
            210,
        );
        bound.facts = vec![TypeFactId::new(20)];
        let mut constraint = template_sethood_seed(
            "U",
            "U:constraint:sethood",
            TemplateTypeParameterSethoodSource::ConstraintSupplied,
            TemplateTypeParameterSethoodStatus::Accepted,
            211,
        );
        constraint.facts = vec![TypeFactId::new(21)];
        let mut type_input = TypeAndFactLoweringInput::new(owner);
        type_input.template_type_parameter_sethoods = vec![bound, constraint];
        let type_output = lower_type_and_fact_inputs(&context, type_input).expect("type lowering");

        let mut input = TermAndFormulaLoweringInput::new(owner);
        input.template_type_parameter_sethoods =
            type_output.template_type_parameter_sethoods.clone();
        input.terms = vec![
            term_seed(CoreTermSeedKind::Var(x), 212),
            term_seed(
                CoreTermSeedKind::Fraenkel {
                    functor: symbol("template_fraenkel_T"),
                    origin_functor: symbol("template_fraenkel_T"),
                    key: GeneratedOriginKey::new("fraenkel:template:T"),
                    params: vec![x],
                    args: vec![CoreTermSeedId::new(0)],
                    sethood_evidence: Vec::new(),
                    template_type_parameter_sethood: Some(template_fraenkel_sethood(
                        "T",
                        "T:bound:Nat",
                        210,
                        213,
                    )),
                    membership_obligation: Box::new(FraenkelMembershipObligationSeed::New(
                        CoreObligationSeed::active(
                            ObligationSeedKind::FraenkelMembershipAxiom,
                            CoreFormulaSeedId::new(0),
                            "fraenkel/template/T-membership",
                            "pkg::main::Owner.fraenkel.template.T-membership",
                            direct(214, 215),
                            provenance("checker:fraenkel:template:T-membership"),
                        ),
                    )),
                    missing_sethood_obligation: None,
                },
                213,
            ),
            term_seed(
                CoreTermSeedKind::Fraenkel {
                    functor: symbol("template_fraenkel_U"),
                    origin_functor: symbol("template_fraenkel_U"),
                    key: GeneratedOriginKey::new("fraenkel:template:U"),
                    params: vec![x],
                    args: vec![CoreTermSeedId::new(0)],
                    sethood_evidence: Vec::new(),
                    template_type_parameter_sethood: Some(template_fraenkel_sethood(
                        "U",
                        "U:constraint:sethood",
                        211,
                        216,
                    )),
                    membership_obligation: Box::new(
                        FraenkelMembershipObligationSeed::AlreadyCarried(
                            AlreadyCarriedFraenkelMembershipSeed {
                                source: direct(217, 218),
                                provenance: provenance("checker:fraenkel:template:U-carried"),
                            },
                        ),
                    ),
                    missing_sethood_obligation: None,
                },
                216,
            ),
        ];
        input.formulas = vec![formula_seed(CoreFormulaSeedKind::True, 215)];

        let output = lower_term_and_formula_inputs(&context, input).expect("term lowering");

        assert!(output.diagnostics.is_empty());
        assert_eq!(output.generated_origin_refs.len(), 2);
        assert_eq!(output.generated_obligations.len(), 1);
        assert_eq!(output.already_carried_generated_obligations.len(), 1);
        for (index, expected_template_key, expected_fraenkel_key) in [
            (
                0,
                "checker:template-sethood:T",
                "checker:fraenkel-template-sethood:213",
            ),
            (
                1,
                "checker:template-sethood:U",
                "checker:fraenkel-template-sethood:216",
            ),
        ] {
            let generated = output
                .generated
                .get(output.generated_origin_refs[index].origin)
                .expect("template fraenkel origin");
            assert_eq!(generated.kind, GeneratedOriginKind::FraenkelComprehension);
            assert!(generated.evidence.iter().any(|entry| {
                entry.phase == CoreProvenancePhase::Checker
                    && entry.key.as_str() == expected_template_key
            }));
            assert!(generated.evidence.iter().any(|entry| {
                entry.phase == CoreProvenancePhase::Checker
                    && entry.key.as_str() == expected_fraenkel_key
            }));
        }
        assert_step2_delta_valid(&context, &type_output);
        assert_step3_delta_valid(&context, &output);
    }

    #[test]
    fn fraenkel_bare_template_type_parameter_remains_missing_sethood_error() {
        let x = CoreVarId::new(0);
        let (context, owner) = context_with_var(x);
        let bare = template_sethood_seed(
            "Bare",
            "Bare:bare",
            TemplateTypeParameterSethoodSource::BareParameter,
            TemplateTypeParameterSethoodStatus::Missing,
            220,
        );
        let mut type_input = TypeAndFactLoweringInput::new(owner);
        type_input.template_type_parameter_sethoods = vec![bare];
        let type_output = lower_type_and_fact_inputs(&context, type_input).expect("type lowering");

        let mut input = TermAndFormulaLoweringInput::new(owner);
        input.template_type_parameter_sethoods =
            type_output.template_type_parameter_sethoods.clone();
        input.terms = vec![
            term_seed(CoreTermSeedKind::Var(x), 221),
            term_seed(
                CoreTermSeedKind::Fraenkel {
                    functor: symbol("bare_template_fraenkel"),
                    origin_functor: symbol("bare_template_fraenkel"),
                    key: GeneratedOriginKey::new("fraenkel:template:bare"),
                    params: vec![x],
                    args: vec![CoreTermSeedId::new(0)],
                    sethood_evidence: vec![CoreProvenance::new(
                        CoreProvenancePhase::Checker,
                        "checker:fraenkel:bare-template-raw",
                    )],
                    template_type_parameter_sethood: Some(template_fraenkel_sethood(
                        "Bare",
                        "Bare:bare",
                        220,
                        222,
                    )),
                    membership_obligation: Box::new(
                        FraenkelMembershipObligationSeed::AlreadyCarried(
                            AlreadyCarriedFraenkelMembershipSeed {
                                source: direct(223, 224),
                                provenance: provenance("checker:fraenkel:bare-carried"),
                            },
                        ),
                    ),
                    missing_sethood_obligation: Some(Box::new(CoreObligationSeed::deferred(
                        ObligationSeedKind::GeneratedSethood,
                        "fraenkel/template/bare-missing-sethood",
                        "pkg::main::Owner.fraenkel.template.bare-missing-sethood",
                        direct(224, 225),
                        provenance("checker:fraenkel:bare-missing-sethood"),
                    ))),
                },
                222,
            ),
        ];

        let output = lower_term_and_formula_inputs(&context, input).expect("term lowering");
        let CoreTermKind::Error(diagnostic_id) = output
            .terms
            .get(output.term_map[&CoreTermSeedId::new(1)])
            .expect("bare template fraenkel")
            .kind
        else {
            panic!("expected error term");
        };
        let diagnostic = output.diagnostics.get(diagnostic_id).expect("diagnostic");
        let obligation = output
            .obligation_seeds
            .get(output.generated_obligations[0].obligation)
            .expect("generated sethood obligation");

        assert_eq!(
            diagnostic.message_key.as_str(),
            "missing-fraenkel-sethood-evidence"
        );
        assert!(output.generated_origin_refs.is_empty());
        assert_eq!(obligation.kind, ObligationSeedKind::GeneratedSethood);
        assert_eq!(obligation.status, ObligationSeedStatus::Deferred);
        assert_step2_delta_valid(&context, &type_output);
        assert_step3_delta_valid(&context, &output);
    }

    #[test]
    fn fraenkel_template_sethood_cross_references_fail_closed() {
        let x = CoreVarId::new(0);
        let (context, owner) = context_with_var(x);
        let mut seed = template_sethood_seed(
            "T",
            "T:bound:Nat",
            TemplateTypeParameterSethoodSource::BoundInherited,
            TemplateTypeParameterSethoodStatus::Accepted,
            230,
        );
        seed.facts = vec![TypeFactId::new(30)];
        let mut type_input = TypeAndFactLoweringInput::new(owner);
        type_input.template_type_parameter_sethoods = vec![seed];
        let type_output = lower_type_and_fact_inputs(&context, type_input).expect("type lowering");
        let record = type_output.template_type_parameter_sethoods[0].clone();

        let mut duplicate = TermAndFormulaLoweringInput::new(owner);
        duplicate.template_type_parameter_sethoods = vec![record.clone(), record.clone()];
        assert!(matches!(
            lower_term_and_formula_inputs(&context, duplicate),
            Err(
                TermAndFormulaLoweringError::InvalidTemplateFraenkelSethoodEvidence {
                    reason: TemplateSethoodRecordErrorKind::Duplicate,
                    ..
                }
            )
        ));

        let mut missing_record = TermAndFormulaLoweringInput::new(owner);
        missing_record.terms = vec![
            term_seed(CoreTermSeedKind::Var(x), 231),
            term_seed(
                CoreTermSeedKind::Fraenkel {
                    functor: symbol("missing_template_sethood"),
                    origin_functor: symbol("missing_template_sethood"),
                    key: GeneratedOriginKey::new("fraenkel:template:missing-record"),
                    params: vec![x],
                    args: vec![CoreTermSeedId::new(0)],
                    sethood_evidence: Vec::new(),
                    template_type_parameter_sethood: Some(template_fraenkel_sethood(
                        "T",
                        "T:bound:Nat",
                        230,
                        232,
                    )),
                    membership_obligation: Box::new(
                        FraenkelMembershipObligationSeed::AlreadyCarried(
                            AlreadyCarriedFraenkelMembershipSeed {
                                source: direct(233, 234),
                                provenance: provenance("checker:fraenkel:missing-record-carried"),
                            },
                        ),
                    ),
                    missing_sethood_obligation: None,
                },
                232,
            ),
        ];
        assert!(matches!(
            lower_term_and_formula_inputs(&context, missing_record),
            Err(
                TermAndFormulaLoweringError::InvalidTemplateFraenkelSethoodEvidence {
                    reason: TemplateSethoodRecordErrorKind::Missing,
                    ..
                }
            )
        ));

        let mut normalized_mismatch = TermAndFormulaLoweringInput::new(owner);
        normalized_mismatch.template_type_parameter_sethoods = vec![record.clone()];
        normalized_mismatch.terms = vec![
            term_seed(CoreTermSeedKind::Var(x), 234),
            term_seed(
                CoreTermSeedKind::Fraenkel {
                    functor: symbol("mismatched_template_sethood"),
                    origin_functor: symbol("mismatched_template_sethood"),
                    key: GeneratedOriginKey::new("fraenkel:template:normalized-mismatch"),
                    params: vec![x],
                    args: vec![CoreTermSeedId::new(0)],
                    sethood_evidence: Vec::new(),
                    template_type_parameter_sethood: Some(template_fraenkel_sethood(
                        "T",
                        "T:bound:Nat",
                        231,
                        235,
                    )),
                    membership_obligation: Box::new(
                        FraenkelMembershipObligationSeed::AlreadyCarried(
                            AlreadyCarriedFraenkelMembershipSeed {
                                source: direct(236, 237),
                                provenance: provenance(
                                    "checker:fraenkel:normalized-mismatch-carried",
                                ),
                            },
                        ),
                    ),
                    missing_sethood_obligation: None,
                },
                235,
            ),
        ];
        assert!(matches!(
            lower_term_and_formula_inputs(&context, normalized_mismatch),
            Err(
                TermAndFormulaLoweringError::InvalidTemplateFraenkelSethoodEvidence {
                    reason: TemplateSethoodRecordErrorKind::NormalizedTypeMismatch,
                    ..
                }
            )
        ));

        let referenced_malformed_input = |record: TemplateTypeParameterSethood, start: usize| {
            let mut input = TermAndFormulaLoweringInput::new(owner);
            input.template_type_parameter_sethoods = vec![record];
            input.terms = vec![
                term_seed(CoreTermSeedKind::Var(x), start),
                term_seed(
                    CoreTermSeedKind::Fraenkel {
                        functor: symbol("malformed_template_sethood"),
                        origin_functor: symbol("malformed_template_sethood"),
                        key: GeneratedOriginKey::new(format!(
                            "fraenkel:template:malformed:{start}"
                        )),
                        params: vec![x],
                        args: vec![CoreTermSeedId::new(0)],
                        sethood_evidence: Vec::new(),
                        template_type_parameter_sethood: Some(template_fraenkel_sethood(
                            "T",
                            "T:bound:Nat",
                            230,
                            start + 1,
                        )),
                        membership_obligation: Box::new(
                            FraenkelMembershipObligationSeed::AlreadyCarried(
                                AlreadyCarriedFraenkelMembershipSeed {
                                    source: direct(start + 2, start + 3),
                                    provenance: provenance(
                                        format!("checker:fraenkel:malformed-carried:{start}")
                                            .as_str(),
                                    ),
                                },
                            ),
                        ),
                        missing_sethood_obligation: None,
                    },
                    start + 1,
                ),
            ];
            input
        };

        let mut degraded = record.clone();
        degraded.status = TemplateTypeParameterSethoodStatus::DegradedRecovery;
        degraded.facts.clear();
        let wrong_status = referenced_malformed_input(degraded, 238);
        assert!(matches!(
            lower_term_and_formula_inputs(&context, wrong_status),
            Err(
                TermAndFormulaLoweringError::InvalidTemplateFraenkelSethoodEvidence {
                    reason: TemplateSethoodRecordErrorKind::NotAccepted,
                    ..
                }
            )
        ));

        let mut accepted_bare = record.clone();
        accepted_bare.source_kind = TemplateTypeParameterSethoodSource::BareParameter;
        let wrong_source = referenced_malformed_input(accepted_bare, 242);
        assert!(matches!(
            lower_term_and_formula_inputs(&context, wrong_source),
            Err(
                TermAndFormulaLoweringError::InvalidTemplateFraenkelSethoodEvidence {
                    reason: TemplateSethoodRecordErrorKind::WrongSource,
                    ..
                }
            )
        ));

        let mut missing_evidence = record.clone();
        missing_evidence.facts.clear();
        let missing_evidence_input = referenced_malformed_input(missing_evidence, 246);
        assert!(matches!(
            lower_term_and_formula_inputs(&context, missing_evidence_input),
            Err(
                TermAndFormulaLoweringError::InvalidTemplateFraenkelSethoodEvidence {
                    reason: TemplateSethoodRecordErrorKind::MissingEvidence,
                    ..
                }
            )
        ));

        let mut generated_provenance = record.clone();
        generated_provenance.provenance = vec![CoreProvenance::new(
            CoreProvenancePhase::Generated,
            "generated:template-sethood",
        )];
        let generated_provenance_input = referenced_malformed_input(generated_provenance, 250);
        assert!(matches!(
            lower_term_and_formula_inputs(&context, generated_provenance_input),
            Err(TermAndFormulaLoweringError::InvalidSeedProvenance(_))
        ));

        let mut bad_evidence_seed = template_fraenkel_sethood("T", "T:bound:Nat", 230, 254);
        bad_evidence_seed.provenance = CheckerOwnedProvenance {
            entries: vec![CoreProvenance::new(
                CoreProvenancePhase::Generated,
                "generated:fraenkel-template-sethood",
            )],
        };
        let mut generated_evidence_seed_input = TermAndFormulaLoweringInput::new(owner);
        generated_evidence_seed_input.template_type_parameter_sethoods = vec![record.clone()];
        generated_evidence_seed_input.terms = vec![
            term_seed(CoreTermSeedKind::Var(x), 253),
            term_seed(
                CoreTermSeedKind::Fraenkel {
                    functor: symbol("bad_template_evidence_seed"),
                    origin_functor: symbol("bad_template_evidence_seed"),
                    key: GeneratedOriginKey::new("fraenkel:template:bad-evidence-seed"),
                    params: vec![x],
                    args: vec![CoreTermSeedId::new(0)],
                    sethood_evidence: Vec::new(),
                    template_type_parameter_sethood: Some(bad_evidence_seed),
                    membership_obligation: Box::new(
                        FraenkelMembershipObligationSeed::AlreadyCarried(
                            AlreadyCarriedFraenkelMembershipSeed {
                                source: direct(255, 256),
                                provenance: provenance(
                                    "checker:fraenkel:bad-evidence-seed-carried",
                                ),
                            },
                        ),
                    ),
                    missing_sethood_obligation: None,
                },
                254,
            ),
        ];
        assert!(matches!(
            lower_term_and_formula_inputs(&context, generated_evidence_seed_input),
            Err(TermAndFormulaLoweringError::InvalidSeedProvenance(_))
        ));
    }

    #[test]
    fn fraenkel_missing_sethood_remains_error_and_deferred_seed() {
        let x = CoreVarId::new(0);
        let (context, owner) = context_with_var(x);
        let mut input = TermAndFormulaLoweringInput::new(owner);
        input.terms = vec![
            term_seed(CoreTermSeedKind::Var(x), 104),
            term_seed(
                CoreTermSeedKind::Fraenkel {
                    functor: symbol("fraenkel_missing"),
                    origin_functor: symbol("fraenkel_missing"),
                    key: GeneratedOriginKey::new("fraenkel:missing"),
                    params: vec![x],
                    args: vec![CoreTermSeedId::new(0)],
                    sethood_evidence: Vec::new(),
                    template_type_parameter_sethood: None,
                    membership_obligation: Box::new(
                        FraenkelMembershipObligationSeed::AlreadyCarried(
                            AlreadyCarriedFraenkelMembershipSeed {
                                source: direct(105, 106),
                                provenance: provenance("checker:fraenkel:already-carried"),
                            },
                        ),
                    ),
                    missing_sethood_obligation: Some(Box::new(CoreObligationSeed::deferred(
                        ObligationSeedKind::GeneratedSethood,
                        "fraenkel/missing-sethood",
                        "pkg::main::Owner.fraenkel.missing-sethood",
                        direct(106, 107),
                        provenance("checker:fraenkel:missing-sethood"),
                    ))),
                },
                105,
            ),
        ];

        let output = lower_term_and_formula_inputs(&context, input).expect("lowering");
        let fraenkel_term = output.term_map[&CoreTermSeedId::new(1)];
        let CoreTermKind::Error(diagnostic_id) =
            output.terms.get(fraenkel_term).expect("error term").kind
        else {
            panic!("expected error term");
        };
        let diagnostic = output.diagnostics.get(diagnostic_id).expect("diagnostic");
        let obligation = output
            .obligation_seeds
            .get(output.generated_obligations[0].obligation)
            .expect("deferred obligation");

        assert_eq!(
            diagnostic.message_key.as_str(),
            "missing-fraenkel-sethood-evidence"
        );
        assert!(output.generated_origin_refs.is_empty());
        assert_eq!(obligation.status, ObligationSeedStatus::Deferred);
        assert_eq!(obligation.kind, ObligationSeedKind::GeneratedSethood);
        assert_step3_delta_valid(&context, &output);

        let mut bad_missing = TermAndFormulaLoweringInput::new(owner);
        bad_missing.terms = vec![
            term_seed(CoreTermSeedKind::Var(x), 109),
            term_seed(
                CoreTermSeedKind::Fraenkel {
                    functor: symbol("fraenkel_missing"),
                    origin_functor: symbol("fraenkel_missing"),
                    key: GeneratedOriginKey::new("fraenkel:bad-missing"),
                    params: vec![x],
                    args: vec![CoreTermSeedId::new(0)],
                    sethood_evidence: Vec::new(),
                    template_type_parameter_sethood: None,
                    membership_obligation: Box::new(
                        FraenkelMembershipObligationSeed::AlreadyCarried(
                            AlreadyCarriedFraenkelMembershipSeed {
                                source: direct(109, 110),
                                provenance: provenance("checker:fraenkel:already-carried:bad"),
                            },
                        ),
                    ),
                    missing_sethood_obligation: Some(Box::new(CoreObligationSeed::deferred(
                        ObligationSeedKind::DefinitionCorrectness,
                        "fraenkel/bad-missing-sethood",
                        "pkg::main::Owner.fraenkel.bad-missing-sethood",
                        direct(110, 111),
                        provenance("checker:fraenkel:bad-missing-sethood"),
                    ))),
                },
                110,
            ),
        ];
        assert!(matches!(
            lower_term_and_formula_inputs(&context, bad_missing),
            Err(TermAndFormulaLoweringError::InvalidFraenkelMissingSethoodObligation { kind })
                if kind == ObligationSeedKind::DefinitionCorrectness
        ));
    }

    #[test]
    fn failed_semantic_sites_lower_to_error_nodes_and_diagnostics() {
        let x = CoreVarId::new(0);
        let (context, owner) = context_with_var(x);
        let mut input = TermAndFormulaLoweringInput::new(owner);
        input.terms = vec![term_seed(
            CoreTermSeedKind::Error(failed_site("failed-term-overload", 108)),
            108,
        )];
        input.formulas = vec![formula_seed(
            CoreFormulaSeedKind::Error(failed_site("unsupported-formula", 109)),
            109,
        )];
        input.failed_sites = vec![failed_site("standalone-failed-site", 110)];

        let output = lower_term_and_formula_inputs(&context, input).expect("lowering");
        let term = output.term_map[&CoreTermSeedId::new(0)];
        let formula = output.formula_map[&CoreFormulaSeedId::new(0)];

        assert!(matches!(
            output.terms.get(term).expect("term").kind,
            CoreTermKind::Error(_)
        ));
        assert!(matches!(
            output.formulas.get(formula).expect("formula").kind,
            CoreFormulaKind::Error(_)
        ));
        assert_eq!(output.failed_sites.len(), 1);
        assert_eq!(output.diagnostics.len(), 3);
        assert_step3_delta_valid(&context, &output);
    }

    #[test]
    fn definition_lowering_records_boundary_policy_params_and_correctness() {
        let x = CoreVarId::new(0);
        let (context, owner) = context_with_var(x);
        let term_formula = lower_test_terms_and_formulas(
            &context,
            owner,
            vec![term_seed(CoreTermSeedKind::Var(x), 112)],
            vec![formula_seed(CoreFormulaSeedKind::True, 113)],
        );
        let term = term_formula.term_map[&CoreTermSeedId::new(0)];
        let goal = term_formula.formula_map[&CoreFormulaSeedId::new(0)];
        let mut seed = definition_seed(owner, symbol("Owner"), DefinitionBodySeed::Term(term), 114);
        seed.params = vec![test_binder(x, Some(goal), 115)];
        seed.correctness = vec![DefinitionCorrectnessSeed::New(Box::new(
            DefinitionObligationSeed::active(
                goal,
                "definition/owner/coherence",
                "pkg::main::Owner.definition.coherence",
                direct(116, 117),
                provenance("checker:definition:coherence"),
            ),
        ))];

        let output = lower_definition_inputs(
            &context,
            &term_formula,
            DefinitionLoweringInput {
                definitions: vec![seed],
            },
        )
        .expect("definition lowering");
        let definition_id = output.definition_map[&owner];
        let definition = output
            .definitions
            .get(definition_id)
            .expect("definition row");
        let obligation_id = definition.correctness[0];
        let obligation = output
            .obligation_seeds
            .get(obligation_id)
            .expect("correctness obligation");

        assert_eq!(definition.owner.anchor_item(), owner);
        assert_eq!(definition.owner.item(), Some(owner));
        assert_eq!(definition.owner.property_symbol(), None);
        assert!(matches!(definition.body, DefinitionBody::Term(actual) if actual == term));
        assert_eq!(definition.expansion, ExpansionPolicy::Opaque);
        assert_eq!(definition.params[0].ty_guard, Some(goal));
        assert_eq!(output.correctness_obligations.len(), 1);
        assert!(output.correctness_obligations[0].is_new);
        assert_eq!(obligation.kind, ObligationSeedKind::DefinitionCorrectness);
        assert_eq!(obligation.status, ObligationSeedStatus::Active);
        assert!(obligation.core_refs.contains(&CoreNodeRef::Item(owner)));
        assert!(
            obligation
                .core_refs
                .contains(&CoreNodeRef::Definition(definition_id))
        );
        assert!(obligation.core_refs.contains(&CoreNodeRef::Term(term)));
        assert!(obligation.core_refs.contains(&CoreNodeRef::Formula(goal)));
        assert_step4_delta_valid(&context, &term_formula, &output);
    }

    #[test]
    fn definition_correctness_handles_deferred_and_existing_seeds_with_backrefs() {
        let x = CoreVarId::new(0);
        let (context, owner) = context_with_var(x);
        let mut term_formula = lower_test_terms_and_formulas(
            &context,
            owner,
            vec![term_seed(CoreTermSeedKind::Var(x), 118)],
            vec![formula_seed(CoreFormulaSeedKind::True, 119)],
        );
        let term = term_formula.term_map[&CoreTermSeedId::new(0)];
        let goal = term_formula.formula_map[&CoreFormulaSeedId::new(0)];
        let existing_source = direct(120, 121);
        let existing = term_formula.obligation_seeds.insert(ObligationSeed {
            owner,
            kind: ObligationSeedKind::DefinitionCorrectness,
            goal: Some(goal),
            context: Vec::new(),
            local_path: LocalProofOrProgramPath::new("definition/existing"),
            label: None,
            semantic_origin: NormalizedSemanticOrigin::new("pkg::main::Owner.definition.existing"),
            provenance: vec![CoreProvenance::new(
                CoreProvenancePhase::Checker,
                "checker:definition:existing",
            )],
            source: existing_source.clone(),
            core_refs: vec![CoreNodeRef::Item(owner), CoreNodeRef::Formula(goal)],
            status: ObligationSeedStatus::Active,
            diagnostics: Vec::new(),
        });
        term_formula
            .source_map
            .obligation_sources
            .insert(existing, existing_source);

        let mut seed = definition_seed(owner, symbol("Owner"), DefinitionBodySeed::Term(term), 121);
        seed.correctness = vec![
            DefinitionCorrectnessSeed::Existing(existing),
            DefinitionCorrectnessSeed::New(Box::new(DefinitionObligationSeed::deferred(
                "definition/deferred",
                "pkg::main::Owner.definition.deferred",
                direct(122, 123),
                provenance("checker:definition:deferred"),
            ))),
        ];

        let output = lower_definition_inputs(
            &context,
            &term_formula,
            DefinitionLoweringInput {
                definitions: vec![seed],
            },
        )
        .expect("definition lowering");
        let definition = output.definition_map[&owner];
        let row = output.definitions.get(definition).expect("definition");
        let existing_seed = output
            .obligation_seeds
            .get(existing)
            .expect("existing obligation");
        let deferred = row.correctness[1];
        let deferred_seed = output
            .obligation_seeds
            .get(deferred)
            .expect("deferred obligation");

        assert_eq!(row.correctness[0], existing);
        assert_eq!(output.correctness_obligations.len(), 2);
        assert!(!output.correctness_obligations[0].is_new);
        assert!(output.correctness_obligations[1].is_new);
        assert!(
            existing_seed
                .core_refs
                .contains(&CoreNodeRef::Definition(definition))
        );
        assert!(existing_seed.core_refs.contains(&CoreNodeRef::Term(term)));
        assert_eq!(deferred_seed.status, ObligationSeedStatus::Deferred);
        assert!(deferred_seed.goal.is_none());
        assert!(
            deferred_seed
                .core_refs
                .contains(&CoreNodeRef::Definition(definition))
        );
        assert!(deferred_seed.core_refs.contains(&CoreNodeRef::Term(term)));
        assert_step4_delta_valid(&context, &term_formula, &output);
    }

    #[test]
    fn definition_correctness_existing_seed_must_match_definition_owner() {
        let context = prepare_core_context(input_with_items(vec![
            item_seed("Owner", 118),
            item_seed("OtherOwner", 119),
        ]))
        .expect("context");
        let owner = context
            .item_registry()
            .id_for_symbol(&symbol("Owner"))
            .expect("owner");
        let other_owner = context
            .item_registry()
            .id_for_symbol(&symbol("OtherOwner"))
            .expect("other owner");
        let mut term_formula = lower_test_terms_and_formulas(
            &context,
            owner,
            Vec::new(),
            vec![formula_seed(CoreFormulaSeedKind::True, 120)],
        );
        let goal = term_formula.formula_map[&CoreFormulaSeedId::new(0)];
        let source = direct(121, 122);
        let existing = term_formula.obligation_seeds.insert(ObligationSeed {
            owner: other_owner,
            kind: ObligationSeedKind::DefinitionCorrectness,
            goal: Some(goal),
            context: Vec::new(),
            local_path: LocalProofOrProgramPath::new("definition/other-owner"),
            label: None,
            semantic_origin: NormalizedSemanticOrigin::new(
                "pkg::main::OtherOwner.definition.correctness",
            ),
            provenance: vec![CoreProvenance::new(
                CoreProvenancePhase::Checker,
                "checker:definition:other-owner",
            )],
            source: source.clone(),
            core_refs: vec![CoreNodeRef::Item(other_owner), CoreNodeRef::Formula(goal)],
            status: ObligationSeedStatus::Active,
            diagnostics: Vec::new(),
        });
        term_formula
            .source_map
            .obligation_sources
            .insert(existing, source);
        let mut seed = definition_seed(
            owner,
            symbol("Owner"),
            DefinitionBodySeed::Formula(goal),
            122,
        );
        seed.correctness = vec![DefinitionCorrectnessSeed::Existing(existing)];

        assert!(matches!(
            lower_definition_inputs(
                &context,
                &term_formula,
                DefinitionLoweringInput {
                    definitions: vec![seed],
                },
            ),
            Err(DefinitionLoweringError::ExistingCorrectnessOwnerMismatch {
                obligation,
                expected,
                actual,
            }) if obligation == existing && expected == owner && actual == other_owner
        ));
    }

    #[test]
    fn definition_lowering_preserves_all_expansion_policies() {
        let mut input = input_with_items(vec![
            CoreItemSeed::new(
                symbol("OpaqueDef"),
                CoreItemKind::Functor,
                "public",
                direct(120, 121),
                provenance("checker:item:opaque-def"),
            )
            .with_definition_boundary(DefinitionBoundaryKind::DefinitionalItem),
            CoreItemSeed::new(
                symbol("TransparentDef"),
                CoreItemKind::Predicate,
                "public",
                direct(121, 122),
                provenance("checker:item:transparent-def"),
            )
            .with_definition_boundary(DefinitionBoundaryKind::DefinitionalItem),
            CoreItemSeed::new(
                symbol("ReducibleDef"),
                CoreItemKind::Reduction,
                "public",
                direct(122, 123),
                provenance("checker:item:reducible-def"),
            )
            .with_definition_boundary(DefinitionBoundaryKind::Reduction),
            CoreItemSeed::new(
                symbol("ComputableDef"),
                CoreItemKind::Functor,
                "public",
                direct(123, 124),
                provenance("checker:item:computable-def"),
            )
            .with_definition_boundary(DefinitionBoundaryKind::DefinitionalItem),
        ]);
        input.variable_seeds = Vec::new();
        let context = prepare_core_context(input).expect("context");
        let owner = context
            .item_registry()
            .id_for_symbol(&symbol("OpaqueDef"))
            .expect("owner");
        let term_formula = lower_test_terms_and_formulas(
            &context,
            owner,
            vec![term_seed(CoreTermSeedKind::Const(symbol("Const")), 125)],
            vec![formula_seed(CoreFormulaSeedKind::True, 126)],
        );
        let term = term_formula.term_map[&CoreTermSeedId::new(0)];
        let formula = term_formula.formula_map[&CoreFormulaSeedId::new(0)];
        let mut definitions = Vec::new();
        for (name, body, expansion, start) in [
            (
                "OpaqueDef",
                DefinitionBodySeed::Term(term),
                ExpansionPolicy::Opaque,
                127,
            ),
            (
                "TransparentDef",
                DefinitionBodySeed::Formula(formula),
                ExpansionPolicy::Transparent,
                128,
            ),
            (
                "ReducibleDef",
                DefinitionBodySeed::Term(term),
                ExpansionPolicy::Reducible {
                    registration: symbol("ReduceRegistration"),
                },
                129,
            ),
            (
                "ComputableDef",
                DefinitionBodySeed::Term(term),
                ExpansionPolicy::Computable {
                    algorithm: symbol("RuntimeAlgorithm"),
                },
                130,
            ),
        ] {
            let item = context
                .item_registry()
                .id_for_symbol(&symbol(name))
                .expect("definition item");
            let mut seed = definition_seed(item, symbol(name), body, start);
            seed.expansion = expansion;
            definitions.push(seed);
        }

        let output = lower_definition_inputs(
            &context,
            &term_formula,
            DefinitionLoweringInput { definitions },
        )
        .expect("definition lowering");
        let policies = [
            "OpaqueDef",
            "TransparentDef",
            "ReducibleDef",
            "ComputableDef",
        ]
        .iter()
        .map(|name| {
            let item = context
                .item_registry()
                .id_for_symbol(&symbol(name))
                .expect("definition item");
            output
                .definitions
                .get(output.definition_map[&item])
                .expect("definition")
                .expansion
                .clone()
        })
        .collect::<Vec<_>>();

        assert!(matches!(policies[0], ExpansionPolicy::Opaque));
        assert!(matches!(policies[1], ExpansionPolicy::Transparent));
        assert!(matches!(policies[2], ExpansionPolicy::Reducible { .. }));
        assert!(matches!(policies[3], ExpansionPolicy::Computable { .. }));
        assert_step4_delta_valid(&context, &term_formula, &output);
    }

    #[test]
    fn definition_lowering_rejects_invalid_definition_boundaries() {
        let context = prepare_core_context(input_with_items(vec![
            item_seed("HasBoundary", 131),
            CoreItemSeed::new(
                symbol("NoBoundary"),
                CoreItemKind::Functor,
                "public",
                direct(132, 133),
                provenance("checker:item:no-boundary"),
            ),
            CoreItemSeed::new(
                symbol("AlgorithmBoundary"),
                CoreItemKind::Algorithm,
                "public",
                direct(133, 134),
                provenance("checker:item:algorithm-boundary"),
            )
            .with_definition_boundary(DefinitionBoundaryKind::Algorithm),
        ]))
        .expect("context");
        let owner = context
            .item_registry()
            .id_for_symbol(&symbol("HasBoundary"))
            .expect("owner");
        let no_boundary = context
            .item_registry()
            .id_for_symbol(&symbol("NoBoundary"))
            .expect("no boundary");
        let algorithm = context
            .item_registry()
            .id_for_symbol(&symbol("AlgorithmBoundary"))
            .expect("algorithm boundary");
        let term_formula = lower_test_terms_and_formulas(
            &context,
            owner,
            Vec::new(),
            vec![formula_seed(CoreFormulaSeedKind::True, 134)],
        );
        let formula = term_formula.formula_map[&CoreFormulaSeedId::new(0)];

        let duplicate = definition_seed(
            owner,
            symbol("HasBoundary"),
            DefinitionBodySeed::Formula(formula),
            135,
        );
        assert!(matches!(
            lower_definition_inputs(
                &context,
                &term_formula,
                DefinitionLoweringInput {
                    definitions: vec![duplicate.clone(), duplicate],
                },
            ),
            Err(DefinitionLoweringError::DuplicateDefinitionOwner { owner: actual })
                if actual == owner
        ));

        assert!(matches!(
            lower_definition_inputs(
                &context,
                &term_formula,
                DefinitionLoweringInput {
                    definitions: vec![definition_seed(
                        owner,
                        symbol("WrongSymbol"),
                        DefinitionBodySeed::Formula(formula),
                        136,
                    )],
                },
            ),
            Err(DefinitionLoweringError::DefinitionSymbolMismatch { owner: actual, .. })
                if actual == owner
        ));

        assert!(matches!(
            lower_definition_inputs(
                &context,
                &term_formula,
                DefinitionLoweringInput {
                    definitions: vec![definition_seed(
                        no_boundary,
                        symbol("NoBoundary"),
                        DefinitionBodySeed::Formula(formula),
                        137,
                    )],
                },
            ),
            Err(DefinitionLoweringError::MissingDefinitionBoundary { owner: actual })
                if actual == no_boundary
        ));

        assert!(matches!(
            lower_definition_inputs(
                &context,
                &term_formula,
                DefinitionLoweringInput {
                    definitions: vec![definition_seed(
                        algorithm,
                        symbol("AlgorithmBoundary"),
                        DefinitionBodySeed::Formula(formula),
                        138,
                    )],
                },
            ),
            Err(DefinitionLoweringError::AlgorithmBoundaryRequiresDeferredBody { owner: actual })
                if actual == algorithm
        ));
    }

    #[test]
    fn guarded_definition_otherwise_records_checker_owned_exclusions() {
        let x = CoreVarId::new(0);
        let (context, owner) = context_with_var(x);
        let term_formula = lower_test_terms_and_formulas(
            &context,
            owner,
            vec![term_seed(CoreTermSeedKind::Var(x), 132)],
            vec![
                formula_seed(CoreFormulaSeedKind::True, 133),
                formula_seed(CoreFormulaSeedKind::False, 134),
                formula_seed(CoreFormulaSeedKind::True, 135),
            ],
        );
        let term = term_formula.term_map[&CoreTermSeedId::new(0)];
        let guard_a = term_formula.formula_map[&CoreFormulaSeedId::new(0)];
        let guard_b = term_formula.formula_map[&CoreFormulaSeedId::new(1)];
        let otherwise = term_formula.formula_map[&CoreFormulaSeedId::new(2)];
        let guarded = DefinitionBodySeed::Guarded(vec![
            GuardedDefinitionBranchSeed {
                guard: DefinitionGuardSeed::Explicit(guard_a),
                body: DefinitionBranchBody::Term(term),
            },
            GuardedDefinitionBranchSeed {
                guard: DefinitionGuardSeed::Explicit(guard_b),
                body: DefinitionBranchBody::Formula(guard_b),
            },
            GuardedDefinitionBranchSeed {
                guard: DefinitionGuardSeed::Otherwise {
                    guard: otherwise,
                    excludes: vec![guard_a, guard_b],
                    provenance: provenance("checker:otherwise"),
                },
                body: DefinitionBranchBody::Term(term),
            },
        ]);

        let output = lower_definition_inputs(
            &context,
            &term_formula,
            DefinitionLoweringInput {
                definitions: vec![definition_seed(owner, symbol("Owner"), guarded, 136)],
            },
        )
        .expect("definition lowering");
        let definition = output
            .definitions
            .get(output.definition_map[&owner])
            .expect("definition");

        assert!(matches!(
            &definition.body,
            DefinitionBody::Guarded(branches)
                if branches.len() == 3 && branches[2].guard == otherwise
        ));
        assert_eq!(output.otherwise_guards.len(), 1);
        assert_eq!(output.otherwise_guards[0].excludes, vec![guard_a, guard_b]);
        assert_step4_delta_valid(&context, &term_formula, &output);

        let bad_guarded = DefinitionBodySeed::Guarded(vec![
            GuardedDefinitionBranchSeed {
                guard: DefinitionGuardSeed::Explicit(guard_a),
                body: DefinitionBranchBody::Term(term),
            },
            GuardedDefinitionBranchSeed {
                guard: DefinitionGuardSeed::Otherwise {
                    guard: otherwise,
                    excludes: vec![guard_b],
                    provenance: provenance("checker:otherwise:bad"),
                },
                body: DefinitionBranchBody::Term(term),
            },
        ]);
        assert!(matches!(
            lower_definition_inputs(
                &context,
                &term_formula,
                DefinitionLoweringInput {
                    definitions: vec![definition_seed(owner, symbol("Owner"), bad_guarded, 137)],
                },
            ),
            Err(DefinitionLoweringError::OtherwiseExcludesMismatch { branch: 1 })
        ));
    }

    #[test]
    fn definition_generated_dependencies_are_reachable_through_formula_bodies() {
        let x = CoreVarId::new(0);
        let key = GeneratedOriginKey::new("choice:exported");
        let (context, owner, existing_origin) = context_with_existing_choice_origin(x, key.clone());
        let mut input = TermAndFormulaLoweringInput::new(owner);
        input.terms = vec![
            term_seed(CoreTermSeedKind::Var(x), 138),
            term_seed(
                CoreTermSeedKind::StableChoice {
                    functor: symbol("choice_existing"),
                    origin_functor: symbol("choice_existing"),
                    key,
                    params: vec![x],
                    args: vec![CoreTermSeedId::new(0)],
                    evidence: vec![CoreProvenance::new(
                        CoreProvenancePhase::Checker,
                        "checker:exported-choice",
                    )],
                },
                139,
            ),
        ];
        input.formulas = vec![
            formula_seed(
                CoreFormulaSeedKind::Atom {
                    predicate: symbol("HasChoice"),
                    args: vec![CoreTermSeedId::new(1)],
                },
                140,
            ),
            formula_seed(CoreFormulaSeedKind::True, 141),
        ];
        let term_formula = lower_term_and_formula_inputs(&context, input).expect("lowering");
        let body = term_formula.formula_map[&CoreFormulaSeedId::new(0)];
        let unreachable_body = term_formula.formula_map[&CoreFormulaSeedId::new(1)];
        let choice_term = term_formula.term_map[&CoreTermSeedId::new(1)];
        let mut seed = definition_seed(
            owner,
            symbol("Owner"),
            DefinitionBodySeed::Formula(body),
            142,
        );
        seed.generated_dependencies = vec![existing_origin];

        let output = lower_definition_inputs(
            &context,
            &term_formula,
            DefinitionLoweringInput {
                definitions: vec![seed],
            },
        )
        .expect("definition lowering");

        assert_eq!(output.generated_dependencies.len(), 1);
        assert_eq!(output.generated_dependencies[0].origin, existing_origin);
        assert!(term_formula.generated_delta.is_empty());
        assert!(term_formula.generated_origin_refs[0].reused_existing);
        assert_eq!(
            output.generated_dependencies[0].use_terms,
            vec![choice_term]
        );
        assert_step4_delta_valid(&context, &term_formula, &output);

        let missing_dependency = definition_seed(
            owner,
            symbol("Owner"),
            DefinitionBodySeed::Formula(body),
            143,
        );
        assert!(matches!(
            lower_definition_inputs(
                &context,
                &term_formula,
                DefinitionLoweringInput {
                    definitions: vec![missing_dependency],
                },
            ),
            Err(DefinitionLoweringError::MissingGeneratedDependency { origin })
                if origin == existing_origin
        ));

        let mut spurious_dependency = definition_seed(
            owner,
            symbol("Owner"),
            DefinitionBodySeed::Formula(unreachable_body),
            144,
        );
        spurious_dependency.generated_dependencies = vec![existing_origin];
        assert!(matches!(
            lower_definition_inputs(
                &context,
                &term_formula,
                DefinitionLoweringInput {
                    definitions: vec![spurious_dependency],
                },
            ),
            Err(DefinitionLoweringError::SpuriousGeneratedDependency { origin })
                if origin == existing_origin
        ));
    }

    #[test]
    fn algorithm_and_unavailable_definition_bodies_remain_deferred_or_error() {
        let context = prepare_core_context(input_with_items(vec![
            CoreItemSeed::new(
                symbol("AlgorithmDef"),
                CoreItemKind::Algorithm,
                "public",
                direct(144, 145),
                provenance("checker:item:algorithm-def"),
            )
            .with_definition_boundary(DefinitionBoundaryKind::Algorithm),
            CoreItemSeed::new(
                symbol("UnavailableDef"),
                CoreItemKind::Functor,
                "public",
                direct(146, 147),
                provenance("checker:item:unavailable-def"),
            )
            .with_definition_boundary(DefinitionBoundaryKind::DefinitionalItem),
        ]))
        .expect("context");
        let owner = context
            .item_registry()
            .id_for_symbol(&symbol("AlgorithmDef"))
            .expect("algorithm owner");
        let other_owner = context
            .item_registry()
            .id_for_symbol(&symbol("UnavailableDef"))
            .expect("unavailable owner");
        let term_formula = lower_test_terms_and_formulas(&context, owner, Vec::new(), Vec::new());
        let definitions = vec![
            definition_seed(
                owner,
                symbol("AlgorithmDef"),
                DefinitionBodySeed::AlgorithmDeferred(failed_site("algorithm-body-deferred", 148)),
                148,
            ),
            definition_seed(
                other_owner,
                symbol("UnavailableDef"),
                DefinitionBodySeed::Unavailable(failed_site("definition-prerequisite-error", 149)),
                149,
            ),
        ];

        let output = lower_definition_inputs(
            &context,
            &term_formula,
            DefinitionLoweringInput { definitions },
        )
        .expect("definition lowering");

        assert_eq!(
            output
                .item_status_updates
                .iter()
                .map(|update| (update.item, update.status))
                .collect::<Vec<_>>(),
            vec![
                (owner, CoreItemStatus::Skipped),
                (other_owner, CoreItemStatus::Error)
            ]
        );
        for item in [owner, other_owner] {
            let definition = output
                .definitions
                .get(output.definition_map[&item])
                .expect("definition");
            assert!(matches!(definition.body, DefinitionBody::Unavailable(_)));
        }
        assert_eq!(output.diagnostics.len(), 2);
        assert_step4_delta_valid(&context, &term_formula, &output);
    }

    #[test]
    fn definition_lowering_rejects_nonterm_params_and_wrong_obligations() {
        let x = CoreVarId::new(0);
        let (context, owner) = context_with_var_sort(x, NormalizedVarSort::Formula);
        let term_formula = lower_test_terms_and_formulas(
            &context,
            owner,
            Vec::new(),
            vec![formula_seed(CoreFormulaSeedKind::True, 150)],
        );
        let formula = term_formula.formula_map[&CoreFormulaSeedId::new(0)];
        let mut bad_param = definition_seed(
            owner,
            symbol("Owner"),
            DefinitionBodySeed::Formula(formula),
            151,
        );
        bad_param.params = vec![test_binder(x, None, 152)];

        assert!(matches!(
            lower_definition_inputs(
                &context,
                &term_formula,
                DefinitionLoweringInput {
                    definitions: vec![bad_param],
                },
            ),
            Err(DefinitionLoweringError::NonTermDefinitionParam { var, sort })
                if var == x && sort == NormalizedVarSort::Formula
        ));

        let (context, owner) = context_with_var(CoreVarId::new(1));
        let term_formula = lower_test_terms_and_formulas(
            &context,
            owner,
            Vec::new(),
            vec![formula_seed(CoreFormulaSeedKind::True, 153)],
        );
        let formula = term_formula.formula_map[&CoreFormulaSeedId::new(0)];
        let mut wrong_obligation = definition_seed(
            owner,
            symbol("Owner"),
            DefinitionBodySeed::Formula(formula),
            154,
        );
        wrong_obligation.correctness = vec![DefinitionCorrectnessSeed::New(Box::new(
            DefinitionObligationSeed {
                kind: ObligationSeedKind::TheoremProof,
                status: ObligationSeedStatus::Active,
                goal: Some(formula),
                context: Vec::new(),
                local_path: LocalProofOrProgramPath::new("definition/wrong-kind"),
                label: None,
                semantic_origin: NormalizedSemanticOrigin::new(
                    "pkg::main::Owner.definition.wrong-kind",
                ),
                source: direct(155, 156),
                provenance: provenance("checker:definition:wrong-kind"),
            },
        ))];

        assert!(matches!(
            lower_definition_inputs(
                &context,
                &term_formula,
                DefinitionLoweringInput {
                    definitions: vec![wrong_obligation],
                },
            ),
            Err(DefinitionLoweringError::InvalidCorrectnessObligation { kind, status })
                if kind == ObligationSeedKind::TheoremProof
                    && status == ObligationSeedStatus::Active
        ));
    }

    #[test]
    fn proof_lowering_replaces_thesis_and_emits_terminal_obligation_backrefs() {
        let x = CoreVarId::new(0);
        let (context, owner) = context_with_var(x);
        let term_formula = lower_test_terms_and_formulas(
            &context,
            owner,
            Vec::new(),
            vec![formula_seed(CoreFormulaSeedKind::True, 160)],
        );
        let definitions = empty_definition_output(&context, &term_formula);
        let proposition = term_formula.formula_map[&CoreFormulaSeedId::new(0)];
        let assumption_label = CoreLabelRef::new("A1");
        let terminal_citations = vec![CoreCitation::Label(assumption_label.clone())];
        let terminal = ProofNodeSeed::TerminalGoal(
            ProofTerminalGoalSeed::active(
                ProofFormulaRef::Thesis,
                "proof/terminal",
                "pkg::main::Owner.proof.terminal",
                direct(164, 165),
                provenance("checker:proof:terminal"),
            )
            .with_context(vec![proposition])
            .with_citations(terminal_citations.clone()),
        );
        let skeleton = ProofSkeletonSeed::Node(ProofNodeSeed::IntroduceBinder {
            binder: test_binder(x, Some(proposition), 161),
            child: Box::new(ProofNodeSeed::Assume {
                label: Some(assumption_label.clone()),
                formula: ProofFormulaRef::Thesis,
                child: Box::new(terminal),
                source: direct(162, 163),
                provenance: provenance("checker:proof:assume"),
            }),
            source: direct(161, 162),
            provenance: provenance("checker:proof:introduce"),
        });

        let output = lower_proof_inputs(
            &context,
            &term_formula,
            &definitions,
            ProofLoweringInput {
                proofs: vec![proof_seed(
                    owner,
                    symbol("Owner"),
                    proposition,
                    CoreProofStatus::Open,
                    skeleton,
                    165,
                )],
            },
        )
        .expect("proof lowering");
        let proof_id = output.proof_map[&owner];
        let proof = output.proofs.get(proof_id).expect("proof");
        let CoreProofNodeKind::IntroduceBinder {
            binder,
            child: assume,
        } = &output.proof_nodes.get(proof.root).expect("root").kind
        else {
            panic!("expected introduced binder root");
        };
        let CoreProofNodeKind::Assume {
            label,
            formula,
            child: terminal,
        } = &output.proof_nodes.get(*assume).expect("assume").kind
        else {
            panic!("expected assumption node");
        };
        let CoreProofNodeKind::TerminalGoal {
            obligation: obligation_id,
            citations,
        } = &output.proof_nodes.get(*terminal).expect("terminal").kind
        else {
            panic!("expected terminal goal");
        };
        let obligation = output
            .obligation_seeds
            .get(*obligation_id)
            .expect("terminal obligation");

        assert_eq!(proof.status, CoreProofStatus::Open);
        assert_eq!(proof.proposition, proposition);
        assert_eq!(binder.var, x);
        assert_eq!(binder.ty_guard, Some(proposition));
        assert_eq!(label.as_ref(), Some(&assumption_label));
        assert_eq!(*formula, proposition);
        assert_eq!(obligation.kind, ObligationSeedKind::TheoremProof);
        assert_eq!(obligation.status, ObligationSeedStatus::Active);
        assert_eq!(obligation.goal, Some(proposition));
        assert_eq!(obligation.context, vec![proposition]);
        assert!(obligation.core_refs.contains(&CoreNodeRef::Item(owner)));
        assert!(obligation.core_refs.contains(&CoreNodeRef::Proof(proof_id)));
        assert!(
            obligation
                .core_refs
                .contains(&CoreNodeRef::ProofNode(*terminal))
        );
        assert!(
            obligation
                .core_refs
                .contains(&CoreNodeRef::Formula(proposition))
        );
        assert_eq!(output.terminal_obligations.len(), 1);
        assert_eq!(output.terminal_citations.len(), 1);
        assert_eq!(output.terminal_citations[0].proof, proof_id);
        assert_eq!(output.terminal_citations[0].node, *terminal);
        assert_eq!(output.terminal_citations[0].obligation, *obligation_id);
        assert_eq!(citations, &terminal_citations);
        assert_eq!(output.terminal_citations[0].citations, terminal_citations);
        assert_step5_delta_valid(&context, &term_formula, &definitions, &output);
    }

    #[test]
    fn proof_lowering_tracks_current_goal_sequence_labels_and_active_formulas() {
        let (context, owner) = context_with_var(CoreVarId::new(0));
        let term_formula = lower_test_terms_and_formulas(
            &context,
            owner,
            Vec::new(),
            vec![
                formula_seed(CoreFormulaSeedKind::True, 166),
                formula_seed(CoreFormulaSeedKind::False, 167),
                formula_seed(CoreFormulaSeedKind::True, 168),
            ],
        );
        let definitions = empty_definition_output(&context, &term_formula);
        let proposition = term_formula.formula_map[&CoreFormulaSeedId::new(0)];
        let current_goal = term_formula.formula_map[&CoreFormulaSeedId::new(1)];
        let assumption_formula = term_formula.formula_map[&CoreFormulaSeedId::new(2)];
        let step_label = CoreLabelRef::new("SEQ1");
        let terminal_citations = vec![CoreCitation::Label(step_label.clone())];
        let skeleton = ProofSkeletonSeed::Node(ProofNodeSeed::Sequence {
            children: vec![
                ProofNodeSeed::Step {
                    label: Some(step_label.clone()),
                    formula: ProofFormulaRef::Formula(assumption_formula),
                    justification: ProofJustificationSeed::new(
                        Vec::new(),
                        direct(169, 170),
                        provenance("checker:proof:sequence-step:justification"),
                    ),
                    source: direct(170, 171),
                    provenance: provenance("checker:proof:sequence-step"),
                },
                ProofNodeSeed::CurrentGoal {
                    thesis: ProofFormulaRef::Formula(current_goal),
                    child: Box::new(ProofNodeSeed::Assume {
                        label: None,
                        formula: ProofFormulaRef::Formula(assumption_formula),
                        child: Box::new(ProofNodeSeed::TerminalGoal(
                            ProofTerminalGoalSeed::active(
                                ProofFormulaRef::Thesis,
                                "proof/current-goal/terminal",
                                "pkg::main::Owner.proof.current-goal.terminal",
                                direct(171, 172),
                                provenance("checker:proof:current-goal-terminal"),
                            )
                            .with_citations(terminal_citations.clone()),
                        )),
                        source: direct(172, 173),
                        provenance: provenance("checker:proof:current-goal-assume"),
                    }),
                    source: direct(173, 174),
                    provenance: provenance("checker:proof:current-goal"),
                },
            ],
            source: direct(174, 175),
            provenance: provenance("checker:proof:sequence"),
        });

        let output = lower_proof_inputs(
            &context,
            &term_formula,
            &definitions,
            ProofLoweringInput {
                proofs: vec![proof_seed(
                    owner,
                    symbol("Owner"),
                    proposition,
                    CoreProofStatus::Conditional,
                    skeleton,
                    175,
                )],
            },
        )
        .expect("proof lowering");
        let proof_id = output.proof_map[&owner];
        let proof = output.proofs.get(proof_id).expect("proof");
        let CoreProofNodeKind::Sequence { children } =
            &output.proof_nodes.get(proof.root).expect("sequence").kind
        else {
            panic!("expected sequence root");
        };
        let CoreProofNodeKind::CurrentGoal {
            thesis,
            child: assume,
        } = &output
            .proof_nodes
            .get(children[1])
            .expect("current goal")
            .kind
        else {
            panic!("expected current-goal child");
        };
        let CoreProofNodeKind::Assume {
            child: terminal, ..
        } = output.proof_nodes.get(*assume).expect("assume").kind
        else {
            panic!("expected assumption child");
        };
        let CoreProofNodeKind::TerminalGoal {
            obligation,
            citations,
        } = &output.proof_nodes.get(terminal).expect("terminal").kind
        else {
            panic!("expected terminal goal");
        };
        let obligation = output
            .obligation_seeds
            .get(*obligation)
            .expect("terminal obligation");

        assert_eq!(*thesis, current_goal);
        assert_eq!(obligation.goal, Some(current_goal));
        assert_eq!(obligation.context, vec![assumption_formula]);
        assert!(
            obligation
                .core_refs
                .contains(&CoreNodeRef::Formula(current_goal))
        );
        assert!(
            obligation
                .core_refs
                .contains(&CoreNodeRef::Formula(assumption_formula))
        );
        assert_eq!(citations, &terminal_citations);
        assert_eq!(output.terminal_citations[0].citations, terminal_citations);
        assert_step5_delta_valid(&context, &term_formula, &definitions, &output);
    }

    #[test]
    fn proof_lowering_preserves_branches_steps_citations_and_generated_refs() {
        let x = CoreVarId::new(0);
        let key = GeneratedOriginKey::new("choice:proof");
        let (context, owner, existing_origin) = context_with_existing_choice_origin(x, key.clone());
        let mut input = TermAndFormulaLoweringInput::new(owner);
        input.terms = vec![
            term_seed(CoreTermSeedKind::Var(x), 166),
            term_seed(
                CoreTermSeedKind::StableChoice {
                    functor: symbol("choice_existing"),
                    origin_functor: symbol("choice_existing"),
                    key,
                    params: vec![x],
                    args: vec![CoreTermSeedId::new(0)],
                    evidence: vec![CoreProvenance::new(
                        CoreProvenancePhase::Checker,
                        "checker:proof:choice",
                    )],
                },
                167,
            ),
        ];
        input.formulas = vec![formula_seed(
            CoreFormulaSeedKind::Atom {
                predicate: symbol("ProofPredicate"),
                args: vec![CoreTermSeedId::new(1)],
            },
            168,
        )];
        let term_formula = lower_term_and_formula_inputs(&context, input).expect("lowering");
        let definitions = empty_definition_output(&context, &term_formula);
        let proposition = term_formula.formula_map[&CoreFormulaSeedId::new(0)];
        let label = CoreLabelRef::new("A1");
        let step = ProofNodeSeed::Assume {
            label: Some(label.clone()),
            formula: ProofFormulaRef::Thesis,
            child: Box::new(ProofNodeSeed::Step {
                label: Some(CoreLabelRef::new("S1")),
                formula: ProofFormulaRef::Thesis,
                justification: ProofJustificationSeed::new(
                    vec![
                        CoreCitation::Label(label.clone()),
                        CoreCitation::Generated(existing_origin),
                        CoreCitation::Symbol(symbol("Owner")),
                    ],
                    direct(169, 170),
                    provenance("checker:proof:justification"),
                ),
                source: direct(170, 171),
                provenance: provenance("checker:proof:step"),
            }),
            source: direct(171, 172),
            provenance: provenance("checker:proof:assume:branch"),
        };
        let terminal = ProofNodeSeed::TerminalGoal(
            ProofTerminalGoalSeed::active(
                ProofFormulaRef::Thesis,
                "proof/branch/open",
                "pkg::main::Owner.proof.branch.open",
                direct(172, 173),
                provenance("checker:proof:branch-terminal"),
            )
            .with_citations(vec![
                CoreCitation::Generated(existing_origin),
                CoreCitation::Symbol(symbol("Owner")),
            ]),
        );
        let skeleton = ProofSkeletonSeed::Node(ProofNodeSeed::Branch {
            kind: ProofBranchKind::Cases,
            children: vec![step, terminal],
            source: direct(173, 174),
            provenance: provenance("checker:proof:cases"),
        });

        let output = lower_proof_inputs(
            &context,
            &term_formula,
            &definitions,
            ProofLoweringInput {
                proofs: vec![proof_seed(
                    owner,
                    symbol("Owner"),
                    proposition,
                    CoreProofStatus::Conditional,
                    skeleton,
                    174,
                )],
            },
        )
        .expect("proof lowering");
        let proof = output.proofs.get(output.proof_map[&owner]).expect("proof");
        let CoreProofNodeKind::Branch { kind, children } =
            &output.proof_nodes.get(proof.root).expect("branch").kind
        else {
            panic!("expected branch root");
        };
        let CoreProofNodeKind::Assume { child: step_id, .. } = output
            .proof_nodes
            .get(children[0])
            .expect("branch child")
            .kind
        else {
            panic!("expected assumption child");
        };
        let CoreProofNodeKind::Step { justification, .. } =
            &output.proof_nodes.get(step_id).expect("step").kind
        else {
            panic!("expected step");
        };
        let terminal_record = &output.terminal_obligations[0];
        let obligation = output
            .obligation_seeds
            .get(terminal_record.obligation)
            .expect("terminal obligation");
        let terminal_citation_record = &output.terminal_citations[0];

        assert_eq!(proof.status, CoreProofStatus::Conditional);
        assert_eq!(kind, &ProofBranchKind::Cases);
        assert_eq!(children.len(), 2);
        assert_eq!(
            justification.citations,
            vec![
                CoreCitation::Label(label),
                CoreCitation::Generated(existing_origin),
                CoreCitation::Symbol(symbol("Owner"))
            ]
        );
        assert!(term_formula.generated_delta.is_empty());
        assert!(term_formula.generated_origin_refs[0].reused_existing);
        assert!(
            obligation
                .core_refs
                .contains(&CoreNodeRef::Generated(existing_origin))
        );
        assert!(obligation.core_refs.contains(&CoreNodeRef::Item(owner)));
        assert_eq!(terminal_citation_record.proof, output.proof_map[&owner]);
        assert_eq!(terminal_citation_record.node, terminal_record.node);
        assert_eq!(
            terminal_citation_record.obligation,
            terminal_record.obligation
        );
        assert_eq!(
            terminal_citation_record.citations,
            vec![
                CoreCitation::Generated(existing_origin),
                CoreCitation::Symbol(symbol("Owner"))
            ]
        );
        assert_step5_delta_valid(&context, &term_formula, &definitions, &output);
    }

    #[test]
    fn proof_lowering_lowers_lemma_stable_choice_terminal_citations() {
        let x = CoreVarId::new(0);
        let key = GeneratedOriginKey::new("choice:lemma-proof");
        let mut context_input = input_with_items(vec![
            CoreItemSeed::new(
                symbol("LemmaOwner"),
                CoreItemKind::Lemma,
                "public",
                direct(175, 176),
                provenance("checker:item:lemma-owner"),
            )
            .with_definition_boundary(DefinitionBoundaryKind::Lemma),
        ]);
        context_input.variable_seeds = vec![CoreVariableSeed::new(
            x,
            NormalizedVarClass::Free,
            "term-binder",
            NormalizedVarSort::Term,
            provenance("checker:lemma-choice-var"),
        )];
        context_input.binder_seeds = vec![CoreBinderSeed::new(
            x,
            direct(176, 177),
            provenance("checker:lemma-choice-binder"),
        )];
        context_input.generated_origin_seeds = vec![
            GeneratedOriginSeed::new(
                symbol("LemmaOwner"),
                GeneratedOriginKind::StableChoice,
                key.clone(),
                direct(177, 178),
                provenance("checker:lemma-existing-choice"),
            )
            .with_functor(symbol("lemma_choice"))
            .with_params(vec![x])
            .with_evidence(vec![CoreProvenance::new(
                CoreProvenancePhase::Checker,
                "checker:lemma-existing-choice:evidence",
            )]),
        ];
        let context = prepare_core_context(context_input).expect("context");
        let owner = context
            .item_registry()
            .id_for_symbol(&symbol("LemmaOwner"))
            .expect("lemma owner");
        let existing_origin = context
            .generated_origins()
            .get_by_key(owner, GeneratedOriginKind::StableChoice, &key)
            .expect("existing generated origin");
        let mut input = TermAndFormulaLoweringInput::new(owner);
        input.terms = vec![
            term_seed(CoreTermSeedKind::Var(x), 178),
            term_seed(
                CoreTermSeedKind::StableChoice {
                    functor: symbol("lemma_choice"),
                    origin_functor: symbol("lemma_choice"),
                    key,
                    params: vec![x],
                    args: vec![CoreTermSeedId::new(0)],
                    evidence: vec![CoreProvenance::new(
                        CoreProvenancePhase::Checker,
                        "checker:lemma-proof:choice",
                    )],
                },
                179,
            ),
        ];
        input.formulas = vec![formula_seed(
            CoreFormulaSeedKind::Atom {
                predicate: symbol("LemmaProofPredicate"),
                args: vec![CoreTermSeedId::new(1)],
            },
            180,
        )];
        let term_formula = lower_term_and_formula_inputs(&context, input).expect("lowering");
        let definitions = empty_definition_output(&context, &term_formula);
        let proposition = term_formula.formula_map[&CoreFormulaSeedId::new(0)];
        let skeleton = ProofSkeletonSeed::Node(ProofNodeSeed::TerminalGoal(
            ProofTerminalGoalSeed::active(
                ProofFormulaRef::Thesis,
                "proof/lemma/stable-choice",
                "pkg::main::LemmaOwner.proof.stable-choice",
                direct(181, 182),
                provenance("checker:proof:lemma-terminal"),
            )
            .with_citations(vec![CoreCitation::Generated(existing_origin)]),
        ));

        let output = lower_proof_inputs(
            &context,
            &term_formula,
            &definitions,
            ProofLoweringInput {
                proofs: vec![proof_seed(
                    owner,
                    symbol("LemmaOwner"),
                    proposition,
                    CoreProofStatus::Conditional,
                    skeleton,
                    182,
                )],
            },
        )
        .expect("lemma proof lowering");
        let terminal_record = &output.terminal_obligations[0];
        let obligation = output
            .obligation_seeds
            .get(terminal_record.obligation)
            .expect("terminal obligation");

        assert!(term_formula.generated_delta.is_empty());
        assert!(term_formula.generated_origin_refs[0].reused_existing);
        assert!(
            obligation
                .core_refs
                .contains(&CoreNodeRef::Generated(existing_origin))
        );
        assert_eq!(output.terminal_citations.len(), 1);
        assert_eq!(
            output.terminal_citations[0].citations,
            vec![CoreCitation::Generated(existing_origin)]
        );
        assert_step5_delta_valid(&context, &term_formula, &definitions, &output);
    }

    #[test]
    fn proof_lowering_reports_malformed_and_status_boundaries() {
        let (context, owner) = context_with_var(CoreVarId::new(0));
        let term_formula = lower_test_terms_and_formulas(
            &context,
            owner,
            Vec::new(),
            vec![formula_seed(CoreFormulaSeedKind::True, 175)],
        );
        let definitions = empty_definition_output(&context, &term_formula);
        let proposition = term_formula.formula_map[&CoreFormulaSeedId::new(0)];
        let output = lower_proof_inputs(
            &context,
            &term_formula,
            &definitions,
            ProofLoweringInput {
                proofs: vec![proof_seed(
                    owner,
                    symbol("Owner"),
                    proposition,
                    CoreProofStatus::Error,
                    ProofSkeletonSeed::Missing(malformed_proof("missing-proof-skeleton", 176)),
                    177,
                )],
            },
        )
        .expect("proof lowering");
        let proof = output.proofs.get(output.proof_map[&owner]).expect("proof");
        let CoreProofNodeKind::Error(diagnostic_id) =
            output.proof_nodes.get(proof.root).expect("error root").kind
        else {
            panic!("expected error root");
        };
        let diagnostic = output.diagnostics.get(diagnostic_id).expect("diagnostic");

        assert_eq!(proof.status, CoreProofStatus::Error);
        assert_eq!(
            diagnostic.class,
            CoreDiagnosticClass::MalformedProofSkeleton
        );
        assert!(output.terminal_obligations.is_empty());
        assert_step5_delta_valid(&context, &term_formula, &definitions, &output);

        let explicit_error = lower_proof_inputs(
            &context,
            &term_formula,
            &definitions,
            ProofLoweringInput {
                proofs: vec![proof_seed(
                    owner,
                    symbol("Owner"),
                    proposition,
                    CoreProofStatus::Error,
                    ProofSkeletonSeed::Node(ProofNodeSeed::Error(malformed_proof(
                        "explicit-error-proof",
                        178,
                    ))),
                    179,
                )],
            },
        )
        .expect("explicit error proof lowering");
        let proof = explicit_error
            .proofs
            .get(explicit_error.proof_map[&owner])
            .expect("proof");
        let CoreProofNodeKind::Error(diagnostic_id) = explicit_error
            .proof_nodes
            .get(proof.root)
            .expect("error")
            .kind
        else {
            panic!("expected explicit error root");
        };
        let diagnostic = explicit_error
            .diagnostics
            .get(diagnostic_id)
            .expect("diagnostic");

        assert_eq!(
            diagnostic.class,
            CoreDiagnosticClass::MalformedProofSkeleton
        );
        assert!(explicit_error.terminal_obligations.is_empty());
        assert_step5_delta_valid(&context, &term_formula, &definitions, &explicit_error);

        assert!(matches!(
            lower_proof_inputs(
                &context,
                &term_formula,
                &definitions,
                ProofLoweringInput {
                    proofs: vec![proof_seed(
                        owner,
                        symbol("Owner"),
                        proposition,
                        CoreProofStatus::Open,
                        ProofSkeletonSeed::Missing(malformed_proof("missing-open-proof", 180)),
                        181,
                    )],
                },
            ),
            Err(ProofLoweringError::MalformedSkeletonRequiresErrorStatus {
                status: CoreProofStatus::Open
            })
        ));

        assert!(matches!(
            lower_proof_inputs(
                &context,
                &term_formula,
                &definitions,
                ProofLoweringInput {
                    proofs: vec![proof_seed(
                        owner,
                        symbol("Owner"),
                        proposition,
                        CoreProofStatus::Open,
                        ProofSkeletonSeed::Node(ProofNodeSeed::Error(malformed_proof(
                            "open-error-root",
                            182,
                        ))),
                        183,
                    )],
                },
            ),
            Err(ProofLoweringError::MalformedSkeletonRequiresErrorStatus {
                status: CoreProofStatus::Open
            })
        ));

        assert!(matches!(
            lower_proof_inputs(
                &context,
                &term_formula,
                &definitions,
                ProofLoweringInput {
                    proofs: vec![proof_seed(
                        owner,
                        symbol("Owner"),
                        proposition,
                        CoreProofStatus::Error,
                        ProofSkeletonSeed::Node(ProofNodeSeed::Step {
                            label: None,
                            formula: ProofFormulaRef::Thesis,
                            justification: ProofJustificationSeed::new(
                                Vec::new(),
                                direct(184, 185),
                                provenance("checker:proof:error-status-justification"),
                            ),
                            source: direct(185, 186),
                            provenance: provenance("checker:proof:error-status-step"),
                        }),
                        186,
                    )],
                },
            ),
            Err(ProofLoweringError::ErrorStatusRequiresMalformedSkeleton)
        ));
    }

    #[test]
    fn proof_lowering_validates_introduced_binders() {
        let x = CoreVarId::new(0);
        let y = CoreVarId::new(1);
        let (context, owner) = context_with_var(x);
        let term_formula = lower_test_terms_and_formulas(
            &context,
            owner,
            Vec::new(),
            vec![formula_seed(CoreFormulaSeedKind::True, 183)],
        );
        let definitions = empty_definition_output(&context, &term_formula);
        let proposition = term_formula.formula_map[&CoreFormulaSeedId::new(0)];
        let undeclared = ProofSkeletonSeed::Node(ProofNodeSeed::IntroduceBinder {
            binder: test_binder(y, None, 184),
            child: Box::new(proof_step_node(185)),
            source: direct(186, 187),
            provenance: provenance("checker:proof:undeclared-binder"),
        });

        assert!(matches!(
            lower_proof_inputs(
                &context,
                &term_formula,
                &definitions,
                ProofLoweringInput {
                    proofs: vec![proof_seed(
                        owner,
                        symbol("Owner"),
                        proposition,
                        CoreProofStatus::Open,
                        undeclared,
                        187,
                    )],
                },
            ),
            Err(ProofLoweringError::UndeclaredIntroducedBinder { var }) if var == y
        ));

        let (context, owner) = context_with_var_sort(x, NormalizedVarSort::Formula);
        let term_formula = lower_test_terms_and_formulas(
            &context,
            owner,
            Vec::new(),
            vec![formula_seed(CoreFormulaSeedKind::True, 188)],
        );
        let definitions = empty_definition_output(&context, &term_formula);
        let proposition = term_formula.formula_map[&CoreFormulaSeedId::new(0)];
        let nonterm = ProofSkeletonSeed::Node(ProofNodeSeed::IntroduceBinder {
            binder: test_binder(x, None, 189),
            child: Box::new(proof_step_node(190)),
            source: direct(191, 192),
            provenance: provenance("checker:proof:nonterm-binder"),
        });

        assert!(matches!(
            lower_proof_inputs(
                &context,
                &term_formula,
                &definitions,
                ProofLoweringInput {
                    proofs: vec![proof_seed(
                        owner,
                        symbol("Owner"),
                        proposition,
                        CoreProofStatus::Open,
                        nonterm,
                        192,
                    )],
                },
            ),
            Err(ProofLoweringError::NonTermIntroducedBinder { var, sort })
                if var == x && sort == NormalizedVarSort::Formula
        ));
    }

    #[test]
    fn proof_lowering_accepts_assumed_steps_and_now_suppose_branches() {
        let (context, owner) = context_with_var(CoreVarId::new(0));
        let term_formula = lower_test_terms_and_formulas(
            &context,
            owner,
            Vec::new(),
            vec![formula_seed(CoreFormulaSeedKind::True, 193)],
        );
        let definitions = empty_definition_output(&context, &term_formula);
        let proposition = term_formula.formula_map[&CoreFormulaSeedId::new(0)];
        let assumed = lower_proof_inputs(
            &context,
            &term_formula,
            &definitions,
            ProofLoweringInput {
                proofs: vec![proof_seed(
                    owner,
                    symbol("Owner"),
                    proposition,
                    CoreProofStatus::Assumed,
                    proof_step_skeleton(194),
                    195,
                )],
            },
        )
        .expect("assumed proof without terminal goals is accepted");
        let proof = assumed
            .proofs
            .get(assumed.proof_map[&owner])
            .expect("assumed proof");

        assert_eq!(proof.status, CoreProofStatus::Assumed);
        assert!(assumed.terminal_obligations.is_empty());
        assert!(assumed.terminal_citations.is_empty());
        assert_step5_delta_valid(&context, &term_formula, &definitions, &assumed);

        for (kind, start) in [(ProofBranchKind::Now, 196), (ProofBranchKind::Suppose, 200)] {
            let skeleton = ProofSkeletonSeed::Node(ProofNodeSeed::Branch {
                kind: kind.clone(),
                children: vec![proof_step_node(start)],
                source: direct(start + 2, start + 3),
                provenance: provenance(format!("checker:proof:branch:{start}").as_str()),
            });
            let output = lower_proof_inputs(
                &context,
                &term_formula,
                &definitions,
                ProofLoweringInput {
                    proofs: vec![proof_seed(
                        owner,
                        symbol("Owner"),
                        proposition,
                        CoreProofStatus::Open,
                        skeleton,
                        start + 3,
                    )],
                },
            )
            .expect("branch proof lowering");
            let proof = output.proofs.get(output.proof_map[&owner]).expect("proof");
            let CoreProofNodeKind::Branch {
                kind: actual,
                children,
            } = &output.proof_nodes.get(proof.root).expect("branch").kind
            else {
                panic!("expected branch root");
            };

            assert_eq!(actual, &kind);
            assert_eq!(children.len(), 1);
            assert_step5_delta_valid(&context, &term_formula, &definitions, &output);
        }
    }

    #[test]
    fn proof_lowering_validates_labels_citations_and_owners() {
        let (context, owner) = context_with_var(CoreVarId::new(0));
        let term_formula = lower_test_terms_and_formulas(
            &context,
            owner,
            Vec::new(),
            vec![formula_seed(CoreFormulaSeedKind::True, 183)],
        );
        let definitions = empty_definition_output(&context, &term_formula);
        let proposition = term_formula.formula_map[&CoreFormulaSeedId::new(0)];
        let duplicate = ProofSkeletonSeed::Node(ProofNodeSeed::Branch {
            kind: ProofBranchKind::Now,
            children: vec![
                ProofNodeSeed::Step {
                    label: Some(CoreLabelRef::new("DUP")),
                    formula: ProofFormulaRef::Thesis,
                    justification: ProofJustificationSeed::new(
                        Vec::new(),
                        direct(184, 185),
                        provenance("checker:proof:dup:left:justification"),
                    ),
                    source: direct(185, 186),
                    provenance: provenance("checker:proof:dup:left"),
                },
                ProofNodeSeed::Step {
                    label: Some(CoreLabelRef::new("DUP")),
                    formula: ProofFormulaRef::Thesis,
                    justification: ProofJustificationSeed::new(
                        Vec::new(),
                        direct(186, 187),
                        provenance("checker:proof:dup:right:justification"),
                    ),
                    source: direct(187, 188),
                    provenance: provenance("checker:proof:dup:right"),
                },
            ],
            source: direct(188, 189),
            provenance: provenance("checker:proof:dup:branch"),
        });
        assert!(matches!(
            lower_proof_inputs(
                &context,
                &term_formula,
                &definitions,
                ProofLoweringInput {
                    proofs: vec![proof_seed(
                        owner,
                        symbol("Owner"),
                        proposition,
                        CoreProofStatus::Open,
                        duplicate,
                        189,
                    )],
                },
            ),
            Err(ProofLoweringError::DuplicateProofLabel { label })
                if label.as_str() == "DUP"
        ));

        let sibling_label = CoreLabelRef::new("SIB");
        let sibling_citation = ProofSkeletonSeed::Node(ProofNodeSeed::Branch {
            kind: ProofBranchKind::Cases,
            children: vec![
                ProofNodeSeed::Step {
                    label: Some(sibling_label.clone()),
                    formula: ProofFormulaRef::Thesis,
                    justification: ProofJustificationSeed::new(
                        Vec::new(),
                        direct(190, 191),
                        provenance("checker:proof:sibling:left:justification"),
                    ),
                    source: direct(191, 192),
                    provenance: provenance("checker:proof:sibling:left"),
                },
                ProofNodeSeed::Step {
                    label: None,
                    formula: ProofFormulaRef::Thesis,
                    justification: ProofJustificationSeed::new(
                        vec![CoreCitation::Label(sibling_label.clone())],
                        direct(192, 193),
                        provenance("checker:proof:sibling:right:justification"),
                    ),
                    source: direct(193, 194),
                    provenance: provenance("checker:proof:sibling:right"),
                },
            ],
            source: direct(194, 195),
            provenance: provenance("checker:proof:sibling:branch"),
        });
        assert!(matches!(
            lower_proof_inputs(
                &context,
                &term_formula,
                &definitions,
                ProofLoweringInput {
                    proofs: vec![proof_seed(
                        owner,
                        symbol("Owner"),
                        proposition,
                        CoreProofStatus::Open,
                        sibling_citation,
                        195,
                    )],
                },
            ),
            Err(ProofLoweringError::UnknownProofLabel { label })
                if label == sibling_label
        ));

        let bad_symbol = ProofSkeletonSeed::Node(ProofNodeSeed::Step {
            label: None,
            formula: ProofFormulaRef::Thesis,
            justification: ProofJustificationSeed::new(
                vec![CoreCitation::Symbol(symbol("MissingTheorem"))],
                direct(196, 197),
                provenance("checker:proof:bad-symbol:justification"),
            ),
            source: direct(197, 198),
            provenance: provenance("checker:proof:bad-symbol"),
        });
        assert!(matches!(
            lower_proof_inputs(
                &context,
                &term_formula,
                &definitions,
                ProofLoweringInput {
                    proofs: vec![proof_seed(
                        owner,
                        symbol("Owner"),
                        proposition,
                        CoreProofStatus::Open,
                        bad_symbol,
                        198,
                    )],
                },
            ),
            Err(ProofLoweringError::InvalidSymbolCitation { symbol: cited })
                if cited.as_ref() == &symbol("MissingTheorem")
        ));

        let empty_label = ProofSkeletonSeed::Node(ProofNodeSeed::Step {
            label: Some(CoreLabelRef::new("")),
            formula: ProofFormulaRef::Thesis,
            justification: ProofJustificationSeed::new(
                Vec::new(),
                direct(199, 200),
                provenance("checker:proof:empty-label:justification"),
            ),
            source: direct(200, 201),
            provenance: provenance("checker:proof:empty-label"),
        });
        assert!(matches!(
            lower_proof_inputs(
                &context,
                &term_formula,
                &definitions,
                ProofLoweringInput {
                    proofs: vec![proof_seed(
                        owner,
                        symbol("Owner"),
                        proposition,
                        CoreProofStatus::Open,
                        empty_label,
                        201,
                    )],
                },
            ),
            Err(ProofLoweringError::InvalidProofLabel { label })
                if label.as_str().is_empty()
        ));

        let forward_label = CoreLabelRef::new("FWD");
        let forward_citation = ProofSkeletonSeed::Node(ProofNodeSeed::Step {
            label: Some(forward_label.clone()),
            formula: ProofFormulaRef::Thesis,
            justification: ProofJustificationSeed::new(
                vec![CoreCitation::Label(forward_label.clone())],
                direct(202, 203),
                provenance("checker:proof:forward-label:justification"),
            ),
            source: direct(203, 204),
            provenance: provenance("checker:proof:forward-label"),
        });
        assert!(matches!(
            lower_proof_inputs(
                &context,
                &term_formula,
                &definitions,
                ProofLoweringInput {
                    proofs: vec![proof_seed(
                        owner,
                        symbol("Owner"),
                        proposition,
                        CoreProofStatus::Open,
                        forward_citation,
                        204,
                    )],
                },
            ),
            Err(ProofLoweringError::UnknownProofLabel { label })
                if label == forward_label
        ));

        let missing_generated = GeneratedOriginId::new(99);
        let bad_generated = ProofSkeletonSeed::Node(ProofNodeSeed::Step {
            label: None,
            formula: ProofFormulaRef::Thesis,
            justification: ProofJustificationSeed::new(
                vec![CoreCitation::Generated(missing_generated)],
                direct(205, 206),
                provenance("checker:proof:bad-generated:justification"),
            ),
            source: direct(206, 207),
            provenance: provenance("checker:proof:bad-generated"),
        });
        assert!(matches!(
            lower_proof_inputs(
                &context,
                &term_formula,
                &definitions,
                ProofLoweringInput {
                    proofs: vec![proof_seed(
                        owner,
                        symbol("Owner"),
                        proposition,
                        CoreProofStatus::Open,
                        bad_generated,
                        207,
                    )],
                },
            ),
            Err(ProofLoweringError::MissingGeneratedCitation { origin })
                if origin == missing_generated
        ));
    }

    #[test]
    fn proof_lowering_rejects_wrong_owner_kind_and_assumed_terminal_goals() {
        let mut context_input = input_with_items(vec![
            item_seed("Owner", 199),
            CoreItemSeed::new(
                symbol("FunctorOwner"),
                CoreItemKind::Functor,
                "public",
                direct(200, 201),
                provenance("checker:item:functor-owner"),
            )
            .with_definition_boundary(DefinitionBoundaryKind::DefinitionalItem),
        ]);
        context_input.variable_seeds = Vec::new();
        context_input.dependency_summaries = vec![
            CoreDependencySummary::new(
                external_symbol("ExternalTheorem"),
                CoreItemKind::Theorem,
                "public",
                provenance("checker:dependency:external-theorem"),
            ),
            CoreDependencySummary::new(
                external_symbol("ExternalFunctor"),
                CoreItemKind::Functor,
                "public",
                provenance("checker:dependency:external-functor"),
            ),
        ];
        let context = prepare_core_context(context_input).expect("context");
        let owner = context
            .item_registry()
            .id_for_symbol(&symbol("Owner"))
            .expect("owner");
        let functor_owner = context
            .item_registry()
            .id_for_symbol(&symbol("FunctorOwner"))
            .expect("functor owner");
        let term_formula = lower_test_terms_and_formulas(
            &context,
            owner,
            Vec::new(),
            vec![formula_seed(CoreFormulaSeedKind::True, 201)],
        );
        let definitions = empty_definition_output(&context, &term_formula);
        let proposition = term_formula.formula_map[&CoreFormulaSeedId::new(0)];
        let terminal =
            ProofSkeletonSeed::Node(ProofNodeSeed::TerminalGoal(ProofTerminalGoalSeed::active(
                ProofFormulaRef::Thesis,
                "proof/assumed-terminal",
                "pkg::main::Owner.proof.assumed-terminal",
                direct(202, 203),
                provenance("checker:proof:assumed-terminal"),
            )));

        assert!(matches!(
            lower_proof_inputs(
                &context,
                &term_formula,
                &definitions,
                ProofLoweringInput {
                    proofs: vec![proof_seed(
                        functor_owner,
                        symbol("FunctorOwner"),
                        proposition,
                        CoreProofStatus::Open,
                        ProofSkeletonSeed::Node(ProofNodeSeed::Step {
                            label: None,
                            formula: ProofFormulaRef::Thesis,
                            justification: ProofJustificationSeed::new(
                                Vec::new(),
                                direct(203, 204),
                                provenance("checker:proof:functor:justification"),
                            ),
                            source: direct(204, 205),
                            provenance: provenance("checker:proof:functor"),
                        }),
                        205,
                    )],
                },
            ),
            Err(ProofLoweringError::UnsupportedProofItemKind { owner: actual, .. })
                if actual == functor_owner
        ));

        assert!(matches!(
            lower_proof_inputs(
                &context,
                &term_formula,
                &definitions,
                ProofLoweringInput {
                    proofs: vec![proof_seed(
                        CoreItemId::new(99),
                        symbol("MissingOwner"),
                        proposition,
                        CoreProofStatus::Open,
                        proof_step_skeleton(208),
                        209,
                    )],
                },
            ),
            Err(ProofLoweringError::MissingOwnerItem { owner })
                if owner == CoreItemId::new(99)
        ));

        assert!(matches!(
            lower_proof_inputs(
                &context,
                &term_formula,
                &definitions,
                ProofLoweringInput {
                    proofs: vec![proof_seed(
                        owner,
                        symbol("WrongOwnerSymbol"),
                        proposition,
                        CoreProofStatus::Open,
                        proof_step_skeleton(210),
                        211,
                    )],
                },
            ),
            Err(ProofLoweringError::ProofSymbolMismatch { owner: actual, .. })
                if actual == owner
        ));

        assert!(matches!(
            lower_proof_inputs(
                &context,
                &term_formula,
                &definitions,
                ProofLoweringInput {
                    proofs: vec![
                        proof_seed(
                            owner,
                            symbol("Owner"),
                            proposition,
                            CoreProofStatus::Open,
                            proof_step_skeleton(212),
                            213,
                        ),
                        proof_seed(
                            owner,
                            symbol("Owner"),
                            proposition,
                            CoreProofStatus::Open,
                            proof_step_skeleton(214),
                            215,
                        ),
                    ],
                },
            ),
            Err(ProofLoweringError::DuplicateProofOwner { owner: actual })
                if actual == owner
        ));

        let external_theorem = ProofSkeletonSeed::Node(ProofNodeSeed::Step {
            label: None,
            formula: ProofFormulaRef::Thesis,
            justification: ProofJustificationSeed::new(
                vec![CoreCitation::Symbol(external_symbol("ExternalTheorem"))],
                direct(216, 217),
                provenance("checker:proof:external-theorem:justification"),
            ),
            source: direct(217, 218),
            provenance: provenance("checker:proof:external-theorem"),
        });
        lower_proof_inputs(
            &context,
            &term_formula,
            &definitions,
            ProofLoweringInput {
                proofs: vec![proof_seed(
                    owner,
                    symbol("Owner"),
                    proposition,
                    CoreProofStatus::Assumed,
                    external_theorem,
                    218,
                )],
            },
        )
        .expect("external theorem citations from dependency summaries are accepted");

        let external_terminal = ProofSkeletonSeed::Node(ProofNodeSeed::TerminalGoal(
            ProofTerminalGoalSeed::active(
                ProofFormulaRef::Thesis,
                "proof/external-terminal",
                "pkg::main::Owner.proof.external-terminal",
                direct(219, 220),
                provenance("checker:proof:external-terminal"),
            )
            .with_citations(vec![CoreCitation::Symbol(external_symbol(
                "ExternalTheorem",
            ))]),
        ));
        let output = lower_proof_inputs(
            &context,
            &term_formula,
            &definitions,
            ProofLoweringInput {
                proofs: vec![proof_seed(
                    owner,
                    symbol("Owner"),
                    proposition,
                    CoreProofStatus::Conditional,
                    external_terminal,
                    220,
                )],
            },
        )
        .expect("external theorem terminal citations are accepted");
        let terminal_record = &output.terminal_obligations[0];
        let obligation = output
            .obligation_seeds
            .get(terminal_record.obligation)
            .expect("terminal obligation");
        let terminal_node = output
            .proof_nodes
            .get(terminal_record.node)
            .expect("terminal node");
        let external_citation = CoreCitation::Symbol(external_symbol("ExternalTheorem"));
        let CoreProofNodeKind::TerminalGoal { citations, .. } = &terminal_node.kind else {
            panic!("expected terminal goal");
        };

        assert_eq!(citations, &vec![external_citation.clone()]);
        assert_eq!(
            output.terminal_citations[0].citations,
            vec![external_citation]
        );
        assert_eq!(
            obligation
                .core_refs
                .iter()
                .filter(|reference| matches!(reference, CoreNodeRef::Item(_)))
                .count(),
            1
        );
        assert!(obligation.core_refs.contains(&CoreNodeRef::Item(owner)));

        let local_functor_citation = ProofSkeletonSeed::Node(ProofNodeSeed::Step {
            label: None,
            formula: ProofFormulaRef::Thesis,
            justification: ProofJustificationSeed::new(
                vec![CoreCitation::Symbol(symbol("FunctorOwner"))],
                direct(221, 222),
                provenance("checker:proof:local-functor-citation:justification"),
            ),
            source: direct(222, 223),
            provenance: provenance("checker:proof:local-functor-citation"),
        });
        assert!(matches!(
            lower_proof_inputs(
                &context,
                &term_formula,
                &definitions,
                ProofLoweringInput {
                    proofs: vec![proof_seed(
                        owner,
                        symbol("Owner"),
                        proposition,
                        CoreProofStatus::Open,
                        local_functor_citation,
                        223,
                    )],
                },
            ),
            Err(ProofLoweringError::InvalidSymbolCitation { symbol: cited })
                if cited.as_ref() == &symbol("FunctorOwner")
        ));

        let external_functor_citation = ProofSkeletonSeed::Node(ProofNodeSeed::Step {
            label: None,
            formula: ProofFormulaRef::Thesis,
            justification: ProofJustificationSeed::new(
                vec![CoreCitation::Symbol(external_symbol("ExternalFunctor"))],
                direct(224, 225),
                provenance("checker:proof:external-functor-citation:justification"),
            ),
            source: direct(225, 226),
            provenance: provenance("checker:proof:external-functor-citation"),
        });
        assert!(matches!(
            lower_proof_inputs(
                &context,
                &term_formula,
                &definitions,
                ProofLoweringInput {
                    proofs: vec![proof_seed(
                        owner,
                        symbol("Owner"),
                        proposition,
                        CoreProofStatus::Open,
                        external_functor_citation,
                        226,
                    )],
                },
            ),
            Err(ProofLoweringError::InvalidSymbolCitation { symbol: cited })
                if cited.as_ref() == &external_symbol("ExternalFunctor")
        ));

        assert!(matches!(
            lower_proof_inputs(
                &context,
                &term_formula,
                &definitions,
                ProofLoweringInput {
                    proofs: vec![proof_seed(
                        owner,
                        symbol("Owner"),
                        proposition,
                        CoreProofStatus::Assumed,
                        terminal,
                        206,
                    )],
                },
            ),
            Err(ProofLoweringError::AssumedProofCannotHaveTerminalGoals { owner: actual })
                if actual == owner
        ));
    }

    #[test]
    fn algorithm_lowering_preserves_shells_contracts_pick_and_nested_order() {
        let x = CoreVarId::new(0);
        let y = CoreVarId::new(1);
        let z = CoreVarId::new(2);
        let result = CoreVarId::new(3);
        let ghost_pick = CoreVarId::new(4);
        let (context, owner) = context_with_algorithm_var_sorts(vec![
            (x, NormalizedVarSort::Term),
            (y, NormalizedVarSort::Term),
            (z, NormalizedVarSort::Term),
            (result, NormalizedVarSort::Term),
            (ghost_pick, NormalizedVarSort::Term),
        ]);
        let term_formula = lower_test_terms_and_formulas(
            &context,
            owner,
            vec![
                term_seed(CoreTermSeedKind::Var(x), 230),
                term_seed(CoreTermSeedKind::Var(y), 231),
            ],
            vec![
                formula_seed(CoreFormulaSeedKind::True, 232),
                formula_seed(CoreFormulaSeedKind::False, 233),
                formula_seed(
                    CoreFormulaSeedKind::TypePred {
                        subject: CoreTermSeedId::new(0),
                        ty: CoreTypePredicate::new("set"),
                    },
                    234,
                ),
            ],
        );
        let definitions = empty_definition_output(&context, &term_formula);
        let proofs = empty_proof_output(&context, &term_formula, &definitions);
        let term_x = term_formula.term_map[&CoreTermSeedId::new(0)];
        let term_y = term_formula.term_map[&CoreTermSeedId::new(1)];
        let requires = term_formula.formula_map[&CoreFormulaSeedId::new(0)];
        let ensures = term_formula.formula_map[&CoreFormulaSeedId::new(1)];
        let invariant = term_formula.formula_map[&CoreFormulaSeedId::new(2)];
        let param = test_binder(x, Some(requires), 235);
        let result_binder = test_binder(result, Some(ensures), 236);
        let runtime_pick = test_binder(z, Some(invariant), 237);
        let ghost_pick_binder = test_binder(ghost_pick, Some(requires), 238);
        let mut seed = algorithm_seed(
            owner,
            symbol("Owner"),
            AlgorithmPayloadSeed::Statements(vec![
                AlgorithmStmtSeed::Let {
                    binder: test_binder(y, Some(invariant), 239),
                    value: Some(term_x),
                    ghost: false,
                    source: direct(240, 241),
                    provenance: provenance("checker:algorithm:let"),
                },
                AlgorithmStmtSeed::Pick {
                    binder: runtime_pick.clone(),
                    witness_ty: Some(invariant),
                    ghost: false,
                    source: direct(241, 242),
                    provenance: provenance("checker:algorithm:pick:runtime"),
                },
                AlgorithmStmtSeed::Pick {
                    binder: ghost_pick_binder.clone(),
                    witness_ty: Some(requires),
                    ghost: true,
                    source: direct(242, 243),
                    provenance: provenance("checker:algorithm:pick:ghost"),
                },
                AlgorithmStmtSeed::If {
                    condition: requires,
                    then_body: vec![AlgorithmStmtSeed::Assert {
                        formula: ensures,
                        source: direct(243, 244),
                        provenance: provenance("checker:algorithm:if:assert"),
                    }],
                    else_body: vec![AlgorithmStmtSeed::Break {
                        source: direct(244, 245),
                        provenance: provenance("checker:algorithm:if:break"),
                    }],
                    source: direct(245, 246),
                    provenance: provenance("checker:algorithm:if"),
                },
                AlgorithmStmtSeed::While {
                    condition: ensures,
                    invariants: vec![invariant],
                    decreasing: vec![term_x],
                    body: vec![AlgorithmStmtSeed::Continue {
                        source: direct(246, 247),
                        provenance: provenance("checker:algorithm:while:continue"),
                    }],
                    source: direct(247, 248),
                    provenance: provenance("checker:algorithm:while"),
                },
                AlgorithmStmtSeed::Match {
                    scrutinee: term_x,
                    arms: vec![AlgorithmMatchArmSeed {
                        pattern: CoreProvenanceKey::new("case:some"),
                        body: vec![AlgorithmStmtSeed::Return {
                            value: Some(term_y),
                            source: direct(248, 249),
                            provenance: provenance("checker:algorithm:match:return"),
                        }],
                        provenance: provenance("checker:algorithm:match:arm"),
                    }],
                    source: direct(249, 250),
                    provenance: provenance("checker:algorithm:match"),
                },
                AlgorithmStmtSeed::Assign {
                    target: CorePlace::new("result"),
                    value: term_y,
                    source: direct(250, 251),
                    provenance: provenance("checker:algorithm:assign"),
                },
                AlgorithmStmtSeed::Return {
                    value: None,
                    source: direct(251, 252),
                    provenance: provenance("checker:algorithm:return"),
                },
            ]),
            252,
        );
        seed.params = vec![param.clone()];
        seed.result = Some(result_binder.clone());
        seed.contracts = CoreContractSet {
            requires: vec![requires],
            ensures: vec![ensures],
            invariants: vec![invariant],
            assertions: vec![ensures],
            decreasing: vec![term_x],
        };
        seed.ghost_effects = vec![
            GhostEffectKey::new("runtime-state"),
            GhostEffectKey::new("ghost-proof"),
        ];

        let output = lower_algorithm_inputs(
            &context,
            &term_formula,
            &proofs,
            AlgorithmLoweringInput {
                algorithms: vec![seed],
            },
        )
        .expect("algorithm lowering");
        let algorithm_id = output.algorithm_map[&owner];
        let algorithm = output.algorithms.get(algorithm_id).expect("algorithm");

        assert_eq!(algorithm_id, CoreAlgorithmId::new(0));
        assert_eq!(algorithm.item, owner);
        assert_eq!(algorithm.symbol, symbol("Owner"));
        assert_eq!(
            algorithm.source,
            expected_checker_source(252, 253, "checker:algorithm:252")
        );
        assert_eq!(algorithm.params, vec![param]);
        assert_eq!(algorithm.result.as_ref(), Some(&result_binder));
        assert_eq!(algorithm.contracts.requires, vec![requires]);
        assert_eq!(algorithm.contracts.ensures, vec![ensures]);
        assert_eq!(algorithm.contracts.invariants, vec![invariant]);
        assert_eq!(algorithm.contracts.assertions, vec![ensures]);
        assert_eq!(algorithm.contracts.decreasing, vec![term_x]);
        assert_eq!(
            algorithm.ghost_effects,
            vec![
                GhostEffectKey::new("runtime-state"),
                GhostEffectKey::new("ghost-proof")
            ]
        );
        assert_eq!(algorithm.statements.len(), 8);
        assert!(algorithm.diagnostics.is_empty());
        assert!(term_formula.generated_delta.is_empty());
        assert_eq!(
            output.source_map.algorithm_sources.len(),
            output.algorithm_statements.len()
        );
        for (_, statement) in output.algorithm_statements.iter() {
            assert_eq!(statement.owner, algorithm_id);
        }

        let let_statement = output
            .algorithm_statements
            .get(algorithm.statements[0])
            .expect("let statement");
        let expected_let_source = expected_checker_source(240, 241, "checker:algorithm:let");
        assert_eq!(let_statement.source, expected_let_source);
        assert_eq!(
            output.source_map.algorithm_sources[&algorithm.statements[0]],
            expected_let_source
        );
        let CoreAlgorithmStmtKind::Let {
            value,
            ghost,
            binder,
        } = &let_statement.kind
        else {
            panic!("expected let statement");
        };
        assert_eq!(binder.var, y);
        assert_eq!(*value, Some(term_x));
        assert!(!ghost);

        let pick_statement = output
            .algorithm_statements
            .get(algorithm.statements[1])
            .expect("runtime pick statement");
        assert_eq!(
            pick_statement.source,
            expected_checker_source(241, 242, "checker:algorithm:pick:runtime")
        );
        let CoreAlgorithmStmtKind::Pick {
            binder,
            witness_ty,
            ghost,
        } = &pick_statement.kind
        else {
            panic!("expected pick statement");
        };
        assert_eq!(binder, &runtime_pick);
        assert_eq!(*witness_ty, Some(invariant));
        assert!(!ghost);

        let ghost_pick_statement = output
            .algorithm_statements
            .get(algorithm.statements[2])
            .expect("ghost pick statement");
        let CoreAlgorithmStmtKind::Pick { binder, ghost, .. } = &ghost_pick_statement.kind else {
            panic!("expected ghost pick statement");
        };
        assert_eq!(binder, &ghost_pick_binder);
        assert!(ghost);

        let if_statement = output
            .algorithm_statements
            .get(algorithm.statements[3])
            .expect("if statement");
        assert_eq!(
            if_statement.source,
            expected_checker_source(245, 246, "checker:algorithm:if")
        );
        let CoreAlgorithmStmtKind::If {
            condition,
            then_body,
            else_body,
        } = &if_statement.kind
        else {
            panic!("expected if statement");
        };
        assert_eq!(*condition, requires);
        assert_eq!(then_body.len(), 1);
        assert_eq!(else_body.len(), 1);
        assert!(matches!(
            output
                .algorithm_statements
                .get(then_body[0])
                .expect("then assertion")
                .kind,
            CoreAlgorithmStmtKind::Assert { formula } if formula == ensures
        ));
        assert!(matches!(
            output
                .algorithm_statements
                .get(else_body[0])
                .expect("else break")
                .kind,
            CoreAlgorithmStmtKind::Break
        ));

        let while_statement = output
            .algorithm_statements
            .get(algorithm.statements[4])
            .expect("while statement");
        assert_eq!(
            while_statement.source,
            expected_checker_source(247, 248, "checker:algorithm:while")
        );
        let CoreAlgorithmStmtKind::While {
            condition,
            invariants,
            decreasing,
            body,
        } = &while_statement.kind
        else {
            panic!("expected while statement");
        };
        assert_eq!(*condition, ensures);
        assert_eq!(invariants, &vec![invariant]);
        assert_eq!(decreasing, &vec![term_x]);
        assert_eq!(body.len(), 1);
        assert!(matches!(
            output
                .algorithm_statements
                .get(body[0])
                .expect("while continue")
                .kind,
            CoreAlgorithmStmtKind::Continue
        ));

        let match_statement = output
            .algorithm_statements
            .get(algorithm.statements[5])
            .expect("match statement");
        let CoreAlgorithmStmtKind::Match { scrutinee, arms } = &match_statement.kind else {
            panic!("expected match statement");
        };
        assert_eq!(*scrutinee, term_x);
        assert_eq!(arms.len(), 1);
        assert_eq!(arms[0].pattern, CoreProvenanceKey::new("case:some"));
        assert_eq!(arms[0].body.len(), 1);
        assert!(matches!(
            output
                .algorithm_statements
                .get(arms[0].body[0])
                .expect("match return")
                .kind,
            CoreAlgorithmStmtKind::Return(Some(term)) if term == term_y
        ));

        assert!(matches!(
            &output
                .algorithm_statements
                .get(algorithm.statements[6])
                .expect("assignment")
                .kind,
            CoreAlgorithmStmtKind::Assign { target, value }
                if target == &CorePlace::new("result") && *value == term_y
        ));
        assert!(matches!(
            output
                .algorithm_statements
                .get(algorithm.statements[7])
                .expect("return")
                .kind,
            CoreAlgorithmStmtKind::Return(None)
        ));
        assert_step6_delta_valid(&context, &term_formula, &definitions, &proofs, &output);
    }

    #[test]
    fn algorithm_lowering_missing_payload_records_error_statement_and_diagnostic() {
        let (context, owner) = context_with_algorithm_var_sorts(Vec::new());
        let term_formula = lower_test_terms_and_formulas(&context, owner, Vec::new(), Vec::new());
        let definitions = empty_definition_output(&context, &term_formula);
        let proofs = empty_proof_output(&context, &term_formula, &definitions);

        let output = lower_algorithm_inputs(
            &context,
            &term_formula,
            &proofs,
            AlgorithmLoweringInput {
                algorithms: vec![algorithm_seed(
                    owner,
                    symbol("Owner"),
                    AlgorithmPayloadSeed::Missing(failed_site("algorithm-payload-missing", 260)),
                    261,
                )],
            },
        )
        .expect("algorithm lowering");
        let algorithm_id = output.algorithm_map[&owner];
        let algorithm = output.algorithms.get(algorithm_id).expect("algorithm");
        let statement_id = algorithm.statements[0];
        let statement = output
            .algorithm_statements
            .get(statement_id)
            .expect("error statement");
        let expected_source =
            expected_checker_source(260, 261, "checker:failed:algorithm-payload-missing");
        assert_eq!(statement.source, expected_source);
        assert_eq!(
            output.source_map.algorithm_sources[&statement_id],
            expected_source
        );
        let CoreAlgorithmStmtKind::Error(diagnostic_id) = &statement.kind else {
            panic!("expected error statement");
        };
        let diagnostic_id = *diagnostic_id;
        let diagnostic = output
            .diagnostics
            .get(diagnostic_id)
            .expect("algorithm diagnostic");

        assert_eq!(algorithm.statements, vec![statement_id]);
        assert_eq!(algorithm.diagnostics, vec![diagnostic_id]);
        assert_eq!(statement.diagnostics, vec![diagnostic_id]);
        assert_eq!(diagnostic.class, CoreDiagnosticClass::AlgorithmShell);
        assert_eq!(diagnostic.severity, CoreDiagnosticSeverity::Error);
        assert_eq!(diagnostic.recovery, CoreDiagnosticRecovery::Fatal);
        assert_eq!(diagnostic.message_key.as_str(), "algorithm-payload-missing");
        assert_eq!(diagnostic.primary_source, expected_source);
        assert_eq!(
            diagnostic.owner,
            Some(CoreNodeRef::AlgorithmStmt(statement_id))
        );
        assert_step6_delta_valid(&context, &term_formula, &definitions, &proofs, &output);
    }

    #[test]
    fn algorithm_lowering_malformed_statement_records_parent_diagnostic() {
        let (context, owner) = context_with_algorithm_var_sorts(Vec::new());
        let term_formula = lower_test_terms_and_formulas(&context, owner, Vec::new(), Vec::new());
        let definitions = empty_definition_output(&context, &term_formula);
        let proofs = empty_proof_output(&context, &term_formula, &definitions);

        let output = lower_algorithm_inputs(
            &context,
            &term_formula,
            &proofs,
            AlgorithmLoweringInput {
                algorithms: vec![algorithm_seed(
                    owner,
                    symbol("Owner"),
                    AlgorithmPayloadSeed::Statements(vec![AlgorithmStmtSeed::Error(failed_site(
                        "algorithm-statement-malformed",
                        262,
                    ))]),
                    263,
                )],
            },
        )
        .expect("algorithm lowering");
        let algorithm_id = output.algorithm_map[&owner];
        let algorithm = output.algorithms.get(algorithm_id).expect("algorithm");
        let statement_id = algorithm.statements[0];
        let statement = output
            .algorithm_statements
            .get(statement_id)
            .expect("error statement");
        let expected_source =
            expected_checker_source(262, 263, "checker:failed:algorithm-statement-malformed");
        assert_eq!(statement.source, expected_source);
        assert_eq!(
            output.source_map.algorithm_sources[&statement_id],
            expected_source
        );
        let CoreAlgorithmStmtKind::Error(diagnostic_id) = statement.kind else {
            panic!("expected error statement");
        };
        let diagnostic = output
            .diagnostics
            .get(diagnostic_id)
            .expect("algorithm diagnostic");

        assert_eq!(algorithm.statements, vec![statement_id]);
        assert_eq!(algorithm.diagnostics, vec![diagnostic_id]);
        assert_eq!(statement.diagnostics, vec![diagnostic_id]);
        assert_eq!(diagnostic.class, CoreDiagnosticClass::AlgorithmShell);
        assert_eq!(diagnostic.severity, CoreDiagnosticSeverity::Error);
        assert_eq!(diagnostic.recovery, CoreDiagnosticRecovery::Fatal);
        assert_eq!(
            diagnostic.message_key.as_str(),
            "algorithm-statement-malformed"
        );
        assert_eq!(diagnostic.primary_source, expected_source);
        assert_eq!(
            diagnostic.owner,
            Some(CoreNodeRef::AlgorithmStmt(statement_id))
        );
        assert_step6_delta_valid(&context, &term_formula, &definitions, &proofs, &output);
    }

    #[test]
    fn algorithm_lowering_rejects_owner_symbol_and_boundary_mismatches() {
        let context = prepare_core_context(input_with_items(vec![
            item_seed("TheoremOwner", 270),
            algorithm_item_seed("AlgorithmOwner", 271),
            CoreItemSeed::new(
                symbol("NoBoundary"),
                CoreItemKind::Algorithm,
                "public",
                direct(272, 273),
                provenance("checker:item:no-boundary"),
            ),
            CoreItemSeed::new(
                symbol("WrongBoundary"),
                CoreItemKind::Algorithm,
                "public",
                direct(273, 274),
                provenance("checker:item:wrong-boundary"),
            )
            .with_definition_boundary(DefinitionBoundaryKind::DefinitionalItem),
        ]))
        .expect("context");
        let theorem_owner = context
            .item_registry()
            .id_for_symbol(&symbol("TheoremOwner"))
            .expect("theorem owner");
        let algorithm_owner = context
            .item_registry()
            .id_for_symbol(&symbol("AlgorithmOwner"))
            .expect("algorithm owner");
        let no_boundary = context
            .item_registry()
            .id_for_symbol(&symbol("NoBoundary"))
            .expect("no boundary");
        let wrong_boundary = context
            .item_registry()
            .id_for_symbol(&symbol("WrongBoundary"))
            .expect("wrong boundary");
        let term_formula =
            lower_test_terms_and_formulas(&context, algorithm_owner, Vec::new(), Vec::new());
        let definitions = empty_definition_output(&context, &term_formula);
        let proofs = empty_proof_output(&context, &term_formula, &definitions);
        let valid_seed = algorithm_seed(
            algorithm_owner,
            symbol("AlgorithmOwner"),
            AlgorithmPayloadSeed::Statements(Vec::new()),
            274,
        );

        assert!(matches!(
            lower_algorithm_inputs(
                &context,
                &term_formula,
                &proofs,
                AlgorithmLoweringInput {
                    algorithms: vec![valid_seed.clone(), valid_seed],
                },
            ),
            Err(AlgorithmLoweringError::DuplicateAlgorithmOwner { owner })
                if owner == algorithm_owner
        ));

        assert!(matches!(
            lower_algorithm_inputs(
                &context,
                &term_formula,
                &proofs,
                AlgorithmLoweringInput {
                    algorithms: vec![algorithm_seed(
                        CoreItemId::new(99),
                        symbol("MissingOwner"),
                        AlgorithmPayloadSeed::Statements(Vec::new()),
                        275,
                    )],
                },
            ),
            Err(AlgorithmLoweringError::MissingOwnerItem { owner })
                if owner == CoreItemId::new(99)
        ));

        assert!(matches!(
            lower_algorithm_inputs(
                &context,
                &term_formula,
                &proofs,
                AlgorithmLoweringInput {
                    algorithms: vec![algorithm_seed(
                        theorem_owner,
                        symbol("TheoremOwner"),
                        AlgorithmPayloadSeed::Statements(Vec::new()),
                        276,
                    )],
                },
            ),
            Err(AlgorithmLoweringError::UnsupportedAlgorithmItemKind { owner, kind })
                if owner == theorem_owner && kind == CoreItemKind::Theorem
        ));

        assert!(matches!(
            lower_algorithm_inputs(
                &context,
                &term_formula,
                &proofs,
                AlgorithmLoweringInput {
                    algorithms: vec![algorithm_seed(
                        algorithm_owner,
                        symbol("WrongOwnerSymbol"),
                        AlgorithmPayloadSeed::Statements(Vec::new()),
                        277,
                    )],
                },
            ),
            Err(AlgorithmLoweringError::AlgorithmSymbolMismatch { owner, .. })
                if owner == algorithm_owner
        ));

        assert!(matches!(
            lower_algorithm_inputs(
                &context,
                &term_formula,
                &proofs,
                AlgorithmLoweringInput {
                    algorithms: vec![algorithm_seed(
                        no_boundary,
                        symbol("NoBoundary"),
                        AlgorithmPayloadSeed::Statements(Vec::new()),
                        278,
                    )],
                },
            ),
            Err(AlgorithmLoweringError::MissingAlgorithmBoundary { owner })
                if owner == no_boundary
        ));

        assert!(matches!(
            lower_algorithm_inputs(
                &context,
                &term_formula,
                &proofs,
                AlgorithmLoweringInput {
                    algorithms: vec![algorithm_seed(
                        wrong_boundary,
                        symbol("WrongBoundary"),
                        AlgorithmPayloadSeed::Statements(Vec::new()),
                        279,
                    )],
                },
            ),
            Err(AlgorithmLoweringError::AlgorithmBoundaryMismatch { owner, kind })
                if owner == wrong_boundary && kind == DefinitionBoundaryKind::DefinitionalItem
        ));

        let mut skipped_context = context.clone();
        skipped_context
            .definition_boundaries
            .by_item
            .get_mut(&algorithm_owner)
            .expect("algorithm boundary")
            .status = DefinitionBoundaryStatus::Skipped;
        assert!(matches!(
            lower_algorithm_inputs(
                &skipped_context,
                &term_formula,
                &proofs,
                AlgorithmLoweringInput {
                    algorithms: vec![algorithm_seed(
                        algorithm_owner,
                        symbol("AlgorithmOwner"),
                        AlgorithmPayloadSeed::Statements(Vec::new()),
                        280,
                    )],
                },
            ),
            Err(AlgorithmLoweringError::AlgorithmBoundaryNotPending { owner, status })
                if owner == algorithm_owner && status == DefinitionBoundaryStatus::Skipped
        ));
    }

    #[test]
    fn algorithm_lowering_rejects_invalid_binders_terms_formulas_and_targets() {
        let x = CoreVarId::new(0);
        let formula_var = CoreVarId::new(1);
        let (context, owner) = context_with_algorithm_var_sorts(vec![
            (x, NormalizedVarSort::Term),
            (formula_var, NormalizedVarSort::Formula),
        ]);
        let term_formula = lower_test_terms_and_formulas(
            &context,
            owner,
            vec![term_seed(CoreTermSeedKind::Var(x), 280)],
            vec![formula_seed(CoreFormulaSeedKind::True, 281)],
        );
        let definitions = empty_definition_output(&context, &term_formula);
        let proofs = empty_proof_output(&context, &term_formula, &definitions);
        let term = term_formula.term_map[&CoreTermSeedId::new(0)];
        let formula = term_formula.formula_map[&CoreFormulaSeedId::new(0)];

        let mut non_term_binder = algorithm_seed(
            owner,
            symbol("Owner"),
            AlgorithmPayloadSeed::Statements(Vec::new()),
            282,
        );
        non_term_binder.params = vec![test_binder(formula_var, None, 283)];
        assert!(matches!(
            lower_algorithm_inputs(
                &context,
                &term_formula,
                &proofs,
                AlgorithmLoweringInput {
                    algorithms: vec![non_term_binder],
                },
            ),
            Err(AlgorithmLoweringError::NonTermAlgorithmBinder { var, sort })
                if var == formula_var && sort == NormalizedVarSort::Formula
        ));

        let undeclared = CoreVarId::new(99);
        let mut undeclared_binder = algorithm_seed(
            owner,
            symbol("Owner"),
            AlgorithmPayloadSeed::Statements(Vec::new()),
            284,
        );
        undeclared_binder.result = Some(test_binder(undeclared, None, 285));
        assert!(matches!(
            lower_algorithm_inputs(
                &context,
                &term_formula,
                &proofs,
                AlgorithmLoweringInput {
                    algorithms: vec![undeclared_binder],
                },
            ),
            Err(AlgorithmLoweringError::UndeclaredAlgorithmBinder { var })
                if var == undeclared
        ));

        let missing_formula = CoreFormulaId::new(99);
        let mut bad_contract_formula = algorithm_seed(
            owner,
            symbol("Owner"),
            AlgorithmPayloadSeed::Statements(Vec::new()),
            286,
        );
        bad_contract_formula.contracts.requires = vec![missing_formula];
        assert!(matches!(
            lower_algorithm_inputs(
                &context,
                &term_formula,
                &proofs,
                AlgorithmLoweringInput {
                    algorithms: vec![bad_contract_formula],
                },
            ),
            Err(AlgorithmLoweringError::MissingAlgorithmFormula { formula })
                if formula == missing_formula
        ));

        let missing_term = CoreTermId::new(99);
        let mut bad_contract_term = algorithm_seed(
            owner,
            symbol("Owner"),
            AlgorithmPayloadSeed::Statements(Vec::new()),
            287,
        );
        bad_contract_term.contracts.decreasing = vec![missing_term];
        assert!(matches!(
            lower_algorithm_inputs(
                &context,
                &term_formula,
                &proofs,
                AlgorithmLoweringInput {
                    algorithms: vec![bad_contract_term],
                },
            ),
            Err(AlgorithmLoweringError::MissingAlgorithmTerm { term })
                if term == missing_term
        ));

        assert!(matches!(
            lower_algorithm_inputs(
                &context,
                &term_formula,
                &proofs,
                AlgorithmLoweringInput {
                    algorithms: vec![algorithm_seed(
                        owner,
                        symbol("Owner"),
                        AlgorithmPayloadSeed::Statements(vec![AlgorithmStmtSeed::If {
                            condition: missing_formula,
                            then_body: Vec::new(),
                            else_body: Vec::new(),
                            source: direct(288, 289),
                            provenance: provenance("checker:algorithm:bad-if"),
                        }]),
                        289,
                    )],
                },
            ),
            Err(AlgorithmLoweringError::MissingAlgorithmFormula { formula })
                if formula == missing_formula
        ));

        assert!(matches!(
            lower_algorithm_inputs(
                &context,
                &term_formula,
                &proofs,
                AlgorithmLoweringInput {
                    algorithms: vec![algorithm_seed(
                        owner,
                        symbol("Owner"),
                        AlgorithmPayloadSeed::Statements(vec![AlgorithmStmtSeed::While {
                            condition: formula,
                            invariants: vec![missing_formula],
                            decreasing: Vec::new(),
                            body: Vec::new(),
                            source: direct(290, 291),
                            provenance: provenance("checker:algorithm:bad-while-invariant"),
                        }]),
                        291,
                    )],
                },
            ),
            Err(AlgorithmLoweringError::MissingAlgorithmFormula { formula })
                if formula == missing_formula
        ));

        assert!(matches!(
            lower_algorithm_inputs(
                &context,
                &term_formula,
                &proofs,
                AlgorithmLoweringInput {
                    algorithms: vec![algorithm_seed(
                        owner,
                        symbol("Owner"),
                        AlgorithmPayloadSeed::Statements(vec![AlgorithmStmtSeed::While {
                            condition: formula,
                            invariants: Vec::new(),
                            decreasing: vec![missing_term],
                            body: Vec::new(),
                            source: direct(292, 293),
                            provenance: provenance("checker:algorithm:bad-while-decreasing"),
                        }]),
                        293,
                    )],
                },
            ),
            Err(AlgorithmLoweringError::MissingAlgorithmTerm { term })
                if term == missing_term
        ));

        assert!(matches!(
            lower_algorithm_inputs(
                &context,
                &term_formula,
                &proofs,
                AlgorithmLoweringInput {
                    algorithms: vec![algorithm_seed(
                        owner,
                        symbol("Owner"),
                        AlgorithmPayloadSeed::Statements(vec![AlgorithmStmtSeed::Match {
                            scrutinee: missing_term,
                            arms: Vec::new(),
                            source: direct(294, 295),
                            provenance: provenance("checker:algorithm:bad-match"),
                        }]),
                        295,
                    )],
                },
            ),
            Err(AlgorithmLoweringError::MissingAlgorithmTerm { term })
                if term == missing_term
        ));

        assert!(matches!(
            lower_algorithm_inputs(
                &context,
                &term_formula,
                &proofs,
                AlgorithmLoweringInput {
                    algorithms: vec![algorithm_seed(
                        owner,
                        symbol("Owner"),
                        AlgorithmPayloadSeed::Statements(vec![AlgorithmStmtSeed::Return {
                            value: Some(missing_term),
                            source: direct(296, 297),
                            provenance: provenance("checker:algorithm:bad-return"),
                        }]),
                        297,
                    )],
                },
            ),
            Err(AlgorithmLoweringError::MissingAlgorithmTerm { term })
                if term == missing_term
        ));

        assert!(matches!(
            lower_algorithm_inputs(
                &context,
                &term_formula,
                &proofs,
                AlgorithmLoweringInput {
                    algorithms: vec![algorithm_seed(
                        owner,
                        symbol("Owner"),
                        AlgorithmPayloadSeed::Statements(vec![AlgorithmStmtSeed::Assign {
                            target: CorePlace::new("result"),
                            value: missing_term,
                            source: direct(298, 299),
                            provenance: provenance("checker:algorithm:missing-term"),
                        }]),
                        299,
                    )],
                },
            ),
            Err(AlgorithmLoweringError::MissingAlgorithmTerm { term })
                if term == missing_term
        ));

        assert!(matches!(
            lower_algorithm_inputs(
                &context,
                &term_formula,
                &proofs,
                AlgorithmLoweringInput {
                    algorithms: vec![algorithm_seed(
                        owner,
                        symbol("Owner"),
                        AlgorithmPayloadSeed::Statements(vec![AlgorithmStmtSeed::Assert {
                            formula: missing_formula,
                            source: direct(300, 301),
                            provenance: provenance("checker:algorithm:missing-formula"),
                        }]),
                        301,
                    )],
                },
            ),
            Err(AlgorithmLoweringError::MissingAlgorithmFormula { formula })
                if formula == missing_formula
        ));

        assert!(matches!(
            lower_algorithm_inputs(
                &context,
                &term_formula,
                &proofs,
                AlgorithmLoweringInput {
                    algorithms: vec![algorithm_seed(
                        owner,
                        symbol("Owner"),
                        AlgorithmPayloadSeed::Statements(vec![AlgorithmStmtSeed::Assign {
                            target: CorePlace::new(""),
                            value: term,
                            source: direct(302, 303),
                            provenance: provenance("checker:algorithm:bad-target"),
                        }]),
                        303,
                    )],
                },
            ),
            Err(AlgorithmLoweringError::InvalidAlgorithmTarget { target })
                if target.as_str().is_empty()
        ));

        assert!(matches!(
            lower_algorithm_inputs(
                &context,
                &term_formula,
                &proofs,
                AlgorithmLoweringInput {
                    algorithms: vec![algorithm_seed(
                        owner,
                        symbol("Owner"),
                        AlgorithmPayloadSeed::Statements(vec![AlgorithmStmtSeed::Pick {
                            binder: test_binder(x, None, 304),
                            witness_ty: Some(missing_formula),
                            ghost: false,
                            source: direct(304, 305),
                            provenance: provenance("checker:algorithm:bad-pick-witness"),
                        }]),
                        305,
                    )],
                },
            ),
            Err(AlgorithmLoweringError::MissingAlgorithmFormula { formula })
                if formula == missing_formula
        ));

        assert!(matches!(
            lower_algorithm_inputs(
                &context,
                &term_formula,
                &proofs,
                AlgorithmLoweringInput {
                    algorithms: vec![algorithm_seed(
                        owner,
                        symbol("Owner"),
                        AlgorithmPayloadSeed::Statements(vec![AlgorithmStmtSeed::Pick {
                            binder: test_binder(x, Some(missing_formula), 306),
                            witness_ty: Some(formula),
                            ghost: false,
                            source: direct(306, 307),
                            provenance: provenance("checker:algorithm:bad-pick-binder"),
                        }]),
                        307,
                    )],
                },
            ),
            Err(AlgorithmLoweringError::MissingAlgorithmFormula { formula })
                if formula == missing_formula
        ));
    }

    #[test]
    fn context_assigns_item_ids_in_deterministic_source_order() {
        let input = input_with_items(vec![item_seed("Later", 20), item_seed("Earlier", 0)]);

        let context = prepare_core_context(input).expect("context");
        let earlier = context
            .item_registry()
            .id_for_symbol(&symbol("Earlier"))
            .expect("earlier id");
        let later = context
            .item_registry()
            .id_for_symbol(&symbol("Later"))
            .expect("later id");

        assert_eq!(earlier.index(), 0);
        assert_eq!(later.index(), 1);
        assert!(
            context
                .definition_boundaries()
                .get_by_item(earlier)
                .is_some()
        );
        assert_eq!(
            context
                .worklist()
                .entries()
                .iter()
                .map(|entry| &entry.kind)
                .collect::<Vec<_>>(),
            vec![
                &ElaborationWorkItemKind::Item(earlier),
                &ElaborationWorkItemKind::Item(later)
            ]
        );
    }

    #[test]
    fn missing_dependency_summary_is_diagnostic_not_source_inspection() {
        let missing = external_symbol("UnavailableDependency");
        let input = input_with_items(vec![
            item_seed("UsesMissing", 0).with_dependencies(vec![missing]),
        ]);

        let context = prepare_core_context(input).expect("context with diagnostic");
        let item = context
            .item_registry()
            .id_for_symbol(&symbol("UsesMissing"))
            .expect("item id");
        let item_row = context.item_registry().items().get(item).expect("item row");
        let resolution = context
            .item_registry()
            .dependencies(item)
            .expect("dependency resolution");
        let diagnostic = context
            .diagnostics()
            .get(item_row.diagnostics[0])
            .expect("diagnostic");

        assert_eq!(item_row.status, CoreItemStatus::Partial);
        assert_eq!(resolution.missing.len(), 1);
        assert_eq!(
            diagnostic.class,
            CoreDiagnosticClass::UnresolvedSemanticInput
        );
        assert_eq!(
            diagnostic.message_key.as_str(),
            "missing-dependency-summary"
        );
        assert!(matches!(
            context.worklist().entries()[0].status,
            ElaborationWorkStatus::Skipped
        ));
    }

    #[test]
    fn item_registry_uses_canonical_symbol_ids_without_raw_spelling_identity() {
        let canonical = SymbolId::new(
            module_id(),
            LocalSymbolId::new("CanonicalLocal"),
            FullyQualifiedName::new("pkg::main::CanonicalFqn"),
        );
        let raw_spelling = "source wrote a different spelling";
        let input = input_with_items(vec![CoreItemSeed::new(
            canonical.clone(),
            CoreItemKind::Predicate,
            "public",
            direct(0, raw_spelling.len()),
            provenance("checker:item:canonical"),
        )]);

        let context = prepare_core_context(input).expect("context");
        let id = context
            .item_registry()
            .id_for_symbol(&canonical)
            .expect("canonical lookup");
        let item = context.item_registry().items().get(id).expect("item");

        assert_eq!(item.symbol, canonical);
        assert!(!format!("{:?}", context.item_registry()).contains(raw_spelling));
    }

    #[test]
    fn dependency_resolution_uses_exact_canonical_symbol_identity() {
        let local_shared = symbol("Shared");
        let external_shared = external_symbol("Shared");
        let mut input = input_with_items(vec![
            CoreItemSeed::new(
                local_shared.clone(),
                CoreItemKind::Predicate,
                "public",
                direct(0, 3),
                provenance("checker:item:local-shared"),
            ),
            item_seed("UsesBoth", 10)
                .with_dependencies(vec![external_shared.clone(), local_shared.clone()]),
        ]);
        input.dependency_summaries = vec![CoreDependencySummary::new(
            external_shared.clone(),
            CoreItemKind::Predicate,
            "public",
            provenance("checker:dependency:external-shared"),
        )];

        let context = prepare_core_context(input).expect("context");
        let local_id = context
            .item_registry()
            .id_for_symbol(&local_shared)
            .expect("local shared id");
        let uses_id = context
            .item_registry()
            .id_for_symbol(&symbol("UsesBoth"))
            .expect("uses id");
        let resolution = context
            .item_registry()
            .dependencies(uses_id)
            .expect("dependency resolution");

        assert_eq!(resolution.local, vec![local_id]);
        assert_eq!(resolution.external, vec![external_shared]);
        assert!(resolution.missing.is_empty());
    }

    #[test]
    fn definition_boundaries_are_initialized_before_body_lowering() {
        let input = input_with_items(vec![
            item_seed("RecursiveA", 0).with_dependencies(vec![symbol("RecursiveB")]),
            item_seed("RecursiveB", 10).with_dependencies(vec![symbol("RecursiveA")]),
        ]);

        let context = prepare_core_context(input).expect("context");
        let a = context
            .definition_boundaries()
            .get_by_symbol(&symbol("RecursiveA"))
            .expect("boundary a");
        let b = context
            .definition_boundaries()
            .get_by_symbol(&symbol("RecursiveB"))
            .expect("boundary b");

        assert_eq!(a.status, DefinitionBoundaryStatus::PendingBody);
        assert_eq!(b.status, DefinitionBoundaryStatus::PendingBody);
        assert_eq!(a.kind, DefinitionBoundaryKind::Theorem);
        assert_eq!(b.kind, DefinitionBoundaryKind::Theorem);
    }

    #[test]
    fn failed_checker_sites_are_preserved_as_error_work_items() {
        let failed_site =
            CheckerSiteSummary::failed_overload(OverloadResolutionId::new(7), direct(5, 9));
        let resolved = ResolvedTypedAstSummary::new(source_id(), module_id())
            .with_checker_sites(vec![failed_site.clone()]);
        let input = CoreContextInput::new(resolved);

        let context = prepare_core_context(input).expect("context");
        let entry = &context.worklist().entries()[0];
        let diagnostic = context
            .diagnostics()
            .get(entry.diagnostics[0])
            .expect("diagnostic");

        assert_eq!(
            entry.kind,
            ElaborationWorkItemKind::CheckerSite(failed_site.kind)
        );
        assert_eq!(entry.status, ElaborationWorkStatus::Error);
        assert_eq!(diagnostic.class, CoreDiagnosticClass::UnsupportedLowering);
        assert_eq!(
            diagnostic.message_key.as_str(),
            "checker-error-site-preserved"
        );
    }

    #[test]
    fn recovered_checker_sites_are_preserved_as_skipped_work_items() {
        let recovered_diagnostic = ResolvedTypedDiagnosticId::new(3);
        let note_diagnostic = ResolvedTypedDiagnosticId::new(4);
        let recovered_site = CheckerSiteSummary {
            kind: CheckerSiteKind::RecoveredNode {
                node: ResolvedTypedNodeId::new(2),
                recovery: ResolvedNodeRecovery::Recovered,
            },
            source: direct(1, 2),
            diagnostics: vec![recovered_diagnostic],
            severity: CheckerSiteSeverity::Warning,
        };
        let note_site = CheckerSiteSummary {
            kind: CheckerSiteKind::CheckerDiagnostic {
                diagnostic: note_diagnostic,
            },
            source: direct(3, 4),
            diagnostics: vec![note_diagnostic],
            severity: CheckerSiteSeverity::Note,
        };
        let resolved = ResolvedTypedAstSummary::new(source_id(), module_id())
            .with_checker_sites(vec![note_site, recovered_site]);
        let input = CoreContextInput::new(resolved);

        let context = prepare_core_context(input).expect("context");
        let entries = context.worklist().entries();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].status, ElaborationWorkStatus::Skipped);
        assert_eq!(entries[0].checker_diagnostics, vec![recovered_diagnostic]);
        assert_eq!(entries[1].status, ElaborationWorkStatus::Skipped);
        assert_eq!(entries[1].checker_diagnostics, vec![note_diagnostic]);
        assert_eq!(
            context
                .diagnostics()
                .get(entries[0].diagnostics[0])
                .expect("warning diagnostic")
                .message_key
                .as_str(),
            "checker-recovered-site-preserved"
        );
        assert_eq!(
            context
                .diagnostics()
                .get(entries[1].diagnostics[0])
                .expect("note diagnostic")
                .message_key
                .as_str(),
            "checker-note-site-preserved"
        );
    }

    #[test]
    fn source_map_and_generated_origin_registry_are_initialized() {
        let owner = symbol("Owner");
        let origin_key = GeneratedOriginKey::new("choice:Owner:0");
        let mut input = input_with_items(vec![
            CoreItemSeed::new(
                owner.clone(),
                CoreItemKind::GeneratedDefinition,
                "private",
                direct(0, 4),
                provenance("checker:item:owner"),
            )
            .with_definition_boundary(DefinitionBoundaryKind::GeneratedDefinition),
        ]);
        input.generated_origin_seeds = vec![
            GeneratedOriginSeed::new(
                owner.clone(),
                GeneratedOriginKind::StableChoice,
                origin_key.clone(),
                direct(10, 12),
                provenance("checker:generated:choice"),
            )
            .with_params(vec![CoreVarId::new(0)])
            .with_evidence(vec![CoreProvenance::new(
                CoreProvenancePhase::Checker,
                "choice-evidence",
            )]),
        ];

        let context = prepare_core_context(input).expect("context");
        let owner_id = context
            .item_registry()
            .id_for_symbol(&owner)
            .expect("owner id");
        let generated_id = context
            .generated_origins()
            .get_by_key(owner_id, GeneratedOriginKind::StableChoice, &origin_key)
            .expect("generated id");

        assert!(context.source_map().item_sources.contains_key(&owner_id));
        assert!(
            context
                .source_map()
                .generated_sources
                .contains_key(&generated_id)
        );
        assert!(context.source_map().term_sources.is_empty());
        assert_eq!(context.generated_origins().table().len(), 1);
    }

    #[test]
    fn binder_and_variable_seeds_prepare_binder_metadata_context() {
        let var = CoreVarId::new(3);
        let mut input = CoreContextInput::new(summary());
        input.variable_seeds = vec![
            CoreVariableSeed::new(
                var,
                NormalizedVarClass::Free,
                "term-binder",
                NormalizedVarSort::Term,
                provenance("checker:var:3"),
            )
            .with_type_facts(vec![
                TypeFactId::new(2),
                TypeFactId::new(1),
                TypeFactId::new(1),
            ]),
        ];
        input.binder_seeds = vec![CoreBinderSeed::new(
            var,
            direct(15, 16),
            provenance("checker:binder:3"),
        )];

        let context = prepare_core_context(input).expect("context");

        assert!(context.binder_context().free_variables.contains(&var));
        assert_eq!(
            context.binder_context().variable_classes.get(&var),
            Some(&NormalizedVarClass::Free)
        );
        assert_eq!(
            context.binder_context().variable_roles.get(&var),
            Some(&CoreVarRole::new("term-binder"))
        );
        assert_eq!(
            context.binder_context().variable_sorts.get(&var),
            Some(&NormalizedVarSort::Term)
        );
        assert_eq!(
            context.binder_type_facts().get(&var),
            Some(&vec![TypeFactId::new(1), TypeFactId::new(2)])
        );
        assert!(context.binder_sources().get(var).is_some());
    }

    #[test]
    fn binder_source_requires_declared_variable_metadata() {
        let var = CoreVarId::new(9);
        let mut input = CoreContextInput::new(summary());
        input.binder_seeds = vec![CoreBinderSeed::new(
            var,
            direct(20, 21),
            provenance("checker:binder:undeclared"),
        )];

        assert!(matches!(
            prepare_core_context(input),
            Err(CoreContextError::UndeclaredBinderVariable { var: actual }) if actual == var
        ));
    }

    #[test]
    fn current_and_external_module_inputs_are_kept_distinct() {
        let external = external_symbol("ForeignItem");
        let foreign_item_input = input_with_items(vec![CoreItemSeed::new(
            external.clone(),
            CoreItemKind::Theorem,
            "public",
            direct(0, 3),
            provenance("checker:item:foreign"),
        )]);

        assert!(matches!(
            prepare_core_context(foreign_item_input),
            Err(CoreContextError::ForeignItemSeed { symbol, .. }) if symbol.as_ref() == &external
        ));

        let current = symbol("CurrentSummary");
        let mut current_summary_input = CoreContextInput::new(summary());
        current_summary_input.dependency_summaries = vec![CoreDependencySummary::new(
            current.clone(),
            CoreItemKind::Predicate,
            "public",
            provenance("checker:dependency:current"),
        )];

        assert!(matches!(
            prepare_core_context(current_summary_input),
            Err(CoreContextError::CurrentModuleDependencySummary { symbol }) if symbol.as_ref() == &current
        ));
    }

    #[test]
    fn unprovenanced_checker_seed_is_rejected() {
        let fabricated = CheckerOwnedProvenance {
            entries: Vec::new(),
        };
        let input = input_with_items(vec![CoreItemSeed::new(
            symbol("Fabricated"),
            CoreItemKind::Theorem,
            "public",
            direct(0, 3),
            fabricated,
        )]);

        assert!(matches!(
            prepare_core_context(input),
            Err(CoreContextError::MissingProvenance { input: "item seed" })
        ));
    }

    #[test]
    fn non_checker_owned_seed_phase_is_rejected() {
        let generated_phase = CheckerOwnedProvenance {
            entries: vec![CoreProvenance::new(
                CoreProvenancePhase::Generated,
                "generated-only",
            )],
        };
        let input = input_with_items(vec![CoreItemSeed::new(
            symbol("GeneratedOnly"),
            CoreItemKind::Theorem,
            "public",
            direct(0, 3),
            generated_phase,
        )]);

        assert!(matches!(
            prepare_core_context(input),
            Err(CoreContextError::UnsupportedProvenancePhase {
                input: "item seed",
                phase: CoreProvenancePhase::Generated
            })
        ));
    }

    #[test]
    fn generated_origin_evidence_must_be_checker_owned() {
        let owner = symbol("Owner");
        let mut input = input_with_items(vec![CoreItemSeed::new(
            owner.clone(),
            CoreItemKind::GeneratedDefinition,
            "private",
            direct(0, 4),
            provenance("checker:item:owner"),
        )]);
        input.generated_origin_seeds = vec![
            GeneratedOriginSeed::new(
                owner,
                GeneratedOriginKind::StableChoice,
                "choice:bad-evidence",
                direct(10, 12),
                provenance("checker:generated:choice"),
            )
            .with_evidence(vec![CoreProvenance::new(
                CoreProvenancePhase::Generated,
                "generated-only",
            )]),
        ];

        assert!(matches!(
            prepare_core_context(input),
            Err(CoreContextError::UnsupportedProvenancePhase {
                input: "generated origin evidence",
                phase: CoreProvenancePhase::Generated
            })
        ));
    }

    fn task33c4c8_variable_seed(var: CoreVarId, role: &str) -> CoreVariableSeed {
        CoreVariableSeed::new(
            var,
            NormalizedVarClass::Free,
            role,
            NormalizedVarSort::Term,
            provenance(format!("checker:task33c4c8:var:{}", var.index()).as_str()),
        )
    }

    fn task33c4c8_context_with_complete_used_inventory() -> CoreContext {
        let owner = symbol("CaptureOwner");
        let mut input = input_with_items(vec![CoreItemSeed::new(
            owner.clone(),
            CoreItemKind::Functor,
            "public",
            direct(20, 30),
            provenance("checker:task33c4c8:owner"),
        )]);
        for index in [2, 9] {
            let var = CoreVarId::new(index);
            input
                .variable_seeds
                .push(task33c4c8_variable_seed(var, "existing-term"));
            input.binder_seeds.push(CoreBinderSeed::new(
                var,
                direct(30 + index, 31 + index),
                provenance(format!("checker:task33c4c8:binder:{index}").as_str()),
            ));
        }
        input.generated_origin_seeds.push(
            GeneratedOriginSeed::new(
                owner,
                GeneratedOriginKind::StableChoice,
                "task33c4c8-existing-origin",
                direct(50, 55),
                provenance("checker:task33c4c8:origin"),
            )
            .with_params(vec![CoreVarId::new(9)]),
        );
        let mut context = prepare_core_context(input).expect("Task33C4C8 unit context");
        context
            .binder_context
            .frames
            .push(crate::binder_normalization::BinderFrame::new(
                0,
                CoreVarId::new(2),
                "existing-frame",
                direct(60, 61),
            ));
        context
    }

    #[test]
    fn task33c4c8_core_context_shape_is_complete_and_fail_closed() {
        let context = task33c4c8_context_with_complete_used_inventory();
        let used = validate_core_context_shape(&context, &BTreeSet::new())
            .expect("complete used-variable inventory");
        assert_eq!(used, BTreeSet::from([CoreVarId::new(2), CoreVarId::new(9)]));
        assert_eq!(
            allocate_capture_core_vars(&used, 2).expect("checked max-plus-one allocation"),
            [CoreVarId::new(10), CoreVarId::new(11)]
        );

        let mut missing_sort = context.clone();
        missing_sort
            .binder_context
            .variable_sorts
            .remove(&CoreVarId::new(2));
        assert_eq!(
            validate_core_context_shape(&missing_sort, &BTreeSet::new()),
            Err(SourceNestedFraenkelCaptureCoreContextError::InvalidCoreContext)
        );

        let mut reserved_role = context.clone();
        reserved_role.binder_context.variable_roles.insert(
            CoreVarId::new(2),
            CoreVarRole::new(NESTED_FRAENKEL_CAPTURE_CORE_ROLE),
        );
        assert_eq!(
            validate_core_context_shape(&reserved_role, &BTreeSet::new()),
            Err(SourceNestedFraenkelCaptureCoreContextError::InvalidCoreContext)
        );

        let mut undeclared_frame = context.clone();
        undeclared_frame.binder_context.frames[0].original_var = CoreVarId::new(99);
        assert_eq!(
            validate_core_context_shape(&undeclared_frame, &BTreeSet::new()),
            Err(SourceNestedFraenkelCaptureCoreContextError::InvalidCoreContext)
        );

        let mut undeclared_generated_param = context.clone();
        undeclared_generated_param
            .generated_origins
            .table
            .get_mut(GeneratedOriginId::new(0))
            .expect("existing origin")
            .params = vec![CoreVarId::new(99)];
        assert_eq!(
            validate_core_context_shape(&undeclared_generated_param, &BTreeSet::new()),
            Err(SourceNestedFraenkelCaptureCoreContextError::InvalidCoreContext)
        );

        let mut mismatched_binder_record = context;
        mismatched_binder_record
            .binder_sources
            .by_var
            .get_mut(&CoreVarId::new(2))
            .expect("binder record")
            .var = CoreVarId::new(9);
        assert_eq!(
            validate_core_context_shape(&mismatched_binder_record, &BTreeSet::new()),
            Err(SourceNestedFraenkelCaptureCoreContextError::InvalidCoreContext)
        );
    }

    #[test]
    fn task33c4c8_owner_origin_and_item_checks_are_exact() {
        let owner_range = range(20, 30);
        let context = prepare_core_context(input_with_items(vec![CoreItemSeed::new(
            symbol("CaptureOwner"),
            CoreItemKind::Functor,
            "public",
            CoreSourceRef::direct(owner_range),
            provenance("checker:task33c4c8:owner"),
        )]))
        .expect("Task33C4C8 owner context");
        let owner_id = context
            .item_registry
            .id_for_symbol(&symbol("CaptureOwner"))
            .expect("owner item");
        let origin = SemanticOrigin::new(
            source_id(),
            module_id(),
            SourceAnchor::Range(owner_range),
            Vec::new(),
        );
        assert_eq!(validate_owner_origin(&context, &origin), Ok(owner_range));
        assert_eq!(
            validate_owner_item(&context, &symbol("CaptureOwner"), owner_range),
            Ok(owner_id)
        );

        let source_allocator = InMemorySessionIdAllocator::new();
        let source_snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            "09".repeat(32)
        ))
        .expect("valid alternate snapshot id");
        let _discarded = source_allocator
            .next_source_id(source_snapshot)
            .expect("first alternate source id");
        let alternate_source = source_allocator
            .next_source_id(source_snapshot)
            .expect("second alternate source id");
        let wrong_source_origin = SemanticOrigin::new(
            alternate_source,
            module_id(),
            SourceAnchor::Range(owner_range),
            Vec::new(),
        );
        let wrong_module_origin = SemanticOrigin::new(
            source_id(),
            external_module_id(),
            SourceAnchor::Range(owner_range),
            Vec::new(),
        );
        let point_origin = SemanticOrigin::new(
            source_id(),
            module_id(),
            SourceAnchor::Point {
                source_id: source_id(),
                offset: owner_range.start,
            },
            Vec::new(),
        );
        for invalid in [
            wrong_source_origin,
            wrong_module_origin,
            point_origin,
            origin.recovered(),
        ] {
            assert_eq!(
                validate_owner_origin(&context, &invalid),
                Err(SourceNestedFraenkelCaptureCoreContextError::InvalidOwnerAssociation)
            );
        }

        let mut invalid_status = context.clone();
        invalid_status
            .item_registry
            .items
            .get_mut(owner_id)
            .expect("owner")
            .status = CoreItemStatus::Partial;
        let mut wrong_range = context.clone();
        wrong_range
            .item_registry
            .items
            .get_mut(owner_id)
            .expect("owner")
            .source = direct(21, 30);
        wrong_range
            .source_map
            .item_sources
            .insert(owner_id, direct(21, 30));
        let mut missing_source_map = context.clone();
        missing_source_map.source_map.item_sources.remove(&owner_id);
        let mut near_match = context;
        near_match
            .item_registry
            .items
            .get_mut(owner_id)
            .expect("owner")
            .symbol = SymbolId::new(
            module_id(),
            LocalSymbolId::new("CaptureOwner"),
            FullyQualifiedName::new("pkg::main::CaptureOwnerDisplayNearMatch"),
        );
        for invalid in [invalid_status, wrong_range, missing_source_map, near_match] {
            assert_eq!(
                validate_owner_item(&invalid, &symbol("CaptureOwner"), owner_range),
                Err(SourceNestedFraenkelCaptureCoreContextError::InvalidOwnerAssociation)
            );
        }
    }

    fn task33c4c8_capture_validation_fixture() -> (
        CoreContext,
        SourceNestedFraenkelCaptureCoreVariableTable,
        Vec<CaptureAssociation>,
        BTreeSet<CoreVarId>,
    ) {
        let mut input = CoreContextInput::new(summary());
        for index in [2, 9, 10, 11] {
            let var = CoreVarId::new(index);
            let capture = index.checked_sub(10);
            let role = if capture.is_some() {
                NESTED_FRAENKEL_CAPTURE_CORE_ROLE
            } else {
                "existing-term"
            };
            input
                .variable_seeds
                .push(task33c4c8_variable_seed(var, role));
            let (source, binder_provenance) = if let Some(capture) = capture {
                let capture_id = SourceNestedFraenkelCaptureGraphCaptureId::new(capture);
                let key = capture_provenance_key(capture_id);
                (
                    CoreSourceRef::direct(range(100 + capture, 101 + capture)).with_provenance(
                        vec![CoreProvenance::new(
                            CoreProvenancePhase::Checker,
                            key.clone(),
                        )],
                    ),
                    CheckerOwnedProvenance::checker(key),
                )
            } else {
                (
                    direct(70 + index, 71 + index),
                    provenance(format!("checker:task33c4c8:binder:{index}").as_str()),
                )
            };
            input
                .binder_seeds
                .push(CoreBinderSeed::new(var, source, binder_provenance));
        }
        let context = prepare_core_context(input).expect("Task33C4C8 capture validation context");
        let associations = vec![
            CaptureAssociation {
                capture: SourceNestedFraenkelCaptureGraphCaptureId::new(0),
                generator: mizar_checker::source_formula_composition::SourceNestedFraenkelCaptureGraphGeneratorId::new(1),
                resolver_binding: FraenkelGeneratorVariableBindingId::new(1),
                binder_range: range(100, 101),
            },
            CaptureAssociation {
                capture: SourceNestedFraenkelCaptureGraphCaptureId::new(1),
                generator: mizar_checker::source_formula_composition::SourceNestedFraenkelCaptureGraphGeneratorId::new(2),
                resolver_binding: FraenkelGeneratorVariableBindingId::new(2),
                binder_range: range(101, 102),
            },
        ];
        let mut table = SourceNestedFraenkelCaptureCoreVariableTable::empty();
        for (association, core_var) in associations
            .iter()
            .zip([CoreVarId::new(10), CoreVarId::new(11)])
        {
            table.rows.insert(
                association.capture,
                SourceNestedFraenkelCaptureCoreVariable {
                    capture: association.capture,
                    generator: association.generator,
                    resolver_binding: association.resolver_binding,
                    core_var,
                },
            );
        }
        let allowed = capture_vars(&table);
        let used = validate_core_context_shape(&context, &allowed)
            .expect("Task33C4C8 capture validation used inventory");
        (context, table, associations, used)
    }

    #[test]
    fn task33c4c8_capture_row_postvalidation_is_fail_closed() {
        let (context, table, associations, used) = task33c4c8_capture_validation_fixture();
        assert_eq!(
            validate_capture_rows(&context, &table, &used, &associations),
            Ok(())
        );

        let mut missing = table.clone();
        missing
            .rows
            .remove(&SourceNestedFraenkelCaptureGraphCaptureId::new(0));
        let mut extra = table.clone();
        extra.rows.insert(
            SourceNestedFraenkelCaptureGraphCaptureId::new(2),
            SourceNestedFraenkelCaptureCoreVariable {
                capture: SourceNestedFraenkelCaptureGraphCaptureId::new(2),
                generator: mizar_checker::source_formula_composition::SourceNestedFraenkelCaptureGraphGeneratorId::new(3),
                resolver_binding: FraenkelGeneratorVariableBindingId::new(3),
                core_var: CoreVarId::new(12),
            },
        );
        let mut reordered = table.clone();
        reordered
            .rows
            .get_mut(&SourceNestedFraenkelCaptureGraphCaptureId::new(0))
            .expect("capture row")
            .core_var = CoreVarId::new(11);
        reordered
            .rows
            .get_mut(&SourceNestedFraenkelCaptureGraphCaptureId::new(1))
            .expect("capture row")
            .core_var = CoreVarId::new(10);
        let mut mismatched = table.clone();
        mismatched
            .rows
            .get_mut(&SourceNestedFraenkelCaptureGraphCaptureId::new(1))
            .expect("capture row")
            .resolver_binding = FraenkelGeneratorVariableBindingId::new(1);
        for invalid in [missing, extra, reordered, mismatched] {
            assert_eq!(
                validate_capture_rows(&context, &invalid, &used, &associations),
                Err(SourceNestedFraenkelCaptureCoreContextError::InvalidCaptureAssociation)
            );
        }

        let mut duplicate_var = table.clone();
        duplicate_var
            .rows
            .get_mut(&SourceNestedFraenkelCaptureGraphCaptureId::new(1))
            .expect("capture row")
            .core_var = CoreVarId::new(10);
        assert_eq!(
            validate_capture_rows(&context, &duplicate_var, &used, &associations),
            Err(
                SourceNestedFraenkelCaptureCoreContextError::CoreVariableCollision {
                    var: CoreVarId::new(10)
                }
            )
        );

        let mut bad_provenance = context.clone();
        bad_provenance
            .binder_sources
            .by_var
            .get_mut(&CoreVarId::new(10))
            .expect("capture binder")
            .source
            .provenance
            .clear();
        assert_eq!(
            validate_capture_rows(&bad_provenance, &table, &used, &associations),
            Err(SourceNestedFraenkelCaptureCoreContextError::InvalidCaptureAssociation)
        );

        let mut stale_role = context;
        stale_role
            .binder_context
            .variable_roles
            .insert(CoreVarId::new(11), CoreVarRole::new("stale-capture-role"));
        assert_eq!(
            validate_capture_rows(&stale_role, &table, &used, &associations),
            Err(SourceNestedFraenkelCaptureCoreContextError::InvalidCaptureAssociation)
        );
    }

    #[test]
    fn task33c4c8_allocator_is_zero_based_checked_and_deterministic() {
        assert_eq!(
            allocate_capture_core_vars(&BTreeSet::new(), 2).expect("empty allocation"),
            [CoreVarId::new(0), CoreVarId::new(1)]
        );
        let used = BTreeSet::from([CoreVarId::new(2), CoreVarId::new(9)]);
        assert_eq!(
            allocate_capture_core_vars(&used, 2).expect("populated allocation"),
            [CoreVarId::new(10), CoreVarId::new(11)]
        );
        assert_eq!(
            allocate_capture_core_vars(&BTreeSet::from([CoreVarId::new(usize::MAX)]), 1),
            Err(SourceNestedFraenkelCaptureCoreContextError::CoreVariableAllocationOverflow)
        );
        assert_eq!(
            allocate_capture_core_vars(&BTreeSet::from([CoreVarId::new(usize::MAX - 1)]), 2),
            Err(SourceNestedFraenkelCaptureCoreContextError::CoreVariableAllocationOverflow)
        );
    }

    #[test]
    fn source_binding_core_context_builds_exact_checker_order_and_metadata() {
        let context =
            prepare_core_context(CoreContextInput::new(summary())).expect("empty Core context");
        let first = SourceBindingCoreContextProducer::build(context, source_binding_env())
            .expect("source binding Core handoff");
        let second = SourceBindingCoreContextProducer::build(
            prepare_core_context(CoreContextInput::new(summary())).expect("empty Core context"),
            source_binding_env(),
        )
        .expect("deterministic source binding Core handoff");

        assert_eq!(first, second);
        assert_eq!(
            first.debug_text(),
            "source-binding-core-context-v1|module=pkg.main|bindings=2|variables=0:0,1:1"
        );
        let rows = first.variables().iter().collect::<Vec<_>>();
        let [(first_binding, first_row), (second_binding, second_row)] = rows.as_slice() else {
            panic!("two source binding rows expected");
        };
        assert_eq!(*first_binding, BindingId::new(0));
        assert_eq!(*second_binding, BindingId::new(1));
        assert_eq!(first_row.binding(), BindingId::new(0));
        assert_eq!(second_row.binding(), BindingId::new(1));
        assert_eq!(first_row.core_var(), CoreVarId::new(0));
        assert_eq!(second_row.core_var(), CoreVarId::new(1));
        assert_eq!(first.variables().get(BindingId::new(0)), Some(*first_row));
        assert_eq!(first.variables().get(BindingId::new(1)), Some(*second_row));

        let context = first.context();
        for (binding, row, declaration_range, role) in [
            (
                BindingId::new(0),
                *first_row,
                range(10, 11),
                SOURCE_BINDING_CORE_RESERVED_ROLE,
            ),
            (
                BindingId::new(1),
                *second_row,
                range(20, 21),
                SOURCE_BINDING_CORE_RESERVED_ROLE,
            ),
        ] {
            let var = row.core_var();
            let key = source_binding_provenance_key(binding);
            let expected_provenance =
                CoreProvenance::new(CoreProvenancePhase::Checker, key.clone());
            assert_eq!(
                context.binder_context().variable_classes.get(&var),
                Some(&NormalizedVarClass::Free)
            );
            assert_eq!(
                context.binder_context().variable_sorts.get(&var),
                Some(&NormalizedVarSort::Term)
            );
            assert_eq!(
                context.binder_context().variable_roles.get(&var),
                Some(&CoreVarRole::new(role))
            );
            assert_eq!(context.binder_type_facts().get(&var), Some(&Vec::new()));
            let record = context
                .binder_sources()
                .get(var)
                .expect("source binding binder source");
            assert_eq!(
                record.source,
                CoreSourceRef::direct(declaration_range).with_provenance(vec![expected_provenance])
            );
            assert_eq!(record.provenance, CheckerOwnedProvenance::checker(key));
        }

        let reserved = first
            .binding_env()
            .bindings()
            .get(BindingId::new(0))
            .expect("reserved binding");
        assert_eq!(reserved.kind, BindingKind::ReservedVariable);
        assert_eq!(reserved.status, BindingStatus::Reserved);
        assert_eq!(reserved.owner_context, BindingContextId::new(0));
        let second_reserved = first
            .binding_env()
            .bindings()
            .get(BindingId::new(1))
            .expect("second reserved binding");
        assert_eq!(second_reserved.kind, BindingKind::ReservedVariable);
        assert_eq!(second_reserved.status, BindingStatus::Reserved);
        assert_eq!(second_reserved.owner_context, BindingContextId::new(0));
        assert_eq!(
            second_reserved.identity,
            BinderIdentity::ReservedVariable {
                spelling: "y".to_owned(),
                declaration_range: range(20, 21),
            }
        );
        let module_context = first
            .binding_env()
            .contexts()
            .get(BindingContextId::new(0))
            .expect("module context");
        assert_eq!(
            module_context.bindings,
            vec![BindingId::new(0), BindingId::new(1)]
        );
        assert_eq!(module_context.visible_bindings, module_context.bindings);
    }

    #[test]
    fn source_binding_core_context_allocates_above_complete_used_inventory() {
        let handoff = SourceBindingCoreContextProducer::build(
            task33c4c8_context_with_complete_used_inventory(),
            source_binding_env(),
        )
        .expect("source binding Core handoff above existing variables");
        let rows = handoff
            .variables()
            .iter()
            .map(|(binding, row)| (binding, row.core_var()))
            .collect::<Vec<_>>();
        assert_eq!(
            rows,
            vec![
                (BindingId::new(0), CoreVarId::new(10)),
                (BindingId::new(1), CoreVarId::new(11))
            ]
        );
        assert!(
            handoff
                .context()
                .binder_context()
                .free_variables
                .is_superset(&BTreeSet::from([
                    CoreVarId::new(2),
                    CoreVarId::new(9),
                    CoreVarId::new(10),
                    CoreVarId::new(11),
                ]))
        );
        for (binding, var, role) in [
            (
                BindingId::new(0),
                CoreVarId::new(10),
                SOURCE_BINDING_CORE_RESERVED_ROLE,
            ),
            (
                BindingId::new(1),
                CoreVarId::new(11),
                SOURCE_BINDING_CORE_RESERVED_ROLE,
            ),
        ] {
            assert_eq!(
                handoff.context().binder_sources().get(var).unwrap().var,
                var
            );
            assert_eq!(
                handoff.context().binder_context().variable_roles.get(&var),
                Some(&CoreVarRole::new(role))
            );
            assert_eq!(handoff.variables().get(binding).unwrap().core_var(), var);
        }
    }

    #[test]
    fn source_binding_core_context_postvalidation_rejects_malformed_association() {
        let context =
            prepare_core_context(CoreContextInput::new(summary())).expect("empty Core context");
        let baseline = SourceBindingCoreContextProducer::build(context, source_binding_env())
            .expect("source binding Core handoff");

        let mut missing = baseline.clone();
        missing.variables.rows.remove(0);
        assert_eq!(
            missing.validate(),
            Err(SourceBindingCoreContextError::InvalidCoreContext)
        );

        let mut extra_orphan = baseline.clone();
        extra_orphan.variables.rows.push((
            BindingId::new(99),
            SourceBindingCoreVariable {
                binding: BindingId::new(99),
                core_var: CoreVarId::new(99),
            },
        ));
        assert_eq!(
            extra_orphan.validate(),
            Err(SourceBindingCoreContextError::InvalidBindingAssociation)
        );

        let mut duplicate_core_var = baseline.clone();
        duplicate_core_var.variables.rows[1].1.core_var = CoreVarId::new(0);
        duplicate_core_var
            .context
            .binder_context
            .variable_roles
            .insert(CoreVarId::new(1), CoreVarRole::new("existing-term"));
        assert_eq!(
            duplicate_core_var.validate(),
            Err(SourceBindingCoreContextError::CoreVariableCollision {
                var: CoreVarId::new(0)
            })
        );

        let mut reordered = baseline.clone();
        reordered.variables.rows.swap(0, 1);
        assert_eq!(
            reordered.validate(),
            Err(SourceBindingCoreContextError::InvalidBindingAssociation)
        );

        let mut stale = baseline.clone();
        stale.variables.rows[0].1.binding = BindingId::new(1);
        assert_eq!(
            stale.validate(),
            Err(SourceBindingCoreContextError::InvalidBindingAssociation)
        );

        let mut mismatched_key = baseline.clone();
        mismatched_key.variables.rows[0].0 = BindingId::new(1);
        assert_eq!(
            mismatched_key.validate(),
            Err(SourceBindingCoreContextError::InvalidBindingAssociation)
        );

        let mut wrong_role = baseline.clone();
        wrong_role
            .context
            .binder_context
            .variable_roles
            .insert(CoreVarId::new(0), CoreVarRole::new("wrong-role"));
        assert_eq!(
            wrong_role.validate(),
            Err(SourceBindingCoreContextError::InvalidBindingAssociation)
        );

        let mut wrong_class = baseline.clone();
        wrong_class
            .context
            .binder_context
            .variable_classes
            .insert(CoreVarId::new(0), NormalizedVarClass::Schematic);
        assert_eq!(
            wrong_class.validate(),
            Err(SourceBindingCoreContextError::InvalidBindingAssociation)
        );

        let mut wrong_sort = baseline.clone();
        wrong_sort
            .context
            .binder_context
            .variable_sorts
            .insert(CoreVarId::new(0), NormalizedVarSort::Formula);
        assert_eq!(
            wrong_sort.validate(),
            Err(SourceBindingCoreContextError::InvalidBindingAssociation)
        );

        let mut wrong_range = baseline.clone();
        wrong_range
            .context
            .binder_sources
            .by_var
            .get_mut(&CoreVarId::new(0))
            .expect("reserved binder source")
            .source
            .anchor = CoreSourceAnchor::SourceRange(range(11, 12));
        assert_eq!(
            wrong_range.validate(),
            Err(SourceBindingCoreContextError::InvalidBindingAssociation)
        );

        let mut wrong_provenance = baseline.clone();
        wrong_provenance
            .context
            .binder_sources
            .by_var
            .get_mut(&CoreVarId::new(0))
            .expect("reserved binder source")
            .provenance = CheckerOwnedProvenance::checker("wrong-source-binding-provenance");
        assert_eq!(
            wrong_provenance.validate(),
            Err(SourceBindingCoreContextError::InvalidBindingAssociation)
        );

        let mut nonempty_facts = baseline;
        nonempty_facts
            .context
            .binder_type_facts
            .insert(CoreVarId::new(0), vec![TypeFactId::new(0)]);
        assert_eq!(
            nonempty_facts.validate(),
            Err(SourceBindingCoreContextError::InvalidBindingAssociation)
        );
    }

    #[test]
    fn source_binding_core_context_rejects_mismatch_unsupported_and_overflow() {
        let source_mismatch = source_binding_env_with_one(
            alternate_source_id(),
            module_id(),
            BindingKind::ReservedVariable,
            BinderIdentity::ReservedVariable {
                spelling: "x".to_owned(),
                declaration_range: SourceRange {
                    source_id: alternate_source_id(),
                    start: 10,
                    end: 11,
                },
            },
            BindingStatus::Reserved,
        );
        assert_eq!(
            SourceBindingCoreContextProducer::build(
                prepare_core_context(CoreContextInput::new(summary())).expect("Core context"),
                source_mismatch,
            ),
            Err(SourceBindingCoreContextError::EnvironmentMismatch)
        );

        let module_mismatch = source_binding_env_with_one(
            source_id(),
            external_module_id(),
            BindingKind::ReservedVariable,
            BinderIdentity::ReservedVariable {
                spelling: "x".to_owned(),
                declaration_range: range(10, 11),
            },
            BindingStatus::Reserved,
        );
        assert_eq!(
            SourceBindingCoreContextProducer::build(
                prepare_core_context(CoreContextInput::new(summary())).expect("Core context"),
                module_mismatch,
            ),
            Err(SourceBindingCoreContextError::EnvironmentMismatch)
        );

        assert_eq!(
            SourceBindingCoreContextProducer::build(
                prepare_core_context(CoreContextInput::new(summary())).expect("Core context"),
                empty_source_binding_env(),
            ),
            Err(SourceBindingCoreContextError::InvalidBindingEnvironment)
        );

        let mut diagnostics = BindingDiagnosticTable::new();
        diagnostics.insert(BindingDiagnosticDraft {
            source_range: Some(range(1, 2)),
            class: BindingDiagnosticClass::UnsupportedSourceShape,
            severity: BindingDiagnosticSeverity::Error,
            message_key: "source-binding-test-diagnostic".to_owned(),
            recovery: BindingDiagnosticRecovery::Degraded,
        });
        assert_eq!(
            SourceBindingCoreContextProducer::build(
                prepare_core_context(CoreContextInput::new(summary())).expect("Core context"),
                source_binding_env_with_options(
                    BindingContextRecovery::Normal,
                    source_binding_reserved_draft(),
                    source_binding_second_reserved_draft(),
                    diagnostics,
                ),
            ),
            Err(SourceBindingCoreContextError::InvalidBindingEnvironment)
        );

        assert_eq!(
            SourceBindingCoreContextProducer::build(
                prepare_core_context(CoreContextInput::new(summary())).expect("Core context"),
                source_binding_env_with_options(
                    BindingContextRecovery::Recovered,
                    source_binding_reserved_draft(),
                    source_binding_second_reserved_draft(),
                    BindingDiagnosticTable::new(),
                ),
            ),
            Err(SourceBindingCoreContextError::InvalidBindingEnvironment)
        );
        assert_eq!(
            SourceBindingCoreContextProducer::build(
                prepare_core_context(CoreContextInput::new(summary())).expect("Core context"),
                source_binding_env_with_options(
                    BindingContextRecovery::Degraded,
                    source_binding_reserved_draft(),
                    source_binding_second_reserved_draft(),
                    BindingDiagnosticTable::new(),
                ),
            ),
            Err(SourceBindingCoreContextError::InvalidBindingEnvironment)
        );

        let mut recovered_binding = source_binding_reserved_draft();
        recovered_binding.recovery = BindingRecoveryState::Recovered;
        assert_eq!(
            SourceBindingCoreContextProducer::build(
                prepare_core_context(CoreContextInput::new(summary())).expect("Core context"),
                source_binding_env_with_options(
                    BindingContextRecovery::Normal,
                    recovered_binding,
                    source_binding_second_reserved_draft(),
                    BindingDiagnosticTable::new(),
                ),
            ),
            Err(SourceBindingCoreContextError::InvalidBindingEnvironment)
        );

        let mut degraded_binding = source_binding_reserved_draft();
        degraded_binding.recovery = BindingRecoveryState::Degraded;
        assert_eq!(
            SourceBindingCoreContextProducer::build(
                prepare_core_context(CoreContextInput::new(summary())).expect("Core context"),
                source_binding_env_with_options(
                    BindingContextRecovery::Normal,
                    degraded_binding,
                    source_binding_second_reserved_draft(),
                    BindingDiagnosticTable::new(),
                ),
            ),
            Err(SourceBindingCoreContextError::InvalidBindingEnvironment)
        );

        let mut degraded_status = source_binding_reserved_draft();
        degraded_status.status = BindingStatus::Degraded;
        assert_eq!(
            SourceBindingCoreContextProducer::build(
                prepare_core_context(CoreContextInput::new(summary())).expect("Core context"),
                source_binding_env_with_options(
                    BindingContextRecovery::Normal,
                    degraded_status,
                    source_binding_second_reserved_draft(),
                    BindingDiagnosticTable::new(),
                ),
            ),
            Err(SourceBindingCoreContextError::InvalidBindingEnvironment)
        );

        let mut captured_binding = source_binding_reserved_draft();
        captured_binding.captured =
            CapturedFreeVariables::new(vec![BinderIdentity::ReservedVariable {
                spelling: "captured".to_owned(),
                declaration_range: range(30, 31),
            }]);
        assert_eq!(
            SourceBindingCoreContextProducer::build(
                prepare_core_context(CoreContextInput::new(summary())).expect("Core context"),
                source_binding_env_with_options(
                    BindingContextRecovery::Normal,
                    captured_binding,
                    source_binding_second_reserved_draft(),
                    BindingDiagnosticTable::new(),
                ),
            ),
            Err(SourceBindingCoreContextError::InvalidBindingEnvironment)
        );

        let mut status_mismatch = source_binding_reserved_draft();
        status_mismatch.status = BindingStatus::Active;
        assert_eq!(
            SourceBindingCoreContextProducer::build(
                prepare_core_context(CoreContextInput::new(summary())).expect("Core context"),
                source_binding_env_with_options(
                    BindingContextRecovery::Normal,
                    status_mismatch,
                    source_binding_second_reserved_draft(),
                    BindingDiagnosticTable::new(),
                ),
            ),
            Err(SourceBindingCoreContextError::InvalidBindingEnvironment)
        );

        let mut omitted_status = source_binding_reserved_draft();
        omitted_status.status = BindingStatus::Omitted;
        assert_eq!(
            SourceBindingCoreContextProducer::build(
                prepare_core_context(CoreContextInput::new(summary())).expect("Core context"),
                source_binding_env_with_options(
                    BindingContextRecovery::Normal,
                    omitted_status,
                    source_binding_second_reserved_draft(),
                    BindingDiagnosticTable::new(),
                ),
            ),
            Err(SourceBindingCoreContextError::InvalidBindingEnvironment)
        );

        let mut binding_diagnostics = BindingDiagnosticTable::new();
        let binding_diagnostic = binding_diagnostics.insert(BindingDiagnosticDraft {
            source_range: Some(range(10, 11)),
            class: BindingDiagnosticClass::UnsupportedSourceShape,
            severity: BindingDiagnosticSeverity::Error,
            message_key: "source-binding-test-row-diagnostic".to_owned(),
            recovery: BindingDiagnosticRecovery::Degraded,
        });
        let mut diagnostic_binding = source_binding_reserved_draft();
        diagnostic_binding.diagnostics.push(binding_diagnostic);
        assert_eq!(
            SourceBindingCoreContextProducer::build(
                prepare_core_context(CoreContextInput::new(summary())).expect("Core context"),
                source_binding_env_with_options(
                    BindingContextRecovery::Normal,
                    diagnostic_binding,
                    source_binding_second_reserved_draft(),
                    binding_diagnostics,
                ),
            ),
            Err(SourceBindingCoreContextError::InvalidBindingEnvironment)
        );

        let unsupported = source_binding_env_with_one(
            source_id(),
            module_id(),
            BindingKind::LocalAbbreviation,
            BinderIdentity::ResolverLocal {
                scope: LocalTermScope::new(vec![0]),
                ordinal: 0,
                declaration_range: range(10, 11),
            },
            BindingStatus::Active,
        );
        assert_eq!(
            SourceBindingCoreContextProducer::build(
                prepare_core_context(CoreContextInput::new(summary())).expect("Core context"),
                unsupported,
            ),
            Err(SourceBindingCoreContextError::InvalidBindingEnvironment)
        );

        let mut invalid_existing_input = CoreContextInput::new(summary());
        invalid_existing_input
            .variable_seeds
            .push(CoreVariableSeed::new(
                CoreVarId::new(7),
                NormalizedVarClass::Free,
                "existing-term",
                NormalizedVarSort::Term,
                provenance("checker:source-binding:invalid-existing"),
            ));
        let mut invalid_existing_context =
            prepare_core_context(invalid_existing_input).expect("invalid-existing context");
        invalid_existing_context
            .binder_context
            .variable_sorts
            .remove(&CoreVarId::new(7));
        assert_eq!(
            SourceBindingCoreContextProducer::build(invalid_existing_context, source_binding_env()),
            Err(SourceBindingCoreContextError::InvalidCoreContext)
        );

        let mut unauthenticated_role_input = CoreContextInput::new(summary());
        unauthenticated_role_input
            .variable_seeds
            .push(CoreVariableSeed::new(
                CoreVarId::new(7),
                NormalizedVarClass::Free,
                SOURCE_BINDING_CORE_RESERVED_ROLE,
                NormalizedVarSort::Term,
                provenance("checker:source-binding:unauthenticated-role"),
            ));
        let unauthenticated_role_context =
            prepare_core_context(unauthenticated_role_input).expect("reserved-role context");
        assert_eq!(
            SourceBindingCoreContextProducer::build(
                unauthenticated_role_context,
                source_binding_env()
            ),
            Err(SourceBindingCoreContextError::InvalidCoreContext)
        );

        let mut overflow_input = CoreContextInput::new(summary());
        overflow_input.variable_seeds.push(CoreVariableSeed::new(
            CoreVarId::new(usize::MAX),
            NormalizedVarClass::Free,
            "existing-term",
            NormalizedVarSort::Term,
            provenance("checker:source-binding:overflow"),
        ));
        let overflow_context =
            prepare_core_context(overflow_input).expect("overflow source binding context");
        assert_eq!(
            SourceBindingCoreContextProducer::build(overflow_context, source_binding_env()),
            Err(SourceBindingCoreContextError::CoreVariableAllocationOverflow)
        );
    }
}
