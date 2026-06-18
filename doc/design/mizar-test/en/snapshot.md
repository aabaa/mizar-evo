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

Snapshots must exclude:

- absolute local paths unless the profile explicitly tests local diagnostics;
- wall-clock timestamps;
- process ids;
- task ids;
- memory addresses;
- backend runtime duration;
- unordered map/set iteration order.

Snapshots must include enough schema and profile data to explain their hash.

## Update Policy

Snapshot updates are explicit.

Allowed update reasons:

- intentional schema change;
- intentional diagnostic/failure contract change;
- intentional semantic behavior change reviewed against architecture;
- minimizing or promoting a fuzz/property reproducer.

The harness must not update snapshots during normal test runs.

Current implemented slice: parser task 38 wires active parse-only `SurfaceAst`
baselines through a transitional sidecar field
`snapshots = "snapshots/parser/<id>.surface_ast.snap"`. This compares committed
`SurfaceAst::snapshot_text()` output byte-for-byte after diagnostics match. It
does not implement the future general snapshot hash registry or update mode.

## Determinism Checks

Snapshot tests run at least:

1. clean sequential build;
2. repeated clean sequential build;
3. parallel build when the test profile supports it.

The externally visible snapshot hashes must match.

## Tests

Key scenarios:

- identical input produces identical snapshot hash;
- diagnostic wording-only change does not invalidate semantic snapshots;
- dependency slice changes invalidate relevant cache/snapshot expectations;
- parallel verification produces the same `VerifiedArtifact` snapshot as sequential verification.

## Constraints and Assumptions

- Snapshot format is versioned.
- Snapshot hash is computed from canonical content.
- Snapshot tests are regression contracts, not debug dumps.
