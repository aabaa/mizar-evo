# mizar-artifact Store And Canonical Form

> 正本は英語です。英語版: [../en/store.md](../en/store.md)。

## 目的

この文書は、source implementation が着地する前に、公開される
`mizar-artifact` スキーマが共有する store-level の規則を定義する。

これは [architecture 11](../../architecture/ja/11.artifact_and_incremental_build.md)
と [internal 02](../../internal/ja/02.artifact_store_cache_key_and_manifest.md)
の phase 15 artifact storage を精緻化する。公開 artifact は安定した UTF-8 JSON
射影であり、atomic に書かれ、manifest に index され、決定的な正準 byte から
hash される。

## 範囲

この store spec が所有するもの:

- package artifact root layout と path policy。
- 正準 UTF-8 JSON serialization 要件。
- top-level `schema_version` 互換性検査。
- semantic、implementation、diagnostic、local-metadata hash の分離。
- 公開 file に対する atomic write と validating read の要件。

この store spec が所有しないもの:

- manifest publication を安全にする store 要件を超えた manifest transaction
  semantics。
- internal cache record、cache-key lookup、cluster-db storage、proof reuse。
- raw compiler IR dump や scheduler state。
- proof authority、kernel acceptance、proof search、policy decision。

## Store Layout

各 package は 1 つの artifact root を持つ。通常は package の `artifact_dir`
（既定では `build/`）である。portable な公開 path は package-relative または
workspace-relative である。absolute path は、明示的に local diagnostic または
debug payload である場合だけ現れてよく、その field は semantic hash と
publication-equivalence hash から除外される。

標準の package manifest file は次のとおりである。

```text
artifact-manifest.json
```

manifest は唯一の publication index である。reader は artifact root 内の任意の
file を走査して公開 artifact を発見してはならない。

store は internal 02 由来の次の root-level area を認識する。

```text
build/
  artifact-manifest.json
  <module>.mizir.json
  proof-witnesses/
    <module>/
      <witness-file>
  logs/
    <module>.atp-log
  missing_facts.json
  explanations/
    <module>/
      <diagnostic-id>.json
```

schema-specific spec は、module summary、registration summary、witness reference、
verified artifact の file name をさらに精緻化してよい。ただし、すべての公開
artifact path は manifest から到達可能でなければならず、path normalization 後も
package artifact root の下に残らなければならない。portable published path では
`..` traversal、symlink escape、drive-root change を拒否する。

`.mizar-cache/blobs/` のような internal cache directory 配下の content-addressed
blob は `mizar-cache` に属し、`mizar-artifact` には属さない。`mizar-artifact`
が hash-named published file を書いてよいのは、schema-specific artifact または
witness spec がその公開 path 形状を要求する場合だけである。

artifact root 内の参照されない file は公開 artifact ではない。failed write や
interrupted session により残ってよいが、reader はそれらを無視する。

## 正準 UTF-8 JSON

後続の公開 schema が別形式を明示しない限り、公開 artifact は正準 UTF-8 JSON である。

正準 JSON byte は次の規則に従う。

- byte stream は byte order mark を持たない UTF-8 である。
- 各 artifact は 1 つの JSON value と末尾の 1 つの `\n` からなる。
- object member name は一意で、Unicode scalar value 順に並ぶ。
- schema field は canonical field order で出力される。この順序は、schema が
  より厳密な stable order を定義しない限り、object-member sort と同じである。
- map と set は schema-defined canonical key で sort された array として
  serialize される。
- source order、proof order、diagnostic order、その他の意味を持つ順序付き列を
  表す array は、その順序を保持する。
- optional field は、schema が明示的に `null` を要求しない限り、存在しない場合は
  省略される。
- string は必要な JSON escaping を除き、source text をそのまま保持する。
- path は serialization 前に normalize され、portable field では `/` separator を
  使う。
- schema が hash string を reader に公開する場合、その hash string は algorithm
  または domain prefix を含む。
- insertion order、map iteration order、filesystem directory order、wall clock
  order、worker completion order が正準 byte に影響してはならない。

実装は、disk から読んだ公開 artifact 内の重複 JSON object key を拒否しなければならない。
parse 可能だが非正準な artifact を reader が黙って書き換えてはならない。それは拒否
されるか、consuming command により非 authoritative と扱われる。

## Schema Versions

すべての公開 top-level JSON schema は必須の `schema_version` string を持つ。
schema version は正準 byte の一部であり、その schema change により意味が変わる
すべての hash domain の一部である。

reader は artifact を信頼する前に互換性検査を行う。

- `schema_version` の欠落は incompatible。
- unknown schema name または schema family は incompatible。
- より新しい major version は incompatible。
- より古い major version は、schema-specific migration が明示的に実装されない限り
  incompatible。
- より新しい minor version は、reader が無視してよい追加 field すべてについて
  schema が forward compatibility を宣言している場合に限り read できる。
- malformed version string は incompatible。

互換性エラーは schema family、実際の version、supported version range、artifact path を
報告する。それらは、unsupported artifact を proof authority に変えるような cache fallback
を起動しない。

## Hash Separation

store は 4 種類の hash class を区別する。

| Hash class | 対象 | 除外 | Consumer |
|---|---|---|---|
| `interface_hash` | dependency-facing exported signature、accepted exported proof status、importer の解釈に影響する schema version | implementation body、local diagnostic、local metadata | downstream semantic phase |
| `implementation_hash` | module の full stable published projection | hash-excluded local metadata | local rebuild、LSP、docs |
| `diagnostic_hash` | projected diagnostic と structured local explanation handle | semantic field と proof authority | diagnostics、LSP refresh |
| `artifact_hash` | 宣言済み hash exclusion を適用した後の公開 file の正準 byte | temporary name、session id、wall-clock local field | manifest validation と publication integrity |

各 hash class は domain-separated である。ある hash class に有効な byte string を、別の class
として直接再利用してはならない。具体的な hash algorithm と domain tag は、task 3 の共有
canonical hashing implementation で導入する implementation constant である。ただし、公開
される hash string は、reader が mismatch を拒否できるだけの algorithm/domain 情報を記録する。

diagnostic artifact と development artifact は独自の hash を持ってよいが、それらの hash は
semantic dependency compatibility を決定しない。internal cache key と cache record は
`mizar-cache` に属し、この crate には属さない。

## Hash-Excluded Local Fields

local build session により変動する field は、schema が hash-excluded と mark した場合だけ許可される。
初期の hash-excluded local field は次のとおりである。

- `verified_at`。
- temporary file name。
- build session id と task id。
- local diagnostic または debug payload にだけ現れる absolute path。
- schema が semantic input ではなく local provenance と分類する wall-clock timing と backend process metadata。

hash-excluded field は、存在する場合でも parse と validation の対象である。それらは、後続 schema が
field を stable provenance hash domain へ明示的に移さない限り、`interface_hash`、
`implementation_hash`、`artifact_hash`、publication equivalence から除外される。reader は
proof result の受理、dependency compatibility の検証、package publication eligibility の判定に
hash-excluded field を使ってはならない。

## Atomic Writes

公開 file は、target directory または同一 filesystem 上の store-owned temporary directory で
temp-and-rename protocol により書かれる。

1. 公開 path ではない temporary file へ正準 byte を serialize する。
2. file contents を flush する。
3. writer mode が要求する場合、temporary file を再読込または hash する。
4. temporary file を final artifact path へ atomic rename する。
5. platform が対応する場合、containing directory を flush する。
6. final path と hash を manifest transaction へ返す。

reader は以前の完全な file か、新しい完全な file のどちらかを見る。部分的に書かれた JSON file は
build error であり、公開 artifact として受理してはならない。manifest commit 前に write が失敗した
場合、以前の manifest が authoritative のまま残る。新しく書かれた参照されない file は reader に
無視され、後で clean up してよい。

manifest transaction protocol は task 12 に deferred されており、その task が
`manifest.md` を追加する。それまでは internal 02 を manifest transaction の参照元とする。
この store spec は、manifest entry が参照する file が manifest により公開される前に、すでに
write、flush、hash validation 済みであることだけを要求する。

## Validating Reads

公開 artifact reader は次を行う。

- artifact root の走査ではなく manifest を通じて path を解決する。
- normalization 後に package artifact root の外へ出る path を拒否する。
- UTF-8 JSON を parse し、parse failure には artifact path と、利用可能なら有用な byte、line、column
  location を報告する。
- schema-specific field を解釈する前に `schema_version` を検査する。
- 重複 object key を拒否する。
- manifest entry または consuming command が要求する hash を検証する。
- publication policy が要求する場合、欠落した proof witness file を拒否する。

read failure は artifact diagnostic である。internal cache record へ黙って fallback してはならず、
proof authority も確立しない。

## Deferred Implementation

task 2 はこの仕様だけを追加する。source implementation は deferred とする。

- task 3 は共有の canonical serialization、hash domain、version check を実装する。
- task 13 は store write、atomicity、corruption-detecting read を実装する。
- task 14 は manifest transaction を実装する。
- schema-specific reader/writer behavior は各 schema task で実装する。
