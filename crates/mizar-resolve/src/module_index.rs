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
mod tests {
    use super::*;
    use mizar_build::module_index::PackageIndexSource;
    use mizar_session::{Edition, Hash, ModulePath};
    use semver::Version;

    #[test]
    fn stub_provider_feeds_multi_module_fixture() {
        let provider = fixture_provider();
        let input = ModuleIndexInput::new(&provider);
        let app_package = PackageId::new("app");
        let dep_package = PackageId::new("dep");

        let app_modules = input.modules_for_package(&app_package).unwrap();
        assert_eq!(app_modules.len(), 2);
        assert_eq!(app_modules[0].module.path.as_str(), "main");
        assert_eq!(app_modules[1].module.path.as_str(), "util");

        let dep_modules = input.modules_for_package(&dep_package).unwrap();
        assert_eq!(dep_modules.len(), 1);
        assert_eq!(dep_modules[0].module.path.as_str(), "logic");

        let pub_prefix = vec!["math".to_owned()];
        let package = input
            .package_for_namespace(&NamespaceRoot::Pub, &pub_prefix)
            .unwrap();
        assert_eq!(package.package_id.as_str(), "dep");

        let module = indexed_module("dep", "logic");
        let entry = input.module(&module).unwrap();
        assert_eq!(entry.module.path.as_str(), "logic");
        assert_eq!(
            input.dependency_summary(&module).unwrap().artifact,
            "dep.logic.summary"
        );
    }

    #[test]
    fn forwarded_packages_preserve_provider_order_and_namespaces_are_canonical() {
        let provider = fixture_provider();
        let input = ModuleIndexInput::new(&provider);
        let packages = input
            .packages()
            .iter()
            .map(|package| package.package_id.as_str())
            .collect::<Vec<_>>();
        let namespace_keys = input
            .namespace_bindings()
            .iter()
            .map(|binding| {
                (
                    binding.root,
                    binding.prefix.join("."),
                    binding.package_id.as_str(),
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(packages, vec!["dep", "app"]);
        assert_eq!(
            namespace_keys,
            vec![
                (NamespaceRoot::PackageName, "app".to_owned(), "app"),
                (NamespaceRoot::PackageName, "dep".to_owned(), "dep"),
                (NamespaceRoot::Pub, "math".to_owned(), "dep"),
                (NamespaceRoot::Ext, "renamed_math".to_owned(), "dep"),
            ]
        );
    }

    #[test]
    fn module_identity_is_alias_independent() {
        let provider = fixture_provider();
        let input = ModuleIndexInput::new(&provider);
        let pub_prefix = vec!["math".to_owned()];
        let ext_prefix = vec!["renamed_math".to_owned()];

        let pub_package = input
            .package_for_namespace(&NamespaceRoot::Pub, &pub_prefix)
            .unwrap();
        let ext_package = input
            .package_for_namespace(&NamespaceRoot::Ext, &ext_prefix)
            .unwrap();
        let module_from_pub =
            IndexedModuleId::new(pub_package.package_id.clone(), ModulePath::new("logic"));
        let module_from_ext =
            IndexedModuleId::new(ext_package.package_id.clone(), ModulePath::new("logic"));

        assert_eq!(
            input.resolver_module_id(&module_from_pub),
            input.resolver_module_id(&module_from_ext)
        );
        assert_eq!(
            input.resolver_module_id(&module_from_pub),
            resolved_ast::ModuleId::new(PackageId::new("dep"), ModulePath::new("logic"))
        );
    }

    #[test]
    fn provider_errors_are_deterministic() {
        let provider = fixture_provider();
        let input = ModuleIndexInput::new(&provider);
        let unknown_package = PackageId::new("missing");
        let unknown_prefix = vec!["missing".to_owned()];
        let unknown_module = indexed_module("app", "missing");
        let workspace_module = indexed_module("app", "main");

        assert_eq!(
            input.package(&unknown_package).unwrap_err(),
            input.package(&unknown_package).unwrap_err()
        );
        assert_eq!(
            input
                .package_for_namespace(&NamespaceRoot::Std, &unknown_prefix)
                .unwrap_err(),
            input
                .package_for_namespace(&NamespaceRoot::Std, &unknown_prefix)
                .unwrap_err()
        );
        assert_eq!(
            input.module(&unknown_module).unwrap_err(),
            input.module(&unknown_module).unwrap_err()
        );
        assert_eq!(
            input.modules_for_package(&unknown_package).unwrap_err(),
            input.modules_for_package(&unknown_package).unwrap_err()
        );
        assert_eq!(
            input.dependency_summary(&unknown_module).unwrap_err(),
            input.dependency_summary(&unknown_module).unwrap_err()
        );
        assert_eq!(
            input.dependency_summary(&workspace_module).unwrap_err(),
            input.dependency_summary(&workspace_module).unwrap_err()
        );
    }

    fn fixture_provider() -> WorkspaceStubModuleIndexProvider {
        WorkspaceStubModuleIndexProvider::new(
            vec![package("dep"), package("app")],
            vec![
                namespace(NamespaceRoot::PackageName, &["dep"], "dep"),
                namespace(NamespaceRoot::PackageName, &["app"], "app"),
                namespace(NamespaceRoot::Pub, &["math"], "dep"),
                namespace(NamespaceRoot::Ext, &["renamed_math"], "dep"),
            ],
            vec![
                workspace_module("app", "util"),
                dependency_module("dep", "logic"),
                workspace_module("app", "main"),
            ],
            vec![DependencyModuleSummaryRef {
                module: indexed_module("dep", "logic"),
                artifact: "dep.logic.summary".to_owned(),
                content_hash: Hash::from_bytes([7; Hash::BYTE_LEN]),
            }],
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

    fn dependency_module(package_id: &str, path: &str) -> ModuleIndexEntry {
        ModuleIndexEntry {
            module: indexed_module(package_id, path),
            package_id: PackageId::new(package_id),
            module_path: ModulePath::new(path),
            location: ModuleIndexLocation::DependencySummary {
                artifact: format!("{package_id}.{path}.summary"),
                content_hash: Hash::from_bytes([7; Hash::BYTE_LEN]),
            },
            edition: Edition::new("2026"),
        }
    }

    fn indexed_module(package_id: &str, path: &str) -> IndexedModuleId {
        IndexedModuleId::new(PackageId::new(package_id), ModulePath::new(path))
    }
}
