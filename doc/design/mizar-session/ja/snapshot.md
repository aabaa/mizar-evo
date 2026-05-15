# Module: snapshot

> Canonical language: English. English canonical version: [../en/snapshot.md](../en/snapshot.md).

## Purpose

この module は `mizar-session` の immutable build snapshot identity を定義する。

`BuildSnapshot` は一つの batch、watch、LSP build request が観測する complete build input state を識別する。Source versions、dependency artifacts、lockfile state、toolchain identity、verifier configuration を含む。Downstream crates は `BuildSnapshotId` を使い、stale handles を拒否し、previous outputs を reuse する前に cache validation が必要かどうかを判断する。

## Public API

```rust
pub struct BuildSnapshot {
    pub id: BuildSnapshotId,
    pub workspace_root: WorkspaceRoot,
    pub sources: Vec<SourceVersion>,
    pub dependency_artifacts: Vec<DependencyArtifactRef>,
    pub lockfile_hash: Hash,
    pub toolchain: ToolchainInfo,
    pub verifier_config_hash: Hash,
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

pub struct SnapshotLease {
    pub snapshot: BuildSnapshotId,
    pub reason: RetentionReason,
}

pub trait SnapshotRegistry {
    fn create_snapshot(&self, input: SnapshotInput) -> Result<BuildSnapshot, SnapshotError>;
    fn get(&self, id: BuildSnapshotId) -> Option<BuildSnapshotRef>;
    fn acquire_lease(&self, id: BuildSnapshotId, reason: RetentionReason) -> Result<SnapshotLease, SnapshotError>;
    fn release_lease(&self, lease: SnapshotLease);
    fn is_current_for_request(&self, id: BuildSnapshotId, request: BuildRequestId) -> bool;
}
```

Concrete registry は snapshots を memory に保持し、source/cache-facing fingerprints だけを persist してよい。Public identifiers は opaque であり、paths、timestamps、memory addresses、task-local counters を encode してはならない。

## Dependencies

- Internal: `SourceVersion` に attach される source coordinate tables のための `source_map`
- External: path normalization、hashing、package metadata、LSP document-version types

この module は `mizar-build`、`mizar-ir`、`mizar-cache`、`mizar-artifact`、`mizar-diagnostics`、`mizar-lsp` から consume される。

## Data Structures

### Snapshot Identity

`BuildSnapshotId` は canonical snapshot input から導出される。

- normalization 後の workspace root identity
- sorted source-version summaries
- dependency artifact identity and content hashes
- lockfile hash
- toolchain identity and relevant schema versions
- verifier configuration hash

Build-session ids、scheduler task ids、wall-clock time、memory addresses、retention leases からは導出しない。

同じ source text でも dependency artifacts、lockfile state、toolchain identity、verifier configuration が異なる場合、別の `BuildSnapshotId` を受け取らなければならない。

### Source Version

`SourceVersion` は cache keys、artifacts、diagnostics、LSP overlays が使う source-facing unit である。

記録するもの:

- snapshot 内の stable source identity
- package and module identity
- 可能な場合は workspace or package root からの normalized path
- source content hash
- language edition
- LSP builds 用の open-buffer versions を含む origin

`SourceId` は snapshot に scope される。Published artifacts は `SourceId` を compatibility promise として expose せず、module path、normalized path、source hash を通して stable source identity を project しなければならない。

### Snapshot Lease

`SnapshotLease` は external consumer が snapshot を参照している可能性がある間、その snapshot が collect されることを防ぐ。

Lease reasons:

- active build request
- watch baseline
- published LSP snapshot
- diagnostic index
- explanation request
- `mizar-ir` における phase-output retention
- pending cache or artifact writer

Leases は snapshot metadata and source maps を retain する。すべての IR outputs をそれだけで retain するわけではない。`mizar-ir` が phase-output retention を所有し、snapshot への lease を別途持ってよい。

## Algorithm / Logic

### Snapshot Creation

1. Workspace、package、source paths を normalize する。
2. Request が選択した disk files、open buffers、generated sources から `SourceVersion` records を作る。
3. Source and dependency summaries を canonical keys で sort する。
4. Canonical snapshot input を hash して `BuildSnapshotId` を作る。
5. Immutable snapshot を registry に insert する。
6. Snapshot と active-build lease を caller に返す。

### Freshness Check

Snapshot は、その request generation に accepted された most recent snapshot である場合だけ current である。Watch and LSP builds は diagnostics and editor display のため older snapshots を alive に保ってよいが、older snapshots を current build results として報告してはならない。

Downstream crates は handles を consume する前に `BuildSnapshotId` を比較するべきである。Ids が異なる場合、consumer は handle を stale として reject するか、responsible cache layer の cache compatibility validation を呼び出さなければならない。

### Retention and Collection

Registry は次の条件を満たす snapshot を collect してよい。

- lease が参照していない
- current request generation が名前を持っていない
- retained source map or diagnostic explanation が指していない
- `mizar-ir` がその snapshot の phase-output references を release 済み

Collection は in-memory source text and maps を remove する。ただし、別 layer が stable artifact or cache data として明示的に保存したものは除く。

## Error Handling

`SnapshotError` includes:

- invalid or non-normalizable source path
- one package snapshot 内の duplicate module path
- build plan が参照する missing dependency artifact
- unsupported lockfile or toolchain metadata
- stale open-buffer version
- unknown snapshot id
- lease release mismatch

Source readability and UTF-8 validation diagnostics は frontend source-loading flow が produce する。この module は source loading が valid source identity を produce した後でのみ resulting source version を record する。

## Tests

Key scenarios:

- identical canonical inputs produce the same `BuildSnapshotId`
- source text changes change the snapshot id
- dependency artifact hash changes change the snapshot id
- verifier configuration changes change the snapshot id
- path normalization prevents duplicate source identities
- open-buffer versions supersede disk versions only for the targeted LSP request
- stale `BuildSnapshotId` values are rejected by freshness checks
- leases keep snapshots alive until all consumers release them
- collected snapshots cannot be retrieved by `get`

## Constraints and Assumptions

- Snapshot identity must be deterministic across machines for the same normalized workspace inputs.
- Absolute paths are not included in published artifacts unless explicitly requested for local diagnostics.
- `BuildSnapshotId` is an identity and freshness token, not proof authority.
- Cache reuse across snapshots belongs to `mizar-cache`; this module only provides the inputs needed to validate equivalence.
