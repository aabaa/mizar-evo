# mizar-proof Witness Store

> 正本: [../en/witness_store.md](../en/witness_store.md)。

## 目的

`mizar-proof` は、選択された trusted proof outcome の proof witness draft staging と
publication reference を所有する。

witness store は proof authority ではない。proof を accept せず、kernel を再実行せず、
ATP backend を実行せず、SAT solving を行わず、cache を query せず、artifact manifest
を書かない。すでに `mizar-kernel` が accept した evidence の byte と metadata を記録し、
安定した content hash を計算し、artifact publication boundary が witness の manifest
到達可能性を証明した後にだけ publication reference を公開する。

## 入力

store は、同じ obligation の deterministic status projection とも一致する trusted
selection に対する `ProofWitnessDraft` だけを消費する:

- 対応する accepted proof-obligation kernel result を持つ `KernelVerified`
  formula/substitution kernel evidence;
- `DischargedBuiltin` は internal staged outcome とし、下記の明示 file route も unpublished reference のみを持つ。

draft は次を持つ:

- `obligation_id`、`ObligationAnchor`、`obligation_fingerprint`;
- status projection と kernel metadata token から copy された selected winner class と
  selected evidence hash;
- witness payload schema family と schema version;
- canonical witness payload bytes;
- `ProofWitnessRef` version `2.0` が要求する kernel-acceptance metadata;
- verifier policy fingerprint と checker/evidence schema version;
- target binding、formula evidence、substitution evidence、provenance、optional formula
  context、accepted result、dependency artifact、build snapshot fingerprint を含む
  provenance metadata;
- non-trusted attachment のための optional diagnostics-owned provenance reference。

externally attested evidence、policy assumption、open obligation、rejected obligation、
no-selectable evidence、backend log、backend proof method、resolution trace、SMT proof object、
cache record、backend-reported axiom list は trusted witness draft を作らない。

## State Machine

witness handling には 3 つの状態がある:

1. `ProofWitnessDraft`: producer-owned bytes、trusted metadata、対応 status projection。保存・公開されず、それ自体で cache validation しない。
   metadata field は opaque、generic binder は crate-internal とし、限定 [Status token production](./status.md#source-theorem-trusted-witness-metadata-with-wire) と [Draft composition](./status.md#source-theorem-witness-draft) が fresh な検査済み metadata を供給する。
2. `ProofWitnessStagedRef`: generic `stage` は draft の path/hash/schema、obligation、selected class、provenance と `KernelVerified` の unpublished reference candidate を記録する。
   下記の明示 file route は検証した file profile/hash と unpublished builtin reference を記録し、どちらも committed reachability を与えない。
3. `ProofWitnessPublishedRef`: artifact publication boundary が staged tuple と一致し、
   committed main artifact から到達可能にする committed witness publication proof を
   供給した後にだけ `publish_ref` が返す。

artifact layer が witness bytes を write/hash できるよう、`stage` は artifact commit の前に
行わなければならない。`publish_ref` は committed manifest entry が同じ witness path、
obligation id、obligation fingerprint、witness artifact hash を参照し、module manifest
entry が同じ main `VerifiedArtifact` proof-witness set を参照した後にだけ行う。単独で一致
する witness tuple だけでは十分ではない。artifact-owned な
`CommittedWitnessPublicationProof` input は、witness entry を committed module artifact
entry または同等の committed `VerifiedArtifact` reference set に bind し、manifest の
`proof_witnesses` array が artifact の `proof_witnesses` collection をちょうど cover することを
示さなければならない。その artifact-owned token が存在するまで、source API は proof token
を public constructor なしの opaque token として保つ。caller は一致する tuple を合成しては
ならない。その publication proof が存在する前に `publish_ref` を呼ぶと error である。

`publish_ref` は commit 後に新しい artifact reference を発明しない。`stage` が出した
unpublished reference candidate が committed `VerifiedArtifact` と manifest に記録された
同じ reference であることを検証し、その同じ reference に対する published wrapper を返す。

## Stable Hashing

generic `stage` の proof witness hash は、正確な staged payload bytes と payload schema identity に対する
artifact-framed hash である:

- schema family;
- schema version;
- canonical payload bytes;
- witness payload hash domain;
- selected accepted evidence hash;
- verifier policy fingerprint;
- obligation fingerprint。

hash には temporary file path、staging directory name、arrival order、backend completion
time、process id、wall-clock time、random data、cache-hit metadata、manifest commit timing を
含めてはならない。

2 つの draft が同一の payload bytes と同一の hash input を持つ場合、同じ witness hash を
生む。accepted evidence hash、policy fingerprint、obligation fingerprint、payload schema
version、payload bytes など trusted input が変わる場合、witness hash は変わるか staging
が reject される。

### Payload hash value

`ProofWitnessDraft::payload_hash_value(&self) -> CanonicalJson` は既存 hash encoder を変更せず、7 key の generic OBJECT の owned copy を返す。key は hash_domain、obligation_fingerprint、payload_bytes_hex、payload_schema_family、payload_schema_version、selected_evidence_hash、verifier_policy_fingerprint とする。既存 encoding/domain を保持し、caller の編集は draft を変更しない。I/O、validation、新 producer/schema は追加しない。[published schema](../../mizar-artifact/ja/store.md#schema-versions) に必須の top-level schema_version を持たず、実 file は [unreferenced/unpublished](../../mizar-artifact/ja/store.md#store-layout) のままである。この旧 profile は header なしのままとし、[別の file schema](#source-theorem-witness-file)でも変更しない。ここへの header/exclusion の追加、raw STRING/composite profile の代用、staging・Builtin/publication guard・acceptance・reuse credit の変更は禁止する。

既存 source_theorem_reflexivity_witness_draft_matches_captured_inputs の元の pure check 後だけを拡張し、従来の全 line/helper/fixture を保持する。実 native Disk/OpenBuffer capture の draft/wire から独立に全7 key の expected OBJECT を組み、新しい値、既存 Artifact writer/readback の全値・canonical 全 bytes、writer/read/stage digest と family/version/domain、replay、draft purity を比較する。既存 temporary root の unpublished-witness-hash-values subroot で draft path は変更せず、従来の no-file assertion を維持する。Builtin staged reference は None のままとし、committed metadata の捏造、新 helper、lower I/O matrix は禁止する。publication、currentness、Hit、CompleteSlice、広い gate 達成とはしない。

## Source theorem witness file
Status の[実生成](./status.md#source-theorem-witness-file-candidate)が、schema_version、schema_family、obligation_id、obligation_fingerprint、payload_schema_family、payload_schema_version、payload_bytes_hex、kernel_acceptance の8 key の別 top-level OBJECT を作る。kernel_acceptance 以外は string。file family は `mizar-proof/source-theorem-witness-file`、version1.0、Artifact class、全 canonical OBJECT を除外なしに hash する。crate-private SOURCE_THEOREM_WITNESS_FILE_SCHEMA_FAMILY は生成 hash・読戻し互換性・読戻し hash で共有し、公開 schema 型/helper は作らない。
元の実 obligation id と Interface fingerprint family `mizar-proof/source-theorem-original-obligation`1.0 を使う。payload は同じ検査の wire family `mizar-proof/source-theorem-reflexivity-checked-wire`2.0 の非空・小文字・偶数長 ASCII hex。kernel_acceptance は既存 [Artifact metadata OBJECT](../../mizar-artifact/ja/proof_witness.md#conceptual-shape)そのもので、formula_context_hash 不在は明示 null、evidence_schema_version は2.0。旧7 key hash、raw STRING/composite identity/ref と混同しない。
`read_source_theorem_witness_file(&PublishedArtifactRead, &ProofWitnessRef) -> Result<Vec<u8>, String>` は構造データだけを検証する。OBJECT を要求し、path付き SchemaVersionSupport(family,1,0,UpToSupported) で schema_version を最初に確認し、次に schema_family の完全一致（不互換は family・実 version・対応範囲・path を示す）、正確な key/型、固定 payload profile、安全な hex 復元を検査する。migration/forward-minor は宣言しない。byte slice より先に非ASCII、空・奇数長・大文字・非hex を拒否する。
全 file hash を新 domain で再計算し、read.artifact_hash=Some(再計算値) を要求する。file の元 identity/metadata、実 read.path、再計算した typed hash、固定 Builtin/formula class から普通の reference3 JSON を組む。実 path と expected.witness_artifact_hash で未変更 Artifact read_builtin_proof_witness_ref を呼び、返る ref 全体と expected の一致、および元 fingerprint/evidence version の profile 一致を要求する。wire bytes だけを返し、下位 error は String で伝播する。kernel evidence の parse/replay、受理、確定公開 token は作らない。
canonical parse・duplicate key/UTF-8/path/I/O integrity・ref/metadata 検証は Artifact が所有する。Builtin stage/publication guard、status/cache gap、不完全 slice は不変で、別の committed publication owner ができるまで新 file/ref は unreferenced である。
### Source theorem witness file tests
既存 term_proof_tests に source_theorem_witness_file_matches_genuine_artifact_transport と source_theorem_witness_file_rejects_invalid_schema_and_associations だけを追記し、旧全 byte/import/helper/fixture を保持する。実 native Disk/OpenBuffer・Frontend・元 resolver 入力を使う。新しい各成功の前に未変更の元ソース draft 取得を独立に呼び、triple 全体と独立に組んだ8 key/file/metadata/hash/ref を比較する。実 Artifact write/read の canonical 全 byte・hash/domain・復元 wire・反復・入力不変性・実 source/policy 変化と、元 Open/incomplete/miss/publication gap/stageNone を確認する。
負例は実生成物から作った非信頼データを実 Artifact transport に通す。内容検証より先の互換性、family、unknown/missing/型、hex、metadata、別 identity/policy/path/ref/hash、read hash 不在/不一致を扱う。schema error に到達する generic read の expected hash 省略は authority ではない。実上流 error/優先順は旧取得と比較し、下位 parser/I/O/kernel 行列を複製せず、架空の正の Accepted/token/receipt/complete slice を作らない。構成不能な guard は静的に確認する。

## Explicit source theorem file staging
`ProofWitnessDraft::stage_source_theorem_file(self, read: &PublishedArtifactRead, expected: &ProofWitnessRef) -> Result<ProofWitnessStagedRef, String>` は既存 draft を消費し、bytes/metadata と別 file profile を構造的に対応付ける。draft 最初の getter 直前に inline method 一つを加える。
最初に既存 read_source_theorem_witness_file(read, expected) を呼び、error String を保つ。続いて draft class DischargedBuiltin、obligation id/fingerprint、witness path、kernel_acceptance 全体の一致、checked-wire family2.0/canonical_bytes_required=true、復元 bytes と draft payload 全体の一致、expected_selected_proof_witness_hash absent を要求する。不一致は内部 source theorem witness file staging mismatch とする。
既存 stage(self) を呼び error String を保つ。payload_schema を SOURCE_THEOREM_WITNESS_FILE_SCHEMA_FAMILY1.0/canonical=true、staged_payload_hash を expected.witness_artifact_hash、artifact_reference_candidate を Some(expected.clone()) にだけ置き換え、他の staged field は全て保つ。旧 generic hash/wire profile と別 canonical-file hash/profile を同一視せず、元 draft の再 hash/再解釈/上書きをしない。
これは unpublished structural association であり任意の caller draft bytes が Kernel 検査済みという認証ではない。checked-wire authority は実 source producer に残す。evidence の parse/replay、Kernel、I/O、filesystem/snapshot currentness 認証は行わない。generic stage、builtin publish_ref 拒否、status/reuse gap、raw 不完全 slice/MISS を保ち、committed token/witness publication/受理/enclosing gate credit を与えない。
### Explicit source theorem file staging tests
既存 term_proof_tests 最後に source_theorem_file_staging_matches_genuine_file_and_legacy_boundaries、source_theorem_file_staging_rejects_genuine_cross_associations、source_theorem_file_staging_preserves_reader_refusals_and_inputs と、それら三つの実 family が使う private with_source_theorem_file_staging_inputs 一つだけを追記する。全 old bytes/helper/fixture を保つ。
新 API 前に実 native Disk/OpenBuffer capture、元 unaugmented Resolver inputs、既存 source file-candidate tuple 全体、generic stage、実 Artifact write/read/recovery を得る。保持する staged 全 field、別 file schema/hash/reference、反復/不変、元 status/slice 全体/publication/MISS を比較する。実 old 値を使い、新 hash grammar や編集間安定性を仮定しない。
実 source/policy/path の別 products では先に reader 成功を示してから対応違いを拒否する。実 file 由来の coherent rehash 付き untrusted wire/identity/metadata 編集と実 lower-reader failure も使い、事前の lower String と比較し実拒否条件を示す。架空 positive draft/check/Core/VC/Accepted/token/commit や下位 matrix 重複を作らず、構築不能 trusted-class/selected-hash guard は静的 review とする。

## Publication References

`KernelVerified` formula/substitution evidence では、`stage` が次を持つ unpublished
`ProofWitnessRef` version `2.0` candidate を準備する:

- `proof_status = "kernel_verified"`;
- `evidence_kind = "formula_substitution_kernel_evidence"`;
- staged witness path と witness artifact hash;
- accepted proof-obligation kernel evidence boundary から copy した kernel acceptance
  metadata。

`publish_ref` に成功すると、committed manifest reachability が証明された後にだけ、その同じ
reference を publication-reachable として返す。

store は non-trusted status の `ProofWitnessRef` を publish してはならない。unsupported
trusted status を `kernel_verified` に書き換えてもならない。

`DischargedBuiltin` publication は `external_dependency_gap` を保ち、実 Artifact 所有 committed reachability が供給されるまで `publish_ref` は拒否する。
Generic `stage` は internal hash/reference absent を保つ。明示 file route は別 file hash の unpublished reference3.0 のみを保持してよい。
どちらも selected_proof_witness_hash を供給せず、必須 deterministic-discharge reuse input を置き換えず、status/publication/cache authority を昇格しない。

## Provenance Metadata

staged record と published record は、diagnostics と reuse validation に必要な provenance を
保持する:

- build snapshot と producer identity;
- selected candidate id と selected winner class;
- kernel evidence origin;
- target VC fingerprint と obligation fingerprint;
- dependency slice と dependency artifact fingerprint;
- verifier policy fingerprint;
- checker と evidence schema version;
- 利用可能な場合の accepted result hash と trusted used-axiom reference hash;
- advisory backend data の diagnostics-owned reference。

trusted value が存在する場合、provenance metadata は selected status projection、kernel
acceptance metadata、trusted used-axiom token と一致しなければならない。provenance
metadata は trust boundary を広げない。dependency artifact fingerprint は task 11 では
producer-owned reuse metadata として保持される。committed dependency artifact への
authoritative binding は artifact/cache integration まで deferred である。backend log、
externally attested citation、cache record、diagnostic hint は diagnostic または
reuse-validation material のままである。

## Failure Semantics

store は次を reject するか gap として報告する:

- unsupported witness class または evidence kind;
- missing または mismatched accepted evidence hash;
- status projection と kernel acceptance metadata に一致しない draft selected class、
  selected evidence hash、obligation identity、policy fingerprint、selected candidate id、
  kernel evidence origin、schema version、trusted `used_axioms` reference;
- malformed payload schema identity;
- payload schema が canonical bytes を要求する場合の empty payload bytes。byte-level の
  canonicality validation は concrete producer-owned schema validator まで deferred;
- `proof-witnesses/` から escape する witness path;
- staged bytes と manifest reference の hash mismatch;
- 1 つの obligation に対する duplicate manifest reference;
- matching committed witness publication proof が存在する前の `publish_ref`;
- committed main artifact に bind されていない、または `VerifiedArtifact.proof_witnesses`
  collection をちょうど cover しない manifest witness entry;
- stale build snapshot または mismatched obligation fingerprint;
- externally attested、assumed、open、rejected、no-selectable evidence を trusted witness
  material として publish しようとすること。

failure は deterministic diagnostic または typed store error である。trusted proof status に
ならず、trusted `used_axioms` も合成しない。

## Cache And Reuse Boundary

publish 可能な `KernelVerified` witness について、selected witness hash と published
witness ref は proof-reuse validation に参加するが、proof authority ではない。cache record
は、witness hash、selected evidence hash、obligation fingerprint、policy fingerprint、
schema version、accepted proof-obligation kernel metadata が現在の validation predicate と
すべて一致する場合だけ proof を reuse してよい。dependency artifact fingerprint は、
downstream artifact/cache owner が authoritative binding を供給した後にだけその predicate
の一部になる。task 11 では producer-owned reuse metadata として保持する。selected
candidate は、現在の
artifact witness schema が参照できる witness payload artifact hash
（`witness_artifact_hash`）を持つ場合、`selected_proof_witness_hash` を保持してよい。
その hash は `ProofWitnessRef` metadata object の hash ではなく、committed publication proof
でもない。committed `ProofWitnessPublishedRef` は、artifact boundary が matching committed
witness publication proof を供給した後に `publish_ref` からだけ得られる。artifact support
が存在するまで、`DischargedBuiltin` reuse は `deterministic_discharge_hash` を使い続ける。
internal staged hash は selected proof witness hash ではない。cache hit は witness を
publish できず、non-trusted status を昇格させられず、trusted `used_axioms` も作れない。

## Deferred And External Dependency Gaps

| Gap | Classification | Handling |
|---|---|---|
| `WITNESS10-G001` | `external_dependency_gap` | `mizar-artifact` `ProofWitnessRef` version `2.0` は現在 `kernel_verified` formula/substitution evidence だけを受理する。`DischargedBuiltin` witness publication は unsupported のままであり、collapse してはならない。 |
| `WITNESS10-G002` | `deferred` | artifact manifest commit integration が、`publish_ref` に必要な committed witness publication proof を供給する。これには committed main artifact への binding と `VerifiedArtifact.proof_witnesses` の exact coverage が含まれる。この spec は token/validation contract を定義するが manifest は書かない。 |
| `WITNESS10-G003` | `deferred` | formula/substitution witness bytes 以外の concrete payload schema は producer-owned のままである。store は schema identity と bytes を hash するが backend proof payload は解釈しない。 |
| `WITNESS11-G001` | `external_dependency_gap` | task 11 は、`mizar-artifact` が artifact-owned committed publication proof token を公開するまで `CommittedWitnessPublicationProof` を opaque かつ external には construct 不可能に保つ。internal test は token を構築してよいが、production caller は artifact integration を待たなければならない。 |
| `WITNESS11-G002` | `external_dependency_gap` | 汎用 caller-metadata 受入れは未提供。限定された検査済み token の経路は [State Machine](#state-machine) が所有する。 |
| `WITNESS11-G003` | `deferred` | task 11 は payload schema が canonical bytes を要求するかを記録し、その attestation 下では empty payload を reject するが、byte-level canonicality は concrete payload schema が validator を公開するまで producer-owned のままである。witness hash は引き続き exact bytes と schema identity を覆う。 |
| `WITNESS11-G004` | `deferred` | task 11 は `dependency_artifact_fingerprint` を producer-owned provenance として保持する。committed dependency artifact と cache validation への binding は artifact/cache integration まで deferred であり、それ単体で witness status を昇格してはならない。 |

## Public Enum Policy

task 14 は public-enum forward-compatibility procedure をこの module に適用する。
public witness-store error enum は downstream-facing API surface であり、
`#[non_exhaustive]` を維持しなければならない。downstream consumer は wildcard match
arm を保つ。新しい error variant には paired spec/test review が必要であり、opaque な
publication-token boundary や trusted-metadata boundary を弱めてはならない。

| Enum | Compatibility decision |
|---|---|
| `ProofWitnessStoreError` | forward-compatible |

No exhaustive public enum exceptions are owned by this module.

## Non-Goals

witness store は proof search、premise selection、substitution invention、ATP/SAT backend 呼び出し、
kernel 呼び出し、cache query、artifact manifest write、placeholder witness reference publication
を行わない。
