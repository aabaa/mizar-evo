//! Proof witness reference schema.
//!
//! Design authority: [proof_witness.md](../../../../doc/design/mizar-artifact/en/proof_witness.md).

use crate::store::{
    ArtifactError, CanonicalJson, check_schema_version, ensure_sorted_by, expect_object, hash_json,
    parse_json, reject_unknown_fields, required_array, required_hash, required_string,
    validate_portable_path_text,
};
use mizar_session::Hash;

/// Current proof witness reference schema version.
pub const PROOF_WITNESS_REF_SCHEMA_VERSION: &str = "mizar-artifact/proof-witness-ref/v1";

const SCHEMA: &str = "ProofWitnessRef";
const SUPPORTED: &[&str] = &[PROOF_WITNESS_REF_SCHEMA_VERSION];

/// Stable reference to an external proof witness file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofWitnessRef {
    /// Reference schema version.
    pub schema_version: String,
    /// Stable witness id.
    pub witness_id: String,
    /// Obligation id proven by this witness.
    pub obligation_id: String,
    /// Portable path to the witness bytes.
    pub path: String,
    /// Content hash of the witness bytes.
    pub content_hash: Hash,
    /// Evidence kind, such as `atp_certificate` or `builtin_certificate`.
    pub evidence_kind: String,
    /// Kernel status projected by proof/kernel producer crates.
    pub kernel_status: String,
    /// Kernel profile used for acceptance.
    pub kernel_profile: String,
    /// Producer that accepted or selected this witness.
    pub accepted_by: String,
    /// Stable used-axiom identifiers.
    pub used_axioms: Vec<String>,
}

impl ProofWitnessRef {
    /// Creates a witness reference from already-hashed bytes.
    #[allow(clippy::too_many_arguments)] // reason: constructor mirrors stable witness reference fields
    pub fn new(
        witness_id: impl Into<String>,
        obligation_id: impl Into<String>,
        path: impl Into<String>,
        content_hash: Hash,
        evidence_kind: impl Into<String>,
        kernel_status: impl Into<String>,
        kernel_profile: impl Into<String>,
        accepted_by: impl Into<String>,
        used_axioms: Vec<String>,
    ) -> Self {
        let mut reference = Self {
            schema_version: PROOF_WITNESS_REF_SCHEMA_VERSION.to_owned(),
            witness_id: witness_id.into(),
            obligation_id: obligation_id.into(),
            path: path.into(),
            content_hash,
            evidence_kind: evidence_kind.into(),
            kernel_status: kernel_status.into(),
            kernel_profile: kernel_profile.into(),
            accepted_by: accepted_by.into(),
            used_axioms,
        };
        reference.used_axioms.sort();
        reference
    }

    /// Creates a reference and computes the witness content hash.
    #[allow(clippy::too_many_arguments)] // reason: constructor mirrors stable witness reference fields
    pub fn from_content(
        witness_id: impl Into<String>,
        obligation_id: impl Into<String>,
        path: impl Into<String>,
        content: &str,
        evidence_kind: impl Into<String>,
        kernel_status: impl Into<String>,
        kernel_profile: impl Into<String>,
        accepted_by: impl Into<String>,
        used_axioms: Vec<String>,
    ) -> Self {
        Self::new(
            witness_id,
            obligation_id,
            path,
            crate::store::content_hash_str(content),
            evidence_kind,
            kernel_status,
            kernel_profile,
            accepted_by,
            used_axioms,
        )
    }

    /// Validates witness bytes against the recorded hash.
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

    /// Serializes this reference as canonical JSON.
    pub fn to_json_string(&self) -> String {
        self.to_json_value().to_canonical_string()
    }

    /// Reads and validates a proof witness reference from JSON text.
    pub fn from_json_str(input: &str) -> Result<Self, ArtifactError> {
        let value = parse_json(input)?;
        Self::from_json_value(&value)
    }

    pub(crate) fn sort_key(&self) -> String {
        format!(
            "{}\u{1f}{}\u{1f}{}\u{1f}{}",
            self.obligation_id,
            self.witness_id,
            self.path,
            crate::store::hash_to_hex(self.content_hash)
        )
    }

    pub(crate) fn to_json_value(&self) -> CanonicalJson {
        CanonicalJson::object([
            ("accepted_by", CanonicalJson::string(&self.accepted_by)),
            ("content_hash", hash_json(self.content_hash)),
            ("evidence_kind", CanonicalJson::string(&self.evidence_kind)),
            (
                "kernel_profile",
                CanonicalJson::string(&self.kernel_profile),
            ),
            ("kernel_status", CanonicalJson::string(&self.kernel_status)),
            ("obligation_id", CanonicalJson::string(&self.obligation_id)),
            ("path", CanonicalJson::string(&self.path)),
            (
                "schema_version",
                CanonicalJson::string(&self.schema_version),
            ),
            (
                "used_axioms",
                CanonicalJson::array(self.used_axioms.iter().map(CanonicalJson::string).collect()),
            ),
            ("witness_id", CanonicalJson::string(&self.witness_id)),
        ])
    }

    pub(crate) fn from_json_value(value: &CanonicalJson) -> Result<Self, ArtifactError> {
        let object = expect_object(value, SCHEMA)?;
        reject_unknown_fields(
            object,
            SCHEMA,
            &[
                "accepted_by",
                "content_hash",
                "evidence_kind",
                "kernel_profile",
                "kernel_status",
                "obligation_id",
                "path",
                "schema_version",
                "used_axioms",
                "witness_id",
            ],
        )?;
        let schema_version = check_schema_version(object, SCHEMA, SUPPORTED)?;
        let path = required_string(object, SCHEMA, "path")?;
        validate_portable_path_text(&path)?;
        let mut reference = Self {
            schema_version,
            witness_id: required_string(object, SCHEMA, "witness_id")?,
            obligation_id: required_string(object, SCHEMA, "obligation_id")?,
            path,
            content_hash: required_hash(object, SCHEMA, "content_hash")?,
            evidence_kind: required_string(object, SCHEMA, "evidence_kind")?,
            kernel_status: required_string(object, SCHEMA, "kernel_status")?,
            kernel_profile: required_string(object, SCHEMA, "kernel_profile")?,
            accepted_by: required_string(object, SCHEMA, "accepted_by")?,
            used_axioms: required_array(object, SCHEMA, "used_axioms")?
                .iter()
                .map(|value| match value {
                    CanonicalJson::String(value) => Ok(value.clone()),
                    _ => Err(ArtifactError::InvalidField {
                        schema: SCHEMA,
                        field: "used_axioms",
                        reason: "expected string entries".to_owned(),
                    }),
                })
                .collect::<Result<Vec<_>, _>>()?,
        };
        ensure_sorted_by(&reference.used_axioms, SCHEMA, "used_axioms", Clone::clone)?;
        reference.used_axioms.sort();
        Ok(reference)
    }
}

#[cfg(test)]
mod tests {
    use super::ProofWitnessRef;
    use crate::store::ArtifactError;

    #[test]
    fn proof_witness_ref_round_trips() {
        let reference = fixture();
        let parsed = ProofWitnessRef::from_json_str(&reference.to_json_string()).unwrap();

        assert_eq!(parsed, reference);
    }

    #[test]
    fn witness_hash_mismatch_is_detected() {
        let reference = fixture();

        assert!(reference.validate_content("{\"proof\":true}").is_ok());
        assert!(matches!(
            reference.validate_content("{\"proof\":false}"),
            Err(ArtifactError::HashMismatch { .. })
        ));
    }

    #[test]
    fn unsupported_version_unsorted_axioms_and_invalid_paths_are_rejected() {
        let unsupported = fixture().to_json_string().replace(
            "mizar-artifact/proof-witness-ref/v1",
            "mizar-artifact/proof-witness-ref/v2",
        );
        assert!(matches!(
            ProofWitnessRef::from_json_str(&unsupported),
            Err(ArtifactError::UnsupportedSchemaVersion { .. })
        ));

        let mut unsorted = fixture();
        unsorted.used_axioms.reverse();
        assert!(matches!(
            ProofWitnessRef::from_json_str(&unsorted.to_json_string()),
            Err(ArtifactError::InvalidField {
                field: "used_axioms",
                ..
            })
        ));

        let invalid_path = fixture()
            .to_json_string()
            .replace("proof-witnesses/alpha/vc-1.json", "../vc-1.json");
        assert!(matches!(
            ProofWitnessRef::from_json_str(&invalid_path),
            Err(ArtifactError::InvalidPath { .. })
        ));
    }

    fn fixture() -> ProofWitnessRef {
        ProofWitnessRef::from_content(
            "wit-1",
            "vc-1",
            "proof-witnesses/alpha/vc-1.json",
            "{\"proof\":true}",
            "atp_certificate",
            "kernel_verified",
            "kernel-profile-v1",
            "mizar-proof",
            vec!["B".to_owned(), "A".to_owned()],
        )
    }
}
