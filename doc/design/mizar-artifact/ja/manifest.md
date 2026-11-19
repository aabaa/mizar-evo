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
- store file I/O implementation。これは `store.md` と `src/store.rs` が所有する。
- scheduler ordering と cache promotion policy。これらは caller-supplied transaction
  freshness check と commit result としてだけ表現される。

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

`artifact_file` は宣言 enclosing profile が選ぶ canonical main artifact file を指す。`artifact_hash` は published
file の store-level artifact hash である。すなわち参照先 schema が宣言する hash exclusion 後の canonical
JSON を class `artifact` で frame した hash であり、raw filesystem-byte hash ではない。`interface_hash` と
`implementation_hash` は参照先 main artifact envelope 内の top-level hash と一致しなければならない。

summary file field は optional である。task staging により、すべての sidecar summary が emit される前に
verified artifact が公開されることがあるためである。3 つの `module_summary_*` field はすべて
`null` であるか、すべて non-null である。`registration_summary_*` field も同じ規則に従う。

`diagnostics_hash` は local diagnostic index または explanation bundle のための optional
diagnostic-payload hash である。proof authority ではなく、`VerifiedArtifact` に埋め込まれた diagnostic の
代替でもない。

## Declared enclosing profiles

manifest schema／通常 API は1.0を保ち、witness entry は既存 id/fingerprint/path/hash tuple のままで status discriminator を加えない。参照する main artifact は宣言 artifact-hash schema version により、既知の通常上位reader profileまたは明示2.0 reader profile を選び、未知 major/minor は拒否する。実 store-framed hash、expected module/interface/implementation hash、source/version 検査、正確な witness coverage と optional file reachability を保つ。共有する既存の上位 reader を直接使い、新 transaction／token／acceptance path は作らない。対応する通常上位artifactと既存 manifest／transaction 動作を保ち、構造的に有効な明示 reference から source 受理・完全 Proof identity・committed Proof publication 権限を導かない。

commented reference は `artifact_hash` の `mizar-artifact/commented-verified-artifact` version1.0 で明示選択し、interface/implementation reference は同じ family/version と既存 class を使う。未知 family と未対応 commented version は retry せず拒否する。store は外側 profile の domain と [codec exclusion](../en/comment_projection.md#structural-codec-profile) を使い、reader expected hash は外側 envelope を束縛する。完全 module/source identity、witness coverage、sidecar 検査は検証済み normal1.2 inner artifact を使い、inner version/hash と outer 値は比較しない。既存 transaction guard と atomic replacement を維持し、source authenticity・runtime currentness・新 publication eligibility は与えない。

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

producer-owned witness writer は、manifest transaction が manifest を commit する前に
`witness_artifact_hash` value を記録し、witness file へ到達可能にする。accepted witness を名指す
committed manifest entry は、`witness_path` で witness file へ到達可能にしなければならない。reader は
witness validation が要求された場合、missing witness file や witness tuple mismatch を拒否する。task 14 は
witness payload hash を再計算しない。concrete witness payload hash construction は
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
| `module_artifact_entry.artifact_hash` | `artifact` | 上記の selected main family/version | published main-envelope store-level artifact hash。 |
| `module_artifact_entry.interface_hash` | `interface` | 上記の selected main family/version | 参照先 envelope と一致しなければならない。 |
| `module_artifact_entry.implementation_hash` | `implementation` | 上記の selected main family/version | 参照先 envelope と一致しなければならない。 |
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
- 記録された hash が参照先 canonical artifact または参照先 artifact field と一致しない。
- task-14 reachability validation が適用される producer-owned payload path へ到達できない。payload hash
  recomputation は producer-owned payload schema を待つ。
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
6. すべての entry path と参照先 canonical artifact hash を disk から検証する。producer-owned payload
   reachability を検証し、payload hash recomputation は producer-owned schema に deferred とする。
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

### Retained commit result
`ManifestCommit` は既存の `ManifestTransaction::commit` 成功後に返る manifest と store write をそのまま保持する。既存 field は private とし、公開 type/path と Debug/Clone/PartialEq/Eq は維持する。caller constructor、mutable access、新 token/transaction は追加しない。
既存値への access は `manifest(&self) -> &ArtifactManifest`、`write(&self) -> &PublishedArtifactWrite`、`into_manifest(self) -> ArtifactManifest` とする。参照は read-only、owned extraction は manifest を move して write を破棄する。Build は実 write hash を先に読み、clone せず manifest を consume する。公開 summary は plain metadata のままとする。
過去の transaction-origin 値の記録であり、現在の file integrity、registry/native/snapshot binding、producer input authenticity、witness payload hash validation、proof acceptance、committed Proof publication、cache promotion は保証しない。既存 validation/error/write/flush 順、serialized schema/profile、caller freshness check と従来の保証は変更しない。
#### Retained commit result tests
既存 manifest/tests.rs に `commit_result_retains_actual_transaction_values` だけを追加する。既存の空の structural seed（proof/module producer なし）で実 transaction を実行し、実際に書かれた manifest/hash を読み、両 accessor の全値と owned extraction を比較する。temporary manifest を一度上書きし、保持結果は不変で fresh read は破損を拒否することを確認する。currentness credit は与えない。
既存 ManifestCommit の Rust doccomment に外部 compile-fail 例を3件置き、caller の field-literal construction と各 field の直接置換を拒否する。型付き関数引数を使い、成功 record を捏造しない。既存 test/helper/fixture の全 byte を維持し、新 helper/type/adapter/dependency や lower-validator matrix は追加しない。

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

## 公開 enum の前方互換性

task 19 は frontend task 25 の public-enum 手続きを manifest API に適用する。この
module が所有するすべての public enum は forward-compatible API surface であり、
`#[non_exhaustive]` のままにしなければならない。downstream consumer は match 時に
wildcard fallback arm を持たなければならない。

これは API 互換性の判断であり、reader の寛容化ルールではない。manifest reader は、
将来の schema revision と version policy が受け入れ方法を明示しない限り、
unknown serialized enum value を引き続き拒否する。

| Enum | 前方互換性の判断 |
|---|---|
| `ManifestError` | manifest schema、transaction、reference-validation、store-wrapping diagnostic を拡張できるよう non-exhaustive。 |

この module は exhaustive な public enum 例外を所有しない。

## Implementation Status

task 12 はこの仕様を追加した。task 13 は artifact-store write と corruption-detecting file read を実装した。
task 14 は `src/manifest.rs` に manifest schema、validating reader、writer、transaction manager を実装する。
これには manifest-first read、決定的 transaction commit、参照先 `VerifiedArtifact` と summary の validation、
正確な `ProofWitnessRef` coverage、development artifact reachability check が含まれる。concrete witness payload
hash construction と development payload hash recomputation は producer-owned external dependency として残る。
task 17 は実 producer projection を store と manifest publication へ接続する。より広い determinism coverage は
task 18 に残る。

## Imported Promotion Authority Boundary

将来の imported promotion publication/loading は真正な Proof-produced evidence と正確な algorithm/contract/equation/group/dependency identity を運び、Proof が acceptance validator のまま。runtime export validation、manifest flag、hash だけでは promoted premise を有効にしない。proof provenance が不足/古い場合、exporting module の再証明で代替せず VC consumer を block する。

| 既存 transport producer/validator | 利用可能な binding と欠けた権威 |
|---|---|
| [ProofWitnessRef、KernelAcceptanceMetadata、read_proof_witness_ref](../../../../crates/mizar-artifact/src/proof_witness.rs) | obligation/fingerprint、status/evidence kind、payload hash、target/formula/substitution/provenance/result、policy/profile/schema metadata。shape/version/hash domain/path を検証するが exporting Proof の真正性や aggregate promotion acceptance は検証しない。 |
| [VerifiedArtifact、VerifiedExport、BuildProvenance、DependencyArtifactHash](../../../../crates/mizar-artifact/src/verified_artifact.rs) | source/module/export signature、interface/implementation identity、projected obligation status、witness、toolchain/lock/config/dependency hash。構造検証は正確な promotion-equation/group-to-proof binding を与えない。 |
| [ModuleSummaryIdentity と ExportedSymbolSummary](../../../../crates/mizar-artifact/src/module_summary.rs)、[project_summary](../../../../crates/mizar-resolve/src/module_summary_reuse.rs) | package/version/lockfile、dependency interface、opaque exported signature/label identity、status projection。Resolver lowering は imported shell を作り、認証済み premise を作らない。 |
| [validate_module_references と validate_witness_coverage](../../../../crates/mizar-artifact/src/manifest.rs) | main artifact hash/module/interface/implementation/source の一致と正確な witness-reference coverage。optional witness-file validation は到達可能性だけを検査し、witness payload 再計算、kernel replay、producer authentication を行わない。 |

portable producer/consumer は resolved algorithm/export identity と正確な signature/contracts、ordinary/definitional promotion と適格 equation、完全な input/guard を持つ適用 E1/E2/E4 obligation、全 mutual-group member と受理、正確な source/library、package/dependency/version/lockfile provenance、許可された proof-policy/kernel/evidence version について一致しなければならない。上記の既存 field は binding input であり、認証済み完全 tuple や固定 wire schema ではない。semantic interface compatibility は proof origin を認証せず、stale evidence reuse を許さない。
Artifact は真正な Proof output の binding を保持して publication/loading を所有し、Proof は VC59 に premise を渡す前の acceptance と compatible-policy validation を所有する。現在の producer/reader にその portable route はない。[Proof trust-transfer rule](../../mizar-proof/ja/policy.md#trust-transfer-rule) が producer-authentication の定義 gap を解決し acceptance を所有する。その boundary は六行の validation と順序付き slice を保持する。transport 実装は真正な local aggregate producer を待ち、Proof import validation と VC59 接続は別の後続 gate に残る。
既存 [witness test](../../../../crates/mizar-artifact/src/proof_witness/tests.rs) と [summary test](../../../../crates/mizar-artifact/src/module_summary/tests.rs) は wire/hash/version/status/identity constraint を検証するが accepted portable promotion は検証しない。logical contract-only import は execution artifact を必要とせず、computation artifact validation は execution owner の別 gate に残る。

### Producer transport design

これは承認済み Proof rule に基づく将来の transport design であり、現在の manifest/witness schema や実行可能 acceptance を変更しない。singleton を含む完全な promotion group ごとに署名付き publication descriptor を一つ用いる。将来の versioned manifest extension が既存 module artifact/summary とともに descriptor と detached signature を参照し、既存 reader はその extension を従来の accepted metadata として解釈できない。

**表現と認証。** [store canonicalization rule](./store.md) の canonical JSON を使い、member/prerequisite set は schema の順序、logical term/guard/equation 内部は本来の順序を保つ。重複 identity/key、未知 field/version、非 canonical byte、未対応 suite は拒否する。この transport には Ed25519 signature と SHA-256 content commitment を選ぶ。曖昧さなく framing した domain、transport version、algorithm suite、完全な canonical descriptor byte に署名し、signature 自体はその byte の外に置く。importer-controlled configuration が signing key を独立に許可した Proof execution authority と許可された implementation/version/policy scope に対応付ける。key identifier や同梱 public key は検索の hint だけであり、trust の確立や scope 拡張を行えない。mechanism はここで選定し、正確な field 名/encoding と API はこの binding を保つ後続実装に残す。

**正確な binding。** descriptor は [Proof boundary](../../mizar-proof/ja/policy.md#imported-promotion-authority-boundary) が要求する完全 tuple を commit する。logical content は inline または正確な canonical byte の SHA-256 reference とし、tuple の再構成に必要な reference は全て必須かつ推移的に保護する。import に使う公開 module artifact と logical summary の正確な byte の SHA-256 も bind する。既存 artifact identity projection が除外する provenance field も含め参照先の全 byte を hash し、既存 interface/hash/status label だけで代用しない。private source/proof body は認証された content identity だけを必要とし、importer の load/replay は不要。logical-only export に runtime artifact は不要。

**受理結果からだけの publication。** 真正で untainted な complete-group result の構築は Proof が所有する。authority-controlled publisher がその result と正確な immutable serialized object から descriptor を導出し、caller-supplied accepted flag や任意 byte の署名要求は受け付けない。signing key はその execution/publication path に制限し、汎用 signing endpoint にしない。Artifact は witness、summary、cache entry、transaction 成功から acceptance を作らない。

まず完全な unsigned descriptor と bound object を既存 transaction/freshness/flush protocol で commit し、この段階では promotion authority を与えない。成功した retained commit にその正確な result-bound descriptor/object closure が含まれることを検証した後にだけ authority が descriptor に署名する。署名はこの committed acceptance publication を証言する。次に同じ freshness check の下で signature と promotion reference を二回目の transaction で公開する。この二回目は local discoverability を与え、二回目の exporter commit の認証証拠ではない。descriptor は signature を含む最終 manifest ではなく committed object を bind し、hash cycle を避ける。rebase や snapshot 変更時は署名または publication 再試行前に accepted-result/object/dependency binding を再検証し、変更された input へ acceptance を移してはならない。全 group の promotion reference は二回目の manifest rename で同時に設置され、commit 成功には既存の flush 手順も必要。最初の commit の失敗は署名を禁止し、unsigned descriptor だけでは premise を与えない。二回目の rename 前の失敗では新しい local promotion entry は見えず、rename 後の flush 失敗は既存の failed-commit/final-manifest recovery rule に従い、producer 成功の主張を与えない。recovery は既存の final-manifest-only rule に従い、temporary/orphan file の走査から authority を再構成しない。署名再試行にも同じ真正 result と成功した descriptor/object commit が必要。認証済みの commit 済み signed descriptor を別の valid manifest で再配布することは許されるが、bound object の変更や authority 拡張はできず、unsigned manifest 単独では exporter commitment を証明しない。

**Loading と handoff。** promotion loading は final manifest、全 descriptor/signature/object reference、必須 content commitment を常に検証し、現在の任意 reference-validation default では不足する。Proof handoff に保持するのと同じ immutable byte を parse/hash し、検証と使用の間の path 差替えを防ぐ。manifest が選んだ module/summary object と descriptor commitment を比較し、root/path 制約を守り、publication の混在、member/reference 不足、破損、未対応 format を拒否する。構造検証の成功が生成するのは未信頼 publication data と正確な signed byte/signature、bound object だけ。後続 Proof import validator が現在の importer trust による signature/authority 検証、正確な resolved-object/dependency 比較、policy/schema compatibility、imported provenance 保持を所有し、全て通るまで member premise を公開しない。Artifact reader 成功を新しい kernel result にしない。logical import は runtime artifact を load せず opaque module を再証明しない。computation は別 gate を保つ。

構造/content/transaction の失敗は Artifact が報告し、authentication、authorization、semantic binding、policy refusal は Proof が所有する。どちらの失敗でも全 group の promoted premise を留保し、status/hash/cache fallback はしない。legacy または descriptor 不在でも既存 opaque signature は提供できるが promotion authority は与えない。この設計で public diagnostic code は追加しない。真正な local aggregate E1/E2/E4/group producer、この transport の実装、Proof import validation、VC59 integration は依然不在であり、実行可能 acceptance/coverage credit は生じない。
