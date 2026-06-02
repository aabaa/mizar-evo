//! Snapshot-facing source version records.
//!
//! ```compile_fail
//! use mizar_session::SourceVersion;
//!
//! fn requires_source_version_id_order(_versions: &mut [SourceVersion]) {
//!     versions.sort_by_key(|version| version.source_id);
//! }
//! ```
//!
//! ```compile_fail
//! use mizar_session::BuildSnapshotId;
//!
//! fn requires_semantic_order<T: Ord>() {}
//! requires_semantic_order::<BuildSnapshotId>();
//! ```

use crate::{
    BuildSnapshotId, Hash, LspDocumentVersion, NormalizedPath, SnapshotLeaseId, SourceId,
    SourcePathError, ids::build_snapshot_id_from_sorted_canonical_bytes,
};
use std::cmp::Ordering;
use std::error::Error;
use std::fmt;

const SNAPSHOT_CANONICAL_SCHEMA_ID: &[u8] = b"mizar-session/snapshot-canonical-input/v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildSnapshot {
    pub id: BuildSnapshotId,
    pub workspace_root: WorkspaceRoot,
    pub source_versions: Vec<SourceVersion>,
    pub dependency_artifacts: Vec<DependencyArtifactRef>,
    pub lockfile_hash: Hash,
    pub toolchain: ToolchainInfo,
    pub verifier_config_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotInput {
    pub workspace_root: WorkspaceRoot,
    pub source_versions: Vec<SourceVersion>,
    pub dependency_artifacts: Vec<DependencyArtifactRef>,
    pub lockfile_hash: Hash,
    pub toolchain: ToolchainInfo,
    pub verifier_config_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorkspaceRoot(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageId(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModulePath(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Edition(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GeneratedSourceKind(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyArtifactRef {
    pub artifact: String,
    pub content_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ToolchainInfo(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceVersion {
    pub source_id: SourceId,
    pub package_id: PackageId,
    pub module_path: ModulePath,
    pub normalized_path: NormalizedPath,
    pub source_hash: Hash,
    pub edition: Edition,
    pub origin: SourceOrigin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceOrigin {
    Disk,
    OpenBuffer { version: LspDocumentVersion },
    Generated { generator: GeneratedSourceKind },
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SnapshotError {
    InvalidSourcePath {
        error: SourcePathError,
    },
    DuplicateModulePath {
        package_id: PackageId,
        module_path: ModulePath,
    },
    MissingDependencyArtifact {
        artifact: String,
    },
    UnsupportedLockfileMetadata {
        metadata: String,
    },
    UnsupportedToolchainMetadata {
        metadata: String,
    },
    StaleOpenBufferVersion {
        expected: LspDocumentVersion,
        actual: LspDocumentVersion,
    },
    UnknownSnapshotId {
        snapshot_id: BuildSnapshotId,
    },
    LeaseReleaseMismatch {
        lease_id: SnapshotLeaseId,
        expected_snapshot: BuildSnapshotId,
        actual_snapshot: BuildSnapshotId,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceVersionCanonicalKey<'a> {
    package_id: &'a str,
    module_path: &'a str,
    normalized_path: &'a str,
    source_hash: &'a [u8; Hash::BYTE_LEN],
}

impl BuildSnapshot {
    pub fn from_input(input: SnapshotInput) -> Self {
        let id = build_snapshot_id(&input);
        let mut source_versions = input.source_versions;
        let mut dependency_artifacts = input.dependency_artifacts;

        sort_source_versions_for_snapshot_identity(&mut source_versions);
        sort_dependency_artifacts_canonical(&mut dependency_artifacts);

        Self {
            id,
            workspace_root: input.workspace_root,
            source_versions,
            dependency_artifacts,
            lockfile_hash: input.lockfile_hash,
            toolchain: input.toolchain,
            verifier_config_hash: input.verifier_config_hash,
        }
    }
}

impl SnapshotInput {
    pub fn build_snapshot(self) -> BuildSnapshot {
        BuildSnapshot::from_input(self)
    }

    pub fn build_snapshot_id(&self) -> BuildSnapshotId {
        build_snapshot_id(self)
    }
}

impl WorkspaceRoot {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl PackageId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ModulePath {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Edition {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl GeneratedSourceKind {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl DependencyArtifactRef {
    pub fn new(artifact: impl Into<String>, content_hash: Hash) -> Self {
        Self {
            artifact: artifact.into(),
            content_hash,
        }
    }
}

impl ToolchainInfo {
    pub fn new(identity: impl Into<String>) -> Self {
        Self(identity.into())
    }

    pub fn identity(&self) -> &str {
        &self.0
    }
}

impl SourceVersion {
    pub fn canonical_sort_key(&self) -> SourceVersionCanonicalKey<'_> {
        SourceVersionCanonicalKey {
            package_id: self.package_id.as_str(),
            module_path: self.module_path.as_str(),
            normalized_path: self.normalized_path.as_str(),
            source_hash: self.source_hash.as_bytes(),
        }
    }
}

pub fn sort_source_versions_canonical(source_versions: &mut [SourceVersion]) {
    source_versions
        .sort_by(|left, right| left.canonical_sort_key().cmp(&right.canonical_sort_key()));
}

fn build_snapshot_id(input: &SnapshotInput) -> BuildSnapshotId {
    let canonical_bytes = encode_snapshot_input_canonical(input);
    build_snapshot_id_from_sorted_canonical_bytes(
        SNAPSHOT_CANONICAL_SCHEMA_ID,
        input.toolchain.identity().as_bytes(),
        &canonical_bytes,
    )
}

fn encode_snapshot_input_canonical(input: &SnapshotInput) -> Vec<u8> {
    let mut bytes = Vec::new();
    write_part(
        &mut bytes,
        b"workspace-root",
        input.workspace_root.as_str().as_bytes(),
    );
    write_hash_part(&mut bytes, b"lockfile-hash", input.lockfile_hash);
    write_part(
        &mut bytes,
        b"toolchain-identity",
        input.toolchain.identity().as_bytes(),
    );
    write_hash_part(
        &mut bytes,
        b"verifier-config-hash",
        input.verifier_config_hash,
    );
    write_source_version_summaries(&mut bytes, &input.source_versions);
    write_dependency_summaries(&mut bytes, &input.dependency_artifacts);
    bytes
}

fn write_source_version_summaries(bytes: &mut Vec<u8>, source_versions: &[SourceVersion]) {
    let mut summaries = source_versions.iter().collect::<Vec<_>>();
    summaries.sort_by(|left, right| compare_source_version_identity(left, right));

    write_collection_header(bytes, b"source-version-summaries", summaries.len());
    for version in summaries {
        write_part(
            bytes,
            b"source/package-id",
            version.package_id.as_str().as_bytes(),
        );
        write_part(
            bytes,
            b"source/module-path",
            version.module_path.as_str().as_bytes(),
        );
        write_part(
            bytes,
            b"source/normalized-path",
            version.normalized_path.as_str().as_bytes(),
        );
        write_hash_part(bytes, b"source/source-hash", version.source_hash);
        write_part(
            bytes,
            b"source/edition",
            version.edition.as_str().as_bytes(),
        );
    }
}

fn write_dependency_summaries(bytes: &mut Vec<u8>, dependency_artifacts: &[DependencyArtifactRef]) {
    let mut summaries = dependency_artifacts.iter().collect::<Vec<_>>();
    summaries.sort_by(|left, right| compare_dependency_artifact_identity(left, right));

    write_collection_header(bytes, b"dependency-artifact-summaries", summaries.len());
    for artifact in summaries {
        write_part(bytes, b"dependency/artifact", artifact.artifact.as_bytes());
        write_hash_part(bytes, b"dependency/content-hash", artifact.content_hash);
    }
}

fn sort_source_versions_for_snapshot_identity(source_versions: &mut [SourceVersion]) {
    source_versions.sort_by(compare_source_version_identity);
}

fn sort_dependency_artifacts_canonical(dependency_artifacts: &mut [DependencyArtifactRef]) {
    dependency_artifacts.sort_by(compare_dependency_artifact_identity);
}

fn compare_source_version_identity(left: &SourceVersion, right: &SourceVersion) -> Ordering {
    left.package_id
        .as_str()
        .cmp(right.package_id.as_str())
        .then_with(|| left.module_path.as_str().cmp(right.module_path.as_str()))
        .then_with(|| {
            left.normalized_path
                .as_str()
                .cmp(right.normalized_path.as_str())
        })
        .then_with(|| {
            left.source_hash
                .as_bytes()
                .cmp(right.source_hash.as_bytes())
        })
        .then_with(|| left.edition.as_str().cmp(right.edition.as_str()))
}

fn compare_dependency_artifact_identity(
    left: &DependencyArtifactRef,
    right: &DependencyArtifactRef,
) -> Ordering {
    left.artifact.cmp(&right.artifact).then_with(|| {
        left.content_hash
            .as_bytes()
            .cmp(right.content_hash.as_bytes())
    })
}

fn write_collection_header(bytes: &mut Vec<u8>, label: &[u8], len: usize) {
    write_part(bytes, label, &(len as u64).to_le_bytes());
}

fn write_hash_part(bytes: &mut Vec<u8>, label: &[u8], hash: Hash) {
    write_part(bytes, label, hash.as_bytes());
}

fn write_part(bytes: &mut Vec<u8>, label: &[u8], value: &[u8]) {
    bytes.extend_from_slice(&(label.len() as u64).to_le_bytes());
    bytes.extend_from_slice(label);
    bytes.extend_from_slice(&(value.len() as u64).to_le_bytes());
    bytes.extend_from_slice(value);
}

impl Ord for SourceVersionCanonicalKey<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.package_id
            .cmp(other.package_id)
            .then_with(|| self.module_path.cmp(other.module_path))
            .then_with(|| self.normalized_path.cmp(other.normalized_path))
            .then_with(|| self.source_hash.cmp(other.source_hash))
    }
}

impl PartialOrd for SourceVersionCanonicalKey<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for PackageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Display for WorkspaceRoot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Display for ModulePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Display for Edition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Display for GeneratedSourceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Display for ToolchainInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSourcePath { error } => {
                write!(f, "invalid or non-normalizable source path: {error}")
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
            Self::MissingDependencyArtifact { artifact } => {
                write!(f, "missing dependency artifact `{artifact}`")
            }
            Self::UnsupportedLockfileMetadata { metadata } => {
                write!(f, "unsupported lockfile metadata `{metadata}`")
            }
            Self::UnsupportedToolchainMetadata { metadata } => {
                write!(f, "unsupported toolchain metadata `{metadata}`")
            }
            Self::StaleOpenBufferVersion { expected, actual } => {
                write!(
                    f,
                    "stale open-buffer version `{actual}`, expected `{expected}`"
                )
            }
            Self::UnknownSnapshotId { snapshot_id } => {
                write!(f, "unknown snapshot id `{snapshot_id:?}`")
            }
            Self::LeaseReleaseMismatch {
                lease_id,
                expected_snapshot,
                actual_snapshot,
            } => {
                write!(
                    f,
                    "lease `{lease_id:?}` belongs to `{actual_snapshot:?}`, not `{expected_snapshot:?}`"
                )
            }
        }
    }
}

impl Error for SnapshotError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidSourcePath { error } => Some(error),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BuildSnapshot, DependencyArtifactRef, Edition, GeneratedSourceKind, ModulePath, PackageId,
        SnapshotError, SnapshotInput, SourceOrigin, SourceVersion, ToolchainInfo, WorkspaceRoot,
        sort_source_versions_canonical,
    };
    use crate::{
        BuildSnapshotId, Hash, InMemorySessionIdAllocator, NormalizedPath, SessionIdAllocator,
        SourcePathError, normalize_source_path,
    };
    use std::path::Path;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_FIXTURE_ID: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn identical_canonical_snapshot_inputs_produce_identical_ids() {
        let input = snapshot_input();
        let same = snapshot_input();

        assert_eq!(input.build_snapshot_id(), same.build_snapshot_id());
        assert_eq!(
            BuildSnapshot::from_input(input).id,
            same.build_snapshot_id()
        );
    }

    #[test]
    fn source_summary_changes_change_snapshot_id() {
        let input = snapshot_input();
        let mut changed_package_id = snapshot_input();
        changed_package_id.source_versions[0].package_id = PackageId::new("archive");
        let mut changed_module_path = snapshot_input();
        changed_module_path.source_versions[0].module_path = ModulePath::new("different.module");
        let mut changed_normalized_path = snapshot_input();
        changed_normalized_path.source_versions[0].normalized_path =
            normalized_path("src/different.miz");
        let mut changed_source_hash = snapshot_input();
        changed_source_hash.source_versions[0].source_hash = hash(44);
        let mut changed_edition = snapshot_input();
        changed_edition.source_versions[0].edition = Edition::new("2027");

        assert_ne!(
            input.build_snapshot_id(),
            changed_package_id.build_snapshot_id()
        );
        assert_ne!(
            input.build_snapshot_id(),
            changed_module_path.build_snapshot_id()
        );
        assert_ne!(
            input.build_snapshot_id(),
            changed_normalized_path.build_snapshot_id()
        );
        assert_ne!(
            input.build_snapshot_id(),
            changed_source_hash.build_snapshot_id()
        );
        assert_ne!(
            input.build_snapshot_id(),
            changed_edition.build_snapshot_id()
        );
    }

    #[test]
    fn dependency_artifact_changes_change_snapshot_id() {
        let input = snapshot_input();
        let mut changed_dependency_hash = snapshot_input();
        changed_dependency_hash.dependency_artifacts[0].content_hash = hash(55);
        let mut changed_dependency_identity = snapshot_input();
        changed_dependency_identity.dependency_artifacts[0].artifact =
            "kernel/different.vo".to_owned();

        assert_ne!(
            input.build_snapshot_id(),
            changed_dependency_hash.build_snapshot_id()
        );
        assert_ne!(
            input.build_snapshot_id(),
            changed_dependency_identity.build_snapshot_id()
        );
    }

    #[test]
    fn lockfile_toolchain_and_verifier_config_changes_change_snapshot_id() {
        let input = snapshot_input();
        let mut changed_lockfile = snapshot_input();
        changed_lockfile.lockfile_hash = hash(66);
        let mut changed_toolchain = snapshot_input();
        changed_toolchain.toolchain = ToolchainInfo::new("mizar-2026.2");
        let mut changed_verifier_config = snapshot_input();
        changed_verifier_config.verifier_config_hash = hash(77);

        assert_ne!(
            input.build_snapshot_id(),
            changed_lockfile.build_snapshot_id()
        );
        assert_ne!(
            input.build_snapshot_id(),
            changed_toolchain.build_snapshot_id()
        );
        assert_ne!(
            input.build_snapshot_id(),
            changed_verifier_config.build_snapshot_id()
        );
    }

    #[test]
    fn workspace_root_change_changes_snapshot_id() {
        let input = snapshot_input();
        let mut changed_workspace = snapshot_input();
        changed_workspace.workspace_root = WorkspaceRoot::new("other-workspace");

        assert_ne!(
            input.build_snapshot_id(),
            changed_workspace.build_snapshot_id()
        );
    }

    #[test]
    fn source_and_dependency_insertion_order_do_not_change_snapshot_id() {
        let input = snapshot_input();
        let mut reordered = snapshot_input();
        reordered.source_versions.reverse();
        reordered.dependency_artifacts.reverse();

        assert_eq!(input.build_snapshot_id(), reordered.build_snapshot_id());

        let snapshot = reordered.build_snapshot();
        assert_eq!(
            canonical_summary(&snapshot.source_versions),
            vec![
                ("mml", "alpha", "src/alpha.miz", 1),
                ("mml", "beta", "src/beta.miz", 2),
            ]
        );
        assert_eq!(
            snapshot
                .dependency_artifacts
                .iter()
                .map(|artifact| artifact.artifact.as_str())
                .collect::<Vec<_>>(),
            vec!["kernel/base.vo", "kernel/order.vo"]
        );
    }

    #[test]
    fn source_identity_tie_breakers_do_not_make_hashing_insertion_order_dependent() {
        let snapshot_id = snapshot_id(13);
        let ids = InMemorySessionIdAllocator::new();
        let alpha_path = source_version(
            ids.next_source_id(snapshot_id).unwrap(),
            "mml",
            "same.module",
            "src/alpha.miz",
            hash(8),
            SourceOrigin::Disk,
        );
        let beta_path = source_version(
            ids.next_source_id(snapshot_id).unwrap(),
            "mml",
            "same.module",
            "src/beta.miz",
            hash(1),
            SourceOrigin::Disk,
        );
        let path_order = snapshot_input_with_sources(vec![beta_path.clone(), alpha_path.clone()]);
        let reverse_path_order = snapshot_input_with_sources(vec![alpha_path, beta_path]);

        assert_eq!(
            path_order.build_snapshot_id(),
            reverse_path_order.build_snapshot_id()
        );
        assert_eq!(
            canonical_summary(&path_order.build_snapshot().source_versions),
            vec![
                ("mml", "same.module", "src/alpha.miz", 8),
                ("mml", "same.module", "src/beta.miz", 1),
            ]
        );

        let lower_hash = source_version(
            ids.next_source_id(snapshot_id).unwrap(),
            "mml",
            "same.module",
            "src/same.miz",
            hash(2),
            SourceOrigin::Disk,
        );
        let higher_hash = source_version(
            ids.next_source_id(snapshot_id).unwrap(),
            "mml",
            "same.module",
            "src/same.miz",
            hash(9),
            SourceOrigin::Disk,
        );
        let hash_order = snapshot_input_with_sources(vec![higher_hash.clone(), lower_hash.clone()]);
        let reverse_hash_order = snapshot_input_with_sources(vec![lower_hash, higher_hash]);

        assert_eq!(
            hash_order.build_snapshot_id(),
            reverse_hash_order.build_snapshot_id()
        );
        assert_eq!(
            canonical_summary(&hash_order.build_snapshot().source_versions),
            vec![
                ("mml", "same.module", "src/same.miz", 2),
                ("mml", "same.module", "src/same.miz", 9),
            ]
        );
    }

    #[test]
    fn dependency_content_hash_tie_breaker_does_not_make_hashing_insertion_order_dependent() {
        let mut input = snapshot_input();
        input.dependency_artifacts = vec![
            DependencyArtifactRef::new("kernel/same.vo", hash(9)),
            DependencyArtifactRef::new("kernel/same.vo", hash(2)),
        ];
        let mut reverse_order = snapshot_input();
        reverse_order.dependency_artifacts = input
            .dependency_artifacts
            .iter()
            .cloned()
            .rev()
            .collect::<Vec<_>>();

        assert_eq!(input.build_snapshot_id(), reverse_order.build_snapshot_id());
        assert_eq!(
            dependency_summary(&input.build_snapshot().dependency_artifacts),
            vec![("kernel/same.vo", 2), ("kernel/same.vo", 9)]
        );
    }

    #[test]
    fn session_local_source_ids_and_origins_are_absent_from_snapshot_hash() {
        let snapshot_id = snapshot_id(10);
        let ids = InMemorySessionIdAllocator::new();
        let first_source = ids.next_source_id(snapshot_id).unwrap();
        let second_source = ids.next_source_id(snapshot_id).unwrap();
        let mut first = snapshot_input_with_sources(vec![source_version(
            first_source,
            "mml",
            "alpha",
            "src/alpha.miz",
            hash(1),
            SourceOrigin::Disk,
        )]);
        let second = snapshot_input_with_sources(vec![source_version(
            second_source,
            "mml",
            "alpha",
            "src/alpha.miz",
            hash(1),
            SourceOrigin::OpenBuffer { version: 99 },
        )]);
        first.workspace_root = WorkspaceRoot::new("workspace");

        assert_ne!(
            first.source_versions[0].source_id,
            second.source_versions[0].source_id
        );
        assert_ne!(
            first.source_versions[0].origin,
            second.source_versions[0].origin
        );
        assert_eq!(first.build_snapshot_id(), second.build_snapshot_id());
    }

    #[test]
    fn equal_source_canonical_keys_do_not_make_hashing_insertion_order_dependent() {
        let snapshot_id = snapshot_id(11);
        let ids = InMemorySessionIdAllocator::new();
        let old_edition = source_version_with_edition(
            ids.next_source_id(snapshot_id).unwrap(),
            "mml",
            "same.module",
            "src/same.miz",
            hash(9),
            Edition::new("2025"),
            SourceOrigin::Disk,
        );
        let new_edition = source_version_with_edition(
            ids.next_source_id(snapshot_id).unwrap(),
            "mml",
            "same.module",
            "src/same.miz",
            hash(9),
            Edition::new("2026"),
            SourceOrigin::Generated {
                generator: GeneratedSourceKind::new("different-origin"),
            },
        );

        assert_eq!(
            old_edition.canonical_sort_key(),
            new_edition.canonical_sort_key()
        );

        let insertion_order =
            snapshot_input_with_sources(vec![old_edition.clone(), new_edition.clone()]);
        let reverse_order = snapshot_input_with_sources(vec![new_edition, old_edition]);

        assert_eq!(
            insertion_order.build_snapshot_id(),
            reverse_order.build_snapshot_id()
        );
    }

    #[test]
    fn equal_source_identity_summaries_do_not_make_hashing_insertion_order_dependent() {
        let snapshot_id = snapshot_id(12);
        let ids = InMemorySessionIdAllocator::new();
        let disk = source_version(
            ids.next_source_id(snapshot_id).unwrap(),
            "mml",
            "same.module",
            "src/same.miz",
            hash(9),
            SourceOrigin::Disk,
        );
        let open_buffer = source_version(
            ids.next_source_id(snapshot_id).unwrap(),
            "mml",
            "same.module",
            "src/same.miz",
            hash(9),
            SourceOrigin::OpenBuffer { version: 99 },
        );

        assert_eq!(disk.canonical_sort_key(), open_buffer.canonical_sort_key());
        assert_eq!(disk.edition, open_buffer.edition);
        assert_ne!(disk.source_id, open_buffer.source_id);
        assert_ne!(disk.origin, open_buffer.origin);

        let insertion_order = snapshot_input_with_sources(vec![disk.clone(), open_buffer.clone()]);
        let reverse_order = snapshot_input_with_sources(vec![open_buffer, disk]);

        assert_eq!(
            insertion_order.build_snapshot_id(),
            reverse_order.build_snapshot_id()
        );
    }

    #[test]
    fn source_versions_sort_deterministically_by_canonical_key() {
        let snapshot_id = snapshot_id(1);
        let ids = InMemorySessionIdAllocator::new();
        let first_source = ids.next_source_id(snapshot_id).unwrap();
        let second_source = ids.next_source_id(snapshot_id).unwrap();
        let third_source = ids.next_source_id(snapshot_id).unwrap();
        let fourth_source = ids.next_source_id(snapshot_id).unwrap();

        let versions = vec![
            source_version(
                third_source,
                "pkg-b",
                "alpha",
                "src/alpha.miz",
                hash(1),
                SourceOrigin::Disk,
            ),
            source_version(
                first_source,
                "pkg-a",
                "beta",
                "src/beta.miz",
                hash(3),
                SourceOrigin::OpenBuffer { version: 7 },
            ),
            source_version(
                fourth_source,
                "pkg-a",
                "alpha",
                "src/alpha.miz",
                hash(9),
                SourceOrigin::Generated {
                    generator: GeneratedSourceKind::new("test-generator"),
                },
            ),
            source_version(
                second_source,
                "pkg-a",
                "alpha",
                "src/alpha.miz",
                hash(2),
                SourceOrigin::Disk,
            ),
        ];
        let mut insertion_order = versions.clone();
        let mut reverse_order = versions.into_iter().rev().collect::<Vec<_>>();

        sort_source_versions_canonical(&mut insertion_order);
        sort_source_versions_canonical(&mut reverse_order);

        assert_eq!(
            canonical_summary(&insertion_order),
            vec![
                ("pkg-a", "alpha", "src/alpha.miz", 2),
                ("pkg-a", "alpha", "src/alpha.miz", 9),
                ("pkg-a", "beta", "src/beta.miz", 3),
                ("pkg-b", "alpha", "src/alpha.miz", 1),
            ]
        );
        assert_eq!(
            canonical_summary(&insertion_order),
            canonical_summary(&reverse_order)
        );
    }

    #[test]
    fn source_version_canonical_key_uses_all_specified_fields_in_order() {
        let snapshot_id = snapshot_id(2);
        let ids = InMemorySessionIdAllocator::new();
        let base_source = ids.next_source_id(snapshot_id).unwrap();
        let package_tie_breaker_source = ids.next_source_id(snapshot_id).unwrap();
        let module_tie_breaker_source = ids.next_source_id(snapshot_id).unwrap();
        let path_tie_breaker_source = ids.next_source_id(snapshot_id).unwrap();
        let hash_tie_breaker_source = ids.next_source_id(snapshot_id).unwrap();

        let base = source_version(
            base_source,
            "pkg-a",
            "alpha",
            "src/alpha.miz",
            hash(1),
            SourceOrigin::Disk,
        );
        let package_tie_breaker = source_version(
            package_tie_breaker_source,
            "pkg-b",
            "alpha",
            "src/alpha.miz",
            hash(0),
            SourceOrigin::Disk,
        );
        let module_tie_breaker = source_version(
            module_tie_breaker_source,
            "pkg-a",
            "beta",
            "src/alpha.miz",
            hash(0),
            SourceOrigin::Disk,
        );
        let path_tie_breaker = source_version(
            path_tie_breaker_source,
            "pkg-a",
            "alpha",
            "src/beta.miz",
            hash(0),
            SourceOrigin::Disk,
        );
        let hash_tie_breaker = source_version(
            hash_tie_breaker_source,
            "pkg-a",
            "alpha",
            "src/alpha.miz",
            hash(2),
            SourceOrigin::Disk,
        );

        assert!(base.canonical_sort_key() < package_tie_breaker.canonical_sort_key());
        assert!(base.canonical_sort_key() < module_tie_breaker.canonical_sort_key());
        assert!(base.canonical_sort_key() < path_tie_breaker.canonical_sort_key());
        assert!(base.canonical_sort_key() < hash_tie_breaker.canonical_sort_key());
    }

    #[test]
    fn source_versions_sort_by_normalized_path_before_source_hash() {
        let snapshot_id = snapshot_id(3);
        let ids = InMemorySessionIdAllocator::new();
        let mut versions = vec![
            source_version(
                ids.next_source_id(snapshot_id).unwrap(),
                "pkg",
                "same.module",
                "src/beta.miz",
                hash(0),
                SourceOrigin::Disk,
            ),
            source_version(
                ids.next_source_id(snapshot_id).unwrap(),
                "pkg",
                "same.module",
                "src/alpha.miz",
                hash(255),
                SourceOrigin::Disk,
            ),
        ];

        sort_source_versions_canonical(&mut versions);

        assert_eq!(
            canonical_summary(&versions),
            vec![
                ("pkg", "same.module", "src/alpha.miz", 255),
                ("pkg", "same.module", "src/beta.miz", 0),
            ]
        );
    }

    #[test]
    fn source_version_canonical_key_excludes_session_local_and_non_key_fields() {
        let snapshot_id = snapshot_id(4);
        let ids = InMemorySessionIdAllocator::new();
        let disk = source_version_with_edition(
            ids.next_source_id(snapshot_id).unwrap(),
            "pkg",
            "same.module",
            "src/same.miz",
            hash(7),
            Edition::new("2026"),
            SourceOrigin::Disk,
        );
        let open_buffer = source_version_with_edition(
            ids.next_source_id(snapshot_id).unwrap(),
            "pkg",
            "same.module",
            "src/same.miz",
            hash(7),
            Edition::new("future-test-edition"),
            SourceOrigin::OpenBuffer { version: 99 },
        );

        assert_ne!(disk.source_id, open_buffer.source_id);
        assert_ne!(disk.edition, open_buffer.edition);
        assert_ne!(disk.origin, open_buffer.origin);
        assert_eq!(disk.canonical_sort_key(), open_buffer.canonical_sort_key());
    }

    #[test]
    fn source_version_records_source_id_origin_hash_and_path_identity() {
        let snapshot_id = snapshot_id(5);
        let source_id = InMemorySessionIdAllocator::new()
            .next_source_id(snapshot_id)
            .unwrap();
        let normalized_path = normalized_path("src/groups/basic.miz");
        let source_hash = hash(42);
        let version = SourceVersion {
            source_id,
            package_id: PackageId::new("mml"),
            module_path: ModulePath::new("groups.basic"),
            normalized_path: normalized_path.clone(),
            source_hash,
            edition: Edition::new("2026"),
            origin: SourceOrigin::Disk,
        };

        assert_eq!(version.source_id, source_id);
        assert_eq!(version.package_id.as_str(), "mml");
        assert_eq!(version.module_path.as_str(), "groups.basic");
        assert_eq!(version.normalized_path, normalized_path);
        assert_eq!(version.source_hash, source_hash);
        assert_eq!(version.edition.as_str(), "2026");
        assert_eq!(version.origin, SourceOrigin::Disk);
    }

    #[test]
    fn source_origin_represents_disk_open_buffer_and_generated_sources() {
        let disk = SourceOrigin::Disk;
        let open_buffer = SourceOrigin::OpenBuffer { version: 11 };
        let generated = SourceOrigin::Generated {
            generator: GeneratedSourceKind::new("macro-expansion"),
        };

        assert_eq!(disk, SourceOrigin::Disk);
        assert_eq!(open_buffer, SourceOrigin::OpenBuffer { version: 11 });
        assert_eq!(
            generated,
            SourceOrigin::Generated {
                generator: GeneratedSourceKind::new("macro-expansion")
            }
        );
    }

    #[test]
    fn snapshot_error_basic_variants_exist() {
        let source_path_error = SourcePathError::UnsupportedExtension {
            path: Path::new("src/basic.txt").to_owned(),
        };
        let unknown = snapshot_id(6);
        let expected = snapshot_id(7);
        let actual = snapshot_id(8);
        let lease_id = InMemorySessionIdAllocator::new()
            .next_lease_id(expected)
            .unwrap();

        let invalid_source_path = SnapshotError::InvalidSourcePath {
            error: source_path_error,
        };
        let duplicate_module_path = SnapshotError::DuplicateModulePath {
            package_id: PackageId::new("mml"),
            module_path: ModulePath::new("groups.basic"),
        };
        let missing_dependency_artifact = SnapshotError::MissingDependencyArtifact {
            artifact: "dep.artifact".to_owned(),
        };
        let unsupported_lockfile_metadata = SnapshotError::UnsupportedLockfileMetadata {
            metadata: "lock-v99".to_owned(),
        };
        let unsupported_toolchain_metadata = SnapshotError::UnsupportedToolchainMetadata {
            metadata: "toolchain-v99".to_owned(),
        };
        let stale_open_buffer_version = SnapshotError::StaleOpenBufferVersion {
            expected: 12,
            actual: 11,
        };
        let unknown_snapshot_id = SnapshotError::UnknownSnapshotId {
            snapshot_id: unknown,
        };
        let lease_release_mismatch = SnapshotError::LeaseReleaseMismatch {
            lease_id,
            expected_snapshot: expected,
            actual_snapshot: actual,
        };

        assert!(matches!(
            invalid_source_path,
            SnapshotError::InvalidSourcePath {
                error: SourcePathError::UnsupportedExtension { .. }
            }
        ));
        assert!(matches!(
            duplicate_module_path,
            SnapshotError::DuplicateModulePath {
                package_id,
                module_path
            } if package_id.as_str() == "mml" && module_path.as_str() == "groups.basic"
        ));
        assert!(matches!(
            missing_dependency_artifact,
            SnapshotError::MissingDependencyArtifact { artifact } if artifact == "dep.artifact"
        ));
        assert!(matches!(
            unsupported_lockfile_metadata,
            SnapshotError::UnsupportedLockfileMetadata { metadata } if metadata == "lock-v99"
        ));
        assert!(matches!(
            unsupported_toolchain_metadata,
            SnapshotError::UnsupportedToolchainMetadata { metadata } if metadata == "toolchain-v99"
        ));
        assert!(matches!(
            stale_open_buffer_version,
            SnapshotError::StaleOpenBufferVersion {
                expected: 12,
                actual: 11
            }
        ));
        assert!(matches!(
            unknown_snapshot_id,
            SnapshotError::UnknownSnapshotId { snapshot_id } if snapshot_id == unknown
        ));
        assert!(matches!(
            lease_release_mismatch,
            SnapshotError::LeaseReleaseMismatch {
                lease_id: actual_lease_id,
                expected_snapshot,
                actual_snapshot,
            } if actual_lease_id == lease_id
                && expected_snapshot == expected
                && actual_snapshot == actual
        ));
    }

    #[test]
    fn snapshot_error_exposes_source_path_error_as_error_source() {
        let invalid_source_path = SnapshotError::InvalidSourcePath {
            error: SourcePathError::UnsupportedExtension {
                path: Path::new("src/basic.txt").to_owned(),
            },
        };
        let missing_dependency_artifact = SnapshotError::MissingDependencyArtifact {
            artifact: "dep.artifact".to_owned(),
        };

        assert!(std::error::Error::source(&invalid_source_path).is_some());
        assert!(std::error::Error::source(&missing_dependency_artifact).is_none());
    }

    #[test]
    fn source_version_canonical_order_does_not_require_source_id_ordering() {
        fn requires_ord<T: Ord>() {}

        requires_ord::<super::SourceVersionCanonicalKey<'_>>();

        let snapshot_id = snapshot_id(6);
        let ids = InMemorySessionIdAllocator::new();
        let low_allocated_source = ids.next_source_id(snapshot_id).unwrap();
        let high_allocated_source = ids.next_source_id(snapshot_id).unwrap();
        let mut versions = vec![
            source_version(
                low_allocated_source,
                "pkg",
                "zeta",
                "src/zeta.miz",
                hash(0),
                SourceOrigin::Disk,
            ),
            source_version(
                high_allocated_source,
                "pkg",
                "alpha",
                "src/alpha.miz",
                hash(0),
                SourceOrigin::Disk,
            ),
        ];

        sort_source_versions_canonical(&mut versions);

        assert_eq!(versions[0].source_id, high_allocated_source);
        assert_eq!(versions[0].module_path.as_str(), "alpha");
    }

    fn source_version(
        source_id: crate::SourceId,
        package_id: &str,
        module_path: &str,
        normalized_path: &str,
        source_hash: Hash,
        origin: SourceOrigin,
    ) -> SourceVersion {
        source_version_with_edition(
            source_id,
            package_id,
            module_path,
            normalized_path,
            source_hash,
            Edition::new("2026"),
            origin,
        )
    }

    fn source_version_with_edition(
        source_id: crate::SourceId,
        package_id: &str,
        module_path: &str,
        normalized_path: &str,
        source_hash: Hash,
        edition: Edition,
        origin: SourceOrigin,
    ) -> SourceVersion {
        SourceVersion {
            source_id,
            package_id: PackageId::new(package_id),
            module_path: ModulePath::new(module_path),
            normalized_path: normalized_path_from_source_path(normalized_path),
            source_hash,
            edition,
            origin,
        }
    }

    fn snapshot_input() -> SnapshotInput {
        let snapshot_id = snapshot_id(9);
        let ids = InMemorySessionIdAllocator::new();
        snapshot_input_with_sources(vec![
            source_version(
                ids.next_source_id(snapshot_id).unwrap(),
                "mml",
                "beta",
                "src/beta.miz",
                hash(2),
                SourceOrigin::Disk,
            ),
            source_version(
                ids.next_source_id(snapshot_id).unwrap(),
                "mml",
                "alpha",
                "src/alpha.miz",
                hash(1),
                SourceOrigin::OpenBuffer { version: 12 },
            ),
        ])
    }

    fn snapshot_input_with_sources(source_versions: Vec<SourceVersion>) -> SnapshotInput {
        SnapshotInput {
            workspace_root: WorkspaceRoot::new("workspace"),
            source_versions,
            dependency_artifacts: vec![
                DependencyArtifactRef::new("kernel/order.vo", hash(4)),
                DependencyArtifactRef::new("kernel/base.vo", hash(3)),
            ],
            lockfile_hash: hash(5),
            toolchain: ToolchainInfo::new("mizar-2026.1"),
            verifier_config_hash: hash(6),
        }
    }

    fn canonical_summary(versions: &[SourceVersion]) -> Vec<(&str, &str, &str, u8)> {
        versions
            .iter()
            .map(|version| {
                (
                    version.package_id.as_str(),
                    version.module_path.as_str(),
                    version.normalized_path.as_str(),
                    version.source_hash.as_bytes()[0],
                )
            })
            .collect()
    }

    fn dependency_summary(artifacts: &[DependencyArtifactRef]) -> Vec<(&str, u8)> {
        artifacts
            .iter()
            .map(|artifact| {
                (
                    artifact.artifact.as_str(),
                    artifact.content_hash.as_bytes()[0],
                )
            })
            .collect()
    }

    fn hash(first_byte: u8) -> Hash {
        let mut bytes = [0; Hash::BYTE_LEN];
        bytes[0] = first_byte;
        Hash::from_bytes(bytes)
    }

    fn snapshot_id(first_byte: u8) -> BuildSnapshotId {
        let mut serialized = String::from(
            "mizar-session-build-snapshot-v1:0000000000000000000000000000000000000000000000000000000000000000",
        );
        let hex = format!("{first_byte:02x}");
        serialized.replace_range(
            "mizar-session-build-snapshot-v1:".len().."mizar-session-build-snapshot-v1:".len() + 2,
            &hex,
        );
        BuildSnapshotId::from_published_schema_str(&serialized).unwrap()
    }

    fn normalized_path(path: &str) -> NormalizedPath {
        normalized_path_from_source_path(path)
    }

    fn normalized_path_from_source_path(path: &str) -> NormalizedPath {
        let fixture = SourcePathFixture::new();
        fixture.write(path, "");
        normalize_source_path(fixture.root(), Path::new(path)).unwrap()
    }

    struct SourcePathFixture {
        base: std::path::PathBuf,
        root: std::path::PathBuf,
    }

    impl SourcePathFixture {
        fn new() -> Self {
            let id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
            let base = std::env::temp_dir().join(format!(
                "mizar_session_snapshot_source_path_{}_{}",
                std::process::id(),
                id
            ));
            let root = base.join("package");
            std::fs::create_dir_all(root.join("src")).unwrap();
            Self { base, root }
        }

        fn root(&self) -> &Path {
            &self.root
        }

        fn write(&self, relative: &str, content: &str) {
            let path = self.root.join(relative);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            std::fs::write(path, content).unwrap();
        }
    }

    impl Drop for SourcePathFixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.base);
        }
    }
}
