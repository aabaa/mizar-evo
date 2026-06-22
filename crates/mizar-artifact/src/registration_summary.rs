//! Checker-facing registration summary schema.
//!
//! Design authority: [registration_summary.md](../../../../doc/design/mizar-artifact/en/registration_summary.md).

use crate::store::{
    ArtifactError, ArtifactStore, CanonicalJson, canonical_hash, check_schema_version,
    ensure_sorted_by, ensure_unique_by, expect_object, hash_json, optional_string,
    optional_string_json, parse_json, reject_unknown_fields, required_array, required_hash,
    required_string, validate_portable_path_text,
};
use mizar_session::Hash;

/// Current `RegistrationSummary` schema version.
pub const REGISTRATION_SUMMARY_SCHEMA_VERSION: &str = "mizar-artifact/registration-summary/v1";

const SCHEMA: &str = "RegistrationSummary";
const SUPPORTED: &[&str] = &[REGISTRATION_SUMMARY_SCHEMA_VERSION];

/// Activated exported registration projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportedRegistration {
    /// Stable registration id.
    pub registration_id: String,
    /// User-facing label.
    pub label: String,
    /// Registration kind.
    pub kind: String,
    /// Checker trigger key.
    pub trigger_key: String,
    /// Canonical source pattern.
    pub pattern: String,
    /// Generated fact or reduction summary.
    pub result: String,
    /// Accepted proof status projection.
    pub proof_status: String,
    /// Optional witness id.
    pub witness_id: Option<String>,
}

/// Content-hash reference to an external cluster/reduction trace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterTraceRef {
    /// Portable trace path.
    pub path: String,
    /// Trace content hash.
    pub content_hash: Hash,
    /// Trace kind.
    pub trace_kind: String,
    /// Trace schema version.
    pub trace_schema_version: String,
}

/// Checker-facing active-registration projection for one module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrationSummary {
    /// Schema version.
    pub schema_version: String,
    /// Package id.
    pub package_id: String,
    /// Module path.
    pub module_path: String,
    /// Hash of active registrations and trace references.
    pub registration_hash: Hash,
    /// Activated registration entries.
    pub registrations: Vec<ExportedRegistration>,
    /// External cluster or reduction trace references.
    pub cluster_trace_refs: Vec<ClusterTraceRef>,
    /// Local metadata excluded from `registration_hash`.
    pub local_metadata: Option<String>,
}

impl ExportedRegistration {
    /// Creates an exported registration entry.
    #[allow(clippy::too_many_arguments)] // reason: constructor mirrors stable registration summary fields
    pub fn new(
        registration_id: impl Into<String>,
        label: impl Into<String>,
        kind: impl Into<String>,
        trigger_key: impl Into<String>,
        pattern: impl Into<String>,
        result: impl Into<String>,
        proof_status: impl Into<String>,
        witness_id: Option<String>,
    ) -> Self {
        Self {
            registration_id: registration_id.into(),
            label: label.into(),
            kind: kind.into(),
            trigger_key: trigger_key.into(),
            pattern: pattern.into(),
            result: result.into(),
            proof_status: proof_status.into(),
            witness_id,
        }
    }

    fn sort_key(&self) -> String {
        format!(
            "{}\u{1f}{}\u{1f}{}\u{1f}{}",
            self.kind, self.trigger_key, self.registration_id, self.label
        )
    }

    fn to_json_value(&self) -> CanonicalJson {
        CanonicalJson::object([
            ("kind", CanonicalJson::string(&self.kind)),
            ("label", CanonicalJson::string(&self.label)),
            ("pattern", CanonicalJson::string(&self.pattern)),
            ("proof_status", CanonicalJson::string(&self.proof_status)),
            (
                "registration_id",
                CanonicalJson::string(&self.registration_id),
            ),
            ("result", CanonicalJson::string(&self.result)),
            ("trigger_key", CanonicalJson::string(&self.trigger_key)),
            ("witness_id", optional_string_json(&self.witness_id)),
        ])
    }

    fn from_json_value(value: &CanonicalJson) -> Result<Self, ArtifactError> {
        let object = expect_object(value, SCHEMA)?;
        reject_unknown_fields(
            object,
            SCHEMA,
            &[
                "kind",
                "label",
                "pattern",
                "proof_status",
                "registration_id",
                "result",
                "trigger_key",
                "witness_id",
            ],
        )?;
        Ok(Self::new(
            required_string(object, SCHEMA, "registration_id")?,
            required_string(object, SCHEMA, "label")?,
            required_string(object, SCHEMA, "kind")?,
            required_string(object, SCHEMA, "trigger_key")?,
            required_string(object, SCHEMA, "pattern")?,
            required_string(object, SCHEMA, "result")?,
            required_string(object, SCHEMA, "proof_status")?,
            optional_string(object, SCHEMA, "witness_id")?,
        ))
    }
}

impl ClusterTraceRef {
    /// Creates a trace reference.
    pub fn new(
        path: impl Into<String>,
        content_hash: Hash,
        trace_kind: impl Into<String>,
        trace_schema_version: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into(),
            content_hash,
            trace_kind: trace_kind.into(),
            trace_schema_version: trace_schema_version.into(),
        }
    }

    /// Validates that trace bytes match the recorded hash.
    pub fn validate_content(&self, content: &str) -> Result<(), ArtifactError> {
        let actual = crate::store::content_hash_str(content);
        if actual == self.content_hash {
            Ok(())
        } else {
            Err(ArtifactError::HashMismatch {
                expected: self.content_hash,
                actual,
            })
        }
    }

    fn sort_key(&self) -> String {
        format!(
            "{}\u{1f}{}",
            self.path,
            crate::store::hash_to_hex(self.content_hash)
        )
    }

    fn to_json_value(&self) -> CanonicalJson {
        CanonicalJson::object([
            ("content_hash", hash_json(self.content_hash)),
            ("path", CanonicalJson::string(&self.path)),
            ("trace_kind", CanonicalJson::string(&self.trace_kind)),
            (
                "trace_schema_version",
                CanonicalJson::string(&self.trace_schema_version),
            ),
        ])
    }

    fn from_json_value(value: &CanonicalJson) -> Result<Self, ArtifactError> {
        let object = expect_object(value, SCHEMA)?;
        reject_unknown_fields(
            object,
            SCHEMA,
            &["content_hash", "path", "trace_kind", "trace_schema_version"],
        )?;
        let path = required_string(object, SCHEMA, "path")?;
        validate_portable_path_text(&path)?;
        Ok(Self::new(
            path,
            required_hash(object, SCHEMA, "content_hash")?,
            required_string(object, SCHEMA, "trace_kind")?,
            required_string(object, SCHEMA, "trace_schema_version")?,
        ))
    }
}

impl RegistrationSummary {
    /// Creates a summary and computes `registration_hash`.
    pub fn new(
        package_id: impl Into<String>,
        module_path: impl Into<String>,
        registrations: Vec<ExportedRegistration>,
        cluster_trace_refs: Vec<ClusterTraceRef>,
        local_metadata: Option<String>,
    ) -> Self {
        let mut summary = Self {
            schema_version: REGISTRATION_SUMMARY_SCHEMA_VERSION.to_owned(),
            package_id: package_id.into(),
            module_path: module_path.into(),
            registration_hash: Hash::from_bytes([0; Hash::BYTE_LEN]),
            registrations,
            cluster_trace_refs,
            local_metadata,
        };
        summary.sort_canonical();
        summary.registration_hash = summary.compute_registration_hash();
        summary
    }

    /// Serializes the summary as canonical JSON.
    pub fn to_json_string(&self) -> String {
        self.to_json_value().to_canonical_string()
    }

    /// Reads and validates a summary from JSON text.
    pub fn from_json_str(input: &str) -> Result<Self, ArtifactError> {
        let value = parse_json(input)?;
        let object = expect_object(&value, SCHEMA)?;
        reject_unknown_fields(
            object,
            SCHEMA,
            &[
                "schema_version",
                "package_id",
                "module_path",
                "registration_hash",
                "registrations",
                "cluster_trace_refs",
                "local_metadata",
            ],
        )?;
        let schema_version = check_schema_version(object, SCHEMA, SUPPORTED)?;
        let mut summary = Self {
            schema_version,
            package_id: required_string(object, SCHEMA, "package_id")?,
            module_path: required_string(object, SCHEMA, "module_path")?,
            registration_hash: required_hash(object, SCHEMA, "registration_hash")?,
            registrations: required_array(object, SCHEMA, "registrations")?
                .iter()
                .map(ExportedRegistration::from_json_value)
                .collect::<Result<Vec<_>, _>>()?,
            cluster_trace_refs: required_array(object, SCHEMA, "cluster_trace_refs")?
                .iter()
                .map(ClusterTraceRef::from_json_value)
                .collect::<Result<Vec<_>, _>>()?,
            local_metadata: optional_string(object, SCHEMA, "local_metadata")?,
        };
        summary.validate_order_and_uniqueness()?;
        let computed = summary.compute_registration_hash();
        if computed != summary.registration_hash {
            return Err(ArtifactError::HashMismatch {
                expected: summary.registration_hash,
                actual: computed,
            });
        }
        summary.sort_canonical();
        Ok(summary)
    }

    /// Writes this summary through an artifact store.
    pub fn write_to_store(
        &self,
        store: &ArtifactStore,
        relative_path: &str,
    ) -> Result<Hash, ArtifactError> {
        store.write_named_str(relative_path, &self.to_json_string())
    }

    /// Reads this summary through an artifact store.
    pub fn read_from_store(
        store: &ArtifactStore,
        relative_path: &str,
        expected_hash: Option<Hash>,
    ) -> Result<Self, ArtifactError> {
        let content = store.read_named_str(relative_path, expected_hash)?;
        Self::from_json_str(&content)
    }

    fn to_json_value(&self) -> CanonicalJson {
        CanonicalJson::object([
            (
                "cluster_trace_refs",
                CanonicalJson::array(
                    self.cluster_trace_refs
                        .iter()
                        .map(ClusterTraceRef::to_json_value)
                        .collect(),
                ),
            ),
            ("local_metadata", optional_string_json(&self.local_metadata)),
            ("module_path", CanonicalJson::string(&self.module_path)),
            ("package_id", CanonicalJson::string(&self.package_id)),
            ("registration_hash", hash_json(self.registration_hash)),
            (
                "registrations",
                CanonicalJson::array(
                    self.registrations
                        .iter()
                        .map(ExportedRegistration::to_json_value)
                        .collect(),
                ),
            ),
            (
                "schema_version",
                CanonicalJson::string(&self.schema_version),
            ),
        ])
    }

    fn registration_hash_value(&self) -> CanonicalJson {
        CanonicalJson::object([
            (
                "cluster_trace_refs",
                CanonicalJson::array(
                    self.cluster_trace_refs
                        .iter()
                        .map(ClusterTraceRef::to_json_value)
                        .collect(),
                ),
            ),
            ("module_path", CanonicalJson::string(&self.module_path)),
            ("package_id", CanonicalJson::string(&self.package_id)),
            (
                "registrations",
                CanonicalJson::array(
                    self.registrations
                        .iter()
                        .map(ExportedRegistration::to_json_value)
                        .collect(),
                ),
            ),
            (
                "schema_version",
                CanonicalJson::string(&self.schema_version),
            ),
        ])
    }

    fn compute_registration_hash(&self) -> Hash {
        canonical_hash(
            "registration-summary/active-registrations/v1",
            &self.registration_hash_value(),
        )
    }

    fn sort_canonical(&mut self) {
        self.registrations
            .sort_by_key(ExportedRegistration::sort_key);
        self.cluster_trace_refs
            .sort_by_key(ClusterTraceRef::sort_key);
    }

    fn validate_order_and_uniqueness(&self) -> Result<(), ArtifactError> {
        ensure_sorted_by(
            &self.registrations,
            SCHEMA,
            "registrations",
            ExportedRegistration::sort_key,
        )?;
        ensure_sorted_by(
            &self.cluster_trace_refs,
            SCHEMA,
            "cluster_trace_refs",
            ClusterTraceRef::sort_key,
        )?;
        ensure_unique_by(&self.registrations, SCHEMA, ExportedRegistration::sort_key)?;
        ensure_unique_by(&self.cluster_trace_refs, SCHEMA, ClusterTraceRef::sort_key)
    }
}

#[cfg(test)]
mod tests {
    use super::{ClusterTraceRef, ExportedRegistration, RegistrationSummary};
    use crate::store::{ArtifactError, content_hash_str};

    #[test]
    fn registration_summary_round_trips() {
        let summary = fixture(Some("local".to_owned()));
        let parsed = RegistrationSummary::from_json_str(&summary.to_json_string()).unwrap();

        assert_eq!(parsed, summary);
    }

    #[test]
    fn trace_reference_validates_by_hash() {
        let trace = ClusterTraceRef::new(
            "traces/alpha.cluster.json",
            content_hash_str("{\"trace\":1}"),
            "cluster",
            "trace-v1",
        );

        assert!(trace.validate_content("{\"trace\":1}").is_ok());
        assert!(matches!(
            trace.validate_content("{\"trace\":2}"),
            Err(ArtifactError::HashMismatch { .. })
        ));
    }

    #[test]
    fn deterministic_order_is_enforced() {
        let mut summary = fixture(None);
        summary.registrations.reverse();

        assert!(matches!(
            RegistrationSummary::from_json_str(&summary.to_json_string()),
            Err(ArtifactError::InvalidField {
                field: "registrations",
                ..
            })
        ));
    }

    #[test]
    fn tampered_registration_hash_and_invalid_trace_path_are_rejected() {
        let mut tampered = fixture(None);
        tampered.registration_hash = content_hash_str("wrong registrations");
        assert!(matches!(
            RegistrationSummary::from_json_str(&tampered.to_json_string()),
            Err(ArtifactError::HashMismatch { .. })
        ));

        let json = fixture(None)
            .to_json_string()
            .replace("traces/alpha.cluster.json", "../trace.json");
        assert!(matches!(
            RegistrationSummary::from_json_str(&json),
            Err(ArtifactError::InvalidPath { .. })
        ));
    }

    fn fixture(local_metadata: Option<String>) -> RegistrationSummary {
        RegistrationSummary::new(
            "pkg",
            "alpha.core",
            vec![
                ExportedRegistration::new(
                    "reg-2",
                    "RegB",
                    "cluster",
                    "T:integer",
                    "cluster integer -> non empty",
                    "non empty integer",
                    "kernel_verified",
                    Some("wit-2".to_owned()),
                ),
                ExportedRegistration::new(
                    "reg-1",
                    "RegA",
                    "existential",
                    "T:set",
                    "cluster set -> non empty",
                    "inhabited set",
                    "kernel_verified",
                    Some("wit-1".to_owned()),
                ),
            ],
            vec![ClusterTraceRef::new(
                "traces/alpha.cluster.json",
                content_hash_str("{\"trace\":1}"),
                "cluster",
                "trace-v1",
            )],
            local_metadata,
        )
    }
}
