//! Snapshot-facing source version records.
//!
//! ```compile_fail
//! use mizar_session::SourceVersion;
//!
//! fn requires_source_version_id_order(versions: &mut [SourceVersion]) {
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
//!
//! Snapshot construction is validated through `SnapshotRegistry::create_snapshot`.
//! The unchecked identity helpers used by this crate are not public API.
//! A `BuildSnapshot` assembled with public fields is only a detached data record;
//! it is not retained, current, or retrievable from a registry unless
//! `SnapshotRegistry::create_snapshot` produced it.
//!
//! ```
//! use mizar_session::{
//!     BuildSnapshot, BuildSnapshotId, Hash, SnapshotRegistry, ToolchainInfo, WorkspaceRoot,
//! };
//!
//! let snapshot = BuildSnapshot {
//!     id: BuildSnapshotId::from_published_schema_str(
//!         "mizar-session-build-snapshot-v1:\
//!          0000000000000000000000000000000000000000000000000000000000000000",
//!     )
//!     .unwrap(),
//!     workspace_root: WorkspaceRoot::new("detached"),
//!     source_versions: Vec::new(),
//!     dependency_artifacts: Vec::new(),
//!     lockfile_hash: Hash::from_bytes([0; Hash::BYTE_LEN]),
//!     toolchain: ToolchainInfo::new(""),
//!     verifier_config_hash: Hash::from_bytes([0; Hash::BYTE_LEN]),
//! };
//!
//! let registry = SnapshotRegistry::new();
//! assert_eq!(registry.get(snapshot.id), None);
//! ```
//!
//! ```compile_fail
//! use mizar_session::{BuildSnapshot, SnapshotInput};
//!
//! let input: SnapshotInput = unimplemented!();
//! let _snapshot = BuildSnapshot::from_input(input);
//! ```
//!
//! ```compile_fail
//! use mizar_session::SnapshotInput;
//!
//! let input: SnapshotInput = unimplemented!();
//! let _id = input.build_snapshot_id();
//! ```
//!
//! ```compile_fail
//! use mizar_session::SnapshotInput;
//!
//! let input: SnapshotInput = unimplemented!();
//! let _snapshot = input.build_snapshot();
//! ```
//!
//! ```compile_fail
//! use mizar_session::{BuildSnapshot, SnapshotInput};
//!
//! let input: SnapshotInput = unimplemented!();
//! let _snapshot = BuildSnapshot::from_input_unchecked(input);
//! ```
//!
//! ```compile_fail
//! use mizar_session::SnapshotInput;
//!
//! let input: SnapshotInput = unimplemented!();
//! let _id = input.build_snapshot_id_unchecked();
//! ```
//!
//! ```compile_fail
//! use mizar_session::SnapshotInput;
//!
//! let input: SnapshotInput = unimplemented!();
//! let _snapshot = input.build_snapshot_unchecked();
//! ```
//!
use crate::{
    BuildRequestId, BuildSnapshotId, Hash, IdError, InMemorySessionIdAllocator, LspDocumentVersion,
    NormalizedPath, SessionIdAllocator, SnapshotLeaseId, SourceId, SourcePathError,
    identity::is_language_identifier, ids::build_snapshot_id_from_sorted_canonical_bytes,
};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;
use std::sync::Mutex;

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

#[derive(Debug)]
pub struct SnapshotRegistry<A = InMemorySessionIdAllocator> {
    allocator: A,
    state: Mutex<SnapshotRegistryState>,
}

#[derive(Debug, Default)]
struct SnapshotRegistryState {
    snapshots: HashMap<BuildSnapshotId, BuildSnapshot>,
    current_by_request: HashMap<BuildRequestId, BuildSnapshotId>,
    leases: HashMap<SnapshotLeaseId, SnapshotLease>,
    lease_counts: HashMap<BuildSnapshotId, HashMap<RetentionReason, usize>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotLease {
    pub lease_id: SnapshotLeaseId,
    pub snapshot: BuildSnapshotId,
    pub reason: RetentionReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum RetentionReason {
    ActiveBuild,
    CurrentWatchBaseline,
    PublishedLspSnapshot,
    OpenBufferOverlay,
    DiagnosticIndex,
    ExplanationRequest,
    PhaseOutputReference,
    PendingWrite,
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
    InvalidWorkspaceRoot {
        workspace_root: WorkspaceRoot,
    },
    InvalidPackageId {
        package_id: PackageId,
    },
    InvalidModulePath {
        package_id: PackageId,
        module_path: ModulePath,
    },
    InvalidEdition {
        package_id: PackageId,
        module_path: ModulePath,
        edition: Edition,
    },
    InvalidSourcePath {
        error: SourcePathError,
    },
    DuplicateModulePath {
        package_id: PackageId,
        module_path: ModulePath,
    },
    DuplicateSourceVersionIdentity {
        package_id: PackageId,
        module_path: ModulePath,
        normalized_path: NormalizedPath,
        source_hash: Hash,
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
    GeneratedSourceWithoutMetadata {
        module_path: ModulePath,
    },
    UnknownSnapshotId {
        snapshot_id: BuildSnapshotId,
    },
    LeaseReleaseMismatch {
        lease_id: SnapshotLeaseId,
        expected_snapshot: BuildSnapshotId,
        actual_snapshot: BuildSnapshotId,
    },
    UnknownSnapshotLease {
        lease_id: SnapshotLeaseId,
    },
    LeaseIdAllocation {
        error: IdError,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash)]
pub struct SourceVersionCanonicalKey<'a> {
    package_id: &'a str,
    module_path: &'a str,
    normalized_path: &'a str,
    source_hash: &'a [u8; Hash::BYTE_LEN],
}

impl BuildSnapshot {
    /// Builds a snapshot identity without creation validation or registry insertion.
    ///
    /// This is crate-private so public callers cannot accidentally bypass
    /// `SnapshotRegistry::create_snapshot`.
    pub(crate) fn from_input_unchecked(input: SnapshotInput) -> Self {
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

#[cfg(test)]
impl SnapshotInput {
    /// Builds a snapshot without validation; for crate-local identity tests only.
    pub(crate) fn build_snapshot_unchecked(self) -> BuildSnapshot {
        BuildSnapshot::from_input_unchecked(self)
    }

    /// Computes identity bytes without validating creation invariants.
    pub(crate) fn build_snapshot_id_unchecked(&self) -> BuildSnapshotId {
        build_snapshot_id(self)
    }
}

impl SnapshotRegistry<InMemorySessionIdAllocator> {
    pub fn new() -> Self {
        Self::with_allocator(InMemorySessionIdAllocator::new())
    }
}

impl Default for SnapshotRegistry<InMemorySessionIdAllocator> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A> SnapshotRegistry<A> {
    pub fn with_allocator(allocator: A) -> Self {
        Self {
            allocator,
            state: Mutex::new(SnapshotRegistryState::default()),
        }
    }
}

impl<A: SessionIdAllocator> SnapshotRegistry<A> {
    pub fn create_snapshot(
        &self,
        request: BuildRequestId,
        input: SnapshotInput,
    ) -> Result<(BuildSnapshot, SnapshotLease), SnapshotError> {
        validate_snapshot_input(&input)?;

        let snapshot = BuildSnapshot::from_input_unchecked(input);
        let lease = self.allocate_lease(snapshot.id, RetentionReason::ActiveBuild)?;

        let mut state = self.state.lock().expect("snapshot registry mutex poisoned");
        state.snapshots.insert(snapshot.id, snapshot.clone());
        state.current_by_request.insert(request, snapshot.id);
        state.record_lease(lease.clone());

        Ok((snapshot, lease))
    }

    pub fn acquire_lease(
        &self,
        snapshot: BuildSnapshotId,
        reason: RetentionReason,
    ) -> Result<SnapshotLease, SnapshotError> {
        {
            let state = self.state.lock().expect("snapshot registry mutex poisoned");
            if !state.snapshots.contains_key(&snapshot) {
                return Err(SnapshotError::UnknownSnapshotId {
                    snapshot_id: snapshot,
                });
            }
        }

        let lease = self.allocate_lease(snapshot, reason)?;
        let mut state = self.state.lock().expect("snapshot registry mutex poisoned");
        state.record_lease(lease.clone());
        Ok(lease)
    }

    pub fn release_lease(
        &self,
        snapshot: BuildSnapshotId,
        lease_id: SnapshotLeaseId,
    ) -> Result<(), SnapshotError> {
        let mut state = self.state.lock().expect("snapshot registry mutex poisoned");
        if !state.snapshots.contains_key(&snapshot) {
            return Err(SnapshotError::UnknownSnapshotId {
                snapshot_id: snapshot,
            });
        }

        let Some(lease) = state.leases.get(&lease_id).cloned() else {
            return Err(SnapshotError::UnknownSnapshotLease { lease_id });
        };

        if lease.snapshot != snapshot {
            return Err(SnapshotError::LeaseReleaseMismatch {
                lease_id,
                expected_snapshot: snapshot,
                actual_snapshot: lease.snapshot,
            });
        }

        state.leases.remove(&lease_id);
        state.decrement_lease_count(lease.snapshot, lease.reason);
        Ok(())
    }

    pub fn get(&self, id: BuildSnapshotId) -> Option<BuildSnapshot> {
        self.state
            .lock()
            .expect("snapshot registry mutex poisoned")
            .snapshots
            .get(&id)
            .cloned()
    }

    pub fn is_current_for_request(&self, id: BuildSnapshotId, request: BuildRequestId) -> bool {
        self.state
            .lock()
            .expect("snapshot registry mutex poisoned")
            .current_by_request
            .get(&request)
            .is_some_and(|current| *current == id)
    }

    fn allocate_lease(
        &self,
        snapshot: BuildSnapshotId,
        reason: RetentionReason,
    ) -> Result<SnapshotLease, SnapshotError> {
        Ok(SnapshotLease {
            lease_id: self
                .allocator
                .next_lease_id(snapshot)
                .map_err(|error| SnapshotError::LeaseIdAllocation { error })?,
            snapshot,
            reason,
        })
    }
}

impl SnapshotRegistryState {
    fn record_lease(&mut self, lease: SnapshotLease) {
        *self
            .lease_counts
            .entry(lease.snapshot)
            .or_default()
            .entry(lease.reason)
            .or_default() += 1;
        self.leases.insert(lease.lease_id, lease);
    }

    fn decrement_lease_count(&mut self, snapshot: BuildSnapshotId, reason: RetentionReason) {
        let Some(counts_by_reason) = self.lease_counts.get_mut(&snapshot) else {
            return;
        };
        let Some(count) = counts_by_reason.get_mut(&reason) else {
            return;
        };

        *count -= 1;
        if *count == 0 {
            counts_by_reason.remove(&reason);
        }
        if counts_by_reason.is_empty() {
            self.lease_counts.remove(&snapshot);
        }
    }
}

#[cfg(test)]
impl<A> SnapshotRegistry<A> {
    fn lease_count_for_test(&self, snapshot: BuildSnapshotId, reason: RetentionReason) -> usize {
        self.state
            .lock()
            .expect("snapshot registry mutex poisoned")
            .lease_counts
            .get(&snapshot)
            .and_then(|counts_by_reason| counts_by_reason.get(&reason))
            .copied()
            .unwrap_or_default()
    }

    fn total_lease_count_for_test(&self, snapshot: BuildSnapshotId) -> usize {
        self.state
            .lock()
            .expect("snapshot registry mutex poisoned")
            .lease_counts
            .get(&snapshot)
            .map(|counts_by_reason| counts_by_reason.values().sum())
            .unwrap_or_default()
    }

    fn live_lease_count_for_test(&self, snapshot: BuildSnapshotId) -> usize {
        self.state
            .lock()
            .expect("snapshot registry mutex poisoned")
            .leases
            .values()
            .filter(|lease| lease.snapshot == snapshot)
            .count()
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

fn validate_snapshot_input(input: &SnapshotInput) -> Result<(), SnapshotError> {
    reject_invalid_workspace_root(&input.workspace_root)?;
    reject_invalid_source_identity_values(&input.source_versions)?;
    reject_duplicate_source_version_identities(&input.source_versions)?;
    reject_duplicate_module_paths(&input.source_versions)?;
    reject_missing_dependency_artifacts(&input.dependency_artifacts)?;
    reject_unsupported_lockfile_metadata(input.lockfile_hash)?;
    reject_unsupported_toolchain_metadata(&input.toolchain)?;
    reject_invalid_open_buffer_versions(&input.source_versions)?;
    Ok(())
}

fn reject_invalid_workspace_root(workspace_root: &WorkspaceRoot) -> Result<(), SnapshotError> {
    if workspace_root.as_str().trim().is_empty() {
        return Err(SnapshotError::InvalidWorkspaceRoot {
            workspace_root: workspace_root.clone(),
        });
    }
    Ok(())
}

fn reject_invalid_source_identity_values(
    source_versions: &[SourceVersion],
) -> Result<(), SnapshotError> {
    for version in source_versions {
        if !is_valid_package_id(version.package_id.as_str()) {
            return Err(SnapshotError::InvalidPackageId {
                package_id: version.package_id.clone(),
            });
        }
        if !is_valid_module_path(version.module_path.as_str()) {
            return Err(SnapshotError::InvalidModulePath {
                package_id: version.package_id.clone(),
                module_path: version.module_path.clone(),
            });
        }
        if version.edition.as_str().trim().is_empty() {
            return Err(SnapshotError::InvalidEdition {
                package_id: version.package_id.clone(),
                module_path: version.module_path.clone(),
                edition: version.edition.clone(),
            });
        }
        if let SourceOrigin::Generated { generator } = &version.origin
            && generator.as_str().trim().is_empty()
        {
            return Err(SnapshotError::GeneratedSourceWithoutMetadata {
                module_path: version.module_path.clone(),
            });
        }
    }
    Ok(())
}

fn reject_duplicate_source_version_identities(
    source_versions: &[SourceVersion],
) -> Result<(), SnapshotError> {
    let mut seen = HashSet::new();
    for version in source_versions {
        if !seen.insert(version.canonical_sort_key()) {
            return Err(SnapshotError::DuplicateSourceVersionIdentity {
                package_id: version.package_id.clone(),
                module_path: version.module_path.clone(),
                normalized_path: version.normalized_path.clone(),
                source_hash: version.source_hash,
            });
        }
    }
    Ok(())
}

fn reject_duplicate_module_paths(source_versions: &[SourceVersion]) -> Result<(), SnapshotError> {
    let mut seen = HashSet::new();
    for version in source_versions {
        let key = (version.package_id.as_str(), version.module_path.as_str());
        if !seen.insert(key) {
            return Err(SnapshotError::DuplicateModulePath {
                package_id: version.package_id.clone(),
                module_path: version.module_path.clone(),
            });
        }
    }
    Ok(())
}

fn reject_missing_dependency_artifacts(
    dependency_artifacts: &[DependencyArtifactRef],
) -> Result<(), SnapshotError> {
    for artifact in dependency_artifacts {
        if artifact.artifact.trim().is_empty() || hash_is_zero(artifact.content_hash) {
            return Err(SnapshotError::MissingDependencyArtifact {
                artifact: artifact.artifact.clone(),
            });
        }
    }
    Ok(())
}

fn reject_unsupported_lockfile_metadata(lockfile_hash: Hash) -> Result<(), SnapshotError> {
    if hash_is_zero(lockfile_hash) {
        return Err(SnapshotError::UnsupportedLockfileMetadata {
            metadata: "missing-lockfile-hash".to_owned(),
        });
    }
    Ok(())
}

fn reject_unsupported_toolchain_metadata(toolchain: &ToolchainInfo) -> Result<(), SnapshotError> {
    if toolchain.identity().trim().is_empty() {
        return Err(SnapshotError::UnsupportedToolchainMetadata {
            metadata: toolchain.identity().to_owned(),
        });
    }
    Ok(())
}

fn reject_invalid_open_buffer_versions(
    source_versions: &[SourceVersion],
) -> Result<(), SnapshotError> {
    for version in source_versions {
        // Task 11 only has loaded snapshot input; expected-version comparison lives in source loading.
        if let SourceOrigin::OpenBuffer { version: actual } = &version.origin
            && *actual < 0
        {
            return Err(SnapshotError::StaleOpenBufferVersion {
                expected: 0,
                actual: *actual,
            });
        }
    }
    Ok(())
}

fn hash_is_zero(hash: Hash) -> bool {
    hash.as_bytes().iter().all(|byte| *byte == 0)
}

fn is_valid_package_id(value: &str) -> bool {
    !value.trim().is_empty() && !value.chars().any(char::is_whitespace)
}

fn is_valid_module_path(value: &str) -> bool {
    !value.is_empty() && value.split('.').all(is_language_identifier)
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
    left.canonical_sort_key()
        .cmp(&right.canonical_sort_key())
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
            Self::InvalidWorkspaceRoot { workspace_root } => {
                write!(f, "invalid workspace root `{workspace_root}`")
            }
            Self::InvalidPackageId { package_id } => {
                write!(f, "invalid package id `{package_id}`")
            }
            Self::InvalidModulePath {
                package_id,
                module_path,
            } => {
                write!(
                    f,
                    "invalid module path `{module_path}` in package `{package_id}`"
                )
            }
            Self::InvalidEdition {
                package_id,
                module_path,
                edition,
            } => {
                write!(
                    f,
                    "invalid edition `{edition}` for `{module_path}` in package `{package_id}`"
                )
            }
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
            Self::DuplicateSourceVersionIdentity {
                package_id,
                module_path,
                normalized_path,
                ..
            } => {
                write!(
                    f,
                    "duplicate source version identity for `{module_path}` in package `{package_id}` at `{normalized_path}`"
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
            Self::GeneratedSourceWithoutMetadata { module_path } => {
                write!(
                    f,
                    "generated source for module `{module_path}` is missing required generator metadata"
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
            Self::UnknownSnapshotLease { lease_id } => {
                write!(f, "unknown snapshot lease `{lease_id:?}`")
            }
            Self::LeaseIdAllocation { error } => {
                write!(f, "could not allocate snapshot lease id: {error}")
            }
        }
    }
}

impl Error for SnapshotError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidSourcePath { error } => Some(error),
            Self::LeaseIdAllocation { error } => Some(error),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BuildSnapshot, DependencyArtifactRef, Edition, GeneratedSourceKind, ModulePath, PackageId,
        RetentionReason, SnapshotError, SnapshotInput, SnapshotRegistry, SnapshotRegistryState,
        SourceOrigin, SourceVersion, ToolchainInfo, WorkspaceRoot, sort_source_versions_canonical,
    };
    use crate::{
        BuildRequestId, BuildSessionId, BuildSnapshotId, Hash, IdError, InMemorySessionIdAllocator,
        NormalizedPath, SessionIdAllocator, SnapshotLeaseId, SourceId, SourceMapId,
        SourcePathError, normalize_source_path,
    };
    use std::path::Path;
    use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

    static NEXT_FIXTURE_ID: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn identical_canonical_snapshot_inputs_produce_identical_ids() {
        let input = snapshot_input();
        let same = snapshot_input();

        assert_eq!(
            input.build_snapshot_id_unchecked(),
            same.build_snapshot_id_unchecked()
        );
        assert_eq!(
            BuildSnapshot::from_input_unchecked(input).id,
            same.build_snapshot_id_unchecked()
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
            input.build_snapshot_id_unchecked(),
            changed_package_id.build_snapshot_id_unchecked()
        );
        assert_ne!(
            input.build_snapshot_id_unchecked(),
            changed_module_path.build_snapshot_id_unchecked()
        );
        assert_ne!(
            input.build_snapshot_id_unchecked(),
            changed_normalized_path.build_snapshot_id_unchecked()
        );
        assert_ne!(
            input.build_snapshot_id_unchecked(),
            changed_source_hash.build_snapshot_id_unchecked()
        );
        assert_ne!(
            input.build_snapshot_id_unchecked(),
            changed_edition.build_snapshot_id_unchecked()
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
            input.build_snapshot_id_unchecked(),
            changed_dependency_hash.build_snapshot_id_unchecked()
        );
        assert_ne!(
            input.build_snapshot_id_unchecked(),
            changed_dependency_identity.build_snapshot_id_unchecked()
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
            input.build_snapshot_id_unchecked(),
            changed_lockfile.build_snapshot_id_unchecked()
        );
        assert_ne!(
            input.build_snapshot_id_unchecked(),
            changed_toolchain.build_snapshot_id_unchecked()
        );
        assert_ne!(
            input.build_snapshot_id_unchecked(),
            changed_verifier_config.build_snapshot_id_unchecked()
        );
    }

    #[test]
    fn workspace_root_change_changes_snapshot_id() {
        let input = snapshot_input();
        let mut changed_workspace = snapshot_input();
        changed_workspace.workspace_root = WorkspaceRoot::new("other-workspace");

        assert_ne!(
            input.build_snapshot_id_unchecked(),
            changed_workspace.build_snapshot_id_unchecked()
        );
    }

    #[test]
    fn source_and_dependency_insertion_order_do_not_change_snapshot_id() {
        let input = snapshot_input();
        let mut reordered = snapshot_input();
        reordered.source_versions.reverse();
        reordered.dependency_artifacts.reverse();

        assert_eq!(
            input.build_snapshot_id_unchecked(),
            reordered.build_snapshot_id_unchecked()
        );

        let snapshot = reordered.build_snapshot_unchecked();
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
            path_order.build_snapshot_id_unchecked(),
            reverse_path_order.build_snapshot_id_unchecked()
        );
        assert_eq!(
            canonical_summary(&path_order.build_snapshot_unchecked().source_versions),
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
            hash_order.build_snapshot_id_unchecked(),
            reverse_hash_order.build_snapshot_id_unchecked()
        );
        assert_eq!(
            canonical_summary(&hash_order.build_snapshot_unchecked().source_versions),
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

        assert_eq!(
            input.build_snapshot_id_unchecked(),
            reverse_order.build_snapshot_id_unchecked()
        );
        assert_eq!(
            dependency_summary(&input.build_snapshot_unchecked().dependency_artifacts),
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
        assert_eq!(
            first.build_snapshot_id_unchecked(),
            second.build_snapshot_id_unchecked()
        );
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
            insertion_order.build_snapshot_id_unchecked(),
            reverse_order.build_snapshot_id_unchecked()
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
            insertion_order.build_snapshot_id_unchecked(),
            reverse_order.build_snapshot_id_unchecked()
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
    fn created_snapshot_is_retrievable_and_returns_active_build_lease() {
        let registry = SnapshotRegistry::new();
        let request = request_id();
        let input = snapshot_input();
        let expected_snapshot_id = input.build_snapshot_id_unchecked();

        let (snapshot, lease) = registry.create_snapshot(request, input).unwrap();

        assert_eq!(snapshot.id, expected_snapshot_id);
        assert_eq!(registry.get(snapshot.id), Some(snapshot.clone()));
        assert_eq!(lease.snapshot, snapshot.id);
        assert_eq!(lease.reason, RetentionReason::ActiveBuild);
        assert!(registry.is_current_for_request(snapshot.id, request));
        assert_eq!(
            registry.lease_count_for_test(snapshot.id, RetentionReason::ActiveBuild),
            1
        );
    }

    #[test]
    fn repeated_snapshot_creation_returns_distinct_active_build_leases() {
        let registry = SnapshotRegistry::new();
        let request = request_id();

        let (first_snapshot, first_lease) =
            registry.create_snapshot(request, snapshot_input()).unwrap();
        let (second_snapshot, second_lease) =
            registry.create_snapshot(request, snapshot_input()).unwrap();

        assert_eq!(first_snapshot.id, second_snapshot.id);
        assert_eq!(first_lease.snapshot, first_snapshot.id);
        assert_eq!(second_lease.snapshot, second_snapshot.id);
        assert_eq!(first_lease.reason, RetentionReason::ActiveBuild);
        assert_eq!(second_lease.reason, RetentionReason::ActiveBuild);
        assert_ne!(first_lease.lease_id, second_lease.lease_id);
        assert_eq!(
            registry.lease_count_for_test(first_snapshot.id, RetentionReason::ActiveBuild),
            2
        );
    }

    #[test]
    fn acquire_lease_increments_count_for_reason() {
        let registry = SnapshotRegistry::new();
        let (snapshot, _) = registry
            .create_snapshot(request_id(), snapshot_input())
            .unwrap();

        let lease = registry
            .acquire_lease(snapshot.id, RetentionReason::DiagnosticIndex)
            .unwrap();

        assert_eq!(lease.snapshot, snapshot.id);
        assert_eq!(lease.reason, RetentionReason::DiagnosticIndex);
        assert_eq!(
            registry.lease_count_for_test(snapshot.id, RetentionReason::ActiveBuild),
            1
        );
        assert_eq!(
            registry.lease_count_for_test(snapshot.id, RetentionReason::DiagnosticIndex),
            1
        );
    }

    #[test]
    fn acquire_lease_tracks_each_retention_reason_independently() {
        let registry = SnapshotRegistry::new();
        let (snapshot, _) = registry
            .create_snapshot(request_id(), snapshot_input())
            .unwrap();

        let leases = all_retention_reasons()
            .into_iter()
            .map(|reason| registry.acquire_lease(snapshot.id, reason).unwrap())
            .collect::<Vec<_>>();

        for reason in all_retention_reasons() {
            let expected_count = if reason == RetentionReason::ActiveBuild {
                2
            } else {
                1
            };
            assert_eq!(
                registry.lease_count_for_test(snapshot.id, reason),
                expected_count,
                "lease count for {reason:?}"
            );
        }
        assert_eq!(
            registry.total_lease_count_for_test(snapshot.id),
            all_retention_reasons().len() + 1
        );

        for lease in leases {
            registry.release_lease(snapshot.id, lease.lease_id).unwrap();
        }

        assert_eq!(
            registry.lease_count_for_test(snapshot.id, RetentionReason::ActiveBuild),
            1
        );
        for reason in all_retention_reasons()
            .into_iter()
            .filter(|reason| *reason != RetentionReason::ActiveBuild)
        {
            assert_eq!(
                registry.lease_count_for_test(snapshot.id, reason),
                0,
                "lease count after release for {reason:?}"
            );
        }
        assert_eq!(registry.total_lease_count_for_test(snapshot.id), 1);
    }

    #[test]
    fn release_lease_decrements_count_for_reason() {
        let registry = SnapshotRegistry::new();
        let (snapshot, _) = registry
            .create_snapshot(request_id(), snapshot_input())
            .unwrap();
        let first = registry
            .acquire_lease(snapshot.id, RetentionReason::DiagnosticIndex)
            .unwrap();
        let second = registry
            .acquire_lease(snapshot.id, RetentionReason::DiagnosticIndex)
            .unwrap();

        registry.release_lease(snapshot.id, first.lease_id).unwrap();

        assert_eq!(
            registry.lease_count_for_test(snapshot.id, RetentionReason::DiagnosticIndex),
            1
        );
        assert_eq!(
            registry.lease_count_for_test(snapshot.id, RetentionReason::ActiveBuild),
            1
        );

        registry
            .release_lease(snapshot.id, second.lease_id)
            .unwrap();

        assert_eq!(
            registry.lease_count_for_test(snapshot.id, RetentionReason::DiagnosticIndex),
            0
        );
    }

    #[test]
    fn repeated_acquire_lease_returns_unique_ids_and_counts_by_reason() {
        let registry = SnapshotRegistry::new();
        let (snapshot, _) = registry
            .create_snapshot(request_id(), snapshot_input())
            .unwrap();

        let first = registry
            .acquire_lease(snapshot.id, RetentionReason::DiagnosticIndex)
            .unwrap();
        let second = registry
            .acquire_lease(snapshot.id, RetentionReason::DiagnosticIndex)
            .unwrap();
        let explanation = registry
            .acquire_lease(snapshot.id, RetentionReason::ExplanationRequest)
            .unwrap();

        assert_ne!(first.lease_id, second.lease_id);
        assert_ne!(first.lease_id, explanation.lease_id);
        assert_ne!(second.lease_id, explanation.lease_id);
        assert_eq!(
            registry.lease_count_for_test(snapshot.id, RetentionReason::DiagnosticIndex),
            2
        );
        assert_eq!(
            registry.lease_count_for_test(snapshot.id, RetentionReason::ExplanationRequest),
            1
        );
        assert_eq!(
            registry.lease_count_for_test(snapshot.id, RetentionReason::ActiveBuild),
            1
        );
        assert_eq!(registry.total_lease_count_for_test(snapshot.id), 4);
    }

    #[test]
    fn acquire_lease_allocates_lease_id_without_holding_registry_mutex() {
        let registry = SnapshotRegistry::with_allocator(RegistryMutexProbeAllocator::new());
        registry.allocator.bind_state(&registry.state);
        let (snapshot, _) = registry
            .create_snapshot(request_id(), snapshot_input())
            .unwrap();
        registry
            .allocator
            .registry_mutex_available_checks
            .store(0, Ordering::Relaxed);

        let lease = registry
            .acquire_lease(snapshot.id, RetentionReason::DiagnosticIndex)
            .unwrap();

        assert_eq!(lease.snapshot, snapshot.id);
        assert_eq!(lease.reason, RetentionReason::DiagnosticIndex);
        assert_eq!(
            registry
                .allocator
                .registry_mutex_available_checks
                .load(Ordering::Relaxed),
            1
        );
    }

    #[test]
    fn release_lease_only_decrements_the_released_reason() {
        let registry = SnapshotRegistry::new();
        let (snapshot, _) = registry
            .create_snapshot(request_id(), snapshot_input())
            .unwrap();
        let diagnostic = registry
            .acquire_lease(snapshot.id, RetentionReason::DiagnosticIndex)
            .unwrap();
        let explanation = registry
            .acquire_lease(snapshot.id, RetentionReason::ExplanationRequest)
            .unwrap();

        registry
            .release_lease(snapshot.id, diagnostic.lease_id)
            .unwrap();

        assert_eq!(
            registry.lease_count_for_test(snapshot.id, RetentionReason::DiagnosticIndex),
            0
        );
        assert_eq!(
            registry.lease_count_for_test(snapshot.id, RetentionReason::ExplanationRequest),
            1
        );
        assert_eq!(
            registry.lease_count_for_test(snapshot.id, RetentionReason::ActiveBuild),
            1
        );
        assert_eq!(registry.total_lease_count_for_test(snapshot.id), 2);

        registry
            .release_lease(snapshot.id, explanation.lease_id)
            .unwrap();

        assert_eq!(registry.total_lease_count_for_test(snapshot.id), 1);
    }

    #[test]
    fn release_active_build_lease_from_create_snapshot_is_accounted() {
        let registry = SnapshotRegistry::new();
        let (snapshot, active_build) = registry
            .create_snapshot(request_id(), snapshot_input())
            .unwrap();

        registry
            .release_lease(snapshot.id, active_build.lease_id)
            .unwrap();

        assert_eq!(
            registry.lease_count_for_test(snapshot.id, RetentionReason::ActiveBuild),
            0
        );
        assert_eq!(registry.get(snapshot.id), Some(snapshot));
    }

    #[test]
    fn acquire_lease_for_unknown_snapshot_returns_unknown_snapshot_id() {
        let registry = SnapshotRegistry::new();
        let unknown = snapshot_id(201);

        let error = registry
            .acquire_lease(unknown, RetentionReason::ExplanationRequest)
            .unwrap_err();

        assert!(matches!(
            error,
            SnapshotError::UnknownSnapshotId { snapshot_id } if snapshot_id == unknown
        ));
    }

    #[test]
    fn acquire_lease_for_unknown_snapshot_does_not_allocate_lease_id() {
        let registry = SnapshotRegistry::with_allocator(CountingLeaseAllocator::new());
        let unknown = snapshot_id(203);

        let error = registry
            .acquire_lease(unknown, RetentionReason::PendingWrite)
            .unwrap_err();

        assert!(matches!(
            error,
            SnapshotError::UnknownSnapshotId { snapshot_id } if snapshot_id == unknown
        ));
        assert_eq!(
            registry.allocator.lease_allocations.load(Ordering::Relaxed),
            0
        );
    }

    #[test]
    fn release_lease_for_wrong_snapshot_returns_mismatch_without_changing_counts() {
        let registry = SnapshotRegistry::new();
        let (first_snapshot, first_lease) = registry
            .create_snapshot(request_id(), snapshot_input())
            .unwrap();
        let mut second_input = snapshot_input();
        second_input.source_versions[0].source_hash = hash(222);
        let (second_snapshot, _) = registry
            .create_snapshot(request_id(), second_input)
            .unwrap();

        let error = registry
            .release_lease(second_snapshot.id, first_lease.lease_id)
            .unwrap_err();

        assert!(matches!(
            error,
            SnapshotError::LeaseReleaseMismatch {
                lease_id,
                expected_snapshot,
                actual_snapshot,
            } if lease_id == first_lease.lease_id
                && expected_snapshot == second_snapshot.id
                && actual_snapshot == first_snapshot.id
        ));
        assert_eq!(
            registry.lease_count_for_test(first_snapshot.id, RetentionReason::ActiveBuild),
            1
        );
        assert_eq!(
            registry.lease_count_for_test(second_snapshot.id, RetentionReason::ActiveBuild),
            1
        );

        registry
            .release_lease(first_snapshot.id, first_lease.lease_id)
            .unwrap();

        assert_eq!(
            registry.lease_count_for_test(first_snapshot.id, RetentionReason::ActiveBuild),
            0
        );
        assert_eq!(
            registry.lease_count_for_test(second_snapshot.id, RetentionReason::ActiveBuild),
            1
        );
    }

    #[test]
    fn release_lease_for_unknown_snapshot_returns_unknown_snapshot_id() {
        let registry = SnapshotRegistry::new();
        let (snapshot, lease) = registry
            .create_snapshot(request_id(), snapshot_input())
            .unwrap();
        let unknown = snapshot_id(202);

        let error = registry.release_lease(unknown, lease.lease_id).unwrap_err();

        assert_ne!(snapshot.id, unknown);
        assert!(matches!(
            error,
            SnapshotError::UnknownSnapshotId { snapshot_id } if snapshot_id == unknown
        ));
        assert_eq!(
            registry.lease_count_for_test(snapshot.id, RetentionReason::ActiveBuild),
            1
        );

        registry.release_lease(snapshot.id, lease.lease_id).unwrap();

        assert_eq!(
            registry.lease_count_for_test(snapshot.id, RetentionReason::ActiveBuild),
            0
        );
    }

    #[test]
    fn double_release_returns_unknown_snapshot_lease_without_underflow() {
        let registry = SnapshotRegistry::new();
        let (snapshot, lease) = registry
            .create_snapshot(request_id(), snapshot_input())
            .unwrap();

        registry.release_lease(snapshot.id, lease.lease_id).unwrap();
        let error = registry
            .release_lease(snapshot.id, lease.lease_id)
            .unwrap_err();

        assert!(matches!(
            error,
            SnapshotError::UnknownSnapshotLease { lease_id } if lease_id == lease.lease_id
        ));
        assert_eq!(
            registry.lease_count_for_test(snapshot.id, RetentionReason::ActiveBuild),
            0
        );
    }

    #[test]
    fn unknown_lease_id_returns_unknown_snapshot_lease() {
        let registry = SnapshotRegistry::new();
        let (snapshot, _) = registry
            .create_snapshot(request_id(), snapshot_input())
            .unwrap();
        let ids = InMemorySessionIdAllocator::new();
        let _known_shape_but_not_this_lease = ids.next_lease_id(snapshot.id).unwrap();
        let unknown_lease = ids.next_lease_id(snapshot.id).unwrap();

        let error = registry
            .release_lease(snapshot.id, unknown_lease)
            .unwrap_err();

        assert!(matches!(
            error,
            SnapshotError::UnknownSnapshotLease { lease_id } if lease_id == unknown_lease
        ));
        assert_eq!(
            registry.lease_count_for_test(snapshot.id, RetentionReason::ActiveBuild),
            1
        );
    }

    #[test]
    fn release_lease_after_final_reason_count_removes_the_snapshot_total() {
        let registry = SnapshotRegistry::new();
        let (snapshot, active_build) = registry
            .create_snapshot(request_id(), snapshot_input())
            .unwrap();

        registry
            .release_lease(snapshot.id, active_build.lease_id)
            .unwrap();

        assert_eq!(registry.total_lease_count_for_test(snapshot.id), 0);
        assert_eq!(
            registry.lease_count_for_test(snapshot.id, RetentionReason::ActiveBuild),
            0
        );
        assert_eq!(registry.get(snapshot.id), Some(snapshot));
    }

    #[test]
    fn acquire_lease_id_allocation_failure_does_not_change_registry_state() {
        let registry = SnapshotRegistry::with_allocator(LeaseAllocatorFailsAfter::new(1));
        let request = request_id();
        let (snapshot, _) = registry.create_snapshot(request, snapshot_input()).unwrap();
        let snapshot_before = registry.get(snapshot.id);
        let current_before = registry.is_current_for_request(snapshot.id, request);
        let active_build_count_before =
            registry.lease_count_for_test(snapshot.id, RetentionReason::ActiveBuild);
        let published_count_before =
            registry.lease_count_for_test(snapshot.id, RetentionReason::PublishedLspSnapshot);
        let total_count_before = registry.total_lease_count_for_test(snapshot.id);
        let live_lease_count_before = registry.live_lease_count_for_test(snapshot.id);

        let error = registry
            .acquire_lease(snapshot.id, RetentionReason::PublishedLspSnapshot)
            .unwrap_err();

        assert!(matches!(
            error,
            SnapshotError::LeaseIdAllocation {
                error: IdError::AllocatorOverflow
            }
        ));
        assert_eq!(registry.get(snapshot.id), snapshot_before);
        assert_eq!(
            registry.is_current_for_request(snapshot.id, request),
            current_before
        );
        assert_eq!(
            registry.lease_count_for_test(snapshot.id, RetentionReason::ActiveBuild),
            active_build_count_before
        );
        assert_eq!(
            registry.lease_count_for_test(snapshot.id, RetentionReason::PublishedLspSnapshot),
            published_count_before
        );
        assert_eq!(
            registry.total_lease_count_for_test(snapshot.id),
            total_count_before
        );
        assert_eq!(
            registry.live_lease_count_for_test(snapshot.id),
            live_lease_count_before
        );
    }

    #[test]
    fn retrieved_snapshot_clone_cannot_mutate_registry_snapshot() {
        let registry = SnapshotRegistry::new();
        let request = request_id();
        let (snapshot, _) = registry.create_snapshot(request, snapshot_input()).unwrap();

        let mut retrieved = registry.get(snapshot.id).unwrap();
        retrieved.source_versions.clear();
        retrieved.dependency_artifacts.clear();

        assert_eq!(registry.get(snapshot.id), Some(snapshot));
    }

    #[test]
    fn stale_snapshot_id_is_not_current_for_request() {
        let registry = SnapshotRegistry::new();
        let request = request_id();
        let (snapshot, _) = registry.create_snapshot(request, snapshot_input()).unwrap();
        let stale = snapshot_id(200);

        assert_ne!(snapshot.id, stale);
        assert!(!registry.is_current_for_request(stale, request));
    }

    #[test]
    fn current_snapshot_is_tracked_independently_per_request_generation() {
        let registry = SnapshotRegistry::new();
        let ids = InMemorySessionIdAllocator::new();
        let first_request = ids.next_request_id().unwrap();
        let second_request = ids.next_request_id().unwrap();
        let (first_snapshot, _) = registry
            .create_snapshot(first_request, snapshot_input())
            .unwrap();
        let mut second_input = snapshot_input();
        second_input.source_versions[0].source_hash = hash(98);

        let (second_snapshot, _) = registry
            .create_snapshot(second_request, second_input)
            .unwrap();

        assert_ne!(first_request, second_request);
        assert_ne!(first_snapshot.id, second_snapshot.id);
        assert!(registry.is_current_for_request(first_snapshot.id, first_request));
        assert!(!registry.is_current_for_request(first_snapshot.id, second_request));
        assert!(registry.is_current_for_request(second_snapshot.id, second_request));
        assert!(!registry.is_current_for_request(second_snapshot.id, first_request));
    }

    #[test]
    fn older_snapshot_is_not_current_after_new_snapshot_for_same_request() {
        let registry = SnapshotRegistry::new();
        let request = request_id();
        let (older, _) = registry.create_snapshot(request, snapshot_input()).unwrap();
        let mut newer_input = snapshot_input();
        newer_input.source_versions[0].source_hash = hash(99);

        let (newer, _) = registry.create_snapshot(request, newer_input).unwrap();

        assert_ne!(older.id, newer.id);
        assert!(!registry.is_current_for_request(older.id, request));
        assert!(registry.is_current_for_request(newer.id, request));
        assert_eq!(registry.get(older.id), Some(older));
    }

    #[test]
    fn stale_but_registered_snapshot_can_still_acquire_lease_without_becoming_current() {
        let registry = SnapshotRegistry::new();
        let request = request_id();
        let (older, _) = registry.create_snapshot(request, snapshot_input()).unwrap();
        let mut newer_input = snapshot_input();
        newer_input.source_versions[0].source_hash = hash(100);
        let (newer, _) = registry.create_snapshot(request, newer_input).unwrap();

        let lease = registry
            .acquire_lease(older.id, RetentionReason::PhaseOutputReference)
            .unwrap();

        assert_eq!(lease.snapshot, older.id);
        assert!(!registry.is_current_for_request(older.id, request));
        assert!(registry.is_current_for_request(newer.id, request));
        assert_eq!(
            registry.lease_count_for_test(older.id, RetentionReason::PhaseOutputReference),
            1
        );
        assert_eq!(
            registry.lease_count_for_test(newer.id, RetentionReason::PhaseOutputReference),
            0
        );

        registry.release_lease(older.id, lease.lease_id).unwrap();

        assert!(!registry.is_current_for_request(older.id, request));
        assert!(registry.is_current_for_request(newer.id, request));
        assert_eq!(
            registry.lease_count_for_test(older.id, RetentionReason::PhaseOutputReference),
            0
        );
    }

    #[test]
    fn rejected_snapshot_does_not_allocate_lease_insert_snapshot_or_replace_current() {
        let registry = SnapshotRegistry::with_allocator(CountingLeaseAllocator::new());
        let request = request_id();
        let (accepted, _) = registry.create_snapshot(request, snapshot_input()).unwrap();
        let snapshot_id = snapshot_id(16);
        let ids = InMemorySessionIdAllocator::new();
        let rejected_input = snapshot_input_with_sources(vec![
            source_version(
                ids.next_source_id(snapshot_id).unwrap(),
                "mml",
                "same.module",
                "src/alpha.miz",
                hash(1),
                SourceOrigin::Disk,
            ),
            source_version(
                ids.next_source_id(snapshot_id).unwrap(),
                "mml",
                "same.module",
                "src/beta.miz",
                hash(2),
                SourceOrigin::Disk,
            ),
        ]);
        let rejected_id = rejected_input.build_snapshot_id_unchecked();
        let lease_allocations_before = registry.allocator.lease_allocations.load(Ordering::Relaxed);

        let error = registry
            .create_snapshot(request, rejected_input)
            .unwrap_err();

        assert!(matches!(error, SnapshotError::DuplicateModulePath { .. }));
        assert_eq!(
            registry.allocator.lease_allocations.load(Ordering::Relaxed),
            lease_allocations_before
        );
        assert_eq!(registry.get(rejected_id), None);
        assert!(registry.is_current_for_request(accepted.id, request));
        assert!(!registry.is_current_for_request(rejected_id, request));
    }

    #[test]
    fn unchecked_identity_helpers_do_not_create_registry_snapshots() {
        let registry = SnapshotRegistry::new();
        let request = request_id();
        let mut input = snapshot_input();
        input.toolchain = ToolchainInfo::new(" ");

        let unchecked_id = input.build_snapshot_id_unchecked();
        let unchecked_snapshot = input.clone().build_snapshot_unchecked();
        let error = registry.create_snapshot(request, input).unwrap_err();

        assert_eq!(unchecked_snapshot.id, unchecked_id);
        assert!(matches!(
            error,
            SnapshotError::UnsupportedToolchainMetadata { metadata } if metadata == " "
        ));
        assert_eq!(registry.get(unchecked_id), None);
        assert!(!registry.is_current_for_request(unchecked_id, request));
        assert_eq!(registry.total_lease_count_for_test(unchecked_id), 0);
    }

    #[test]
    fn public_struct_literal_snapshot_is_only_a_detached_record() {
        let registry = SnapshotRegistry::new();
        let request = request_id();
        let snapshot = BuildSnapshot {
            id: snapshot_id(204),
            workspace_root: WorkspaceRoot::new("detached"),
            source_versions: Vec::new(),
            dependency_artifacts: Vec::new(),
            lockfile_hash: zero_hash(),
            toolchain: ToolchainInfo::new(" "),
            verifier_config_hash: zero_hash(),
        };

        let error = registry
            .acquire_lease(snapshot.id, RetentionReason::DiagnosticIndex)
            .unwrap_err();

        assert!(matches!(
            error,
            SnapshotError::UnknownSnapshotId { snapshot_id } if snapshot_id == snapshot.id
        ));
        assert_eq!(registry.get(snapshot.id), None);
        assert!(!registry.is_current_for_request(snapshot.id, request));
        assert_eq!(registry.total_lease_count_for_test(snapshot.id), 0);
    }

    #[test]
    fn lease_id_allocation_failure_does_not_insert_snapshot_or_mark_current() {
        let registry = SnapshotRegistry::with_allocator(LeaseFailingAllocator::new());
        let request = request_id();
        let input = snapshot_input();
        let snapshot_id = input.build_snapshot_id_unchecked();

        let error = registry.create_snapshot(request, input).unwrap_err();

        assert!(matches!(
            error,
            SnapshotError::LeaseIdAllocation {
                error: IdError::AllocatorOverflow
            }
        ));
        assert_eq!(registry.get(snapshot_id), None);
        assert!(!registry.is_current_for_request(snapshot_id, request));
        assert_eq!(registry.total_lease_count_for_test(snapshot_id), 0);
    }

    #[test]
    fn duplicate_module_path_is_rejected_before_snapshot_hashing() {
        let registry = SnapshotRegistry::new();
        let snapshot_id = snapshot_id(14);
        let ids = InMemorySessionIdAllocator::new();
        let duplicate_module = snapshot_input_with_sources(vec![
            source_version(
                ids.next_source_id(snapshot_id).unwrap(),
                "mml",
                "same.module",
                "src/alpha.miz",
                hash(1),
                SourceOrigin::Disk,
            ),
            source_version(
                ids.next_source_id(snapshot_id).unwrap(),
                "mml",
                "same.module",
                "src/beta.miz",
                hash(2),
                SourceOrigin::Disk,
            ),
        ]);

        let error = registry
            .create_snapshot(request_id(), duplicate_module)
            .unwrap_err();

        assert!(matches!(
            error,
            SnapshotError::DuplicateModulePath {
                package_id,
                module_path,
            } if package_id.as_str() == "mml" && module_path.as_str() == "same.module"
        ));
    }

    #[test]
    fn duplicate_source_version_identity_is_rejected_before_snapshot_hashing() {
        let registry = SnapshotRegistry::new();
        let snapshot_id = snapshot_id(15);
        let ids = InMemorySessionIdAllocator::new();
        let duplicate_identity = snapshot_input_with_sources(vec![
            source_version(
                ids.next_source_id(snapshot_id).unwrap(),
                "mml",
                "same.module",
                "src/same.miz",
                hash(7),
                SourceOrigin::Disk,
            ),
            source_version(
                ids.next_source_id(snapshot_id).unwrap(),
                "mml",
                "same.module",
                "src/same.miz",
                hash(7),
                SourceOrigin::OpenBuffer { version: 12 },
            ),
        ]);

        let error = registry
            .create_snapshot(request_id(), duplicate_identity)
            .unwrap_err();

        assert!(matches!(
            error,
            SnapshotError::DuplicateSourceVersionIdentity {
                package_id,
                module_path,
                normalized_path,
                source_hash,
            } if package_id.as_str() == "mml"
                && module_path.as_str() == "same.module"
                && normalized_path.as_str() == "src/same.miz"
                && source_hash == hash(7)
        ));
    }

    #[test]
    fn missing_dependency_artifact_is_rejected_by_creation_validation() {
        let registry = SnapshotRegistry::new();
        let mut input = snapshot_input();
        input.dependency_artifacts[0].artifact = "   ".to_owned();

        let error = registry.create_snapshot(request_id(), input).unwrap_err();

        assert!(matches!(
            error,
            SnapshotError::MissingDependencyArtifact { artifact } if artifact == "   "
        ));
    }

    #[test]
    fn dependency_artifact_with_missing_content_hash_is_rejected_by_creation_validation() {
        let registry = SnapshotRegistry::new();
        let mut input = snapshot_input();
        input.dependency_artifacts[0].content_hash = zero_hash();

        let error = registry.create_snapshot(request_id(), input).unwrap_err();

        assert!(matches!(
            error,
            SnapshotError::MissingDependencyArtifact { artifact }
                if artifact == "kernel/order.vo"
        ));
    }

    #[test]
    fn unsupported_lockfile_metadata_is_rejected_by_creation_validation() {
        let registry = SnapshotRegistry::new();
        let mut input = snapshot_input();
        input.lockfile_hash = zero_hash();

        let error = registry.create_snapshot(request_id(), input).unwrap_err();

        assert!(matches!(
            error,
            SnapshotError::UnsupportedLockfileMetadata { metadata }
                if metadata == "missing-lockfile-hash"
        ));
    }

    #[test]
    fn unsupported_toolchain_metadata_is_rejected_by_creation_validation() {
        let registry = SnapshotRegistry::new();
        let mut input = snapshot_input();
        input.toolchain = ToolchainInfo::new(" ");

        let error = registry.create_snapshot(request_id(), input).unwrap_err();

        assert!(matches!(
            error,
            SnapshotError::UnsupportedToolchainMetadata { metadata } if metadata == " "
        ));
    }

    #[test]
    fn invalid_workspace_root_is_rejected_before_snapshot_registration() {
        let registry = SnapshotRegistry::with_allocator(CountingLeaseAllocator::new());
        let request = request_id();
        let mut input = snapshot_input();
        input.workspace_root = WorkspaceRoot::new(" ");

        let error = registry.create_snapshot(request, input).unwrap_err();

        assert!(matches!(
            error,
            SnapshotError::InvalidWorkspaceRoot { workspace_root }
                if workspace_root.as_str() == " "
        ));
        assert_eq!(
            registry.allocator.lease_allocations.load(Ordering::Relaxed),
            0
        );
    }

    #[test]
    fn blank_package_id_is_rejected_before_snapshot_registration() {
        for invalid_package_id in [" ", "mml core"] {
            let registry = SnapshotRegistry::with_allocator(CountingLeaseAllocator::new());
            let request = request_id();
            let mut input = snapshot_input();
            input.source_versions[0].package_id = PackageId::new(invalid_package_id);

            let error = registry.create_snapshot(request, input).unwrap_err();

            assert!(
                matches!(
                    error,
                    SnapshotError::InvalidPackageId { package_id }
                        if package_id.as_str() == invalid_package_id
                ),
                "{invalid_package_id:?}"
            );
            assert_eq!(
                registry.allocator.lease_allocations.load(Ordering::Relaxed),
                0,
                "{invalid_package_id:?}"
            );
        }
    }

    #[test]
    fn invalid_module_paths_are_rejected_before_duplicate_module_validation() {
        for invalid_module_path in [
            "",
            "groups..basic",
            "bad-name",
            "groups.bad-name.basic",
            "1bad",
            "groups.1bad.basic",
            "theorem",
            "groups.theorem.basic",
        ] {
            let registry = SnapshotRegistry::with_allocator(CountingLeaseAllocator::new());
            let request = request_id();
            let mut input = snapshot_input();
            input.source_versions[0].module_path = ModulePath::new(invalid_module_path);
            input.source_versions[1].module_path = ModulePath::new(invalid_module_path);

            let error = registry.create_snapshot(request, input).unwrap_err();

            assert!(
                matches!(
                    error,
                    SnapshotError::InvalidModulePath {
                        package_id,
                        module_path,
                    } if package_id.as_str() == "mml"
                        && module_path.as_str() == invalid_module_path
                ),
                "{invalid_module_path:?}"
            );
            assert_eq!(
                registry.allocator.lease_allocations.load(Ordering::Relaxed),
                0,
                "{invalid_module_path:?}"
            );
        }
    }

    #[test]
    fn blank_edition_is_rejected_before_snapshot_registration() {
        let registry = SnapshotRegistry::with_allocator(CountingLeaseAllocator::new());
        let request = request_id();
        let mut input = snapshot_input();
        input.source_versions[0].edition = Edition::new(" ");

        let error = registry.create_snapshot(request, input).unwrap_err();

        assert!(matches!(
            error,
            SnapshotError::InvalidEdition {
                package_id,
                module_path,
                edition,
            } if package_id.as_str() == "mml"
                && module_path.as_str() == "beta"
                && edition.as_str() == " "
        ));
        assert_eq!(
            registry.allocator.lease_allocations.load(Ordering::Relaxed),
            0
        );
    }

    #[test]
    fn generated_source_without_metadata_is_rejected_by_snapshot_creation() {
        let registry = SnapshotRegistry::with_allocator(CountingLeaseAllocator::new());
        let request = request_id();
        let mut input = snapshot_input();
        input.source_versions[0].origin = SourceOrigin::Generated {
            generator: GeneratedSourceKind::new(" "),
        };

        let error = registry.create_snapshot(request, input).unwrap_err();

        assert!(matches!(
            error,
            SnapshotError::GeneratedSourceWithoutMetadata { module_path }
                if module_path.as_str() == "beta"
        ));
        assert_eq!(
            registry.allocator.lease_allocations.load(Ordering::Relaxed),
            0
        );
    }

    #[test]
    fn non_negative_open_buffer_version_is_accepted_by_creation_validation() {
        let registry = SnapshotRegistry::new();
        let mut input = snapshot_input();
        input.source_versions[0].origin = SourceOrigin::OpenBuffer { version: 0 };

        let (snapshot, _) = registry.create_snapshot(request_id(), input).unwrap();

        assert!(
            snapshot.source_versions.iter().any(|version| {
                matches!(version.origin, SourceOrigin::OpenBuffer { version: 0 })
            })
        );
    }

    #[test]
    fn negative_open_buffer_version_is_rejected_by_creation_validation() {
        let registry = SnapshotRegistry::new();
        let mut input = snapshot_input();
        input.source_versions[0].origin = SourceOrigin::OpenBuffer { version: -1 };

        let error = registry.create_snapshot(request_id(), input).unwrap_err();

        assert!(matches!(
            error,
            SnapshotError::StaleOpenBufferVersion {
                expected: 0,
                actual: -1
            }
        ));
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
        let invalid_workspace_root = SnapshotError::InvalidWorkspaceRoot {
            workspace_root: WorkspaceRoot::new(" "),
        };
        let invalid_package_id = SnapshotError::InvalidPackageId {
            package_id: PackageId::new(""),
        };
        let invalid_module_path = SnapshotError::InvalidModulePath {
            package_id: PackageId::new("mml"),
            module_path: ModulePath::new(""),
        };
        let invalid_edition = SnapshotError::InvalidEdition {
            package_id: PackageId::new("mml"),
            module_path: ModulePath::new("groups.basic"),
            edition: Edition::new(" "),
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
        let generated_source_without_metadata = SnapshotError::GeneratedSourceWithoutMetadata {
            module_path: ModulePath::new("generated.basic"),
        };
        let unknown_snapshot_id = SnapshotError::UnknownSnapshotId {
            snapshot_id: unknown,
        };
        let lease_release_mismatch = SnapshotError::LeaseReleaseMismatch {
            lease_id,
            expected_snapshot: expected,
            actual_snapshot: actual,
        };
        let unknown_snapshot_lease = SnapshotError::UnknownSnapshotLease { lease_id };
        let duplicate_source_version_identity = SnapshotError::DuplicateSourceVersionIdentity {
            package_id: PackageId::new("mml"),
            module_path: ModulePath::new("groups.basic"),
            normalized_path: normalized_path("src/groups/basic.miz"),
            source_hash: hash(42),
        };
        let lease_id_allocation = SnapshotError::LeaseIdAllocation {
            error: crate::IdError::AllocatorOverflow,
        };

        assert!(matches!(
            invalid_source_path,
            SnapshotError::InvalidSourcePath {
                error: SourcePathError::UnsupportedExtension { .. }
            }
        ));
        assert!(matches!(
            invalid_workspace_root,
            SnapshotError::InvalidWorkspaceRoot { workspace_root }
                if workspace_root.as_str() == " "
        ));
        assert!(matches!(
            invalid_package_id,
            SnapshotError::InvalidPackageId { package_id } if package_id.as_str().is_empty()
        ));
        assert!(matches!(
            invalid_module_path,
            SnapshotError::InvalidModulePath {
                package_id,
                module_path,
            } if package_id.as_str() == "mml" && module_path.as_str().is_empty()
        ));
        assert!(matches!(
            invalid_edition,
            SnapshotError::InvalidEdition {
                package_id,
                module_path,
                edition,
            } if package_id.as_str() == "mml"
                && module_path.as_str() == "groups.basic"
                && edition.as_str() == " "
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
            generated_source_without_metadata,
            SnapshotError::GeneratedSourceWithoutMetadata { module_path }
                if module_path.as_str() == "generated.basic"
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
        assert!(matches!(
            unknown_snapshot_lease,
            SnapshotError::UnknownSnapshotLease {
                lease_id: actual_lease_id,
            } if actual_lease_id == lease_id
        ));
        assert!(matches!(
            duplicate_source_version_identity,
            SnapshotError::DuplicateSourceVersionIdentity {
                package_id,
                module_path,
                normalized_path,
                source_hash,
            } if package_id.as_str() == "mml"
                && module_path.as_str() == "groups.basic"
                && normalized_path.as_str() == "src/groups/basic.miz"
                && source_hash == hash(42)
        ));
        assert!(matches!(
            lease_id_allocation,
            SnapshotError::LeaseIdAllocation {
                error: crate::IdError::AllocatorOverflow,
            }
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

    #[derive(Debug)]
    struct CountingLeaseAllocator {
        inner: InMemorySessionIdAllocator,
        lease_allocations: AtomicUsize,
    }

    impl CountingLeaseAllocator {
        fn new() -> Self {
            Self {
                inner: InMemorySessionIdAllocator::new(),
                lease_allocations: AtomicUsize::new(0),
            }
        }
    }

    impl SessionIdAllocator for CountingLeaseAllocator {
        fn next_session_id(&self) -> Result<BuildSessionId, IdError> {
            self.inner.next_session_id()
        }

        fn next_request_id(&self) -> Result<BuildRequestId, IdError> {
            self.inner.next_request_id()
        }

        fn next_source_id(&self, snapshot: BuildSnapshotId) -> Result<SourceId, IdError> {
            self.inner.next_source_id(snapshot)
        }

        fn next_source_map_id(&self, snapshot: BuildSnapshotId) -> Result<SourceMapId, IdError> {
            self.inner.next_source_map_id(snapshot)
        }

        fn next_lease_id(&self, snapshot: BuildSnapshotId) -> Result<SnapshotLeaseId, IdError> {
            self.lease_allocations.fetch_add(1, Ordering::Relaxed);
            self.inner.next_lease_id(snapshot)
        }
    }

    #[derive(Debug)]
    struct RegistryMutexProbeAllocator {
        inner: InMemorySessionIdAllocator,
        state: AtomicPtr<std::sync::Mutex<SnapshotRegistryState>>,
        registry_mutex_available_checks: AtomicUsize,
    }

    impl RegistryMutexProbeAllocator {
        fn new() -> Self {
            Self {
                inner: InMemorySessionIdAllocator::new(),
                state: AtomicPtr::new(std::ptr::null_mut()),
                registry_mutex_available_checks: AtomicUsize::new(0),
            }
        }

        fn bind_state(&self, state: &std::sync::Mutex<SnapshotRegistryState>) {
            self.state
                .store(std::ptr::from_ref(state).cast_mut(), Ordering::Relaxed);
        }

        fn record_registry_mutex_available(&self) {
            let state = self.state.load(Ordering::Relaxed);
            assert!(
                !state.is_null(),
                "registry mutex probe allocator was not bound to registry state"
            );
            // SAFETY: `bind_state` stores a pointer to this test's registry state,
            // and the registry outlives every allocator call in the test.
            let state = unsafe { &*state };
            match state.try_lock() {
                Ok(_guard) => {
                    self.registry_mutex_available_checks
                        .fetch_add(1, Ordering::Relaxed);
                }
                Err(std::sync::TryLockError::WouldBlock) => {
                    panic!("registry mutex was held during lease id allocation");
                }
                Err(std::sync::TryLockError::Poisoned(_)) => {
                    panic!("snapshot registry mutex poisoned");
                }
            }
        }
    }

    impl SessionIdAllocator for RegistryMutexProbeAllocator {
        fn next_session_id(&self) -> Result<BuildSessionId, IdError> {
            self.inner.next_session_id()
        }

        fn next_request_id(&self) -> Result<BuildRequestId, IdError> {
            self.inner.next_request_id()
        }

        fn next_source_id(&self, snapshot: BuildSnapshotId) -> Result<SourceId, IdError> {
            self.inner.next_source_id(snapshot)
        }

        fn next_source_map_id(&self, snapshot: BuildSnapshotId) -> Result<SourceMapId, IdError> {
            self.inner.next_source_map_id(snapshot)
        }

        fn next_lease_id(&self, snapshot: BuildSnapshotId) -> Result<SnapshotLeaseId, IdError> {
            self.record_registry_mutex_available();
            self.inner.next_lease_id(snapshot)
        }
    }

    #[derive(Debug)]
    struct LeaseFailingAllocator {
        inner: InMemorySessionIdAllocator,
    }

    impl LeaseFailingAllocator {
        fn new() -> Self {
            Self {
                inner: InMemorySessionIdAllocator::new(),
            }
        }
    }

    impl SessionIdAllocator for LeaseFailingAllocator {
        fn next_session_id(&self) -> Result<BuildSessionId, IdError> {
            self.inner.next_session_id()
        }

        fn next_request_id(&self) -> Result<BuildRequestId, IdError> {
            self.inner.next_request_id()
        }

        fn next_source_id(&self, snapshot: BuildSnapshotId) -> Result<SourceId, IdError> {
            self.inner.next_source_id(snapshot)
        }

        fn next_source_map_id(&self, snapshot: BuildSnapshotId) -> Result<SourceMapId, IdError> {
            self.inner.next_source_map_id(snapshot)
        }

        fn next_lease_id(&self, _snapshot: BuildSnapshotId) -> Result<SnapshotLeaseId, IdError> {
            Err(IdError::AllocatorOverflow)
        }
    }

    #[derive(Debug)]
    struct LeaseAllocatorFailsAfter {
        inner: InMemorySessionIdAllocator,
        successful_lease_allocations_remaining: AtomicUsize,
    }

    impl LeaseAllocatorFailsAfter {
        fn new(successful_lease_allocations: usize) -> Self {
            Self {
                inner: InMemorySessionIdAllocator::new(),
                successful_lease_allocations_remaining: AtomicUsize::new(
                    successful_lease_allocations,
                ),
            }
        }
    }

    impl SessionIdAllocator for LeaseAllocatorFailsAfter {
        fn next_session_id(&self) -> Result<BuildSessionId, IdError> {
            self.inner.next_session_id()
        }

        fn next_request_id(&self) -> Result<BuildRequestId, IdError> {
            self.inner.next_request_id()
        }

        fn next_source_id(&self, snapshot: BuildSnapshotId) -> Result<SourceId, IdError> {
            self.inner.next_source_id(snapshot)
        }

        fn next_source_map_id(&self, snapshot: BuildSnapshotId) -> Result<SourceMapId, IdError> {
            self.inner.next_source_map_id(snapshot)
        }

        fn next_lease_id(&self, snapshot: BuildSnapshotId) -> Result<SnapshotLeaseId, IdError> {
            self.successful_lease_allocations_remaining
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |remaining| {
                    remaining.checked_sub(1)
                })
                .map_err(|_| IdError::AllocatorOverflow)?;
            self.inner.next_lease_id(snapshot)
        }
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

    fn all_retention_reasons() -> [RetentionReason; 8] {
        [
            RetentionReason::ActiveBuild,
            RetentionReason::CurrentWatchBaseline,
            RetentionReason::PublishedLspSnapshot,
            RetentionReason::OpenBufferOverlay,
            RetentionReason::DiagnosticIndex,
            RetentionReason::ExplanationRequest,
            RetentionReason::PhaseOutputReference,
            RetentionReason::PendingWrite,
        ]
    }

    fn hash(first_byte: u8) -> Hash {
        let mut bytes = [0; Hash::BYTE_LEN];
        bytes[0] = first_byte;
        Hash::from_bytes(bytes)
    }

    fn zero_hash() -> Hash {
        Hash::from_bytes([0; Hash::BYTE_LEN])
    }

    fn request_id() -> crate::BuildRequestId {
        InMemorySessionIdAllocator::new().next_request_id().unwrap()
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
