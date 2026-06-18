use std::{collections::BTreeSet, error::Error, fmt};

use mizar_session::{Edition, PackageId};
use semver::Version;
use toml::{Table, Value};

const CURRENT_STABLE_EDITION: &str = "2025";
const PACKAGE_ID_PATTERN: &str = "[a-z][a-z0-9]*(?:_[a-z0-9]+)*";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageManifest {
    pub name: String,
    pub version: Version,
    pub authors: Vec<String>,
    pub license: String,
    pub description: String,
    pub edition: Edition,
    pub dependencies: Vec<ManifestDependency>,
    pub dev_dependencies: Vec<ManifestDependency>,
    pub verifier: VerifierConfig,
    pub build: BuildConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceManifest {
    pub members: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lockfile {
    pub schema_version: u64,
    pub packages: Vec<LockedPackage>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockedPackage {
    pub name: String,
    pub version: Version,
    pub source: LockSource,
    pub dependencies: Vec<LockedDependency>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockedDependency {
    pub name: String,
    pub version: Version,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LockSource {
    Workspace { path: String },
    Registry { registry: String, checksum: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestDependency {
    pub package_id: String,
    pub version: VersionConstraint,
    pub kind: DependencyKind,
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DependencyKind {
    Normal,
    Dev,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionConstraint {
    Exact(Version),
    Caret(Version),
    Tilde(Version),
    Range(Vec<VersionComparator>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionComparator {
    pub op: VersionComparison,
    pub version: Version,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionComparison {
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifierConfig {
    pub max_axioms: u32,
    pub atp_timeout: u32,
    pub default_solver: Solver,
    pub require_kernel_certificates: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Solver {
    Auto,
    Vampire,
    E,
    Cvc5,
    Z3,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildConfig {
    pub incremental: bool,
    pub cache_dir: String,
    pub artifact_dir: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedPackageManifest {
    pub package_id: PackageId,
    pub manifest: PackageManifest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestDiagnostics {
    diagnostics: Vec<ManifestDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestDiagnostic {
    pub location: DiagnosticLocation,
    pub kind: ManifestDiagnosticKind,
    pub value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticLocation {
    pub path: String,
    pub key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ManifestDiagnosticKind {
    InvalidToml,
    MissingField,
    UnknownField,
    InvalidType,
    InvalidPackageId { expected: &'static str },
    InvalidVersion,
    InvalidVersionConstraint,
    InvalidWorkspaceMemberPath,
    DuplicateWorkspaceMember,
    InvalidBuildPath,
    InvalidSolver,
    InvalidLockfileSchema,
    DuplicateLockPackage,
    MissingLockPackage,
    LockVersionMismatch,
    UnknownLockedDependency,
    InvalidLockSource,
    InvalidDependencyVersion,
    DuplicateFeature,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ManifestValidationError {
    InvalidPackageId {
        package_id: String,
        expected: &'static str,
    },
}

impl PackageManifest {
    #[must_use]
    pub fn package_id(&self) -> PackageId {
        PackageId::new(self.name.clone())
    }
}

impl ManifestDiagnostics {
    #[must_use]
    pub fn new(mut diagnostics: Vec<ManifestDiagnostic>) -> Self {
        sort_diagnostics(&mut diagnostics);
        Self { diagnostics }
    }

    #[must_use]
    pub fn diagnostics(&self) -> &[ManifestDiagnostic] {
        &self.diagnostics
    }

    #[must_use]
    pub fn into_diagnostics(self) -> Vec<ManifestDiagnostic> {
        self.diagnostics
    }
}

impl ManifestDiagnostic {
    fn new(
        path: impl Into<String>,
        key: impl Into<String>,
        kind: ManifestDiagnosticKind,
        value: Option<String>,
    ) -> Self {
        Self {
            location: DiagnosticLocation {
                path: path.into(),
                key: key.into(),
            },
            kind,
            value,
        }
    }
}

impl Default for VerifierConfig {
    fn default() -> Self {
        Self {
            max_axioms: 128,
            atp_timeout: 30,
            default_solver: Solver::Auto,
            require_kernel_certificates: true,
        }
    }
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            incremental: true,
            cache_dir: ".mizar-cache".to_owned(),
            artifact_dir: "build".to_owned(),
        }
    }
}

pub fn parse_package_manifest(input: &str) -> Result<PackageManifest, ManifestDiagnostics> {
    let document = parse_toml_document("<package manifest>", input)?;
    let mut diagnostics = Vec::new();
    collect_unknown_fields(
        "<package manifest>",
        "",
        &document,
        &[
            "package",
            "dependencies",
            "dev-dependencies",
            "verifier",
            "build",
        ],
        &mut diagnostics,
    );

    let Some(package_table) =
        required_table("<package manifest>", &document, "package", &mut diagnostics)
    else {
        return Err(ManifestDiagnostics::new(diagnostics));
    };

    let manifest = parse_package_manifest_from_tables(package_table, &document, &mut diagnostics);
    finish_with_diagnostics(manifest, diagnostics)
}

pub fn parse_workspace_manifest(input: &str) -> Result<WorkspaceManifest, ManifestDiagnostics> {
    let document = parse_toml_document("<workspace manifest>", input)?;
    let mut diagnostics = Vec::new();
    collect_unknown_fields(
        "<workspace manifest>",
        "",
        &document,
        &["workspace"],
        &mut diagnostics,
    );

    let Some(workspace_table) = required_table(
        "<workspace manifest>",
        &document,
        "workspace",
        &mut diagnostics,
    ) else {
        return Err(ManifestDiagnostics::new(diagnostics));
    };

    collect_unknown_fields(
        "<workspace manifest>",
        "workspace",
        workspace_table,
        &["members"],
        &mut diagnostics,
    );

    let members = read_string_array(
        "<workspace manifest>",
        workspace_table,
        "workspace.members",
        &mut diagnostics,
    )
    .unwrap_or_default();
    let members = validate_workspace_members(members, &mut diagnostics);

    finish_with_diagnostics(WorkspaceManifest { members }, diagnostics)
}

pub fn parse_lockfile(input: &str) -> Result<Lockfile, ManifestDiagnostics> {
    let document = parse_toml_document("<lockfile>", input)?;
    let mut diagnostics = Vec::new();
    collect_unknown_fields(
        "<lockfile>",
        "",
        &document,
        &["schema_version", "package"],
        &mut diagnostics,
    );

    let schema_version =
        read_u64("<lockfile>", &document, "schema_version", &mut diagnostics).unwrap_or_default();
    if schema_version != 1 {
        diagnostics.push(ManifestDiagnostic::new(
            "<lockfile>",
            "schema_version",
            ManifestDiagnosticKind::InvalidLockfileSchema,
            Some(schema_version.to_string()),
        ));
    }

    let mut packages = parse_locked_packages(&document, &mut diagnostics);
    packages.sort_by(|left, right| left.name.cmp(&right.name));
    validate_locked_package_uniqueness(&packages, &mut diagnostics);
    validate_locked_dependency_targets(&packages, &mut diagnostics);

    finish_with_diagnostics(
        Lockfile {
            schema_version,
            packages,
        },
        diagnostics,
    )
}

pub fn validate_package_manifest(
    manifest: &PackageManifest,
) -> Result<ValidatedPackageManifest, ManifestValidationError> {
    validate_package_id_spelling(&manifest.name)?;
    Ok(ValidatedPackageManifest {
        package_id: manifest.package_id(),
        manifest: manifest.clone(),
    })
}

pub fn validate_package_id_spelling(package_id: &str) -> Result<(), ManifestValidationError> {
    if is_lowercase_snake_case_package_id(package_id) {
        Ok(())
    } else {
        Err(ManifestValidationError::InvalidPackageId {
            package_id: package_id.to_owned(),
            expected: PACKAGE_ID_PATTERN,
        })
    }
}

pub fn validate_lockfile_for_workspace(
    manifests: &[PackageManifest],
    lockfile: &Lockfile,
) -> Result<(), ManifestDiagnostics> {
    let mut diagnostics = Vec::new();
    for manifest in manifests {
        match lockfile
            .packages
            .iter()
            .find(|locked| locked.name == manifest.name)
        {
            Some(locked) if locked.version == manifest.version => {}
            Some(locked) => diagnostics.push(ManifestDiagnostic::new(
                "<lockfile>",
                manifest.name.clone(),
                ManifestDiagnosticKind::LockVersionMismatch,
                Some(format!("{} != {}", locked.version, manifest.version)),
            )),
            None => diagnostics.push(ManifestDiagnostic::new(
                "<lockfile>",
                manifest.name.clone(),
                ManifestDiagnosticKind::MissingLockPackage,
                Some(manifest.version.to_string()),
            )),
        }

        if let Some(locked) = lockfile
            .packages
            .iter()
            .find(|locked| locked.name == manifest.name)
            && !matches!(locked.source, LockSource::Workspace { .. })
        {
            diagnostics.push(ManifestDiagnostic::new(
                "<lockfile>",
                format!("{}.source.kind", manifest.name),
                ManifestDiagnosticKind::InvalidLockSource,
                Some("registry".to_owned()),
            ));
        }

        for dependency in manifest
            .dependencies
            .iter()
            .chain(manifest.dev_dependencies.iter())
        {
            validate_manifest_dependency_against_lockfile(
                manifest,
                dependency,
                lockfile,
                &mut diagnostics,
            );
        }
    }

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(ManifestDiagnostics::new(diagnostics))
    }
}

#[must_use]
pub fn is_lowercase_snake_case_package_id(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_lowercase() {
        return false;
    }

    let mut previous_was_underscore = false;
    for ch in chars {
        match ch {
            'a'..='z' | '0'..='9' => previous_was_underscore = false,
            '_' if !previous_was_underscore => previous_was_underscore = true,
            _ => return false,
        }
    }

    !previous_was_underscore
}

fn parse_package_manifest_from_tables(
    package_table: &Table,
    document: &Table,
    diagnostics: &mut Vec<ManifestDiagnostic>,
) -> PackageManifest {
    collect_unknown_fields(
        "<package manifest>",
        "package",
        package_table,
        &[
            "name",
            "version",
            "authors",
            "license",
            "description",
            "edition",
        ],
        diagnostics,
    );

    let name = read_string(
        "<package manifest>",
        package_table,
        "package.name",
        diagnostics,
    )
    .unwrap_or_default();
    if !name.is_empty() && !is_lowercase_snake_case_package_id(&name) {
        diagnostics.push(ManifestDiagnostic::new(
            "<package manifest>",
            "package.name",
            ManifestDiagnosticKind::InvalidPackageId {
                expected: PACKAGE_ID_PATTERN,
            },
            Some(name.clone()),
        ));
    }

    let version_text = read_string(
        "<package manifest>",
        package_table,
        "package.version",
        diagnostics,
    )
    .unwrap_or_default();
    let version = parse_version_or_default(
        "<package manifest>",
        "package.version",
        &version_text,
        diagnostics,
    );

    let authors = read_optional_string_array(
        "<package manifest>",
        package_table,
        "package.authors",
        diagnostics,
    )
    .unwrap_or_default();
    let license = read_optional_string(
        "<package manifest>",
        package_table,
        "package.license",
        diagnostics,
    )
    .unwrap_or_default();
    let description = read_optional_string(
        "<package manifest>",
        package_table,
        "package.description",
        diagnostics,
    )
    .unwrap_or_default();
    let edition = read_optional_string(
        "<package manifest>",
        package_table,
        "package.edition",
        diagnostics,
    )
    .unwrap_or_else(|| CURRENT_STABLE_EDITION.to_owned());

    let dependencies = parse_dependency_table(
        document,
        "dependencies",
        DependencyKind::Normal,
        diagnostics,
    );
    let dev_dependencies = parse_dependency_table(
        document,
        "dev-dependencies",
        DependencyKind::Dev,
        diagnostics,
    );
    let verifier = parse_verifier_config(document, diagnostics);
    let build = parse_build_config(document, diagnostics);

    PackageManifest {
        name,
        version,
        authors,
        license,
        description,
        edition: Edition::new(edition),
        dependencies,
        dev_dependencies,
        verifier,
        build,
    }
}

fn parse_dependency_table(
    document: &Table,
    table_name: &str,
    kind: DependencyKind,
    diagnostics: &mut Vec<ManifestDiagnostic>,
) -> Vec<ManifestDependency> {
    let Some(value) = document.get(table_name) else {
        return Vec::new();
    };
    let Some(table) = value.as_table() else {
        diagnostics.push(ManifestDiagnostic::new(
            "<package manifest>",
            table_name,
            ManifestDiagnosticKind::InvalidType,
            Some(value.type_str().to_owned()),
        ));
        return Vec::new();
    };

    let mut dependencies = Vec::new();
    for (package_id, dependency_value) in table {
        if !is_lowercase_snake_case_package_id(package_id) {
            diagnostics.push(ManifestDiagnostic::new(
                "<package manifest>",
                format!("{table_name}.{package_id}"),
                ManifestDiagnosticKind::InvalidPackageId {
                    expected: PACKAGE_ID_PATTERN,
                },
                Some(package_id.clone()),
            ));
            continue;
        }

        match dependency_value {
            Value::String(version_text) => {
                if let Some(version) = parse_version_constraint(
                    "<package manifest>",
                    &format!("{table_name}.{package_id}"),
                    version_text,
                    diagnostics,
                ) {
                    dependencies.push(ManifestDependency {
                        package_id: package_id.clone(),
                        version,
                        kind,
                        features: Vec::new(),
                    });
                }
            }
            Value::Table(inline) => {
                collect_unknown_fields(
                    "<package manifest>",
                    &format!("{table_name}.{package_id}"),
                    inline,
                    &["version", "features"],
                    diagnostics,
                );
                let Some(version_text) = read_string(
                    "<package manifest>",
                    inline,
                    &format!("{table_name}.{package_id}.version"),
                    diagnostics,
                ) else {
                    continue;
                };
                let Some(version) = parse_version_constraint(
                    "<package manifest>",
                    &format!("{table_name}.{package_id}.version"),
                    &version_text,
                    diagnostics,
                ) else {
                    continue;
                };
                let features = read_optional_string_array(
                    "<package manifest>",
                    inline,
                    &format!("{table_name}.{package_id}.features"),
                    diagnostics,
                )
                .unwrap_or_default();
                let features = validate_features(
                    &format!("{table_name}.{package_id}.features"),
                    features,
                    diagnostics,
                );
                dependencies.push(ManifestDependency {
                    package_id: package_id.clone(),
                    version,
                    kind,
                    features,
                });
            }
            _ => diagnostics.push(ManifestDiagnostic::new(
                "<package manifest>",
                format!("{table_name}.{package_id}"),
                ManifestDiagnosticKind::InvalidType,
                Some(dependency_value.type_str().to_owned()),
            )),
        }
    }

    dependencies.sort_by(|left, right| {
        left.kind
            .cmp(&right.kind)
            .then_with(|| left.package_id.cmp(&right.package_id))
            .then_with(|| left.features.cmp(&right.features))
    });
    dependencies
}

fn parse_verifier_config(
    document: &Table,
    diagnostics: &mut Vec<ManifestDiagnostic>,
) -> VerifierConfig {
    let Some(value) = document.get("verifier") else {
        return VerifierConfig::default();
    };
    let Some(table) = value.as_table() else {
        diagnostics.push(ManifestDiagnostic::new(
            "<package manifest>",
            "verifier",
            ManifestDiagnosticKind::InvalidType,
            Some(value.type_str().to_owned()),
        ));
        return VerifierConfig::default();
    };
    collect_unknown_fields(
        "<package manifest>",
        "verifier",
        table,
        &[
            "max_axioms",
            "atp_timeout",
            "default_solver",
            "require_kernel_certificates",
        ],
        diagnostics,
    );

    let default = VerifierConfig::default();
    VerifierConfig {
        max_axioms: read_optional_u32(
            "<package manifest>",
            table,
            "verifier.max_axioms",
            diagnostics,
        )
        .unwrap_or(default.max_axioms),
        atp_timeout: read_optional_u32(
            "<package manifest>",
            table,
            "verifier.atp_timeout",
            diagnostics,
        )
        .unwrap_or(default.atp_timeout),
        default_solver: read_solver(table, diagnostics).unwrap_or(default.default_solver),
        require_kernel_certificates: read_optional_bool(
            "<package manifest>",
            table,
            "verifier.require_kernel_certificates",
            diagnostics,
        )
        .unwrap_or(default.require_kernel_certificates),
    }
}

fn parse_build_config(document: &Table, diagnostics: &mut Vec<ManifestDiagnostic>) -> BuildConfig {
    let Some(value) = document.get("build") else {
        return BuildConfig::default();
    };
    let Some(table) = value.as_table() else {
        diagnostics.push(ManifestDiagnostic::new(
            "<package manifest>",
            "build",
            ManifestDiagnosticKind::InvalidType,
            Some(value.type_str().to_owned()),
        ));
        return BuildConfig::default();
    };
    collect_unknown_fields(
        "<package manifest>",
        "build",
        table,
        &["incremental", "cache_dir", "artifact_dir"],
        diagnostics,
    );

    let default = BuildConfig::default();
    let cache_dir =
        read_optional_string("<package manifest>", table, "build.cache_dir", diagnostics)
            .unwrap_or(default.cache_dir);
    let artifact_dir = read_optional_string(
        "<package manifest>",
        table,
        "build.artifact_dir",
        diagnostics,
    )
    .unwrap_or(default.artifact_dir);
    validate_relative_path(
        "<package manifest>",
        "build.cache_dir",
        &cache_dir,
        ManifestDiagnosticKind::InvalidBuildPath,
        diagnostics,
    );
    validate_relative_path(
        "<package manifest>",
        "build.artifact_dir",
        &artifact_dir,
        ManifestDiagnosticKind::InvalidBuildPath,
        diagnostics,
    );

    BuildConfig {
        incremental: read_optional_bool(
            "<package manifest>",
            table,
            "build.incremental",
            diagnostics,
        )
        .unwrap_or(default.incremental),
        cache_dir,
        artifact_dir,
    }
}

fn parse_locked_packages(
    document: &Table,
    diagnostics: &mut Vec<ManifestDiagnostic>,
) -> Vec<LockedPackage> {
    let Some(value) = document.get("package") else {
        diagnostics.push(ManifestDiagnostic::new(
            "<lockfile>",
            "package",
            ManifestDiagnosticKind::MissingField,
            None,
        ));
        return Vec::new();
    };
    let Some(package_entries) = value.as_array() else {
        diagnostics.push(ManifestDiagnostic::new(
            "<lockfile>",
            "package",
            ManifestDiagnosticKind::InvalidType,
            Some(value.type_str().to_owned()),
        ));
        return Vec::new();
    };

    let mut packages = Vec::new();
    for (index, entry) in package_entries.iter().enumerate() {
        let key_prefix = format!("package[{index}]");
        let Some(table) = entry.as_table() else {
            diagnostics.push(ManifestDiagnostic::new(
                "<lockfile>",
                key_prefix,
                ManifestDiagnosticKind::InvalidType,
                Some(entry.type_str().to_owned()),
            ));
            continue;
        };
        collect_unknown_fields(
            "<lockfile>",
            &key_prefix,
            table,
            &["name", "version", "source", "dependencies"],
            diagnostics,
        );

        let name = read_string(
            "<lockfile>",
            table,
            &format!("{key_prefix}.name"),
            diagnostics,
        )
        .unwrap_or_default();
        if !name.is_empty() && !is_lowercase_snake_case_package_id(&name) {
            diagnostics.push(ManifestDiagnostic::new(
                "<lockfile>",
                format!("{key_prefix}.name"),
                ManifestDiagnosticKind::InvalidPackageId {
                    expected: PACKAGE_ID_PATTERN,
                },
                Some(name.clone()),
            ));
        }
        let version_text = read_string(
            "<lockfile>",
            table,
            &format!("{key_prefix}.version"),
            diagnostics,
        )
        .unwrap_or_default();
        let version = parse_version_or_default(
            "<lockfile>",
            &format!("{key_prefix}.version"),
            &version_text,
            diagnostics,
        );
        let source = parse_lock_source(table, &key_prefix, diagnostics);
        let dependencies = parse_locked_dependencies(table, &key_prefix, diagnostics);
        packages.push(LockedPackage {
            name,
            version,
            source,
            dependencies,
        });
    }

    packages.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| left.version.cmp(&right.version))
            .then_with(|| {
                lock_source_sort_key(&left.source).cmp(&lock_source_sort_key(&right.source))
            })
    });
    packages
}

fn parse_lock_source(
    table: &Table,
    key_prefix: &str,
    diagnostics: &mut Vec<ManifestDiagnostic>,
) -> LockSource {
    let Some(value) = table.get("source") else {
        diagnostics.push(ManifestDiagnostic::new(
            "<lockfile>",
            format!("{key_prefix}.source"),
            ManifestDiagnosticKind::MissingField,
            None,
        ));
        return LockSource::Workspace {
            path: String::new(),
        };
    };
    let Some(source_table) = value.as_table() else {
        diagnostics.push(ManifestDiagnostic::new(
            "<lockfile>",
            format!("{key_prefix}.source"),
            ManifestDiagnosticKind::InvalidType,
            Some(value.type_str().to_owned()),
        ));
        return LockSource::Workspace {
            path: String::new(),
        };
    };

    collect_unknown_fields(
        "<lockfile>",
        &format!("{key_prefix}.source"),
        source_table,
        &["kind", "path", "registry", "checksum"],
        diagnostics,
    );
    let kind = read_string(
        "<lockfile>",
        source_table,
        &format!("{key_prefix}.source.kind"),
        diagnostics,
    )
    .unwrap_or_default();

    match kind.as_str() {
        "workspace" => {
            let path = read_string(
                "<lockfile>",
                source_table,
                &format!("{key_prefix}.source.path"),
                diagnostics,
            )
            .unwrap_or_default();
            validate_relative_path_allow_root(
                "<lockfile>",
                &format!("{key_prefix}.source.path"),
                &path,
                ManifestDiagnosticKind::InvalidWorkspaceMemberPath,
                diagnostics,
            );
            LockSource::Workspace { path }
        }
        "registry" => {
            let registry = read_string(
                "<lockfile>",
                source_table,
                &format!("{key_prefix}.source.registry"),
                diagnostics,
            )
            .unwrap_or_default();
            let checksum = read_string(
                "<lockfile>",
                source_table,
                &format!("{key_prefix}.source.checksum"),
                diagnostics,
            )
            .unwrap_or_default();
            LockSource::Registry { registry, checksum }
        }
        _ => {
            diagnostics.push(ManifestDiagnostic::new(
                "<lockfile>",
                format!("{key_prefix}.source.kind"),
                ManifestDiagnosticKind::InvalidLockSource,
                Some(kind),
            ));
            LockSource::Workspace {
                path: String::new(),
            }
        }
    }
}

fn parse_locked_dependencies(
    table: &Table,
    key_prefix: &str,
    diagnostics: &mut Vec<ManifestDiagnostic>,
) -> Vec<LockedDependency> {
    let Some(value) = table.get("dependencies") else {
        diagnostics.push(ManifestDiagnostic::new(
            "<lockfile>",
            format!("{key_prefix}.dependencies"),
            ManifestDiagnosticKind::MissingField,
            None,
        ));
        return Vec::new();
    };
    let Some(entries) = value.as_array() else {
        diagnostics.push(ManifestDiagnostic::new(
            "<lockfile>",
            format!("{key_prefix}.dependencies"),
            ManifestDiagnosticKind::InvalidType,
            Some(value.type_str().to_owned()),
        ));
        return Vec::new();
    };

    let mut dependencies = Vec::new();
    for (index, dependency) in entries.iter().enumerate() {
        let dependency_key = format!("{key_prefix}.dependencies[{index}]");
        let Some(dependency_table) = dependency.as_table() else {
            diagnostics.push(ManifestDiagnostic::new(
                "<lockfile>",
                dependency_key,
                ManifestDiagnosticKind::InvalidType,
                Some(dependency.type_str().to_owned()),
            ));
            continue;
        };
        collect_unknown_fields(
            "<lockfile>",
            &dependency_key,
            dependency_table,
            &["name", "version"],
            diagnostics,
        );
        let name = read_string(
            "<lockfile>",
            dependency_table,
            &format!("{dependency_key}.name"),
            diagnostics,
        )
        .unwrap_or_default();
        if !name.is_empty() && !is_lowercase_snake_case_package_id(&name) {
            diagnostics.push(ManifestDiagnostic::new(
                "<lockfile>",
                format!("{dependency_key}.name"),
                ManifestDiagnosticKind::InvalidPackageId {
                    expected: PACKAGE_ID_PATTERN,
                },
                Some(name.clone()),
            ));
        }
        let version_text = read_string(
            "<lockfile>",
            dependency_table,
            &format!("{dependency_key}.version"),
            diagnostics,
        )
        .unwrap_or_default();
        let version = parse_version_or_default(
            "<lockfile>",
            &format!("{dependency_key}.version"),
            &version_text,
            diagnostics,
        );
        dependencies.push(LockedDependency { name, version });
    }
    dependencies.sort_by(|left, right| left.name.cmp(&right.name));
    dependencies
}

fn validate_locked_package_uniqueness(
    packages: &[LockedPackage],
    diagnostics: &mut Vec<ManifestDiagnostic>,
) {
    let mut seen = BTreeSet::new();
    for package in packages {
        if packages
            .iter()
            .filter(|candidate| candidate.name == package.name)
            .count()
            > 1
            && seen.insert((
                package.name.clone(),
                package.version.clone(),
                lock_source_sort_key(&package.source),
            ))
        {
            diagnostics.push(ManifestDiagnostic::new(
                "<lockfile>",
                package.name.clone(),
                ManifestDiagnosticKind::DuplicateLockPackage,
                Some(package.version.to_string()),
            ));
        }
    }
}

fn validate_locked_dependency_targets(
    packages: &[LockedPackage],
    diagnostics: &mut Vec<ManifestDiagnostic>,
) {
    let locked_names = packages
        .iter()
        .map(|package| package.name.as_str())
        .collect::<BTreeSet<_>>();
    for package in packages {
        for dependency in &package.dependencies {
            let locked_dependency = packages
                .iter()
                .find(|locked| locked.name == dependency.name);
            if !locked_names.contains(dependency.name.as_str()) {
                diagnostics.push(ManifestDiagnostic::new(
                    "<lockfile>",
                    format!("{}.dependencies.{}", package.name, dependency.name),
                    ManifestDiagnosticKind::UnknownLockedDependency,
                    Some(dependency.version.to_string()),
                ));
            } else if locked_dependency.is_some_and(|locked| locked.version != dependency.version) {
                diagnostics.push(ManifestDiagnostic::new(
                    "<lockfile>",
                    format!("{}.dependencies.{}", package.name, dependency.name),
                    ManifestDiagnosticKind::LockVersionMismatch,
                    Some(dependency.version.to_string()),
                ));
            }
        }
    }
}

fn validate_workspace_members(
    members: Vec<String>,
    diagnostics: &mut Vec<ManifestDiagnostic>,
) -> Vec<String> {
    let mut accepted = Vec::new();
    let mut seen = BTreeSet::new();

    for member in members {
        let valid = member == "." || relative_path_is_valid(&member);
        if !valid {
            diagnostics.push(ManifestDiagnostic::new(
                "<workspace manifest>",
                "workspace.members",
                ManifestDiagnosticKind::InvalidWorkspaceMemberPath,
                Some(member.clone()),
            ));
            continue;
        }
        if !seen.insert(member.clone()) {
            diagnostics.push(ManifestDiagnostic::new(
                "<workspace manifest>",
                "workspace.members",
                ManifestDiagnosticKind::DuplicateWorkspaceMember,
                Some(member.clone()),
            ));
            continue;
        }
        accepted.push(member);
    }

    accepted.sort();
    accepted
}

fn validate_features(
    key: &str,
    mut features: Vec<String>,
    diagnostics: &mut Vec<ManifestDiagnostic>,
) -> Vec<String> {
    features.sort();
    let mut previous: Option<&str> = None;
    for feature in &features {
        if previous == Some(feature.as_str()) {
            diagnostics.push(ManifestDiagnostic::new(
                "<package manifest>",
                key,
                ManifestDiagnosticKind::DuplicateFeature,
                Some(feature.clone()),
            ));
        }
        previous = Some(feature);
    }
    features.dedup();
    features
}

fn parse_version_constraint(
    path: &str,
    key: &str,
    value: &str,
    diagnostics: &mut Vec<ManifestDiagnostic>,
) -> Option<VersionConstraint> {
    let value = value.trim();
    if let Some(version) = value.strip_prefix('^') {
        return parse_version(
            path,
            key,
            version,
            ManifestDiagnosticKind::InvalidVersionConstraint,
            diagnostics,
        )
        .map(VersionConstraint::Caret);
    }
    if let Some(version) = value.strip_prefix('~') {
        return parse_version(
            path,
            key,
            version,
            ManifestDiagnosticKind::InvalidVersionConstraint,
            diagnostics,
        )
        .map(VersionConstraint::Tilde);
    }
    if comparator_starts(value) {
        let mut comparators = Vec::new();
        for part in value.split(',') {
            let comparator = parse_comparator(path, key, part.trim(), diagnostics)?;
            comparators.push(comparator);
        }
        return Some(VersionConstraint::Range(comparators));
    }
    parse_version(
        path,
        key,
        value,
        ManifestDiagnosticKind::InvalidVersionConstraint,
        diagnostics,
    )
    .map(VersionConstraint::Exact)
}

fn comparator_starts(value: &str) -> bool {
    value.starts_with('>') || value.starts_with('<')
}

fn parse_comparator(
    path: &str,
    key: &str,
    value: &str,
    diagnostics: &mut Vec<ManifestDiagnostic>,
) -> Option<VersionComparator> {
    let (op, version_text) = if let Some(version) = value.strip_prefix(">=") {
        (VersionComparison::GreaterEqual, version)
    } else if let Some(version) = value.strip_prefix("<=") {
        (VersionComparison::LessEqual, version)
    } else if let Some(version) = value.strip_prefix('>') {
        (VersionComparison::Greater, version)
    } else if let Some(version) = value.strip_prefix('<') {
        (VersionComparison::Less, version)
    } else {
        diagnostics.push(ManifestDiagnostic::new(
            path,
            key,
            ManifestDiagnosticKind::InvalidVersionConstraint,
            Some(value.to_owned()),
        ));
        return None;
    };
    parse_partial_version(path, key, version_text.trim(), diagnostics)
        .map(|version| VersionComparator { op, version })
}

fn parse_partial_version(
    path: &str,
    key: &str,
    value: &str,
    diagnostics: &mut Vec<ManifestDiagnostic>,
) -> Option<Version> {
    let dot_count = value.chars().filter(|ch| *ch == '.').count();
    let normalized = match dot_count {
        0 => format!("{value}.0.0"),
        1 => format!("{value}.0"),
        _ => value.to_owned(),
    };
    parse_version(
        path,
        key,
        &normalized,
        ManifestDiagnosticKind::InvalidVersionConstraint,
        diagnostics,
    )
}

fn parse_version_or_default(
    path: &str,
    key: &str,
    value: &str,
    diagnostics: &mut Vec<ManifestDiagnostic>,
) -> Version {
    parse_version(
        path,
        key,
        value,
        ManifestDiagnosticKind::InvalidVersion,
        diagnostics,
    )
    .unwrap_or_else(|| Version::new(0, 0, 0))
}

fn parse_version(
    path: &str,
    key: &str,
    value: &str,
    kind: ManifestDiagnosticKind,
    diagnostics: &mut Vec<ManifestDiagnostic>,
) -> Option<Version> {
    match Version::parse(value) {
        Ok(version) => Some(version),
        Err(_) => {
            diagnostics.push(ManifestDiagnostic::new(
                path,
                key,
                kind,
                Some(value.to_owned()),
            ));
            None
        }
    }
}

fn parse_toml_document(path: &str, input: &str) -> Result<Table, ManifestDiagnostics> {
    match input.parse::<Value>() {
        Ok(Value::Table(table)) => Ok(table),
        Ok(value) => Err(ManifestDiagnostics::new(vec![ManifestDiagnostic::new(
            path,
            "",
            ManifestDiagnosticKind::InvalidType,
            Some(value.type_str().to_owned()),
        )])),
        Err(error) => Err(ManifestDiagnostics::new(vec![ManifestDiagnostic::new(
            path,
            "",
            ManifestDiagnosticKind::InvalidToml,
            Some(error.to_string()),
        )])),
    }
}

fn required_table<'a>(
    path: &str,
    table: &'a Table,
    key: &str,
    diagnostics: &mut Vec<ManifestDiagnostic>,
) -> Option<&'a Table> {
    match table.get(key) {
        Some(Value::Table(value)) => Some(value),
        Some(value) => {
            diagnostics.push(ManifestDiagnostic::new(
                path,
                key,
                ManifestDiagnosticKind::InvalidType,
                Some(value.type_str().to_owned()),
            ));
            None
        }
        None => {
            diagnostics.push(ManifestDiagnostic::new(
                path,
                key,
                ManifestDiagnosticKind::MissingField,
                None,
            ));
            None
        }
    }
}

fn collect_unknown_fields(
    path: &str,
    key_prefix: &str,
    table: &Table,
    allowed: &[&str],
    diagnostics: &mut Vec<ManifestDiagnostic>,
) {
    for key in table.keys() {
        if !allowed.contains(&key.as_str()) {
            let full_key = if key_prefix.is_empty() {
                key.clone()
            } else {
                format!("{key_prefix}.{key}")
            };
            diagnostics.push(ManifestDiagnostic::new(
                path,
                full_key,
                ManifestDiagnosticKind::UnknownField,
                None,
            ));
        }
    }
}

fn read_string(
    path: &str,
    table: &Table,
    key: &str,
    diagnostics: &mut Vec<ManifestDiagnostic>,
) -> Option<String> {
    read_optional_string(path, table, key, diagnostics).or_else(|| {
        diagnostics.push(ManifestDiagnostic::new(
            path,
            key,
            ManifestDiagnosticKind::MissingField,
            None,
        ));
        None
    })
}

fn read_optional_string(
    path: &str,
    table: &Table,
    key: &str,
    diagnostics: &mut Vec<ManifestDiagnostic>,
) -> Option<String> {
    let field = key.rsplit('.').next().expect("manifest key is non-empty");
    match table.get(field) {
        Some(Value::String(value)) => Some(value.clone()),
        Some(value) => {
            diagnostics.push(ManifestDiagnostic::new(
                path,
                key,
                ManifestDiagnosticKind::InvalidType,
                Some(value.type_str().to_owned()),
            ));
            None
        }
        None => None,
    }
}

fn read_string_array(
    path: &str,
    table: &Table,
    key: &str,
    diagnostics: &mut Vec<ManifestDiagnostic>,
) -> Option<Vec<String>> {
    read_optional_string_array(path, table, key, diagnostics).or_else(|| {
        diagnostics.push(ManifestDiagnostic::new(
            path,
            key,
            ManifestDiagnosticKind::MissingField,
            None,
        ));
        None
    })
}

fn read_optional_string_array(
    path: &str,
    table: &Table,
    key: &str,
    diagnostics: &mut Vec<ManifestDiagnostic>,
) -> Option<Vec<String>> {
    let field = key.rsplit('.').next().expect("manifest key is non-empty");
    match table.get(field) {
        Some(Value::Array(values)) => {
            let mut strings = Vec::new();
            for (index, value) in values.iter().enumerate() {
                if let Some(text) = value.as_str() {
                    strings.push(text.to_owned());
                } else {
                    diagnostics.push(ManifestDiagnostic::new(
                        path,
                        format!("{key}[{index}]"),
                        ManifestDiagnosticKind::InvalidType,
                        Some(value.type_str().to_owned()),
                    ));
                }
            }
            Some(strings)
        }
        Some(value) => {
            diagnostics.push(ManifestDiagnostic::new(
                path,
                key,
                ManifestDiagnosticKind::InvalidType,
                Some(value.type_str().to_owned()),
            ));
            None
        }
        None => None,
    }
}

fn read_u64(
    path: &str,
    table: &Table,
    key: &str,
    diagnostics: &mut Vec<ManifestDiagnostic>,
) -> Option<u64> {
    let field = key.rsplit('.').next().expect("manifest key is non-empty");
    match table.get(field) {
        Some(Value::Integer(value)) => u64::try_from(*value).ok().or_else(|| {
            diagnostics.push(ManifestDiagnostic::new(
                path,
                key,
                ManifestDiagnosticKind::InvalidType,
                Some(value.to_string()),
            ));
            None
        }),
        Some(value) => {
            diagnostics.push(ManifestDiagnostic::new(
                path,
                key,
                ManifestDiagnosticKind::InvalidType,
                Some(value.type_str().to_owned()),
            ));
            None
        }
        None => {
            diagnostics.push(ManifestDiagnostic::new(
                path,
                key,
                ManifestDiagnosticKind::MissingField,
                None,
            ));
            None
        }
    }
}

fn read_optional_u32(
    path: &str,
    table: &Table,
    key: &str,
    diagnostics: &mut Vec<ManifestDiagnostic>,
) -> Option<u32> {
    let field = key.rsplit('.').next().expect("manifest key is non-empty");
    match table.get(field) {
        Some(Value::Integer(value)) => u32::try_from(*value).ok().or_else(|| {
            diagnostics.push(ManifestDiagnostic::new(
                path,
                key,
                ManifestDiagnosticKind::InvalidType,
                Some(value.to_string()),
            ));
            None
        }),
        Some(value) => {
            diagnostics.push(ManifestDiagnostic::new(
                path,
                key,
                ManifestDiagnosticKind::InvalidType,
                Some(value.type_str().to_owned()),
            ));
            None
        }
        None => None,
    }
}

fn read_optional_bool(
    path: &str,
    table: &Table,
    key: &str,
    diagnostics: &mut Vec<ManifestDiagnostic>,
) -> Option<bool> {
    let field = key.rsplit('.').next().expect("manifest key is non-empty");
    match table.get(field) {
        Some(Value::Boolean(value)) => Some(*value),
        Some(value) => {
            diagnostics.push(ManifestDiagnostic::new(
                path,
                key,
                ManifestDiagnosticKind::InvalidType,
                Some(value.type_str().to_owned()),
            ));
            None
        }
        None => None,
    }
}

fn read_solver(table: &Table, diagnostics: &mut Vec<ManifestDiagnostic>) -> Option<Solver> {
    let solver = read_optional_string(
        "<package manifest>",
        table,
        "verifier.default_solver",
        diagnostics,
    )?;
    match solver.as_str() {
        "auto" => Some(Solver::Auto),
        "vampire" => Some(Solver::Vampire),
        "e" => Some(Solver::E),
        "cvc5" => Some(Solver::Cvc5),
        "z3" => Some(Solver::Z3),
        _ => {
            diagnostics.push(ManifestDiagnostic::new(
                "<package manifest>",
                "verifier.default_solver",
                ManifestDiagnosticKind::InvalidSolver,
                Some(solver),
            ));
            None
        }
    }
}

fn validate_relative_path(
    path: &str,
    key: &str,
    value: &str,
    kind: ManifestDiagnosticKind,
    diagnostics: &mut Vec<ManifestDiagnostic>,
) {
    if !relative_path_is_valid(value) {
        diagnostics.push(ManifestDiagnostic::new(
            path,
            key,
            kind,
            Some(value.to_owned()),
        ));
    }
}

fn validate_relative_path_allow_root(
    path: &str,
    key: &str,
    value: &str,
    kind: ManifestDiagnosticKind,
    diagnostics: &mut Vec<ManifestDiagnostic>,
) {
    if value != "." && !relative_path_is_valid(value) {
        diagnostics.push(ManifestDiagnostic::new(
            path,
            key,
            kind,
            Some(value.to_owned()),
        ));
    }
}

fn relative_path_is_valid(value: &str) -> bool {
    if value.is_empty()
        || value.starts_with('/')
        || value.contains('\\')
        || value.contains(':')
        || value.contains("//")
    {
        return false;
    }
    value
        .split('/')
        .all(|component| !component.is_empty() && component != "." && component != "..")
}

fn lock_source_sort_key(source: &LockSource) -> String {
    match source {
        LockSource::Workspace { path } => format!("workspace:{path}"),
        LockSource::Registry { registry, checksum } => format!("registry:{registry}:{checksum}"),
    }
}

fn validate_manifest_dependency_against_lockfile(
    manifest: &PackageManifest,
    dependency: &ManifestDependency,
    lockfile: &Lockfile,
    diagnostics: &mut Vec<ManifestDiagnostic>,
) {
    match lockfile
        .packages
        .iter()
        .find(|locked| locked.name == dependency.package_id)
    {
        Some(locked) if dependency.version.matches(&locked.version) => {}
        Some(locked) => diagnostics.push(ManifestDiagnostic::new(
            "<lockfile>",
            format!("{}.dependencies.{}", manifest.name, dependency.package_id),
            ManifestDiagnosticKind::InvalidDependencyVersion,
            Some(format!(
                "{} does not satisfy {:?}",
                locked.version, dependency.version
            )),
        )),
        None => diagnostics.push(ManifestDiagnostic::new(
            "<lockfile>",
            format!("{}.dependencies.{}", manifest.name, dependency.package_id),
            ManifestDiagnosticKind::MissingLockPackage,
            None,
        )),
    }
}

impl VersionConstraint {
    fn matches(&self, version: &Version) -> bool {
        match self {
            Self::Exact(exact) => version == exact,
            Self::Caret(base) => version >= base && version < &upper_bound_caret(base),
            Self::Tilde(base) => {
                version >= base && version < &Version::new(base.major, base.minor + 1, 0)
            }
            Self::Range(comparators) => comparators
                .iter()
                .all(|comparator| comparator.matches(version)),
        }
    }
}

impl VersionComparator {
    fn matches(&self, version: &Version) -> bool {
        match self.op {
            VersionComparison::Greater => version > &self.version,
            VersionComparison::GreaterEqual => version >= &self.version,
            VersionComparison::Less => version < &self.version,
            VersionComparison::LessEqual => version <= &self.version,
        }
    }
}

fn upper_bound_caret(base: &Version) -> Version {
    if base.major > 0 {
        Version::new(base.major + 1, 0, 0)
    } else if base.minor > 0 {
        Version::new(0, base.minor + 1, 0)
    } else {
        Version::new(0, 0, base.patch + 1)
    }
}

fn finish_with_diagnostics<T>(
    value: T,
    diagnostics: Vec<ManifestDiagnostic>,
) -> Result<T, ManifestDiagnostics> {
    if diagnostics.is_empty() {
        Ok(value)
    } else {
        Err(ManifestDiagnostics::new(diagnostics))
    }
}

fn sort_diagnostics(diagnostics: &mut [ManifestDiagnostic]) {
    diagnostics.sort_by(|left, right| {
        left.location
            .path
            .cmp(&right.location.path)
            .then_with(|| left.location.key.cmp(&right.location.key))
            .then_with(|| diagnostic_rank(&left.kind).cmp(&diagnostic_rank(&right.kind)))
            .then_with(|| left.value.cmp(&right.value))
    });
}

fn diagnostic_rank(kind: &ManifestDiagnosticKind) -> u8 {
    match kind {
        ManifestDiagnosticKind::InvalidToml => 0,
        ManifestDiagnosticKind::MissingField => 1,
        ManifestDiagnosticKind::UnknownField => 2,
        ManifestDiagnosticKind::InvalidType => 3,
        ManifestDiagnosticKind::InvalidPackageId { .. } => 4,
        ManifestDiagnosticKind::InvalidVersion => 5,
        ManifestDiagnosticKind::InvalidVersionConstraint => 6,
        ManifestDiagnosticKind::InvalidWorkspaceMemberPath => 7,
        ManifestDiagnosticKind::DuplicateWorkspaceMember => 8,
        ManifestDiagnosticKind::InvalidBuildPath => 9,
        ManifestDiagnosticKind::InvalidSolver => 10,
        ManifestDiagnosticKind::InvalidLockfileSchema => 11,
        ManifestDiagnosticKind::DuplicateLockPackage => 12,
        ManifestDiagnosticKind::MissingLockPackage => 13,
        ManifestDiagnosticKind::LockVersionMismatch => 14,
        ManifestDiagnosticKind::UnknownLockedDependency => 15,
        ManifestDiagnosticKind::InvalidLockSource => 16,
        ManifestDiagnosticKind::InvalidDependencyVersion => 17,
        ManifestDiagnosticKind::DuplicateFeature => 18,
    }
}

impl fmt::Display for ManifestDiagnostics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, diagnostic) in self.diagnostics.iter().enumerate() {
            if index > 0 {
                writeln!(f)?;
            }
            write!(f, "{diagnostic}")?;
        }
        Ok(())
    }
}

impl Error for ManifestDiagnostics {}

impl fmt::Display for ManifestDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.location, self.kind)?;
        if let Some(value) = &self.value {
            write!(f, " `{value}`")?;
        }
        Ok(())
    }
}

impl fmt::Display for DiagnosticLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.key.is_empty() {
            f.write_str(&self.path)
        } else {
            write!(f, "{}:{}", self.path, self.key)
        }
    }
}

impl fmt::Display for ManifestDiagnosticKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidToml => f.write_str("invalid TOML"),
            Self::MissingField => f.write_str("missing required field"),
            Self::UnknownField => f.write_str("unknown field"),
            Self::InvalidType => f.write_str("invalid type"),
            Self::InvalidPackageId { expected } => {
                write!(f, "invalid package id; expected `{expected}`")
            }
            Self::InvalidVersion => f.write_str("invalid semantic version"),
            Self::InvalidVersionConstraint => f.write_str("invalid version constraint"),
            Self::InvalidWorkspaceMemberPath => f.write_str("invalid workspace member path"),
            Self::DuplicateWorkspaceMember => f.write_str("duplicate workspace member"),
            Self::InvalidBuildPath => f.write_str("invalid build path"),
            Self::InvalidSolver => f.write_str("invalid solver"),
            Self::InvalidLockfileSchema => f.write_str("invalid lockfile schema"),
            Self::DuplicateLockPackage => f.write_str("duplicate locked package"),
            Self::MissingLockPackage => f.write_str("missing locked package"),
            Self::LockVersionMismatch => f.write_str("lockfile version mismatch"),
            Self::UnknownLockedDependency => f.write_str("unknown locked dependency"),
            Self::InvalidLockSource => f.write_str("invalid lock source"),
            Self::InvalidDependencyVersion => f.write_str("invalid dependency version"),
            Self::DuplicateFeature => f.write_str("duplicate feature"),
        }
    }
}

impl fmt::Display for ManifestValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPackageId {
                package_id,
                expected,
            } => {
                write!(
                    f,
                    "invalid package id `{package_id}`; expected lowercase snake_case matching `{expected}`"
                )
            }
        }
    }
}

impl Error for ManifestValidationError {}

#[cfg(test)]
mod tests {
    use super::{
        BuildConfig, DependencyKind, LockSource, ManifestDependency, ManifestDiagnosticKind,
        ManifestValidationError, Solver, VerifierConfig, VersionComparison, VersionConstraint,
        is_lowercase_snake_case_package_id, parse_lockfile, parse_package_manifest,
        parse_workspace_manifest, validate_lockfile_for_workspace, validate_package_id_spelling,
        validate_package_manifest,
    };
    use semver::Version;

    #[test]
    fn package_ids_accept_lowercase_snake_case() {
        for package_id in ["a", "mml", "mathcomp_mizar", "pkg1", "pkg_1_core2"] {
            assert!(
                is_lowercase_snake_case_package_id(package_id),
                "{package_id:?}"
            );
            validate_package_id_spelling(package_id).expect("valid package id");
        }
    }

    #[test]
    fn package_ids_reject_hyphenated_or_normalized_spellings() {
        for package_id in [
            "mathcomp-mizar",
            "MathComp",
            "mathcomp__mizar",
            "mathcomp_",
            "_mathcomp",
            "mathcomp mizar",
            "mathcomp.mizar",
            "1mathcomp",
            "",
        ] {
            let error = validate_package_id_spelling(package_id).unwrap_err();
            assert!(
                matches!(
                    error,
                    ManifestValidationError::InvalidPackageId {
                        package_id: ref rejected,
                        expected: "[a-z][a-z0-9]*(?:_[a-z0-9]+)*",
                    } if rejected == package_id
                ),
                "{package_id:?}: {error:?}"
            );
        }
    }

    #[test]
    fn package_manifest_validation_preserves_spelling_without_hyphen_normalization() {
        let manifest = parse_package_manifest(
            r#"
            [package]
            name = "mathcomp_mizar"
            version = "1.2.3"
            "#,
        )
        .expect("valid manifest");
        let validated = validate_package_manifest(&manifest).expect("valid manifest");
        assert_eq!(validated.package_id.as_str(), "mathcomp_mizar");

        let error = parse_package_manifest(
            r#"
            [package]
            name = "mathcomp-mizar"
            version = "1.2.3"
            "#,
        )
        .unwrap_err();
        assert!(error.diagnostics().iter().any(|diagnostic| {
            matches!(
                diagnostic.kind,
                ManifestDiagnosticKind::InvalidPackageId {
                    expected: "[a-z][a-z0-9]*(?:_[a-z0-9]+)*",
                }
            ) && diagnostic.value.as_deref() == Some("mathcomp-mizar")
        }));
    }

    #[test]
    fn parses_valid_package_manifest_with_defaults_and_dependency_forms() {
        let manifest = parse_package_manifest(
            r#"
            [package]
            name = "algebra"
            version = "2.3.1"

            [dependencies]
            mml_core = "^1.0.0"
            topology = { version = "~0.9.0", features = ["metric", "compact"] }

            [dev-dependencies]
            test_utils = "0.2.0"
            "#,
        )
        .expect("valid package manifest");

        assert_eq!(manifest.name, "algebra");
        assert_eq!(manifest.version, Version::new(2, 3, 1));
        assert_eq!(manifest.edition.as_str(), "2025");
        assert_eq!(manifest.verifier, VerifierConfig::default());
        assert_eq!(manifest.build, BuildConfig::default());
        assert_eq!(
            manifest.dependencies,
            vec![
                ManifestDependency {
                    package_id: "mml_core".to_owned(),
                    version: VersionConstraint::Caret(Version::new(1, 0, 0)),
                    kind: DependencyKind::Normal,
                    features: Vec::new(),
                },
                ManifestDependency {
                    package_id: "topology".to_owned(),
                    version: VersionConstraint::Tilde(Version::new(0, 9, 0)),
                    kind: DependencyKind::Normal,
                    features: vec!["compact".to_owned(), "metric".to_owned()],
                },
            ]
        );
        assert_eq!(manifest.dev_dependencies[0].kind, DependencyKind::Dev);
    }

    #[test]
    fn parses_verifier_and_build_config() {
        let manifest = parse_package_manifest(
            r#"
            [package]
            name = "algebra"
            version = "2.3.1"
            edition = "2025"

            [verifier]
            max_axioms = 64
            atp_timeout = 10
            default_solver = "z3"
            require_kernel_certificates = false

            [build]
            incremental = false
            cache_dir = "cache"
            artifact_dir = "out"
            "#,
        )
        .expect("valid manifest");

        assert_eq!(manifest.verifier.max_axioms, 64);
        assert_eq!(manifest.verifier.atp_timeout, 10);
        assert_eq!(manifest.verifier.default_solver, Solver::Z3);
        assert!(!manifest.verifier.require_kernel_certificates);
        assert!(!manifest.build.incremental);
        assert_eq!(manifest.build.cache_dir, "cache");
        assert_eq!(manifest.build.artifact_dir, "out");
    }

    #[test]
    fn invalid_manifest_reports_deterministic_error_order() {
        let diagnostics = parse_package_manifest(
            r#"
            unexpected = true

            [build]
            artifact_dir = "../out"
            cache_dir = "/tmp/cache"

            [package]
            version = "not-semver"
            name = "bad-name"

            [verifier]
            default_solver = "missing"
            "#,
        )
        .unwrap_err()
        .into_diagnostics();

        let keys = diagnostics
            .iter()
            .map(|diagnostic| diagnostic.location.key.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            keys,
            vec![
                "build.artifact_dir",
                "build.cache_dir",
                "package.name",
                "package.version",
                "unexpected",
                "verifier.default_solver",
            ]
        );
    }

    #[test]
    fn workspace_manifest_validates_members_and_root_member() {
        let manifest = parse_workspace_manifest(
            r#"
            [workspace]
            members = ["topology", ".", "algebra"]
            "#,
        )
        .expect("valid workspace manifest");
        assert_eq!(manifest.members, vec![".", "algebra", "topology"]);

        let diagnostics = parse_workspace_manifest(
            r#"
            [workspace]
            members = ["algebra", "algebra", "../outside", ""]
            "#,
        )
        .unwrap_err()
        .into_diagnostics();
        assert!(diagnostics.iter().any(|diagnostic| matches!(
            diagnostic.kind,
            ManifestDiagnosticKind::DuplicateWorkspaceMember
        )));
        assert_eq!(
            diagnostics
                .iter()
                .filter(|diagnostic| matches!(
                    diagnostic.kind,
                    ManifestDiagnosticKind::InvalidWorkspaceMemberPath
                ))
                .count(),
            2
        );
    }

    #[test]
    fn lockfile_parses_sources_and_validates_internal_dependency_targets() {
        let lockfile = parse_lockfile(
            r#"
            schema_version = 1

            [[package]]
            name = "algebra"
            version = "1.2.3"
            source = { kind = "workspace", path = "algebra" }
            dependencies = [{ name = "mml_core", version = "1.0.0" }]

            [[package]]
            name = "mml_core"
            version = "1.0.0"
            source = { kind = "registry", registry = "default", checksum = "sha256:abcd" }
            dependencies = []
            "#,
        )
        .expect("valid lockfile");

        assert_eq!(lockfile.schema_version, 1);
        assert_eq!(lockfile.packages[0].name, "algebra");
        assert_eq!(
            lockfile.packages[0].source,
            LockSource::Workspace {
                path: "algebra".to_owned()
            }
        );
        assert_eq!(lockfile.packages[1].name, "mml_core");

        let diagnostics = parse_lockfile(
            r#"
            schema_version = 1

            [[package]]
            name = "algebra"
            version = "1.2.3"
            source = { kind = "workspace", path = "algebra" }
            dependencies = [{ name = "missing_dep", version = "1.0.0" }]
            "#,
        )
        .unwrap_err();
        assert!(diagnostics.diagnostics().iter().any(|diagnostic| matches!(
            diagnostic.kind,
            ManifestDiagnosticKind::UnknownLockedDependency
        )));
    }

    #[test]
    fn lockfile_workspace_validation_reports_missing_and_mismatched_versions() {
        let algebra = parse_package_manifest(
            r#"
            [package]
            name = "algebra"
            version = "1.2.3"
            "#,
        )
        .expect("valid manifest");
        let topology = parse_package_manifest(
            r#"
            [package]
            name = "topology"
            version = "0.1.0"
            "#,
        )
        .expect("valid manifest");
        let lockfile = parse_lockfile(
            r#"
            schema_version = 1

            [[package]]
            name = "algebra"
            version = "1.2.4"
            source = { kind = "workspace", path = "algebra" }
            dependencies = []
            "#,
        )
        .expect("valid lockfile");

        let diagnostics =
            validate_lockfile_for_workspace(&[algebra, topology], &lockfile).unwrap_err();
        assert_eq!(
            diagnostics
                .diagnostics()
                .iter()
                .map(|diagnostic| &diagnostic.kind)
                .collect::<Vec<_>>(),
            vec![
                &ManifestDiagnosticKind::LockVersionMismatch,
                &ManifestDiagnosticKind::MissingLockPackage,
            ]
        );
    }

    #[test]
    fn lockfile_workspace_validation_requires_workspace_source_kind() {
        let algebra = parse_package_manifest(
            r#"
            [package]
            name = "algebra"
            version = "1.2.3"
            "#,
        )
        .expect("valid manifest");
        let lockfile = parse_lockfile(
            r#"
            schema_version = 1

            [[package]]
            name = "algebra"
            version = "1.2.3"
            source = { kind = "registry", registry = "default", checksum = "sha256:abcd" }
            dependencies = []
            "#,
        )
        .expect("valid lockfile");

        let diagnostics = validate_lockfile_for_workspace(&[algebra], &lockfile).unwrap_err();
        assert!(diagnostics.diagnostics().iter().any(|diagnostic| {
            matches!(diagnostic.kind, ManifestDiagnosticKind::InvalidLockSource)
                && diagnostic.location.key == "algebra.source.kind"
        }));
    }

    #[test]
    fn lockfile_workspace_validation_checks_manifest_dependency_versions() {
        let algebra = parse_package_manifest(
            r#"
            [package]
            name = "algebra"
            version = "1.0.0"

            [dependencies]
            mml_core = "^1.0.0"
            "#,
        )
        .expect("valid manifest");
        let valid_lockfile = parse_lockfile(
            r#"
            schema_version = 1

            [[package]]
            name = "algebra"
            version = "1.0.0"
            source = { kind = "workspace", path = "algebra" }
            dependencies = [{ name = "mml_core", version = "1.2.0" }]

            [[package]]
            name = "mml_core"
            version = "1.2.0"
            source = { kind = "registry", registry = "default", checksum = "sha256:abcd" }
            dependencies = []
            "#,
        )
        .expect("valid lockfile");
        validate_lockfile_for_workspace(std::slice::from_ref(&algebra), &valid_lockfile)
            .expect("dependency satisfies caret constraint");

        let invalid_lockfile = parse_lockfile(
            r#"
            schema_version = 1

            [[package]]
            name = "algebra"
            version = "1.0.0"
            source = { kind = "workspace", path = "algebra" }
            dependencies = [{ name = "mml_core", version = "2.0.0" }]

            [[package]]
            name = "mml_core"
            version = "2.0.0"
            source = { kind = "registry", registry = "default", checksum = "sha256:abcd" }
            dependencies = []
            "#,
        )
        .expect("internally consistent lockfile");
        let diagnostics = validate_lockfile_for_workspace(&[algebra], &invalid_lockfile)
            .expect_err("manifest dependency constraint rejects lock version");
        assert!(diagnostics.diagnostics().iter().any(|diagnostic| {
            matches!(
                diagnostic.kind,
                ManifestDiagnosticKind::InvalidDependencyVersion
            ) && diagnostic.location.key == "algebra.dependencies.mml_core"
        }));
    }

    #[test]
    fn lockfile_dependency_edges_require_exact_locked_versions() {
        let diagnostics = parse_lockfile(
            r#"
            schema_version = 1

            [[package]]
            name = "algebra"
            version = "1.0.0"
            source = { kind = "workspace", path = "algebra" }
            dependencies = [{ name = "mml_core", version = "9.9.9" }]

            [[package]]
            name = "mml_core"
            version = "1.0.0"
            source = { kind = "registry", registry = "default", checksum = "sha256:abcd" }
            dependencies = []
            "#,
        )
        .unwrap_err();
        assert!(diagnostics.diagnostics().iter().any(|diagnostic| {
            matches!(diagnostic.kind, ManifestDiagnosticKind::LockVersionMismatch)
                && diagnostic.location.key == "algebra.dependencies.mml_core"
        }));
    }

    #[test]
    fn required_field_and_toml_errors_are_reported() {
        assert!(parse_package_manifest("not =").is_err_and(|diagnostics| {
            diagnostics
                .diagnostics()
                .iter()
                .any(|diagnostic| matches!(diagnostic.kind, ManifestDiagnosticKind::InvalidToml))
        }));

        let package_diagnostics = parse_package_manifest(
            r#"
            [package]
            name = "algebra"
            "#,
        )
        .unwrap_err();
        assert!(
            package_diagnostics
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.location.key == "package.version"
                    && matches!(diagnostic.kind, ManifestDiagnosticKind::MissingField))
        );

        let workspace_diagnostics = parse_workspace_manifest("[workspace]\n").unwrap_err();
        assert!(
            workspace_diagnostics
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.location.key == "workspace.members"
                    && matches!(diagnostic.kind, ManifestDiagnosticKind::MissingField))
        );

        let lockfile_diagnostics = parse_lockfile("schema_version = 1\n").unwrap_err();
        assert!(
            lockfile_diagnostics
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.location.key == "package"
                    && matches!(diagnostic.kind, ManifestDiagnosticKind::MissingField))
        );
    }

    #[test]
    fn paths_reject_host_specific_or_noncanonical_spelling() {
        let workspace_diagnostics = parse_workspace_manifest(
            r#"
            [workspace]
            members = ["foo\\bar", "foo//bar"]
            "#,
        )
        .unwrap_err();
        assert_eq!(
            workspace_diagnostics
                .diagnostics()
                .iter()
                .filter(|diagnostic| matches!(
                    diagnostic.kind,
                    ManifestDiagnosticKind::InvalidWorkspaceMemberPath
                ))
                .count(),
            2
        );

        let build_diagnostics = parse_package_manifest(
            r#"
            [package]
            name = "algebra"
            version = "1.0.0"

            [build]
            cache_dir = "C:\\cache"
            "#,
        )
        .unwrap_err();
        assert!(
            build_diagnostics
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.location.key == "build.cache_dir"
                    && matches!(diagnostic.kind, ManifestDiagnosticKind::InvalidBuildPath))
        );

        parse_lockfile(
            r#"
            schema_version = 1

            [[package]]
            name = "root_pkg"
            version = "1.0.0"
            source = { kind = "workspace", path = "." }
            dependencies = []
            "#,
        )
        .expect("root workspace lock path is valid");
    }

    #[test]
    fn lockfile_reports_missing_dependency_array() {
        let diagnostics = parse_lockfile(
            r#"
            schema_version = 1

            [[package]]
            name = "algebra"
            version = "1.0.0"
            source = { kind = "workspace", path = "algebra" }
            "#,
        )
        .unwrap_err();
        assert!(diagnostics.diagnostics().iter().any(|diagnostic| {
            diagnostic.location.key == "package[0].dependencies"
                && matches!(diagnostic.kind, ManifestDiagnosticKind::MissingField)
        }));
    }

    #[test]
    fn duplicate_lock_package_diagnostics_are_order_independent() {
        let first = parse_lockfile(
            r#"
            schema_version = 1

            [[package]]
            name = "algebra"
            version = "2.0.0"
            source = { kind = "workspace", path = "algebra" }
            dependencies = []

            [[package]]
            name = "algebra"
            version = "1.0.0"
            source = { kind = "workspace", path = "algebra" }
            dependencies = []
            "#,
        )
        .unwrap_err()
        .into_diagnostics();
        let second = parse_lockfile(
            r#"
            schema_version = 1

            [[package]]
            name = "algebra"
            version = "1.0.0"
            source = { kind = "workspace", path = "algebra" }
            dependencies = []

            [[package]]
            name = "algebra"
            version = "2.0.0"
            source = { kind = "workspace", path = "algebra" }
            dependencies = []
            "#,
        )
        .unwrap_err()
        .into_diagnostics();

        let duplicate_values = |diagnostics: Vec<super::ManifestDiagnostic>| {
            diagnostics
                .into_iter()
                .filter(|diagnostic| {
                    matches!(
                        diagnostic.kind,
                        ManifestDiagnosticKind::DuplicateLockPackage
                    )
                })
                .map(|diagnostic| diagnostic.value)
                .collect::<Vec<_>>()
        };
        assert_eq!(duplicate_values(first), duplicate_values(second));
    }

    #[test]
    fn parses_explicit_version_ranges() {
        let manifest = parse_package_manifest(
            r#"
            [package]
            name = "ranges"
            version = "1.0.0"

            [dependencies]
            algebra = ">= 1.0, < 2.0"
            "#,
        )
        .expect("valid range");

        let VersionConstraint::Range(comparators) = &manifest.dependencies[0].version else {
            panic!("expected range constraint");
        };
        assert_eq!(comparators[0].op, VersionComparison::GreaterEqual);
        assert_eq!(comparators[0].version, Version::new(1, 0, 0));
        assert_eq!(comparators[1].op, VersionComparison::Less);
        assert_eq!(comparators[1].version, Version::new(2, 0, 0));
    }
}
