# mizar-artifact TODO

> Canonical language: English. Japanese companion: [../ja/todo.md](../ja/todo.md).

## Status Legend

- [ ] not started
- [~] in progress
- [x] done

## Module Implementation

Module specs are written by their own spec tasks (English and Japanese in the
same change) before the implementation tasks that cite them. Completed spec
tasks add their files before the corresponding source task starts. Module names
follow the minimum split of
[internal 07](../../internal/en/07.crate_module_layout.md) (plus the summary
schemas the resolver and checker consume); the crate refines architecture 11
and 18 and internal 02 and 06.

| Module | Spec | Source | Status |
|---|---|---|---|
| store | `store.md` (task 2) | `src/store.rs` | [ ] |
| module_summary | `module_summary.md` (task 4) | `src/module_summary.rs` | [ ] |
| registration_summary | `registration_summary.md` (task 6) | `src/registration_summary.rs` | [ ] |
| proof_witness | `proof_witness.md` (task 8) | `src/proof_witness.rs` | [ ] |
| verified_artifact | `verified_artifact.md` (task 10) | `src/verified_artifact.rs` | [ ] |
| manifest | `manifest.md` (task 12) | `src/manifest.rs` | [ ] |

`mizar-artifact` owns the stable external projections of the pipeline:
artifact schemas (`ModuleSummary`, `RegistrationSummary`, `ProofWitnessRef`,
`VerifiedArtifact`), the artifact store with atomic writes, and manifest
transactions (phase 15). It is deliberately a low-dependency schema/store
crate: producers (`mizar-resolve`, `mizar-checker`, `mizar-vc`,
`mizar-kernel`, `mizar-proof`) depend on it to construct projections, never
the reverse. It is built in two waves: **wave A** (schemas and readers) lands
early because cross-module resolution reuses `ModuleSummary` artifacts;
**wave B** (store, manifest transactions, full emission) completes phase 15
once proof outputs exist.

Dependency order: `store` foundations → summary schemas (`module_summary`,
`registration_summary`) → `proof_witness` / `verified_artifact` →
`manifest` / emission.

Each task below is deliberately small — one module spec, or one behavior slice
of one module — so that a single task can be implemented, tested, and
committed autonomously without holding the rest of the crate in flight.

## Crate Prerequisites

The crate depends on `mizar-session` only. Wave A has no other gates and can
start alongside `mizar-resolve`; wave B emission integration is gated on
kernel/proof outputs existing. Architecture:
[11.artifact_and_incremental_build.md](../../architecture/en/11.artifact_and_incremental_build.md),
[18.dependency_fingerprint.md](../../architecture/en/18.dependency_fingerprint.md);
internal: [02](../../internal/en/02.artifact_store_cache_key_and_manifest.md),
[06](../../internal/en/06.ir_storage_and_snapshot_handles.md).

## Resolved And Open Decisions

- **Dependency direction: resolved by internal 02/06.** This crate is a
  leaf schema/store crate; producer crates depend on it and the projection
  boundary of internal 06 assembles `VerifiedArtifact`s from
  producer-built projections. The compiler-internal IR never appears in
  schemas.
- **Cache ownership: resolved by internal 07.** Cache keys, dependency
  fingerprints, proof reuse validation, and cluster-db storage belong to
  `mizar-cache`; this crate owns artifact schemas, the artifact store, and
  manifest transactions. The two share the canonical-hash rules defined
  here (task 2).
- **Hash separation: resolved by internal 07 constraints.** Semantic hashes
  and diagnostic/development hashes are separate; locally variable fields
  (such as `verified_at`) are excluded from canonical hashes (task 2
  encodes both).

## Ordered Task List

Keep `cargo test -p mizar-artifact` green after each task (see
[Recommended Verification](#recommended-verification)).

### Wave A: canonical form and summary schemas

1. **Crate scaffold and lint-policy guard.** [x]
   - Add the `mizar-artifact` workspace member depending on `mizar-session`
     only; add `tests/lint_policy.rs` mirroring the `mizar-frontend` guard.
   - Tests: lint-policy guard passes; workspace builds.
   - Deps: none. Spec: architecture 11.

2. **Spec: `store.md`.** [x]
   - Write the store/canonical-form spec (English and Japanese, no code):
     store layout per internal 02, canonical UTF-8 JSON serialization with
     deterministic ordering, `schema_version` and compatibility checks,
     semantic-versus-diagnostic hash separation, hash-excluded local
     fields, and atomic-write requirements.
   - Deps: 1. Spec: [internal 02](../../internal/en/02.artifact_store_cache_key_and_manifest.md),
     architecture 11 "Deterministic Artifact Output"/"Atomic Writes".

3. **Canonical serialization and schema-version checks.** [ ]
   - Implement canonical serialization, hashing rules, and schema-version
     compatibility checking shared by all schemas.
   - Tests: byte-identical serialization across runs/platforms; version
     mismatch detection; excluded fields do not affect hashes.
   - Deps: 2. Spec: `store.md`.

4. **Spec: `module_summary.md`.** [ ]
   - Write the `ModuleSummary` schema spec (English and Japanese, no code):
     exported interface projection per architecture 03 "Module Summary",
     keyed by `interface_hash` (architecture 18), excluding bodies and
     proofs per the resident-set rules.
   - Deps: 2. Spec: architecture 03,
     [18.dependency_fingerprint.md](../../architecture/en/18.dependency_fingerprint.md).

5. **`ModuleSummary` schema, writer, and reader.** [ ]
   - Implement the schema with writer and validating reader; this unblocks
     `mizar-resolve` task 24 (summary-backed resolution).
   - Tests: round-trips; interface-hash stability under body-only changes;
     incompatible-version reads fail cleanly.
   - Deps: 3, 4. Spec: `module_summary.md`.

6. **Spec: `registration_summary.md`.** [ ]
   - Write the `RegistrationSummary` schema spec (English and Japanese, no
     code): exported registrations and cluster-trace artifact references
     per architecture 04 and 17.
   - Deps: 2. Spec: architecture 04,
     [17.cluster_trace_format.md](../../architecture/en/17.cluster_trace_format.md).

7. **`RegistrationSummary` schema, writer, and reader.** [ ]
   - Implement the schema with writer and validating reader for checker
     cross-module reuse.
   - Tests: round-trips; trace references resolve by hash; deterministic
     ordering.
   - Deps: 3, 6. Spec: `registration_summary.md`.

### Wave A/B boundary: witnesses and the verified artifact

8. **Spec: `proof_witness.md`.** [ ]
   - Write the proof-witness reference spec (English and Japanese, no
     code): witness files referenced by hash, kernel acceptance metadata,
     and the rule that witnesses stay external to the main artifact
     (resident-set discipline).
   - Deps: 2. Spec: [internal 02](../../internal/en/02.artifact_store_cache_key_and_manifest.md),
     [internal 04](../../internal/en/04.atp_portfolio_and_kernel_check_integration.md)
     "Proof Witness Store".

9. **`ProofWitnessRef` schema.** [ ]
   - Implement witness references with hash validation.
   - Tests: round-trips; hash mismatch detection.
   - Deps: 3, 8. Spec: `proof_witness.md`.

10. **Spec: `verified_artifact.md`.** [ ]
    - Write the `VerifiedArtifact` schema spec (English and Japanese, no
      code): exports, expression metadata, obligation statuses, witness
      references, projected diagnostics, and the compatibility policy.
    - Deps: 4, 8. Spec: architecture 11 "Verified Artifact Schema",
      [01.ir_layers.md](../../architecture/en/01.ir_layers.md).

11. **`VerifiedArtifact` schema and projection inputs.** [ ]
    - Implement the schema and the projection-input contract that producer
      crates fill (stub producers in tests until the real ones land).
    - Tests: round-trips; schema rejects raw-IR-shaped payloads; projected
      diagnostics keep stable codes and order.
    - Deps: 9, 10. Spec: `verified_artifact.md`.

### Wave B: store, manifest, and emission

12. **Spec: `manifest.md`.** [ ]
    - Write the manifest spec (English and Japanese, no code): package
      artifact manifest, manifest transaction protocol (begin/commit,
      reader visibility), and recovery from interrupted commits.
    - Deps: 2. Spec: [internal 02](../../internal/en/02.artifact_store_cache_key_and_manifest.md)
      "Manifest Transaction", architecture 11 "Artifact Manifest".

13. **Artifact store with atomic writes.** [ ]
    - Implement the store: stable published artifact writes, schema-required
      hash-addressed published files such as witnesses, temp-and-rename
      atomicity, and corruption detection on read; interrupted writes never
      look like complete output. Internal cache blobs remain owned by
      `mizar-cache`.
    - Tests: kill-during-write fixtures leave no visible partial artifact;
      corrupted artifact or witness reads fail with positions.
    - Deps: 3, 12. Spec: `store.md`, `manifest.md`.

14. **Manifest transactions.** [ ]
    - Implement transactional manifest updates with deterministic entry
      ordering and reader-side validation.
    - Tests: concurrent reader sees old-or-new, never mixed; replayed
      commits are idempotent.
    - Deps: 13. Spec: `manifest.md`.

15. **Provenance and reproducibility metadata.** [ ]
    - Implement build provenance records (toolchain, edition, settings,
      dependency hashes) attached to emitted artifacts, with hash-excluded
      local fields handled per task 2.
    - Tests: provenance round-trips; canonical hash unaffected by local
      fields.
    - Deps: 11, 14. Spec: architecture 11 "VerifiedArtifact Is a
      Projection", [18.dependency_fingerprint.md](../../architecture/en/18.dependency_fingerprint.md).

16. **Interface/implementation hash inputs.** [ ]
    - Compute and expose the interface-hash and implementation-hash inputs
      that `mizar-cache` fingerprints consume (architecture 18), with
      canonical ordering documented per hash.
    - Tests: interface hash stable under implementation-only edits; hash
      input ordering deterministic.
    - Deps: 15. Spec: architecture 11 "Interface Hashes and Implementation
      Hashes".

17. **Phase 15 emission integration.** [ ]
    - Wire full `VerifiedArtifact` emission from real producer projections
      once kernel/proof outputs exist; emit through the store and manifest
      transactions only.
    - Tests: end-to-end emission fixture over a small verified module;
      re-emission is byte-identical.
    - Deps: 14, 15, `mizar-kernel` task 16, `mizar-proof` task 11
      (witness staging/publication). Spec: `verified_artifact.md`,
      `manifest.md`.

### Hardening and cross-cutting follow-ups

18. **Determinism suite.** [ ]
    - Property coverage that identical inputs produce byte-identical
      artifacts, manifests, and hashes across runs and platforms.
    - Deps: 16. Spec: [20.test_strategy.md](../../architecture/en/20.test_strategy.md).

19. **Public-enum forward-compatibility policy.** [ ]
    - Apply the `mizar-frontend` task-25 procedure to each public enum;
      schema enums additionally follow the artifact compatibility policy.
    - Deps: 16. Spec: all module specs.

20. **Source/spec correspondence audit.** [ ]
    - Trace every public API and promised behavior in the module specs to
      implementation and tests; record gaps as follow-up tasks.
    - Deps: 19. Spec: all module specs and this TODO.

21. **Bilingual documentation sync audit.** [ ]
    - Compare each English canonical document under
      `doc/design/mizar-artifact/en/` with its Japanese companion and
      synchronize content.
    - Deps: 20. Spec: repository documentation policy.

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

## Recommended Verification

Run after each task:

```text
cargo test -p mizar-artifact
cargo clippy -p mizar-artifact --all-targets -- -D warnings
```

For tasks that change schemas consumed elsewhere, also run the consumers:

```text
cargo test -p mizar-resolve
cargo test -p mizar-checker
```

Check the task off here once tests pass.

## Notes

- Published artifacts are stable projections, never raw IR dumps; internal
  IRs may change between versions, schemas may not (without
  `schema_version` handling).
- All paths in portable artifacts are package- or workspace-relative;
  artifacts must be readable without compiler-internal cache records.
- Wave A is on the critical path of cross-module resolution; keep it small
  and land it early.
- Cache records, cache keys, and proof-reuse validation live in
  `mizar-cache`, not here; the shared canonical-hash rules are task 2's.
