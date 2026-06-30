# mizar-build Failure State

> 正本は英語です。英語版:
> [../en/failure_state.md](../en/failure_state.md)。

## 目的

この文書は `mizar-build` が所有する failure-state contract を定義する。

Failure state は scheduling、propagation、reporting の境界である。これにより
build scheduler は direct task failure を記録し、blocked work を明示的に保ち、
independent work を継続できるが、source semantics、proof authority、cache
validation、artifact publication authority は変えない。

## 文脈

- [architecture 14](../../architecture/ja/14.parallel_verification_and_scheduling.md)
- [architecture 19](../../architecture/ja/19.failure_semantics.md)
- [architecture 22](../../architecture/ja/22.incremental_verification_contract.md)
- [internal 01](../../internal/ja/01.compiler_driver_and_pipeline_scheduler.md)
- [scheduler.md](./scheduler.md)
- [task_graph.md](./task_graph.md)
- [resource.md](./resource.md)
- [cancel.md](./cancel.md)

## 範囲

`failure_state` が所有するもの:

- direct task failure、blocked task state、それらを決定的に collate するための
  task / snapshot context の build-side record。
- task-graph correctness edge 上での blocking terminal state の有界伝播。
- 「失敗した」と「dependency が failed / cancelled、または required coverage を
  欠いたため実行されなかった」を区別する安定した build-side block reason。
- worker count や completion order に依存しない failure / blocked-work report の
  決定的順序。
- failed または blocked work の output publication 禁止規則。

`failure_state` が所有しないもの:

- parser、resolver、checker、VC、ATP、kernel、proof-policy、artifact の
  semantic failure detection。
- 将来の `mizar-diagnostics` / LSP surface が所有する diagnostic registry、
  diagnostic rendering、LSP publication、failure snapshot storage。
- `mizar-driver` request、session、watch/LSP event stream、phase registry、
  `salsa` query boundary。
- `mizar-ir` output handle、sealed blob、snapshot rehydration。
- `mizar-cache` の `CacheKey` 構築、dependency fingerprint 構築、
  cache-store compatibility check、proof-reuse validation。
- artifact manifest transaction 内部、producer publication token、proof witness
  publication、artifact write。
- proof search、proof acceptance、kernel trust、backend winner selection、
  trusted-status promotion。

## gap 分類

| ID | Class | Evidence | Action |
|---|---|---|---|
| FAIL-G001 | `design_drift` | `todo.md` は `failure_state.md` を要求していたが、task 15 以前には module spec がなかった。 | task 15 でこの仕様と日本語 companion を追加する。 |
| FAIL-G002 | `source_drift` / `test_gap` | `src/failure_state.rs` と focused failure-state tests は task 16 まで存在しない。 | task 16 で build-side record、有界 propagation helper、決定的 ordering、focused tests を実装する。 |
| FAIL-G003 | `external_dependency_gap` | この checkout には stable diagnostic registry や rendered failure snapshot を担う `mizar-diagnostics` crate がない。 | failure record は synthetic / build-side に保ち、diagnostic-registry API を `mizar-build` 内で創作しない。 |
| FAIL-G004 | `external_dependency_gap` | この checkout には `mizar-driver` と real phase-service failure emission が存在しない。 | 将来の phase failure record は値として消費し、driver dependency や placeholder phase-service API を追加しない。 |
| FAIL-G005 | `external_dependency_gap` | `mizar-ir` output storage と phase output handle が存在しない。 | failed / blocked output handle を publish せず、real output handle が存在するまで tests は synthetic に保つ。 |
| FAIL-G006 | `deferred` | non-semantic follow-up work の細かな degraded-mode permission は task 16 以前には実装しない。 | この文書では permission boundary を定義し、task 16 は tests が覆う明示的な場合だけ実装してよい。 |
| FAIL-G007 | `deferred` | cache-aware scheduling は task 18 であり、cache validation は `mizar-cache` が所有する。 | cache miss または default cache error handling は semantic failure ではない。explicit cache-required failure は将来 work に残す。 |

## データモデル

以下の形は contract を示すものであり、最終的な Rust 名とは限らない。

```rust
enum FailureCategory {
    ParseError,
    ResolveError,
    TypeError,
    OverloadAmbiguity,
    ClusterLoop,
    AtpTimeout,
    CertificateRejection,
    KernelRejection,
}

enum BlockReason {
    DependencyFailed,
    DependencyBlocked,
    DependencyCancelled,
    MissingDependencyCoverage,
    ImpossibleResourceRequest,
    NoSchedulablePath,
}

struct FailureSourceOrder {
    package_id: Option<String>,
    module_path: Option<String>,
    source_range: Option<String>,
}

struct BuildFailureRecord {
    task_id: TaskId,
    snapshot: BuildSnapshotId,
    category: FailureCategory,
    phase: PipelinePhase,
    source_order: Option<FailureSourceOrder>,
    severity_rank: usize,
    canonical_order: SchedulerOrderKey,
    diagnostic_code: String,
    stable_detail_key: String,
    rejection_reason: Option<String>,
}

struct BlockedTaskRecord {
    task_id: TaskId,
    snapshot: BuildSnapshotId,
    // impossible resource や remaining schedulable path なしなどの direct
    // scheduler block では空である。
    blocked_by: Vec<TaskId>,
    reason: BlockReason,
    canonical_order: SchedulerOrderKey,
}
```

`FailureCategory` は architecture 19 の安定した phase-level category を反映する。
`mizar-build` はそれらの category を保存し sort してよいが、semantic detection は
producer crates が所有する。`BlockReason` は scheduler-local である。これは task が
なぜ実行されなかったかを説明するものであり、その task 自身の semantic failure として
報告してはならない。Direct failure record は、利用可能な場合は producer diagnostic
ordering metadata を持ち、synthetic または build-side-only failure のために scheduler
order fallback も持つ。

## direct failure と blocked work

Direct failure は、task が実行された、または所有 phase がその task の failed outcome
を報告し、その failure がその task の phase boundary に属することを意味する。

Blocked work は、少なくとも 1 つの required correctness dependency が current run の
output を提供できないか、scheduler がその task はこの run で valid result を生成できないと
直接判断したため、その task が実行されなかったことを意味する。Direct scheduler block には
missing dependency coverage、impossible resource request、remaining pending task に
schedulable path がない場合が含まれる。Dependency 起因の blocked work は predecessor
failure の二重記録ではない。

規則:

1. `Failed` record は failure diagnostics、安定した category metadata、構造化された
   rejection detail を持ってよい。
2. `Blocked` record は 0 個以上の `blocked_by` task ids と build-side `BlockReason` を
   持つが、producer outputs や producer failure diagnostics を blocked task result に
   copy しない。空の `blocked_by` は direct scheduler block に限って valid である。
3. `Cancelled` は `Failed` ではない。correctness dependents を block してよいが、
   semantic failure を証明せず proof diagnostics も生成しない。
4. `Skipped` は failure ではない。parent task が child を不要にした、静的に宣言された
   conditional subgraph に対してのみ unblock する。
5. `CacheHit` は proof evidence ではない。cache validation が unavailable、incomplete、
   unsupported の場合、将来の explicit cache-required mode が別の扱いを定めない限り、
   scheduling は lookup を miss として扱わなければならない。

## 有界 propagation

Failure propagation は task-graph correctness edge のみに従う。

- direct failure は、failed task の output を必要とする dependents を block する。
- cancelled または superseded work は `Cancelled` として記録する。それが correctness
  dependent を block する場合、dependent の block reason は `DependencyCancelled` である。
- blocked task は自身の dependents を block してよいが、各 blocked record は upstream
  chain 全体を複製せず、最も近い blocking predecessor set を保持する。
- missing dependency coverage、impossible resource request、remaining schedulable
  path なしなどの direct scheduler block は、predecessor failure を創作せず
  `BlockReason` を記録する。
- required dependencies が successful、validated cache hits、または明示的に
  unblocking skips のままである independent tasks は継続する。
- failed VC は同一 module 内の independent VC failures を隠してはならない。
- ATP timeout は、別の accepted kernel-evidence result がない限り obligation を open の
  ままにする。proof acceptance にはならない。
- artifact I/O failure は影響を受けた commit / documentation work を block するが、
  memory 内で既に計算済みの proof result を遡って無効化しない。

scheduler は、現在有用な work がすべて terminal になった後に停止してよいが、blocked
tasks を黙って省略してはならない。user や test は、「この task は失敗した」と
「この task はこれらの dependency が failed / cancelled または required coverage を
欠いたため実行されなかった」を区別できなければならない。

## 安定した failure category

`failure_state` は、direct failure の安定した machine-readable classification として
architecture 19 の phase category を使う。

| Category | Build-side meaning |
|---|---|
| `parse_error` | source text を dependent semantic work に使える syntax へ parse できなかった。 |
| `resolve_error` | module、namespace、label、import、symbol resolution が失敗した。 |
| `type_error` | type checking または registration/type prerequisite checking が失敗した。 |
| `overload_ambiguity` | overload/refinement selection が健全な一意 candidate を選べなかった。 |
| `cluster_loop` | registration または cluster expansion が cycle または bounded saturation failure に達した。 |
| `atp_timeout` | ATP/backend work が timeout または同等の non-acceptance により accepted evidence なしで終了した。 |
| `certificate_rejection` | evidence/certificate material が malformed、unsupported、または evidence-level validation で失敗した。 |
| `kernel_rejection` | kernel replay/checking が proof evidence を reject した。 |

`malformed_certificate`、`unsupported_certificate_format`、
`invalid_substitution`、`invalid_sat_refutation` のようなより具体的な rejection reason は
record 上の structured detail のままである。それらは diagnostics を精緻化するが、
propagation と stable ordering に使う phase-level category を置き換えない。

## 決定的 ordering

Failure と blocked-work report は安定した input だけで sort する。Completion order、
worker id、resource-admission timing、cache lookup timing、backend runtime、
temporary path spelling は ordering input ではない。

build-side の canonical ordering key:

1. `BuildSnapshotId`
2. 既知の場合は source package id と module path
3. 既知の場合は source range
4. pipeline phase rank
5. severity
6. diagnostic code
7. stable detail key
8. task graph index
9. `TaskId`
10. structured rejection reason または block reason

source coordinates が利用できない場合は、task graph order と `TaskId` が決定的 fallback
になる。Blocked record は blocked task 自身の order を使い、`blocked_by` task ids を
canonical に sort し、`blocked_by` が空でも direct scheduler block reason を保持する。
Direct failure record は producer が supplied した `source_order` と `severity_rank` を
使い、それらがなければ scheduler の `canonical_order` と `TaskId` に fallback する。

## publication と authority

Failure state は proof authority ではない。

- `Failed` task は failure diagnostics を publish してよいが、phase outputs、
  cache records、artifact drafts、proof witnesses、artifact commits を dependents へ
  publish してはならない。
- `Blocked` task は blocked-state metadata と任意の scheduler diagnostics だけを
  publish する。producer outputs や inherited failure diagnostics を、その task が
  実行されたかのように publish してはならない。
- failed または blocked task は proof trust、semantic acceptance、backend evidence、
  cache reuse、artifact records を昇格しない。
- theorem proving success は、より早い deterministic predecessor failure を無視することに
  依存してはならない。
- resource exhaustion と timeout は non-acceptance outcome である。それらは diagnostics
  になってよいが、所有する proof/kernel phase がその category を emit しない限り
  proof rejection ではない。

Artifact と cache records は `mizar-artifact` と `mizar-cache` が所有する別個の
authority である。`mizar-build` は文書化された seam でそれらの scheduling outcome だけを
消費する。

## task 16 の coverage

task 15 は documentation-only である。task 16 は以下の focused Rust coverage を
追加しなければならない。

- 1 つの failed task が、その output を必要とする correctness dependents だけを
  block すること。
- blocked record が最も近い `blocked_by` dependencies を保持し、upstream chain 全体を
  複製しないこと。
- missing coverage、impossible resource、schedulable path なしに対する direct
  scheduler block が、dependency failure を創作せず表現されること。
- graph 内の別の場所で failure が起きた後も、independent failures と independent
  successful tasks が可視のままであること。
- shuffled ready order、completion order、worker count の下で failure / blocked-record
  ordering が決定的であること。
- failed tasks が diagnostics を持ってよいが output references を持たないこと。
- blocked / cancelled tasks が synthetic producer outputs や inherited producer diagnostics
  を持たないこと。
- cancellation は failure と別物のままであり、必要な場合だけ correctness dependents を
  block すること。
- cache miss / unavailable / default error-as-miss behavior が failure category や proof
  authority を作らないこと。
- `mizar-driver`、`mizar-ir`、diagnostic-registry、cache-key、dependency-fingerprint、
  proof-reuse、publication-token、proof-authority placeholder を導入しないこと。
