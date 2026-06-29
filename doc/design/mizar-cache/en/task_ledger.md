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
| 3 | done | pending self-hash | Spec/test/full/source-doc reviews: no blocking/high/medium findings after fixes. | `cargo fmt --check`; `cargo test -p mizar-cache`; `cargo clippy -p mizar-cache --all-targets -- -D warnings`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test`; `git diff --check`; `git diff --cached --check` | Cache-key builder. |
| 4 | pending | pending | pending | pending | Spec: `dependency_fingerprint.md`. |
| 5 | pending | pending | pending | pending | Dependency-slice and fingerprint computation. |
| 6 | pending | pending | pending | pending | Rebuild-trigger evaluation. |
| 7 | pending | pending | pending | pending | Spec: `cache_store.md`. |
| 8 | pending | pending | pending | pending | Record store. |
| 9 | pending | pending | pending | pending | Blob store. |
| 10 | pending | pending | pending | pending | Spec: `proof_reuse.md`. |
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
