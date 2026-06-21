//! Resolved AST data shapes.

mod snapshot;
mod validation;

use mizar_session::{
    GeneratedSpanAnchor, ModulePath, PackageId, SourceAnchor, SourceId, SourceRange,
};
use mizar_syntax::SurfaceNodeKind;
use std::cmp::Ordering;
use std::error::Error;
use std::fmt;

use validation::{range_key, recovery_from_origin, validate_nodes, validate_resolved_ast};

/// Canonical, alias-independent module identity consumed by resolver outputs.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModuleId {
    package: PackageId,
    path: ModulePath,
}

impl ModuleId {
    /// Creates a canonical module identity from package and module path parts.
    pub const fn new(package: PackageId, path: ModulePath) -> Self {
        Self { package, path }
    }

    /// Returns the package component.
    pub const fn package(&self) -> &PackageId {
        &self.package
    }

    /// Returns the module-path component.
    pub const fn path(&self) -> &ModulePath {
        &self.path
    }
}

impl Ord for ModuleId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.package
            .as_str()
            .cmp(other.package.as_str())
            .then_with(|| self.path.as_str().cmp(other.path.as_str()))
    }
}

impl PartialOrd for ModuleId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Stable local symbol identity within a module.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocalSymbolId(String);

impl LocalSymbolId {
    /// Creates a local symbol identity.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the serialized local symbol component.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Fully qualified symbol name projection used for deterministic ordering.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FullyQualifiedName(String);

impl FullyQualifiedName {
    /// Creates a fully qualified name projection.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the fully qualified spelling.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Stable fully qualified symbol identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SymbolId {
    module: ModuleId,
    local: LocalSymbolId,
    fqn: FullyQualifiedName,
}

impl SymbolId {
    /// Creates a stable symbol identity.
    pub const fn new(module: ModuleId, local: LocalSymbolId, fqn: FullyQualifiedName) -> Self {
        Self { module, local, fqn }
    }

    /// Returns the declaring module.
    pub const fn module(&self) -> &ModuleId {
        &self.module
    }

    /// Returns the local symbol component.
    pub const fn local(&self) -> &LocalSymbolId {
        &self.local
    }

    /// Returns the fully qualified name projection.
    pub const fn fqn(&self) -> &FullyQualifiedName {
        &self.fqn
    }
}

impl Ord for SymbolId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.fqn
            .cmp(other.fqn())
            .then_with(|| self.module.cmp(other.module()))
            .then_with(|| self.local.cmp(other.local()))
    }
}

impl PartialOrd for SymbolId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Stable builtin identity for resolver-visible builtins.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BuiltinId(String);

impl BuiltinId {
    /// Creates a builtin identity.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the builtin identity spelling.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Stable label origin path used by later proof and VC phases.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LabelOriginPath(String);

impl LabelOriginPath {
    /// Creates a label origin path.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the label origin path.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Stable id for an arena node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResolvedNodeId(usize);

impl ResolvedNodeId {
    const fn new(index: usize) -> Self {
        Self(index)
    }

    /// Returns the zero-based arena index.
    pub const fn index(self) -> usize {
        self.0
    }
}

/// Stable id for a name-reference table entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NameRefId(usize);

impl NameRefId {
    const fn new(index: usize) -> Self {
        Self(index)
    }

    /// Returns the zero-based table index.
    pub const fn index(self) -> usize {
        self.0
    }
}

/// Stable id for a label-reference table entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LabelRefId(usize);

impl LabelRefId {
    const fn new(index: usize) -> Self {
        Self(index)
    }

    /// Returns the zero-based table index.
    pub const fn index(self) -> usize {
        self.0
    }
}

/// Stable id for an import entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResolvedImportId(usize);

impl ResolvedImportId {
    const fn new(index: usize) -> Self {
        Self(index)
    }

    /// Returns the zero-based table index.
    pub const fn index(self) -> usize {
        self.0
    }
}

/// Stable id for an export entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResolvedExportId(usize);

impl ResolvedExportId {
    const fn new(index: usize) -> Self {
        Self(index)
    }

    /// Returns the zero-based table index.
    pub const fn index(self) -> usize {
        self.0
    }
}

/// Normalized provenance attached to resolved nodes and table entries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticOrigin {
    source_id: SourceId,
    module_id: ModuleId,
    anchor: SourceAnchor,
    structural_path: Vec<u32>,
    import_edge: Option<ResolvedImportId>,
    recovered: bool,
}

impl SemanticOrigin {
    /// Creates normalized provenance for a source-shaped semantic object.
    pub fn new(
        source_id: SourceId,
        module_id: ModuleId,
        anchor: SourceAnchor,
        structural_path: Vec<u32>,
    ) -> Self {
        Self {
            source_id,
            module_id,
            anchor,
            structural_path,
            import_edge: None,
            recovered: false,
        }
    }

    /// Marks this origin as coming through an import edge.
    pub const fn with_import_edge(mut self, import_edge: ResolvedImportId) -> Self {
        self.import_edge = Some(import_edge);
        self
    }

    /// Marks this origin as recovered.
    pub const fn recovered(mut self) -> Self {
        self.recovered = true;
        self
    }

    /// Returns the source id.
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    /// Returns the module id.
    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    /// Returns the source or generated anchor.
    pub const fn anchor(&self) -> &SourceAnchor {
        &self.anchor
    }

    /// Returns the deterministic structural path.
    pub fn structural_path(&self) -> &[u32] {
        &self.structural_path
    }

    /// Returns the import edge provenance, if any.
    pub const fn import_edge(&self) -> Option<ResolvedImportId> {
        self.import_edge
    }

    /// Returns whether this origin came from recovered syntax.
    pub const fn is_recovered(&self) -> bool {
        self.recovered
    }
}

/// Recovery state of a resolved node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RecoveryState {
    /// The node was resolved from non-recovered surface syntax.
    Normal,
    /// The node is a preserved semantic shell for recovered syntax.
    Recovered,
}

/// Traversal-visible resolution state of a resolved node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum NodeResolutionState {
    /// The node does not carry a resolver outcome.
    NotApplicable,
    /// The node carries a resolved outcome.
    Resolved,
    /// The node carries an unresolved outcome.
    Unresolved,
    /// The node carries an ambiguous outcome.
    Ambiguous,
    /// The node carries a checker-deferred outcome.
    Deferred,
}

/// Stable key from a node to the table entry carrying detailed resolver data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum NodeReferenceKey {
    /// Key into `NameRefTable`.
    Name(NameRefId),
    /// Key into `LabelRefTable`.
    Label(LabelRefId),
    /// Key into `ResolvedImports` imports.
    Import(ResolvedImportId),
    /// Key into `ResolvedImports` exports.
    Export(ResolvedExportId),
}

/// Source-shaped node with resolver-owned metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedNode {
    kind: SurfaceNodeKind,
    children: Vec<ResolvedNodeId>,
    recovery: RecoveryState,
    resolution: NodeResolutionState,
    reference_key: Option<NodeReferenceKey>,
    origin: SemanticOrigin,
}

impl ResolvedNode {
    /// Creates a resolved node with no node-local resolver outcome.
    pub fn new(
        kind: SurfaceNodeKind,
        children: Vec<ResolvedNodeId>,
        origin: SemanticOrigin,
    ) -> Self {
        let recovery = recovery_from_origin(&origin);
        Self {
            kind,
            children,
            recovery,
            resolution: NodeResolutionState::NotApplicable,
            reference_key: None,
            origin,
        }
    }

    /// Marks the node as recovered.
    pub const fn with_recovery(mut self, recovery: RecoveryState) -> Self {
        self.recovery = recovery;
        self.origin.recovered = matches!(recovery, RecoveryState::Recovered);
        self
    }

    /// Attaches traversal-visible resolution state.
    pub const fn with_resolution(mut self, resolution: NodeResolutionState) -> Self {
        self.resolution = resolution;
        self
    }

    /// Attaches a stable key into a resolver table.
    pub const fn with_reference_key(mut self, reference_key: NodeReferenceKey) -> Self {
        self.reference_key = Some(reference_key);
        self
    }

    /// Returns the source-shaped node kind.
    pub const fn kind(&self) -> &SurfaceNodeKind {
        &self.kind
    }

    /// Returns child node ids in source-shaped order.
    pub fn children(&self) -> &[ResolvedNodeId] {
        &self.children
    }

    /// Returns the recovery state.
    pub const fn recovery(&self) -> RecoveryState {
        self.recovery
    }

    /// Returns the traversal-visible resolution state.
    pub const fn resolution(&self) -> NodeResolutionState {
        self.resolution
    }

    /// Returns the table key for node-local resolver details.
    pub const fn reference_key(&self) -> Option<NodeReferenceKey> {
        self.reference_key
    }

    /// Returns the semantic origin.
    pub const fn origin(&self) -> &SemanticOrigin {
        &self.origin
    }
}

/// Arena of resolved nodes with validated child links and acyclicity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedArena {
    root: ResolvedNodeId,
    nodes: Vec<ResolvedNode>,
}

impl ResolvedArena {
    /// Validates and creates an arena from resolved nodes.
    pub fn try_new(
        root: ResolvedNodeId,
        nodes: Vec<ResolvedNode>,
    ) -> Result<Self, ResolvedArenaError> {
        validate_nodes(&nodes)?;
        if root.index() >= nodes.len() {
            return Err(ResolvedArenaError::InvalidRoot { root });
        }
        Ok(Self { root, nodes })
    }

    /// Returns the root node id.
    pub const fn root(&self) -> ResolvedNodeId {
        self.root
    }

    /// Returns the node for an id.
    pub fn node(&self, id: ResolvedNodeId) -> Option<&ResolvedNode> {
        self.nodes.get(id.index())
    }

    /// Returns an iterator over ids and nodes in deterministic arena order.
    pub fn iter(&self) -> impl Iterator<Item = (ResolvedNodeId, &ResolvedNode)> {
        self.nodes
            .iter()
            .enumerate()
            .map(|(index, node)| (ResolvedNodeId::new(index), node))
    }

    /// Returns the number of nodes.
    pub const fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns whether the arena is empty.
    pub const fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

/// Builder that allocates deterministic arena ids in insertion order.
#[derive(Debug, Clone, Default)]
pub struct ResolvedArenaBuilder {
    nodes: Vec<ResolvedNode>,
}

impl ResolvedArenaBuilder {
    /// Creates an empty arena builder.
    pub const fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Pushes a node and returns its stable id.
    pub fn push(&mut self, node: ResolvedNode) -> Result<ResolvedNodeId, ResolvedArenaError> {
        let id = ResolvedNodeId::new(self.nodes.len());
        for child in node.children() {
            if child.index() >= self.nodes.len() {
                return Err(ResolvedArenaError::InvalidChild {
                    node: id,
                    child: *child,
                });
            }
        }
        self.nodes.push(node);
        Ok(id)
    }

    /// Validates and finishes the arena.
    pub fn finish(self, root: ResolvedNodeId) -> Result<ResolvedArena, ResolvedArenaError> {
        ResolvedArena::try_new(root, self.nodes)
    }
}

/// Arena validation error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ResolvedArenaError {
    /// The root node id does not exist in the arena.
    InvalidRoot {
        /// Invalid root id.
        root: ResolvedNodeId,
    },
    /// A child id does not exist in the arena.
    InvalidChild {
        /// Parent node containing the invalid child id.
        node: ResolvedNodeId,
        /// Invalid child id.
        child: ResolvedNodeId,
    },
    /// A cycle was found during arena validation.
    Cycle {
        /// Node participating in a cycle.
        node: ResolvedNodeId,
    },
}

impl fmt::Display for ResolvedArenaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRoot { root } => write!(
                formatter,
                "resolved arena root {} does not exist",
                root.index()
            ),
            Self::InvalidChild { node, child } => write!(
                formatter,
                "resolved arena node {} references missing child {}",
                node.index(),
                child.index()
            ),
            Self::Cycle { node } => {
                write!(
                    formatter,
                    "resolved arena has a cycle at node {}",
                    node.index()
                )
            }
        }
    }
}

impl Error for ResolvedArenaError {}

/// Reference use site shared by name and label tables.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceSite {
    node: ResolvedNodeId,
    range: SourceRange,
    spelling: String,
}

impl ReferenceSite {
    /// Creates a stable reference site.
    pub fn new(node: ResolvedNodeId, range: SourceRange, spelling: impl Into<String>) -> Self {
        Self {
            node,
            range,
            spelling: spelling.into(),
        }
    }

    /// Returns the node containing this reference.
    pub const fn node(&self) -> ResolvedNodeId {
        self.node
    }

    /// Returns the use-site range.
    pub const fn range(&self) -> SourceRange {
        self.range
    }

    /// Returns the source spelling at the use site.
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
}

/// A resolved symbol reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolRef {
    symbol: SymbolId,
    range: SourceRange,
    import: Option<ResolvedImportId>,
    spelling: Option<String>,
}

impl SymbolRef {
    /// Creates a symbol reference.
    pub const fn new(symbol: SymbolId, range: SourceRange) -> Self {
        Self {
            symbol,
            range,
            import: None,
            spelling: None,
        }
    }

    /// Attaches import provenance.
    pub const fn with_import(mut self, import: ResolvedImportId) -> Self {
        self.import = Some(import);
        self
    }

    /// Attaches source spelling for diagnostics.
    pub fn with_spelling(mut self, spelling: impl Into<String>) -> Self {
        self.spelling = Some(spelling.into());
        self
    }

    /// Returns the target symbol id.
    pub const fn symbol(&self) -> &SymbolId {
        &self.symbol
    }

    /// Returns the use-site range.
    pub const fn range(&self) -> SourceRange {
        self.range
    }

    /// Returns import provenance, if any.
    pub const fn import(&self) -> Option<ResolvedImportId> {
        self.import
    }

    /// Returns the source spelling, if one was recorded.
    pub fn spelling(&self) -> Option<&str> {
        self.spelling.as_deref()
    }
}

/// A resolver-visible builtin reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuiltinRef {
    builtin: BuiltinId,
    range: SourceRange,
    spelling: String,
}

impl BuiltinRef {
    /// Creates a builtin reference.
    pub fn new(builtin: BuiltinId, range: SourceRange, spelling: impl Into<String>) -> Self {
        Self {
            builtin,
            range,
            spelling: spelling.into(),
        }
    }

    /// Returns the builtin id.
    pub const fn builtin(&self) -> &BuiltinId {
        &self.builtin
    }

    /// Returns the use-site range.
    pub const fn range(&self) -> SourceRange {
        self.range
    }

    /// Returns the source spelling.
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
}

/// Checker-deferred selector decision for dotted syntax.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeferredSelectorRef {
    base: ResolvedNodeId,
    member: String,
    range: SourceRange,
}

impl DeferredSelectorRef {
    /// Creates a deferred selector reference.
    pub fn new(base: ResolvedNodeId, member: impl Into<String>, range: SourceRange) -> Self {
        Self {
            base,
            member: member.into(),
            range,
        }
    }

    /// Returns the base node id.
    pub const fn base(&self) -> ResolvedNodeId {
        self.base
    }

    /// Returns the deferred member spelling.
    pub fn member(&self) -> &str {
        &self.member
    }

    /// Returns the use-site range.
    pub const fn range(&self) -> SourceRange {
        self.range
    }
}

/// Lookup class for unresolved name references.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum NameLookupClass {
    /// Module lookup failed.
    Module,
    /// Namespace lookup failed.
    Namespace,
    /// Symbol lookup failed.
    Symbol,
    /// Builtin lookup failed.
    Builtin,
    /// Selector finalization failed.
    Selector,
}

/// Candidate for an ambiguous name reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameResolutionCandidate {
    symbol: SymbolId,
    range: SourceRange,
}

impl NameResolutionCandidate {
    /// Creates a name-resolution candidate.
    pub const fn new(symbol: SymbolId, range: SourceRange) -> Self {
        Self { symbol, range }
    }

    /// Returns the candidate symbol.
    pub const fn symbol(&self) -> &SymbolId {
        &self.symbol
    }

    /// Returns the candidate declaration range.
    pub const fn range(&self) -> SourceRange {
        self.range
    }
}

impl Ord for NameResolutionCandidate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.symbol
            .fqn()
            .cmp(other.symbol().fqn())
            .then_with(|| self.symbol.module().cmp(other.symbol().module()))
            .then_with(|| range_key(self.range).cmp(&range_key(other.range)))
            .then_with(|| self.symbol.local().cmp(other.symbol().local()))
    }
}

impl PartialOrd for NameResolutionCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Ambiguous name reference with deterministic candidate ordering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AmbiguousNameRef {
    spelling: String,
    range: SourceRange,
    candidates: Vec<NameResolutionCandidate>,
}

impl AmbiguousNameRef {
    /// Creates an ambiguous name reference and sorts candidates deterministically.
    pub fn new(
        spelling: impl Into<String>,
        range: SourceRange,
        mut candidates: Vec<NameResolutionCandidate>,
    ) -> Self {
        candidates.sort();
        Self {
            spelling: spelling.into(),
            range,
            candidates,
        }
    }

    /// Returns sorted candidates.
    pub fn candidates(&self) -> &[NameResolutionCandidate] {
        &self.candidates
    }

    /// Returns the ambiguous source spelling.
    pub fn spelling(&self) -> &str {
        &self.spelling
    }

    /// Returns the use-site range.
    pub const fn range(&self) -> SourceRange {
        self.range
    }
}

/// Unresolved name reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnresolvedNameRef {
    spelling: String,
    range: SourceRange,
    lookup: NameLookupClass,
}

impl UnresolvedNameRef {
    /// Creates an unresolved name reference.
    pub fn new(spelling: impl Into<String>, range: SourceRange, lookup: NameLookupClass) -> Self {
        Self {
            spelling: spelling.into(),
            range,
            lookup,
        }
    }

    /// Returns the lookup class that failed.
    pub const fn lookup(&self) -> NameLookupClass {
        self.lookup
    }

    /// Returns the unresolved source spelling.
    pub fn spelling(&self) -> &str {
        &self.spelling
    }

    /// Returns the use-site range.
    pub const fn range(&self) -> SourceRange {
        self.range
    }
}

/// Name-resolution outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum NameResolution {
    /// Resolved source declaration.
    Resolved(SymbolRef),
    /// Resolved builtin.
    ResolvedBuiltin(BuiltinRef),
    /// Checker-deferred selector.
    DeferredSelector(DeferredSelectorRef),
    /// Ambiguous reference.
    Ambiguous(AmbiguousNameRef),
    /// Unresolved reference.
    Unresolved(UnresolvedNameRef),
}

/// Name reference table entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameRefEntry {
    site: ReferenceSite,
    resolution: NameResolution,
    origin: SemanticOrigin,
    recovery: RecoveryState,
}

impl NameRefEntry {
    /// Creates a name reference entry.
    pub const fn new(
        site: ReferenceSite,
        resolution: NameResolution,
        origin: SemanticOrigin,
    ) -> Self {
        let recovery = recovery_from_origin(&origin);
        Self {
            site,
            resolution,
            origin,
            recovery,
        }
    }

    /// Returns the reference site.
    pub const fn site(&self) -> &ReferenceSite {
        &self.site
    }

    /// Returns the resolution.
    pub const fn resolution(&self) -> &NameResolution {
        &self.resolution
    }

    /// Returns normalized provenance for this reference.
    pub const fn origin(&self) -> &SemanticOrigin {
        &self.origin
    }

    /// Returns recovered-shell state for this reference.
    pub const fn recovery(&self) -> RecoveryState {
        self.recovery
    }
}

/// Table of name-reference outcomes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NameRefTable {
    entries: Vec<NameRefEntry>,
}

impl NameRefTable {
    /// Creates an empty name-reference table.
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Inserts an entry and returns its stable id.
    pub fn insert(&mut self, entry: NameRefEntry) -> NameRefId {
        let id = NameRefId::new(self.entries.len());
        self.entries.push(entry);
        id
    }

    /// Returns an entry by id.
    pub fn get(&self, id: NameRefId) -> Option<&NameRefEntry> {
        self.entries.get(id.index())
    }

    /// Iterates entries in deterministic insertion order.
    pub fn iter(&self) -> impl Iterator<Item = (NameRefId, &NameRefEntry)> {
        self.entries
            .iter()
            .enumerate()
            .map(|(index, entry)| (NameRefId::new(index), entry))
    }

    /// Returns the number of entries.
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether the table is empty.
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Label kind stored in label reference results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum LabelKind {
    /// Theorem or lemma label.
    Theorem,
    /// Definition label.
    Definition,
    /// Proof-step label.
    ProofStep,
    /// Registration label.
    Registration,
}

/// Expected label scope family for unresolved label references.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum LabelExpectation {
    /// `by` citation accepting a local proof-step or theorem/lemma label.
    ProofOrTheorem,
    /// Theorem or lemma label only.
    Theorem,
    /// Proof-step label only.
    ProofStep,
    /// Definition label only.
    Definition,
    /// Registration label only.
    Registration,
}

impl LabelExpectation {
    /// Returns whether this expectation accepts a concrete label kind.
    pub const fn accepts(self, kind: LabelKind) -> bool {
        matches!(
            (self, kind),
            (
                Self::ProofOrTheorem,
                LabelKind::ProofStep | LabelKind::Theorem
            ) | (Self::Theorem, LabelKind::Theorem)
                | (Self::ProofStep, LabelKind::ProofStep)
                | (Self::Definition, LabelKind::Definition)
                | (Self::Registration, LabelKind::Registration)
        )
    }
}

/// A resolved label reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelRef {
    origin: LabelOriginPath,
    kind: LabelKind,
    range: SourceRange,
}

impl LabelRef {
    /// Creates a label reference.
    pub const fn new(origin: LabelOriginPath, kind: LabelKind, range: SourceRange) -> Self {
        Self {
            origin,
            kind,
            range,
        }
    }

    /// Returns the label origin path.
    pub const fn origin(&self) -> &LabelOriginPath {
        &self.origin
    }

    /// Returns the label kind.
    pub const fn kind(&self) -> LabelKind {
        self.kind
    }

    /// Returns the use-site range.
    pub const fn range(&self) -> SourceRange {
        self.range
    }
}

/// Candidate for an ambiguous label reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelCandidate {
    origin: LabelOriginPath,
    kind: LabelKind,
    range: SourceRange,
}

impl LabelCandidate {
    /// Creates a label candidate.
    pub const fn new(origin: LabelOriginPath, kind: LabelKind, range: SourceRange) -> Self {
        Self {
            origin,
            kind,
            range,
        }
    }

    /// Returns the candidate label origin path.
    pub const fn origin(&self) -> &LabelOriginPath {
        &self.origin
    }

    /// Returns the label kind.
    pub const fn kind(&self) -> LabelKind {
        self.kind
    }

    /// Returns the candidate range.
    pub const fn range(&self) -> SourceRange {
        self.range
    }
}

impl Ord for LabelCandidate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.origin
            .cmp(&other.origin)
            .then_with(|| self.kind.cmp(&other.kind))
            .then_with(|| range_key(self.range).cmp(&range_key(other.range)))
    }
}

impl PartialOrd for LabelCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Ambiguous label reference with deterministic candidate ordering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AmbiguousLabelRef {
    spelling: String,
    range: SourceRange,
    candidates: Vec<LabelCandidate>,
}

impl AmbiguousLabelRef {
    /// Creates an ambiguous label reference and sorts candidates deterministically.
    pub fn new(
        spelling: impl Into<String>,
        range: SourceRange,
        mut candidates: Vec<LabelCandidate>,
    ) -> Self {
        candidates.sort();
        Self {
            spelling: spelling.into(),
            range,
            candidates,
        }
    }

    /// Returns sorted candidates.
    pub fn candidates(&self) -> &[LabelCandidate] {
        &self.candidates
    }

    /// Returns the ambiguous label spelling.
    pub fn spelling(&self) -> &str {
        &self.spelling
    }

    /// Returns the use-site range.
    pub const fn range(&self) -> SourceRange {
        self.range
    }
}

/// Unresolved label reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnresolvedLabelRef {
    spelling: String,
    range: SourceRange,
    expectation: LabelExpectation,
}

impl UnresolvedLabelRef {
    /// Creates an unresolved label reference.
    pub fn new(
        spelling: impl Into<String>,
        range: SourceRange,
        expectation: LabelExpectation,
    ) -> Self {
        Self {
            spelling: spelling.into(),
            range,
            expectation,
        }
    }

    /// Returns the unresolved label spelling.
    pub fn spelling(&self) -> &str {
        &self.spelling
    }

    /// Returns the use-site range.
    pub const fn range(&self) -> SourceRange {
        self.range
    }

    /// Returns the expected label scope family.
    pub const fn expectation(&self) -> LabelExpectation {
        self.expectation
    }
}

/// Label-resolution outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum LabelResolution {
    /// Resolved label.
    Resolved(LabelRef),
    /// Ambiguous label.
    Ambiguous(AmbiguousLabelRef),
    /// Unresolved label.
    Unresolved(UnresolvedLabelRef),
}

/// Label reference table entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelRefEntry {
    site: ReferenceSite,
    resolution: LabelResolution,
    origin: SemanticOrigin,
    recovery: RecoveryState,
}

impl LabelRefEntry {
    /// Creates a label reference entry.
    pub const fn new(
        site: ReferenceSite,
        resolution: LabelResolution,
        origin: SemanticOrigin,
    ) -> Self {
        let recovery = recovery_from_origin(&origin);
        Self {
            site,
            resolution,
            origin,
            recovery,
        }
    }

    /// Returns the reference site.
    pub const fn site(&self) -> &ReferenceSite {
        &self.site
    }

    /// Returns the label resolution.
    pub const fn resolution(&self) -> &LabelResolution {
        &self.resolution
    }

    /// Returns normalized provenance for this reference.
    pub const fn origin(&self) -> &SemanticOrigin {
        &self.origin
    }

    /// Returns recovered-shell state for this reference.
    pub const fn recovery(&self) -> RecoveryState {
        self.recovery
    }
}

/// Table of label-reference outcomes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LabelRefTable {
    entries: Vec<LabelRefEntry>,
}

impl LabelRefTable {
    /// Creates an empty label-reference table.
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Inserts an entry and returns its stable id.
    pub fn insert(&mut self, entry: LabelRefEntry) -> LabelRefId {
        let id = LabelRefId::new(self.entries.len());
        self.entries.push(entry);
        id
    }

    /// Returns an entry by id.
    pub fn get(&self, id: LabelRefId) -> Option<&LabelRefEntry> {
        self.entries.get(id.index())
    }

    /// Iterates entries in deterministic insertion order.
    pub fn iter(&self) -> impl Iterator<Item = (LabelRefId, &LabelRefEntry)> {
        self.entries
            .iter()
            .enumerate()
            .map(|(index, entry)| (LabelRefId::new(index), entry))
    }

    /// Returns the number of entries.
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether the table is empty.
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Import-resolution outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ImportResolution {
    /// Import resolved to a canonical module.
    Resolved(ModuleId),
    /// Import could not be resolved.
    Unresolved(UnresolvedImport),
    /// Import was ambiguous between canonical modules.
    Ambiguous(AmbiguousImport),
}

/// Ambiguous import candidates in deterministic canonical order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AmbiguousImport {
    candidates: Vec<ModuleId>,
}

impl AmbiguousImport {
    /// Creates an ambiguous import result and sorts candidates deterministically.
    pub fn new(mut candidates: Vec<ModuleId>) -> Self {
        candidates.sort();
        Self { candidates }
    }

    /// Returns sorted candidate modules.
    pub fn candidates(&self) -> &[ModuleId] {
        &self.candidates
    }
}

/// Unresolved import details.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnresolvedImport {
    spelling: String,
    range: SourceRange,
    class: ImportFailureClass,
}

impl UnresolvedImport {
    /// Creates unresolved import details.
    pub fn new(spelling: impl Into<String>, range: SourceRange, class: ImportFailureClass) -> Self {
        Self {
            spelling: spelling.into(),
            range,
            class,
        }
    }

    /// Returns the unresolved source spelling.
    pub fn spelling(&self) -> &str {
        &self.spelling
    }

    /// Returns the source range.
    pub const fn range(&self) -> SourceRange {
        self.range
    }

    /// Returns the failure class.
    pub const fn class(&self) -> ImportFailureClass {
        self.class
    }
}

/// Import failure class without public diagnostic-code commitment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ImportFailureClass {
    /// Module was not found.
    ModuleNotFound,
    /// Relative path escaped the package root.
    RelativePathEscapesPackage,
    /// Import was recovered from malformed syntax.
    RecoveredSyntax,
}

/// Resolved import entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedImport {
    owner: ResolvedNodeId,
    range: SourceRange,
    source_spelling: String,
    alias: Option<String>,
    resolution: ImportResolution,
    origin: SemanticOrigin,
    recovery: RecoveryState,
}

impl ResolvedImport {
    /// Creates a resolved import entry.
    pub fn new(
        owner: ResolvedNodeId,
        range: SourceRange,
        source_spelling: impl Into<String>,
        alias: Option<String>,
        resolution: ImportResolution,
        origin: SemanticOrigin,
    ) -> Self {
        let recovery = recovery_from_origin(&origin);
        Self {
            owner,
            range,
            source_spelling: source_spelling.into(),
            alias,
            resolution,
            origin,
            recovery,
        }
    }

    /// Returns the import resolution.
    pub const fn resolution(&self) -> &ImportResolution {
        &self.resolution
    }

    /// Returns the arena node that owns this import directive outcome.
    pub const fn owner(&self) -> ResolvedNodeId {
        self.owner
    }

    /// Returns the source range.
    pub const fn range(&self) -> SourceRange {
        self.range
    }

    /// Returns the source spelling.
    pub fn source_spelling(&self) -> &str {
        &self.source_spelling
    }

    /// Returns the local alias spelling.
    pub fn alias(&self) -> Option<&str> {
        self.alias.as_deref()
    }

    /// Returns normalized provenance for this import.
    pub const fn origin(&self) -> &SemanticOrigin {
        &self.origin
    }

    /// Returns recovered-shell state for this import.
    pub const fn recovery(&self) -> RecoveryState {
        self.recovery
    }
}

/// Export-resolution failure class without public diagnostic-code commitment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExportFailureClass {
    /// Export target was not found.
    TargetNotFound,
    /// Export target is not visible.
    NotVisible,
    /// Export target was recovered from malformed syntax.
    RecoveredSyntax,
}

/// Unresolved export details.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnresolvedExport {
    spelling: String,
    range: SourceRange,
    class: ExportFailureClass,
}

impl UnresolvedExport {
    /// Creates unresolved export details.
    pub fn new(spelling: impl Into<String>, range: SourceRange, class: ExportFailureClass) -> Self {
        Self {
            spelling: spelling.into(),
            range,
            class,
        }
    }

    /// Returns the unresolved source spelling.
    pub fn spelling(&self) -> &str {
        &self.spelling
    }

    /// Returns the source range.
    pub const fn range(&self) -> SourceRange {
        self.range
    }

    /// Returns the failure class.
    pub const fn class(&self) -> ExportFailureClass {
        self.class
    }
}

/// Export target.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExportTarget {
    /// Exported module.
    Module(ModuleId),
    /// Exported import alias and canonical target module.
    ImportAlias { alias: String, module: ModuleId },
    /// Exported symbol.
    Symbol(SymbolId),
    /// Export target could not be resolved.
    Unresolved(UnresolvedExport),
}

/// Resolved export entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedExport {
    owner: ResolvedNodeId,
    range: SourceRange,
    target: ExportTarget,
    source_spelling: String,
    origin: SemanticOrigin,
    recovery: RecoveryState,
}

impl ResolvedExport {
    /// Creates an export entry.
    pub fn new(
        owner: ResolvedNodeId,
        range: SourceRange,
        source_spelling: impl Into<String>,
        target: ExportTarget,
        origin: SemanticOrigin,
    ) -> Self {
        let recovery = recovery_from_origin(&origin);
        Self {
            owner,
            range,
            target,
            source_spelling: source_spelling.into(),
            origin,
            recovery,
        }
    }

    /// Returns the export target.
    pub const fn target(&self) -> &ExportTarget {
        &self.target
    }

    /// Returns the arena node that owns this export directive outcome.
    pub const fn owner(&self) -> ResolvedNodeId {
        self.owner
    }

    /// Returns the source range.
    pub const fn range(&self) -> SourceRange {
        self.range
    }

    /// Returns the source spelling.
    pub fn source_spelling(&self) -> &str {
        &self.source_spelling
    }

    /// Returns normalized provenance for this export.
    pub const fn origin(&self) -> &SemanticOrigin {
        &self.origin
    }

    /// Returns recovered-shell state for this export.
    pub const fn recovery(&self) -> RecoveryState {
        self.recovery
    }
}

/// Import and export outcomes for a resolved module.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResolvedImports {
    imports: Vec<ResolvedImport>,
    exports: Vec<ResolvedExport>,
}

impl ResolvedImports {
    /// Creates empty import/export tables.
    pub const fn new() -> Self {
        Self {
            imports: Vec::new(),
            exports: Vec::new(),
        }
    }

    /// Inserts an import entry.
    pub fn push_import(&mut self, import: ResolvedImport) -> ResolvedImportId {
        let id = ResolvedImportId::new(self.imports.len());
        self.imports.push(import);
        id
    }

    /// Inserts an export entry.
    pub fn push_export(&mut self, export: ResolvedExport) -> ResolvedExportId {
        let id = ResolvedExportId::new(self.exports.len());
        self.exports.push(export);
        id
    }

    /// Returns an import by id.
    pub fn import(&self, id: ResolvedImportId) -> Option<&ResolvedImport> {
        self.imports.get(id.index())
    }

    /// Returns an export by id.
    pub fn export(&self, id: ResolvedExportId) -> Option<&ResolvedExport> {
        self.exports.get(id.index())
    }

    /// Iterates imports in source order.
    pub fn imports(&self) -> impl Iterator<Item = (ResolvedImportId, &ResolvedImport)> {
        self.imports
            .iter()
            .enumerate()
            .map(|(index, import)| (ResolvedImportId::new(index), import))
    }

    /// Iterates exports in source order.
    pub fn exports(&self) -> impl Iterator<Item = (ResolvedExportId, &ResolvedExport)> {
        self.exports
            .iter()
            .enumerate()
            .map(|(index, export)| (ResolvedExportId::new(index), export))
    }

    /// Returns resolved canonical import modules in deterministic canonical order.
    pub fn canonical_import_modules(&self) -> Vec<&ModuleId> {
        let mut modules = self
            .imports
            .iter()
            .filter_map(|import| match import.resolution() {
                ImportResolution::Resolved(module) => Some(module),
                ImportResolution::Unresolved(_) => None,
                ImportResolution::Ambiguous(_) => None,
            })
            .collect::<Vec<_>>();
        modules.sort();
        modules
    }
}

/// Resolver output for one source module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedAst {
    source_id: SourceId,
    module_id: ModuleId,
    nodes: ResolvedArena,
    name_refs: NameRefTable,
    label_refs: LabelRefTable,
    imports: ResolvedImports,
}

impl ResolvedAst {
    /// Creates a resolved AST data-shape snapshot.
    pub fn try_new(
        source_id: SourceId,
        module_id: ModuleId,
        nodes: ResolvedArena,
        name_refs: NameRefTable,
        label_refs: LabelRefTable,
        imports: ResolvedImports,
    ) -> Result<Self, ResolvedAstError> {
        validate_resolved_ast(
            source_id,
            &module_id,
            &nodes,
            &name_refs,
            &label_refs,
            &imports,
        )?;
        Ok(Self {
            source_id,
            module_id,
            nodes,
            name_refs,
            label_refs,
            imports,
        })
    }

    /// Returns the source id.
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    /// Returns the canonical module id.
    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    /// Returns the node arena.
    pub const fn nodes(&self) -> &ResolvedArena {
        &self.nodes
    }

    /// Returns the name-reference table.
    pub const fn name_refs(&self) -> &NameRefTable {
        &self.name_refs
    }

    /// Returns the label-reference table.
    pub const fn label_refs(&self) -> &LabelRefTable {
        &self.label_refs
    }

    /// Returns import/export outcomes.
    pub const fn imports(&self) -> &ResolvedImports {
        &self.imports
    }

    /// Renders a stable human-readable debug snapshot.
    pub fn snapshot_text(&self) -> String {
        snapshot::resolved_ast_snapshot_text(self)
    }
}

/// Top-level resolved AST validation error.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ResolvedAstError {
    /// Arena validation failed.
    Arena(ResolvedArenaError),
    /// A node origin belongs to a different module.
    NodeModuleMismatch {
        /// Node with the mismatched origin.
        node: ResolvedNodeId,
    },
    /// A name reference points at a missing node.
    InvalidNameReferenceSite {
        /// Missing node id.
        node: ResolvedNodeId,
    },
    /// A label reference points at a missing node.
    InvalidLabelReferenceSite {
        /// Missing node id.
        node: ResolvedNodeId,
    },
    /// A node-local table key points at a missing table entry.
    InvalidNodeReferenceKey {
        /// Node carrying the stale key.
        node: ResolvedNodeId,
        /// Stale key.
        key: NodeReferenceKey,
    },
    /// A table or import/export origin belongs to a different module.
    OriginModuleMismatch,
    /// A node, table, import, or export origin belongs to a different source.
    OriginSourceMismatch,
    /// A provenance field points at a missing import edge.
    InvalidImportEdge {
        /// Missing import edge id.
        import: ResolvedImportId,
    },
    /// A recovered origin disagrees with the exposed recovery state.
    RecoveryOriginMismatch,
    /// An anchor or use-site range belongs to a different source.
    PayloadSourceMismatch,
    /// A deferred selector points at a missing base node.
    InvalidDeferredSelectorBase {
        /// Missing base node id.
        node: ResolvedNodeId,
    },
    /// A node-local reference key points at another node's entry.
    NodeReferenceSiteMismatch {
        /// Node carrying the mismatched key.
        node: ResolvedNodeId,
        /// Mismatched key.
        key: NodeReferenceKey,
    },
    /// An import or export entry points at a missing owner node.
    InvalidDirectiveOwner {
        /// Missing owner node id.
        node: ResolvedNodeId,
    },
    /// Import provenance fields disagree.
    ImportProvenanceMismatch {
        /// Import edge recorded on the name result.
        symbol_import: Option<ResolvedImportId>,
        /// Import edge recorded on the semantic origin.
        origin_import: Option<ResolvedImportId>,
    },
}

impl fmt::Display for ResolvedAstError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Arena(error) => write!(formatter, "{error}"),
            Self::NodeModuleMismatch { node } => {
                write!(
                    formatter,
                    "resolved node {} belongs to another module",
                    node.index()
                )
            }
            Self::InvalidNameReferenceSite { node } => {
                write!(
                    formatter,
                    "name reference points at missing node {}",
                    node.index()
                )
            }
            Self::InvalidLabelReferenceSite { node } => {
                write!(
                    formatter,
                    "label reference points at missing node {}",
                    node.index()
                )
            }
            Self::InvalidNodeReferenceKey { node, key } => write!(
                formatter,
                "resolved node {} has stale reference key {key:?}",
                node.index()
            ),
            Self::OriginModuleMismatch => {
                write!(formatter, "resolved table entry belongs to another module")
            }
            Self::OriginSourceMismatch => {
                write!(formatter, "resolved table entry belongs to another source")
            }
            Self::InvalidImportEdge { import } => write!(
                formatter,
                "resolved provenance points at missing import edge {}",
                import.index()
            ),
            Self::RecoveryOriginMismatch => {
                write!(formatter, "resolved recovery state disagrees with origin")
            }
            Self::PayloadSourceMismatch => {
                write!(formatter, "resolved payload belongs to another source")
            }
            Self::InvalidDeferredSelectorBase { node } => write!(
                formatter,
                "deferred selector points at missing base node {}",
                node.index()
            ),
            Self::NodeReferenceSiteMismatch { node, key } => write!(
                formatter,
                "resolved node {} has table key for another node: {key:?}",
                node.index()
            ),
            Self::InvalidDirectiveOwner { node } => write!(
                formatter,
                "resolved directive entry points at missing owner node {}",
                node.index()
            ),
            Self::ImportProvenanceMismatch {
                symbol_import,
                origin_import,
            } => write!(
                formatter,
                "resolved import provenance disagrees: symbol import {symbol_import:?}, origin import {origin_import:?}"
            ),
        }
    }
}

impl Error for ResolvedAstError {}

impl From<ResolvedArenaError> for ResolvedAstError {
    fn from(error: ResolvedArenaError) -> Self {
        Self::Arena(error)
    }
}

#[cfg(test)]
mod tests;
