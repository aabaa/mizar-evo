# Module: snapshot

> Canonical language: English. Japanese companion: [../ja/snapshot.md](../ja/snapshot.md).

## Purpose

This module defines snapshot artifacts used by `mizar-test`.

Snapshots make deterministic IR, certificate, artifact, dependency, and failure output observable during test-first development.

## Snapshot Kinds

Required snapshot kinds:

- `SurfaceAst`;
- `TypedAst`;
- `CoreIr`;
- `VcIr`;
- SAT clauses;
- `ProofCertificate`;
- `VerifiedArtifact`;
- dependency slices and fingerprints;
- failure records.

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

pub enum SnapshotUpdateReason {
    SchemaChange,
    DiagnosticContractChange,
    SemanticBehaviorChange,
    FuzzPropertyReproducer,
}

pub enum SnapshotUpdateMode {
    VerifyOnly,
    Update { reason: SnapshotUpdateReason },
}

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

## Canonicalization

Snapshots must exclude:

- absolute local paths unless the profile explicitly tests local diagnostics;
- wall-clock timestamps;
- process ids;
- task ids;
- memory addresses;
- backend runtime duration;
- unordered map/set iteration order.

Snapshots must include enough schema and profile data to explain their hash.

The current general snapshot API computes `content_hash` from a canonical
record input containing `schema_version`, `test_id`, `kind`, the complete
`SnapshotProfile`, and the normalized `SnapshotBody`. Including `test_id` is
intentional: the hash identifies a committed snapshot record, not a body-only
digest. The body text normalizes CRLF and CR line endings to LF and records the
canonical byte length so adding or removing a final newline changes the hash.
Toolchain metadata is stored in a sorted map before hashing. Parallel profiles
include their worker count.

Scalar identity and profile fields use length-framed canonical entries, so
metadata such as key `a = b` with value `c` cannot collide with key `a` and
value `b = c`.

Unless `allow_local_paths` is true, validation rejects local absolute paths in
free-form snapshot body text for these supported forms: Unix absolute paths
such as `/tmp/file`, `file:///` URIs, Windows drive paths such as
`C:\tmp\file` or `C:/tmp/file`, and UNC paths such as `\\server\share`.

Snapshot comparison recomputes hashes from the current public record fields
before comparing, then reports the first canonical body-text difference with
expected and actual lines when hashes differ. If the body text is identical but
identity/profile fields differ, comparison falls back to the first canonical
record-text difference. `canonical_text()` reports `StaleContentHash` when a
record's public fields no longer match its stored `content_hash`.

## Update Policy

Snapshot updates are explicit.

Allowed update reasons:

- intentional schema change;
- intentional diagnostic/failure contract change;
- intentional semantic behavior change reviewed against architecture;
- minimizing or promoting a fuzz/property reproducer.

The harness must not update snapshots during normal test runs.

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

Current implemented slice: parser task 38 wires active parse-only `SurfaceAst`
baselines through a transitional sidecar field
`snapshots = "snapshots/parser/<id>.surface_ast.snap"`. This compares committed
`SurfaceAst::snapshot_text()` output byte-for-byte after diagnostics match. It
does not implement the future general snapshot hash registry or update mode.

Current general snapshot API slice: tasks 4 and 5 provide canonical in-memory
`SnapshotRecord` construction, hashing, validation, comparison, explicit
verify/update baseline IO, and repeat-render determinism comparison. General
`[[snapshots]]` sidecar parsing and runner integration remain future harness
work.

## Determinism Checks

Snapshot tests run at least:

1. clean sequential build;
2. repeated clean sequential build;
3. parallel build when the test profile supports it.

The externally visible snapshot hashes must match.

The in-memory determinism helper compares each repeated `SnapshotRecord`
against the first record after recomputing hashes from current public fields.
The first mismatch reports the baseline index, candidate index, and the
underlying `SnapshotMismatch`.

Task 11 also provides `verify_snapshot_parallel_equivalence(sequential,
parallel)`. It normalizes only `ParallelismProfile` before comparing canonical
snapshot records, so worker-count changes alone do not fail equivalence, while
body, identity, toolchain, verifier-config, and path-policy differences still
report ordinary snapshot mismatches.

## Tests

Key scenarios:

- identical input produces identical snapshot hash;
- diagnostic wording-only change does not invalidate semantic snapshots;
- dependency slice changes invalidate relevant cache/snapshot expectations;
- parallel equivalence produces the same `VerifiedArtifact` snapshot as
  sequential verification when only `ParallelismProfile` differs.

## Constraints and Assumptions

- Snapshot format is versioned.
- Snapshot hash is computed from canonical content.
- Snapshot tests are regression contracts, not debug dumps.
