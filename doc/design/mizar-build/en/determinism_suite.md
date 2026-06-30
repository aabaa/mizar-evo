# Determinism Suite

> Canonical language: English. Japanese companion:
> [../ja/determinism_suite.md](../ja/determinism_suite.md).

## Purpose

Task 20 broadens cross-boundary determinism coverage for the `mizar-build`
seams that are implemented today. The suite checks that identical logical
inputs produce stable plans, module indexes, task graphs, clean scheduler
records/events, cache-equivalent public payloads, and artifact manifest commits
even when input order, worker count, ready-queue timing, cache decisions, or
manifest-update arrival order vary.

Task 24 extends this suite with the implemented-seam architecture-22
equivalence gate. It compares clean sequential, clean parallel, incremental
sequential, and incremental parallel scheduler runs by their externally visible
manifest, hash, diagnostic, failure, and blocked-record projections.

The suite is an integration and regression layer over focused module tests. It
does not add driver-owned build sessions, `mizar-ir` handles, cache-key
construction, dependency fingerprint construction, proof-reuse validation, or
producer publication tokens.

## Gap Classification

| ID | Class | Evidence | Action |
|---|---|---|---|
| BUILD-G-014 | `test_gap` | Before task 20, planner, module-index, scheduler, cache seam, and artifact-commit modules had focused determinism tests, but no cross-boundary suite compared the implemented seams as one deterministic pipeline projection. | Add a table-driven integration suite for plan/index/graph/scheduler/commit determinism. |
| BUILD-G-015 | `external_dependency_gap` | Real `mizar-driver` sessions, real `mizar-ir` output handles, producer publication tokens, and full clean/incremental build execution are unavailable. | Keep task 20 on implemented seams and leave full clean/incremental equivalence to later external integration tasks; do not add placeholders. |
| BUILD-G-017 | `external_dependency_gap` | Task 24 covers architecture-22 equivalence for implemented seams, but real driver sessions, real IR output rehydration, producer-owned artifact projection, and producer publication tokens remain unavailable. | Keep the task-24 gate synthetic and build-side until those external seams exist. |

## Boundary Rules

- Clean sequential scheduler runs are the reference for deterministic
  scheduler projections.
- Worker count, synthetic completion order, ready-queue priority hints, and
  manifest-update arrival order must not affect canonical result/event
  collation or artifact publication order for clean implemented-seam runs.
- Cache hit/miss timing may change execution progress and event shape because
  a validated hit skips normal execution. It must not change the matched public
  payload, committed artifact projection, manifest hash, semantic acceptance,
  or proof trust.
- Cache hits remain execution-skip records only. A hit may match clean outputs,
  but it must not become semantic acceptance, proof authority, producer
  publication authority, or trusted-status promotion.
- Clean sequential, clean parallel, incremental sequential, and incremental
  parallel implemented-seam runs over the same snapshot must publish identical
  visible manifest, hash, diagnostic, failure, and blocked-record projections.
- Stale validated hits and superseded snapshots must not publish current
  manifest updates.
- Artifact manifest commits remain serialization boundaries. Manifest updates
  are sorted by the build-side deterministic key before being passed to
  `mizar-artifact`.
- The suite must not reimplement `mizar-cache` `CacheKey`, dependency
  fingerprint, cache-store compatibility, or proof-reuse validation logic.

## Test Shape

Task-20 fixtures should cover:

- shuffled but logically equivalent package/source inputs producing identical
  `BuildPlan`, `ModuleIndex`, and `TaskGraph` values;
- clean scheduler runs with different worker counts, priority hints, and
  completion orders producing identical canonical `SchedulerRun` results and
  events;
- cache-hit and cache-miss decision placement preserving the matched public
  payload and leaving committed artifacts and manifest hashes identical to the
  clean reference when the validated hit carries the same public payload;
- shuffled manifest updates committing identical manifests and build-side
  commit records.
- the architecture-22 implemented-seam equivalence gate for clean sequential,
  clean parallel, incremental sequential, and incremental parallel scheduler
  runs;
- stale validated-hit and superseded-snapshot publication guards.

Unavailable real driver/IR/producer-token paths are recorded as
`external_dependency_gap`; no skipped placeholder APIs should be added for
them.
