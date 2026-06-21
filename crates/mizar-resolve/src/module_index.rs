//! Resolver-side module-index phase input seam.
//!
//! The resolver consumes the build-side module-index provider as an input
//! service. This module keeps the resolver boundary narrow: it forwards
//! package, namespace, module, and dependency-summary lookups, and only
//! normalizes build-owned module identities into resolver-owned identities.

use crate::resolved_ast;
pub use mizar_build::module_index::{
    DependencyModuleSummaryRef, ModuleId as IndexedModuleId, ModuleIndex, ModuleIndexEntry,
    ModuleIndexLocation, ModuleIndexProvider, ModuleIndexProviderError, NamespaceIndexEntry,
    NamespaceRoot, PackageIndexEntry,
};
use mizar_session::PackageId;

/// Borrowed resolver phase input backed by a build-side module-index provider.
#[derive(Clone, Copy)]
pub struct ModuleIndexInput<'a> {
    provider: &'a dyn ModuleIndexProvider,
}

impl<'a> ModuleIndexInput<'a> {
    /// Creates a resolver phase input from the build-side provider contract.
    pub fn new(provider: &'a dyn ModuleIndexProvider) -> Self {
        Self { provider }
    }

    /// Returns the underlying build-side provider.
    pub fn provider(&self) -> &'a dyn ModuleIndexProvider {
        self.provider
    }

    /// Returns package entries in provider-defined canonical order.
    pub fn packages(&self) -> &'a [PackageIndexEntry] {
        self.provider.packages()
    }

    /// Returns namespace bindings in provider-defined canonical order.
    pub fn namespace_bindings(&self) -> &'a [NamespaceIndexEntry] {
        self.provider.namespace_bindings()
    }

    /// Looks up a package entry by canonical package identity.
    pub fn package(
        &self,
        package: &PackageId,
    ) -> Result<&'a PackageIndexEntry, ModuleIndexProviderError> {
        self.provider.package(package)
    }

    /// Resolves a namespace root and prefix through the build-side index.
    pub fn package_for_namespace(
        &self,
        root: &NamespaceRoot,
        prefix: &[String],
    ) -> Result<&'a PackageIndexEntry, ModuleIndexProviderError> {
        self.provider.package_for_namespace(root, prefix)
    }

    /// Looks up a module entry by canonical, alias-independent module identity.
    pub fn module(
        &self,
        module: &IndexedModuleId,
    ) -> Result<&'a ModuleIndexEntry, ModuleIndexProviderError> {
        self.provider.module(module)
    }

    /// Returns the package's modules in provider-defined canonical order.
    pub fn modules_for_package(
        &self,
        package: &PackageId,
    ) -> Result<&'a [ModuleIndexEntry], ModuleIndexProviderError> {
        self.provider.modules_for_package(package)
    }

    /// Looks up a dependency module summary reference without parsing artifacts.
    pub fn dependency_summary(
        &self,
        module: &IndexedModuleId,
    ) -> Result<&'a DependencyModuleSummaryRef, ModuleIndexProviderError> {
        self.provider.dependency_summary(module)
    }

    /// Converts a build-side module identity into the resolver output identity.
    pub fn resolver_module_id(&self, module: &IndexedModuleId) -> resolved_ast::ModuleId {
        resolver_module_id(module)
    }

    /// Returns the resolver identity for a module-index entry.
    pub fn module_identity(&self, entry: &ModuleIndexEntry) -> resolved_ast::ModuleId {
        resolver_module_id(&entry.module)
    }
}

/// Converts a build-side module identity into the resolver output identity.
#[must_use]
pub fn resolver_module_id(module: &IndexedModuleId) -> resolved_ast::ModuleId {
    resolved_ast::ModuleId::new(module.package.clone(), module.path.clone())
}

/// Resolver-owned workspace stub for module-index unit tests and fixtures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceStubModuleIndexProvider {
    index: ModuleIndex,
}

impl WorkspaceStubModuleIndexProvider {
    /// Creates a deterministic in-memory provider from explicit index entries.
    ///
    /// Package order is preserved as the caller-provided build-plan order;
    /// namespace, module, and dependency-summary slices are canonicalized.
    #[must_use]
    pub fn new(
        packages: Vec<PackageIndexEntry>,
        namespace_bindings: Vec<NamespaceIndexEntry>,
        modules: Vec<ModuleIndexEntry>,
        dependency_summaries: Vec<DependencyModuleSummaryRef>,
    ) -> Self {
        Self::from_index(ModuleIndex {
            packages,
            namespace_bindings,
            modules,
            dependency_summaries,
        })
    }

    /// Creates a deterministic stub from a build-side module index.
    #[must_use]
    pub fn from_index(mut index: ModuleIndex) -> Self {
        sort_index(&mut index);
        Self { index }
    }

    /// Returns the stored module index.
    #[must_use]
    pub fn index(&self) -> &ModuleIndex {
        &self.index
    }
}

impl ModuleIndexProvider for WorkspaceStubModuleIndexProvider {
    fn packages(&self) -> &[PackageIndexEntry] {
        self.index.packages()
    }

    fn namespace_bindings(&self) -> &[NamespaceIndexEntry] {
        self.index.namespace_bindings()
    }

    fn package(&self, package: &PackageId) -> Result<&PackageIndexEntry, ModuleIndexProviderError> {
        self.index.package(package)
    }

    fn package_for_namespace(
        &self,
        root: &NamespaceRoot,
        prefix: &[String],
    ) -> Result<&PackageIndexEntry, ModuleIndexProviderError> {
        self.index.package_for_namespace(root, prefix)
    }

    fn module(
        &self,
        module: &IndexedModuleId,
    ) -> Result<&ModuleIndexEntry, ModuleIndexProviderError> {
        self.index.module(module)
    }

    fn modules_for_package(
        &self,
        package: &PackageId,
    ) -> Result<&[ModuleIndexEntry], ModuleIndexProviderError> {
        self.index.modules_for_package(package)
    }

    fn dependency_summary(
        &self,
        module: &IndexedModuleId,
    ) -> Result<&DependencyModuleSummaryRef, ModuleIndexProviderError> {
        self.index.dependency_summary(module)
    }
}

fn sort_index(index: &mut ModuleIndex) {
    index.namespace_bindings.sort_by(|left, right| {
        left.root
            .cmp(&right.root)
            .then_with(|| left.prefix.cmp(&right.prefix))
            .then_with(|| left.package_id.as_str().cmp(right.package_id.as_str()))
    });
    index.modules.sort_by(module_entry_cmp);
    index
        .dependency_summaries
        .sort_by(|left, right| module_id_cmp(&left.module, &right.module));
}

fn module_entry_cmp(left: &ModuleIndexEntry, right: &ModuleIndexEntry) -> std::cmp::Ordering {
    left.package_id
        .as_str()
        .cmp(right.package_id.as_str())
        .then_with(|| left.module_path.as_str().cmp(right.module_path.as_str()))
        .then_with(|| location_key(&left.location).cmp(&location_key(&right.location)))
}

fn module_id_cmp(left: &IndexedModuleId, right: &IndexedModuleId) -> std::cmp::Ordering {
    left.package
        .as_str()
        .cmp(right.package.as_str())
        .then_with(|| left.path.as_str().cmp(right.path.as_str()))
}

fn location_key(location: &ModuleIndexLocation) -> Vec<String> {
    match location {
        ModuleIndexLocation::WorkspaceFile {
            normalized_path, ..
        } => vec!["0".to_owned(), normalized_path.clone()],
        ModuleIndexLocation::DependencySummary {
            artifact,
            content_hash,
        } => vec![
            "1".to_owned(),
            artifact.clone(),
            hash_hex(content_hash.as_bytes()),
        ],
    }
}

fn hash_hex(bytes: &[u8; mizar_session::Hash::BYTE_LEN]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}

#[cfg(test)]
mod tests;
