# mizar-ir Storage

> 正本は英語です。英語版:
> [../en/storage.md](../en/storage.md)。

## 目的

この文書は `mizar-ir` の storage 境界を定義する。

`mizar-ir` は、seal 済み phase output の compiler-internal storage slot と、
型付き `PhaseOutputRef<T>` handle を所有する。snapshot と IR-local identity は
`mizar-session` と `mizar-ir::identity` から消費する。artifact publication、
cache-key construction、dependency fingerprint、proof acceptance、trusted
status、verifier policy selection、kernel acceptance は所有しない。

storage は最適化と lifetime の境界である。出力を resident memory と
content-addressed blob の間で移しても、source semantics、artifact schema、
cache authority、proof status は変わってはならない。

## Storage model

storage service は phase output ごとに 1 つの record を保存する:

- identity metadata: `PhaseOutputId`、`BuildSnapshotId`、`PipelinePhase`、
  `WorkUnit`、`OutputKind`、content hash、side-table hash、lineage。
- 保存 payload の runtime kind tag と schema version。
- typed payload location: resident memory または internal blob。
- source-map、diagnostic、explanation、documentation-attachment 参照の
  side-table record。
- storage-visible lifetime state: sealed/unsealed、retain owner、protected
  root、collection status。

current/obsolete publication eligibility は storage が所有しない。後続の publisher と
snapshot-replacement logic が currentness check と明示的な protection root を供給する。
storage は owner の `BuildSnapshotId` と retention state を記録するが、どの snapshot が
current かは決めない。

consumer に返す handle は `PhaseOutputRef<T>` である。型 parameter は出力 payload
に対する compile-time expectation である。runtime storage は `Arc<T>` を返す前に
kind tag を確認し、handle が byte を別の IR kind として解釈できないようにする。

slot number、arena id、memory address、worker id、temporary filename、retain
counter は storage implementation detail である。安定 identity input ではなく、
published artifact に出してはならない。

## Slot lifecycle and sealing

storage は 2 段階の lifecycle を使う:

1. producer が 1 つの snapshot、phase、work unit、output kind に対する private
   pending slot を割り当てる。
2. publisher が producer context を検証した後、producer は complete payload、
   side table、lineage で slot をちょうど一度だけ seal する。

`PhaseOutputRef<T>` は seal した場合だけ返る。pending slot は他 task、artifact
projection、cache writing、current publication から不可視である。pending slot を
id で読むことは `UnsealedOutput` として拒否する。slot の二重 seal は
`AlreadySealed` として拒否する。seal 済み output の変更は support しない。
payload を保存する前に、storage は supplied lineage が identity module によって割り当てられた
canonical `PhaseOutputId` とまだ一致していることを検証する。clone または mutate された
lineage の content hash、side-table hash、parent、named input、producer metadata が output id
と一致しなくなっている場合、`InvalidLineage` として拒否し、その pending slot を abandoned にする。

seal 済み handle は immutable である:

- payload location は resident または blob-backed でよいが、logical payload は固定。
- side-table record は固定され、その hash で識別される。
- content hash と side-table hash は固定。
- lineage は固定。
- retain と collect state が変えてよいのは storage lifetime だけで、output value
  ではない。

allocation 後に seal が失敗した場合、その pending slot は abandoned になり回収可能
となる。abandoned pending slot が current output になることはない。

## Typed handles

`PhaseOutputRef<T>` は、producer に問い合わせず read を検証できるだけの metadata を
含まなければならない:

- `PhaseOutputId`。
- owner の `BuildSnapshotId`。
- phase、work unit、output kind。
- content hash と side-table hash。
- storage generation または同等の stale-handle guard。
- `T` に期待される runtime kind tag。

`get<T>(&PhaseOutputRef<T>)` は、slot が存在し、seal 済みで、collect されておらず、
handle に記録された同じ generation に属し、期待される runtime kind tag を持つ場合だけ
成功する。type mismatch は internal API error であり fail closed しなければならない。
storage は別の IR kind として downcast したり deserialize したりしてはならない。

blob-backed payload の typed read path は、まず handle generation、runtime kind tag、
schema version、blob content hash を検証する。その後、その output kind と `T` に正確に
登録された schema binding だけを通じて decode する。schema mismatch、kind mismatch、
corrupt bytes、codec data の欠落は storage error として fail closed する。read path は
別 kind を推測したり、byte を再解釈したり、cache validation を合成したりしてはならない。

handle は proof evidence ではない。後続 adapter が cache から rehydrate した handle を
含め、handle を持つことは proof status や trusted status を昇格しない。

## Placement policy

resident-set rule は次の通り:

- handle metadata、identity index、lineage、小さな side-table index は resident に
  保つ。
- 大きな payload byte は content-addressed internal blob へ退避してよい。
- collection は unreferenced payload storage を消すが、published artifact や
  source-level semantics は変えない。

既定の spill threshold は **canonical payload bytes で 64 KiB** とする。canonical byte
length が 64 KiB を超える payload は既定で blob-backed になり、64 KiB 以下の payload は
既定で resident に残る。この threshold は性能と memory の policy であり、identity rule
ではない。変更しても `PhaseOutputId`、content hash、proof status、artifact projection は
変わってはならない。

Task 6 は明示的な `StoragePolicy` でこの policy を実装し、test や将来の build profile が
より低い、または高い threshold を選べるようにする。この crate は blob path を artifact
data として公開してはならない。blob reference は payload hash と schema version を key と
する internal content-addressed reference である。

## Side tables

side table は seal 済み output の横に保存し、published artifact payload の中には置かない。
以下の stable reference を含めてよい:

- source map と source range。
- diagnostic draft または diagnostic identifier。
- explanation request reference。
- documentation attachment identifier。

real `mizar-diagnostics` integration は、その crate が存在し integration seam を公開するまで
`external_dependency_gap` である。それまでは、diagnostics crate API を捏造せず、stable な
side-table record を `mizar-ir` が保存する。

side-table change は semantic payload byte とは別に hash する。後続の publisher spec が、
side table がいつ semantic input になるかを定義する。storage は渡された hash を保存し、
cache compatibility は決めない。

## Retain and collect

storage lifetime は明示的である。seal 済み output は次の owner により retain され得る:

- input handle をまだ必要とする dependent task。
- watch/LSP semantic snapshot。
- diagnostic または explanation request。
- cache writer。
- seal 済み output を読んでいる artifact projection task。

`retain` は `PhaseOutputId` の lifetime guard または registered owner を作る。`release` は
その owner を外す。current snapshot root、active cache writer、artifact projection reader は、
hidden driver や downstream query ではなく、明示的な retain owner または caller-provided
protection root として storage に表現される。`collect` が drop してよいのは、seal 済み
または abandoned であり、retain owner がなく、caller-provided protection root の外にあり、
active in-crate guard に覆われていない output だけである。

downstream ownership state が不明な場合、caller は該当 output を retain または protect し続け
なければならない。collection は fail-closed である。不明な protection は collectable set から
外してまだ live と扱い、storage は liveness を調べるために `mizar-driver`、
`mizar-diagnostics`、publisher-token、cache-token、artifact-token API を捏造してはならない。

batch build では、dependent task は downstream output が seal されるか task が失敗した時点で
owner を release する。watch と LSP では、古い snapshot handle は stale diagnostic、
explanation request、semantic snapshot reader に retain されている間は読み取り可能でよい。
release 後、collection は古い payload を除去してよく、その後の read は
`CollectedOutput` として失敗する。

collection は冪等である。同じ snapshot と owner state に対して 2 回実行しても、2 回目の
summary は追加 drop が 0 であること以外変わってはならない。collection は seal 済み payload
content、published artifact、dependency fingerprint、proof status、cache validation result を
変更してはならない。

## Snapshot replacement boundary

Task 13 は current/obsolete snapshot state を追加する。storage はすべての slot に owner の
`BuildSnapshotId` を記録し、lifetime state を current-publication eligibility から分離する
ことで、この規則に備える。

replacement 後、古い snapshot output は retain 中なら読み取り可能なままでよく、また後続の
cache-adapter logic により validated cache input として使われ得る。current result として
publish してはならない。storage は `BuildSnapshotId` の順序や hash 比較から currentness を
推測しない。

## Error handling

| Condition | Handling |
|---|---|
| seal 前の read | `UnsealedOutput` として拒否し、handle を publish しない。 |
| 二重 seal | `AlreadySealed` として拒否し、最初の seal 済み value を保つ。 |
| pending slot がこの storage service に存在しない | `UnknownSlot` として拒否する。 |
| unknown sealed output id | `UnknownOutput` として拒否する。 |
| collect 済み slot の read | `CollectedOutput` として拒否し、handle を暗黙に再作成しない。 |
| runtime kind mismatch | `TypeMismatch` として拒否し、byte を再解釈しない。 |
| lineage が canonical output id と一致しなくなっている | `InvalidLineage` として拒否し、pending slot を abandoned にする。 |
| corrupt internal blob | `CorruptBlob` として拒否し、caller が producer を再実行するか cache-origin value を miss として扱う。 |
| stale generation | `StaleHandle` として拒否し、非互換 handle 間で storage slot を再利用しない。 |

すべての storage error は fail closed である。cache rehydration 由来の error は、後続の cache
adapter が handle を公開する前に cache miss として扱う。

## Tests

Task 5 と 6 は以下を cover しなければならない:

- pending output は不可視であり、seal 済み handle として読めない。
- 二重 seal は拒否され、最初の seal 済み value は変わらない。
- `PhaseOutputRef<T>` が期待された型を round-trip し、mismatch を拒否する。
- configured spill threshold を超える payload が content-addressed blob 経由で round-trip する。
- collection は unretained かつ eligible な output だけを drop する。
- retain された output は release まで session または snapshot replacement を生き延びる。
- collection は冪等である。
- handle の所持は proof authority または trusted status を意味しない。
