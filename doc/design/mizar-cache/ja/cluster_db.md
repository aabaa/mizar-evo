# Module: cluster_db

> 正本は英語です。英語版:
> [../en/cluster_db.md](../en/cluster_db.md)。

Status: task 12 で仕様化。task 13 は origin write、stale-origin removal、
aggregate index rebuild を実装する。import スコープ view materialization は task 14
から開始する。

## 目的

`cluster_db` は `cluster-db/` の cache 側 storage 契約を所有する。すなわち、
受理済み registration、cluster、reduction、subsumption-DAG contribution に
対する内部 index と、それらの index から導出する import スコープ view である。

この module は最適化 surface である。proof authority、checker authority、
artifact publication authority ではない。`cluster-db/`、origin record、
import スコープ view のいずれを削除しても、再 index または再 check が必要に
なるだけでなければならない。source semantics、proof acceptance、trusted
`used_axioms`、interface hash、artifact publication status を変えてはならない。

owner が受理済みかつ importer-visible として project した contribution だけが
visible index に入れる。parsed declaration、pending registration、recovered
registration、rejected proof、open obligation、externally attested proof material、
backend diagnostics、backend logs、timing metadata、cache hit/miss state は
importer-visible な cluster-db input では決してない。

## authority と input

この文書は以下を精緻化する。

- [spec 23.7.7](../../../spec/ja/23.package_management_and_build_system.md#2377-ストレージフォーマットcluster-dbresolution-tracediagnostic-explanation)
- [architecture 11](../../architecture/ja/11.artifact_and_incremental_build.md)
- [architecture 17](../../architecture/ja/17.cluster_trace_format.md)
- [architecture 22](../../architecture/ja/22.incremental_verification_contract.md)
- [internal 02](../../internal/ja/02.artifact_store_cache_key_and_manifest.md)

`cluster_db` は producer が所有する snapshot を消費する。

- checker/artifact owner からの accepted registration contribution summary
- accepted cluster / reduction contribution payload summary
- proof-backed accepted contribution に対する accepted witness または
  deterministic discharge identity hash
- verifier policy fingerprint と schema/toolchain compatibility field
- origin identity、export visibility、dependency-facing interface hash、
  trace replay hash

これらの値は validation と indexing の input にすぎない。`mizar-cache` は raw
source、raw checker diagnostic、proof witness bytes、ATP evidence、cache record から
accepted contribution を構築してはならない。

## 概念的 surface

task 13 は source-level write/origin-tracking data layer を公開する。

```rust
pub const CLUSTER_DB_SCHEMA_VERSION: &str;
pub const CLUSTER_DB_HASH_DOMAIN: &str;

pub struct ClusterContributionOrigin { ... }
pub struct ClusterContributionRecord { ... }
pub struct ClusterIndexEntry { ... }
pub struct ClusterAggregateRow { ... }
pub struct ClusterAggregateIndexes { ... }
pub struct ClusterIndexSnapshot { ... }
pub struct ClusterDbIndex { ... }
pub struct ClusterDbUpdateReport { ... }

pub enum ClusterDbWriteRejection {
    UnsupportedSchema { ... },
    MissingRequiredIdentity { ... },
    UnacceptedContribution { ... },
    NotImporterVisible { ... },
    IncompleteOrigin { ... },
    Uncacheable { ... },
    ConflictingDuplicateOrigin { ... },
    OriginKeyCollision { ... },
    UnknownCompatibility { ... },
}
```

write と rebuild は fail closed でなければならない。不明な schema version、不明な
schema/toolchain compatibility、不完全な origin footprint、欠落した origin record、
古い origin record、uncacheable marker、互換性のない verifier policy、未受理の
contribution は、visible row ではなく reject/rebuild を強制する。

task 14 は `ImportScopedViewKey`、`ImportScopedView`、`ClusterDbLookupOutcome` などの
import スコープ view 名を追加してよい。task 13 はそれらの view materialization API を
公開してはならない。

## store layout

default root は設定された `cache_dir`、通常は `.mizar-cache/` である。

```text
.mizar-cache/
  cluster-db/
    origins/
      <origin-key>.mcd
    graph.json
    subsumption-dag.json
    attr-index.json
    type-index.json
    reduction-index.json
    views/
      <import-view-key>.view
    tmp/
    quarantine/
```

top-level JSON file 名は spec 23.7.7 と揃える。

- `graph.json` は conditional cluster graph と struct inheritance graph を、
  import filtering 用の origin identifier 付き compact adjacency format で保持する。
- `subsumption-dag.json` は registration subsumption DAG skeleton を保持し、
  import closure に対して filter すべき precondition も含む。
- `attr-index.json` は生成される attribute から origin-backed contribution への
  mapping である。
- `type-index.json` は type または mode trigger key から origin-backed
  contribution への mapping である。
- `reduction-index.json` は解決済み reduction `LHS` head から accepted reduction
  rule、guard summary、simplification-order metadata、visibility origin への
  mapping である。

`origins/` は cache 側 invalidation の source of truth である。aggregate index は
origin record からの deterministic projection である。`views/` は import スコープの
materialization を保存し、origin や aggregate index と独立に削除してよい。

spec 23.7.7 の aggregate file は、top-level schema version を持つ canonical UTF-8
JSON surface のままである。logical identity と canonical validation rule が変わらない
限り、実装は private な `origins/` と `views/` record を digest prefix で shard してよく、
それらの record に private binary encoding を選んでもよい。directory order、file
modification time、writer process id、temporary file name、cache insertion order、
record arrival order は view や trace の input では決してない。

## origin metadata

各 visible contribution は正確に 1 つの origin identity を持つ。その identity は同じ
semantic contribution の rebuild 間で安定し、rename または deletion で変化する。

| Field | Meaning |
|---|---|
| `origin_key` | package id、module path、stable contribution id、contribution kind、schema family からの domain-separated hash。 |
| `package_id` | 所有 package。 |
| `module_path` | 所有 source module。 |
| `stable_contribution_id` | registration、cluster rule、reduction rule、DAG node に対する producer-owned identity。 |
| `label` | source label または stable generated label。diagnostics での表示だけに使う場合でも、label change は visible view を invalid にする。 |
| `contribution_kind` | `existential`、`conditional_cluster`、`functorial_cluster`、`reduction`、`struct_inheritance`、`subsumption_node`、`subsumption_edge`。 |
| `target_pattern_hash` | canonical target/type/term pattern または DAG-node key。 |
| `guard_hash` | canonical guard と side-condition summary。 |
| `declared_contribution_hash` | canonical generated attribute、existence fact、result fact、reduction rule、inheritance edge、DAG edge payload。 |
| `accepted_visibility` | owner が project した export/import visibility。private または local-only contribution は importer から見えない。 |
| `accepted_status` | owner-projected contribution status。`Accepted` だけが visible index に入れる。 |
| `accepted_status_projection_hash` | visibility に使った owner-projected accepted registration status の hash。proof authority ではない。 |
| `accepted_witness_or_discharge_hash` | proof-backed accepted contribution に対して owner が export する accepted witness または deterministic discharge identity。欠落していれば、その origin は incomplete/uncacheable である。validation metadata にすぎない。 |
| `proof_backed` | visibility が proof または deterministic discharge validation に依存するかどうか。true の場合、witness/discharge identity は必須。 |
| `verifier_policy_fingerprint` | contribution が visible になった policy。互換性のない policy は miss。 |
| `policy_compatibility` | verifier-policy compatibility field。欠落、不明、unsupported compatibility は fail closed。 |
| `schema_compatibility` | producer schema compatibility field。欠落、不明、unsupported compatibility は fail closed。 |
| `toolchain_compatibility` | producer toolchain compatibility field。欠落、不明、unsupported compatibility は fail closed。 |
| `producer_schema_versions` | contribution の解釈に必要な schema family。不明な required schema は miss。 |
| `trace_replay_hashes` | contribution の audit または replay に必要な ResolutionTrace replay hash。 |
| `dependency_interface_hashes` | contribution が消費する dependency-facing module/registration interface hash。 |
| `origin_footprint_hash` | stale-origin detection と view invalidation のための complete footprint hash。 |
| `footprint_completeness` | origin dependency footprint の completeness state。`IncompleteUncacheable` は rejection を強制する。 |
| `uncacheable` | insertion と reuse を禁止する明示 marker。 |

cache は origin record の横に diagnostic reference を保存してよいが、diagnostic は
importer visibility の一部ではなく、rejected proof や external proof を visible にしては
ならない。

同一 payload を持つ重複 `origin_key` record は coalesce してよい。異なる payload を
持つ重複 origin key は invalid であり、影響する aggregate index と import スコープ
view を miss にする。

## accepted-only visibility

importer-visible index は以下をすべて満たす contribution だけを含んでよい。

1. producer が contribution を accepted かつ importer-visible として mark した。
2. contribution が complete origin metadata を持つ。
3. verifier policy と schema/toolchain compatibility が既知かつ support 済みである。
4. contribution が `uncacheable` と mark されていない。
5. dependency interface hash と required trace replay hash が利用可能である。
6. proof-backed accepted contribution が、owner から export された accepted witness
   hash または deterministic discharge hash を持つ。
7. view を使う前に、同じ origin の stale record が削除済みである。

未受理の coherence、existence、compatibility、reducibility proof は `graph.json`、
`subsumption-dag.json`、`attr-index.json`、`type-index.json`、`reduction-index.json`、
`views/`、`ResolutionTrace`、`interface_hash`、downstream dependency summary を
seed してはならない。

externally attested proof evidence は accepted cluster-db visibility source ではない。
上流 policy が external evidence を diagnostics や status explanation のために記録しても、
`cluster_db` はその material を importer index では non-visible と扱う。cache record
自体は accepted contribution projection を提供したり昇格したりできない。

## aggregate index

aggregate index は accepted origin record の canonical projection である。

- graph vertex と edge は graph kind、target/type key、source attribute または
  type key、generated attribute または target key、origin key、contribution
  fingerprint で sort する。
- DAG node は `(symbol, guard_hash, origin_key)` で sort する。
- DAG edge は stronger node、weaker node、precondition hash、origin key、
  contribution fingerprint で sort する。
- attribute index row は attribute key、target graph key、origin key、
  contribution fingerprint で sort する。
- type index row は type または mode trigger key、origin key、contribution
  fingerprint で sort する。
- reduction index row は resolved `LHS` head、rule FQN、guard hash、
  simplification key、origin key、contribution fingerprint で sort する。

各 row は origin identifier を保持するため、import スコープ view は import closure ごとに
full graph を複製せず filter できる。index は architecture 17 の deterministic traversal
order、すなわち source type canonical id、cluster origin module path、declaration
source order、generated attribute canonical id、registration fingerprint を保存しなければ
ならない。cache write order と hash-map iteration order は selected trace に影響しては
ならない。

## import スコープ view

import スコープ view は aggregate index 上の deterministic filter である。key は次を含む。

- importing package id と module path
- canonical import-closure identity
- sort 済み visible origin set hash
- verifier policy fingerprint
- cluster-db schema version
- contributing origin が必要とする producer schema version
- toolchain compatibility identity
- graph closure または reduction strategy に影響する traversal/profile setting

view は import closure から visible な `origin_key` の row だけを含む。view は compact
adjacency list、filtered DAG edge、filtered attr/type/reduction row、stable diagnostic
reference を含んでよい。

異なる import closure は同じ aggregate index の上で異なる view を見てよい。cache は
origin を import closure ごとに物理複製してはならず、`ResolutionTrace` に存在しない
cluster step や reduction step を推論してはならない。

## invalidation

次のいずれかが変わると、その origin を見ることができる全 aggregate index row と
import スコープ view が invalid になる。

- registration label
- target pattern
- guard または side-condition summary
- declared cluster、reduction、inheritance、DAG contribution
- coherence、existence、compatibility、reducibility の accepted status
- proof-backed contribution の accepted witness または deterministic discharge identity
- verifier policy fingerprint
- origin identity
- export visibility
- required trace replay hash
- dependency-facing interface hash
- producer schema または toolchain compatibility

registration の deletion または rename は、dependent cache hit が accepted される前に
stale origin record を除去しなければならない。stale-origin removal を証明できない場合、
影響する footprint は `IncompleteUncacheable` であり lookup は miss する。

import-set change は、aggregate origin record が valid のままなら影響する import スコープ
view だけを invalid にする。現在 package 内の source change は、変更された origin、
その origin から project される aggregate row、その origin を visible origin set に含む
view を invalid にする。

## ResolutionTrace との関係

`cluster_db` は rule index と view filter を保存する。`ResolutionTrace` construction、
kernel replay、reduction selection は所有しない。

trace producer は明示的な cluster step と reduction step を記録しなければならない。
consumer は cluster-db index から hidden transitive expansion、implicit cluster insertion、
unrecorded reduction step を推論してはならない。`ResolutionTrace` replay hash は、derived
cluster / reduction fact を使う output の dependency fingerprint と cache key に参加する。

## failure semantics

compatibility、visibility、origin、integrity check の失敗はすべて cache miss または
rebuild request である。proof rejection でも proof acceptance でもない。

cache は次を説明する stable diagnostic を出してよい。

- unknown cluster-db schema
- unsupported producer schema
- unknown toolchain compatibility
- incompatible verifier policy
- incomplete origin footprint
- missing / stale / duplicate-conflicting origin record
- unaccepted または externally attested material が visible input として渡されたこと
- missing trace replay hash
- invalid canonical ordering または duplicate aggregate row

diagnostic wording、backend logs、timing metadata、filesystem freshness、cache hit/miss
timing は、view content、selected trace、proof selection、proof acceptance、artifact
order に影響しない。

## deferred と external dependency gap

| Gap | Classification | Handling |
|---|---|---|
| `CLUSTERDB-G001` | `external_dependency_gap` | task 13 は concrete checker/artifact accepted-contribution producer が存在すれば消費してよい。producer seam が不足している場合は、欠けている field を記録して defer し、raw source を parse したり accepted status を fabricate したりしない。 |
| `CLUSTERDB-G002` | `deferred` | task 13 は in-memory の origin-write と aggregate-index data layer を実装する。durable な `cluster-db/` file materialization は、persistent cluster-db storage task が scheduled されるまで deferred のままにする。 |
| `CLUSTERDB-G003` | `deferred` | task 14 が import-scoped view materialization と invalidation test を所有する。この task は view identity と invalidation rule だけを仕様化する。 |
| `CLUSTERDB-G004` | `external_dependency_gap` | build scheduler integration は task 15 の owner gate に残る。`cluster_db` は placeholder scheduler API を追加してはならない。 |
| `CLUSTERDB-G005` | `external_dependency_gap` | IR cache adapter integration は task 15 の owner gate に残る。`cluster_db` は placeholder `mizar-ir` API を追加してはならない。 |

## task 13 と task 14 の test

task 13 は少なくとも次を cover する。

- accepted contribution が origin record と aggregate index に insert されること
- accepted だが importer-visible でない private/local-only contribution、および
  rejected、pending、recovered、uncacheable、externally attested contribution が
  visible index から除外されること
- incomplete origin metadata、incomplete origin footprint、missing dependency
  interface hash、missing trace replay hash、proof-backed accepted origin の missing proof
  witness/discharge identity、不明または unsupported schema/toolchain compatibility が
  visible row ではなく miss を強制すること
- rename と deletion が reuse 前に stale origin を除去すること
- conflict する重複 origin と cross-module origin-key collision が mutation なしで
  miss を強制すること
- aggregate rebuild が影響 origin だけに触れること
- same-index bucket を含め、aggregate ordering が write order に依存しないこと

task 14 は少なくとも次を cover する。

- 無関係な変更をまたぐ import-scoped view reuse
- visible-origin change が正確に影響 view だけを invalid にすること
- private/local-only origin visibility、incomplete origin footprint、missing dependency
  interface hash、missing trace replay hash、および policy、schema、toolchain、
  traversal-profile change が miss になること
- view content が cache hit/miss timing と record arrival order に依存しないこと
- explicit trace を越えた hidden cluster/reduction step inference がないこと

## non-goal

task 12 と task 13 は scheduler integration、`mizar-ir` adapter integration、artifact
publication-token integration、proof status projection、proof winner selection、kernel
checking、`ResolutionTrace` construction を実装しない。task 13 は import スコープ view
も materialize しない。それは task 14 に残る。
