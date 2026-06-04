use crate::identity::is_language_identifier;
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
        expected_version: LspDocumentVersion,
        actual_version: LspDocumentVersion,
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
    pub generated_anchor: Option<SourceAnchor>,
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

    fn normalize_open_buffer_uri(
        &self,
        uri: &DocumentUri,
    ) -> Result<NormalizedPath, SourceLoadError> {
        let path = file_path_from_document_uri(uri)
            .ok_or_else(|| SourceLoadError::UnmappedOpenBufferUri { uri: uri.clone() })?;
        normalize_source_path(&self.package_root, &path).map_err(|error| match error {
            SourcePathError::OutsidePackageRoot { .. } => {
                SourceLoadError::UnmappedOpenBufferUri { uri: uri.clone() }
            }
            error => SourceLoadError::from_source_path_error(&self.package_root, error),
        })
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
            normalized_path,
            edition,
            origin,
        } = input;
        let common = LoadedSourceCommon {
            package_id,
            module_path,
            normalized_path,
            edition,
        };

        match origin {
            SourceOriginInput::Disk { path } => {
                let normalized_path = self.normalize_path(&self.package_root, &path)?;
                let read_path = self.package_root.join(normalized_path.as_str());
                let bytes = fs::read(&read_path).map_err(|error| {
                    SourceLoadError::UnreadableSourceFile {
                        path: read_path.clone(),
                        kind: error.kind(),
                    }
                })?;
                let loaded_text = normalize_disk_source_bytes(&read_path, &bytes)?;
                let loading_origin = LoadingOrigin::DiskBytes {
                    normalized_path: normalized_path.clone(),
                };
                assemble_loaded_source(
                    snapshot,
                    ids,
                    common.with_normalized_path(normalized_path),
                    loaded_text,
                    SourceOrigin::Disk,
                    Some(loading_origin),
                    None,
                )
            }
            SourceOriginInput::OpenBuffer {
                uri,
                expected_version,
                actual_version,
                text,
            } => {
                if expected_version < 0 {
                    return Err(SourceLoadError::StaleLspDocumentVersion {
                        expected: 0,
                        actual: expected_version,
                    });
                }
                if actual_version != expected_version {
                    return Err(SourceLoadError::StaleLspDocumentVersion {
                        expected: expected_version,
                        actual: actual_version,
                    });
                }

                let normalized_path = self.normalize_open_buffer_uri(&uri)?;
                let loaded_text = normalize_source_text(&text);
                assemble_loaded_source(
                    snapshot,
                    ids,
                    common.with_normalized_path(normalized_path),
                    loaded_text,
                    SourceOrigin::OpenBuffer {
                        version: actual_version,
                    },
                    Some(LoadingOrigin::OpenBufferText {
                        uri,
                        version: actual_version,
                    }),
                    None,
                )
            }
            SourceOriginInput::Generated {
                generator,
                text,
                anchor,
            } => {
                if generator.as_str().trim().is_empty() {
                    return Err(SourceLoadError::GeneratedSourceWithoutMetadata {
                        module_path: common.module_path.clone(),
                    });
                }

                assemble_loaded_source_from_text(
                    snapshot,
                    ids,
                    common,
                    text,
                    SourceOrigin::Generated { generator },
                    None,
                    anchor,
                )
            }
        }
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
struct NormalizedLoadedText {
    text: String,
    loading_segments: Option<Vec<LoadingMapSegment>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LoadedSourceCommon {
    package_id: PackageId,
    module_path: ModulePath,
    normalized_path: NormalizedPath,
    edition: Edition,
}

impl LoadedSourceCommon {
    fn with_normalized_path(mut self, normalized_path: NormalizedPath) -> Self {
        self.normalized_path = normalized_path;
        self
    }
}

fn assemble_loaded_source(
    snapshot: BuildSnapshotId,
    ids: &dyn SessionIdAllocator,
    common: LoadedSourceCommon,
    loaded_text: NormalizedLoadedText,
    origin: SourceOrigin,
    loading_origin: Option<LoadingOrigin>,
    generated_anchor: Option<SourceAnchor>,
) -> Result<LoadedSource, SourceLoadError> {
    let NormalizedLoadedText {
        text,
        loading_segments,
    } = loaded_text;
    assemble_loaded_source_from_text(
        snapshot,
        ids,
        common,
        Arc::from(text),
        origin,
        loading_origin.zip(loading_segments),
        generated_anchor,
    )
}

fn assemble_loaded_source_from_text(
    snapshot: BuildSnapshotId,
    ids: &dyn SessionIdAllocator,
    common: LoadedSourceCommon,
    text: Arc<str>,
    origin: SourceOrigin,
    loading_map_input: Option<(LoadingOrigin, Vec<LoadingMapSegment>)>,
    generated_anchor: Option<SourceAnchor>,
) -> Result<LoadedSource, SourceLoadError> {
    let source_hash = hash_text(&text);
    let source_id = allocate_source_id(snapshot, ids)?;
    let line_map = LineMap::new(source_id, &text);
    let loading_map = loading_map_input
        .map(|(origin, segments)| LoadingMap::new(source_id, &text, origin, segments));

    Ok(LoadedSource {
        source_id,
        package_id: common.package_id,
        module_path: common.module_path,
        normalized_path: common.normalized_path,
        text,
        source_hash,
        edition: common.edition,
        origin,
        line_map,
        loading_map,
        generated_anchor,
    })
}

fn normalize_disk_source_bytes(
    path: &Path,
    bytes: &[u8],
) -> Result<NormalizedLoadedText, SourceLoadError> {
    let text = std::str::from_utf8(bytes).map_err(|_| SourceLoadError::InvalidUtf8 {
        path: Some(path.to_owned()),
    })?;
    Ok(normalize_source_text(text))
}

fn normalize_source_text(text: &str) -> NormalizedLoadedText {
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
            return NormalizedLoadedText {
                text: text.to_owned(),
                loading_segments: None,
            };
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
        return NormalizedLoadedText {
            text: source_text.to_owned(),
            loading_segments: Some(segments),
        };
    }

    let normalized = normalize_crlf_to_lf(source_text, original_base, &mut segments);
    NormalizedLoadedText {
        text: normalized,
        loading_segments: Some(segments),
    }
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

fn allocate_source_id(
    snapshot: BuildSnapshotId,
    ids: &dyn SessionIdAllocator,
) -> Result<SourceId, SourceLoadError> {
    ids.next_source_id(snapshot)
        .map_err(|error| SourceLoadError::SourceIdAllocation { error })
}

fn file_path_from_document_uri(uri: &str) -> Option<PathBuf> {
    let rest = uri.strip_prefix("file://")?;
    let (authority, path) = if rest.starts_with('/') {
        ("", rest)
    } else {
        let slash = rest.find('/')?;
        rest.split_at(slash)
    };
    if authority.contains(['?', '#']) || path.contains(['?', '#']) {
        return None;
    }
    platform_file_path_from_uri_parts(authority, &percent_decode_uri_path(path)?)
}

#[cfg(windows)]
fn platform_file_path_from_uri_parts(authority: &str, path: &str) -> Option<PathBuf> {
    if authority.is_empty() || authority.eq_ignore_ascii_case("localhost") {
        return Some(PathBuf::from(windows_drive_path_from_uri_path(path)?));
    }

    let path = path.strip_prefix('/')?;
    if path.is_empty() {
        return None;
    }
    let mut unc_path = String::from(r"\\");
    unc_path.push_str(authority);
    unc_path.push('\\');
    unc_path.push_str(&path.replace('/', "\\"));
    Some(PathBuf::from(unc_path))
}

#[cfg(windows)]
fn windows_drive_path_from_uri_path(path: &str) -> Option<String> {
    let path = path.strip_prefix('/')?;
    let bytes = path.as_bytes();
    if bytes.len() < 2 || bytes[1] != b':' || !bytes[0].is_ascii_alphabetic() {
        return None;
    }
    Some(path.replace('/', "\\"))
}

#[cfg(not(windows))]
fn platform_file_path_from_uri_parts(authority: &str, path: &str) -> Option<PathBuf> {
    if !authority.is_empty() && !authority.eq_ignore_ascii_case("localhost") {
        return None;
    }
    Some(PathBuf::from(path))
}

fn percent_decode_uri_path(path: &str) -> Option<String> {
    let mut decoded = Vec::with_capacity(path.len());
    let mut bytes = path.bytes();
    while let Some(byte) = bytes.next() {
        if byte == b'%' {
            let high = bytes.next()?;
            let low = bytes.next()?;
            decoded.push(hex_value(high)? << 4 | hex_value(low)?);
        } else {
            decoded.push(byte);
        }
    }
    String::from_utf8(decoded).ok()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
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
    let namespace_count = components.len();
    for (index, component) in components.iter().enumerate().skip(1) {
        let namespace_component = if index + 1 == namespace_count {
            component.strip_suffix(".miz").unwrap_or(component)
        } else {
            component.as_str()
        };
        if !is_identifier_shaped(namespace_component) {
            return Err(SourcePathError::InvalidNamespaceComponent {
                component: namespace_component.to_owned(),
            });
        }
    }
    Ok(())
}

fn is_identifier_shaped(value: &str) -> bool {
    is_language_identifier(value)
}

#[cfg(test)]
mod tests;
