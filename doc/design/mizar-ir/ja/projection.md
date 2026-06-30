# mizar-ir Artifact Projection

> 正本は英語です。英語版:
> [../en/projection.md](../en/projection.md)。

## 目的

この文書は `mizar-ir` の artifact projection 境界を定義する。

`ArtifactProjectionService` は seal 済み `PhaseOutputRef<T>` と producer が
供給した安定射影入力を読み、`mizar-artifact` の `VerifiedArtifact` schema に
基づく未 publish の `VerifiedArtifactDraft` を構築する。draft は publish 候補に
すぎない。artifact store へ書き込まれず、manifest transaction へ commit されず、
proof authority でもない。

projection 境界の目的は、downstream consumer に compiler-internal IR storage ではなく
安定した外部 schema を見せることである。この境界は internal id を fully qualified
name、label、source range、正規化 signature、obligation metadata、
proof-witness reference、diagnostic、explanation reference、documentation
reference、LSP metadata へ写してよい。ただし raw IR value や storage handle を
公開してはならない。

real producer projection payload、`mizar-diagnostics` integration、artifact
publication token、phase-15 manifest transaction は、所有 crate が real seam を
公開するまで `external_dependency_gap` である。`mizar-ir` は placeholder の
producer token、diagnostics token、driver session、publication token API を追加して
はならず、`mizar-driver` に依存してはならない。

## 所有境界

`mizar-ir` が所有するもの:

- 射影される source item がすべて、意図した current snapshot の seal 済みで
  読み取り可能な output から来ていることの検証。
- crate-local な projection input record から `mizar-artifact` の安定した
  `VerifiedArtifact` schema shape への変換。
- export、expression metadata、obligation、witness reference、diagnostic、
  dependency provenance の決定的な順序付け。
- draft を返す前の raw internal IR 漏出の拒否。
- seal 済み IR output を変更しない、非権威な projection error。

`mizar-ir` が所有しないもの:

- `mizar-artifact` の schema version、canonical JSON serialization、manifest
  publication、artifact-store write、artifact hash framing。
- proof acceptance、trusted status、verifier-policy selection、決定的な proof
  winner selection、kernel acceptance、witness payload validation。
- `mizar-cache` `CacheKey`、dependency fingerprint、dependency slice、
  proof-reuse validation。
- real diagnostic rendering / registry ownership。
- source parsing、name resolution、type checking、VC generation、ATP translation、
  kernel-internal state。

## Draft model

`VerifiedArtifactDraft` は `mizar-ir` task 12 が所有する未 publish の in-memory
候補である。これは `mizar-artifact` の `VerifiedArtifact` 値と、draft がどのように
作られたかを説明するのに十分な projection-local provenance を含む。

draft は projection 時に使った target `BuildSnapshotId` に対してだけ current である。
obsolete snapshot の draft を current artifact 候補として返してはならない。retain された
stale output は diagnostic、explanation request、cache validation のために読んでよいが、
current projection に使うには明示的な current-snapshot validation path が必要である。
cache hit と rehydrated handle は proof trust や publication authority を昇格しない。

`mizar-ir` の外側が所有する phase 15 が、draft の serialize、manifest publication
precondition の検証、artifact の commit または rejection を担当する。projection failure は
seal 済み IR output を変更せず、以前の published manifest を authoritative なままにする。

## 入力

projection request には以下が必要である:

- target current `BuildSnapshotId`。
- build owner が供給する package/module identity、正規化済み source file path、
  source hash、language edition、toolchain id、lockfile hash reference、verifier
  configuration hash reference、dependency artifact hash reference。
- projection を正当化する producer output の seal 済み phase-output handle。
- export、expression metadata、obligation、proof-witness reference、diagnostic、
  任意の documentation / explanation reference 用の安定 producer projection record。
- publisher / snapshot replacement 境界から来る明示的な currentness 情報。

projection request は task-local mutable builder、raw AST arena node、raw type-table
row、raw `CoreIr`、raw `VcIr`、raw `AtpProblem`、kernel-internal proof state、
storage slot id、storage generation、memory address、worker id、temporary path、
manifest publication token を含んではならない。

real producer crate が typed projection record を公開するまで、task 12 はすでに
安定外部 string、source range、`mizar-artifact` hash reference になっている
crate-local projection input record を定義してよい。これらは placeholder downstream API
ではなく、test と future integration のための local validation boundary である。

crate-local projection input record は、この文書に列挙した `mizar-artifact` field に
強く型付けされていなければならない。任意の extension map、raw JSON blob、byte
payload、producer object の passthrough を含んではならない。この構造規則により、
"extension" の逃げ道から raw IR が持ち込まれることを防ぐ。test は、それでも
raw らしい sentinel string を使って、draft が返る前に漏出ガードが失敗することを
検証してよい。

## 射影されるデータ

### Exports

export は `VerifiedExport` record へ射影される。各 export は stable origin id、fully
qualified name、namespace path、visibility、producer-owned export kind、source range、
importer-visible な rendered signature、interface fingerprint、任意の projected proof
status、任意の documentation reference を含む。

signature は `mizar-ir` に届く前に producer 境界で正規化または render されていなければ
ならない。projection はそれらを sort / validate してよいが、type checking を再実行したり
overload semantics を決定したりしてはならない。

### Expression metadata

expression metadata は IDE、documentation、AI tooling のための安定した source-shaped
metadata である。rendered surface text、rendered inferred type、resolved symbol summary、
inserted coercion summary、active thesis summary、overload-resolution summary を含んで
よい。

これらの field は射影である。serialized `TypedAst` / `ResolvedTypedAst` node、arena
index、debug formatter dump、checker-local object identity、raw type-fact table を
含んではならない。

### Obligations and witnesses

obligation は `ObligationMetadata` record へ射影される。obligation id、任意の
cross-edit anchor、owner export origin id、source range、producer-owned obligation
kind、rendered statement summary、obligation fingerprint、VC fingerprint、local context
fingerprint、dependency-slice fingerprint、verifier-policy fingerprint、projected status、
任意の accepted witness obligation id、任意の deterministic no-witness discharge hash、
任意の diagnostic reference を含む。

`mizar-ir` は proof status を供給された projected data として記録する。proof が accepted、
trusted、kernel accepted、policy compatible であると決定してはならない。accepted
obligation は、供給された `ProofWitnessRef` と obligation metadata が
`mizar-artifact` schema の整合規則を満たす場合にだけ含めてよい。`mizar-ir` は schema
整合性を検証してよいが、witness payload bytes の検証や kernel evidence の replay は
行ってはならない。

obligation、VC、local-context、dependency-slice、verifier-policy の各 fingerprint は、
producer / cache / proof owner が供給する opaque hash reference である。`mizar-ir` は
それらが `mizar-artifact` schema shape を持ち、witness reference と内部整合することを
確認してよいが、それらの fingerprint を構築、再計算、再解釈、policy validation しては
ならない。

### Diagnostics and explanation references

diagnostic は stable id、code、severity、source range、message key、rendered message、
related location、任意の explanation reference を持つ `ArtifactDiagnostic` record へ
射影される。

この checkout に `mizar-diagnostics` は存在しない。そのため task 12 は crate-local な
安定 diagnostic projection record を使い、real diagnostic registry / renderer integration
を `external_dependency_gap` と分類しなければならない。projection は diagnostic
publication token を捏造してはならず、rendered diagnostic を proof authority として
扱ってもならない。

### Provenance

build provenance は toolchain、language edition、lockfile hash、verifier configuration
hash、dependency artifact hash、`mizar-artifact` schema が受け入れる任意の非権威
cache key string を記録する。

任意の cache key は stable artifact hash から除外される local metadata である。
`mizar-ir` はこの field のために `mizar-cache` key や dependency fingerprint を構築して
はならない。upstream owner が cache-key string を供給しない場合、projection は `None`
を使う。

## 漏出ガード

published artifact は以下の raw 情報を決して含んではならない:

- `SurfaceAst`。
- `TypedAst` または `ResolvedTypedAst`。
- `CoreIr` または `ControlFlowIr`。
- `VcIr`。
- `AtpProblem`。
- ATP backend process log または certificate の inline payload。
- proof witness payload bytes。
- kernel-internal proof state。
- storage handle、storage slot id、blob id、retain owner、local worker state。

task 12 は、stable string field を使ってこれらの raw internal category を運ぼうとする
projection input を拒否しなければならない。crate-local projection input record は任意の
extension record や raw byte / JSON passthrough field を提供してはならない。internal cache
record が raw IR bytes を含むことは許されるが、projection service は
`VerifiedArtifactDraft` を構築する前にそれらを drop または reject しなければならない。

## エラー

projection は、必要な入力がすべて seal 済み、読み取り可能、target snapshot に対して
current、schema compatible、canonical order へ sort 済み、raw IR 漏出がない場合にだけ
draft を返す。

failure condition:

| Condition | Handling |
|---|---|
| missing または unsealed output | projection 失敗。draft を返さない。 |
| obsolete snapshot の output を current として使用 | projection 失敗。draft を返さない。 |
| collected または unreadable handle | projection 失敗。draft を返さない。 |
| `mizar-artifact` との schema/version mismatch | projection 失敗。draft を返さない。 |
| unsorted または duplicate projected id | projection 失敗。draft を返さない。 |
| projected field 内の raw IR または storage internal | projection 失敗。draft を返さない。 |
| witness / obligation schema inconsistency | projection 失敗。draft を返さない。 |
| real diagnostics / producer / publication integration の欠落 | `external_dependency_gap` と分類し、stub は追加しない。 |

projection error は proof failure ではなく、storage を変更しない。scheduler は producer を
再実行してよく、または以前の artifact manifest をそのままにしてよい。

## Tests

task 12 は以下を cover しなければならない:

- `mizar-artifact` で write / read できる `VerifiedArtifact` を含む draft の構築。
- export、expression metadata、obligation、witness reference、diagnostic、
  explanation reference、provenance の順序付け。
- unsealed、collected、stale、obsolete、wrong-snapshot handle の拒否。
- raw `SurfaceAst`、`TypedAst`、`CoreIr`、`ControlFlowIr`、`VcIr`、`AtpProblem`、
  kernel-state、storage-handle、inline witness bytes の漏出拒否。
- accepted-obligation witness consistency を `mizar-artifact` schema に委譲し、
  `mizar-ir` が proof authority にならないこと。
- cache-key と dependency-fingerprint を所有しないこと。
- `mizar-driver` と placeholder diagnostics / publication-token API が存在しないこと。
