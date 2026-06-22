//! Resolver-facing module summary schema.
//!
//! Design authority: [module_summary.md](../../../../doc/design/mizar-artifact/en/module_summary.md).

use crate::store::{
    ArtifactError, ArtifactStore, CanonicalJson, canonical_hash, check_schema_version,
    ensure_sorted_by, ensure_unique_by, expect_object, hash_json, optional_string,
    optional_string_json, parse_json, reject_unknown_fields, required_array, required_hash,
    required_string,
};
use mizar_session::Hash;

/// Current `ModuleSummary` schema version.
pub const MODULE_SUMMARY_SCHEMA_VERSION: &str = "mizar-artifact/module-summary/v1";

const SCHEMA: &str = "ModuleSummary";
const SUPPORTED: &[&str] = &[MODULE_SUMMARY_SCHEMA_VERSION];

/// Dependency-facing exported declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportedItem {
    /// Declaration kind, such as `predicate`, `functor`, or `theorem`.
    pub kind: String,
    /// Stable fully-qualified name.
    pub fqn: String,
    /// Exported surface spelling.
    pub spelling: String,
    /// Export visibility spelling.
    pub visibility: String,
    /// Canonical dependency-facing signature text.
    pub signature: String,
    /// Defining module when this export is a re-export.
    pub defining_module: Option<String>,
}

/// Exported theorem, proof, definition, or registration label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportedLabel {
    /// Stable label id.
    pub label_id: String,
    /// User-facing label spelling.
    pub spelling: String,
    /// Label kind.
    pub kind: String,
    /// Optional item FQN that owns the label.
    pub owner_fqn: Option<String>,
}

/// Interface hash imported by this module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedInterface {
    /// Imported module path.
    pub module_path: String,
    /// Imported module interface hash.
    pub interface_hash: Hash,
}

/// Resolver-facing exported interface projection for one module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleSummary {
    /// Schema version.
    pub schema_version: String,
    /// Package id.
    pub package_id: String,
    /// Module path.
    pub module_path: String,
    /// Source hash, excluded from `interface_hash`.
    pub source_hash: Hash,
    /// Hash of importer-visible interface content.
    pub interface_hash: Hash,
    /// Exported declarations.
    pub exports: Vec<ExportedItem>,
    /// Exported labels.
    pub exported_labels: Vec<ExportedLabel>,
    /// Direct imported interface hashes.
    pub imported_interfaces: Vec<ImportedInterface>,
    /// Local metadata excluded from `interface_hash`.
    pub local_metadata: Option<String>,
}

impl ExportedItem {
    /// Creates an exported item projection.
    pub fn new(
        kind: impl Into<String>,
        fqn: impl Into<String>,
        spelling: impl Into<String>,
        visibility: impl Into<String>,
        signature: impl Into<String>,
        defining_module: Option<String>,
    ) -> Self {
        Self {
            kind: kind.into(),
            fqn: fqn.into(),
            spelling: spelling.into(),
            visibility: visibility.into(),
            signature: signature.into(),
            defining_module,
        }
    }

    pub(crate) fn sort_key(&self) -> String {
        format!(
            "{}\u{1f}{}\u{1f}{}\u{1f}{}",
            self.kind, self.fqn, self.spelling, self.signature
        )
    }

    pub(crate) fn to_json_value(&self) -> CanonicalJson {
        CanonicalJson::object([
            (
                "defining_module",
                optional_string_json(&self.defining_module),
            ),
            ("fqn", CanonicalJson::string(&self.fqn)),
            ("kind", CanonicalJson::string(&self.kind)),
            ("signature", CanonicalJson::string(&self.signature)),
            ("spelling", CanonicalJson::string(&self.spelling)),
            ("visibility", CanonicalJson::string(&self.visibility)),
        ])
    }

    pub(crate) fn from_json_value(value: &CanonicalJson) -> Result<Self, ArtifactError> {
        let object = expect_object(value, SCHEMA)?;
        reject_unknown_fields(
            object,
            SCHEMA,
            &[
                "defining_module",
                "fqn",
                "kind",
                "signature",
                "spelling",
                "visibility",
            ],
        )?;
        Ok(Self::new(
            required_string(object, SCHEMA, "kind")?,
            required_string(object, SCHEMA, "fqn")?,
            required_string(object, SCHEMA, "spelling")?,
            required_string(object, SCHEMA, "visibility")?,
            required_string(object, SCHEMA, "signature")?,
            optional_string(object, SCHEMA, "defining_module")?,
        ))
    }
}

impl ExportedLabel {
    /// Creates an exported label projection.
    pub fn new(
        label_id: impl Into<String>,
        spelling: impl Into<String>,
        kind: impl Into<String>,
        owner_fqn: Option<String>,
    ) -> Self {
        Self {
            label_id: label_id.into(),
            spelling: spelling.into(),
            kind: kind.into(),
            owner_fqn,
        }
    }

    fn sort_key(&self) -> String {
        format!(
            "{}\u{1f}{}\u{1f}{}",
            self.kind, self.label_id, self.spelling
        )
    }

    fn to_json_value(&self) -> CanonicalJson {
        CanonicalJson::object([
            ("kind", CanonicalJson::string(&self.kind)),
            ("label_id", CanonicalJson::string(&self.label_id)),
            ("owner_fqn", optional_string_json(&self.owner_fqn)),
            ("spelling", CanonicalJson::string(&self.spelling)),
        ])
    }

    fn from_json_value(value: &CanonicalJson) -> Result<Self, ArtifactError> {
        let object = expect_object(value, SCHEMA)?;
        reject_unknown_fields(
            object,
            SCHEMA,
            &["kind", "label_id", "owner_fqn", "spelling"],
        )?;
        Ok(Self::new(
            required_string(object, SCHEMA, "label_id")?,
            required_string(object, SCHEMA, "spelling")?,
            required_string(object, SCHEMA, "kind")?,
            optional_string(object, SCHEMA, "owner_fqn")?,
        ))
    }
}

impl ImportedInterface {
    /// Creates an imported interface hash record.
    pub fn new(module_path: impl Into<String>, interface_hash: Hash) -> Self {
        Self {
            module_path: module_path.into(),
            interface_hash,
        }
    }

    fn sort_key(&self) -> String {
        format!(
            "{}\u{1f}{}",
            self.module_path,
            crate::store::hash_to_hex(self.interface_hash)
        )
    }

    fn to_json_value(&self) -> CanonicalJson {
        CanonicalJson::object([
            ("interface_hash", hash_json(self.interface_hash)),
            ("module_path", CanonicalJson::string(&self.module_path)),
        ])
    }

    fn from_json_value(value: &CanonicalJson) -> Result<Self, ArtifactError> {
        let object = expect_object(value, SCHEMA)?;
        reject_unknown_fields(object, SCHEMA, &["interface_hash", "module_path"])?;
        Ok(Self::new(
            required_string(object, SCHEMA, "module_path")?,
            required_hash(object, SCHEMA, "interface_hash")?,
        ))
    }
}

impl ModuleSummary {
    /// Creates a summary and computes `interface_hash`.
    pub fn new(
        package_id: impl Into<String>,
        module_path: impl Into<String>,
        source_hash: Hash,
        exports: Vec<ExportedItem>,
        exported_labels: Vec<ExportedLabel>,
        imported_interfaces: Vec<ImportedInterface>,
        local_metadata: Option<String>,
    ) -> Self {
        let mut summary = Self {
            schema_version: MODULE_SUMMARY_SCHEMA_VERSION.to_owned(),
            package_id: package_id.into(),
            module_path: module_path.into(),
            source_hash,
            interface_hash: Hash::from_bytes([0; Hash::BYTE_LEN]),
            exports,
            exported_labels,
            imported_interfaces,
            local_metadata,
        };
        summary.sort_canonical();
        summary.interface_hash = summary.compute_interface_hash();
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
                "source_hash",
                "interface_hash",
                "exports",
                "exported_labels",
                "imported_interfaces",
                "local_metadata",
            ],
        )?;
        let schema_version = check_schema_version(object, SCHEMA, SUPPORTED)?;
        let mut summary = Self {
            schema_version,
            package_id: required_string(object, SCHEMA, "package_id")?,
            module_path: required_string(object, SCHEMA, "module_path")?,
            source_hash: required_hash(object, SCHEMA, "source_hash")?,
            interface_hash: required_hash(object, SCHEMA, "interface_hash")?,
            exports: required_array(object, SCHEMA, "exports")?
                .iter()
                .map(ExportedItem::from_json_value)
                .collect::<Result<Vec<_>, _>>()?,
            exported_labels: required_array(object, SCHEMA, "exported_labels")?
                .iter()
                .map(ExportedLabel::from_json_value)
                .collect::<Result<Vec<_>, _>>()?,
            imported_interfaces: required_array(object, SCHEMA, "imported_interfaces")?
                .iter()
                .map(ImportedInterface::from_json_value)
                .collect::<Result<Vec<_>, _>>()?,
            local_metadata: optional_string(object, SCHEMA, "local_metadata")?,
        };
        summary.validate_order_and_uniqueness()?;
        let computed = summary.compute_interface_hash();
        if computed != summary.interface_hash {
            return Err(ArtifactError::HashMismatch {
                expected: summary.interface_hash,
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
                "exported_labels",
                CanonicalJson::array(
                    self.exported_labels
                        .iter()
                        .map(ExportedLabel::to_json_value)
                        .collect(),
                ),
            ),
            (
                "exports",
                CanonicalJson::array(
                    self.exports
                        .iter()
                        .map(ExportedItem::to_json_value)
                        .collect(),
                ),
            ),
            ("imported_interfaces", self.imported_interfaces_json()),
            ("interface_hash", hash_json(self.interface_hash)),
            ("local_metadata", optional_string_json(&self.local_metadata)),
            ("module_path", CanonicalJson::string(&self.module_path)),
            ("package_id", CanonicalJson::string(&self.package_id)),
            (
                "schema_version",
                CanonicalJson::string(&self.schema_version),
            ),
            ("source_hash", hash_json(self.source_hash)),
        ])
    }

    fn interface_hash_value(&self) -> CanonicalJson {
        CanonicalJson::object([
            (
                "exported_labels",
                CanonicalJson::array(
                    self.exported_labels
                        .iter()
                        .map(ExportedLabel::to_json_value)
                        .collect(),
                ),
            ),
            (
                "exports",
                CanonicalJson::array(
                    self.exports
                        .iter()
                        .map(ExportedItem::to_json_value)
                        .collect(),
                ),
            ),
            ("imported_interfaces", self.imported_interfaces_json()),
            ("module_path", CanonicalJson::string(&self.module_path)),
            ("package_id", CanonicalJson::string(&self.package_id)),
            (
                "schema_version",
                CanonicalJson::string(&self.schema_version),
            ),
        ])
    }

    fn imported_interfaces_json(&self) -> CanonicalJson {
        CanonicalJson::array(
            self.imported_interfaces
                .iter()
                .map(ImportedInterface::to_json_value)
                .collect(),
        )
    }

    fn compute_interface_hash(&self) -> Hash {
        canonical_hash("module-summary/interface/v1", &self.interface_hash_value())
    }

    fn sort_canonical(&mut self) {
        self.exports.sort_by_key(ExportedItem::sort_key);
        self.exported_labels.sort_by_key(ExportedLabel::sort_key);
        self.imported_interfaces
            .sort_by_key(ImportedInterface::sort_key);
    }

    fn validate_order_and_uniqueness(&self) -> Result<(), ArtifactError> {
        ensure_sorted_by(&self.exports, SCHEMA, "exports", ExportedItem::sort_key)?;
        ensure_sorted_by(
            &self.exported_labels,
            SCHEMA,
            "exported_labels",
            ExportedLabel::sort_key,
        )?;
        ensure_sorted_by(
            &self.imported_interfaces,
            SCHEMA,
            "imported_interfaces",
            ImportedInterface::sort_key,
        )?;
        ensure_unique_by(&self.exports, SCHEMA, ExportedItem::sort_key)?;
        ensure_unique_by(&self.exported_labels, SCHEMA, ExportedLabel::sort_key)?;
        ensure_unique_by(
            &self.imported_interfaces,
            SCHEMA,
            ImportedInterface::sort_key,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{ExportedItem, ExportedLabel, ImportedInterface, ModuleSummary};
    use crate::store::{ArtifactError, content_hash_str};

    #[test]
    fn module_summary_round_trips() {
        let summary = fixture("body one", Some("local timestamp".to_owned()));
        let json = summary.to_json_string();
        let parsed = ModuleSummary::from_json_str(&json).unwrap();

        assert_eq!(parsed, summary);
        assert!(json.contains("\"schema_version\":\"mizar-artifact/module-summary/v1\""));
    }

    #[test]
    fn interface_hash_ignores_source_hash_and_local_metadata() {
        let first = fixture("body one", Some("run a".to_owned()));
        let second = fixture("body two", Some("run b".to_owned()));

        assert_ne!(first.source_hash, second.source_hash);
        assert_eq!(first.interface_hash, second.interface_hash);
    }

    #[test]
    fn incompatible_versions_fail_cleanly() {
        let json = fixture("body one", None).to_json_string().replace(
            "mizar-artifact/module-summary/v1",
            "mizar-artifact/module-summary/v2",
        );

        assert!(matches!(
            ModuleSummary::from_json_str(&json),
            Err(ArtifactError::UnsupportedSchemaVersion { .. })
        ));
    }

    #[test]
    fn nondeterministic_export_order_is_rejected() {
        let mut summary = fixture("body one", None);
        summary.exports.reverse();
        let json = summary.to_json_string();

        assert!(matches!(
            ModuleSummary::from_json_str(&json),
            Err(ArtifactError::InvalidField {
                field: "exports",
                ..
            })
        ));
    }

    #[test]
    fn tampered_interface_hash_and_duplicate_exports_are_rejected() {
        let mut tampered = fixture("body one", None);
        tampered.interface_hash = content_hash_str("wrong interface");
        assert!(matches!(
            ModuleSummary::from_json_str(&tampered.to_json_string()),
            Err(ArtifactError::HashMismatch { .. })
        ));

        let mut duplicate = fixture("body one", None);
        duplicate.exports.push(duplicate.exports[0].clone());
        duplicate.exports.sort_by_key(ExportedItem::sort_key);
        duplicate.interface_hash = duplicate.compute_interface_hash();
        assert!(matches!(
            ModuleSummary::from_json_str(&duplicate.to_json_string()),
            Err(ArtifactError::DuplicateEntry { .. })
        ));
    }

    fn fixture(source: &str, local_metadata: Option<String>) -> ModuleSummary {
        ModuleSummary::new(
            "pkg",
            "alpha.core",
            content_hash_str(source),
            vec![
                ExportedItem::new(
                    "theorem",
                    "pkg::alpha.core::Th2",
                    "Th2",
                    "public",
                    "for x holds x = x",
                    None,
                ),
                ExportedItem::new(
                    "predicate",
                    "pkg::alpha.core::P/1",
                    "P",
                    "public",
                    "P(x: set)",
                    None,
                ),
            ],
            vec![ExportedLabel::new(
                "label:Th2",
                "Th2",
                "theorem",
                Some("pkg::alpha.core::Th2".to_owned()),
            )],
            vec![ImportedInterface::new(
                "mml.hidden",
                content_hash_str("hidden interface"),
            )],
            local_metadata,
        )
    }
}
