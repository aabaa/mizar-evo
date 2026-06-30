# mizar-ir TODO

> Canonical language: English. Japanese companion: [../ja/todo.md](../ja/todo.md).

## Status Legend

- [ ] not started
- [~] in progress
- [x] done

## Module Implementation

Module specs do not exist yet; each is written by its own spec task (English
and Japanese in the same change) before the implementation tasks that cite it.
Module names follow the minimum split of
[internal 07](../../internal/en/07.crate_module_layout.md) (`storage`,
`identity`) plus the publisher/cache-adapter/projection services of internal
06; the crate refines architecture 01 and 18 and internal 06.

| Module | Spec | Source | Status |
|---|---|---|---|
| identity | `identity.md` (task 2) | `src/identity.rs` | [x] |
| storage | `storage.md` (task 4) | `src/storage.rs` | [~] |
| publisher | `publisher.md` (task 7) | `src/publisher.rs` | [x] |
| cache_adapter | `cache_adapter.md` (task 9) | `src/cache_adapter.rs` | [x] |
| projection | `projection.md` (task 11) | `src/projection.rs` | [x] |

`mizar-ir` owns compiler-internal IR storage and snapshot output handles:
immutable storage slots for phase outputs, typed `PhaseOutputRef<T>` handles,
IR-local identity assignment scoped by `mizar-session` `BuildSnapshotId`, the
phase-output publisher that seals outputs, the cache adapter boundary that
converts sealed outputs to cache records and rehydrates handles only from
validated `mizar-cache` records, and the artifact projection boundary that
turns sealed internal IR into `VerifiedArtifactDraft`s. It implements the
resident-set discipline: keep interfaces and indexes resident, spill large
outputs to content-addressed blobs, and collect unreferenced outputs.

Dependency order: `identity` → `storage` → `publisher` → `cache_adapter` /
`projection`.

Each task below is deliberately small — one module spec, or one behavior slice
of one module — so that a single task can be implemented, tested, and
committed autonomously without holding the rest of the crate in flight.

## Crate Prerequisites

The task-1 scaffold depends only on `mizar-session` for snapshot and source
identity. Later projection tasks may add `mizar-artifact` for stable draft
schemas, and later cache-adapter tasks may add `mizar-cache` consumption
through a seam; they must not reimplement `CacheKey`, dependency fingerprints,
or proof-reuse validation. Real pipeline wiring with `mizar-driver` and the
`mizar-build` scheduling wave is an `external_dependency_gap` risk tag until
the owning crates expose real seams; phase services themselves stay independent
of this crate (they receive context handles, not storage internals).
Architecture:
[01.ir_layers.md](../../architecture/en/01.ir_layers.md),
[18.dependency_fingerprint.md](../../architecture/en/18.dependency_fingerprint.md);
internal: [06](../../internal/en/06.ir_storage_and_snapshot_handles.md).

## Resolved And Open Decisions

- **Blob spill thresholds: resolved by task 4.** `storage.md` sets the default
  spill threshold to 64 KiB of canonical payload bytes and keeps the threshold
  as a storage policy, not an identity, proof, cache, or artifact rule. Task 6
  implements the policy and collection behavior.
- **Identity stability under edits: resolved by architecture 01.** IDs are
  deterministic for identical inputs; where edits make perfect stability
  impossible they degrade predictably, and arena indices are never exposed
  as stable API. `identity.md` restates this.
- **Cache authority: resolved by internal 06.** Cache hits are never proof
  authority; the adapter validates before reconstructing handles and
  refuses to upgrade evidence classes.

## Ordered Task List

Keep `cargo test -p mizar-ir` green after each task (see
[Recommended Verification](#recommended-verification)).

### Identity and storage

1. **Crate scaffold and lint-policy guard.** [x]
   - Add the `mizar-ir` workspace member depending only on `mizar-session`;
     add `tests/lint_policy.rs` mirroring the `mizar-frontend` guard.
   - Tests: lint-policy guard passes; workspace builds.
   - Deps: none. Spec: internal 06.

2. **Spec: `identity.md`.** [x]
   - Write the identity spec (English and Japanese, no code):
     consumption of `mizar-session` `BuildSnapshotId` and `SourceId`,
     IR-local per-snapshot id families (`ModuleId`, `ItemId`, `ExprId`,
     `VcId`, `PhaseOutputId`), parent/derived output relationships, and the
     no-reuse-across-incompatible-snapshots rule.
   - Deps: 1. Spec: [internal 06](../../internal/en/06.ir_storage_and_snapshot_handles.md)
     "Snapshot Handle Registry", architecture 01 "Cross-Layer Identity".
     `mizar-ir` consumes `BuildSnapshotId` from `mizar-session`; it does not
     own source/snapshot id construction.

3. **Snapshot handle registry.** [x]
   - Implement the registry with deterministic id assignment and
     parent/derived tracking.
   - Tests: id determinism for identical states; conflicting duplicate
     identity keys rejected; incompatible-snapshot reuse rejected; derived
     links round-trip; IR-local ids are not proof-reuse authority.
   - Deps: 2. Spec: `identity.md`.

4. **Spec: `storage.md`.** [x]
   - Write the storage spec (English and Japanese, no code): immutable
     storage slots, typed `PhaseOutputRef<T>`, sealing semantics,
     in-memory versus blob placement (with the spill-threshold decision),
     and `retain`/`collect` lifetime rules across batch/watch/LSP.
   - Deps: 2. Spec: [internal 06](../../internal/en/06.ir_storage_and_snapshot_handles.md)
     "IR Storage Service".

5. **Storage slots and sealing.** [x]
   - Implement slot allocation, sealing, and typed handle return; sealed
     outputs are immutable and unsealed outputs are invisible to other
     tasks.
   - Tests: double-seal rejected; access before seal fails; handle typing
     round-trips.
   - Deps: 3, 4. Spec: `storage.md`.

6. **Content-addressed blobs and collection.** [x]
   - Implement blob spill per the threshold decision and
     `retain`/`collect` over reference tracking (dependent tasks, LSP
     snapshots, diagnostics, explanation requests, cache writers).
   - Tests: spill round-trips by hash; collect drops exactly the
     unreferenced outputs; retained outputs survive session replacement.
   - Deps: 5. Spec: `storage.md`.

### Publication and adapters

7. **Spec: `publisher.md`.** [x]
   - Write the publisher spec (English and Japanese, no code): snapshot/
     work-unit validation, canonical-encoding content hashes, source-map
     and diagnostic side-table attachment, obsolete-snapshot publication
     rejection, open-buffer output non-publication, and the
     no-partial-IR-exposure rule.
   - Deps: 4. Spec: [internal 06](../../internal/en/06.ir_storage_and_snapshot_handles.md)
     "Phase Output Publisher".

8. **Phase output publisher.** [x]
   - Implement the narrow sealing API used by phase services.
   - Tests: wrong-snapshot or obsolete-snapshot publication rejected; hashes
     stable; side tables retrievable from handles.
   - Deps: 5, 7. Spec: `publisher.md`.

9. **Spec: `cache_adapter.md`.** [x]
   - Write the cache-adapter spec (English and Japanese, no code): which
     outputs are cacheable, record serialization with schema versions and
     dependency summaries, hit validation before handle reconstruction, and
     the no-proof-authority rule. Include the architecture-22 rule that
     incomplete dependency footprints and `uncacheable` markers force a miss
     before any `PhaseOutputRef` is reconstructed.
   - Deps: 7. Spec: [internal 06](../../internal/en/06.ir_storage_and_snapshot_handles.md)
     "IR Cache Adapter", [internal 02](../../internal/en/02.artifact_store_cache_key_and_manifest.md).

10. **Cache adapter.** [x]
    - Implement record conversion and hit rehydration behind a cache seam that
      consumes `mizar-cache` validation results.
    - Tests: round-trip through mock cache; invalid hits, incomplete
      dependency footprints, and `uncacheable` records are rejected;
      tampered payload or side-table hashes miss before sealing;
      rehydrated handles equal originals.
    - Deps: 8, 9. Spec: `cache_adapter.md`.

11. **Spec: `projection.md`.** [x]
    - Write the projection spec (English and Japanese, no code): the
      artifact projection boundary — exported symbols, normalized
      signatures, proof status and witness refs, diagnostics and
      explanation refs — and the prohibition on publishing raw
      `SurfaceAst`/`TypedAst`/`CoreIr`/`ControlFlowIr`/`VcIr`/`AtpProblem`
      or kernel-internal state.
    - Deps: 7. Spec: [internal 06](../../internal/en/06.ir_storage_and_snapshot_handles.md)
      "Artifact Projection Boundary".

12. **Artifact projection service.** [x]
    - Implement projection from sealed outputs into
      `VerifiedArtifactDraft` values using the `mizar-artifact` schemas.
    - Tests: projection fixtures with test-local sealed fixture outputs;
      raw-IR leakage fails the projection.
    - Deps: 8, 11, `mizar-artifact` task 11. Spec: `projection.md`.

13. **Watch/LSP snapshot replacement.** [ ]
   - Implement snapshot replacement: new snapshots supersede old ones
     while retained references keep old outputs alive until released. Old
     outputs may remain readable or become validated cache inputs, but they
     cannot publish as current results after supersession.
   - Tests: replacement fixtures; stale handles stay readable until release,
     then collect; superseded outputs are rejected as current publications but
     remain eligible for cache validation.
    - Deps: 6, 8. Spec: [internal 06](../../internal/en/06.ir_storage_and_snapshot_handles.md)
      "Snapshot Replacement for Watch and LSP".

### Hardening and cross-cutting follow-ups

14. **Determinism and lifetime property suite.** [ ]
    - Property coverage: identical inputs yield identical ids and hashes;
      no use-after-collect; collect is idempotent; reference counting never
      leaks under randomized retain/release sequences.
    - Deps: 13. Spec: [20.test_strategy.md](../../architecture/en/20.test_strategy.md).

15. **Public-enum forward-compatibility policy.** [ ]
    - Apply the `mizar-frontend` task-25 procedure to each public enum.
    - Deps: 12. Spec: all module specs.

16. **Source/spec correspondence audit.** [ ]
    - Trace every public API and promised behavior in the module specs to
      implementation and tests; record gaps as follow-up tasks.
    - Deps: 15. Spec: all module specs and this TODO.

17. **Bilingual documentation sync audit.** [ ]
    - Compare each English canonical document under
      `doc/design/mizar-ir/en/` with its Japanese companion and
      synchronize content.
    - Deps: 16. Spec: repository documentation policy.

18. **Architecture-22 follow-up audit.** [ ]
    - Re-run the source/spec correspondence and bilingual documentation sync
      audits for the publisher, cache-adapter, and snapshot-replacement
      contract added for architecture 22: obsolete outputs cannot publish as
      current, open-buffer outputs do not become package artifacts, and old
      outputs may be used only as validated cache inputs.
    - Deps: 10, 13, 14, 17. Spec: all module specs, this TODO, and repository
      documentation policy.

19. **Module-boundary refactor gate.** [ ]
    - Before treating the crate as ready for downstream consumers, audit the
      source layout for oversized files, mixed responsibilities, and private
      helpers that should be split along the module table and spec boundaries.
      Split any review-bottleneck implementation files into private modules
      without changing public APIs, diagnostics, deterministic renderings,
      artifact-facing schemas, or consumer-visible behavior.
    - After any split, update this module table/source paths as needed and
      re-run the source/spec and bilingual documentation audit scopes for the
      moved APIs. Do not mix behavior cleanup or API exposure into the move;
      those require their own spec tasks.
    - Deps: 18. Spec: this TODO,
      [internal 07](../../internal/en/07.crate_module_layout.md), all module
      specs.

## Recommended Verification

Run after each task:

```text
cargo test -p mizar-ir
cargo clippy -p mizar-ir --all-targets -- -D warnings
```

For projection tasks, also run:

```text
cargo test -p mizar-artifact
```

Check the task off here once tests pass.

## Notes

- Sealed outputs are immutable; partially-built IR is never visible to
  other tasks.
- Cache hits are optimization results, never proof authority; evidence
  classes are never upgraded by rehydration.
- Cache records may contain raw internal IR encodings; published artifacts
  must not — the projection boundary enforces this.
- The resident-set discipline (interfaces resident, bodies lazy, witnesses
  external) is qualitative; budgets and benchmarks live in the test
  strategy.
