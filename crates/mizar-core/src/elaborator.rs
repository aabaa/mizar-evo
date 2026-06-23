//! Core elaboration context preparation.
//!
//! Implements the task-8, task-9, and task-10 elaboration slices specified in
//! [elaborator.md](../../../../doc/design/mizar-core/en/elaborator.md).

use crate::{
    binder_normalization::{BinderContext, NormalizedVarClass, NormalizedVarSort},
    core_ir::{
        CoreBinder, CoreDiagnostic, CoreDiagnosticClass, CoreDiagnosticId,
        CoreDiagnosticMessageKey, CoreDiagnosticRecovery, CoreDiagnosticSeverity,
        CoreDiagnosticTable, CoreFormula, CoreFormulaId, CoreFormulaKind, CoreFormulaTable,
        CoreItem, CoreItemId, CoreItemKind, CoreItemStatus, CoreItemTable, CoreNodeRef,
        CoreProvenance, CoreProvenanceKey, CoreProvenancePhase, CoreSourceAnchor, CoreSourceMap,
        CoreSourceRef, CoreTerm, CoreTermId, CoreTermKind, CoreTermTable, CoreTypePredicate,
        CoreVarId, CoreVarRole, CoreVisibility, GeneratedOrigin, GeneratedOriginId,
        GeneratedOriginKey, GeneratedOriginKind, GeneratedOriginTable, LocalProofOrProgramPath,
        NormalizedSemanticOrigin, ObligationSeed, ObligationSeedId, ObligationSeedKind,
        ObligationSeedStatus, ObligationSeedTable,
    },
};
use mizar_checker::{
    cluster_trace::ClusterFactId,
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
pub struct ViewExplanationSeed {
    pub kind: ViewExplanationKind,
    pub inserted_view: Option<CoercionInsertionId>,
    pub target_type: Option<NormalizedTypeId>,
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
    pub evidence_facts: Vec<TypeFactId>,
    pub source: CoreSourceRef,
    pub provenance: Vec<CoreProvenance>,
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
                self.push_view_explanation(explanation);
                Ok(lowered)
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
            evidence_facts: seed.evidence_facts,
            source: source_with_provenance(seed.source, &seed.provenance),
            provenance: seed.provenance.as_slice().to_vec(),
        });
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
                evidence_facts: vec![TypeFactId::new(2), TypeFactId::new(1), TypeFactId::new(1)],
                source: direct(20, 23),
                provenance: provenance("checker:view:source-qua"),
            },
            ViewExplanationSeed {
                kind: ViewExplanationKind::InsertedView,
                inserted_view: Some(CoercionInsertionId::new(0)),
                target_type: Some(NormalizedTypeId::new(2)),
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
        assert_eq!(
            output.view_explanations[1].evidence_facts,
            vec![TypeFactId::new(5)]
        );
        assert_step3_delta_valid(&context, &output);
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
