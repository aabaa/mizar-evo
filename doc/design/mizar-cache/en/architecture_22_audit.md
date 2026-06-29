# mizar-cache Architecture-22 Follow-up Audit

> Canonical language: English. Japanese companion:
> [../ja/architecture_22_audit.md](../ja/architecture_22_audit.md).

Task 21 reruns the source/spec and bilingual documentation checks after the
task-20 fail-closed cache contract tests. It changes no source behavior,
public API, cache lookup policy, proof acceptance policy, artifact publication
policy, or downstream integration.

## Scope

The audit checks the crate-owned parts of
[architecture 22](../../architecture/en/22.incremental_verification_contract.md)
against the current `mizar-cache` design and tests:

- canonical cache-key construction;
- dependency-footprint completeness and fail-closed unknown markers;
- cache record lookup/insert compatibility checks;
- proof-reuse validation over `mizar-proof` metadata;
- cache deletion and cache timing as non-semantic observations;
- external evidence non-promotion;
- cluster-db accepted-only visibility where architecture 22 overlaps
  registration/cluster activation.

The audit also rechecks the paired English/Japanese documents touched by task
20 and the gap classifications in `task_ledger.md` and
`source_spec_audit.md`.

## Architecture-22 Contract Check

| Architecture-22 rule | Current `mizar-cache` coverage | Finding |
|---|---|---|
| Cache is optimization, not proof authority. | Crate root, module specs, lint guards, and proof-reuse tests forbid `KernelCheckResult`, proof-status projection, trusted `used_axioms`, scheduler hooks, IR adapters, and publication-token shortcuts. | no finding |
| Missing cache data, unsupported schema, incomplete dependency slices, unknown toolchain compatibility, incompatible policy, or proof witness/discharge mismatch must miss. | `cache_key`, `dependency_fingerprint`, `cache_store`, and `proof_reuse` unit tests plus `tests/incremental_contract.rs`. | no finding |
| Reused proof results require matching `ObligationAnchor`, canonical VC fingerprint, local-context fingerprint, dependency-slice fingerprints, policy, and witness/discharge hashes. | `proof_reuse` unit tests cover each metadata mismatch; task-20 integration tests tie the proof environment, key validation inputs, dependency footprint, and proof metadata together. | no finding |
| Cache miss and cache deletion affect performance only. | `tests/determinism_suite.rs` and `tests/incremental_contract.rs` cover record/blob deletion, lookup availability, and diagnostic-order independence within crate-owned APIs. | no finding |
| Cache lookup timing, hit/miss timing, record arrival/write order, backend runtime, and diagnostics are not semantic inputs. | `cache_key` excludes runtime inputs; `cache_store` and `proof_reuse` tests cover write/order and diagnostic-order independence; no timing APIs are exposed. | no finding |
| Externally attested evidence must not become kernel-verified by cache reuse. | `proof_reuse` rejects non-trusted classes and synthesized trusted axiom refs; `dependency_fingerprint` projects external-only validation to uncacheable; task-20 integration tests cover both. | no finding |
| Only accepted registration contributions may enter importer-visible cluster/reduction views. | `cluster_db` accepts only visible accepted origin records and rejects recovered, external, unaccepted, inferred, or incomplete origins. | no finding |

## Bilingual And Source/Spec Check

No EN/JA semantic drift is observed for the task-20 changes. The paired
documents agree that:

- `tests/incremental_contract.rs` covers the crate-owned architecture-22
  fail-closed cache contract;
- `PROOFREUSE-G005` is an `external_dependency_gap`, not a remaining
  `mizar-cache` task, because the unresolved clean/incremental equivalence
  depends on scheduler and artifact-publication owners;
- source/test changes do not add scheduler hooks, `mizar-ir` placeholder APIs,
  artifact publication shortcuts, or proof-authority projections.

No `repo_metadata_conflict`, unclassified `spec_gap`, unclassified `test_gap`,
`design_drift`, `source_drift`, `source_undocumented_behavior`,
`test_expectation_drift`, or `boundary_violation` is observed.

## Residual Gaps

Task 21 adds no new gap IDs. Existing residual work remains classified in
`task_ledger.md` and `source_spec_audit.md`.

| Gap | Classification | Task-21 disposition |
|---|---|---|
| `CACHE-G-003`, `DEPFPR-G001`, `CACHESTORE-G001`, `PROOFREUSE-G001`, `CLUSTERDB-G004`, `CACHE15-G001` | `external_dependency_gap` | `mizar-build` scheduler integration and dependency-fingerprint consumption are still owner-gated; no placeholder scheduling API is added. |
| `CACHE-G-004`, `DEPFPR-G002`, `CACHESTORE-G002`, `PROOFREUSE-G002`, `CLUSTERDB-G005`, `CACHE15-G002` | `external_dependency_gap` | `mizar-ir` cache adapter integration is absent; no placeholder crate or adapter API is added. |
| `CACHE-G-005`, `DEPFPR-G003`, `CACHESTORE-G003`, `PROOFREUSE-G003`, `CACHE15-G003` | `external_dependency_gap` | artifact/proof publication-token integration remains artifact/proof owned; cache compares hashes and validation metadata only. |
| `DEPFPR-G005`, `PROOFREUSE-G004`, `PROOFREUSE-G005` | `external_dependency_gap` | downstream proof/cache/artifact consumers, trusted `DischargedBuiltin` publication, and cross-crate clean/incremental equivalence remain external to this crate. |
| `CLUSTERDB-G001` | `external_dependency_gap` | accepted-contribution producer fields remain checker/artifact owned; cache records missing fields and does not fabricate accepted status. |
| `DEPFPR-G004`, `CLUSTERDB-G002`, `CLUSTERDB-G003`, `CACHESTORE-G004` | `deferred` | finer producer slices and durable cluster-db/view storage are later cache/producer work, not task-21 behavior. |

## Conclusion

No unresolved blocking or high-severity fail-closed/trust-boundary finding
remains for the crate-owned architecture-22 cache contract. `mizar-cache`
remains an internal optimization owner. Cache records, dependency
fingerprints, externally attested evidence, diagnostics, logs, timing
metadata, cluster-db data, and proof-reuse validation metadata do not become
kernel-verified proof status or trusted `used_axioms`.
