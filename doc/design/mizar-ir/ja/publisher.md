# mizar-ir Publisher

> 正本は英語です。英語版:
> [../en/publisher.md](../en/publisher.md)。

## 目的

この文書は `mizar-ir` の phase-output publisher 境界を定義する。

publisher は、phase service が complete な producer-local output を seal 済み
`PhaseOutputRef<T>` に変えるための狭い API である。producer context を検証し、
canonical input から content hash と side-table hash を導出し、phase-output lineage を
登録し、storage placement を `IrStorageService` に委譲する。

publisher は scheduling、real driver session、diagnostics rendering、artifact manifest
publication token、proof acceptance、trusted status、verifier policy selection、kernel
acceptance、`mizar-cache` `CacheKey`、dependency fingerprint、proof-reuse validation を
所有しない。

## 入力

publish request は以下を含む:

- target `BuildSnapshotId`。
- producing `PipelinePhase`、`WorkUnit`、`OutputKind`。
- payload とその schema version。
- content hashing と任意の blob placement に使う canonical payload byte sequence。
- parent `PhaseOutputId` と named non-output input hash。
- source map、diagnostic、explanation、documentation attachment の side-table record。
- output origin classification: package source、retained stale snapshot input、validated cache
  input、または open-buffer/editor-only input。
- publication target: current/package output または retained internal-only output。

payload はすでに complete でなければならない。publisher は task-local builder、mutable AST、
部分生成された VC set、部分的な ATP problem、kernel-internal mutable state を受け取らない。

real producer publication token は、owning phase/driver crate が公開するまで
`external_dependency_gap` である。そのため Task 8 は明示 input field に対する crate-local な
producer context validation だけを実装し、placeholder `mizar-driver` API を作ってはならない。

## Snapshot and work-unit validation

publisher は以下を検証する:

- request snapshot が local publisher state に登録されている。
- request snapshot が current publication に対して obsolete ではない。
- work unit が publish request に付属する、または publisher に登録された explicit crate-local
  allowed work-unit context の中で、その phase/output kind に許されている。
- すべての parent output id が同じ current snapshot の seal 済み parent handle に属している。
- pending storage slot が同じ snapshot、phase、work unit、output kind、schema version に
  対して割り当てられている。
- open-buffer/editor-only input と retained stale-snapshot input は current/package publication では
  拒否される。future integration が editor-only handle または stale read を必要とする場合に限り
  internal-only output として retain してよいが、Task 8 は current output の test では拒否する。

currentness は明示 state であり、`BuildSnapshotId` value の比較ではない。publisher は
test と downstream adapter のために crate-local な `register_current_snapshot`、
`mark_obsolete`、`replace_current_snapshot` operation を公開してよいが、hash から
currentness を推測したり、存在しない driver に問い合わせたりしてはならない。

`replace_current_snapshot(old, new)` は、新しい snapshot を current として登録し、
古い snapshot を obsolete として mark することを atomic に行う。古い snapshot は
known なまま残るため、retain された handle は storage を通して読め、cache adapter は
古い output を fail-closed な cache input として encode または validate してよい。
supersede 済み snapshot を再登録しても current publication state として復活してはならない。

obsolete output は storage に retain されている間は読み取り可能なままでよく、cache adapter が
validated cache input として使ってよい。snapshot が obsolete と mark された後、current result
として publish してはならない。

Task 8 は cache reuse を検証してはならない。publish request と異なる snapshot の parent
handle を拒否する。Task 10 は後で validated `mizar-cache` hit を消費し、current snapshot の
新しい seal 済み output として rehydrate してよい。その current-snapshot handle だけを
publisher の parent として渡せる。

retained stale-snapshot origin も同じ規則に従う。後続 cache adapter が検証し current-snapshot
handle を作るまでは internal-only input である。publisher は origin label だけで retained stale
output を current result または package result に昇格してはならない。

## Canonical hashing

publisher は seal 前に 2 つの hash を導出する:

- `content_hash`: canonical payload bytes、seal 済み parent handle/storage metadata から読む
  parent output content hash、named input hash の semantic hash。
- `side_table_hash`: source-map、diagnostic、explanation、documentation side-table record の
  hash。

hash input は domain-separated かつ length-delimited である。collection-valued input は
hash 前に stable key で sort する。publisher は `PhaseOutputId`、storage slot id、storage
generation、memory address、worker id、task id、wall-clock time、temporary filename、retain
counter、cache lookup timing をどちらの hash にも入れてはならない。

diagnostic wording、explanation preview、development-only side table は既定では side-table
input であり、semantic content input ではない。将来の phase が side table を semantic と
宣言する場合は、spec-owned phase contract を通じてのみ行う。

publisher は `mizar-cache` `CacheKey` や dependency fingerprint を作らない。producer-declared
named input hash と parent output relation を記録し、後続 cache adapter が cache ownership を
再実装せずに validated `mizar-cache` result を消費できるようにする。

producer-supplied parent hash field は authority ではない。parent content hash は、publisher が
snapshot と storage metadata を検証した後、seal 済み parent handle から取得しなければならない。

## Side tables

publisher を通じて添付する side table は、seal 済み output の横に保存される stable record で
ある:

- source-map reference と source-range summary。
- diagnostic draft identifier または stable diagnostic record。
- explanation reference。
- documentation attachment identifier。

real `mizar-diagnostics` integration は `external_dependency_gap` である。publisher は
diagnostic-shaped side-table record を保存してよいが、diagnostics crate API、renderer、
severity policy、publication token を捏造してはならない。

side-table attachment は seal 後 immutable である。producer が異なる side table を必要とする
場合、その変化を side-table hash に反映した別の output を publish しなければならない。

## Partial IR を公開しない

publisher は以下を拒否する:

- phase、work unit、output kind label の欠落または空文字。
- canonical payload bytes の欠落。
- runtime output kind が pending slot と一致しない payload。
- incompatible snapshot の parent output。
- obsolete snapshot publication as current output。
- retained stale-snapshot origin as package/current output。
- open-buffer/editor-only output as package/current output。
- unsealed output id または pending slot を公開する試み。

dependent task が受け取るのは seal 済み `PhaseOutputRef<T>` handle だけである。task-local mutable
value や partial IR は観測できない。published artifact は後続の projection boundary が作り、
raw `SurfaceAst`、`TypedAst`、`CoreIr`、`ControlFlowIr`、`VcIr`、`AtpProblem`、kernel-internal
state を含んではならない。

## Cache and proof boundaries

publisher が作る handle は、phase が完了し output が seal されたことを示す通常の internal IR
evidence である。proof acceptance、trusted status、verifier policy approval、kernel
acceptance、proof-reuse validation result ではない。

cache hit は optimization data である。後続 cache adapter は `mizar-cache` validation が成功した
後に限って seal 済み handle を作ってよい。incomplete、unknown、uncacheable、incompatible、
corrupt record は、`PhaseOutputRef<T>` を復元する前に miss となる。

## Errors

| Condition | Handling |
|---|---|
| unknown snapshot | `UnknownSnapshot` として拒否する。 |
| obsolete snapshot used for current publication | `ObsoleteSnapshot` として拒否する。 |
| open-buffer/editor-only output requested as current/package output | `OpenBufferOutput` として拒否する。 |
| wrong phase/work-unit/output-kind/slot metadata | seal 前に拒否し、pending slot を abandon する。 |
| incompatible parent snapshot | storage handle reconstruction の前に拒否する。 |
| missing canonical payload bytes | `MissingCanonicalPayload` として拒否する。 |
| side-table hash mismatch または invalid side-table record | seal 前に拒否する。 |
| storage seal error | handle を publish せず storage error を伝搬する。 |

publisher error はすべて fail closed である。失敗した publish attempt は handle を返さず、
dependent task、cache writer、projection、current publication から output を見えるようにしない。

## Tests

Task 8 は以下を cover しなければならない:

- 同一 canonical input に対する deterministic content hash と side-table hash。
- parent と named-input の canonicalization。
- wrong snapshot、obsolete snapshot、retained stale-origin current publication、
  open-buffer current publication の拒否。
- pending-slot metadata mismatch の拒否。
- seal 済み handle から side table を取得できること。
- failed publication 後に partial output が可視にならないこと。
- publisher-created handle が proof authority、trusted status、cache-key authority、
  dependency-fingerprint ownership を持たないこと。
