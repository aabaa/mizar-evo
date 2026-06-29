# mizar-cache Task Ledger

> 正本は英語です。英語版:
> [../en/task_ledger.md](../en/task_ledger.md)。

この ledger は autonomous crate task ごとに 1 行を記録する。各行は、その
task を完了する commit の中で更新する。

| Task | Status | Commit | Reviews | Verification | Notes |
|---:|---|---|---|---|---|
| 0 | done | `8f1d2ab443bc52a50db98f419b527aaa95737d17` | Spec/test/full/source-doc review: fix 後に blocking/high/medium finding なし。 | `git diff --check`, `git diff --cached --check` | Crate plan と ledger。 |
| 1 | done | pending self-hash | Spec/test/full/source-doc review: fix 後に blocking/high/medium finding なし。 | `cargo fmt --check`; `cargo test -p mizar-cache`; `cargo clippy -p mizar-cache --all-targets -- -D warnings`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test`; `git diff --check`; `git diff --cached --check` | Crate scaffold と lint-policy guard。正式 mizar-cache scaffold に合わせて ATP closeout metadata drift を補正した。 |
| 2 | pending | pending | pending | pending | Spec: `cache_key.md`。 |
| 3 | pending | pending | pending | pending | Cache-key builder。 |
| 4 | pending | pending | pending | pending | Spec: `dependency_fingerprint.md`。 |
| 5 | pending | pending | pending | pending | Dependency-slice と fingerprint computation。 |
| 6 | pending | pending | pending | pending | Rebuild-trigger evaluation。 |
| 7 | pending | pending | pending | pending | Spec: `cache_store.md`。 |
| 8 | pending | pending | pending | pending | Record store。 |
| 9 | pending | pending | pending | pending | Blob store。 |
| 10 | pending | pending | pending | pending | Spec: `proof_reuse.md`。 |
| 11 | pending | pending | pending | pending | Proof-reuse validation。 |
| 12 | pending | pending | pending | pending | Spec: `cluster_db.md`。 |
| 13 | pending | pending | pending | pending | Cluster-db writes と origin tracking。 |
| 14 | pending | pending | pending | pending | Import-scoped view と invalidation。 |
| 15 | pending | pending | pending | pending | Scheduler と IR-adapter integration readiness/defer check。 |
| 16 | pending | pending | pending | pending | Determinism と deletability suite。 |
| 17 | pending | pending | pending | pending | Public-enum forward-compatibility policy。 |
| 18 | pending | pending | pending | pending | Source/spec correspondence audit。 |
| 19 | pending | pending | pending | pending | Bilingual documentation sync audit。 |
| 20 | pending | pending | pending | pending | Incremental verification fail-closed cache contract。 |
| 21 | pending | pending | pending | pending | Architecture-22 follow-up audit。 |
| 22 | pending | pending | pending | pending | Module-boundary refactor gate。 |
| 23 | pending | pending | pending | pending | Crate exit report と quality review。 |

## Deferred / External Dependency Gap Register

| ID | Class | Owner | Unblock condition |
|---|---|---|---|
| CACHE-G-003 | `external_dependency_gap` | `mizar-build` | Cache-aware scheduler seam task が landing する。 |
| CACHE-G-004 | `external_dependency_gap` | `mizar-ir` | IR cache adapter crate/task が landing する。 |
| CACHE-G-005 | `external_dependency_gap` | `mizar-artifact` / `mizar-proof` | committed witness publication token が存在する。 |
