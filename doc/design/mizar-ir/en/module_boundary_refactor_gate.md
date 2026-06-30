# mizar-ir Module-Boundary Refactor Gate

> Canonical language: English. Japanese companion:
> [../ja/module_boundary_refactor_gate.md](../ja/module_boundary_refactor_gate.md).

## Scope

This task-19 gate audits `crates/mizar-ir/src` for oversized implementation
files, mixed responsibilities, and private helpers that should be split along
the module table and module specifications before downstream consumers depend
on the crate.

The audit found no source split required in this task. Each implementation
module production file matches one public module and one owning module
specification. `src/lib.rs` remains the crate boundary. Large total line counts
are test-heavy and remain attached to the module whose behavior they exercise.

## Layout Inventory

| Source file | Production lines | Test lines | Total lines | Module boundary result |
|---|---:|---:|---:|---|
| `src/lib.rs` | 32 | 0 | 32 | Crate boundary only. |
| `src/identity.rs` | 1090 | 642 | 1732 | Matches `identity.md`; helpers are identity canonicalization/registry helpers. |
| `src/storage.rs` | 1459 | 903 | 2362 | Matches `storage.md`; slot lifecycle, typed handles, blob placement, side tables, retain/release, and collection are all storage responsibilities. |
| `src/publisher.rs` | 874 | 1009 | 1883 | Matches `publisher.md`; helpers cover currentness, allowed work units, parent validation, canonical hashes, and publish rollback. |
| `src/cache_adapter.rs` | 689 | 905 | 1594 | Matches `cache_adapter.md`; private payload codec helpers remain inside the cache-adapter boundary. |
| `src/projection.rs` | 786 | 844 | 1630 | Matches `projection.md`; sort keys and leakage scanners remain projection-local. |

## Refactor Decision

No file currently crosses ownership boundaries:

- `identity.rs` does not own `BuildSnapshotId` construction, dependency
  fingerprints, proof reuse, or proof authority.
- `storage.rs` is the largest production file, but all private helpers refine
  storage slot/handle/blob/collection behavior described by `storage.md`.
  Storage still does not own current publication, cache validation, artifact
  projection, proof status, or driver/diagnostics integration.
- `publisher.rs` owns publication validation and deterministic hashing, not
  cache-key construction or proof acceptance.
- `cache_adapter.rs` consumes `mizar-cache` lookup/record outcomes and keeps
  private payload encoding inside the adapter; it does not reimplement
  `CacheKey`, dependency fingerprints, or proof-reuse policy.
- `projection.rs` owns the stable artifact projection boundary and leakage
  guard; it does not publish manifests or expose raw IR/kernel/storage state.

Because no source split is performed, this task does not need Rust formatting,
Clippy, or crate tests beyond the crate-closeout verification. If a future
module-boundary audit moves Rust code or tests, that task must re-run
`cargo fmt --check`, `cargo test -p mizar-ir`,
`cargo clippy -p mizar-ir --all-targets -- -D warnings`, and the source/spec
plus bilingual audits for the moved APIs before committing.

## Classified Gaps

No current source-layout-specific `source_drift`, `design_drift`,
`boundary_violation`, `test_expectation_drift`, or `repo_metadata_conflict` was
found in `mizar-ir`.

System-level downstream integration remains the existing
`external_dependency_gap` risk: this gate does not add placeholder
`mizar-driver`, `mizar-diagnostics`, producer-token, artifact-publication-token,
cache-key, dependency-fingerprint, proof-policy, or kernel-acceptance APIs.

## Audit Result

Task 19 closes with no source moves. The module table remains at the public
module level specified by internal 07 and the `mizar-ir` TODO, and all five
implementation modules are ready for crate closeout verification.
