# Batch Integration Suite

> Canonical language: English. Japanese companion:
> [../ja/batch_integration.md](../ja/batch_integration.md).

## Purpose

Task 19 covers the build-side integration path that was available before
driver/IR owner seams and real producer publication tokens were consumable by
`mizar-build`:

1. produce a deterministic `BuildPlan`;
2. build a `ModuleIndex` from the plan and a source-layout provider;
3. expand the index into a `TaskGraph`;
4. run the scheduler in batch mode;
5. commit caller-supplied artifact manifest updates through `mizar-artifact`.

The suite is an integration check for existing `mizar-build` boundaries. It is
not a replacement for driver, IR, cache-key construction, dependency
fingerprint construction, proof-reuse validation, or producer artifact
publication integration.

## Gap Classification

| ID | Class | Evidence | Action |
|---|---|---|---|
| BUILD-G-010 | `source_drift` / `test_gap` | Before task 19, planner, module-index, task-graph, scheduler, cache seam, and artifact-commit behavior had focused tests but no single batch fixture covered plan -> graph -> schedule -> commit. | Add an integration test that exercises the available public APIs together. |
| BUILD-G-011 | historical `external_dependency_gap`; task-27 `source_drift` / `test_gap` | Earlier batch integration was entry-point agnostic because driver-owned requests, sessions, phase registry, event stream, and salsa boundaries were outside `mizar-build`. Task 27 adds only the scheduler-selected callback seam for driver consumption. | Keep batch integration entry-point agnostic. Do not add driver-owned APIs or a `mizar-driver` dependency. |
| BUILD-G-012 | `external_dependency_gap` | Real phase output handles and snapshot-handle rehydration are unavailable through a build-owned seam. | Use scheduler-owned synthetic output refs only for scheduler integration. Do not create placeholder IR handles. |
| BUILD-G-013 | `external_dependency_gap` | Real producer artifact publication tokens and full phase-15 emission inputs are not exposed to `mizar-build`. | Commit only caller-supplied `ModuleArtifactEntry` values through `mizar-artifact`; do not mint tokens or fabricate producer authority. |

## Boundary Rules

- Batch integration must use the public `mizar-build` APIs already owned by the
  crate.
- A cache hit is an execution-skip candidate only. The integration suite must
  not treat cache state as semantic acceptance, proof authority, or a trusted
  status upgrade.
- The suite must not reimplement `mizar-cache` `CacheKey`, dependency
  fingerprint, cache-store compatibility, or proof-reuse validation logic.
- Artifact manifest records are publication records, not proof authority. They
  must not promote scheduler outputs, backend results, externally attested
  proofs, or cached data to accepted proof status.
- Manifest commit ordering remains deterministic and independent of worker
  completion order.
- The suite must record unavailable driver, IR, and producer-token integration
  as `external_dependency_gap` rather than using placeholder implementations.

## Test Shape

The task-19 integration fixture should:

- create a small workspace plan with at least two source modules;
- build a module index from a static source layout;
- provide a complete module-dependency overlay for the known source modules;
- schedule the graph with synthetic phase outputs and no driver dependency;
  this covers frontend-shaped task scheduling without real frontend
  phase-service execution because the driver-owned phase-service boundary is
  not present in `mizar-build`;
- collect only completed `ArtifactCommit` tasks into manifest updates;
- publish minimal verified artifacts through `mizar-artifact` test data and
  commit those entries with `commit_manifest_updates`;
- assert deterministic task/result/module ordering, including publication
  order after intentionally shuffled update input.

The fixture may use helper data to build valid `mizar-artifact` records because
artifact schema validation is owned by `mizar-artifact`. Those helpers must not
create a new build authority, producer token, or proof acceptance path inside
`mizar-build`.
