//! Symbol environment data shapes.

mod snapshot;

use crate::resolved_ast::{
    FullyQualifiedName, LabelKind, LabelOriginPath, LabelRefId, ModuleId, NameRefId, RecoveryState,
    ResolvedExportId, ResolvedImportId, SemanticOrigin, SymbolId,
};
use mizar_session::{GeneratedSpanAnchor, SourceAnchor, SourceId, SourceRange};
use std::cmp::Ordering;

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

/// Stable id for a module lexical summary entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LexicalSummaryId(usize);

impl LexicalSummaryId {
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
    /// Lemma declaration represented as a symbol.
    Lemma,
    /// Algorithm declaration represented by signature and contracts.
    Algorithm,
    /// Scheme declaration represented as a symbol.
    Scheme,
    /// Template declaration represented as a symbol.
    Template,
    /// Synonym relation declaration.
    Synonym,
    /// Antonym relation declaration.
    Antonym,
    /// Redefinition relation declaration.
    Redefinition,
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
    /// Theorem statement shell.
    Theorem,
    /// Lemma statement shell.
    Lemma,
    /// Algorithm signature and contract shell.
    Algorithm,
    /// Scheme signature shell.
    Scheme,
    /// Template signature shell.
    Template,
    /// Synonym relation shell.
    Synonym,
    /// Antonym relation shell.
    Antonym,
    /// Redefinition relation shell.
    Redefinition,
    /// Structure selector or member shell.
    Selector,
}

/// Duplicate or conflict classification for declaration collection.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum DeclarationConflictClass {
    /// Duplicate declaration spelling.
    DuplicateSpelling,
    /// Illegal overload group.
    IllegalOverloadGroup,
    /// Same argument-signature functor declarations with incompatible return
    /// signatures.
    SameSignatureReturnConflict,
    /// Recovered declaration shell.
    RecoveredShell,
    /// Same argument-signature functor declarations with identical return
    /// signatures.
    SameSignatureDefinitionConflict,
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

/// Module lexical summary family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum LexicalSummaryKind {
    /// Public symbolic or identifier notation.
    Notation,
    /// Public selector/member spelling.
    Selector,
    /// Public constructor-like spelling.
    Constructor,
}

/// Public lexical spelling exported by a module for downstream lexing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexicalSummaryEntry {
    id: LexicalSummaryId,
    symbol: SymbolId,
    namespace: NamespacePath,
    spelling: String,
    kind: LexicalSummaryKind,
    arity: Option<u32>,
    contribution: SourceContributionId,
}

impl LexicalSummaryEntry {
    /// Returns the lexical summary id.
    pub const fn id(&self) -> LexicalSummaryId {
        self.id
    }

    /// Returns the source symbol id.
    pub const fn symbol(&self) -> &SymbolId {
        &self.symbol
    }

    /// Returns the namespace path.
    pub const fn namespace(&self) -> &NamespacePath {
        &self.namespace
    }

    /// Returns the public lexical spelling.
    pub fn spelling(&self) -> &str {
        &self.spelling
    }

    /// Returns the summary family.
    pub const fn kind(&self) -> LexicalSummaryKind {
        self.kind
    }

    /// Returns syntactic arity when available.
    pub const fn arity(&self) -> Option<u32> {
        self.arity
    }

    /// Returns source contribution id.
    pub const fn contribution(&self) -> SourceContributionId {
        self.contribution
    }
}

impl Ord for LexicalSummaryEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.namespace
            .cmp(other.namespace())
            .then_with(|| self.spelling.cmp(&other.spelling))
            .then_with(|| self.kind.cmp(&other.kind))
            .then_with(|| self.arity.cmp(&other.arity))
            .then_with(|| self.symbol.cmp(other.symbol()))
            .then_with(|| self.id.cmp(&other.id))
    }
}

impl PartialOrd for LexicalSummaryEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Module lexical summary index.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModuleLexicalSummaryIndex {
    next_id: usize,
    entries: Vec<LexicalSummaryEntry>,
}

impl ModuleLexicalSummaryIndex {
    /// Creates an empty lexical summary index.
    pub const fn new() -> Self {
        Self {
            next_id: 0,
            entries: Vec::new(),
        }
    }

    /// Inserts a lexical summary entry.
    pub fn insert(
        &mut self,
        symbol: SymbolId,
        namespace: NamespacePath,
        spelling: impl Into<String>,
        kind: LexicalSummaryKind,
        arity: Option<u32>,
        contribution: SourceContributionId,
    ) -> LexicalSummaryId {
        let id = LexicalSummaryId::new(self.next_id);
        self.next_id += 1;
        self.entries.push(LexicalSummaryEntry {
            id,
            symbol,
            namespace,
            spelling: spelling.into(),
            kind,
            arity,
            contribution,
        });
        self.entries.sort();
        id
    }

    /// Returns a lexical summary entry by id.
    pub fn get(&self, id: LexicalSummaryId) -> Option<&LexicalSummaryEntry> {
        self.entries.iter().find(|entry| entry.id() == id)
    }

    /// Returns entries visible under a namespace and spelling.
    pub fn visible_candidates(
        &self,
        namespace: &NamespacePath,
        spelling: &str,
    ) -> Vec<&LexicalSummaryEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.namespace() == namespace && entry.spelling() == spelling)
            .collect()
    }

    /// Returns summaries produced by a contribution.
    pub fn by_contribution(&self, contribution: SourceContributionId) -> Vec<&LexicalSummaryEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.contribution() == contribution)
            .collect()
    }

    /// Iterates lexical summaries in deterministic order.
    pub fn iter(&self) -> impl Iterator<Item = &LexicalSummaryEntry> {
        self.entries.iter()
    }

    /// Returns the number of summaries.
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
    lexical_summaries: Vec<LexicalSummaryId>,
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

    /// Returns affected lexical summaries.
    pub fn lexical_summaries(&self) -> &[LexicalSummaryId] {
        &self.lexical_summaries
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

    /// Records an affected lexical summary.
    pub fn add_lexical_summary(&mut self, id: SourceContributionId, summary: LexicalSummaryId) {
        if let Some(record) = self.get_mut(id) {
            insert_sorted_unique(&mut record.effects.lexical_summaries, summary);
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
    /// Module lexical summary index.
    pub lexical_summaries: ModuleLexicalSummaryIndex,
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

    /// Returns the module lexical summary index.
    pub const fn lexical_summaries(&self) -> &ModuleLexicalSummaryIndex {
        &self.indexes.lexical_summaries
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
        snapshot::symbol_env_snapshot_text(self)
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
mod tests;
