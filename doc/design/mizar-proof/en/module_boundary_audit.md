# mizar-proof Module-Boundary Refactor Gate

> Canonical language: English. Japanese companion:
> [../ja/module_boundary_audit.md](../ja/module_boundary_audit.md).

## Scope

Task 19 audits the implemented `mizar-proof` source layout after the
proof-reuse metadata contract:

- `crates/mizar-proof/src/policy.rs`;
- `crates/mizar-proof/src/selection.rs`;
- `crates/mizar-proof/src/status.rs`;
- `crates/mizar-proof/src/witness_store.rs`;
- crate-private test modules under `crates/mizar-proof/src/<module>/tests.rs`;
- integration/lint tests under `crates/mizar-proof/tests/`;
- paired module specs and the internal module-layout ownership map.

The task is move-only. It must not change public APIs, diagnostics,
deterministic renderings, artifact-facing schemas, cache authority, proof
policy, or proof acceptance behavior.

## Refactor Decision

The audit found that the implementation files were review bottlenecks mainly
because crate-private unit tests were embedded inline below production code.
Those tests need access to private helpers, but they do not need to live in the
same physical file.

Task 19 therefore moves each inline unit-test module into a private child
module:

| Parent module | Production file before | Production file after | Private test module | Boundary result |
|---|---:|---:|---|---|
| `policy` | 2,287 lines | 1,374 lines | `src/policy/tests.rs` | public API unchanged |
| `selection` | 2,679 lines | 1,592 lines | `src/selection/tests.rs` | public API unchanged |
| `status` | 2,433 lines | 1,151 lines | `src/status/tests.rs` | public API unchanged |
| `witness_store` | 2,565 lines | 1,439 lines | `src/witness_store/tests.rs` | public API unchanged |

Each parent now declares `#[cfg(test)] mod tests;`. Rust privacy keeps the
child test modules inside the parent module boundary, so they can continue to
exercise private helpers without exposing new production APIs.

No production helper cluster was moved in this task. The remaining production
sections still align with the paired module specs:

- `policy.rs` owns policy classification, policy fingerprints, external
  evidence admission, and early-stop policy queries;
- `selection.rs` owns evidence candidates, deterministic winner ordering,
  selected reuse metadata, and artifact proof-selection merge;
- `status.rs` owns obligation identity, trusted used-axiom propagation,
  status projection, artifact publication availability, and proof-reuse
  validation metadata;
- `witness_store.rs` owns witness draft staging, manifest-gated publication
  references, and witness provenance checks.

Splitting private production helpers further would create more module
plumbing without a current source/spec boundary win. If task 20 quality review
or downstream integration finds a concrete bottleneck, that can be handled as
a later move-only refactor.

## Source/Spec Recheck

| Area | Recheck | Result |
|---|---|---|
| Public API paths | Public modules remain `policy`, `selection`, `status`, and `witness_store`; public enum policy still points at the same production source files. | consistent |
| Deterministic behavior | Test bodies were moved without changing assertions or fixture logic. | consistent |
| Trust boundary | No source module gained kernel calls, ATP backend execution, cache lookup, artifact manifest commit, or trusted promotion logic. | consistent |
| Reuse metadata | Task-17 selection/status APIs and tests remain in the same parent modules. | consistent |
| Lint guard | `proof_crate_tree_contains_task_nineteen_private_test_modules` allowlist was updated only for private test submodules and its message names task 19. | consistent |

## Bilingual Sync

Task 19 adds this English canonical audit and the Japanese companion with
matching sections, tables, task status, and handoff updates. Japanese docs keep
Rust paths, command names, enum names, and gap ids in English where those are
stable identifiers.

## Gap Classification

| ID | Class | Evidence | Handling |
|---|---|---|---|
| `PROOF19-G001` | `deferred` | Remaining production modules are still over 1,100 lines, but each follows an established module spec and no smaller private helper boundary was identified that would reduce complexity without adding review risk. | Reconsider only if task 20 quality review finds a concrete bottleneck or if a later consumer requires a move-only split. |
| `PROOF19-G002` | resolved `repo_metadata_conflict` | During task 19, the `mizar-atp` task-28 closeout guard still treated the now-formal `crates/mizar-proof` crate as a forbidden placeholder. | Report-only in task 19; resolved later by focused metadata correction commit `36d1a9c` before task-20 closeout. |

## Conclusion

Task 19 reduces review friction by moving large inline test modules into
private child modules. The refactor preserves public APIs, diagnostics,
deterministic outputs, artifact-facing schemas, and trust boundaries. No
placeholder cache/artifact/ATP integration is added.
