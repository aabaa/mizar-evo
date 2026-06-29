# Crate Exit Report: mizar-cache

> Canonical language: English. Japanese companion:
> [../ja/crate_exit_report.md](../ja/crate_exit_report.md).

## Result

Status: complete for the `mizar-cache` internal-cache milestone.
Quality score: 94/100.
Score caps applied: none.

## Scope

Milestone scope:

- Build the `mizar-cache` workspace crate from task 0 through task 22 and this
  closeout task.
- Own canonical internal build `CacheKey` construction as a pure projection.
- Own dependency footprints and fingerprints, including complete, unknown,
  unsupported, and uncacheable states.
- Own internal cache record and blob storage with compatibility checks,
  corruption handling, deletability, and deterministic record lookup.
- Own proof-reuse validation over metadata exported by `mizar-proof`.
- Own accepted-only in-memory cluster-db origin/index/view materialization.
- Classify, rather than stub, unfinished scheduler, `mizar-ir`, artifact
  publication-token, persistent cluster-db/view, and producer-slice work.

Excluded:

- Proof authority, proof status projection, deterministic winner selection,
  policy ownership, kernel acceptance, SAT/ATP proof search, artifact manifest
  commit, build scheduling, IR rehydration, or publication-token issuance.
- Promotion of cache records, externally attested evidence, backend
  diagnostics, backend logs, timing metadata, record arrival/write order, or
  cluster-db data into kernel-verified proof status or trusted `used_axioms`.
- Placeholder downstream integration with unfinished `mizar-build`,
  nonexistent `mizar-ir`, or artifact committed-publication-token surfaces.

## Task Commits

| Task | Commit | Subject |
|---:|---|---|
| 0 | `8f1d2ab443bc52a50db98f419b527aaa95737d17` | `docs(cache-task-0): add autonomous crate plan` |
| 1 | `fd4a87509cc73ae1e48d4c59c88e5f7d3f33a970` | `feat(cache-task-1): scaffold cache crate` |
| 2 | `06abae61da6c4e0c2d2b45429fbc8cd273fd3202` | `docs(cache-task-2): specify cache keys` |
| 3 | `27dc5d6a851204cf17003f40a28bb564d602ab54` | `feat(cache-task-3): add canonical cache key builder` |
| 4 | `cf66ac9d3d74da75b998b4c7f299301a181e9e24` | `docs(cache-task-4): specify dependency fingerprints` |
| 5 | `a49dcd8e27f9681726b8c6cb6b28b2834276fe95` | `feat(cache-task-5): compute dependency fingerprints` |
| 6 | `da62b43ff4d5bccfe3220b8257569a43a6e0be96` | `feat(cache-task-6): evaluate rebuild triggers` |
| 7 | `e22a02bb756809b8b71d7e098cf1addf9955684b` | `docs(cache-task-7): specify cache store` |
| 8 | `3ce9076b271c923d6106e43e7e56b9cb97029eda` | `feat(cache-task-8): add record store` |
| 9 | `33ae5456409f8083cde9540d6cead8c56b932902` | `feat(cache-task-9): add blob store` |
| 10 | `4a304392831b7f5c8d75d3519835aa8715cf874c` | `docs(cache-task-10): specify proof reuse` |
| 11 | `e3cc79af51be48bdce4183b4ceba009432c5fb9a` | `feat(cache-task-11): validate proof reuse` |
| 12 | `91d62e4f4c4df39b4a11a93a394b900167eea550` | `docs(cache-task-12): specify cluster db` |
| 13 | `8283a90ad6703337fcbfce795137a8a5fd7997b1` | `feat(cache-task-13): index cluster origins` |
| 14 | `a341da222a17d949ce09dd4ea59c9bbf461ee778` | `feat(cache-task-14): materialize cluster views` |
| 15 | `add05ca79f060fcadf93d7b48d609726353176cf` | `docs(cache-task-15): defer integration seams` |
| 16 | `c6878845d0d86d8444c35d7b9c2fab1bc9a2bc1e` | `test(cache-task-16): add determinism suite` |
| 17 | `c70980eedd1df25c591627c4d4f4958de7fefeed` | `test(cache-task-17): guard public enum policy` |
| 18 | `10bcacceca535f71c666906a4d6a419c06a19020` | `docs(cache-task-18): audit source spec correspondence` |
| 19 | `ee80385a01fdb567a5a46afdb2ca0942bf02741a` | `docs(cache-task-19): audit bilingual docs sync` |
| 20 | `c4f855b37c3e850a030d768f9223faf862704256` | `test(cache-task-20): enforce incremental cache contract` |
| 21 | `b1dbc697d38511613fe207817e142a7092f52486` | `docs(cache-task-21): audit architecture 22 contract` |
| 22 | `486f0640e75a0f079c21ce540c4e6f637341b033` | `refactor(cache-task-22): split private test modules` |
| 23 | pending self-hash | `docs(cache-task-23): add cache crate exit report` |

## Final Owned Surfaces

| Surface | Final shape |
|---|---|
| CacheKey | `CacheKeyBuilder` canonicalizes source identity, input hashes, dependency hashes/slices, schema versions, policy fingerprint, and validation inputs. It rejects unsupported schema, duplicate conflicting identities, incomplete footprint markers, and uncacheable records. It performs no trust decision. |
| Dependency fingerprint | `DependencyFootprintBuilder` records complete dependency slices, producer hashes, proof-reuse validation identities, unknown markers, and uncacheable states. Rebuild triggers overtrigger conservatively but do not permit false-negative reuse. |
| Cache record store | `CacheStoreRoot` reads and writes internal cache records and blobs under deterministic paths, validates embedded keys and compatibility metadata, fails closed on corruption, missing blobs, schema/toolchain/policy mismatches, incomplete footprints, and dependency artifact mismatches, and treats deletion as lookup availability only. |
| Proof reuse | `ProofReuseValidator` consumes `mizar-proof` validation metadata for trusted reusable classes only. It requires matching obligation, VC/context/dependency fingerprints, policy, selected evidence, witness/discharge hashes, trusted axiom-set references, schema/toolchain compatibility, and dependency artifacts. It never owns winner selection or status projection. |
| Cluster DB | `ClusterDbIndex` accepts only visible accepted contribution records with complete producer metadata, maintains in-memory origins/aggregate indexes/views, rejects hidden/recovered/external/unaccepted/incomplete origins, and exposes only accepted importer-visible rows. |
| Integration boundary | Scheduler integration, `mizar-ir` adapters, artifact committed-publication-token linkage, durable cluster-db/view files, and finer producer slices are explicitly classified as `external_dependency_gap` or `deferred`; no placeholder API was added. |

## Hard Gates

| Gate | Status | Evidence |
|---|---|---|
| Specification consistency | passed | Paired module specs, crate plan, architecture-22 audit, module-boundary audit, source/spec audit, and closeout reviews record no unresolved blocking/high inconsistency. |
| Source behavior documented or deferred | passed | Public modules, public APIs, tests, private test module paths, and residual gaps are traced in `source_spec_audit.md`, `task_ledger.md`, and this report. |
| Cache is not proof authority | passed | Lint guards, module specs, proof-reuse tests, and task-20 integration tests forbid cache records, external evidence, diagnostics, logs, timing, and cluster-db data from becoming kernel-verified proof status or trusted `used_axioms`. |
| Fail-closed reuse | passed | Unknown schema/toolchain/policy, incomplete dependency footprints, uncacheable markers, missing/corrupt records, missing dependency artifacts, and proof metadata mismatches all miss. |
| Test expectation integrity | passed | No `.miz` fixture, traceability row, or expectation sidecar was changed to match implementation behavior. This crate's behavior is covered by Rust tests and audits. |
| Design/source synchronization | passed | EN/JA docs are paired and synchronized, and lint-policy guards track source paths, public enums, audit markers, and gap IDs. |
| Downstream gaps classified | passed | Scheduler, IR adapter, publication token, durable storage, producer-field, and fine-grained-slice work is classified rather than stubbed. |
| Verification | passed | Crate-local fmt/test/clippy, full workspace clippy, full workspace test, diff checks, pair inventory, and sync-placeholder scans passed. |

## Score Breakdown

| Category | Points |
|---|---:|
| Specification completeness | 19/20 |
| Test contract and coverage | 19/20 |
| Traceability | 15/15 |
| Implementation correctness | 14/15 |
| Design/source synchronization | 10/10 |
| Boundary discipline | 10/10 |
| Verification health | 5/5 |
| Handoff quality | 2/5 |
| Total | 94/100 |

The score deducts for unfinished downstream scheduler/IR/publication
integration, durable cluster-db/view storage, finer producer slice work, and
accepted-contribution producer fields that remain outside this crate. These are
classified and do not cap the score because they are not implemented with
placeholders and all hard gates pass.

## Review Results

| Review | Result |
|---|---|
| Implementation specification / documentation review | No findings. Closeout scope, task commits, owned surfaces, hard gates, deferred/external gap grouping, verification, and handoff are complete and internally consistent. |
| Test sufficiency review | No missing test gap for closeout substance. The final report records task-20 incremental contract coverage, task-22 moved private tests, lint guards, determinism suite, and broad verification. |
| Full implementation review | No findings. The report matches the implemented crate boundaries, and no production source change is part of closeout. |
| Source/documentation consistency review | No findings. EN/JA exit reports are synchronized, task hashes and subjects match git, and the bilingual sync audit includes this report. |
| Read-only crate quality review | Valid quality score: 94/100. No score cap applies; all hard gates pass. |

## Deferred And External Dependency Items

The complete register is maintained in `task_ledger.md`. Closeout carries
these grouped items forward:

| Area | Classification | Items |
|---|---|---|
| `mizar-build` scheduler | `external_dependency_gap` | `CACHE-G-003`, `DEPFPR-G001`, `CACHESTORE-G001`, `PROOFREUSE-G001`, `CLUSTERDB-G004`, `CACHE15-G001`, `PROOFREUSE-G005` scheduler side. |
| `mizar-ir` adapter | `external_dependency_gap` | `CACHE-G-004`, `DEPFPR-G002`, `CACHESTORE-G002`, `PROOFREUSE-G002`, `CLUSTERDB-G005`, `CACHE15-G002`. |
| Artifact/proof publication token | `external_dependency_gap` | `CACHE-G-005`, `DEPFPR-G003`, `CACHESTORE-G003`, `PROOFREUSE-G003`, `CACHE15-G003`, `PROOFREUSE-G005` publication side. |
| Artifact witness schema | `external_dependency_gap` | `PROOFREUSE-G004` for distinct trusted `DischargedBuiltin` publication support. |
| Downstream proof/cache/artifact consumers | `external_dependency_gap` | `DEPFPR-G005`; cache records validation identities only. |
| Accepted-contribution producers | `external_dependency_gap` | `CLUSTERDB-G001`; cache records missing producer fields and does not fabricate accepted status. |
| Fine-grained producer slices | `deferred` | `DEPFPR-G004`; current slices are conservative. |
| Durable cluster-db and view storage | `deferred` | `CACHESTORE-G004`, `CLUSTERDB-G002`, `CLUSTERDB-G003`; current indexes/views are in-memory. |

## Test Expectation Summary

No language specification, `.miz` tests, coverage traceability metadata, or
expectation sidecars were changed to match implementation behavior.
Milestone-owned behavior is covered by Rust unit tests, integration tests,
lint-policy guards, determinism tests, the incremental contract suite,
source/spec audits, architecture-22 audit, module-boundary audit, and explicit
gap records.

## Verification Commands

| Command | Result |
|---|---|
| `cargo fmt --check` | passed |
| `cargo test -p mizar-cache` | passed |
| `cargo clippy -p mizar-cache --all-targets -- -D warnings` | passed |
| `cargo clippy --all-targets --all-features -- -D warnings` | passed |
| `cargo test` | passed |
| `git diff --check` | passed |
| EN/JA pair inventory check | passed |
| sync-placeholder scan | passed |
| `git diff --cached --check` | passed |

## Next-Phase Handoff

Recommended reasoning: `xhigh`.

Prompt:

```text
Start the next integration phase after the mizar-cache closeout commit exists.
Keep `mizar-cache` as the internal optimization owner for CacheKey,
dependency fingerprints, cache records, proof-reuse validation over
`mizar-proof` metadata, and accepted-only cluster-db indexes. Do not let cache
hits, cache records, external evidence, backend diagnostics/logs, timing, or
cluster-db data become proof authority; trusted acceptance still comes only
from `mizar-kernel` KernelCheckResult through proof/status owners.

A good next task is the owning `mizar-build` cache-aware scheduler integration
or the artifact publication-token integration. Treat `mizar-ir` adapter work as
blocked until the `mizar-ir` crate/tasks exist. Preserve all
external_dependency_gap classifications until the owning crate provides the
real seam; do not add placeholders in `mizar-cache`.
```

Raise reasoning above `xhigh` only for simultaneous scheduler/artifact/proof
integration with broad API migration. Lower to `high` for a narrow follow-up
that only backfills a committed task hash or updates one paired documentation
row.
