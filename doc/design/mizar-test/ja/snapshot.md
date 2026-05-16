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
pub struct SnapshotRecord {
    pub schema_version: SchemaVersion,
    pub test_id: TestCaseId,
    pub kind: SnapshotKind,
    pub profile: SnapshotProfile,
    pub content_hash: Hash,
    pub body: SnapshotBody,
}

pub struct SnapshotProfile {
    pub toolchain: ToolchainInfo,
    pub verifier_config_hash: Hash,
    pub parallelism: ParallelismProfile,
    pub normalize_paths: bool,
}
```

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

## Update Policy

snapshot updates は explicit である。

allowed update reasons:

- intentional schema change
- intentional diagnostic/failure contract change
- architecture と照合済みの intentional semantic behavior change
- fuzz/property reproducer の minimization または promotion

harness は normal test runs 中に snapshots を update してはならない。

## Determinism Checks

snapshot tests は少なくとも次を run する。

1. clean sequential build
2. repeated clean sequential build
3. test profile が support する場合 parallel build

externally visible snapshot hashes は一致しなければならない。

## Tests

key scenarios:

- identical input は identical snapshot hash を生成する
- diagnostic wording-only change は semantic snapshots を invalidate しない
- dependency slice changes は relevant cache/snapshot expectations を invalidate する
- parallel verification は sequential verification と同じ `VerifiedArtifact` snapshot を生成する

## Constraints and Assumptions

- snapshot format は versioned である。
- snapshot hash は canonical content から計算する。
- snapshot tests は regression contracts であり debug dumps ではない。
