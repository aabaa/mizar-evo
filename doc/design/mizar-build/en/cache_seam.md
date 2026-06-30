# mizar-build Cache Seam

> Canonical language: English. Japanese companion:
> [../ja/cache_seam.md](../ja/cache_seam.md).

## Purpose

The cache seam is the `mizar-build` boundary for cache-aware scheduling.
It lets the scheduler consume externally validated cache decisions before a
task is executed. It is an optimization boundary only.

`mizar-cache` owns key construction, dependency fingerprints, cache-store
lookup validation, and proof-reuse validation. `mizar-build` receives only the
post-validation scheduling decision and immutable output references that a
caller is prepared to publish for the current snapshot.

## Context

- [internal 02](../../internal/en/02.artifact_store_cache_key_and_manifest.md)
  "Cache Lookup Before Task Execution"
- [architecture 22](../../architecture/en/22.incremental_verification_contract.md)
- [scheduler.md](./scheduler.md)
- [mizar-cache cache_store.md](../../mizar-cache/en/cache_store.md)

## Scope

`cache_seam` owns:

- caller-supplied task cache decisions keyed by `TaskId`;
- validated cache-hit payloads expressed as scheduler-visible immutable output
  and diagnostic references;
- miss, unavailable, no-key, and error-as-miss outcomes that fall back to
  normal task execution;
- deterministic validation of duplicate cache decisions and decisions for task
  ids absent from the graph at the scheduler input boundary.

`cache_seam` does not own:

- `CacheKey` construction or serialization;
- dependency fingerprint projection;
- proof-reuse validation or proof witness checks;
- cache-store record lookup, insertion, corruption handling, or audit mode;
- phase semantics, semantic acceptance, kernel trust, or trusted-status
  promotion;
- driver sessions, `salsa` queries, `mizar-ir` handles, producer artifact
  publication tokens, or manifest writes.

## Gap Classification

| ID | Class | Evidence | Action |
|---|---|---|---|
| CACHE-SEAM-G001 | `source_drift` / `test_gap` | Before task 18, `scheduler.rs` had a disabled cache policy placeholder and `TaskState::CacheHit`, but no validated-hit input surface or hit-result publication. | Task 18 adds the consumer seam, scheduler integration, and focused tests. |
| CACHE-SEAM-G002 | `external_dependency_gap` | The driver-owned `salsa` query boundary that will call `mizar-cache` is absent because `mizar-driver` is absent. | Accept caller-supplied decisions and do not add a driver dependency or placeholder driver API. |
| CACHE-SEAM-G003 | `external_dependency_gap` | Real sealed output handles and cache-to-IR rehydration are unavailable because `mizar-ir` is absent. | Use synthetic immutable output references in build tests; do not invent IR storage APIs. |
| CACHE-SEAM-G004 | `external_dependency_gap` | Real producer artifact publication tokens are unavailable to `mizar-build`. | A cache hit may record scheduler-visible outputs only; it does not mint publication authority or write artifacts. |

## Data Model

The seam is modeled as a deterministic plan:

```rust
struct CacheSchedulingPlan {
    decisions: Vec<CacheTaskDecision>,
}

struct CacheTaskDecision {
    task_id: TaskId,
    outcome: CacheSchedulingOutcome,
}

enum CacheSchedulingOutcome {
    ValidatedHit(ValidatedCacheHit),
    Miss(CacheFallbackReason),
    NoKey(CacheFallbackReason),
    Unavailable(CacheFallbackReason),
    ErrorAsMiss(CacheFallbackReason),
}

struct ValidatedCacheHit {
    output_refs: Vec<CacheOutputRef>,
    diagnostics: Vec<CacheDiagnosticRef>,
}
```

`ValidatedHit` means the caller has already performed the exact-key,
compatibility, dependency, proof-reuse, and output validation required by
`mizar-cache`. It is not proof evidence. Any other outcome makes the scheduler
run the task normally.

`CacheFallbackReason` values are coarse scheduling labels. Detailed miss
classification remains inside `mizar-cache` and is not reconstructed by
`mizar-build`.

## Scheduler Integration

When a task becomes ready, scheduler cache handling is:

1. Validate cache decision shape at the scheduler input boundary. Duplicate
   decisions and decisions for task ids absent from the graph are diagnostics.
2. If cache scheduling is disabled, ignore valid cache decisions and execute
   normally.
3. If a caller supplied `ValidatedHit`, move the task to `CacheHit`, record the
   validated output references and diagnostics in canonical order, emit the
   normal terminal scheduler event, and do not start execution.
4. If publication freshness rejects the validated hit for the current snapshot,
   record `Cancelled` with no current outputs and block correctness dependents.
5. If the known task has no decision, or its decision is `Miss`, `NoKey`,
   `Unavailable`, `ErrorAsMiss`, or future-compatible miss-like scheduling
   metadata, execute the task normally.

`CacheHit` unblocks dependents exactly like `Completed` for scheduling
purposes. This is only a dependency-scheduling fact. It does not accept a
proof, choose evidence, publish an artifact, or change semantic status.

Validated hits are considered before modeled worker/resource admission.
A cache hit therefore produces no `TaskStarted` event and no execution
resource telemetry for that task. This keeps the hit as an execution-skip
optimization instead of an execution attempt.

## Determinism

Cache decisions are keyed by `TaskId`. Duplicate decisions and decisions for
task ids absent from the graph are input-boundary diagnostics. Validated hit
outputs and diagnostics are sorted and deduplicated before entering
`SchedulerRun`.

For the same `TaskGraph`, scheduler configuration, task outcomes, and cache
decision plan, the run's externally visible records must be deterministic
under different worker counts and completion orders.

## Tests

Task 18 adds focused Rust tests for:

- validated cache hits skipping execution and publishing the same output
  references as the corresponding clean execution;
- cache hits unblocking dependents without creating failure or proof-authority
  records;
- miss, unavailable, no-key, and error-as-miss decisions executing normally;
- disabled cache scheduling ignoring validated hit decisions;
- duplicate cache decisions and decisions for task ids absent from the graph
  failing the scheduler input boundary;
- absence of local cache-key, dependency-fingerprint, proof-reuse, driver, IR,
  publication-token, and proof-authority placeholders.

## Non-Authority Rules

- Cache-aware scheduling is an optimization only.
- A cache hit is a candidate execution skip after external validation, not
  semantic acceptance.
- Cache outputs and artifact records never promote trusted proof status.
- `mizar-build` does not construct cache keys, dependency fingerprints, or
  proof-reuse validation records.
- `mizar-build` does not depend on `mizar-driver`.

## Public Enum Policy

No exhaustive public enum exceptions are owned by this module.

| Enum | Decision |
|---|---|
| `CacheSchedulingOutcome` | `#[non_exhaustive]`; downstream callers must include wildcard match arms. |
| `CacheFallbackReason` | `#[non_exhaustive]`; downstream callers must include wildcard match arms. |
| `CacheSchedulingPlanDiagnosticKind` | `#[non_exhaustive]`; downstream callers must include wildcard match arms. |
