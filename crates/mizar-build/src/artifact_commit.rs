use std::{
    error::Error,
    fmt,
    path::{Path, PathBuf},
};

use mizar_artifact::{
    manifest::{
        ArtifactManifest, ManifestCommitOptions, ManifestError, ManifestFreshnessCheck,
        ManifestTransaction, ModuleArtifactEntry,
    },
    module_summary::ModuleSummaryIdentity,
};
use mizar_session::Hash;

use crate::task_graph::TaskId;

/// Build-side request to fold completed artifact-commit task outputs into a
/// package manifest transaction.
#[derive(Debug, Clone)]
pub struct ManifestCommitRequest {
    /// Package artifact root consumed by `mizar-artifact`.
    pub artifact_root: PathBuf,
    /// Seed manifest passed to `mizar-artifact` when no current manifest exists.
    pub seed_manifest: ArtifactManifest,
    /// Opaque freshness guard owned by the caller.
    pub freshness_guard: Option<String>,
    /// Caller-supplied module updates produced by completed commit tasks.
    pub updates: Vec<ScheduledManifestUpdate>,
}

/// One module manifest update with scheduler ordering metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduledManifestUpdate {
    /// The scheduler task that produced this manifest entry.
    pub task_id: TaskId,
    /// The task graph index used as a deterministic scheduler fallback.
    pub graph_index: usize,
    /// The artifact-owned manifest entry to stage.
    pub entry: ModuleArtifactEntry,
}

/// Successful build-side manifest commit result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestCommitSummary {
    /// Manifest returned by `mizar-artifact`.
    pub manifest: ArtifactManifest,
    /// Store-level hash for the committed manifest file.
    pub manifest_hash: Hash,
    /// Deterministic record of module updates offered to the transaction.
    pub modules: Vec<CommittedModuleUpdate>,
}

/// Deterministic build-side record for one staged module update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommittedModuleUpdate {
    /// The scheduler task that produced this manifest entry.
    pub task_id: TaskId,
    /// The task graph index used as a deterministic scheduler fallback.
    pub graph_index: usize,
    /// Stable module identity consumed from the artifact manifest entry.
    pub module: ModuleSummaryIdentity,
    /// Source file path consumed from the artifact manifest entry.
    pub source_file: String,
    /// Published artifact path consumed from the artifact manifest entry.
    pub artifact_file: String,
}

/// Build-side artifact-commit errors.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ArtifactCommitError {
    /// The artifact-owned manifest transaction rejected the request.
    Manifest(ManifestError),
}

/// Stages module updates in canonical scheduler order and commits them through
/// `mizar-artifact`'s manifest transaction manager.
pub fn commit_manifest_updates(
    request: ManifestCommitRequest,
    freshness_check: Option<&dyn ManifestFreshnessCheck>,
) -> Result<ManifestCommitSummary, ArtifactCommitError> {
    let ManifestCommitRequest {
        artifact_root,
        seed_manifest,
        freshness_guard,
        mut updates,
    } = request;
    sort_manifest_updates(&mut updates);
    let modules = updates
        .iter()
        .map(ScheduledManifestUpdate::committed_record)
        .collect::<Vec<_>>();

    let mut transaction =
        ManifestTransaction::begin(artifact_root.as_path(), seed_manifest, freshness_guard)?;
    for update in updates {
        transaction.stage_module(update.entry)?;
    }
    let commit = transaction.commit(ManifestCommitOptions { freshness_check })?;

    Ok(ManifestCommitSummary {
        manifest: commit.manifest,
        manifest_hash: commit.write.artifact_hash,
        modules,
    })
}

/// Returns the canonical build-side order for caller-supplied manifest updates.
pub fn sorted_manifest_updates(
    updates: impl IntoIterator<Item = ScheduledManifestUpdate>,
) -> Vec<ScheduledManifestUpdate> {
    let mut updates = updates.into_iter().collect::<Vec<_>>();
    sort_manifest_updates(&mut updates);
    updates
}

impl ManifestCommitRequest {
    /// Creates a manifest commit request with no freshness guard.
    pub fn new(
        artifact_root: impl AsRef<Path>,
        seed_manifest: ArtifactManifest,
        updates: Vec<ScheduledManifestUpdate>,
    ) -> Self {
        Self {
            artifact_root: artifact_root.as_ref().to_path_buf(),
            seed_manifest,
            freshness_guard: None,
            updates,
        }
    }

    /// Sets the opaque freshness guard forwarded to `mizar-artifact`.
    pub fn with_freshness_guard(mut self, guard: impl Into<String>) -> Self {
        self.freshness_guard = Some(guard.into());
        self
    }
}

impl ScheduledManifestUpdate {
    /// Creates one scheduled manifest update.
    pub fn new(task_id: TaskId, graph_index: usize, entry: ModuleArtifactEntry) -> Self {
        Self {
            task_id,
            graph_index,
            entry,
        }
    }

    /// Returns the canonical build-side sort key.
    pub fn sort_key(&self) -> (ModuleSummaryIdentity, String, usize, String) {
        (
            self.entry.module.clone(),
            self.entry.source_file.clone(),
            self.graph_index,
            self.task_id.as_str().to_owned(),
        )
    }

    fn committed_record(&self) -> CommittedModuleUpdate {
        CommittedModuleUpdate {
            task_id: self.task_id.clone(),
            graph_index: self.graph_index,
            module: self.entry.module.clone(),
            source_file: self.entry.source_file.clone(),
            artifact_file: self.entry.artifact_file.clone(),
        }
    }
}

impl From<ManifestError> for ArtifactCommitError {
    fn from(error: ManifestError) -> Self {
        Self::Manifest(error)
    }
}

impl fmt::Display for ArtifactCommitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Manifest(error) => write!(formatter, "{error}"),
        }
    }
}

impl Error for ArtifactCommitError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Manifest(error) => Some(error),
        }
    }
}

fn sort_manifest_updates(updates: &mut [ScheduledManifestUpdate]) {
    updates.sort_by_key(ScheduledManifestUpdate::sort_key);
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
    };

    use mizar_artifact::{
        manifest::{
            ARTIFACT_MANIFEST_PATH, ArtifactManifest, ManifestError, ManifestFileReadOptions,
            ManifestProvenance, ModuleArtifactEntry, PackageIdentity, read_manifest_file,
            write_manifest_file,
        },
        module_summary::ModuleSummaryIdentity,
        registration_summary::{ArtifactHashClass, ArtifactHashRef},
        store::{PublishedArtifactPath, artifact_hash_domain, write_published_artifact},
        verified_artifact::{
            BuildProvenance, VERIFIED_ARTIFACT_SCHEMA_FAMILY, VerifiedArtifact,
            artifact_hash_excluded_paths, current_schema_version as verified_schema_version,
            verified_artifact_json,
        },
    };
    use mizar_session::Hash;

    use super::*;
    use crate::task_graph::TaskId;

    static TEST_ROOT_COUNTER: AtomicU64 = AtomicU64::new(1);

    #[test]
    fn shuffled_updates_commit_identical_manifests_and_records() {
        let root_a = TestArtifactRoot::new();
        let root_b = TestArtifactRoot::new();
        let entry_a_root_a = publish_sample_artifact(root_a.path(), "articles/a", 1);
        let entry_b_root_a = publish_sample_artifact(root_a.path(), "articles/b", 2);
        let entry_a_root_b = publish_sample_artifact(root_b.path(), "articles/a", 1);
        let entry_b_root_b = publish_sample_artifact(root_b.path(), "articles/b", 2);
        assert_eq!(entry_a_root_a, entry_a_root_b);
        assert_eq!(entry_b_root_a, entry_b_root_b);

        let first = commit_manifest_updates(
            ManifestCommitRequest::new(
                root_a.path(),
                seed_manifest(),
                vec![
                    ScheduledManifestUpdate::new(
                        TaskId::new_for_test("commit:b"),
                        20,
                        entry_b_root_a,
                    ),
                    ScheduledManifestUpdate::new(
                        TaskId::new_for_test("commit:a"),
                        10,
                        entry_a_root_a,
                    ),
                ],
            ),
            None,
        )
        .expect("first commit succeeds");
        let second = commit_manifest_updates(
            ManifestCommitRequest::new(
                root_b.path(),
                seed_manifest(),
                vec![
                    ScheduledManifestUpdate::new(
                        TaskId::new_for_test("commit:a"),
                        10,
                        entry_a_root_b,
                    ),
                    ScheduledManifestUpdate::new(
                        TaskId::new_for_test("commit:b"),
                        20,
                        entry_b_root_b,
                    ),
                ],
            ),
            None,
        )
        .expect("second commit succeeds");

        assert_eq!(first.manifest, second.manifest);
        assert_eq!(first.manifest_hash, second.manifest_hash);
        assert_eq!(first.modules, second.modules);
        assert_eq!(
            first
                .modules
                .iter()
                .map(|module| module.module.module_path.as_str())
                .collect::<Vec<_>>(),
            vec!["articles/a", "articles/b"]
        );
        assert_eq!(
            fs::read_to_string(root_a.path().join(ARTIFACT_MANIFEST_PATH))
                .expect("first manifest text"),
            fs::read_to_string(root_b.path().join(ARTIFACT_MANIFEST_PATH))
                .expect("second manifest text")
        );
    }

    #[test]
    fn obsolete_freshness_rejection_leaves_previous_manifest_visible() {
        let root = TestArtifactRoot::new();
        let previous_entry = publish_sample_artifact(root.path(), "articles/old", 1);
        let mut previous = seed_manifest();
        previous.modules = vec![previous_entry.clone()];
        write_manifest_file(root.path(), &previous).expect("previous manifest");
        let next_entry = publish_sample_artifact(root.path(), "articles/new", 2);

        let freshness = |guard: Option<&str>| guard == Some("current-snapshot");
        let error = commit_manifest_updates(
            ManifestCommitRequest::new(
                root.path(),
                seed_manifest(),
                vec![ScheduledManifestUpdate::new(
                    TaskId::new_for_test("commit:new"),
                    30,
                    next_entry,
                )],
            )
            .with_freshness_guard("obsolete-snapshot"),
            Some(&freshness),
        )
        .expect_err("obsolete freshness must reject commit");

        assert!(matches!(
            error,
            ArtifactCommitError::Manifest(ManifestError::ObsoleteSnapshot {
                freshness_guard: Some(ref guard)
            }) if guard == "obsolete-snapshot"
        ));
        let after = read_manifest_file(root.path(), ManifestFileReadOptions::default())
            .expect("previous manifest still visible");
        assert_eq!(after.manifest.modules, vec![previous_entry]);
    }

    #[test]
    fn conflicting_updates_remain_artifact_manifest_errors() {
        let root = TestArtifactRoot::new();
        let first_entry = publish_sample_artifact(root.path(), "articles/a", 1);
        let mut conflicting_entry = publish_sample_artifact(root.path(), "articles/a", 2);
        conflicting_entry.artifact_file = "artifacts/articles/a-conflict.json".to_owned();

        let error = commit_manifest_updates(
            ManifestCommitRequest::new(
                root.path(),
                seed_manifest(),
                vec![
                    ScheduledManifestUpdate::new(
                        TaskId::new_for_test("commit:a:1"),
                        10,
                        first_entry,
                    ),
                    ScheduledManifestUpdate::new(
                        TaskId::new_for_test("commit:a:2"),
                        11,
                        conflicting_entry,
                    ),
                ],
            ),
            None,
        )
        .expect_err("conflicting module updates must fail");

        assert!(matches!(
            error,
            ArtifactCommitError::Manifest(ManifestError::ConflictingStagedModule { .. })
        ));
        assert!(
            read_manifest_file(root.path(), ManifestFileReadOptions::default()).is_err(),
            "failed transaction must not publish a manifest"
        );
    }

    #[test]
    fn artifact_commit_boundary_excludes_external_authority_placeholders() {
        let source = include_str!("artifact_commit.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source prefix exists");
        for forbidden in [
            concat!("mizar", "_", "driver"),
            concat!("mizar", "-", "driver"),
            concat!("mizar", "_", "ir"),
            concat!("mizar", "-", "ir"),
            concat!("mizar", "_", "cache"),
            concat!("mizar", "-", "cache"),
            concat!("Driver", "Session"),
            concat!("Driver", "Request"),
            concat!("Ir", "Snapshot", "Handle"),
            "CacheKey",
            concat!("cache", "_", "key"),
            concat!("Dependency", "Fingerprint"),
            concat!("dependency", "_", "fingerprint"),
            concat!("Proof", "Reuse"),
            concat!("proof", "_", "reuse"),
            concat!("Proof", "Authority"),
            concat!("Trusted", "Status"),
            concat!("Publication", "Token"),
            concat!("publication", "_", "token"),
            concat!("Verified", "Artifact"),
            concat!("Proof", "Witness"),
            concat!("proof", "_", "witness"),
            "ArtifactHashRef",
            "ArtifactHashClass",
            "SchemaVersion",
            "current_schema_version",
            "CanonicalHashDomain",
            "PublishedArtifactPath",
            "artifact_hash_domain",
            "write_published_artifact",
            "read_published_artifact",
            "write_manifest_file",
            "read_manifest_file",
            "read_verified_artifact",
            "validate_manifest_references",
        ] {
            assert!(
                !source.contains(forbidden),
                "artifact commit boundary must not contain `{forbidden}`"
            );
        }

        let manifest = include_str!("../Cargo.toml");
        assert!(
            manifest.contains("mizar-artifact"),
            "artifact commit boundary must consume mizar-artifact"
        );
        for forbidden_dependency in [concat!("mizar", "-", "driver"), concat!("mizar", "-", "ir")] {
            assert!(
                !manifest.contains(forbidden_dependency),
                "artifact commit boundary must not add `{forbidden_dependency}` dependency"
            );
        }
    }

    fn seed_manifest() -> ArtifactManifest {
        ArtifactManifest {
            schema_version: mizar_artifact::manifest::current_schema_version(),
            package: PackageIdentity {
                package_id: "pkg".to_owned(),
                package_version: Some("1.0.0".to_owned()),
                lockfile_identity: Some("lock".to_owned()),
            },
            artifact_root: "build".to_owned(),
            lockfile_hash: hash_ref(ArtifactHashClass::Artifact, "mizar-build/lockfile", 11),
            toolchain: "mizar-evo-test".to_owned(),
            language_edition: "2026".to_owned(),
            verifier_config_hash: hash_ref(
                ArtifactHashClass::Interface,
                "mizar-build/verifier-config",
                12,
            ),
            modules: Vec::new(),
            development_artifacts: Vec::new(),
            provenance: ManifestProvenance {
                generated_by: "mizar-build-test".to_owned(),
                manifest_policy: "test-policy".to_owned(),
                transaction_format: "mizar-build-artifact-commit-test-v1".to_owned(),
            },
        }
    }

    fn publish_sample_artifact(root: &Path, module_path: &str, seed: u8) -> ModuleArtifactEntry {
        let identity = identity(module_path);
        let artifact =
            sample_verified_artifact(identity, &format!("{module_path}.miz"), hash(seed), seed);
        publish_verified_artifact(
            root,
            &format!("artifacts/{module_path}.mizir.json"),
            &artifact,
        )
    }

    fn sample_verified_artifact(
        module: ModuleSummaryIdentity,
        source_file: &str,
        source_hash: Hash,
        seed: u8,
    ) -> VerifiedArtifact {
        let mut artifact = VerifiedArtifact {
            schema_version: verified_schema_version(),
            module,
            source_file: source_file.to_owned(),
            source_hash,
            verified_at: None,
            interface_hash: hash(0),
            implementation_hash: hash(0),
            exports: Vec::new(),
            expressions: Vec::new(),
            obligations: Vec::new(),
            proof_witnesses: Vec::new(),
            diagnostics: Vec::new(),
            provenance: BuildProvenance {
                toolchain: "mizar-evo-test".to_owned(),
                language_edition: "2026".to_owned(),
                lockfile_hash: hash_ref(
                    ArtifactHashClass::Artifact,
                    "mizar-build/lockfile",
                    seed.saturating_add(20),
                ),
                verifier_config_hash: hash_ref(
                    ArtifactHashClass::Interface,
                    "mizar-build/verifier-config",
                    seed.saturating_add(30),
                ),
                dependency_artifact_hashes: Vec::new(),
                cache_key: None,
            },
        };
        artifact.refresh_hashes().expect("sample hashes");
        artifact
    }

    fn publish_verified_artifact(
        root: &Path,
        path: &str,
        artifact: &VerifiedArtifact,
    ) -> ModuleArtifactEntry {
        let json = verified_artifact_json(artifact).expect("verified artifact JSON");
        let domain = artifact_hash_domain(VERIFIED_ARTIFACT_SCHEMA_FAMILY, artifact.schema_version);
        let excluded = artifact_hash_excluded_paths();
        let published_path = PublishedArtifactPath::new(path).expect("published path");
        let write = write_published_artifact(root, &published_path, &json, &domain, &excluded)
            .expect("write verified artifact");
        ModuleArtifactEntry {
            module: artifact.module.clone(),
            source_file: artifact.source_file.clone(),
            source_hash: artifact.source_hash,
            artifact_file: path.to_owned(),
            artifact_hash: ArtifactHashRef::new(
                ArtifactHashClass::Artifact,
                VERIFIED_ARTIFACT_SCHEMA_FAMILY,
                artifact.schema_version,
                write.artifact_hash,
            ),
            interface_hash: ArtifactHashRef::new(
                ArtifactHashClass::Interface,
                VERIFIED_ARTIFACT_SCHEMA_FAMILY,
                artifact.schema_version,
                artifact.interface_hash,
            ),
            implementation_hash: ArtifactHashRef::new(
                ArtifactHashClass::Implementation,
                VERIFIED_ARTIFACT_SCHEMA_FAMILY,
                artifact.schema_version,
                artifact.implementation_hash,
            ),
            module_summary_file: None,
            module_summary_hash: None,
            module_summary_interface_hash: None,
            registration_summary_file: None,
            registration_summary_hash: None,
            registration_interface_hash: None,
            proof_witnesses: Vec::new(),
            diagnostics_hash: None,
        }
    }

    fn identity(module_path: &str) -> ModuleSummaryIdentity {
        ModuleSummaryIdentity {
            package_id: "pkg".to_owned(),
            package_version: Some("1.0.0".to_owned()),
            lockfile_identity: Some("lock".to_owned()),
            module_path: module_path.to_owned(),
            language_edition: "2026".to_owned(),
        }
    }

    fn hash(seed: u8) -> Hash {
        Hash::from_bytes([seed; Hash::BYTE_LEN])
    }

    fn hash_ref(class: ArtifactHashClass, schema_family: &str, seed: u8) -> ArtifactHashRef {
        ArtifactHashRef::new(
            class,
            schema_family,
            mizar_artifact::store::SchemaVersion::new(1, 0),
            hash(seed),
        )
    }

    struct TestArtifactRoot {
        path: PathBuf,
    }

    impl TestArtifactRoot {
        fn new() -> Self {
            let path = Self::fresh_path();
            fs::create_dir_all(&path).expect("test artifact root");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }

        fn fresh_path() -> PathBuf {
            let counter = TEST_ROOT_COUNTER.fetch_add(1, Ordering::Relaxed);
            std::env::temp_dir().join(format!(
                "mizar-build-artifact-commit-test-{}-{counter}",
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
