# mizar-artifact RegistrationSummary Schema

> 正本は英語です。英語版: [../en/registration_summary.md](../en/registration_summary.md)。

## 目的

`RegistrationSummary` は activated registration の dependency-facing artifact
projection である。下流 checker phase はこれにより、checker-internal index、
raw `TypedAst`、internal `cluster-db` record を読み込まずに、exported な
existential、conditional、functorial、reduction registration contribution を
読み込める。

この文書は architecture 04「Type and Registration Resolution」と architecture
17「Cluster and Registration Trace Format」を精緻化する。[store.md](./store.md)
の正準 store 規則に依存する。

## 範囲

`registration_summary` schema が所有するもの:

- registration summary の stable module identity と schema version field。
- importer から visible な activated/exported registration contribution。
- 公開 `ResolutionTrace` artifact への hash 参照。
- checker reuse と dependency fingerprint で使う dependency-facing
  `registration_interface_hash`。
- registration contribution と trace reference の canonical ordering rule。
- summary artifact の compatibility と reader validation 要件。

この schema が所有しないもの:

- type checking、cluster search、reduction search、checker fixed-point algorithm。
- proof acceptance、kernel replay、ATP dispatch、proof witness payload。
- architecture 17 が定義する concrete `ResolutionTrace` body schema。
- internal `RegistrationIndex`、`TypeFactTable`、`TypedAst`、`ResolutionTrace`、
  `cluster-db`、cache record、cache-key lookup。
- store canonical rule の利用を超える manifest transaction や artifact-store I/O。

## Top-Level Shape

schema family は `mizar-artifact/registration-summary` である。version `1.0` を task
7 の初期 version とする。

概念上の形:

```rust
struct RegistrationSummary {
    schema_version: String,
    module: ModuleSummaryIdentity,
    source_hash: Hash,
    registration_interface_hash: Hash,
    activated_registrations: Vec<ActivatedRegistrationSummary>,
    trace_artifacts: Vec<RegistrationTraceArtifactRef>,
    dependency_registrations: Vec<DependencyRegistrationRef>,
}
```

task 7 の実装は、この形を canonical JSON として serialize し、validating reader/writer を追加する。

`source_hash` は、summary を生成した source text を記録し、reader が stale artifact
を診断できるようにする。これは `registration_interface_hash` には含めない。
comment-only、proof-body-only、diagnostic-only、local source-range change は、activated
exported registration が変わらない限り importer を invalidation してはならない。

`module` field は [module_summary.md](./module_summary.md) が定義する identity shape を使う。
normalized source path と local filesystem alias は source metadata であり、
registration identity ではない。

## Activated Registrations

`activated_registrations` は、well-formedness と correctness obligation が configured
verifier policy により accepted された registration だけを含む。pending、rejected、
local-only、private、unverified な registration はこの summary から除外し、downstream の
automatic type fact に寄与してはならない。

各 activated registration は次を記録する。

- stable registration origin id。
- source label または stable generated label。
- registration kind: `existential`、`conditional`、`functorial`、`reduction`。
- exported visibility と namespace/module provenance。
- downstream checker index が使う canonical trigger key。
- 正規化済み pattern summary。必要に応じて referenced type head、attribute、functor、
  term head、parameter、guard fingerprint を含む。
- generated contribution summary。produced existence fact、attribute fact、
  functorial result fact、または reduction `source -> target` fingerprint など。
- registration を visible にした accepted proof status と verifier-policy fingerprint。
- replay または diagnostic に必要な cluster expansion、reduction strategy を説明する trace reference。
- optional な diagnostic/navigation source range metadata。

`RegistrationSummary` は projected accepted status を記録するだけで、proof acceptance を決定しない。
proof-producing phase と kernel acceptance はこの crate の外側に残る。

task 7 は次の canonical JSON field shape を使う。

```text
activated_registration = {
  "origin_id": string,
  "label": string | null,
  "registration_kind": "existential" | "conditional" | "functorial" | "reduction",
  "visibility": "public",
  "namespace_path": [string, ...],
  "source_module": module,
  "trigger_key": string,
  "normalized_pattern": registration_pattern,
  "generated_contribution": registration_contribution,
  "accepted_status": "accepted",
  "verifier_policy_fingerprint": interface_hash_string,
  "trace_ids": [string, ...],
  "source_range": source_range | null
}

registration_pattern = {
  "fingerprint": interface_hash_string,
  "type_head": string | null,
  "attribute": string | null,
  "functor": string | null,
  "term_head": string | null,
  "parameters": [string, ...],
  "guards": [interface_hash_string, ...]
}

registration_contribution = {
  "kind": "existence_fact" | "attribute_fact" | "functorial_result" | "reduction_rule",
  "summary": string,
  "fingerprint": interface_hash_string
}

source_range = {
  "start_byte": non_negative_integer,
  "end_byte": non_negative_integer
}
```

optional field は存在しない場合も JSON `null` として出力する。reader はすべての string field と
string-array entry の空文字列を拒否する。これには identity、origin id、label、namespace path、
trigger、pattern head、summary、parameter、artifact path、trace id、used-by origin id field が含まれる。
reader は unknown enum value、start が end より大きい range、duplicate `trace_ids`、`public` 以外の
`visibility`、`accepted` 以外の `accepted_status` も拒否する。pending、private、または unaccepted
registration は alternate status value ではなく、この summary に存在しないことで表現する。

## Hash String Domains

task 7 は published artifact hash に、task 3 の artifact-framed hash string 形式を使う。

```text
mizar-artifact/artifact-framed-hash-text/v1:<class>:<schema_family>:<schema_version>:<digest>
```

`<digest>` は 64 文字の lowercase hexadecimal digest である。top-level
`registration_interface_hash` と
`dependency_registration.registration_interface_hash` は class `interface`、schema family
`mizar-artifact/registration-summary`、summary の `schema_version` を使う。`source_hash` は
artifact-framed ではない。`mizar-session/hash-text/v1:<digest>` という source-text construction
を使う。

artifact-framed な `schema_family` string は、空でない、colon を含まない、slash 区切りの識別子である。
各 segment は空であってはならず、ASCII letter、ASCII digit、hyphen、underscore、dot だけを含む。
`schema_version` は task 3 の store schema version と同じ `major.minor` grammar を使う。

pattern fingerprint、contribution fingerprint、guard fingerprint、`verifier_policy_fingerprint`、
`trace_replay_hash` は semantic interface hash string である。これらは registration-summary domain
へ書き換えず、producer が所有する schema family と schema version を保持する。`artifact_hash` は
class `artifact` を使い、`diagnostic_hash` は class `diagnostic` を使う。task 7 は construction
label、class、version shape、digest spelling を検証し、caller が参照先 trace artifact を供給した場合は
hash reference の完全一致を検証する。将来の `ResolutionTrace` artifact schema family は task 7
では定義しない。これは trace producer/schema task まで `external_dependency_gap` とする。

## Trace Artifact References

`trace_artifacts` は、公開 `ResolutionTrace` artifact への hash-addressed reference を含む。
この reference により artifact validator と diagnostic は、trace body を registration summary に
埋め込まずに trace を見つけられる。

概念上の形:

```text
trace_artifact = {
  "trace_id": string,
  "trace_kind": "cluster" | "reduction",
  "artifact_path": string,
  "artifact_hash": artifact_hash_string,
  "trace_replay_hash": interface_hash_string,
  "diagnostic_hash": diagnostic_hash_string | null,
  "used_by_registration_origin_ids": [string, ...]
}
```

trace body は architecture 17 の trace artifact schema が所有する。この summary が所有するのは
stable reference だけである。trace data の欠落を「trace なし」と解釈してはならない。registration が
replay または diagnostic のため trace を必要とする場合、reference は存在し、reader により hash
validated されなければならない。
reader は bidirectional reference consistency も検証する。`trace_artifacts.trace_id` の集合は、
activated registration が named する trace id の集合と完全に一致しなければならない。また、各 trace
artifact の `used_by_registration_origin_ids` は、その trace id を named している activated
registration origin id の sorted set と完全に一致しなければならない。
activation は architecture 17 の `ResolutionTrace` kind ではない。registration activation は
`accepted_status` と `verifier_policy_fingerprint` により projection する。将来の activation proof
witness reference は proof-witness schema が所有し、`ResolutionTrace` は所有しない。

`trace_replay_hash` は registration compatibility に参加する semantic hash である。これは trace
schema 自身の domain の下で、replay-relevant な cluster または reduction trace projection を識別する。
`artifact_hash` は公開 trace file byte を検証し、`diagnostic_hash` は optional diagnostic payload を
検証する。これらの byte/diagnostic hash は replay hash が変わらない限り
`registration_interface_hash` には含めない。

## Dependency Registration References

`dependency_registrations` は、この module の activated registration projection に影響した依存
registration summary を記録する。dependency data の欠落を「依存なし」と解釈してはならない。不完全な
dependency registration 情報を持つ summary は、`mizar-cache` が所有する reuse decision において
uncacheable になる。

概念上の形:

```text
dependency_registration = {
  "module": module,
  "registration_interface_hash": interface_hash_string
}
```

## Registration Interface Hash

`registration_interface_hash` は importer-visible registration projection に対する canonical
dependency-facing key である。summary file の byte identity ではない。manifest path は公開 summary
file を識別し、store-level の `artifact_hash` は宣言済み hash exclusion 後の
publication-equivalent canonical content を検証する。

hash は task 3 の `HashClass::Interface`、schema family
`mizar-artifact/registration-summary`、current schema version で計算する。

hash に含めるもの:

- schema family と schema version。
- importer の解釈に影響する module identity field。
- language edition。
- activated/exported registration contribution すべて。
- registration kind、trigger key、normalized pattern、generated contribution、
  accepted proof status、verifier-policy fingerprint、export visibility。
- 必須 trace artifact reference の trace id、kind、semantic `trace_replay_hash`。
- dependency registration reference とその registration interface hash。

hash から除外するもの:

- file 全体の `source_hash`。
- diagnostic と navigation の source range。
- proof body、proof witness payload、trace artifact body。
- trace artifact path、trace artifact byte hash、diagnostic trace hash。
- local diagnostic と diagnostic wording。
- timestamp、local absolute path、worker id、backend timing、その他の hash-excluded local metadata。

source hash や source range の byte が異なっていても、activated registration projection が同一である
2 つの summary は同じ `registration_interface_hash` を持つ。一方で、それぞれの manifest entry または
store-level `artifact_hash` は異なってよい。

## Canonical Ordering

すべての collection は決定的に serialize する。

- activated registration は registration kind、trigger key、origin id、label、normalized pattern
  fingerprint、generated contribution fingerprint、accepted proof status で sort する。
  reader は duplicate origin id を拒否する。
- 各 activated registration の `trace_ids` は trace id で sort し、duplicate を拒否する。
- trace reference は trace kind、trace id、artifact path、artifact hash、`trace_replay_hash` で
  sort する。optional `diagnostic_hash` は interface-order の tie-breaker ではない。
  reader は duplicate trace id を拒否する。
- `used_by_registration_origin_ids` は origin id で sort し、duplicate を拒否する。
- dependency registration reference は full module identity と registration interface hash で sort
  する。reader は duplicate module identity を拒否する。

insertion order、source traversal order、filesystem order、cache insertion order、worker completion
order が serialized byte や hash に影響してはならない。

## Reader And Writer Requirements

task 7 writer は `store.md` の正準 UTF-8 JSON rule を使い、current schema version を emit する。task
7 reader は store boundary で生成された `CanonicalJson` value を対象に動作する。file の byte-level
artifact parse と duplicate-key detection は後続の artifact-store I/O task に残す。reader は:

- 上で列挙した schema object field をすべて必須とする。存在しない値を JSON `null` で表す field も含む。
- すべての schema object で unknown field を拒否する。
- summary field を解釈する前に schema-version compatibility を検査する。
- manifest entry、requested module、summary module identity が一致することを検証する。
- consuming command または manifest entry が要求する場合、`registration_interface_hash` を検証する。
- caller が referenced trace artifact を供給した場合、trace reference を hash で検証する。
- unaccepted、pending、private、raw-checker-shaped、raw-trace-shaped、cache-record-shaped payload を拒否する。
- この schema または後続互換 schema が定義する stable projected status field と verifier-policy
  fingerprint なしに accepted proof status を主張する summary を拒否する。

reader failure は artifact diagnostic である。proof authority を確立せず、registration search を
再実行せず、internal cache record へ黙って fallback してはならない。

## 公開 enum の前方互換性

task 19 は frontend task 25 の public-enum 手続きを registration summary API に適用する。
この module が所有するすべての public enum は forward-compatible API surface であり、
`#[non_exhaustive]` のままにしなければならない。downstream consumer は match 時に
wildcard fallback arm を持たなければならない。

これは API 互換性の判断であり、reader の寛容化ルールではない。artifact schema
reader は、将来の schema revision と version policy が受け入れ方法を明示しない限り、
unknown serialized enum value を引き続き拒否する。

| Enum | 前方互換性の判断 |
|---|---|
| `ArtifactHashClass` | producer-owned artifact hash reference が文書化済み schema policy の下で将来 class を命名できるよう non-exhaustive。 |
| `RegistrationKind` | exported registration category を拡張できるよう non-exhaustive。 |
| `RegistrationVisibility` | task 7 が public registration だけを publish する現在の規則を保ちつつ、visibility category を downstream match 破壊なしに拡張できるよう non-exhaustive。 |
| `RegistrationAcceptedStatus` | task 7 の acceptance rule を弱めず、accepted-status category を拡張できるよう non-exhaustive。 |
| `RegistrationContributionKind` | generated contribution category を拡張できるよう non-exhaustive。 |
| `RegistrationTraceKind` | referenced trace category を拡張できるよう non-exhaustive。 |
| `RegistrationSummaryError` | registration-summary validation diagnostic を拡張できるよう non-exhaustive。 |

この module は exhaustive な public enum 例外を所有しない。

## Deferred Implementation

task 7 は `RegistrationSummary` schema、canonical value writer、validating `CanonicalJson` reader、
および round-trip、trace reference の hash 解決、deterministic ordering、incompatible-version read、
registration/hash mismatch rejection の test を追加する。

checker producer integration、concrete `ResolutionTrace` artifact production、proof acceptance、
manifest/file I/O は external または later-task work に残り、task 7 で stub してはならない。file の
byte-level artifact parse と duplicate-key detection は artifact-store I/O に deferred のままとする。
