# mizar-cache TODO

> Canonical language: English. Japanese companion: [../ja/todo.md](../ja/todo.md).

## Status Legend

- [ ] not started
- [~] in progress
- [x] done

## Module Implementation

Module specs are written by their own spec tasks (English and Japanese in the
same change) before the implementation tasks that cite them. Module names
follow the minimum split of
[internal 07](../../internal/en/07.crate_module_layout.md) (`cache_key`,
`dependency_fingerprint`, `proof_reuse`, `cluster_db`) plus the record/blob
store of internal 02; the crate refines architecture 11, 17, and 18 and
internal 02 and 06.

| Module | Spec | Source | Status |
|---|---|---|---|
| cache_key | `cache_key.md` (task 2) | `src/cache_key.rs` | [x] |
| dependency_fingerprint | `dependency_fingerprint.md` (task 4) | `src/dependency_fingerprint.rs` | [x] |
| cache_store | `cache_store.md` (task 7) | `src/cache_store.rs` | [x] |
| proof_reuse | `proof_reuse.md` (task 10) | `src/proof_reuse.rs` | [x] |
| cluster_db | `cluster_db.md` (task 12) | `src/cluster_db.rs` | [x] |

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

The crate depends on `mizar-session`, `mizar-artifact`, `mizar-vc`, and,
starting in task 11, `mizar-proof` for proof-reuse metadata exported by status
projection. `mizar-vc` supplies public per-VC dependency-slice fingerprints.
`mizar-artifact` provides canonical-hash rules, interface/implementation hash
inputs from its task 16, and witness references. Consumers integrate through
seams: the `mizar-build` scheduler (its task 18), the `mizar-ir` cache adapter
(its task 10). Architecture:
[11.artifact_and_incremental_build.md](../../architecture/en/11.artifact_and_incremental_build.md),
[18.dependency_fingerprint.md](../../architecture/en/18.dependency_fingerprint.md),
[17.cluster_trace_format.md](../../architecture/en/17.cluster_trace_format.md);
internal: [02](../../internal/en/02.artifact_store_cache_key_and_manifest.md),
[06](../../internal/en/06.ir_storage_and_snapshot_handles.md).

## Resolved And Open Decisions

- **Key/store split: resolved by internal 02.** `CacheKeyBuilder` only
  produces keys and dependency summaries; compatibility checks and proof
  witness validation decide reuse. Neither grants trust.
- **Slice granularity: resolved by task 4.** `dependency_fingerprint.md`
  keeps theorem, definition, cluster, notation, mode, and attribute as the
  semantic target taxonomy, while task 5 may begin with conservative
  published-summary plus per-VC slice granularity until finer producer slices
  land.
- **Record encoding: resolved by task 7.** `cache_store.md` uses a binary
  record envelope with a canonical UTF-8 JSON header and inline or blob-backed
  payload bytes. Cache records are internal, but `mizar-ir` adapter and raw-IR
  integration remain external dependency gaps; do not add placeholder IR
  storage APIs before the owning task lands.
- **Cluster-db visibility: resolved by task 12.** `cluster_db.md` keeps
  `origins/` as the invalidation source of truth, derives spec-23.7.7 aggregate
  indexes from accepted origin records only, and materializes import-scoped
  views as deletable cache data. Unaccepted, recovered, or externally attested
  material never becomes importer-visible through cache records.

## Ordered Task List

Keep `cargo test -p mizar-cache` green after each task (see
[Recommended Verification](#recommended-verification)).

### Keys and fingerprints

1. **Crate scaffold and lint-policy guard.** [x]
   - Add the `mizar-cache` workspace member depending on `mizar-session`
     and `mizar-artifact`; add `tests/lint_policy.rs` mirroring the
     `mizar-frontend` guard.
   - Tests: lint-policy guard passes; workspace builds.
   - Deps: `mizar-artifact` task 3. Spec: internal 02.

2. **Spec: `cache_key.md`.** [x]
   - Write the cache-key spec (English and Japanese, no code): the
     `CacheKey` fields (phase, work unit, source identity, input/dependency
     hashes, dependency slices, config hash, schema versions, policy
     fingerprint), canonical vector ordering, and the domain-separated
     final hash. Include the architecture-22 validation inputs for verifier
     policy, toolchain/schema compatibility, canonical VC fingerprints,
     local-context fingerprints, and dependency-slice fingerprints.
   - Deps: 1. Spec: [internal 02](../../internal/en/02.artifact_store_cache_key_and_manifest.md)
     "Cache Key".

3. **Cache-key builder.** [x]
   - Implement `CacheKeyBuilder` as a pure projection from identities,
     hashes, schema versions, and policy — no mutable task state.
   - Tests: key determinism; field-order independence (canonical sort);
     any input change changes the key.
   - Deps: 2. Spec: `cache_key.md`.

4. **Spec: `dependency_fingerprint.md`.** [x]
   - Write the fingerprint spec (English and Japanese, no code):
     fingerprint targets, dependency slices, stable inputs (and excluded
     nondeterministic inputs), rebuild triggers, and the API-compatibility
     diff per architecture 18, including the slice-granularity decision.
     Specify complete dependency-footprint requirements and the conservative
     `uncacheable` marker used when a footprint cannot be trusted.
   - Deps: 2. Spec:
     [18.dependency_fingerprint.md](../../architecture/en/18.dependency_fingerprint.md).

5. **Dependency-slice and fingerprint computation.** [x]
   - Compute fingerprints over interface/implementation hash inputs
     (`mizar-artifact` task 16) and per-VC dependency slices
     (`mizar-vc` task 14) at the decided granularity.
   - Tests: fingerprint stability under non-interface edits; slice changes
     flip exactly the dependent fingerprints; deterministic output;
     canonical ordering and duplicate-conflict rejection; stable exclusions for
     comments, formatting, diagnostics, runtime/order, temporary/local paths,
     and snapshot-local ids; missing/unknown/uncacheable miss behavior; and
     proof-reuse validation data staying untrusted.
   - Deps: 3, 4, `mizar-artifact` task 16, `mizar-vc` task 14. Spec:
     `dependency_fingerprint.md`.

6. **Rebuild-trigger evaluation.** [x]
   - Implement the trigger rules: which fingerprint changes invalidate
     which cached phases, conservatively when slices are coarse.
   - Tests: trigger fixtures per change kind (source, import, registration,
     cluster/reduction, policy, toolchain, schema, proof-body,
     diagnostic-only, incomplete footprint, unknown schema/toolchain,
     uncacheable marker, missing proof-reuse validation); no false negatives
     in the conservative mode.
   - Deps: 5. Spec: `dependency_fingerprint.md` (triggers section).

### Store

7. **Spec: `cache_store.md`.** [x]
   - Write the store spec (English and Japanese, no code): the
     `.mizar-cache/` layout (records by phase and key, content-addressed
     blobs), `CacheRecordHeader`, compatibility checks, the record-encoding
     decision, `uncacheable` record handling, miss-on-incomplete-footprint
     behavior, miss-on-unknown-compatibility behavior, and deletability
     guarantees.
   - Deps: 2. Spec: [internal 02](../../internal/en/02.artifact_store_cache_key_and_manifest.md)
     "Cache Store"/"Cache Record".

8. **Record store.** [x]
   - Implement record write/read/validate with header compatibility checks
     (cache schema version, toolchain, output hash).
   - Tests: round-trips; incompatible headers miss instead of erroring;
     corrupted records detected.
   - Deps: 3, 7. Spec: `cache_store.md`.

9. **Blob store.** [x]
   - Implement content-addressed blob storage for large outputs with
     hash-verified reads and safe concurrent writes.
   - Tests: blob round-trips; hash mismatch detection; concurrent writers
     converge.
   - Deps: 8. Spec: `cache_store.md`.

### Proof reuse and cluster db

10. **Spec: `proof_reuse.md`.** [x]
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

11. **Proof-reuse validation.** [x]
    - Implement reuse validation over proof-reuse metadata snapshots derived
      from `mizar-proof` `StatusReuseMetadata`; failures degrade to
      recomputation, never to acceptance.
    - Tests: matching `KernelVerified` and `DischargedBuiltin` evidence
      reuses; each missing or mismatched required component (`ObligationAnchor`, obligation
      fingerprint, canonical VC fingerprint, local-context fingerprint,
      dependency-slice fingerprint, selected witness hash, deterministic
      discharge hash, selected proof class, proof-evidence identity, selected
      evidence hash, selected candidate provenance hash, selection reason,
      tie-break key hash, trusted used-axioms reference hash when exported,
      proof-reuse validation hash, policy, schema, and dependency artifact)
      blocks reuse; incomplete footprints, unsupported
      schemas, unknown toolchains, and uncacheable markers miss; non-trusted
      classes and externally attested evidence never become kernel-verified or
      trusted `used_axioms` via cache, and records that synthesize trusted
      `used_axioms` or trusted used-axioms reference hashes are rejected;
      upstream proof-reuse completeness is honored; local validation is
      independent of record arrival/write order and cache hit/miss timing; a
      lint/source-surface guard confirms no scheduler, `mizar-ir`, artifact
      publication-token, or witness-publication shortcut is added. Cross-crate
      producer wiring and full clean/incremental equivalence remain the task-20
      gate.
    - Deps: 8, 10. Spec: `proof_reuse.md`.

12. **Spec: `cluster_db.md`.** [x]
    - Write the cluster-db spec (English and Japanese, no code): index
      layout, origin metadata per contribution, import-scoped views,
      invalidation by visible-origin changes, and the rule that unaccepted
      registration proofs never enter importer-visible indexes.
    - Completed by task 12: `cluster_db.md` defines the logical
      `.mizar-cache/cluster-db/` layout, origin metadata, accepted-only
      visibility rule, aggregate index ordering, import-scoped view keys,
      visible-origin invalidation, ResolutionTrace boundary, and task-13/14
      planned tests without adding source implementation.
    - Deps: 7. Spec: [internal 02](../../internal/en/02.artifact_store_cache_key_and_manifest.md)
      "Cluster and Registration Cache Update",
      [17.cluster_trace_format.md](../../architecture/en/17.cluster_trace_format.md).

13. **Cluster-db writes and origin tracking.** [x]
    - Implement contribution writes with origin metadata, stale-origin
      removal, and per-origin index rebuilds.
    - Tests: accepted contribution insertion; accepted but private/local-only,
      rejected, pending, recovered, uncacheable, and externally attested
      contributions rejected from visible indexes; incomplete origin metadata,
      incomplete footprints, missing dependency-interface hashes, missing
      trace replay hashes, and missing proof witness/discharge identity for
      proof-backed accepted contributions rejected; unknown schema/toolchain
      compatibility rejected; rename/removal cleans stale origins; rebuilds
      touch only affected origins; deterministic same-index ordering;
      duplicate-conflicting origin and cross-module origin-key collision
      rejection.
    - Implementation: task 13 adds an in-memory cache-side data layer over
      producer-owned `ClusterContributionRecord` snapshots. It does not parse
      raw source, claim proof/checker authority, write scheduler hooks, create
      `mizar-ir` APIs, or materialize import-scoped views.
    - Completed by task 13: `src/cluster_db.rs` exposes
      `ClusterDbIndex::apply_module_update`, fail-closed origin validation,
      stale-origin cleanup, aggregate index snapshots, rebuild reports, and
      lint guards for downstream/proof-authority stubs.
    - Deps: 12, `mizar-checker` task 16. Spec: `cluster_db.md`.

14. **Import-scoped views and invalidation.** [x]
    - Implement import-scoped views keyed by visible origin sets, with
      invalidation on view-set changes.
    - Tests: view reuse across unrelated changes; visible-origin change
      invalidates exactly the affected views; request validation misses for
      missing origins, unsupported schema, missing producer schema metadata,
      unknown policy/schema/toolchain/traversal compatibility, mismatched
      verifier policy, and mismatched producer compatibility metadata;
      deterministic view ordering is independent of record arrival/write order
      and visible-origin request order; hidden cluster/reduction steps are not
      inferred.
    - Implementation: task 14 adds in-memory `ImportScopedViewRequest`,
      `ImportScopedViewKey`, `ImportScopedView`, `ClusterDbViewMiss`, and
      `ClusterDbIndex::import_scoped_view`. The view canonicalizes and
      validates request metadata, checks visible origins against accepted
      origin records, filters aggregate rows by visible origin set, and keeps
      view hits as cache optimization only.
    - Deferred/out of scope for task 14: durable `views/` files, scheduler
      integration, `mizar-ir` adapter integration, artifact publication-token
      linkage, proof status projection, and trace construction remain out of
      scope and are not stubbed.
    - Deps: 13. Spec: `cluster_db.md`.

### Integration and follow-ups

15. **Scheduler and IR-adapter integration readiness.** [x]
    - Plug the store into the `mizar-build` cache seam (its task 18) and
      the `mizar-ir` cache adapter (its task 10); cache hits skip work with
      externally identical results.
    - Tests: hit/miss end-to-end fixtures; hit results byte-identical to
      clean-build results when downstream owners are ready. Task 15 itself is
      documentation/review only because those seams are not ready.
    - Deps: 8, `mizar-build` task 18, `mizar-ir` task 10. Spec:
      [internal 02](../../internal/en/02.artifact_store_cache_key_and_manifest.md)
      "Cache Lookup Before Task Execution".
    - Completed by task 15: [integration_readiness.md](./integration_readiness.md)
      records `external_dependency_gap`s for the still-open `mizar-build`
      cache-aware scheduler seam, absent `mizar-ir` cache adapter, and
      artifact committed-publication token linkage. No placeholder scheduler,
      `mizar-ir`, or artifact-publication-token APIs were added. The
      implementation and end-to-end tests above remain deferred until their
      owners land.

16. **Determinism and deletability suite.** [x]
    - Property coverage: identical inputs produce identical keys and
      records; deleting representative crate-owned record/blob subsets
      changes only lookup availability until deterministic repopulation, not
      canonical identity or proof acceptance. Full build-result equivalence is
      task-20 scope.
    - Deps: 15. Spec: [20.test_strategy.md](../../architecture/en/20.test_strategy.md).
    - Completed by task 16: `tests/determinism_suite.rs` covers canonical
      cache-key ordering, record/blob deletion and repopulation on the
      crate-owned store, proof-reuse diagnostic determinism, and rejection of
      externally attested proof material as non-reusable. Full
      scheduler-level clean/incremental equivalence remains deferred to
      task 20.

17. **Public-enum forward-compatibility policy.** [x]
    - Apply the `mizar-frontend` task-25 procedure to each public enum.
    - Deps: 14. Spec: all module specs.
    - Completed by task 17: all current public enums are documented as
      `#[non_exhaustive]` in the paired module specs and guarded by
      `tests/lint_policy.rs`. `mizar-cache` owns no exhaustive public enum
      exceptions.

18. **Source/spec correspondence audit.** [x]
    - Trace every public API and promised behavior in the module specs to
      implementation and tests; record gaps as follow-up tasks.
    - Deps: 17. Spec: all module specs and this TODO.
    - Completed by task 18: paired `source_spec_audit.md` documents trace the
      public API, promised behavior, tests, and existing deferred or
      external-dependency gaps. No new unclassified source/spec drift or gap ID
      was found.

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

22. **Module-boundary refactor gate.** [ ]
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
    - Deps: 21. Spec: this TODO,
      [internal 07](../../internal/en/07.crate_module_layout.md), all module
      specs.

23. **Crate exit report and quality review.** [ ]
    - Produce the paired crate exit report after tasks 1-22 finish or are
      explicitly deferred as external dependency gaps. Record task commits,
      hard-gate status, review results, verification, deferred items, and a
      valid read-only quality score of at least 90/100.
    - Deps: 22. Spec:
      [autonomous_crate_development.md](../../autonomous_crate_development.md).
    - Status: closeout only; do not mix new feature implementation into this
      task except fixes required by review.

## Recommended Verification

Run after each task:

```text
cargo test -p mizar-cache
cargo clippy -p mizar-cache --all-targets -- -D warnings
cargo fmt --check
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

For Rust source changes, AGENTS.md broad verification also applies before
finalizing unless a command is explicitly justified as unrun:

```text
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Check the task off here once the task-appropriate verification and any required
broad verification pass.

## Notes

- The cache is optimization, never authority: hits satisfy the same
  validation rules as a clean build, and deleting records never changes
  semantics.
- A cache record can never upgrade externally attested evidence to
  kernel-verified status; proof reuse is tied to accepted witness hashes.
- Keys are pure projections; trust decisions live in compatibility checks
  and proof-reuse validation, not in key construction.
- Cache records are internal and may store opaque payload bytes; raw-IR
  encoding and adapter integration remain `mizar-ir` external dependency gaps,
  so this crate must not invent placeholder IR storage APIs.
