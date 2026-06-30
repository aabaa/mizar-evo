# mizar-ir Cache Adapter

> 正本は英語です。英語版:
> [../en/cache_adapter.md](../en/cache_adapter.md)。

## 目的

この文書は `mizar-ir` の cache-adapter 境界を定義する。

adapter は seal 済み `PhaseOutputRef<T>` を内部 cache record payload へ変換し、
すでに検証済みの `mizar-cache` hit からだけ新しい seal 済み handle を再水和する。
これは `mizar-ir` storage と `mizar-cache` record の間の最適化境界である。

adapter は `mizar-cache` `CacheKey`、dependency fingerprint、dependency slice、
proof-reuse validation decision、verifier policy compatibility decision、proof
acceptance、trusted status、kernel acceptance を構築または所有しない。それらの
結果を `mizar-cache` と proof/status owner から消費する。cache hit と再水和
された handle は、決して proof authority ではない。

real scheduler lookup session、`mizar-driver` cache scheduling、producer
dependency-footprint export、artifact publication token は、owning crate が real
seam を公開するまで `external_dependency_gap` である。`mizar-ir` は placeholder
driver API を追加してはならず、`mizar-driver` に依存してはならない。

## 入力

### Encoding input

seal 済み output を encode するには以下が必要である:

- この `IrStorageService` から得た seal 済み `PhaseOutputRef<T>`。
- supported `OutputKind` と payload schema version。
- その output kind と schema 用の phase serializer が生成した canonical payload bytes。
- seal 済み output にすでに添付された side table。
- dependency footprint が complete かつ cacheable であることを示す
  producer/cache boundary 由来の cacheability decision。
- cache owner が供給する `mizar-cache` key または record context。
  `mizar-ir` はそれを構築しない。

adapter は cache record を書く前に encoding を拒否してよい。拒否はそれ自体では
build failure ではない。phase output は seal 済みのまま dependent task が利用できる。
scheduler は cache entry を保存せずに実行してよい。

### Rehydration input

cache から再水和するには以下が必要である:

- target current `BuildSnapshotId`。
- 明示された current work unit、phase、output kind、schema version。
- `mizar-cache` 由来の validated cache-hit result。
- header、key、schema、dependency footprint、policy、toolchain、
  source/dependency hash、proof-reuse metadata が `mizar-cache` によってすでに
  accepted または classified された cache record bytes。
- output lineage が必要とする場合、scheduler/producer context が供給する
  current-snapshot parent handle または named input hash。
- target snapshot 用の `IrStorageService` と `SnapshotHandleRegistry`。

adapter は validation が "validated hit" 状態に到達する前に `PhaseOutputRef<T>` を
作成または返却してはならない。missing、incomplete、unknown、unsupported、
uncacheable、incompatible、corrupt、policy/proof invalid と分類された record は、
handle reconstruction 前に miss になる。

## Cacheability

output が cacheable であるためには、以下すべてが真でなければならない:

- output が seal 済みで、同じ storage service から読み取れる。
- output が package/current build work に属する、または validated current cache input
  を再 encode するものである。
- output kind と payload schema に adapter-owned codec がある。
- canonical payload bytes を決定的に導出できる。
- side-table record が妥当で serialize 可能である。
- producer/cache boundary が、その phase の dependency footprint は complete であると
  報告している。
- record が明示的に `uncacheable` と mark されていない。
- output が proof-shaped であり、record を proof-related work に再利用する場合、
  proof-related reuse metadata は `mizar-cache`/proof owner によって reusable として
  検証済みである。

open-buffer/editor-only output、retained stale-snapshot output、partial output、
unsealed slot、collected handle、unsupported schema version、unknown または
incomplete dependency footprint を持つ output は cacheable ではない。それらは
in-memory IR として有効なままでよいが、adapter は cache metadata を捏造せず、
cache skip または miss を返さなければならない。
拒否済み、unknown、または incomplete な proof-reuse metadata も proof-shaped reuse では
同じ規則に従う。handle reconstruction の前に skip または miss になる。

`mizar-ir` は adapter control flow 用に crate-local な cacheability enum を持ってよい。
ただしそれは cacheable、skip、miss、incompatible のような consumer-facing
classification でなければならない。`mizar-cache` の dependency fingerprint state や
proof-reuse validation rule を、独立した authority として複製してはならない。

## Record payload shape

adapter-owned payload は内部 cache data である。cache record は published artifact
ではないため、canonical internal IR bytes を含んでよい。

cache record payload が含んでよいものは多くても以下である:

- output kind と payload schema version。
- payload content hash と side-table hash。
- canonical payload bytes、または cache store が供給する `mizar-cache` blob reference。
- seal 済み output を再構築するために必要な side-table record。
- current-snapshot lineage を導出するために必要な parent content-hash summary と
  named input hash。
- producer phase と work-unit label などの非権威 provenance。

record payload は以下を含んではならない:

- old snapshot 由来の再利用可能な `PhaseOutputRef<T>`。
- compatibility input としての storage slot id、storage generation、memory address、
  retain owner、その他 process-local storage internal。
- `mizar-cache` `CacheKey` construction logic。
- dependency fingerprint construction logic。
- `mizar-ir` authority としての proof accepted/trusted status、verifier-policy selection、
  kernel acceptance、trusted `used_axioms` state。
- artifact publication token または manifest transaction state。

payload が raw internal IR bytes を含む場合、それは内部 cache bytes のままである。
artifact projection boundary から返したり、published `*.mizir.json` artifact として
書いたりしてはならない。

## Validation before rehydration

再水和は 2 段階の fail-closed process である:

1. `mizar-cache` が lookup result を検証する。これには cache key identity、
   dependency footprint completeness、schema/toolchain/policy compatibility、
   source/dependency hash、cache record integrity、該当する場合の proof-reuse
   metadata が含まれる。
2. `mizar-ir` が、record payload を target snapshot の seal 済み output にできるかを
   検証する。これには output kind と schema support、payload hash match、
   side-table hash match、current-snapshot parent handle validation、deterministic
   lineage derivation、storage sealing が含まれる。

adapter は validated でないすべての状態を miss として扱う:

| State | Handling |
|---|---|
| missing record | Miss。handle を返さない。 |
| incomplete dependency footprint | payload bytes を decode する前に miss。 |
| unknown schema/toolchain/policy compatibility | payload bytes を decode する前に miss。 |
| explicit `uncacheable` marker | payload bytes を decode する前に miss。 |
| incompatible key/header/dependency/proof validation | payload bytes を decode する前に miss。 |
| corrupt record bytes または blob hash mismatch | Miss。必要なら cache owner を通じて cache-integrity diagnostic を報告する。 |
| unsupported output kind または schema | Miss。可能なら producer を再実行する。 |
| payload または side-table hash mismatch | Miss。seal しない。 |
| parent handle が missing、stale、collected、または別 snapshot 由来 | Miss。seal しない。 |
| storage sealing error | Miss。新規登録した lineage を rollback し、handle を公開しない。 |

adapter は構造化された miss reason を記録または返却してよいが、miss reason は
diagnostic/optimization data である。proof acceptance、published artifact identity、
dependency-facing summary、canonical diagnostic、source-level semantics を変えては
ならない。

## Rehydrated handles

再水和された handle は target `BuildSnapshotId` 内で新しく seal された
`PhaseOutputRef<T>` である。old storage slot への復元 pointer ではなく、cache record から
コピーした保存済み handle でもない。

adapter は target snapshot、phase、work unit、output kind、content hash、
side-table hash、validated current-snapshot parent/named input を使って
current-snapshot lineage を導出する。同じ current snapshot と input がすでに登録・seal
済み output を生成している場合、rehydration は既存 handle を返してよい。snapshot が
異なる場合、content hash が等しくても handle identity や current publication right が
等しいことを意味しない。

rehydration は最適化としてだけ許される。失敗した場合、scheduler は producer を再実行する。
再水和された handle は producer-published handle より強い trust を持たない。特に以下を
昇格してはならない:

- externally attested proof evidence から kernel-verified status へ。
- backend success から proof acceptance へ。
- cache lookup success から trusted status へ。
- dependency-fingerprint equality から proof acceptance へ。
- obsolete snapshot output から validation なしの current result へ。

## Snapshot and freshness boundaries

cache record は古い snapshot で生成されている場合がある。adapter は old
`PhaseOutputId` や old storage handle を current result として再利用してはならない。
代わりに、validation が record payload が target `BuildSnapshotId` に適用できるかを決定し、
その validation が成功した後に限り `mizar-ir` が current-snapshot output を seal する。

obsolete snapshot output は LSP、diagnostics、explanation request、cache writing のために
retain されてよい。current output として publish できず、origin label だけでは再水和
できない。stale data から current handle への唯一の経路は、上記の validated cache-hit path
である。

open-buffer/editor-only output は dependency artifact ではない。open-buffer dry-run 用の
cache record は、owning scheduler と LSP crate が real seam を定義するまで out of scope である。
`mizar-ir` はその integration を `external_dependency_gap` として分類し、placeholder API を
作ってはならない。

## Errors and results

adapter は 2 つの成功 outcome を公開する:

- `Encoded`: seal 済み output が cache owner に保存させる内部 cache record payload へ
  変換された。
- `Rehydrated`: validated hit が current-snapshot `PhaseOutputRef<T>` として seal された。

その他の outcome はすべて非権威である:

- `Skip`: output は valid IR だが cache すべきではない。
- `Miss`: cache record を使えず、producer を実行すべきである。
- `Incompatible`: adapter はこの output kind/schema を support していない。
- `Corrupt`: cache data が integrity check に失敗し、cache owner によって破棄されるべきである。

どの error path も部分的に再構築された handle を返さない。どの error path も published
artifact や proof status を変更しない。

## Tests

Task 10 は以下を cover しなければならない:

- mock `mizar-cache` validation seam 経由の encoding と rehydration round-trip。
- mock seam が validated hit を報告する前に handle が作られないこと。
- missing、incomplete dependency footprint、unknown compatibility、`uncacheable`、
  incompatible、corrupt、unsupported schema、bad proof validation state がすべて
  handle reconstruction 前に miss になること。
- 改ざんされた payload bytes/content hash と改ざんされた side-table records/hash が、
  handle を seal せず registered lineage も残さず miss になること。
- rehydrated handle が seal 済み・typed・current-snapshot handle であり、その
  content hash、side-table hash、payload、side table が original output と一致すること。
- old snapshot handle が cache record からコピーされないこと。
- cache hit と再水和された handle が proof authority、trusted status、`CacheKey` ownership、
  dependency-fingerprint ownership、kernel acceptance を持たないこと。
