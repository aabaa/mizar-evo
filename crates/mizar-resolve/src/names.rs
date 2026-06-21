//! Namespace and symbol-name resolution.
//!
//! This module implements the R-013 namespace slice, the R-014 ordinary
//! symbol-name lookup slice, the R-015 internal name-diagnostic slice, and the
//! R-016 dot-chain finalization slice. It resolves source-shaped namespace path
//! candidates to canonical module namespaces, resolves ordinary name references
//! through preliminary symbol projections, finalizes dotted chains as
//! checker-deferred selectors or namespace-qualified symbol references, and
//! keeps unresolved/ambiguous diagnostic roots explicit without checking
//! selectors, choosing overload winners, assigning public diagnostic codes, or
//! assigning full signature-bearing symbol entries.

mod diagnostics;

use crate::env::{NamespacePath, SymbolKind, Visibility};
use crate::imports::{ImportPathFailureClass, ResolvedImportCandidate, UnresolvedImportCandidate};
use crate::module_index::{
    IndexedModuleId, ModuleIndexInput, ModuleIndexProviderError, NamespaceIndexEntry, NamespaceRoot,
};
use crate::resolved_ast::{
    AmbiguousNameRef, BuiltinId, BuiltinRef, DeferredSelectorRef, ModuleId, NameLookupClass,
    NameRefEntry, NameRefId, NameRefTable, NameResolution, NameResolutionCandidate, ReferenceSite,
    ResolvedNodeId, SemanticOrigin, SymbolId, SymbolRef,
};
use mizar_session::{ModulePath, PackageId, SourceRange};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

/// One represented namespace path segment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamespacePathSegment {
    spelling: String,
    range: SourceRange,
}

impl NamespacePathSegment {
    /// Creates a namespace path segment.
    #[must_use]
    pub fn new(spelling: impl Into<String>, range: SourceRange) -> Self {
        Self {
            spelling: spelling.into(),
            range,
        }
    }

    /// Returns the represented spelling.
    #[must_use]
    pub fn spelling(&self) -> &str {
        &self.spelling
    }

    /// Returns the segment source range.
    #[must_use]
    pub const fn range(&self) -> SourceRange {
        self.range
    }
}

/// Source-shaped namespace path candidate collected before symbol lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamespacePathCandidate {
    segments: Vec<NamespacePathSegment>,
    range: SourceRange,
    ordinal: usize,
    recovered: bool,
}

impl NamespacePathCandidate {
    /// Creates a namespace path candidate.
    #[must_use]
    pub fn new(segments: Vec<NamespacePathSegment>, range: SourceRange, ordinal: usize) -> Self {
        Self {
            segments,
            range,
            ordinal,
            recovered: false,
        }
    }

    /// Marks this candidate as parser-recovered.
    #[must_use]
    pub const fn with_recovered(mut self) -> Self {
        self.recovered = true;
        self
    }

    /// Returns represented path segments.
    #[must_use]
    pub fn segments(&self) -> &[NamespacePathSegment] {
        &self.segments
    }

    /// Returns the candidate source range.
    #[must_use]
    pub const fn range(&self) -> SourceRange {
        self.range
    }

    /// Returns the source-order ordinal.
    #[must_use]
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }

    /// Returns whether parser recovery was involved.
    #[must_use]
    pub const fn recovered(&self) -> bool {
        self.recovered
    }

    fn spelling(&self) -> String {
        self.segments
            .iter()
            .map(|segment| segment.spelling())
            .collect::<Vec<_>>()
            .join(".")
    }
}

/// Deterministic namespace resolution result.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NamespacePathResolution {
    resolved: Vec<ResolvedNamespacePath>,
    unresolved: Vec<UnresolvedNamespacePath>,
}

impl NamespacePathResolution {
    fn new(
        mut resolved: Vec<ResolvedNamespacePath>,
        mut unresolved: Vec<UnresolvedNamespacePath>,
    ) -> Self {
        resolved.sort_by(resolved_namespace_path_cmp);
        unresolved.sort_by(unresolved_namespace_path_cmp);
        Self {
            resolved,
            unresolved,
        }
    }

    /// Returns resolved namespace paths in deterministic source order.
    #[must_use]
    pub fn resolved(&self) -> &[ResolvedNamespacePath] {
        &self.resolved
    }

    /// Returns unresolved namespace paths in deterministic source order.
    #[must_use]
    pub fn unresolved(&self) -> &[UnresolvedNamespacePath] {
        &self.unresolved
    }

    /// Returns whether any namespace path failed.
    #[must_use]
    pub const fn has_unresolved(&self) -> bool {
        !self.unresolved.is_empty()
    }
}

/// Resolved namespace path target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedNamespacePath {
    target: ModuleId,
    origin: NamespaceResolutionOrigin,
    spelling: String,
    segments: Vec<NamespacePathSegment>,
    range: SourceRange,
    ordinal: usize,
}

impl ResolvedNamespacePath {
    fn new(
        candidate: &NamespacePathCandidate,
        target: ModuleId,
        origin: NamespaceResolutionOrigin,
    ) -> Self {
        Self {
            target,
            origin,
            spelling: candidate.spelling(),
            segments: candidate.segments.clone(),
            range: candidate.range,
            ordinal: candidate.ordinal,
        }
    }

    /// Returns the canonical module namespace target.
    #[must_use]
    pub const fn target(&self) -> &ModuleId {
        &self.target
    }

    /// Returns namespace resolution provenance.
    #[must_use]
    pub const fn origin(&self) -> &NamespaceResolutionOrigin {
        &self.origin
    }

    /// Returns the represented path spelling.
    #[must_use]
    pub fn spelling(&self) -> &str {
        &self.spelling
    }

    /// Returns represented path segments.
    #[must_use]
    pub fn segments(&self) -> &[NamespacePathSegment] {
        &self.segments
    }

    /// Returns the candidate source range.
    #[must_use]
    pub const fn range(&self) -> SourceRange {
        self.range
    }

    /// Returns the source-order ordinal.
    #[must_use]
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
}

/// Namespace resolution provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum NamespaceResolutionOrigin {
    /// Local import alias.
    ImportAlias {
        /// Alias spelling.
        alias: String,
        /// Alias source range when represented.
        alias_range: Option<SourceRange>,
    },
    /// Reserved namespace root such as `std` or `pub`.
    ReservedRoot {
        /// Reserved root.
        root: NamespaceRoot,
        /// Matched package namespace prefix after the root.
        matched_prefix: Vec<String>,
        /// Package selected by the root binding.
        package: PackageId,
    },
    /// Package-name namespace binding.
    PackageNameBinding {
        /// Matched package-name prefix.
        matched_prefix: Vec<String>,
        /// Package selected by the binding.
        package: PackageId,
    },
    /// Current package fallback.
    CurrentPackage {
        /// Current package identity.
        package: PackageId,
    },
}

/// Unresolved namespace path retained for recovery and diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnresolvedNamespacePath {
    spelling: String,
    segments: Vec<NamespacePathSegment>,
    range: SourceRange,
    ordinal: usize,
    class: NamespaceFailureClass,
    failed_segment: Option<NamespacePathSegment>,
    partial: Option<NamespacePartialCandidate>,
    import_dependencies: Vec<NamespaceImportDependency>,
    candidate_targets: Vec<NamespaceCandidateTarget>,
    recovered: bool,
}

impl UnresolvedNamespacePath {
    fn from_candidate(
        candidate: &NamespacePathCandidate,
        class: NamespaceFailureClass,
        failed_segment: Option<NamespacePathSegment>,
        partial: Option<NamespacePartialCandidate>,
        import_dependencies: Vec<NamespaceImportDependency>,
        candidate_targets: Vec<NamespaceCandidateTarget>,
    ) -> Self {
        let mut import_dependencies = import_dependencies;
        import_dependencies.sort_by(namespace_import_dependency_cmp);
        let mut candidate_targets = candidate_targets;
        candidate_targets.sort_by(namespace_candidate_target_cmp);
        Self {
            spelling: candidate.spelling(),
            segments: candidate.segments.clone(),
            range: candidate.range,
            ordinal: candidate.ordinal,
            class,
            failed_segment,
            partial,
            import_dependencies,
            candidate_targets,
            recovered: candidate.recovered,
        }
    }

    /// Returns the represented path spelling.
    #[must_use]
    pub fn spelling(&self) -> &str {
        &self.spelling
    }

    /// Returns represented path segments.
    #[must_use]
    pub fn segments(&self) -> &[NamespacePathSegment] {
        &self.segments
    }

    /// Returns the candidate source range.
    #[must_use]
    pub const fn range(&self) -> SourceRange {
        self.range
    }

    /// Returns the source-order ordinal.
    #[must_use]
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }

    /// Returns the crate-local failure class.
    #[must_use]
    pub const fn class(&self) -> NamespaceFailureClass {
        self.class
    }

    /// Returns the earliest decisive failing segment, when known.
    #[must_use]
    pub const fn failed_segment(&self) -> Option<&NamespacePathSegment> {
        self.failed_segment.as_ref()
    }

    /// Returns partial namespace provenance, if available.
    #[must_use]
    pub const fn partial(&self) -> Option<&NamespacePartialCandidate> {
        self.partial.as_ref()
    }

    /// Returns unresolved import dependencies that caused this namespace
    /// failure.
    #[must_use]
    pub fn import_dependencies(&self) -> &[NamespaceImportDependency] {
        &self.import_dependencies
    }

    /// Returns deterministic candidate targets for ambiguous namespace
    /// records.
    #[must_use]
    pub fn candidate_targets(&self) -> &[NamespaceCandidateTarget] {
        &self.candidate_targets
    }

    /// Returns whether parser recovery was involved.
    #[must_use]
    pub const fn recovered(&self) -> bool {
        self.recovered
    }
}

/// Crate-local namespace failure class. These are not public diagnostic codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum NamespaceFailureClass {
    /// No usable path segments were represented.
    EmptyPath,
    /// Parser recovery was involved.
    RecoveredSyntax,
    /// A namespace root, package binding, or namespace segment was unknown.
    UnknownNamespaceSegment,
    /// A package was known, but the module namespace was absent.
    UnknownModule,
    /// An import alias names multiple canonical modules.
    AmbiguousImportAlias,
    /// A referenced import alias exists only through unresolved import records.
    UnresolvedImportAlias,
    /// Provider state was inconsistent with the namespace index.
    ProviderError,
    /// The candidate shape was not semantically usable.
    IllegalCandidateState,
}

/// Unresolved import dependency provenance for a namespace failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamespaceImportDependency {
    alias: String,
    range: SourceRange,
    alias_range: Option<SourceRange>,
    ordinal: usize,
    class: ImportPathFailureClass,
}

impl NamespaceImportDependency {
    fn new(
        alias: String,
        range: SourceRange,
        alias_range: Option<SourceRange>,
        ordinal: usize,
        class: ImportPathFailureClass,
    ) -> Self {
        Self {
            alias,
            range,
            alias_range,
            ordinal,
            class,
        }
    }

    /// Returns the referenced alias spelling.
    #[must_use]
    pub fn alias(&self) -> &str {
        &self.alias
    }

    /// Returns the unresolved import source range.
    #[must_use]
    pub const fn range(&self) -> SourceRange {
        self.range
    }

    /// Returns the explicit alias range when one was represented.
    #[must_use]
    pub const fn alias_range(&self) -> Option<SourceRange> {
        self.alias_range
    }

    /// Returns the source-order import ordinal.
    #[must_use]
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }

    /// Returns the import failure class.
    #[must_use]
    pub const fn class(&self) -> ImportPathFailureClass {
        self.class
    }
}

/// Candidate target provenance for ambiguous namespace records.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamespaceCandidateTarget {
    target: ModuleId,
    range: SourceRange,
    alias_range: Option<SourceRange>,
    ordinal: usize,
}

impl NamespaceCandidateTarget {
    fn new(
        target: ModuleId,
        range: SourceRange,
        alias_range: Option<SourceRange>,
        ordinal: usize,
    ) -> Self {
        Self {
            target,
            range,
            alias_range,
            ordinal,
        }
    }

    /// Returns the canonical candidate target.
    #[must_use]
    pub const fn target(&self) -> &ModuleId {
        &self.target
    }

    /// Returns the import source range.
    #[must_use]
    pub const fn range(&self) -> SourceRange {
        self.range
    }

    /// Returns the explicit alias range when one was represented.
    #[must_use]
    pub const fn alias_range(&self) -> Option<SourceRange> {
        self.alias_range
    }

    /// Returns the source-order import ordinal.
    #[must_use]
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
}

/// Partial namespace provenance for unresolved records.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamespacePartialCandidate {
    origin: NamespacePartialOrigin,
    matched_prefix: Vec<String>,
    package: Option<PackageId>,
    remaining_segments: Vec<String>,
}

impl NamespacePartialCandidate {
    fn new(
        origin: NamespacePartialOrigin,
        matched_prefix: Vec<String>,
        package: Option<PackageId>,
        remaining_segments: Vec<String>,
    ) -> Self {
        Self {
            origin,
            matched_prefix,
            package,
            remaining_segments,
        }
    }

    /// Returns the partial origin.
    #[must_use]
    pub const fn origin(&self) -> NamespacePartialOrigin {
        self.origin
    }

    /// Returns matched namespace/package prefix components.
    #[must_use]
    pub fn matched_prefix(&self) -> &[String] {
        &self.matched_prefix
    }

    /// Returns matched package identity, if one was found.
    #[must_use]
    pub const fn package(&self) -> Option<&PackageId> {
        self.package.as_ref()
    }

    /// Returns remaining unresolved namespace path components.
    #[must_use]
    pub fn remaining_segments(&self) -> &[String] {
        &self.remaining_segments
    }
}

/// Source of partial namespace provenance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum NamespacePartialOrigin {
    /// Import alias provenance.
    ImportAlias,
    /// Reserved root provenance.
    ReservedRoot,
    /// Package-name binding provenance.
    PackageNameBinding,
    /// Current package fallback provenance.
    CurrentPackage,
}

/// Source provenance of a preliminary name-symbol projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum NameProjectionSource {
    /// Current-module declaration shell projection.
    CurrentModule {
        /// The declaration is visible after this source-order ordinal.
        visible_after_ordinal: usize,
    },
    /// Imported public or dependency-summary projection.
    Imported,
}

/// Preliminary symbol projection used by R-014 name lookup.
///
/// This is intentionally smaller than a complete `SymbolEnv` entry. Later
/// symbol/signature tasks refine the same `SymbolId`s with kind-specific
/// signatures and export summaries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameSymbolProjection {
    symbol: SymbolId,
    namespace: NamespacePath,
    primary_spelling: String,
    kind: SymbolKind,
    visibility: Visibility,
    declaration_range: SourceRange,
    source: NameProjectionSource,
    overload_group: Option<SymbolId>,
}

impl NameSymbolProjection {
    /// Creates a current-module declaration projection.
    #[must_use]
    pub fn current_module(
        symbol: SymbolId,
        namespace: NamespacePath,
        primary_spelling: impl Into<String>,
        kind: SymbolKind,
        visibility: Visibility,
        declaration_range: SourceRange,
        visible_after_ordinal: usize,
    ) -> Self {
        Self {
            symbol,
            namespace,
            primary_spelling: primary_spelling.into(),
            kind,
            visibility,
            declaration_range,
            source: NameProjectionSource::CurrentModule {
                visible_after_ordinal,
            },
            overload_group: None,
        }
    }

    /// Creates an imported declaration or summary projection.
    #[must_use]
    pub fn imported(
        symbol: SymbolId,
        namespace: NamespacePath,
        primary_spelling: impl Into<String>,
        kind: SymbolKind,
        visibility: Visibility,
        declaration_range: SourceRange,
    ) -> Self {
        Self {
            symbol,
            namespace,
            primary_spelling: primary_spelling.into(),
            kind,
            visibility,
            declaration_range,
            source: NameProjectionSource::Imported,
            overload_group: None,
        }
    }

    /// Attaches a resolver-visible overload-group placeholder.
    #[must_use]
    pub fn with_overload_group(mut self, overload_group: SymbolId) -> Self {
        self.overload_group = Some(overload_group);
        self
    }

    /// Returns the projected symbol id.
    #[must_use]
    pub const fn symbol(&self) -> &SymbolId {
        &self.symbol
    }

    /// Returns the namespace projection.
    #[must_use]
    pub const fn namespace(&self) -> &NamespacePath {
        &self.namespace
    }

    /// Returns the primary spelling.
    #[must_use]
    pub fn primary_spelling(&self) -> &str {
        &self.primary_spelling
    }

    /// Returns the symbol kind family.
    #[must_use]
    pub const fn kind(&self) -> SymbolKind {
        self.kind
    }

    /// Returns resolver visibility.
    #[must_use]
    pub const fn visibility(&self) -> Visibility {
        self.visibility
    }

    /// Returns the declaration/source range used for candidate ordering.
    #[must_use]
    pub const fn declaration_range(&self) -> SourceRange {
        self.declaration_range
    }

    /// Returns source provenance.
    #[must_use]
    pub const fn source(&self) -> NameProjectionSource {
        self.source
    }

    /// Returns an overload-group placeholder, if this projection belongs to one.
    #[must_use]
    pub const fn overload_group(&self) -> Option<&SymbolId> {
        self.overload_group.as_ref()
    }
}

/// Enabled builtin spelling available to name lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuiltinNameProjection {
    builtin: BuiltinId,
    spelling: String,
}

impl BuiltinNameProjection {
    /// Creates an enabled builtin projection.
    #[must_use]
    pub fn new(builtin: BuiltinId, spelling: impl Into<String>) -> Self {
        Self {
            builtin,
            spelling: spelling.into(),
        }
    }

    /// Returns the builtin id.
    #[must_use]
    pub const fn builtin(&self) -> &BuiltinId {
        &self.builtin
    }

    /// Returns the source spelling.
    #[must_use]
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
}

/// Scope for an ordinary name-reference candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum NameReferenceScope {
    /// Unqualified lookup in the current namespace.
    Unqualified,
    /// Qualified lookup restricted to one namespace target.
    Qualified {
        /// Canonical module namespace.
        module: ModuleId,
        /// Namespace projection used by symbol entries.
        namespace: NamespacePath,
    },
    /// Lookup depends on a namespace path that already failed.
    FailedNamespace,
}

/// Source-shaped ordinary name reference collected for R-014 lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameReferenceCandidate {
    site: ReferenceSite,
    origin: SemanticOrigin,
    ordinal: usize,
    scope: NameReferenceScope,
}

impl NameReferenceCandidate {
    /// Creates an unqualified name-reference candidate.
    #[must_use]
    pub const fn unqualified(site: ReferenceSite, origin: SemanticOrigin, ordinal: usize) -> Self {
        Self {
            site,
            origin,
            ordinal,
            scope: NameReferenceScope::Unqualified,
        }
    }

    /// Creates a module-qualified name-reference candidate.
    #[must_use]
    pub fn qualified_module(
        site: ReferenceSite,
        origin: SemanticOrigin,
        ordinal: usize,
        module: ModuleId,
    ) -> Self {
        let namespace = NamespacePath::new(module.path().as_str());
        Self::qualified(site, origin, ordinal, module, namespace)
    }

    /// Creates a qualified name-reference candidate.
    #[must_use]
    pub const fn qualified(
        site: ReferenceSite,
        origin: SemanticOrigin,
        ordinal: usize,
        module: ModuleId,
        namespace: NamespacePath,
    ) -> Self {
        Self {
            site,
            origin,
            ordinal,
            scope: NameReferenceScope::Qualified { module, namespace },
        }
    }

    /// Creates a candidate blocked by an unresolved namespace.
    #[must_use]
    pub const fn failed_namespace(
        site: ReferenceSite,
        origin: SemanticOrigin,
        ordinal: usize,
    ) -> Self {
        Self {
            site,
            origin,
            ordinal,
            scope: NameReferenceScope::FailedNamespace,
        }
    }

    /// Returns the reference site.
    #[must_use]
    pub const fn site(&self) -> &ReferenceSite {
        &self.site
    }

    /// Returns normalized provenance.
    #[must_use]
    pub const fn origin(&self) -> &SemanticOrigin {
        &self.origin
    }

    /// Returns the source-order ordinal.
    #[must_use]
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }

    /// Returns the lookup scope.
    #[must_use]
    pub const fn scope(&self) -> &NameReferenceScope {
        &self.scope
    }
}

/// Lexical scope key for local term bindings.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocalTermScope {
    path: Vec<u32>,
}

impl LocalTermScope {
    /// Creates a scope key from a stable lexical path.
    #[must_use]
    pub fn new(path: impl Into<Vec<u32>>) -> Self {
        Self { path: path.into() }
    }

    /// Returns the lexical path.
    #[must_use]
    pub fn path(&self) -> &[u32] {
        &self.path
    }

    fn contains(&self, other: &Self) -> bool {
        other.path.starts_with(&self.path)
    }
}

/// Local term/binder name that can shadow a namespace segment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalTermBinding {
    spelling: String,
    scope: LocalTermScope,
    declaration_range: SourceRange,
    visible_after_ordinal: usize,
}

impl LocalTermBinding {
    /// Creates a local term binding.
    #[must_use]
    pub fn new(
        spelling: impl Into<String>,
        scope: LocalTermScope,
        declaration_range: SourceRange,
        visible_after_ordinal: usize,
    ) -> Self {
        Self {
            spelling: spelling.into(),
            scope,
            declaration_range,
            visible_after_ordinal,
        }
    }

    /// Returns the local binding spelling.
    #[must_use]
    pub fn spelling(&self) -> &str {
        &self.spelling
    }

    /// Returns the lexical scope that owns this binding.
    #[must_use]
    pub const fn scope(&self) -> &LocalTermScope {
        &self.scope
    }

    /// Returns the declaration range.
    #[must_use]
    pub const fn declaration_range(&self) -> SourceRange {
        self.declaration_range
    }

    /// Returns the source-order ordinal after which this binding is visible.
    #[must_use]
    pub const fn visible_after_ordinal(&self) -> usize {
        self.visible_after_ordinal
    }
}

/// One represented segment in a source-shaped dotted chain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DotChainSegment {
    spelling: String,
    range: SourceRange,
}

impl DotChainSegment {
    /// Creates a dotted-chain segment.
    #[must_use]
    pub fn new(spelling: impl Into<String>, range: SourceRange) -> Self {
        Self {
            spelling: spelling.into(),
            range,
        }
    }

    /// Returns the segment spelling.
    #[must_use]
    pub fn spelling(&self) -> &str {
        &self.spelling
    }

    /// Returns the segment range.
    #[must_use]
    pub const fn range(&self) -> SourceRange {
        self.range
    }
}

/// Source-shaped dotted chain that needs selector-vs-namespace finalization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DotChainCandidate {
    segments: Vec<DotChainSegment>,
    site: ReferenceSite,
    origin: SemanticOrigin,
    base_node: ResolvedNodeId,
    scope: LocalTermScope,
    ordinal: usize,
    recovered: bool,
}

impl DotChainCandidate {
    /// Creates a dotted-chain candidate.
    #[must_use]
    pub fn new(
        segments: Vec<DotChainSegment>,
        site: ReferenceSite,
        origin: SemanticOrigin,
        base_node: ResolvedNodeId,
        scope: LocalTermScope,
        ordinal: usize,
    ) -> Self {
        Self {
            segments,
            site,
            origin,
            base_node,
            scope,
            ordinal,
            recovered: false,
        }
    }

    /// Marks this candidate as parser-recovered.
    #[must_use]
    pub const fn with_recovered(mut self) -> Self {
        self.recovered = true;
        self
    }

    /// Returns represented segments.
    #[must_use]
    pub fn segments(&self) -> &[DotChainSegment] {
        &self.segments
    }

    /// Returns the full-chain reference site.
    #[must_use]
    pub const fn site(&self) -> &ReferenceSite {
        &self.site
    }

    /// Returns normalized provenance.
    #[must_use]
    pub const fn origin(&self) -> &SemanticOrigin {
        &self.origin
    }

    /// Returns the use-site base term node used by deferred selector records.
    #[must_use]
    pub const fn base_node(&self) -> ResolvedNodeId {
        self.base_node
    }

    /// Returns the use-site lexical scope.
    #[must_use]
    pub const fn scope(&self) -> &LocalTermScope {
        &self.scope
    }

    /// Returns the source-order ordinal.
    #[must_use]
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }

    /// Returns whether parser recovery was involved.
    #[must_use]
    pub const fn recovered(&self) -> bool {
        self.recovered
    }

    fn spelling(&self) -> String {
        self.segments
            .iter()
            .map(|segment| segment.spelling())
            .collect::<Vec<_>>()
            .join(".")
    }
}

/// Deterministic dot-chain finalization result.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DotChainResolution {
    table: NameRefTable,
    ids: Vec<NameRefId>,
    namespaces: NamespacePathResolution,
}

impl DotChainResolution {
    fn new(table: NameRefTable, ids: Vec<NameRefId>, namespaces: NamespacePathResolution) -> Self {
        Self {
            table,
            ids,
            namespaces,
        }
    }

    /// Returns the populated name-reference table.
    #[must_use]
    pub const fn table(&self) -> &NameRefTable {
        &self.table
    }

    /// Returns table ids in deterministic chain order.
    #[must_use]
    pub fn ids(&self) -> &[NameRefId] {
        &self.ids
    }

    /// Returns namespace path outcomes produced while finalizing chains.
    #[must_use]
    pub const fn namespaces(&self) -> &NamespacePathResolution {
        &self.namespaces
    }
}

/// Deterministic name-reference lookup result.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NameReferenceResolution {
    table: NameRefTable,
    ids: Vec<NameRefId>,
}

impl NameReferenceResolution {
    fn new(table: NameRefTable, ids: Vec<NameRefId>) -> Self {
        Self { table, ids }
    }

    /// Returns the populated name-reference table.
    #[must_use]
    pub const fn table(&self) -> &NameRefTable {
        &self.table
    }

    /// Returns table ids in deterministic candidate order.
    #[must_use]
    pub fn ids(&self) -> &[NameRefId] {
        &self.ids
    }

    /// Returns whether any name reference failed.
    #[must_use]
    pub fn has_unresolved(&self) -> bool {
        self.ids.iter().any(|id| {
            self.table
                .get(*id)
                .is_some_and(|entry| matches!(entry.resolution(), NameResolution::Unresolved(_)))
        })
    }
}

/// Stable id for an internal resolver diagnostic record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NameDiagnosticId(usize);

impl NameDiagnosticId {
    const fn new(index: usize) -> Self {
        Self(index)
    }

    /// Returns the zero-based diagnostic index.
    #[must_use]
    pub const fn index(self) -> usize {
        self.0
    }
}

/// Stable id for a crate-local unresolved or ambiguous diagnostic root.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NameDiagnosticRootId(usize);

impl NameDiagnosticRootId {
    const fn new(index: usize) -> Self {
        Self(index)
    }

    /// Returns the zero-based root index.
    #[must_use]
    pub const fn index(self) -> usize {
        self.0
    }
}

/// Whether a diagnostic is the primary report for a root or a dependent
/// cascade record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum NameDiagnosticRole {
    /// Primary report for a distinct unresolved or ambiguous root.
    Primary,
    /// Dependent record linked to a root so user-facing diagnostics can avoid
    /// cascaded primaries.
    Cascade,
}

/// Crate-local name-resolution diagnostic class.
///
/// These classes are not public user-facing diagnostic codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum NameDiagnosticKind {
    /// A name reference could not be resolved.
    UnresolvedName {
        /// Lookup layer that failed.
        lookup: NameLookupClass,
    },
    /// Multiple visible symbols remained at the names phase.
    AmbiguousName,
    /// A namespace path could not be resolved.
    UnresolvedNamespace {
        /// Crate-local namespace failure class.
        class: NamespaceFailureClass,
    },
    /// A namespace path or alias had multiple deterministic targets.
    AmbiguousNamespace {
        /// Crate-local namespace failure class.
        class: NamespaceFailureClass,
    },
    /// A namespace failure depends on an unresolved import alias.
    UnresolvedImportAliasDependency {
        /// Crate-local import failure class.
        class: ImportPathFailureClass,
    },
}

/// Namespace candidate payload retained by internal name diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameDiagnosticNamespaceCandidate {
    stable_variant: &'static str,
    target: ModuleId,
    range: SourceRange,
    alias_range: Option<SourceRange>,
    ordinal: usize,
}

impl NameDiagnosticNamespaceCandidate {
    fn from_namespace_target(target: &NamespaceCandidateTarget) -> Self {
        Self {
            stable_variant: "import-alias-target",
            target: target.target().clone(),
            range: target.range(),
            alias_range: target.alias_range(),
            ordinal: target.ordinal(),
        }
    }

    /// Returns the stable candidate variant name used for deterministic keys.
    #[must_use]
    pub const fn stable_variant(&self) -> &'static str {
        self.stable_variant
    }

    /// Returns the canonical namespace target.
    #[must_use]
    pub const fn target(&self) -> &ModuleId {
        &self.target
    }

    /// Returns the source range that introduced this candidate.
    #[must_use]
    pub const fn range(&self) -> SourceRange {
        self.range
    }

    /// Returns the explicit alias range when represented.
    #[must_use]
    pub const fn alias_range(&self) -> Option<SourceRange> {
        self.alias_range
    }

    /// Returns the source-order ordinal.
    #[must_use]
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
}

/// Internal resolver diagnostic record for R-015.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameDiagnostic {
    id: NameDiagnosticId,
    root: NameDiagnosticRootId,
    role: NameDiagnosticRole,
    kind: NameDiagnosticKind,
    root_range: SourceRange,
    range: SourceRange,
    attempted_spelling: String,
    normalized_namespace_prefix: Vec<String>,
    name_ref: Option<NameRefId>,
    secondary_ranges: Vec<SourceRange>,
    symbol_candidates: Vec<NameResolutionCandidate>,
    namespace_candidates: Vec<NameDiagnosticNamespaceCandidate>,
    dependent_ranges: Vec<SourceRange>,
}

impl NameDiagnostic {
    /// Returns this diagnostic record id.
    #[must_use]
    pub const fn id(&self) -> NameDiagnosticId {
        self.id
    }

    /// Returns the root id shared by cascaded diagnostics.
    #[must_use]
    pub const fn root(&self) -> NameDiagnosticRootId {
        self.root
    }

    /// Returns whether this is a primary or cascade record.
    #[must_use]
    pub const fn role(&self) -> NameDiagnosticRole {
        self.role
    }

    /// Returns the crate-local diagnostic kind.
    #[must_use]
    pub const fn kind(&self) -> NameDiagnosticKind {
        self.kind
    }

    /// Returns the primary root range used for deterministic grouping and
    /// sorting.
    #[must_use]
    pub const fn root_range(&self) -> SourceRange {
        self.root_range
    }

    /// Returns this record's own source range.
    #[must_use]
    pub const fn range(&self) -> SourceRange {
        self.range
    }

    /// Returns the attempted spelling associated with this record.
    #[must_use]
    pub fn attempted_spelling(&self) -> &str {
        &self.attempted_spelling
    }

    /// Returns the normalized namespace prefix, when known.
    #[must_use]
    pub fn normalized_namespace_prefix(&self) -> &[String] {
        &self.normalized_namespace_prefix
    }

    /// Returns the linked name-reference table id, when this record came from
    /// `NameRefTable`.
    #[must_use]
    pub const fn name_ref(&self) -> Option<NameRefId> {
        self.name_ref
    }

    /// Returns secondary source ranges that explain the root.
    #[must_use]
    pub fn secondary_ranges(&self) -> &[SourceRange] {
        &self.secondary_ranges
    }

    /// Returns deterministic ambiguous symbol candidates.
    #[must_use]
    pub fn symbol_candidates(&self) -> &[NameResolutionCandidate] {
        &self.symbol_candidates
    }

    /// Returns deterministic ambiguous namespace candidates.
    #[must_use]
    pub fn namespace_candidates(&self) -> &[NameDiagnosticNamespaceCandidate] {
        &self.namespace_candidates
    }

    /// Returns dependent source ranges linked to the same root.
    #[must_use]
    pub fn dependent_ranges(&self) -> &[SourceRange] {
        &self.dependent_ranges
    }
}

/// Deterministic crate-local diagnostics for unresolved and ambiguous names.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NameDiagnosticReport {
    records: Vec<NameDiagnostic>,
}

impl NameDiagnosticReport {
    fn new(mut records: Vec<NameDiagnostic>) -> Self {
        records.sort_by(diagnostics::name_diagnostic_cmp);
        for (index, record) in records.iter_mut().enumerate() {
            record.id = NameDiagnosticId::new(index);
        }
        Self { records }
    }

    /// Returns a diagnostic by id.
    #[must_use]
    pub fn get(&self, id: NameDiagnosticId) -> Option<&NameDiagnostic> {
        self.records.get(id.index())
    }

    /// Iterates diagnostics in deterministic order.
    pub fn iter(&self) -> impl Iterator<Item = &NameDiagnostic> {
        self.records.iter()
    }

    /// Iterates primary diagnostics only.
    pub fn primary(&self) -> impl Iterator<Item = &NameDiagnostic> {
        self.records
            .iter()
            .filter(|diagnostic| diagnostic.role() == NameDiagnosticRole::Primary)
    }

    /// Iterates cascade diagnostics only.
    pub fn cascades(&self) -> impl Iterator<Item = &NameDiagnostic> {
        self.records
            .iter()
            .filter(|diagnostic| diagnostic.role() == NameDiagnosticRole::Cascade)
    }

    /// Returns the number of diagnostic records.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.records.len()
    }

    /// Returns whether the report is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

/// Collects crate-local name diagnostics without assigning public diagnostic
/// codes.
#[derive(Clone, Copy)]
pub struct NameDiagnosticCollector<'a> {
    namespace_roots: &'a [UnresolvedNamespacePath],
}

impl<'a> NameDiagnosticCollector<'a> {
    /// Creates a collector with no namespace-root side input.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            namespace_roots: &[],
        }
    }

    /// Provides unresolved namespace roots that failed before a final symbol
    /// spelling was reached.
    #[must_use]
    pub const fn with_namespace_roots(
        mut self,
        namespace_roots: &'a [UnresolvedNamespacePath],
    ) -> Self {
        self.namespace_roots = namespace_roots;
        self
    }

    /// Collects diagnostics from a populated name-reference table.
    #[must_use]
    pub fn collect(self, name_refs: &NameRefTable) -> NameDiagnosticReport {
        diagnostics::collect_name_diagnostics(name_refs, self.namespace_roots)
    }

    /// Collects diagnostics from a name-reference resolution result.
    #[must_use]
    pub fn collect_resolution(self, resolution: &NameReferenceResolution) -> NameDiagnosticReport {
        self.collect(resolution.table())
    }
}

impl Default for NameDiagnosticCollector<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// Resolves ordinary symbol-name references after namespace lookup.
#[derive(Clone, Copy)]
pub struct SymbolNameResolver<'a> {
    projections: &'a [NameSymbolProjection],
    builtins: &'a [BuiltinNameProjection],
}

impl<'a> SymbolNameResolver<'a> {
    /// Creates a symbol-name resolver over preliminary projections.
    #[must_use]
    pub const fn new(
        projections: &'a [NameSymbolProjection],
        builtins: &'a [BuiltinNameProjection],
    ) -> Self {
        Self {
            projections,
            builtins,
        }
    }

    /// Resolves name-reference candidates for the current module.
    #[must_use]
    pub fn resolve(
        self,
        current_module: &ModuleId,
        current_namespace: &NamespacePath,
        candidates: &[NameReferenceCandidate],
    ) -> NameReferenceResolution {
        let mut ordered = candidates.iter().collect::<Vec<_>>();
        ordered.sort_by(|left, right| name_reference_candidate_cmp(left, right));
        let mut table = NameRefTable::new();
        let mut ids = Vec::with_capacity(ordered.len());
        for candidate in ordered {
            let resolution = self.resolve_one(current_module, current_namespace, candidate);
            let id = table.insert(NameRefEntry::new(
                candidate.site().clone(),
                resolution,
                candidate.origin().clone(),
            ));
            ids.push(id);
        }
        NameReferenceResolution::new(table, ids)
    }

    fn resolve_one(
        self,
        current_module: &ModuleId,
        current_namespace: &NamespacePath,
        candidate: &NameReferenceCandidate,
    ) -> NameResolution {
        let spelling = candidate.site().spelling();
        if candidate.origin().is_recovered() || spelling.is_empty() {
            return unresolved_name(candidate, NameLookupClass::Symbol);
        }
        match candidate.scope() {
            NameReferenceScope::Unqualified => {
                self.resolve_unqualified(current_module, current_namespace, candidate)
            }
            NameReferenceScope::Qualified { module, namespace } => {
                self.resolve_qualified(current_module, module, namespace, candidate)
            }
            NameReferenceScope::FailedNamespace => {
                unresolved_name(candidate, NameLookupClass::Namespace)
            }
        }
    }

    fn resolve_unqualified(
        self,
        current_module: &ModuleId,
        current_namespace: &NamespacePath,
        candidate: &NameReferenceCandidate,
    ) -> NameResolution {
        let current = self
            .projections
            .iter()
            .filter(|projection| {
                projection_matches(projection, current_namespace, candidate.site().spelling())
                    && current_projection_visible(projection, current_module, candidate.ordinal())
            })
            .collect::<Vec<_>>();
        let imported = self
            .projections
            .iter()
            .filter(|projection| {
                projection_matches(projection, current_namespace, candidate.site().spelling())
                    && imported_projection_visible(projection)
            })
            .collect::<Vec<_>>();
        let effective = effective_unqualified_candidates(current, imported);
        if !effective.is_empty() {
            return resolution_from_symbol_candidates(candidate, effective);
        }
        if let Some(builtin) = self
            .builtins
            .iter()
            .filter(|builtin| builtin.spelling() == candidate.site().spelling())
            .min_by(|left, right| left.builtin().cmp(right.builtin()))
        {
            return NameResolution::ResolvedBuiltin(BuiltinRef::new(
                builtin.builtin().clone(),
                candidate.site().range(),
                candidate.site().spelling(),
            ));
        }
        unresolved_name(candidate, NameLookupClass::Symbol)
    }

    fn resolve_qualified(
        self,
        current_module: &ModuleId,
        module: &ModuleId,
        namespace: &NamespacePath,
        candidate: &NameReferenceCandidate,
    ) -> NameResolution {
        let candidates = self
            .projections
            .iter()
            .filter(|projection| {
                projection_matches(projection, namespace, candidate.site().spelling())
                    && qualified_projection_visible(
                        projection,
                        current_module,
                        module,
                        candidate.ordinal(),
                    )
            })
            .collect::<Vec<_>>();
        if candidates.is_empty() {
            unresolved_name(candidate, NameLookupClass::Symbol)
        } else {
            resolution_from_symbol_candidates(candidate, candidates)
        }
    }
}

/// Finalizes dotted chains as either selector access or namespace-qualified
/// symbol lookup.
#[derive(Clone, Copy)]
pub struct DotChainFinalizer<'a> {
    namespace_resolver: NamespaceResolver<'a>,
    symbol_resolver: SymbolNameResolver<'a>,
    local_terms: &'a [LocalTermBinding],
}

struct DotChainFinalizeContext<'a> {
    current_module: &'a ModuleId,
    current_namespace: &'a NamespacePath,
    resolved_imports: &'a [ResolvedImportCandidate],
    unresolved_imports: &'a [UnresolvedImportCandidate],
}

#[derive(Default)]
struct DotChainNamespaceSink {
    resolved: Vec<ResolvedNamespacePath>,
    unresolved: Vec<UnresolvedNamespacePath>,
}

impl<'a> DotChainFinalizer<'a> {
    /// Creates a dot-chain finalizer.
    #[must_use]
    pub const fn new(
        namespace_resolver: NamespaceResolver<'a>,
        symbol_resolver: SymbolNameResolver<'a>,
        local_terms: &'a [LocalTermBinding],
    ) -> Self {
        Self {
            namespace_resolver,
            symbol_resolver,
            local_terms,
        }
    }

    /// Finalizes source-shaped dotted chains for the current module.
    #[must_use]
    pub fn finalize(
        self,
        current_module: &ModuleId,
        current_namespace: &NamespacePath,
        resolved_imports: &[ResolvedImportCandidate],
        unresolved_imports: &[UnresolvedImportCandidate],
        chains: &[DotChainCandidate],
    ) -> DotChainResolution {
        let mut ordered = chains.iter().collect::<Vec<_>>();
        ordered.sort_by(|left, right| dot_chain_candidate_cmp(left, right));

        let mut table = NameRefTable::new();
        let mut ids = Vec::with_capacity(ordered.len());
        let mut namespaces = DotChainNamespaceSink::default();
        let context = DotChainFinalizeContext {
            current_module,
            current_namespace,
            resolved_imports,
            unresolved_imports,
        };

        for chain in ordered {
            let id = self.finalize_one(&context, chain, &mut table, &mut namespaces);
            ids.push(id);
        }

        DotChainResolution::new(
            table,
            ids,
            NamespacePathResolution::new(namespaces.resolved, namespaces.unresolved),
        )
    }

    fn finalize_one(
        self,
        context: &DotChainFinalizeContext<'_>,
        chain: &DotChainCandidate,
        table: &mut NameRefTable,
        namespaces: &mut DotChainNamespaceSink,
    ) -> NameRefId {
        let resolution = self.resolve_chain(context, chain, namespaces);
        let origin = if chain.recovered() {
            chain.origin().clone().recovered()
        } else {
            chain.origin().clone()
        };
        table.insert(NameRefEntry::new(chain.site().clone(), resolution, origin))
    }

    fn resolve_chain(
        self,
        context: &DotChainFinalizeContext<'_>,
        chain: &DotChainCandidate,
        namespaces: &mut DotChainNamespaceSink,
    ) -> NameResolution {
        let Some(malformed) = malformed_dot_chain_segment(chain) else {
            if chain.segments().len() < 2 {
                return unresolved_dot_chain(
                    chain,
                    chain.site().range(),
                    NameLookupClass::Selector,
                );
            }
            if chain.recovered() || chain.origin().is_recovered() {
                return unresolved_dot_chain(
                    chain,
                    chain.site().range(),
                    NameLookupClass::Selector,
                );
            }
            if self.local_term_binding(chain).is_some() {
                return NameResolution::DeferredSelector(DeferredSelectorRef::new(
                    chain.base_node(),
                    dot_chain_member_spelling(chain),
                    chain.site().range(),
                ));
            }
            return self.resolve_namespace_chain(context, chain, namespaces);
        };
        unresolved_dot_chain(chain, malformed.range(), NameLookupClass::Selector)
    }

    fn resolve_namespace_chain(
        self,
        context: &DotChainFinalizeContext<'_>,
        chain: &DotChainCandidate,
        namespaces: &mut DotChainNamespaceSink,
    ) -> NameResolution {
        let namespace_candidate = namespace_candidate_from_dot_chain(chain);
        let namespace_resolution = self.namespace_resolver.resolve(
            context.current_module,
            context.resolved_imports,
            context.unresolved_imports,
            &[namespace_candidate],
        );
        namespaces
            .resolved
            .extend(namespace_resolution.resolved().iter().cloned());
        namespaces
            .unresolved
            .extend(namespace_resolution.unresolved().iter().cloned());

        if let Some(unresolved) = namespace_resolution.unresolved().first() {
            let range = unresolved
                .failed_segment()
                .map(NamespacePathSegment::range)
                .unwrap_or_else(|| unresolved.range());
            return NameResolution::Unresolved(crate::resolved_ast::UnresolvedNameRef::new(
                unresolved.spelling(),
                range,
                NameLookupClass::Namespace,
            ));
        }

        let Some(namespace) = namespace_resolution.resolved().first() else {
            return unresolved_dot_chain(chain, chain.site().range(), NameLookupClass::Namespace);
        };
        let Some(final_segment) = chain.segments().last() else {
            return unresolved_dot_chain(chain, chain.site().range(), NameLookupClass::Selector);
        };
        let name_candidate = NameReferenceCandidate::qualified_module(
            ReferenceSite::new(
                chain.site().node(),
                final_segment.range(),
                final_segment.spelling(),
            ),
            chain.origin().clone(),
            chain.ordinal(),
            namespace.target().clone(),
        );
        let name_resolution = self.symbol_resolver.resolve(
            context.current_module,
            context.current_namespace,
            &[name_candidate],
        );
        let Some(id) = name_resolution.ids().first() else {
            return unresolved_dot_chain(chain, final_segment.range(), NameLookupClass::Symbol);
        };
        name_resolution
            .table()
            .get(*id)
            .map(|entry| entry.resolution().clone())
            .unwrap_or_else(|| {
                unresolved_dot_chain(chain, final_segment.range(), NameLookupClass::Symbol)
            })
    }

    fn local_term_binding(self, chain: &DotChainCandidate) -> Option<&'a LocalTermBinding> {
        let first = chain.segments().first()?;
        self.local_terms
            .iter()
            .filter(|binding| local_term_binding_visible(binding, chain, first.spelling()))
            .max_by(|left, right| local_term_binding_cmp(left, right))
    }
}

/// Resolves source-shaped namespace path candidates.
#[derive(Clone, Copy)]
pub struct NamespaceResolver<'a> {
    module_index: ModuleIndexInput<'a>,
}

impl<'a> NamespaceResolver<'a> {
    /// Creates a namespace resolver backed by the module-index input seam.
    #[must_use]
    pub const fn new(module_index: ModuleIndexInput<'a>) -> Self {
        Self { module_index }
    }

    /// Resolves namespace path candidates for the current module.
    #[must_use]
    pub fn resolve(
        self,
        current_module: &ModuleId,
        resolved_imports: &[ResolvedImportCandidate],
        unresolved_imports: &[UnresolvedImportCandidate],
        candidates: &[NamespacePathCandidate],
    ) -> NamespacePathResolution {
        let alias_bindings = import_alias_bindings(resolved_imports);
        let unresolved_alias_bindings = unresolved_import_alias_bindings(unresolved_imports);
        let mut ordered = candidates.iter().collect::<Vec<_>>();
        ordered.sort_by(|left, right| namespace_path_candidate_cmp(left, right));

        let mut resolved = Vec::new();
        let mut unresolved = Vec::new();
        for candidate in ordered {
            match self.resolve_one(
                current_module,
                &alias_bindings,
                &unresolved_alias_bindings,
                candidate,
            ) {
                NamespaceCandidateResolution::Resolved(path) => resolved.push(path),
                NamespaceCandidateResolution::Unresolved(path) => unresolved.push(path),
            }
        }
        NamespacePathResolution::new(resolved, unresolved)
    }

    fn resolve_one(
        self,
        current_module: &ModuleId,
        alias_bindings: &BTreeMap<String, Vec<ImportAliasBinding>>,
        unresolved_alias_bindings: &BTreeMap<String, Vec<UnresolvedImportAliasBinding>>,
        candidate: &NamespacePathCandidate,
    ) -> NamespaceCandidateResolution {
        if candidate.recovered() {
            return unresolved(
                candidate,
                NamespaceFailureClass::RecoveredSyntax,
                candidate.segments().first().cloned(),
                None,
            );
        }

        let Some(first) = candidate.segments().first() else {
            return unresolved(candidate, NamespaceFailureClass::EmptyPath, None, None);
        };
        if first.spelling().is_empty() {
            return unresolved(
                candidate,
                NamespaceFailureClass::IllegalCandidateState,
                Some(first.clone()),
                None,
            );
        }

        if let Some(bindings) = alias_bindings.get(first.spelling()) {
            return resolve_import_alias(candidate, bindings);
        }
        if let Some(bindings) = unresolved_alias_bindings.get(first.spelling()) {
            return resolve_unresolved_import_alias(candidate, bindings);
        }

        let components = candidate
            .segments()
            .iter()
            .map(|segment| segment.spelling().to_owned())
            .collect::<Vec<_>>();
        if let Some(root) = reserved_namespace_root(&components[0]) {
            return self.resolve_reserved_root(candidate, root, &components[1..]);
        }

        if let Some(binding) =
            longest_namespace_binding(self.module_index.namespace_bindings(), &components)
        {
            return self.resolve_binding(
                candidate,
                binding,
                &components,
                0,
                NamespacePartialOrigin::PackageNameBinding,
            );
        }

        self.resolve_current_package(current_module, candidate)
    }

    fn resolve_reserved_root(
        self,
        candidate: &NamespacePathCandidate,
        root: NamespaceRoot,
        components_after_root: &[String],
    ) -> NamespaceCandidateResolution {
        let Some(binding) = longest_root_binding(
            self.module_index.namespace_bindings(),
            root,
            components_after_root,
        ) else {
            let failed_segment = candidate
                .segments()
                .get(1)
                .or_else(|| candidate.segments().first())
                .cloned();
            return unresolved(
                candidate,
                NamespaceFailureClass::UnknownNamespaceSegment,
                failed_segment,
                Some(NamespacePartialCandidate::new(
                    NamespacePartialOrigin::ReservedRoot,
                    Vec::new(),
                    None,
                    components_after_root.to_vec(),
                )),
            );
        };

        self.resolve_binding(
            candidate,
            binding,
            components_after_root,
            1,
            NamespacePartialOrigin::ReservedRoot,
        )
    }

    fn resolve_binding(
        self,
        candidate: &NamespacePathCandidate,
        binding: &NamespaceIndexEntry,
        components: &[String],
        segment_offset: usize,
        origin: NamespacePartialOrigin,
    ) -> NamespaceCandidateResolution {
        let remaining_components = components[binding.prefix.len()..].to_vec();
        let remaining_segments =
            candidate.segments()[segment_offset + binding.prefix.len()..].to_vec();
        let partial = NamespacePartialCandidate::new(
            origin,
            binding.prefix.clone(),
            Some(binding.package_id.clone()),
            remaining_components.clone(),
        );
        let resolution_origin = match origin {
            NamespacePartialOrigin::ReservedRoot => NamespaceResolutionOrigin::ReservedRoot {
                root: binding.root,
                matched_prefix: binding.prefix.clone(),
                package: binding.package_id.clone(),
            },
            NamespacePartialOrigin::PackageNameBinding => {
                NamespaceResolutionOrigin::PackageNameBinding {
                    matched_prefix: binding.prefix.clone(),
                    package: binding.package_id.clone(),
                }
            }
            NamespacePartialOrigin::ImportAlias | NamespacePartialOrigin::CurrentPackage => {
                return unresolved(
                    candidate,
                    NamespaceFailureClass::IllegalCandidateState,
                    candidate.segments().first().cloned(),
                    Some(partial),
                );
            }
        };
        if self.module_index.package(&binding.package_id).is_err() {
            return unresolved(
                candidate,
                NamespaceFailureClass::ProviderError,
                provider_error_binding_segment(candidate, segment_offset, binding.prefix.len()),
                Some(partial),
            );
        }
        self.resolve_package_module(
            candidate,
            &binding.package_id,
            &remaining_components,
            &remaining_segments,
            resolution_origin,
            partial,
        )
    }

    fn resolve_current_package(
        self,
        current_module: &ModuleId,
        candidate: &NamespacePathCandidate,
    ) -> NamespaceCandidateResolution {
        let components = candidate
            .segments()
            .iter()
            .map(|segment| segment.spelling().to_owned())
            .collect::<Vec<_>>();
        let partial = NamespacePartialCandidate::new(
            NamespacePartialOrigin::CurrentPackage,
            Vec::new(),
            Some(current_module.package().clone()),
            components.clone(),
        );
        self.resolve_package_module(
            candidate,
            current_module.package(),
            &components,
            candidate.segments(),
            NamespaceResolutionOrigin::CurrentPackage {
                package: current_module.package().clone(),
            },
            partial,
        )
    }

    fn resolve_package_module(
        self,
        candidate: &NamespacePathCandidate,
        package: &PackageId,
        path_components: &[String],
        path_segments: &[NamespacePathSegment],
        origin: NamespaceResolutionOrigin,
        partial: NamespacePartialCandidate,
    ) -> NamespaceCandidateResolution {
        let Some(module_path) = module_path_from_components(path_components) else {
            let failed_segment = path_segments
                .last()
                .cloned()
                .or_else(|| candidate.segments().last().cloned());
            return unresolved(
                candidate,
                NamespaceFailureClass::UnknownModule,
                failed_segment,
                Some(partial),
            );
        };
        let indexed = IndexedModuleId::new(package.clone(), module_path);
        match self.module_index.module(&indexed) {
            Ok(entry) => NamespaceCandidateResolution::Resolved(ResolvedNamespacePath::new(
                candidate,
                self.module_index.module_identity(entry),
                origin,
            )),
            Err(ModuleIndexProviderError::UnknownModule { .. }) => {
                let failed_segment = match self.first_missing_module_segment(package, path_segments)
                {
                    Ok(segment) => segment
                        .or_else(|| path_segments.last().cloned())
                        .or_else(|| candidate.segments().last().cloned()),
                    Err(_) => {
                        return unresolved(
                            candidate,
                            NamespaceFailureClass::ProviderError,
                            path_segments
                                .first()
                                .cloned()
                                .or_else(|| candidate.segments().first().cloned()),
                            Some(partial),
                        );
                    }
                };
                unresolved(
                    candidate,
                    NamespaceFailureClass::UnknownModule,
                    failed_segment,
                    Some(partial),
                )
            }
            Err(_) => unresolved(
                candidate,
                NamespaceFailureClass::ProviderError,
                path_segments
                    .first()
                    .cloned()
                    .or_else(|| candidate.segments().first().cloned()),
                Some(partial),
            ),
        }
    }

    fn first_missing_module_segment(
        self,
        package: &PackageId,
        path_segments: &[NamespacePathSegment],
    ) -> Result<Option<NamespacePathSegment>, ModuleIndexProviderError> {
        let modules = self.module_index.modules_for_package(package)?;
        let module_paths = modules
            .iter()
            .map(|entry| split_module_path(&entry.module.path))
            .collect::<Vec<_>>();
        let mut prefix = Vec::<String>::new();
        for segment in path_segments {
            prefix.push(segment.spelling().to_owned());
            if module_paths.iter().any(|path| path.starts_with(&prefix)) {
                continue;
            }
            return Ok(Some(segment.clone()));
        }
        Ok(path_segments.last().cloned())
    }
}

fn projection_matches(
    projection: &NameSymbolProjection,
    namespace: &NamespacePath,
    spelling: &str,
) -> bool {
    projection.namespace() == namespace && projection.primary_spelling() == spelling
}

fn current_projection_visible(
    projection: &NameSymbolProjection,
    current_module: &ModuleId,
    use_ordinal: usize,
) -> bool {
    if projection.symbol().module() != current_module {
        return false;
    }
    match projection.source() {
        NameProjectionSource::CurrentModule {
            visible_after_ordinal,
        } => visible_after_ordinal < use_ordinal,
        NameProjectionSource::Imported => false,
    }
}

fn imported_projection_visible(projection: &NameSymbolProjection) -> bool {
    matches!(projection.source(), NameProjectionSource::Imported)
        && projection.visibility() == Visibility::Public
}

fn qualified_projection_visible(
    projection: &NameSymbolProjection,
    current_module: &ModuleId,
    target_module: &ModuleId,
    use_ordinal: usize,
) -> bool {
    if projection.symbol().module() != target_module {
        return false;
    }
    if target_module == current_module {
        match projection.source() {
            NameProjectionSource::CurrentModule {
                visible_after_ordinal,
            } => visible_after_ordinal < use_ordinal,
            NameProjectionSource::Imported => projection.visibility() == Visibility::Public,
        }
    } else {
        matches!(projection.source(), NameProjectionSource::Imported)
            && projection.visibility() == Visibility::Public
    }
}

fn effective_unqualified_candidates<'a>(
    mut current: Vec<&'a NameSymbolProjection>,
    imported: Vec<&'a NameSymbolProjection>,
) -> Vec<&'a NameSymbolProjection> {
    if current.is_empty() {
        return sorted_name_symbol_projections(imported);
    }
    let current_groups = current
        .iter()
        .filter_map(|projection| projection.overload_group().cloned())
        .collect::<BTreeSet<_>>();
    current.extend(imported.into_iter().filter(|projection| {
        projection
            .overload_group()
            .is_some_and(|group| current_groups.contains(group))
    }));
    sorted_name_symbol_projections(current)
}

fn resolution_from_symbol_candidates(
    candidate: &NameReferenceCandidate,
    projections: Vec<&NameSymbolProjection>,
) -> NameResolution {
    let projections = sorted_name_symbol_projections(projections);
    if projections.len() == 1 {
        return NameResolution::Resolved(
            SymbolRef::new(projections[0].symbol().clone(), candidate.site().range())
                .with_spelling(candidate.site().spelling()),
        );
    }
    if let Some(group) = collapsed_overload_group(&projections) {
        return NameResolution::Resolved(
            SymbolRef::new(group, candidate.site().range())
                .with_spelling(candidate.site().spelling()),
        );
    }
    NameResolution::Ambiguous(AmbiguousNameRef::new(
        candidate.site().spelling(),
        candidate.site().range(),
        projections
            .iter()
            .map(|projection| {
                NameResolutionCandidate::new(
                    projection.symbol().clone(),
                    projection.declaration_range(),
                )
            })
            .collect(),
    ))
}

fn collapsed_overload_group(projections: &[&NameSymbolProjection]) -> Option<SymbolId> {
    if projections.len() < 2 {
        return None;
    }
    let mut groups = BTreeSet::<SymbolId>::new();
    for projection in projections {
        groups.insert(projection.overload_group()?.clone());
    }
    (groups.len() == 1).then(|| groups.into_iter().next().unwrap())
}

fn unresolved_name(candidate: &NameReferenceCandidate, lookup: NameLookupClass) -> NameResolution {
    NameResolution::Unresolved(crate::resolved_ast::UnresolvedNameRef::new(
        candidate.site().spelling(),
        candidate.site().range(),
        lookup,
    ))
}

fn unresolved_dot_chain(
    chain: &DotChainCandidate,
    range: SourceRange,
    lookup: NameLookupClass,
) -> NameResolution {
    NameResolution::Unresolved(crate::resolved_ast::UnresolvedNameRef::new(
        chain.spelling(),
        range,
        lookup,
    ))
}

fn malformed_dot_chain_segment(chain: &DotChainCandidate) -> Option<&DotChainSegment> {
    chain
        .segments()
        .iter()
        .find(|segment| segment.spelling().is_empty())
}

fn namespace_candidate_from_dot_chain(chain: &DotChainCandidate) -> NamespacePathCandidate {
    let namespace_segments = chain
        .segments()
        .iter()
        .take(chain.segments().len().saturating_sub(1))
        .map(|segment| NamespacePathSegment::new(segment.spelling(), segment.range()))
        .collect::<Vec<_>>();
    NamespacePathCandidate::new(
        namespace_segments,
        dot_chain_namespace_range(chain),
        chain.ordinal(),
    )
}

fn dot_chain_namespace_range(chain: &DotChainCandidate) -> SourceRange {
    let first = chain.segments().first().map(DotChainSegment::range);
    let namespace_len = chain.segments().len().saturating_sub(1);
    let last = namespace_len
        .checked_sub(1)
        .and_then(|index| chain.segments().get(index))
        .map(DotChainSegment::range);
    match (first, last) {
        (Some(first), Some(last)) => SourceRange {
            source_id: first.source_id,
            start: first.start,
            end: last.end,
        },
        _ => chain.site().range(),
    }
}

fn dot_chain_member_spelling(chain: &DotChainCandidate) -> String {
    chain
        .segments()
        .iter()
        .skip(1)
        .map(|segment| segment.spelling())
        .collect::<Vec<_>>()
        .join(".")
}

fn local_term_binding_visible(
    binding: &LocalTermBinding,
    chain: &DotChainCandidate,
    spelling: &str,
) -> bool {
    binding.spelling() == spelling
        && binding.visible_after_ordinal() < chain.ordinal()
        && binding.scope().contains(chain.scope())
}

fn local_term_binding_cmp(left: &LocalTermBinding, right: &LocalTermBinding) -> Ordering {
    left.scope()
        .path()
        .len()
        .cmp(&right.scope().path().len())
        .then_with(|| {
            left.visible_after_ordinal()
                .cmp(&right.visible_after_ordinal())
        })
        .then_with(|| {
            range_key(left.declaration_range()).cmp(&range_key(right.declaration_range()))
        })
        .then_with(|| left.spelling().cmp(right.spelling()))
}

fn sorted_name_symbol_projections(
    mut projections: Vec<&NameSymbolProjection>,
) -> Vec<&NameSymbolProjection> {
    projections.sort_by(|left, right| name_symbol_projection_cmp(left, right));
    projections
}

fn name_symbol_projection_cmp(
    left: &NameSymbolProjection,
    right: &NameSymbolProjection,
) -> Ordering {
    left.symbol()
        .cmp(right.symbol())
        .then_with(|| left.namespace().cmp(right.namespace()))
        .then_with(|| left.primary_spelling().cmp(right.primary_spelling()))
        .then_with(|| left.kind().cmp(&right.kind()))
        .then_with(|| left.visibility().cmp(&right.visibility()))
        .then_with(|| {
            range_key(left.declaration_range()).cmp(&range_key(right.declaration_range()))
        })
        .then_with(|| left.source().cmp(&right.source()))
}

fn name_reference_candidate_cmp(
    left: &NameReferenceCandidate,
    right: &NameReferenceCandidate,
) -> Ordering {
    left.ordinal()
        .cmp(&right.ordinal())
        .then_with(|| range_key(left.site().range()).cmp(&range_key(right.site().range())))
        .then_with(|| left.site().spelling().cmp(right.site().spelling()))
}

fn dot_chain_candidate_cmp(left: &DotChainCandidate, right: &DotChainCandidate) -> Ordering {
    left.ordinal()
        .cmp(&right.ordinal())
        .then_with(|| range_key(left.site().range()).cmp(&range_key(right.site().range())))
        .then_with(|| left.spelling().cmp(&right.spelling()))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ImportAliasBinding {
    alias: String,
    target: ModuleId,
    alias_range: Option<SourceRange>,
    range: SourceRange,
    ordinal: usize,
}

fn import_alias_bindings(
    imports: &[ResolvedImportCandidate],
) -> BTreeMap<String, Vec<ImportAliasBinding>> {
    let mut bindings = BTreeMap::<String, Vec<ImportAliasBinding>>::new();
    for import in imports {
        bindings
            .entry(import.alias().to_owned())
            .or_default()
            .push(ImportAliasBinding {
                alias: import.alias().to_owned(),
                target: import.target().clone(),
                alias_range: import.alias_range(),
                range: import.range(),
                ordinal: import.ordinal(),
            });
    }
    for values in bindings.values_mut() {
        values.sort_by(import_alias_binding_cmp);
    }
    bindings
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct UnresolvedImportAliasBinding {
    alias: String,
    range: SourceRange,
    alias_range: Option<SourceRange>,
    ordinal: usize,
    class: ImportPathFailureClass,
    candidate_target: Option<ModuleId>,
}

fn unresolved_import_alias_bindings(
    imports: &[UnresolvedImportCandidate],
) -> BTreeMap<String, Vec<UnresolvedImportAliasBinding>> {
    let mut bindings = BTreeMap::<String, Vec<UnresolvedImportAliasBinding>>::new();
    for import in imports {
        let Some(alias) = unresolved_import_alias(import) else {
            continue;
        };
        if alias.is_empty() {
            continue;
        }
        bindings
            .entry(alias.clone())
            .or_default()
            .push(UnresolvedImportAliasBinding {
                alias,
                range: import.range(),
                alias_range: import.alias_range(),
                ordinal: import.ordinal(),
                class: import.class(),
                candidate_target: import.candidate_target().cloned(),
            });
    }
    for values in bindings.values_mut() {
        values.sort_by(unresolved_import_alias_binding_cmp);
    }
    bindings
}

fn unresolved_import_alias(import: &UnresolvedImportCandidate) -> Option<String> {
    import
        .alias()
        .map(str::to_owned)
        .or_else(|| import.components().last().cloned())
}

fn resolve_import_alias(
    candidate: &NamespacePathCandidate,
    bindings: &[ImportAliasBinding],
) -> NamespaceCandidateResolution {
    let targets = bindings
        .iter()
        .map(|binding| binding.target.clone())
        .collect::<BTreeSet<_>>();
    let Some(first_binding) = bindings.first() else {
        return unresolved(
            candidate,
            NamespaceFailureClass::UnknownNamespaceSegment,
            candidate.segments().first().cloned(),
            None,
        );
    };
    let candidate_targets = namespace_candidate_targets_from_import_aliases(bindings);
    let partial = NamespacePartialCandidate::new(
        NamespacePartialOrigin::ImportAlias,
        vec![first_binding.alias.clone()],
        (targets.len() == 1).then(|| first_binding.target.package().clone()),
        candidate
            .segments()
            .iter()
            .skip(1)
            .map(|segment| segment.spelling().to_owned())
            .collect(),
    );
    if targets.len() > 1 {
        return unresolved_with_payload(
            candidate,
            NamespaceFailureClass::AmbiguousImportAlias,
            candidate.segments().first().cloned(),
            Some(partial),
            Vec::new(),
            candidate_targets,
        );
    }
    if let Some(extra) = candidate.segments().get(1) {
        return unresolved(
            candidate,
            NamespaceFailureClass::UnknownNamespaceSegment,
            Some(extra.clone()),
            Some(partial),
        );
    }
    NamespaceCandidateResolution::Resolved(ResolvedNamespacePath::new(
        candidate,
        first_binding.target.clone(),
        NamespaceResolutionOrigin::ImportAlias {
            alias: first_binding.alias.clone(),
            alias_range: first_binding.alias_range,
        },
    ))
}

fn resolve_unresolved_import_alias(
    candidate: &NamespacePathCandidate,
    bindings: &[UnresolvedImportAliasBinding],
) -> NamespaceCandidateResolution {
    let Some(first_binding) = bindings.first() else {
        return unresolved(
            candidate,
            NamespaceFailureClass::UnknownNamespaceSegment,
            candidate.segments().first().cloned(),
            None,
        );
    };
    let import_dependencies = namespace_import_dependencies_from_unresolved_aliases(bindings);
    let candidate_targets = namespace_candidate_targets_from_unresolved_aliases(bindings);
    let partial = NamespacePartialCandidate::new(
        NamespacePartialOrigin::ImportAlias,
        vec![first_binding.alias.clone()],
        None,
        candidate
            .segments()
            .iter()
            .skip(1)
            .map(|segment| segment.spelling().to_owned())
            .collect(),
    );
    let class = if candidate_targets.len() > 1 {
        NamespaceFailureClass::AmbiguousImportAlias
    } else {
        NamespaceFailureClass::UnresolvedImportAlias
    };
    unresolved_with_payload(
        candidate,
        class,
        candidate.segments().first().cloned(),
        Some(partial),
        import_dependencies,
        candidate_targets,
    )
}

fn namespace_import_dependencies_from_unresolved_aliases(
    bindings: &[UnresolvedImportAliasBinding],
) -> Vec<NamespaceImportDependency> {
    bindings
        .iter()
        .map(|binding| {
            NamespaceImportDependency::new(
                binding.alias.clone(),
                binding.range,
                binding.alias_range,
                binding.ordinal,
                binding.class,
            )
        })
        .collect()
}

fn namespace_candidate_targets_from_import_aliases(
    bindings: &[ImportAliasBinding],
) -> Vec<NamespaceCandidateTarget> {
    let mut targets = BTreeMap::<ModuleId, NamespaceCandidateTarget>::new();
    for binding in bindings {
        targets.entry(binding.target.clone()).or_insert_with(|| {
            NamespaceCandidateTarget::new(
                binding.target.clone(),
                binding.range,
                binding.alias_range,
                binding.ordinal,
            )
        });
    }
    targets.into_values().collect()
}

fn namespace_candidate_targets_from_unresolved_aliases(
    bindings: &[UnresolvedImportAliasBinding],
) -> Vec<NamespaceCandidateTarget> {
    let mut targets = BTreeMap::<ModuleId, NamespaceCandidateTarget>::new();
    for binding in bindings {
        let Some(target) = binding.candidate_target.as_ref() else {
            continue;
        };
        targets.entry(target.clone()).or_insert_with(|| {
            NamespaceCandidateTarget::new(
                target.clone(),
                binding.range,
                binding.alias_range,
                binding.ordinal,
            )
        });
    }
    targets.into_values().collect()
}

enum NamespaceCandidateResolution {
    Resolved(ResolvedNamespacePath),
    Unresolved(UnresolvedNamespacePath),
}

fn unresolved(
    candidate: &NamespacePathCandidate,
    class: NamespaceFailureClass,
    failed_segment: Option<NamespacePathSegment>,
    partial: Option<NamespacePartialCandidate>,
) -> NamespaceCandidateResolution {
    unresolved_with_payload(
        candidate,
        class,
        failed_segment,
        partial,
        Vec::new(),
        Vec::new(),
    )
}

fn unresolved_with_payload(
    candidate: &NamespacePathCandidate,
    class: NamespaceFailureClass,
    failed_segment: Option<NamespacePathSegment>,
    partial: Option<NamespacePartialCandidate>,
    import_dependencies: Vec<NamespaceImportDependency>,
    candidate_targets: Vec<NamespaceCandidateTarget>,
) -> NamespaceCandidateResolution {
    NamespaceCandidateResolution::Unresolved(UnresolvedNamespacePath::from_candidate(
        candidate,
        class,
        failed_segment,
        partial,
        import_dependencies,
        candidate_targets,
    ))
}

fn reserved_namespace_root(value: &str) -> Option<NamespaceRoot> {
    match value {
        "std" => Some(NamespaceRoot::Std),
        "pub" => Some(NamespaceRoot::Pub),
        "pkg" => Some(NamespaceRoot::Pkg),
        "dev" => Some(NamespaceRoot::Dev),
        "ext" => Some(NamespaceRoot::Ext),
        _ => None,
    }
}

fn longest_namespace_binding<'a>(
    bindings: &'a [NamespaceIndexEntry],
    components: &[String],
) -> Option<&'a NamespaceIndexEntry> {
    bindings
        .iter()
        .filter(|binding| {
            binding.root == NamespaceRoot::PackageName
                && components.starts_with(binding.prefix.as_slice())
        })
        .max_by(|left, right| namespace_binding_match_cmp(left, right))
}

fn longest_root_binding<'a>(
    bindings: &'a [NamespaceIndexEntry],
    root: NamespaceRoot,
    components: &[String],
) -> Option<&'a NamespaceIndexEntry> {
    bindings
        .iter()
        .filter(|binding| binding.root == root && components.starts_with(binding.prefix.as_slice()))
        .max_by(|left, right| namespace_binding_match_cmp(left, right))
}

fn namespace_binding_match_cmp(
    left: &NamespaceIndexEntry,
    right: &NamespaceIndexEntry,
) -> Ordering {
    left.prefix
        .len()
        .cmp(&right.prefix.len())
        .then_with(|| left.prefix.cmp(&right.prefix))
        .then_with(|| left.package_id.as_str().cmp(right.package_id.as_str()))
}

fn provider_error_binding_segment(
    candidate: &NamespacePathCandidate,
    segment_offset: usize,
    prefix_len: usize,
) -> Option<NamespacePathSegment> {
    let segment_index = if segment_offset > 0 && prefix_len == 0 {
        0
    } else {
        segment_offset
    };
    candidate
        .segments()
        .get(segment_index)
        .cloned()
        .or_else(|| candidate.segments().first().cloned())
}

fn module_path_from_components(components: &[String]) -> Option<ModulePath> {
    if components.is_empty() || components.iter().any(String::is_empty) {
        None
    } else {
        Some(ModulePath::new(components.join(".")))
    }
}

fn split_module_path(path: &ModulePath) -> Vec<String> {
    path.as_str()
        .split('.')
        .filter(|component| !component.is_empty())
        .map(str::to_owned)
        .collect()
}

fn namespace_path_candidate_cmp(
    left: &NamespacePathCandidate,
    right: &NamespacePathCandidate,
) -> Ordering {
    left.ordinal()
        .cmp(&right.ordinal())
        .then_with(|| range_key(left.range()).cmp(&range_key(right.range())))
        .then_with(|| left.spelling().cmp(&right.spelling()))
}

fn resolved_namespace_path_cmp(
    left: &ResolvedNamespacePath,
    right: &ResolvedNamespacePath,
) -> Ordering {
    left.ordinal()
        .cmp(&right.ordinal())
        .then_with(|| range_key(left.range()).cmp(&range_key(right.range())))
        .then_with(|| left.spelling().cmp(right.spelling()))
        .then_with(|| left.target().cmp(right.target()))
}

fn unresolved_namespace_path_cmp(
    left: &UnresolvedNamespacePath,
    right: &UnresolvedNamespacePath,
) -> Ordering {
    left.ordinal()
        .cmp(&right.ordinal())
        .then_with(|| range_key(left.range()).cmp(&range_key(right.range())))
        .then_with(|| left.class().cmp(&right.class()))
        .then_with(|| failed_segment_key(left).cmp(&failed_segment_key(right)))
        .then_with(|| left.spelling().cmp(right.spelling()))
}

fn import_alias_binding_cmp(left: &ImportAliasBinding, right: &ImportAliasBinding) -> Ordering {
    left.ordinal
        .cmp(&right.ordinal)
        .then_with(|| range_key(left.range).cmp(&range_key(right.range)))
        .then_with(|| left.target.cmp(&right.target))
}

fn unresolved_import_alias_binding_cmp(
    left: &UnresolvedImportAliasBinding,
    right: &UnresolvedImportAliasBinding,
) -> Ordering {
    left.ordinal
        .cmp(&right.ordinal)
        .then_with(|| range_key(left.range).cmp(&range_key(right.range)))
        .then_with(|| left.alias.cmp(&right.alias))
        .then_with(|| left.class.cmp(&right.class))
        .then_with(|| left.candidate_target.cmp(&right.candidate_target))
}

fn namespace_import_dependency_cmp(
    left: &NamespaceImportDependency,
    right: &NamespaceImportDependency,
) -> Ordering {
    left.ordinal
        .cmp(&right.ordinal)
        .then_with(|| range_key(left.range).cmp(&range_key(right.range)))
        .then_with(|| left.alias.cmp(&right.alias))
        .then_with(|| left.class.cmp(&right.class))
}

fn namespace_candidate_target_cmp(
    left: &NamespaceCandidateTarget,
    right: &NamespaceCandidateTarget,
) -> Ordering {
    left.target
        .cmp(&right.target)
        .then_with(|| left.ordinal.cmp(&right.ordinal))
        .then_with(|| range_key(left.range).cmp(&range_key(right.range)))
}

fn failed_segment_key(path: &UnresolvedNamespacePath) -> Vec<String> {
    path.failed_segment()
        .map(|segment| {
            vec![
                segment.spelling().to_owned(),
                segment.range().start.to_string(),
                segment.range().end.to_string(),
            ]
        })
        .unwrap_or_default()
}

const fn range_key(range: SourceRange) -> (usize, usize) {
    (range.start, range.end)
}

fn source_range_cmp(left: SourceRange, right: SourceRange) -> Ordering {
    range_key(left).cmp(&range_key(right))
}

fn range_key_string(range: SourceRange) -> String {
    format!("{}..{}", range.start, range.end)
}

#[cfg(test)]
mod tests;
