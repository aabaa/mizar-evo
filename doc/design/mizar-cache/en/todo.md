# mizar-cache TODO

> Canonical language: English. Japanese companion: [../ja/todo.md](../ja/todo.md).

## Status Legend

- [ ] not started
- [~] in progress
- [x] done

## Module Implementation

Module specs do not exist yet; each is written by its own spec task (English
and Japanese in the same change) before the implementation tasks that cite it.
Module names follow the minimum split of
[internal 07](../../internal/en/07.crate_module_layout.md) (`cache_key`,
`dependency_fingerprint`, `proof_reuse`, `cluster_db`) plus the record/blob
store of internal 02; the crate refines architecture 11, 17, and 18 and
internal 02 and 06.

| Module | Spec | Source | Status |
|---|---|---|---|
| cache_key | `cache_key.md` (task 2) | `src/cache_key.rs` | [ ] |
| dependency_fingerprint | `dependency_fingerprint.md` (task 4) | `src/dependency_fingerprint.rs` | [ ] |
| cache_store | `cache_store.md` (task 7) | `src/cache_store.rs` | [ ] |
| proof_reuse | `proof_reuse.md` (task 10) | `src/proof_reuse.rs` | [ ] |
| cluster_db | `cluster_db.md` (task 12) | `src/cluster_db.rs` | [ ] |

`mizar-cache` owns the internal build cache: canonical `CacheKey`
construction, dependency slices and fingerprints with their rebuild
triggers, the cache record/blob store under `cache_dir`, proof-reuse
validation tied to witness hashes, and the `cluster-db/` indexes with
import-scoped views. The cache is an optimization, never authority: records
may be deleted at any time without changing source-level semantics, hits
must satisfy the same validation rules as a clean build, and a cache record
can never upgrade externally attested evidence to kernel-verified status.

Dependency order: `cache_key` → `dependency_fingerprint` → `cache_store` →
`proof_reuse` / `cluster_db`.

Each task below is deliberately small — one module spec, or one behavior slice
of one module — so that a single task can be implemented, tested, and
committed autonomously without holding the rest of the crate in flight.

## Crate Prerequisites

The crate depends on `mizar-session` and `mizar-artifact` (canonical-hash
rules, interface/implementation hash inputs from its task 16, witness
references). Consumers integrate through seams: the `mizar-build` scheduler
(its task 18), the `mizar-ir` cache adapter (its task 10). Architecture:
[11.artifact_and_incremental_build.md](../../architecture/en/11.artifact_and_incremental_build.md),
[18.dependency_fingerprint.md](../../architecture/en/18.dependency_fingerprint.md),
[17.cluster_trace_format.md](../../architecture/en/17.cluster_trace_format.md);
internal: [02](../../internal/en/02.artifact_store_cache_key_and_manifest.md),
[06](../../internal/en/06.ir_storage_and_snapshot_handles.md).

## Resolved And Open Decisions

- **Key/store split: resolved by internal 02.** `CacheKeyBuilder` only
  produces keys and dependency summaries; compatibility checks and proof
  witness validation decide reuse. Neither grants trust.
- **Slice granularity: open, resolved by task 5.** Architecture 18 names
  theorem, definition, cluster, notation, mode, and attribute slices;
  decide the initial slice granularity actually computed (coarser is
  allowed if rebuild triggers stay conservative) and record it in
  `dependency_fingerprint.md`.
- **Record encoding: open, resolved by task 7.** Decide the cache record
  binary encoding and its `cache_schema_version` evolution rules; cache
  records are internal and may embed raw IR encodings, unlike artifacts.

## Ordered Task List

Keep `cargo test -p mizar-cache` green after each task (see
[Recommended Verification](#recommended-verification)).

### Keys and fingerprints

1. **Crate scaffold and lint-policy guard.** [ ]
   - Add the `mizar-cache` workspace member depending on `mizar-session`
     and `mizar-artifact`; add `tests/lint_policy.rs` mirroring the
     `mizar-frontend` guard.
   - Tests: lint-policy guard passes; workspace builds.
   - Deps: `mizar-artifact` task 3. Spec: internal 02.

2. **Spec: `cache_key.md`.** [ ]
   - Write the cache-key spec (English and Japanese, no code): the
     `CacheKey` fields (phase, work unit, source identity, input/dependency
     hashes, dependency slices, config hash, schema versions, policy
     fingerprint), canonical vector ordering, and the domain-separated
     final hash. Include the architecture-22 validation inputs for verifier
     policy, toolchain/schema compatibility, canonical VC fingerprints,
     local-context fingerprints, and dependency-slice fingerprints.
   - Deps: 1. Spec: [internal 02](../../internal/en/02.artifact_store_cache_key_and_manifest.md)
     "Cache Key".

3. **Cache-key builder.** [ ]
   - Implement `CacheKeyBuilder` as a pure projection from identities,
     hashes, schema versions, and policy — no mutable task state.
   - Tests: key determinism; field-order independence (canonical sort);
     any input change changes the key.
   - Deps: 2. Spec: `cache_key.md`.

4. **Spec: `dependency_fingerprint.md`.** [ ]
   - Write the fingerprint spec (English and Japanese, no code):
     fingerprint targets, dependency slices, stable inputs (and excluded
     nondeterministic inputs), rebuild triggers, and the API-compatibility
     diff per architecture 18, including the slice-granularity decision.
     Specify complete dependency-footprint requirements and the conservative
     `uncacheable` marker used when a footprint cannot be trusted.
   - Deps: 2. Spec:
     [18.dependency_fingerprint.md](../../architecture/en/18.dependency_fingerprint.md).

5. **Dependency-slice and fingerprint computation.** [ ]
   - Compute fingerprints over interface/implementation hash inputs
     (`mizar-artifact` task 16) and per-VC dependency slices
     (`mizar-vc` task 14) at the decided granularity.
   - Tests: fingerprint stability under non-interface edits; slice changes
     flip exactly the dependent fingerprints.
   - Deps: 3, 4, `mizar-artifact` task 16. Spec:
     `dependency_fingerprint.md`.

6. **Rebuild-trigger evaluation.** [ ]
   - Implement the trigger rules: which fingerprint changes invalidate
     which cached phases, conservatively when slices are coarse.
   - Tests: trigger fixtures per change kind (source, import, registration,
     policy, toolchain); no false negatives in the conservative mode.
   - Deps: 5. Spec: `dependency_fingerprint.md` (triggers section).

### Store

7. **Spec: `cache_store.md`.** [ ]
   - Write the store spec (English and Japanese, no code): the
     `.mizar-cache/` layout (records by phase and key, content-addressed
     blobs), `CacheRecordHeader`, compatibility checks, the record-encoding
     decision, `uncacheable` record handling, miss-on-incomplete-footprint
     behavior, miss-on-unknown-compatibility behavior, and deletability
     guarantees.
   - Deps: 2. Spec: [internal 02](../../internal/en/02.artifact_store_cache_key_and_manifest.md)
     "Cache Store"/"Cache Record".

8. **Record store.** [ ]
   - Implement record write/read/validate with header compatibility checks
     (cache schema version, toolchain, output hash).
   - Tests: round-trips; incompatible headers miss instead of erroring;
     corrupted records detected.
   - Deps: 3, 7. Spec: `cache_store.md`.

9. **Blob store.** [ ]
   - Implement content-addressed blob storage for large outputs with
     hash-verified reads and safe concurrent writes.
   - Tests: blob round-trips; hash mismatch detection; concurrent writers
     converge.
   - Deps: 8. Spec: `cache_store.md`.

### Proof reuse and cluster db

10. **Spec: `proof_reuse.md`.** [ ]
    - Write the proof-reuse spec (English and Japanese, no code): reuse
      requires matching `ObligationAnchor`, canonical VC fingerprint,
      local-context fingerprint, dependency-slice fingerprint, selected proof
      witness hash or deterministic discharge hash, dependency artifact
      hashes, policy fingerprint, and schema versions; deterministic built-in
      discharge records; and the rule that records never upgrade evidence
      class.
    - Deps: 7. Spec: [internal 02](../../internal/en/02.artifact_store_cache_key_and_manifest.md),
      [internal 04](../../internal/en/04.atp_portfolio_and_kernel_check_integration.md)
      "Proof Witness and Artifact Flow".

11. **Proof-reuse validation.** [ ]
    - Implement reuse validation over `ProofReuseEvidence`; failures
      degrade to recomputation, never to acceptance.
    - Tests: matching evidence reuses; each mismatched component
      (`ObligationAnchor`, canonical VC fingerprint, local-context
      fingerprint, dependency-slice fingerprint, selected witness hash,
      deterministic discharge hash, policy, schema) blocks reuse; external
      evidence never becomes kernel-verified via cache. Cross-crate producer
      wiring, cache hit/miss timing, and clean/incremental equivalence remain
      the task-20 gate.
    - Deps: 8, 10. Spec: `proof_reuse.md`.

12. **Spec: `cluster_db.md`.** [ ]
    - Write the cluster-db spec (English and Japanese, no code): index
      layout, origin metadata per contribution, import-scoped views,
      invalidation by visible-origin changes, and the rule that unaccepted
      registration proofs never enter importer-visible indexes.
    - Deps: 7. Spec: [internal 02](../../internal/en/02.artifact_store_cache_key_and_manifest.md)
      "Cluster and Registration Cache Update",
      [17.cluster_trace_format.md](../../architecture/en/17.cluster_trace_format.md).

13. **Cluster-db writes and origin tracking.** [ ]
    - Implement contribution writes with origin metadata, stale-origin
      removal, and per-origin index rebuilds.
    - Tests: rename/removal cleans stale origins; rebuilds touch only
      affected origins; unaccepted contributions rejected.
    - Deps: 12, `mizar-checker` task 16. Spec: `cluster_db.md`.

14. **Import-scoped views and invalidation.** [ ]
    - Implement import-scoped views keyed by visible origin sets, with
      invalidation on view-set changes.
    - Tests: view reuse across unrelated changes; visible-origin change
      invalidates exactly the affected views.
    - Deps: 13. Spec: `cluster_db.md`.

### Integration and follow-ups

15. **Scheduler and IR-adapter integration.** [ ]
    - Plug the store into the `mizar-build` cache seam (its task 18) and
      the `mizar-ir` cache adapter (its task 10); cache hits skip work with
      externally identical results.
    - Tests: hit/miss end-to-end fixtures; hit results byte-identical to
      clean-build results.
    - Deps: 8, `mizar-build` task 18, `mizar-ir` task 10. Spec:
      [internal 02](../../internal/en/02.artifact_store_cache_key_and_manifest.md)
      "Cache Lookup Before Task Execution".

16. **Determinism and deletability suite.** [ ]
    - Property coverage: identical inputs produce identical keys and
      records; deleting arbitrary cache subsets never changes build
      results, only build time.
    - Deps: 15. Spec: [20.test_strategy.md](../../architecture/en/20.test_strategy.md).

17. **Public-enum forward-compatibility policy.** [ ]
    - Apply the `mizar-frontend` task-25 procedure to each public enum.
    - Deps: 14. Spec: all module specs.

18. **Source/spec correspondence audit.** [ ]
    - Trace every public API and promised behavior in the module specs to
      implementation and tests; record gaps as follow-up tasks.
    - Deps: 17. Spec: all module specs and this TODO.

19. **Bilingual documentation sync audit.** [ ]
    - Compare each English canonical document under
      `doc/design/mizar-cache/en/` with its Japanese companion and
      synchronize content.
    - Deps: 18. Spec: repository documentation policy.

20. **Incremental verification fail-closed cache contract.** [ ]
    - Implement and test the architecture-22 cache contract across
      `cache_key`, `dependency_fingerprint`, `cache_store`, and `proof_reuse`:
      a reusable output must have a complete dependency slice/footprint, an
      `uncacheable` marker always forces a miss, unknown schema/toolchain
      compatibility forces a miss, and proof reuse validates policy,
      `ObligationAnchor`, canonical VC fingerprint, local-context fingerprint,
      dependency-slice fingerprints, and proof witness or deterministic
      discharge hashes.
    - Tests: each missing or mismatched field independently blocks reuse;
      deleting any cache subset changes only performance; cache hit/miss timing
      cannot change diagnostics, artifact order, or proof acceptance;
      externally attested evidence is never upgraded by a reused record.
    - Deps: 5, 6, 11, 15, 16, `mizar-vc` task 20, `mizar-proof` task 17.
      Spec:
      [22.incremental_verification_contract.md](../../architecture/en/22.incremental_verification_contract.md),
      [11.artifact_and_incremental_build.md](../../architecture/en/11.artifact_and_incremental_build.md),
      [18.dependency_fingerprint.md](../../architecture/en/18.dependency_fingerprint.md).

21. **Architecture-22 follow-up audit.** [ ]
    - Re-run the source/spec correspondence and bilingual documentation sync
      audits for the task-20 cache-key, dependency-footprint, store, and
      proof-reuse contract; record any remaining fail-closed or trust-boundary
      gaps as follow-up tasks.
    - Deps: 20. Spec: all module specs, this TODO, and repository
      documentation policy.

## Recommended Verification

Run after each task:

```text
cargo test -p mizar-cache
cargo clippy -p mizar-cache --all-targets -- -D warnings
```

For integration tasks, also run:

```text
cargo test -p mizar-artifact
cargo test -p mizar-build
cargo test -p mizar-ir
```

For the architecture-22 proof-reuse contract, also run the producers of the
reuse identity and metadata:

```text
cargo test -p mizar-vc
cargo test -p mizar-proof
```

Check the task off here once tests pass.

## Notes

- The cache is optimization, never authority: hits satisfy the same
  validation rules as a clean build, and deleting records never changes
  semantics.
- A cache record can never upgrade externally attested evidence to
  kernel-verified status; proof reuse is tied to accepted witness hashes.
- Keys are pure projections; trust decisions live in compatibility checks
  and proof-reuse validation, not in key construction.
- Cache records are internal and may embed raw IR encodings; published
  artifacts may not (that boundary is `mizar-artifact`/`mizar-ir`'s).
