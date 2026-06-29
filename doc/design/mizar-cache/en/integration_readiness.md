# Integration Readiness

> Canonical language: English. Japanese companion:
> [../ja/integration_readiness.md](../ja/integration_readiness.md).

Status: task 15 readiness audit. No source integration is implemented because
the owning downstream seams are not ready.

## Purpose

Task 15 evaluates whether `mizar-cache` can be wired into the build scheduler,
IR cache adapter, or artifact committed-publication token flow. It is a
gatekeeping task, not a placeholder-integration task.

The result is deferred integration. The crate-owned cache surfaces are
available for future consumers, but the task does not add scheduler hooks,
`mizar-ir` APIs, artifact publication-token shortcuts, proof status projection,
or publication authority.

## Current Cache Surface

`mizar-cache` currently owns:

- pure `CacheKey` construction;
- dependency-slice and dependency-footprint fingerprints;
- fail-closed cache record and blob storage;
- proof-reuse validation over `mizar-proof` metadata;
- accepted-only cluster-db origins and in-memory import-scoped views.

These surfaces are optimization inputs only. A cache hit may skip work only
after its validation predicates pass. It does not provide proof acceptance,
trusted `used_axioms`, artifact publication, proof winner selection, scheduler
policy, or IR handle reconstruction authority.

## Readiness Inventory

| Integration | Classification | Evidence | Task-15 handling |
|---|---|---|---|
| `mizar-build` cache-aware scheduler | `external_dependency_gap` | `mizar-build` wave B task-graph and scheduler tasks 7-10 are open, and cache-aware scheduling task 18 is open. The task-18 seam must also be consumed through the future driver-owned query boundary. | Do not add scheduler hooks or a cache scheduling trait in `mizar-cache`. Future integration should consume the existing cache APIs after `mizar-build` owns the seam. |
| `mizar-ir` cache adapter | `external_dependency_gap` | `crates/mizar-ir` is absent. `doc/design/mizar-ir/en/todo.md` still has scaffold task 1, cache-adapter spec task 9, and cache-adapter implementation task 10 open. | Do not create a placeholder crate, mock adapter, or rehydration API. Future integration belongs to `mizar-ir` and should validate cache records before reconstructing handles. |
| Artifact committed publication token linkage | `external_dependency_gap` | `mizar-proof` keeps `CommittedWitnessPublicationProof` opaque until `mizar-artifact` exposes an artifact-owned committed publication proof token. The artifact crate does not expose that production token. | Do not link cache validation to artifact publication shortcuts. Cache may compare witness, dependency-artifact, and reuse metadata, but publication remains artifact/proof owned. |

## Deferred Work

When downstream owners are ready, a later task may add integration tests that
show cache hits skip execution while producing byte-identical externally
visible results to a clean build. That future task must still prove:

- cache deletion changes only performance;
- cache hit/miss timing does not affect diagnostics, artifact order, proof
  selection, proof acceptance, or cache publication;
- incomplete dependency footprints, unknown schema/toolchain/policy metadata,
  uncacheable markers, and proof-reuse mismatches fail closed;
- externally attested evidence and cache records never become
  kernel-verified status or trusted `used_axioms`.

Until those owner seams exist, task 15 is complete by recording the
`external_dependency_gap`s and preserving the no-stub boundary.

## Verification

Task 15 is documentation-only. It does not change Rust source, manifests,
tests, `.miz` fixtures, expectations, or traceability metadata. Verification is
documentation review plus diff checks.
