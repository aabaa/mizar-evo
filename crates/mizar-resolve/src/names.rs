//! Namespace resolution before symbol lookup.
//!
//! This module implements the R-013 namespace slice of name resolution. It
//! resolves source-shaped namespace path candidates to canonical module
//! namespaces and keeps unresolved namespace failures explicit without looking
//! up final symbols, checking selectors, or assigning declaration symbols.

use crate::imports::{ImportPathFailureClass, ResolvedImportCandidate, UnresolvedImportCandidate};
use crate::module_index::{
    IndexedModuleId, ModuleIndexInput, ModuleIndexProviderError, NamespaceIndexEntry, NamespaceRoot,
};
use crate::resolved_ast::ModuleId;
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
    use mizar_build::module_index::{
        DependencyModuleSummaryRef, ModuleIndexEntry, ModuleIndexLocation, PackageIndexEntry,
        PackageIndexSource,
    };
    use mizar_session::{
        BuildSnapshotId, Edition, Hash, InMemorySessionIdAllocator, SessionIdAllocator, SourceId,
    };
    use semver::Version;

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
