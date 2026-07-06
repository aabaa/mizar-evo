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
        CoreDefinitionTable, CoreDiagnostic, CoreDiagnosticClass, CoreDiagnosticId,
        CoreDiagnosticMessageKey, CoreDiagnosticRecovery, CoreDiagnosticSeverity,
        CoreDiagnosticTable, CoreFormula, CoreFormulaId, CoreFormulaKind, CoreFormulaTable,
        CoreItem, CoreItemId, CoreItemKind, CoreItemStatus, CoreItemTable, CoreJustification,
        CoreLabelRef, CoreNodeRef, CorePlace, CoreProof, CoreProofId, CoreProofNode,
        CoreProofNodeId, CoreProofNodeKind, CoreProofNodeTable, CoreProofStatus, CoreProofTable,
        CoreProvenance, CoreProvenanceKey, CoreProvenancePhase, CoreSourceAnchor, CoreSourceMap,
        CoreSourceRef, CoreTerm, CoreTermId, CoreTermKind, CoreTermTable, CoreTypePredicate,
        CoreVarId, CoreVarRole, CoreVisibility, DefinitionBody, DefinitionBranchBody,
        ExpansionPolicy, GeneratedOrigin, GeneratedOriginId, GeneratedOriginKey,
        GeneratedOriginKind, GeneratedOriginTable, GhostEffectKey, GuardedDefinitionBranch,
        LocalProofOrProgramPath, NormalizedSemanticOrigin, ObligationSeed, ObligationSeedId,
        ObligationSeedKind, ObligationSeedStatus, ObligationSeedTable, ProofBranchKind,
    },
};
use mizar_checker::{
    cluster_trace::ClusterFactId,
    overload_resolution::QuaPathKey,
    resolved_typed_ast::{
        CoercionInsertionId, OverloadResolutionId, ResolvedNodeRecovery, ResolvedTypedAst,
        ResolvedTypedDiagnosticId, ResolvedTypedDiagnosticSeverity, ResolvedTypedNodeId,
        ResolvedTypedNodeKind,
    },
    typed_ast::{
        InitialObligationId, InitialObligationKind, NormalizedTypeId, Polarity, TypeDiagnosticId,
        TypeFactId,
    },
};
use mizar_resolve::resolved_ast::{ModuleId, SymbolId};
use mizar_session::{SourceId, SourceRange};
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
    let mut binder_guards = Vec::new();
    let mut assumptions = Vec::new();
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
        reconsidered_binders,
        carried_obligations,
        missing_evidence,
    })
}

fn validate_type_and_fact_input(
    context: &CoreContext,
    input: &TypeAndFactLoweringInput,
) -> TypeAndFactResult<()> {
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
    pub terms: Vec<CoreTermSeed>,
    pub formulas: Vec<CoreFormulaSeed>,
    pub failed_sites: Vec<FailedSemanticSiteSeed>,
}

impl TermAndFormulaLoweringInput {
    pub const fn new(owner: CoreItemId) -> Self {
        Self {
            owner,
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
                sethood_evidence,
                membership_obligation,
                missing_sethood_obligation,
            } => {
                validate_generated_functor(&key, &origin_functor, &functor)?;
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

fn validate_term_and_formula_input(
    context: &CoreContext,
    input: &TermAndFormulaLoweringInput,
) -> TermAndFormulaResult<()> {
    for seed in &input.terms {
        validate_checker_owned_provenance("term seed", seed.provenance.as_slice())?;
        validate_term_seed_kind(context, &seed.kind)?;
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
            item: seed.owner,
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
}
