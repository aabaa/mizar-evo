//! Resolved AST data shapes.

use mizar_session::{
    GeneratedSpanAnchor, ModulePath, PackageId, SourceAnchor, SourceId, SourceRange,
};
use mizar_syntax::ast::SurfaceNodeKind;
use std::cmp::Ordering;
use std::error::Error;
use std::fmt;

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
    kind: LabelKind,
}

impl UnresolvedLabelRef {
    /// Creates an unresolved label reference.
    pub fn new(spelling: impl Into<String>, range: SourceRange, kind: LabelKind) -> Self {
        Self {
            spelling: spelling.into(),
            range,
            kind,
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

    /// Returns the label scope family.
    pub const fn kind(&self) -> LabelKind {
        self.kind
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum VisitState {
    Visiting,
    Done,
}

fn validate_resolved_ast(
    source_id: SourceId,
    module_id: &ModuleId,
    nodes: &ResolvedArena,
    name_refs: &NameRefTable,
    label_refs: &LabelRefTable,
    imports: &ResolvedImports,
) -> Result<(), ResolvedAstError> {
    if nodes.node(nodes.root()).is_none() {
        return Err(ResolvedArenaError::InvalidRoot { root: nodes.root() }.into());
    }

    for (node_id, node) in nodes.iter() {
        if node.origin().source_id() != source_id {
            return Err(ResolvedAstError::OriginSourceMismatch);
        }
        validate_source_anchor(source_id, node.origin().anchor())?;
        if node.origin().module_id() != module_id {
            return Err(ResolvedAstError::NodeModuleMismatch { node: node_id });
        }
        validate_origin_import_edge(node.origin(), imports)?;
        if node.recovery() != recovery_from_origin(node.origin()) {
            return Err(ResolvedAstError::RecoveryOriginMismatch);
        }
        if let Some(key) = node.reference_key() {
            validate_node_reference_key(node_id, key, name_refs, label_refs, imports)?;
        }
    }

    for (_, entry) in name_refs.iter() {
        if nodes.node(entry.site().node()).is_none() {
            return Err(ResolvedAstError::InvalidNameReferenceSite {
                node: entry.site().node(),
            });
        }
        validate_source_range(source_id, entry.site().range())?;
        if entry.origin().module_id() != module_id {
            return Err(ResolvedAstError::OriginModuleMismatch);
        }
        if entry.origin().source_id() != source_id {
            return Err(ResolvedAstError::OriginSourceMismatch);
        }
        validate_source_anchor(source_id, entry.origin().anchor())?;
        validate_origin_import_edge(entry.origin(), imports)?;
        if entry.recovery() != recovery_from_origin(entry.origin()) {
            return Err(ResolvedAstError::RecoveryOriginMismatch);
        }
        validate_name_resolution(
            source_id,
            nodes,
            imports,
            entry.origin(),
            entry.resolution(),
        )?;
    }

    for (_, entry) in label_refs.iter() {
        if nodes.node(entry.site().node()).is_none() {
            return Err(ResolvedAstError::InvalidLabelReferenceSite {
                node: entry.site().node(),
            });
        }
        validate_source_range(source_id, entry.site().range())?;
        if entry.origin().module_id() != module_id {
            return Err(ResolvedAstError::OriginModuleMismatch);
        }
        if entry.origin().source_id() != source_id {
            return Err(ResolvedAstError::OriginSourceMismatch);
        }
        validate_source_anchor(source_id, entry.origin().anchor())?;
        validate_origin_import_edge(entry.origin(), imports)?;
        if entry.recovery() != recovery_from_origin(entry.origin()) {
            return Err(ResolvedAstError::RecoveryOriginMismatch);
        }
        validate_label_resolution(source_id, entry.resolution())?;
    }

    for (_, import) in imports.imports() {
        if nodes.node(import.owner()).is_none() {
            return Err(ResolvedAstError::InvalidDirectiveOwner {
                node: import.owner(),
            });
        }
        validate_source_range(source_id, import.range())?;
        if import.origin().module_id() != module_id {
            return Err(ResolvedAstError::OriginModuleMismatch);
        }
        if import.origin().source_id() != source_id {
            return Err(ResolvedAstError::OriginSourceMismatch);
        }
        validate_source_anchor(source_id, import.origin().anchor())?;
        validate_origin_import_edge(import.origin(), imports)?;
        if import.recovery() != recovery_from_origin(import.origin()) {
            return Err(ResolvedAstError::RecoveryOriginMismatch);
        }
        validate_import_resolution(source_id, import.resolution())?;
    }
    for (_, export) in imports.exports() {
        if nodes.node(export.owner()).is_none() {
            return Err(ResolvedAstError::InvalidDirectiveOwner {
                node: export.owner(),
            });
        }
        validate_source_range(source_id, export.range())?;
        if export.origin().module_id() != module_id {
            return Err(ResolvedAstError::OriginModuleMismatch);
        }
        if export.origin().source_id() != source_id {
            return Err(ResolvedAstError::OriginSourceMismatch);
        }
        validate_source_anchor(source_id, export.origin().anchor())?;
        validate_origin_import_edge(export.origin(), imports)?;
        if export.recovery() != recovery_from_origin(export.origin()) {
            return Err(ResolvedAstError::RecoveryOriginMismatch);
        }
        validate_export_target(source_id, export.target())?;
    }

    Ok(())
}

fn validate_origin_import_edge(
    origin: &SemanticOrigin,
    imports: &ResolvedImports,
) -> Result<(), ResolvedAstError> {
    if let Some(import) = origin.import_edge()
        && imports.import(import).is_none()
    {
        return Err(ResolvedAstError::InvalidImportEdge { import });
    }
    Ok(())
}

fn validate_node_reference_key(
    node_id: ResolvedNodeId,
    key: NodeReferenceKey,
    name_refs: &NameRefTable,
    label_refs: &LabelRefTable,
    imports: &ResolvedImports,
) -> Result<(), ResolvedAstError> {
    match key {
        NodeReferenceKey::Name(id) => {
            let Some(entry) = name_refs.get(id) else {
                return Err(ResolvedAstError::InvalidNodeReferenceKey { node: node_id, key });
            };
            if entry.site().node() != node_id {
                return Err(ResolvedAstError::NodeReferenceSiteMismatch { node: node_id, key });
            }
        }
        NodeReferenceKey::Label(id) => {
            let Some(entry) = label_refs.get(id) else {
                return Err(ResolvedAstError::InvalidNodeReferenceKey { node: node_id, key });
            };
            if entry.site().node() != node_id {
                return Err(ResolvedAstError::NodeReferenceSiteMismatch { node: node_id, key });
            }
        }
        NodeReferenceKey::Import(id) => {
            let Some(import) = imports.import(id) else {
                return Err(ResolvedAstError::InvalidNodeReferenceKey { node: node_id, key });
            };
            if import.owner() != node_id {
                return Err(ResolvedAstError::NodeReferenceSiteMismatch { node: node_id, key });
            }
        }
        NodeReferenceKey::Export(id) => {
            let Some(export) = imports.export(id) else {
                return Err(ResolvedAstError::InvalidNodeReferenceKey { node: node_id, key });
            };
            if export.owner() != node_id {
                return Err(ResolvedAstError::NodeReferenceSiteMismatch { node: node_id, key });
            }
        }
    }
    Ok(())
}

fn validate_name_resolution(
    source_id: SourceId,
    nodes: &ResolvedArena,
    imports: &ResolvedImports,
    origin: &SemanticOrigin,
    resolution: &NameResolution,
) -> Result<(), ResolvedAstError> {
    match resolution {
        NameResolution::Resolved(symbol) => {
            validate_source_range(source_id, symbol.range())?;
            if let Some(import) = symbol.import() {
                validate_import_edge(import, imports)?;
            }
            if symbol.import() != origin.import_edge() {
                return Err(ResolvedAstError::ImportProvenanceMismatch {
                    symbol_import: symbol.import(),
                    origin_import: origin.import_edge(),
                });
            }
        }
        NameResolution::ResolvedBuiltin(builtin) => {
            validate_source_range(source_id, builtin.range())?
        }
        NameResolution::DeferredSelector(selector) => {
            validate_source_range(source_id, selector.range())?;
            if nodes.node(selector.base()).is_none() {
                return Err(ResolvedAstError::InvalidDeferredSelectorBase {
                    node: selector.base(),
                });
            }
        }
        NameResolution::Ambiguous(ambiguous) => {
            validate_source_range(source_id, ambiguous.range())?;
        }
        NameResolution::Unresolved(unresolved) => {
            validate_source_range(source_id, unresolved.range())?
        }
    }
    Ok(())
}

fn validate_label_resolution(
    source_id: SourceId,
    resolution: &LabelResolution,
) -> Result<(), ResolvedAstError> {
    match resolution {
        LabelResolution::Resolved(label) => validate_source_range(source_id, label.range())?,
        LabelResolution::Ambiguous(ambiguous) => {
            validate_source_range(source_id, ambiguous.range())?;
        }
        LabelResolution::Unresolved(unresolved) => {
            validate_source_range(source_id, unresolved.range())?
        }
    }
    Ok(())
}

fn validate_import_resolution(
    source_id: SourceId,
    resolution: &ImportResolution,
) -> Result<(), ResolvedAstError> {
    match resolution {
        ImportResolution::Resolved(_) => {}
        ImportResolution::Ambiguous(_) => {}
        ImportResolution::Unresolved(unresolved) => {
            validate_source_range(source_id, unresolved.range())?;
        }
    }
    Ok(())
}

fn validate_export_target(
    source_id: SourceId,
    target: &ExportTarget,
) -> Result<(), ResolvedAstError> {
    match target {
        ExportTarget::Module(_) | ExportTarget::ImportAlias { .. } | ExportTarget::Symbol(_) => {}
        ExportTarget::Unresolved(unresolved) => {
            validate_source_range(source_id, unresolved.range())?
        }
    }
    Ok(())
}

fn validate_import_edge(
    import: ResolvedImportId,
    imports: &ResolvedImports,
) -> Result<(), ResolvedAstError> {
    if imports.import(import).is_none() {
        return Err(ResolvedAstError::InvalidImportEdge { import });
    }
    Ok(())
}

fn validate_source_range(source_id: SourceId, range: SourceRange) -> Result<(), ResolvedAstError> {
    if range.source_id != source_id {
        return Err(ResolvedAstError::PayloadSourceMismatch);
    }
    Ok(())
}

fn validate_source_anchor(
    source_id: SourceId,
    anchor: &SourceAnchor,
) -> Result<(), ResolvedAstError> {
    if let Some(anchor_source_id) = source_anchor_id(anchor)
        && anchor_source_id != source_id
    {
        return Err(ResolvedAstError::PayloadSourceMismatch);
    }
    Ok(())
}

fn source_anchor_id(anchor: &SourceAnchor) -> Option<SourceId> {
    match anchor {
        SourceAnchor::Range(range) => Some(range.source_id),
        SourceAnchor::Point { source_id, .. } => Some(*source_id),
        SourceAnchor::Generated(origin) => generated_span_anchor_id(origin.anchor()),
        _ => None,
    }
}

const fn generated_span_anchor_id(anchor: GeneratedSpanAnchor) -> Option<SourceId> {
    match anchor {
        GeneratedSpanAnchor::Range(range) => Some(range.source_id),
        GeneratedSpanAnchor::Point { source_id, .. } => Some(source_id),
        _ => None,
    }
}

fn validate_nodes(nodes: &[ResolvedNode]) -> Result<(), ResolvedArenaError> {
    for (index, node) in nodes.iter().enumerate() {
        let node_id = ResolvedNodeId::new(index);
        for child in node.children() {
            if child.index() >= nodes.len() {
                return Err(ResolvedArenaError::InvalidChild {
                    node: node_id,
                    child: *child,
                });
            }
        }
    }

    let mut states = vec![None; nodes.len()];
    for index in 0..nodes.len() {
        visit_node(index, nodes, &mut states)?;
    }
    Ok(())
}

fn visit_node(
    index: usize,
    nodes: &[ResolvedNode],
    states: &mut [Option<VisitState>],
) -> Result<(), ResolvedArenaError> {
    match states[index] {
        Some(VisitState::Done) => return Ok(()),
        Some(VisitState::Visiting) => {
            return Err(ResolvedArenaError::Cycle {
                node: ResolvedNodeId::new(index),
            });
        }
        None => {}
    }

    states[index] = Some(VisitState::Visiting);
    for child in nodes[index].children() {
        visit_node(child.index(), nodes, states)?;
    }
    states[index] = Some(VisitState::Done);
    Ok(())
}

const fn range_key(range: SourceRange) -> (usize, usize) {
    (range.start, range.end)
}

const fn recovery_from_origin(origin: &SemanticOrigin) -> RecoveryState {
    if origin.recovered {
        RecoveryState::Recovered
    } else {
        RecoveryState::Normal
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mizar_session::{
        BuildSnapshotId, Hash, InMemorySessionIdAllocator, SessionIdAllocator, SourceAnchor,
    };
    use mizar_syntax::ast::SurfaceNodeKind;

    #[test]
    fn module_and_symbol_ids_are_deterministic_and_alias_independent() {
        let module = module_id("pkg", "algebra.group");
        let same_module = module_id("pkg", "algebra.group");
        let alias_spelling_does_not_participate = module_id("pkg", "algebra.group");

        assert_eq!(module, same_module);
        assert_eq!(module, alias_spelling_does_not_participate);

        let symbol = symbol_id(module.clone(), "pred/0", "pkg::algebra.group::pred/0");
        let same_symbol = symbol_id(same_module, "pred/0", "pkg::algebra.group::pred/0");

        assert_eq!(symbol, same_symbol);
        assert_eq!(symbol.fqn().as_str(), "pkg::algebra.group::pred/0");
    }

    #[test]
    fn arena_allocates_deterministic_ids_and_validates_children() {
        let source_id = source_id(1);
        let module = module_id("pkg", "main");
        let origin = origin(source_id, module);
        let mut builder = ResolvedArenaBuilder::new();

        let child = builder
            .push(ResolvedNode::new(
                SurfaceNodeKind::PlaceholderItem,
                Vec::new(),
                origin.clone(),
            ))
            .unwrap();
        let parent = builder
            .push(ResolvedNode::new(
                SurfaceNodeKind::ItemList,
                vec![child],
                origin.clone(),
            ))
            .unwrap();
        let arena = builder.finish(parent).unwrap();

        assert_eq!(child.index(), 0);
        assert_eq!(parent.index(), 1);
        assert_eq!(arena.node(parent).unwrap().children(), &[child]);

        let invalid = ResolvedNode::new(
            SurfaceNodeKind::CompilationUnit,
            vec![ResolvedNodeId::new(99)],
            origin,
        );
        assert!(matches!(
            ResolvedArena::try_new(ResolvedNodeId::new(0), vec![invalid]),
            Err(ResolvedArenaError::InvalidChild { .. })
        ));
    }

    #[test]
    fn arena_rejects_cycles() {
        let source_id = source_id(2);
        let module = module_id("pkg", "main");
        let origin = origin(source_id, module);
        let first = ResolvedNode::new(
            SurfaceNodeKind::ItemList,
            vec![ResolvedNodeId::new(1)],
            origin.clone(),
        );
        let second = ResolvedNode::new(
            SurfaceNodeKind::PlaceholderItem,
            vec![ResolvedNodeId::new(0)],
            origin,
        );

        assert!(matches!(
            ResolvedArena::try_new(ResolvedNodeId::new(0), vec![first, second]),
            Err(ResolvedArenaError::Cycle { .. })
        ));
    }

    #[test]
    fn name_ref_table_round_trips_all_current_result_kinds() {
        let source_id = source_id(3);
        let site_range = range(source_id, 1, 4);
        let node = ResolvedNodeId::new(0);
        let module = module_id("pkg", "main");
        let entry_origin = origin(source_id, module.clone());
        let symbol = symbol_id(module, "pred/0", "pkg::main::pred/0");
        let candidate_a = NameResolutionCandidate::new(symbol.clone(), range(source_id, 10, 12));
        let candidate_b = NameResolutionCandidate::new(
            symbol_id(module_id("pkg", "other"), "pred/0", "pkg::other::pred/0"),
            range(source_id, 8, 9),
        );
        let mut table = NameRefTable::new();

        let resolved = table.insert(NameRefEntry::new(
            ReferenceSite::new(node, site_range, "P"),
            NameResolution::Resolved(SymbolRef::new(symbol, site_range)),
            entry_origin.clone(),
        ));
        let builtin = table.insert(NameRefEntry::new(
            ReferenceSite::new(node, site_range, "true"),
            NameResolution::ResolvedBuiltin(BuiltinRef::new(
                BuiltinId::new("true"),
                site_range,
                "true",
            )),
            entry_origin.clone(),
        ));
        let deferred = table.insert(NameRefEntry::new(
            ReferenceSite::new(node, site_range, "x.y"),
            NameResolution::DeferredSelector(DeferredSelectorRef::new(node, "y", site_range)),
            entry_origin.clone(),
        ));
        let ambiguous = table.insert(NameRefEntry::new(
            ReferenceSite::new(node, site_range, "P"),
            NameResolution::Ambiguous(AmbiguousNameRef::new(
                "P",
                site_range,
                vec![candidate_a, candidate_b],
            )),
            entry_origin.clone(),
        ));
        let unresolved = table.insert(NameRefEntry::new(
            ReferenceSite::new(node, site_range, "Missing"),
            NameResolution::Unresolved(UnresolvedNameRef::new(
                "Missing",
                site_range,
                NameLookupClass::Symbol,
            )),
            entry_origin,
        ));

        assert!(matches!(
            table.get(resolved).unwrap().resolution(),
            NameResolution::Resolved(_)
        ));
        let NameResolution::Resolved(symbol_ref) = table.get(resolved).unwrap().resolution() else {
            panic!("expected resolved symbol ref");
        };
        assert_eq!(symbol_ref.range(), site_range);
        assert_eq!(symbol_ref.spelling(), None);
        assert!(matches!(
            table.get(builtin).unwrap().resolution(),
            NameResolution::ResolvedBuiltin(_)
        ));
        assert!(matches!(
            table.get(deferred).unwrap().resolution(),
            NameResolution::DeferredSelector(_)
        ));
        let NameResolution::Ambiguous(ambiguous_ref) = table.get(ambiguous).unwrap().resolution()
        else {
            panic!("expected ambiguous name ref");
        };
        assert_eq!(
            ambiguous_ref
                .candidates()
                .iter()
                .map(|candidate| candidate.symbol().fqn().as_str())
                .collect::<Vec<_>>(),
            vec!["pkg::main::pred/0", "pkg::other::pred/0"]
        );
        assert!(matches!(
            table.get(unresolved).unwrap().resolution(),
            NameResolution::Unresolved(_)
        ));
    }

    #[test]
    fn ambiguous_name_candidates_tie_break_by_range_before_local_symbol_id() {
        let source_id = source_id(10);
        let module = module_id("pkg", "main");
        let late_range = range(source_id, 20, 22);
        let early_range = range(source_id, 10, 12);
        let ambiguous = AmbiguousNameRef::new(
            "P",
            range(source_id, 0, 1),
            vec![
                NameResolutionCandidate::new(
                    symbol_id(module.clone(), "a-local", "pkg::main::P"),
                    late_range,
                ),
                NameResolutionCandidate::new(
                    symbol_id(module.clone(), "z-local", "pkg::main::P"),
                    early_range,
                ),
            ],
        );

        assert_eq!(
            ambiguous
                .candidates()
                .iter()
                .map(|candidate| (candidate.symbol().local().as_str(), candidate.range().start))
                .collect::<Vec<_>>(),
            vec![("z-local", 10), ("a-local", 20)]
        );

        let local_tie = AmbiguousNameRef::new(
            "P",
            range(source_id, 0, 1),
            vec![
                NameResolutionCandidate::new(
                    symbol_id(module.clone(), "z-local", "pkg::main::P"),
                    range(source_id, 0, 1),
                ),
                NameResolutionCandidate::new(
                    symbol_id(module.clone(), "a-local", "pkg::main::P"),
                    range(source_id, 0, 1),
                ),
            ],
        );
        assert_eq!(
            local_tie
                .candidates()
                .iter()
                .map(|candidate| candidate.symbol().local().as_str())
                .collect::<Vec<_>>(),
            vec!["a-local", "z-local"]
        );

        let module_tie = AmbiguousNameRef::new(
            "P",
            range(source_id, 0, 1),
            vec![
                NameResolutionCandidate::new(
                    symbol_id(module_id("pkg", "zeta"), "same-local", "pkg::P"),
                    range(source_id, 0, 1),
                ),
                NameResolutionCandidate::new(
                    symbol_id(module_id("pkg", "alpha"), "same-local", "pkg::P"),
                    range(source_id, 0, 1),
                ),
            ],
        );
        assert_eq!(
            module_tie
                .candidates()
                .iter()
                .map(|candidate| candidate.symbol().module().path().as_str())
                .collect::<Vec<_>>(),
            vec!["alpha", "zeta"]
        );
    }

    #[test]
    fn label_ref_table_round_trips_all_current_result_kinds() {
        let source_id = source_id(4);
        let range = range(source_id, 0, 2);
        let node = ResolvedNodeId::new(0);
        let entry_origin = origin(source_id, module_id("pkg", "main"));
        let mut table = LabelRefTable::new();

        let resolved = table.insert(LabelRefEntry::new(
            ReferenceSite::new(node, range, "A1"),
            LabelResolution::Resolved(LabelRef::new(
                LabelOriginPath::new("pkg::main::A1"),
                LabelKind::Theorem,
                range,
            )),
            entry_origin.clone(),
        ));
        let ambiguous = table.insert(LabelRefEntry::new(
            ReferenceSite::new(node, range, "A1"),
            LabelResolution::Ambiguous(AmbiguousLabelRef::new(
                "A1",
                range,
                vec![
                    LabelCandidate::new(
                        LabelOriginPath::new("pkg::main::B1"),
                        LabelKind::ProofStep,
                        range,
                    ),
                    LabelCandidate::new(
                        LabelOriginPath::new("pkg::main::A1"),
                        LabelKind::Theorem,
                        range,
                    ),
                ],
            )),
            entry_origin.clone(),
        ));
        let unresolved = table.insert(LabelRefEntry::new(
            ReferenceSite::new(node, range, "A2"),
            LabelResolution::Unresolved(UnresolvedLabelRef::new("A2", range, LabelKind::Theorem)),
            entry_origin,
        ));

        assert!(matches!(
            table.get(resolved).unwrap().resolution(),
            LabelResolution::Resolved(_)
        ));
        let LabelResolution::Ambiguous(ambiguous_ref) = table.get(ambiguous).unwrap().resolution()
        else {
            panic!("expected ambiguous label ref");
        };
        assert_eq!(
            ambiguous_ref
                .candidates()
                .iter()
                .map(|candidate| candidate.origin.as_str())
                .collect::<Vec<_>>(),
            vec!["pkg::main::A1", "pkg::main::B1"]
        );
        assert!(matches!(
            table.get(unresolved).unwrap().resolution(),
            LabelResolution::Unresolved(_)
        ));
    }

    #[test]
    fn resolved_imports_round_trip_and_project_canonical_modules() {
        let source_id = source_id(5);
        let first = module_id("pkg", "zeta");
        let second = module_id("pkg", "alpha");
        let import_origin = origin(source_id, module_id("pkg", "main"));
        let mut imports = ResolvedImports::new();

        let first_id = imports.push_import(ResolvedImport::new(
            ResolvedNodeId::new(0),
            range(source_id, 0, 10),
            "import zeta;",
            Some("z".to_owned()),
            ImportResolution::Resolved(first.clone()),
            import_origin.clone(),
        ));
        let second_id = imports.push_import(ResolvedImport::new(
            ResolvedNodeId::new(1),
            range(source_id, 11, 20),
            "import alpha;",
            None,
            ImportResolution::Resolved(second.clone()),
            import_origin.clone(),
        ));
        let unresolved_id = imports.push_import(ResolvedImport::new(
            ResolvedNodeId::new(2),
            range(source_id, 21, 30),
            "import missing;",
            None,
            ImportResolution::Unresolved(UnresolvedImport::new(
                "missing",
                range(source_id, 21, 30),
                ImportFailureClass::ModuleNotFound,
            )),
            import_origin.clone(),
        ));
        let ambiguous_id = imports.push_import(ResolvedImport::new(
            ResolvedNodeId::new(3),
            range(source_id, 31, 40),
            "import ambiguous;",
            None,
            ImportResolution::Ambiguous(AmbiguousImport::new(vec![
                module_id("pkg", "omega"),
                module_id("pkg", "beta"),
            ])),
            import_origin.clone(),
        ));
        let export_id = imports.push_export(ResolvedExport::new(
            ResolvedNodeId::new(4),
            range(source_id, 41, 50),
            "export zeta;",
            ExportTarget::Module(first.clone()),
            import_origin,
        ));

        assert!(matches!(
            imports.import(first_id).unwrap().resolution(),
            ImportResolution::Resolved(module) if module == &first
        ));
        assert!(matches!(
            imports.import(second_id).unwrap().resolution(),
            ImportResolution::Resolved(module) if module == &second
        ));
        assert!(matches!(
            imports.import(unresolved_id).unwrap().resolution(),
            ImportResolution::Unresolved(_)
        ));
        let ImportResolution::Ambiguous(ambiguous) =
            imports.import(ambiguous_id).unwrap().resolution()
        else {
            panic!("expected ambiguous import");
        };
        assert_eq!(
            ambiguous
                .candidates()
                .iter()
                .map(|module| module.path().as_str())
                .collect::<Vec<_>>(),
            vec!["beta", "omega"]
        );
        assert!(matches!(
            imports.export(export_id).unwrap().target(),
            ExportTarget::Module(module) if module == &first
        ));
        assert_eq!(
            imports
                .canonical_import_modules()
                .iter()
                .map(|module| module.path().as_str())
                .collect::<Vec<_>>(),
            vec!["alpha", "zeta"]
        );
    }

    #[test]
    fn node_resolution_state_and_reference_key_are_preserved() {
        let source_id = source_id(6);
        let module = module_id("pkg", "main");
        let origin = origin(source_id, module).recovered();
        let node = ResolvedNode::new(SurfaceNodeKind::TermReference, Vec::new(), origin)
            .with_resolution(NodeResolutionState::Unresolved)
            .with_reference_key(NodeReferenceKey::Name(NameRefId::new(3)));

        assert_eq!(node.recovery(), RecoveryState::Recovered);
        assert_eq!(node.resolution(), NodeResolutionState::Unresolved);
        assert_eq!(
            node.reference_key(),
            Some(NodeReferenceKey::Name(NameRefId::new(3)))
        );
        assert!(node.origin().is_recovered());
    }

    #[test]
    fn resolved_ast_validates_node_keys_and_preserves_traversal_states() {
        let source_id = source_id(7);
        let module = module_id("pkg", "main");
        let normal_origin = origin(source_id, module.clone());
        let recovered_origin = origin(source_id, module.clone()).recovered();
        let mut name_refs = NameRefTable::new();
        let mut label_refs = LabelRefTable::new();
        let mut imports = ResolvedImports::new();
        let name_id = name_refs.insert(NameRefEntry::new(
            ReferenceSite::new(ResolvedNodeId::new(0), range(source_id, 0, 1), "Missing"),
            NameResolution::Unresolved(UnresolvedNameRef::new(
                "Missing",
                range(source_id, 0, 1),
                NameLookupClass::Symbol,
            )),
            recovered_origin
                .clone()
                .with_import_edge(ResolvedImportId::new(0)),
        ));
        let label_id = label_refs.insert(LabelRefEntry::new(
            ReferenceSite::new(ResolvedNodeId::new(1), range(source_id, 1, 2), "A1"),
            LabelResolution::Unresolved(UnresolvedLabelRef::new(
                "A1",
                range(source_id, 1, 2),
                LabelKind::Theorem,
            )),
            normal_origin.clone(),
        ));
        let import_id = imports.push_import(ResolvedImport::new(
            ResolvedNodeId::new(2),
            range(source_id, 2, 3),
            "import dep;",
            None,
            ImportResolution::Resolved(module_id("pkg", "dep")),
            normal_origin.clone(),
        ));
        let imported_name_id = name_refs.insert(NameRefEntry::new(
            ReferenceSite::new(ResolvedNodeId::new(0), range(source_id, 4, 5), "Imported"),
            NameResolution::Resolved(
                SymbolRef::new(
                    symbol_id(module_id("pkg", "dep"), "pred/0", "pkg::dep::pred/0"),
                    range(source_id, 4, 5),
                )
                .with_import(import_id),
            ),
            normal_origin.clone().with_import_edge(import_id),
        ));
        let export_id = imports.push_export(ResolvedExport::new(
            ResolvedNodeId::new(3),
            range(source_id, 3, 4),
            "export Missing;",
            ExportTarget::Unresolved(UnresolvedExport::new(
                "Missing",
                range(source_id, 3, 4),
                ExportFailureClass::TargetNotFound,
            )),
            recovered_origin.clone(),
        ));

        let mut builder = ResolvedArenaBuilder::new();
        let unresolved = builder
            .push(
                ResolvedNode::new(SurfaceNodeKind::TermReference, Vec::new(), recovered_origin)
                    .with_recovery(RecoveryState::Recovered)
                    .with_resolution(NodeResolutionState::Unresolved)
                    .with_reference_key(NodeReferenceKey::Name(name_id)),
            )
            .unwrap();
        let ambiguous = builder
            .push(
                ResolvedNode::new(
                    SurfaceNodeKind::Reference,
                    Vec::new(),
                    normal_origin.clone(),
                )
                .with_resolution(NodeResolutionState::Ambiguous)
                .with_reference_key(NodeReferenceKey::Label(label_id)),
            )
            .unwrap();
        let deferred = builder
            .push(
                ResolvedNode::new(
                    SurfaceNodeKind::ImportItem,
                    Vec::new(),
                    normal_origin.clone(),
                )
                .with_resolution(NodeResolutionState::Deferred)
                .with_reference_key(NodeReferenceKey::Import(import_id)),
            )
            .unwrap();
        let resolved = builder
            .push(
                ResolvedNode::new(
                    SurfaceNodeKind::ExportItem,
                    Vec::new(),
                    normal_origin.clone(),
                )
                .with_resolution(NodeResolutionState::Resolved)
                .with_reference_key(NodeReferenceKey::Export(export_id)),
            )
            .unwrap();
        let root = builder
            .push(ResolvedNode::new(
                SurfaceNodeKind::CompilationUnit,
                vec![unresolved, ambiguous, deferred, resolved],
                normal_origin,
            ))
            .unwrap();
        let arena = builder.finish(root).unwrap();
        let ast =
            ResolvedAst::try_new(source_id, module, arena, name_refs, label_refs, imports).unwrap();

        assert_eq!(ast.nodes().root(), root);
        assert_eq!(
            ast.nodes()
                .iter()
                .take(4)
                .map(|(_, node)| node.resolution())
                .collect::<Vec<_>>(),
            vec![
                NodeResolutionState::Unresolved,
                NodeResolutionState::Ambiguous,
                NodeResolutionState::Deferred,
                NodeResolutionState::Resolved,
            ]
        );
        assert_eq!(
            ast.nodes()
                .iter()
                .take(4)
                .map(|(_, node)| node.reference_key())
                .collect::<Vec<_>>(),
            vec![
                Some(NodeReferenceKey::Name(name_id)),
                Some(NodeReferenceKey::Label(label_id)),
                Some(NodeReferenceKey::Import(import_id)),
                Some(NodeReferenceKey::Export(export_id)),
            ]
        );
        assert_eq!(
            ast.nodes().node(root).unwrap().origin().anchor(),
            &SourceAnchor::Range(range(source_id, 0, 1))
        );
        assert_eq!(
            ast.nodes().node(root).unwrap().origin().structural_path(),
            &[0]
        );
        let name_entry = ast.name_refs().get(name_id).unwrap();
        assert_eq!(name_entry.recovery(), RecoveryState::Recovered);
        assert_eq!(
            name_entry.origin().anchor(),
            &SourceAnchor::Range(range(source_id, 0, 1))
        );
        assert_eq!(name_entry.origin().structural_path(), &[0]);
        assert_eq!(
            name_entry.origin().import_edge(),
            Some(ResolvedImportId::new(0))
        );
        assert_eq!(
            ast.imports().import(import_id).unwrap().origin().anchor(),
            &SourceAnchor::Range(range(source_id, 0, 1))
        );
        let imported_name = ast.name_refs().get(imported_name_id).unwrap();
        assert!(matches!(
            imported_name.resolution(),
            NameResolution::Resolved(symbol) if symbol.import() == Some(import_id)
        ));
        assert_eq!(imported_name.origin().import_edge(), Some(import_id));
        let export = ast.imports().export(export_id).unwrap();
        assert_eq!(export.recovery(), RecoveryState::Recovered);
        assert_eq!(
            export.origin().anchor(),
            &SourceAnchor::Range(range(source_id, 0, 1))
        );
        assert_eq!(export.origin().structural_path(), &[0]);
        assert!(matches!(
            export.target(),
            ExportTarget::Unresolved(unresolved)
                if unresolved.class() == ExportFailureClass::TargetNotFound
                    && unresolved.spelling() == "Missing"
        ));
    }

    #[test]
    fn resolved_ast_rejects_stale_keys_and_mismatched_modules() {
        let (primary_source_id, other_source_id) = source_id_pair(8);
        let module = module_id("pkg", "main");
        let other_module = module_id("pkg", "other");
        let module_origin = origin(primary_source_id, module.clone());
        let mut builder = ResolvedArenaBuilder::new();
        let root = builder
            .push(
                ResolvedNode::new(SurfaceNodeKind::CompilationUnit, Vec::new(), module_origin)
                    .with_reference_key(NodeReferenceKey::Name(NameRefId::new(99))),
            )
            .unwrap();
        let arena = builder.finish(root).unwrap();

        assert!(matches!(
            ResolvedAst::try_new(
                primary_source_id,
                module.clone(),
                arena,
                NameRefTable::new(),
                LabelRefTable::new(),
                ResolvedImports::new(),
            ),
            Err(ResolvedAstError::InvalidNodeReferenceKey { .. })
        ));

        let mismatched_arena = ResolvedArena::try_new(
            root,
            vec![ResolvedNode::new(
                SurfaceNodeKind::CompilationUnit,
                Vec::new(),
                origin(primary_source_id, other_module),
            )],
        )
        .unwrap();
        assert!(matches!(
            ResolvedAst::try_new(
                primary_source_id,
                module,
                mismatched_arena,
                NameRefTable::new(),
                LabelRefTable::new(),
                ResolvedImports::new(),
            ),
            Err(ResolvedAstError::NodeModuleMismatch { .. })
        ));

        let wrong_source_arena = ResolvedArena::try_new(
            root,
            vec![ResolvedNode::new(
                SurfaceNodeKind::CompilationUnit,
                Vec::new(),
                origin(other_source_id, module_id("pkg", "main")),
            )],
        )
        .unwrap();
        assert!(matches!(
            ResolvedAst::try_new(
                primary_source_id,
                module_id("pkg", "main"),
                wrong_source_arena,
                NameRefTable::new(),
                LabelRefTable::new(),
                ResolvedImports::new(),
            ),
            Err(ResolvedAstError::OriginSourceMismatch)
        ));

        let mut stale_import_edge_refs = NameRefTable::new();
        stale_import_edge_refs.insert(NameRefEntry::new(
            ReferenceSite::new(root, range(primary_source_id, 0, 1), "VisibleByImport"),
            NameResolution::Unresolved(UnresolvedNameRef::new(
                "VisibleByImport",
                range(primary_source_id, 0, 1),
                NameLookupClass::Symbol,
            )),
            origin(primary_source_id, module_id("pkg", "main"))
                .with_import_edge(ResolvedImportId::new(99)),
        ));
        let stale_import_edge_arena = ResolvedArena::try_new(
            root,
            vec![ResolvedNode::new(
                SurfaceNodeKind::CompilationUnit,
                Vec::new(),
                origin(primary_source_id, module_id("pkg", "main")),
            )],
        )
        .unwrap();
        assert!(matches!(
            ResolvedAst::try_new(
                primary_source_id,
                module_id("pkg", "main"),
                stale_import_edge_arena,
                stale_import_edge_refs,
                LabelRefTable::new(),
                ResolvedImports::new(),
            ),
            Err(ResolvedAstError::InvalidImportEdge { .. })
        ));

        let wrong_anchor_arena = ResolvedArena::try_new(
            root,
            vec![ResolvedNode::new(
                SurfaceNodeKind::CompilationUnit,
                Vec::new(),
                SemanticOrigin::new(
                    primary_source_id,
                    module_id("pkg", "main"),
                    SourceAnchor::Range(range(other_source_id, 0, 1)),
                    vec![0],
                ),
            )],
        )
        .unwrap();
        assert!(matches!(
            ResolvedAst::try_new(
                primary_source_id,
                module_id("pkg", "main"),
                wrong_anchor_arena,
                NameRefTable::new(),
                LabelRefTable::new(),
                ResolvedImports::new(),
            ),
            Err(ResolvedAstError::PayloadSourceMismatch)
        ));

        let mut wrong_site_refs = NameRefTable::new();
        wrong_site_refs.insert(NameRefEntry::new(
            ReferenceSite::new(root, range(other_source_id, 0, 1), "WrongSource"),
            NameResolution::Unresolved(UnresolvedNameRef::new(
                "WrongSource",
                range(primary_source_id, 0, 1),
                NameLookupClass::Symbol,
            )),
            origin(primary_source_id, module_id("pkg", "main")),
        ));
        let wrong_site_arena = ResolvedArena::try_new(
            root,
            vec![ResolvedNode::new(
                SurfaceNodeKind::CompilationUnit,
                Vec::new(),
                origin(primary_source_id, module_id("pkg", "main")),
            )],
        )
        .unwrap();
        assert!(matches!(
            ResolvedAst::try_new(
                primary_source_id,
                module_id("pkg", "main"),
                wrong_site_arena,
                wrong_site_refs,
                LabelRefTable::new(),
                ResolvedImports::new(),
            ),
            Err(ResolvedAstError::PayloadSourceMismatch)
        ));

        let mut stale_symbol_import_refs = NameRefTable::new();
        stale_symbol_import_refs.insert(NameRefEntry::new(
            ReferenceSite::new(root, range(primary_source_id, 0, 1), "P"),
            NameResolution::Resolved(
                SymbolRef::new(
                    symbol_id(module_id("pkg", "main"), "pred/0", "pkg::main::pred/0"),
                    range(primary_source_id, 0, 1),
                )
                .with_import(ResolvedImportId::new(99)),
            ),
            origin(primary_source_id, module_id("pkg", "main")),
        ));
        let stale_symbol_import_arena = ResolvedArena::try_new(
            root,
            vec![ResolvedNode::new(
                SurfaceNodeKind::CompilationUnit,
                Vec::new(),
                origin(primary_source_id, module_id("pkg", "main")),
            )],
        )
        .unwrap();
        assert!(matches!(
            ResolvedAst::try_new(
                primary_source_id,
                module_id("pkg", "main"),
                stale_symbol_import_arena,
                stale_symbol_import_refs,
                LabelRefTable::new(),
                ResolvedImports::new(),
            ),
            Err(ResolvedAstError::InvalidImportEdge { .. })
        ));

        let mut missing_base_refs = NameRefTable::new();
        missing_base_refs.insert(NameRefEntry::new(
            ReferenceSite::new(root, range(primary_source_id, 0, 1), "x.y"),
            NameResolution::DeferredSelector(DeferredSelectorRef::new(
                ResolvedNodeId::new(99),
                "y",
                range(primary_source_id, 0, 1),
            )),
            origin(primary_source_id, module_id("pkg", "main")),
        ));
        let missing_base_arena = ResolvedArena::try_new(
            root,
            vec![ResolvedNode::new(
                SurfaceNodeKind::CompilationUnit,
                Vec::new(),
                origin(primary_source_id, module_id("pkg", "main")),
            )],
        )
        .unwrap();
        assert!(matches!(
            ResolvedAst::try_new(
                primary_source_id,
                module_id("pkg", "main"),
                missing_base_arena,
                missing_base_refs,
                LabelRefTable::new(),
                ResolvedImports::new(),
            ),
            Err(ResolvedAstError::InvalidDeferredSelectorBase { .. })
        ));

        let mut mismatched_site_refs = NameRefTable::new();
        let mismatched_name_id = mismatched_site_refs.insert(NameRefEntry::new(
            ReferenceSite::new(
                ResolvedNodeId::new(0),
                range(primary_source_id, 0, 1),
                "OwnedByChild",
            ),
            NameResolution::Unresolved(UnresolvedNameRef::new(
                "OwnedByChild",
                range(primary_source_id, 0, 1),
                NameLookupClass::Symbol,
            )),
            origin(primary_source_id, module_id("pkg", "main")),
        ));
        let child = ResolvedNode::new(
            SurfaceNodeKind::TermReference,
            Vec::new(),
            origin(primary_source_id, module_id("pkg", "main")),
        );
        let keyed_parent = ResolvedNode::new(
            SurfaceNodeKind::CompilationUnit,
            vec![ResolvedNodeId::new(0)],
            origin(primary_source_id, module_id("pkg", "main")),
        )
        .with_reference_key(NodeReferenceKey::Name(mismatched_name_id));
        let mismatched_site_arena =
            ResolvedArena::try_new(ResolvedNodeId::new(1), vec![child, keyed_parent]).unwrap();
        assert!(matches!(
            ResolvedAst::try_new(
                primary_source_id,
                module_id("pkg", "main"),
                mismatched_site_arena,
                mismatched_site_refs,
                LabelRefTable::new(),
                ResolvedImports::new(),
            ),
            Err(ResolvedAstError::NodeReferenceSiteMismatch { .. })
        ));

        let mut mismatched_label_refs = LabelRefTable::new();
        let mismatched_label_id = mismatched_label_refs.insert(LabelRefEntry::new(
            ReferenceSite::new(
                ResolvedNodeId::new(0),
                range(primary_source_id, 0, 1),
                "OwnedByChildLabel",
            ),
            LabelResolution::Unresolved(UnresolvedLabelRef::new(
                "OwnedByChildLabel",
                range(primary_source_id, 0, 1),
                LabelKind::Theorem,
            )),
            origin(primary_source_id, module_id("pkg", "main")),
        ));
        let label_child = ResolvedNode::new(
            SurfaceNodeKind::AnnotationLabel,
            Vec::new(),
            origin(primary_source_id, module_id("pkg", "main")),
        );
        let keyed_label_parent = ResolvedNode::new(
            SurfaceNodeKind::CompilationUnit,
            vec![ResolvedNodeId::new(0)],
            origin(primary_source_id, module_id("pkg", "main")),
        )
        .with_reference_key(NodeReferenceKey::Label(mismatched_label_id));
        let mismatched_label_site_arena = ResolvedArena::try_new(
            ResolvedNodeId::new(1),
            vec![label_child, keyed_label_parent],
        )
        .unwrap();
        assert!(matches!(
            ResolvedAst::try_new(
                primary_source_id,
                module_id("pkg", "main"),
                mismatched_label_site_arena,
                NameRefTable::new(),
                mismatched_label_refs,
                ResolvedImports::new(),
            ),
            Err(ResolvedAstError::NodeReferenceSiteMismatch { .. })
        ));

        let mut import_owner_imports = ResolvedImports::new();
        let owner_mismatched_import_id = import_owner_imports.push_import(ResolvedImport::new(
            ResolvedNodeId::new(0),
            range(primary_source_id, 0, 1),
            "import dep;",
            None,
            ImportResolution::Resolved(module_id("pkg", "dep")),
            origin(primary_source_id, module_id("pkg", "main")),
        ));
        let import_owner_child = ResolvedNode::new(
            SurfaceNodeKind::ImportItem,
            Vec::new(),
            origin(primary_source_id, module_id("pkg", "main")),
        );
        let import_keyed_parent = ResolvedNode::new(
            SurfaceNodeKind::CompilationUnit,
            vec![ResolvedNodeId::new(0)],
            origin(primary_source_id, module_id("pkg", "main")),
        )
        .with_reference_key(NodeReferenceKey::Import(owner_mismatched_import_id));
        let import_owner_arena = ResolvedArena::try_new(
            ResolvedNodeId::new(1),
            vec![import_owner_child, import_keyed_parent],
        )
        .unwrap();
        assert!(matches!(
            ResolvedAst::try_new(
                primary_source_id,
                module_id("pkg", "main"),
                import_owner_arena,
                NameRefTable::new(),
                LabelRefTable::new(),
                import_owner_imports,
            ),
            Err(ResolvedAstError::NodeReferenceSiteMismatch { .. })
        ));

        let mut export_owner_imports = ResolvedImports::new();
        let owner_mismatched_export_id = export_owner_imports.push_export(ResolvedExport::new(
            ResolvedNodeId::new(0),
            range(primary_source_id, 0, 1),
            "export dep;",
            ExportTarget::Module(module_id("pkg", "dep")),
            origin(primary_source_id, module_id("pkg", "main")),
        ));
        let export_owner_child = ResolvedNode::new(
            SurfaceNodeKind::ExportItem,
            Vec::new(),
            origin(primary_source_id, module_id("pkg", "main")),
        );
        let export_keyed_parent = ResolvedNode::new(
            SurfaceNodeKind::CompilationUnit,
            vec![ResolvedNodeId::new(0)],
            origin(primary_source_id, module_id("pkg", "main")),
        )
        .with_reference_key(NodeReferenceKey::Export(owner_mismatched_export_id));
        let export_owner_arena = ResolvedArena::try_new(
            ResolvedNodeId::new(1),
            vec![export_owner_child, export_keyed_parent],
        )
        .unwrap();
        assert!(matches!(
            ResolvedAst::try_new(
                primary_source_id,
                module_id("pkg", "main"),
                export_owner_arena,
                NameRefTable::new(),
                LabelRefTable::new(),
                export_owner_imports,
            ),
            Err(ResolvedAstError::NodeReferenceSiteMismatch { .. })
        ));

        let mut provenance_imports = ResolvedImports::new();
        let first_import = provenance_imports.push_import(ResolvedImport::new(
            ResolvedNodeId::new(1),
            range(primary_source_id, 1, 2),
            "import one;",
            None,
            ImportResolution::Resolved(module_id("pkg", "one")),
            origin(primary_source_id, module_id("pkg", "main")),
        ));
        let second_import = provenance_imports.push_import(ResolvedImport::new(
            ResolvedNodeId::new(2),
            range(primary_source_id, 2, 3),
            "import two;",
            None,
            ImportResolution::Resolved(module_id("pkg", "two")),
            origin(primary_source_id, module_id("pkg", "main")),
        ));
        let mut provenance_refs = NameRefTable::new();
        let provenance_name_id = provenance_refs.insert(NameRefEntry::new(
            ReferenceSite::new(ResolvedNodeId::new(0), range(primary_source_id, 0, 1), "P"),
            NameResolution::Resolved(
                SymbolRef::new(
                    symbol_id(module_id("pkg", "dep"), "pred/0", "pkg::dep::pred/0"),
                    range(primary_source_id, 0, 1),
                )
                .with_import(second_import),
            ),
            origin(primary_source_id, module_id("pkg", "main")).with_import_edge(first_import),
        ));
        let provenance_arena = ResolvedArena::try_new(
            ResolvedNodeId::new(3),
            vec![
                ResolvedNode::new(
                    SurfaceNodeKind::Reference,
                    Vec::new(),
                    origin(primary_source_id, module_id("pkg", "main")),
                )
                .with_reference_key(NodeReferenceKey::Name(provenance_name_id)),
                ResolvedNode::new(
                    SurfaceNodeKind::ImportItem,
                    Vec::new(),
                    origin(primary_source_id, module_id("pkg", "main")),
                ),
                ResolvedNode::new(
                    SurfaceNodeKind::ImportItem,
                    Vec::new(),
                    origin(primary_source_id, module_id("pkg", "main")),
                ),
                ResolvedNode::new(
                    SurfaceNodeKind::CompilationUnit,
                    vec![
                        ResolvedNodeId::new(0),
                        ResolvedNodeId::new(1),
                        ResolvedNodeId::new(2),
                    ],
                    origin(primary_source_id, module_id("pkg", "main")),
                ),
            ],
        )
        .unwrap();
        assert!(matches!(
            ResolvedAst::try_new(
                primary_source_id,
                module_id("pkg", "main"),
                provenance_arena.clone(),
                provenance_refs,
                LabelRefTable::new(),
                provenance_imports.clone(),
            ),
            Err(ResolvedAstError::ImportProvenanceMismatch { .. })
        ));

        let mut symbol_only_provenance_refs = NameRefTable::new();
        symbol_only_provenance_refs.insert(NameRefEntry::new(
            ReferenceSite::new(ResolvedNodeId::new(0), range(primary_source_id, 0, 1), "P"),
            NameResolution::Resolved(
                SymbolRef::new(
                    symbol_id(module_id("pkg", "dep"), "pred/0", "pkg::dep::pred/0"),
                    range(primary_source_id, 0, 1),
                )
                .with_import(first_import),
            ),
            origin(primary_source_id, module_id("pkg", "main")),
        ));
        assert!(matches!(
            ResolvedAst::try_new(
                primary_source_id,
                module_id("pkg", "main"),
                provenance_arena.clone(),
                symbol_only_provenance_refs,
                LabelRefTable::new(),
                provenance_imports.clone(),
            ),
            Err(ResolvedAstError::ImportProvenanceMismatch { .. })
        ));

        let mut origin_only_provenance_refs = NameRefTable::new();
        origin_only_provenance_refs.insert(NameRefEntry::new(
            ReferenceSite::new(ResolvedNodeId::new(0), range(primary_source_id, 0, 1), "P"),
            NameResolution::Resolved(SymbolRef::new(
                symbol_id(module_id("pkg", "dep"), "pred/0", "pkg::dep::pred/0"),
                range(primary_source_id, 0, 1),
            )),
            origin(primary_source_id, module_id("pkg", "main")).with_import_edge(first_import),
        ));
        assert!(matches!(
            ResolvedAst::try_new(
                primary_source_id,
                module_id("pkg", "main"),
                provenance_arena,
                origin_only_provenance_refs,
                LabelRefTable::new(),
                provenance_imports,
            ),
            Err(ResolvedAstError::ImportProvenanceMismatch { .. })
        ));

        let stale_owner_arena = ResolvedArena::try_new(
            root,
            vec![ResolvedNode::new(
                SurfaceNodeKind::CompilationUnit,
                Vec::new(),
                origin(primary_source_id, module_id("pkg", "main")),
            )],
        )
        .unwrap();
        let mut stale_import_owner_imports = ResolvedImports::new();
        stale_import_owner_imports.push_import(ResolvedImport::new(
            ResolvedNodeId::new(99),
            range(primary_source_id, 0, 1),
            "import stale;",
            None,
            ImportResolution::Resolved(module_id("pkg", "stale")),
            origin(primary_source_id, module_id("pkg", "main")),
        ));
        assert!(matches!(
            ResolvedAst::try_new(
                primary_source_id,
                module_id("pkg", "main"),
                stale_owner_arena.clone(),
                NameRefTable::new(),
                LabelRefTable::new(),
                stale_import_owner_imports,
            ),
            Err(ResolvedAstError::InvalidDirectiveOwner { .. })
        ));

        let mut stale_export_owner_imports = ResolvedImports::new();
        stale_export_owner_imports.push_export(ResolvedExport::new(
            ResolvedNodeId::new(99),
            range(primary_source_id, 0, 1),
            "export stale;",
            ExportTarget::Module(module_id("pkg", "stale")),
            origin(primary_source_id, module_id("pkg", "main")),
        ));
        assert!(matches!(
            ResolvedAst::try_new(
                primary_source_id,
                module_id("pkg", "main"),
                stale_owner_arena,
                NameRefTable::new(),
                LabelRefTable::new(),
                stale_export_owner_imports,
            ),
            Err(ResolvedAstError::InvalidDirectiveOwner { .. })
        ));
    }

    #[test]
    fn node_reference_keys_are_stable_for_equivalent_builds() {
        let first = reference_key_snapshot(source_id(11), module_id("pkg", "main"));
        let second = reference_key_snapshot(source_id(11), module_id("pkg", "main"));

        assert_eq!(first, second);
        assert_eq!(
            first,
            vec![
                Some(NodeReferenceKey::Name(NameRefId::new(0))),
                Some(NodeReferenceKey::Label(LabelRefId::new(0))),
                Some(NodeReferenceKey::Import(ResolvedImportId::new(0))),
                Some(NodeReferenceKey::Export(ResolvedExportId::new(0))),
            ]
        );
    }

    #[test]
    fn table_and_import_export_iteration_is_stable() {
        let source_id = source_id(9);
        let module = module_id("pkg", "main");
        let origin = origin(source_id, module);
        let node = ResolvedNodeId::new(0);
        let mut name_refs = NameRefTable::new();
        let first_name = name_refs.insert(NameRefEntry::new(
            ReferenceSite::new(node, range(source_id, 0, 1), "A"),
            NameResolution::Unresolved(UnresolvedNameRef::new(
                "A",
                range(source_id, 0, 1),
                NameLookupClass::Symbol,
            )),
            origin.clone(),
        ));
        let second_name = name_refs.insert(NameRefEntry::new(
            ReferenceSite::new(node, range(source_id, 1, 2), "B"),
            NameResolution::Unresolved(UnresolvedNameRef::new(
                "B",
                range(source_id, 1, 2),
                NameLookupClass::Symbol,
            )),
            origin.clone(),
        ));

        let mut label_refs = LabelRefTable::new();
        let first_label = label_refs.insert(LabelRefEntry::new(
            ReferenceSite::new(node, range(source_id, 2, 3), "L1"),
            LabelResolution::Unresolved(UnresolvedLabelRef::new(
                "L1",
                range(source_id, 2, 3),
                LabelKind::Theorem,
            )),
            origin.clone(),
        ));
        let second_label = label_refs.insert(LabelRefEntry::new(
            ReferenceSite::new(node, range(source_id, 3, 4), "L2"),
            LabelResolution::Unresolved(UnresolvedLabelRef::new(
                "L2",
                range(source_id, 3, 4),
                LabelKind::Theorem,
            )),
            origin.clone(),
        ));

        let mut imports = ResolvedImports::new();
        let first_import = imports.push_import(ResolvedImport::new(
            ResolvedNodeId::new(0),
            range(source_id, 4, 5),
            "import a;",
            None,
            ImportResolution::Resolved(module_id("pkg", "a")),
            origin.clone(),
        ));
        let second_import = imports.push_import(ResolvedImport::new(
            ResolvedNodeId::new(1),
            range(source_id, 5, 6),
            "import b;",
            None,
            ImportResolution::Resolved(module_id("pkg", "b")),
            origin.clone(),
        ));
        let first_export = imports.push_export(ResolvedExport::new(
            ResolvedNodeId::new(2),
            range(source_id, 6, 7),
            "export a;",
            ExportTarget::Module(module_id("pkg", "a")),
            origin.clone(),
        ));
        let second_export = imports.push_export(ResolvedExport::new(
            ResolvedNodeId::new(3),
            range(source_id, 7, 8),
            "export b;",
            ExportTarget::Module(module_id("pkg", "b")),
            origin,
        ));

        assert_eq!(
            name_refs.iter().map(|(id, _)| id).collect::<Vec<_>>(),
            vec![first_name, second_name]
        );
        assert_eq!(
            label_refs.iter().map(|(id, _)| id).collect::<Vec<_>>(),
            vec![first_label, second_label]
        );
        assert_eq!(
            imports.imports().map(|(id, _)| id).collect::<Vec<_>>(),
            vec![first_import, second_import]
        );
        assert_eq!(
            imports.exports().map(|(id, _)| id).collect::<Vec<_>>(),
            vec![first_export, second_export]
        );
    }

    fn reference_key_snapshot(
        source_id: SourceId,
        module: ModuleId,
    ) -> Vec<Option<NodeReferenceKey>> {
        let origin = origin(source_id, module.clone());
        let mut name_refs = NameRefTable::new();
        let mut label_refs = LabelRefTable::new();
        let mut imports = ResolvedImports::new();
        let name_id = name_refs.insert(NameRefEntry::new(
            ReferenceSite::new(ResolvedNodeId::new(0), range(source_id, 0, 1), "N"),
            NameResolution::Unresolved(UnresolvedNameRef::new(
                "N",
                range(source_id, 0, 1),
                NameLookupClass::Symbol,
            )),
            origin.clone(),
        ));
        let label_id = label_refs.insert(LabelRefEntry::new(
            ReferenceSite::new(ResolvedNodeId::new(1), range(source_id, 1, 2), "L"),
            LabelResolution::Unresolved(UnresolvedLabelRef::new(
                "L",
                range(source_id, 1, 2),
                LabelKind::Theorem,
            )),
            origin.clone(),
        ));
        let import_id = imports.push_import(ResolvedImport::new(
            ResolvedNodeId::new(2),
            range(source_id, 2, 3),
            "import dep;",
            None,
            ImportResolution::Resolved(module_id("pkg", "dep")),
            origin.clone(),
        ));
        let export_id = imports.push_export(ResolvedExport::new(
            ResolvedNodeId::new(3),
            range(source_id, 3, 4),
            "export dep;",
            ExportTarget::Module(module_id("pkg", "dep")),
            origin.clone(),
        ));

        let mut builder = ResolvedArenaBuilder::new();
        let name_node = builder
            .push(
                ResolvedNode::new(SurfaceNodeKind::Reference, Vec::new(), origin.clone())
                    .with_reference_key(NodeReferenceKey::Name(name_id)),
            )
            .unwrap();
        let label_node = builder
            .push(
                ResolvedNode::new(SurfaceNodeKind::AnnotationLabel, Vec::new(), origin.clone())
                    .with_reference_key(NodeReferenceKey::Label(label_id)),
            )
            .unwrap();
        let import_node = builder
            .push(
                ResolvedNode::new(SurfaceNodeKind::ImportItem, Vec::new(), origin.clone())
                    .with_reference_key(NodeReferenceKey::Import(import_id)),
            )
            .unwrap();
        let export_node = builder
            .push(
                ResolvedNode::new(SurfaceNodeKind::ExportItem, Vec::new(), origin.clone())
                    .with_reference_key(NodeReferenceKey::Export(export_id)),
            )
            .unwrap();
        let root = builder
            .push(ResolvedNode::new(
                SurfaceNodeKind::CompilationUnit,
                vec![name_node, label_node, import_node, export_node],
                origin,
            ))
            .unwrap();
        let arena = builder.finish(root).unwrap();
        ResolvedAst::try_new(source_id, module, arena, name_refs, label_refs, imports)
            .unwrap()
            .nodes()
            .iter()
            .take(4)
            .map(|(_, node)| node.reference_key())
            .collect()
    }

    fn source_id(seed: u8) -> SourceId {
        let snapshot_id = snapshot_id(seed);
        InMemorySessionIdAllocator::new()
            .next_source_id(snapshot_id)
            .unwrap()
    }

    fn source_id_pair(seed: u8) -> (SourceId, SourceId) {
        let snapshot_id = snapshot_id(seed);
        let allocator = InMemorySessionIdAllocator::new();
        (
            allocator.next_source_id(snapshot_id).unwrap(),
            allocator.next_source_id(snapshot_id).unwrap(),
        )
    }

    fn snapshot_id(seed: u8) -> BuildSnapshotId {
        let hex = format!("{seed:02x}").repeat(Hash::BYTE_LEN);
        BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{hex}"
        ))
        .unwrap()
    }

    const fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
        SourceRange {
            source_id,
            start,
            end,
        }
    }

    fn origin(source_id: SourceId, module_id: ModuleId) -> SemanticOrigin {
        SemanticOrigin::new(
            source_id,
            module_id,
            SourceAnchor::Range(range(source_id, 0, 1)),
            vec![0],
        )
    }

    fn module_id(package: &str, path: &str) -> ModuleId {
        ModuleId::new(PackageId::new(package), ModulePath::new(path))
    }

    fn symbol_id(module: ModuleId, local: &str, fqn: &str) -> SymbolId {
        SymbolId::new(
            module,
            LocalSymbolId::new(local),
            FullyQualifiedName::new(fqn),
        )
    }
}
