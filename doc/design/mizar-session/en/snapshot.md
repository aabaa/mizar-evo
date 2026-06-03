# Module: snapshot

> Canonical language: English. Japanese companion: [../ja/snapshot.md](../ja/snapshot.md).

## Purpose

This module defines immutable build snapshot identity for `mizar-session`.

A `BuildSnapshot` identifies the complete build input state observed by one batch, watch, or LSP build request. It covers source versions, dependency artifacts, lockfile state, toolchain identity, and verifier configuration. Downstream crates use `BuildSnapshotId` to reject stale handles and to decide whether cache validation is required before reusing previous outputs.

## Public API

```rust
pub struct BuildSnapshot {
    pub id: BuildSnapshotId,
    pub workspace_root: WorkspaceRoot,
    pub source_versions: Vec<SourceVersion>,
    pub dependency_artifacts: Vec<DependencyArtifactRef>,
    pub lockfile_hash: Hash,
    pub toolchain: ToolchainInfo,
    pub verifier_config_hash: Hash,
}

pub struct SnapshotInput {
    pub workspace_root: WorkspaceRoot,
    pub source_versions: Vec<SourceVersion>,
    pub dependency_artifacts: Vec<DependencyArtifactRef>,
    pub lockfile_hash: Hash,
    pub toolchain: ToolchainInfo,
    pub verifier_config_hash: Hash,
}

pub struct WorkspaceRoot(String);
pub struct DependencyArtifactRef {
    pub artifact: String,
    pub content_hash: Hash,
}
pub struct ToolchainInfo(String);

impl WorkspaceRoot {
    pub fn new(value: impl Into<String>) -> Self;
    pub fn as_str(&self) -> &str;
}

impl DependencyArtifactRef {
    pub fn new(artifact: impl Into<String>, content_hash: Hash) -> Self;
}

impl ToolchainInfo {
    pub fn new(identity: impl Into<String>) -> Self;
    pub fn identity(&self) -> &str;
}

pub struct PackageId(String);
pub struct ModulePath(String);
pub struct Edition(String);
pub struct GeneratedSourceKind(String);

impl PackageId {
    pub fn new(value: impl Into<String>) -> Self;
    pub fn as_str(&self) -> &str;
}

impl ModulePath {
    pub fn new(value: impl Into<String>) -> Self;
    pub fn as_str(&self) -> &str;
}

impl Edition {
    pub fn new(value: impl Into<String>) -> Self;
    pub fn as_str(&self) -> &str;
}

impl GeneratedSourceKind {
    pub fn new(value: impl Into<String>) -> Self;
    pub fn as_str(&self) -> &str;
}

pub struct SourceVersion {
    pub source_id: SourceId,
    pub package_id: PackageId,
    pub module_path: ModulePath,
    pub normalized_path: NormalizedPath,
    pub source_hash: Hash,
    pub edition: Edition,
    pub origin: SourceOrigin,
}

#[non_exhaustive]
pub enum SourceOrigin {
    Disk,
    OpenBuffer { version: LspDocumentVersion },
    Generated { generator: GeneratedSourceKind },
}

impl SourceVersion {
    pub fn canonical_sort_key(&self) -> SourceVersionCanonicalKey<'_>;
}

// Opaque comparison key: package id, module path, normalized path, source hash.
pub struct SourceVersionCanonicalKey<'a> { /* private fields */ }

pub fn sort_source_versions_canonical(source_versions: &mut [SourceVersion]);

pub struct SnapshotRegistry<A = InMemorySessionIdAllocator> { /* private fields */ }

impl SnapshotRegistry<InMemorySessionIdAllocator> {
    pub fn new() -> Self;
}

impl Default for SnapshotRegistry<InMemorySessionIdAllocator> {
    fn default() -> Self;
}

impl<A> SnapshotRegistry<A> {
    pub fn with_allocator(allocator: A) -> Self;
}

impl<A: SessionIdAllocator> SnapshotRegistry<A> {
    pub fn create_snapshot(
        &self,
        request: BuildRequestId,
        input: SnapshotInput,
    ) -> Result<(BuildSnapshot, SnapshotLease), SnapshotError>;
    pub fn acquire_lease(
        &self,
        snapshot: BuildSnapshotId,
        reason: RetentionReason,
    ) -> Result<SnapshotLease, SnapshotError>;
    pub fn release_lease(
        &self,
        snapshot: BuildSnapshotId,
        lease_id: SnapshotLeaseId,
    ) -> Result<(), SnapshotError>;
    pub fn get(&self, id: BuildSnapshotId) -> Option<BuildSnapshot>;
    pub fn is_current_for_request(&self, id: BuildSnapshotId, request: BuildRequestId) -> bool;
}

// Owned by the snapshot/shared lease layer; re-exported by the `retention` module.
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

pub struct SnapshotLease {
    pub lease_id: SnapshotLeaseId,
    pub snapshot: BuildSnapshotId,
    pub reason: RetentionReason,
}

#[non_exhaustive]
pub enum SnapshotError {
    InvalidWorkspaceRoot { workspace_root: WorkspaceRoot },
    InvalidPackageId { package_id: PackageId },
    InvalidModulePath { package_id: PackageId, module_path: ModulePath },
    InvalidEdition { package_id: PackageId, module_path: ModulePath, edition: Edition },
    InvalidSourcePath { error: SourcePathError },
    DuplicateModulePath { package_id: PackageId, module_path: ModulePath },
    DuplicateSourceVersionIdentity {
        package_id: PackageId,
        module_path: ModulePath,
        normalized_path: NormalizedPath,
        source_hash: Hash,
    },
    MissingDependencyArtifact { artifact: String },
    UnsupportedLockfileMetadata { metadata: String },
    UnsupportedToolchainMetadata { metadata: String },
    StaleOpenBufferVersion { expected: LspDocumentVersion, actual: LspDocumentVersion },
    GeneratedSourceWithoutMetadata { module_path: ModulePath },
    UnknownSnapshotId { snapshot_id: BuildSnapshotId },
    LeaseReleaseMismatch {
        lease_id: SnapshotLeaseId,
        expected_snapshot: BuildSnapshotId,
        actual_snapshot: BuildSnapshotId,
    },
    UnknownSnapshotLease { lease_id: SnapshotLeaseId },
    DuplicateLeaseIdAllocation {
        lease_id: SnapshotLeaseId,
        existing_snapshot: BuildSnapshotId,
        allocated_snapshot: BuildSnapshotId,
    },
    LeaseIdAllocation { error: IdError },
}
```

`SnapshotRegistry::create_snapshot` is the public validated construction path for
registry snapshots. Direct input-to-snapshot and input-to-id helpers are
crate-private unchecked identity utilities used by `mizar-session` tests and
internal registry code; they do not validate creation invariants and do not
insert a snapshot into the registry. Because `BuildSnapshot` fields are public,
downstream code may assemble detached snapshot records for copying or tests, but
only snapshots returned by `create_snapshot` are registry snapshots that can be
retrieved, reported current, or leased.

The concrete registry may keep snapshots in memory and persist only source/cache-facing fingerprints. Public identifiers are opaque and must not encode paths, timestamps, memory addresses, or task-local counters.

## Dependencies

- Internal: `source_map` for coordinate tables keyed by `SourceId` and consumed
  alongside `SourceVersion` identity
- External: path normalization, hashing, package metadata, LSP document-version types
- Shared: `SnapshotLease.reason` uses the `RetentionReason` enum defined in this snapshot/shared lease layer. The `retention` module re-exports and reuses this enum rather than redefining it.

This module is consumed by `mizar-build`, `mizar-ir`, `mizar-cache`, `mizar-artifact`, `mizar-diagnostics`, and `mizar-lsp`.

## Data Structures

### Snapshot Identity

`BuildSnapshotId` is derived from canonical snapshot input:

- workspace root identity, provided in normalized form by the caller/source-loading layer;
- sorted source-version summaries;
- dependency artifact identity and content hashes;
- lockfile hash;
- toolchain identity and relevant schema versions;
- verifier configuration hash.

It is not derived from build-session ids, scheduler task ids, wall-clock time, memory addresses, or retention leases.
The source-version summary used for hashing includes package id, module path,
normalized path, source content hash, and edition. It excludes `SourceId` and
source origin metadata so allocator-issued ids and LSP/session-local overlay
details do not affect the content identity.

Two snapshots with identical source text but different dependency artifacts, lockfile state, toolchain identity, or verifier configuration must receive different `BuildSnapshotId` values.

### Source Version

`SourceVersion` is the source-facing unit used by cache keys, artifacts, diagnostics, and LSP overlays.

It records:

- stable source identity within the snapshot;
- package and module identity;
- normalized path relative to the workspace or package root where possible;
- source content hash;
- language edition;
- origin, including open-buffer versions for LSP builds.

`SourceId` is scoped to a snapshot. Published artifacts must project stable source identity through module path, normalized path, and source hash rather than exposing `SourceId` as a compatibility promise.

`WorkspaceRoot`, `PackageId`, `ModulePath`, `Edition`, and `GeneratedSourceKind`
constructors remain infallible boundary wrappers. The upstream build-plan and
source-loading layers should provide normalized, semantically valid values, while
`SnapshotRegistry::create_snapshot` is the final pre-hash guard for registry
snapshots. It rejects a blank `WorkspaceRoot`, blank or whitespace-containing
`PackageId`, empty or language-identifier-invalid `ModulePath` components
(including reserved words), blank `Edition` values, blank generated-source
metadata in manually assembled `SourceVersion` records, duplicate source-version
identities, and duplicate module paths before allocating a lease, registering the
snapshot, or hashing accepted input. The exact nonblank package-name spelling
rule is deferred to the upstream build-plan layer until the package-management
and module-namespace specs are aligned.

### Snapshot Lease

`SnapshotLease` is the snapshot-layer handle that retention collection
uses to keep a snapshot alive while an external consumer may still reference it.
The registry tracks live lease counts per `RetentionReason`; the active-build
lease returned by `create_snapshot` is counted the same way as leases acquired
with `acquire_lease`.

`acquire_lease` rejects unknown snapshots before requesting a lease id. For a
known snapshot, it allocates the lease id outside the registry mutex and records
the resulting lease under the mutex. If lease-id allocation fails, snapshot
records, current marks, live leases, and lease counts remain unchanged.
Although `SessionIdAllocator` is required to issue unique allocator ids, the
registry defensively rejects a duplicate live lease id before mutating its state.
Duplicate lease-id allocation is reported as an internal registry/allocation
error, and snapshot records, current marks, live leases, and lease counts remain
unchanged.

Lease reasons include:

- active build request;
- watch baseline;
- published LSP snapshot;
- open-buffer overlay;
- diagnostic index;
- explanation request;
- phase-output retention in `mizar-ir`;
- pending cache or artifact writer.

Leases currently retain snapshot metadata in the registry accounting. The
retention module bridges the same lease state into source and source-map
retention.
They do not by themselves retain all IR outputs; `mizar-ir` owns phase-output
retention and may hold its own lease back to the snapshot.

## Algorithm / Logic

### Snapshot Creation

1. Accept already-loaded `SourceVersion` records from the source-loading layer.
2. Validate source identities, dependency artifact references, lockfile metadata, toolchain metadata, and structurally valid open-buffer versions.
3. Sort source and dependency summaries by canonical keys.
4. Hash canonical snapshot input into `BuildSnapshotId`.
5. Insert the immutable snapshot into the registry.
6. Return the snapshot and an active-build lease to the caller.

Disk, open-buffer, and generated-source loading are implemented by the source
module. This registry records and validates the resulting snapshot input; it
does not read source text from disk or editor buffers. Expected-vs-actual
open-buffer staleness is checked by the source-loading layer, which owns
request document-version metadata.
Generated-source loading rejects missing generator metadata before allocating a
`SourceId`; snapshot creation repeats that validation for direct `SourceVersion`
inputs so unchecked construction cannot feed blank generated-source metadata into
snapshot identity. Duplicate module paths are always rechecked here as the final
whole-snapshot validation boundary before hashing.

### Freshness Check

A snapshot is current for a request only when it is the most recent snapshot accepted for that request generation. Watch and LSP builds may keep older snapshots alive for diagnostics and editor display, but older snapshots must not be reported as current build results.

Downstream crates should compare `BuildSnapshotId` before consuming handles. If the ids differ, the consumer must either reject the handle as stale or invoke cache compatibility validation in the responsible cache layer.

### Retention and Collection

The current snapshot registry tracks leases and current request state, but it
does not implement collection. The retention module may collect a snapshot when:

- no live lease references it, including diagnostic, LSP, IR, cache, or
  artifact-writer leases;
- no retention current mark names it;

Collection removes in-memory source text and map resources recorded as retention
accounting metadata unless another layer explicitly stores stable artifact or
cache data.

## Error Handling

`SnapshotError` includes:

- blank workspace root identity;
- blank or whitespace-containing package id, invalid module path, or blank edition identity;
- invalid or non-normalizable source path;
- duplicate module path in one package snapshot;
- duplicate source-version identity before snapshot hashing;
- missing dependency artifact or content fingerprint referenced by the build plan;
- unsupported lockfile or toolchain metadata;
- stale open-buffer version. During task 11 this is limited to structurally invalid version values in already-loaded snapshot input; expected-vs-actual stale checks are performed by source loading;
- generated source without required generator metadata in direct snapshot input;
- unknown snapshot id;
- lease release mismatch;
- unknown snapshot lease id, including an already-released lease id;
- duplicate lease id allocated by a custom or registry-aware allocator;
- lease id allocation failure.

Source readability and UTF-8 validation diagnostics are produced by the frontend source-loading flow. This module records the resulting source version only after source loading has produced a valid source identity.
`SnapshotError::InvalidSourcePath` is reserved for snapshot construction paths
that accept or revalidate raw source path descriptors. The current public
`SnapshotRegistry::create_snapshot` API consumes already-normalized
`SourceVersion` records, and public callers cannot construct a malformed
`NormalizedPath` value. Therefore the default registry has no current public
observable path that emits this variant. It remains in the public
non-exhaustive enum so future direct snapshot-construction or registry
revalidation flows can report `SourcePathError` without adding a breaking error
variant. Source-loading path failures that are observable today are reported by
the `source` module, usually through `SourceLoadError::InvalidSourcePath` or one
of its more specific path categories.

## Tests

Key scenarios:

- identical canonical inputs produce the same `BuildSnapshotId`;
- source text changes change the snapshot id;
- dependency artifact hash changes change the snapshot id;
- verifier configuration changes change the snapshot id;
- path normalization prevents duplicate source identities;
- blank workspace roots, blank or whitespace-containing package ids, blank module/edition identities, identifier-invalid or reserved-word module-path components, and generated sources without generator metadata are rejected before snapshot hashing or lease allocation;
- structurally invalid open-buffer versions are rejected; source-loading tasks handle expected-vs-actual staleness and disk/open-buffer override behavior;
- stale `BuildSnapshotId` values are rejected by freshness checks;
- leases are accounted by reason, repeated acquisition produces unique lease ids,
  allocator failure and duplicate lease-id allocation leave registry state
  unchanged, and release reports unknown or mismatched leases;
- direct unchecked helpers are unavailable publicly, and public-field `BuildSnapshot` records are detached until `create_snapshot` registers them.

## Constraints and Assumptions

- Snapshot identity must be deterministic across machines for the same normalized workspace inputs.
- Absolute paths are not included in published artifacts unless explicitly requested for local diagnostics.
- `BuildSnapshotId` is an identity and freshness token, not proof authority.
- Cache reuse across snapshots belongs to `mizar-cache`; this module only provides the inputs needed to validate equivalence.
