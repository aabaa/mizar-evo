# Store And Canonical Form

> 正本は英語です。英語版: [../en/store.md](../en/store.md)。

## 目的

この文書は `mizar-artifact` の公開スキーマが共有する store と canonical-form
規則を定義する。

## 責務

`store` module が所有するもの:

- artifact schema 向けの正準 UTF-8 JSON シリアライゼーション。
- schema-version 互換性チェック。
- semantic、implementation、diagnostic、content hash helper。
- artifact root 配下の content-addressed blob storage。
- temp file と rename による named artifact の atomic write。

所有しないもの:

- cache record、cache reuse、proof acceptance、producer projection construction。

## 正準 JSON

公開 structured artifact は UTF-8 JSON である。正準シリアライゼーションは
次を使う:

- object keys は Unicode scalar value の辞書順。
- arrays は owning schema が要求する正準順序を保持する。
- 不要な whitespace は出力しない。
- `true`、`false`、`null` は lowercase。
- strings は JSON が必要とする場合だけ escape する。
- numbers は schema が明示的に使う非負整数だけ。

Schema modules は canonical arrays を作る前に set/map を sort しなければ
ならない。hash-map iteration order、filesystem scan order、worker scheduling
order、temporary file names は canonical input にしてはならない。

## Schema versions

すべての公開 schema は `schema_version` string を持つ。reader は field が
欠けている場合、string でない場合、その schema の supported version と一致
しない場合に reject しなければならない。

最初の implementation wave では、各 schema はただ 1 つの version
`mizar-artifact/<schema>/v1` を support する。

## Hash classes

Hash は domain-separated canonical bytes を使う。domain name は hash input
の一部であり、異なる役割の同一 JSON bytes が同じ artifact fact と扱われない
ようにする。

Semantic hash は importer-visible meaning を覆う。次の local metadata を除外する:

- `verified_at`。
- transaction/session/temp file identifiers。
- local run ids。
- schema が diagnostic metadata と明示しない wall-clock timings。

Diagnostic hash は stable diagnostic/development projections を覆い、semantic
hash を変えずに変化してよい。

Implementation hash は明示的 local fields を除く 1 module の full stable
verified projection を覆う。proof-body-only change や diagnostic change により
downstream semantic imports を変えずに変化してよい。

## Store layout

Store root は package artifact directory である。Reader は manifest と記録済み
path を使う。任意 file scan は publication mechanism ではない。

```text
artifact_dir/
  artifact-manifest.json
  blobs/
    <content-hash>.json
  tmp/
    <writer-local-name>
  <module>.mizir.json
  logs/
  explanations/
  missing_facts.json
```

Blob file name は recorded content hash から導出する。Temporary file name は
local metadata であり、canonical artifact content や semantic hash には入らない。

## Development artifacts

Store は `missing_facts.json`、ATP logs、diagnostic explanation previews、
optional debug dumps などの development artifacts も write してよい。これらは
later manifest schema が明示的に参照しない限り、published package interface
ではない。

Development artifacts は同じ temp-and-rename write discipline と content hash
helpers を使うが、`interface_hash` には含めない。Stable diagnostic/explanation
projection が `VerifiedArtifact` から参照される場合、reference は portable path
と content hash を記録する。それ以外の場合、reader は local development output
として扱い、package compatibility では無視しなければならない。

この folded surface は現在の task sequence において separate
`development_artifact.md` を置き換える。Full producer-driven explanation emission
は、producer diagnostics/proof services が concrete projection inputs を公開する
まで範囲外である。

## Atomic writes

Named artifact write は次を満たす:

1. publication 前に bytes を完全に serialize する。
2. target directory 内の temporary file へ write する。
3. file contents を flush する。
4. temporary file を final path へ rename する。
5. platform が support する場合、containing directory を flush する。

Reader は temporary file を published と扱ってはならない。final file が存在しても
parse できない、または content hash が一致しない場合、partial bytes を黙って
受理せず structured artifact error にする。

## Error policy

Store errors は diagnostics に使える程度に structured/stable である:

- unsupported or missing schema version。
- byte position 付き malformed JSON。
- unexpected or missing schema field。
- invalid hash spelling。
- content hash mismatch。
- invalid portable path。
- path と error kind 付き I/O error。

Error variants は forward-compatible であり、public enums は non-exhaustive である。
