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
| 9 | done | `33ae5456409f8083cde9540d6cead8c56b932902` | Spec/test/full/source-doc review: fix 後に finding なし。 | `cargo fmt --check`; `cargo test -p mizar-cache`; `cargo clippy -p mizar-cache --all-targets -- -D warnings`; `cargo test -p mizar-artifact`; `cargo test -p mizar-vc`; `cargo test -p mizar-proof`; `cargo test -p mizar-build`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test`; `git diff --check` | Blob store。 |
| 10 | done | `4a304392831b7f5c8d75d3519835aa8715cf874c` | Spec/test/full/source-doc review: fix 後に finding なし。 | `git diff --check`; `git diff --cached --check` | Spec: `proof_reuse.md`。 |
| 11 | done | `e3cc79af51be48bdce4183b4ceba009432c5fb9a` | Spec/test/full/source-doc review: fix 後に finding なし。 | `cargo fmt --check`; `cargo test -p mizar-cache`; `cargo clippy -p mizar-cache --all-targets -- -D warnings`; `cargo test -p mizar-artifact`; `cargo test -p mizar-vc`; `cargo test -p mizar-proof`; `cargo test -p mizar-build`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test`; `git diff --check` | Proof-reuse validation。 |
| 12 | done | `91d62e4f4c4df39b4a11a93a394b900167eea550` | Spec/test/full/source-doc review: fix 後に finding なし。 | `git diff --check`; `git diff --cached --check` | Spec: `cluster_db.md`。 |
| 13 | done | `8283a90ad6703337fcbfce795137a8a5fd7997b1` | Spec/test/full/source-doc review: fix 後に finding なし。 | `cargo fmt --check`; `cargo test -p mizar-cache`; `cargo clippy -p mizar-cache --all-targets -- -D warnings`; `cargo test -p mizar-artifact`; `cargo test -p mizar-vc`; `cargo test -p mizar-proof`; `cargo test -p mizar-build`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test`; `git diff --check` | Cluster-db writes と origin tracking。 |
| 14 | done | `a341da222a17d949ce09dd4ea59c9bbf461ee778` | Spec/test/full/source-doc review: fix 後に finding なし。 | `cargo fmt --check`; `cargo test -p mizar-cache`; `cargo clippy -p mizar-cache --all-targets -- -D warnings`; `cargo test -p mizar-artifact`; `cargo test -p mizar-vc`; `cargo test -p mizar-proof`; `cargo test -p mizar-build`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test`; `git diff --check` | Import-scoped view と invalidation。durable な `views/` file は deferred のまま。 |
| 15 | done | `add05ca79f060fcadf93d7b48d609726353176cf` | Spec/test/full/source-doc review: finding なし。 | `git diff --check`; `git diff --cached --check` | Scheduler と IR-adapter integration readiness/defer check。paired readiness report を追加し、source stub は追加しない。 |
| 16 | done | `c6878845d0d86d8444c35d7b9c2fab1bc9a2bc1e` | Spec/test/full/source-doc review: fix 後に finding なし。 | `cargo fmt --check`; `cargo test -p mizar-cache`; `cargo clippy -p mizar-cache --all-targets -- -D warnings`; `cargo test -p mizar-artifact`; `cargo test -p mizar-vc`; `cargo test -p mizar-proof`; `cargo test -p mizar-build`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test`; `git diff --check`; `git diff --cached --check` | crate-owned cache surface の determinism と deletability suite。 |
| 17 | done | `c70980eedd1df25c591627c4d4f4958de7fefeed` | Spec/test/full/source-doc review: fix 後に finding なし。 | `cargo fmt --check`; `cargo test -p mizar-cache`; `cargo clippy -p mizar-cache --all-targets -- -D warnings`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test`; `git diff --check`; `git diff --cached --check` | Public-enum forward-compatibility policy、paired docs、lint guard。 |
| 18 | done | `10bcacceca535f71c666906a4d6a419c06a19020` | Spec/test/full/source-doc review: fix 後に finding なし。 | `cargo fmt --check`; `cargo test -p mizar-cache`; `cargo clippy -p mizar-cache --all-targets -- -D warnings`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test`; `git diff --check`; `git diff --cached --check` | Source/spec correspondence audit と derived lint guard。 |
| 19 | done | `ee80385a01fdb567a5a46afdb2ca0942bf02741a` | Spec/test/full/source-doc review: bookkeeping finalization 後に finding なし。 | `git diff --check`; EN/JA pair inventory check; sync-placeholder scan; `git diff --cached --check` | Bilingual documentation sync audit。 |
| 20 | done | `c4f855b37c3e850a030d768f9223faf862704256` | Spec/test/full/source-doc review: fix 後に finding なし。 | `cargo fmt --check`; `cargo test -p mizar-cache`; `cargo clippy -p mizar-cache --all-targets -- -D warnings`; `cargo test -p mizar-artifact`; `cargo test -p mizar-vc`; `cargo test -p mizar-proof`; `cargo test -p mizar-build`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test`; `git diff --check`; `git diff --cached --check` | Incremental verification fail-closed cache contract。 |
| 21 | done | pending self-hash | Spec/test/full/source-doc review: fix 後に finding なし。 | `git diff --check`; EN/JA pair inventory check; sync-placeholder scan; `git diff --cached --check` | Architecture-22 follow-up audit。 |
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
| PROOFREUSE-G001 | `external_dependency_gap` | `mizar-build` | placeholder scheduling なしで scheduler integration が proof-reuse validation を消費できる。 |
| PROOFREUSE-G002 | `external_dependency_gap` | `mizar-ir` | IR cache adapter integration が存在する。それまでは IR placeholder API を作らない。 |
| PROOFREUSE-G003 | `external_dependency_gap` | `mizar-artifact` | artifact-owned committed witness publication token が存在する。それまでは cache は selected witness hash だけを比較する。 |
| PROOFREUSE-G004 | `external_dependency_gap` | `mizar-artifact` | artifact witness schema が distinct trusted `DischargedBuiltin` class を support する。 |
| PROOFREUSE-G005 | `external_dependency_gap` | `mizar-build` / `mizar-artifact` | task 20 は crate-owned cache lookup と proof-reuse validation contract を cover する。cross-crate clean/incremental equivalence は scheduler と artifact publication integration に残る。 |
| CLUSTERDB-G001 | `external_dependency_gap` | checker/artifact producer | concrete accepted-contribution producer field が存在する。なければ task 13 は欠けている field を記録して defer し、accepted status を fabricate しない。 |
| CLUSTERDB-G002 | `deferred` | `mizar-cache` | task 13 は in-memory origin record write、stale-origin removal、aggregate index rebuild を実装する。durable な `cluster-db/` file materialization は persistent cluster-db storage task が scheduled されるまで deferred。 |
| CLUSTERDB-G003 | `deferred` | `mizar-cache` | task 14 は in-memory import-scoped view materialization と invalidation test を実装する。durable な `views/` file は persistent cluster-db storage が landing するまで deferred。 |
| CLUSTERDB-G004 | `external_dependency_gap` | `mizar-build` | scheduler integration は owner gate に残る。placeholder scheduler API は作らない。 |
| CLUSTERDB-G005 | `external_dependency_gap` | `mizar-ir` | IR cache adapter integration は owner gate に残る。placeholder `mizar-ir` API は作らない。 |
| CACHE15-G001 | `external_dependency_gap` | `mizar-build` | cache-aware scheduler seam task 18 は open。task 15 は scheduler hook や cache scheduling trait を追加しない。 |
| CACHE15-G002 | `external_dependency_gap` | `mizar-ir` | `crates/mizar-ir` は存在せず、cache-adapter task 9-10 も open。task 15 は placeholder crate、mock adapter、rehydration API を追加しない。 |
| CACHE15-G003 | `external_dependency_gap` | `mizar-artifact` / `mizar-proof` | artifact-owned committed witness publication proof token は存在しない。task 15 は publication-token shortcut を追加しない。 |
