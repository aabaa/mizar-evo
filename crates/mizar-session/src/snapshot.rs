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
    DuplicateLeaseIdAllocation {
        lease_id: SnapshotLeaseId,
        existing_snapshot: BuildSnapshotId,
        allocated_snapshot: BuildSnapshotId,
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
        state.record_lease(lease.clone())?;
        state.snapshots.insert(snapshot.id, snapshot.clone());
        state.current_by_request.insert(request, snapshot.id);

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
        state.record_lease(lease.clone())?;
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
    fn record_lease(&mut self, lease: SnapshotLease) -> Result<(), SnapshotError> {
        if let Some(existing) = self.leases.get(&lease.lease_id) {
            return Err(SnapshotError::DuplicateLeaseIdAllocation {
                lease_id: lease.lease_id,
                existing_snapshot: existing.snapshot,
                allocated_snapshot: lease.snapshot,
            });
        }

        *self
            .lease_counts
            .entry(lease.snapshot)
            .or_default()
            .entry(lease.reason)
            .or_default() += 1;
        self.leases.insert(lease.lease_id, lease);
        Ok(())
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
            Self::DuplicateLeaseIdAllocation {
                lease_id,
                existing_snapshot,
                allocated_snapshot,
            } => {
                write!(
                    f,
                    "duplicate snapshot lease id `{lease_id:?}` allocated for `{allocated_snapshot:?}`; already belongs to `{existing_snapshot:?}`"
                )
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
mod tests;
