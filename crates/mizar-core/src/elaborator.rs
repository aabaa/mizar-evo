//! Core elaboration context preparation.
//!
//! Implements the task-8 context-preparation slice specified in
//! [elaborator.md](../../../../doc/design/mizar-core/en/elaborator.md).

use crate::{
    binder_normalization::{BinderContext, NormalizedVarClass, NormalizedVarSort},
    core_ir::{
        CoreDiagnostic, CoreDiagnosticClass, CoreDiagnosticId, CoreDiagnosticMessageKey,
        CoreDiagnosticRecovery, CoreDiagnosticSeverity, CoreDiagnosticTable, CoreItem, CoreItemId,
        CoreItemKind, CoreItemStatus, CoreItemTable, CoreNodeRef, CoreProvenance,
        CoreProvenanceKey, CoreProvenancePhase, CoreSourceAnchor, CoreSourceMap, CoreSourceRef,
        CoreVarId, CoreVarRole, CoreVisibility, GeneratedOrigin, GeneratedOriginId,
        GeneratedOriginKey, GeneratedOriginKind, GeneratedOriginTable,
    },
};
use mizar_checker::resolved_typed_ast::{
    OverloadResolutionId, ResolvedNodeRecovery, ResolvedTypedAst, ResolvedTypedDiagnosticId,
    ResolvedTypedDiagnosticSeverity, ResolvedTypedNodeId, ResolvedTypedNodeKind,
};
use mizar_checker::typed_ast::TypeFactId;
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

#[cfg(test)]
mod tests {
    use super::*;
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
