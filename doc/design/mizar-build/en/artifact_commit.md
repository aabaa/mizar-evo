# mizar-build Artifact Commit Boundary

> Canonical language: English. Japanese companion:
> [../ja/artifact_commit.md](../ja/artifact_commit.md).

## Purpose

This document specifies the deterministic artifact-commit boundary owned by
`mizar-build`.

`mizar-build` decides the scheduler-side order in which completed
`ArtifactCommit` tasks are offered to the artifact manifest transaction. It
does not own artifact schemas, stable artifact hashes, proof witness payloads,
or the atomic manifest writer. Those remain `mizar-artifact` responsibilities.

## References

- [scheduler.md](./scheduler.md)
- [internal 01](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md)
- [internal 02](../../internal/en/02.artifact_store_cache_key_and_manifest.md)
- [architecture 11](../../architecture/en/11.artifact_and_incremental_build.md)
- [architecture 14](../../architecture/en/14.parallel_verification_and_scheduling.md)
- [`mizar-artifact` manifest spec](../../mizar-artifact/en/manifest.md)

## Scope

The build-side commit boundary owns:

- deterministic ordering of manifest updates derived from scheduled
  `ArtifactCommit` tasks;
- forwarding caller-supplied module manifest entries to
  `mizar_artifact::manifest::ManifestTransaction`;
- forwarding opaque freshness guards/checks supplied by the build coordinator;
- deterministic records of which module updates were offered to the manifest
  transaction;
- fail-closed propagation of manifest transaction errors.

The build-side commit boundary does not own:

- writing or validating `VerifiedArtifact`, `ModuleSummary`,
  `RegistrationSummary`, proof witness, or development-artifact payload files;
- `mizar-artifact` manifest schema validation, hash construction, reference
  validation, or atomic file replacement;
- producer-owned phase-15 artifact projection;
- proof authority, proof witness acceptance, kernel acceptance, or trusted
  status promotion;
- cache-key construction, dependency-fingerprint construction, proof-reuse
  validation, or cache promotion;
- `mizar-driver` build sessions, event streams, registry dispatch, or `salsa`
  query boundaries.

## Gap Classification

| ID | Class | Evidence | Action |
|---|---|---|---|
| COMMIT-G001 | `source_drift` / `test_gap` | `artifact_commit.md` did not exist before task 17, and `mizar-build` had only modeled scheduler `ArtifactCommit` tasks. | Task 17 adds the build-side manifest-transaction consumer boundary and focused tests. |
| COMMIT-G002 | `external_dependency_gap` | Real producer phase-15 artifact projection and publication tokens are not available to `mizar-build`. | Accept already-written `mizar-artifact` module entries from the caller; do not invent producer payloads or publication tokens. |
| COMMIT-G003 | `external_dependency_gap` | `mizar-driver` sessions, event streams, and `salsa` freshness state are absent in this checkout. | Accept an opaque freshness guard/check; do not depend on `mizar-driver` or add driver-owned APIs. |
| COMMIT-G004 | `external_dependency_gap` | `mizar-ir` sealed output handles and snapshot rehydration are absent. | Do not add IR handle placeholders; tests use already-published artifact files and manifest entries. |

## Data Model

The Rust names may differ, but the build-side boundary has these shapes:

```rust
struct ManifestCommitRequest {
    artifact_root: PathBuf,
    seed_manifest: ArtifactManifest,
    freshness_guard: Option<String>,
    updates: Vec<ScheduledManifestUpdate>,
}

struct ScheduledManifestUpdate {
    task_id: TaskId,
    graph_index: usize,
    entry: ModuleArtifactEntry,
}

struct ManifestCommitSummary {
    manifest: ArtifactManifest,
    manifest_hash: Hash,
    modules: Vec<CommittedModuleUpdate>,
}

struct CommittedModuleUpdate {
    task_id: TaskId,
    graph_index: usize,
    module: ModuleSummaryIdentity,
    source_file: String,
    artifact_file: String,
}
```

`ModuleArtifactEntry`, `ArtifactManifest`, manifest hashes, and the atomic
transaction result come from `mizar-artifact`. `mizar-build` may store those
values, sort them, and pass them through, but it must not reconstruct their
schema semantics.

## Canonical Ordering

Manifest updates are sorted before staging into a transaction. The canonical key
is:

1. stable module identity from `ModuleArtifactEntry.module`;
2. source file path;
3. scheduler graph index;
4. `TaskId`.

This key makes commit staging independent from worker completion order and from
the input order in which the caller reports completed tasks. The manifest
transaction may further sort manifest entries by its own schema rules; that
artifact-owned sort is not reimplemented by `mizar-build`.

Duplicate updates for the same module are accepted only when the artifact-owned
transaction accepts them. A conflicting duplicate remains a manifest error.

## Commit Protocol

1. The caller supplies a package artifact root, seed manifest, optional
   freshness guard, and module updates whose payload files have already been
   written through `mizar-artifact`.
2. `mizar-build` sorts the updates by canonical commit order.
3. `mizar-build` begins a `mizar-artifact` `ManifestTransaction`.
4. `mizar-build` stages each sorted `ModuleArtifactEntry` without rewriting or
   revalidating artifact schema internals.
5. `mizar-build` commits the transaction with the caller-supplied freshness
   check.
6. On success, the returned manifest and module records are sorted
   deterministically and can be reported as build-side commit results.
7. On any manifest error, the error is returned and no build-side artifact or
   proof authority status is promoted.

The manifest transaction owns the atomic visibility rule. If a transaction is
obsolete, interrupted, or fails reference validation, readers continue to use
the previously committed manifest. Unreferenced files are ignored because
readers must start from the manifest.

## Non-Authority Rules

- A committed manifest proves only reachability and hash consistency according
  to `mizar-artifact`; it does not create proof authority.
- `mizar-build` must not treat an artifact record, manifest record, skipped
  task, or cache hit as semantic acceptance or trusted proof status.
- Cache promotion after artifact commit remains out of scope for task 17.
- Missing producer tokens are recorded as `external_dependency_gap`, not
  replaced by synthetic tokens.

## Tests

Task 17 adds focused Rust tests that:

- commit the same already-written module artifacts through shuffled update
  order and observe identical manifests and commit records;
- reject obsolete freshness checks and leave the previous manifest visible;
- propagate conflicting or invalid manifest transaction errors without
  publishing a build-side authority record;
- guard the source boundary against driver, IR, manifest-transaction-internal,
  artifact-schema, hash-construction, proof-witness-validation, cache-key,
  dependency-fingerprint, proof-reuse, proof-authority, and
  producer-publication-token placeholders.

## Public Enum Policy

No exhaustive public enum exceptions are owned by this module.

| Enum | Decision |
|---|---|
| `ArtifactCommitError` | `#[non_exhaustive]`; downstream callers must include wildcard match arms. |
