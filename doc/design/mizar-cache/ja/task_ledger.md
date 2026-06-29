# mizar-cache Task Ledger

> 正本は英語です。英語版:
> [../en/task_ledger.md](../en/task_ledger.md)。

この ledger は autonomous crate task ごとに 1 行を記録する。各行は、その
task を完了する commit の中で更新する。

| Task | Status | Commit | Reviews | Verification | Notes |
|---:|---|---|---|---|---|
| 0 | done | `8f1d2ab443bc52a50db98f419b527aaa95737d17` | Spec/test/full/source-doc review: fix 後に blocking/high/medium finding なし。 | `git diff --check`, `git diff --cached --check` | Crate plan と ledger。 |
| 1 | done | `fd4a87509cc73ae1e48d4c59c88e5f7d3f33a970` | Spec/test/full/source-doc review: fix 後に blocking/high/medium finding なし。 | `cargo fmt --check`; `cargo test -p mizar-cache`; `cargo clippy -p mizar-cache --all-targets -- -D warnings`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test`; `git diff --check`; `git diff --cached --check` | Crate scaffold と lint-policy guard。正式 mizar-cache scaffold に合わせて ATP closeout metadata drift を補正した。 |
| 2 | done | `06abae61da6c4e0c2d2b45429fbc8cd273fd3202` | Spec/test/full/source-doc review: fix 後に blocking/high/medium finding なし。 | `git diff --check`; `git diff --cached --check` | Spec: `cache_key.md`。 |
| 3 | done | `27dc5d6a851204cf17003f40a28bb564d602ab54` | Spec/test/full/source-doc review: fix 後に blocking/high/medium finding なし。 | `cargo fmt --check`; `cargo test -p mizar-cache`; `cargo clippy -p mizar-cache --all-targets -- -D warnings`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test`; `git diff --check`; `git diff --cached --check` | Cache-key builder。 |
| 4 | done | `cf66ac9d3d74da75b998b4c7f299301a181e9e24` | Spec/test/full/source-doc review: fix 後に blocking/high/medium finding なし。 | `git diff --check`; `git diff --cached --check` | Spec: `dependency_fingerprint.md`。 |
| 5 | done | `a49dcd8e27f9681726b8c6cb6b28b2834276fe95` | Spec/test/full/source-doc review: fix 後に finding なし。 | `cargo fmt --check`; `cargo test -p mizar-cache`; `cargo clippy -p mizar-cache --all-targets -- -D warnings`; `cargo test -p mizar-artifact`; `cargo test -p mizar-vc`; `cargo test -p mizar-proof`; `cargo test -p mizar-build`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test`; `git diff --check` | Dependency-slice と fingerprint computation。 |
| 6 | done | `da62b43ff4d5bccfe3220b8257569a43a6e0be96` | Spec/test/full/source-doc review: fix 後に finding なし。 | `cargo fmt --check`; `cargo test -p mizar-cache`; `cargo clippy -p mizar-cache --all-targets -- -D warnings`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test`; `git diff --check` | Rebuild-trigger evaluation。 |
| 7 | done | `e22a02bb756809b8b71d7e098cf1addf9955684b` | Spec/test/full/source-doc review: fix 後に finding なし。 | `git diff --check`; `git diff --cached --check` | Spec: `cache_store.md`。 |
| 8 | done | `3ce9076b271c923d6106e43e7e56b9cb97029eda` | Spec/test/full/source-doc review: fix 後に finding なし。 | `cargo fmt --check`; `cargo test -p mizar-cache`; `cargo clippy -p mizar-cache --all-targets -- -D warnings`; `cargo test -p mizar-artifact`; `cargo test -p mizar-vc`; `cargo test -p mizar-proof`; `cargo test -p mizar-build`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test`; `git diff --check` | Record store。 |
| 9 | done | pending self-hash | Spec/test/full/source-doc review: fix 後に finding なし。 | `cargo fmt --check`; `cargo test -p mizar-cache`; `cargo clippy -p mizar-cache --all-targets -- -D warnings`; `cargo test -p mizar-artifact`; `cargo test -p mizar-vc`; `cargo test -p mizar-proof`; `cargo test -p mizar-build`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test`; `git diff --check` | Blob store。 |
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
| DEPFPR-G001 | `external_dependency_gap` | `mizar-build` | Dependency-fingerprint consumer が scheduler cache seam に接続できる。 |
| DEPFPR-G002 | `external_dependency_gap` | `mizar-ir` | IR cache adapter が placeholder API なしで dependency-fingerprint input を公開する。 |
| DEPFPR-G003 | `external_dependency_gap` | `mizar-artifact` | Artifact committed publication token integration が存在する。それまでは availability/hash input だけを記録する。 |
| DEPFPR-G004 | `deferred` | `mizar-cache` / producer | より細かい theorem/definition/cluster/notation/mode/attribute producer slice が landing する。task 5 は conservative な published-summary と per-VC 粒度から開始する。 |
| DEPFPR-G005 | `external_dependency_gap` | proof/cache/artifact consumer | proof-reuse metadata の downstream consumer は owner gate 待ち。cache は validation identity だけを記録する。 |
| CACHESTORE-G001 | `external_dependency_gap` | `mizar-build` | placeholder scheduling なしで cache lookup/insert semantics が scheduler に接続できる。 |
| CACHESTORE-G002 | `external_dependency_gap` | `mizar-ir` | IR cache adapter が placeholder API なしで record payload integration を公開する。 |
| CACHESTORE-G003 | `external_dependency_gap` | `mizar-artifact` | Artifact committed publication-token integration が存在する。それまでは現在の cache store は local dependency artifact availability と記録された domain/digest だけを check する。 |
| CACHESTORE-G004 | `deferred` | `mizar-cache` | 後続の cluster-db index storage task が landing する。record store spec は unaccepted registration を publish しない。 |
