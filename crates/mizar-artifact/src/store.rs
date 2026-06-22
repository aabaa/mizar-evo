//! Canonical JSON, hash helpers, and artifact store writes.
//!
//! Design authority: [store.md](../../../../doc/design/mizar-artifact/en/store.md).

use mizar_session::{Hash, hash_text};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::fmt::Write as _;
use std::fs;
use std::io::Write as _;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

/// Forward-compatible error surface for artifact serialization and storage.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ArtifactError {
    /// A filesystem operation failed at `path`.
    Io {
        /// Path involved in the failing operation.
        path: PathBuf,
        /// Stable I/O error kind.
        kind: std::io::ErrorKind,
    },
    /// A portable artifact path was absolute, empty, or escaped the store root.
    InvalidPath {
        /// Rejected path spelling.
        path: PathBuf,
    },
    /// UTF-8 text was required but bytes were invalid.
    InvalidUtf8 {
        /// Optional path for the invalid bytes.
        path: Option<PathBuf>,
    },
    /// JSON parsing failed at a byte offset.
    MalformedJson {
        /// Byte offset where parsing failed.
        offset: usize,
    },
    /// JSON contained trailing non-whitespace data.
    TrailingJson {
        /// Byte offset where trailing data starts.
        offset: usize,
    },
    /// A JSON object repeated a key.
    DuplicateJsonKey {
        /// Repeated key.
        key: String,
    },
    /// A required schema field is missing.
    MissingField {
        /// Schema being read.
        schema: &'static str,
        /// Missing field name.
        field: &'static str,
    },
    /// A schema field is unknown.
    UnexpectedField {
        /// Schema being read.
        schema: &'static str,
        /// Unexpected field name.
        field: String,
    },
    /// A schema field has invalid shape or value.
    InvalidField {
        /// Schema being read.
        schema: &'static str,
        /// Invalid field name.
        field: &'static str,
        /// Human-readable reason.
        reason: String,
    },
    /// The schema version is not supported by this reader.
    UnsupportedSchemaVersion {
        /// Schema being read.
        schema: &'static str,
        /// Version found in the record.
        actual: String,
        /// Versions supported by this reader.
        supported: &'static [&'static str],
    },
    /// A hash string is not lowercase hex with the expected width.
    InvalidHash {
        /// Rejected hash value.
        value: String,
    },
    /// Recorded and actual content hashes differ.
    HashMismatch {
        /// Expected hash.
        expected: Hash,
        /// Actual hash.
        actual: Hash,
    },
    /// A record contained raw compiler-internal payload fields.
    RawIrPayload {
        /// Rejected field name.
        field: String,
    },
    /// A schema contained duplicate logical entries.
    DuplicateEntry {
        /// Schema being read.
        schema: &'static str,
        /// Duplicate key.
        key: String,
    },
}

/// JSON value used by artifact schemas for canonical rendering.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CanonicalJson {
    /// JSON null.
    Null,
    /// JSON boolean.
    Bool(bool),
    /// Non-negative JSON integer.
    Integer(u64),
    /// JSON string.
    String(String),
    /// JSON array whose order is already canonical for its schema.
    Array(Vec<CanonicalJson>),
    /// JSON object rendered with lexicographically sorted keys.
    Object(BTreeMap<String, CanonicalJson>),
}

/// Content-addressed blob written by [`ArtifactStore`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredBlob {
    /// Portable path relative to the store root.
    pub path: String,
    /// Content hash of the written bytes.
    pub content_hash: Hash,
}

/// Store rooted at one package artifact directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactStore {
    root: PathBuf,
}

impl CanonicalJson {
    /// Builds a canonical JSON object from unsorted fields.
    pub fn object<I, K>(fields: I) -> Self
    where
        I: IntoIterator<Item = (K, CanonicalJson)>,
        K: Into<String>,
    {
        let mut object = BTreeMap::new();
        for (key, value) in fields {
            object.insert(key.into(), value);
        }
        Self::Object(object)
    }

    /// Builds a JSON string.
    pub fn string(value: impl Into<String>) -> Self {
        Self::String(value.into())
    }

    /// Builds a JSON array.
    pub fn array(values: Vec<CanonicalJson>) -> Self {
        Self::Array(values)
    }

    /// Renders this value as canonical JSON bytes encoded in UTF-8.
    pub fn to_canonical_string(&self) -> String {
        let mut output = String::new();
        render_json(self, &mut output);
        output
    }
}

impl ArtifactStore {
    /// Creates a store rooted at `artifact_dir`.
    pub fn new(artifact_dir: impl Into<PathBuf>) -> Self {
        Self {
            root: artifact_dir.into(),
        }
    }

    /// Returns the filesystem root for this store.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Writes UTF-8 content as a content-addressed blob.
    pub fn write_blob_str(&self, content: &str) -> Result<StoredBlob, ArtifactError> {
        let content_hash = content_hash_str(content);
        let path = format!("blobs/{}.json", hash_to_hex(content_hash));
        self.write_named_str(&path, content)?;
        Ok(StoredBlob { path, content_hash })
    }

    /// Reads a content-addressed UTF-8 blob and verifies its hash.
    pub fn read_blob_str(&self, content_hash: Hash) -> Result<String, ArtifactError> {
        let path = format!("blobs/{}.json", hash_to_hex(content_hash));
        self.read_named_str(&path, Some(content_hash))
    }

    /// Writes a named UTF-8 artifact with temp-and-rename atomicity.
    pub fn write_named_str(
        &self,
        relative_path: &str,
        content: &str,
    ) -> Result<Hash, ArtifactError> {
        let final_path = self.portable_path(relative_path)?;
        if let Some(parent) = final_path.parent() {
            create_dir_all(parent)?;
        }

        let temp_path = temp_path_for(&final_path);
        {
            let mut file = fs::File::create(&temp_path).map_err(|error| ArtifactError::Io {
                path: temp_path.clone(),
                kind: error.kind(),
            })?;
            file.write_all(content.as_bytes())
                .map_err(|error| ArtifactError::Io {
                    path: temp_path.clone(),
                    kind: error.kind(),
                })?;
            file.sync_all().map_err(|error| ArtifactError::Io {
                path: temp_path.clone(),
                kind: error.kind(),
            })?;
        }

        fs::rename(&temp_path, &final_path).map_err(|error| ArtifactError::Io {
            path: final_path.clone(),
            kind: error.kind(),
        })?;
        if let Some(parent) = final_path.parent() {
            flush_directory(parent);
        }
        Ok(content_hash_str(content))
    }

    /// Reads a named UTF-8 artifact and optionally verifies its content hash.
    pub fn read_named_str(
        &self,
        relative_path: &str,
        expected_hash: Option<Hash>,
    ) -> Result<String, ArtifactError> {
        let path = self.portable_path(relative_path)?;
        let bytes = fs::read(&path).map_err(|error| ArtifactError::Io {
            path: path.clone(),
            kind: error.kind(),
        })?;
        let content = String::from_utf8(bytes).map_err(|_| ArtifactError::InvalidUtf8 {
            path: Some(path.clone()),
        })?;
        if let Some(expected) = expected_hash {
            let actual = content_hash_str(&content);
            if actual != expected {
                return Err(ArtifactError::HashMismatch { expected, actual });
            }
        }
        Ok(content)
    }

    fn portable_path(&self, relative_path: &str) -> Result<PathBuf, ArtifactError> {
        validate_portable_path_text(relative_path)?;
        Ok(self.root.join(Path::new(relative_path)))
    }
}

pub(crate) fn validate_portable_path_text(relative_path: &str) -> Result<(), ArtifactError> {
    if relative_path.is_empty() {
        return Err(ArtifactError::InvalidPath {
            path: PathBuf::from(relative_path),
        });
    }
    let path = Path::new(relative_path);
    if path.is_absolute() || relative_path.contains('\\') {
        return Err(ArtifactError::InvalidPath {
            path: path.to_owned(),
        });
    }
    for component in path.components() {
        if !matches!(component, Component::Normal(_)) {
            return Err(ArtifactError::InvalidPath {
                path: path.to_owned(),
            });
        }
    }
    Ok(())
}

/// Renders a [`Hash`] as lowercase hex.
pub fn hash_to_hex(hash: Hash) -> String {
    let mut output = String::with_capacity(Hash::BYTE_LEN * 2);
    for byte in hash.as_bytes() {
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}

/// Parses a lowercase-hex [`Hash`].
pub fn hash_from_hex(value: &str) -> Result<Hash, ArtifactError> {
    if value.len() != Hash::BYTE_LEN * 2 {
        return Err(ArtifactError::InvalidHash {
            value: value.to_owned(),
        });
    }
    let mut bytes = [0; Hash::BYTE_LEN];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        let high = decode_hex_nibble(pair[0]).ok_or_else(|| ArtifactError::InvalidHash {
            value: value.to_owned(),
        })?;
        let low = decode_hex_nibble(pair[1]).ok_or_else(|| ArtifactError::InvalidHash {
            value: value.to_owned(),
        })?;
        bytes[index] = (high << 4) | low;
    }
    Ok(Hash::from_bytes(bytes))
}

/// Hashes canonical JSON under a role-specific domain.
pub fn canonical_hash(domain: &str, value: &CanonicalJson) -> Hash {
    let canonical = value.to_canonical_string();
    hash_text(&format!(
        "mizar-artifact/canonical-hash/v1\n{}\n{}\n{}",
        domain,
        canonical.len(),
        canonical
    ))
}

/// Hashes UTF-8 artifact content.
pub fn content_hash_str(content: &str) -> Hash {
    hash_text(&format!(
        "mizar-artifact/content/v1\n{}\n{}",
        content.len(),
        content
    ))
}

pub(crate) fn parse_json(input: &str) -> Result<CanonicalJson, ArtifactError> {
    JsonParser::new(input).parse()
}

pub(crate) fn expect_object<'a>(
    value: &'a CanonicalJson,
    schema: &'static str,
) -> Result<&'a BTreeMap<String, CanonicalJson>, ArtifactError> {
    match value {
        CanonicalJson::Object(object) => Ok(object),
        _ => Err(ArtifactError::InvalidField {
            schema,
            field: "<root>",
            reason: "expected JSON object".to_owned(),
        }),
    }
}

pub(crate) fn reject_unknown_fields(
    object: &BTreeMap<String, CanonicalJson>,
    schema: &'static str,
    allowed: &[&str],
) -> Result<(), ArtifactError> {
    for key in object.keys() {
        if !allowed.iter().any(|allowed_key| key == allowed_key) {
            if is_raw_ir_field(key) {
                return Err(ArtifactError::RawIrPayload { field: key.clone() });
            }
            return Err(ArtifactError::UnexpectedField {
                schema,
                field: key.clone(),
            });
        }
    }
    Ok(())
}

pub(crate) fn required_string(
    object: &BTreeMap<String, CanonicalJson>,
    schema: &'static str,
    field: &'static str,
) -> Result<String, ArtifactError> {
    match object.get(field) {
        Some(CanonicalJson::String(value)) => Ok(value.clone()),
        Some(_) => Err(ArtifactError::InvalidField {
            schema,
            field,
            reason: "expected string".to_owned(),
        }),
        None => Err(ArtifactError::MissingField { schema, field }),
    }
}

pub(crate) fn optional_string(
    object: &BTreeMap<String, CanonicalJson>,
    schema: &'static str,
    field: &'static str,
) -> Result<Option<String>, ArtifactError> {
    match object.get(field) {
        Some(CanonicalJson::String(value)) => Ok(Some(value.clone())),
        Some(CanonicalJson::Null) | None => Ok(None),
        Some(_) => Err(ArtifactError::InvalidField {
            schema,
            field,
            reason: "expected string or null".to_owned(),
        }),
    }
}

pub(crate) fn required_u64(
    object: &BTreeMap<String, CanonicalJson>,
    schema: &'static str,
    field: &'static str,
) -> Result<u64, ArtifactError> {
    match object.get(field) {
        Some(CanonicalJson::Integer(value)) => Ok(*value),
        Some(_) => Err(ArtifactError::InvalidField {
            schema,
            field,
            reason: "expected non-negative integer".to_owned(),
        }),
        None => Err(ArtifactError::MissingField { schema, field }),
    }
}

pub(crate) fn required_hash(
    object: &BTreeMap<String, CanonicalJson>,
    schema: &'static str,
    field: &'static str,
) -> Result<Hash, ArtifactError> {
    let value = required_string(object, schema, field)?;
    hash_from_hex(&value)
}

pub(crate) fn optional_hash(
    object: &BTreeMap<String, CanonicalJson>,
    schema: &'static str,
    field: &'static str,
) -> Result<Option<Hash>, ArtifactError> {
    optional_string(object, schema, field)?
        .map(|value| hash_from_hex(&value))
        .transpose()
}

pub(crate) fn required_array<'a>(
    object: &'a BTreeMap<String, CanonicalJson>,
    schema: &'static str,
    field: &'static str,
) -> Result<&'a [CanonicalJson], ArtifactError> {
    match object.get(field) {
        Some(CanonicalJson::Array(values)) => Ok(values),
        Some(_) => Err(ArtifactError::InvalidField {
            schema,
            field,
            reason: "expected array".to_owned(),
        }),
        None => Err(ArtifactError::MissingField { schema, field }),
    }
}

pub(crate) fn check_schema_version(
    object: &BTreeMap<String, CanonicalJson>,
    schema: &'static str,
    supported: &'static [&'static str],
) -> Result<String, ArtifactError> {
    let actual = required_string(object, schema, "schema_version")?;
    if supported.iter().any(|version| *version == actual) {
        Ok(actual)
    } else {
        Err(ArtifactError::UnsupportedSchemaVersion {
            schema,
            actual,
            supported,
        })
    }
}

pub(crate) fn hash_json(hash: Hash) -> CanonicalJson {
    CanonicalJson::string(hash_to_hex(hash))
}

pub(crate) fn optional_string_json(value: &Option<String>) -> CanonicalJson {
    value
        .as_ref()
        .map_or(CanonicalJson::Null, CanonicalJson::string)
}

pub(crate) fn optional_hash_json(value: Option<Hash>) -> CanonicalJson {
    value.map_or(CanonicalJson::Null, hash_json)
}

pub(crate) fn ensure_sorted_by<T, F>(
    values: &[T],
    schema: &'static str,
    field: &'static str,
    key: F,
) -> Result<(), ArtifactError>
where
    F: Fn(&T) -> String,
{
    if values
        .windows(2)
        .any(|window| key(&window[0]) > key(&window[1]))
    {
        return Err(ArtifactError::InvalidField {
            schema,
            field,
            reason: "entries are not in canonical order".to_owned(),
        });
    }
    Ok(())
}

pub(crate) fn ensure_unique_by<T, F>(
    values: &[T],
    schema: &'static str,
    key: F,
) -> Result<(), ArtifactError>
where
    F: Fn(&T) -> String,
{
    let mut seen = BTreeMap::new();
    for value in values {
        let key = key(value);
        if seen.insert(key.clone(), ()).is_some() {
            return Err(ArtifactError::DuplicateEntry { schema, key });
        }
    }
    Ok(())
}

pub(crate) fn reject_raw_ir_fields(value: &CanonicalJson) -> Result<(), ArtifactError> {
    match value {
        CanonicalJson::Object(object) => {
            for (key, value) in object {
                if is_raw_ir_field(key) {
                    return Err(ArtifactError::RawIrPayload { field: key.clone() });
                }
                reject_raw_ir_fields(value)?;
            }
            Ok(())
        }
        CanonicalJson::Array(values) => {
            for value in values {
                reject_raw_ir_fields(value)?;
            }
            Ok(())
        }
        CanonicalJson::Null
        | CanonicalJson::Bool(_)
        | CanonicalJson::Integer(_)
        | CanonicalJson::String(_) => Ok(()),
    }
}

fn render_json(value: &CanonicalJson, output: &mut String) {
    match value {
        CanonicalJson::Null => output.push_str("null"),
        CanonicalJson::Bool(true) => output.push_str("true"),
        CanonicalJson::Bool(false) => output.push_str("false"),
        CanonicalJson::Integer(number) => write!(output, "{number}").expect("String write"),
        CanonicalJson::String(value) => render_string(value, output),
        CanonicalJson::Array(values) => {
            output.push('[');
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                render_json(value, output);
            }
            output.push(']');
        }
        CanonicalJson::Object(object) => {
            output.push('{');
            for (index, (key, value)) in object.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                render_string(key, output);
                output.push(':');
                render_json(value, output);
            }
            output.push('}');
        }
    }
}

fn render_string(value: &str, output: &mut String) {
    output.push('"');
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            '\u{08}' => output.push_str("\\b"),
            '\u{0c}' => output.push_str("\\f"),
            '\u{00}'..='\u{1f}' => {
                write!(output, "\\u{:04x}", character as u32).expect("String write");
            }
            _ => output.push(character),
        }
    }
    output.push('"');
}

fn decode_hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

fn create_dir_all(path: &Path) -> Result<(), ArtifactError> {
    fs::create_dir_all(path).map_err(|error| ArtifactError::Io {
        path: path.to_owned(),
        kind: error.kind(),
    })
}

fn temp_path_for(final_path: &Path) -> PathBuf {
    let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
    let file_name = final_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("artifact");
    final_path.with_file_name(format!(".{file_name}.{}.{}.tmp", std::process::id(), id))
}

fn flush_directory(path: &Path) {
    let _ = fs::File::open(path).and_then(|directory| directory.sync_all());
}

fn is_raw_ir_field(field: &str) -> bool {
    let normalized = field.to_ascii_lowercase();
    [
        "raw_ir",
        "surface_ast",
        "resolved_ast",
        "typed_ast",
        "core_ir",
        "control_flow_ir",
        "vc_ir",
        "atp_problem",
        "kernel_state",
    ]
    .iter()
    .any(|forbidden| normalized.contains(forbidden))
}

struct JsonParser<'a> {
    input: &'a [u8],
    cursor: usize,
}

impl<'a> JsonParser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input: input.as_bytes(),
            cursor: 0,
        }
    }

    fn parse(mut self) -> Result<CanonicalJson, ArtifactError> {
        self.skip_ws();
        let value = self.parse_value()?;
        self.skip_ws();
        if self.cursor == self.input.len() {
            Ok(value)
        } else {
            Err(ArtifactError::TrailingJson {
                offset: self.cursor,
            })
        }
    }

    fn parse_value(&mut self) -> Result<CanonicalJson, ArtifactError> {
        self.skip_ws();
        match self.peek() {
            Some(b'n') => self.parse_literal(b"null", CanonicalJson::Null),
            Some(b't') => self.parse_literal(b"true", CanonicalJson::Bool(true)),
            Some(b'f') => self.parse_literal(b"false", CanonicalJson::Bool(false)),
            Some(b'"') => self.parse_string().map(CanonicalJson::String),
            Some(b'[') => self.parse_array(),
            Some(b'{') => self.parse_object(),
            Some(b'0'..=b'9') => self.parse_integer().map(CanonicalJson::Integer),
            _ => Err(ArtifactError::MalformedJson {
                offset: self.cursor,
            }),
        }
    }

    fn parse_literal(
        &mut self,
        literal: &[u8],
        value: CanonicalJson,
    ) -> Result<CanonicalJson, ArtifactError> {
        if self.input.get(self.cursor..self.cursor + literal.len()) == Some(literal) {
            self.cursor += literal.len();
            Ok(value)
        } else {
            Err(ArtifactError::MalformedJson {
                offset: self.cursor,
            })
        }
    }

    fn parse_string(&mut self) -> Result<String, ArtifactError> {
        self.expect_byte(b'"')?;
        let mut output = String::new();
        while let Some(byte) = self.bump() {
            match byte {
                b'"' => return Ok(output),
                b'\\' => output.push(self.parse_escape()?),
                0x00..=0x1f => {
                    return Err(ArtifactError::MalformedJson {
                        offset: self.cursor.saturating_sub(1),
                    });
                }
                _ => {
                    let start = self.cursor - 1;
                    let Some(character) = std::str::from_utf8(&self.input[start..])
                        .ok()
                        .and_then(|remaining| remaining.chars().next())
                    else {
                        return Err(ArtifactError::MalformedJson { offset: start });
                    };
                    self.cursor = start + character.len_utf8();
                    output.push(character);
                }
            }
        }
        Err(ArtifactError::MalformedJson {
            offset: self.cursor,
        })
    }

    fn parse_escape(&mut self) -> Result<char, ArtifactError> {
        match self.bump() {
            Some(b'"') => Ok('"'),
            Some(b'\\') => Ok('\\'),
            Some(b'/') => Ok('/'),
            Some(b'b') => Ok('\u{08}'),
            Some(b'f') => Ok('\u{0c}'),
            Some(b'n') => Ok('\n'),
            Some(b'r') => Ok('\r'),
            Some(b't') => Ok('\t'),
            Some(b'u') => self.parse_unicode_escape(),
            _ => Err(ArtifactError::MalformedJson {
                offset: self.cursor.saturating_sub(1),
            }),
        }
    }

    fn parse_unicode_escape(&mut self) -> Result<char, ArtifactError> {
        let start = self.cursor;
        let mut code = 0_u32;
        for _ in 0..4 {
            let Some(byte) = self.bump() else {
                return Err(ArtifactError::MalformedJson { offset: start });
            };
            let Some(nibble) = decode_json_hex_nibble(byte) else {
                return Err(ArtifactError::MalformedJson {
                    offset: self.cursor - 1,
                });
            };
            code = (code << 4) | u32::from(nibble);
        }
        char::from_u32(code).ok_or(ArtifactError::MalformedJson { offset: start })
    }

    fn parse_integer(&mut self) -> Result<u64, ArtifactError> {
        let start = self.cursor;
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.cursor += 1;
        }
        if self.cursor > start + 1 && self.input[start] == b'0' {
            return Err(ArtifactError::MalformedJson { offset: start });
        }
        std::str::from_utf8(&self.input[start..self.cursor])
            .ok()
            .and_then(|number| number.parse::<u64>().ok())
            .ok_or(ArtifactError::MalformedJson { offset: start })
    }

    fn parse_array(&mut self) -> Result<CanonicalJson, ArtifactError> {
        self.expect_byte(b'[')?;
        self.skip_ws();
        let mut values = Vec::new();
        if self.consume_byte(b']') {
            return Ok(CanonicalJson::Array(values));
        }
        loop {
            values.push(self.parse_value()?);
            self.skip_ws();
            if self.consume_byte(b']') {
                return Ok(CanonicalJson::Array(values));
            }
            self.expect_byte(b',')?;
        }
    }

    fn parse_object(&mut self) -> Result<CanonicalJson, ArtifactError> {
        self.expect_byte(b'{')?;
        self.skip_ws();
        let mut object = BTreeMap::new();
        if self.consume_byte(b'}') {
            return Ok(CanonicalJson::Object(object));
        }
        loop {
            self.skip_ws();
            let key = self.parse_string()?;
            self.skip_ws();
            self.expect_byte(b':')?;
            let value = self.parse_value()?;
            if object.insert(key.clone(), value).is_some() {
                return Err(ArtifactError::DuplicateJsonKey { key });
            }
            self.skip_ws();
            if self.consume_byte(b'}') {
                return Ok(CanonicalJson::Object(object));
            }
            self.expect_byte(b',')?;
        }
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.cursor += 1;
        }
    }

    fn expect_byte(&mut self, expected: u8) -> Result<(), ArtifactError> {
        if self.consume_byte(expected) {
            Ok(())
        } else {
            Err(ArtifactError::MalformedJson {
                offset: self.cursor,
            })
        }
    }

    fn consume_byte(&mut self, expected: u8) -> bool {
        if self.peek() == Some(expected) {
            self.cursor += 1;
            true
        } else {
            false
        }
    }

    fn bump(&mut self) -> Option<u8> {
        let byte = self.peek()?;
        self.cursor += 1;
        Some(byte)
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.cursor).copied()
    }
}

fn decode_json_hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

impl fmt::Display for ArtifactError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, kind } => {
                write!(f, "artifact I/O error at `{}`: {kind}", path.display())
            }
            Self::InvalidPath { path } => {
                write!(f, "artifact path `{}` is not portable", path.display())
            }
            Self::InvalidUtf8 { path: Some(path) } => {
                write!(f, "artifact `{}` is not valid UTF-8", path.display())
            }
            Self::InvalidUtf8 { path: None } => f.write_str("artifact bytes are not valid UTF-8"),
            Self::MalformedJson { offset } => {
                write!(f, "artifact JSON is malformed at byte {offset}")
            }
            Self::TrailingJson { offset } => {
                write!(f, "artifact JSON has trailing data at byte {offset}")
            }
            Self::DuplicateJsonKey { key } => {
                write!(f, "artifact JSON object repeats key `{key}`")
            }
            Self::MissingField { schema, field } => {
                write!(f, "{schema} artifact is missing field `{field}`")
            }
            Self::UnexpectedField { schema, field } => {
                write!(f, "{schema} artifact has unexpected field `{field}`")
            }
            Self::InvalidField {
                schema,
                field,
                reason,
            } => {
                write!(f, "{schema} artifact field `{field}` is invalid: {reason}")
            }
            Self::UnsupportedSchemaVersion {
                schema,
                actual,
                supported,
            } => {
                write!(
                    f,
                    "{schema} artifact schema version `{actual}` is unsupported; supported: {}",
                    supported.join(", ")
                )
            }
            Self::InvalidHash { value } => {
                write!(f, "artifact hash `{value}` is not canonical lowercase hex")
            }
            Self::HashMismatch { expected, actual } => {
                write!(
                    f,
                    "artifact hash mismatch: expected {}, got {}",
                    hash_to_hex(*expected),
                    hash_to_hex(*actual)
                )
            }
            Self::RawIrPayload { field } => {
                write!(f, "artifact contains forbidden raw-IR field `{field}`")
            }
            Self::DuplicateEntry { schema, key } => {
                write!(f, "{schema} artifact repeats logical entry `{key}`")
            }
        }
    }
}

impl Error for ArtifactError {}

#[cfg(test)]
mod tests {
    use super::{
        ArtifactError, ArtifactStore, CanonicalJson, canonical_hash, content_hash_str,
        hash_from_hex, hash_to_hex, parse_json,
    };
    use std::fs;

    #[test]
    fn canonical_json_sorts_object_keys_and_escapes_strings() {
        let value = CanonicalJson::object([
            ("z", CanonicalJson::Integer(2)),
            ("a", CanonicalJson::string("line\n\"quoted\"")),
        ]);

        assert_eq!(
            value.to_canonical_string(),
            "{\"a\":\"line\\n\\\"quoted\\\"\",\"z\":2}"
        );
    }

    #[test]
    fn hashes_are_hex_round_trippable_and_domain_separated() {
        let value = CanonicalJson::object([("a", CanonicalJson::Integer(1))]);
        let first = canonical_hash("one", &value);
        let same = canonical_hash("one", &value);
        let different = canonical_hash("two", &value);

        assert_eq!(first, same);
        assert_ne!(first, different);
        assert_eq!(hash_from_hex(&hash_to_hex(first)), Ok(first));
        assert!(matches!(
            hash_from_hex("ABCD"),
            Err(ArtifactError::InvalidHash { .. })
        ));
    }

    #[test]
    fn parser_reports_malformed_positions_and_duplicate_keys() {
        assert_eq!(
            parse_json("{\"a\":"),
            Err(ArtifactError::MalformedJson { offset: 5 })
        );
        assert_eq!(
            parse_json("{\"a\":1,\"a\":2}"),
            Err(ArtifactError::DuplicateJsonKey {
                key: "a".to_owned()
            })
        );
    }

    #[test]
    fn store_writes_named_artifacts_atomically_and_verifies_hashes() {
        let root = temp_root("named");
        let store = ArtifactStore::new(&root);
        let hash = store
            .write_named_str("modules/alpha.mizir.json", "{\"ok\":true}")
            .unwrap();

        assert_eq!(
            store
                .read_named_str("modules/alpha.mizir.json", Some(hash))
                .unwrap(),
            "{\"ok\":true}"
        );
        assert!(matches!(
            store.read_named_str(
                "modules/alpha.mizir.json",
                Some(content_hash_str("different"))
            ),
            Err(ArtifactError::HashMismatch { .. })
        ));
        assert!(matches!(
            store.write_named_str("../escape.json", "{}"),
            Err(ArtifactError::InvalidPath { .. })
        ));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn interrupted_temp_files_are_not_visible_as_published_artifacts() {
        let root = temp_root("temp");
        let store = ArtifactStore::new(&root);
        let old_hash = store
            .write_named_str("artifact.json", "{\"old\":true}")
            .unwrap();
        fs::write(root.join(".artifact.json.999.tmp"), "{\"new\":true}").unwrap();

        assert_eq!(
            store
                .read_named_str("artifact.json", Some(old_hash))
                .unwrap(),
            "{\"old\":true}"
        );
        assert!(matches!(
            store.read_named_str(".artifact.json.999.tmp", Some(old_hash)),
            Err(ArtifactError::HashMismatch { .. })
        ));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn stored_malformed_json_reports_position() {
        let root = temp_root("malformed");
        let store = ArtifactStore::new(&root);
        store
            .write_named_str("bad.json", "{\"schema_version\":")
            .unwrap();
        let content = store.read_named_str("bad.json", None).unwrap();

        assert_eq!(
            parse_json(&content),
            Err(ArtifactError::MalformedJson { offset: 18 })
        );

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn blob_paths_are_content_addressed() {
        let root = temp_root("blob");
        let store = ArtifactStore::new(&root);
        let blob = store.write_blob_str("{\"artifact\":1}").unwrap();

        assert!(blob.path.starts_with("blobs/"));
        assert_eq!(
            store.read_blob_str(blob.content_hash).unwrap(),
            "{\"artifact\":1}"
        );

        fs::remove_dir_all(root).unwrap();
    }

    fn temp_root(name: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!(
            "mizar_artifact_store_{name}_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        root
    }
}
