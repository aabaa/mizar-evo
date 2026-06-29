# mizar-cache Module Boundary Audit

> Canonical language: English. Japanese companion:
> [../ja/module_boundary_audit.md](../ja/module_boundary_audit.md).

Task 22 audits the `mizar-cache` source layout after the architecture-22
follow-up. It performs one behavior-preserving refactor: inline unit tests are
split into private `#[cfg(test)]` child modules. Runtime implementation code,
public modules, public APIs, diagnostics, deterministic renderings,
artifact-facing schemas, cache lookup policy, proof-reuse validation policy,
proof acceptance policy, and downstream integration status are unchanged.

## Boundary Decision

The public module table remains exactly the one exported by `src/lib.rs`:

- `cache_key`
- `dependency_fingerprint`
- `cache_store`
- `proof_reuse`
- `cluster_db`

No new production module is added. The only new source paths are private test
submodules:

- `src/cache_key/tests.rs`
- `src/dependency_fingerprint/tests.rs`
- `src/cache_store/tests.rs`
- `src/proof_reuse/tests.rs`
- `src/cluster_db/tests.rs`

The moved files retain their previous `use super::*` relationship to the
owning module and are compiled only by tests through `#[cfg(test)] mod tests;`.
They do not expose API, scheduling hooks, `mizar-ir` adapters, publication
tokens, proof-status projection, or kernel authority shortcuts.

## Source Layout Result

| Module | Before task 22 | After task 22 | Decision |
|---|---:|---:|---|
| `cache_key` | 2005 lines inline | 1241 implementation lines plus 759 test lines | Split tests only. |
| `dependency_fingerprint` | 2501 lines inline | 1455 implementation lines plus 1039 test lines | Split tests only. |
| `cache_store` | 2632 lines inline | 1726 implementation lines plus 896 test lines | Split tests only. |
| `proof_reuse` | 1094 lines inline | 602 implementation lines plus 491 test lines | Split tests only. |
| `cluster_db` | 2626 lines inline | 1491 implementation lines plus 1133 test lines | Split tests only. |

The implementation files remain responsible for their module-owned data types,
builders, validators, stores, and deterministic canonicalization. Test helpers
remain colocated under the same module namespace but no longer make production
files the review bottleneck.

## Source/Spec And Bilingual Check

`source_spec_audit.md` is updated to list the private test submodule paths so
the lint guard continues to check every Rust source file. The public module
source paths remain unchanged. The paired English/Japanese design documents
agree on the move-only scope and no new gap IDs are introduced.

The refactor preserves these boundaries:

- cache records and cache hits are optimization inputs only;
- cache records, external evidence, diagnostics, logs, timing metadata, and
  write/arrival order do not become kernel-verified status or trusted
  `used_axioms`;
- proof reuse continues to consume `mizar-proof` validation metadata only;
- scheduler integration, `mizar-ir` adapters, and artifact publication-token
  integration remain existing external dependency gaps.

## Verification Scope

Because Task 22 moves Rust test modules, the required verification is the Rust
source-change set: `cargo fmt --check`, `cargo test -p mizar-cache`,
`cargo clippy -p mizar-cache --all-targets -- -D warnings`, and final broader
verification as required by the crate workflow before completion.

## Conclusion

Task 22 resolves the concrete review bottleneck caused by oversized
implementation files with inline unit tests. It does not change behavior,
public API, authority boundaries, cache semantics, proof-reuse semantics,
cluster-db visibility, or integration readiness.
