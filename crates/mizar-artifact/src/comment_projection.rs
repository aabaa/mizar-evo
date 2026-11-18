//! Portable source-comment envelopes around validated normal 1.2 artifacts.

use std::collections::{BTreeMap, BTreeSet};

use mizar_session::{Hash, LoadingMapSegment, TextRange, hash_text, normalize_source_text};

use crate::{
    registration_summary::{ArtifactHashClass, ArtifactHashRef},
    store::{
        CanonicalHashDomain, CanonicalJson, FieldPath, HashClass, MinorVersionPolicy,
        SchemaVersion, SchemaVersionSupport, canonical_json_bytes,
    },
    verified_artifact::{
        VerifiedArtifact, VerifiedArtifactError, VerifiedArtifactReadOptions,
        implementation_hash_input_json, interface_hash_input_json, read_verified_artifact,
        verified_artifact_json,
    },
};

/// Schema family selected by commented-envelope store and manifest references.
pub const COMMENTED_ARTIFACT_SCHEMA_FAMILY: &str = "mizar-artifact/commented-verified-artifact";
/// Exact supported commented-envelope version.
pub const COMMENTED_ARTIFACT_SCHEMA_VERSION: SchemaVersion = SchemaVersion::new(1, 0);

/// Returns store-level exclusions for local metadata inside the envelope.
pub fn artifact_hash_excluded_paths() -> Vec<FieldPath> {
    [
        &["artifact", "verified_at"][..],
        &["artifact", "provenance", "cache_key"][..],
        &["artifact", "provenance", "captured_source", "source_id"][..],
    ]
    .into_iter()
    .map(|path| FieldPath::new(path.iter().copied()).expect("nonempty artifact path"))
    .collect()
}

/// An artifact and optional comment projection, validated at the codec boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommentedArtifact {
    /// Enclosing source identity and verified exports, using normal schema 1.2.
    pub artifact: VerifiedArtifact,
    /// Null for unavailable metadata, otherwise the complete structural projection.
    pub comments: CanonicalJson,
}

/// Builds the canonical envelope after validating both artifact and comments.
pub fn commented_artifact_json(
    value: &CommentedArtifact,
) -> Result<CanonicalJson, VerifiedArtifactError> {
    let artifact = value.validate()?;
    CanonicalJson::object([
        ("schema_version", CanonicalJson::string("1.0")),
        ("artifact", artifact),
        ("comments", value.comments.clone()),
        (
            "interface_hash",
            CanonicalJson::string(hash_string(
                ArtifactHashClass::Interface,
                value.compute_interface_hash()?,
            )),
        ),
        (
            "implementation_hash",
            CanonicalJson::string(hash_string(
                ArtifactHashClass::Implementation,
                value.compute_implementation_hash()?,
            )),
        ),
    ])
    .map_err(VerifiedArtifactError::CanonicalJson)
}

/// Writes a completely validated envelope as canonical UTF8 JSON bytes.
pub fn write_commented_artifact(
    value: &CommentedArtifact,
) -> Result<Vec<u8>, VerifiedArtifactError> {
    commented_artifact_json(value).map(|json| canonical_json_bytes(&json))
}

/// Reads a complete envelope; caller expected hashes bind the outer envelope.
pub fn read_commented_artifact(
    value: &CanonicalJson,
    options: VerifiedArtifactReadOptions<'_>,
) -> Result<CommentedArtifact, VerifiedArtifactError> {
    let fields = object(value, "$")?;
    let version = string(field(fields, "schema_version", "$")?, "$.schema_version")?;
    let support = SchemaVersionSupport::new(
        COMMENTED_ARTIFACT_SCHEMA_FAMILY,
        1,
        0,
        MinorVersionPolicy::UpToSupported,
    );
    match options.artifact_path {
        Some(path) => support.check_at_path(Some(version), path),
        None => support.check(Some(version)),
    }
    .map_err(VerifiedArtifactError::SchemaVersion)?;
    exact_fields(
        fields,
        &[
            "schema_version",
            "artifact",
            "comments",
            "interface_hash",
            "implementation_hash",
        ],
        "$",
    )?;
    let artifact = read_verified_artifact(
        field(fields, "artifact", "$")?,
        VerifiedArtifactReadOptions {
            expected_interface_hash: None,
            expected_implementation_hash: None,
            ..options
        },
    )?;
    let result = CommentedArtifact {
        artifact,
        comments: field(fields, "comments", "$")?.clone(),
    };
    result.validate()?;
    let interface = result.compute_interface_hash()?;
    let implementation = result.compute_implementation_hash()?;
    let actual = string(field(fields, "interface_hash", "$")?, "$.interface_hash")?;
    let expected = hash_string(ArtifactHashClass::Interface, interface);
    if actual != expected {
        return Err(VerifiedArtifactError::InterfaceHashMismatch {
            expected,
            actual: actual.to_owned(),
        });
    }
    let actual = string(
        field(fields, "implementation_hash", "$")?,
        "$.implementation_hash",
    )?;
    let expected = hash_string(ArtifactHashClass::Implementation, implementation);
    if actual != expected {
        return Err(VerifiedArtifactError::ImplementationHashMismatch {
            expected,
            actual: actual.to_owned(),
        });
    }
    if let Some(expected) = options.expected_interface_hash
        && expected != interface
    {
        return Err(VerifiedArtifactError::ExpectedInterfaceHashMismatch {
            expected: hash_string(ArtifactHashClass::Interface, expected),
            actual: hash_string(ArtifactHashClass::Interface, interface),
        });
    }
    if let Some(expected) = options.expected_implementation_hash
        && expected != implementation
    {
        return Err(VerifiedArtifactError::ExpectedImplementationHashMismatch {
            expected: hash_string(ArtifactHashClass::Implementation, expected),
            actual: hash_string(ArtifactHashClass::Implementation, implementation),
        });
    }
    Ok(result)
}

impl CommentedArtifact {
    /// Computes the importer-visible projection in the envelope hash domain.
    pub fn compute_interface_hash(&self) -> Result<Hash, VerifiedArtifactError> {
        self.validate()?;
        let mut projection = interface_hash_input_json(&self.artifact)?;
        if let CanonicalJson::Object(fields) = &mut projection {
            fields.insert("schema_version".to_owned(), CanonicalJson::string("1.0"));
        }
        Ok(CanonicalHashDomain::new(
            HashClass::Interface,
            COMMENTED_ARTIFACT_SCHEMA_FAMILY,
            COMMENTED_ARTIFACT_SCHEMA_VERSION,
        )
        .hash(&projection, &[]))
    }

    /// Computes the stable artifact projection together with all comment metadata.
    pub fn compute_implementation_hash(&self) -> Result<Hash, VerifiedArtifactError> {
        self.validate()?;
        let projection = CanonicalJson::object([
            ("schema_version", CanonicalJson::string("1.0")),
            ("artifact", implementation_hash_input_json(&self.artifact)?),
            ("comments", self.comments.clone()),
        ])
        .map_err(VerifiedArtifactError::CanonicalJson)?;
        Ok(CanonicalHashDomain::new(
            HashClass::Implementation,
            COMMENTED_ARTIFACT_SCHEMA_FAMILY,
            COMMENTED_ARTIFACT_SCHEMA_VERSION,
        )
        .hash(&projection, &[]))
    }

    /// Computes the canonical envelope hash with the inherited local exclusions.
    pub fn compute_artifact_hash(&self) -> Result<Hash, VerifiedArtifactError> {
        let json = commented_artifact_json(self)?;
        let excluded = artifact_hash_excluded_paths();
        Ok(CanonicalHashDomain::new(
            HashClass::Artifact,
            COMMENTED_ARTIFACT_SCHEMA_FAMILY,
            COMMENTED_ARTIFACT_SCHEMA_VERSION,
        )
        .hash(&json, &excluded))
    }

    fn validate(&self) -> Result<CanonicalJson, VerifiedArtifactError> {
        if self.artifact.schema_version != SchemaVersion::new(1, 2) {
            return Err(invalid(
                "$.artifact.schema_version",
                "requires normal schema version 1.2",
            ));
        }
        let artifact = verified_artifact_json(&self.artifact)?;
        if self.artifact.module.language_edition != self.artifact.provenance.language_edition {
            return Err(invalid(
                "$.artifact.provenance.language_edition",
                "must equal the module edition",
            ));
        }
        if self.comments == CanonicalJson::Null {
            return Ok(artifact);
        }
        let fields = object(&self.comments, "$.comments")?;
        exact_fields(
            fields,
            &[
                "original",
                "loaded",
                "lexical",
                "loading",
                "preprocess",
                "comments",
                "whitespace",
                "landmarks",
            ],
            "$.comments",
        )?;
        let original = string(
            field(fields, "original", "$.comments")?,
            "$.comments.original",
        )?;
        let loaded = string(field(fields, "loaded", "$.comments")?, "$.comments.loaded")?;
        let lexical = string(
            field(fields, "lexical", "$.comments")?,
            "$.comments.lexical",
        )?;
        if hash_text(loaded) != self.artifact.source_hash {
            return Err(invalid(
                "$.comments.loaded",
                "does not match the artifact source hash",
            ));
        }
        let (normalized, segments) = normalize_source_text(original);
        let segments = segments.unwrap_or_else(|| {
            vec![LoadingMapSegment::Original {
                loaded: TextRange {
                    start: 0,
                    end: loaded.len(),
                },
                original: TextRange {
                    start: 0,
                    end: original.len(),
                },
            }]
        });
        let loading = array(
            field(fields, "loading", "$.comments")?,
            "$.comments.loading",
        )?;
        if normalized != loaded || loading.len() != segments.len() {
            return Err(invalid(
                "$.comments.loading",
                "must exactly match Session normalization",
            ));
        }
        for (row, segment) in loading.iter().zip(segments) {
            let row = tuple(row, 3, "$.comments.loading")?;
            let (kind, loaded_range, original_range) = match segment {
                LoadingMapSegment::Original { loaded, original } => {
                    ("original", Some(loaded), original)
                }
                LoadingMapSegment::NormalizedNewline { loaded, original } => {
                    ("newline", Some(loaded), original)
                }
                LoadingMapSegment::RemovedLeadingBom { original } => ("bom", None, original),
                _ => {
                    return Err(invalid(
                        "$.comments.loading",
                        "unsupported normalization segment",
                    ));
                }
            };
            let actual_loaded = if row[1] == CanonicalJson::Null {
                None
            } else {
                Some(range(&row[1], loaded, "$.comments.loading")?)
            };
            let actual_original = range(&row[2], original, "$.comments.loading")?;
            if string(&row[0], "$.comments.loading")? != kind
                || actual_loaded != loaded_range.map(|r| (r.start, r.end))
                || actual_original != (original_range.start, original_range.end)
            {
                return Err(invalid(
                    "$.comments.loading",
                    "must exactly match Session normalization",
                ));
            }
        }
        let mut removed = Vec::new();
        let mut current_removed = None;
        let (mut loaded_end, mut lexical_end, mut previous_source_start) = (0, 0, 0);
        let preprocess = array(
            field(fields, "preprocess", "$.comments")?,
            "$.comments.preprocess",
        )?;
        if loaded.is_empty() && preprocess.len() > 1 {
            return Err(invalid(
                "$.comments.preprocess",
                "duplicate empty frame segments",
            ));
        }
        for row in preprocess {
            let row = tuple(row, 4, "$.comments.preprocess")?;
            let kind = string(&row[0], "$.comments.preprocess")?;
            let source = range(&row[2], loaded, "$.comments.preprocess")?;
            if source.0 < previous_source_start {
                return Err(invalid(
                    "$.comments.preprocess",
                    "segments must retain source order",
                ));
            }
            previous_source_start = source.0;
            match kind {
                "original" | "removed" => {
                    if source.0 != loaded_end || (source.0 == source.1 && !loaded.is_empty()) {
                        return Err(invalid(
                            "$.comments.preprocess",
                            "loaded segments must cover the frame once in order",
                        ));
                    }
                    loaded_end = source.1;
                    if kind == "removed" {
                        if row[1] != CanonicalJson::Null {
                            return Err(invalid(
                                "$.comments.preprocess",
                                "removed segment has no lexical range",
                            ));
                        }
                        let comment_kind = string(&row[3], "$.comments.preprocess")?;
                        let raw = &loaded[source.0..source.1];
                        let valid = match comment_kind {
                            "documentation" => raw.starts_with(":::"),
                            "single_line" => {
                                raw.starts_with("::")
                                    && !raw.starts_with(":::")
                                    && !raw.starts_with("::=")
                            }
                            "multi_line" => raw.starts_with("::="),
                            _ => false,
                        };
                        if !valid {
                            return Err(invalid(
                                "$.comments.comments",
                                "unknown comment kind or inconsistent marker",
                            ));
                        }
                        removed.push((source, comment_kind));
                        current_removed = Some(source);
                        continue;
                    }
                    current_removed = None;
                }
                "synthetic" => {
                    if !current_removed.is_some_and(|removed: (usize, usize)| {
                        removed.0 <= source.0 && source.1 <= removed.1
                    }) {
                        return Err(invalid(
                            "$.comments.preprocess",
                            "synthetic anchor must lie in the current removed comment",
                        ));
                    }
                }
                _ => return Err(invalid("$.comments.preprocess", "unknown segment kind")),
            }
            if row[3] != CanonicalJson::Null {
                return Err(invalid(
                    "$.comments.preprocess",
                    "nonremoved segment has no comment kind",
                ));
            }
            let output = range(&row[1], lexical, "$.comments.preprocess")?;
            if output.0 != lexical_end
                || (output.0 == output.1 && !(kind == "original" && loaded.is_empty()))
            {
                return Err(invalid(
                    "$.comments.preprocess",
                    "lexical segments must cover the frame once in order",
                ));
            }
            lexical_end = output.1;
            if (kind == "original" && lexical[output.0..output.1] != loaded[source.0..source.1])
                || (kind == "synthetic"
                    && !lexical[output.0..output.1].chars().all(char::is_whitespace))
            {
                return Err(invalid(
                    "$.comments.preprocess",
                    "original slices must agree and synthetic slices must be whitespace",
                ));
            }
        }
        if loaded_end != loaded.len() || lexical_end != lexical.len() {
            return Err(invalid(
                "$.comments.preprocess",
                "incomplete frames or synthetic anchor outside removed comments",
            ));
        }
        let comments = array(
            field(fields, "comments", "$.comments")?,
            "$.comments.comments",
        )?;
        if comments.len() != removed.len() {
            return Err(invalid(
                "$.comments.comments",
                "must match removed segments one-to-one",
            ));
        }
        for (row, (removed_range, removed_kind)) in comments.iter().zip(&removed) {
            let row = tuple(row, 3, "$.comments.comments")?;
            let span = range(&row[1], loaded, "$.comments.comments")?;
            if span != *removed_range
                || string(&row[0], "$.comments.comments")? != *removed_kind
                || string(&row[2], "$.comments.comments")? != &loaded[span.0..span.1]
            {
                return Err(invalid(
                    "$.comments.comments",
                    "must match raw removed comment slices in order",
                ));
            }
        }
        let mut previous_end = 0;
        for row in array(
            field(fields, "whitespace", "$.comments")?,
            "$.comments.whitespace",
        )? {
            let span = range(row, loaded, "$.comments.whitespace")?;
            if span.0 < previous_end
                || span.0 == span.1
                || !loaded[span.0..span.1].chars().all(char::is_whitespace)
                || removed
                    .iter()
                    .any(|(comment, _)| span.0 < comment.1 && comment.0 < span.1)
            {
                return Err(invalid(
                    "$.comments.whitespace",
                    "requires ordered whitespace outside comments",
                ));
            }
            previous_end = span.1;
        }
        let mut landmarks = Vec::new();
        let mut seen = BTreeSet::new();
        let mut previous_key = None;
        for row in array(
            field(fields, "landmarks", "$.comments")?,
            "$.comments.landmarks",
        )? {
            let row = tuple(row, 5, "$.comments.landmarks")?;
            let kind = string(&row[0], "$.comments.landmarks")?;
            if kind.is_empty() || !kind.chars().all(|ch| ch.is_alphanumeric() || ch == '_') {
                return Err(invalid(
                    "$.comments.landmarks",
                    "requires a nonempty symbolic syntax kind",
                ));
            }
            let span = range(&row[1], loaded, "$.comments.landmarks")?;
            let parent = match &row[2] {
                CanonicalJson::Null => None,
                CanonicalJson::Integer(index) => Some(
                    usize::try_from(*index)
                        .map_err(|_| invalid("$.comments.landmarks", "invalid parent index"))?,
                ),
                _ => {
                    return Err(invalid(
                        "$.comments.landmarks",
                        "parent must be an index or null",
                    ));
                }
            };
            if let Some(parent) = parent {
                let parent_span: &(usize, usize) = landmarks.get(parent).ok_or_else(|| {
                    invalid("$.comments.landmarks", "parent must precede its child")
                })?;
                if parent_span.0 > span.0 || span.1 > parent_span.1 {
                    return Err(invalid(
                        "$.comments.landmarks",
                        "parent must contain its child",
                    ));
                }
            }
            let key = (span.0, std::cmp::Reverse(span.1));
            if previous_key.is_some_and(|previous| previous > key) {
                return Err(invalid(
                    "$.comments.landmarks",
                    "landmarks must retain source order",
                ));
            }
            previous_key = Some(key);
            if !seen.insert((kind, span, parent)) {
                return Err(invalid("$.comments.landmarks", "duplicate landmark"));
            }
            if !matches!(row[3], CanonicalJson::Bool(_)) {
                return Err(invalid("$.comments.landmarks", "recovered must be boolean"));
            }
            if row[4] != CanonicalJson::Null {
                let target = tuple(&row[4], 2, "$.comments.landmarks")?;
                let origin = string(&target[0], "$.comments.landmarks")?;
                let fqn = string(&target[1], "$.comments.landmarks")?;
                if self
                    .artifact
                    .exports
                    .iter()
                    .filter(|export| {
                        export.origin_id == origin && export.fully_qualified_name == fqn
                    })
                    .count()
                    != 1
                {
                    return Err(invalid(
                        "$.comments.landmarks",
                        "target must identify exactly one enclosing export",
                    ));
                }
            }
            landmarks.push(span);
        }
        Ok(artifact)
    }
}

fn hash_string(class: ArtifactHashClass, digest: Hash) -> String {
    ArtifactHashRef::new(
        class,
        COMMENTED_ARTIFACT_SCHEMA_FAMILY,
        COMMENTED_ARTIFACT_SCHEMA_VERSION,
        digest,
    )
    .to_artifact_hash_string()
}

fn invalid(path: &str, reason: &str) -> VerifiedArtifactError {
    VerifiedArtifactError::InvalidField {
        path: path.to_owned(),
        reason: reason.to_owned(),
    }
}

fn object<'a>(
    value: &'a CanonicalJson,
    path: &str,
) -> Result<&'a BTreeMap<String, CanonicalJson>, VerifiedArtifactError> {
    match value {
        CanonicalJson::Object(fields) => Ok(fields),
        _ => Err(VerifiedArtifactError::UnexpectedType {
            path: path.to_owned(),
            expected: "object",
        }),
    }
}

fn field<'a>(
    fields: &'a BTreeMap<String, CanonicalJson>,
    name: &str,
    path: &str,
) -> Result<&'a CanonicalJson, VerifiedArtifactError> {
    fields
        .get(name)
        .ok_or_else(|| VerifiedArtifactError::MissingField {
            path: format!("{path}.{name}"),
        })
}

fn exact_fields(
    fields: &BTreeMap<String, CanonicalJson>,
    names: &[&str],
    path: &str,
) -> Result<(), VerifiedArtifactError> {
    for name in fields.keys() {
        if !names.contains(&name.as_str()) {
            return Err(VerifiedArtifactError::UnknownField {
                path: path.to_owned(),
                field: name.clone(),
            });
        }
    }
    for name in names {
        field(fields, name, path)?;
    }
    Ok(())
}

fn string<'a>(value: &'a CanonicalJson, path: &str) -> Result<&'a str, VerifiedArtifactError> {
    match value {
        CanonicalJson::String(value) => Ok(value),
        _ => Err(VerifiedArtifactError::UnexpectedType {
            path: path.to_owned(),
            expected: "string",
        }),
    }
}

fn array<'a>(
    value: &'a CanonicalJson,
    path: &str,
) -> Result<&'a [CanonicalJson], VerifiedArtifactError> {
    match value {
        CanonicalJson::Array(values) => Ok(values),
        _ => Err(VerifiedArtifactError::UnexpectedType {
            path: path.to_owned(),
            expected: "array",
        }),
    }
}

fn tuple<'a>(
    value: &'a CanonicalJson,
    length: usize,
    path: &str,
) -> Result<&'a [CanonicalJson], VerifiedArtifactError> {
    let row = array(value, path)?;
    if row.len() != length {
        return Err(invalid(path, "wrong tuple arity"));
    }
    Ok(row)
}

fn range(
    value: &CanonicalJson,
    text: &str,
    path: &str,
) -> Result<(usize, usize), VerifiedArtifactError> {
    let row = tuple(value, 2, path)?;
    let [CanonicalJson::Integer(start), CanonicalJson::Integer(end)] = row else {
        return Err(invalid(path, "range offsets must be integers"));
    };
    let start = usize::try_from(*start).map_err(|_| invalid(path, "invalid byte offset"))?;
    let end = usize::try_from(*end).map_err(|_| invalid(path, "invalid byte offset"))?;
    if text.get(start..end).is_none() {
        return Err(invalid(path, "range must be bounded and UTF8-aligned"));
    }
    Ok((start, end))
}
