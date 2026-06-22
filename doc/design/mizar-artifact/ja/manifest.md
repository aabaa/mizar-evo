# mizar-artifact Artifact Manifest Schema

> 正本は英語です。英語版: [../en/manifest.md](../en/manifest.md)。

## Purpose

package artifact manifest は `mizar-artifact` の publication index である。downstream build、
LSP、documentation、extraction、audit tool は、module artifact や proof witness file を読む前に
manifest を読む。artifact root 内の任意の file を scan して published artifact を発見してはならない。

この文書は task 12 として [store.md](./store.md)、
[architecture 11](../../architecture/ja/11.artifact_and_incremental_build.md)、
[internal 02](../../internal/ja/02.artifact_store_cache_key_and_manifest.md)
を詳細化する。manifest file schema と、後続 task が実装する transaction protocol を仕様化する。

## Scope

manifest が所有するもの:

- package-level publication metadata。
- module artifact entry とその store-level artifact hash。
- proof witness file の reachability と producer-owned artifact hash。
- それらの file が artifact root を通じて意図的に公開される場合の optional development artifact reachability。
- manifest transaction の visibility、validation、recovery rule。

manifest が所有しないもの:

- `VerifiedArtifact`、`ModuleSummary`、`RegistrationSummary`、`ProofWitnessRef` object schema。
- raw compiler IR dump、scheduler state、internal cache record、`mizar-cache` content-addressed blob。
- proof authority、witness replay、kernel acceptance、verifier policy decision。
- store file I/O implementation。これは task 13 に残る。
- Rust の manifest reader/writer と transaction manager。これは task 14 に残る。

manifest は integrity と reachability metadata である。manifest entry は、参照先 artifact が
store-level artifact hash により名指された publication-equivalent content を持つことを示せるが、
externally attested evidence を kernel-accepted proof evidence に変えることはできない。

manifest transaction は obsolete writer を拒否するために、caller が供給する build snapshot freshness state を
運んでもよい。この state は transaction control data であり、stable artifact content ではない。
`mizar-artifact` は scheduler ordering を定義しない。

## File Name And Version

標準の package artifact manifest path は次の通りである。

```text
artifact-manifest.json
```

manifest path は package artifact root からの相対 path である。temporary manifest path は
published path ではなく、consumer が読んではならない。

schema family は `mizar-artifact/manifest` である。version `1.0` が初期 version である。task 14
reader は `1.0..=1.0` を support し、後続仕様が forward-compatible minor-version handling を明示的に
宣言しない限り、missing、malformed、異なる major、新しすぎる minor version を拒否する。

manifest file 自身の artifact hash は、store または manifest manager が canonical manifest JSON に対して
task 3 の artifact-framed hash construction を使い、class `artifact`、schema family
`mizar-artifact/manifest`、manifest `schema_version` で計算する。これは store の hash-exclusion model に従い、
raw filesystem-byte hash ではない。schema version `1.0` は hash-excluded manifest field を宣言しない。
manifest は自分自身の hash を JSON file 内には保存しない。

## Top-Level Shape

task 12 は次の canonical JSON field shape を仕様化する。

```text
artifact_manifest = {
  "schema_version": "1.0",
  "package": package_identity,
  "artifact_root": string,
  "lockfile_hash": artifact_hash_string,
  "toolchain": string,
  "language_edition": string,
  "verifier_config_hash": interface_hash_string,
  "modules": [module_artifact_entry, ...],
  "development_artifacts": [development_artifact_entry, ...],
  "provenance": manifest_provenance
}

package_identity = {
  "package_id": string,
  "package_version": string | null,
  "lockfile_identity": string | null
}

manifest_provenance = {
  "generated_by": string,
  "manifest_policy": string,
  "transaction_format": string
}
```

列挙した field はすべて必須である。存在しない値に意味がある field は JSON `null` で符号化する。
reader は必須 field の省略と、すべての schema object の unknown field を拒否する。

`artifact_root` は package root から見た artifact root の正規化済み package-relative path である。
absolute path であってはならず、空 segment、`.`、`..` を含んではならない。separator は `/` を使う。

`toolchain`、`language_edition`、`generated_by`、`manifest_policy`、`transaction_format` は
安定した空でない string である。これらは manifest metadata であり、proof authority ではない。

## Module Entries

`modules` は、この manifest から到達可能な published module artifact ごとに 1 entry を含む。

```text
module_artifact_entry = {
  "module": module,
  "source_file": string,
  "source_hash": source_hash_string,
  "artifact_file": string,
  "artifact_hash": artifact_hash_string,
  "interface_hash": interface_hash_string,
  "implementation_hash": implementation_hash_string,
  "module_summary_file": string | null,
  "module_summary_hash": artifact_hash_string | null,
  "module_summary_interface_hash": interface_hash_string | null,
  "registration_summary_file": string | null,
  "registration_summary_hash": artifact_hash_string | null,
  "registration_interface_hash": interface_hash_string | null,
  "proof_witnesses": [manifest_proof_witness_entry, ...],
  "diagnostics_hash": diagnostic_hash_string | null
}
```

`module` は `ModuleSummary` と `VerifiedArtifact` と同じ identity shape を使う。`source_file` は
`VerifiedArtifact` と同じ portable path rule に従う。`artifact_file`、summary file path、proof witness
path は package artifact root-relative path である。normalization 後も artifact root の下に残らなければ
ならず、absolute path であってはならない。

`artifact_file` は module の canonical `VerifiedArtifact` file を指す。`artifact_hash` は published
file の store-level artifact hash である。すなわち参照先 schema が宣言する hash exclusion 後の canonical
JSON を class `artifact` で frame した hash であり、raw filesystem-byte hash ではない。`interface_hash` と
`implementation_hash` は参照先 `VerifiedArtifact` 内の top-level hash と一致しなければならない。

summary file field は optional である。task staging により、すべての sidecar summary が emit される前に
verified artifact が公開されることがあるためである。3 つの `module_summary_*` field はすべて
`null` であるか、すべて non-null である。`registration_summary_*` field も同じ規則に従う。

`diagnostics_hash` は local diagnostic index または explanation bundle のための optional
diagnostic-payload hash である。proof authority ではなく、`VerifiedArtifact` に埋め込まれた diagnostic の
代替でもない。

## Proof Witness Entries

manifest proof witness entry は、module artifact entry から外部保存された witness file へ到達可能にする。

```text
manifest_proof_witness_entry = {
  "obligation_id": string,
  "obligation_fingerprint": interface_hash_string,
  "witness_path": string,
  "witness_artifact_hash": artifact_hash_string
}
```

各 entry は、参照先 `VerifiedArtifact` 内のちょうど 1 つの `ProofWitnessRef` と一致しなければならない。

- `obligation_id` は `ProofWitnessRef.obligation_id` と一致する。
- `obligation_fingerprint` は `ProofWitnessRef.obligation_fingerprint` と一致する。
- `witness_path` は `ProofWitnessRef.witness_path` と一致する。
- `witness_artifact_hash` は `ProofWitnessRef.witness_artifact_hash` と一致する。

module manifest entry の `proof_witnesses` array は、参照先 `VerifiedArtifact.proof_witnesses`
collection を過不足なく cover しなければならない。reader は、missing manifest witness entry、extra
manifest witness entry、参照先 artifact と identity または hash tuple が異なる entry を拒否する。これにより、
trusted witness を必要とする accepted obligation はすべて manifest を通じて到達可能であることが保証される。

manifest transaction は、manifest を commit する前に witness file を write し、producer-owned な
`witness_artifact_hash` value を記録する。accepted witness を名指す committed manifest entry は、
`witness_path` で witness file へ到達可能にしなければならない。reader は witness validation が要求された場合、
missing witness file や witness hash mismatch を拒否する。concrete witness payload hash construction は
[proof_witness.md](./proof_witness.md) が説明する通り producer-owned のまま残る。

## Development Artifact Entries

development artifact は diagnostics、missing-facts workflow、ATP log、explanation preview、debug/audit
tooling のための optional file である。

```text
development_artifact_entry = {
  "kind": string,
  "path": string,
  "artifact_hash": artifact_hash_string | null,
  "diagnostic_hash": diagnostic_hash_string | null,
  "related_module": module | null
}
```

development entry は manifest に意図的に列挙された場合だけ publish される。後続の
development-artifact schema が file を local/debug-only として明示し、semantic artifact hash から除外しない限り、
raw compiler IR を含んではならない。development artifact は proof witness requirement を満たさない。

後続 schema が multi-hash development file を定義しない限り、`artifact_hash` と `diagnostic_hash` のうち
ちょうど 1 つが non-null でなければならない。両方の hash field が null の entry、または両方が non-null の
entry は invalid である。`related_module` は optional navigation metadata である。

## Hash String Domains

`source_hash` は `ModuleSummary` と `VerifiedArtifact` と同じ source text hash string を使う。

```text
mizar-session/hash-text/v1:<digest>
```

その他すべての hash field は task 3 の artifact-framed 形式を使う。

```text
mizar-artifact/artifact-framed-hash-text/v1:<class>:<schema_family>:<schema_version>:<digest>
```

task 14 は少なくとも次の domain を検証する。

| Field | Required class | Required schema family/version | Notes |
|---|---|---|---|
| manifest file hash | `artifact` | `mizar-artifact/manifest`、manifest `schema_version` | canonical manifest JSON から外部で計算され、file 内には保存しない。 |
| `lockfile_hash` | `artifact` | producer-owned、valid grammar | lockfile または lock projection hash。 |
| `verifier_config_hash` | `interface` | producer-owned、valid grammar | verifier configuration fingerprint。 |
| `module_artifact_entry.source_hash` | `mizar-session/hash-text/v1` | なし | 厳密な source text hash。 |
| `module_artifact_entry.artifact_hash` | `artifact` | `mizar-artifact/verified-artifact`、参照先 artifact schema version | published `VerifiedArtifact` store-level artifact hash。 |
| `module_artifact_entry.interface_hash` | `interface` | `mizar-artifact/verified-artifact`、参照先 artifact schema version | 参照先 artifact と一致しなければならない。 |
| `module_artifact_entry.implementation_hash` | `implementation` | `mizar-artifact/verified-artifact`、参照先 artifact schema version | 参照先 artifact と一致しなければならない。 |
| `module_summary_hash` | `artifact` | `mizar-artifact/module-summary`、参照先 summary schema version | optional `ModuleSummary` store-level artifact hash。 |
| `module_summary_interface_hash` | `interface` | `mizar-artifact/module-summary`、参照先 summary schema version | optional summary interface hash。 |
| `registration_summary_hash` | `artifact` | `mizar-artifact/registration-summary`、参照先 summary schema version | optional `RegistrationSummary` store-level artifact hash。 |
| `registration_interface_hash` | `interface` | `mizar-artifact/registration-summary`、参照先 summary schema version | optional registration interface hash。 |
| `manifest_proof_witness_entry.obligation_fingerprint` | `interface` | producer-owned、valid grammar | `ProofWitnessRef` と一致しなければならない。 |
| `manifest_proof_witness_entry.witness_artifact_hash` | `artifact` | producer-owned、valid grammar | `ProofWitnessRef` が記録する witness payload hash と一致しなければならない。 |
| `diagnostics_hash` | `diagnostic` | producer-owned、valid grammar | optional diagnostics/explanation hash。 |
| `development_artifact_entry.artifact_hash` | `artifact` | producer-owned、valid grammar | optional development artifact hash。 |
| `development_artifact_entry.diagnostic_hash` | `diagnostic` | producer-owned、valid grammar | optional diagnostic payload hash。 |

field が別の mizar-artifact schema を参照する場合、reader は参照先 file が記録する class と schema
family/version の両方を検証する。producer-owned hash は自身の family と version を保持するが、valid な
artifact-framed spelling と required class を使わなければならない。

## Canonical Ordering

writer は serialization 前に collection を sort する。reader は unsorted collection と duplicate
identity key を拒否する。

ordering key:

- `modules`: module identity。
- `module_artifact_entry.proof_witnesses`: `obligation_id`、`obligation_fingerprint`、`witness_path`。
- `development_artifacts`: `kind`、`path`、`related_module`。

nullable な `related_module` では、`null` はどの module identity よりも先に sort する。non-null value は
`modules` と同じ module identity ordering で sort する。

duplicate identity key:

- `modules`: module identity。
- `module_artifact_entry.proof_witnesses`: `obligation_id`。
- `development_artifacts`: `kind`、`path`。

source traversal order、worker completion order、filesystem order、manifest transaction staging order は
canonical manifest byte に影響してはならない。

## Reader Requirements

manifest reader は:

- temporary manifest path ではなく `artifact-manifest.json` だけを読む。
- field を解釈する前に schema-version compatibility を検査する。
- unknown field、missing required field、malformed path、malformed hash string、wrong hash class、
  duplicate identity、unsorted collection を拒否する。
- normalization 後に artifact root を出る path を拒否する。
- 参照先 module artifact を manifest entry だけを通じて load する。
- consuming command、publication policy、transaction commit path が要求する場合、参照先 artifact hash を
  検証する。
- module entry を検証する場合、参照先 `VerifiedArtifact` の module identity、source hash、
  interface hash、implementation hash、full proof witness reference set が manifest entry と厳密に一致することを
  要求する。
- bad manifest を修復または trust するために internal cache record へ fallback しない。

reader failure は artifact diagnostic である。missing または corrupt な参照先 file は proof evidence ではなく、
cache hit によって黙って置き換えてはならない。

## Manifest Transaction Protocol

manifest manager は package ごとに serial に実行される。parallel module artifact write は独立した
module update を生成し、manifest manager がそれらを 1 つの package-level publication transaction に畳み込む。

### Begin

transaction の begin は次を記録する。

- package identity と artifact root。
- current manifest hash。manifest が存在しない場合は `null`。
- obsolete-writer rejection のために caller が供給する build snapshot freshness guard。
- diagnostics と temporary file name に使う local session/snapshot/task identifier。

local session と snapshot identifier は、freshness guard を含めて transaction state に限られる。published
manifest へ serialize されず、manifest canonical byte にも参加しない。freshness guard は manifest schema
から見れば opaque であり、snapshot ordering を所有する build coordinator が供給する。

### Stage

commit 前に、staged module update が参照する file はすべて write、flush、hash validation 済みでなければならない。
staged update は final path と hash だけを記録する。temporary path は candidate manifest に現れてはならない。

manager は次の場合に staged update を拒否する。

- path が artifact root の外にある。
- 必須の参照先 file が存在しない。
- 記録された hash が参照先 canonical artifact、producer-owned payload、または参照先 artifact field と一致しない。
- 同じ module identity を持つ 2 つの staged module update が異なる canonical content または hash を持つ。
- update が raw IR、scheduler state、internal cache record、proof authority state を manifest entry として
  publish しようとする。

### Commit

commit は次の手順を行う。

1. current manifest を読み直し、その hash を transaction の `base_manifest_hash` と比較する。
2. transaction の snapshot freshness guard を、現在 active な package snapshot に対して検査する。
3. transaction が obsolete なら abort する。base hash が変わっていれば abort または rebase する。rebase は
   active snapshot に対して staged content を refresh し、freshness check を繰り返さなければならない。
4. unchanged current entry と staged update を merge し、すべての entry を canonical key で sort する。
5. candidate manifest を canonical JSON で serialize する。
6. すべての entry path と参照先 hash を disk から検証する。
7. candidate byte を artifact root 内の temporary manifest path に書く。
8. temporary manifest file を flush する。
9. temporary manifest を `artifact-manifest.json` の上へ atomic rename する。
10. platform が対応していれば containing directory を flush する。
11. final manifest path が新しい entry を名指した後にだけ build event を publish する。

commit success は、必要な rename と、対応 platform における directory flush が完了した時点で初めて成立する。
atomic rename 前のいずれかの step が失敗した場合、以前の manifest が authoritative のまま残る。atomic rename
が成功したが、directory flush に対応する platform で containing directory flush が失敗した場合、commit は
artifact I/O diagnostic を返し、build event は publish されず、cache promotion は許可されず、caller は以前の
manifest hash を last durable authority として扱う。recovery はなお manifest-first である。crash または reopen
の後、consumer は surviving final `artifact-manifest.json` だけを読み検証し、temporary file や orphaned file は
読まない。rename と必要な flush が成功した場合、temporary file や orphaned file の cleanup が失敗しても、
新しい manifest が authoritative である。

successful manifest commit は、新しい publication に依存する cache record または cache index を promote できる
最初の時点である。`mizar-artifact` は manifest commit の success または failure を報告する。
`mizar-cache` が cache promotion と discard policy を所有する。

## Recovery From Interrupted Commits

recovery は manifest-first である。

- manifest rename 前に中断された transaction では、以前の manifest が authoritative のまま残る。
- manifest rename 後の supported directory-flush failure は、producer、event、cache-promotion semantics では
  failed commit である。recovery はそれでも surviving final manifest path だけを読み検証する。
- committed manifest から到達できない completed artifact または witness file は reader から無視され、後で
  garbage collect してよい。
- temporary manifest file は publication index ではなく、active transaction が所有していなければ削除してよい。
- final manifest file が missing、unreadable、unsupported schema の場合、consumer は artifact root を scan する
  代わりに artifact diagnostic を報告する。
- final manifest が missing または hash-mismatched file を参照する場合、consumer は該当 entry について
  artifact integrity diagnostic を報告する。
- failed manifest commit のために staged された cache record または cache index は `mizar-cache` により
  staged のまま残されるか破棄される。proof authority になってはならない。

この recovery rule により、package は一度にちょうど 1 つの complete manifest version を通じて観測される:
以前の committed manifest、または新しい committed manifest である。

## Deferred Implementation

task 12 はこの仕様だけを追加する。task 13 は artifact-store write と corruption-detecting file read を実装する。
task 14 は manifest schema、validating reader、writer、transaction manager を実装する。task 17 は実 producer
projection を store と manifest publication へ接続する。より広い determinism coverage は task 18 に残る。
