//! Namespace and symbol-name resolution.
//!
//! This module implements the R-013 namespace slice and the R-014 ordinary
//! symbol-name lookup slice. It resolves source-shaped namespace path
//! candidates to canonical module namespaces, resolves ordinary name references
//! through preliminary symbol projections, and keeps unresolved records
//! explicit without checking selectors, choosing overload winners, or assigning
//! full signature-bearing symbol entries.

use crate::env::{NamespacePath, SymbolKind, Visibility};
use crate::imports::{ImportPathFailureClass, ResolvedImportCandidate, UnresolvedImportCandidate};
use crate::module_index::{
    IndexedModuleId, ModuleIndexInput, ModuleIndexProviderError, NamespaceIndexEntry, NamespaceRoot,
};
use crate::resolved_ast::{
    AmbiguousNameRef, BuiltinId, BuiltinRef, ModuleId, NameLookupClass, NameRefEntry, NameRefId,
    NameRefTable, NameResolution, NameResolutionCandidate, ReferenceSite, SemanticOrigin, SymbolId,
    SymbolRef,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::imports::{ImportPathCandidate, ImportPathPrefix, ImportPathResolver};
    use crate::module_index::WorkspaceStubModuleIndexProvider;
    use crate::resolved_ast::{
        FullyQualifiedName, LocalSymbolId, NameResolution, ResolvedArenaBuilder, ResolvedNode,
        SymbolId,
    };
    use mizar_build::module_index::{
        DependencyModuleSummaryRef, ModuleIndexEntry, ModuleIndexLocation, PackageIndexEntry,
        PackageIndexSource,
    };
    use mizar_session::{
        BuildSnapshotId, Edition, Hash, InMemorySessionIdAllocator, SessionIdAllocator,
        SourceAnchor, SourceId,
    };
    use mizar_syntax::ast::SurfaceNodeKind;
    use semver::Version;

    #[test]
    fn unqualified_lookup_uses_declaration_point_shadowing_and_builtins() {
        let source_id = source_id();
        let current = module_id("app", "main");
        let dep = module_id("dep", "logic");
        let namespace = NamespacePath::new("main");
        let projections = vec![
            imported_projection(
                source_id,
                dep.clone(),
                namespace.clone(),
                "P",
                "dep::logic::P",
                Visibility::Public,
                0,
            ),
            current_private_projection(
                source_id,
                current.clone(),
                namespace.clone(),
                "P",
                "app::main::P",
                10,
                5,
            ),
            current_public_projection(
                source_id,
                current.clone(),
                namespace.clone(),
                "Fwd",
                "app::main::Fwd",
                20,
                20,
            ),
        ];
        let builtins = vec![
            BuiltinNameProjection::new(BuiltinId::new("builtin:P"), "P"),
            BuiltinNameProjection::new(BuiltinId::new("builtin:TRUE"), "TRUE"),
        ];
        let candidates = vec![
            name_candidate(source_id, current.clone(), 12, 50, "TRUE"),
            name_candidate(source_id, current.clone(), 10, 40, "Fwd"),
            name_candidate(source_id, current.clone(), 9, 30, "P"),
        ];

        let resolved = SymbolNameResolver::new(&projections, &builtins).resolve(
            &current,
            &namespace,
            &candidates,
        );

        assert!(resolved.has_unresolved());
        assert_resolved_symbol(&resolved, 0, "app::main::P");
        assert_unresolved(&resolved, 1, NameLookupClass::Symbol, "Fwd");
        assert_resolved_builtin(&resolved, 2, "builtin:TRUE");
    }

    #[test]
    fn qualified_lookup_restricts_namespace_and_visibility() {
        let source_id = source_id();
        let current = module_id("app", "main");
        let dep = module_id("dep", "logic");
        let current_namespace = NamespacePath::new("main");
        let dep_namespace = NamespacePath::new("logic");
        let projections = vec![
            imported_projection(
                source_id,
                dep.clone(),
                dep_namespace.clone(),
                "Q",
                "dep::logic::Q",
                Visibility::Public,
                0,
            ),
            imported_projection(
                source_id,
                dep.clone(),
                dep_namespace.clone(),
                "Secret",
                "dep::logic::Secret",
                Visibility::Private,
                1,
            ),
            current_private_projection(
                source_id,
                current.clone(),
                current_namespace.clone(),
                "Secret",
                "app::main::Secret",
                2,
                0,
            ),
        ];
        let builtins = vec![BuiltinNameProjection::new(BuiltinId::new("builtin:Q"), "Q")];
        let qualified_public = qualified_name_candidate(
            source_id,
            current.clone(),
            0,
            20,
            "Q",
            dep.clone(),
            dep_namespace.clone(),
        );
        let qualified_private_dep = qualified_name_candidate(
            source_id,
            current.clone(),
            1,
            30,
            "Secret",
            dep,
            dep_namespace,
        );
        let qualified_current_private = qualified_name_candidate(
            source_id,
            current.clone(),
            2,
            40,
            "Secret",
            current.clone(),
            current_namespace.clone(),
        );
        let qualified_missing = qualified_name_candidate(
            source_id,
            current.clone(),
            3,
            50,
            "Missing",
            current.clone(),
            current_namespace.clone(),
        );

        let resolved = SymbolNameResolver::new(&projections, &builtins).resolve(
            &current,
            &current_namespace,
            &[
                qualified_missing,
                qualified_current_private,
                qualified_private_dep,
                qualified_public,
            ],
        );

        assert_resolved_symbol(&resolved, 0, "dep::logic::Q");
        assert_unresolved(&resolved, 1, NameLookupClass::Symbol, "Secret");
        assert_resolved_symbol(&resolved, 2, "app::main::Secret");
        assert_unresolved(&resolved, 3, NameLookupClass::Symbol, "Missing");
    }

    #[test]
    fn overload_groups_collapse_without_names_phase_ambiguity() {
        let source_id = source_id();
        let current = module_id("app", "main");
        let dep = module_id("dep", "logic");
        let namespace = NamespacePath::new("main");
        let overload_group = symbol_id(current.clone(), "P#group", "app::main::P#group");
        let projections = vec![
            current_public_projection(
                source_id,
                current.clone(),
                namespace.clone(),
                "P/1",
                "app::main::P/1",
                0,
                0,
            )
            .with_overload_group(overload_group.clone()),
            imported_projection(
                source_id,
                dep.clone(),
                namespace.clone(),
                "P/3",
                "dep::logic::P/3",
                Visibility::Public,
                2,
            )
            .with_overload_group(overload_group),
            imported_projection(
                source_id,
                dep.clone(),
                namespace.clone(),
                "Q/dep",
                "dep::logic::Q",
                Visibility::Public,
                3,
            ),
            imported_projection(
                source_id,
                module_id("dep", "alt"),
                namespace.clone(),
                "Q/alt",
                "dep::alt::Q",
                Visibility::Public,
                4,
            ),
        ];
        let candidates = vec![
            name_candidate(source_id, current.clone(), 11, 40, "Q"),
            name_candidate(source_id, current.clone(), 10, 30, "P"),
        ];

        let resolved =
            SymbolNameResolver::new(&projections, &[]).resolve(&current, &namespace, &candidates);

        assert_resolved_symbol(&resolved, 0, "app::main::P#group");
        let NameResolution::Ambiguous(ambiguous) = resolved
            .table()
            .get(resolved.ids()[1])
            .unwrap()
            .resolution()
        else {
            panic!("expected ambiguous Q");
        };
        let candidates = ambiguous
            .candidates()
            .iter()
            .map(|candidate| candidate.symbol().fqn().as_str())
            .collect::<Vec<_>>();
        assert_eq!(candidates, vec!["dep::alt::Q", "dep::logic::Q"]);
    }

    #[test]
    fn failed_recovered_and_malformed_name_candidates_are_unresolved_in_order() {
        let source_id = source_id();
        let current = module_id("app", "main");
        let namespace = NamespacePath::new("main");
        let projections = vec![current_public_projection(
            source_id,
            current.clone(),
            namespace.clone(),
            "Recovered",
            "app::main::Recovered",
            0,
            0,
        )];
        let candidates = vec![
            recovered_name_candidate(source_id, current.clone(), 2, 40, "Recovered"),
            empty_name_candidate(source_id, current.clone(), 1, 30),
            failed_namespace_candidate(source_id, current.clone(), 0, 20, "Ns.Missing"),
        ];

        let resolved =
            SymbolNameResolver::new(&projections, &[]).resolve(&current, &namespace, &candidates);

        assert!(resolved.has_unresolved());
        assert_unresolved(&resolved, 0, NameLookupClass::Namespace, "Ns.Missing");
        assert_unresolved(&resolved, 1, NameLookupClass::Symbol, "");
        assert_unresolved(&resolved, 2, NameLookupClass::Symbol, "Recovered");
    }

    #[test]
    fn resolver_resolves_alias_roots_and_package_names_deterministically() {
        let source_id = source_id();
        let provider = fixture_provider();
        let input = ModuleIndexInput::new(&provider);
        let current = module_id("app", "main");
        let import_resolution = ImportPathResolver::new(input).resolve(
            &current,
            &[ImportPathCandidate::new(
                vec!["dep".to_owned(), "logic".to_owned()],
                ImportPathPrefix::Unprefixed,
                Some("Logic".to_owned()),
                range(source_id, 0, 16),
                0,
            )
            .with_alias_range(range(source_id, 14, 19))],
        );
        let candidates = vec![
            candidate(source_id, 0, 0, &["Logic"]),
            candidate(source_id, 1, 20, &["pub", "math", "algebra", "group"]),
            candidate(source_id, 2, 50, &["dep", "logic"]),
            candidate(source_id, 3, 70, &["util"]),
            candidate(source_id, 4, 90, &["std", "core"]),
            candidate(source_id, 5, 105, &["pkg", "vendor", "lib"]),
            candidate(source_id, 6, 125, &["dev", "sandbox", "tools"]),
            candidate(source_id, 7, 150, &["ext", "mirror", "logic"]),
        ];

        let resolved = NamespaceResolver::new(input).resolve(
            &current,
            import_resolution.resolved(),
            import_resolution.unresolved(),
            &candidates,
        );

        assert!(resolved.unresolved().is_empty());
        let targets = resolved
            .resolved()
            .iter()
            .map(|path| {
                (
                    path.spelling().to_owned(),
                    path.target().package().as_str().to_owned(),
                    path.target().path().as_str().to_owned(),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            targets,
            vec![
                ("Logic".to_owned(), "dep".to_owned(), "logic".to_owned()),
                (
                    "pub.math.algebra.group".to_owned(),
                    "dep".to_owned(),
                    "algebra.group".to_owned()
                ),
                ("dep.logic".to_owned(), "dep".to_owned(), "logic".to_owned()),
                ("util".to_owned(), "app".to_owned(), "util".to_owned()),
                (
                    "std.core".to_owned(),
                    "stdpkg".to_owned(),
                    "core".to_owned()
                ),
                (
                    "pkg.vendor.lib".to_owned(),
                    "pkgdep".to_owned(),
                    "lib".to_owned()
                ),
                (
                    "dev.sandbox.tools".to_owned(),
                    "devdep".to_owned(),
                    "tools".to_owned()
                ),
                (
                    "ext.mirror.logic".to_owned(),
                    "extdep".to_owned(),
                    "logic".to_owned()
                ),
            ]
        );
        assert!(matches!(
            resolved.resolved()[0].origin(),
            NamespaceResolutionOrigin::ImportAlias { alias, .. } if alias == "Logic"
        ));
        assert!(matches!(
            resolved.resolved()[1].origin(),
            NamespaceResolutionOrigin::ReservedRoot { root: NamespaceRoot::Pub, matched_prefix, .. }
                if matched_prefix == &vec!["math".to_owned()]
        ));
        assert!(matches!(
            resolved.resolved()[2].origin(),
            NamespaceResolutionOrigin::PackageNameBinding { matched_prefix, .. }
                if matched_prefix == &vec!["dep".to_owned()]
        ));
        assert!(matches!(
            resolved.resolved()[3].origin(),
            NamespaceResolutionOrigin::CurrentPackage { package } if package.as_str() == "app"
        ));
        assert!(matches!(
            resolved.resolved()[4].origin(),
            NamespaceResolutionOrigin::ReservedRoot { root: NamespaceRoot::Std, matched_prefix, .. }
                if matched_prefix.is_empty()
        ));
        assert!(matches!(
            resolved.resolved()[5].origin(),
            NamespaceResolutionOrigin::ReservedRoot { root: NamespaceRoot::Pkg, matched_prefix, .. }
                if matched_prefix == &vec!["vendor".to_owned()]
        ));
        assert!(matches!(
            resolved.resolved()[6].origin(),
            NamespaceResolutionOrigin::ReservedRoot { root: NamespaceRoot::Dev, matched_prefix, .. }
                if matched_prefix == &vec!["sandbox".to_owned()]
        ));
        assert!(matches!(
            resolved.resolved()[7].origin(),
            NamespaceResolutionOrigin::ReservedRoot { root: NamespaceRoot::Ext, matched_prefix, .. }
                if matched_prefix == &vec!["mirror".to_owned()]
        ));
    }

    #[test]
    fn missing_namespace_records_the_earliest_failing_segment_range() {
        let source_id = source_id();
        let provider = fixture_provider();
        let input = ModuleIndexInput::new(&provider);
        let current = module_id("app", "main");
        let candidates = vec![
            candidate(source_id, 0, 10, &["pub", "unknown", "logic"]),
            candidate(source_id, 1, 40, &["pub", "math", "missing"]),
            candidate(source_id, 2, 70, &["dep", "algebra", "missing"]),
        ];

        let resolved = NamespaceResolver::new(input).resolve(&current, &[], &[], &candidates);

        assert!(resolved.resolved().is_empty());
        let failures = resolved
            .unresolved()
            .iter()
            .map(|path| {
                (
                    path.spelling().to_owned(),
                    path.class(),
                    path.failed_segment().map(NamespacePathSegment::spelling),
                    path.failed_segment().map(NamespacePathSegment::range),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            failures,
            vec![
                (
                    "pub.unknown.logic".to_owned(),
                    NamespaceFailureClass::UnknownNamespaceSegment,
                    Some("unknown"),
                    Some(range(source_id, 14, 21)),
                ),
                (
                    "pub.math.missing".to_owned(),
                    NamespaceFailureClass::UnknownModule,
                    Some("missing"),
                    Some(range(source_id, 49, 56)),
                ),
                (
                    "dep.algebra.missing".to_owned(),
                    NamespaceFailureClass::UnknownModule,
                    Some("missing"),
                    Some(range(source_id, 82, 89)),
                ),
            ]
        );
    }

    #[test]
    fn longest_namespace_bindings_win_over_shorter_prefixes() {
        let source_id = source_id();
        let provider = fixture_provider();
        let input = ModuleIndexInput::new(&provider);
        let current = module_id("app", "main");
        let candidates = vec![
            candidate(source_id, 0, 0, &["dep", "nested", "logic"]),
            candidate(source_id, 1, 30, &["pub", "math", "logic", "core"]),
        ];

        let resolved = NamespaceResolver::new(input).resolve(&current, &[], &[], &candidates);

        assert!(resolved.unresolved().is_empty());
        let targets = resolved
            .resolved()
            .iter()
            .map(|path| {
                (
                    path.target().package().as_str().to_owned(),
                    path.target().path().as_str().to_owned(),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            targets,
            vec![
                ("altdep".to_owned(), "logic".to_owned()),
                ("altdep".to_owned(), "core".to_owned()),
            ]
        );
        assert!(matches!(
            resolved.resolved()[0].origin(),
            NamespaceResolutionOrigin::PackageNameBinding { matched_prefix, .. }
                if matched_prefix == &vec!["dep".to_owned(), "nested".to_owned()]
        ));
        assert!(matches!(
            resolved.resolved()[1].origin(),
            NamespaceResolutionOrigin::ReservedRoot { root: NamespaceRoot::Pub, matched_prefix, .. }
                if matched_prefix == &vec!["math".to_owned(), "logic".to_owned()]
        ));
    }

    #[test]
    fn malformed_namespace_paths_are_unresolved_in_deterministic_order() {
        let source_id = source_id();
        let provider = fixture_provider();
        let input = ModuleIndexInput::new(&provider);
        let current = module_id("app", "main");
        let candidates = vec![
            candidate(source_id, 3, 50, &["Recovered"]).with_recovered(),
            candidate(source_id, 1, 20, &[""]),
            NamespacePathCandidate::new(Vec::new(), range(source_id, 0, 0), 0),
            candidate(source_id, 2, 30, &["pub", "unknown"]),
        ];

        let resolved = NamespaceResolver::new(input).resolve(&current, &[], &[], &candidates);

        assert!(resolved.resolved().is_empty());
        let failures = resolved
            .unresolved()
            .iter()
            .map(|path| {
                (
                    path.ordinal(),
                    path.class(),
                    path.failed_segment().map(NamespacePathSegment::spelling),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            failures,
            vec![
                (0, NamespaceFailureClass::EmptyPath, None),
                (1, NamespaceFailureClass::IllegalCandidateState, Some("")),
                (
                    2,
                    NamespaceFailureClass::UnknownNamespaceSegment,
                    Some("unknown")
                ),
                (3, NamespaceFailureClass::RecoveredSyntax, Some("Recovered")),
            ]
        );
    }

    #[test]
    fn recovered_and_ambiguous_alias_paths_remain_explicit() {
        let source_id = source_id();
        let provider = fixture_provider();
        let input = ModuleIndexInput::new(&provider);
        let current = module_id("app", "main");
        let import_resolution = ImportPathResolver::new(input).resolve(
            &current,
            &[
                ImportPathCandidate::new(
                    vec!["dep".to_owned(), "logic".to_owned()],
                    ImportPathPrefix::Unprefixed,
                    Some("Shared".to_owned()),
                    range(source_id, 0, 16),
                    0,
                ),
                ImportPathCandidate::new(
                    vec!["dep".to_owned(), "algebra".to_owned(), "group".to_owned()],
                    ImportPathPrefix::Unprefixed,
                    Some("Group".to_owned()),
                    range(source_id, 18, 40),
                    1,
                ),
            ],
        );
        let mut ambiguous_imports = import_resolution.resolved().to_vec();
        let duplicate_target = ImportPathResolver::new(input)
            .resolve(
                &current,
                &[ImportPathCandidate::new(
                    vec!["app".to_owned(), "util".to_owned()],
                    ImportPathPrefix::Unprefixed,
                    Some("Shared".to_owned()),
                    range(source_id, 42, 58),
                    2,
                )],
            )
            .resolved()[0]
            .clone();
        ambiguous_imports.push(duplicate_target);
        let candidates = vec![
            candidate(source_id, 0, 60, &["Shared"]),
            candidate(source_id, 1, 70, &["Group", "extra"]),
            candidate(source_id, 2, 85, &["Recovered"]).with_recovered(),
        ];

        let resolved = NamespaceResolver::new(input).resolve(
            &current,
            &ambiguous_imports,
            import_resolution.unresolved(),
            &candidates,
        );

        assert!(resolved.resolved().is_empty());
        let failures = resolved
            .unresolved()
            .iter()
            .map(|path| {
                (
                    path.spelling().to_owned(),
                    path.class(),
                    path.failed_segment().map(NamespacePathSegment::spelling),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            failures,
            vec![
                (
                    "Shared".to_owned(),
                    NamespaceFailureClass::AmbiguousImportAlias,
                    Some("Shared"),
                ),
                (
                    "Group.extra".to_owned(),
                    NamespaceFailureClass::UnknownNamespaceSegment,
                    Some("extra"),
                ),
                (
                    "Recovered".to_owned(),
                    NamespaceFailureClass::RecoveredSyntax,
                    Some("Recovered"),
                ),
            ]
        );
        let ambiguous = &resolved.unresolved()[0];
        assert!(ambiguous.partial().unwrap().package().is_none());
        let candidate_targets = ambiguous
            .candidate_targets()
            .iter()
            .map(|target| {
                (
                    target.target().package().as_str().to_owned(),
                    target.target().path().as_str().to_owned(),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            candidate_targets,
            vec![
                ("app".to_owned(), "util".to_owned()),
                ("dep".to_owned(), "logic".to_owned()),
            ]
        );
        assert!(resolved.unresolved()[2].recovered());
    }

    #[test]
    fn duplicate_import_aliases_drive_ambiguous_namespace_payloads_deterministically() {
        let source_id = source_id();
        let provider = fixture_provider();
        let input = ModuleIndexInput::new(&provider);
        let current = module_id("app", "main");
        let import_resolution = ImportPathResolver::new(input).resolve(
            &current,
            &[
                ImportPathCandidate::new(
                    vec!["app".to_owned(), "util".to_owned()],
                    ImportPathPrefix::Unprefixed,
                    Some("Shared".to_owned()),
                    range(source_id, 30, 46),
                    9,
                ),
                ImportPathCandidate::new(
                    vec!["dep".to_owned(), "logic".to_owned()],
                    ImportPathPrefix::Unprefixed,
                    Some("Shared".to_owned()),
                    range(source_id, 0, 16),
                    3,
                ),
            ],
        );
        assert!(import_resolution.resolved().is_empty());

        let candidates = vec![
            candidate(source_id, 2, 60, &["Shared"]),
            candidate(source_id, 1, 80, &["pub", "math", "algebra", "group"]),
            candidate(source_id, 0, 110, &["util"]),
        ];

        let resolved = NamespaceResolver::new(input).resolve(
            &current,
            import_resolution.resolved(),
            import_resolution.unresolved(),
            &candidates,
        );

        let resolved_spellings = resolved
            .resolved()
            .iter()
            .map(ResolvedNamespacePath::spelling)
            .collect::<Vec<_>>();
        assert_eq!(resolved_spellings, vec!["util", "pub.math.algebra.group"]);
        let ambiguous = &resolved.unresolved()[0];
        assert_eq!(ambiguous.spelling(), "Shared");
        assert_eq!(
            ambiguous.class(),
            NamespaceFailureClass::AmbiguousImportAlias
        );
        let dependency_ordinals = ambiguous
            .import_dependencies()
            .iter()
            .map(|dependency| (dependency.ordinal(), dependency.class()))
            .collect::<Vec<_>>();
        assert_eq!(
            dependency_ordinals,
            vec![
                (3, ImportPathFailureClass::DuplicateAlias),
                (9, ImportPathFailureClass::DuplicateAlias),
            ]
        );
        let candidate_targets = ambiguous
            .candidate_targets()
            .iter()
            .map(|target| {
                (
                    target.target().package().as_str().to_owned(),
                    target.target().path().as_str().to_owned(),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            candidate_targets,
            vec![
                ("app".to_owned(), "util".to_owned()),
                ("dep".to_owned(), "logic".to_owned()),
            ]
        );
    }

    #[test]
    fn unresolved_import_alias_blocks_namespace_fallback_with_dependency_payload() {
        let source_id = source_id();
        let provider = fixture_provider();
        let input = ModuleIndexInput::new(&provider);
        let current = module_id("app", "main");
        let import_resolution = ImportPathResolver::new(input).resolve(
            &current,
            &[ImportPathCandidate::new(
                vec!["missing".to_owned(), "thing".to_owned()],
                ImportPathPrefix::Unprefixed,
                Some("util".to_owned()),
                range(source_id, 0, 18),
                0,
            )
            .with_alias_range(range(source_id, 14, 18))],
        );
        assert!(import_resolution.resolved().is_empty());

        let candidates = vec![candidate(source_id, 0, 25, &["util"])];
        let resolved = NamespaceResolver::new(input).resolve(
            &current,
            import_resolution.resolved(),
            import_resolution.unresolved(),
            &candidates,
        );

        assert!(resolved.resolved().is_empty());
        let unresolved = &resolved.unresolved()[0];
        assert_eq!(
            unresolved.class(),
            NamespaceFailureClass::UnresolvedImportAlias
        );
        assert_eq!(
            unresolved
                .failed_segment()
                .map(NamespacePathSegment::spelling),
            Some("util")
        );
        assert!(unresolved.candidate_targets().is_empty());
        let dependencies = unresolved.import_dependencies();
        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].alias(), "util");
        assert_eq!(
            dependencies[0].class(),
            ImportPathFailureClass::UnknownModule
        );
        assert_eq!(
            dependencies[0].alias_range(),
            Some(range(source_id, 14, 18))
        );
    }

    #[test]
    fn stale_namespace_bindings_are_provider_errors() {
        let source_id = source_id();
        let provider = WorkspaceStubModuleIndexProvider::new(
            vec![package("app")],
            vec![namespace(NamespaceRoot::PackageName, &["ghost"], "ghost")],
            vec![workspace_module("app", "main")],
            Vec::new(),
        );
        let input = ModuleIndexInput::new(&provider);
        let current = module_id("app", "main");
        let candidates = vec![candidate(source_id, 0, 0, &["ghost", "logic"])];

        let resolved = NamespaceResolver::new(input).resolve(&current, &[], &[], &candidates);

        assert!(resolved.resolved().is_empty());
        let unresolved = &resolved.unresolved()[0];
        assert_eq!(unresolved.class(), NamespaceFailureClass::ProviderError);
        assert_eq!(
            unresolved
                .failed_segment()
                .map(NamespacePathSegment::spelling),
            Some("ghost")
        );
    }

    #[test]
    fn stale_empty_prefix_reserved_root_bindings_report_the_root_segment() {
        let source_id = source_id();
        let provider = WorkspaceStubModuleIndexProvider::new(
            vec![package("app")],
            vec![namespace(NamespaceRoot::Std, &[], "missingstd")],
            vec![workspace_module("app", "main")],
            Vec::new(),
        );
        let input = ModuleIndexInput::new(&provider);
        let current = module_id("app", "main");
        let candidates = vec![candidate(source_id, 0, 0, &["std", "core"])];

        let resolved = NamespaceResolver::new(input).resolve(&current, &[], &[], &candidates);

        assert!(resolved.resolved().is_empty());
        let unresolved = &resolved.unresolved()[0];
        assert_eq!(unresolved.class(), NamespaceFailureClass::ProviderError);
        assert_eq!(
            unresolved
                .failed_segment()
                .map(NamespacePathSegment::spelling),
            Some("std")
        );
    }

    fn name_candidate(
        source_id: SourceId,
        module: ModuleId,
        ordinal: usize,
        start: usize,
        spelling: &str,
    ) -> NameReferenceCandidate {
        let (site, origin) = reference_site(source_id, module, start, spelling, ordinal);
        NameReferenceCandidate::unqualified(site, origin, ordinal)
    }

    fn qualified_name_candidate(
        source_id: SourceId,
        module: ModuleId,
        ordinal: usize,
        start: usize,
        spelling: &str,
        target: ModuleId,
        namespace: NamespacePath,
    ) -> NameReferenceCandidate {
        let (site, origin) = reference_site(source_id, module, start, spelling, ordinal);
        NameReferenceCandidate::qualified(site, origin, ordinal, target, namespace)
    }

    fn failed_namespace_candidate(
        source_id: SourceId,
        module: ModuleId,
        ordinal: usize,
        start: usize,
        spelling: &str,
    ) -> NameReferenceCandidate {
        let (site, origin) = reference_site(source_id, module, start, spelling, ordinal);
        NameReferenceCandidate::failed_namespace(site, origin, ordinal)
    }

    fn recovered_name_candidate(
        source_id: SourceId,
        module: ModuleId,
        ordinal: usize,
        start: usize,
        spelling: &str,
    ) -> NameReferenceCandidate {
        let (site, origin) = reference_site(source_id, module, start, spelling, ordinal);
        NameReferenceCandidate::unqualified(site, origin.recovered(), ordinal)
    }

    fn empty_name_candidate(
        source_id: SourceId,
        module: ModuleId,
        ordinal: usize,
        start: usize,
    ) -> NameReferenceCandidate {
        name_candidate(source_id, module, ordinal, start, "")
    }

    fn reference_site(
        source_id: SourceId,
        module: ModuleId,
        start: usize,
        spelling: &str,
        ordinal: usize,
    ) -> (ReferenceSite, SemanticOrigin) {
        let range = range(source_id, start, start + spelling.len());
        let origin = SemanticOrigin::new(
            source_id,
            module,
            SourceAnchor::Range(range),
            vec![ordinal as u32],
        );
        let mut arena = ResolvedArenaBuilder::new();
        let node = arena
            .push(ResolvedNode::new(
                SurfaceNodeKind::Reference,
                Vec::new(),
                origin.clone(),
            ))
            .unwrap();
        (ReferenceSite::new(node, range, spelling), origin)
    }

    fn current_public_projection(
        source_id: SourceId,
        module: ModuleId,
        namespace: NamespacePath,
        local: &str,
        fqn: &str,
        declaration_start: usize,
        visible_after_ordinal: usize,
    ) -> NameSymbolProjection {
        NameSymbolProjection::current_module(
            symbol_id(module, local, fqn),
            namespace,
            primary_spelling(local),
            SymbolKind::Predicate,
            Visibility::Public,
            range(
                source_id,
                declaration_start,
                declaration_start + local.len(),
            ),
            visible_after_ordinal,
        )
    }

    fn current_private_projection(
        source_id: SourceId,
        module: ModuleId,
        namespace: NamespacePath,
        local: &str,
        fqn: &str,
        declaration_start: usize,
        visible_after_ordinal: usize,
    ) -> NameSymbolProjection {
        NameSymbolProjection::current_module(
            symbol_id(module, local, fqn),
            namespace,
            primary_spelling(local),
            SymbolKind::Predicate,
            Visibility::Private,
            range(
                source_id,
                declaration_start,
                declaration_start + local.len(),
            ),
            visible_after_ordinal,
        )
    }

    fn imported_projection(
        source_id: SourceId,
        module: ModuleId,
        namespace: NamespacePath,
        local: &str,
        fqn: &str,
        visibility: Visibility,
        declaration_start: usize,
    ) -> NameSymbolProjection {
        NameSymbolProjection::imported(
            symbol_id(module, local, fqn),
            namespace,
            primary_spelling(local),
            SymbolKind::Predicate,
            visibility,
            range(
                source_id,
                declaration_start,
                declaration_start + local.len(),
            ),
        )
    }

    fn primary_spelling(local: &str) -> String {
        local.split('/').next().unwrap_or(local).to_owned()
    }

    fn assert_resolved_symbol(
        resolution: &NameReferenceResolution,
        index: usize,
        expected_fqn: &str,
    ) {
        let entry = resolution.table().get(resolution.ids()[index]).unwrap();
        let NameResolution::Resolved(symbol) = entry.resolution() else {
            panic!("expected resolved symbol at index {index}");
        };
        assert_eq!(symbol.symbol().fqn().as_str(), expected_fqn);
    }

    fn assert_resolved_builtin(
        resolution: &NameReferenceResolution,
        index: usize,
        expected_builtin: &str,
    ) {
        let entry = resolution.table().get(resolution.ids()[index]).unwrap();
        let NameResolution::ResolvedBuiltin(builtin) = entry.resolution() else {
            panic!("expected resolved builtin at index {index}");
        };
        assert_eq!(builtin.builtin().as_str(), expected_builtin);
    }

    fn assert_unresolved(
        resolution: &NameReferenceResolution,
        index: usize,
        expected_lookup: NameLookupClass,
        expected_spelling: &str,
    ) {
        let entry = resolution.table().get(resolution.ids()[index]).unwrap();
        let NameResolution::Unresolved(unresolved) = entry.resolution() else {
            panic!("expected unresolved name at index {index}");
        };
        assert_eq!(unresolved.lookup(), expected_lookup);
        assert_eq!(unresolved.spelling(), expected_spelling);
    }

    fn candidate(
        source_id: SourceId,
        ordinal: usize,
        start: usize,
        spellings: &[&str],
    ) -> NamespacePathCandidate {
        let mut cursor = start;
        let mut segments = Vec::new();
        for spelling in spellings {
            segments.push(NamespacePathSegment::new(
                *spelling,
                range(source_id, cursor, cursor + spelling.len()),
            ));
            cursor += spelling.len() + 1;
        }
        NamespacePathCandidate::new(
            segments,
            range(source_id, start, cursor.saturating_sub(1)),
            ordinal,
        )
    }

    fn fixture_provider() -> WorkspaceStubModuleIndexProvider {
        WorkspaceStubModuleIndexProvider::new(
            vec![
                package("dep"),
                package("app"),
                package("stdpkg"),
                package("pkgdep"),
                package("devdep"),
                package("extdep"),
                package("altdep"),
            ],
            vec![
                namespace(NamespaceRoot::PackageName, &["dep"], "dep"),
                namespace(NamespaceRoot::PackageName, &["dep", "nested"], "altdep"),
                namespace(NamespaceRoot::PackageName, &["app"], "app"),
                namespace(NamespaceRoot::Std, &[], "stdpkg"),
                namespace(NamespaceRoot::Pub, &["math"], "dep"),
                namespace(NamespaceRoot::Pub, &["math", "logic"], "altdep"),
                namespace(NamespaceRoot::Pkg, &["vendor"], "pkgdep"),
                namespace(NamespaceRoot::Dev, &["sandbox"], "devdep"),
                namespace(NamespaceRoot::Ext, &["mirror"], "extdep"),
            ],
            vec![
                workspace_module("app", "main"),
                workspace_module("app", "util"),
                dependency_module("dep", "logic"),
                dependency_module("dep", "algebra.group"),
                dependency_module("stdpkg", "core"),
                dependency_module("pkgdep", "lib"),
                dependency_module("devdep", "tools"),
                dependency_module("extdep", "logic"),
                dependency_module("altdep", "logic"),
                dependency_module("altdep", "core"),
            ],
            vec![
                dependency_summary("dep", "logic", 3),
                dependency_summary("dep", "algebra.group", 4),
                dependency_summary("stdpkg", "core", 5),
                dependency_summary("pkgdep", "lib", 6),
                dependency_summary("devdep", "tools", 7),
                dependency_summary("extdep", "logic", 8),
                dependency_summary("altdep", "logic", 9),
                dependency_summary("altdep", "core", 10),
            ],
        )
    }

    fn package(package_id: &str) -> PackageIndexEntry {
        PackageIndexEntry {
            package_id: PackageId::new(package_id),
            version: Version::new(0, 1, 0),
            edition: Edition::new("2026"),
            source: PackageIndexSource::Workspace {
                package_root: format!("/workspace/{package_id}"),
                source_root: format!("/workspace/{package_id}/src"),
                manifest_path: format!("/workspace/{package_id}/mizar.toml"),
            },
            dependencies: Vec::new(),
        }
    }

    fn namespace(root: NamespaceRoot, prefix: &[&str], package_id: &str) -> NamespaceIndexEntry {
        NamespaceIndexEntry {
            root,
            prefix: prefix
                .iter()
                .map(|component| (*component).to_owned())
                .collect(),
            package_id: PackageId::new(package_id),
        }
    }

    fn workspace_module(package_id: &str, path: &str) -> ModuleIndexEntry {
        ModuleIndexEntry {
            package_id: PackageId::new(package_id),
            module_path: ModulePath::new(path),
            module: IndexedModuleId::new(PackageId::new(package_id), ModulePath::new(path)),
            location: ModuleIndexLocation::WorkspaceFile {
                source_root: format!("/workspace/{package_id}/src"),
                normalized_path: format!(
                    "/workspace/{package_id}/src/{}.miz",
                    path.replace('.', "/")
                ),
                source_relative_path: format!("{}.miz", path.replace('.', "/")),
            },
            edition: Edition::new("2026"),
        }
    }

    fn dependency_module(package_id: &str, path: &str) -> ModuleIndexEntry {
        ModuleIndexEntry {
            package_id: PackageId::new(package_id),
            module_path: ModulePath::new(path),
            module: IndexedModuleId::new(PackageId::new(package_id), ModulePath::new(path)),
            location: ModuleIndexLocation::DependencySummary {
                artifact: format!("{package_id}.{path}.summary"),
                content_hash: Hash::from_bytes([1; Hash::BYTE_LEN]),
            },
            edition: Edition::new("2026"),
        }
    }

    fn dependency_summary(package_id: &str, path: &str, byte: u8) -> DependencyModuleSummaryRef {
        DependencyModuleSummaryRef {
            module: IndexedModuleId::new(PackageId::new(package_id), ModulePath::new(path)),
            artifact: format!("{package_id}.{path}.summary"),
            content_hash: Hash::from_bytes([byte; Hash::BYTE_LEN]),
        }
    }

    fn module_id(package_id: &str, path: &str) -> ModuleId {
        ModuleId::new(PackageId::new(package_id), ModulePath::new(path))
    }

    fn symbol_id(module: ModuleId, local: &str, fqn: &str) -> SymbolId {
        SymbolId::new(
            module,
            LocalSymbolId::new(local),
            FullyQualifiedName::new(fqn),
        )
    }

    fn source_id() -> SourceId {
        let snapshot_id = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            "05".repeat(Hash::BYTE_LEN)
        ))
        .unwrap();
        InMemorySessionIdAllocator::new()
            .next_source_id(snapshot_id)
            .unwrap()
    }

    const fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
        SourceRange {
            source_id,
            start,
            end,
        }
    }
}
