//! Semantic import graph construction.
//!
//! This module resolves source-shaped import path candidates into canonical
//! module candidates, records path/alias recovery, and builds the deterministic
//! accepted acyclic graph used by later import, name, and symbol resolution
//! tasks. Direct `SurfaceAst` collection and export validation feed this layer
//! in follow-on tasks.

use crate::module_index::{
    IndexedModuleId, ModuleIndexInput, ModuleIndexProviderError, NamespaceIndexEntry, NamespaceRoot,
};
use crate::resolved_ast::ModuleId;
use mizar_session::{ModulePath, PackageId, SourceRange};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

/// Import path prefix before canonical module identity resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum ImportPathPrefix {
    /// Package-local or namespace-qualified path with no relative marker.
    Unprefixed,
    /// `.` path relative to the current module's containing module directory.
    Current,
    /// `..` path relative to the parent of the containing module directory.
    Parent,
}

/// Source-shaped import path candidate collected before semantic validation.
///
/// Branch imports are represented as one candidate per branch member. The
/// optional branch provenance fields let a later `SurfaceAst` walker preserve
/// both the shared base span and the member span without making this resolver
/// seam own parser syntax.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportPathCandidate {
    components: Vec<String>,
    prefix: ImportPathPrefix,
    alias: Option<String>,
    alias_range: Option<SourceRange>,
    range: SourceRange,
    ordinal: usize,
    branch_base_range: Option<SourceRange>,
    branch_member_range: Option<SourceRange>,
    recovered: bool,
}

impl ImportPathCandidate {
    /// Creates an import path candidate.
    #[must_use]
    pub fn new(
        components: Vec<String>,
        prefix: ImportPathPrefix,
        alias: Option<String>,
        range: SourceRange,
        ordinal: usize,
    ) -> Self {
        Self {
            components,
            prefix,
            alias,
            alias_range: None,
            range,
            ordinal,
            branch_base_range: None,
            branch_member_range: None,
            recovered: false,
        }
    }

    /// Attaches the exact alias span from the source directive.
    #[must_use]
    pub const fn with_alias_range(mut self, range: SourceRange) -> Self {
        self.alias_range = Some(range);
        self
    }

    /// Attaches branch-import base and member provenance.
    #[must_use]
    pub const fn with_branch_provenance(
        mut self,
        base_range: SourceRange,
        member_range: SourceRange,
    ) -> Self {
        self.branch_base_range = Some(base_range);
        self.branch_member_range = Some(member_range);
        self
    }

    /// Marks a parser-recovered candidate that should stay explicit.
    #[must_use]
    pub const fn with_recovered(mut self) -> Self {
        self.recovered = true;
        self
    }

    /// Returns normalized path components.
    #[must_use]
    pub fn components(&self) -> &[String] {
        &self.components
    }

    /// Returns the relative or unprefixed path marker.
    #[must_use]
    pub const fn prefix(&self) -> ImportPathPrefix {
        self.prefix
    }

    /// Returns the explicit alias spelling, if present.
    #[must_use]
    pub fn alias(&self) -> Option<&str> {
        self.alias.as_deref()
    }

    /// Returns the explicit alias source span, if present.
    #[must_use]
    pub const fn alias_range(&self) -> Option<SourceRange> {
        self.alias_range
    }

    /// Returns the candidate source span.
    #[must_use]
    pub const fn range(&self) -> SourceRange {
        self.range
    }

    /// Returns the source-order ordinal.
    #[must_use]
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }

    /// Returns the shared branch base span, if this came from a branch import.
    #[must_use]
    pub const fn branch_base_range(&self) -> Option<SourceRange> {
        self.branch_base_range
    }

    /// Returns the branch member span, if this came from a branch import.
    #[must_use]
    pub const fn branch_member_range(&self) -> Option<SourceRange> {
        self.branch_member_range
    }

    /// Returns whether parser recovery was involved.
    #[must_use]
    pub const fn recovered(&self) -> bool {
        self.recovered
    }

    fn spelling(&self) -> String {
        path_spelling(self.prefix, &self.components)
    }

    fn effective_alias(&self) -> Option<String> {
        self.alias
            .clone()
            .or_else(|| self.components.last().cloned())
    }
}

/// Package, namespace, or module candidate found before a path-resolution
/// failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportPathPartialCandidate {
    namespace_root: Option<NamespaceRoot>,
    matched_prefix: Vec<String>,
    package: Option<PackageId>,
    remaining_components: Vec<String>,
}

impl ImportPathPartialCandidate {
    fn new(
        namespace_root: Option<NamespaceRoot>,
        matched_prefix: Vec<String>,
        package: Option<PackageId>,
        remaining_components: Vec<String>,
    ) -> Self {
        Self {
            namespace_root,
            matched_prefix,
            package,
            remaining_components,
        }
    }

    /// Returns the namespace root that matched, if any.
    #[must_use]
    pub const fn namespace_root(&self) -> Option<NamespaceRoot> {
        self.namespace_root
    }

    /// Returns the matched namespace/package prefix.
    #[must_use]
    pub fn matched_prefix(&self) -> &[String] {
        &self.matched_prefix
    }

    /// Returns the matched package candidate, if one was found.
    #[must_use]
    pub const fn package(&self) -> Option<&PackageId> {
        self.package.as_ref()
    }

    /// Returns module-path components remaining after namespace/package
    /// selection.
    #[must_use]
    pub fn remaining_components(&self) -> &[String] {
        &self.remaining_components
    }
}

/// Resolved source-shaped import before graph-edge deduplication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedImportCandidate {
    target: ModuleId,
    alias: String,
    explicit_alias: Option<String>,
    range: SourceRange,
    alias_range: Option<SourceRange>,
    ordinal: usize,
    spelling: String,
    components: Vec<String>,
    prefix: ImportPathPrefix,
    binding: Option<ImportPathPartialCandidate>,
    branch_base_range: Option<SourceRange>,
    branch_member_range: Option<SourceRange>,
    recovered: bool,
}

impl ResolvedImportCandidate {
    fn new(
        candidate: &ImportPathCandidate,
        target: ModuleId,
        alias: String,
        binding: Option<ImportPathPartialCandidate>,
    ) -> Self {
        Self {
            target,
            alias,
            explicit_alias: candidate.alias.clone(),
            range: candidate.range,
            alias_range: candidate.alias_range,
            ordinal: candidate.ordinal,
            spelling: candidate.spelling(),
            components: candidate.components.clone(),
            prefix: candidate.prefix,
            binding,
            branch_base_range: candidate.branch_base_range,
            branch_member_range: candidate.branch_member_range,
            recovered: candidate.recovered,
        }
    }

    /// Returns the canonical, alias-independent target module.
    #[must_use]
    pub const fn target(&self) -> &ModuleId {
        &self.target
    }

    /// Returns the local import alias spelling.
    #[must_use]
    pub fn alias(&self) -> &str {
        &self.alias
    }

    /// Returns the explicit alias spelling, if the source used `as`.
    #[must_use]
    pub fn explicit_alias(&self) -> Option<&str> {
        self.explicit_alias.as_deref()
    }

    /// Returns the source span that introduced the import.
    #[must_use]
    pub const fn range(&self) -> SourceRange {
        self.range
    }

    /// Returns the explicit alias source span, if present.
    #[must_use]
    pub const fn alias_range(&self) -> Option<SourceRange> {
        self.alias_range
    }

    /// Returns the source-order ordinal.
    #[must_use]
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }

    /// Returns the source path spelling.
    #[must_use]
    pub fn spelling(&self) -> &str {
        &self.spelling
    }

    /// Returns normalized path components.
    #[must_use]
    pub fn components(&self) -> &[String] {
        &self.components
    }

    /// Returns the relative or unprefixed path marker.
    #[must_use]
    pub const fn prefix(&self) -> ImportPathPrefix {
        self.prefix
    }

    /// Returns namespace/package binding provenance, if cross-package
    /// resolution was used.
    #[must_use]
    pub const fn binding(&self) -> Option<&ImportPathPartialCandidate> {
        self.binding.as_ref()
    }

    /// Returns the shared branch base span, if this came from a branch import.
    #[must_use]
    pub const fn branch_base_range(&self) -> Option<SourceRange> {
        self.branch_base_range
    }

    /// Returns the branch member span, if this came from a branch import.
    #[must_use]
    pub const fn branch_member_range(&self) -> Option<SourceRange> {
        self.branch_member_range
    }

    /// Returns whether parser recovery was involved.
    #[must_use]
    pub const fn recovered(&self) -> bool {
        self.recovered
    }

    /// Converts the resolved source candidate into a canonical graph edge
    /// candidate.
    #[must_use]
    pub fn graph_candidate(&self) -> ImportEdgeCandidate {
        ImportEdgeCandidate::new(self.target.clone(), self.range, self.ordinal)
    }
}

/// Unresolved source-shaped import retained for recovery and later
/// diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnresolvedImportCandidate {
    spelling: String,
    components: Vec<String>,
    prefix: ImportPathPrefix,
    alias: Option<String>,
    alias_range: Option<SourceRange>,
    range: SourceRange,
    ordinal: usize,
    class: ImportPathFailureClass,
    partial: Option<ImportPathPartialCandidate>,
    candidate_target: Option<ModuleId>,
    branch_base_range: Option<SourceRange>,
    branch_member_range: Option<SourceRange>,
    recovered: bool,
}

impl UnresolvedImportCandidate {
    fn from_candidate(
        candidate: &ImportPathCandidate,
        class: ImportPathFailureClass,
        partial: Option<ImportPathPartialCandidate>,
        candidate_target: Option<ModuleId>,
    ) -> Self {
        Self {
            spelling: candidate.spelling(),
            components: candidate.components.clone(),
            prefix: candidate.prefix,
            alias: candidate.alias.clone(),
            alias_range: candidate.alias_range,
            range: candidate.range,
            ordinal: candidate.ordinal,
            class,
            partial,
            candidate_target,
            branch_base_range: candidate.branch_base_range,
            branch_member_range: candidate.branch_member_range,
            recovered: candidate.recovered,
        }
    }

    fn from_resolved(resolved: ResolvedImportCandidate, class: ImportPathFailureClass) -> Self {
        Self {
            spelling: resolved.spelling,
            components: resolved.components,
            prefix: resolved.prefix,
            alias: resolved.explicit_alias,
            alias_range: resolved.alias_range,
            range: resolved.range,
            ordinal: resolved.ordinal,
            class,
            partial: resolved.binding,
            candidate_target: Some(resolved.target),
            branch_base_range: resolved.branch_base_range,
            branch_member_range: resolved.branch_member_range,
            recovered: resolved.recovered,
        }
    }

    /// Returns the unresolved source path spelling.
    #[must_use]
    pub fn spelling(&self) -> &str {
        &self.spelling
    }

    /// Returns normalized path components when parseable.
    #[must_use]
    pub fn components(&self) -> &[String] {
        &self.components
    }

    /// Returns the relative or unprefixed path marker.
    #[must_use]
    pub const fn prefix(&self) -> ImportPathPrefix {
        self.prefix
    }

    /// Returns the explicit alias spelling, if present.
    #[must_use]
    pub fn alias(&self) -> Option<&str> {
        self.alias.as_deref()
    }

    /// Returns the explicit alias source span, if present.
    #[must_use]
    pub const fn alias_range(&self) -> Option<SourceRange> {
        self.alias_range
    }

    /// Returns the candidate source span.
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
    pub const fn class(&self) -> ImportPathFailureClass {
        self.class
    }

    /// Returns partial namespace/package/module provenance, if any.
    #[must_use]
    pub const fn partial(&self) -> Option<&ImportPathPartialCandidate> {
        self.partial.as_ref()
    }

    /// Returns a canonical module candidate found before a later failure, such
    /// as duplicate alias rejection.
    #[must_use]
    pub const fn candidate_target(&self) -> Option<&ModuleId> {
        self.candidate_target.as_ref()
    }

    /// Returns the shared branch base span, if this came from a branch import.
    #[must_use]
    pub const fn branch_base_range(&self) -> Option<SourceRange> {
        self.branch_base_range
    }

    /// Returns the branch member span, if this came from a branch import.
    #[must_use]
    pub const fn branch_member_range(&self) -> Option<SourceRange> {
        self.branch_member_range
    }

    /// Returns whether parser recovery was involved.
    #[must_use]
    pub const fn recovered(&self) -> bool {
        self.recovered
    }
}

/// Crate-local import path failure class. These are not public diagnostic
/// codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum ImportPathFailureClass {
    /// The candidate had no usable path components.
    EmptyPath,
    /// The namespace root or package-name binding was unknown.
    UnknownNamespaceOrPackage,
    /// The package was known, but the module path was absent.
    UnknownModule,
    /// A `..` prefix would escape the current package root.
    RelativePathEscapesPackage,
    /// The parser recovered a malformed directive.
    RecoveredSyntax,
    /// The alias was bound to multiple canonical modules.
    DuplicateAlias,
    /// The alias collided with a reserved namespace root.
    AliasRootConflict,
    /// The candidate shape was not semantically usable.
    IllegalCandidateState,
}

/// Source-shaped import path resolution result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportPathResolution {
    resolved: Vec<ResolvedImportCandidate>,
    unresolved: Vec<UnresolvedImportCandidate>,
}

impl ImportPathResolution {
    fn new(
        mut resolved: Vec<ResolvedImportCandidate>,
        mut unresolved: Vec<UnresolvedImportCandidate>,
    ) -> Self {
        resolved.sort_by(resolved_import_candidate_cmp);
        unresolved.sort_by(unresolved_import_candidate_cmp);
        Self {
            resolved,
            unresolved,
        }
    }

    /// Returns resolved imports in deterministic source order.
    #[must_use]
    pub fn resolved(&self) -> &[ResolvedImportCandidate] {
        &self.resolved
    }

    /// Returns unresolved imports in deterministic source order.
    #[must_use]
    pub fn unresolved(&self) -> &[UnresolvedImportCandidate] {
        &self.unresolved
    }

    /// Returns whether any candidate failed semantic path resolution.
    #[must_use]
    pub const fn has_unresolved(&self) -> bool {
        !self.unresolved.is_empty()
    }

    /// Produces canonical graph candidates for this module from the resolved
    /// import subset.
    #[must_use]
    pub fn module_candidates(&self, module: ModuleId) -> ModuleImportCandidates {
        ModuleImportCandidates::new(
            module,
            self.resolved
                .iter()
                .map(ResolvedImportCandidate::graph_candidate)
                .collect(),
        )
    }
}

/// Resolves source-shaped import paths through the resolver module-index seam.
#[derive(Clone, Copy)]
pub struct ImportPathResolver<'a> {
    module_index: ModuleIndexInput<'a>,
}

impl<'a> ImportPathResolver<'a> {
    /// Creates a path resolver backed by the resolver module-index input.
    #[must_use]
    pub const fn new(module_index: ModuleIndexInput<'a>) -> Self {
        Self { module_index }
    }

    /// Resolves import path candidates for the current module.
    #[must_use]
    pub fn resolve(
        self,
        current_module: &ModuleId,
        candidates: &[ImportPathCandidate],
    ) -> ImportPathResolution {
        let mut ordered = candidates.iter().collect::<Vec<_>>();
        ordered.sort_by(|left, right| import_path_candidate_cmp(left, right));

        let mut resolved = Vec::new();
        let mut unresolved = Vec::new();

        for candidate in ordered {
            match self.resolve_one(current_module, candidate) {
                PathCandidateResolution::Resolved(candidate) => resolved.push(candidate),
                PathCandidateResolution::Unresolved(candidate) => unresolved.push(candidate),
            }
        }

        let conflict_aliases = conflicting_aliases(&resolved);
        if !conflict_aliases.is_empty() {
            let mut kept = Vec::with_capacity(resolved.len());
            for candidate in resolved {
                if conflict_aliases.contains(candidate.alias()) {
                    unresolved.push(UnresolvedImportCandidate::from_resolved(
                        candidate,
                        ImportPathFailureClass::DuplicateAlias,
                    ));
                } else {
                    kept.push(candidate);
                }
            }
            resolved = kept;
        }

        ImportPathResolution::new(resolved, unresolved)
    }

    fn resolve_one(
        self,
        current_module: &ModuleId,
        candidate: &ImportPathCandidate,
    ) -> PathCandidateResolution {
        if candidate.recovered() {
            return self.unresolved(
                candidate,
                ImportPathFailureClass::RecoveredSyntax,
                None,
                None,
            );
        }

        if candidate.components().is_empty() {
            return self.unresolved(candidate, ImportPathFailureClass::EmptyPath, None, None);
        }

        let Some(alias) = candidate.effective_alias() else {
            return self.unresolved(candidate, ImportPathFailureClass::EmptyPath, None, None);
        };
        if alias.is_empty() {
            return self.unresolved(
                candidate,
                ImportPathFailureClass::IllegalCandidateState,
                None,
                None,
            );
        }
        if reserved_namespace_root(&alias).is_some() {
            return self.unresolved(
                candidate,
                ImportPathFailureClass::AliasRootConflict,
                None,
                None,
            );
        }

        match candidate.prefix() {
            ImportPathPrefix::Current => {
                self.resolve_current_relative(current_module, candidate, alias)
            }
            ImportPathPrefix::Parent => {
                self.resolve_parent_relative(current_module, candidate, alias)
            }
            ImportPathPrefix::Unprefixed => {
                self.resolve_unprefixed(current_module, candidate, alias)
            }
        }
    }

    fn resolve_current_relative(
        self,
        current_module: &ModuleId,
        candidate: &ImportPathCandidate,
        alias: String,
    ) -> PathCandidateResolution {
        let mut path = module_dir_components(current_module);
        path.extend(candidate.components().iter().cloned());
        let partial = package_local_partial(current_module.package(), &path);
        self.resolve_package_module(
            candidate,
            current_module.package().clone(),
            path,
            alias,
            Some(partial),
        )
    }

    fn resolve_parent_relative(
        self,
        current_module: &ModuleId,
        candidate: &ImportPathCandidate,
        alias: String,
    ) -> PathCandidateResolution {
        let mut path = module_dir_components(current_module);
        if path.pop().is_none() {
            return self.unresolved(
                candidate,
                ImportPathFailureClass::RelativePathEscapesPackage,
                None,
                None,
            );
        }
        path.extend(candidate.components().iter().cloned());
        let partial = package_local_partial(current_module.package(), &path);
        self.resolve_package_module(
            candidate,
            current_module.package().clone(),
            path,
            alias,
            Some(partial),
        )
    }

    fn resolve_unprefixed(
        self,
        current_module: &ModuleId,
        candidate: &ImportPathCandidate,
        alias: String,
    ) -> PathCandidateResolution {
        let components = candidate.components();
        if let Some(root) = reserved_namespace_root(&components[0]) {
            return self.resolve_reserved_root(candidate, root, &components[1..], alias);
        }

        if let Some(binding) =
            longest_namespace_binding(self.module_index.namespace_bindings(), components)
        {
            return self.resolve_binding(candidate, binding, components, alias);
        }

        self.resolve_package_module(
            candidate,
            current_module.package().clone(),
            components.to_vec(),
            alias,
            Some(ImportPathPartialCandidate::new(
                None,
                Vec::new(),
                Some(current_module.package().clone()),
                components.to_vec(),
            )),
        )
    }

    fn resolve_reserved_root(
        self,
        candidate: &ImportPathCandidate,
        root: NamespaceRoot,
        components_after_root: &[String],
        alias: String,
    ) -> PathCandidateResolution {
        let Some(binding) = longest_root_binding(
            self.module_index.namespace_bindings(),
            root,
            components_after_root,
        ) else {
            return self.unresolved(
                candidate,
                ImportPathFailureClass::UnknownNamespaceOrPackage,
                Some(ImportPathPartialCandidate::new(
                    Some(root),
                    Vec::new(),
                    None,
                    components_after_root.to_vec(),
                )),
                None,
            );
        };

        self.resolve_binding(candidate, binding, components_after_root, alias)
    }

    fn resolve_binding(
        self,
        candidate: &ImportPathCandidate,
        binding: &NamespaceIndexEntry,
        components: &[String],
        alias: String,
    ) -> PathCandidateResolution {
        let remaining_components = components[binding.prefix.len()..].to_vec();
        let partial = ImportPathPartialCandidate::new(
            Some(binding.root),
            binding.prefix.clone(),
            Some(binding.package_id.clone()),
            remaining_components.clone(),
        );

        match self.module_index.package(&binding.package_id) {
            Ok(_) => self.resolve_package_module(
                candidate,
                binding.package_id.clone(),
                remaining_components,
                alias,
                Some(partial),
            ),
            Err(_) => self.unresolved(
                candidate,
                ImportPathFailureClass::UnknownNamespaceOrPackage,
                Some(partial),
                None,
            ),
        }
    }

    fn resolve_package_module(
        self,
        candidate: &ImportPathCandidate,
        package: PackageId,
        path_components: Vec<String>,
        alias: String,
        partial: Option<ImportPathPartialCandidate>,
    ) -> PathCandidateResolution {
        let Some(module_path) = module_path_from_components(&path_components) else {
            return self.unresolved(
                candidate,
                ImportPathFailureClass::UnknownModule,
                partial,
                None,
            );
        };
        let module = IndexedModuleId::new(package, module_path);
        match self.module_index.module(&module) {
            Ok(entry) => PathCandidateResolution::Resolved(ResolvedImportCandidate::new(
                candidate,
                self.module_index.module_identity(entry),
                alias,
                partial,
            )),
            Err(_) => self.unresolved(
                candidate,
                ImportPathFailureClass::UnknownModule,
                partial,
                None,
            ),
        }
    }

    fn unresolved(
        self,
        candidate: &ImportPathCandidate,
        class: ImportPathFailureClass,
        partial: Option<ImportPathPartialCandidate>,
        candidate_target: Option<ModuleId>,
    ) -> PathCandidateResolution {
        PathCandidateResolution::Unresolved(UnresolvedImportCandidate::from_candidate(
            candidate,
            class,
            partial,
            candidate_target,
        ))
    }
}

enum PathCandidateResolution {
    Resolved(ResolvedImportCandidate),
    Unresolved(UnresolvedImportCandidate),
}

fn path_spelling(prefix: ImportPathPrefix, components: &[String]) -> String {
    let path = components.join(".");
    match prefix {
        ImportPathPrefix::Unprefixed => path,
        ImportPathPrefix::Current => {
            if path.is_empty() {
                ".".to_owned()
            } else {
                format!(".{path}")
            }
        }
        ImportPathPrefix::Parent => {
            if path.is_empty() {
                "..".to_owned()
            } else {
                format!("..{path}")
            }
        }
    }
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

fn module_dir_components(module: &ModuleId) -> Vec<String> {
    let mut components = split_module_path(module.path());
    components.pop();
    components
}

fn split_module_path(path: &ModulePath) -> Vec<String> {
    path.as_str()
        .split('.')
        .filter(|component| !component.is_empty())
        .map(str::to_owned)
        .collect()
}

fn module_path_from_components(components: &[String]) -> Option<ModulePath> {
    if components.is_empty() || components.iter().any(String::is_empty) {
        None
    } else {
        Some(ModulePath::new(components.join(".")))
    }
}

fn package_local_partial(
    package: &PackageId,
    path_components: &[String],
) -> ImportPathPartialCandidate {
    ImportPathPartialCandidate::new(
        None,
        Vec::new(),
        Some(package.clone()),
        path_components.to_vec(),
    )
}

fn conflicting_aliases(candidates: &[ResolvedImportCandidate]) -> BTreeSet<String> {
    let mut targets_by_alias = BTreeMap::<String, BTreeSet<ModuleId>>::new();
    for candidate in candidates {
        targets_by_alias
            .entry(candidate.alias().to_owned())
            .or_default()
            .insert(candidate.target().clone());
    }
    targets_by_alias
        .into_iter()
        .filter_map(|(alias, targets)| (targets.len() > 1).then_some(alias))
        .collect()
}

fn import_path_candidate_cmp(left: &ImportPathCandidate, right: &ImportPathCandidate) -> Ordering {
    left.ordinal()
        .cmp(&right.ordinal())
        .then_with(|| range_key(left.range()).cmp(&range_key(right.range())))
        .then_with(|| left.prefix().cmp(&right.prefix()))
        .then_with(|| left.components().cmp(right.components()))
        .then_with(|| left.alias().cmp(&right.alias()))
}

fn resolved_import_candidate_cmp(
    left: &ResolvedImportCandidate,
    right: &ResolvedImportCandidate,
) -> Ordering {
    left.ordinal()
        .cmp(&right.ordinal())
        .then_with(|| range_key(left.range()).cmp(&range_key(right.range())))
        .then_with(|| left.spelling().cmp(right.spelling()))
        .then_with(|| left.alias().cmp(right.alias()))
        .then_with(|| left.target().cmp(right.target()))
}

fn unresolved_import_candidate_cmp(
    left: &UnresolvedImportCandidate,
    right: &UnresolvedImportCandidate,
) -> Ordering {
    left.ordinal()
        .cmp(&right.ordinal())
        .then_with(|| range_key(left.range()).cmp(&range_key(right.range())))
        .then_with(|| left.class().cmp(&right.class()))
        .then_with(|| left.spelling().cmp(right.spelling()))
}

/// Canonical import candidates for one source module.
///
/// Callers must pass an explicit empty candidate set for a zero-import module
/// that should participate in graph ordering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleImportCandidates {
    module: ModuleId,
    imports: Vec<ImportEdgeCandidate>,
}

impl ModuleImportCandidates {
    /// Creates a candidate set for one module.
    #[must_use]
    pub fn new(module: ModuleId, imports: Vec<ImportEdgeCandidate>) -> Self {
        Self { module, imports }
    }

    /// Returns the source module.
    #[must_use]
    pub const fn module(&self) -> &ModuleId {
        &self.module
    }

    /// Returns canonical import-edge candidates in source collection order.
    #[must_use]
    pub fn imports(&self) -> &[ImportEdgeCandidate] {
        &self.imports
    }
}

/// A canonical import edge candidate before graph deduplication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportEdgeCandidate {
    target: ModuleId,
    range: SourceRange,
    ordinal: usize,
}

impl ImportEdgeCandidate {
    /// Creates a canonical import edge candidate.
    #[must_use]
    pub const fn new(target: ModuleId, range: SourceRange, ordinal: usize) -> Self {
        Self {
            target,
            range,
            ordinal,
        }
    }

    /// Returns the target module.
    #[must_use]
    pub const fn target(&self) -> &ModuleId {
        &self.target
    }

    /// Returns the source range that introduced this candidate.
    #[must_use]
    pub const fn range(&self) -> SourceRange {
        self.range
    }

    /// Returns the source-order ordinal for this candidate.
    #[must_use]
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
}

/// Builds import graphs against the resolver module-index input.
#[derive(Clone, Copy)]
pub struct ImportGraphBuilder<'a> {
    module_index: ModuleIndexInput<'a>,
}

impl<'a> ImportGraphBuilder<'a> {
    /// Creates a graph builder backed by the resolver module-index input.
    #[must_use]
    pub const fn new(module_index: ModuleIndexInput<'a>) -> Self {
        Self { module_index }
    }

    /// Builds a deterministic import graph and rejects cyclic components.
    ///
    /// Inputs must already contain canonical module identities. Unknown source
    /// or target modules are invalid builder inputs and are reported as build
    /// errors; semantic unresolved-import recovery is a higher-level concern for
    /// the alias/path recovery task.
    pub fn build(
        self,
        modules: &[ModuleImportCandidates],
    ) -> Result<ImportGraphResolution, ImportGraphBuildError> {
        let mut nodes = BTreeSet::<ModuleId>::new();
        let mut canonical_edges = BTreeMap::<(ModuleId, ModuleId), ImportGraphEdge>::new();

        for module in modules {
            self.ensure_source_module(module.module())?;
            nodes.insert(module.module().clone());
            for candidate in module.imports() {
                self.ensure_target_module(module.module(), candidate)?;
                nodes.insert(candidate.target().clone());
                let edge = ImportGraphEdge::new(
                    module.module().clone(),
                    candidate.target().clone(),
                    candidate.range(),
                    candidate.ordinal(),
                );
                canonical_edges
                    .entry((edge.source().clone(), edge.target().clone()))
                    .and_modify(|existing| {
                        if edge_provenance_cmp(&edge, existing).is_lt() {
                            *existing = edge.clone();
                        }
                    })
                    .or_insert(edge);
            }
        }

        let edges = canonical_edges.into_values().collect::<Vec<_>>();
        let cycles = detect_cycles(&nodes, &edges);
        let cyclic_nodes = cycles
            .iter()
            .flat_map(|cycle| cycle.modules().iter().cloned())
            .collect::<BTreeSet<_>>();
        let accepted_nodes = nodes
            .into_iter()
            .filter(|node| !cyclic_nodes.contains(node))
            .collect::<BTreeSet<_>>();
        let accepted_edges = edges
            .iter()
            .filter(|edge| {
                !cyclic_nodes.contains(edge.source()) && !cyclic_nodes.contains(edge.target())
            })
            .cloned()
            .collect::<Vec<_>>();
        let topological_order =
            dependency_first_topological_order(&accepted_nodes, &accepted_edges);
        let graph = ImportGraph::new(
            accepted_nodes.into_iter().collect(),
            accepted_edges,
            topological_order,
        );

        Ok(ImportGraphResolution::new(graph, cycles))
    }

    fn ensure_source_module(&self, module: &ModuleId) -> Result<(), ImportGraphBuildError> {
        self.module_index
            .module(&indexed_module_id(module))
            .map(|_| ())
            .map_err(|lookup| ImportGraphBuildError::UnknownSourceModule {
                module: Box::new(module.clone()),
                lookup: Box::new(lookup),
            })
    }

    fn ensure_target_module(
        &self,
        source: &ModuleId,
        candidate: &ImportEdgeCandidate,
    ) -> Result<(), ImportGraphBuildError> {
        self.module_index
            .module(&indexed_module_id(candidate.target()))
            .map(|_| ())
            .map_err(|lookup| ImportGraphBuildError::UnknownTargetModule {
                source: Box::new(source.clone()),
                target: Box::new(candidate.target().clone()),
                range: candidate.range(),
                ordinal: candidate.ordinal(),
                lookup: Box::new(lookup),
            })
    }
}

/// Completed import graph construction result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportGraphResolution {
    graph: ImportGraph,
    cycles: Vec<ImportCycle>,
}

impl ImportGraphResolution {
    fn new(graph: ImportGraph, mut cycles: Vec<ImportCycle>) -> Self {
        cycles.sort_by(import_cycle_cmp);
        Self { graph, cycles }
    }

    /// Returns the accepted acyclic graph portion.
    #[must_use]
    pub const fn graph(&self) -> &ImportGraph {
        &self.graph
    }

    /// Returns rejected cycles in deterministic order.
    #[must_use]
    pub fn cycles(&self) -> &[ImportCycle] {
        &self.cycles
    }

    /// Returns whether any cycles were rejected.
    #[must_use]
    pub const fn has_cycles(&self) -> bool {
        !self.cycles.is_empty()
    }
}

/// Accepted acyclic import graph portion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportGraph {
    nodes: Vec<ModuleId>,
    edges: Vec<ImportGraphEdge>,
    topological_order: Vec<ModuleId>,
}

impl ImportGraph {
    fn new(
        mut nodes: Vec<ModuleId>,
        mut edges: Vec<ImportGraphEdge>,
        topological_order: Vec<ModuleId>,
    ) -> Self {
        nodes.sort();
        edges.sort_by(import_edge_cmp);
        Self {
            nodes,
            edges,
            topological_order,
        }
    }

    /// Returns accepted graph nodes in canonical order.
    #[must_use]
    pub fn nodes(&self) -> &[ModuleId] {
        &self.nodes
    }

    /// Returns accepted graph edges in deterministic canonical order.
    #[must_use]
    pub fn edges(&self) -> &[ImportGraphEdge] {
        &self.edges
    }

    /// Returns dependency-first topological order for accepted graph nodes.
    #[must_use]
    pub fn topological_order(&self) -> &[ModuleId] {
        &self.topological_order
    }
}

/// A canonical import graph edge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportGraphEdge {
    source: ModuleId,
    target: ModuleId,
    range: SourceRange,
    ordinal: usize,
}

impl ImportGraphEdge {
    fn new(source: ModuleId, target: ModuleId, range: SourceRange, ordinal: usize) -> Self {
        Self {
            source,
            target,
            range,
            ordinal,
        }
    }

    /// Returns the importing module.
    #[must_use]
    pub const fn source(&self) -> &ModuleId {
        &self.source
    }

    /// Returns the imported module.
    #[must_use]
    pub const fn target(&self) -> &ModuleId {
        &self.target
    }

    /// Returns the source range for the retained edge provenance.
    #[must_use]
    pub const fn range(&self) -> SourceRange {
        self.range
    }

    /// Returns the source-order ordinal for the retained edge provenance.
    #[must_use]
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
}

/// A rejected import cycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportCycle {
    modules: Vec<ModuleId>,
    edges: Vec<ImportGraphEdge>,
}

impl ImportCycle {
    fn new(mut modules: Vec<ModuleId>, mut edges: Vec<ImportGraphEdge>) -> Self {
        modules.sort();
        edges.sort_by(cycle_edge_cmp);
        Self { modules, edges }
    }

    /// Returns cyclic modules in canonical order.
    #[must_use]
    pub fn modules(&self) -> &[ModuleId] {
        &self.modules
    }

    /// Returns internal cycle edges in deterministic order.
    #[must_use]
    pub fn edges(&self) -> &[ImportGraphEdge] {
        &self.edges
    }
}

/// Build error for canonical import graph construction.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ImportGraphBuildError {
    /// Source module was not present in the module index.
    UnknownSourceModule {
        /// Missing source module.
        module: Box<ModuleId>,
        /// Underlying module-index lookup error.
        lookup: Box<ModuleIndexProviderError>,
    },
    /// Target module was not present in the module index.
    UnknownTargetModule {
        /// Source module that requested the import.
        source: Box<ModuleId>,
        /// Missing target module.
        target: Box<ModuleId>,
        /// Source range for the import candidate.
        range: SourceRange,
        /// Source-order ordinal for the import candidate.
        ordinal: usize,
        /// Underlying module-index lookup error.
        lookup: Box<ModuleIndexProviderError>,
    },
}

fn indexed_module_id(module: &ModuleId) -> IndexedModuleId {
    IndexedModuleId::new(module.package().clone(), module.path().clone())
}

fn detect_cycles(nodes: &BTreeSet<ModuleId>, edges: &[ImportGraphEdge]) -> Vec<ImportCycle> {
    let adjacency = adjacency(nodes, edges);
    let self_edges = edges
        .iter()
        .filter(|edge| edge.source() == edge.target())
        .map(|edge| edge.source().clone())
        .collect::<BTreeSet<_>>();
    let components = strongly_connected_components(nodes, &adjacency);
    components
        .into_iter()
        .filter(|component| component.len() > 1 || self_edges.contains(&component[0]))
        .map(|component| {
            let component_nodes = component.iter().cloned().collect::<BTreeSet<_>>();
            let cycle_edges = edges
                .iter()
                .filter(|edge| {
                    component_nodes.contains(edge.source())
                        && component_nodes.contains(edge.target())
                })
                .cloned()
                .collect();
            ImportCycle::new(component, cycle_edges)
        })
        .collect()
}

fn adjacency(
    nodes: &BTreeSet<ModuleId>,
    edges: &[ImportGraphEdge],
) -> BTreeMap<ModuleId, Vec<ModuleId>> {
    let mut adjacency = nodes
        .iter()
        .map(|node| (node.clone(), BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();
    for edge in edges {
        adjacency
            .entry(edge.source().clone())
            .or_default()
            .insert(edge.target().clone());
    }
    adjacency
        .into_iter()
        .map(|(node, targets)| (node, targets.into_iter().collect()))
        .collect()
}

fn strongly_connected_components(
    nodes: &BTreeSet<ModuleId>,
    adjacency: &BTreeMap<ModuleId, Vec<ModuleId>>,
) -> Vec<Vec<ModuleId>> {
    let mut state = TarjanState::default();
    for node in nodes {
        if !state.indices.contains_key(node) {
            strong_connect(node.clone(), adjacency, &mut state);
        }
    }
    state.components
}

#[derive(Default)]
struct TarjanState {
    next_index: usize,
    indices: BTreeMap<ModuleId, usize>,
    lowlinks: BTreeMap<ModuleId, usize>,
    stack: Vec<ModuleId>,
    on_stack: BTreeSet<ModuleId>,
    components: Vec<Vec<ModuleId>>,
}

fn strong_connect(
    node: ModuleId,
    adjacency: &BTreeMap<ModuleId, Vec<ModuleId>>,
    state: &mut TarjanState,
) {
    let index = state.next_index;
    state.next_index += 1;
    state.indices.insert(node.clone(), index);
    state.lowlinks.insert(node.clone(), index);
    state.stack.push(node.clone());
    state.on_stack.insert(node.clone());

    if let Some(targets) = adjacency.get(&node) {
        for target in targets {
            if !state.indices.contains_key(target) {
                strong_connect(target.clone(), adjacency, state);
                let target_lowlink = state.lowlinks[target];
                let node_lowlink = state
                    .lowlinks
                    .get_mut(&node)
                    .expect("node lowlink must exist after recursive visit");
                *node_lowlink = (*node_lowlink).min(target_lowlink);
            } else if state.on_stack.contains(target) {
                let target_index = state.indices[target];
                let node_lowlink = state
                    .lowlinks
                    .get_mut(&node)
                    .expect("node lowlink must exist for stack target");
                *node_lowlink = (*node_lowlink).min(target_index);
            }
        }
    }

    if state.lowlinks[&node] == state.indices[&node] {
        let mut component = Vec::new();
        loop {
            let stacked = state
                .stack
                .pop()
                .expect("root node must be present on Tarjan stack");
            state.on_stack.remove(&stacked);
            let is_root = stacked == node;
            component.push(stacked);
            if is_root {
                break;
            }
        }
        component.sort();
        state.components.push(component);
    }
}

fn dependency_first_topological_order(
    nodes: &BTreeSet<ModuleId>,
    edges: &[ImportGraphEdge],
) -> Vec<ModuleId> {
    let mut dependency_counts = nodes
        .iter()
        .map(|node| (node.clone(), 0usize))
        .collect::<BTreeMap<_, _>>();
    let mut dependents_by_dependency = BTreeMap::<ModuleId, BTreeSet<ModuleId>>::new();

    for edge in edges {
        if let Some(count) = dependency_counts.get_mut(edge.source()) {
            *count += 1;
        }
        dependents_by_dependency
            .entry(edge.target().clone())
            .or_default()
            .insert(edge.source().clone());
    }

    let mut ready = dependency_counts
        .iter()
        .filter_map(|(node, count)| (*count == 0).then_some(node.clone()))
        .collect::<BTreeSet<_>>();
    let mut order = Vec::with_capacity(nodes.len());

    while let Some(node) = ready.pop_first() {
        order.push(node.clone());
        if let Some(dependents) = dependents_by_dependency.get(&node) {
            for dependent in dependents {
                let count = dependency_counts
                    .get_mut(dependent)
                    .expect("dependent must be present in accepted node set");
                *count -= 1;
                if *count == 0 {
                    ready.insert(dependent.clone());
                }
            }
        }
    }

    order
}

fn import_edge_cmp(left: &ImportGraphEdge, right: &ImportGraphEdge) -> Ordering {
    left.source()
        .cmp(right.source())
        .then_with(|| left.target().cmp(right.target()))
        .then_with(|| edge_provenance_cmp(left, right))
}

fn edge_provenance_cmp(left: &ImportGraphEdge, right: &ImportGraphEdge) -> Ordering {
    left.ordinal()
        .cmp(&right.ordinal())
        .then_with(|| range_key(left.range()).cmp(&range_key(right.range())))
}

fn import_cycle_cmp(left: &ImportCycle, right: &ImportCycle) -> Ordering {
    first_cycle_edge(left)
        .and_then(|left_edge| {
            first_cycle_edge(right).map(|right_edge| cycle_edge_cmp(left_edge, right_edge))
        })
        .unwrap_or_else(|| left.edges().len().cmp(&right.edges().len()))
        .then_with(|| left.modules().cmp(right.modules()))
        .then_with(|| edge_slice_cmp(left.edges(), right.edges(), cycle_edge_cmp))
}

fn first_cycle_edge(cycle: &ImportCycle) -> Option<&ImportGraphEdge> {
    cycle.edges().first()
}

const fn range_key(range: SourceRange) -> (usize, usize) {
    (range.start, range.end)
}

fn cycle_edge_cmp(left: &ImportGraphEdge, right: &ImportGraphEdge) -> Ordering {
    range_key(left.range())
        .cmp(&range_key(right.range()))
        .then_with(|| left.source().cmp(right.source()))
        .then_with(|| left.target().cmp(right.target()))
        .then_with(|| left.ordinal().cmp(&right.ordinal()))
}

fn edge_slice_cmp(
    left: &[ImportGraphEdge],
    right: &[ImportGraphEdge],
    cmp: fn(&ImportGraphEdge, &ImportGraphEdge) -> Ordering,
) -> Ordering {
    for (left_edge, right_edge) in left.iter().zip(right.iter()) {
        let ordering = cmp(left_edge, right_edge);
        if !ordering.is_eq() {
            return ordering;
        }
    }
    left.len().cmp(&right.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module_index::{
        DependencyModuleSummaryRef, ModuleIndexLocation, NamespaceIndexEntry, PackageIndexEntry,
        WorkspaceStubModuleIndexProvider,
    };
    use mizar_build::module_index::PackageIndexSource;
    use mizar_session::{
        BuildSnapshotId, Edition, Hash, InMemorySessionIdAllocator, ModulePath, PackageId,
        SessionIdAllocator, SourceId,
    };
    use semver::Version;

    #[test]
    fn acyclic_fixture_builds_expected_graph_and_dependency_first_order() {
        let provider = fixture_provider();
        let graph = ImportGraphBuilder::new(ModuleIndexInput::new(&provider))
            .build(&[
                module_candidates(
                    module_id("app", "main"),
                    vec![
                        candidate("dep", "logic", 20, 24, 2),
                        candidate("app", "util", 10, 14, 0),
                        candidate("dep", "logic", 30, 34, 1),
                        candidate("dep", "logic", 25, 29, 1),
                    ],
                ),
                module_candidates(
                    module_id("app", "util"),
                    vec![candidate("dep", "logic", 40, 44, 0)],
                ),
                module_candidates(module_id("dep", "logic"), Vec::new()),
            ])
            .unwrap();

        assert!(!graph.has_cycles());
        assert_eq!(
            module_paths(graph.graph().nodes()),
            vec!["app:main", "app:util", "dep:logic"]
        );
        assert_eq!(
            edge_paths(graph.graph().edges()),
            vec![
                ("app:main", "app:util", 0, (10, 14)),
                ("app:main", "dep:logic", 1, (25, 29)),
                ("app:util", "dep:logic", 0, (40, 44)),
            ]
        );
        assert_eq!(
            module_paths(graph.graph().topological_order()),
            vec!["dep:logic", "app:util", "app:main"]
        );
    }

    #[test]
    fn independent_acyclic_components_use_canonical_ready_ties() {
        let provider = fixture_provider();
        let graph = ImportGraphBuilder::new(ModuleIndexInput::new(&provider))
            .build(&[
                module_candidates(
                    module_id("app", "main"),
                    vec![candidate("app", "util", 10, 14, 0)],
                ),
                module_candidates(
                    module_id("app", "alpha"),
                    vec![candidate("dep", "logic", 20, 24, 0)],
                ),
                module_candidates(module_id("app", "util"), Vec::new()),
                module_candidates(module_id("app", "beta"), Vec::new()),
                module_candidates(module_id("dep", "logic"), Vec::new()),
            ])
            .unwrap();

        assert_eq!(
            module_paths(graph.graph().topological_order()),
            vec!["app:beta", "app:util", "app:main", "dep:logic", "app:alpha",]
        );
    }

    #[test]
    fn cycle_fixture_is_rejected_deterministically() {
        let provider = fixture_provider();
        let first = ImportGraphBuilder::new(ModuleIndexInput::new(&provider))
            .build(&cycle_fixture_one())
            .unwrap();
        let second = ImportGraphBuilder::new(ModuleIndexInput::new(&provider))
            .build(&cycle_fixture_two())
            .unwrap();

        assert_eq!(first, second);
        assert!(first.has_cycles());
        assert_eq!(first.cycles().len(), 1);
        assert_eq!(
            module_paths(first.cycles()[0].modules()),
            vec!["app:main", "app:util"]
        );
        assert_eq!(
            edge_paths(first.cycles()[0].edges()),
            vec![
                ("app:main", "app:util", 0, (10, 14)),
                ("app:util", "app:main", 0, (20, 24)),
            ]
        );
        assert_eq!(
            module_paths(first.graph().topological_order()),
            vec!["dep:logic", "app:facade"]
        );
        assert_eq!(
            edge_paths(first.graph().edges()),
            vec![("app:facade", "dep:logic", 0, (30, 34))]
        );
    }

    #[test]
    fn self_cycle_is_rejected_deterministically() {
        let provider = fixture_provider();
        let graph = ImportGraphBuilder::new(ModuleIndexInput::new(&provider))
            .build(&[module_candidates(
                module_id("app", "main"),
                vec![candidate("app", "main", 50, 54, 0)],
            )])
            .unwrap();

        assert!(graph.has_cycles());
        assert_eq!(module_paths(graph.cycles()[0].modules()), vec!["app:main"]);
        assert!(graph.graph().nodes().is_empty());
        assert!(graph.graph().edges().is_empty());
        assert!(graph.graph().topological_order().is_empty());
    }

    #[test]
    fn independent_cycles_sort_by_source_provenance() {
        let provider = fixture_provider();
        let graph = ImportGraphBuilder::new(ModuleIndexInput::new(&provider))
            .build(&[
                module_candidates(
                    module_id("app", "alpha"),
                    vec![candidate("app", "beta", 100, 104, 0)],
                ),
                module_candidates(
                    module_id("app", "beta"),
                    vec![candidate("app", "alpha", 110, 114, 0)],
                ),
                module_candidates(
                    module_id("app", "yankee"),
                    vec![candidate("app", "zulu", 10, 14, 0)],
                ),
                module_candidates(
                    module_id("app", "zulu"),
                    vec![candidate("app", "yankee", 20, 24, 0)],
                ),
            ])
            .unwrap();

        assert_eq!(graph.cycles().len(), 2);
        assert_eq!(
            module_paths(graph.cycles()[0].modules()),
            vec!["app:yankee", "app:zulu"]
        );
        assert_eq!(
            module_paths(graph.cycles()[1].modules()),
            vec!["app:alpha", "app:beta"]
        );
    }

    #[test]
    fn unknown_modules_are_rejected_before_graph_publication() {
        let provider = fixture_provider();
        let builder = ImportGraphBuilder::new(ModuleIndexInput::new(&provider));

        let unknown_source = builder
            .build(&[module_candidates(module_id("missing", "main"), Vec::new())])
            .unwrap_err();
        match unknown_source {
            ImportGraphBuildError::UnknownSourceModule { module, lookup } => {
                assert_eq!(module_key(&module), "missing:main");
                match *lookup {
                    ModuleIndexProviderError::UnknownModule { module } => {
                        assert_eq!(module.package.as_str(), "missing");
                        assert_eq!(module.path.as_str(), "main");
                    }
                    other => panic!("expected unknown source module lookup, got {other:?}"),
                }
            }
            other => panic!("expected unknown source module, got {other:?}"),
        }

        let unknown_target = builder
            .build(&[module_candidates(
                module_id("app", "main"),
                vec![candidate("missing", "dep", 60, 64, 0)],
            )])
            .unwrap_err();
        match unknown_target {
            ImportGraphBuildError::UnknownTargetModule {
                source,
                target,
                range: actual_range,
                ordinal,
                lookup,
            } => {
                assert_eq!(module_key(&source), "app:main");
                assert_eq!(module_key(&target), "missing:dep");
                assert_eq!(actual_range, range(60, 64));
                assert_eq!(ordinal, 0);
                match *lookup {
                    ModuleIndexProviderError::UnknownModule { module } => {
                        assert_eq!(module.package.as_str(), "missing");
                        assert_eq!(module.path.as_str(), "dep");
                    }
                    other => panic!("expected unknown target module lookup, got {other:?}"),
                }
            }
            other => panic!("expected unknown target module, got {other:?}"),
        }
    }

    #[test]
    fn aliases_do_not_change_canonical_targets_or_graph_candidates() {
        let provider = fixture_provider();
        let input = ModuleIndexInput::new(&provider);
        let resolution = ImportPathResolver::new(input).resolve(
            &module_id("app", "main"),
            &[
                path_candidate(
                    &["dep", "logic"],
                    ImportPathPrefix::Unprefixed,
                    None,
                    10,
                    14,
                    0,
                ),
                path_candidate(
                    &["dep", "logic"],
                    ImportPathPrefix::Unprefixed,
                    Some("LogicAlias"),
                    20,
                    24,
                    1,
                )
                .with_alias_range(range(25, 35)),
            ],
        );

        assert!(resolution.unresolved().is_empty());
        assert_eq!(
            resolved_imports(&resolution),
            vec![
                ("dep:logic".to_owned(), "logic".to_owned(), None),
                (
                    "dep:logic".to_owned(),
                    "LogicAlias".to_owned(),
                    Some("LogicAlias".to_owned())
                ),
            ]
        );

        let graph_candidates = resolution.module_candidates(module_id("app", "main"));
        assert_eq!(
            graph_candidates
                .imports()
                .iter()
                .map(|candidate| module_key(candidate.target()).to_owned())
                .collect::<Vec<_>>(),
            vec!["dep:logic", "dep:logic"]
        );
    }

    #[test]
    fn relative_prefixes_use_dot_separated_module_directories() {
        let provider = fixture_provider();
        let input = ModuleIndexInput::new(&provider);
        let nested = ImportPathResolver::new(input).resolve(
            &module_id("app", "dir.main"),
            &[
                path_candidate(&["sibling"], ImportPathPrefix::Current, None, 10, 14, 0),
                path_candidate(&["common"], ImportPathPrefix::Parent, None, 20, 24, 1),
                path_candidate(&["missing"], ImportPathPrefix::Current, None, 25, 29, 2),
            ],
        );

        assert_eq!(
            resolved_targets(&nested),
            vec!["app:dir.sibling", "app:common"]
        );
        assert_eq!(
            unresolved_classes(&nested),
            vec![ImportPathFailureClass::UnknownModule]
        );
        assert_eq!(
            nested.unresolved()[0]
                .partial()
                .and_then(ImportPathPartialCandidate::package)
                .map(PackageId::as_str),
            Some("app")
        );
        assert_eq!(
            nested.unresolved()[0]
                .partial()
                .map(ImportPathPartialCandidate::remaining_components),
            Some(&["dir".to_owned(), "missing".to_owned()][..])
        );

        let recovered = ImportPathResolver::new(input).resolve(
            &module_id("app", "main"),
            &[
                path_candidate(&["util"], ImportPathPrefix::Parent, None, 30, 34, 0),
                path_candidate(&["util"], ImportPathPrefix::Unprefixed, None, 40, 44, 1),
            ],
        );

        assert_eq!(resolved_targets(&recovered), vec!["app:util"]);
        assert_eq!(
            unresolved_classes(&recovered),
            vec![ImportPathFailureClass::RelativePathEscapesPackage]
        );
    }

    #[test]
    fn namespace_bindings_win_over_package_local_fallback() {
        let provider = fixture_provider();
        let input = ModuleIndexInput::new(&provider);
        let resolution = ImportPathResolver::new(input).resolve(
            &module_id("app", "main"),
            &[
                path_candidate(
                    &["dep", "logic"],
                    ImportPathPrefix::Unprefixed,
                    None,
                    10,
                    14,
                    0,
                ),
                path_candidate(&["util"], ImportPathPrefix::Unprefixed, None, 20, 24, 1),
                path_candidate(
                    &["pub", "math", "logic"],
                    ImportPathPrefix::Unprefixed,
                    None,
                    30,
                    34,
                    2,
                ),
                path_candidate(
                    &["dep", "missing"],
                    ImportPathPrefix::Unprefixed,
                    None,
                    40,
                    44,
                    3,
                ),
                path_candidate(
                    &["std", "missing"],
                    ImportPathPrefix::Unprefixed,
                    None,
                    50,
                    54,
                    4,
                ),
            ],
        );

        assert_eq!(
            resolved_targets(&resolution),
            vec!["dep:logic", "app:util", "dep:logic"]
        );
        assert_eq!(
            unresolved_classes(&resolution),
            vec![
                ImportPathFailureClass::UnknownModule,
                ImportPathFailureClass::UnknownNamespaceOrPackage,
            ]
        );
        assert_eq!(
            resolution.unresolved()[0]
                .partial()
                .and_then(ImportPathPartialCandidate::package)
                .map(PackageId::as_str),
            Some("dep")
        );
        assert_eq!(
            resolution.unresolved()[1]
                .partial()
                .and_then(ImportPathPartialCandidate::namespace_root),
            Some(NamespaceRoot::Std)
        );
    }

    #[test]
    fn unresolved_imports_do_not_abort_later_candidates() {
        let provider = fixture_provider();
        let input = ModuleIndexInput::new(&provider);
        let resolution = ImportPathResolver::new(input).resolve(
            &module_id("app", "main"),
            &[
                path_candidate(&["missing"], ImportPathPrefix::Unprefixed, None, 10, 14, 0),
                path_candidate(&["util"], ImportPathPrefix::Unprefixed, None, 20, 24, 1),
                path_candidate(&["logic"], ImportPathPrefix::Unprefixed, None, 30, 34, 2)
                    .with_recovered(),
            ],
        );

        assert_eq!(resolved_targets(&resolution), vec!["app:util"]);
        assert_eq!(
            unresolved_classes(&resolution),
            vec![
                ImportPathFailureClass::UnknownModule,
                ImportPathFailureClass::RecoveredSyntax,
            ]
        );
        assert!(resolution.unresolved()[1].recovered());
        assert_eq!(
            resolution.unresolved()[1].components(),
            &["logic".to_owned()]
        );
    }

    #[test]
    fn duplicate_aliases_and_reserved_aliases_are_unresolved_deterministically() {
        let provider = fixture_provider();
        let input = ModuleIndexInput::new(&provider);
        let resolution = ImportPathResolver::new(input).resolve(
            &module_id("app", "main"),
            &[
                path_candidate(
                    &["dep", "logic"],
                    ImportPathPrefix::Unprefixed,
                    Some("Shared"),
                    10,
                    14,
                    0,
                )
                .with_alias_range(range(15, 21)),
                path_candidate(
                    &["util"],
                    ImportPathPrefix::Unprefixed,
                    Some("Shared"),
                    20,
                    24,
                    1,
                )
                .with_alias_range(range(25, 31))
                .with_branch_provenance(range(22, 24), range(25, 31)),
                path_candidate(
                    &["dep", "logic"],
                    ImportPathPrefix::Unprefixed,
                    Some("Logic"),
                    30,
                    34,
                    2,
                )
                .with_branch_provenance(range(32, 33), range(33, 34)),
                path_candidate(
                    &["dep", "logic"],
                    ImportPathPrefix::Unprefixed,
                    Some("Logic"),
                    40,
                    44,
                    3,
                ),
                path_candidate(
                    &["dep", "logic"],
                    ImportPathPrefix::Unprefixed,
                    Some("std"),
                    50,
                    54,
                    4,
                )
                .with_alias_range(range(55, 58)),
            ],
        );

        assert_eq!(
            resolved_imports(&resolution),
            vec![
                (
                    "dep:logic".to_owned(),
                    "Logic".to_owned(),
                    Some("Logic".to_owned())
                ),
                (
                    "dep:logic".to_owned(),
                    "Logic".to_owned(),
                    Some("Logic".to_owned())
                ),
            ]
        );
        assert_eq!(
            resolution
                .resolved()
                .iter()
                .map(|candidate| {
                    (
                        candidate.branch_base_range().map(range_key),
                        candidate.branch_member_range().map(range_key),
                    )
                })
                .collect::<Vec<_>>(),
            vec![(Some((32, 33)), Some((33, 34))), (None, None)]
        );
        assert_eq!(
            unresolved_classes(&resolution),
            vec![
                ImportPathFailureClass::DuplicateAlias,
                ImportPathFailureClass::DuplicateAlias,
                ImportPathFailureClass::AliasRootConflict,
            ]
        );
        assert_eq!(
            resolution
                .unresolved()
                .iter()
                .map(|candidate| (
                    candidate.alias().map(str::to_owned),
                    candidate.alias_range().map(range_key),
                    candidate.candidate_target().map(module_key),
                ))
                .collect::<Vec<_>>(),
            vec![
                (Some("Shared".to_owned()), Some((15, 21)), Some("dep:logic")),
                (Some("Shared".to_owned()), Some((25, 31)), Some("app:util")),
                (Some("std".to_owned()), Some((55, 58)), None),
            ]
        );
        assert_eq!(
            resolution.unresolved()[1]
                .branch_base_range()
                .map(range_key),
            Some((22, 24))
        );
        assert_eq!(
            resolution.unresolved()[1]
                .branch_member_range()
                .map(range_key),
            Some((25, 31))
        );
    }

    fn cycle_fixture_one() -> Vec<ModuleImportCandidates> {
        vec![
            module_candidates(
                module_id("app", "main"),
                vec![candidate("app", "util", 10, 14, 0)],
            ),
            module_candidates(
                module_id("app", "util"),
                vec![candidate("app", "main", 20, 24, 0)],
            ),
            module_candidates(
                module_id("app", "facade"),
                vec![candidate("dep", "logic", 30, 34, 0)],
            ),
            module_candidates(module_id("dep", "logic"), Vec::new()),
        ]
    }

    fn cycle_fixture_two() -> Vec<ModuleImportCandidates> {
        vec![
            module_candidates(module_id("dep", "logic"), Vec::new()),
            module_candidates(
                module_id("app", "facade"),
                vec![candidate("dep", "logic", 30, 34, 0)],
            ),
            module_candidates(
                module_id("app", "util"),
                vec![candidate("app", "main", 20, 24, 0)],
            ),
            module_candidates(
                module_id("app", "main"),
                vec![candidate("app", "util", 10, 14, 0)],
            ),
        ]
    }

    fn module_candidates(
        module: ModuleId,
        imports: Vec<ImportEdgeCandidate>,
    ) -> ModuleImportCandidates {
        ModuleImportCandidates::new(module, imports)
    }

    fn candidate(
        package: &str,
        path: &str,
        start: usize,
        end: usize,
        ordinal: usize,
    ) -> ImportEdgeCandidate {
        ImportEdgeCandidate::new(module_id(package, path), range(start, end), ordinal)
    }

    fn path_candidate(
        components: &[&str],
        prefix: ImportPathPrefix,
        alias: Option<&str>,
        start: usize,
        end: usize,
        ordinal: usize,
    ) -> ImportPathCandidate {
        ImportPathCandidate::new(
            components
                .iter()
                .map(|component| (*component).to_owned())
                .collect(),
            prefix,
            alias.map(str::to_owned),
            range(start, end),
            ordinal,
        )
    }

    fn fixture_provider() -> WorkspaceStubModuleIndexProvider {
        WorkspaceStubModuleIndexProvider::new(
            vec![package("app"), package("dep")],
            vec![
                namespace(NamespaceRoot::PackageName, &["app"], "app"),
                namespace(NamespaceRoot::PackageName, &["dep"], "dep"),
                namespace(NamespaceRoot::Pub, &["math"], "dep"),
            ],
            vec![
                workspace_module("app", "main"),
                workspace_module("app", "util"),
                workspace_module("app", "common"),
                workspace_module("app", "dir.main"),
                workspace_module("app", "dir.sibling"),
                workspace_module("app", "dep.logic"),
                workspace_module("app", "dep.missing"),
                workspace_module("app", "std.missing"),
                workspace_module("app", "facade"),
                workspace_module("app", "alpha"),
                workspace_module("app", "beta"),
                workspace_module("app", "yankee"),
                workspace_module("app", "zulu"),
                dependency_module("dep", "logic"),
                dependency_module("dep", "algebra.group"),
            ],
            vec![DependencyModuleSummaryRef {
                module: indexed_module("dep", "logic"),
                artifact: "dep.logic.summary".to_owned(),
                content_hash: Hash::from_bytes([9; Hash::BYTE_LEN]),
            }],
        )
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

    fn workspace_module(package_id: &str, path: &str) -> crate::module_index::ModuleIndexEntry {
        crate::module_index::ModuleIndexEntry {
            module: indexed_module(package_id, path),
            package_id: PackageId::new(package_id),
            module_path: ModulePath::new(path),
            location: ModuleIndexLocation::WorkspaceFile {
                source_root: format!("/workspace/{package_id}/src"),
                normalized_path: format!("/workspace/{package_id}/src/{path}.miz"),
                source_relative_path: format!("{path}.miz"),
            },
            edition: Edition::new("2026"),
        }
    }

    fn dependency_module(package_id: &str, path: &str) -> crate::module_index::ModuleIndexEntry {
        crate::module_index::ModuleIndexEntry {
            module: indexed_module(package_id, path),
            package_id: PackageId::new(package_id),
            module_path: ModulePath::new(path),
            location: ModuleIndexLocation::DependencySummary {
                artifact: format!("{package_id}.{path}.summary"),
                content_hash: Hash::from_bytes([9; Hash::BYTE_LEN]),
            },
            edition: Edition::new("2026"),
        }
    }

    fn indexed_module(package_id: &str, path: &str) -> IndexedModuleId {
        IndexedModuleId::new(PackageId::new(package_id), ModulePath::new(path))
    }

    fn module_id(package: &str, path: &str) -> ModuleId {
        ModuleId::new(PackageId::new(package), ModulePath::new(path))
    }

    fn module_paths(modules: &[ModuleId]) -> Vec<String> {
        modules
            .iter()
            .map(|module| format!("{}:{}", module.package().as_str(), module.path().as_str()))
            .collect()
    }

    fn edge_paths(edges: &[ImportGraphEdge]) -> Vec<(&str, &str, usize, (usize, usize))> {
        edges
            .iter()
            .map(|edge| {
                (
                    module_key(edge.source()),
                    module_key(edge.target()),
                    edge.ordinal(),
                    range_key(edge.range()),
                )
            })
            .collect()
    }

    fn module_key(module: &ModuleId) -> &str {
        match (module.package().as_str(), module.path().as_str()) {
            ("app", "main") => "app:main",
            ("app", "util") => "app:util",
            ("app", "common") => "app:common",
            ("app", "dir.main") => "app:dir.main",
            ("app", "dir.sibling") => "app:dir.sibling",
            ("app", "dep.logic") => "app:dep.logic",
            ("app", "dep.missing") => "app:dep.missing",
            ("app", "std.missing") => "app:std.missing",
            ("app", "facade") => "app:facade",
            ("app", "alpha") => "app:alpha",
            ("app", "beta") => "app:beta",
            ("app", "yankee") => "app:yankee",
            ("app", "zulu") => "app:zulu",
            ("dep", "logic") => "dep:logic",
            ("dep", "algebra.group") => "dep:algebra.group",
            ("missing", "dep") => "missing:dep",
            ("missing", "main") => "missing:main",
            _ => "unknown",
        }
    }

    fn resolved_targets(resolution: &ImportPathResolution) -> Vec<&str> {
        resolution
            .resolved()
            .iter()
            .map(|candidate| module_key(candidate.target()))
            .collect()
    }

    fn resolved_imports(
        resolution: &ImportPathResolution,
    ) -> Vec<(String, String, Option<String>)> {
        resolution
            .resolved()
            .iter()
            .map(|candidate| {
                (
                    module_key(candidate.target()).to_owned(),
                    candidate.alias().to_owned(),
                    candidate.explicit_alias().map(str::to_owned),
                )
            })
            .collect()
    }

    fn unresolved_classes(resolution: &ImportPathResolution) -> Vec<ImportPathFailureClass> {
        resolution
            .unresolved()
            .iter()
            .map(UnresolvedImportCandidate::class)
            .collect()
    }

    fn source_id() -> SourceId {
        let snapshot_id = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            "03".repeat(Hash::BYTE_LEN)
        ))
        .unwrap();
        InMemorySessionIdAllocator::new()
            .next_source_id(snapshot_id)
            .unwrap()
    }

    fn range(start: usize, end: usize) -> SourceRange {
        SourceRange {
            source_id: source_id(),
            start,
            end,
        }
    }
}
