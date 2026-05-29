use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NormalizedPath(String);

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourcePathError {
    UnsupportedPathEncoding {
        path: PathBuf,
    },
    PackageRootUnavailable {
        path: PathBuf,
        kind: io::ErrorKind,
    },
    SourcePathUnavailable {
        path: PathBuf,
        kind: io::ErrorKind,
    },
    OutsidePackageRoot {
        package_root: PathBuf,
        path: PathBuf,
    },
    NonCanonicalPathAlias {
        requested: PathBuf,
        canonical: PathBuf,
    },
    NonCanonicalPathSpelling {
        requested: PathBuf,
        canonical: PathBuf,
    },
    InvalidNamespaceComponent {
        component: String,
    },
    MissingSourceRoot {
        path: PathBuf,
    },
    UnsupportedExtension {
        path: PathBuf,
    },
}

impl NormalizedPath {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NormalizedPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Display for SourcePathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPathEncoding { path } => {
                write!(f, "source path `{}` must be valid UTF-8", path.display())
            }
            Self::PackageRootUnavailable { path, kind } => {
                write!(
                    f,
                    "package root `{}` could not be canonicalized: {kind}",
                    path.display()
                )
            }
            Self::SourcePathUnavailable { path, kind } => {
                write!(
                    f,
                    "source path `{}` could not be canonicalized: {kind}",
                    path.display()
                )
            }
            Self::OutsidePackageRoot { package_root, path } => {
                write!(
                    f,
                    "source path `{}` must stay inside package root `{}`",
                    path.display(),
                    package_root.display()
                )
            }
            Self::NonCanonicalPathAlias {
                requested,
                canonical,
            } => {
                write!(
                    f,
                    "source path `{}` must not alias canonical path `{}`",
                    requested.display(),
                    canonical.display()
                )
            }
            Self::NonCanonicalPathSpelling {
                requested,
                canonical,
            } => {
                write!(
                    f,
                    "source path `{}` must use canonical spelling `{}`",
                    requested.display(),
                    canonical.display()
                )
            }
            Self::InvalidNamespaceComponent { component } => {
                write!(f, "invalid source path namespace component `{component}`")
            }
            Self::MissingSourceRoot { path } => {
                write!(
                    f,
                    "source path `{}` must be under the package `src` root",
                    path.display()
                )
            }
            Self::UnsupportedExtension { path } => {
                write!(f, "source path `{}` must end with `.miz`", path.display())
            }
        }
    }
}

impl Error for SourcePathError {}

pub fn normalize_source_path(
    package_root: &Path,
    path: &Path,
) -> Result<NormalizedPath, SourcePathError> {
    let canonical_root = fs::canonicalize(package_root).map_err(|error| {
        SourcePathError::PackageRootUnavailable {
            path: package_root.to_owned(),
            kind: error.kind(),
        }
    })?;

    let separator_normalized = path_with_normalized_separators(path)?;
    let absolute_path = if separator_normalized.is_absolute() {
        separator_normalized
    } else {
        canonical_root.join(separator_normalized)
    };
    let lexical_path = normalize_lexically(&absolute_path);
    let canonical_path = fs::canonicalize(&lexical_path).map_err(|error| {
        SourcePathError::SourcePathUnavailable {
            path: lexical_path.clone(),
            kind: error.kind(),
        }
    })?;

    if !canonical_path.starts_with(&canonical_root) {
        return Err(SourcePathError::OutsidePackageRoot {
            package_root: canonical_root,
            path: canonical_path,
        });
    }
    if canonical_path
        .extension()
        .and_then(|extension| extension.to_str())
        != Some("miz")
    {
        return Err(SourcePathError::UnsupportedExtension {
            path: canonical_path,
        });
    }

    let package_relative = canonical_path.strip_prefix(&canonical_root).map_err(|_| {
        SourcePathError::OutsidePackageRoot {
            package_root: canonical_root.clone(),
            path: canonical_path.clone(),
        }
    })?;
    let mut components = package_relative.components();
    if !matches!(components.next(), Some(Component::Normal(component)) if component == "src") {
        return Err(SourcePathError::MissingSourceRoot {
            path: canonical_path,
        });
    }
    if lexical_path != canonical_path {
        let requested = lexical_path
            .strip_prefix(&canonical_root)
            .unwrap_or(lexical_path.as_path());
        let canonical = if requested.is_absolute() {
            canonical_path.as_path()
        } else {
            package_relative
        };
        reject_non_canonical_alias(requested, canonical)?;
    }
    validate_namespace_components(package_relative)?;

    let normalized = package_relative_to_utf8(package_relative)?;
    Ok(NormalizedPath(normalized))
}

fn path_with_normalized_separators(path: &Path) -> Result<PathBuf, SourcePathError> {
    let raw = path
        .to_str()
        .ok_or_else(|| SourcePathError::UnsupportedPathEncoding {
            path: path.to_owned(),
        })?;
    Ok(PathBuf::from(raw.replace('\\', "/")))
}

fn normalize_lexically(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

fn package_relative_to_utf8(path: &Path) -> Result<String, SourcePathError> {
    let mut normalized = Vec::new();
    for component in path.components() {
        let Component::Normal(component) = component else {
            return Err(SourcePathError::UnsupportedPathEncoding {
                path: path.to_owned(),
            });
        };
        let component =
            component
                .to_str()
                .ok_or_else(|| SourcePathError::UnsupportedPathEncoding {
                    path: path.to_owned(),
                })?;
        normalized.push(component.to_owned());
    }
    Ok(normalized.join("/"))
}

fn reject_non_canonical_alias(requested: &Path, canonical: &Path) -> Result<(), SourcePathError> {
    let requested_components = normal_utf8_components(requested)?;
    let canonical_components = normal_utf8_components(canonical)?;
    if requested_components.len() != canonical_components.len() {
        return Ok(());
    }
    if requested_components == canonical_components {
        return Ok(());
    }
    if requested_components
        .iter()
        .zip(&canonical_components)
        .all(|(requested, canonical)| requested.eq_ignore_ascii_case(canonical))
    {
        return Err(SourcePathError::NonCanonicalPathSpelling {
            requested: requested.to_owned(),
            canonical: canonical.to_owned(),
        });
    }
    Err(SourcePathError::NonCanonicalPathAlias {
        requested: requested.to_owned(),
        canonical: canonical.to_owned(),
    })
}

fn normal_utf8_components(path: &Path) -> Result<Vec<String>, SourcePathError> {
    let mut components = Vec::new();
    for component in path.components() {
        let Component::Normal(component) = component else {
            return Err(SourcePathError::UnsupportedPathEncoding {
                path: path.to_owned(),
            });
        };
        let component =
            component
                .to_str()
                .ok_or_else(|| SourcePathError::UnsupportedPathEncoding {
                    path: path.to_owned(),
                })?;
        components.push(component.to_owned());
    }
    Ok(components)
}

fn validate_namespace_components(path: &Path) -> Result<(), SourcePathError> {
    let components = normal_utf8_components(path)?;
    for component in components.iter().skip(1) {
        let namespace_component = component.strip_suffix(".miz").unwrap_or(component);
        if !is_identifier_shaped(namespace_component) {
            return Err(SourcePathError::InvalidNamespaceComponent {
                component: namespace_component.to_owned(),
            });
        }
    }
    Ok(())
}

fn is_identifier_shaped(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    is_identifier_start(first) && chars.all(is_identifier_continue)
}

fn is_identifier_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

fn is_identifier_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '\''
}

#[cfg(test)]
mod tests {
    use super::{
        NormalizedPath, SourcePathError, normalize_source_path, reject_non_canonical_alias,
    };
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn source_path_normalization_removes_dot_components() {
        let package = PackageFixture::new();
        package.write("src/groups/basic.miz", "");

        let normalized = normalize_source_path(
            package.root(),
            &package.root().join("./src/./groups/../groups/basic.miz"),
        );

        assert_eq!(normalized, Ok(path("src/groups/basic.miz")));
    }

    #[test]
    fn source_path_normalization_rejects_package_root_escape_attempts() {
        let package = PackageFixture::new();
        package.write("src/main.miz", "");
        package.write_outside("outside.miz", "");

        let normalized = normalize_source_path(package.root(), Path::new("../outside.miz"));

        assert!(matches!(
            normalized,
            Err(SourcePathError::OutsidePackageRoot { .. })
        ));
    }

    #[test]
    fn source_path_normalization_rejects_sources_outside_src() {
        let package = PackageFixture::new();
        package.write("other/main.miz", "");

        let normalized = normalize_source_path(package.root(), Path::new("other/main.miz"));

        assert!(matches!(
            normalized,
            Err(SourcePathError::MissingSourceRoot { .. })
        ));
    }

    #[test]
    fn source_path_normalization_rejects_non_miz_files() {
        let package = PackageFixture::new();
        package.write("src/main.txt", "");

        let normalized = normalize_source_path(package.root(), Path::new("src/main.txt"));

        assert!(matches!(
            normalized,
            Err(SourcePathError::UnsupportedExtension { .. })
        ));
    }

    #[test]
    fn source_path_normalization_uses_canonical_case_spelling() {
        let package = PackageFixture::new();
        package.write("src/MixedCase.miz", "");

        let normalized = normalize_source_path(package.root(), Path::new("src/MixedCase.miz"));

        assert_eq!(normalized, Ok(path("src/MixedCase.miz")));
    }

    #[test]
    fn source_path_normalization_rejects_non_canonical_case_variants() {
        let rejected = reject_non_canonical_alias(
            Path::new("src/mixedcase.miz"),
            Path::new("src/MixedCase.miz"),
        );

        assert!(matches!(
            rejected,
            Err(SourcePathError::NonCanonicalPathSpelling { .. })
        ));
    }

    #[test]
    fn source_path_normalization_rejects_symlink_spelling_aliases() {
        let rejected =
            reject_non_canonical_alias(Path::new("src/alias.miz"), Path::new("src/real.miz"));

        assert!(matches!(
            rejected,
            Err(SourcePathError::NonCanonicalPathAlias { .. })
        ));
    }

    #[test]
    fn source_path_normalization_rejects_invalid_namespace_components() {
        let package = PackageFixture::new();
        package.write("src/bad-name.miz", "");

        let normalized = normalize_source_path(package.root(), Path::new("src/bad-name.miz"));

        assert!(matches!(
            normalized,
            Err(SourcePathError::InvalidNamespaceComponent { .. })
        ));
    }

    #[test]
    fn source_path_normalization_rejects_non_ascii_namespace_components() {
        let package = PackageFixture::new();
        package.write("src/naive_é.miz", "");

        let normalized = normalize_source_path(package.root(), Path::new("src/naive_é.miz"));

        assert!(matches!(
            normalized,
            Err(SourcePathError::InvalidNamespaceComponent { .. })
        ));
    }

    #[test]
    fn source_path_normalization_accepts_platform_specific_separators() {
        let package = PackageFixture::new();
        package.write("src/groups/basic.miz", "");

        let normalized = normalize_source_path(package.root(), Path::new("src\\groups\\basic.miz"));

        assert_eq!(normalized, Ok(path("src/groups/basic.miz")));
    }

    #[cfg(unix)]
    #[test]
    fn source_path_normalization_rejects_symlink_aliases_inside_package() {
        use std::os::unix::fs::symlink;

        let package = PackageFixture::new();
        package.write("src/real.miz", "");
        symlink(
            package.root().join("src/real.miz"),
            package.root().join("src/alias.miz"),
        )
        .expect("symlink should be created");

        let normalized = normalize_source_path(package.root(), Path::new("src/alias.miz"));

        assert!(matches!(
            normalized,
            Err(SourcePathError::NonCanonicalPathAlias { .. })
        ));
    }

    #[cfg(unix)]
    #[test]
    fn source_path_normalization_rejects_symlink_escapes() {
        use std::os::unix::fs::symlink;

        let package = PackageFixture::new();
        package.write_outside("outside.miz", "");
        symlink(
            package.outside_path("outside.miz"),
            package.root().join("src/escape.miz"),
        )
        .expect("symlink should be created");

        let normalized = normalize_source_path(package.root(), Path::new("src/escape.miz"));

        assert!(matches!(
            normalized,
            Err(SourcePathError::OutsidePackageRoot { .. })
        ));
    }

    #[cfg(unix)]
    #[test]
    fn source_path_normalization_rejects_absolute_symlink_aliases_inside_package() {
        use std::os::unix::fs::symlink;

        let package = PackageFixture::new();
        package.write("src/real.miz", "");
        symlink(
            package.root().join("src/real.miz"),
            package.root().join("src/alias.miz"),
        )
        .expect("symlink should be created");

        let normalized =
            normalize_source_path(package.root(), &package.root().join("src/alias.miz"));

        assert!(matches!(
            normalized,
            Err(SourcePathError::NonCanonicalPathAlias { .. })
        ));
    }

    fn path(path: &str) -> NormalizedPath {
        NormalizedPath(path.to_owned())
    }

    struct PackageFixture {
        base: PathBuf,
        root: PathBuf,
    }

    impl PackageFixture {
        fn new() -> Self {
            let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!(
                "mizar_session_source_path_{}_{}",
                std::process::id(),
                id
            ));
            let package_root = root.join("package");
            fs::create_dir_all(package_root.join("src"))
                .expect("package src directory should be created");
            Self {
                base: root,
                root: package_root,
            }
        }

        fn root(&self) -> &Path {
            &self.root
        }

        fn write(&self, relative: &str, content: &str) {
            let path = self.root.join(relative);
            self.write_path(&path, content);
        }

        fn write_outside(&self, relative: &str, content: &str) {
            let path = self.outside_path(relative);
            self.write_path(&path, content);
        }

        fn outside_path(&self, relative: &str) -> PathBuf {
            self.base.join(relative)
        }

        fn write_path(&self, path: &Path, content: &str) {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("parent directory should be created");
            }
            fs::write(path, content).expect("fixture file should be written");
        }
    }

    impl Drop for PackageFixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.base);
        }
    }
}
