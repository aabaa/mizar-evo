use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
};

use crate::planner::{BuildPlan, PackagePlan, PackagePlanSource, ResolvedPackageDependency};
use mizar_session::{Edition, Hash, ModulePath, PackageId};
use semver::Version;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleIndex {
    pub packages: Vec<PackageIndexEntry>,
    pub namespace_bindings: Vec<NamespaceIndexEntry>,
    pub modules: Vec<ModuleIndexEntry>,
    pub dependency_summaries: Vec<DependencyModuleSummaryRef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageIndexEntry {
    pub package_id: PackageId,
    pub version: Version,
    pub edition: Edition,
    pub source: PackageIndexSource,
    pub dependencies: Vec<PackageId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PackageIndexSource {
    Workspace {
        package_root: String,
        source_root: String,
        manifest_path: String,
    },
    RegistryArtifact {
        registry: String,
        checksum: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamespaceIndexEntry {
    pub root: NamespaceRoot,
    pub prefix: Vec<String>,
    pub package_id: PackageId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum NamespaceRoot {
    PackageName,
    Std,
    Pub,
    Pkg,
    Dev,
    Ext,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleId {
    pub package: PackageId,
    pub path: ModulePath,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleIndexEntry {
    pub module: ModuleId,
    pub package_id: PackageId,
    pub module_path: ModulePath,
    pub location: ModuleIndexLocation,
    pub edition: Edition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ModuleIndexLocation {
    WorkspaceFile {
        source_root: String,
        normalized_path: String,
        source_relative_path: String,
    },
    DependencySummary {
        artifact: String,
        content_hash: Hash,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyModuleSummaryRef {
    pub module: ModuleId,
    pub artifact: String,
    pub content_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyArtifactIndex {
    pub package_id: PackageId,
    pub namespace_bindings: Vec<ArtifactNamespaceBinding>,
    pub summaries: Vec<DependencyModuleSummaryRef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactNamespaceBinding {
    pub root: NamespaceRoot,
    pub prefix: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSourcePackage {
    pub package_id: PackageId,
    pub files: Vec<WorkspaceSourceFile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSourceFile {
    pub normalized_path: String,
    pub source_relative_path: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StaticSourceLayout {
    files_by_package: BTreeMap<String, Vec<WorkspaceSourceFile>>,
}

pub trait SourceLayoutProvider {
    fn workspace_source_files(
        &self,
        package: &PackagePlan,
    ) -> Result<Vec<WorkspaceSourceFile>, ModuleIndexDiagnostics>;
}

pub trait ModuleIndexProvider {
    fn packages(&self) -> &[PackageIndexEntry];
    fn namespace_bindings(&self) -> &[NamespaceIndexEntry];
    fn package(&self, package: &PackageId) -> Result<&PackageIndexEntry, ModuleIndexProviderError>;
    fn package_for_namespace(
        &self,
        root: &NamespaceRoot,
        prefix: &[String],
    ) -> Result<&PackageIndexEntry, ModuleIndexProviderError>;
    fn module(&self, module: &ModuleId) -> Result<&ModuleIndexEntry, ModuleIndexProviderError>;
    fn modules_for_package(
        &self,
        package: &PackageId,
    ) -> Result<&[ModuleIndexEntry], ModuleIndexProviderError>;
    fn dependency_summary(
        &self,
        module: &ModuleId,
    ) -> Result<&DependencyModuleSummaryRef, ModuleIndexProviderError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleIndexDiagnostics {
    diagnostics: Vec<ModuleIndexDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleIndexDiagnostic {
    pub package_id: Option<String>,
    pub module: Option<ModuleId>,
    pub normalized_path: Option<String>,
    pub kind: ModuleIndexDiagnosticKind,
    pub value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ModuleIndexDiagnosticKind {
    MissingSourceLayout,
    InvalidSourcePath,
    InvalidModuleComponent,
    EmptyModulePath,
    DuplicateModule,
    DuplicateNamespaceBinding,
    UnsupportedNamespaceRoot,
    InvalidNamespacePrefix,
    UnknownPackage,
    MissingDependencySummary,
    MalformedSummaryIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ModuleIndexProviderError {
    UnknownPackage {
        package_id: String,
    },
    UnknownNamespace {
        root: NamespaceRoot,
        prefix: Vec<String>,
    },
    UnknownModule {
        module: ModuleId,
    },
    UnavailableDependencySummary {
        module: ModuleId,
    },
}

impl ModuleId {
    #[must_use]
    pub fn new(package: PackageId, path: ModulePath) -> Self {
        Self { package, path }
    }
}

impl DependencyModuleSummaryRef {
    /// Reads a current-schema summary under the caller's explicit artifact root.
    /// Checks the store hash and known module identity, not cache/proof acceptance.
    pub fn read_current_summary(
        &self,
        artifact_root: &std::path::Path,
    ) -> Option<mizar_artifact::store::CanonicalJson> {
        use mizar_artifact::{
            module_summary::{
                MODULE_SUMMARY_SCHEMA_FAMILY, ModuleSummaryReadOptions, current_schema_version,
                read_module_summary,
            },
            store::{
                PublishedArtifactPath, PublishedArtifactReadOptions, artifact_hash_domain,
                read_published_artifact,
            },
        };
        let path = PublishedArtifactPath::new(&self.artifact).ok()?;
        let domain = artifact_hash_domain(MODULE_SUMMARY_SCHEMA_FAMILY, current_schema_version());
        let read = read_published_artifact(
            artifact_root,
            &path,
            PublishedArtifactReadOptions {
                artifact_hash_domain: Some(&domain),
                expected_artifact_hash: Some(self.content_hash),
                hash_excluded_paths: &[],
            },
        )
        .ok()?;
        let summary = read_module_summary(
            &read.value,
            ModuleSummaryReadOptions {
                artifact_path: Some(path.as_str()),
                ..ModuleSummaryReadOptions::default()
            },
        )
        .ok()?;
        (summary.schema_version == current_schema_version()
            && summary.module.package_id == self.module.package.as_str()
            && summary.module.module_path == self.module.path.as_str())
        .then_some(read.value)
    }
}

impl DependencyArtifactIndex {
    /// Projects canonical manifest metadata bound to a planned package.
    /// Does not validate referenced files or grant cache/proof reuse credit.
    pub fn from_manifest(
        package: &PackagePlan,
        value: &mizar_artifact::store::CanonicalJson,
        namespace_bindings: Vec<ArtifactNamespaceBinding>,
    ) -> Option<Self> {
        use mizar_artifact::manifest::{ArtifactManifestReadOptions, read_artifact_manifest};

        let manifest =
            read_artifact_manifest(value, ArtifactManifestReadOptions::default()).ok()?;
        let version = package.version.to_string();
        if manifest.package.package_id != package.package_id.as_str()
            || manifest.package.package_version.as_deref() != Some(version.as_str())
            || manifest.language_edition != package.edition.as_str()
            || manifest.modules.iter().any(|entry| {
                entry.module.package_id != package.package_id.as_str()
                    || entry.module.package_version.as_deref() != Some(version.as_str())
                    || entry.module.language_edition != package.edition.as_str()
                    || entry.module.lockfile_identity != manifest.package.lockfile_identity
            })
        {
            return None;
        }
        let mut summaries = Vec::new();
        for entry in manifest.modules {
            if let (Some(artifact), Some(hash)) =
                (entry.module_summary_file, entry.module_summary_hash)
            {
                if hash.schema_version != mizar_artifact::module_summary::current_schema_version() {
                    return None;
                }
                summaries.push(DependencyModuleSummaryRef {
                    module: ModuleId::new(
                        package.package_id.clone(),
                        ModulePath::new(entry.module.module_path),
                    ),
                    artifact,
                    content_hash: hash.digest,
                });
            }
        }
        Some(Self::new(
            package.package_id.clone(),
            namespace_bindings,
            summaries,
        ))
    }

    #[must_use]
    pub fn new(
        package_id: PackageId,
        namespace_bindings: Vec<ArtifactNamespaceBinding>,
        summaries: Vec<DependencyModuleSummaryRef>,
    ) -> Self {
        Self {
            package_id,
            namespace_bindings,
            summaries,
        }
    }
}

impl ArtifactNamespaceBinding {
    #[must_use]
    pub fn new(root: NamespaceRoot, prefix: Vec<String>) -> Self {
        Self { root, prefix }
    }
}

impl WorkspaceSourceFile {
    #[must_use]
    pub fn new(
        normalized_path: impl Into<String>,
        source_relative_path: impl Into<String>,
    ) -> Self {
        Self {
            normalized_path: normalized_path.into(),
            source_relative_path: source_relative_path.into(),
        }
    }
}

impl StaticSourceLayout {
    /// Acquires metadata-only workspace layout before source capture and indexing.
    pub fn from_native_disk(plan: &BuildPlan) -> Result<Self, String> {
        use std::{fs, path::Path};

        let workspace = Path::new(plan.workspace_root.as_str())
            .components()
            .collect::<std::path::PathBuf>();
        if !fs::symlink_metadata(&workspace)
            .map_err(|error| error.to_string())?
            .is_dir()
        {
            return Err("workspace root must be an ordinary directory".into());
        }
        let workspace = fs::canonicalize(workspace).map_err(|error| error.to_string())?;
        let mut packages = Vec::new();
        let mut package_ids = BTreeSet::new();
        for package in &plan.packages {
            if !package_ids.insert(package.package_id.as_str()) {
                return Err("duplicate package ID in source layout plan".into());
            }
            let PackagePlanSource::Workspace {
                root, source_root, ..
            } = &package.source
            else {
                continue;
            };
            if root != "." && (!is_normalized_relative_path(root) || Path::new(root).is_absolute())
            {
                return Err("package root must be a normalized relative path".into());
            }
            let expected_source_root = if root == "." {
                "src".to_owned()
            } else {
                format!("{root}/src")
            };
            if source_root != &expected_source_root {
                return Err("source root must be the planned package src directory".into());
            }
            let mut package_root = workspace.clone();
            if root != "." {
                for component in Path::new(root).components() {
                    if !matches!(component, std::path::Component::Normal(_)) {
                        return Err("unsupported package root component".into());
                    }
                    package_root.push(component);
                    if !fs::symlink_metadata(&package_root)
                        .map_err(|error| error.to_string())?
                        .is_dir()
                    {
                        return Err("package root must contain only ordinary directories".into());
                    }
                }
            }
            let package_root = fs::canonicalize(package_root).map_err(|error| error.to_string())?;
            if !package_root.starts_with(&workspace) {
                return Err("package root escapes workspace".into());
            }
            let source = package_root.join("src");
            if !fs::symlink_metadata(&source)
                .map_err(|error| error.to_string())?
                .is_dir()
                || fs::canonicalize(&source).map_err(|error| error.to_string())? != source
            {
                return Err("source root must be the ordinary package src directory".into());
            }
            let mut directories = vec![source.clone()];
            let mut files = Vec::new();
            while let Some(directory) = directories.pop() {
                if !fs::symlink_metadata(&directory)
                    .map_err(|error| error.to_string())?
                    .is_dir()
                {
                    return Err("source directory changed or is unsupported".into());
                }
                for entry in fs::read_dir(directory).map_err(|error| error.to_string())? {
                    let path = entry.map_err(|error| error.to_string())?.path();
                    let metadata =
                        fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
                    if metadata.is_dir() {
                        path.to_str()
                            .ok_or("unsupported source directory encoding")?;
                        directories.push(path);
                    } else if metadata.is_file() {
                        if path.extension() != Some(std::ffi::OsStr::new("miz")) {
                            continue;
                        }
                        let relative = path
                            .strip_prefix(&source)
                            .map_err(|error| error.to_string())?
                            .components()
                            .map(|component| {
                                component
                                    .as_os_str()
                                    .to_str()
                                    .ok_or("unsupported source path encoding")
                            })
                            .collect::<Result<Vec<_>, _>>()?
                            .join("/");
                        let normalized = mizar_session::normalize_source_path(&package_root, &path)
                            .map_err(|error| error.to_string())?;
                        let file = WorkspaceSourceFile::new(normalized.as_str(), relative);
                        let mut diagnostics = Vec::new();
                        if file.normalized_path != format!("src/{}", file.source_relative_path)
                            || validate_workspace_source_file(package, &file, &mut diagnostics)
                                .is_none()
                        {
                            return Err(
                                "source path does not match its lexical src coordinate".into()
                            );
                        }
                        files.push(file);
                    } else {
                        return Err("unsupported source tree entry".into());
                    }
                }
            }
            files.sort_by(|left, right| {
                (&left.normalized_path, &left.source_relative_path)
                    .cmp(&(&right.normalized_path, &right.source_relative_path))
            });
            packages.push(WorkspaceSourcePackage {
                package_id: package.package_id.clone(),
                files,
            });
        }
        Ok(Self::new(packages))
    }

    #[must_use]
    pub fn new(packages: Vec<WorkspaceSourcePackage>) -> Self {
        let files_by_package = packages
            .into_iter()
            .map(|package| (package.package_id.as_str().to_owned(), package.files))
            .collect();
        Self { files_by_package }
    }
}

impl SourceLayoutProvider for StaticSourceLayout {
    fn workspace_source_files(
        &self,
        package: &PackagePlan,
    ) -> Result<Vec<WorkspaceSourceFile>, ModuleIndexDiagnostics> {
        self.files_by_package
            .get(package.package_id.as_str())
            .cloned()
            .ok_or_else(|| {
                ModuleIndexDiagnostics::new(vec![ModuleIndexDiagnostic::new(
                    Some(package.package_id.as_str().to_owned()),
                    None,
                    None,
                    ModuleIndexDiagnosticKind::MissingSourceLayout,
                    Some(package.package_id.as_str().to_owned()),
                )])
            })
    }
}

impl ModuleIndex {
    /// Associates a raw buffer with a unique native indexed file before source capture.
    pub fn native_open_buffer_input(
        &self,
        workspace_root: &mizar_session::WorkspaceRoot,
        package: &PackageId,
        loader: &mizar_session::DiskSourceLoader,
        origin: mizar_session::SourceOriginInput,
    ) -> Result<mizar_session::SourceInput, String> {
        use std::{fs, path::Path};

        let mizar_session::SourceOriginInput::OpenBuffer { uri, .. } = &origin else {
            return Err("native buffer association requires an OpenBuffer origin".into());
        };
        let mut packages = self
            .packages
            .iter()
            .filter(|entry| &entry.package_id == package);
        let indexed = packages.next().ok_or("missing indexed package")?;
        if packages.next().is_some() {
            return Err("ambiguous indexed package".into());
        }
        let PackageIndexSource::Workspace {
            package_root,
            source_root,
            ..
        } = &indexed.source
        else {
            return Err("native buffer association requires a workspace package".into());
        };
        if package_root != "."
            && (!is_normalized_relative_path(package_root) || Path::new(package_root).is_absolute())
        {
            return Err("package root must be a normalized relative path".into());
        }
        let expected_source_root = if package_root == "." {
            "src".to_owned()
        } else {
            format!("{package_root}/src")
        };
        if source_root != &expected_source_root {
            return Err("source root must be the indexed package src directory".into());
        }
        let workspace =
            fs::canonicalize(workspace_root.as_str()).map_err(|error| error.to_string())?;
        let expected_root =
            fs::canonicalize(workspace.join(package_root)).map_err(|error| error.to_string())?;
        if !expected_root.starts_with(&workspace)
            || fs::canonicalize(loader.package_root()).map_err(|error| error.to_string())?
                != expected_root
        {
            return Err("loader root does not match the indexed workspace package".into());
        }
        let normalized_path = loader
            .normalize_open_buffer_uri(uri)
            .map_err(|error| error.to_string())?;
        let mut entries = self.modules.iter().filter(|entry| {
            &entry.package_id == package
                && matches!(&entry.location, ModuleIndexLocation::WorkspaceFile { normalized_path: path, .. }
                    if path == normalized_path.as_str())
        });
        let entry = entries.next().ok_or("missing indexed native file")?;
        if entries.next().is_some()
            || self
                .modules
                .iter()
                .filter(|candidate| {
                    candidate.module == entry.module
                        || (candidate.package_id == entry.package_id
                            && candidate.module_path == entry.module_path)
                })
                .count()
                != 1
        {
            return Err("ambiguous indexed native file or module identity".into());
        }
        let ModuleIndexLocation::WorkspaceFile {
            source_root: entry_root,
            normalized_path: entry_path,
            source_relative_path,
        } = &entry.location
        else {
            return Err("indexed module is not a native file".into());
        };
        let components = module_path_components_from_source_relative_path(source_relative_path)
            .ok_or("invalid indexed source-relative module path")?;
        let module_path = components.join(".");
        if entry.module.package != *package
            || entry.module.path != entry.module_path
            || entry.edition != indexed.edition
            || entry_root != source_root
            || entry_path != &format!("src/{source_relative_path}")
            || !is_valid_module_path(&module_path)
            || entry.module_path.as_str() != module_path
        {
            return Err("inconsistent indexed native module coordinates".into());
        }
        Ok(mizar_session::SourceInput {
            package_id: entry.package_id.clone(),
            module_path: entry.module_path.clone(),
            normalized_path,
            edition: entry.edition.clone(),
            origin,
        })
    }
}

impl ModuleIndexDiagnostics {
    #[must_use]
    pub fn new(mut diagnostics: Vec<ModuleIndexDiagnostic>) -> Self {
        sort_diagnostics(&mut diagnostics);
        Self { diagnostics }
    }

    #[must_use]
    pub fn diagnostics(&self) -> &[ModuleIndexDiagnostic] {
        &self.diagnostics
    }

    #[must_use]
    pub fn into_diagnostics(self) -> Vec<ModuleIndexDiagnostic> {
        self.diagnostics
    }
}

impl ModuleIndexDiagnostic {
    fn new(
        package_id: Option<String>,
        module: Option<ModuleId>,
        normalized_path: Option<String>,
        kind: ModuleIndexDiagnosticKind,
        value: Option<String>,
    ) -> Self {
        Self {
            package_id,
            module,
            normalized_path,
            kind,
            value,
        }
    }
}

impl fmt::Display for ModuleIndexProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownPackage { package_id } => {
                write!(f, "unknown package `{package_id}`")
            }
            Self::UnknownNamespace { root, prefix } => {
                write!(f, "unknown namespace `{:?}:{}`", root, prefix.join("."))
            }
            Self::UnknownModule { module } => {
                write!(
                    f,
                    "unknown module `{}:{}`",
                    module.package.as_str(),
                    module.path.as_str()
                )
            }
            Self::UnavailableDependencySummary { module } => {
                write!(
                    f,
                    "dependency summary is unavailable for module `{}:{}`",
                    module.package.as_str(),
                    module.path.as_str()
                )
            }
        }
    }
}

impl Error for ModuleIndexProviderError {}

impl ModuleIndexProvider for ModuleIndex {
    fn packages(&self) -> &[PackageIndexEntry] {
        &self.packages
    }

    fn namespace_bindings(&self) -> &[NamespaceIndexEntry] {
        &self.namespace_bindings
    }

    fn package(&self, package: &PackageId) -> Result<&PackageIndexEntry, ModuleIndexProviderError> {
        self.packages
            .iter()
            .find(|entry| entry.package_id.as_str() == package.as_str())
            .ok_or_else(|| ModuleIndexProviderError::UnknownPackage {
                package_id: package.as_str().to_owned(),
            })
    }

    fn package_for_namespace(
        &self,
        root: &NamespaceRoot,
        prefix: &[String],
    ) -> Result<&PackageIndexEntry, ModuleIndexProviderError> {
        let binding = self
            .namespace_bindings
            .iter()
            .find(|binding| binding.root == *root && binding.prefix == prefix)
            .ok_or_else(|| ModuleIndexProviderError::UnknownNamespace {
                root: *root,
                prefix: prefix.to_vec(),
            })?;
        self.package(&binding.package_id)
    }

    fn module(&self, module: &ModuleId) -> Result<&ModuleIndexEntry, ModuleIndexProviderError> {
        self.modules
            .iter()
            .find(|entry| module_id_eq(&entry.module, module))
            .ok_or_else(|| ModuleIndexProviderError::UnknownModule {
                module: module.clone(),
            })
    }

    fn modules_for_package(
        &self,
        package: &PackageId,
    ) -> Result<&[ModuleIndexEntry], ModuleIndexProviderError> {
        self.package(package)?;
        let Some(start) = self
            .modules
            .iter()
            .position(|module| module.package_id.as_str() == package.as_str())
        else {
            return Ok(&self.modules[0..0]);
        };
        let end = self.modules[start..]
            .iter()
            .position(|module| module.package_id.as_str() != package.as_str())
            .map_or(self.modules.len(), |offset| start + offset);
        Ok(&self.modules[start..end])
    }

    fn dependency_summary(
        &self,
        module: &ModuleId,
    ) -> Result<&DependencyModuleSummaryRef, ModuleIndexProviderError> {
        if let Some(summary) = self
            .dependency_summaries
            .iter()
            .find(|summary| module_id_eq(&summary.module, module))
        {
            return Ok(summary);
        }
        if self
            .modules
            .iter()
            .any(|entry| module_id_eq(&entry.module, module))
        {
            Err(ModuleIndexProviderError::UnavailableDependencySummary {
                module: module.clone(),
            })
        } else {
            Err(ModuleIndexProviderError::UnknownModule {
                module: module.clone(),
            })
        }
    }
}

pub fn build_module_index(
    plan: &BuildPlan,
    source_layout: &dyn SourceLayoutProvider,
    dependency_artifacts: &[DependencyArtifactIndex],
) -> Result<ModuleIndex, ModuleIndexDiagnostics> {
    let mut diagnostics = Vec::new();
    let planned_packages = plan
        .packages
        .iter()
        .map(|package| package.package_id.as_str().to_owned())
        .collect::<BTreeSet<_>>();
    let artifacts_by_package =
        artifacts_by_package(dependency_artifacts, &planned_packages, &mut diagnostics);

    let packages = plan
        .packages
        .iter()
        .map(package_index_entry)
        .collect::<Vec<_>>();
    let namespace_bindings = namespace_bindings(plan, &artifacts_by_package, &mut diagnostics);
    let mut modules = workspace_modules(plan, source_layout, &mut diagnostics);
    let package_editions = plan
        .packages
        .iter()
        .map(|package| {
            (
                package.package_id.as_str().to_owned(),
                package.edition.clone(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let dependency_summaries = dependency_modules(
        &artifacts_by_package,
        &package_editions,
        &mut modules,
        &mut diagnostics,
    );
    validate_dependency_artifact_coverage(plan, &dependency_summaries, &mut diagnostics);

    modules.sort_by_key(module_entry_key);
    detect_duplicate_modules(&modules, &mut diagnostics);

    if !diagnostics.is_empty() {
        return Err(ModuleIndexDiagnostics::new(diagnostics));
    }

    Ok(ModuleIndex {
        packages,
        namespace_bindings,
        modules,
        dependency_summaries,
    })
}

fn package_index_entry(package: &PackagePlan) -> PackageIndexEntry {
    PackageIndexEntry {
        package_id: package.package_id.clone(),
        version: package.version.clone(),
        edition: package.edition.clone(),
        source: package_index_source(&package.source),
        dependencies: sorted_dependency_package_ids(&package.dependencies),
    }
}

fn package_index_source(source: &PackagePlanSource) -> PackageIndexSource {
    match source {
        PackagePlanSource::Workspace {
            root,
            source_root,
            manifest_path,
        } => PackageIndexSource::Workspace {
            package_root: root.clone(),
            source_root: source_root.clone(),
            manifest_path: manifest_path.clone(),
        },
        PackagePlanSource::Registry { registry, checksum } => {
            PackageIndexSource::RegistryArtifact {
                registry: registry.clone(),
                checksum: checksum.clone(),
            }
        }
    }
}

fn sorted_dependency_package_ids(dependencies: &[ResolvedPackageDependency]) -> Vec<PackageId> {
    dependencies
        .iter()
        .map(|dependency| dependency.package_id.as_str())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(PackageId::new)
        .collect()
}

fn artifacts_by_package<'a>(
    dependency_artifacts: &'a [DependencyArtifactIndex],
    planned_packages: &BTreeSet<String>,
    diagnostics: &mut Vec<ModuleIndexDiagnostic>,
) -> BTreeMap<String, Vec<&'a DependencyArtifactIndex>> {
    let mut artifacts = BTreeMap::<String, Vec<&DependencyArtifactIndex>>::new();
    for artifact in dependency_artifacts {
        let package_id = artifact.package_id.as_str().to_owned();
        if planned_packages.contains(&package_id) {
            artifacts.entry(package_id).or_default().push(artifact);
        } else {
            diagnostics.push(ModuleIndexDiagnostic::new(
                Some(package_id.clone()),
                None,
                None,
                ModuleIndexDiagnosticKind::UnknownPackage,
                Some(package_id),
            ));
        }
    }
    artifacts
}

fn namespace_bindings(
    plan: &BuildPlan,
    artifacts_by_package: &BTreeMap<String, Vec<&DependencyArtifactIndex>>,
    diagnostics: &mut Vec<ModuleIndexDiagnostic>,
) -> Vec<NamespaceIndexEntry> {
    let mut bindings = Vec::new();
    for package in &plan.packages {
        bindings.push(NamespaceIndexEntry {
            root: NamespaceRoot::PackageName,
            prefix: vec![package.package_id.as_str().to_owned()],
            package_id: package.package_id.clone(),
        });
        if let Some(artifacts) = artifacts_by_package.get(package.package_id.as_str()) {
            for artifact in artifacts {
                for binding in &artifact.namespace_bindings {
                    if binding.root == NamespaceRoot::PackageName {
                        diagnostics.push(ModuleIndexDiagnostic::new(
                            Some(artifact.package_id.as_str().to_owned()),
                            None,
                            None,
                            ModuleIndexDiagnosticKind::UnsupportedNamespaceRoot,
                            Some(format!("{:?}", binding.root)),
                        ));
                        continue;
                    }
                    if let Some(component) = invalid_namespace_prefix_component(&binding.prefix) {
                        diagnostics.push(ModuleIndexDiagnostic::new(
                            Some(artifact.package_id.as_str().to_owned()),
                            None,
                            None,
                            ModuleIndexDiagnosticKind::InvalidNamespacePrefix,
                            Some(component.to_owned()),
                        ));
                        continue;
                    }
                    bindings.push(NamespaceIndexEntry {
                        root: binding.root,
                        prefix: binding.prefix.clone(),
                        package_id: artifact.package_id.clone(),
                    });
                }
            }
        }
    }

    bindings.sort_by_key(namespace_key);
    let mut unique = Vec::new();
    let mut seen = BTreeMap::<(NamespaceRoot, Vec<String>), String>::new();
    for binding in bindings {
        let key = (binding.root, binding.prefix.clone());
        match seen.get(&key) {
            Some(package_id) if package_id != binding.package_id.as_str() => {
                diagnostics.push(ModuleIndexDiagnostic::new(
                    Some(binding.package_id.as_str().to_owned()),
                    None,
                    None,
                    ModuleIndexDiagnosticKind::DuplicateNamespaceBinding,
                    Some(format!("{:?}:{}", binding.root, binding.prefix.join("."))),
                ));
            }
            Some(_) => {}
            None => {
                seen.insert(key, binding.package_id.as_str().to_owned());
                unique.push(binding);
            }
        }
    }
    unique
}

fn workspace_modules(
    plan: &BuildPlan,
    source_layout: &dyn SourceLayoutProvider,
    diagnostics: &mut Vec<ModuleIndexDiagnostic>,
) -> Vec<ModuleIndexEntry> {
    let mut modules = Vec::new();
    for package in &plan.packages {
        let PackagePlanSource::Workspace { source_root, .. } = &package.source else {
            continue;
        };
        let files = match source_layout.workspace_source_files(package) {
            Ok(files) => files,
            Err(errors) => {
                diagnostics.extend(errors.into_diagnostics());
                continue;
            }
        };
        for file in files {
            if let Some(module_path) = validate_workspace_source_file(package, &file, diagnostics) {
                modules.push(ModuleIndexEntry {
                    module: ModuleId::new(package.package_id.clone(), module_path.clone()),
                    package_id: package.package_id.clone(),
                    module_path,
                    location: ModuleIndexLocation::WorkspaceFile {
                        source_root: source_root.clone(),
                        normalized_path: file.normalized_path,
                        source_relative_path: file.source_relative_path,
                    },
                    edition: package.edition.clone(),
                });
            }
        }
    }
    modules
}

fn dependency_modules(
    artifacts_by_package: &BTreeMap<String, Vec<&DependencyArtifactIndex>>,
    package_editions: &BTreeMap<String, Edition>,
    modules: &mut Vec<ModuleIndexEntry>,
    diagnostics: &mut Vec<ModuleIndexDiagnostic>,
) -> Vec<DependencyModuleSummaryRef> {
    let mut summaries = Vec::new();
    for (package_id, artifacts) in artifacts_by_package {
        for artifact in artifacts {
            for summary in &artifact.summaries {
                if summary.module.package.as_str() != package_id {
                    diagnostics.push(ModuleIndexDiagnostic::new(
                        Some(package_id.clone()),
                        Some(summary.module.clone()),
                        None,
                        ModuleIndexDiagnosticKind::MalformedSummaryIdentity,
                        Some(summary.module.package.as_str().to_owned()),
                    ));
                    continue;
                }
                if !is_valid_module_path(summary.module.path.as_str()) {
                    diagnostics.push(ModuleIndexDiagnostic::new(
                        Some(package_id.clone()),
                        Some(summary.module.clone()),
                        None,
                        ModuleIndexDiagnosticKind::MalformedSummaryIdentity,
                        Some(summary.module.path.as_str().to_owned()),
                    ));
                    continue;
                }
                summaries.push(summary.clone());
                modules.push(ModuleIndexEntry {
                    module: summary.module.clone(),
                    package_id: summary.module.package.clone(),
                    module_path: summary.module.path.clone(),
                    location: ModuleIndexLocation::DependencySummary {
                        artifact: summary.artifact.clone(),
                        content_hash: summary.content_hash,
                    },
                    edition: package_editions
                        .get(package_id)
                        .cloned()
                        .unwrap_or_else(|| Edition::new("")),
                });
            }
        }
    }
    summaries.sort_by_key(|summary| module_id_key(&summary.module));
    summaries
}

fn validate_dependency_artifact_coverage(
    plan: &BuildPlan,
    dependency_summaries: &[DependencyModuleSummaryRef],
    diagnostics: &mut Vec<ModuleIndexDiagnostic>,
) {
    for package in &plan.packages {
        if matches!(package.source, PackagePlanSource::Workspace { .. }) {
            continue;
        }
        if !dependency_summaries
            .iter()
            .any(|summary| summary.module.package.as_str() == package.package_id.as_str())
        {
            diagnostics.push(ModuleIndexDiagnostic::new(
                Some(package.package_id.as_str().to_owned()),
                None,
                None,
                ModuleIndexDiagnosticKind::MissingDependencySummary,
                Some(package.package_id.as_str().to_owned()),
            ));
        }
    }
}

fn validate_workspace_source_file(
    package: &PackagePlan,
    file: &WorkspaceSourceFile,
    diagnostics: &mut Vec<ModuleIndexDiagnostic>,
) -> Option<ModulePath> {
    if !is_valid_normalized_source_path(&file.normalized_path)
        || !is_valid_source_relative_path(&file.source_relative_path)
        || file.normalized_path != format!("src/{}", file.source_relative_path)
    {
        diagnostics.push(ModuleIndexDiagnostic::new(
            Some(package.package_id.as_str().to_owned()),
            None,
            Some(file.normalized_path.clone()),
            ModuleIndexDiagnosticKind::InvalidSourcePath,
            Some(file.source_relative_path.clone()),
        ));
        return None;
    }

    let Some(module_path_components) =
        module_path_components_from_source_relative_path(&file.source_relative_path)
    else {
        diagnostics.push(ModuleIndexDiagnostic::new(
            Some(package.package_id.as_str().to_owned()),
            None,
            Some(file.normalized_path.clone()),
            ModuleIndexDiagnosticKind::EmptyModulePath,
            Some(file.source_relative_path.clone()),
        ));
        return None;
    };
    let invalid_component = module_path_components
        .iter()
        .find(|component| !is_language_identifier(component));
    if let Some(component) = invalid_component {
        diagnostics.push(ModuleIndexDiagnostic::new(
            Some(package.package_id.as_str().to_owned()),
            None,
            Some(file.normalized_path.clone()),
            ModuleIndexDiagnosticKind::InvalidModuleComponent,
            Some((*component).to_owned()),
        ));
        return None;
    }

    Some(ModulePath::new(module_path_components.join(".")))
}

fn module_path_components_from_source_relative_path(
    source_relative_path: &str,
) -> Option<Vec<&str>> {
    let stem = source_relative_path.strip_suffix(".miz")?;
    if stem.is_empty() {
        return None;
    }
    Some(stem.split('/').collect())
}

fn is_valid_normalized_source_path(path: &str) -> bool {
    path.starts_with("src/") && path.ends_with(".miz") && is_normalized_relative_path(path)
}

fn is_valid_source_relative_path(path: &str) -> bool {
    !path.starts_with('/') && path.ends_with(".miz") && is_normalized_relative_path(path)
}

fn is_valid_module_path(module_path: &str) -> bool {
    !module_path.is_empty()
        && module_path
            .split('.')
            .all(|component| !component.is_empty() && is_language_identifier(component))
}

fn invalid_namespace_prefix_component(prefix: &[String]) -> Option<&str> {
    prefix
        .iter()
        .find(|component| !is_language_identifier(component))
        .map(String::as_str)
}

fn is_normalized_relative_path(path: &str) -> bool {
    !path.is_empty()
        && !path.contains('\\')
        && !path.contains("//")
        && !path
            .split('/')
            .any(|component| component.is_empty() || component == "." || component == "..")
}

fn is_language_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '\'')
        && !is_reserved_word(value)
}

fn is_reserved_word(value: &str) -> bool {
    RESERVED_WORDS.contains(&value)
}

fn detect_duplicate_modules(
    modules: &[ModuleIndexEntry],
    diagnostics: &mut Vec<ModuleIndexDiagnostic>,
) {
    let mut seen = BTreeSet::<(String, String)>::new();
    for module in modules {
        let key = (
            module.package_id.as_str().to_owned(),
            module.module_path.as_str().to_owned(),
        );
        if !seen.insert(key) {
            diagnostics.push(ModuleIndexDiagnostic::new(
                Some(module.package_id.as_str().to_owned()),
                Some(module.module.clone()),
                normalized_path_for_diagnostic(&module.location),
                ModuleIndexDiagnosticKind::DuplicateModule,
                Some(location_key(&module.location).join(":")),
            ));
        }
    }
}

fn normalized_path_for_diagnostic(location: &ModuleIndexLocation) -> Option<String> {
    match location {
        ModuleIndexLocation::WorkspaceFile {
            normalized_path, ..
        } => Some(normalized_path.clone()),
        ModuleIndexLocation::DependencySummary { .. } => None,
    }
}

fn module_id_eq(left: &ModuleId, right: &ModuleId) -> bool {
    left.package.as_str() == right.package.as_str() && left.path.as_str() == right.path.as_str()
}

fn namespace_key(entry: &NamespaceIndexEntry) -> (NamespaceRoot, Vec<String>, String) {
    (
        entry.root,
        entry.prefix.clone(),
        entry.package_id.as_str().to_owned(),
    )
}

fn module_entry_key(entry: &ModuleIndexEntry) -> (String, String, Vec<String>) {
    (
        entry.package_id.as_str().to_owned(),
        entry.module_path.as_str().to_owned(),
        location_key(&entry.location),
    )
}

fn module_id_key(module: &ModuleId) -> (String, String) {
    (
        module.package.as_str().to_owned(),
        module.path.as_str().to_owned(),
    )
}

fn location_key(location: &ModuleIndexLocation) -> Vec<String> {
    match location {
        ModuleIndexLocation::WorkspaceFile {
            normalized_path, ..
        } => vec!["0".to_owned(), normalized_path.clone()],
        ModuleIndexLocation::DependencySummary {
            artifact,
            content_hash,
        } => vec!["1".to_owned(), artifact.clone(), hash_hex(content_hash)],
    }
}

fn hash_hex(hash: &Hash) -> String {
    let mut encoded = String::with_capacity(Hash::BYTE_LEN * 2);
    for byte in hash.as_bytes() {
        encoded.push_str(&format!("{byte:02x}"));
    }
    encoded
}

fn sort_diagnostics(diagnostics: &mut [ModuleIndexDiagnostic]) {
    diagnostics.sort_by_key(diagnostic_key);
}

fn diagnostic_key(diagnostic: &ModuleIndexDiagnostic) -> (String, String, String, u8, String) {
    (
        diagnostic.package_id.clone().unwrap_or_default(),
        diagnostic
            .module
            .as_ref()
            .map(|module| module.path.as_str().to_owned())
            .unwrap_or_default(),
        diagnostic.normalized_path.clone().unwrap_or_default(),
        diagnostic_rank(&diagnostic.kind),
        diagnostic.value.clone().unwrap_or_default(),
    )
}

fn diagnostic_rank(kind: &ModuleIndexDiagnosticKind) -> u8 {
    match kind {
        ModuleIndexDiagnosticKind::MissingSourceLayout => 0,
        ModuleIndexDiagnosticKind::InvalidSourcePath => 1,
        ModuleIndexDiagnosticKind::InvalidModuleComponent => 2,
        ModuleIndexDiagnosticKind::EmptyModulePath => 3,
        ModuleIndexDiagnosticKind::DuplicateModule => 4,
        ModuleIndexDiagnosticKind::DuplicateNamespaceBinding => 5,
        ModuleIndexDiagnosticKind::UnsupportedNamespaceRoot => 6,
        ModuleIndexDiagnosticKind::InvalidNamespacePrefix => 7,
        ModuleIndexDiagnosticKind::UnknownPackage => 8,
        ModuleIndexDiagnosticKind::MissingDependencySummary => 9,
        ModuleIndexDiagnosticKind::MalformedSummaryIdentity => 10,
    }
}

const RESERVED_WORDS: &[&str] = &[
    "algorithm",
    "and",
    "antonym",
    "as",
    "assert",
    "assume",
    "assumed",
    "asymmetry",
    "attr",
    "be",
    "being",
    "break",
    "by",
    "case",
    "cases",
    "claim",
    "cluster",
    "coherence",
    "commutativity",
    "compatibility",
    "computation",
    "conditional",
    "connectedness",
    "const",
    "consider",
    "consistency",
    "continue",
    "contradiction",
    "decreasing",
    "deffunc",
    "definition",
    "defpred",
    "do",
    "does",
    "downto",
    "else",
    "end",
    "ensures",
    "equals",
    "ex",
    "expr",
    "existence",
    "export",
    "extends",
    "fail",
    "field",
    "for",
    "from",
    "func",
    "ghost",
    "given",
    "hence",
    "hereby",
    "holds",
    "idempotence",
    "if",
    "iff",
    "implies",
    "import",
    "in",
    "infix_operator",
    "inherit",
    "invariant",
    "involutiveness",
    "irreflexivity",
    "is",
    "it",
    "left",
    "lemma",
    "let",
    "match",
    "means",
    "mode",
    "nest",
    "non",
    "none",
    "not",
    "now",
    "object",
    "of",
    "open",
    "or",
    "otherwise",
    "over",
    "per",
    "postfix_operator",
    "pred",
    "prefix_operator",
    "private",
    "processed",
    "projectivity",
    "proof",
    "property",
    "public",
    "qua",
    "reconsider",
    "reduce",
    "reducibility",
    "redefine",
    "reflexivity",
    "registration",
    "requires",
    "reserve",
    "return",
    "right",
    "set",
    "sethood",
    "snapshot",
    "st",
    "step",
    "struct",
    "such",
    "suppose",
    "symmetry",
    "synonym",
    "take",
    "terminating",
    "that",
    "the",
    "then",
    "theorem",
    "thesis",
    "thus",
    "to",
    "transitivity",
    "type",
    "uniqueness",
    "var",
    "where",
    "while",
    "with",
];

#[cfg(test)]
mod tests;
