//! Canonical store primitives shared by artifact schemas.
//!
//! Canonical behavior is specified in the
//! [store design spec](../../../../doc/design/mizar-artifact/en/store.md).

use std::{collections::BTreeMap, error::Error, fmt, str::FromStr};

use mizar_session::{Hash, hash_text};

/// Hash construction label for artifact-framed hashes.
pub const ARTIFACT_HASH_CONSTRUCTION: &str = "mizar-artifact/artifact-framed-hash-text/v1";

const HASH_FRAME_VERSION: &str = "mizar-artifact/hash-frame/v1";

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
    fn as_str(self) -> &'static str {
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
        SchemaVersion, SchemaVersionError, SchemaVersionSupport, canonical_json_string,
    };

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
}
