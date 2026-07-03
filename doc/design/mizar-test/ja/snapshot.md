# Module: snapshot

> Canonical language: English. English canonical version: [../en/snapshot.md](../en/snapshot.md).

## 目的

この module は `mizar-test` が使う snapshot artifacts を定義する。

snapshots は deterministic IR、certificate、artifact、dependency、failure output を test-first development 中に観測可能にする。

## Snapshot Kinds

required snapshot kinds:

- `SurfaceAst`
- `TypedAst`
- `CoreIr`
- `VcIr`
- SAT clauses
- `ProofCertificate`
- `VerifiedArtifact`
- dependency slices and fingerprints
- failure records

## Public API

```rust
pub struct SchemaVersion(pub u32);

pub struct SnapshotRecord {
    pub schema_version: SchemaVersion,
    pub test_id: TestCaseId,
    pub kind: SnapshotKind,
    pub profile: SnapshotProfile,
    pub content_hash: Hash,
    pub body: SnapshotBody,
}

#[non_exhaustive]
pub enum SnapshotKind {
    SurfaceAst,
    TypedAst,
    CoreIr,
    VcIr,
    SatClauses,
    ProofCertificate,
    VerifiedArtifact,
    DependencySlice,
    DependencyFingerprint,
    FailureRecord,
}

pub struct SnapshotProfile {
    pub toolchain: ToolchainInfo,
    pub verifier_config_hash: Hash,
    pub parallelism: ParallelismProfile,
    pub normalize_paths: bool,
    pub allow_local_paths: bool,
}

pub struct ToolchainInfo {
    pub name: String,
    pub version: String,
    pub metadata: BTreeMap<String, String>,
}

#[non_exhaustive]
pub enum ParallelismProfile {
    Sequential,
    Parallel { workers: u32 },
}

pub struct SnapshotBody {
    pub text: String,
}

pub struct SnapshotMismatch {
    pub expected_hash: Hash,
    pub actual_hash: Hash,
    pub first_difference: Option<SnapshotTextDiff>,
}

pub struct SnapshotTextDiff {
    pub line: usize,
    pub expected: Option<String>,
    pub actual: Option<String>,
}

#[non_exhaustive]
pub enum SnapshotUpdateReason {
    SchemaChange,
    DiagnosticContractChange,
    SemanticBehaviorChange,
    FuzzPropertyReproducer,
}

#[non_exhaustive]
pub enum SnapshotUpdateMode {
    VerifyOnly,
    Update { reason: SnapshotUpdateReason },
}

#[non_exhaustive]
pub enum SnapshotBaselineStatus {
    Matched,
    Created,
    Updated,
}

pub struct SnapshotBaselineReport {
    pub path: PathBuf,
    pub status: SnapshotBaselineStatus,
    pub update_reason: Option<SnapshotUpdateReason>,
}

pub struct SnapshotBaselineMismatch {
    pub expected_hash: Option<Hash>,
    pub actual_hash: Hash,
    pub first_difference: Option<SnapshotTextDiff>,
}

#[non_exhaustive]
pub enum SnapshotBaselineError {
    Snapshot(SnapshotError),
    InvalidBaselinePath { path: PathBuf },
    Io { path: PathBuf, message: String },
    MissingBaseline { path: PathBuf },
    Mismatch {
        path: PathBuf,
        mismatch: Box<SnapshotBaselineMismatch>,
    },
}

pub struct SnapshotDeterminismFailure {
    pub baseline_index: usize,
    pub candidate_index: usize,
    pub mismatch: Box<SnapshotMismatch>,
}

#[non_exhaustive]
pub enum SnapshotError {
    EmptyTestId,
    EmptyToolchainName,
    EmptyToolchainVersion,
    EmptyMetadataKey,
    ParallelWorkerCountZero,
    LocalPath { token: String },
    StaleContentHash { stored: Hash, recomputed: Hash },
}

impl SchemaVersion {
    pub const CURRENT: Self;
}

impl SnapshotRecord {
    pub fn new(
        test_id: TestCaseId,
        kind: SnapshotKind,
        profile: SnapshotProfile,
        body: SnapshotBody,
    ) -> Result<Self, SnapshotError>;

    pub fn canonical_hash_input(&self) -> Result<String, SnapshotError>;
    pub fn recomputed_content_hash(&self) -> Result<Hash, SnapshotError>;
    pub fn canonical_text(&self) -> Result<String, SnapshotError>;
}

impl SnapshotBody {
    pub fn text(text: impl Into<String>) -> Self;
    pub fn canonical_text(&self) -> String;
}

impl ToolchainInfo {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self;
    pub fn with_metadata(
        self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self;
}

impl SnapshotKind {
    pub fn as_str(self) -> &'static str;
}

pub fn compare_snapshot_records(
    expected: &SnapshotRecord,
    actual: &SnapshotRecord,
) -> Result<(), SnapshotMismatch>;

pub fn verify_or_update_snapshot_baseline(
    tests_root: impl AsRef<Path>,
    relative_path: impl AsRef<Path>,
    record: &SnapshotRecord,
    mode: SnapshotUpdateMode,
) -> Result<SnapshotBaselineReport, SnapshotBaselineError>;

pub fn verify_snapshot_determinism(
    records: &[SnapshotRecord],
) -> Result<(), SnapshotDeterminismFailure>;

pub fn verify_snapshot_parallel_equivalence(
    sequential: &SnapshotRecord,
    parallel: &SnapshotRecord,
) -> Result<(), SnapshotMismatch>;
```

## Public Enum Forward Compatibility

task 12 は `mizar-frontend` task 25 の手続きを snapshot enum に適用する。
これらの enum は downstream-visible artifact identity、update policy、failure surface
を表すため、`#[non_exhaustive]` を維持しなければならない。downstream caller は
wildcard match arm を保つ必要がある。crate 内部の match は現在知られている variant
に対して exhaustive のままでよい。

| Public enum | Decision |
|---|---|
| `SnapshotKind` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `ParallelismProfile` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `SnapshotUpdateReason` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `SnapshotUpdateMode` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `SnapshotBaselineStatus` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `SnapshotBaselineError` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `SnapshotError` | `#[non_exhaustive]` downstream forward-compatible surface。 |

この module が所有する exhaustive public enum exception はない。

## Canonicalization

snapshots は次を exclude しなければならない。

- profile が local diagnostics を明示的に test する場合を除く absolute local paths
- wall-clock timestamps
- process ids
- task ids
- memory addresses
- backend runtime duration
- unordered map/set iteration order

snapshots は hash を説明するのに十分な schema and profile data を含める。

現在の general snapshot API は、`schema_version`、`test_id`、`kind`、完全な
`SnapshotProfile`、正規化済み `SnapshotBody` を含む canonical record input から
`content_hash` を計算する。`test_id` を含めるのは意図的である。この hash は
body-only digest ではなく、commit 済み snapshot record を識別する。Body text は
CRLF と CR line ending を LF に正規化し、canonical byte length も記録するため、
final newline の追加・削除は hash を変える。Toolchain metadata は hash 前に sorted
map として扱う。Parallel profile は worker count を含める。

Scalar identity and profile fields は length-framed canonical entries を使う。
そのため、key `a = b` / value `c` の metadata は、key `a` / value `b = c` と
collision しない。

`allow_local_paths` が true でない限り、validation は free-form snapshot body text
内の local absolute paths を reject する。対象 form は Unix absolute path
（例: `/tmp/file`）、`file:///` URI、Windows drive path（例:
`C:\tmp\file` または `C:/tmp/file`）、UNC path（例: `\\server\share`）である。

Snapshot comparison は現在の public record fields から hash を再計算してから比較し、
hash が異なる場合は canonical body text の最初の差分を expected / actual line とともに
report する。Body text が同一で identity/profile fields が異なる場合は、canonical
record text の最初の差分に fallback する。`canonical_text()` は record の public fields
が stored `content_hash` と一致しなくなった場合、`StaleContentHash` を report する。

## Update Policy

snapshot updates は explicit である。

allowed update reasons:

- intentional schema change
- intentional diagnostic/failure contract change
- architecture と照合済みの intentional semantic behavior change
- fuzz/property reproducer の minimization または promotion

harness は normal test runs 中に snapshots を update してはならない。

The general baseline helper enforces that policy with
`SnapshotUpdateMode::VerifyOnly` as the non-mutating default. Verify-only mode
never creates parent directories and never writes baseline files: missing
baselines report `MissingBaseline`, and differing baselines report
`Mismatch` with a precise text diff. Baselines are created or rewritten only
when the caller passes `SnapshotUpdateMode::Update { reason }`, where `reason`
is one of the allowed update reasons above. Update mode creates missing parent
directories, writes the new baseline to a temporary file in the destination
directory, and then renames it into place.

General snapshot baseline paths are clean tests-root-relative `.snap` paths
under `snapshots/`. Paths outside `snapshots/`, absolute paths, parent
traversals, and non-`.snap` files are rejected before any IO.

現在の実装済み slice: parser task 38 は、移行用 sidecar field
`snapshots = "snapshots/parser/<id>.surface_ast.snap"` を通じて active parse-only
`SurfaceAst` baseline を接続する。diagnostics が一致した後、commit 済みの
`SurfaceAst::snapshot_text()` output を byte-for-byte で比較する。これは将来の
一般 snapshot hash registry や update mode を実装するものではない。

現在の general snapshot API slice: task 4 と task 5 は canonical in-memory
`SnapshotRecord` construction、hashing、validation、comparison、explicit
verify/update baseline IO、repeat-render determinism comparison を提供する。
General `[[snapshots]]` sidecar parsing と runner integration は future harness
work として残る。

## Determinism Checks

snapshot tests は少なくとも次を run する。

1. clean sequential build
2. repeated clean sequential build
3. test profile が support する場合 parallel build

externally visible snapshot hashes は一致しなければならない。

The in-memory determinism helper compares each repeated `SnapshotRecord`
against the first record after recomputing hashes from current public fields.
The first mismatch reports the baseline index, candidate index, and the
underlying `SnapshotMismatch`.

task 11 は `verify_snapshot_parallel_equivalence(sequential, parallel)` も提供する。
これは比較前に `ParallelismProfile` だけを normalize するため、worker count の違い
だけでは equivalence failure にならない。一方、body、identity、toolchain、
verifier-config、path-policy の差分は通常の snapshot mismatch として report される。

## Tests

key scenarios:

- identical input は identical snapshot hash を生成する
- diagnostic wording-only change は semantic snapshots を invalidate しない
- dependency slice changes は relevant cache/snapshot expectations を invalidate する
- parallel equivalence は `ParallelismProfile` だけが異なる場合、sequential
  verification と同じ `VerifiedArtifact` snapshot を生成する

## Constraints and Assumptions

- snapshot format は versioned である。
- snapshot hash は canonical content から計算する。
- snapshot tests は regression contracts であり debug dumps ではない。
