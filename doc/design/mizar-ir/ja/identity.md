# mizar-ir Identity

> 正本は英語です。英語版:
> [../en/identity.md](../en/identity.md)。

## 目的

この文書は `mizar-ir` の identity 境界を定義する。

`mizar-ir` は `mizar-session` から `BuildSnapshotId` と session-owned な
source identity を消費する。build snapshot、source id、cache key、
dependency fingerprint、proof-reuse identity、proof-trust state は作らない。
単一の `BuildSnapshotId` の中で、決定的な IR-local identity を割り当て、
seal 済み phase output 間の親/派生関係を記録する。

## 消費する identity と所有する identity

| Identity | Owner | Stability |
|---|---|---|
| `BuildSnapshotId` | `mizar-session` | 正確な source、dependency、lockfile、toolchain、verifier-config state に対して決定的。 |
| `SourceId` | `mizar-session` | non-canonical metadata として受け取る session-owned source handle。Task 3 は保持しない。 |
| `ModuleId` | `mizar-ir` | package/module identity と source identity から、1 つの `BuildSnapshotId` 内で決定的。 |
| `ItemId` | `mizar-ir` | owning module と producer-declared item key から、1 つの `BuildSnapshotId` 内で決定的。 |
| `ExprId` | `mizar-ir` | owning item または module と producer-declared expression key から、1 つの `BuildSnapshotId` 内で決定的。 |
| `VcId` | `mizar-ir` | owning obligation order key から、1 つの `BuildSnapshotId` 内で決定的。cross-edit proof-reuse identity ではない。 |
| `PhaseOutputId` | `mizar-ir` | phase、work unit、output kind、content hash、side-table hash、dependency output ids、sort 済み named input hashes から、1 つの `BuildSnapshotId` 内で決定的。 |

`BuildSnapshotId` と `SourceId` は消費する session identity である。`mizar-ir` は
それらを割り当てず、永続化しない。

すべての IR-owned id は、artifact projection が安定 published schema へ写す場合を
除いて snapshot-scoped である。arena index、memory address、slot number、task id、
worker id、filesystem temporary name、cache lookup timing、runtime duration は
安定 identity input にならない。

## Snapshot identity input

`BuildSnapshotId` はすでに正確な build input state を覆っている:

- normalized source versions と source hashes。
- dependency artifacts とその content hashes。
- lockfile hash。
- toolchain identity。
- verifier configuration hash。

`mizar-ir` はこの identity を弱めてはならない。ある `BuildSnapshotId` で作られた
handle は、別の snapshot の current result として使えない。許される
cross-snapshot reuse path は、validated cache rehydration または後続の owning
integration task が定義する explicit unchanged-input path だけである。どちらの
path も fail-closed で、validation 後にだけ current snapshot の新しい handle を
作る。

## Canonical ID assignment

各 `mizar-ir` id は domain-separated な canonical byte sequence から導出する:

```text
mizar-ir/<identity-family>/v1
snapshot = <BuildSnapshotId published-schema string>
canonical fields = fixed per-family producer-owned identity fields
```

現在の実装は、下表に示す family ごとの固定 field order を使う。family 内の
collection-valued field は、hash 前に stable key で sort する。

各 family の canonical fields は次の通り:

| Family | Required fields |
|---|---|
| Module | package id、module path、source hash |
| Item | module id、item kind、normalized origin key、declaration order key |
| Expression | module id、任意の item id、expression kind、producer path key |
| VC | module id、任意の item id、obligation order key、利用可能なら canonical VC fingerprint |
| Phase output | phase、work unit、runtime output kind tag、content hash、side-table hash、dependency output ids、sort 済み named input hashes |

registry は各 family について logical duplicate key も保持する。producer payload が
final hash を正当に変え得る場合、duplicate key は final id より狭い:

- module duplicate key: package id と module path。source hash は payload。
- VC duplicate key: module id、任意の item id、obligation order key。
- phase-output duplicate key: phase、work unit、output kind。

1 つの snapshot 内で同じ logical key を異なる payload で登録すると、2 つ目の
current identity を黙って作るのではなく conflicting duplicate として拒否する。

Producer path key は、owning phase が供給する semantic または source-shaped key
である。`mizar-ir` は ordering、domain、snapshot compatibility を検証するが、
name resolution、type semantics、obligation anchor、proof reuse、proof
acceptance は決定しない。

`SourceId` は、後続 task が non-canonical source metadata plumbing を追加できるよう
module identity input の横で受け取ってよい。ただし Task 3 は registry に保持せず、
意図的に `ModuleId` の hash へ入れない。`mizar-session` は `SourceId` を
non-persistable な session handle として扱う。一方で `BuildSnapshotId`、package id、
module path、source hash が決定的な identity input を提供する。

Identity input に使う collection は、stable string または hash key で sort して
から hash する。payload が衝突する duplicate identity key は拒否する。必須
identity field の欠落は、空 default で置き換えず拒否する。

## Parent and derived output relationships

snapshot handle registry は各 seal 済み output について lineage を記録する:

- `producer`: output を生成した phase/work-unit identity。
- `parents`: producer が消費した input `PhaseOutputId`。
- `named_input_hashes`: producer が宣言した non-output inputs。
- `content_hash`: seal 済み output の semantic hash。
- `side_table_hash`: source map、diagnostic、explanation ref、documentation
  attachment などの side table hash。

derived output は、cache adapter が親を current snapshot に validation and
rehydration した後でない限り、すべての parent と同じ `BuildSnapshotId` を持たなければ
ならない。parent link は registry で round trip でき、storage、publisher、
cache adapter、snapshot replacement logic が使う。lineage は proof evidence ではなく、
trusted status へ昇格してはならない。

## Incompatible snapshot reuse

registry は以下を拒否する:

- 別 snapshot の parent handle を持つ output の登録。
- 同じ snapshot に登録されていない owning module または item から item、
  expression、VC id を割り当てること。
- 記録済み owning module が supplied module と異なる item から expression または
  VC id を割り当てること。
- 後続の publisher と snapshot-replacement task が current/obsolete state を追加した後、
  obsolete snapshot output を current result として publish すること。
- registry が知らない snapshot に対する IR-local id の割り当て。
- `ModuleId`、`ItemId`、`ExprId`、`VcId`、source range、arena id、output hash の
  一致だけを cross-snapshot validation として扱うこと。
- `mizar-cache` が schema、dependency footprint、policy compatibility、
  dependency artifacts、必要な場合は proof-reuse metadata を検証する前に cache
  record を rehydrate すること。

cache hit は optimization data である。validated cache hit は current snapshot の
通常の seal 済み handle を復元してよいが、proof status を昇格せず、proof authority
境界を変えない。

## Snapshot replacement

Task 13 はこの registry に snapshot replacement を拡張する。新しい snapshot が
古い snapshot を supersede すると、registry は古い snapshot を current publication
不能として mark する。既存の retained handle は stale diagnostic、explanation、
LSP request、または validated cache input のために読み取り可能なままでよい。
supersession 後に current result として報告してはならない。

snapshot replacement は明示的である。current snapshot は registry property であり、
id value の比較ではない。Task 3 は known snapshots と lineage だけを記録する。
current/obsolete publication check は後続の publisher と snapshot-replacement task が
実装する。`BuildSnapshotId` は hash-like な opaque id なので、semantic ordering を
推測してはならない。

## Tests

task 3 は以下を cover しなければならない:

- 同一 snapshot/id input が同一 IR-local id を生成する。
- 衝突する duplicate identity key が拒否される。
- 非互換 snapshot の handle が current parent として reuse できない。
- parent/derived output lineage が round-trip する。
- `VcId` と他の IR ids が proof-reuse authority として振る舞わない。
