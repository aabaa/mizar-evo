# mizar-build Artifact Commit Boundary

> 正本は英語です。英語版: [../en/artifact_commit.md](../en/artifact_commit.md)。

## Purpose

この文書は `mizar-build` が所有する deterministic artifact-commit boundary を
定義する。

`mizar-build` は、完了した `ArtifactCommit` tasks を artifact manifest transaction
へ渡す scheduler-side order を決める。artifact schema、stable artifact hash、
proof witness payload、atomic manifest writer は所有しない。これらは
`mizar-artifact` の責務である。

## References

- [scheduler.md](./scheduler.md)
- [internal 01](../../internal/ja/01.compiler_driver_and_pipeline_scheduler.md)
- [internal 02](../../internal/ja/02.artifact_store_cache_key_and_manifest.md)
- [architecture 11](../../architecture/ja/11.artifact_and_incremental_build.md)
- [architecture 14](../../architecture/ja/14.parallel_verification_and_scheduling.md)
- [`mizar-artifact` manifest spec](../../mizar-artifact/ja/manifest.md)

## Scope

build-side commit boundary が所有するもの:

- scheduled `ArtifactCommit` tasks から導かれる manifest updates の決定的 ordering。
- caller が供給する module manifest entries を
  `mizar_artifact::manifest::ManifestTransaction` へ渡すこと。
- build coordinator が供給する opaque freshness guard / check を渡すこと。
- どの module updates が manifest transaction へ渡されたかの決定的 record。
- manifest transaction error の fail-closed propagation。

build-side commit boundary が所有しないもの:

- `VerifiedArtifact`、`ModuleSummary`、`RegistrationSummary`、proof witness、
  development-artifact payload file の write または validation。
- `mizar-artifact` manifest schema validation、hash construction、reference
  validation、atomic file replacement。
- producer-owned phase-15 artifact projection。
- proof authority、proof witness acceptance、kernel acceptance、trusted status promotion。
- cache-key construction、dependency-fingerprint construction、proof-reuse
  validation、cache promotion。
- `mizar-driver` build session、event stream、registry dispatch、`salsa`
  query boundary。

## Gap Classification

| ID | Class | Evidence | Action |
|---|---|---|---|
| COMMIT-G001 | `source_drift` / `test_gap` | task 17 以前は `artifact_commit.md` が存在せず、`mizar-build` には modeled scheduler `ArtifactCommit` tasks だけがあった。 | task 17 で build-side manifest-transaction consumer boundary と focused tests を追加する。 |
| COMMIT-G002 | `external_dependency_gap` | real producer phase-15 artifact projection と publication token は `mizar-build` から利用できない。 | caller から already-written `mizar-artifact` module entries を受け取り、producer payload や publication token を創作しない。 |
| COMMIT-G003 | `external_dependency_gap` | driver sessions、event streams、`salsa` freshness state は `mizar-build` の外側にある。 | opaque freshness guard / check を受け取り、`mizar-driver` に依存せず driver-owned API を追加しない。 |
| COMMIT-G004 | `external_dependency_gap` | real IR sealed output handles と snapshot rehydration は build-owned seam 経由では利用できない。 | IR handle placeholder を追加しない。tests は already-published artifact files と manifest entries を使う。 |

## Data Model

Rust 名は変わってよいが、build-side boundary は次の形を持つ:

```rust
struct ManifestCommitRequest {
    artifact_root: PathBuf,
    seed_manifest: ArtifactManifest,
    freshness_guard: Option<String>,
    updates: Vec<ScheduledManifestUpdate>,
}

struct ScheduledManifestUpdate {
    task_id: TaskId,
    graph_index: usize,
    entry: ModuleArtifactEntry,
}

struct ManifestCommitSummary {
    manifest: ArtifactManifest,
    manifest_hash: Hash,
    modules: Vec<CommittedModuleUpdate>,
}

struct CommittedModuleUpdate {
    task_id: TaskId,
    graph_index: usize,
    module: ModuleSummaryIdentity,
    source_file: String,
    artifact_file: String,
}
```

`ModuleArtifactEntry`、`ArtifactManifest`、manifest hash、atomic transaction
result は `mizar-artifact` から来る。`mizar-build` はそれらの値を保存、sort、
pass-through してよいが、schema semantics を再構築してはならない。

## Canonical Ordering

Manifest updates は transaction に stage される前に sort される。canonical key は:

1. `ModuleArtifactEntry.module` の stable module identity。
2. source file path。
3. scheduler graph index。
4. `TaskId`。

この key により、commit staging は worker completion order と、caller が completed tasks
を報告した入力順から独立する。manifest transaction は自身の schema rule により
manifest entries をさらに sort してよい。その artifact-owned sort は `mizar-build` が
再実装しない。

同じ module の duplicate update は artifact-owned transaction が受理する場合だけ受理される。
conflicting duplicate は manifest error のままである。

## Commit Protocol

1. caller は package artifact root、seed manifest、optional freshness guard、
   payload files がすでに `mizar-artifact` を通じて書かれている module updates を供給する。
2. `mizar-build` は updates を canonical commit order で sort する。
3. `mizar-build` は `mizar-artifact` の `ManifestTransaction` を開始する。
4. `mizar-build` は sorted `ModuleArtifactEntry` を、artifact schema internals を
   rewrite または revalidate せずに stage する。
5. `mizar-build` は caller-supplied freshness check 付きで transaction を commit する。
6. 成功時、返された manifest と module records は決定的に sort され、build-side
   commit results として報告できる。
7. manifest error があれば error を返し、build-side artifact または proof authority
   status を昇格しない。

atomic visibility rule は manifest transaction が所有する。transaction が obsolete、
interrupted、または reference validation failure になった場合、reader は以前に commit
された manifest を使い続ける。reader は manifest から読み始めなければならないため、
unreferenced files は無視される。

## Non-Authority Rules

- committed manifest は `mizar-artifact` による reachability と hash consistency だけを
  示す。proof authority は作らない。
- `mizar-build` は artifact record、manifest record、skipped task、cache hit を
  semantic acceptance または trusted proof status として扱ってはならない。
- artifact commit 後の cache promotion は task 17 の scope 外である。
- missing producer tokens は `external_dependency_gap` として記録し、synthetic token で
  置き換えない。

## Tests

task 17 は focused Rust tests を追加する:

- already-written module artifacts を shuffled update order で commit しても、同一の
  manifest と commit records が得られること。
- obsolete freshness check を拒否し、previous manifest が visible のまま残ること。
- conflicting または invalid manifest transaction error を、build-side authority record を
  publish せずに propagate すること。
- driver、IR、manifest-transaction-internal、artifact-schema、hash-construction、
  proof-witness-validation、cache-key、dependency-fingerprint、proof-reuse、
  proof-authority、producer-publication-token placeholders が source boundary にないこと。

## 公開 enum policy

この module が所有する exhaustive public enum exception はない。

| Enum | Policy |
|---|---|
| `ArtifactCommitError` | `#[non_exhaustive]`; downstream callers は wildcard match arms を含めなければならない。 |
