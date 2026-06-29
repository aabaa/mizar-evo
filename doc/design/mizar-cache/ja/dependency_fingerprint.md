# Module: dependency_fingerprint

> 正本は英語です。英語版:
> [../en/dependency_fingerprint.md](../en/dependency_fingerprint.md)。

状態: task 4 で仕様化する。ソース実装は task 5 で開始する。

## 目的

`dependency_fingerprint` は、`mizar-cache` の cache 側 dependency
fingerprint と dependency-footprint contract を所有する。

Dependency fingerprint は内部最適化の入力である。cached phase output が
再利用候補かどうかを決めるために使うが、proof acceptance、artifact
publication、trusted `used_axioms` は決めない。cache を削除したり、すべての
fingerprint を再計算したりしても、変わるのは性能だけでなければならない。

この module は
[architecture 18](../../architecture/ja/18.dependency_fingerprint.md) と
[architecture 22](../../architecture/ja/22.incremental_verification_contract.md)
の cache validation 部分を精緻化する。`mizar-artifact` と `mizar-vc` から
producer-owned hash と slice を消費するが、`mizar-build`、`mizar-ir`、proof
witness publication token、artifact commit scheduling の placeholder integration
は作らない。

## 責務

この module が所有するもの:

- cache 側 fingerprint target 名、domain、schema version、stable ordering;
- phase が complete、conservative、または uncacheable coverage を持つかを示す
  dependency footprint record;
- `mizar-cache` が最初に使う slice 粒度;
- fingerprint 変化に対する rebuild-trigger 分類;
- cache invalidation と architecture 18 の semver-check handoff で使う
  compatibility-diff input;
- dependency footprint を信頼できない場合に使う保守的な `uncacheable`
  marker。

対象外:

- proof status projection、deterministic proof winner selection、ATP backend
  policy、kernel acceptance、trusted proof evidence;
- cache record、blob storage、cluster-db index の読み書き;
- producer 側 interface hash、registration summary、dependency slice、VC
  fingerprint、proof-reuse metadata の計算;
- scheduler integration、IR adapter integration、artifact committed
  publication-token integration、end-to-end cache hit scheduling。

## Public Conceptual API

task 5 で正確な Rust 名は調整してよいが、semantic surface は以下とする:

```rust
pub const DEPENDENCY_FINGERPRINT_SCHEMA_VERSION: &str;
pub const DEPENDENCY_FINGERPRINT_HASH_DOMAIN: &str;

pub struct DependencyFingerprint {
    pub target: FingerprintTarget,
    pub identity: FingerprintIdentity,
    pub value_hash: Hash,
    pub schema_version: SchemaVersion,
}

pub struct DependencyFootprint {
    pub owner: FootprintOwner,
    pub phase: PipelinePhase,
    pub fingerprints: Vec<DependencyFingerprint>,
    pub slices: Vec<DependencySliceFingerprint>,
    pub completeness: FootprintCompleteness,
    pub uncacheable: bool,
}

pub enum FootprintCompleteness {
    Complete,
    ConservativeComplete,
    IncompleteUncacheable,
}

pub enum RebuildTrigger {
    ReuseAllowed,
    RebuildPhase,
    RebuildDependents,
    UncacheableMiss,
}
```

task 5 は fingerprint API に必要な場合に `RebuildTrigger` のような共有 data
shape を定義してよいが、trigger evaluation は実装しない。fingerprint delta
を phase invalidation へ写す処理と、source/import/registration/policy/toolchain
の trigger fixture は task 6 の範囲である。

`Complete` は、その phase に必要な dependency が現在の粒度ですべて分かって
いることを意味する。`ConservativeComplete` は理想より粗い可能性はあるが、
sound reuse に必要な dependency family をすべて含むことを意味する。偽陽性の
rebuild は許される。`IncompleteUncacheable` は、少なくとも 1 つの必須
dependency family が欠落、不安定、未対応、または opaque local id でしか
利用できないことを意味する。これは `uncacheable = true` を設定し、cache miss
を強制しなければならない。

## 初期 slice 粒度

architecture の理想形は theorem、definition、cluster、notation、mode、
attribute の slice を挙げている。`mizar-cache` の初期実装は、保守的な
2 レベル粒度を使う:

1. **Published dependency projection 粒度。** imported module と registration
   dependency は `mizar-artifact` の dependency-facing hash で key 化する:
   - module summary `interface_hash`;
   - registration summary `registration_interface_hash`;
   - manifest と lockfile identity hash;
   - implementation/artifact hash は、local refresh または implementation body
     に意味が依存する phase output に限って使う。
2. **Per-VC dependency-slice 粒度。** proof/VC 関連 cache entry は `mizar-vc`
   の per-VC dependency-slice fingerprint、canonical VC fingerprint、
   local-context fingerprint、`ObligationAnchor` fingerprint、利用可能な
   deterministic-discharge または witness validation hash を消費する。

producer が theorem/definition/cluster/notation/mode/attribute の細分化 slice を
まだ公開していない場合、task 5 はより粗い importer-visible fingerprint を
計算してよい。この粗さが正当なのは保守的な場合だけである。粗い dependency
の変化が必要以上の work を rebuild してもよいが、粗い dependency が変化して
いないことによって cached output が実際に使った dependency の変化を隠しては
ならない。

theorem/definition/cluster/notation/mode/attribute の family は semantic target
taxonomy として残し、target 名と diagnostics に保存しなければならない。
fine-grained producer hash が landing したら、cache-authority boundary を変えずに
より細かい target entry を schema に追加してよい。

## Fingerprint Target

cache 側 target taxonomy は以下とする:

| Target | 必須 input |
|---|---|
| `source` | source content hash、package/module identity、language edition、source に影響する schema version |
| `lexical_parse` | token/AST に影響する source hash、imported lexical summary fingerprint、parser/lexer schema、active notation/operator view hash |
| `module_interface` | module summary identity と `mizar-artifact` の `interface_hash` |
| `module_implementation` | local body refresh 用の implementation/artifact hash。importer-visible proof authority ではない |
| `registration_interface` | registration summary identity と `registration_interface_hash` |
| `cluster_trace` | accepted registration/cluster/reduction trace replay hash と visible origin identity |
| `definition` | definition identity、statement/signature fingerprint、transparency/opacity policy、unfolding boundary |
| `theorem_statement` | theorem origin id、exported statement fingerprint、accepted-status visibility boundary |
| `proof_body` | local refresh 専用の proof-body または witness-producing implementation hash |
| `vc_slice` | `mizar-vc` dependency-slice fingerprint、canonical VC/context fingerprint、status/evidence boundary |
| `proof_reuse_identity` | proof-reuse validation hash、witness/discharge hash identity、policy fingerprint、proof metadata schema |
| `policy_toolchain` | verifier policy、backend profile、toolchain/schema compatibility、language edition |
| `lockfile_manifest` | package graph、manifest hash、dependency artifact availability、dependency publication identity |

すべての target は identity key と value hash を持つ。identity は baseline と
candidate の同じ semantic dependency を対応付け、value hash が変化を判定する。
identity と value を混同してはならない。

## Stable Input と除外

fingerprint に含めるもの:

- schema family と schema version;
- 解釈に影響する cache-key schema version;
- package id、module path、normalized public origin id、language edition、
  必要な lockfile identity;
- producer-owned interface、implementation、registration-interface、trace、
  witness、discharge、VC、local-context、dependency-slice hash;
- output の意味または reuse eligibility に影響する verifier policy と
  toolchain compatibility field;
- dependency coverage が不完全な場合の conservative unknown marker;
- importer visibility が projected status に依存する場合だけ accepted proof
  または registration status。

fingerprint から除外するもの:

- wall-clock time、backend runtime duration、cache hit/miss timing、scheduler
  priority、worker id、process id、thread id、record arrival order、write
  order、temporary path、local absolute path;
- owning phase が diagnostic artifact を semantic output と明示した場合を除く
  backend log、backend diagnostic、diagnostic wording、explanation preview;
- cross-edit reusable fingerprint を計算する場合の `VcId`、dense
  generated-formula id、source-map id、arena id、row id、source range などの
  raw snapshot-local id;
- trusted status の根拠としての unaccepted、recovered、open、rejected、
  externally attested、backend-only proof material。

## Canonical Ordering And Hashing

すべての collection は hashing 前に canonicalize する:

- fingerprint target: `(target_kind, owner_package, owner_module, origin_id,
  target_name, schema_family)`;
- dependency slice: `(slice_kind, owner, name, domain)`;
- artifact availability entry: `(package_id, module_path, artifact_kind,
  artifact_path, domain)`;
- compatibility field: `(family, field_name)`;
- unknown marker: `(family, reason, owner)`;
- rebuild-trigger row: `(change_kind, target_kind, dependent_phase,
  slice_precision)`。

同一 identity key かつ同一 payload の重複は coalesce する。同一 identity key で
payload が異なる重複は structurally invalid であり、diagnostic footprint をまだ
作れる場合は `IncompleteUncacheable`、schema または identity data が壊れていて
canonical footprint を作れない場合は no-footprint/no-key rejection にしなければ
ならない。

hashing は length-prefixed、field-tagged、domain-separated とし、domain は以下を
使う:

```text
mizar-cache/dependency-fingerprint/v1
```

producer-owned hash domain は保持する。`mizar-cache` は cache 側 field tag でそれを
包んでよいが、artifact hash、proof witness hash、deterministic discharge hash、
kernel evidence hash を proof authority として再解釈してはならない。

## Dependency Footprint Completeness

再利用可能な footprint は、phase に必要な dependency family が選択した粒度で
すべて表現されていることを示さなければならない。

completeness 要件:

- source-backed phase は source content、language edition、phase schema、関連する
  direct input hash を必要とする;
- import-aware phase は manifest/lockfile identity と visible な module summary /
  registration summary hash をすべて必要とする;
- registration、cluster、reduction phase は、使用したすべての visible contribution
  について accepted visible origin と replay/trace hash を必要とする;
- VC/proof phase は canonical VC fingerprint、local-context fingerprint、
  dependency-slice fingerprint、`ObligationAnchor` fingerprint、policy fingerprint、
  proof-reuse metadata schema、reuse が recomputation を skip する場合の witness
  または deterministic discharge validation hash を必要とする;
- artifact-facing phase は dependency artifact availability hash と
  publication-equivalent artifact hash を必要とするが、artifact commit timing は
  必要としない。

以下の場合は `IncompleteUncacheable` を使い、`uncacheable = true` を設定する:

- 必須 producer hash が欠けている;
- dependency family が opaque local id または diagnostic string でしか分からない;
- schema または toolchain compatibility field が unknown;
- dependency-slice producer が unknown coverage を報告した;
- proof/VC reuse input が欠落、mismatch、external attestation のみ、または
  unsupported evidence kind に属する;
- downstream owner seam が landing しておらず、その seam が clean-build equivalence
  の確立に必要である。

欠落 data を空 dependency set と解釈してはならない。
したがって task 5 は、compatibility field の欠落、空の compatibility field
value、または `unknown`、`unsupported`、`incompatible`、`missing`、`opaque`
のような value を fail-closed unknown marker として扱う。VC/proof phase の
footprint に VC ごとの slice fingerprint または proof-reuse validation metadata
がない場合も `IncompleteUncacheable` になる。

## Rebuild Trigger

trigger result は「cache hit を受け入れる前に何を rerun する必要があるか」を
答える。proof acceptance でも semver classification でもない。

task 6 は `(change_kind, target_kind, dependent_phase, slice_precision)` からなる
明示的な trigger row を評価する。`slice_precision` は exact または
conservative-coarse である。coarse slice は必要以上の dependent を rebuild して
よいが、exact slice なら rebuild する semantic change に `ReuseAllowed` を返しては
ならない。caller が複数 row をまとめる場合の優先順位は
`UncacheableMiss > RebuildDependents > RebuildPhase > ReuseAllowed` とする。

既知かつ対応済みの policy/toolchain/schema/lockfile/manifest change は、task 6 では
affected phase の `RebuildPhase` に写す。`UncacheableMiss` は unknown、incomplete、
unsupported、uncacheable、または missing validation input に限る。evaluator は
純粋な分類だけを返し、work scheduling、cluster-db index の読み書き、cache record
integration は行わない。

| Change | Trigger |
|---|---|
| fingerprint から除外される comment-only または diagnostic-wording-only change | semantic output では `ReuseAllowed` |
| token/AST shape を変える source content change | lexical/parse と全 dependent phase を rebuild |
| module `interface_hash` change | その module summary に依存する importer を rebuild |
| interface change を伴わない implementation/artifact hash change | local implementation output を refresh。exported semantics だけを理由に importer を rebuild しない |
| registration interface または accepted visible origin change | その origin を見られる registration、cluster、reduction、resolve/type、VC、proof、cluster-db view を rebuild |
| theorem statement と accepted-status boundary が同じ proof body change | local proof witness / implementation output を refresh。importer は rebuild しない |
| theorem statement、definition signature、mode、attribute、notation、cluster、exported algorithm contract change | その target を見られる dependent footprint を rebuild |
| verifier policy、toolchain compatibility、schema version、lockfile、manifest identity change | affected phase を miss/rebuild |
| incomplete footprint、unknown schema、unknown toolchain、uncacheable marker、missing proof-reuse validation input | `UncacheableMiss` |

coarse slice は over-trigger してよい。under-trigger してはならない。

## API Compatibility Diff

API compatibility diff は architecture 18 に従う:

- identity は baseline と candidate build の element を対応付ける;
- fingerprint value comparison は対応付け後の semantic content change を検出する;
- identity 欠落は removal または rename;
- new identity は addition;
- 同一 identity で theorem statement、definition signature、algorithm contract、
  mode、attribute、notation、registration-visible value、export visibility、
  language edition が変わった場合は interface change。

diff result は semver-check と diagnostics の入力である。cache reuse predicate では
ない。cache reuse には dependency fingerprint、schema、policy、toolchain、artifact
availability、proof/VC validation の exact match がなお必要である。

call-site trace がない場合、overload-resolution shift detection はこの層では heuristic
のままでよい。heuristic は conservative rebuild を要求してよいが、cache reuse を
許可してはならない。

## Proof And Trust Boundary

`dependency_fingerprint` は proof-reuse validation identity、producer-owned summary
からの accepted status projection、witness hash、deterministic discharge hash、
kernel evidence handoff hash を運んでよい。これらは reuse input にすぎない。

この module は以下をしてはならない:

- `KernelCheckResult` を作る;
- proof を kernel-verified と mark する;
- trusted `used_axioms` を project する;
- proof winner を選ぶ;
- external attestation、backend success、backend log、diagnostic、cache record を
  accepted proof evidence に変換する。

trusted acceptance は、proof/status layer が消費する `mizar-kernel` result にだけ
由来する。

## Planned Tests

task 5 は以下を cover する:

- identical input に対する deterministic fingerprint output;
- すべての collection の canonical ordering と duplicate conflict rejection;
- comment-only、formatting-only、diagnostic-only、backend-runtime-only、
  scheduler-order-only、cache-order-only change に対する fingerprint の安定性;
- temporary path、local absolute path、source range、`VcId`、dense
  generated-formula id、source-map id、arena id、row id、その他の
  snapshot-local id が変わっても stable dependency payload が変わらない場合の
  reusable fingerprint の安定性;
- interface hash change が importer-visible fingerprint を invalidate すること;
- interface と accepted-status boundary が一致する場合、implementation/proof-body-only
  change が importer-visible semantic fingerprint を invalidate しないこと;
- 選択した粒度で、per-VC dependency-slice change が依存する VC/proof fingerprint
  だけを変えること;
- missing producer hash、unknown schema/toolchain、incomplete dependency slice、
  unknown coverage、uncacheable marker、incomplete footprint が miss outcome を
  強制すること;
- mismatched proof/VC reuse input、external attestation のみの evidence、
  unsupported proof evidence kind、missing proof-reuse validation input が miss
  outcome を強制すること;
- proof-reuse validation input が untrusted validation data として参加し、proof
  authority にならないこと。

task 6 は source、import、registration、cluster/reduction、policy、toolchain、
schema、proof-body、diagnostic-only、incomplete footprint、unknown schema/toolchain、
uncacheable marker、missing proof-reuse validation change の rebuild-trigger fixture を
追加し、conservative coarse-slice mode で偽陰性がないことを cover する。

## Deferred And External Dependency Gaps

| Gap | Classification | Handling |
|---|---|---|
| `DEPFPR-G001` | `external_dependency_gap` | `mizar-build` scheduler cache seam は未準備。placeholder scheduler integration は追加しない。 |
| `DEPFPR-G002` | `external_dependency_gap` | `mizar-ir` cache adapter は存在しない。adapter stub は作らない。 |
| `DEPFPR-G003` | `external_dependency_gap` | artifact committed publication token integration は未利用。availability/hash input だけを記録する。 |
| `DEPFPR-G004` | `deferred` | より細かい theorem/definition/cluster/notation/mode/attribute producer slice は後で追加してよい。task 5 は artifact summary と `mizar-vc` per-VC slice から保守的に開始する。 |
| `DEPFPR-G005` | `external_dependency_gap` | proof-reuse metadata の downstream proof/cache/artifact consumer は owner gate 待ち。この module は validation identity だけを記録する。 |

## Non-Goals

この module は cache record の読み書き、proof evidence の選択、ATP 呼び出し、
kernel 呼び出し、artifact publication、build task scheduling、IR cache adapter 公開を
行わない。
