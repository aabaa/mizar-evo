# mizar-build Cancellation

> 正本は英語です。英語版:
> [../en/cancel.md](../en/cancel.md)。

## 目的

この文書は `mizar-build` が所有する cancellation contract を定義する。

Cancellation は scheduling と freshness の制御である。`mizar-build` は
obsolete な作業を停止または破棄できるが、決定的な結果、clean-build
equivalence、proof authority 境界、cache validation 境界、partial
publication 禁止規則を保たなければならない。

## 文脈

- [architecture 14](../../architecture/ja/14.parallel_verification_and_scheduling.md)
- [architecture 22](../../architecture/ja/22.incremental_verification_contract.md)
- [internal 01](../../internal/ja/01.compiler_driver_and_pipeline_scheduler.md)
- [scheduler.md](./scheduler.md)
- [resource.md](./resource.md)

## 範囲

`cancel` が所有するもの:

- scheduler run と task-graph snapshot の build-side cancellation state。
- cooperative cancellation token と単調増加する cancellation generation。
- pending、ready、running、completed-obsolete work に対する決定的な
  cancellation decision。
- scheduler-visible result、diagnostic、cache record、artifact commit attempt
  の前に行う freshness check。
- cancelled task が保持していた resource reservation の release handoff。

`cancel` が所有しないもの:

- `mizar-driver` の build request、watch/LSP event stream、live session、
  `salsa` query、phase-service registry。
- worker thread の hard kill や OS process supervision。
- ATP/backend process の spawn、termination、stdout/stderr capture、backend
  protocol detail。
- `mizar-ir` の output storage、sealed handle、snapshot-handle rehydration。
- `mizar-cache` の `CacheKey` 構築、dependency fingerprint 構築、
  cache-store compatibility check、proof-reuse validation。
- artifact schema ownership、manifest transaction 内部、producer-owned
  publication token、artifact write。
- proof search、proof acceptance、kernel trust、backend winner selection、
  trusted-status promotion。

## gap 分類

| ID | Class | Evidence | Action |
|---|---|---|---|
| CAN-G001 | `design_drift` | `todo.md` は `cancel.md` を要求していたが、task 13 以前には module spec がなかった。 | task 13 でこの仕様と日本語 companion を追加する。 |
| CAN-G002 | `source_drift` / `test_gap` | `src/cancel.rs` と cancellation tests は task 14 以前には存在しなかった。 | task 14 で versioned token、snapshot invalidation、publication guard、scheduler integration、focused tests を追加する。 |
| CAN-G003 | `external_dependency_gap` | この checkout には `mizar-driver` request/session/watch/LSP integration が存在しない。 | cancellation を input-driven かつ snapshot-oriented に保ち、driver dependency や placeholder driver API を追加しない。 |
| CAN-G004 | `external_dependency_gap` | この checkout には `mizar-ir` output storage と snapshot-handle rehydration が存在しない。 | real IR seam が存在するまで cancelled/obsolete output handling は synthetic に保ち、IR handle や storage API を創作しない。 |
| CAN-G005 | `external_dependency_gap` | ATP/backend process manager は `mizar-build` の外側にある。 | cooperative cancellation と backend cancellation outcome だけを model 化し、OS process を spawn、kill、supervise しない。 |
| CAN-G006 | `external_dependency_gap` | real producer artifact publication token は `mizar-build` が利用できる状態ではない。 | freshness と partial-publication 禁止 guard を仕様化し、fake publication token を作らない。 |
| CAN-G007 | `deferred` | cache-aware scheduling は task 18 であり、cache key、fingerprint、proof-reuse validation は `mizar-cache` が所有する。 | obsolete work の再利用は将来の external cache validation を通じてのみ行う。ここで cache internals を再実装しない。 |
| CAN-G008 | `deferred` | 詳細な failure taxonomy と bounded propagation は tasks 15-16 である。 | cancellation は terminal scheduling state として定義し、詳細な failure category は `failure_state.md` に残す。 |

## データモデル

以下の形は cancellation contract を示すものであり、最終的な Rust 名とは限らない。

```rust
struct CancellationInput {
    current_snapshot: BuildSnapshotId,
    superseded_snapshots: Vec<BuildSnapshotId>,
    explicit_task_cancellations: Vec<TaskId>,
    generation: CancellationGeneration,
}

struct CancellationToken {
    snapshot: BuildSnapshotId,
    generation: CancellationGeneration,
    reason: Option<CancellationReason>,
}

enum CancellationReason {
    SupersededSnapshot,
    ExplicitRequest,
    Shutdown,
    BudgetPolicy,
}

enum CancellationDecision {
    Continue,
    CancelBeforeStart,
    CancelAtCheckpoint,
    DiscardObsoleteResult,
    CommitAlreadyStarted,
}
```

すべての cancellation decision は task graph version と `BuildSnapshotId` に
scope される。将来の driver は snapshot stream を供給してよいが、
`mizar-build` は `mizar-driver` に依存せず identifier と decision を消費する。

`current_snapshot` が既知で graph snapshot と異なる場合、`mizar-build` はその
graph を scheduling 上すでに superseded として扱う。pending と ready work は
開始前に cancel され、running work は次の safe checkpoint を観測し、既に完了済みの
result も scheduler-visible output になる前に publication freshness guard を通る。
例外は atomic transaction が既に開始された artifact commit である。`mizar-build` は
その transaction を中断せず、visible outcome は deterministic commit boundary が決める。

## versioned cancellation

すべての scheduled task はちょうど 1 つの task graph version と 1 つの
`BuildSnapshotId` に属する。cancellation state は scheduler run 内で単調増加する
generation によって versioned である。

規則:

1. より新しい source/dependency snapshot は古い snapshot を supersede してよい。
2. snapshot が cancelled になると、その snapshot のすべての task は同じかより新しい
   generation の cancellation token を観測する。
3. 同じ snapshot または task への repeated cancellation は idempotent である。
4. cancellation decision は snapshot、task graph order、`TaskId` によって
   決定的に sort される。
5. `Cancelled` は terminal scheduling state であり、proof status、cache
   validation result、semantic acceptance、artifact publication state ではない。

Cancellation はどの作業を実行するかを変えてよいが、cancel されていない結果の
意味を変えてはならない。

## checkpoint と状態遷移

Cancellation は in-process の `mizar-build` 作業に対して cooperative である。

| 既存 state | Cancellation behavior |
|---|---|
| `Pending` | ready、running、output publication に進まず `Cancelled` へ移る。 |
| `Ready` | dispatch/admission から外して `Cancelled` へ移る。 |
| `Running` | 次の phase boundary または safe checkpoint まで進み、その後 `Cancelled` へ移り resource を正確に一度 release する。 |
| completed but obsolete | 将来の cache seam が current snapshot への reuse を外部 validation する場合を除き、publication 前に scheduler-visible output を破棄する。 |
| artifact commit not started | atomic publication transaction を開く前に cancel する。 |
| artifact commit already started | `mizar-build` 内では transaction を中断しない。visible outcome は deterministic commit boundary が決める。 |

safe checkpoint には scheduler dispatch boundary、resource-admission boundary、
task phase boundary、cache-probe boundary、publication freshness check が含まれる。
長時間実行される phase がより細かい cancellation を必要とする場合、その phase
service が自身の safe checkpoint を公開しなければならない。`mizar-build` は
thread を kill してその挙動を強制してはならない。

external ATP/backend process については、backend manager が termination または
cancellation outcome を報告した後に、`mizar-build` が task cancelled を記録してよい。
process creation と termination はこの crate の外側に残る。

## publication と freshness

Cancellation は current-snapshot publication を保護する:

- cancelled または superseded work からの diagnostics は current diagnostics として
  publish してはならない。
- cancelled または superseded work からの proof status は accepted、trusted、
  より authoritative な状態になってはならない。
- obsolete snapshot の phase output、cache record、artifact draft、commit request は
  current result として publish してはならない。
- LSP/watch consumer は新しい snapshot が置き換えるまで既に表示されている stale
  diagnostics を保持してよいが、`mizar-build` は stale work が current work を
  装えないよう snapshot ごとに result を label し collate しなければならない。
- open editor buffer は source snapshot であり artifact ではない。
- cancellation が scheduler run を中断したために partial artifact が visible に
  なってはならない。

cancelled task が resource を reserve していた場合、task が `Cancelled` に到達した
時点で scheduler/resource boundary がそれを release する。resource release は
output を publish しない。

## cache、artifact、proof 境界

Cancellation は cache validation ではない。obsolete snapshot から完成した結果が
後続 snapshot で有用になるのは、将来の cache-aware seam が `mizar-cache` に問い合わせ、
外部で validation された decision を受け取った場合だけである。この crate はその reuse
を正当化するために local cache key、dependency fingerprint、proof reuse validator を
構築してはならない。

Cancellation は artifact authority ではない。stale commit attempt の開始を防いで
よいが、manifest transaction、artifact schema、publication token を所有しない。

Cancellation は proof authority ではない。cancelled task は failure、success、
incompleteness、acceptance を証明しない。cancelled snapshot のその task の作業を
current として publish してはならない、という scheduling 上の事実だけを表す。

## 決定性

Cancellation timing が変えてよいのは latency と wasted work の量だけである。

同じ task graph、snapshot sequence、cancellation generation、synthetic task
outcome、resource configuration、cache decision に対して、collation 後の canonical
scheduler run は worker count と worker completion order をまたいで同一でなければ
ならない。

canonical diagnostics、artifact ordering、cache publication ordering、proof status
ordering は、worker が cancellation token を観測した瞬間に依存してはならない。
cancellation telemetry は破棄された作業を説明してよいが、semantic ordering key ではない。

## task 14 の coverage

task 14 は以下の focused Rust coverage を追加する:

- pending と ready work が開始前に cancel されること。
- running work が safe checkpoint でのみ cancel されること。
- cancellation generation が単調に進み、cancelled snapshot の tasks へ同じか
  より新しい token が伝播し、cancellation decision が snapshot、task graph order、
  `TaskId` によって canonical order になること。
- 同じ snapshot または task の repeated cancellation が idempotent であること。
- snapshot supersession が obsolete completed result を publication 前に破棄すること。
- cancelled work が current diagnostic、output、cache record、artifact commit attempt を
  生成しないこと。
- admitted cancelled task の resource が正確に一度 release されること。
- shuffled ready/completion order と異なる worker count の下で cancellation result が
  決定的であること。
- modeled atomic transaction の開始前と開始後における commit-boundary 挙動。
- `mizar-driver`、`mizar-cache`、`mizar-ir`、OS process、fake artifact token、
  proof-authority placeholder が存在しないこと。

## non-authority 規則

- cache hit は external cache validation 後に限る execution-skip candidate であり、
  cancellation は hit を validation しない。
- artifact record や cancellation event は trusted status を昇格しない。
- cancelled proof task は kernel acceptance、backend victory、semantic rejection として
  数えない。
- `mizar-build` は `mizar-driver` から独立したままである。driver-owned session がこの
  contract を呼び出してよいが、この crate はそれらに依存してはならない。

## 公開 enum policy

この module が所有する exhaustive public enum exception はない。

| Enum | Policy |
|---|---|
| `CancellationReason` | `#[non_exhaustive]`; downstream callers は wildcard match arms を含めなければならない。 |
| `CancellationDecision` | `#[non_exhaustive]`; downstream callers は wildcard match arms を含めなければならない。 |
| `CancellationCheckpoint` | `#[non_exhaustive]`; downstream callers は wildcard match arms を含めなければならない。 |
