use crate::ids::{BuildSnapshotId, Hash, IdError, SessionIdAllocator, SourceId};
use crate::snapshot::{Edition, GeneratedSourceKind, ModulePath, PackageId, SourceOrigin};
use crate::source_map::{
    DocumentUri, LineMap, LoadingMap, LoadingMapSegment, LoadingOrigin, LspDocumentVersion,
    SourceAnchor, TextRange, hash_source_text,
};
use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

const UTF8_BOM: &str = "\u{feff}";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NormalizedPath(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiskSourceLoader {
    package_root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceInput {
    pub package_id: PackageId,
    pub module_path: ModulePath,
    pub normalized_path: NormalizedPath,
    pub edition: Edition,
    pub origin: SourceOriginInput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceOriginInput {
    Disk {
        path: PathBuf,
    },
    OpenBuffer {
        uri: DocumentUri,
        version: LspDocumentVersion,
        text: Arc<str>,
    },
    Generated {
        generator: GeneratedSourceKind,
        text: Arc<str>,
        anchor: Option<SourceAnchor>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedSource {
    pub source_id: SourceId,
    pub package_id: PackageId,
    pub module_path: ModulePath,
    pub normalized_path: NormalizedPath,
    pub text: Arc<str>,
    pub source_hash: Hash,
    pub edition: Edition,
    pub origin: SourceOrigin,
    pub line_map: LineMap,
    pub loading_map: Option<LoadingMap>,
}

pub trait SourceLoader {
    fn load(
        &self,
        snapshot: BuildSnapshotId,
        input: SourceInput,
        ids: &dyn SessionIdAllocator,
    ) -> Result<LoadedSource, SourceLoadError>;

    fn normalize_path(
        &self,
        package_root: &Path,
        path: &Path,
    ) -> Result<NormalizedPath, SourceLoadError> {
        normalize_path(package_root, path)
    }

    fn hash_text(&self, text: &str) -> Hash {
        hash_text(text)
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceLoadError {
    SourcePathOutsidePackageRoot {
        package_root: PathBuf,
        path: PathBuf,
    },
    UnsupportedFileExtension {
        path: PathBuf,
    },
    InvalidUtf8 {
        path: Option<PathBuf>,
    },
    UnreadableSourceFile {
        path: PathBuf,
        kind: io::ErrorKind,
    },
    DuplicateModulePath {
        package_id: PackageId,
        module_path: ModulePath,
    },
    StaleLspDocumentVersion {
        expected: LspDocumentVersion,
        actual: LspDocumentVersion,
    },
    UnmappedOpenBufferUri {
        uri: DocumentUri,
    },
    GeneratedSourceWithoutMetadata {
        module_path: ModulePath,
    },
    SourceIdAllocation {
        error: IdError,
    },
    UnsupportedSourceOrigin {
        origin: SourceOriginKind,
    },
    InvalidSourcePath {
        error: SourcePathError,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceOriginKind {
    Disk,
    OpenBuffer,
    Generated,
}

impl NormalizedPath {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl DiskSourceLoader {
    pub fn new(package_root: impl Into<PathBuf>) -> Self {
        Self {
            package_root: package_root.into(),
        }
    }

    pub fn package_root(&self) -> &Path {
        &self.package_root
    }
}

impl fmt::Display for NormalizedPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Display for SourceLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourcePathOutsidePackageRoot { package_root, path } => {
                write!(
                    f,
                    "source path `{}` must stay inside package root `{}`",
                    path.display(),
                    package_root.display()
                )
            }
            Self::UnsupportedFileExtension { path } => {
                write!(f, "source path `{}` must end with `.miz`", path.display())
            }
            Self::InvalidUtf8 { path } => {
                if let Some(path) = path {
                    write!(f, "source `{}` must be valid UTF-8", path.display())
                } else {
                    f.write_str("source text must be valid UTF-8")
                }
            }
            Self::UnreadableSourceFile { path, kind } => {
                write!(
                    f,
                    "source file `{}` could not be read: {kind}",
                    path.display()
                )
            }
            Self::DuplicateModulePath {
                package_id,
                module_path,
            } => {
                write!(
                    f,
                    "duplicate module path `{module_path}` in package `{package_id}`"
                )
            }
            Self::StaleLspDocumentVersion { expected, actual } => {
                write!(
                    f,
                    "stale LSP document version `{actual}`, expected `{expected}`"
                )
            }
            Self::UnmappedOpenBufferUri { uri } => {
                write!(
                    f,
                    "open-buffer URI `{uri}` cannot be mapped to a package source"
                )
            }
            Self::GeneratedSourceWithoutMetadata { module_path } => {
                write!(
                    f,
                    "generated source for module `{module_path}` is missing required generator metadata"
                )
            }
            Self::SourceIdAllocation { error } => {
                write!(f, "could not allocate source id: {error}")
            }
            Self::UnsupportedSourceOrigin { origin } => {
                write!(
                    f,
                    "source origin `{origin}` is not supported by this loader"
                )
            }
            Self::InvalidSourcePath { error } => {
                write!(f, "invalid or non-normalizable source path: {error}")
            }
        }
    }
}

impl Error for SourceLoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::SourceIdAllocation { error } => Some(error),
            Self::InvalidSourcePath { error } => Some(error),
            _ => None,
        }
    }
}

impl fmt::Display for SourceOriginKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Disk => f.write_str("disk"),
            Self::OpenBuffer => f.write_str("open-buffer"),
            Self::Generated => f.write_str("generated"),
        }
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

pub fn normalize_path(package_root: &Path, path: &Path) -> Result<NormalizedPath, SourceLoadError> {
    normalize_source_path(package_root, path)
        .map_err(|error| SourceLoadError::from_source_path_error(package_root, error))
}

pub fn hash_text(text: &str) -> Hash {
    hash_source_text(text)
}

impl SourceLoader for DiskSourceLoader {
    fn load(
        &self,
        snapshot: BuildSnapshotId,
        input: SourceInput,
        ids: &dyn SessionIdAllocator,
    ) -> Result<LoadedSource, SourceLoadError> {
        let SourceInput {
            package_id,
            module_path,
            normalized_path: _,
            edition,
            origin,
        } = input;

        let SourceOriginInput::Disk { path } = origin else {
            let origin = match origin {
                SourceOriginInput::Disk { .. } => SourceOriginKind::Disk,
                SourceOriginInput::OpenBuffer { .. } => SourceOriginKind::OpenBuffer,
                SourceOriginInput::Generated { .. } => SourceOriginKind::Generated,
            };
            return Err(SourceLoadError::UnsupportedSourceOrigin { origin });
        };

        let normalized_path = self.normalize_path(&self.package_root, &path)?;
        let read_path = self.package_root.join(normalized_path.as_str());
        let bytes =
            fs::read(&read_path).map_err(|error| SourceLoadError::UnreadableSourceFile {
                path: read_path.clone(),
                kind: error.kind(),
            })?;
        let loaded_text = normalize_disk_source_bytes(&read_path, &bytes)?;
        let source_hash = self.hash_text(&loaded_text.text);
        let source_id = ids
            .next_source_id(snapshot)
            .map_err(|error| SourceLoadError::SourceIdAllocation { error })?;
        let line_map = LineMap::new(source_id, &loaded_text.text);
        let loading_map = loaded_text.loading_segments.map(|segments| {
            LoadingMap::new(
                source_id,
                &loaded_text.text,
                LoadingOrigin::DiskBytes {
                    normalized_path: normalized_path.clone(),
                },
                segments,
            )
        });
        let text = Arc::from(loaded_text.text);

        Ok(LoadedSource {
            source_id,
            package_id,
            module_path,
            normalized_path,
            text,
            source_hash,
            edition,
            origin: SourceOrigin::Disk,
            line_map,
            loading_map,
        })
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
struct NormalizedDiskText {
    text: String,
    loading_segments: Option<Vec<LoadingMapSegment>>,
}

fn normalize_disk_source_bytes(
    path: &Path,
    bytes: &[u8],
) -> Result<NormalizedDiskText, SourceLoadError> {
    let text = std::str::from_utf8(bytes).map_err(|_| SourceLoadError::InvalidUtf8 {
        path: Some(path.to_owned()),
    })?;
    let (source_text, original_base, mut segments) =
        if let Some(stripped) = text.strip_prefix(UTF8_BOM) {
            (
                stripped,
                UTF8_BOM.len(),
                vec![LoadingMapSegment::RemovedLeadingBom {
                    original: TextRange {
                        start: 0,
                        end: UTF8_BOM.len(),
                    },
                }],
            )
        } else {
            (text, 0, Vec::new())
        };

    if !source_text.contains("\r\n") {
        if segments.is_empty() {
            return Ok(NormalizedDiskText {
                text: text.to_owned(),
                loading_segments: None,
            });
        }
        segments.push(LoadingMapSegment::Original {
            loaded: TextRange {
                start: 0,
                end: source_text.len(),
            },
            original: TextRange {
                start: original_base,
                end: original_base + source_text.len(),
            },
        });
        return Ok(NormalizedDiskText {
            text: source_text.to_owned(),
            loading_segments: Some(segments),
        });
    }

    let normalized = normalize_crlf_to_lf(source_text, original_base, &mut segments);
    Ok(NormalizedDiskText {
        text: normalized,
        loading_segments: Some(segments),
    })
}

fn normalize_crlf_to_lf(
    source_text: &str,
    original_base: usize,
    segments: &mut Vec<LoadingMapSegment>,
) -> String {
    let mut normalized = String::with_capacity(source_text.len());
    let mut cursor = 0;
    let mut next_crlf = source_text.find("\r\n");

    while let Some(crlf_start) = next_crlf {
        let loaded_start = normalized.len();
        normalized.push_str(&source_text[cursor..crlf_start]);
        if cursor < crlf_start {
            segments.push(LoadingMapSegment::Original {
                loaded: TextRange {
                    start: loaded_start,
                    end: normalized.len(),
                },
                original: TextRange {
                    start: original_base + cursor,
                    end: original_base + crlf_start,
                },
            });
        }

        let loaded_start = normalized.len();
        normalized.push('\n');
        segments.push(LoadingMapSegment::NormalizedNewline {
            loaded: TextRange {
                start: loaded_start,
                end: loaded_start + 1,
            },
            original: TextRange {
                start: original_base + crlf_start,
                end: original_base + crlf_start + 2,
            },
        });

        cursor = crlf_start + 2;
        next_crlf = source_text[cursor..]
            .find("\r\n")
            .map(|relative| cursor + relative);
    }

    let loaded_start = normalized.len();
    normalized.push_str(&source_text[cursor..]);
    if cursor < source_text.len() {
        segments.push(LoadingMapSegment::Original {
            loaded: TextRange {
                start: loaded_start,
                end: normalized.len(),
            },
            original: TextRange {
                start: original_base + cursor,
                end: original_base + source_text.len(),
            },
        });
    }

    normalized
}

impl SourceLoadError {
    fn from_source_path_error(package_root: &Path, error: SourcePathError) -> Self {
        match error {
            SourcePathError::OutsidePackageRoot { package_root, path } => {
                Self::SourcePathOutsidePackageRoot { package_root, path }
            }
            SourcePathError::MissingSourceRoot { path } => Self::SourcePathOutsidePackageRoot {
                package_root: package_root.to_owned(),
                path,
            },
            SourcePathError::UnsupportedExtension { path } => {
                Self::UnsupportedFileExtension { path }
            }
            SourcePathError::PackageRootUnavailable { path, kind }
            | SourcePathError::SourcePathUnavailable { path, kind } => {
                Self::UnreadableSourceFile { path, kind }
            }
            SourcePathError::UnsupportedPathEncoding { path } => {
                Self::InvalidUtf8 { path: Some(path) }
            }
            SourcePathError::NonCanonicalPathAlias { .. }
            | SourcePathError::NonCanonicalPathSpelling { .. }
            | SourcePathError::InvalidNamespaceComponent { .. } => {
                Self::InvalidSourcePath { error }
            }
        }
    }
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
        DiskSourceLoader, LoadedSource, NormalizedPath, SourceInput, SourceLoadError, SourceLoader,
        SourceOriginInput, SourceOriginKind, SourcePathError, hash_text, normalize_path,
        normalize_source_path, reject_non_canonical_alias,
    };
    use crate::{
        BuildRequestId, BuildSessionId, BuildSnapshotId, Edition, GeneratedSourceKind, IdError,
        InMemorySessionIdAllocator, LineMap, LoadingMapSegment, LoadingOrigin, ModulePath,
        PackageId, SessionIdAllocator, SnapshotLeaseId, SourceId, SourceMapId, SourceOrigin,
        TextRange,
    };
    use std::fs;
    use std::io;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn source_hash_is_text_only_and_excludes_path_or_document_version_metadata() {
        let text = "environ\nbegin\n";
        let snapshot = snapshot_id(1);
        let ids = InMemorySessionIdAllocator::new();
        let loader = TextOnlyLoader::with_disk_text(text);
        let disk = loader
            .load(
                snapshot,
                source_input(SourceOriginInput::Disk {
                    path: PathBuf::from("/absolute/package/src/basic.miz"),
                }),
                &ids,
            )
            .unwrap();
        let same_text_different_path = loader
            .load(
                snapshot,
                source_input(SourceOriginInput::Disk {
                    path: PathBuf::from("/other/root/src/basic.miz"),
                }),
                &ids,
            )
            .unwrap();
        let open_v1 = loader
            .load(
                snapshot,
                source_input(SourceOriginInput::OpenBuffer {
                    uri: "file:///absolute/package/src/basic.miz".to_owned(),
                    version: 1,
                    text: Arc::from(text),
                }),
                &ids,
            )
            .unwrap();
        let open_v2 = loader
            .load(
                snapshot,
                source_input(SourceOriginInput::OpenBuffer {
                    uri: "file:///absolute/package/src/basic.miz".to_owned(),
                    version: 2,
                    text: Arc::from(text),
                }),
                &ids,
            )
            .unwrap();

        assert_eq!(disk.source_hash, hash_text(text));
        assert_eq!(disk.source_hash, same_text_different_path.source_hash);
        assert_eq!(disk.source_hash, open_v1.source_hash);
        assert_eq!(open_v1.source_hash, open_v2.source_hash);
        assert_eq!(hash_text(text), disk.line_map.text_hash());
    }

    #[test]
    fn identical_text_hashes_match_across_disk_open_buffer_and_generated_origins() {
        let text = "theorem Th1;\n";
        let snapshot = snapshot_id(2);
        let ids = InMemorySessionIdAllocator::new();
        let loader = TextOnlyLoader::with_disk_text(text);
        let disk = loader
            .load(
                snapshot,
                source_input(SourceOriginInput::Disk {
                    path: PathBuf::from("/absolute/package/src/basic.miz"),
                }),
                &ids,
            )
            .unwrap();
        let open_buffer = loader
            .load(
                snapshot,
                source_input(SourceOriginInput::OpenBuffer {
                    uri: "file:///absolute/package/src/basic.miz".to_owned(),
                    version: 4,
                    text: Arc::from(text),
                }),
                &ids,
            )
            .unwrap();
        let generated = loader
            .load(
                snapshot,
                source_input(SourceOriginInput::Generated {
                    generator: GeneratedSourceKind::new("test-generator"),
                    text: Arc::from(text),
                    anchor: None,
                }),
                &ids,
            )
            .unwrap();

        assert_eq!(disk.source_hash, open_buffer.source_hash);
        assert_eq!(open_buffer.source_hash, generated.source_hash);
        assert!(matches!(disk.origin, SourceOrigin::Disk));
        assert!(matches!(
            open_buffer.origin,
            SourceOrigin::OpenBuffer { version: 4 }
        ));
        assert!(matches!(
            generated.origin,
            SourceOrigin::Generated { ref generator } if generator.as_str() == "test-generator"
        ));
    }

    #[test]
    fn source_loader_load_uses_snapshot_scoped_source_id_allocation() {
        let snapshot = snapshot_id(3);
        let allocator = RecordingAllocator::new();
        let loader = TextOnlyLoader::default();
        let input = source_input(SourceOriginInput::OpenBuffer {
            uri: "file:///package/src/basic.miz".to_owned(),
            version: 7,
            text: Arc::from("environ\nbegin\n"),
        });

        let loaded = loader.load(snapshot, input, &allocator).unwrap();

        assert_eq!(allocator.source_snapshots(), vec![snapshot]);
        assert_eq!(loaded.line_map.source_id(), loaded.source_id);
        assert_eq!(loaded.line_map.text_hash(), loaded.source_hash);
        assert!(matches!(
            loaded.origin,
            SourceOrigin::OpenBuffer { version: 7 }
        ));
    }

    #[test]
    fn loaded_source_origin_reuses_snapshot_source_origin() {
        let loaded = loaded_source(
            InMemorySessionIdAllocator::new()
                .next_source_id(snapshot_id(4))
                .unwrap(),
            SourceOrigin::Generated {
                generator: GeneratedSourceKind::new("doc-extract"),
            },
            "definition X;\n",
        );

        let origin: SourceOrigin = loaded.origin.clone();

        assert!(matches!(
            origin,
            SourceOrigin::Generated { generator } if generator.as_str() == "doc-extract"
        ));
    }

    #[test]
    fn source_loader_normalize_path_reuses_source_path_normalization() {
        let package = PackageFixture::new();
        package.write("src/groups/basic.miz", "");
        package.write("src/groups/basic.txt", "");
        let loader = TextOnlyLoader::default();

        let normalized = loader
            .normalize_path(package.root(), Path::new("src/groups/basic.miz"))
            .unwrap();
        let unsupported = normalize_path(package.root(), Path::new("src/groups/basic.txt"))
            .expect_err("non-miz source should be rejected");

        assert_eq!(normalized, path("src/groups/basic.miz"));
        assert!(matches!(
            unsupported,
            SourceLoadError::UnsupportedFileExtension { .. }
        ));
    }

    #[test]
    fn disk_source_loader_reads_disk_text_and_builds_loaded_source_metadata() {
        let package = PackageFixture::new();
        package.write("src/groups/basic.miz", "environ\nbegin\n");
        let loader = DiskSourceLoader::new(package.root());
        let ids = InMemorySessionIdAllocator::new();

        let loaded = loader
            .load(
                snapshot_id(20),
                disk_source_input("src/groups/basic.miz"),
                &ids,
            )
            .unwrap();

        assert_eq!(loaded.text.as_ref(), "environ\nbegin\n");
        assert_eq!(loaded.normalized_path, path("src/groups/basic.miz"));
        assert_eq!(loaded.source_hash, hash_text("environ\nbegin\n"));
        assert_eq!(loaded.line_map.source(), "environ\nbegin\n");
        assert_eq!(loaded.line_map.line_starts(), &[0, 8, 14]);
        assert_eq!(loaded.line_map.text_hash(), loaded.source_hash);
        assert!(matches!(loaded.origin, SourceOrigin::Disk));
        assert!(loaded.loading_map.is_none());
    }

    #[test]
    fn disk_source_loader_rejects_invalid_utf8_before_source_id_allocation() {
        let package = PackageFixture::new();
        package.write_bytes("src/groups/invalid.miz", &[0xff]);
        let loader = DiskSourceLoader::new(package.root());
        let allocator = RecordingAllocator::new();

        let error = loader
            .load(
                snapshot_id(21),
                disk_source_input("src/groups/invalid.miz"),
                &allocator,
            )
            .expect_err("invalid UTF-8 should be rejected");

        assert!(matches!(
            error,
            SourceLoadError::InvalidUtf8 { path: Some(path) }
                if path.ends_with("src/groups/invalid.miz")
        ));
        assert!(allocator.source_snapshots().is_empty());
    }

    #[test]
    fn disk_source_loader_rejects_unsupported_extension_before_decoding() {
        let package = PackageFixture::new();
        package.write_bytes("src/groups/basic.txt", &[0xff]);
        let loader = DiskSourceLoader::new(package.root());

        let error = loader
            .load(
                snapshot_id(22),
                disk_source_input("src/groups/basic.txt"),
                &InMemorySessionIdAllocator::new(),
            )
            .expect_err("non-miz extension should be rejected");

        assert!(matches!(
            error,
            SourceLoadError::UnsupportedFileExtension { path }
                if path.ends_with("src/groups/basic.txt")
        ));
    }

    #[test]
    fn disk_source_loader_strips_leading_bom_and_maps_loaded_zero_to_original_three() {
        let package = PackageFixture::new();
        package.write_bytes("src/groups/bom.miz", b"\xef\xbb\xbfalpha");
        let loader = DiskSourceLoader::new(package.root());
        let loaded = loader
            .load(
                snapshot_id(23),
                disk_source_input("src/groups/bom.miz"),
                &InMemorySessionIdAllocator::new(),
            )
            .unwrap();

        assert_eq!(loaded.text.as_ref(), "alpha");
        assert_eq!(loaded.source_hash, hash_text("alpha"));
        assert_ne!(loaded.source_hash, hash_text("\u{feff}alpha"));
        let loading_map = loaded
            .loading_map
            .as_ref()
            .expect("leading BOM should emit a loading map");
        assert_eq!(
            loading_map.origin,
            LoadingOrigin::DiskBytes {
                normalized_path: path("src/groups/bom.miz")
            }
        );
        assert_eq!(
            loading_map.segments,
            vec![
                LoadingMapSegment::RemovedLeadingBom {
                    original: TextRange { start: 0, end: 3 },
                },
                LoadingMapSegment::Original {
                    loaded: TextRange { start: 0, end: 5 },
                    original: TextRange { start: 3, end: 8 },
                },
            ]
        );
        assert_eq!(
            loading_map.original_offset_for_loaded(loaded.source_id, 0),
            Ok(3)
        );
    }

    #[test]
    fn disk_source_loader_maps_file_that_contains_only_bom_to_original_eof() {
        let package = PackageFixture::new();
        package.write_bytes("src/groups/bom_only.miz", b"\xef\xbb\xbf");
        let loader = DiskSourceLoader::new(package.root());

        let loaded = loader
            .load(
                snapshot_id(24),
                disk_source_input("src/groups/bom_only.miz"),
                &InMemorySessionIdAllocator::new(),
            )
            .unwrap();

        assert_eq!(loaded.text.as_ref(), "");
        assert_eq!(loaded.source_hash, hash_text(""));
        let loading_map = loaded
            .loading_map
            .as_ref()
            .expect("leading BOM should emit a loading map");
        assert_eq!(
            loading_map.segments,
            vec![
                LoadingMapSegment::RemovedLeadingBom {
                    original: TextRange { start: 0, end: 3 },
                },
                LoadingMapSegment::Original {
                    loaded: TextRange { start: 0, end: 0 },
                    original: TextRange { start: 3, end: 3 },
                },
            ]
        );
        assert_eq!(
            loading_map.original_offset_for_loaded(loaded.source_id, 0),
            Ok(3)
        );
    }

    #[test]
    fn disk_source_loader_preserves_non_leading_bom() {
        let package = PackageFixture::new();
        package.write("src/groups/non_leading_bom.miz", "alpha\u{feff}beta");
        let loader = DiskSourceLoader::new(package.root());

        let loaded = loader
            .load(
                snapshot_id(24),
                disk_source_input("src/groups/non_leading_bom.miz"),
                &InMemorySessionIdAllocator::new(),
            )
            .unwrap();

        assert_eq!(loaded.text.as_ref(), "alpha\u{feff}beta");
        assert!(loaded.loading_map.is_none());
    }

    #[test]
    fn disk_source_loader_normalizes_crlf_but_preserves_lone_cr() {
        let package = PackageFixture::new();
        package.write("src/groups/newlines.miz", "alpha\r\nbeta\rgamma\r\n");
        let loader = DiskSourceLoader::new(package.root());

        let loaded = loader
            .load(
                snapshot_id(25),
                disk_source_input("src/groups/newlines.miz"),
                &InMemorySessionIdAllocator::new(),
            )
            .unwrap();

        assert_eq!(loaded.text.as_ref(), "alpha\nbeta\rgamma\n");
        assert_eq!(loaded.source_hash, hash_text("alpha\nbeta\rgamma\n"));
        assert_ne!(loaded.source_hash, hash_text("alpha\r\nbeta\rgamma\r\n"));
        assert_eq!(loaded.line_map.line_starts(), &[0, 6, 17]);
        assert_eq!(
            loaded.loading_map.as_ref().map(|map| &map.segments),
            Some(&vec![
                LoadingMapSegment::Original {
                    loaded: TextRange { start: 0, end: 5 },
                    original: TextRange { start: 0, end: 5 },
                },
                LoadingMapSegment::NormalizedNewline {
                    loaded: TextRange { start: 5, end: 6 },
                    original: TextRange { start: 5, end: 7 },
                },
                LoadingMapSegment::Original {
                    loaded: TextRange { start: 6, end: 16 },
                    original: TextRange { start: 7, end: 17 },
                },
                LoadingMapSegment::NormalizedNewline {
                    loaded: TextRange { start: 16, end: 17 },
                    original: TextRange { start: 17, end: 19 },
                },
            ])
        );
    }

    #[test]
    fn disk_source_loader_combines_leading_bom_and_crlf_loading_map_offsets() {
        let package = PackageFixture::new();
        package.write_bytes("src/groups/bom_crlf.miz", b"\xef\xbb\xbfalpha\r\nbeta");
        let loader = DiskSourceLoader::new(package.root());

        let loaded = loader
            .load(
                snapshot_id(26),
                disk_source_input("src/groups/bom_crlf.miz"),
                &InMemorySessionIdAllocator::new(),
            )
            .unwrap();

        assert_eq!(loaded.text.as_ref(), "alpha\nbeta");
        assert_eq!(loaded.source_hash, hash_text("alpha\nbeta"));
        assert_ne!(loaded.source_hash, hash_text("\u{feff}alpha\r\nbeta"));
        let loading_map = loaded
            .loading_map
            .as_ref()
            .expect("combined normalization should emit a loading map");
        assert_eq!(
            loading_map.segments,
            vec![
                LoadingMapSegment::RemovedLeadingBom {
                    original: TextRange { start: 0, end: 3 },
                },
                LoadingMapSegment::Original {
                    loaded: TextRange { start: 0, end: 5 },
                    original: TextRange { start: 3, end: 8 },
                },
                LoadingMapSegment::NormalizedNewline {
                    loaded: TextRange { start: 5, end: 6 },
                    original: TextRange { start: 8, end: 10 },
                },
                LoadingMapSegment::Original {
                    loaded: TextRange { start: 6, end: 10 },
                    original: TextRange { start: 10, end: 14 },
                },
            ]
        );
        assert_eq!(
            loading_map.original_offset_for_loaded(loaded.source_id, 0),
            Ok(3)
        );
        assert_eq!(
            loading_map.original_offset_for_loaded(loaded.source_id, 5),
            Ok(8)
        );
        assert_eq!(
            loading_map.original_offset_for_loaded(loaded.source_id, 6),
            Ok(10)
        );
    }

    #[test]
    fn disk_source_loader_reads_through_normalized_dot_component_path() {
        let package = PackageFixture::new();
        package.write("src/groups/dotted.miz", "environ\n");
        let loader = DiskSourceLoader::new(package.root());

        let loaded = loader
            .load(
                snapshot_id(27),
                disk_source_input("src/./groups/../groups/dotted.miz"),
                &InMemorySessionIdAllocator::new(),
            )
            .unwrap();

        assert_eq!(loaded.normalized_path, path("src/groups/dotted.miz"));
        assert_eq!(loaded.text.as_ref(), "environ\n");
    }

    #[test]
    fn disk_source_loader_rejects_paths_outside_package_root() {
        let package = PackageFixture::new();
        package.write_outside("outside.miz", "environ\n");
        let loader = DiskSourceLoader::new(package.root());

        let error = loader
            .load(
                snapshot_id(28),
                source_input(SourceOriginInput::Disk {
                    path: package.outside_path("outside.miz"),
                }),
                &InMemorySessionIdAllocator::new(),
            )
            .expect_err("outside package path should be rejected");

        assert!(matches!(
            error,
            SourceLoadError::SourcePathOutsidePackageRoot { .. }
        ));
    }

    #[test]
    fn disk_source_loader_does_not_preimplement_non_disk_origins() {
        let package = PackageFixture::new();
        let loader = DiskSourceLoader::new(package.root());

        let error = loader
            .load(
                snapshot_id(29),
                source_input(SourceOriginInput::OpenBuffer {
                    uri: "file:///package/src/basic.miz".to_owned(),
                    version: 1,
                    text: Arc::from("environ\n"),
                }),
                &InMemorySessionIdAllocator::new(),
            )
            .expect_err("disk-only loader should reject open buffers");

        assert!(matches!(
            error,
            SourceLoadError::UnsupportedSourceOrigin {
                origin: SourceOriginKind::OpenBuffer
            }
        ));
    }

    #[test]
    fn source_load_error_variants_and_error_sources_are_available() {
        let allocation = SourceLoadError::SourceIdAllocation {
            error: IdError::AllocatorOverflow,
        };
        let invalid_path = SourceLoadError::InvalidSourcePath {
            error: SourcePathError::InvalidNamespaceComponent {
                component: "bad-name".to_owned(),
            },
        };
        let duplicate = SourceLoadError::DuplicateModulePath {
            package_id: PackageId::new("mml"),
            module_path: ModulePath::new("groups.basic"),
        };
        let stale = SourceLoadError::StaleLspDocumentVersion {
            expected: 12,
            actual: 11,
        };
        let unmapped = SourceLoadError::UnmappedOpenBufferUri {
            uri: "untitled:basic".to_owned(),
        };
        let missing_generator = SourceLoadError::GeneratedSourceWithoutMetadata {
            module_path: ModulePath::new("generated.basic"),
        };
        let invalid_utf8 = SourceLoadError::InvalidUtf8 { path: None };
        let unreadable = SourceLoadError::UnreadableSourceFile {
            path: PathBuf::from("src/missing.miz"),
            kind: io::ErrorKind::NotFound,
        };
        let unsupported_origin = SourceLoadError::UnsupportedSourceOrigin {
            origin: SourceOriginKind::Generated,
        };

        assert!(std::error::Error::source(&allocation).is_some());
        assert!(std::error::Error::source(&invalid_path).is_some());
        assert!(matches!(
            duplicate,
            SourceLoadError::DuplicateModulePath {
                package_id,
                module_path,
            } if package_id.as_str() == "mml" && module_path.as_str() == "groups.basic"
        ));
        assert!(matches!(
            stale,
            SourceLoadError::StaleLspDocumentVersion {
                expected: 12,
                actual: 11
            }
        ));
        assert!(matches!(
            unmapped,
            SourceLoadError::UnmappedOpenBufferUri { uri } if uri == "untitled:basic"
        ));
        assert!(matches!(
            missing_generator,
            SourceLoadError::GeneratedSourceWithoutMetadata { module_path }
                if module_path.as_str() == "generated.basic"
        ));
        assert!(matches!(
            invalid_utf8,
            SourceLoadError::InvalidUtf8 { path: None }
        ));
        assert!(matches!(
            unreadable,
            SourceLoadError::UnreadableSourceFile {
                kind: io::ErrorKind::NotFound,
                ..
            }
        ));
        assert!(matches!(
            unsupported_origin,
            SourceLoadError::UnsupportedSourceOrigin {
                origin: SourceOriginKind::Generated
            }
        ));
    }

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

    fn source_input(origin: SourceOriginInput) -> SourceInput {
        SourceInput {
            package_id: PackageId::new("mml"),
            module_path: ModulePath::new("groups.basic"),
            normalized_path: path("src/groups/basic.miz"),
            edition: Edition::new("2026"),
            origin,
        }
    }

    fn disk_source_input(relative_path: &str) -> SourceInput {
        let mut input = source_input(SourceOriginInput::Disk {
            path: PathBuf::from(relative_path),
        });
        input.normalized_path = path(relative_path);
        input
    }

    fn loaded_source(source_id: SourceId, origin: SourceOrigin, text: &str) -> LoadedSource {
        LoadedSource {
            source_id,
            package_id: PackageId::new("mml"),
            module_path: ModulePath::new("groups.basic"),
            normalized_path: path("src/groups/basic.miz"),
            text: Arc::from(text),
            source_hash: hash_text(text),
            edition: Edition::new("2026"),
            origin,
            line_map: LineMap::new(source_id, text),
            loading_map: None,
        }
    }

    fn snapshot_id(byte: u8) -> BuildSnapshotId {
        let hex = format!("{byte:02x}").repeat(32);
        BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{hex}"
        ))
        .unwrap()
    }

    #[derive(Debug, Default)]
    struct TextOnlyLoader {
        disk_text: Option<Arc<str>>,
    }

    impl TextOnlyLoader {
        fn with_disk_text(text: &str) -> Self {
            Self {
                disk_text: Some(Arc::from(text)),
            }
        }
    }

    impl SourceLoader for TextOnlyLoader {
        fn load(
            &self,
            snapshot: BuildSnapshotId,
            input: SourceInput,
            ids: &dyn SessionIdAllocator,
        ) -> Result<LoadedSource, SourceLoadError> {
            let source_id = ids
                .next_source_id(snapshot)
                .map_err(|error| SourceLoadError::SourceIdAllocation { error })?;
            let SourceInput {
                package_id,
                module_path,
                normalized_path,
                edition,
                origin,
            } = input;
            let (text, origin) = match origin {
                SourceOriginInput::Disk { path } => (
                    self.disk_text.clone().ok_or({
                        SourceLoadError::UnreadableSourceFile {
                            path,
                            kind: io::ErrorKind::Unsupported,
                        }
                    })?,
                    SourceOrigin::Disk,
                ),
                SourceOriginInput::OpenBuffer { version, text, .. } => {
                    (text, SourceOrigin::OpenBuffer { version })
                }
                SourceOriginInput::Generated {
                    generator, text, ..
                } => (text, SourceOrigin::Generated { generator }),
            };
            let source_hash = self.hash_text(&text);
            let line_map = LineMap::new(source_id, &text);

            Ok(LoadedSource {
                source_id,
                package_id,
                module_path,
                normalized_path,
                text,
                source_hash,
                edition,
                origin,
                line_map,
                loading_map: None,
            })
        }
    }

    #[derive(Debug)]
    struct RecordingAllocator {
        inner: InMemorySessionIdAllocator,
        source_snapshots: Mutex<Vec<BuildSnapshotId>>,
    }

    impl RecordingAllocator {
        fn new() -> Self {
            Self {
                inner: InMemorySessionIdAllocator::new(),
                source_snapshots: Mutex::new(Vec::new()),
            }
        }

        fn source_snapshots(&self) -> Vec<BuildSnapshotId> {
            self.source_snapshots
                .lock()
                .expect("recording allocator mutex poisoned")
                .clone()
        }
    }

    impl SessionIdAllocator for RecordingAllocator {
        fn next_session_id(&self) -> Result<BuildSessionId, IdError> {
            self.inner.next_session_id()
        }

        fn next_request_id(&self) -> Result<BuildRequestId, IdError> {
            self.inner.next_request_id()
        }

        fn next_source_id(&self, snapshot: BuildSnapshotId) -> Result<SourceId, IdError> {
            self.source_snapshots
                .lock()
                .expect("recording allocator mutex poisoned")
                .push(snapshot);
            self.inner.next_source_id(snapshot)
        }

        fn next_source_map_id(&self, snapshot: BuildSnapshotId) -> Result<SourceMapId, IdError> {
            self.inner.next_source_map_id(snapshot)
        }

        fn next_lease_id(&self, snapshot: BuildSnapshotId) -> Result<SnapshotLeaseId, IdError> {
            self.inner.next_lease_id(snapshot)
        }
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

        fn write_bytes(&self, relative: &str, content: &[u8]) {
            let path = self.root.join(relative);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("parent directory should be created");
            }
            fs::write(path, content).expect("fixture bytes should be written");
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
