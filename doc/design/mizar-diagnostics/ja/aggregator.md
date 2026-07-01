# Diagnostic Aggregator

> 正本は英語です。英語版:
> [../en/aggregator.md](../en/aggregator.md)。

## 目的

この文書は `mizar-diagnostics` が所有する build-snapshot diagnostic aggregator
を定義する。aggregator は immutable producer batch を consume し、duplicate を除去し、
deterministic な snapshot-local handle を割り当て、後続の CLI、artifact、LSP projection
のために immutable `BuildDiagnosticIndex` を publish する。

aggregator は `DiagnosticRecord` publication の最初の owner である。ただし phase
success、proof acceptance、trusted status、kernel acceptance、driver session
orchestration、artifact mutation、LSP protocol conversion は所有しない。

## Scope

aggregator が所有するもの:

- producer sink から sealed `DiagnosticBatch` を受け取ること。
- obsolete producer snapshot を current publication から filter すること。
- current `DiagnosticDraft` を `DiagnosticRecord` へ project すること。
- message text ではなく stable structured identity によって deduplicate すること。
- deterministic な `DiagnosticId` と `DiagnosticHandle` を assign すること。
- record を deterministic な source-indexed publication order に sort すること。
- immutable `BuildDiagnosticIndex` と tests 用 byte-stable debug snapshot を生成すること。

aggregator が所有しないもの:

- diagnostic code 作成や registry compatibility の決定。
- phase-local draft の構築や producer-local fact の validation。
- CLI formatting、terminal style、source excerpt、line/column conversion。
- LSP UTF-16 range、JSON-RPC publication、overlay、code action。
- phase status、scheduler recovery、proof/kernel/trusted acceptance、build success。
- cache write、artifact commit、source-map path allocation、driver event stream。

## Inputs

task 9 aggregation は概念的に次の input を consume する。

```rust
struct DiagnosticAggregationInput {
    publication_snapshot: BuildSnapshotId,
    batches: Vec<DiagnosticBatch>,
}
```

各 batch は `DiagnosticProducerScope` と validated `DiagnosticDraft` の local-order list
を持つ。sink は batch 内のすべての draft が batch scope と同じ phase/source snapshot
を持つことを既に確認している。aggregation は防御的にその invariant を再確認してよいが、
sink error を user-facing diagnostic output として扱ってはならない。

`publication_snapshot` は current publication の coherent build snapshot である。
aggregator は caller からこれを受け取る。snapshot を作成したり driver session がいつ
coherent かを決めたりしない。

## BuildDiagnosticIndex

task 9 は次と等価な immutable index を公開するべきである。

```rust
struct BuildDiagnosticIndex {
    snapshot: BuildSnapshotId,
    records: Vec<DiagnosticRecord>,
    by_source: BTreeMap<SourceKey, Vec<DiagnosticHandle>>,
    by_id: BTreeMap<DiagnosticId, usize>,
}
```

`records` は canonical publication order である。`by_source` は primary span source ごとに
handle を index する。`by_id` は snapshot-local id から canonical record position へ map
する。具体実装は `SourceKey` を method の背後に隠してよいが、lookup behavior は
deterministic でなければならない。

`mizar-session` の `SourceId` は compiler-native identity である。現時点では persistable
ではなく semantic ordering も実装していない。task 9 では canonical source key を、
session id が serialize できる場合は published-schema string、できない場合は `Debug`
rendering とする。これは deterministic test key であり、durable artifact path ではない。
workspace path normalization は caller または source-map の責務である。

task 9 の index には phase status を意図的に含めない。architecture sketch は phase status
を diagnostics の隣に示しているが、`mizar-diagnostics` は phase semantics や driver
orchestration を所有しない。phase status と index の join は将来の driver/LSP adoption
point であり、ここでは placeholder field ではなく `external_dependency_gap` である。

## Freshness And Obsolete Snapshots

`source_snapshot` が `publication_snapshot` と一致する draft だけが
`BuildDiagnosticIndex` の current record になれる。

それ以外の source snapshot から来た draft は current publication に対して obsolete である。
task 9 はそれらを `DiagnosticFreshness::Current` として publish してはならない。初期実装は
それらを `records` から除外し、tests 用の deterministic stale/obsolete accounting を公開
するべきである。将来 LSP overlay や artifact read のために明示的な stale/historical view
を生成してもよいが、それらは current `BuildDiagnosticIndex` output ではない。

この rule は code、span、message text が同じ場合でも厳格である。stale diagnostic が visible
のままでよいのは、LSP bridge のような crate 外 owner が stale と mark し unsafe action を
suppress する場合だけである。

## Deduplication Identity

deduplication は stable machine-readable field を key にする。

1. `DiagnosticCode`。
2. `PipelinePhase`。
3. `FailureCategory`。
4. primary span の source key、start、end、role、freshness、zero-width intent。
5. `stable_detail_key`。
6. canonical order の structured details。
7. ordered canonical fix payload: suggestion id、producer key、applicability、
   safety、expected text を含む ordered edit、optional command reference、
   snapshot/hash precondition。
8. optional explanation identity。

message text、localized text、rendered label、terminal styling、LSP range、source excerpt、
producer order は identity ではない。message を変更しても、2 つの diagnostic が
deduplicate されるかどうかを変えてはならない。

aggregator は primary span、structured detail、canonical fix payload、explanation identity
が異なる diagnostic を merge してはならない。fix title、diagnostic message、rendered help、
localized text は presentation であり identity ではない。secondary span と note は canonical
representative から保存される。producer は identity-bearing な違いを human text に頼らず、
structured detail、fix、explanation ref に入れるべきである。

同じ dedup identity を持つ複数 draft が異なる presentation payload を持つ場合、aggregator
は full presentation payload を tie-breaker として sort し、deterministic に representative
を選ぶ。この tie-breaker は表示される human text だけに影響し、diagnostic identity ではない。

## Ordering And Handles

aggregation は producer order と batch order に依存してはならない。同じ content を持つ input
batch や draft を shuffle しても、byte-identical な index と同じ handle が生成されなければ
ならない。

canonical record order は次の通り。

1. Primary source key。
2. Primary range start。
3. Primary range end。
4. `PipelinePhase` の phase order。
5. Registry severity order。
6. `DiagnosticCode`。
7. `FailureCategory`。
8. `stable_detail_key`。
9. Dedup identity key。
10. Full presentation tie-breaker。

deduplication と sorting の後、`DiagnosticId` は canonical record order の dense zero-based
ordinal として assign される。`DiagnosticHandle` は
`(publication_snapshot, DiagnosticId)` としてだけ意味を持つ。consumer は unrelated snapshot
をまたいで bare id を persist してはならず、durable workflow は `DiagnosticCode` と
structured field を key にしなければならない。

## Debug Snapshot

`BuildDiagnosticIndex::debug_snapshot()` は deterministic test output であり CLI rendering
ではない。LF line ending、color 無し、localized field name 無しで、field order は次の通り。

1. `kind=index`。
2. `snapshot`。
3. `record_count`。
4. `obsolete_count`。
5. canonical order の `record[0]`、`record[1]`、...。各 entry は対応する
   `DiagnosticRecord::debug_snapshot()` から trailing newline を削除し、内部 newline を
   `\n` として escape して embed する。

task 9 が obsolete accounting entry を公開する場合、その debug form は source snapshot、
producer name、local draft ordinal、draft debug snapshot で sort され、`record[n]` line の
後に `obsolete[n]` line として render されなければならない。snapshot は memory address、
thread id、hash-map iteration order、key としての localized text、process-local ordering
を含めてはならない。

## Boundary Rules

- aggregation は record を publish するが、render はしない。
- diagnostic severity は ordering に影響してよいが、phase success や proof/kernel
  acceptance を決定できない。
- `DiagnosticCode` は stable tool identity のままである。message text は deduplication、
  lookup、consumer behavior の authority ではない。
- 既存 lexer/frontend/parser/resolver diagnostic migration は、real consumer が shared index
  を採用するまで deferred のままである。task 9 はそれらの crate の placeholder adapter や
  stub API を追加してはならない。
- driver、LSP、artifact、resolver adoption は external dependency である。準備ができていない
  場合は gap を記録し、`mizar-diagnostics` を独立に保つ。
