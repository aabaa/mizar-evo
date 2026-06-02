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

impl BuildSnapshot {
    pub fn from_input(input: SnapshotInput) -> Self;
}

impl SnapshotInput {
    pub fn build_snapshot(self) -> BuildSnapshot;
    pub fn build_snapshot_id(&self) -> BuildSnapshotId;
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

impl<A> SnapshotRegistry<A> {
    pub fn with_allocator(allocator: A) -> Self;
}

impl<A: SessionIdAllocator> SnapshotRegistry<A> {
    pub fn create_snapshot(
        &self,
        request: BuildRequestId,
        input: SnapshotInput,
    ) -> Result<(BuildSnapshot, SnapshotLease), SnapshotError>;
    pub fn get(&self, id: BuildSnapshotId) -> Option<BuildSnapshot>;
    pub fn is_current_for_request(&self, id: BuildSnapshotId, request: BuildRequestId) -> bool;
}

// Owned by the snapshot/shared lease layer; re-exported by `retention`.
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
```

The concrete registry may keep snapshots in memory and persist only source/cache-facing fingerprints. Public identifiers are opaque and must not encode paths, timestamps, memory addresses, or task-local counters.

## Dependencies

- Internal: `source_map` for source coordinate tables attached to `SourceVersion`
- External: path normalization, hashing, package metadata, LSP document-version types
- Shared: `SnapshotLease.reason` uses the `RetentionReason` enum defined in this snapshot/shared lease layer and re-exported by the `retention` module

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

### Snapshot Lease

`SnapshotLease` prevents a snapshot from being collected while an external consumer may still reference it.

Lease reasons include:

- active build request;
- watch baseline;
- published LSP snapshot;
- open-buffer overlay;
- diagnostic index;
- explanation request;
- phase-output retention in `mizar-ir`;
- pending cache or artifact writer.

Leases retain snapshot metadata and source maps. They do not by themselves retain all IR outputs; `mizar-ir` owns phase-output retention and may hold its own lease back to the snapshot.

## Algorithm / Logic

### Snapshot Creation

1. Accept already-loaded `SourceVersion` records from the source-loading layer.
2. Validate source identities, dependency artifact references, lockfile metadata, toolchain metadata, and structurally valid open-buffer versions.
3. Sort source and dependency summaries by canonical keys.
4. Hash canonical snapshot input into `BuildSnapshotId`.
5. Insert the immutable snapshot into the registry.
6. Return the snapshot and an active-build lease to the caller.

Disk, open-buffer, and generated-source loading are implemented by later source-loading tasks. This registry records and validates the resulting snapshot input; it does not read source text from disk or editor buffers. Expected-vs-actual open-buffer staleness is checked by the source-loading layer, which owns request document-version metadata.

### Freshness Check

A snapshot is current for a request only when it is the most recent snapshot accepted for that request generation. Watch and LSP builds may keep older snapshots alive for diagnostics and editor display, but older snapshots must not be reported as current build results.

Downstream crates should compare `BuildSnapshotId` before consuming handles. If the ids differ, the consumer must either reject the handle as stale or invoke cache compatibility validation in the responsible cache layer.

### Retention and Collection

The registry may collect a snapshot when:

- no lease references it;
- no current request generation names it;
- no retained source map or diagnostic explanation points to it;
- `mizar-ir` has released phase-output references for that snapshot.

Collection removes in-memory source text and maps unless another layer explicitly stores stable artifact or cache data.

## Error Handling

`SnapshotError` includes:

- invalid or non-normalizable source path;
- duplicate module path in one package snapshot;
- duplicate source-version identity before snapshot hashing;
- missing dependency artifact or content fingerprint referenced by the build plan;
- unsupported lockfile or toolchain metadata;
- stale open-buffer version. During task 11 this is limited to structurally invalid version values in already-loaded snapshot input; expected-vs-actual stale checks are performed by source loading;
- unknown snapshot id;
- lease release mismatch.
- lease id allocation failure.

Source readability and UTF-8 validation diagnostics are produced by the frontend source-loading flow. This module records the resulting source version only after source loading has produced a valid source identity.

## Tests

Key scenarios:

- identical canonical inputs produce the same `BuildSnapshotId`;
- source text changes change the snapshot id;
- dependency artifact hash changes change the snapshot id;
- verifier configuration changes change the snapshot id;
- path normalization prevents duplicate source identities;
- open-buffer versions supersede disk versions only for the targeted LSP request;
- stale `BuildSnapshotId` values are rejected by freshness checks;
- leases keep snapshots alive until all consumers release them;
- collected snapshots cannot be retrieved by `get`.

## Constraints and Assumptions

- Snapshot identity must be deterministic across machines for the same normalized workspace inputs.
- Absolute paths are not included in published artifacts unless explicitly requested for local diagnostics.
- `BuildSnapshotId` is an identity and freshness token, not proof authority.
- Cache reuse across snapshots belongs to `mizar-cache`; this module only provides the inputs needed to validate equivalence.
