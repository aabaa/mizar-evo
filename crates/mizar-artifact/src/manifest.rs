//! Artifact manifest schema and transaction helper.
//!
//! Design authority: [manifest.md](../../../../doc/design/mizar-artifact/en/manifest.md).

use crate::store::{
    ArtifactError, ArtifactStore, CanonicalJson, check_schema_version, ensure_sorted_by,
    ensure_unique_by, expect_object, hash_json, optional_hash, optional_hash_json, parse_json,
    reject_unknown_fields, required_array, required_hash, required_string,
    validate_portable_path_text,
};
use mizar_session::Hash;

/// Current artifact manifest schema version.
pub const MANIFEST_SCHEMA_VERSION: &str = "mizar-artifact/manifest/v1";

/// Standard package artifact manifest file name.
pub const MANIFEST_FILE_NAME: &str = "artifact-manifest.json";

const SCHEMA: &str = "ArtifactManifest";
const SUPPORTED: &[&str] = &[MANIFEST_SCHEMA_VERSION];

fn compare_hash(field: &'static str, expected: Hash, actual: Hash) -> Result<(), ArtifactError> {
    if expected == actual {
        Ok(())
    } else {
        Err(ArtifactError::InvalidField {
            schema: SCHEMA,
            field,
            reason: format!(
                "manifest hash {} differs from artifact hash {}",
                crate::store::hash_to_hex(expected),
                crate::store::hash_to_hex(actual)
            ),
        })
    }
}

/// Proof witness entry stored in the package manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofWitnessEntry {
    /// Stable witness id.
    pub witness_id: String,
    /// Portable witness path.
    pub path: String,
    /// Witness content hash.
    pub content_hash: Hash,
}

/// One module artifact entry in the package manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleArtifactEntry {
    /// Module path.
    pub module_path: String,
    /// Portable source file path.
    pub source_file: String,
    /// Source content hash.
    pub source_hash: Hash,
    /// Portable verified artifact file path.
    pub artifact_file: String,
    /// Verified artifact content hash.
    pub artifact_hash: Hash,
    /// Importer-visible interface hash.
    pub interface_hash: Hash,
    /// Stable implementation hash.
    pub implementation_hash: Hash,
    /// Proof witness entries referenced by this module.
    pub proof_witnesses: Vec<ProofWitnessEntry>,
    /// Optional projected diagnostics hash.
    pub diagnostics_hash: Option<Hash>,
}

/// Package-level artifact manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactManifest {
    /// Schema version.
    pub schema_version: String,
    /// Package id.
    pub package_id: String,
    /// Package version.
    pub package_version: String,
    /// Lockfile hash.
    pub lockfile_hash: Hash,
    /// Toolchain identity.
    pub toolchain: String,
    /// Verifier configuration hash.
    pub verifier_config_hash: Hash,
    /// Module artifact entries.
    pub modules: Vec<ModuleArtifactEntry>,
}

/// In-memory manifest update transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestTransaction {
    manifest: ArtifactManifest,
}

impl ProofWitnessEntry {
    /// Creates a manifest proof witness entry.
    pub fn new(witness_id: impl Into<String>, path: impl Into<String>, content_hash: Hash) -> Self {
        Self {
            witness_id: witness_id.into(),
            path: path.into(),
            content_hash,
        }
    }

    fn sort_key(&self) -> String {
        format!(
            "{}\u{1f}{}\u{1f}{}",
            self.witness_id,
            self.path,
            crate::store::hash_to_hex(self.content_hash)
        )
    }

    fn to_json_value(&self) -> CanonicalJson {
        CanonicalJson::object([
            ("content_hash", hash_json(self.content_hash)),
            ("path", CanonicalJson::string(&self.path)),
            ("witness_id", CanonicalJson::string(&self.witness_id)),
        ])
    }

    fn from_json_value(value: &CanonicalJson) -> Result<Self, ArtifactError> {
        let object = expect_object(value, SCHEMA)?;
        reject_unknown_fields(object, SCHEMA, &["content_hash", "path", "witness_id"])?;
        let path = required_string(object, SCHEMA, "path")?;
        validate_portable_path_text(&path)?;
        Ok(Self::new(
            required_string(object, SCHEMA, "witness_id")?,
            path,
            required_hash(object, SCHEMA, "content_hash")?,
        ))
    }
}

impl ModuleArtifactEntry {
    /// Creates a module artifact entry and sorts witness entries.
    #[allow(clippy::too_many_arguments)] // reason: constructor mirrors stable manifest entry fields
    pub fn new(
        module_path: impl Into<String>,
        source_file: impl Into<String>,
        source_hash: Hash,
        artifact_file: impl Into<String>,
        artifact_hash: Hash,
        interface_hash: Hash,
        implementation_hash: Hash,
        proof_witnesses: Vec<ProofWitnessEntry>,
        diagnostics_hash: Option<Hash>,
    ) -> Self {
        let mut entry = Self {
            module_path: module_path.into(),
            source_file: source_file.into(),
            source_hash,
            artifact_file: artifact_file.into(),
            artifact_hash,
            interface_hash,
            implementation_hash,
            proof_witnesses,
            diagnostics_hash,
        };
        entry
            .proof_witnesses
            .sort_by_key(ProofWitnessEntry::sort_key);
        entry
    }

    fn sort_key(&self) -> String {
        format!("{}\u{1f}{}", self.module_path, self.artifact_file)
    }

    fn to_json_value(&self) -> CanonicalJson {
        CanonicalJson::object([
            ("artifact_file", CanonicalJson::string(&self.artifact_file)),
            ("artifact_hash", hash_json(self.artifact_hash)),
            (
                "diagnostics_hash",
                optional_hash_json(self.diagnostics_hash),
            ),
            ("implementation_hash", hash_json(self.implementation_hash)),
            ("interface_hash", hash_json(self.interface_hash)),
            ("module_path", CanonicalJson::string(&self.module_path)),
            (
                "proof_witnesses",
                CanonicalJson::array(
                    self.proof_witnesses
                        .iter()
                        .map(ProofWitnessEntry::to_json_value)
                        .collect(),
                ),
            ),
            ("source_file", CanonicalJson::string(&self.source_file)),
            ("source_hash", hash_json(self.source_hash)),
        ])
    }

    fn validate_against_artifact(
        &self,
        artifact: &crate::verified_artifact::VerifiedArtifact,
    ) -> Result<(), ArtifactError> {
        if self.module_path != artifact.module_path {
            return Err(ArtifactError::InvalidField {
                schema: SCHEMA,
                field: "module_path",
                reason: "manifest module path differs from artifact".to_owned(),
            });
        }
        if self.source_file != artifact.source_file {
            return Err(ArtifactError::InvalidField {
                schema: SCHEMA,
                field: "source_file",
                reason: "manifest source file differs from artifact".to_owned(),
            });
        }
        compare_hash("source_hash", self.source_hash, artifact.source_hash)?;
        compare_hash(
            "interface_hash",
            self.interface_hash,
            artifact.interface_hash,
        )?;
        compare_hash(
            "implementation_hash",
            self.implementation_hash,
            artifact.implementation_hash,
        )?;
        if let Some(expected) = self.diagnostics_hash {
            compare_hash("diagnostics_hash", expected, artifact.diagnostics_hash())?;
        }

        let mut artifact_witnesses = artifact
            .proof_witnesses
            .iter()
            .map(|witness| {
                ProofWitnessEntry::new(
                    witness.witness_id.clone(),
                    witness.path.clone(),
                    witness.content_hash,
                )
            })
            .collect::<Vec<_>>();
        artifact_witnesses.sort_by_key(ProofWitnessEntry::sort_key);
        if self.proof_witnesses != artifact_witnesses {
            return Err(ArtifactError::InvalidField {
                schema: SCHEMA,
                field: "proof_witnesses",
                reason: "manifest witness entries differ from artifact witness refs".to_owned(),
            });
        }
        Ok(())
    }

    fn from_json_value(value: &CanonicalJson) -> Result<Self, ArtifactError> {
        let object = expect_object(value, SCHEMA)?;
        reject_unknown_fields(
            object,
            SCHEMA,
            &[
                "artifact_file",
                "artifact_hash",
                "diagnostics_hash",
                "implementation_hash",
                "interface_hash",
                "module_path",
                "proof_witnesses",
                "source_file",
                "source_hash",
            ],
        )?;
        let source_file = required_string(object, SCHEMA, "source_file")?;
        let artifact_file = required_string(object, SCHEMA, "artifact_file")?;
        validate_portable_path_text(&source_file)?;
        validate_portable_path_text(&artifact_file)?;
        let proof_witnesses = required_array(object, SCHEMA, "proof_witnesses")?
            .iter()
            .map(ProofWitnessEntry::from_json_value)
            .collect::<Result<Vec<_>, _>>()?;
        let entry = Self {
            module_path: required_string(object, SCHEMA, "module_path")?,
            source_file,
            source_hash: required_hash(object, SCHEMA, "source_hash")?,
            artifact_file,
            artifact_hash: required_hash(object, SCHEMA, "artifact_hash")?,
            interface_hash: required_hash(object, SCHEMA, "interface_hash")?,
            implementation_hash: required_hash(object, SCHEMA, "implementation_hash")?,
            proof_witnesses,
            diagnostics_hash: optional_hash(object, SCHEMA, "diagnostics_hash")?,
        };
        ensure_sorted_by(
            &entry.proof_witnesses,
            SCHEMA,
            "proof_witnesses",
            ProofWitnessEntry::sort_key,
        )?;
        Ok(entry)
    }
}

impl ArtifactManifest {
    /// Creates a package artifact manifest and sorts module entries.
    pub fn new(
        package_id: impl Into<String>,
        package_version: impl Into<String>,
        lockfile_hash: Hash,
        toolchain: impl Into<String>,
        verifier_config_hash: Hash,
        modules: Vec<ModuleArtifactEntry>,
    ) -> Self {
        let mut manifest = Self {
            schema_version: MANIFEST_SCHEMA_VERSION.to_owned(),
            package_id: package_id.into(),
            package_version: package_version.into(),
            lockfile_hash,
            toolchain: toolchain.into(),
            verifier_config_hash,
            modules,
        };
        manifest.sort_canonical();
        manifest
    }

    /// Serializes this manifest as canonical JSON.
    pub fn to_json_string(&self) -> String {
        self.to_json_value().to_canonical_string()
    }

    /// Reads and validates a manifest from JSON text.
    pub fn from_json_str(input: &str) -> Result<Self, ArtifactError> {
        let value = parse_json(input)?;
        let object = expect_object(&value, SCHEMA)?;
        reject_unknown_fields(
            object,
            SCHEMA,
            &[
                "schema_version",
                "package_id",
                "package_version",
                "lockfile_hash",
                "toolchain",
                "verifier_config_hash",
                "modules",
            ],
        )?;
        let schema_version = check_schema_version(object, SCHEMA, SUPPORTED)?;
        let mut manifest = Self {
            schema_version,
            package_id: required_string(object, SCHEMA, "package_id")?,
            package_version: required_string(object, SCHEMA, "package_version")?,
            lockfile_hash: required_hash(object, SCHEMA, "lockfile_hash")?,
            toolchain: required_string(object, SCHEMA, "toolchain")?,
            verifier_config_hash: required_hash(object, SCHEMA, "verifier_config_hash")?,
            modules: required_array(object, SCHEMA, "modules")?
                .iter()
                .map(ModuleArtifactEntry::from_json_value)
                .collect::<Result<Vec<_>, _>>()?,
        };
        manifest.validate_order_and_uniqueness()?;
        manifest.sort_canonical();
        Ok(manifest)
    }

    /// Reads the standard manifest file from a store.
    pub fn read_from_store(store: &ArtifactStore) -> Result<Self, ArtifactError> {
        let content = store.read_named_str(MANIFEST_FILE_NAME, None)?;
        Self::from_json_str(&content)
    }

    /// Validates all referenced artifact and witness files against recorded hashes.
    pub fn validate_references(&self, store: &ArtifactStore) -> Result<(), ArtifactError> {
        for module in &self.modules {
            validate_portable_path_text(&module.source_file)?;
            validate_portable_path_text(&module.artifact_file)?;
            let artifact =
                store.read_named_str(&module.artifact_file, Some(module.artifact_hash))?;
            let artifact = crate::verified_artifact::VerifiedArtifact::from_json_str(&artifact)?;
            module.validate_against_artifact(&artifact)?;
            for witness in &module.proof_witnesses {
                validate_portable_path_text(&witness.path)?;
                store.read_named_str(&witness.path, Some(witness.content_hash))?;
            }
        }
        Ok(())
    }

    fn to_json_value(&self) -> CanonicalJson {
        CanonicalJson::object([
            ("lockfile_hash", hash_json(self.lockfile_hash)),
            (
                "modules",
                CanonicalJson::array(
                    self.modules
                        .iter()
                        .map(ModuleArtifactEntry::to_json_value)
                        .collect(),
                ),
            ),
            ("package_id", CanonicalJson::string(&self.package_id)),
            (
                "package_version",
                CanonicalJson::string(&self.package_version),
            ),
            (
                "schema_version",
                CanonicalJson::string(&self.schema_version),
            ),
            ("toolchain", CanonicalJson::string(&self.toolchain)),
            ("verifier_config_hash", hash_json(self.verifier_config_hash)),
        ])
    }

    fn sort_canonical(&mut self) {
        self.modules.sort_by_key(ModuleArtifactEntry::sort_key);
        for module in &mut self.modules {
            module
                .proof_witnesses
                .sort_by_key(ProofWitnessEntry::sort_key);
        }
    }

    fn validate_order_and_uniqueness(&self) -> Result<(), ArtifactError> {
        ensure_sorted_by(
            &self.modules,
            SCHEMA,
            "modules",
            ModuleArtifactEntry::sort_key,
        )?;
        ensure_unique_by(&self.modules, SCHEMA, |entry| entry.module_path.clone())?;
        for module in &self.modules {
            ensure_sorted_by(
                &module.proof_witnesses,
                SCHEMA,
                "proof_witnesses",
                ProofWitnessEntry::sort_key,
            )?;
            ensure_unique_by(&module.proof_witnesses, SCHEMA, ProofWitnessEntry::sort_key)?;
        }
        Ok(())
    }
}

impl ManifestTransaction {
    /// Begins a manifest transaction from an optional base manifest.
    pub fn begin(
        package_id: impl Into<String>,
        package_version: impl Into<String>,
        lockfile_hash: Hash,
        toolchain: impl Into<String>,
        verifier_config_hash: Hash,
        base: Option<ArtifactManifest>,
    ) -> Self {
        let mut manifest = base.unwrap_or_else(|| {
            ArtifactManifest::new(
                package_id,
                package_version,
                lockfile_hash,
                toolchain,
                verifier_config_hash,
                Vec::new(),
            )
        });
        manifest.sort_canonical();
        Self { manifest }
    }

    /// Stages or replaces a module entry.
    pub fn stage_module(&mut self, update: ModuleArtifactEntry) {
        self.manifest
            .modules
            .retain(|entry| entry.module_path != update.module_path);
        self.manifest.modules.push(update);
        self.manifest.sort_canonical();
    }

    /// Commits the manifest transaction through the store.
    pub fn commit(self, store: &ArtifactStore) -> Result<ArtifactManifest, ArtifactError> {
        self.manifest.validate_references(store)?;
        store.write_named_str(MANIFEST_FILE_NAME, &self.manifest.to_json_string())?;
        Ok(self.manifest)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ArtifactManifest, MANIFEST_FILE_NAME, MANIFEST_SCHEMA_VERSION, ManifestTransaction,
        ModuleArtifactEntry, ProofWitnessEntry,
    };
    use crate::module_summary::ExportedItem;
    use crate::proof_witness::ProofWitnessRef;
    use crate::store::{ArtifactError, ArtifactStore, content_hash_str};
    use crate::verified_artifact::{
        BuildProvenance, DependencyArtifactHash, ExpressionMetadata, MetadataField,
        ObligationMetadata, ProjectedDiagnostic, VerifiedArtifact,
    };
    use std::fs;

    #[test]
    fn manifest_round_trips_and_sorts_entries() {
        let manifest = manifest_fixture(vec![entry("beta"), entry("alpha")]);
        let parsed = ArtifactManifest::from_json_str(&manifest.to_json_string()).unwrap();

        assert_eq!(parsed.modules[0].module_path, "alpha");
        assert_eq!(
            parsed,
            manifest_fixture(vec![entry("alpha"), entry("beta")])
        );
    }

    #[test]
    fn transaction_validates_references_and_is_idempotent() {
        let root = temp_root("transaction");
        let store = ArtifactStore::new(&root);
        let artifact = valid_artifact();
        let artifact_hash = store
            .write_named_str("alpha.mizir.json", &artifact.to_json_string())
            .unwrap();
        let witness_hash = store
            .write_named_str("proof-witnesses/alpha/vc-1.json", "{\"proof\":true}")
            .unwrap();
        let entry = entry_for_artifact(&artifact, "alpha.mizir.json", artifact_hash, witness_hash);

        let mut tx = ManifestTransaction::begin(
            "pkg",
            "0.1.0",
            content_hash_str("lock"),
            "toolchain",
            content_hash_str("config"),
            None,
        );
        tx.stage_module(entry.clone());
        let committed = tx.commit(&store).unwrap();
        let replay = ManifestTransaction::begin(
            "pkg",
            "0.1.0",
            content_hash_str("lock"),
            "toolchain",
            content_hash_str("config"),
            Some(committed.clone()),
        );
        assert_eq!(replay.commit(&store).unwrap(), committed);
        assert_eq!(
            ArtifactManifest::read_from_store(&store).unwrap(),
            committed
        );

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn failed_transaction_leaves_previous_manifest_authoritative() {
        let root = temp_root("failed_transaction");
        let store = ArtifactStore::new(&root);
        let old = manifest_fixture(Vec::new());
        store
            .write_named_str(MANIFEST_FILE_NAME, &old.to_json_string())
            .unwrap();

        let missing = entry_with_paths(
            "alpha",
            "missing.mizir.json",
            content_hash_str("missing"),
            Vec::new(),
        );
        let mut missing_tx = ManifestTransaction::begin(
            "pkg",
            "0.1.0",
            content_hash_str("lock"),
            "toolchain",
            content_hash_str("config"),
            Some(old.clone()),
        );
        missing_tx.stage_module(missing);
        assert!(matches!(
            missing_tx.commit(&store),
            Err(ArtifactError::Io { .. })
        ));
        assert_eq!(ArtifactManifest::read_from_store(&store).unwrap(), old);

        let artifact = valid_artifact();
        let artifact_hash = store
            .write_named_str("alpha.mizir.json", &artifact.to_json_string())
            .unwrap();
        let witness_hash = store
            .write_named_str("proof-witnesses/alpha/vc-1.json", "{\"proof\":false}")
            .unwrap();
        let mismatch = entry_for_artifact(
            &artifact,
            "alpha.mizir.json",
            artifact_hash,
            content_hash_str("{\"proof\":true}"),
        );
        assert_ne!(witness_hash, content_hash_str("{\"proof\":true}"));
        let mut mismatch_tx = ManifestTransaction::begin(
            "pkg",
            "0.1.0",
            content_hash_str("lock"),
            "toolchain",
            content_hash_str("config"),
            Some(old.clone()),
        );
        mismatch_tx.stage_module(mismatch);
        assert!(matches!(
            mismatch_tx.commit(&store),
            Err(ArtifactError::HashMismatch { .. })
        ));
        assert_eq!(ArtifactManifest::read_from_store(&store).unwrap(), old);

        let mut omitted_witness = entry_for_artifact(
            &artifact,
            "alpha.mizir.json",
            artifact_hash,
            content_hash_str("{\"proof\":true}"),
        );
        omitted_witness.proof_witnesses.clear();
        let mut omitted_tx = ManifestTransaction::begin(
            "pkg",
            "0.1.0",
            content_hash_str("lock"),
            "toolchain",
            content_hash_str("config"),
            Some(old.clone()),
        );
        omitted_tx.stage_module(omitted_witness);
        assert!(matches!(
            omitted_tx.commit(&store),
            Err(ArtifactError::InvalidField {
                field: "proof_witnesses",
                ..
            })
        ));
        assert_eq!(ArtifactManifest::read_from_store(&store).unwrap(), old);

        let mut hash_mismatch =
            entry_for_artifact(&artifact, "alpha.mizir.json", artifact_hash, witness_hash);
        hash_mismatch.interface_hash = content_hash_str("wrong interface");
        let mut hash_mismatch_tx = ManifestTransaction::begin(
            "pkg",
            "0.1.0",
            content_hash_str("lock"),
            "toolchain",
            content_hash_str("config"),
            Some(old.clone()),
        );
        hash_mismatch_tx.stage_module(hash_mismatch);
        assert!(matches!(
            hash_mismatch_tx.commit(&store),
            Err(ArtifactError::InvalidField {
                field: "interface_hash",
                ..
            })
        ));
        assert_eq!(ArtifactManifest::read_from_store(&store).unwrap(), old);

        let malformed_hash = store
            .write_named_str("bad.mizir.json", "{\"schema_version\":")
            .unwrap();
        let malformed = entry_with_paths("bad", "bad.mizir.json", malformed_hash, Vec::new());
        let mut malformed_tx = ManifestTransaction::begin(
            "pkg",
            "0.1.0",
            content_hash_str("lock"),
            "toolchain",
            content_hash_str("config"),
            Some(old.clone()),
        );
        malformed_tx.stage_module(malformed);
        assert!(matches!(
            malformed_tx.commit(&store),
            Err(ArtifactError::MalformedJson { .. })
        ));
        assert_eq!(ArtifactManifest::read_from_store(&store).unwrap(), old);

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn manifest_read_ignores_temp_files_and_rejects_corrupt_entries() {
        let root = temp_root("old_new");
        let store = ArtifactStore::new(&root);
        store
            .write_named_str(
                MANIFEST_FILE_NAME,
                &manifest_fixture(vec![entry("alpha")]).to_json_string(),
            )
            .unwrap();
        fs::write(
            root.join(".artifact-manifest.json.999.tmp"),
            "{\"bad\":true}",
        )
        .unwrap();

        assert_eq!(
            ArtifactManifest::read_from_store(&store)
                .unwrap()
                .modules
                .len(),
            1
        );
        let corrupt = manifest_fixture(vec![entry("alpha")])
            .to_json_string()
            .replace("mizar-artifact/manifest/v1", "mizar-artifact/manifest/v2");
        store.write_named_str(MANIFEST_FILE_NAME, &corrupt).unwrap();
        assert!(matches!(
            ArtifactManifest::read_from_store(&store),
            Err(ArtifactError::UnsupportedSchemaVersion { .. })
        ));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn manifest_reader_rejects_nonportable_paths() {
        let json = manifest_fixture(vec![entry("alpha")])
            .to_json_string()
            .replace("src/alpha.miz", "../alpha.miz");

        assert!(matches!(
            ArtifactManifest::from_json_str(&json),
            Err(ArtifactError::InvalidPath { .. })
        ));
    }

    #[test]
    fn manifest_reader_rejects_duplicate_modules_and_unsorted_witnesses() {
        let duplicate = manifest_fixture(vec![
            entry_with_paths(
                "alpha",
                "alpha-a.mizir.json",
                content_hash_str("alpha artifact a"),
                Vec::new(),
            ),
            entry_with_paths(
                "alpha",
                "alpha-b.mizir.json",
                content_hash_str("alpha artifact b"),
                Vec::new(),
            ),
        ]);
        assert!(matches!(
            ArtifactManifest::from_json_str(&duplicate.to_json_string()),
            Err(ArtifactError::DuplicateEntry { .. })
        ));

        let mut unsorted_witnesses = entry("alpha");
        unsorted_witnesses.proof_witnesses = vec![
            ProofWitnessEntry::new(
                "wit-2",
                "proof-witnesses/alpha/vc-2.json",
                content_hash_str("2"),
            ),
            ProofWitnessEntry::new(
                "wit-1",
                "proof-witnesses/alpha/vc-1.json",
                content_hash_str("1"),
            ),
        ];
        let manifest = ArtifactManifest {
            schema_version: MANIFEST_SCHEMA_VERSION.to_owned(),
            package_id: "pkg".to_owned(),
            package_version: "0.1.0".to_owned(),
            lockfile_hash: content_hash_str("lock"),
            toolchain: "toolchain".to_owned(),
            verifier_config_hash: content_hash_str("config"),
            modules: vec![unsorted_witnesses],
        };
        assert!(matches!(
            ArtifactManifest::from_json_str(&manifest.to_json_string()),
            Err(ArtifactError::InvalidField {
                field: "proof_witnesses",
                ..
            })
        ));
    }

    fn manifest_fixture(modules: Vec<ModuleArtifactEntry>) -> ArtifactManifest {
        ArtifactManifest::new(
            "pkg",
            "0.1.0",
            content_hash_str("lock"),
            "toolchain",
            content_hash_str("config"),
            modules,
        )
    }

    fn entry(module: &str) -> ModuleArtifactEntry {
        entry_with_paths(
            module,
            format!("{module}.mizir.json"),
            content_hash_str(&format!("{module} artifact")),
            Vec::new(),
        )
    }

    fn entry_with_paths(
        module: &str,
        artifact_file: impl Into<String>,
        artifact_hash: mizar_session::Hash,
        proof_witnesses: Vec<ProofWitnessEntry>,
    ) -> ModuleArtifactEntry {
        ModuleArtifactEntry::new(
            module,
            format!("src/{module}.miz"),
            content_hash_str(&format!("{module} source")),
            artifact_file,
            artifact_hash,
            content_hash_str(&format!("{module} interface")),
            content_hash_str(&format!("{module} implementation")),
            proof_witnesses,
            None,
        )
    }

    fn entry_for_artifact(
        artifact: &VerifiedArtifact,
        artifact_file: impl Into<String>,
        artifact_hash: mizar_session::Hash,
        witness_hash: mizar_session::Hash,
    ) -> ModuleArtifactEntry {
        ModuleArtifactEntry::new(
            artifact.module_path.clone(),
            artifact.source_file.clone(),
            artifact.source_hash,
            artifact_file,
            artifact_hash,
            artifact.interface_hash,
            artifact.implementation_hash,
            vec![ProofWitnessEntry::new(
                "wit-1",
                "proof-witnesses/alpha/vc-1.json",
                witness_hash,
            )],
            Some(artifact.diagnostics_hash()),
        )
    }

    fn valid_artifact() -> VerifiedArtifact {
        VerifiedArtifact::new(
            "src/alpha.miz",
            content_hash_str("source"),
            None,
            "alpha",
            vec![ExportedItem::new(
                "theorem",
                "pkg::alpha::Th1",
                "Th1",
                "public",
                "for x holds x = x",
                None,
            )],
            vec![ExpressionMetadata::new(
                "expr-1",
                1,
                "term",
                "set",
                vec![MetadataField::new("hover", "x: set").unwrap()],
            )],
            vec![ObligationMetadata::new(
                "vc-1",
                1,
                "theorem",
                "kernel_verified",
                Some("wit-1".to_owned()),
            )],
            vec![ProofWitnessRef::from_content(
                "wit-1",
                "vc-1",
                "proof-witnesses/alpha/vc-1.json",
                "{\"proof\":true}",
                "atp_certificate",
                "kernel_verified",
                "kernel-profile-v1",
                "mizar-proof",
                vec!["A".to_owned()],
            )],
            vec![ProjectedDiagnostic::new(
                1, "W0001", "warning", "message", None,
            )],
            BuildProvenance::new(
                "toolchain",
                "2024",
                content_hash_str("lock"),
                content_hash_str("config"),
                vec![DependencyArtifactHash::new("dep", content_hash_str("dep"))],
                content_hash_str("cache"),
                None,
            ),
        )
    }

    fn temp_root(name: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!(
            "mizar_artifact_manifest_{name}_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        root
    }
}
