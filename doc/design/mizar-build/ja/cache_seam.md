# mizar-build Cache Seam

> 正本は英語です。英語版:
> [../en/cache_seam.md](../en/cache_seam.md).

## Purpose

cache seam は cache-aware scheduling のための `mizar-build` 境界である。
task execution の前に、scheduler が外部で検証済みの cache decision を消費できる
ようにする。これは optimization boundary だけである。

`mizar-cache` は key construction、dependency fingerprints、cache-store lookup
validation、proof-reuse validation を所有する。`mizar-build` は post-validation の
scheduling decision と、caller が current snapshot に publish してよい immutable
output references だけを受け取る。

## Context

- [internal 02](../../internal/ja/02.artifact_store_cache_key_and_manifest.md)
  "Cache Lookup Before Task Execution"
- [architecture 22](../../architecture/ja/22.incremental_verification_contract.md)
- [scheduler.md](./scheduler.md)
- [mizar-cache cache_store.md](../../mizar-cache/ja/cache_store.md)

## Scope

`cache_seam` が所有するもの:

- `TaskId` で key 付けされた caller-supplied task cache decisions。
- scheduler-visible immutable output / diagnostic references として表現された
  validated cache-hit payload。
- normal task execution へ fallback する miss、unavailable、no-key、
  error-as-miss outcomes。
- scheduler input boundary における duplicate cache decisions と graph に存在しない
  task id への decisions の deterministic validation。

`cache_seam` が所有しないもの:

- `CacheKey` construction や serialization。
- dependency fingerprint projection。
- proof-reuse validation や proof witness checks。
- cache-store record lookup、insertion、corruption handling、audit mode。
- phase semantics、semantic acceptance、kernel trust、trusted-status promotion。
- driver sessions、`salsa` queries、`mizar-ir` handles、producer artifact
  publication tokens、manifest writes。

## Gap Classification

| ID | Class | Evidence | Action |
|---|---|---|---|
| CACHE-SEAM-G001 | `source_drift` / `test_gap` | task 18 以前の `scheduler.rs` には disabled cache policy placeholder と `TaskState::CacheHit` があったが、validated-hit input surface や hit-result publication はなかった。 | task 18 が consumer seam、scheduler integration、focused tests を追加する。 |
| CACHE-SEAM-G002 | `external_dependency_gap` | `mizar-cache` を呼ぶ driver-owned `salsa` query boundary は `mizar-driver` がないため存在しない。 | caller-supplied decisions を受け取り、driver dependency や placeholder driver API を追加しない。 |
| CACHE-SEAM-G003 | `external_dependency_gap` | `mizar-ir` がないため、real sealed output handles と cache-to-IR rehydration は利用できない。 | build tests では synthetic immutable output references を使い、IR storage API を創作しない。 |
| CACHE-SEAM-G004 | `external_dependency_gap` | real producer artifact publication tokens は `mizar-build` から利用できない。 | cache hit は scheduler-visible outputs だけを記録してよく、publication authority を mint したり artifact を書いたりしない。 |

## Data Model

seam は deterministic plan として model される:

```rust
struct CacheSchedulingPlan {
    decisions: Vec<CacheTaskDecision>,
}

struct CacheTaskDecision {
    task_id: TaskId,
    outcome: CacheSchedulingOutcome,
}

enum CacheSchedulingOutcome {
    ValidatedHit(ValidatedCacheHit),
    Miss(CacheFallbackReason),
    NoKey(CacheFallbackReason),
    Unavailable(CacheFallbackReason),
    ErrorAsMiss(CacheFallbackReason),
}

struct ValidatedCacheHit {
    output_refs: Vec<CacheOutputRef>,
    diagnostics: Vec<CacheDiagnosticRef>,
}
```

`ValidatedHit` は、caller が `mizar-cache` に要求される exact-key、compatibility、
dependency、proof-reuse、output validation をすでに完了したことを意味する。これは
proof evidence ではない。他の outcome は scheduler に通常どおり task を実行させる。

`CacheFallbackReason` values は粗い scheduling labels である。詳細な miss
classification は `mizar-cache` 内に残り、`mizar-build` は再構築しない。

## Scheduler Integration

task が ready になったとき、scheduler cache handling は次のとおりである:

1. scheduler input boundary で cache decision shape を検証する。duplicate decisions と
   graph に存在しない task id への decisions は diagnostics である。
2. cache scheduling が disabled なら valid cache decisions を無視し、通常実行する。
3. caller が `ValidatedHit` を渡した場合、task を `CacheHit` に遷移させ、validated
   output references と diagnostics を canonical order で記録し、通常の terminal
   scheduler event を emit し、execution は開始しない。
4. publication freshness が current snapshot に対して validated hit を拒否した場合、
   current outputs なしの `Cancelled` を記録し、correctness dependents を block する。
5. 既知 task に decision がない場合、または decision が `Miss`、`NoKey`、
   `Unavailable`、`ErrorAsMiss`、future-compatible miss-like scheduling metadata の
   いずれかなら、task を通常どおり実行する。

`CacheHit` は scheduling 上は `Completed` と同じく dependents を unblock する。
これは dependency-scheduling fact に過ぎない。proof を accept せず、evidence を選ばず、
artifact を publish せず、semantic status も変更しない。

validated hits は modeled worker / resource admission の前に考慮される。そのため
cache hit task は `TaskStarted` event も execution resource telemetry も生成しない。
hit を execution attempt ではなく execution-skip optimization として保つためである。

## Determinism

Cache decisions は `TaskId` で key 付けされる。duplicate decisions と graph に存在しない
task id への decisions は input-boundary diagnostics である。validated hit outputs と
diagnostics は `SchedulerRun` に入る前に sort / deduplicate される。

同じ `TaskGraph`、scheduler configuration、task outcomes、cache decision plan を使う
run は、worker count や completion order が異なっても externally visible records が
deterministic でなければならない。

## Tests

task 18 は focused Rust tests を追加する:

- validated cache hit が execution を skip し、対応する clean execution と同じ
  output references を publish すること。
- cache hit が dependents を unblock し、failure record や proof-authority record を
  作らないこと。
- miss、unavailable、no-key、error-as-miss decisions が通常実行になること。
- disabled cache scheduling が validated hit decisions を無視すること。
- duplicate cache decisions と graph に存在しない task id への decisions が
  scheduler input boundary で失敗すること。
- local cache-key、dependency-fingerprint、proof-reuse、driver、IR、
  publication-token、proof-authority placeholders が存在しないこと。

## Non-Authority Rules

- Cache-aware scheduling は optimization only である。
- cache hit は external validation 後の execution skip 候補であり、semantic
  acceptance ではない。
- cache outputs と artifact records は trusted proof status を昇格しない。
- `mizar-build` は cache keys、dependency fingerprints、proof-reuse validation
  records を構築しない。
- `mizar-build` は `mizar-driver` に依存しない。

## 公開 enum policy

この module が所有する exhaustive public enum exception はない。

| Enum | Policy |
|---|---|
| `CacheSchedulingOutcome` | `#[non_exhaustive]`; downstream callers は wildcard match arms を含めなければならない。 |
| `CacheFallbackReason` | `#[non_exhaustive]`; downstream callers は wildcard match arms を含めなければならない。 |
| `CacheSchedulingPlanDiagnosticKind` | `#[non_exhaustive]`; downstream callers は wildcard match arms を含めなければならない。 |
