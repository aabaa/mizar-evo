//! Canonical store primitives shared by artifact schemas.
//!
//! Canonical behavior is specified in the
//! [store design spec](../../../../doc/design/mizar-artifact/en/store.md).

use std::{
    collections::BTreeMap,
    error::Error,
    fmt, fs, io,
    path::{Path, PathBuf},
    str::FromStr,
    sync::atomic::{AtomicU64, Ordering},
};

use mizar_session::{Hash, hash_text};

/// Hash construction label for artifact-framed hashes.
pub const ARTIFACT_HASH_CONSTRUCTION: &str = "mizar-artifact/artifact-framed-hash-text/v1";

const HASH_FRAME_VERSION: &str = "mizar-artifact/hash-frame/v1";
const TEMP_FILE_PREFIX: &str = ".mizar-artifact-tmp";
const TEMP_FILE_ATTEMPTS: usize = 32;

static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Schema-independent canonical JSON value used by published artifact schemas.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CanonicalJson {
    /// JSON `null`.
    Null,
    /// JSON boolean.
    Bool(bool),
    /// JSON integer number.
    Integer(i64),
    /// JSON string.
    String(String),
    /// JSON array.
    Array(Vec<CanonicalJson>),
    /// JSON object with sorted, unique member names.
    Object(BTreeMap<String, CanonicalJson>),
}

/// Errors produced while constructing canonical JSON values.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CanonicalJsonError {
    /// An object field name appeared more than once.
    DuplicateObjectKey { key: String },
}

/// A schema version in `major.minor` form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SchemaVersion {
    major: u16,
    minor: u16,
}

/// Errors produced while parsing a schema version string.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SchemaVersionParseError {
    /// The version string does not have `major.minor` shape.
    Malformed { actual: String },
}

/// Policy for minor-version compatibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum MinorVersionPolicy {
    /// Accept versions up to the supported minor version.
    UpToSupported,
    /// Accept newer minor versions after the schema declares forward compatibility.
    AllowNewer,
}

/// Supported schema-version range for one schema family.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaVersionSupport {
    family: String,
    major: u16,
    max_minor: u16,
    minor_policy: MinorVersionPolicy,
}

/// Errors produced while checking schema-version compatibility.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SchemaVersionError {
    /// The artifact did not contain a `schema_version` field.
    Missing {
        /// Diagnostic context shared by all schema-version errors.
        context: SchemaVersionErrorContext,
    },
    /// The artifact version could not be parsed.
    Malformed {
        /// Diagnostic context shared by all schema-version errors.
        context: SchemaVersionErrorContext,
        /// The malformed version string.
        actual: String,
    },
    /// The artifact major version differs from the supported major version.
    MajorMismatch {
        /// Diagnostic context shared by all schema-version errors.
        context: SchemaVersionErrorContext,
        /// Supported major version.
        expected: u16,
        /// Actual major version.
        actual: u16,
        /// Full actual schema version.
        actual_version: SchemaVersion,
    },
    /// The artifact minor version is newer than the supported range.
    MinorTooNew {
        /// Diagnostic context shared by all schema-version errors.
        context: SchemaVersionErrorContext,
        /// Highest directly supported minor version.
        supported: u16,
        /// Actual minor version.
        actual: u16,
        /// Full actual schema version.
        actual_version: SchemaVersion,
    },
}

/// Diagnostic context shared by schema-version compatibility errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaVersionErrorContext {
    family: String,
    supported_range: String,
    artifact_path: Option<String>,
}

/// Artifact hash class used for domain separation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum HashClass {
    /// Dependency-facing exported interface hash.
    Interface,
    /// Full stable published projection hash.
    Implementation,
    /// Projected diagnostics and explanation-handle hash.
    Diagnostic,
    /// Published artifact equivalence hash.
    Artifact,
}

/// Object-key path used to exclude local metadata from hash inputs.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FieldPath {
    segments: Vec<String>,
}

/// Errors produced while constructing field paths.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum FieldPathError {
    /// Field paths must contain at least one object-key segment.
    Empty,
}

/// Portable artifact-root-relative published path.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, std::hash::Hash)]
pub struct PublishedArtifactPath {
    relative: String,
}

/// A completed store-level write result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishedArtifactWrite {
    /// Published path relative to the artifact root.
    pub path: PublishedArtifactPath,
    /// Store-level artifact hash over canonical content after exclusions.
    pub artifact_hash: Hash,
}

/// A completed store-level read result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishedArtifactRead {
    /// Published path relative to the artifact root.
    pub path: PublishedArtifactPath,
    /// Canonical JSON value reconstructed from disk.
    pub value: CanonicalJson,
    /// Store-level artifact hash when the caller supplied a hash domain.
    pub artifact_hash: Option<Hash>,
}

/// Additional validation requested during a store-level read.
#[derive(Debug, Clone, Copy, Default)]
pub struct PublishedArtifactReadOptions<'a> {
    /// Store-level artifact hash domain to compute while reading.
    pub artifact_hash_domain: Option<&'a CanonicalHashDomain>,
    /// Hash-excluded paths declared by the referenced schema.
    pub hash_excluded_paths: &'a [FieldPath],
    /// Expected artifact hash from a manifest entry.
    pub expected_artifact_hash: Option<Hash>,
}

/// Location in artifact text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArtifactTextLocation {
    /// Zero-based byte offset.
    pub byte_offset: usize,
    /// One-based line number.
    pub line: usize,
    /// One-based Unicode scalar column.
    pub column: usize,
}

/// Path validation failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PublishedPathError {
    /// The path is empty.
    Empty,
    /// The path is absolute.
    Absolute,
    /// The path uses a shell-home prefix.
    HomeRelative,
    /// The path contains a backslash separator.
    Backslash,
    /// The path contains an empty segment.
    EmptySegment,
    /// The path contains a `.` segment.
    CurrentSegment,
    /// The path contains a `..` segment.
    ParentSegment,
    /// The path contains a drive-prefix or colon segment.
    DrivePrefix,
    /// The path resolves outside the artifact root.
    RootEscape,
    /// The final path is a symlink.
    SymlinkEscape,
}

/// Filesystem operation that failed while reading or writing an artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum StoreIoOperation {
    /// Create an artifact root or parent directory.
    CreateDirectory,
    /// Resolve a path to its canonical filesystem location.
    Canonicalize,
    /// Open a temporary artifact file.
    OpenTemporary,
    /// Write a temporary artifact file.
    WriteTemporary,
    /// Flush a temporary artifact file.
    FlushTemporary,
    /// Read a temporary artifact file for post-write validation.
    VerifyTemporary,
    /// Rename a temporary artifact file to the final path.
    Rename,
    /// Flush the containing directory.
    FlushDirectory,
    /// Read a published artifact file.
    Read,
}

/// Store-level I/O and corruption errors.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum StoreIoError {
    /// Published path validation failed.
    Path {
        /// Path involved in the error.
        path: String,
        /// Validation failure reason.
        reason: PublishedPathError,
    },
    /// A filesystem operation failed.
    Io {
        /// Path involved in the error.
        path: String,
        /// Operation that failed.
        operation: StoreIoOperation,
        /// Platform error kind.
        kind: io::ErrorKind,
        /// Platform error text.
        message: String,
    },
    /// A caller supplied a non-artifact hash domain for publication integrity.
    NonArtifactHashDomain {
        /// Supplied hash class.
        class: HashClass,
    },
    /// Expected artifact hash validation requires a hash domain.
    MissingHashDomain {
        /// Path involved in the error.
        path: String,
    },
    /// Post-write verification observed different bytes than were written.
    TemporaryVerificationMismatch {
        /// Temporary path involved in the error.
        path: String,
    },
    /// Store could not allocate a temporary artifact file name.
    TemporaryNameExhausted {
        /// Final path that could not be published.
        path: String,
    },
    /// A published artifact was not valid UTF-8.
    InvalidUtf8 {
        /// Artifact path.
        path: String,
        /// Location of the invalid byte.
        location: ArtifactTextLocation,
    },
    /// A published artifact was not valid canonical JSON.
    CorruptCanonicalJson {
        /// Artifact path.
        path: String,
        /// Location of the parse failure.
        location: ArtifactTextLocation,
        /// Human-readable reason.
        reason: String,
    },
    /// A published artifact parsed as JSON but was not in canonical spelling.
    NonCanonicalJson {
        /// Artifact path.
        path: String,
        /// Location of the canonicality failure.
        location: ArtifactTextLocation,
    },
    /// The computed artifact hash did not match the expected hash.
    ArtifactHashMismatch {
        /// Artifact path.
        path: String,
        /// Expected hash.
        expected: Hash,
        /// Actual hash.
        actual: Hash,
    },
}

/// Domain metadata for artifact-framed canonical hashes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalHashDomain {
    class: HashClass,
    schema_family: String,
    schema_version: SchemaVersion,
}

impl CanonicalJson {
    /// Builds a canonical JSON object while rejecting duplicate field names.
    pub fn object<I, K>(fields: I) -> Result<Self, CanonicalJsonError>
    where
        I: IntoIterator<Item = (K, Self)>,
        K: Into<String>,
    {
        let mut object = BTreeMap::new();
        for (key, value) in fields {
            let key = key.into();
            if object.insert(key.clone(), value).is_some() {
                return Err(CanonicalJsonError::DuplicateObjectKey { key });
            }
        }
        Ok(Self::Object(object))
    }

    /// Builds a canonical JSON array.
    pub fn array(values: impl IntoIterator<Item = Self>) -> Self {
        Self::Array(values.into_iter().collect())
    }

    /// Builds a canonical JSON string.
    pub fn string(value: impl Into<String>) -> Self {
        Self::String(value.into())
    }

    /// Builds a canonical JSON integer.
    pub const fn integer(value: i64) -> Self {
        Self::Integer(value)
    }

    /// Builds a canonical JSON boolean.
    pub const fn bool(value: bool) -> Self {
        Self::Bool(value)
    }

    /// Builds a canonical JSON null.
    pub const fn null() -> Self {
        Self::Null
    }
}

impl fmt::Display for CanonicalJsonError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateObjectKey { key } => {
                write!(formatter, "duplicate canonical JSON object key `{key}`")
            }
        }
    }
}

impl Error for CanonicalJsonError {}

impl SchemaVersion {
    /// Builds a schema version.
    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }

    /// Returns the major version.
    pub const fn major(self) -> u16 {
        self.major
    }

    /// Returns the minor version.
    pub const fn minor(self) -> u16 {
        self.minor
    }
}

impl fmt::Display for SchemaVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}", self.major, self.minor)
    }
}

impl FromStr for SchemaVersion {
    type Err = SchemaVersionParseError;

    fn from_str(actual: &str) -> Result<Self, Self::Err> {
        let Some((major, minor)) = actual.split_once('.') else {
            return Err(SchemaVersionParseError::Malformed {
                actual: actual.to_owned(),
            });
        };
        if minor.contains('.') || major.is_empty() || minor.is_empty() {
            return Err(SchemaVersionParseError::Malformed {
                actual: actual.to_owned(),
            });
        }
        let Ok(major) = major.parse::<u16>() else {
            return Err(SchemaVersionParseError::Malformed {
                actual: actual.to_owned(),
            });
        };
        let Ok(minor) = minor.parse::<u16>() else {
            return Err(SchemaVersionParseError::Malformed {
                actual: actual.to_owned(),
            });
        };
        Ok(Self { major, minor })
    }
}

impl fmt::Display for SchemaVersionParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed { actual } => {
                write!(formatter, "malformed schema version `{actual}`")
            }
        }
    }
}

impl Error for SchemaVersionParseError {}

impl SchemaVersionSupport {
    /// Builds a supported schema-version range.
    pub fn new(
        family: impl Into<String>,
        major: u16,
        max_minor: u16,
        minor_policy: MinorVersionPolicy,
    ) -> Self {
        Self {
            family: family.into(),
            major,
            max_minor,
            minor_policy,
        }
    }

    /// Checks an artifact schema version against this support policy.
    pub fn check(&self, actual: Option<&str>) -> Result<SchemaVersion, SchemaVersionError> {
        self.check_with_context(actual, None)
    }

    /// Checks an artifact schema version and records the artifact path in errors.
    pub fn check_at_path(
        &self,
        actual: Option<&str>,
        artifact_path: impl Into<String>,
    ) -> Result<SchemaVersion, SchemaVersionError> {
        self.check_with_context(actual, Some(artifact_path.into()))
    }

    /// Returns the human-readable supported version range.
    pub fn supported_range(&self) -> String {
        match self.minor_policy {
            MinorVersionPolicy::UpToSupported => {
                format!("{}.0..={}.{}", self.major, self.major, self.max_minor)
            }
            MinorVersionPolicy::AllowNewer => format!("{}.0..={}.x", self.major, self.major),
        }
    }

    fn check_with_context(
        &self,
        actual: Option<&str>,
        artifact_path: Option<String>,
    ) -> Result<SchemaVersion, SchemaVersionError> {
        let context = self.error_context(artifact_path);
        let Some(actual) = actual else {
            return Err(SchemaVersionError::Missing { context });
        };
        let version =
            actual
                .parse::<SchemaVersion>()
                .map_err(|_| SchemaVersionError::Malformed {
                    context: context.clone(),
                    actual: actual.to_owned(),
                })?;
        if version.major != self.major {
            return Err(SchemaVersionError::MajorMismatch {
                context,
                expected: self.major,
                actual: version.major,
                actual_version: version,
            });
        }
        if version.minor > self.max_minor && self.minor_policy == MinorVersionPolicy::UpToSupported
        {
            return Err(SchemaVersionError::MinorTooNew {
                context,
                supported: self.max_minor,
                actual: version.minor,
                actual_version: version,
            });
        }
        Ok(version)
    }

    fn error_context(&self, artifact_path: Option<String>) -> SchemaVersionErrorContext {
        SchemaVersionErrorContext {
            family: self.family.clone(),
            supported_range: self.supported_range(),
            artifact_path,
        }
    }
}

impl SchemaVersionErrorContext {
    /// Returns the schema family.
    pub fn family(&self) -> &str {
        &self.family
    }

    /// Returns the supported version range.
    pub fn supported_range(&self) -> &str {
        &self.supported_range
    }

    /// Returns the artifact path when the caller supplied one.
    pub fn artifact_path(&self) -> Option<&str> {
        self.artifact_path.as_deref()
    }
}

impl fmt::Display for SchemaVersionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing { context } => {
                write!(
                    formatter,
                    "{} artifact{} is missing schema_version; supported versions: {}",
                    context.family(),
                    display_artifact_path(context),
                    context.supported_range()
                )
            }
            Self::Malformed { context, actual } => {
                write!(
                    formatter,
                    "{} artifact{} has malformed schema_version `{actual}`; supported versions: {}",
                    context.family(),
                    display_artifact_path(context),
                    context.supported_range()
                )
            }
            Self::MajorMismatch {
                context,
                expected,
                actual,
                actual_version,
            } => {
                write!(
                    formatter,
                    "{} artifact{} schema version {actual_version} has major version {actual}, \
                     which is incompatible with supported major version {expected}; supported \
                     versions: {}",
                    context.family(),
                    display_artifact_path(context),
                    context.supported_range()
                )
            }
            Self::MinorTooNew {
                context,
                supported,
                actual,
                actual_version,
            } => {
                write!(
                    formatter,
                    "{} artifact{} schema version {actual_version} has minor version {actual}, \
                     which is newer than supported minor version {supported}; supported versions: {}",
                    context.family(),
                    display_artifact_path(context),
                    context.supported_range()
                )
            }
        }
    }
}

impl Error for SchemaVersionError {}

impl HashClass {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Interface => "interface",
            Self::Implementation => "implementation",
            Self::Diagnostic => "diagnostic",
            Self::Artifact => "artifact",
        }
    }
}

impl FieldPath {
    /// Builds an object-key field path.
    ///
    /// Paths address object keys only. Missing paths have no effect, parent
    /// exclusions remove the whole subtree, and paths do not traverse arrays.
    pub fn new<I, S>(segments: I) -> Result<Self, FieldPathError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let segments = segments.into_iter().map(Into::into).collect::<Vec<_>>();
        if segments.is_empty() {
            return Err(FieldPathError::Empty);
        }
        Ok(Self { segments })
    }

    fn segments(&self) -> &[String] {
        &self.segments
    }
}

impl fmt::Display for FieldPathError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(formatter, "hash-exclusion field path must not be empty"),
        }
    }
}

impl Error for FieldPathError {}

impl fmt::Display for HashClass {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl fmt::Display for PublishedPathError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("path must not be empty"),
            Self::Absolute => formatter.write_str("path must be artifact-root-relative"),
            Self::HomeRelative => formatter.write_str("path must not use a home-relative prefix"),
            Self::Backslash => formatter.write_str("path must use `/` separators"),
            Self::EmptySegment => formatter.write_str("path must not contain empty segments"),
            Self::CurrentSegment => formatter.write_str("path must not contain `.` segments"),
            Self::ParentSegment => formatter.write_str("path must not contain `..` segments"),
            Self::DrivePrefix => {
                formatter.write_str("path must not contain drive-prefix or colon segments")
            }
            Self::RootEscape => formatter.write_str("path resolves outside the artifact root"),
            Self::SymlinkEscape => formatter.write_str("final path must not be a symlink"),
        }
    }
}

impl fmt::Display for StoreIoOperation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CreateDirectory => formatter.write_str("create directory"),
            Self::Canonicalize => formatter.write_str("canonicalize path"),
            Self::OpenTemporary => formatter.write_str("open temporary artifact"),
            Self::WriteTemporary => formatter.write_str("write temporary artifact"),
            Self::FlushTemporary => formatter.write_str("flush temporary artifact"),
            Self::VerifyTemporary => formatter.write_str("verify temporary artifact"),
            Self::Rename => formatter.write_str("rename temporary artifact"),
            Self::FlushDirectory => formatter.write_str("flush containing directory"),
            Self::Read => formatter.write_str("read artifact"),
        }
    }
}

impl fmt::Display for StoreIoError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Path { path, reason } => write!(
                formatter,
                "`{path}` is not a valid published artifact path: {reason}"
            ),
            Self::Io {
                path,
                operation,
                kind,
                message,
            } => write!(
                formatter,
                "failed to {operation} at `{path}` ({kind:?}): {message}"
            ),
            Self::NonArtifactHashDomain { class } => write!(
                formatter,
                "store-level artifact publication requires hash class `artifact`, got `{class}`"
            ),
            Self::MissingHashDomain { path } => write!(
                formatter,
                "expected artifact hash for `{path}` cannot be checked without an artifact hash domain"
            ),
            Self::TemporaryVerificationMismatch { path } => write!(
                formatter,
                "temporary artifact `{path}` did not re-read as the bytes just written"
            ),
            Self::TemporaryNameExhausted { path } => write!(
                formatter,
                "could not allocate a temporary artifact path while publishing `{path}`"
            ),
            Self::InvalidUtf8 { path, location } => write!(
                formatter,
                "`{path}` is not UTF-8 at byte {}, line {}, column {}",
                location.byte_offset, location.line, location.column
            ),
            Self::CorruptCanonicalJson {
                path,
                location,
                reason,
            } => write!(
                formatter,
                "`{path}` is not canonical JSON at byte {}, line {}, column {}: {reason}",
                location.byte_offset, location.line, location.column
            ),
            Self::NonCanonicalJson { path, location } => write!(
                formatter,
                "`{path}` is not in canonical JSON spelling at byte {}, line {}, column {}",
                location.byte_offset, location.line, location.column
            ),
            Self::ArtifactHashMismatch {
                path,
                expected,
                actual,
            } => write!(
                formatter,
                "`{path}` artifact hash mismatch: expected {expected:?}, actual {actual:?}"
            ),
        }
    }
}

impl Error for StoreIoError {}

impl PublishedArtifactPath {
    /// Builds and validates a portable published artifact path.
    pub fn new(path: impl Into<String>) -> Result<Self, StoreIoError> {
        let relative = path.into();
        validate_published_path(&relative)?;
        Ok(Self { relative })
    }

    /// Returns the artifact-root-relative path string.
    pub fn as_str(&self) -> &str {
        &self.relative
    }

    fn join_under(&self, root: &Path) -> PathBuf {
        let mut path = root.to_path_buf();
        for segment in self.relative.split('/') {
            path.push(segment);
        }
        path
    }

    fn parent_segments(&self) -> Vec<&str> {
        let mut segments = self.relative.split('/').collect::<Vec<_>>();
        segments.pop();
        segments
    }
}

impl fmt::Display for PublishedArtifactPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.relative)
    }
}

impl FromStr for PublishedArtifactPath {
    type Err = StoreIoError;

    fn from_str(path: &str) -> Result<Self, Self::Err> {
        Self::new(path)
    }
}

impl CanonicalHashDomain {
    /// Builds a canonical hash domain.
    pub fn new(
        class: HashClass,
        schema_family: impl Into<String>,
        schema_version: SchemaVersion,
    ) -> Self {
        Self {
            class,
            schema_family: schema_family.into(),
            schema_version,
        }
    }

    /// Returns the hash class.
    pub const fn class(&self) -> HashClass {
        self.class
    }

    /// Returns the artifact-framed hash input text before it is hashed.
    pub fn hash_input(&self, value: &CanonicalJson, excluded_paths: &[FieldPath]) -> String {
        let filtered = without_field_paths(value, excluded_paths);
        let canonical_json = canonical_json_string(&filtered);
        let mut input = String::new();
        push_frame_part(&mut input, "frame", HASH_FRAME_VERSION);
        push_frame_part(&mut input, "construction", ARTIFACT_HASH_CONSTRUCTION);
        push_frame_part(&mut input, "class", self.class.as_str());
        push_frame_part(&mut input, "schema-family", &self.schema_family);
        push_frame_part(
            &mut input,
            "schema-version",
            &self.schema_version.to_string(),
        );
        push_frame_part(&mut input, "canonical-json", &canonical_json);
        input
    }

    /// Computes the artifact-framed hash for a canonical JSON value.
    pub fn hash(&self, value: &CanonicalJson, excluded_paths: &[FieldPath]) -> Hash {
        hash_text(&self.hash_input(value, excluded_paths))
    }
}

/// Builds the store-level artifact hash domain for a published schema.
pub fn artifact_hash_domain(
    schema_family: impl Into<String>,
    schema_version: SchemaVersion,
) -> CanonicalHashDomain {
    CanonicalHashDomain::new(HashClass::Artifact, schema_family, schema_version)
}

/// Atomically writes a canonical JSON published artifact under an artifact root.
pub fn write_published_artifact(
    artifact_root: impl AsRef<Path>,
    path: &PublishedArtifactPath,
    value: &CanonicalJson,
    domain: &CanonicalHashDomain,
    excluded_paths: &[FieldPath],
) -> Result<PublishedArtifactWrite, StoreIoError> {
    ensure_artifact_hash_domain(domain)?;
    let root = canonical_artifact_root_for_write(artifact_root.as_ref())?;
    let final_path = path.join_under(&root);
    let parent = ensure_parent_directories(&root, path)?;

    let artifact_hash = domain.hash(value, excluded_paths);
    let bytes = canonical_json_bytes(value);
    commit_with_temporary_candidates(
        &final_path,
        &parent,
        &bytes,
        (0..TEMP_FILE_ATTEMPTS).map(|_| temporary_path_for(&final_path)),
    )?;

    Ok(PublishedArtifactWrite {
        path: path.clone(),
        artifact_hash,
    })
}

/// Reads and validates a canonical JSON published artifact under an artifact root.
pub fn read_published_artifact(
    artifact_root: impl AsRef<Path>,
    path: &PublishedArtifactPath,
    options: PublishedArtifactReadOptions<'_>,
) -> Result<PublishedArtifactRead, StoreIoError> {
    if let Some(domain) = options.artifact_hash_domain {
        ensure_artifact_hash_domain(domain)?;
    } else if options.expected_artifact_hash.is_some() {
        return Err(StoreIoError::MissingHashDomain {
            path: path.as_str().to_owned(),
        });
    }

    let root = canonical_artifact_root_for_read(artifact_root.as_ref())?;
    let final_path = path.join_under(&root);
    let metadata = fs::symlink_metadata(&final_path)
        .map_err(|error| io_error(path.as_str(), StoreIoOperation::Read, error))?;
    if metadata.file_type().is_symlink() {
        return Err(StoreIoError::Path {
            path: path.as_str().to_owned(),
            reason: PublishedPathError::SymlinkEscape,
        });
    }
    let canonical_final = fs::canonicalize(&final_path)
        .map_err(|error| io_error(path.as_str(), StoreIoOperation::Canonicalize, error))?;
    ensure_under_root(path.as_str(), &root, &canonical_final)?;

    let bytes = fs::read(&final_path)
        .map_err(|error| io_error(path.as_str(), StoreIoOperation::Read, error))?;
    let text = std::str::from_utf8(&bytes).map_err(|error| StoreIoError::InvalidUtf8 {
        path: path.as_str().to_owned(),
        location: location_for_byte_prefix(&bytes[..error.valid_up_to()]),
    })?;
    let value = parse_canonical_json_artifact(text, path.as_str())?;
    let artifact_hash = options
        .artifact_hash_domain
        .map(|domain| domain.hash(&value, options.hash_excluded_paths));

    if let (Some(expected), Some(actual)) = (options.expected_artifact_hash, artifact_hash)
        && expected != actual
    {
        return Err(StoreIoError::ArtifactHashMismatch {
            path: path.as_str().to_owned(),
            expected,
            actual,
        });
    }

    Ok(PublishedArtifactRead {
        path: path.clone(),
        value,
        artifact_hash,
    })
}

/// Serializes a canonical JSON value as UTF-8 bytes ending in one newline.
pub fn canonical_json_bytes(value: &CanonicalJson) -> Vec<u8> {
    canonical_json_string(value).into_bytes()
}

/// Serializes a canonical JSON value as a UTF-8 string ending in one newline.
pub fn canonical_json_string(value: &CanonicalJson) -> String {
    let mut output = String::new();
    write_canonical_json(value, &mut output);
    output.push('\n');
    output
}

fn validate_published_path(path: &str) -> Result<(), StoreIoError> {
    if path.is_empty() {
        return Err(path_error(path, PublishedPathError::Empty));
    }
    if path.starts_with('/') || Path::new(path).is_absolute() {
        return Err(path_error(path, PublishedPathError::Absolute));
    }
    if path.starts_with('~') {
        return Err(path_error(path, PublishedPathError::HomeRelative));
    }
    if path.contains('\\') {
        return Err(path_error(path, PublishedPathError::Backslash));
    }
    for segment in path.split('/') {
        if segment.is_empty() {
            return Err(path_error(path, PublishedPathError::EmptySegment));
        }
        if segment == "." {
            return Err(path_error(path, PublishedPathError::CurrentSegment));
        }
        if segment == ".." {
            return Err(path_error(path, PublishedPathError::ParentSegment));
        }
        if segment.contains(':') {
            return Err(path_error(path, PublishedPathError::DrivePrefix));
        }
    }
    Ok(())
}

fn path_error(path: &str, reason: PublishedPathError) -> StoreIoError {
    StoreIoError::Path {
        path: path.to_owned(),
        reason,
    }
}

fn ensure_artifact_hash_domain(domain: &CanonicalHashDomain) -> Result<(), StoreIoError> {
    if domain.class() != HashClass::Artifact {
        return Err(StoreIoError::NonArtifactHashDomain {
            class: domain.class(),
        });
    }
    Ok(())
}

fn canonical_artifact_root_for_write(root: &Path) -> Result<PathBuf, StoreIoError> {
    fs::create_dir_all(root).map_err(|error| {
        io_error(
            root.display().to_string(),
            StoreIoOperation::CreateDirectory,
            error,
        )
    })?;
    fs::canonicalize(root).map_err(|error| {
        io_error(
            root.display().to_string(),
            StoreIoOperation::Canonicalize,
            error,
        )
    })
}

fn canonical_artifact_root_for_read(root: &Path) -> Result<PathBuf, StoreIoError> {
    fs::canonicalize(root).map_err(|error| {
        io_error(
            root.display().to_string(),
            StoreIoOperation::Canonicalize,
            error,
        )
    })
}

fn ensure_parent_directories(
    root: &Path,
    path: &PublishedArtifactPath,
) -> Result<PathBuf, StoreIoError> {
    let mut current = root.to_path_buf();
    for segment in path.parent_segments() {
        current.push(segment);
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_dir() || metadata.file_type().is_symlink() => {
                let canonical = fs::canonicalize(&current).map_err(|error| {
                    io_error(
                        current.display().to_string(),
                        StoreIoOperation::Canonicalize,
                        error,
                    )
                })?;
                ensure_under_root(path.as_str(), root, &canonical)?;
            }
            Ok(_) => {
                return Err(io_error(
                    current.display().to_string(),
                    StoreIoOperation::CreateDirectory,
                    io::Error::new(
                        io::ErrorKind::AlreadyExists,
                        "path exists but is not a directory",
                    ),
                ));
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                fs::create_dir(&current).map_err(|error| {
                    io_error(
                        current.display().to_string(),
                        StoreIoOperation::CreateDirectory,
                        error,
                    )
                })?;
                let canonical = fs::canonicalize(&current).map_err(|error| {
                    io_error(
                        current.display().to_string(),
                        StoreIoOperation::Canonicalize,
                        error,
                    )
                })?;
                ensure_under_root(path.as_str(), root, &canonical)?;
            }
            Err(error) => {
                return Err(io_error(
                    current.display().to_string(),
                    StoreIoOperation::Canonicalize,
                    error,
                ));
            }
        }
    }
    Ok(final_parent(root, path))
}

fn final_parent(root: &Path, path: &PublishedArtifactPath) -> PathBuf {
    let mut parent = root.to_path_buf();
    for segment in path.parent_segments() {
        parent.push(segment);
    }
    parent
}

fn ensure_under_root(path: &str, root: &Path, resolved: &Path) -> Result<(), StoreIoError> {
    if resolved.starts_with(root) {
        Ok(())
    } else {
        Err(path_error(path, PublishedPathError::RootEscape))
    }
}

fn temporary_path_for(final_path: &Path) -> PathBuf {
    let parent = final_path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = final_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("artifact");
    let counter = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
    parent.join(format!(
        "{TEMP_FILE_PREFIX}-{}-{counter}-{file_name}",
        std::process::id()
    ))
}

fn commit_with_temporary_candidates(
    final_path: &Path,
    parent: &Path,
    bytes: &[u8],
    candidates: impl IntoIterator<Item = PathBuf>,
) -> Result<(), StoreIoError> {
    for temp_path in candidates.into_iter().take(TEMP_FILE_ATTEMPTS) {
        let result = write_temporary_artifact(&temp_path, bytes)
            .and_then(|()| rename_temporary_artifact(&temp_path, final_path))
            .and_then(|()| flush_directory(parent));

        match result {
            Ok(()) => return Ok(()),
            Err(error) if is_temporary_name_collision(&error) => {}
            Err(error) => {
                let _ = fs::remove_file(&temp_path);
                return Err(error);
            }
        }
    }

    Err(StoreIoError::TemporaryNameExhausted {
        path: final_path.display().to_string(),
    })
}

fn is_temporary_name_collision(error: &StoreIoError) -> bool {
    matches!(
        error,
        StoreIoError::Io {
            operation: StoreIoOperation::OpenTemporary,
            kind: io::ErrorKind::AlreadyExists,
            ..
        }
    )
}

fn write_temporary_artifact(temp_path: &Path, bytes: &[u8]) -> Result<(), StoreIoError> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(temp_path)
        .map_err(|error| {
            io_error(
                temp_path.display().to_string(),
                StoreIoOperation::OpenTemporary,
                error,
            )
        })?;
    io::Write::write_all(&mut file, bytes).map_err(|error| {
        io_error(
            temp_path.display().to_string(),
            StoreIoOperation::WriteTemporary,
            error,
        )
    })?;
    file.sync_all().map_err(|error| {
        io_error(
            temp_path.display().to_string(),
            StoreIoOperation::FlushTemporary,
            error,
        )
    })?;
    drop(file);

    let observed = fs::read(temp_path).map_err(|error| {
        io_error(
            temp_path.display().to_string(),
            StoreIoOperation::VerifyTemporary,
            error,
        )
    })?;
    if observed != bytes {
        return Err(StoreIoError::TemporaryVerificationMismatch {
            path: temp_path.display().to_string(),
        });
    }
    Ok(())
}

fn rename_temporary_artifact(temp_path: &Path, final_path: &Path) -> Result<(), StoreIoError> {
    fs::rename(temp_path, final_path).map_err(|error| {
        io_error(
            final_path.display().to_string(),
            StoreIoOperation::Rename,
            error,
        )
    })
}

#[cfg(windows)]
fn flush_directory(_directory: &Path) -> Result<(), StoreIoError> {
    Ok(())
}

#[cfg(not(windows))]
fn flush_directory(directory: &Path) -> Result<(), StoreIoError> {
    let directory_file = match fs::File::open(directory) {
        Ok(file) => file,
        Err(error) if is_directory_sync_unsupported(&error) => return Ok(()),
        Err(error) => {
            return Err(io_error(
                directory.display().to_string(),
                StoreIoOperation::FlushDirectory,
                error,
            ));
        }
    };
    match directory_file.sync_all() {
        Ok(()) => Ok(()),
        Err(error) if is_directory_sync_unsupported(&error) => Ok(()),
        Err(error) => Err(io_error(
            directory.display().to_string(),
            StoreIoOperation::FlushDirectory,
            error,
        )),
    }
}

#[cfg(not(windows))]
fn is_directory_sync_unsupported(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        io::ErrorKind::Unsupported | io::ErrorKind::InvalidInput
    )
}

fn io_error(
    path: impl Into<String>,
    operation: StoreIoOperation,
    error: io::Error,
) -> StoreIoError {
    StoreIoError::Io {
        path: path.into(),
        operation,
        kind: error.kind(),
        message: error.to_string(),
    }
}

fn parse_canonical_json_artifact(text: &str, path: &str) -> Result<CanonicalJson, StoreIoError> {
    let mut parser = CanonicalJsonParser::new(text, path);
    let value = parser.parse_artifact()?;
    if canonical_json_string(&value) != text {
        return Err(StoreIoError::NonCanonicalJson {
            path: path.to_owned(),
            location: ArtifactTextLocation {
                byte_offset: 0,
                line: 1,
                column: 1,
            },
        });
    }
    Ok(value)
}

struct CanonicalJsonParser<'a> {
    text: &'a str,
    path: &'a str,
    position: usize,
}

impl<'a> CanonicalJsonParser<'a> {
    fn new(text: &'a str, path: &'a str) -> Self {
        Self {
            text,
            path,
            position: 0,
        }
    }

    fn parse_artifact(&mut self) -> Result<CanonicalJson, StoreIoError> {
        let value = self.parse_value()?;
        if self.take_byte(b'\n') {
            if self.position == self.text.len() {
                Ok(value)
            } else {
                Err(self.error_at(self.position, "unexpected content after final newline"))
            }
        } else {
            Err(self.error_at(self.position, "expected final newline"))
        }
    }

    fn parse_value(&mut self) -> Result<CanonicalJson, StoreIoError> {
        match self.peek_byte() {
            Some(b'n') => self.parse_literal("null", CanonicalJson::Null),
            Some(b't') => self.parse_literal("true", CanonicalJson::Bool(true)),
            Some(b'f') => self.parse_literal("false", CanonicalJson::Bool(false)),
            Some(b'"') => self.parse_string().map(CanonicalJson::String),
            Some(b'[') => self.parse_array(),
            Some(b'{') => self.parse_object(),
            Some(b'-' | b'0'..=b'9') => self.parse_integer().map(CanonicalJson::Integer),
            Some(_) => Err(self.error_at(self.position, "expected JSON value")),
            None => Err(self.error_at(self.position, "expected JSON value")),
        }
    }

    fn parse_literal(
        &mut self,
        literal: &str,
        value: CanonicalJson,
    ) -> Result<CanonicalJson, StoreIoError> {
        if self.text[self.position..].starts_with(literal) {
            self.position += literal.len();
            Ok(value)
        } else {
            Err(self.error_at(self.position, "malformed literal"))
        }
    }

    fn parse_integer(&mut self) -> Result<i64, StoreIoError> {
        let start = self.position;
        if self.take_byte(b'-') && self.peek_byte().is_none() {
            return Err(self.error_at(start, "malformed integer"));
        }

        match self.peek_byte() {
            Some(b'0') => {
                self.position += 1;
                if matches!(self.peek_byte(), Some(b'0'..=b'9')) {
                    return Err(self.error_at(start, "integer must use canonical spelling"));
                }
            }
            Some(b'1'..=b'9') => {
                while matches!(self.peek_byte(), Some(b'0'..=b'9')) {
                    self.position += 1;
                }
            }
            _ => return Err(self.error_at(start, "malformed integer")),
        }

        self.text[start..self.position]
            .parse::<i64>()
            .map_err(|_| self.error_at(start, "integer is outside i64 range"))
    }

    fn parse_string(&mut self) -> Result<String, StoreIoError> {
        let start = self.position;
        self.expect_byte(b'"', "expected string")?;
        let mut value = String::new();
        while let Some(byte) = self.peek_byte() {
            match byte {
                b'"' => {
                    self.position += 1;
                    return Ok(value);
                }
                b'\\' => {
                    self.position += 1;
                    value.push(self.parse_escape()?);
                }
                0x00..=0x1f => {
                    return Err(self.error_at(self.position, "unescaped control character"));
                }
                _ => {
                    let character = self.text[self.position..]
                        .chars()
                        .next()
                        .expect("peeked byte implies a character");
                    value.push(character);
                    self.position += character.len_utf8();
                }
            }
        }
        Err(self.error_at(start, "unterminated string"))
    }

    fn parse_escape(&mut self) -> Result<char, StoreIoError> {
        let escape_position = self.position.saturating_sub(1);
        let Some(byte) = self.peek_byte() else {
            return Err(self.error_at(escape_position, "unterminated escape"));
        };
        self.position += 1;
        match byte {
            b'"' => Ok('"'),
            b'\\' => Ok('\\'),
            b'b' => Ok('\u{0008}'),
            b't' => Ok('\t'),
            b'n' => Ok('\n'),
            b'f' => Ok('\u{000c}'),
            b'r' => Ok('\r'),
            b'u' => self.parse_unicode_escape(escape_position),
            _ => Err(self.error_at(escape_position, "unsupported string escape")),
        }
    }

    fn parse_unicode_escape(&mut self, escape_position: usize) -> Result<char, StoreIoError> {
        let end = self.position + 4;
        let Some(digits) = self.text.get(self.position..end) else {
            return Err(self.error_at(escape_position, "truncated unicode escape"));
        };
        if digits.len() != 4
            || !digits
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(self.error_at(escape_position, "malformed unicode escape"));
        }
        self.position = end;
        let code = u32::from_str_radix(digits, 16)
            .map_err(|_| self.error_at(escape_position, "malformed unicode escape"))?;
        if code > 0x1f {
            return Err(self.error_at(
                escape_position,
                "unicode escape is not canonical for non-control characters",
            ));
        }
        char::from_u32(code).ok_or_else(|| self.error_at(escape_position, "invalid unicode escape"))
    }

    fn parse_array(&mut self) -> Result<CanonicalJson, StoreIoError> {
        self.expect_byte(b'[', "expected array")?;
        let mut values = Vec::new();
        if self.take_byte(b']') {
            return Ok(CanonicalJson::Array(values));
        }
        loop {
            values.push(self.parse_value()?);
            if self.take_byte(b']') {
                return Ok(CanonicalJson::Array(values));
            }
            self.expect_byte(b',', "expected array separator")?;
        }
    }

    fn parse_object(&mut self) -> Result<CanonicalJson, StoreIoError> {
        self.expect_byte(b'{', "expected object")?;
        let mut fields = BTreeMap::new();
        if self.take_byte(b'}') {
            return Ok(CanonicalJson::Object(fields));
        }
        loop {
            let key_position = self.position;
            let key = self.parse_string()?;
            self.expect_byte(b':', "expected object key separator")?;
            let value = self.parse_value()?;
            if fields.insert(key.clone(), value).is_some() {
                return Err(self.error_at(key_position, &format!("duplicate object key `{key}`")));
            }
            if self.take_byte(b'}') {
                return Ok(CanonicalJson::Object(fields));
            }
            self.expect_byte(b',', "expected object separator")?;
        }
    }

    fn expect_byte(&mut self, expected: u8, reason: &str) -> Result<(), StoreIoError> {
        if self.take_byte(expected) {
            Ok(())
        } else {
            Err(self.error_at(self.position, reason))
        }
    }

    fn take_byte(&mut self, expected: u8) -> bool {
        if self.peek_byte() == Some(expected) {
            self.position += 1;
            true
        } else {
            false
        }
    }

    fn peek_byte(&self) -> Option<u8> {
        self.text.as_bytes().get(self.position).copied()
    }

    fn error_at(&self, byte_offset: usize, reason: &str) -> StoreIoError {
        StoreIoError::CorruptCanonicalJson {
            path: self.path.to_owned(),
            location: location_for_text(self.text, byte_offset),
            reason: reason.to_owned(),
        }
    }
}

fn location_for_byte_prefix(bytes: &[u8]) -> ArtifactTextLocation {
    let text = std::str::from_utf8(bytes).unwrap_or("");
    location_for_text(text, bytes.len())
}

fn location_for_text(text: &str, byte_offset: usize) -> ArtifactTextLocation {
    let bounded_offset = byte_offset.min(text.len());
    let mut line = 1;
    let mut column = 1;
    for character in text[..bounded_offset].chars() {
        if character == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    ArtifactTextLocation {
        byte_offset: bounded_offset,
        line,
        column,
    }
}

fn write_canonical_json(value: &CanonicalJson, output: &mut String) {
    match value {
        CanonicalJson::Null => output.push_str("null"),
        CanonicalJson::Bool(value) => output.push_str(if *value { "true" } else { "false" }),
        CanonicalJson::Integer(value) => output.push_str(&value.to_string()),
        CanonicalJson::String(value) => write_json_string(value, output),
        CanonicalJson::Array(values) => {
            output.push('[');
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                write_canonical_json(value, output);
            }
            output.push(']');
        }
        CanonicalJson::Object(fields) => {
            output.push('{');
            for (index, (key, value)) in fields.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                write_json_string(key, output);
                output.push(':');
                write_canonical_json(value, output);
            }
            output.push('}');
        }
    }
}

fn write_json_string(value: &str, output: &mut String) {
    output.push('"');
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\u{08}' => output.push_str("\\b"),
            '\t' => output.push_str("\\t"),
            '\n' => output.push_str("\\n"),
            '\u{0c}' => output.push_str("\\f"),
            '\r' => output.push_str("\\r"),
            '\u{00}'..='\u{1f}' => push_control_escape(character, output),
            _ => output.push(character),
        }
    }
    output.push('"');
}

fn push_control_escape(character: char, output: &mut String) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let code = character as usize;
    output.push_str("\\u00");
    output.push(HEX[(code >> 4) & 0xf] as char);
    output.push(HEX[code & 0xf] as char);
}

fn without_field_paths(value: &CanonicalJson, excluded_paths: &[FieldPath]) -> CanonicalJson {
    let path_slices = excluded_paths
        .iter()
        .map(FieldPath::segments)
        .collect::<Vec<_>>();
    without_field_path_slices(value, &path_slices)
}

fn without_field_path_slices(value: &CanonicalJson, excluded_paths: &[&[String]]) -> CanonicalJson {
    let CanonicalJson::Object(fields) = value else {
        return value.clone();
    };

    let mut filtered = BTreeMap::new();
    for (key, value) in fields {
        let matching_paths = excluded_paths
            .iter()
            .filter_map(|path| path.split_first().filter(|(head, _)| *head == key))
            .map(|(_, tail)| tail)
            .collect::<Vec<_>>();

        if matching_paths.iter().any(|path| path.is_empty()) {
            continue;
        }

        if matching_paths.is_empty() {
            filtered.insert(key.clone(), value.clone());
        } else {
            filtered.insert(
                key.clone(),
                without_field_path_slices(value, &matching_paths),
            );
        }
    }
    CanonicalJson::Object(filtered)
}

fn push_frame_part(output: &mut String, label: &str, value: &str) {
    output.push_str(label);
    output.push(':');
    output.push_str(&value.len().to_string());
    output.push(':');
    output.push_str(value);
    output.push('\n');
}

fn display_artifact_path(context: &SchemaVersionErrorContext) -> String {
    context
        .artifact_path()
        .map(|path| format!(" at `{path}`"))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{
        CanonicalHashDomain, CanonicalJson, FieldPath, HashClass, MinorVersionPolicy,
        PublishedArtifactPath, PublishedArtifactReadOptions, PublishedPathError, SchemaVersion,
        SchemaVersionError, SchemaVersionSupport, StoreIoError, StoreIoOperation, TEMP_FILE_PREFIX,
        artifact_hash_domain, canonical_json_string, commit_with_temporary_candidates,
        read_published_artifact, write_published_artifact,
    };
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
    };

    static TEST_ROOT_COUNTER: AtomicU64 = AtomicU64::new(1);

    #[test]
    fn canonical_json_sorts_object_keys_and_ends_with_newline() {
        let first = CanonicalJson::object([
            ("zeta", CanonicalJson::integer(2)),
            ("alpha", CanonicalJson::integer(1)),
        ])
        .expect("unique keys");
        let second = CanonicalJson::object([
            ("alpha", CanonicalJson::integer(1)),
            ("zeta", CanonicalJson::integer(2)),
        ])
        .expect("unique keys");

        assert_eq!(
            canonical_json_string(&first),
            canonical_json_string(&second)
        );
        assert_eq!(canonical_json_string(&first), "{\"alpha\":1,\"zeta\":2}\n");
    }

    #[test]
    fn canonical_json_uses_required_string_escapes_only() {
        let value = CanonicalJson::string(
            "quote:\" slash:\\ backspace:\u{0008} tab:\t newline:\n formfeed:\u{000c} \
             carriage:\r ctrl:\u{001f} snow:\u{2603}",
        );

        assert_eq!(
            canonical_json_string(&value),
            "\"quote:\\\" slash:\\\\ backspace:\\b tab:\\t newline:\\n formfeed:\\f \
             carriage:\\r ctrl:\\u001f snow:\u{2603}\"\n"
        );
    }

    #[test]
    fn canonical_json_rejects_duplicate_object_keys() {
        let error = CanonicalJson::object([
            ("alpha", CanonicalJson::integer(1)),
            ("alpha", CanonicalJson::integer(2)),
        ])
        .expect_err("duplicate key must be rejected");

        assert_eq!(
            error.to_string(),
            "duplicate canonical JSON object key `alpha`"
        );
    }

    #[test]
    fn published_paths_reject_non_portable_or_escaping_spelling() {
        let cases = [
            ("", PublishedPathError::Empty),
            ("/absolute.json", PublishedPathError::Absolute),
            ("~/artifact.json", PublishedPathError::HomeRelative),
            ("module\\artifact.json", PublishedPathError::Backslash),
            ("module//artifact.json", PublishedPathError::EmptySegment),
            ("module/./artifact.json", PublishedPathError::CurrentSegment),
            ("module/../artifact.json", PublishedPathError::ParentSegment),
            ("C:/artifact.json", PublishedPathError::DrivePrefix),
            ("module/C:/artifact.json", PublishedPathError::DrivePrefix),
            ("module/artifact:ads.json", PublishedPathError::DrivePrefix),
        ];

        for (raw, reason) in cases {
            let error = PublishedArtifactPath::new(raw).expect_err("path must be rejected");
            assert!(
                matches!(error, StoreIoError::Path { reason: actual, .. } if actual == reason),
                "{raw} should fail with {reason:?}, got {error:?}"
            );
        }
    }

    #[test]
    fn atomic_write_publishes_final_canonical_artifact_and_hash() {
        let root = TestArtifactRoot::new();
        let path = PublishedArtifactPath::new("modules/alpha.mizir.json").expect("valid path");
        let value = CanonicalJson::object([
            ("schema_version", CanonicalJson::string("1.0")),
            ("module", CanonicalJson::string("alpha")),
        ])
        .expect("unique keys");
        let domain = artifact_hash_domain("store-test", SchemaVersion::new(1, 0));

        let written = write_published_artifact(root.path(), &path, &value, &domain, &[])
            .expect("write succeeds");
        assert_eq!(written.path, path);
        assert_eq!(written.artifact_hash, domain.hash(&value, &[]));
        assert_eq!(
            fs::read_to_string(root.path().join(path.as_str())).expect("final artifact exists"),
            canonical_json_string(&value)
        );
        assert!(
            !fs::read_dir(root.path().join("modules"))
                .expect("published directory")
                .any(|entry| entry
                    .expect("dir entry")
                    .file_name()
                    .to_string_lossy()
                    .starts_with(TEMP_FILE_PREFIX)),
            "successful rename must not leave visible temporary files"
        );

        let read = read_published_artifact(
            root.path(),
            &path,
            PublishedArtifactReadOptions {
                artifact_hash_domain: Some(&domain),
                hash_excluded_paths: &[],
                expected_artifact_hash: Some(written.artifact_hash),
            },
        )
        .expect("read succeeds");
        assert_eq!(read.path, path);
        assert_eq!(read.value, value);
        assert_eq!(read.artifact_hash, Some(written.artifact_hash));
    }

    #[test]
    fn published_artifact_write_is_deterministic_for_identical_inputs() {
        let root = TestArtifactRoot::new();
        let path = PublishedArtifactPath::new("modules/alpha.mizir.json").expect("valid path");
        let value = CanonicalJson::object([
            ("schema_version", CanonicalJson::string("1.0")),
            ("module", CanonicalJson::string("alpha")),
            ("exports", CanonicalJson::array([])),
        ])
        .expect("unique keys");
        let domain = artifact_hash_domain("store-test", SchemaVersion::new(1, 0));

        let first_write = write_published_artifact(root.path(), &path, &value, &domain, &[])
            .expect("first write succeeds");
        let first_bytes = fs::read(root.path().join(path.as_str())).expect("first artifact bytes");
        let second_write = write_published_artifact(root.path(), &path, &value, &domain, &[])
            .expect("second write succeeds");
        let second_bytes =
            fs::read(root.path().join(path.as_str())).expect("second artifact bytes");

        assert_eq!(first_write.path, second_write.path);
        assert_eq!(first_write.artifact_hash, second_write.artifact_hash);
        assert_eq!(first_write.artifact_hash, domain.hash(&value, &[]));
        assert_eq!(first_bytes, second_bytes);
        assert_eq!(first_bytes, canonical_json_string(&value).into_bytes());
    }

    #[test]
    fn atomic_write_replaces_previous_complete_artifact() {
        let root = TestArtifactRoot::new();
        let path = PublishedArtifactPath::new("modules/alpha.mizir.json").expect("valid path");
        let domain = artifact_hash_domain("store-test", SchemaVersion::new(1, 0));
        let first = CanonicalJson::object([
            ("schema_version", CanonicalJson::string("1.0")),
            ("value", CanonicalJson::integer(1)),
        ])
        .expect("unique keys");
        let second = CanonicalJson::object([
            ("schema_version", CanonicalJson::string("1.0")),
            ("value", CanonicalJson::integer(2)),
        ])
        .expect("unique keys");

        let first_write = write_published_artifact(root.path(), &path, &first, &domain, &[])
            .expect("first write");
        assert_eq!(
            read_published_artifact(
                root.path(),
                &path,
                PublishedArtifactReadOptions {
                    artifact_hash_domain: Some(&domain),
                    hash_excluded_paths: &[],
                    expected_artifact_hash: Some(first_write.artifact_hash),
                },
            )
            .expect("first read")
            .value,
            first
        );

        let second_write = write_published_artifact(root.path(), &path, &second, &domain, &[])
            .expect("replacement write");
        let read = read_published_artifact(
            root.path(),
            &path,
            PublishedArtifactReadOptions {
                artifact_hash_domain: Some(&domain),
                hash_excluded_paths: &[],
                expected_artifact_hash: Some(second_write.artifact_hash),
            },
        )
        .expect("replacement read");

        assert_eq!(read.value, second);
        assert_ne!(first_write.artifact_hash, second_write.artifact_hash);
    }

    #[test]
    fn stale_temporary_name_collision_retries_before_publishing() {
        let root = TestArtifactRoot::new();
        let final_path = root.path().join("alpha.json");
        let stale_temp = root.path().join(".mizar-artifact-tmp-stale-alpha.json");
        let fresh_temp = root.path().join(".mizar-artifact-tmp-fresh-alpha.json");
        fs::write(&stale_temp, b"stale").expect("stale temp fixture");
        let value = CanonicalJson::object([("schema_version", CanonicalJson::string("1.0"))])
            .expect("unique keys");
        let bytes = super::canonical_json_bytes(&value);

        commit_with_temporary_candidates(
            &final_path,
            root.path(),
            &bytes,
            [stale_temp.clone(), fresh_temp.clone()],
        )
        .expect("retry should publish with second temp candidate");

        assert_eq!(
            fs::read_to_string(&final_path).expect("published artifact"),
            canonical_json_string(&value)
        );
        assert!(
            stale_temp.exists(),
            "stale temp files from other sessions must not be removed on collision"
        );
        assert!(
            !fresh_temp.exists(),
            "successful publish should rename away the temp file it created"
        );
    }

    #[test]
    fn write_creates_fresh_artifact_root_before_canonicalizing_it() {
        let root = TestArtifactRoot::new_removed();
        let path = PublishedArtifactPath::new("alpha.json").expect("valid path");
        let value = CanonicalJson::object([("schema_version", CanonicalJson::string("1.0"))])
            .expect("unique keys");
        let domain = artifact_hash_domain("store-test", SchemaVersion::new(1, 0));

        write_published_artifact(root.path(), &path, &value, &domain, &[])
            .expect("write creates artifact root");

        assert!(root.path().join("alpha.json").is_file());
    }

    #[test]
    fn interrupted_temporary_file_is_not_visible_as_final_artifact() {
        let root = TestArtifactRoot::new();
        let final_path = PublishedArtifactPath::new("modules/alpha.json").expect("valid path");
        let partial_dir = root.path().join("modules");
        fs::create_dir_all(&partial_dir).expect("partial directory");
        fs::write(
            partial_dir.join(format!("{TEMP_FILE_PREFIX}-partial-alpha.json")),
            b"{\"schema_version\":",
        )
        .expect("partial temporary fixture");

        let error = read_published_artifact(
            root.path(),
            &final_path,
            PublishedArtifactReadOptions::default(),
        )
        .expect_err("reader must not discover temporary files");

        assert!(
            matches!(
                error,
                StoreIoError::Io {
                    operation: StoreIoOperation::Read,
                    kind: std::io::ErrorKind::NotFound,
                    ..
                }
            ),
            "missing final path should be reported instead of reading temp content: {error:?}"
        );
    }

    #[test]
    fn read_reports_corruption_with_artifact_positions() {
        let root = TestArtifactRoot::new();
        let utf8_path = PublishedArtifactPath::new("bad-utf8.json").expect("valid path");
        fs::write(root.path().join(utf8_path.as_str()), [0xff]).expect("bad utf8 fixture");
        let error = read_published_artifact(
            root.path(),
            &utf8_path,
            PublishedArtifactReadOptions::default(),
        )
        .expect_err("invalid UTF-8 must fail");
        assert!(
            matches!(
                error,
                StoreIoError::InvalidUtf8 {
                    location: super::ArtifactTextLocation {
                        byte_offset: 0,
                        line: 1,
                        column: 1
                    },
                    ..
                }
            ),
            "invalid UTF-8 should carry first byte location: {error:?}"
        );

        let malformed_path = PublishedArtifactPath::new("malformed.json").expect("valid path");
        fs::write(root.path().join(malformed_path.as_str()), b"{\"a\":\n")
            .expect("malformed fixture");
        let error = read_published_artifact(
            root.path(),
            &malformed_path,
            PublishedArtifactReadOptions::default(),
        )
        .expect_err("malformed JSON must fail");
        assert!(
            matches!(
                error,
                StoreIoError::CorruptCanonicalJson {
                    location: super::ArtifactTextLocation {
                        byte_offset: 5,
                        line: 1,
                        column: 6
                    },
                    ..
                }
            ),
            "malformed JSON should include byte/line/column: {error:?}"
        );
    }

    #[test]
    fn read_rejects_duplicate_keys_and_noncanonical_spelling() {
        let root = TestArtifactRoot::new();
        let duplicate_path = PublishedArtifactPath::new("duplicate.json").expect("valid path");
        fs::write(
            root.path().join(duplicate_path.as_str()),
            b"{\"a\":1,\"a\":2}\n",
        )
        .expect("duplicate fixture");
        let error = read_published_artifact(
            root.path(),
            &duplicate_path,
            PublishedArtifactReadOptions::default(),
        )
        .expect_err("duplicate object key must fail");
        assert!(
            matches!(error, StoreIoError::CorruptCanonicalJson { ref reason, .. } if reason.contains("duplicate object key")),
            "duplicate key should be a positioned canonical JSON error: {error:?}"
        );

        let unsorted_path = PublishedArtifactPath::new("unsorted.json").expect("valid path");
        fs::write(
            root.path().join(unsorted_path.as_str()),
            b"{\"b\":1,\"a\":2}\n",
        )
        .expect("unsorted fixture");
        let error = read_published_artifact(
            root.path(),
            &unsorted_path,
            PublishedArtifactReadOptions::default(),
        )
        .expect_err("unsorted canonical JSON must fail");
        assert!(
            matches!(error, StoreIoError::NonCanonicalJson { .. }),
            "noncanonical key order should fail after parsing: {error:?}"
        );
    }

    #[test]
    fn expected_hash_mismatch_and_non_artifact_domains_are_rejected() {
        let root = TestArtifactRoot::new();
        let path = PublishedArtifactPath::new("alpha.json").expect("valid path");
        let value = CanonicalJson::object([("schema_version", CanonicalJson::string("1.0"))])
            .expect("unique keys");
        let domain = artifact_hash_domain("store-test", SchemaVersion::new(1, 0));
        write_published_artifact(root.path(), &path, &value, &domain, &[]).expect("write");

        let other_value = CanonicalJson::object([("schema_version", CanonicalJson::string("1.1"))])
            .expect("unique keys");
        let wrong_hash = domain.hash(&other_value, &[]);
        let error = read_published_artifact(
            root.path(),
            &path,
            PublishedArtifactReadOptions {
                artifact_hash_domain: Some(&domain),
                hash_excluded_paths: &[],
                expected_artifact_hash: Some(wrong_hash),
            },
        )
        .expect_err("hash mismatch must fail");
        assert!(matches!(error, StoreIoError::ArtifactHashMismatch { .. }));

        let interface_domain =
            CanonicalHashDomain::new(HashClass::Interface, "store-test", SchemaVersion::new(1, 0));
        let error = write_published_artifact(root.path(), &path, &value, &interface_domain, &[])
            .expect_err("non-artifact domain must fail");
        assert!(matches!(
            error,
            StoreIoError::NonArtifactHashDomain {
                class: HashClass::Interface
            }
        ));

        let error = read_published_artifact(
            root.path(),
            &path,
            PublishedArtifactReadOptions {
                artifact_hash_domain: Some(&interface_domain),
                hash_excluded_paths: &[],
                expected_artifact_hash: None,
            },
        )
        .expect_err("read with non-artifact domain must fail");
        assert!(matches!(
            error,
            StoreIoError::NonArtifactHashDomain {
                class: HashClass::Interface
            }
        ));

        let error = read_published_artifact(
            root.path(),
            &path,
            PublishedArtifactReadOptions {
                artifact_hash_domain: None,
                hash_excluded_paths: &[],
                expected_artifact_hash: Some(domain.hash(&value, &[])),
            },
        )
        .expect_err("expected hash without domain must fail");
        assert!(matches!(error, StoreIoError::MissingHashDomain { .. }));
    }

    #[cfg(unix)]
    #[test]
    fn write_rejects_parent_symlink_escape() {
        use std::os::unix::fs::symlink;

        let root = TestArtifactRoot::new();
        let outside = TestArtifactRoot::new();
        symlink(outside.path(), root.path().join("escape")).expect("parent symlink");
        let path = PublishedArtifactPath::new("escape/sub/out.json").expect("valid lexical path");
        let value = CanonicalJson::object([("schema_version", CanonicalJson::string("1.0"))])
            .expect("unique keys");
        let domain = artifact_hash_domain("store-test", SchemaVersion::new(1, 0));

        let error = write_published_artifact(root.path(), &path, &value, &domain, &[])
            .expect_err("parent symlink escape must fail");

        assert!(matches!(
            error,
            StoreIoError::Path {
                reason: PublishedPathError::RootEscape,
                ..
            }
        ));
        assert!(
            !outside.path().join("sub").exists(),
            "write validation must reject ancestor symlink before creating directories outside root"
        );
    }

    #[cfg(unix)]
    #[test]
    fn read_rejects_final_symlink_escape_before_following_it() {
        use std::os::unix::fs::symlink;

        let root = TestArtifactRoot::new();
        let outside = TestArtifactRoot::new();
        let target = outside.path().join("secret.json");
        fs::write(&target, b"{\"schema_version\":\"1.0\"}\n").expect("outside target");
        symlink(&target, root.path().join("link.json")).expect("final symlink");

        let path = PublishedArtifactPath::new("link.json").expect("valid lexical path");
        let error =
            read_published_artifact(root.path(), &path, PublishedArtifactReadOptions::default())
                .expect_err("final symlink must be rejected before read");

        assert!(matches!(
            error,
            StoreIoError::Path {
                reason: PublishedPathError::SymlinkEscape,
                ..
            }
        ));
    }

    #[cfg(unix)]
    #[test]
    fn read_rejects_ancestor_symlink_escape() {
        use std::os::unix::fs::symlink;

        let root = TestArtifactRoot::new();
        let outside = TestArtifactRoot::new();
        fs::write(
            outside.path().join("out.json"),
            b"{\"schema_version\":\"1.0\"}\n",
        )
        .expect("outside artifact");
        symlink(outside.path(), root.path().join("escape")).expect("ancestor symlink");

        let path = PublishedArtifactPath::new("escape/out.json").expect("valid lexical path");
        let error =
            read_published_artifact(root.path(), &path, PublishedArtifactReadOptions::default())
                .expect_err("ancestor symlink escape must be rejected");

        assert!(matches!(
            error,
            StoreIoError::Path {
                reason: PublishedPathError::RootEscape,
                ..
            }
        ));
    }

    #[test]
    fn schema_version_checks_detect_mismatches() {
        let support =
            SchemaVersionSupport::new("store-test", 1, 2, MinorVersionPolicy::UpToSupported);

        assert_eq!(
            support.check(Some("1.2")).expect("supported version"),
            SchemaVersion::new(1, 2)
        );
        assert!(matches!(
            support.check(None),
            Err(SchemaVersionError::Missing { .. })
        ));
        assert!(matches!(
            support.check(Some("1")),
            Err(SchemaVersionError::Malformed { .. })
        ));
        assert!(matches!(
            support.check(Some("2.0")),
            Err(SchemaVersionError::MajorMismatch { .. })
        ));
        assert!(matches!(
            support.check(Some("0.9")),
            Err(SchemaVersionError::MajorMismatch { .. })
        ));
        assert!(matches!(
            support.check(Some("1.3")),
            Err(SchemaVersionError::MinorTooNew { .. })
        ));
    }

    #[test]
    fn schema_version_policy_can_allow_newer_minor_versions() {
        let support = SchemaVersionSupport::new("store-test", 1, 2, MinorVersionPolicy::AllowNewer);

        assert_eq!(
            support
                .check(Some("1.99"))
                .expect("schema declared newer-minor compatibility"),
            SchemaVersion::new(1, 99)
        );
    }

    #[test]
    fn schema_version_errors_carry_supported_range_and_artifact_path() {
        let support =
            SchemaVersionSupport::new("store-test", 1, 2, MinorVersionPolicy::UpToSupported);

        assert_eq!(support.supported_range(), "1.0..=1.2");
        let error = support
            .check_at_path(Some("1.3"), "build/alpha.mizir.json")
            .expect_err("newer minor should be rejected");

        let SchemaVersionError::MinorTooNew {
            context,
            supported,
            actual,
            actual_version,
        } = &error
        else {
            panic!("expected minor-too-new error");
        };
        assert_eq!(context.family(), "store-test");
        assert_eq!(context.supported_range(), "1.0..=1.2");
        assert_eq!(context.artifact_path(), Some("build/alpha.mizir.json"));
        assert_eq!(*supported, 2);
        assert_eq!(*actual, 3);
        assert_eq!(*actual_version, SchemaVersion::new(1, 3));
        assert!(error.to_string().contains("build/alpha.mizir.json"));
        assert!(error.to_string().contains("1.3"));
        assert!(error.to_string().contains("1.0..=1.2"));
    }

    #[test]
    fn hash_domains_are_separated() {
        let version = SchemaVersion::new(1, 0);
        let value = CanonicalJson::object([("stable", CanonicalJson::string("same"))])
            .expect("unique keys");
        let classes = [
            HashClass::Interface,
            HashClass::Implementation,
            HashClass::Diagnostic,
            HashClass::Artifact,
        ];

        for (left_index, left_class) in classes.iter().enumerate() {
            let left = CanonicalHashDomain::new(*left_class, "store-test", version);
            for right_class in classes.iter().skip(left_index + 1) {
                let right = CanonicalHashDomain::new(*right_class, "store-test", version);
                assert_ne!(left.hash(&value, &[]), right.hash(&value, &[]));
                assert_ne!(left.hash_input(&value, &[]), right.hash_input(&value, &[]));
            }
        }
    }

    #[test]
    fn hash_frame_includes_schema_family_and_version() {
        let value = CanonicalJson::object([("stable", CanonicalJson::string("same"))])
            .expect("unique keys");
        let baseline =
            CanonicalHashDomain::new(HashClass::Interface, "store-test", SchemaVersion::new(1, 0));
        let other_family = CanonicalHashDomain::new(
            HashClass::Interface,
            "other-store-test",
            SchemaVersion::new(1, 0),
        );
        let other_version =
            CanonicalHashDomain::new(HashClass::Interface, "store-test", SchemaVersion::new(1, 1));

        assert_ne!(
            baseline.hash_input(&value, &[]),
            other_family.hash_input(&value, &[])
        );
        assert_ne!(baseline.hash(&value, &[]), other_family.hash(&value, &[]));
        assert_ne!(
            baseline.hash_input(&value, &[]),
            other_version.hash_input(&value, &[])
        );
        assert_ne!(baseline.hash(&value, &[]), other_version.hash(&value, &[]));
    }

    #[test]
    fn hash_excluded_fields_do_not_affect_hashes() {
        let baseline = CanonicalJson::object([
            ("stable", CanonicalJson::string("same")),
            ("verified_at", CanonicalJson::string("2026-01-01T00:00:00Z")),
        ])
        .expect("unique keys");
        let changed_local = CanonicalJson::object([
            ("stable", CanonicalJson::string("same")),
            ("verified_at", CanonicalJson::string("2026-01-02T00:00:00Z")),
        ])
        .expect("unique keys");
        let domain = CanonicalHashDomain::new(
            HashClass::Implementation,
            "store-test",
            SchemaVersion::new(1, 0),
        );
        let excluded = [FieldPath::new(["verified_at"]).expect("non-empty path")];

        assert_ne!(
            domain.hash(&baseline, &[]),
            domain.hash(&changed_local, &[])
        );
        assert_eq!(
            domain.hash(&baseline, &excluded),
            domain.hash(&changed_local, &excluded)
        );
    }

    #[test]
    fn hash_exclusion_paths_ignore_absent_paths_and_parent_paths_win() {
        let value = CanonicalJson::object([
            (
                "local",
                CanonicalJson::object([
                    ("verified_at", CanonicalJson::string("now")),
                    ("session", CanonicalJson::string("local")),
                ])
                .expect("unique nested keys"),
            ),
            ("stable", CanonicalJson::string("same")),
        ])
        .expect("unique keys");
        let parent_only = [FieldPath::new(["local"]).expect("non-empty path")];
        let parent_and_child = [
            FieldPath::new(["local"]).expect("non-empty path"),
            FieldPath::new(["local", "verified_at"]).expect("non-empty path"),
            FieldPath::new(["missing"]).expect("non-empty path"),
        ];
        let domain =
            CanonicalHashDomain::new(HashClass::Artifact, "store-test", SchemaVersion::new(1, 0));

        assert_eq!(
            domain.hash_input(&value, &parent_only),
            domain.hash_input(&value, &parent_and_child)
        );
    }

    #[test]
    fn hash_exclusion_paths_remove_nested_child_fields_only() {
        let first = CanonicalJson::object([
            (
                "local",
                CanonicalJson::object([
                    ("verified_at", CanonicalJson::string("now")),
                    ("session", CanonicalJson::string("same-session")),
                ])
                .expect("unique nested keys"),
            ),
            ("stable", CanonicalJson::string("same")),
        ])
        .expect("unique keys");
        let changed_child = CanonicalJson::object([
            (
                "local",
                CanonicalJson::object([
                    ("verified_at", CanonicalJson::string("later")),
                    ("session", CanonicalJson::string("same-session")),
                ])
                .expect("unique nested keys"),
            ),
            ("stable", CanonicalJson::string("same")),
        ])
        .expect("unique keys");
        let changed_sibling = CanonicalJson::object([
            (
                "local",
                CanonicalJson::object([
                    ("verified_at", CanonicalJson::string("later")),
                    ("session", CanonicalJson::string("different-session")),
                ])
                .expect("unique nested keys"),
            ),
            ("stable", CanonicalJson::string("same")),
        ])
        .expect("unique keys");
        let domain =
            CanonicalHashDomain::new(HashClass::Artifact, "store-test", SchemaVersion::new(1, 0));
        let excluded_child = [FieldPath::new(["local", "verified_at"]).expect("non-empty path")];

        assert_eq!(
            domain.hash(&first, &excluded_child),
            domain.hash(&changed_child, &excluded_child)
        );
        assert_ne!(
            domain.hash(&first, &excluded_child),
            domain.hash(&changed_sibling, &excluded_child)
        );
    }

    #[test]
    fn hash_exclusion_paths_do_not_traverse_arrays() {
        let value = CanonicalJson::object([(
            "items",
            CanonicalJson::array([CanonicalJson::object([(
                "verified_at",
                CanonicalJson::string("local"),
            )])
            .expect("unique nested keys")]),
        )])
        .expect("unique keys");
        let domain =
            CanonicalHashDomain::new(HashClass::Artifact, "store-test", SchemaVersion::new(1, 0));
        let attempted_array_path =
            [FieldPath::new(["items", "verified_at"]).expect("non-empty path")];

        assert_eq!(
            domain.hash_input(&value, &[]),
            domain.hash_input(&value, &attempted_array_path)
        );
    }

    struct TestArtifactRoot {
        path: PathBuf,
    }

    impl TestArtifactRoot {
        fn new() -> Self {
            let root = Self::fresh_path();
            fs::create_dir_all(&root).expect("test artifact root");
            Self { path: root }
        }

        fn new_removed() -> Self {
            let root = Self::fresh_path();
            let _ = fs::remove_dir_all(&root);
            Self { path: root }
        }

        fn path(&self) -> &Path {
            &self.path
        }

        fn fresh_path() -> PathBuf {
            let counter = TEST_ROOT_COUNTER.fetch_add(1, Ordering::Relaxed);
            std::env::temp_dir().join(format!(
                "mizar-artifact-store-test-{}-{counter}",
                std::process::id()
            ))
        }
    }

    impl Drop for TestArtifactRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
