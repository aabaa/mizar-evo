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
| `mizar-build` cache-aware scheduler | `external_dependency_gap` | `mizar-build` now owns task-graph, scheduler, cache-aware decision, and scheduler-selected dispatch seams. End-to-end cache lookup/use still needs an owner-scoped build/driver integration task that calls `mizar-cache` without moving cache authority into the scheduler. | Do not add scheduler hooks or a driver-owned cache scheduling trait in `mizar-cache`. Future integration should consume existing cache APIs from the owning build/driver path. |
| `mizar-ir` cache adapter | `external_dependency_gap` | `mizar-ir` now exists and owns cache-adapter validation-before-rehydration boundaries. End-to-end rehydration through build/driver execution is not wired in this cache milestone. | Do not move IR handle reconstruction or driver orchestration into `mizar-cache`. Future integration belongs to the owner path and should validate cache records before reconstructing handles. |
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

Until those owner integration tasks wire the existing seams together, task 15
is complete by recording the `external_dependency_gap`s and preserving the
no-stub boundary.

## Verification

Task 15 is documentation-only. It does not change Rust source, manifests,
tests, `.miz` fixtures, expectations, or traceability metadata. Verification is
documentation review plus diff checks.
