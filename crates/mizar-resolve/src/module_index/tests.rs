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
