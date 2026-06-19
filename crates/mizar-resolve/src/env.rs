//! Symbol environment data shapes.

use crate::resolved_ast::{
    FullyQualifiedName, LabelKind, LabelOriginPath, LabelRefId, ModuleId, NameRefId, RecoveryState,
    ResolvedExportId, ResolvedImportId, SemanticOrigin, SymbolId,
};
use mizar_session::{GeneratedSpanAnchor, SourceAnchor, SourceId, SourceRange};
use std::cmp::Ordering;
use std::fmt::Write as _;

/// Stable id for a definition entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DefinitionId(usize);

impl DefinitionId {
    const fn new(index: usize) -> Self {
        Self(index)
    }

    /// Returns the zero-based local index.
    pub const fn index(self) -> usize {
        self.0
    }
}

/// Stable id for an overload group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OverloadGroupId(usize);

impl OverloadGroupId {
    const fn new(index: usize) -> Self {
        Self(index)
    }

    /// Returns the zero-based local index.
    pub const fn index(self) -> usize {
        self.0
    }
}

/// Stable id for a registration declaration entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RegistrationId(usize);

impl RegistrationId {
    const fn new(index: usize) -> Self {
        Self(index)
    }

    /// Returns the zero-based local index.
    pub const fn index(self) -> usize {
        self.0
    }
}

/// Stable id for a namespace graph node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NamespaceNodeId(usize);

impl NamespaceNodeId {
    const fn new(index: usize) -> Self {
        Self(index)
    }

    /// Returns the zero-based local index.
    pub const fn index(self) -> usize {
        self.0
    }
}

/// Stable id for a namespace graph edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NamespaceEdgeId(usize);

impl NamespaceEdgeId {
    const fn new(index: usize) -> Self {
        Self(index)
    }

    /// Returns the zero-based local index.
    pub const fn index(self) -> usize {
        self.0
    }
}

/// Stable id for a declaration dependency edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeclarationDependencyId(usize);

impl DeclarationDependencyId {
    const fn new(index: usize) -> Self {
        Self(index)
    }

    /// Returns the zero-based local index.
    pub const fn index(self) -> usize {
        self.0
    }
}

/// Stable id for a source contribution record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceContributionId(usize);

impl SourceContributionId {
    const fn new(index: usize) -> Self {
        Self(index)
    }

    /// Returns the zero-based local index.
    pub const fn index(self) -> usize {
        self.0
    }
}

/// Stable id for a dependency module summary entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModuleSummaryId(usize);

impl ModuleSummaryId {
    const fn new(index: usize) -> Self {
        Self(index)
    }

    /// Returns the zero-based local index.
    pub const fn index(self) -> usize {
        self.0
    }
}

/// Stable id for a diagnostic or failure anchor preserved by the resolver.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DiagnosticAnchorId(usize);

impl DiagnosticAnchorId {
    /// Creates an opaque diagnostic anchor id from a local index.
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    /// Returns the zero-based local index.
    pub const fn index(self) -> usize {
        self.0
    }
}

/// Namespace path used for visible-name projections.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NamespacePath(String);

impl NamespacePath {
    /// Creates a namespace path projection.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the namespace path.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Opaque resolver shell id for future signature and declaration payloads.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResolverShellId(String);

impl ResolverShellId {
    /// Creates a resolver-owned shell id.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the shell id spelling.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Opaque dependency summary identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModuleSummaryIdentity(String);

impl ModuleSummaryIdentity {
    /// Creates a dependency summary identity or fingerprint placeholder.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the opaque identity string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Symbol kind known to the resolver index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SymbolKind {
    /// Predicate declaration.
    Predicate,
    /// Functor declaration.
    Functor,
    /// Mode declaration.
    Mode,
    /// Attribute declaration.
    Attribute,
    /// Structure declaration.
    Structure,
    /// Selector declaration.
    Selector,
    /// Registration declaration.
    Registration,
    /// Theorem-like declaration represented as a symbol.
    Theorem,
    /// Built-in resolver-visible symbol.
    Builtin,
}

/// Resolver-level visibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Visibility {
    /// Visible only inside the declaring module.
    Private,
    /// Visible to importers when exported.
    Public,
}

/// Export status of a resolver entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ExportStatus {
    /// Not exported from the current environment.
    LocalOnly,
    /// Exported by the current module.
    Exported,
    /// Re-exported through a facade.
    ReExported,
}

/// Opaque signature shell that does not encode checker-owned type facts.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SignatureShell {
    /// Signature collection has not populated this shell yet.
    Pending,
    /// Resolver-owned opaque shell payload.
    Opaque {
        /// Schema or producer name.
        schema: String,
        /// Opaque payload identity.
        payload: String,
    },
    /// Malformed or recovered signature shell.
    Malformed {
        /// Crate-local failure class.
        class: String,
    },
}

/// Resolver-owned relation metadata kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum RelationKind {
    /// Synonym relation.
    Synonym,
    /// Antonym relation.
    Antonym,
    /// Redefinition relation.
    Redefinition,
}

/// Resolver-owned relation metadata.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RelationMetadata {
    kind: RelationKind,
    target: SymbolId,
}

impl RelationMetadata {
    /// Creates relation metadata.
    pub const fn new(kind: RelationKind, target: SymbolId) -> Self {
        Self { kind, target }
    }

    /// Returns the relation kind.
    pub const fn kind(&self) -> RelationKind {
        self.kind
    }

    /// Returns the relation target.
    pub const fn target(&self) -> &SymbolId {
        &self.target
    }
}

/// Symbol index entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolEntry {
    symbol: SymbolId,
    kind: SymbolKind,
    visibility: Visibility,
    export_status: ExportStatus,
    namespace: NamespacePath,
    primary_spelling: String,
    notation_spelling: Option<String>,
    origin: SemanticOrigin,
    contribution: SourceContributionId,
    signature: Option<SignatureShell>,
    relations: Vec<RelationMetadata>,
}

impl SymbolEntry {
    /// Creates a symbol entry with local-only visibility defaults.
    pub fn new(
        symbol: SymbolId,
        kind: SymbolKind,
        namespace: NamespacePath,
        primary_spelling: impl Into<String>,
        origin: SemanticOrigin,
        contribution: SourceContributionId,
    ) -> Self {
        Self {
            symbol,
            kind,
            visibility: Visibility::Private,
            export_status: ExportStatus::LocalOnly,
            namespace,
            primary_spelling: primary_spelling.into(),
            notation_spelling: None,
            origin,
            contribution,
            signature: None,
            relations: Vec::new(),
        }
    }

    /// Sets visibility.
    pub const fn with_visibility(mut self, visibility: Visibility) -> Self {
        self.visibility = visibility;
        self
    }

    /// Sets export status.
    pub const fn with_export_status(mut self, export_status: ExportStatus) -> Self {
        self.export_status = export_status;
        self
    }

    /// Sets notation spelling.
    pub fn with_notation_spelling(mut self, notation_spelling: impl Into<String>) -> Self {
        self.notation_spelling = Some(notation_spelling.into());
        self
    }

    /// Sets an opaque signature shell.
    pub fn with_signature(mut self, signature: SignatureShell) -> Self {
        self.signature = Some(signature);
        self
    }

    /// Sets resolver-owned relation metadata.
    pub fn with_relations(mut self, mut relations: Vec<RelationMetadata>) -> Self {
        relations.sort();
        self.relations = relations;
        self
    }

    /// Returns the symbol id.
    pub const fn symbol(&self) -> &SymbolId {
        &self.symbol
    }

    /// Returns the symbol kind.
    pub const fn kind(&self) -> SymbolKind {
        self.kind
    }

    /// Returns the visibility.
    pub const fn visibility(&self) -> Visibility {
        self.visibility
    }

    /// Returns the export status.
    pub const fn export_status(&self) -> ExportStatus {
        self.export_status
    }

    /// Returns the namespace path.
    pub const fn namespace(&self) -> &NamespacePath {
        &self.namespace
    }

    /// Returns the primary spelling.
    pub fn primary_spelling(&self) -> &str {
        &self.primary_spelling
    }

    /// Returns notation spelling when present.
    pub fn notation_spelling(&self) -> Option<&str> {
        self.notation_spelling.as_deref()
    }

    /// Returns normalized origin.
    pub const fn origin(&self) -> &SemanticOrigin {
        &self.origin
    }

    /// Returns the source contribution id.
    pub const fn contribution(&self) -> SourceContributionId {
        self.contribution
    }

    /// Returns the opaque signature shell.
    pub const fn signature(&self) -> Option<&SignatureShell> {
        self.signature.as_ref()
    }

    /// Returns relation metadata.
    pub fn relations(&self) -> &[RelationMetadata] {
        &self.relations
    }
}

impl Ord for SymbolEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.symbol.cmp(other.symbol())
    }
}

impl PartialOrd for SymbolEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Index of all environment-visible symbols.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SymbolIndex {
    entries: Vec<SymbolEntry>,
}

impl SymbolIndex {
    /// Creates an empty symbol index.
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Inserts an entry and keeps deterministic order.
    pub fn insert(&mut self, entry: SymbolEntry) {
        self.entries.push(entry);
        self.entries.sort();
    }

    /// Returns a symbol entry by id.
    pub fn get(&self, symbol: &SymbolId) -> Option<&SymbolEntry> {
        self.entries.iter().find(|entry| entry.symbol() == symbol)
    }

    /// Returns an entry by fully qualified name.
    pub fn by_fqn(&self, fqn: &FullyQualifiedName) -> Option<&SymbolEntry> {
        self.entries
            .iter()
            .find(|entry| entry.symbol().fqn() == fqn)
    }

    /// Returns visible candidates by namespace and spelling.
    pub fn visible_candidates(
        &self,
        namespace: &NamespacePath,
        spelling: &str,
    ) -> Vec<&SymbolEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.namespace() == namespace && entry.primary_spelling() == spelling)
            .collect()
    }

    /// Returns exported entries defined by a module.
    pub fn exported_by_module(&self, module: &ModuleId) -> Vec<&SymbolEntry> {
        self.entries
            .iter()
            .filter(|entry| {
                entry.symbol().module() == module
                    && !matches!(entry.export_status(), ExportStatus::LocalOnly)
            })
            .collect()
    }

    /// Returns entries produced by a contribution.
    pub fn by_contribution(&self, contribution: SourceContributionId) -> Vec<&SymbolEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.contribution() == contribution)
            .collect()
    }

    /// Iterates entries in deterministic symbol-id order.
    pub fn iter(&self) -> impl Iterator<Item = &SymbolEntry> {
        self.entries.iter()
    }

    /// Returns the number of entries.
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether the index is empty.
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Label index entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelEntry {
    origin_path: LabelOriginPath,
    kind: LabelKind,
    visibility: Visibility,
    export_status: ExportStatus,
    namespace: NamespacePath,
    primary_spelling: String,
    origin: SemanticOrigin,
    contribution: SourceContributionId,
    recovery: RecoveryState,
}

impl LabelEntry {
    /// Creates a label entry.
    pub fn new(
        origin_path: LabelOriginPath,
        kind: LabelKind,
        namespace: NamespacePath,
        primary_spelling: impl Into<String>,
        origin: SemanticOrigin,
        contribution: SourceContributionId,
    ) -> Self {
        let recovery = if origin.is_recovered() {
            RecoveryState::Recovered
        } else {
            RecoveryState::Normal
        };
        Self {
            origin_path,
            kind,
            visibility: Visibility::Private,
            export_status: ExportStatus::LocalOnly,
            namespace,
            primary_spelling: primary_spelling.into(),
            origin,
            contribution,
            recovery,
        }
    }

    /// Sets visibility.
    pub const fn with_visibility(mut self, visibility: Visibility) -> Self {
        self.visibility = visibility;
        self
    }

    /// Sets export status.
    pub const fn with_export_status(mut self, export_status: ExportStatus) -> Self {
        self.export_status = export_status;
        self
    }

    /// Returns the label origin path.
    pub const fn origin_path(&self) -> &LabelOriginPath {
        &self.origin_path
    }

    /// Returns the label kind.
    pub const fn kind(&self) -> LabelKind {
        self.kind
    }

    /// Returns the visibility.
    pub const fn visibility(&self) -> Visibility {
        self.visibility
    }

    /// Returns the export status.
    pub const fn export_status(&self) -> ExportStatus {
        self.export_status
    }

    /// Returns the namespace path.
    pub const fn namespace(&self) -> &NamespacePath {
        &self.namespace
    }

    /// Returns the primary spelling.
    pub fn primary_spelling(&self) -> &str {
        &self.primary_spelling
    }

    /// Returns normalized origin.
    pub const fn origin(&self) -> &SemanticOrigin {
        &self.origin
    }

    /// Returns the source contribution id.
    pub const fn contribution(&self) -> SourceContributionId {
        self.contribution
    }

    /// Returns recovered-shell state.
    pub const fn recovery(&self) -> RecoveryState {
        self.recovery
    }
}

impl Ord for LabelEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.origin_path
            .cmp(other.origin_path())
            .then_with(|| self.kind.cmp(&other.kind()))
    }
}

impl PartialOrd for LabelEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Index of visible labels.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LabelIndex {
    entries: Vec<LabelEntry>,
}

impl LabelIndex {
    /// Creates an empty label index.
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Inserts an entry and keeps deterministic order.
    pub fn insert(&mut self, entry: LabelEntry) {
        self.entries.push(entry);
        self.entries.sort();
    }

    /// Returns a label by origin path.
    pub fn get(&self, origin_path: &LabelOriginPath) -> Option<&LabelEntry> {
        self.entries
            .iter()
            .find(|entry| entry.origin_path() == origin_path)
    }

    /// Returns visible label candidates by namespace and spelling.
    pub fn visible_candidates(
        &self,
        namespace: &NamespacePath,
        spelling: &str,
    ) -> Vec<&LabelEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.namespace() == namespace && entry.primary_spelling() == spelling)
            .collect()
    }

    /// Returns labels produced by a contribution.
    pub fn by_contribution(&self, contribution: SourceContributionId) -> Vec<&LabelEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.contribution() == contribution)
            .collect()
    }

    /// Iterates labels in deterministic origin-path order.
    pub fn iter(&self) -> impl Iterator<Item = &LabelEntry> {
        self.entries.iter()
    }

    /// Returns the number of entries.
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether the index is empty.
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Declaration definition kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum DefinitionKind {
    /// Predicate definition shell.
    Predicate,
    /// Functor definition shell.
    Functor,
    /// Mode definition shell.
    Mode,
    /// Attribute definition shell.
    Attribute,
    /// Structure definition shell.
    Structure,
    /// Registration definition shell.
    Registration,
}

/// Duplicate or conflict classification for declaration collection.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum DeclarationConflictClass {
    /// Duplicate declaration spelling.
    DuplicateSpelling,
    /// Illegal overload group.
    IllegalOverloadGroup,
    /// Recovered declaration shell.
    RecoveredShell,
}

/// Resolver-owned definition shell inserted into `DefinitionIndex`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinitionShell {
    symbol: SymbolId,
    kind: DefinitionKind,
    visibility: Visibility,
    parameters: Vec<ResolverShellId>,
    binders: Vec<ResolverShellId>,
    arity: Option<u32>,
    notation_shape: Option<String>,
    doc_attachment: Option<ResolverShellId>,
    origin: SemanticOrigin,
    contribution: SourceContributionId,
    conflict: Option<DeclarationConflictClass>,
    dependencies: Vec<DeclarationDependencyId>,
    signature: Option<SignatureShell>,
}

impl DefinitionShell {
    /// Creates a definition shell.
    pub fn new(
        symbol: SymbolId,
        kind: DefinitionKind,
        origin: SemanticOrigin,
        contribution: SourceContributionId,
    ) -> Self {
        Self {
            symbol,
            kind,
            visibility: Visibility::Private,
            parameters: Vec::new(),
            binders: Vec::new(),
            arity: None,
            notation_shape: None,
            doc_attachment: None,
            origin,
            contribution,
            conflict: None,
            dependencies: Vec::new(),
            signature: None,
        }
    }

    /// Sets visibility.
    pub const fn with_visibility(mut self, visibility: Visibility) -> Self {
        self.visibility = visibility;
        self
    }

    /// Sets parameter shell ids.
    pub fn with_parameters(mut self, parameters: Vec<ResolverShellId>) -> Self {
        self.parameters = parameters;
        self
    }

    /// Sets binder shell ids.
    pub fn with_binders(mut self, binders: Vec<ResolverShellId>) -> Self {
        self.binders = binders;
        self
    }

    /// Sets syntactic arity.
    pub const fn with_arity(mut self, arity: u32) -> Self {
        self.arity = Some(arity);
        self
    }

    /// Sets notation shape.
    pub fn with_notation_shape(mut self, notation_shape: impl Into<String>) -> Self {
        self.notation_shape = Some(notation_shape.into());
        self
    }

    /// Sets source doc/comment attachment id.
    pub fn with_doc_attachment(mut self, doc_attachment: ResolverShellId) -> Self {
        self.doc_attachment = Some(doc_attachment);
        self
    }

    /// Sets duplicate or conflict classification.
    pub fn with_conflict(mut self, conflict: DeclarationConflictClass) -> Self {
        self.conflict = Some(conflict);
        self
    }

    /// Sets syntactic dependency references.
    pub fn with_dependencies(mut self, mut dependencies: Vec<DeclarationDependencyId>) -> Self {
        dependencies.sort();
        self.dependencies = dependencies;
        self
    }

    /// Sets opaque signature payload.
    pub fn with_signature(mut self, signature: SignatureShell) -> Self {
        self.signature = Some(signature);
        self
    }
}

/// Stored definition entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinitionEntry {
    id: DefinitionId,
    shell: DefinitionShell,
}

impl DefinitionEntry {
    /// Returns the definition id.
    pub const fn id(&self) -> DefinitionId {
        self.id
    }

    /// Returns the symbol id.
    pub const fn symbol(&self) -> &SymbolId {
        &self.shell.symbol
    }

    /// Returns the definition kind.
    pub const fn kind(&self) -> DefinitionKind {
        self.shell.kind
    }

    /// Returns visibility.
    pub const fn visibility(&self) -> Visibility {
        self.shell.visibility
    }

    /// Returns parameter shell ids.
    pub fn parameters(&self) -> &[ResolverShellId] {
        &self.shell.parameters
    }

    /// Returns binder shell ids.
    pub fn binders(&self) -> &[ResolverShellId] {
        &self.shell.binders
    }

    /// Returns syntactic arity.
    pub const fn arity(&self) -> Option<u32> {
        self.shell.arity
    }

    /// Returns notation shape.
    pub fn notation_shape(&self) -> Option<&str> {
        self.shell.notation_shape.as_deref()
    }

    /// Returns doc/comment attachment id.
    pub const fn doc_attachment(&self) -> Option<&ResolverShellId> {
        self.shell.doc_attachment.as_ref()
    }

    /// Returns normalized origin.
    pub const fn origin(&self) -> &SemanticOrigin {
        &self.shell.origin
    }

    /// Returns source contribution id.
    pub const fn contribution(&self) -> SourceContributionId {
        self.shell.contribution
    }

    /// Returns conflict classification.
    pub const fn conflict(&self) -> Option<&DeclarationConflictClass> {
        self.shell.conflict.as_ref()
    }

    /// Returns syntactic dependency references.
    pub fn dependencies(&self) -> &[DeclarationDependencyId] {
        &self.shell.dependencies
    }

    /// Returns opaque signature payload.
    pub const fn signature(&self) -> Option<&SignatureShell> {
        self.shell.signature.as_ref()
    }
}

impl Ord for DefinitionEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.symbol()
            .cmp(other.symbol())
            .then_with(|| self.id.cmp(&other.id))
    }
}

impl PartialOrd for DefinitionEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Index of declaration definitions.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DefinitionIndex {
    next_id: usize,
    entries: Vec<DefinitionEntry>,
}

impl DefinitionIndex {
    /// Creates an empty definition index.
    pub const fn new() -> Self {
        Self {
            next_id: 0,
            entries: Vec::new(),
        }
    }

    /// Inserts a definition shell and returns its stable id.
    pub fn insert(&mut self, shell: DefinitionShell) -> DefinitionId {
        let id = DefinitionId::new(self.next_id);
        self.next_id += 1;
        self.entries.push(DefinitionEntry { id, shell });
        self.entries.sort();
        id
    }

    /// Returns a definition by id.
    pub fn get(&self, id: DefinitionId) -> Option<&DefinitionEntry> {
        self.entries.iter().find(|entry| entry.id() == id)
    }

    /// Returns a definition by symbol id.
    pub fn by_symbol(&self, symbol: &SymbolId) -> Option<&DefinitionEntry> {
        self.entries.iter().find(|entry| entry.symbol() == symbol)
    }

    /// Returns definitions produced by a contribution.
    pub fn by_contribution(&self, contribution: SourceContributionId) -> Vec<&DefinitionEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.contribution() == contribution)
            .collect()
    }

    /// Iterates definitions in deterministic symbol-id order.
    pub fn iter(&self) -> impl Iterator<Item = &DefinitionEntry> {
        self.entries.iter()
    }

    /// Returns the number of definitions.
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether the index is empty.
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Overload grouping key.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OverloadKey {
    namespace: NamespacePath,
    spelling: String,
    kind: SymbolKind,
    arity: Option<u32>,
}

impl OverloadKey {
    /// Creates an overload grouping key.
    pub fn new(
        namespace: NamespacePath,
        spelling: impl Into<String>,
        kind: SymbolKind,
        arity: Option<u32>,
    ) -> Self {
        Self {
            namespace,
            spelling: spelling.into(),
            kind,
            arity,
        }
    }

    /// Returns the namespace path.
    pub const fn namespace(&self) -> &NamespacePath {
        &self.namespace
    }

    /// Returns the surface spelling or notation slot.
    pub fn spelling(&self) -> &str {
        &self.spelling
    }

    /// Returns the symbol kind family.
    pub const fn kind(&self) -> SymbolKind {
        self.kind
    }

    /// Returns syntactic arity when available.
    pub const fn arity(&self) -> Option<u32> {
        self.arity
    }
}

/// Overload candidate group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverloadGroup {
    id: OverloadGroupId,
    key: OverloadKey,
    candidates: Vec<SymbolId>,
    diagnostics: Vec<DiagnosticAnchorId>,
    contribution: SourceContributionId,
}

impl OverloadGroup {
    /// Returns the overload group id.
    pub const fn id(&self) -> OverloadGroupId {
        self.id
    }

    /// Returns the grouping key.
    pub const fn key(&self) -> &OverloadKey {
        &self.key
    }

    /// Returns candidates in deterministic symbol-id order.
    pub fn candidates(&self) -> &[SymbolId] {
        &self.candidates
    }

    /// Returns illegal grouping diagnostics or failure anchors.
    pub fn diagnostics(&self) -> &[DiagnosticAnchorId] {
        &self.diagnostics
    }

    /// Returns the source contribution id.
    pub const fn contribution(&self) -> SourceContributionId {
        self.contribution
    }
}

impl Ord for OverloadGroup {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key
            .cmp(other.key())
            .then_with(|| self.id.cmp(&other.id))
    }
}

impl PartialOrd for OverloadGroup {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Index of overload groups before winner selection.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OverloadIndex {
    next_id: usize,
    groups: Vec<OverloadGroup>,
}

impl OverloadIndex {
    /// Creates an empty overload index.
    pub const fn new() -> Self {
        Self {
            next_id: 0,
            groups: Vec::new(),
        }
    }

    /// Inserts a group and returns its stable id.
    pub fn insert(
        &mut self,
        key: OverloadKey,
        mut candidates: Vec<SymbolId>,
        contribution: SourceContributionId,
    ) -> OverloadGroupId {
        let id = OverloadGroupId::new(self.next_id);
        self.next_id += 1;
        candidates.sort();
        self.groups.push(OverloadGroup {
            id,
            key,
            candidates,
            diagnostics: Vec::new(),
            contribution,
        });
        self.groups.sort();
        id
    }

    /// Records an illegal grouping diagnostic for an overload group.
    pub fn add_diagnostic(&mut self, id: OverloadGroupId, diagnostic: DiagnosticAnchorId) -> bool {
        let Some(group) = self.groups.iter_mut().find(|group| group.id() == id) else {
            return false;
        };
        insert_sorted_unique(&mut group.diagnostics, diagnostic);
        true
    }

    /// Returns a group by id.
    pub fn get(&self, id: OverloadGroupId) -> Option<&OverloadGroup> {
        self.groups.iter().find(|group| group.id() == id)
    }

    /// Returns a group by key.
    pub fn by_key(&self, key: &OverloadKey) -> Option<&OverloadGroup> {
        self.groups.iter().find(|group| group.key() == key)
    }

    /// Returns groups produced by a contribution.
    pub fn by_contribution(&self, contribution: SourceContributionId) -> Vec<&OverloadGroup> {
        self.groups
            .iter()
            .filter(|group| group.contribution() == contribution)
            .collect()
    }

    /// Iterates groups in deterministic key order.
    pub fn iter(&self) -> impl Iterator<Item = &OverloadGroup> {
        self.groups.iter()
    }

    /// Returns the number of groups.
    pub const fn len(&self) -> usize {
        self.groups.len()
    }

    /// Returns whether the index is empty.
    pub const fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }
}

/// Registration declaration kind before checker activation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum RegistrationKind {
    /// Cluster registration.
    Cluster,
    /// Identify registration.
    Identify,
    /// Reduction registration.
    Reduction,
    /// Property registration.
    Property,
}

/// Registration declaration entry before checker activation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrationEntry {
    id: RegistrationId,
    symbol: Option<SymbolId>,
    kind: RegistrationKind,
    target: SignatureShell,
    visibility: Visibility,
    export_status: ExportStatus,
    origin: SemanticOrigin,
    contribution: SourceContributionId,
    dependencies: Vec<DeclarationDependencyId>,
    recovery: RecoveryState,
}

impl RegistrationEntry {
    /// Returns the registration id.
    pub const fn id(&self) -> RegistrationId {
        self.id
    }

    /// Returns the optional registration symbol id.
    pub const fn symbol(&self) -> Option<&SymbolId> {
        self.symbol.as_ref()
    }

    /// Returns the registration kind.
    pub const fn kind(&self) -> RegistrationKind {
        self.kind
    }

    /// Returns the opaque syntactic target shell.
    pub const fn target(&self) -> &SignatureShell {
        &self.target
    }

    /// Returns visibility.
    pub const fn visibility(&self) -> Visibility {
        self.visibility
    }

    /// Returns export status.
    pub const fn export_status(&self) -> ExportStatus {
        self.export_status
    }

    /// Returns normalized origin.
    pub const fn origin(&self) -> &SemanticOrigin {
        &self.origin
    }

    /// Returns source contribution id.
    pub const fn contribution(&self) -> SourceContributionId {
        self.contribution
    }

    /// Returns syntactic dependency references.
    pub fn dependencies(&self) -> &[DeclarationDependencyId] {
        &self.dependencies
    }

    /// Returns recovered-shell state.
    pub const fn recovery(&self) -> RecoveryState {
        self.recovery
    }

    /// Sets visibility metadata.
    pub fn set_visibility(&mut self, visibility: Visibility) -> &mut Self {
        self.visibility = visibility;
        self
    }

    /// Sets export status metadata.
    pub fn set_export_status(&mut self, export_status: ExportStatus) -> &mut Self {
        self.export_status = export_status;
        self
    }

    /// Sets syntactic dependency references.
    pub fn set_dependencies(
        &mut self,
        mut dependencies: Vec<DeclarationDependencyId>,
    ) -> &mut Self {
        dependencies.sort();
        dependencies.dedup();
        self.dependencies = dependencies;
        self
    }
}

impl Ord for RegistrationEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.symbol
            .as_ref()
            .cmp(&other.symbol.as_ref())
            .then_with(|| self.kind.cmp(&other.kind))
            .then_with(|| self.id.cmp(&other.id))
    }
}

impl PartialOrd for RegistrationEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Index of registration declarations before checker activation.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RegistrationIndex {
    next_id: usize,
    entries: Vec<RegistrationEntry>,
}

impl RegistrationIndex {
    /// Creates an empty registration index.
    pub const fn new() -> Self {
        Self {
            next_id: 0,
            entries: Vec::new(),
        }
    }

    /// Inserts a registration declaration and returns its stable id.
    pub fn insert(
        &mut self,
        symbol: Option<SymbolId>,
        kind: RegistrationKind,
        target: SignatureShell,
        origin: SemanticOrigin,
        contribution: SourceContributionId,
    ) -> RegistrationId {
        let id = RegistrationId::new(self.next_id);
        self.next_id += 1;
        let recovery = if origin.is_recovered() {
            RecoveryState::Recovered
        } else {
            RecoveryState::Normal
        };
        self.entries.push(RegistrationEntry {
            id,
            symbol,
            kind,
            target,
            visibility: Visibility::Private,
            export_status: ExportStatus::LocalOnly,
            origin,
            contribution,
            dependencies: Vec::new(),
            recovery,
        });
        self.entries.sort();
        id
    }

    /// Returns a mutable registration entry for builder-style setup.
    pub fn get_mut(&mut self, id: RegistrationId) -> Option<&mut RegistrationEntry> {
        self.entries.iter_mut().find(|entry| entry.id() == id)
    }

    /// Returns a registration by id.
    pub fn get(&self, id: RegistrationId) -> Option<&RegistrationEntry> {
        self.entries.iter().find(|entry| entry.id() == id)
    }

    /// Returns registrations produced by a contribution.
    pub fn by_contribution(&self, contribution: SourceContributionId) -> Vec<&RegistrationEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.contribution() == contribution)
            .collect()
    }

    /// Iterates registrations in deterministic order.
    pub fn iter(&self) -> impl Iterator<Item = &RegistrationEntry> {
        self.entries.iter()
    }

    /// Returns the number of registrations.
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether the index is empty.
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Namespace graph node kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum NamespaceNodeKind {
    /// Canonical module node.
    Module,
    /// Local import alias node.
    Alias,
    /// Namespace segment node.
    Segment,
    /// Built-in prelude root.
    BuiltinPrelude,
    /// Unresolved or recovered node.
    Unresolved,
}

/// Namespace graph edge kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum NamespaceEdgeKind {
    /// Import visibility edge.
    Import,
    /// Export facade edge.
    Export,
    /// Re-export facade edge.
    ReExport,
    /// Qualified namespace segment edge.
    Segment,
    /// Built-in prelude edge.
    BuiltinPrelude,
    /// Recovered unresolved edge.
    Unresolved,
}

/// Canonical target of a namespace edge.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum NamespaceTarget {
    /// Module target.
    Module(ModuleId),
    /// Symbol target.
    Symbol(SymbolId),
    /// Label target.
    Label(LabelOriginPath),
    /// Unresolved spelling preserved for recovery.
    Unresolved(String),
}

/// Namespace graph node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamespaceNode {
    id: NamespaceNodeId,
    kind: NamespaceNodeKind,
    module: Option<ModuleId>,
    spelling: String,
    contribution: SourceContributionId,
}

impl NamespaceNode {
    /// Returns the node id.
    pub const fn id(&self) -> NamespaceNodeId {
        self.id
    }

    /// Returns the node kind.
    pub const fn kind(&self) -> NamespaceNodeKind {
        self.kind
    }

    /// Returns the canonical module, if any.
    pub const fn module(&self) -> Option<&ModuleId> {
        self.module.as_ref()
    }

    /// Returns local spelling.
    pub fn spelling(&self) -> &str {
        &self.spelling
    }

    /// Returns source contribution id.
    pub const fn contribution(&self) -> SourceContributionId {
        self.contribution
    }
}

impl Ord for NamespaceNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.kind
            .cmp(&other.kind)
            .then_with(|| self.module.as_ref().cmp(&other.module.as_ref()))
            .then_with(|| {
                if self.kind == NamespaceNodeKind::Alias
                    && other.kind == NamespaceNodeKind::Alias
                    && self.module.is_some()
                    && other.module.is_some()
                {
                    Ordering::Equal
                } else {
                    self.spelling.cmp(&other.spelling)
                }
            })
            .then_with(|| self.id.cmp(&other.id))
    }
}

impl PartialOrd for NamespaceNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Namespace graph edge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamespaceEdge {
    id: NamespaceEdgeId,
    from: NamespaceNodeId,
    to: NamespaceNodeId,
    kind: NamespaceEdgeKind,
    anchor: SourceAnchor,
    contribution: SourceContributionId,
    visibility: Visibility,
    target: Option<NamespaceTarget>,
    local_spelling: Option<String>,
}

/// Namespace edge insertion data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamespaceEdgeSpec {
    endpoints: (NamespaceNodeId, NamespaceNodeId),
    kind: NamespaceEdgeKind,
    anchor: SourceAnchor,
    contribution: SourceContributionId,
    visibility: Visibility,
    target: Option<NamespaceTarget>,
    local_spelling: Option<String>,
}

impl NamespaceEdgeSpec {
    /// Creates namespace edge insertion data with private visibility.
    pub fn new(
        endpoints: (NamespaceNodeId, NamespaceNodeId),
        kind: NamespaceEdgeKind,
        anchor: SourceAnchor,
        contribution: SourceContributionId,
    ) -> Self {
        Self {
            endpoints,
            kind,
            anchor,
            contribution,
            visibility: Visibility::Private,
            target: None,
            local_spelling: None,
        }
    }

    /// Sets edge visibility.
    pub const fn with_visibility(mut self, visibility: Visibility) -> Self {
        self.visibility = visibility;
        self
    }

    /// Sets canonical target identity.
    pub fn with_target(mut self, target: NamespaceTarget) -> Self {
        self.target = Some(target);
        self
    }

    /// Sets local spelling.
    pub fn with_local_spelling(mut self, local_spelling: impl Into<String>) -> Self {
        self.local_spelling = Some(local_spelling.into());
        self
    }
}

impl NamespaceEdge {
    /// Returns the edge id.
    pub const fn id(&self) -> NamespaceEdgeId {
        self.id
    }

    /// Returns the source node.
    pub const fn from(&self) -> NamespaceNodeId {
        self.from
    }

    /// Returns the target node.
    pub const fn to(&self) -> NamespaceNodeId {
        self.to
    }

    /// Returns the edge kind.
    pub const fn kind(&self) -> NamespaceEdgeKind {
        self.kind
    }

    /// Returns source or recovered anchor.
    pub const fn anchor(&self) -> &SourceAnchor {
        &self.anchor
    }

    /// Returns source contribution id.
    pub const fn contribution(&self) -> SourceContributionId {
        self.contribution
    }

    /// Returns visibility.
    pub const fn visibility(&self) -> Visibility {
        self.visibility
    }

    /// Returns canonical target identity when resolved.
    pub const fn target(&self) -> Option<&NamespaceTarget> {
        self.target.as_ref()
    }

    /// Returns local spelling when it differs from canonical identity.
    pub fn local_spelling(&self) -> Option<&str> {
        self.local_spelling.as_deref()
    }
}

impl Ord for NamespaceEdge {
    fn cmp(&self, other: &Self) -> Ordering {
        self.from
            .cmp(&other.from)
            .then_with(|| self.to.cmp(&other.to))
            .then_with(|| self.kind.cmp(&other.kind))
            .then_with(|| self.id.cmp(&other.id))
    }
}

impl PartialOrd for NamespaceEdge {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Resolver-visible namespace graph.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NamespaceGraph {
    next_node_id: usize,
    next_edge_id: usize,
    nodes: Vec<NamespaceNode>,
    edges: Vec<NamespaceEdge>,
}

impl NamespaceGraph {
    /// Creates an empty namespace graph.
    pub const fn new() -> Self {
        Self {
            next_node_id: 0,
            next_edge_id: 0,
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Inserts a namespace node.
    pub fn insert_node(
        &mut self,
        kind: NamespaceNodeKind,
        module: Option<ModuleId>,
        spelling: impl Into<String>,
        contribution: SourceContributionId,
    ) -> NamespaceNodeId {
        let id = NamespaceNodeId::new(self.next_node_id);
        self.next_node_id += 1;
        self.nodes.push(NamespaceNode {
            id,
            kind,
            module,
            spelling: spelling.into(),
            contribution,
        });
        self.nodes.sort();
        id
    }

    /// Inserts a namespace edge.
    pub fn insert_edge(&mut self, spec: NamespaceEdgeSpec) -> NamespaceEdgeId {
        let id = NamespaceEdgeId::new(self.next_edge_id);
        self.next_edge_id += 1;
        self.edges.push(NamespaceEdge {
            id,
            from: spec.endpoints.0,
            to: spec.endpoints.1,
            kind: spec.kind,
            anchor: spec.anchor,
            contribution: spec.contribution,
            visibility: spec.visibility,
            target: spec.target,
            local_spelling: spec.local_spelling,
        });
        self.edges.sort();
        id
    }

    /// Returns a node by id.
    pub fn node(&self, id: NamespaceNodeId) -> Option<&NamespaceNode> {
        self.nodes.iter().find(|node| node.id() == id)
    }

    /// Returns an edge by id.
    pub fn edge(&self, id: NamespaceEdgeId) -> Option<&NamespaceEdge> {
        self.edges.iter().find(|edge| edge.id() == id)
    }

    /// Returns edges produced by a contribution.
    pub fn edges_by_contribution(&self, contribution: SourceContributionId) -> Vec<&NamespaceEdge> {
        self.edges
            .iter()
            .filter(|edge| edge.contribution() == contribution)
            .collect()
    }

    /// Iterates nodes in deterministic order.
    pub fn nodes(&self) -> impl Iterator<Item = &NamespaceNode> {
        self.nodes.iter()
    }

    /// Iterates edges in deterministic order.
    pub fn edges(&self) -> impl Iterator<Item = &NamespaceEdge> {
        self.edges.iter()
    }

    /// Returns the number of nodes.
    pub const fn node_len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the number of edges.
    pub const fn edge_len(&self) -> usize {
        self.edges.len()
    }

    /// Returns whether the graph has no nodes and no edges.
    pub const fn is_empty(&self) -> bool {
        self.nodes.is_empty() && self.edges.is_empty()
    }
}

/// Declaration dependency endpoint.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum DependencyEndpoint {
    /// Declaration symbol endpoint.
    Symbol(SymbolId),
    /// Import entry endpoint.
    Import(ResolvedImportId),
    /// Export entry endpoint.
    Export(ResolvedExportId),
    /// Namespace graph edge endpoint.
    NamespaceEdge(NamespaceEdgeId),
    /// Label endpoint.
    Label(LabelOriginPath),
    /// Unresolved name reference endpoint.
    UnresolvedName(NameRefId),
    /// Unresolved label reference endpoint.
    UnresolvedLabel(LabelRefId),
    /// Module endpoint.
    Module(ModuleId),
}

/// Declaration dependency kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum DeclarationDependencyKind {
    /// Import dependency edge.
    Import,
    /// Re-export dependency edge.
    ReExport,
    /// Signature mention dependency edge.
    SignatureMention,
    /// Synonym target dependency edge.
    SynonymTarget,
    /// Antonym target dependency edge.
    AntonymTarget,
    /// Redefinition target dependency edge.
    RedefinitionTarget,
    /// Registration mention dependency edge.
    RegistrationMention,
    /// Label citation dependency edge.
    LabelCitation,
}

/// Resolver-visible declaration dependency edge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclarationDependency {
    id: DeclarationDependencyId,
    source: DependencyEndpoint,
    target: DependencyEndpoint,
    kind: DeclarationDependencyKind,
    anchor: SourceAnchor,
    contribution: SourceContributionId,
}

impl DeclarationDependency {
    /// Returns the dependency id.
    pub const fn id(&self) -> DeclarationDependencyId {
        self.id
    }

    /// Returns the source endpoint.
    pub const fn source(&self) -> &DependencyEndpoint {
        &self.source
    }

    /// Returns the target endpoint.
    pub const fn target(&self) -> &DependencyEndpoint {
        &self.target
    }

    /// Returns the dependency kind.
    pub const fn kind(&self) -> DeclarationDependencyKind {
        self.kind
    }

    /// Returns the source or recovered anchor.
    pub const fn anchor(&self) -> &SourceAnchor {
        &self.anchor
    }

    /// Returns source contribution id.
    pub const fn contribution(&self) -> SourceContributionId {
        self.contribution
    }
}

impl Ord for DeclarationDependency {
    fn cmp(&self, other: &Self) -> Ordering {
        self.source
            .cmp(other.source())
            .then_with(|| self.target.cmp(other.target()))
            .then_with(|| self.kind.cmp(&other.kind))
            .then_with(|| self.id.cmp(&other.id))
    }
}

impl PartialOrd for DeclarationDependency {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Index of resolver-visible declaration dependency edges.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DeclarationDependencyIndex {
    next_id: usize,
    entries: Vec<DeclarationDependency>,
}

impl DeclarationDependencyIndex {
    /// Creates an empty declaration dependency index.
    pub const fn new() -> Self {
        Self {
            next_id: 0,
            entries: Vec::new(),
        }
    }

    /// Inserts a dependency edge and returns its stable id.
    pub fn insert(
        &mut self,
        source: DependencyEndpoint,
        target: DependencyEndpoint,
        kind: DeclarationDependencyKind,
        anchor: SourceAnchor,
        contribution: SourceContributionId,
    ) -> DeclarationDependencyId {
        let id = DeclarationDependencyId::new(self.next_id);
        self.next_id += 1;
        self.entries.push(DeclarationDependency {
            id,
            source,
            target,
            kind,
            anchor,
            contribution,
        });
        self.entries.sort();
        id
    }

    /// Returns a dependency by id.
    pub fn get(&self, id: DeclarationDependencyId) -> Option<&DeclarationDependency> {
        self.entries.iter().find(|entry| entry.id() == id)
    }

    /// Returns dependencies produced by a contribution.
    pub fn by_contribution(
        &self,
        contribution: SourceContributionId,
    ) -> Vec<&DeclarationDependency> {
        self.entries
            .iter()
            .filter(|entry| entry.contribution() == contribution)
            .collect()
    }

    /// Iterates dependency edges in deterministic order.
    pub fn iter(&self) -> impl Iterator<Item = &DeclarationDependency> {
        self.entries.iter()
    }

    /// Returns the number of dependency edges.
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether the index is empty.
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Import projection stored by `SymbolEnv`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportIndexEntry {
    import: ResolvedImportId,
    module: Option<ModuleId>,
    alias: Option<String>,
    contribution: SourceContributionId,
}

impl ImportIndexEntry {
    /// Creates an import projection entry.
    pub fn new(
        import: ResolvedImportId,
        module: Option<ModuleId>,
        alias: Option<String>,
        contribution: SourceContributionId,
    ) -> Self {
        Self {
            import,
            module,
            alias,
            contribution,
        }
    }

    /// Returns the resolved AST import id.
    pub const fn import(&self) -> ResolvedImportId {
        self.import
    }

    /// Returns the canonical target module if resolved.
    pub const fn module(&self) -> Option<&ModuleId> {
        self.module.as_ref()
    }

    /// Returns local alias spelling.
    pub fn alias(&self) -> Option<&str> {
        self.alias.as_deref()
    }

    /// Returns source contribution id.
    pub const fn contribution(&self) -> SourceContributionId {
        self.contribution
    }
}

impl Ord for ImportIndexEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.import.cmp(&other.import)
    }
}

impl PartialOrd for ImportIndexEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Import projection index.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResolvedImportIndex {
    entries: Vec<ImportIndexEntry>,
}

impl ResolvedImportIndex {
    /// Creates an empty import projection index.
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Inserts an import projection.
    pub fn insert(&mut self, entry: ImportIndexEntry) {
        self.entries.push(entry);
        self.entries.sort();
    }

    /// Returns an import projection by id.
    pub fn get(&self, import: ResolvedImportId) -> Option<&ImportIndexEntry> {
        self.entries.iter().find(|entry| entry.import() == import)
    }

    /// Returns imports produced by a contribution.
    pub fn by_contribution(&self, contribution: SourceContributionId) -> Vec<&ImportIndexEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.contribution() == contribution)
            .collect()
    }

    /// Iterates imports in deterministic id order.
    pub fn iter(&self) -> impl Iterator<Item = &ImportIndexEntry> {
        self.entries.iter()
    }

    /// Returns the number of entries.
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether the index is empty.
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Export projection stored by `SymbolEnv`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportIndexEntry {
    export: ResolvedExportId,
    target: Option<DependencyEndpoint>,
    contribution: SourceContributionId,
}

impl ExportIndexEntry {
    /// Creates an export projection entry.
    pub const fn new(
        export: ResolvedExportId,
        target: Option<DependencyEndpoint>,
        contribution: SourceContributionId,
    ) -> Self {
        Self {
            export,
            target,
            contribution,
        }
    }

    /// Returns the resolved AST export id.
    pub const fn export(&self) -> ResolvedExportId {
        self.export
    }

    /// Returns the canonical target when resolved.
    pub const fn target(&self) -> Option<&DependencyEndpoint> {
        self.target.as_ref()
    }

    /// Returns source contribution id.
    pub const fn contribution(&self) -> SourceContributionId {
        self.contribution
    }
}

impl Ord for ExportIndexEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.export.cmp(&other.export)
    }
}

impl PartialOrd for ExportIndexEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Export projection index.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResolvedExportIndex {
    entries: Vec<ExportIndexEntry>,
}

impl ResolvedExportIndex {
    /// Creates an empty export projection index.
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Inserts an export projection.
    pub fn insert(&mut self, entry: ExportIndexEntry) {
        self.entries.push(entry);
        self.entries.sort();
    }

    /// Returns an export projection by id.
    pub fn get(&self, export: ResolvedExportId) -> Option<&ExportIndexEntry> {
        self.entries.iter().find(|entry| entry.export() == export)
    }

    /// Returns exports produced by a contribution.
    pub fn by_contribution(&self, contribution: SourceContributionId) -> Vec<&ExportIndexEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.contribution() == contribution)
            .collect()
    }

    /// Iterates exports in deterministic id order.
    pub fn iter(&self) -> impl Iterator<Item = &ExportIndexEntry> {
        self.entries.iter()
    }

    /// Returns the number of entries.
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether the index is empty.
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Source contribution kind.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ContributionKind {
    /// Current-module source contribution.
    LocalSource {
        /// Source id when available.
        source_id: SourceId,
    },
    /// Imported source-backed dependency contribution.
    ImportedSource {
        /// Source id when available.
        source_id: SourceId,
    },
    /// Summary-backed dependency contribution.
    Summary {
        /// Opaque summary identity.
        identity: ModuleSummaryIdentity,
    },
    /// Built-in or prelude contribution.
    Builtin {
        /// Built-in contribution name.
        name: String,
    },
}

impl ContributionKind {
    /// Returns the source id for source-backed contributions.
    pub const fn source_id(&self) -> Option<SourceId> {
        match self {
            Self::LocalSource { source_id } | Self::ImportedSource { source_id } => {
                Some(*source_id)
            }
            Self::Summary { .. } | Self::Builtin { .. } => None,
        }
    }

    /// Returns the summary identity for summary-backed contributions.
    pub const fn summary_identity(&self) -> Option<&ModuleSummaryIdentity> {
        match self {
            Self::Summary { identity } => Some(identity),
            Self::LocalSource { .. } | Self::ImportedSource { .. } | Self::Builtin { .. } => None,
        }
    }
}

/// Entries affected by a source contribution.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContributionEffects {
    symbols: Vec<SymbolId>,
    definitions: Vec<DefinitionId>,
    overload_groups: Vec<OverloadGroupId>,
    registrations: Vec<RegistrationId>,
    labels: Vec<LabelOriginPath>,
    namespace_edges: Vec<NamespaceEdgeId>,
    declaration_dependencies: Vec<DeclarationDependencyId>,
    imports: Vec<ResolvedImportId>,
    exports: Vec<ResolvedExportId>,
    diagnostics: Vec<DiagnosticAnchorId>,
}

impl ContributionEffects {
    /// Returns affected symbols.
    pub fn symbols(&self) -> &[SymbolId] {
        &self.symbols
    }

    /// Returns affected definitions.
    pub fn definitions(&self) -> &[DefinitionId] {
        &self.definitions
    }

    /// Returns affected overload groups.
    pub fn overload_groups(&self) -> &[OverloadGroupId] {
        &self.overload_groups
    }

    /// Returns affected registrations.
    pub fn registrations(&self) -> &[RegistrationId] {
        &self.registrations
    }

    /// Returns affected labels.
    pub fn labels(&self) -> &[LabelOriginPath] {
        &self.labels
    }

    /// Returns affected namespace edges.
    pub fn namespace_edges(&self) -> &[NamespaceEdgeId] {
        &self.namespace_edges
    }

    /// Returns affected declaration dependencies.
    pub fn declaration_dependencies(&self) -> &[DeclarationDependencyId] {
        &self.declaration_dependencies
    }

    /// Returns affected import entries.
    pub fn imports(&self) -> &[ResolvedImportId] {
        &self.imports
    }

    /// Returns affected export entries.
    pub fn exports(&self) -> &[ResolvedExportId] {
        &self.exports
    }

    /// Returns diagnostic or failure anchors.
    pub fn diagnostics(&self) -> &[DiagnosticAnchorId] {
        &self.diagnostics
    }
}

/// Source contribution record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceContribution {
    id: SourceContributionId,
    module: ModuleId,
    kind: ContributionKind,
    anchor: SourceAnchor,
    effects: ContributionEffects,
}

impl SourceContribution {
    /// Returns the contribution id.
    pub const fn id(&self) -> SourceContributionId {
        self.id
    }

    /// Returns the contributing module.
    pub const fn module(&self) -> &ModuleId {
        &self.module
    }

    /// Returns the contribution kind.
    pub const fn kind(&self) -> &ContributionKind {
        &self.kind
    }

    /// Returns source or generated anchor.
    pub const fn anchor(&self) -> &SourceAnchor {
        &self.anchor
    }

    /// Returns affected entries.
    pub const fn effects(&self) -> &ContributionEffects {
        &self.effects
    }
}

/// Source contribution and invalidation index.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceContributionIndex {
    next_id: usize,
    records: Vec<SourceContribution>,
}

impl SourceContributionIndex {
    /// Creates an empty contribution index.
    pub const fn new() -> Self {
        Self {
            next_id: 0,
            records: Vec::new(),
        }
    }

    /// Inserts a contribution record and returns its stable id.
    pub fn insert(
        &mut self,
        module: ModuleId,
        kind: ContributionKind,
        anchor: SourceAnchor,
    ) -> SourceContributionId {
        let id = SourceContributionId::new(self.next_id);
        self.next_id += 1;
        self.records.push(SourceContribution {
            id,
            module,
            kind,
            anchor,
            effects: ContributionEffects::default(),
        });
        id
    }

    /// Records an affected symbol.
    pub fn add_symbol(&mut self, id: SourceContributionId, symbol: SymbolId) {
        if let Some(record) = self.get_mut(id) {
            insert_sorted_unique(&mut record.effects.symbols, symbol);
        }
    }

    /// Records an affected definition.
    pub fn add_definition(&mut self, id: SourceContributionId, definition: DefinitionId) {
        if let Some(record) = self.get_mut(id) {
            insert_sorted_unique(&mut record.effects.definitions, definition);
        }
    }

    /// Records an affected overload group.
    pub fn add_overload_group(&mut self, id: SourceContributionId, group: OverloadGroupId) {
        if let Some(record) = self.get_mut(id) {
            insert_sorted_unique(&mut record.effects.overload_groups, group);
        }
    }

    /// Records an affected registration.
    pub fn add_registration(&mut self, id: SourceContributionId, registration: RegistrationId) {
        if let Some(record) = self.get_mut(id) {
            insert_sorted_unique(&mut record.effects.registrations, registration);
        }
    }

    /// Records an affected label.
    pub fn add_label(&mut self, id: SourceContributionId, label: LabelOriginPath) {
        if let Some(record) = self.get_mut(id) {
            insert_sorted_unique(&mut record.effects.labels, label);
        }
    }

    /// Records an affected namespace edge.
    pub fn add_namespace_edge(&mut self, id: SourceContributionId, edge: NamespaceEdgeId) {
        if let Some(record) = self.get_mut(id) {
            insert_sorted_unique(&mut record.effects.namespace_edges, edge);
        }
    }

    /// Records an affected declaration dependency.
    pub fn add_declaration_dependency(
        &mut self,
        id: SourceContributionId,
        dependency: DeclarationDependencyId,
    ) {
        if let Some(record) = self.get_mut(id) {
            insert_sorted_unique(&mut record.effects.declaration_dependencies, dependency);
        }
    }

    /// Records an affected import entry.
    pub fn add_import(&mut self, id: SourceContributionId, import: ResolvedImportId) {
        if let Some(record) = self.get_mut(id) {
            insert_sorted_unique(&mut record.effects.imports, import);
        }
    }

    /// Records an affected export entry.
    pub fn add_export(&mut self, id: SourceContributionId, export: ResolvedExportId) {
        if let Some(record) = self.get_mut(id) {
            insert_sorted_unique(&mut record.effects.exports, export);
        }
    }

    /// Records an affected diagnostic or failure anchor.
    pub fn add_diagnostic(&mut self, id: SourceContributionId, diagnostic: DiagnosticAnchorId) {
        if let Some(record) = self.get_mut(id) {
            insert_sorted_unique(&mut record.effects.diagnostics, diagnostic);
        }
    }

    /// Returns a contribution by id.
    pub fn get(&self, id: SourceContributionId) -> Option<&SourceContribution> {
        self.records.iter().find(|record| record.id() == id)
    }

    fn get_mut(&mut self, id: SourceContributionId) -> Option<&mut SourceContribution> {
        self.records.iter_mut().find(|record| record.id() == id)
    }

    /// Returns contribution effects by id.
    pub fn affected_by(&self, id: SourceContributionId) -> Option<&ContributionEffects> {
        self.get(id).map(SourceContribution::effects)
    }

    /// Returns contributions backed by a source id.
    pub fn by_source(&self, source_id: SourceId) -> Vec<&SourceContribution> {
        self.records
            .iter()
            .filter(|record| record.kind().source_id() == Some(source_id))
            .collect()
    }

    /// Returns contributions backed by a dependency summary identity.
    pub fn by_summary(&self, identity: &ModuleSummaryIdentity) -> Vec<&SourceContribution> {
        self.records
            .iter()
            .filter(|record| record.kind().summary_identity() == Some(identity))
            .collect()
    }

    /// Iterates contribution records in deterministic id order.
    pub fn iter(&self) -> impl Iterator<Item = &SourceContribution> {
        self.records.iter()
    }

    /// Returns the number of records.
    pub const fn len(&self) -> usize {
        self.records.len()
    }

    /// Returns whether the index is empty.
    pub const fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

/// Dependency module summary entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleSummaryEntry {
    id: ModuleSummaryId,
    module: ModuleId,
    identity: ModuleSummaryIdentity,
    contribution: SourceContributionId,
}

impl ModuleSummaryEntry {
    /// Returns the module summary id.
    pub const fn id(&self) -> ModuleSummaryId {
        self.id
    }

    /// Returns the module id.
    pub const fn module(&self) -> &ModuleId {
        &self.module
    }

    /// Returns the opaque summary identity.
    pub const fn identity(&self) -> &ModuleSummaryIdentity {
        &self.identity
    }

    /// Returns source contribution id.
    pub const fn contribution(&self) -> SourceContributionId {
        self.contribution
    }
}

impl Ord for ModuleSummaryEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.module
            .cmp(other.module())
            .then_with(|| self.identity.cmp(other.identity()))
            .then_with(|| self.id.cmp(&other.id))
    }
}

impl PartialOrd for ModuleSummaryEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// In-memory dependency module summary index.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModuleSummaryIndex {
    next_id: usize,
    entries: Vec<ModuleSummaryEntry>,
}

impl ModuleSummaryIndex {
    /// Creates an empty module summary index.
    pub const fn new() -> Self {
        Self {
            next_id: 0,
            entries: Vec::new(),
        }
    }

    /// Inserts a summary identity.
    pub fn insert(
        &mut self,
        module: ModuleId,
        identity: ModuleSummaryIdentity,
        contribution: SourceContributionId,
    ) -> ModuleSummaryId {
        let id = ModuleSummaryId::new(self.next_id);
        self.next_id += 1;
        self.entries.push(ModuleSummaryEntry {
            id,
            module,
            identity,
            contribution,
        });
        self.entries.sort();
        id
    }

    /// Returns a summary by id.
    pub fn get(&self, id: ModuleSummaryId) -> Option<&ModuleSummaryEntry> {
        self.entries.iter().find(|entry| entry.id() == id)
    }

    /// Returns a summary by module id.
    pub fn by_module(&self, module: &ModuleId) -> Option<&ModuleSummaryEntry> {
        self.entries.iter().find(|entry| entry.module() == module)
    }

    /// Returns summaries produced by a contribution.
    pub fn by_contribution(&self, contribution: SourceContributionId) -> Vec<&ModuleSummaryEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.contribution() == contribution)
            .collect()
    }

    /// Iterates summaries in deterministic module order.
    pub fn iter(&self) -> impl Iterator<Item = &ModuleSummaryEntry> {
        self.entries.iter()
    }

    /// Returns the number of entries.
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether the index is empty.
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Index family carried by `SymbolEnv`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SymbolEnvIndexes {
    /// Import projection index.
    pub imports: ResolvedImportIndex,
    /// Export projection index.
    pub exports: ResolvedExportIndex,
    /// Symbol index.
    pub symbols: SymbolIndex,
    /// Label index.
    pub labels: LabelIndex,
    /// Definition index.
    pub definitions: DefinitionIndex,
    /// Overload index.
    pub overloads: OverloadIndex,
    /// Registration declaration index.
    pub registrations: RegistrationIndex,
    /// Namespace graph.
    pub namespace_graph: NamespaceGraph,
    /// Declaration dependency index.
    pub declaration_dependencies: DeclarationDependencyIndex,
    /// Source contribution index.
    pub contributions: SourceContributionIndex,
    /// In-memory module summary index.
    pub module_summaries: ModuleSummaryIndex,
}

/// Resolver-owned symbol environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolEnv {
    module_id: ModuleId,
    indexes: SymbolEnvIndexes,
}

impl SymbolEnv {
    /// Creates a symbol environment from index families.
    pub const fn new(module_id: ModuleId, indexes: SymbolEnvIndexes) -> Self {
        Self { module_id, indexes }
    }

    /// Returns the module id represented by this environment.
    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    /// Returns the import projection index.
    pub const fn imports(&self) -> &ResolvedImportIndex {
        &self.indexes.imports
    }

    /// Returns the export projection index.
    pub const fn exports(&self) -> &ResolvedExportIndex {
        &self.indexes.exports
    }

    /// Returns the symbol index.
    pub const fn symbols(&self) -> &SymbolIndex {
        &self.indexes.symbols
    }

    /// Returns the label index.
    pub const fn labels(&self) -> &LabelIndex {
        &self.indexes.labels
    }

    /// Returns the definition index.
    pub const fn definitions(&self) -> &DefinitionIndex {
        &self.indexes.definitions
    }

    /// Returns the overload index.
    pub const fn overloads(&self) -> &OverloadIndex {
        &self.indexes.overloads
    }

    /// Returns the registration index.
    pub const fn registrations(&self) -> &RegistrationIndex {
        &self.indexes.registrations
    }

    /// Returns the namespace graph.
    pub const fn namespace_graph(&self) -> &NamespaceGraph {
        &self.indexes.namespace_graph
    }

    /// Returns the declaration dependency index.
    pub const fn declaration_dependencies(&self) -> &DeclarationDependencyIndex {
        &self.indexes.declaration_dependencies
    }

    /// Returns the source contribution index.
    pub const fn contributions(&self) -> &SourceContributionIndex {
        &self.indexes.contributions
    }

    /// Returns the in-memory dependency summary index.
    pub const fn module_summaries(&self) -> &ModuleSummaryIndex {
        &self.indexes.module_summaries
    }

    /// Renders a stable human-readable debug snapshot.
    pub fn snapshot_text(&self) -> String {
        let mut output = String::from("symbol-env-debug-v1\n");
        output.push_str("module: ");
        write_module_id(&mut output, &self.module_id);
        output.push('\n');
        write_import_index_snapshot(&mut output, self.imports());
        write_export_index_snapshot(&mut output, self.exports());
        write_symbol_index_snapshot(&mut output, self.symbols());
        write_label_index_snapshot(&mut output, self.labels());
        write_definition_index_snapshot(&mut output, self.definitions());
        write_overload_index_snapshot(&mut output, self.overloads());
        write_registration_index_snapshot(&mut output, self.registrations());
        write_namespace_graph_snapshot(&mut output, self.namespace_graph());
        write_declaration_dependency_index_snapshot(&mut output, self.declaration_dependencies());
        write_contribution_index_snapshot(&mut output, self.contributions());
        write_module_summary_index_snapshot(&mut output, self.module_summaries());
        output
    }
}

fn write_import_index_snapshot(output: &mut String, imports: &ResolvedImportIndex) {
    output.push_str("imports:\n");
    if imports.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for entry in imports.iter() {
        let _ = write!(output, "  import#{} module=", entry.import().index());
        match entry.module() {
            Some(module) => write_module_id(output, module),
            None => output.push_str("<none>"),
        }
        output.push_str(" alias=");
        match entry.alias() {
            Some(alias) => {
                output.push('"');
                write_escaped(output, alias);
                output.push('"');
            }
            None => output.push_str("<none>"),
        }
        let _ = writeln!(
            output,
            " contribution=contribution#{}",
            entry.contribution().index()
        );
    }
}

fn write_export_index_snapshot(output: &mut String, exports: &ResolvedExportIndex) {
    output.push_str("exports:\n");
    if exports.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for entry in exports.iter() {
        let _ = write!(output, "  export#{} target=", entry.export().index());
        match entry.target() {
            Some(target) => write_dependency_endpoint(output, target),
            None => output.push_str("<none>"),
        }
        let _ = writeln!(
            output,
            " contribution=contribution#{}",
            entry.contribution().index()
        );
    }
}

fn write_symbol_index_snapshot(output: &mut String, symbols: &SymbolIndex) {
    output.push_str("symbols:\n");
    if symbols.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for entry in symbols.iter() {
        output.push_str("  symbol=");
        write_symbol_id(output, entry.symbol());
        let _ = write!(
            output,
            " kind={} visibility={} export={} namespace=\"",
            symbol_kind_name(entry.kind()),
            visibility_name(entry.visibility()),
            export_status_name(entry.export_status())
        );
        write_escaped(output, entry.namespace().as_str());
        output.push_str("\" spelling=\"");
        write_escaped(output, entry.primary_spelling());
        output.push_str("\" notation=");
        match entry.notation_spelling() {
            Some(spelling) => {
                output.push('"');
                write_escaped(output, spelling);
                output.push('"');
            }
            None => output.push_str("<none>"),
        }
        output.push_str(" signature=");
        write_optional_signature_shell(output, entry.signature());
        output.push_str(" relations=");
        write_relations(output, entry.relations());
        output.push_str(" origin=");
        write_origin(output, entry.origin());
        let _ = writeln!(
            output,
            " contribution=contribution#{}",
            entry.contribution().index()
        );
    }
}

fn write_label_index_snapshot(output: &mut String, labels: &LabelIndex) {
    output.push_str("labels:\n");
    if labels.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for entry in labels.iter() {
        output.push_str("  label=\"");
        write_escaped(output, entry.origin_path().as_str());
        let _ = write!(
            output,
            "\" kind={} visibility={} export={} namespace=\"",
            label_kind_name(entry.kind()),
            visibility_name(entry.visibility()),
            export_status_name(entry.export_status())
        );
        write_escaped(output, entry.namespace().as_str());
        output.push_str("\" spelling=\"");
        write_escaped(output, entry.primary_spelling());
        output.push_str("\" origin=");
        write_origin(output, entry.origin());
        let _ = writeln!(
            output,
            " contribution=contribution#{} recovery={}",
            entry.contribution().index(),
            recovery_state_name(entry.recovery())
        );
    }
}

fn write_definition_index_snapshot(output: &mut String, definitions: &DefinitionIndex) {
    output.push_str("definitions:\n");
    if definitions.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for entry in definitions.iter() {
        let _ = write!(output, "  definition#{} symbol=", entry.id().index());
        write_symbol_id(output, entry.symbol());
        let _ = write!(
            output,
            " kind={} visibility={} parameters=",
            definition_kind_name(entry.kind()),
            visibility_name(entry.visibility())
        );
        write_shell_ids(output, entry.parameters());
        output.push_str(" binders=");
        write_shell_ids(output, entry.binders());
        output.push_str(" arity=");
        match entry.arity() {
            Some(arity) => {
                let _ = write!(output, "{arity}");
            }
            None => output.push_str("<none>"),
        }
        output.push_str(" notation=");
        match entry.notation_shape() {
            Some(shape) => {
                output.push('"');
                write_escaped(output, shape);
                output.push('"');
            }
            None => output.push_str("<none>"),
        }
        output.push_str(" doc=");
        match entry.doc_attachment() {
            Some(doc) => {
                output.push('"');
                write_escaped(output, doc.as_str());
                output.push('"');
            }
            None => output.push_str("<none>"),
        }
        output.push_str(" conflict=");
        match entry.conflict() {
            Some(conflict) => output.push_str(declaration_conflict_class_name(conflict)),
            None => output.push_str("<none>"),
        }
        output.push_str(" dependencies=");
        write_declaration_dependency_ids(output, entry.dependencies());
        output.push_str(" signature=");
        write_optional_signature_shell(output, entry.signature());
        output.push_str(" origin=");
        write_origin(output, entry.origin());
        let _ = writeln!(
            output,
            " contribution=contribution#{}",
            entry.contribution().index()
        );
    }
}

fn write_overload_index_snapshot(output: &mut String, overloads: &OverloadIndex) {
    output.push_str("overloads:\n");
    if overloads.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for group in overloads.iter() {
        let _ = write!(output, "  overload#{} key=", group.id().index());
        write_overload_key(output, group.key());
        output.push_str(" candidates=");
        write_symbol_ids(output, group.candidates());
        output.push_str(" diagnostics=");
        write_diagnostic_anchor_ids(output, group.diagnostics());
        let _ = writeln!(
            output,
            " contribution=contribution#{}",
            group.contribution().index()
        );
    }
}

fn write_registration_index_snapshot(output: &mut String, registrations: &RegistrationIndex) {
    output.push_str("registrations:\n");
    if registrations.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for entry in registrations.iter() {
        let _ = write!(output, "  registration#{} symbol=", entry.id().index());
        match entry.symbol() {
            Some(symbol) => write_symbol_id(output, symbol),
            None => output.push_str("<none>"),
        }
        let _ = write!(
            output,
            " kind={} target=",
            registration_kind_name(entry.kind())
        );
        write_signature_shell(output, entry.target());
        let _ = write!(
            output,
            " visibility={} export={} dependencies=",
            visibility_name(entry.visibility()),
            export_status_name(entry.export_status())
        );
        write_declaration_dependency_ids(output, entry.dependencies());
        output.push_str(" origin=");
        write_origin(output, entry.origin());
        let _ = writeln!(
            output,
            " contribution=contribution#{} recovery={}",
            entry.contribution().index(),
            recovery_state_name(entry.recovery())
        );
    }
}

fn write_namespace_graph_snapshot(output: &mut String, graph: &NamespaceGraph) {
    output.push_str("namespace_graph:\n");
    output.push_str("  nodes:\n");
    if graph.node_len() == 0 {
        output.push_str("    <none>\n");
    } else {
        for node in graph.nodes() {
            let _ = write!(
                output,
                "    node#{} kind={} module=",
                node.id().index(),
                namespace_node_kind_name(node.kind())
            );
            match node.module() {
                Some(module) => write_module_id(output, module),
                None => output.push_str("<none>"),
            }
            output.push_str(" spelling=\"");
            write_escaped(output, node.spelling());
            let _ = writeln!(
                output,
                "\" contribution=contribution#{}",
                node.contribution().index()
            );
        }
    }
    output.push_str("  edges:\n");
    if graph.edge_len() == 0 {
        output.push_str("    <none>\n");
    } else {
        for edge in graph.edges() {
            let _ = write!(
                output,
                "    edge#{} from=node#{} to=node#{} kind={} anchor=",
                edge.id().index(),
                edge.from().index(),
                edge.to().index(),
                namespace_edge_kind_name(edge.kind())
            );
            write_anchor(output, edge.anchor());
            let _ = write!(
                output,
                " visibility={} target=",
                visibility_name(edge.visibility())
            );
            match edge.target() {
                Some(target) => write_namespace_target(output, target),
                None => output.push_str("<none>"),
            }
            output.push_str(" local_spelling=");
            match edge.local_spelling() {
                Some(spelling) => {
                    output.push('"');
                    write_escaped(output, spelling);
                    output.push('"');
                }
                None => output.push_str("<none>"),
            }
            let _ = writeln!(
                output,
                " contribution=contribution#{}",
                edge.contribution().index()
            );
        }
    }
}

fn write_declaration_dependency_index_snapshot(
    output: &mut String,
    dependencies: &DeclarationDependencyIndex,
) {
    output.push_str("declaration_dependencies:\n");
    if dependencies.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for dependency in dependencies.iter() {
        let _ = write!(output, "  dependency#{} source=", dependency.id().index());
        write_dependency_endpoint(output, dependency.source());
        output.push_str(" target=");
        write_dependency_endpoint(output, dependency.target());
        let _ = write!(
            output,
            " kind={} anchor=",
            declaration_dependency_kind_name(dependency.kind())
        );
        write_anchor(output, dependency.anchor());
        let _ = writeln!(
            output,
            " contribution=contribution#{}",
            dependency.contribution().index()
        );
    }
}

fn write_contribution_index_snapshot(output: &mut String, contributions: &SourceContributionIndex) {
    output.push_str("contributions:\n");
    if contributions.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for contribution in contributions.iter() {
        let _ = write!(
            output,
            "  contribution#{} module=",
            contribution.id().index()
        );
        write_module_id(output, contribution.module());
        output.push_str(" kind=");
        write_contribution_kind(output, contribution.kind());
        output.push_str(" anchor=");
        write_anchor(output, contribution.anchor());
        output.push_str(" effects=");
        write_contribution_effects(output, contribution.effects());
        output.push('\n');
    }
}

fn write_module_summary_index_snapshot(output: &mut String, summaries: &ModuleSummaryIndex) {
    output.push_str("module_summaries:\n");
    if summaries.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for entry in summaries.iter() {
        let _ = write!(output, "  summary#{} module=", entry.id().index());
        write_module_id(output, entry.module());
        output.push_str(" identity=\"");
        write_escaped(output, entry.identity().as_str());
        let _ = writeln!(
            output,
            "\" contribution=contribution#{}",
            entry.contribution().index()
        );
    }
}

fn write_overload_key(output: &mut String, key: &OverloadKey) {
    output.push_str("{namespace=\"");
    write_escaped(output, key.namespace().as_str());
    output.push_str("\" spelling=\"");
    write_escaped(output, key.spelling());
    output.push_str("\" kind=");
    output.push_str(symbol_kind_name(key.kind()));
    output.push_str(" arity=");
    match key.arity() {
        Some(arity) => {
            let _ = write!(output, "{arity}");
        }
        None => output.push_str("<none>"),
    }
    output.push('}');
}

fn write_contribution_kind(output: &mut String, kind: &ContributionKind) {
    match kind {
        ContributionKind::LocalSource { .. } => output.push_str("local_source"),
        ContributionKind::ImportedSource { .. } => output.push_str("imported_source"),
        ContributionKind::Summary { identity } => {
            output.push_str("summary identity=\"");
            write_escaped(output, identity.as_str());
            output.push('"');
        }
        ContributionKind::Builtin { name } => {
            output.push_str("builtin name=\"");
            write_escaped(output, name);
            output.push('"');
        }
    }
}

fn write_contribution_effects(output: &mut String, effects: &ContributionEffects) {
    output.push('{');
    output.push_str("symbols=");
    write_symbol_ids(output, effects.symbols());
    output.push_str(" definitions=");
    write_definition_ids(output, effects.definitions());
    output.push_str(" overloads=");
    write_overload_group_ids(output, effects.overload_groups());
    output.push_str(" registrations=");
    write_registration_ids(output, effects.registrations());
    output.push_str(" labels=");
    write_label_origin_paths(output, effects.labels());
    output.push_str(" namespace_edges=");
    write_namespace_edge_ids(output, effects.namespace_edges());
    output.push_str(" declaration_dependencies=");
    write_declaration_dependency_ids(output, effects.declaration_dependencies());
    output.push_str(" imports=");
    write_import_ids(output, effects.imports());
    output.push_str(" exports=");
    write_export_ids(output, effects.exports());
    output.push_str(" diagnostics=");
    write_diagnostic_anchor_ids(output, effects.diagnostics());
    output.push('}');
}

fn write_dependency_endpoint(output: &mut String, endpoint: &DependencyEndpoint) {
    match endpoint {
        DependencyEndpoint::Symbol(symbol) => {
            output.push_str("symbol=");
            write_symbol_id(output, symbol);
        }
        DependencyEndpoint::Import(import) => {
            let _ = write!(output, "import#{}", import.index());
        }
        DependencyEndpoint::Export(export) => {
            let _ = write!(output, "export#{}", export.index());
        }
        DependencyEndpoint::NamespaceEdge(edge) => {
            let _ = write!(output, "namespace_edge#{}", edge.index());
        }
        DependencyEndpoint::Label(label) => {
            output.push_str("label=\"");
            write_escaped(output, label.as_str());
            output.push('"');
        }
        DependencyEndpoint::UnresolvedName(name) => {
            let _ = write!(output, "unresolved_name#{}", name.index());
        }
        DependencyEndpoint::UnresolvedLabel(label) => {
            let _ = write!(output, "unresolved_label#{}", label.index());
        }
        DependencyEndpoint::Module(module) => {
            output.push_str("module=");
            write_module_id(output, module);
        }
    }
}

fn write_namespace_target(output: &mut String, target: &NamespaceTarget) {
    match target {
        NamespaceTarget::Module(module) => {
            output.push_str("module=");
            write_module_id(output, module);
        }
        NamespaceTarget::Symbol(symbol) => {
            output.push_str("symbol=");
            write_symbol_id(output, symbol);
        }
        NamespaceTarget::Label(label) => {
            output.push_str("label=\"");
            write_escaped(output, label.as_str());
            output.push('"');
        }
        NamespaceTarget::Unresolved(spelling) => {
            output.push_str("unresolved=\"");
            write_escaped(output, spelling);
            output.push('"');
        }
    }
}

fn write_relations(output: &mut String, relations: &[RelationMetadata]) {
    output.push('[');
    for (index, relation) in relations.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        output.push_str(relation_kind_name(relation.kind()));
        output.push_str("->");
        write_symbol_id(output, relation.target());
    }
    output.push(']');
}

fn write_optional_signature_shell(output: &mut String, signature: Option<&SignatureShell>) {
    match signature {
        Some(signature) => write_signature_shell(output, signature),
        None => output.push_str("<none>"),
    }
}

fn write_signature_shell(output: &mut String, signature: &SignatureShell) {
    match signature {
        SignatureShell::Pending => output.push_str("pending"),
        SignatureShell::Opaque { schema, payload } => {
            output.push_str("opaque(schema=\"");
            write_escaped(output, schema);
            output.push_str("\", payload=\"");
            write_escaped(output, payload);
            output.push_str("\")");
        }
        SignatureShell::Malformed { class } => {
            output.push_str("malformed(class=\"");
            write_escaped(output, class);
            output.push_str("\")");
        }
    }
}

fn write_origin(output: &mut String, origin: &SemanticOrigin) {
    output.push('{');
    output.push_str("module=");
    write_module_id(output, origin.module_id());
    output.push_str(" anchor=");
    write_anchor(output, origin.anchor());
    output.push_str(" path=");
    write_u32_list(output, origin.structural_path());
    output.push_str(" import=");
    match origin.import_edge() {
        Some(import) => {
            let _ = write!(output, "import#{}", import.index());
        }
        None => output.push_str("<none>"),
    }
    let _ = write!(output, " recovered={}", origin.is_recovered());
    output.push('}');
}

fn write_anchor(output: &mut String, anchor: &SourceAnchor) {
    match anchor {
        SourceAnchor::Range(range) => {
            output.push_str("range(");
            write_range(output, *range);
            output.push(')');
        }
        SourceAnchor::Point { offset, .. } => {
            let _ = write!(output, "point({offset})");
        }
        SourceAnchor::Generated(origin) => {
            output.push_str("generated(");
            write_generated_span_anchor(output, origin.anchor());
            output.push_str(", reason=present)");
        }
        _ => output.push_str("unknown"),
    }
}

fn write_generated_span_anchor(output: &mut String, anchor: GeneratedSpanAnchor) {
    match anchor {
        GeneratedSpanAnchor::Range(range) => {
            output.push_str("range(");
            write_range(output, range);
            output.push(')');
        }
        GeneratedSpanAnchor::Point { offset, .. } => {
            let _ = write!(output, "point({offset})");
        }
        _ => output.push_str("unknown"),
    }
}

fn write_symbol_ids(output: &mut String, ids: &[SymbolId]) {
    output.push('[');
    for (index, id) in ids.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        write_symbol_id(output, id);
    }
    output.push(']');
}

fn write_shell_ids(output: &mut String, ids: &[ResolverShellId]) {
    output.push('[');
    for (index, id) in ids.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        output.push('"');
        write_escaped(output, id.as_str());
        output.push('"');
    }
    output.push(']');
}

fn write_label_origin_paths(output: &mut String, ids: &[LabelOriginPath]) {
    output.push('[');
    for (index, id) in ids.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        output.push('"');
        write_escaped(output, id.as_str());
        output.push('"');
    }
    output.push(']');
}

fn write_definition_ids(output: &mut String, ids: &[DefinitionId]) {
    output.push('[');
    for (index, id) in ids.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "definition#{}", id.index());
    }
    output.push(']');
}

fn write_overload_group_ids(output: &mut String, ids: &[OverloadGroupId]) {
    output.push('[');
    for (index, id) in ids.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "overload#{}", id.index());
    }
    output.push(']');
}

fn write_registration_ids(output: &mut String, ids: &[RegistrationId]) {
    output.push('[');
    for (index, id) in ids.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "registration#{}", id.index());
    }
    output.push(']');
}

fn write_namespace_edge_ids(output: &mut String, ids: &[NamespaceEdgeId]) {
    output.push('[');
    for (index, id) in ids.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "edge#{}", id.index());
    }
    output.push(']');
}

fn write_declaration_dependency_ids(output: &mut String, ids: &[DeclarationDependencyId]) {
    output.push('[');
    for (index, id) in ids.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "dependency#{}", id.index());
    }
    output.push(']');
}

fn write_import_ids(output: &mut String, ids: &[ResolvedImportId]) {
    output.push('[');
    for (index, id) in ids.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "import#{}", id.index());
    }
    output.push(']');
}

fn write_export_ids(output: &mut String, ids: &[ResolvedExportId]) {
    output.push('[');
    for (index, id) in ids.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "export#{}", id.index());
    }
    output.push(']');
}

fn write_diagnostic_anchor_ids(output: &mut String, ids: &[DiagnosticAnchorId]) {
    output.push('[');
    for (index, id) in ids.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "diagnostic#{}", id.index());
    }
    output.push(']');
}

fn write_u32_list(output: &mut String, values: &[u32]) {
    output.push('[');
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "{value}");
    }
    output.push(']');
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

fn write_module_id(output: &mut String, module: &ModuleId) {
    write_escaped(output, module.package().as_str());
    output.push_str("::");
    write_escaped(output, module.path().as_str());
}

fn write_range(output: &mut String, range: SourceRange) {
    let _ = write!(output, "{}..{}", range.start, range.end);
}

fn write_escaped(output: &mut String, value: &str) {
    for character in value.chars() {
        for escaped in character.escape_default() {
            output.push(escaped);
        }
    }
}

fn symbol_kind_name(kind: SymbolKind) -> &'static str {
    match kind {
        SymbolKind::Predicate => "predicate",
        SymbolKind::Functor => "functor",
        SymbolKind::Mode => "mode",
        SymbolKind::Attribute => "attribute",
        SymbolKind::Structure => "structure",
        SymbolKind::Selector => "selector",
        SymbolKind::Registration => "registration",
        SymbolKind::Theorem => "theorem",
        SymbolKind::Builtin => "builtin",
    }
}

fn visibility_name(visibility: Visibility) -> &'static str {
    match visibility {
        Visibility::Private => "private",
        Visibility::Public => "public",
    }
}

fn export_status_name(status: ExportStatus) -> &'static str {
    match status {
        ExportStatus::LocalOnly => "local_only",
        ExportStatus::Exported => "exported",
        ExportStatus::ReExported => "re_exported",
    }
}

fn label_kind_name(kind: LabelKind) -> &'static str {
    match kind {
        LabelKind::Theorem => "theorem",
        LabelKind::Definition => "definition",
        LabelKind::ProofStep => "proof_step",
        LabelKind::Registration => "registration",
    }
}

fn definition_kind_name(kind: DefinitionKind) -> &'static str {
    match kind {
        DefinitionKind::Predicate => "predicate",
        DefinitionKind::Functor => "functor",
        DefinitionKind::Mode => "mode",
        DefinitionKind::Attribute => "attribute",
        DefinitionKind::Structure => "structure",
        DefinitionKind::Registration => "registration",
    }
}

fn declaration_conflict_class_name(class: &DeclarationConflictClass) -> &'static str {
    match class {
        DeclarationConflictClass::DuplicateSpelling => "duplicate_spelling",
        DeclarationConflictClass::IllegalOverloadGroup => "illegal_overload_group",
        DeclarationConflictClass::RecoveredShell => "recovered_shell",
    }
}

fn registration_kind_name(kind: RegistrationKind) -> &'static str {
    match kind {
        RegistrationKind::Cluster => "cluster",
        RegistrationKind::Identify => "identify",
        RegistrationKind::Reduction => "reduction",
        RegistrationKind::Property => "property",
    }
}

fn relation_kind_name(kind: RelationKind) -> &'static str {
    match kind {
        RelationKind::Synonym => "synonym",
        RelationKind::Antonym => "antonym",
        RelationKind::Redefinition => "redefinition",
    }
}

fn namespace_node_kind_name(kind: NamespaceNodeKind) -> &'static str {
    match kind {
        NamespaceNodeKind::Module => "module",
        NamespaceNodeKind::Alias => "alias",
        NamespaceNodeKind::Segment => "segment",
        NamespaceNodeKind::BuiltinPrelude => "builtin_prelude",
        NamespaceNodeKind::Unresolved => "unresolved",
    }
}

fn namespace_edge_kind_name(kind: NamespaceEdgeKind) -> &'static str {
    match kind {
        NamespaceEdgeKind::Import => "import",
        NamespaceEdgeKind::Export => "export",
        NamespaceEdgeKind::ReExport => "re_export",
        NamespaceEdgeKind::Segment => "segment",
        NamespaceEdgeKind::BuiltinPrelude => "builtin_prelude",
        NamespaceEdgeKind::Unresolved => "unresolved",
    }
}

fn declaration_dependency_kind_name(kind: DeclarationDependencyKind) -> &'static str {
    match kind {
        DeclarationDependencyKind::Import => "import",
        DeclarationDependencyKind::ReExport => "re_export",
        DeclarationDependencyKind::SignatureMention => "signature_mention",
        DeclarationDependencyKind::SynonymTarget => "synonym_target",
        DeclarationDependencyKind::AntonymTarget => "antonym_target",
        DeclarationDependencyKind::RedefinitionTarget => "redefinition_target",
        DeclarationDependencyKind::RegistrationMention => "registration_mention",
        DeclarationDependencyKind::LabelCitation => "label_citation",
    }
}

fn recovery_state_name(recovery: RecoveryState) -> &'static str {
    match recovery {
        RecoveryState::Normal => "normal",
        RecoveryState::Recovered => "recovered",
    }
}

fn insert_sorted_unique<T>(values: &mut Vec<T>, value: T)
where
    T: Ord,
{
    if values.binary_search(&value).is_err() {
        values.push(value);
        values.sort();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolved_ast::{
        ExportTarget, FullyQualifiedName, LocalSymbolId, ResolvedArenaBuilder, ResolvedExport,
        ResolvedImport, ResolvedImports, ResolvedNode, SymbolId,
    };
    use mizar_session::{
        BuildSnapshotId, GeneratedSpanAnchor, GeneratedSpanOrigin, Hash,
        InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator, SourceRange,
    };
    use mizar_syntax::ast::SurfaceNodeKind;

    #[test]
    fn index_families_round_trip_insertions_and_lookups() {
        let source_id = source_id(1);
        let module = module_id("pkg", "main");
        let origin = origin(source_id, module.clone());
        let mut contributions = SourceContributionIndex::new();
        let local = contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id },
            SourceAnchor::Range(range(source_id, 0, 1)),
        );
        let summary_contribution = contributions.insert(
            module_id("pkg", "dep"),
            ContributionKind::Summary {
                identity: ModuleSummaryIdentity::new("summary:dep:v1"),
            },
            SourceAnchor::Range(range(source_id, 10, 11)),
        );

        let symbol = symbol_id(module.clone(), "pred/0", "pkg::main::pred/0");
        let imported_symbol = symbol_id(module_id("pkg", "dep"), "func/0", "pkg::dep::func/0");
        let namespace = NamespacePath::new("main");
        let mut symbols = SymbolIndex::new();
        symbols.insert(
            SymbolEntry::new(
                symbol.clone(),
                SymbolKind::Predicate,
                namespace.clone(),
                "P",
                origin.clone(),
                local,
            )
            .with_visibility(Visibility::Public)
            .with_export_status(ExportStatus::Exported)
            .with_signature(SignatureShell::Opaque {
                schema: "signature-shell-v1".to_owned(),
                payload: "pred-shell".to_owned(),
            }),
        );
        symbols.insert(SymbolEntry::new(
            imported_symbol.clone(),
            SymbolKind::Functor,
            namespace.clone(),
            "F",
            origin.clone(),
            summary_contribution,
        ));
        assert_eq!(symbols.get(&symbol).unwrap().primary_spelling(), "P");
        assert_eq!(
            symbols
                .by_fqn(&FullyQualifiedName::new("pkg::main::pred/0"))
                .unwrap()
                .symbol(),
            &symbol
        );
        assert_eq!(symbols.visible_candidates(&namespace, "P").len(), 1);
        assert_eq!(symbols.exported_by_module(&module).len(), 1);
        assert_eq!(symbols.by_contribution(summary_contribution).len(), 1);

        let label = LabelOriginPath::new("pkg::main::T1");
        let mut labels = LabelIndex::new();
        labels.insert(
            LabelEntry::new(
                label.clone(),
                LabelKind::Theorem,
                namespace.clone(),
                "T1",
                origin.clone(),
                local,
            )
            .with_visibility(Visibility::Public)
            .with_export_status(ExportStatus::Exported),
        );
        assert_eq!(labels.get(&label).unwrap().primary_spelling(), "T1");
        assert_eq!(labels.visible_candidates(&namespace, "T1").len(), 1);

        let mut definitions = DefinitionIndex::new();
        let definition = definitions.insert(
            DefinitionShell::new(
                symbol.clone(),
                DefinitionKind::Predicate,
                origin.clone(),
                local,
            )
            .with_parameters(vec![
                ResolverShellId::new("param:z"),
                ResolverShellId::new("param:a"),
            ])
            .with_binders(vec![
                ResolverShellId::new("binder:z"),
                ResolverShellId::new("binder:a"),
            ])
            .with_arity(1)
            .with_signature(SignatureShell::Pending),
        );
        assert_eq!(definitions.get(definition).unwrap().symbol(), &symbol);
        assert_eq!(definitions.by_symbol(&symbol).unwrap().id(), definition);
        assert_eq!(
            definitions
                .get(definition)
                .unwrap()
                .parameters()
                .iter()
                .map(ResolverShellId::as_str)
                .collect::<Vec<_>>(),
            vec!["param:z", "param:a"]
        );
        assert_eq!(
            definitions
                .get(definition)
                .unwrap()
                .binders()
                .iter()
                .map(ResolverShellId::as_str)
                .collect::<Vec<_>>(),
            vec!["binder:z", "binder:a"]
        );

        let mut overloads = OverloadIndex::new();
        let overload = overloads.insert(
            OverloadKey::new(namespace.clone(), "P", SymbolKind::Predicate, Some(1)),
            vec![imported_symbol.clone(), symbol.clone()],
            local,
        );
        assert!(overloads.add_diagnostic(overload, DiagnosticAnchorId::new(5)));
        assert!(overloads.add_diagnostic(overload, DiagnosticAnchorId::new(3)));
        assert_eq!(
            overloads.get(overload).unwrap().candidates(),
            &[imported_symbol.clone(), symbol.clone()]
        );
        assert_eq!(
            overloads.get(overload).unwrap().diagnostics(),
            &[DiagnosticAnchorId::new(3), DiagnosticAnchorId::new(5)]
        );

        let mut registrations = RegistrationIndex::new();
        let registration = registrations.insert(
            Some(symbol.clone()),
            RegistrationKind::Cluster,
            SignatureShell::Opaque {
                schema: "registration-target-v1".to_owned(),
                payload: "cluster-shell".to_owned(),
            },
            origin.clone(),
            local,
        );
        registrations
            .get_mut(registration)
            .unwrap()
            .set_visibility(Visibility::Public)
            .set_export_status(ExportStatus::Exported)
            .set_dependencies(vec![
                DeclarationDependencyId::new(2),
                DeclarationDependencyId::new(1),
            ]);
        assert_eq!(
            registrations.get(registration).unwrap().kind(),
            RegistrationKind::Cluster
        );
        assert_eq!(
            registrations.get(registration).unwrap().visibility(),
            Visibility::Public
        );
        assert_eq!(
            registrations.get(registration).unwrap().export_status(),
            ExportStatus::Exported
        );
        assert_eq!(
            registrations.get(registration).unwrap().dependencies(),
            &[
                DeclarationDependencyId::new(1),
                DeclarationDependencyId::new(2)
            ]
        );

        let mut graph = NamespaceGraph::new();
        let root = graph.insert_node(
            NamespaceNodeKind::Module,
            Some(module.clone()),
            "main",
            local,
        );
        let alias = graph.insert_node(
            NamespaceNodeKind::Alias,
            Some(module_id("pkg", "dep")),
            "D",
            summary_contribution,
        );
        let edge = graph.insert_edge(
            NamespaceEdgeSpec::new(
                (root, alias),
                NamespaceEdgeKind::Import,
                SourceAnchor::Range(range(source_id, 2, 3)),
                local,
            )
            .with_visibility(Visibility::Public)
            .with_target(NamespaceTarget::Module(module_id("pkg", "dep")))
            .with_local_spelling("D"),
        );
        assert_eq!(graph.node(root).unwrap().spelling(), "main");
        assert_eq!(graph.edge(edge).unwrap().local_spelling(), Some("D"));
        assert_eq!(graph.edge(edge).unwrap().visibility(), Visibility::Public);

        let mut dependencies = DeclarationDependencyIndex::new();
        let dependency = dependencies.insert(
            DependencyEndpoint::Symbol(symbol.clone()),
            DependencyEndpoint::NamespaceEdge(edge),
            DeclarationDependencyKind::Import,
            SourceAnchor::Range(range(source_id, 2, 3)),
            local,
        );
        assert_eq!(
            dependencies.get(dependency).unwrap().kind(),
            DeclarationDependencyKind::Import
        );

        let (import_id, export_id) = import_export_ids(source_id, module.clone());
        let mut imports = ResolvedImportIndex::new();
        imports.insert(ImportIndexEntry::new(
            import_id,
            Some(module_id("pkg", "dep")),
            Some("D".to_owned()),
            local,
        ));
        assert_eq!(imports.get(import_id).unwrap().alias(), Some("D"));
        let mut exports = ResolvedExportIndex::new();
        exports.insert(ExportIndexEntry::new(
            export_id,
            Some(DependencyEndpoint::Symbol(symbol.clone())),
            local,
        ));
        assert_eq!(
            exports.get(export_id).unwrap().target(),
            Some(&DependencyEndpoint::Symbol(symbol.clone()))
        );

        let mut summaries = ModuleSummaryIndex::new();
        let summary = summaries.insert(
            module_id("pkg", "dep"),
            ModuleSummaryIdentity::new("summary:dep:v1"),
            summary_contribution,
        );
        assert_eq!(
            summaries.by_module(&module_id("pkg", "dep")).unwrap().id(),
            summary
        );

        contributions.add_symbol(local, symbol.clone());
        contributions.add_label(local, label.clone());
        contributions.add_definition(local, definition);
        contributions.add_overload_group(local, overload);
        contributions.add_registration(local, registration);
        contributions.add_namespace_edge(local, edge);
        contributions.add_declaration_dependency(local, dependency);
        contributions.add_import(local, import_id);
        contributions.add_export(local, export_id);
        contributions.add_diagnostic(local, DiagnosticAnchorId::new(0));

        let env = SymbolEnv::new(
            module.clone(),
            SymbolEnvIndexes {
                imports,
                exports,
                symbols,
                labels,
                definitions,
                overloads,
                registrations,
                namespace_graph: graph,
                declaration_dependencies: dependencies,
                contributions,
                module_summaries: summaries,
            },
        );
        assert_eq!(env.module_id(), &module);
        assert_eq!(env.symbols().len(), 2);
        assert_eq!(
            env.contributions().affected_by(local).unwrap().symbols(),
            &[symbol]
        );
        assert_eq!(
            env.contributions()
                .affected_by(local)
                .unwrap()
                .diagnostics(),
            &[DiagnosticAnchorId::new(0)]
        );
    }

    #[test]
    fn index_iteration_is_deterministic_for_all_families() {
        let primary_source_id = source_id(2);
        let module = module_id("pkg", "main");
        let origin = origin(primary_source_id, module.clone());
        let mut contributions = SourceContributionIndex::new();
        let local = contributions.insert(
            module.clone(),
            ContributionKind::LocalSource {
                source_id: primary_source_id,
            },
            SourceAnchor::Range(range(primary_source_id, 0, 1)),
        );
        let other = contributions.insert(
            module_id("pkg", "dep"),
            ContributionKind::Summary {
                identity: ModuleSummaryIdentity::new("summary:dep"),
            },
            SourceAnchor::Range(range(primary_source_id, 1, 2)),
        );
        let z_symbol = symbol_id(module.clone(), "z/0", "pkg::main::z/0");
        let a_symbol = symbol_id(module.clone(), "a/0", "pkg::main::a/0");
        let namespace = NamespacePath::new("main");

        let mut symbols = SymbolIndex::new();
        symbols.insert(SymbolEntry::new(
            z_symbol.clone(),
            SymbolKind::Predicate,
            namespace.clone(),
            "Z",
            origin.clone(),
            local,
        ));
        symbols.insert(SymbolEntry::new(
            a_symbol.clone(),
            SymbolKind::Predicate,
            namespace.clone(),
            "A",
            origin.clone(),
            local,
        ));
        assert_eq!(
            symbols
                .iter()
                .map(|entry| entry.symbol().local().as_str())
                .collect::<Vec<_>>(),
            vec!["a/0", "z/0"]
        );

        let mut labels = LabelIndex::new();
        labels.insert(LabelEntry::new(
            LabelOriginPath::new("pkg::main::Z1"),
            LabelKind::Theorem,
            namespace.clone(),
            "Z1",
            origin.clone(),
            local,
        ));
        labels.insert(LabelEntry::new(
            LabelOriginPath::new("pkg::main::A1"),
            LabelKind::Theorem,
            namespace.clone(),
            "A1",
            origin.clone(),
            local,
        ));
        assert_eq!(
            labels
                .iter()
                .map(|entry| entry.origin_path().as_str())
                .collect::<Vec<_>>(),
            vec!["pkg::main::A1", "pkg::main::Z1"]
        );

        let mut definitions = DefinitionIndex::new();
        let z_definition = definitions.insert(DefinitionShell::new(
            z_symbol.clone(),
            DefinitionKind::Predicate,
            origin.clone(),
            local,
        ));
        let a_definition = definitions.insert(DefinitionShell::new(
            a_symbol.clone(),
            DefinitionKind::Predicate,
            origin.clone(),
            local,
        ));
        assert_eq!(
            definitions
                .iter()
                .map(DefinitionEntry::id)
                .collect::<Vec<_>>(),
            vec![a_definition, z_definition]
        );

        let mut overloads = OverloadIndex::new();
        let z_group = overloads.insert(
            OverloadKey::new(namespace.clone(), "Z", SymbolKind::Predicate, None),
            vec![z_symbol.clone()],
            local,
        );
        let a_group = overloads.insert(
            OverloadKey::new(namespace.clone(), "A", SymbolKind::Predicate, None),
            vec![a_symbol.clone()],
            local,
        );
        assert_eq!(
            overloads.iter().map(OverloadGroup::id).collect::<Vec<_>>(),
            vec![a_group, z_group]
        );

        let mut registrations = RegistrationIndex::new();
        let z_registration = registrations.insert(
            Some(z_symbol.clone()),
            RegistrationKind::Cluster,
            SignatureShell::Pending,
            origin.clone(),
            local,
        );
        let a_registration = registrations.insert(
            Some(a_symbol.clone()),
            RegistrationKind::Cluster,
            SignatureShell::Pending,
            origin.clone(),
            local,
        );
        assert_eq!(
            registrations
                .iter()
                .map(RegistrationEntry::id)
                .collect::<Vec<_>>(),
            vec![a_registration, z_registration]
        );

        let mut graph = NamespaceGraph::new();
        let z_node =
            graph.insert_node(NamespaceNodeKind::Segment, Some(module.clone()), "z", local);
        let a_node =
            graph.insert_node(NamespaceNodeKind::Segment, Some(module.clone()), "a", local);
        let a_edge = graph.insert_edge(NamespaceEdgeSpec::new(
            (a_node, z_node),
            NamespaceEdgeKind::Segment,
            SourceAnchor::Range(range(primary_source_id, 5, 6)),
            local,
        ));
        let z_edge = graph.insert_edge(NamespaceEdgeSpec::new(
            (z_node, a_node),
            NamespaceEdgeKind::Segment,
            SourceAnchor::Range(range(primary_source_id, 4, 5)),
            local,
        ));
        assert_eq!(
            graph
                .nodes()
                .map(NamespaceNode::spelling)
                .collect::<Vec<_>>(),
            vec!["a", "z"]
        );
        assert_eq!(
            graph.edges().map(NamespaceEdge::id).collect::<Vec<_>>(),
            vec![z_edge, a_edge]
        );

        let mut dependencies = DeclarationDependencyIndex::new();
        let z_dependency = dependencies.insert(
            DependencyEndpoint::Symbol(z_symbol.clone()),
            DependencyEndpoint::Symbol(a_symbol.clone()),
            DeclarationDependencyKind::SignatureMention,
            SourceAnchor::Range(range(primary_source_id, 6, 7)),
            local,
        );
        let a_dependency = dependencies.insert(
            DependencyEndpoint::Symbol(a_symbol.clone()),
            DependencyEndpoint::Symbol(z_symbol.clone()),
            DeclarationDependencyKind::SignatureMention,
            SourceAnchor::Range(range(primary_source_id, 7, 8)),
            local,
        );
        assert_eq!(
            dependencies
                .iter()
                .map(DeclarationDependency::id)
                .collect::<Vec<_>>(),
            vec![a_dependency, z_dependency]
        );

        let ((first_import, first_export), (second_import, second_export)) =
            import_export_id_pair(primary_source_id, module.clone());
        let mut imports = ResolvedImportIndex::new();
        imports.insert(ImportIndexEntry::new(second_import, None, None, local));
        imports.insert(ImportIndexEntry::new(first_import, None, None, local));
        assert_eq!(
            imports
                .iter()
                .map(ImportIndexEntry::import)
                .collect::<Vec<_>>(),
            vec![first_import, second_import]
        );
        let mut exports = ResolvedExportIndex::new();
        exports.insert(ExportIndexEntry::new(second_export, None, local));
        exports.insert(ExportIndexEntry::new(first_export, None, local));
        assert_eq!(
            exports
                .iter()
                .map(ExportIndexEntry::export)
                .collect::<Vec<_>>(),
            vec![first_export, second_export]
        );

        let mut summaries = ModuleSummaryIndex::new();
        let z_summary = summaries.insert(
            module_id("pkg", "z"),
            ModuleSummaryIdentity::new("summary:z"),
            other,
        );
        let a_summary = summaries.insert(
            module_id("pkg", "a"),
            ModuleSummaryIdentity::new("summary:a"),
            other,
        );
        assert_eq!(
            summaries
                .iter()
                .map(ModuleSummaryEntry::id)
                .collect::<Vec<_>>(),
            vec![a_summary, z_summary]
        );

        assert_eq!(
            contributions
                .iter()
                .map(SourceContribution::id)
                .collect::<Vec<_>>(),
            vec![local, other]
        );
    }

    #[test]
    fn contribution_tracking_covers_sources_summaries_builtins_and_invalidation() {
        let (primary_source_id, imported_source_id) = source_id_pair(4);
        let module = module_id("pkg", "main");
        let summary_identity = ModuleSummaryIdentity::new("summary:dep:v2");
        let mut contributions = SourceContributionIndex::new();
        let local = contributions.insert(
            module.clone(),
            ContributionKind::LocalSource {
                source_id: primary_source_id,
            },
            SourceAnchor::Range(range(primary_source_id, 0, 1)),
        );
        let imported = contributions.insert(
            module_id("pkg", "dep_source"),
            ContributionKind::ImportedSource {
                source_id: imported_source_id,
            },
            SourceAnchor::Range(range(imported_source_id, 0, 1)),
        );
        let summary = contributions.insert(
            module_id("pkg", "dep_summary"),
            ContributionKind::Summary {
                identity: summary_identity.clone(),
            },
            SourceAnchor::Range(range(primary_source_id, 1, 2)),
        );
        let builtin = contributions.insert(
            module.clone(),
            ContributionKind::Builtin {
                name: "prelude".to_owned(),
            },
            SourceAnchor::Range(range(primary_source_id, 2, 3)),
        );

        let symbol = symbol_id(module.clone(), "pred/0", "pkg::main::pred/0");
        let definition = DefinitionId::new(0);
        let overload = OverloadGroupId::new(0);
        let registration = RegistrationId::new(0);
        let label = LabelOriginPath::new("pkg::main::T1");
        let namespace_edge = NamespaceEdgeId::new(0);
        let dependency = DeclarationDependencyId::new(0);
        let ((import, export), (later_import, later_export)) =
            import_export_id_pair(primary_source_id, module);

        contributions.add_symbol(local, symbol.clone());
        contributions.add_definition(local, definition);
        contributions.add_overload_group(local, overload);
        contributions.add_registration(local, registration);
        contributions.add_label(local, label.clone());
        contributions.add_namespace_edge(local, namespace_edge);
        contributions.add_declaration_dependency(local, dependency);
        contributions.add_import(local, later_import);
        contributions.add_import(local, import);
        contributions.add_import(local, import);
        contributions.add_export(local, later_export);
        contributions.add_export(local, export);
        contributions.add_export(local, export);
        contributions.add_diagnostic(local, DiagnosticAnchorId::new(7));
        contributions.add_diagnostic(local, DiagnosticAnchorId::new(3));
        contributions.add_diagnostic(local, DiagnosticAnchorId::new(7));
        contributions.add_symbol(local, symbol.clone());

        let effects = contributions.affected_by(local).unwrap();
        assert_eq!(effects.symbols(), &[symbol]);
        assert_eq!(effects.definitions(), &[definition]);
        assert_eq!(effects.overload_groups(), &[overload]);
        assert_eq!(effects.registrations(), &[registration]);
        assert_eq!(effects.labels(), &[label]);
        assert_eq!(effects.namespace_edges(), &[namespace_edge]);
        assert_eq!(effects.declaration_dependencies(), &[dependency]);
        assert_eq!(effects.imports(), &[import, later_import]);
        assert_eq!(effects.exports(), &[export, later_export]);
        assert_eq!(
            effects.diagnostics(),
            &[DiagnosticAnchorId::new(3), DiagnosticAnchorId::new(7)]
        );

        assert_eq!(contributions.by_source(primary_source_id).len(), 1);
        assert_eq!(contributions.by_source(imported_source_id).len(), 1);
        assert_eq!(contributions.by_summary(&summary_identity).len(), 1);
        assert!(matches!(
            contributions.get(builtin).unwrap().kind(),
            ContributionKind::Builtin { name } if name == "prelude"
        ));
        assert_eq!(
            contributions
                .get(imported)
                .unwrap()
                .module()
                .path()
                .as_str(),
            "dep_source"
        );
        assert_eq!(
            contributions.get(summary).unwrap().module().path().as_str(),
            "dep_summary"
        );
    }

    #[test]
    fn namespace_alias_node_order_uses_canonical_module_before_alias_spelling() {
        let source_id = source_id(6);
        let module = module_id("pkg", "main");
        let mut contributions = SourceContributionIndex::new();
        let local = contributions.insert(
            module,
            ContributionKind::LocalSource { source_id },
            SourceAnchor::Range(range(source_id, 0, 1)),
        );
        let mut graph = NamespaceGraph::new();
        let dep = module_id("pkg", "dep");
        graph.insert_node(
            NamespaceNodeKind::Alias,
            Some(dep.clone()),
            "z_alias",
            local,
        );
        graph.insert_node(NamespaceNodeKind::Alias, Some(dep), "a_alias", local);

        assert_eq!(
            graph
                .nodes()
                .map(NamespaceNode::spelling)
                .collect::<Vec<_>>(),
            vec!["z_alias", "a_alias"]
        );

        let mut cross_module_graph = NamespaceGraph::new();
        cross_module_graph.insert_node(
            NamespaceNodeKind::Alias,
            Some(module_id("pkg", "z_dep")),
            "a_alias",
            local,
        );
        cross_module_graph.insert_node(
            NamespaceNodeKind::Alias,
            Some(module_id("pkg", "a_dep")),
            "z_alias",
            local,
        );
        assert_eq!(
            cross_module_graph
                .nodes()
                .map(|node| node.module().unwrap().path().as_str())
                .collect::<Vec<_>>(),
            vec!["a_dep", "z_dep"]
        );
    }

    #[test]
    fn equivalent_construction_is_stable_and_checker_facts_are_absent() {
        let first = build_equivalent_env_snapshot(source_id(7));
        let second = build_equivalent_env_snapshot(source_id(7));

        assert_eq!(first, second);
        assert!(matches!(
            first.0.as_ref(),
            Some(SignatureShell::Opaque { schema, payload })
                if schema == "signature-shell-v1" && payload == "opaque"
        ));
    }

    #[test]
    fn symbol_env_snapshot_text_is_stable_and_covers_index_families() {
        let (local_source, imported_source) = source_id_pair(8);
        let first = debug_snapshot_env_fixture(local_source, imported_source).snapshot_text();
        let second = debug_snapshot_env_fixture(local_source, imported_source).snapshot_text();

        assert_eq!(first, second);
        assert!(first.starts_with("symbol-env-debug-v1\nmodule: pkg::main\n"));
        assert!(!first.contains("SourceId"));
        assert!(!first.contains("/tmp/private"));
        assert!(!first.contains('\r'));
        assert_ordered_fragments(
            &first,
            &[
                "module: pkg::main\n",
                "imports:\n",
                "exports:\n",
                "symbols:\n",
                "labels:\n",
                "definitions:\n",
                "overloads:\n",
                "registrations:\n",
                "namespace_graph:\n",
                "declaration_dependencies:\n",
                "contributions:\n",
                "module_summaries:\n",
            ],
        );
        for expected in [
            "imports:\n  import#0 module=pkg::dep alias=\"D\" contribution=contribution#0",
            "exports:\n  export#0 target=symbol={fqn=\"pkg::main::pred/0\" module=pkg::main local=\"pred/0\"} contribution=contribution#0",
            "symbols:\n  symbol={fqn=\"pkg::main::pred/0\" module=pkg::main local=\"pred/0\"} kind=predicate visibility=public export=exported namespace=\"main\" spelling=\"P\" notation=\"P(_)\" signature=opaque(schema=\"signature-v1\", payload=\"pred-shell\\n\\\"\\\\\") relations=[synonym->",
            "labels:\n  label=\"pkg::main::T1\" kind=theorem visibility=public export=exported namespace=\"main\" spelling=\"T1\"",
            "definitions:\n  definition#0 symbol={fqn=\"pkg::main::pred/0\" module=pkg::main local=\"pred/0\"} kind=predicate visibility=public parameters=[\"param:x\"] binders=[\"binder:x\"] arity=1 notation=\"P(_)\" doc=\"doc:T1\" conflict=duplicate_spelling dependencies=[dependency#0] signature=pending",
            "overloads:\n  overload#0 key={namespace=\"main\" spelling=\"P\" kind=predicate arity=1} candidates=[",
            "diagnostics=[diagnostic#4]",
            "registrations:\n  registration#0 symbol={fqn=\"pkg::main::pred/0\" module=pkg::main local=\"pred/0\"} kind=cluster target=malformed(class=\"recovered-target\") visibility=public export=re_exported dependencies=[dependency#0]",
            "namespace_graph:\n  nodes:\n    node#0 kind=module module=pkg::main spelling=\"main\" contribution=contribution#0",
            "  edges:\n    edge#0 from=node#0 to=node#1 kind=import anchor=range(2..3) visibility=public target=module=pkg::dep local_spelling=\"D\" contribution=contribution#0",
            "declaration_dependencies:\n  dependency#0 source=symbol={fqn=\"pkg::main::pred/0\" module=pkg::main local=\"pred/0\"} target=namespace_edge#0 kind=import anchor=range(2..3) contribution=contribution#0",
            "contributions:\n  contribution#0 module=pkg::main kind=local_source anchor=range(0..1) effects={symbols=[",
            " definitions=[definition#0] overloads=[overload#0] registrations=[registration#0] labels=[\"pkg::main::T1\"] namespace_edges=[edge#0] declaration_dependencies=[dependency#0] imports=[import#0] exports=[export#0] diagnostics=[diagnostic#9]}",
            "contribution#1 module=pkg::dep_source kind=imported_source anchor=point(5)",
            "contribution#2 module=pkg::dep_summary kind=summary identity=\"summary:dep:v1\" anchor=generated(range(1..2), reason=present)",
            "contribution#3 module=pkg::builtin kind=builtin name=\"prelude\"",
            "module_summaries:\n  summary#0 module=pkg::dep identity=\"summary:dep:v1\" contribution=contribution#2",
        ] {
            assert!(
                first.contains(expected),
                "snapshot should contain fixture fragment: {expected}\n{first}"
            );
        }
    }

    fn assert_ordered_fragments(snapshot: &str, fragments: &[&str]) {
        let mut cursor = 0;
        for fragment in fragments {
            let Some(offset) = snapshot[cursor..].find(fragment) else {
                panic!("missing ordered fragment: {fragment}\n{snapshot}");
            };
            cursor += offset + fragment.len();
        }
    }

    fn debug_snapshot_env_fixture(local_source: SourceId, imported_source: SourceId) -> SymbolEnv {
        let module = module_id("pkg", "main");
        let origin = origin(local_source, module.clone());
        let namespace = NamespacePath::new("main");
        let symbol = symbol_id(module.clone(), "pred/0", "pkg::main::pred/0");
        let related = symbol_id(module.clone(), "other/0", "pkg::main::other/0");

        let mut contributions = SourceContributionIndex::new();
        let local = contributions.insert(
            module.clone(),
            ContributionKind::LocalSource {
                source_id: local_source,
            },
            SourceAnchor::Range(range(local_source, 0, 1)),
        );
        contributions.insert(
            module_id("pkg", "dep_source"),
            ContributionKind::ImportedSource {
                source_id: imported_source,
            },
            SourceAnchor::Point {
                source_id: imported_source,
                offset: 5,
            },
        );
        let generated_summary_anchor = GeneratedSpanOrigin::new(
            GeneratedSpanAnchor::Range(range(local_source, 1, 2)),
            "/tmp/private/generated-summary",
        )
        .unwrap();
        let summary = contributions.insert(
            module_id("pkg", "dep_summary"),
            ContributionKind::Summary {
                identity: ModuleSummaryIdentity::new("summary:dep:v1"),
            },
            SourceAnchor::Generated(generated_summary_anchor),
        );
        contributions.insert(
            module_id("pkg", "builtin"),
            ContributionKind::Builtin {
                name: "prelude".to_owned(),
            },
            SourceAnchor::Range(range(local_source, 2, 3)),
        );

        let mut symbols = SymbolIndex::new();
        symbols.insert(
            SymbolEntry::new(
                symbol.clone(),
                SymbolKind::Predicate,
                namespace.clone(),
                "P",
                origin.clone(),
                local,
            )
            .with_visibility(Visibility::Public)
            .with_export_status(ExportStatus::Exported)
            .with_notation_spelling("P(_)")
            .with_signature(SignatureShell::Opaque {
                schema: "signature-v1".to_owned(),
                payload: "pred-shell\n\"\\".to_owned(),
            })
            .with_relations(vec![RelationMetadata::new(
                RelationKind::Synonym,
                related.clone(),
            )]),
        );

        let label = LabelOriginPath::new("pkg::main::T1");
        let mut labels = LabelIndex::new();
        labels.insert(
            LabelEntry::new(
                label.clone(),
                LabelKind::Theorem,
                namespace.clone(),
                "T1",
                origin.clone(),
                local,
            )
            .with_visibility(Visibility::Public)
            .with_export_status(ExportStatus::Exported),
        );

        let mut definitions = DefinitionIndex::new();
        let definition = definitions.insert(
            DefinitionShell::new(
                symbol.clone(),
                DefinitionKind::Predicate,
                origin.clone(),
                local,
            )
            .with_visibility(Visibility::Public)
            .with_parameters(vec![ResolverShellId::new("param:x")])
            .with_binders(vec![ResolverShellId::new("binder:x")])
            .with_arity(1)
            .with_notation_shape("P(_)")
            .with_doc_attachment(ResolverShellId::new("doc:T1"))
            .with_conflict(DeclarationConflictClass::DuplicateSpelling)
            .with_dependencies(vec![DeclarationDependencyId::new(0)])
            .with_signature(SignatureShell::Pending),
        );

        let mut overloads = OverloadIndex::new();
        let overload = overloads.insert(
            OverloadKey::new(namespace.clone(), "P", SymbolKind::Predicate, Some(1)),
            vec![related.clone(), symbol.clone()],
            local,
        );
        assert!(overloads.add_diagnostic(overload, DiagnosticAnchorId::new(4)));

        let mut registrations = RegistrationIndex::new();
        let registration = registrations.insert(
            Some(symbol.clone()),
            RegistrationKind::Cluster,
            SignatureShell::Malformed {
                class: "recovered-target".to_owned(),
            },
            origin.clone(),
            local,
        );
        registrations
            .get_mut(registration)
            .unwrap()
            .set_visibility(Visibility::Public)
            .set_export_status(ExportStatus::ReExported)
            .set_dependencies(vec![DeclarationDependencyId::new(0)]);

        let mut namespace_graph = NamespaceGraph::new();
        let root = namespace_graph.insert_node(
            NamespaceNodeKind::Module,
            Some(module.clone()),
            "main",
            local,
        );
        let alias = namespace_graph.insert_node(
            NamespaceNodeKind::Alias,
            Some(module_id("pkg", "dep")),
            "D",
            summary,
        );
        let namespace_edge = namespace_graph.insert_edge(
            NamespaceEdgeSpec::new(
                (root, alias),
                NamespaceEdgeKind::Import,
                SourceAnchor::Range(range(local_source, 2, 3)),
                local,
            )
            .with_visibility(Visibility::Public)
            .with_target(NamespaceTarget::Module(module_id("pkg", "dep")))
            .with_local_spelling("D"),
        );

        let mut declaration_dependencies = DeclarationDependencyIndex::new();
        let dependency = declaration_dependencies.insert(
            DependencyEndpoint::Symbol(symbol.clone()),
            DependencyEndpoint::NamespaceEdge(namespace_edge),
            DeclarationDependencyKind::Import,
            SourceAnchor::Range(range(local_source, 2, 3)),
            local,
        );

        let (import, export) = import_export_ids(local_source, module.clone());
        let mut imports = ResolvedImportIndex::new();
        imports.insert(ImportIndexEntry::new(
            import,
            Some(module_id("pkg", "dep")),
            Some("D".to_owned()),
            local,
        ));
        let mut exports = ResolvedExportIndex::new();
        exports.insert(ExportIndexEntry::new(
            export,
            Some(DependencyEndpoint::Symbol(symbol.clone())),
            local,
        ));

        let mut module_summaries = ModuleSummaryIndex::new();
        module_summaries.insert(
            module_id("pkg", "dep"),
            ModuleSummaryIdentity::new("summary:dep:v1"),
            summary,
        );

        contributions.add_symbol(local, symbol);
        contributions.add_definition(local, definition);
        contributions.add_overload_group(local, overload);
        contributions.add_registration(local, registration);
        contributions.add_label(local, label);
        contributions.add_namespace_edge(local, namespace_edge);
        contributions.add_declaration_dependency(local, dependency);
        contributions.add_import(local, import);
        contributions.add_export(local, export);
        contributions.add_diagnostic(local, DiagnosticAnchorId::new(9));

        SymbolEnv::new(
            module,
            SymbolEnvIndexes {
                imports,
                exports,
                symbols,
                labels,
                definitions,
                overloads,
                registrations,
                namespace_graph,
                declaration_dependencies,
                contributions,
                module_summaries,
            },
        )
    }

    fn build_equivalent_env_snapshot(
        source_id: SourceId,
    ) -> (Option<SignatureShell>, Vec<String>, Vec<usize>) {
        let module = module_id("pkg", "main");
        let origin = origin(source_id, module.clone());
        let mut contributions = SourceContributionIndex::new();
        let local = contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id },
            SourceAnchor::Range(range(source_id, 0, 1)),
        );
        let symbol = symbol_id(module, "pred/0", "pkg::main::pred/0");
        let namespace = NamespacePath::new("main");
        let mut symbols = SymbolIndex::new();
        symbols.insert(
            SymbolEntry::new(
                symbol.clone(),
                SymbolKind::Predicate,
                namespace.clone(),
                "P",
                origin.clone(),
                local,
            )
            .with_signature(SignatureShell::Opaque {
                schema: "signature-shell-v1".to_owned(),
                payload: "opaque".to_owned(),
            }),
        );
        let mut overloads = OverloadIndex::new();
        let overload = overloads.insert(
            OverloadKey::new(namespace, "P", SymbolKind::Predicate, None),
            vec![symbol.clone()],
            local,
        );
        contributions.add_symbol(local, symbol.clone());
        contributions.add_overload_group(local, overload);

        (
            symbols.get(&symbol).unwrap().signature().cloned(),
            symbols
                .iter()
                .map(|entry| entry.symbol().fqn().as_str().to_owned())
                .collect(),
            contributions
                .affected_by(local)
                .unwrap()
                .overload_groups()
                .iter()
                .map(|id| id.index())
                .collect(),
        )
    }

    fn import_export_ids(
        source_id: SourceId,
        module: ModuleId,
    ) -> (ResolvedImportId, ResolvedExportId) {
        import_export_id_pair(source_id, module).0
    }

    fn import_export_id_pair(
        source_id: SourceId,
        module: ModuleId,
    ) -> (
        (ResolvedImportId, ResolvedExportId),
        (ResolvedImportId, ResolvedExportId),
    ) {
        let origin = origin(source_id, module.clone());
        let mut builder = ResolvedArenaBuilder::new();
        let first_import_node = builder
            .push(ResolvedNode::new(
                SurfaceNodeKind::ImportItem,
                Vec::new(),
                origin.clone(),
            ))
            .unwrap();
        let first_export_node = builder
            .push(ResolvedNode::new(
                SurfaceNodeKind::ExportItem,
                Vec::new(),
                origin.clone(),
            ))
            .unwrap();
        let second_import_node = builder
            .push(ResolvedNode::new(
                SurfaceNodeKind::ImportItem,
                Vec::new(),
                origin.clone(),
            ))
            .unwrap();
        let second_export_node = builder
            .push(ResolvedNode::new(
                SurfaceNodeKind::ExportItem,
                Vec::new(),
                origin.clone(),
            ))
            .unwrap();
        let mut imports = ResolvedImports::new();
        let first_import_id = imports.push_import(ResolvedImport::new(
            first_import_node,
            range(source_id, 0, 1),
            "import dep;",
            None,
            crate::resolved_ast::ImportResolution::Resolved(module_id("pkg", "dep")),
            origin.clone(),
        ));
        let second_import_id = imports.push_import(ResolvedImport::new(
            second_import_node,
            range(source_id, 2, 3),
            "import other;",
            None,
            crate::resolved_ast::ImportResolution::Resolved(module_id("pkg", "other")),
            origin.clone(),
        ));
        let first_export_id = imports.push_export(ResolvedExport::new(
            first_export_node,
            range(source_id, 1, 2),
            "export dep;",
            ExportTarget::Module(module.clone()),
            origin.clone(),
        ));
        let second_export_id = imports.push_export(ResolvedExport::new(
            second_export_node,
            range(source_id, 3, 4),
            "export other;",
            ExportTarget::Module(module),
            origin,
        ));
        (
            (first_import_id, first_export_id),
            (second_import_id, second_export_id),
        )
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
