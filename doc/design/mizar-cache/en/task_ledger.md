# mizar-cache Task Ledger

> Canonical language: English. Japanese companion:
> [../ja/task_ledger.md](../ja/task_ledger.md).

This ledger records one row per autonomous crate task. Each row is updated in
the task commit that completes that task.

| Task | Status | Commit | Reviews | Verification | Notes |
|---:|---|---|---|---|---|
| 0 | done | `8f1d2ab443bc52a50db98f419b527aaa95737d17` | Spec/test/full/source-doc reviews: no blocking/high/medium findings after fixes. | `git diff --check`, `git diff --cached --check` | Crate plan and ledger. |
| 1 | done | `fd4a87509cc73ae1e48d4c59c88e5f7d3f33a970` | Spec/test/full/source-doc reviews: no blocking/high/medium findings after fixes. | `cargo fmt --check`; `cargo test -p mizar-cache`; `cargo clippy -p mizar-cache --all-targets -- -D warnings`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test`; `git diff --check`; `git diff --cached --check` | Crate scaffold and lint-policy guard; ATP closeout metadata drift corrected for formal mizar-cache scaffold. |
| 2 | done | `06abae61da6c4e0c2d2b45429fbc8cd273fd3202` | Spec/test/full/source-doc reviews: no blocking/high/medium findings after fixes. | `git diff --check`; `git diff --cached --check` | Spec: `cache_key.md`. |
| 3 | done | `27dc5d6a851204cf17003f40a28bb564d602ab54` | Spec/test/full/source-doc reviews: no blocking/high/medium findings after fixes. | `cargo fmt --check`; `cargo test -p mizar-cache`; `cargo clippy -p mizar-cache --all-targets -- -D warnings`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test`; `git diff --check`; `git diff --cached --check` | Cache-key builder. |
| 4 | done | `cf66ac9d3d74da75b998b4c7f299301a181e9e24` | Spec/test/full/source-doc reviews: no blocking/high/medium findings after fixes. | `git diff --check`; `git diff --cached --check` | Spec: `dependency_fingerprint.md`. |
| 5 | done | `a49dcd8e27f9681726b8c6cb6b28b2834276fe95` | Spec/test/full/source-doc reviews: no findings after fixes. | `cargo fmt --check`; `cargo test -p mizar-cache`; `cargo clippy -p mizar-cache --all-targets -- -D warnings`; `cargo test -p mizar-artifact`; `cargo test -p mizar-vc`; `cargo test -p mizar-proof`; `cargo test -p mizar-build`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test`; `git diff --check` | Dependency-slice and fingerprint computation. |
| 6 | done | `da62b43ff4d5bccfe3220b8257569a43a6e0be96` | Spec/test/full/source-doc reviews: no findings after fixes. | `cargo fmt --check`; `cargo test -p mizar-cache`; `cargo clippy -p mizar-cache --all-targets -- -D warnings`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test`; `git diff --check` | Rebuild-trigger evaluation. |
| 7 | done | `e22a02bb756809b8b71d7e098cf1addf9955684b` | Spec/test/full/source-doc reviews: no findings after fixes. | `git diff --check`; `git diff --cached --check` | Spec: `cache_store.md`. |
| 8 | done | `3ce9076b271c923d6106e43e7e56b9cb97029eda` | Spec/test/full/source-doc reviews: no findings after fixes. | `cargo fmt --check`; `cargo test -p mizar-cache`; `cargo clippy -p mizar-cache --all-targets -- -D warnings`; `cargo test -p mizar-artifact`; `cargo test -p mizar-vc`; `cargo test -p mizar-proof`; `cargo test -p mizar-build`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test`; `git diff --check` | Record store. |
| 9 | done | `33ae5456409f8083cde9540d6cead8c56b932902` | Spec/test/full/source-doc reviews: no findings after fixes. | `cargo fmt --check`; `cargo test -p mizar-cache`; `cargo clippy -p mizar-cache --all-targets -- -D warnings`; `cargo test -p mizar-artifact`; `cargo test -p mizar-vc`; `cargo test -p mizar-proof`; `cargo test -p mizar-build`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test`; `git diff --check` | Blob store. |
| 10 | done | pending self-hash | Spec/test/full/source-doc reviews: no findings after fixes. | `git diff --check`; `git diff --cached --check` | Spec: `proof_reuse.md`. |
| 11 | pending | pending | pending | pending | Proof-reuse validation. |
| 12 | pending | pending | pending | pending | Spec: `cluster_db.md`. |
| 13 | pending | pending | pending | pending | Cluster-db writes and origin tracking. |
| 14 | pending | pending | pending | pending | Import-scoped views and invalidation. |
| 15 | pending | pending | pending | pending | Scheduler and IR-adapter integration readiness/defer check. |
| 16 | pending | pending | pending | pending | Determinism and deletability suite. |
| 17 | pending | pending | pending | pending | Public-enum forward-compatibility policy. |
| 18 | pending | pending | pending | pending | Source/spec correspondence audit. |
| 19 | pending | pending | pending | pending | Bilingual documentation sync audit. |
| 20 | pending | pending | pending | pending | Incremental verification fail-closed cache contract. |
| 21 | pending | pending | pending | pending | Architecture-22 follow-up audit. |
| 22 | pending | pending | pending | pending | Module-boundary refactor gate. |
| 23 | pending | pending | pending | pending | Crate exit report and quality review. |

## Deferred / External Dependency Gap Register

| ID | Class | Owner | Unblock condition |
|---|---|---|---|
| CACHE-G-003 | `external_dependency_gap` | `mizar-build` | Cache-aware scheduler seam task lands. |
| CACHE-G-004 | `external_dependency_gap` | `mizar-ir` | IR cache adapter crate/tasks land. |
| CACHE-G-005 | `external_dependency_gap` | `mizar-artifact` / `mizar-proof` | Committed witness publication token exists. |
| DEPFPR-G001 | `external_dependency_gap` | `mizar-build` | Dependency-fingerprint consumers can plug into the scheduler cache seam. |
| DEPFPR-G002 | `external_dependency_gap` | `mizar-ir` | IR cache adapter exposes dependency-fingerprint inputs without a placeholder API. |
| DEPFPR-G003 | `external_dependency_gap` | `mizar-artifact` | Artifact committed publication token integration exists; until then cache records only availability/hash inputs. |
| DEPFPR-G004 | `deferred` | `mizar-cache` / producers | Finer theorem/definition/cluster/notation/mode/attribute producer slices land; task 5 starts from conservative published-summary plus per-VC granularity. |
| DEPFPR-G005 | `external_dependency_gap` | proof/cache/artifact consumers | Downstream consumers of proof-reuse metadata are owner-gated; cache records validation identities only. |
| CACHESTORE-G001 | `external_dependency_gap` | `mizar-build` | Cache lookup/insert semantics can plug into the scheduler without placeholder scheduling. |
| CACHESTORE-G002 | `external_dependency_gap` | `mizar-ir` | IR cache adapter exposes record payload integration without placeholder APIs. |
| CACHESTORE-G003 | `external_dependency_gap` | `mizar-artifact` | Artifact committed publication-token integration exists; until then the current cache store checks only local dependency artifact availability plus the recorded domain/digest. |
| CACHESTORE-G004 | `deferred` | `mizar-cache` | Later cluster-db index storage task lands; record store spec does not publish unaccepted registrations. |
| PROOFREUSE-G001 | `external_dependency_gap` | `mizar-build` | Scheduler integration can consume proof-reuse validation without placeholder scheduling. |
| PROOFREUSE-G002 | `external_dependency_gap` | `mizar-ir` | IR cache adapter integration exists; until then no IR placeholder API is created. |
| PROOFREUSE-G003 | `external_dependency_gap` | `mizar-artifact` | Artifact-owned committed witness publication token exists; until then cache compares selected witness hashes only. |
| PROOFREUSE-G004 | `external_dependency_gap` | `mizar-artifact` | Artifact witness schema supports a distinct trusted `DischargedBuiltin` class. |
| PROOFREUSE-G005 | `deferred` | `mizar-cache` | Full clean/incremental equivalence remains the task-20 gate. |
