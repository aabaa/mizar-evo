# Incremental/Parallel Equivalence Gate

> Canonical language: English. Japanese companion:
> [../ja/incremental_parallel_equivalence.md](../ja/incremental_parallel_equivalence.md).

Status: task 24 implemented-seam gate complete.

## Purpose

Task 24 adds the scheduler-level architecture-22 regression gate for the
`mizar-build` seams implemented in this checkout. The gate treats clean
sequential execution as the reference projection and checks that clean
parallel, incremental sequential, and incremental parallel runs over the same
`BuildSnapshotId` and synthetic verifier/artifact policy publish the same
externally visible results.

The gate is intentionally limited to build-side planning, task-graph,
scheduler, cache seam, cancellation, and artifact-commit boundaries that
already exist in `mizar-build`. It does not add driver sessions, `mizar-ir`
output handles, cache-key construction, dependency fingerprints, proof-reuse
validation, producer artifact publication tokens, or proof authority.

## Scope

The implemented-seam equivalence projection compares:

- committed manifest hash;
- package/module identity, source file, artifact file, source hash, artifact
  hash, interface hash, implementation hash, module-summary references,
  registration-summary references, proof-witness entries, and diagnostics hash
  for published module entries;
- scheduler-visible output references and payload labels;
- canonical scheduler diagnostics and result diagnostics;
- failure and blocked task records.

Task execution state is not itself an externally visible semantic projection:
an incremental run may record `CacheHit` where the clean reference records
`Completed`, but the visible manifest, hashes, proof-witness references, and
canonical diagnostics must still match. Cache misses enqueue normal work and
must not perturb the deterministic commit boundary.

The gate also checks stale-publication behavior. A validated hit discarded at
the completed-before-publication checkpoint must not publish that stale module
or downstream dependents as current artifacts. A superseded snapshot must
publish no current manifest updates.

## Gap Classification

| ID | Class | Evidence | Action |
|---|---|---|---|
| BUILD-G-017 | `external_dependency_gap` | Task 24 covers architecture-22 equivalence only for implemented `mizar-build` seams. Real `mizar-driver` build sessions, real `mizar-ir` output rehydration, producer-owned artifact projection, and producer publication tokens remain unavailable. | Keep the gate synthetic and build-side. Do not add placeholder driver, IR, producer-token, cache-key, dependency-fingerprint, proof-reuse, or proof-authority APIs. Full real end-to-end equivalence belongs to later external integration once those seams exist. |

BUILD-G-016 remains a non-blocking `test_gap` for direct standalone
`sorted_manifest_updates` helper coverage. Task 24 does not close that
artifact-commit hardening gap.

## Test Evidence

Task 24 extends `crates/mizar-build/tests/determinism_suite.rs` with:

- `clean_and_incremental_parallel_runs_publish_identical_visible_projection`,
  comparing clean sequential, clean parallel, incremental sequential, and
  incremental parallel runs over the same task graph and snapshot;
- `superseded_or_stale_incremental_results_do_not_publish_current_artifacts`,
  checking stale validated hits and snapshot supersession before manifest
  publication.

The tests use caller-supplied cache decisions. Validated hits are modeled only
as execution skips after external validation; misses fall back to normal
execution. The projection includes artifact-facing hashes, summary references,
and proof-witness entries but does not create proof evidence.

## Non-Authority Rules

- Cache-aware scheduling remains an optimization-only execution skip.
- A cache hit never becomes semantic acceptance, proof authority, producer
  publication authority, or trusted-status promotion.
- Artifact records and manifest commits do not promote proof trust.
- Worker completion order and cache hit/miss timing do not determine canonical
  diagnostics, published artifact order, interface hashes, dependency-facing
  summaries, or proof acceptance.
- Missing real driver, IR, and producer-token integrations remain
  `external_dependency_gap`.

## Handoff

Task 25 should re-run the source/spec correspondence and bilingual
documentation audits against this gate. In that follow-up, include this file,
the updated determinism suite, and the unchanged external dependency gaps in
the audit inputs.
