# mizar-proof Status Projection

> 正本: [../en/status.md](../en/status.md)。

## 目的

`mizar-proof` は、決定的な proof selection を artifact 向けおよび diagnostics
向けの proof status record へ射影する責務を持つ。

status projection は proof acceptance ではない。trusted acceptance と trusted
`used_axioms` は、status が `KernelCheckStatus::Accepted`、evidence check kind が
`ProofObligation` であり、`KernelVerified` または `DischargedBuiltin` として選ばれた
`mizar-kernel` の `KernelCheckResult` だけに由来する。externally attested evidence、
backend diagnostic、backend が報告した axiom list、proof cache record、witness
metadata、policy assumption、accepted consistency check は、kernel-verified status や
trusted `used_axioms` にならない。

## 入力

projection は次を消費する:

- deterministic selection merge が作る `ArtifactProofSelection` stream;
- 現在の build に対する active `VerifierPolicy` または同等の publication profile;
- VC/artifact producer が供給する stable obligation identity:
  `VcId`、obligation id、`ObligationAnchor`、obligation fingerprint、canonical
  VC fingerprint、canonical local-context fingerprint、dependency-slice
  fingerprint;
- selected trusted class に対し、selected accepted evidence hash に bind された
  optional trusted kernel result reference;
- policy、kernel、ATP、diagnostics owner が作った optional diagnostic または
  explanation reference。

projection は stale または不一致な入力を reject しなければならない。`VcId`、
`ObligationAnchor`、arrival order、completion time、runtime duration、backend profile
runtime、cache hit だけは proof identity ではなく、status reuse に十分ではない。

## 状態モデル

internal projection result は selection winner class と projected obligation status を
分離して保持する:

| Winner class | Projected obligation status | Trusted | Notes |
|---|---|---:|---|
| `KernelVerified` | `accepted` | yes | 対応する accepted proof-obligation kernel result が必要である。accepted artifact status を publishable として project するには matching selected kernel-witness metadata も必要である。committed witness ref は `witness_store::publish_ref` が別に publish する。 |
| `DischargedBuiltin` | `accepted` | yes | 対応する accepted proof-obligation kernel result が必要である。artifact schema support が存在するまで final artifact witness publication は `external_dependency_gap` のままである。 |
| `PolicyPermittedExternal` | `externally_attested` | no | policy-controlled evidence のみ。trusted `used_axioms` を持たない。 |
| `PolicyAssumed` | `policy_assumed` internal status | no | accepted status や externally attested status と区別したままにする。現在の artifact schema にはこの public obligation status がないため、後続 schema が追加するまでは artifact publication は `external_dependency_gap` である。 |
| `PolicyOpen` | `open` | no | 利用可能なら explanation reference を持つ。 |
| `Rejected` | `rejected` | no | 選択された rejection または policy/kernel diagnostic reference を持つ。 |
| `NoSelectableEvidence` | `open` または `rejected` | no | active policy が `AllowPolicyOpen` で open obligation の publish を許す場合だけ `open`。`RecordDiagnostic` は diagnostics-only であり `rejected` に射影する。それ以外も `rejected`。no-selectable-evidence explanation を持つ。 |

`not_required` は proof selection を必要としない producer-owned obligation のために
予約される。`ArtifactProofSelection` からは emit しない。

projection は `DischargedBuiltin` を `KernelVerified` に潰してはならず、
`PolicyAssumed` を `externally_attested` に潰してはならず、non-trusted status を
`accepted` に昇格させてはならない。

## Trusted `used_axioms`

trusted `used_axioms` projection は、次をすべて満たす場合だけ許可される:

1. selected winner class が `KernelVerified` または `DischargedBuiltin` である;
2. selected proof selection が trusted `used_axioms` availability を報告している;
3. projection input が accepted proof-obligation kernel result、またはそこから導出された
   trusted kernel-owned reference を含む;
4. accepted proof-obligation kernel evidence hash が selection の selected evidence hash と
   一致する;
5. kernel result が policy-tainted ではなく、status が `Accepted`、evidence check kind が
   `ProofObligation` である。

これらが成り立つ場合、projection は kernel-owned accepted result が供給した
trusted used-axiom reference または ordered axiom list をそのまま公開してよい。
projection は backend-reported axiom list、externally attested citation、cache
dependency record、diagnostic hint、policy-assumption dependency を混ぜてはならない。

すべての non-trusted status では trusted `used_axioms` は absent である。untrusted
diagnostic suggestion は diagnostics-owned reference としてだけ持てるが、accepted
dependency fact ではない。

## Diagnostics And Explanations

すべての non-accepted projection は、metadata と diagnostics-owned reference から
作られる安定した explanation surface を持つ:

- `externally_attested` は external admission status、policy fingerprint、selected
  class、selection reason、optional explanation ref を記録する;
- `policy_assumed` は selected class、selection reason、policy fingerprint、
  optional explanation ref を記録する;
- `open` は selected class、selection reason、ordered diagnostic ref、optional
  explanation ref を記録する;
- `rejected` は selected class、selection reason、ordered diagnostic ref、optional
  explanation ref を記録する;
- `NoSelectableEvidence` は selected class、selection reason、ordered diagnostic ref、
  optional explanation ref、generated diagnostic result id を記録する。

structured policy-assumption、open-obligation、rejection-layer、backend exhaustion
detail は diagnostics-owned payload であり、stable diagnostic または explanation ref
を通じて参照される。status projection はそれらの payload を直接 embed しない。

diagnostic ordering は architecture 19 に従う: source identity、source range、
phase order、severity、diagnostic code、stable detail key である。parallel completion
order、backend runtime、cache lookup timing は diagnostic ordering に参加しない。

## Artifact Projection

projection は stable projection data からだけ artifact obligation field を埋めてよい:

- `status`;
- trusted accepted witness が publish 可能な場合の `accepted_witness_obligation_id`;
- non-accepted outcome の `diagnostic_ref` または explanation reference;
- artifact consistency check に使う policy fingerprint と obligation fingerprint。

現在の `mizar-artifact` `ProofWitnessRef` schema version `2.0` は、
`kernel_verified` formula/substitution evidence に対する trusted `ProofWitnessRef`
だけを受理する。
そのため:

- `KernelVerified` は matching selected witness metadata が利用可能な場合だけ accepted
  artifact status を publishable として project できる。status projection 自体は committed
  witness ref を publish しない;
- `DischargedBuiltin` は accepted internal projection のままだが、まだ trusted
  artifact witness ref を publish できない; その deterministic discharge hash は
  現行 artifact schema では internal projection および proof-reuse metadata に留まり、
  accepted artifact obligation field として書いてはならない;
- `PolicyAssumed` は現在の `ObligationStatus` では lossless に表現できない。

これらは integration gap であり、placeholder witness を emit したり status 名を
置き換えたりする許可ではない。

## Proof Reuse Metadata

status projection は proof reuse の validation metadata を export する:

- selected winner class;
- projected obligation status;
- `ObligationAnchor`;
- obligation fingerprint;
- canonical VC fingerprint;
- canonical local-context fingerprint;
- dependency-slice fingerprint;
- policy fingerprint;
- selected evidence hash;
- trusted kernel selection で存在する場合の accepted proof-obligation goal polarity;
- `KernelVerified` candidate で利用可能な場合の selected proof witness payload artifact hash
  （`witness_artifact_hash`）。これは committed `ProofWitnessPublishedRef` ではない;
- 存在する場合の deterministic discharge hash;
- 存在する場合の trusted used-axiom reference hash;
- 存在する場合の external admission status;
- selected candidate id、selected-candidate provenance hash、selection reason、
  tie-break key hash を含む matching proof-evidence identity;
- artifact/cache boundary が供給する場合の dependency artifact fingerprint と
  compatible な dependency/proof-reuse schema version;
- diagnostic または explanation reference hash。

この metadata は cache validation predicate である。proof authority ではない。
reuse にはさらに、matching `ObligationAnchor`、canonical VC fingerprint、
canonical local-context fingerprint、dependency-slice fingerprint、compatible verifier
policy、matching proof evidence、compatible referenced dependency artifact と schema が
必要である。cache record は externally attested、assumed、open、rejected、
no-selectable outcome を trusted status に昇格させられず、trusted `used_axioms` も
合成できない。

status projection は上記すべての field に対する安定した proof-reuse validation hash も
export する。将来の `mizar-cache` は、その hash と structured field を reuse
predicate として比較してよいが、一致は cache validation 後に再計算を省けることを
示すだけである。dependency artifact/schema compatibility の欠落、policy
incompatibility、witness hash mismatch、deterministic discharge mismatch、
accepted-goal-polarity mismatch を含む proof-evidence identity mismatch は miss である。
一致しても `ExternallyAttested`、
`PolicyAssumed`、`Open`、`Rejected`、`NoSelectableEvidence` が `Accepted` に
昇格することはなく、trusted `used_axioms` も作られない。

complete な proof-reuse predicate は class-aware である。`KernelVerified` には
selected proof witness hash が必要であり、`DischargedBuiltin` には deterministic
discharge hash が必要である。non-trusted class は exported metadata のままで、
complete proof-reuse hit にはならない。

## Deferred And External Dependency Gaps

| Gap | Classification | Handling |
|---|---|---|
| `STATUS8-G001` | `external_dependency_gap` | 現在の artifact obligation status には distinct な public `policy_assumed` value がない。projection は内部で区別を保ち、collapse せず artifact publication を defer する。 |
| `STATUS8-G002` | `external_dependency_gap` | 現在の `ProofWitnessRef` trusted reader は `DischargedBuiltin` witness publication を reject する。projection は trusted status と deterministic discharge hash を記録してよいが、witness publication は deferred のままである。 |
| `STATUS8-G003` | `deferred` | diagnostics、artifact emission、manifest commit、cache lookup、ATP early-stop integration は後続 task がこの projection を消費する。この spec は stable metadata だけを定義する。 |

## Public Enum Policy

task 14 は public-enum forward-compatibility procedure をこの module に適用する。
すべての public status-projection enum は downstream-facing API surface であり、
`#[non_exhaustive]` を維持しなければならない。downstream consumer は wildcard match
arm を保つ。artifact-facing status enum はさらに、新しい variant を publish したり
現在の artifact field に map したりする前に artifact schema compatibility review を必要とする。

| Enum | Compatibility decision |
|---|---|
| `TrustedUsedAxiomsError` | forward-compatible |
| `ProjectedProofStatus` | forward-compatible |
| `CurrentArtifactObligationStatus` | forward-compatible with artifact compatibility review |
| `ArtifactPublicationGap` | forward-compatible with artifact compatibility review |
| `ArtifactStatusPublication` | forward-compatible with artifact compatibility review |
| `StatusProjectionError` | forward-compatible |

No exhaustive public enum exceptions are owned by this module.

## Non-Goals

status projection は ATP backend 実行、SAT solving、kernel 呼び出し、substitution
invention、premise selection、proof cache query、witness の stage/publish、artifact
manifest write、proof acceptance を行わない。

## Source canonical components

`source_registration_canonical_components(&SurfaceResolvedArena, &TypedArena, &SymbolEnv, BuildSnapshotId, VcId) -> Result<(CanonicalVcFingerprint, Hash, Hash), String>` は現在の選択 leaf の VC 全体 fingerprint、goal hash、可視 context hash をこの順で既存型・既存 content profile により返す。
同じ不変入力で source proof checker、既存 Core lowering、source VC generator をこの順に fresh 実行する。既存 Proof の generation schema source-existential-registration-v1 と VC schema vc-v1 を使う。既存 source facade の本体・検査順・error は維持し、この操作はそれらの呼出し、default discharge、Kernel 検査を行わない。
同じ source/env で [arena owner producer](../../mizar-resolve/ja/symbols.md#arena-registration-owner-payloads) を呼ぶ。全 original VC の goal と GeneratedFact premise から、実 generated Ref/Not/And/Or row をたどって到達 Core root の集合を得る。現在の実 id は走査だけに使い、caller graph や再構成 payload を受け取らない。欠損・未対応走査は内部 error registration.proof.invalid_source_components で全体拒否する。
保持した同一 checker result、局所生成 Core、変更していない局所 owner map、全到達 root 集合で [Core formula content](../../mizar-core/ja/elaborator.md#source-registration-formula-payloads) を呼ぶ。続けて全 [canonical VC map](../../mizar-vc/ja/vc_ir.md#source-registration-canonical-vc-fingerprints) を得て現在の選択 VcId を要求し、なければ同じ内部 error とする。同じ Core/map でその [goal/context 値](../../mizar-vc/ja/vc_ir.md#source-registration-canonical-goal-and-context) を得る。選択失敗より先に各 upstream error をそのまま伝播し、全成功時だけ tuple を返す。
この呼出し内で実 producer 出力を関連付け、opaque owner/formula byte を caller 入力にしない。返値は限定 content であり、実行 receipt や移譲可能な trust token ではない。source/snapshot の現在性は caller が所有する。既存 hash algorithm/domain/schema framing、汎用 anchor/dependency/status は維持し、新 encoder・公開型・field・adapter・dependency・下位 API を追加しない。
有効な tuple や未証明・偽の source body から受理を導かない。完全な正規 owner/path/source shape、phase 合成 context/dependency、6 field Proof identity、policy/evidence/witness 関連付け、trusted metadata、再利用資格、source 活性化と公開は別作業である。Proof23/Artifact25/17 と Steps6–8 は保留を維持する。
実 source の追加3系列は既存 setup を再利用する。現在 owner/leaf に対する独立 frozen-profile VC/goal/context 期待値と実下位 producer との一致、中立編集・allocator/現在 id 移動・対応済み declaration/registration 並替・module/owner/body 変更と goal/context 分離、実 foreign/malformed/unsupported 入力・不在選択・upstream error 優先・API 入力不変性を扱う。既存 fixture・期待値・全旧 test は変更せず、架空 accepted control や producer profile matrix の複製を作らない。

## Source candidate checking

`check_source_registration_candidate(&SurfaceResolvedArena, &TypedArena, &SymbolEnv, BuildSnapshotId, VcId, (LogicProfile, ConcreteFormat), &VerifierPolicy) -> Result<(KernelCheckResult, PolicyDecision), String>` は leaf 1件の fresh 検査を所有し、登録活性化や公開を行わない。encoding tuple は既存の明示 profile/format をまとめるだけで型を追加しない。source checker と Core lowering を再実行し、既存 source/VC schema で元 graph を生成して実 default Discharge を適用する。選択 NeedsAtp leaf に [実 source candidate factory](../../mizar-atp/ja/backend.md#source-registration-candidate-inputs) を使う。未対応・foreign input、不正・Discharged target、実行許可のない policy では checked result や fallback なしで error とする。
candidate の payload kind、完全な target、input/provenance hash、label、symbol mapping を局所的に生成した encoded container と比較する。この比較から UncheckedFormulaSubstitution の encoded_problem_matches を得て、既存 FormulaSubstitution 検査実行許可を要求する。caller の一致 boolean、encoded/candidate pair、context、Accepted result、hash は入力しない。backend observation は作らない。
fresh Core/default-discharge graph 全体から同じ認証済み source handoff を独立再構成する。実 target/profile を期待値とし、schema 2 を明示許可して返却 v2 byte を parse し、この別 handoff の完全な identity/producer entry から通常の [Kernel context](../../mizar-kernel/ja/checker.md) を構築する。import・未対応 context class は空 context の捏造で補わず拒否する。既存 default policy/limit と ProofObligation で通常 check_kernel_evidence を呼び、legacy audit・過去 result を使わない。
実 result を KernelPolicyInput::from_kernel_result と明示 AtpFormulaSubstitution origin で包み、既存 evaluator に渡す。実 PolicyDecision と KernelCheckResult を返し、偽の goal の実拒否 detail も保つ。通常 Accepted/untainted proof check は既存 policy により KernelVerified となり、拒否を手動昇格したり Open/Builtin/external evidence へ置換しない。policy criteria/fingerprint を保つ。
返却値はこの呼出しの検査出力であって selection・source 活性化・witness 公開・artifact status・cache reuse の移転可能な権限ではない。caller が現 source/snapshot freshness を所有する。元 Open/Builtin facade の公開動作を保つ。汎用 by/built-in integration・探索 budget・import context・witness/artifact/cache 公開と全 milestone は保留する。

## Source candidate metadata

`check_source_registration_candidate_metadata(&SurfaceResolvedArena, &TypedArena, &SymbolEnv, BuildSnapshotId, VcId, (LogicProfile, ConcreteFormat), &VerifierPolicy) -> Result<(KernelCheckResult, PolicyDecision, Option<KernelAcceptanceMetadata>, BackendCandidateEvidence), String>` はこの呼出しで実際に検査した ATP source candidate と raw acceptance 対応だけを保持する。互換 checker とこの facade は [source witness 投影](#source-candidate-witness-projection) の private payload checker を各一度呼び、不要な出力だけを捨てる。上の producer/binding/scheduling/parser/context/check/error 順序と分類、selector の独立 source/candidate 再構成と全 Builtin path を保ち、raw component/version 対応は変更しない。
通常 factory の実 candidate container を返し、この呼出しで parse/check した Formula-v2 bytes 全体、target、encoded input hash、ATP provenance、label、symbol、candidate identity を保つ。caller candidate/result/metadata は受けず、encoding 成功や返却 container 自体は受理ではない。元 Open/Builtin target を default Discharge 後の NeedsAtp target の代わりにしない。
metadata は KernelVerified と実 Accepted・untainted ProofObligation・AssertFalseForRefutation、AtpFormulaSubstitution origin の KernelPolicyInput から得る実 accepted identity が揃う場合だけ Some とする。実 handoff の local_evidence_component_hashes が必要で、対応不能なら None とし、値の捏造や check/classification 変更をしない。拒否 check は実 candidate と None を保ち、上流 error はそのまま伝える。
raw profile/formula/substitutions/provenance は Interface、family `mizar-vc/local-evidence-component/{profile,formula,substitutions,provenance}`、version 1.0 で包む。[VC canonical-v1 content profile](../../mizar-vc/ja/kernel_evidence_handoff.md#local-evidence-component-hashes) は不変で、v2 section-7 equality-instance name を含まない。返す完全な検査済み candidate bytes は name を含むが、これら component identity は v2 payload 全体の hash ではない。
policy は Interface `mizar-proof/verifier-policy`、version `(policy.schema_version(),0)`、digest は実 policy fingerprint。checker version は `(policy.checker_schema_version(),0)`、evidence version は `(parsed.schema_version(),0)` とし、handoff envelope v1 ではなく実検査 wire v2 を記録する。設定 checker requirement は実行中 implementation/toolchain の認証ではない。
target binding は Interface `mizar-vc/kernel-evidence-handoff-canonical` 1.0 と実 handoff canonical hash、accepted result は Interface `mizar-proof/kernel-accepted-evidence` 1.0 と実 ATP-origin accepted identity。import context は未対応なので formula_context_hash は None とし、通常 pipeline が検証する別の local context identity を代入しない。local component provenance と ATP candidate provenance は別の実 identity として保つ。
plain raw data であって TrustedKernelWitnessMetadata、完全 composite obligation/current-source receipt、witness reference、全 payload content hash、backend 実行、committed publication ではない。既存 trusted-token constructor、normal/Builtin reference guard、staging/unsupported publication/reuse gap は保ち、source/snapshot freshness は caller が持つ。将来の witness producer は equality name を含む完全な検査済み payload と現 context/target/policy/result を公開前に束縛する。
三つの追加実 source family で単一/aggregate・renamed source、両 format、snapshot と policy/schema/profile variant について独立した実 candidate/wire/handoff/component/policy/result を比較する。偽定義に依存する leaf は metadata なしの実拒否を保ち、独立 leaf は自分の identity を保つ。互換 checker/selector の意図、mixed/foreign/unsupported、異なる FormulaSubstitution/Builtin permission failure は不変。既存 fixture/expectation の変更や架空の受理 control は認めない。

## Source candidate witness projection

`project_source_registration_candidate_witness(&SurfaceResolvedArena, &TypedArena, &SymbolEnv, BuildSnapshotId, VcId, (LogicProfile, ConcreteFormat), &VerifierPolicy) -> Result<(ProofSelection, ProofWitnessRef, Vec<u8>), String>` は fresh な未公開の通常 ATP source 投影だけを返す。
private payload checker は既存 source/default-discharge/candidate/通常 check/metadata body 全体を保ち、既存四値と実 handoff を返す。実 caller は互換 checker（三値を破棄）、raw metadata facade（handoff を破棄）、この投影の三つ。公開 signature・check/error 順序・selector の独立再構成・全 Builtin body を保ち、既存の純粋 hex renderer を改名せず再利用する。type/field/adapter/dependency/lower-stage 変更は加えない。
実 KernelVerified decision・同一呼出し metadata・AtpFormulaSubstitution origin の実 KernelPolicyInput を要求する。既存 selector と同様に result target fingerprint と実 candidate target の一致を要求する。実 candidate_id・encoded_problem_hash・ATP provenance と実 target sort_bytes・供給 policy を既存 trusted-input/selection path に渡し、KernelVerified selection を要求する。二度目の producer/check や caller evidence は受けない。非受理・対応欠如・構造失敗は registration.proof.invalid_source_candidate、上流 error は不変で伝える。
返す bytes はこの呼出しで parse/check した Formula-v2 candidate 全体であり、section-7 equality-instance name も含む。実 handoff の別個の local context_identity_hash を保持し、canonical target-binding hash や未使用 import 用 formula_context_hash で代用しない。上の raw metadata 対応、実 parse version 2 と runtime 認証ではない設定 checker requirement を保つ。
Artifact CanonicalHashDomain を excluded path なし・version 1.0 で二つの producer profile に使う。Artifact family `mizar-proof/source-registration-candidate-witness-payload` は検査済み全 byte の小文字 hex を CanonicalJson string として hash する。canonical-v1 component hash や witness-store payload profile とは別である。
Interface family `mizar-proof/source-registration-candidate-obligation` は以下の正確な key の CanonicalJson object を hash する:
`checked_target_bytes`, `local_context_identity_hash`, `candidate_id`, `encoded_problem_hash`, `atp_provenance_hash`, `kernel_profile_fingerprint`, `verifier_policy_fingerprint`, `checker_schema_version`, `evidence_schema_version`, `target_binding_hash`, `formula_evidence_hash`, `substitution_evidence_hash`, `provenance_hash`, `formula_context_hash`, `accepted_result_hash`, `accepted_goal_polarity`, `witness_artifact_hash`。
実 target sort_bytes と raw local-context/input/ATP-provenance digest は小文字 hex、candidate_id は実 string。metadata hash と witness_artifact_hash は完全 artifact-hash string、version は正規 string、import context 不在は null、実 polarity は stable string とする。generic VC/discharge/dependency identity を代入しない。
既存の通常 ProofWitnessRef 2.0 を使い、obligation_id は実 candidate_id、KernelVerified/FormulaSubstitutionKernelEvidence と実 metadata/hash を持つ。path は `proof-witnesses/source-registration-candidate-v1/<lowercase-payload-digest>.bin`。既存の通常 Artifact value writer で構造検証して selection/reference/bytes を返し、Artifact codec/schema は変更しない。
未公開 tuple は TrustedKernelWitnessMetadata・draft・stage/file write・committed-publication proof・selected witness/discharge reuse hash・source 活性化・runtime lease を作らない。source/snapshot freshness は caller 所有。normal/Builtin version guard、不完全 dependency slice、未対応 publication/reuse と全 Proof23/Artifact25/17/Steps6–8 は保留する。
三つの追加実 source family は独立 wire/handoff/metadata/selection と通常 reference round-trip を比較し、単一/aggregate/renamed・両 format・snapshot・policy/schema/encoding profile・全 hash participant（完全 equality-name bytes と別個の local context を含む）を試験する。mixed/foreign/wrong/Discharged/unsupported・異なる scheduling permission・偽定義依存の実 Kernel 拒否と独立受理 leaf identity を保つ。構造 hash/reference mutation を source 受理や公開の証拠にせず、既存 fixture/expectation/test 意図を保つ。

## Source candidate selection

`select_source_registration_candidate(&SurfaceResolvedArena, &TypedArena, &SymbolEnv, BuildSnapshotId, VcId, (LogicProfile, ConcreteFormat), &VerifierPolicy) -> Result<ProofSelection, String>` は同一 source 内の限定 orchestration を所有する。同じ immutable input と供給 policy で check_source_registration_candidate を新たに呼び、caller check tuple・Accepted flag・evidence pair を受け取らない。元 checking/Builtin facade の公開動作を保つ。
同じ input から既存 source checker/Core/original VC/default Discharge producer を独立に再実行し、実 source candidate factory の encoded container と candidate を取得する。既存 schema/API を使い、hash/codec/parser/check 実装を追加しない。fresh Kernel target の algorithm/digest と実 encoded target fingerprint の一致を selection 前に要求する。現 source/snapshot freshness は caller 所有で、局所対応から runtime lease や source reuse receipt は得られない。
実 candidate_id を CandidateSourceId::new に渡す。実 current decision が KernelVerified の場合だけ、この fresh result と AtpFormulaSubstitution origin による ProofEvidenceCandidate::from_trusted_kernel_input を使い、trusted evidence を得られなければ error とする。他は実 decision を ProofEvidenceCandidate::new で保持し、trusted marker・架空 evidence hash を加えない。両者に実 candidate provenance hash を運び、backend/profile/format priority・witness hash を捏造しない。
fresh target の既存 TargetVcFingerprint::sort_bytes identity、実 encoded input_hash、供給 immutable policy の clone で ProofEvidenceSet を構築し、この束縛 candidate 一件を select_winner に渡す。順序・拒否診断・受理 evidence identity/polarity・未公開 reuse metadata は既存 selection が所有する。binding/producer/check/scheduling error は selection なしの Err。実拒否は既存 non-trusted rejected selection とし、別 leaf の受理や Open/Builtin fallback に置換しない。
試験は original/renamed の単一・三登録 source と両 encoding を、実 producer から独立構成した既存 selector 結果へ比較する。false definition の依存 leaf は拒否し、独立 leaf identity を保つ。fresh-source/policy/target/format の失敗で trusted selection を返さず、FormulaSubstitution/Builtin permission を区別する。checking 試験と元 Builtin 挙動は保つ。
返すものはこの呼出しの既存 ProofSelection のみで、selected witness・committed publication を作らない。activation・artifact projection・trusted cache reuse・現 source validity の移転可能な権限ではない。汎用 by/built-in 受理・finder/backend 実行・対応済み witness/obligation metadata・Artifact publication・全 Steps 6–8 は保留する。

## Source discharge checking

`check_source_registration_discharge_candidate(&SurfaceResolvedArena, &TypedArena, &SymbolEnv, BuildSnapshotId, VcId, &VerifierPolicy) -> Result<(KernelCheckResult, PolicyDecision), String>` は元 source graph と実際の既定 Discharge を新たに構成し、VC の関連付けから選択された元 direct-premise handoff を得る。通常のローカル証拠を encode し、既存 BuiltinKernelEvidence の scheduling 許可を要求して元 target/profile で parse・通常の proof-obligation 検査を行う。実結果を BuiltinDischarge origin と供給 policy で分類し、caller 証拠・結果・receipt は受け取らない。
context 組立と通常 Kernel 呼出しだけを既存 ATP 検査・元 built-in 活性化と private に共有する。既存 parser/schema/instance 選択・error・scheduling・origin・policy・活性化の動作を保つ。未対応・foreign 入力や許可されない format は拒否し、偽定義の direct leaf も不変の premise/goal を証明する。汎用受理・信頼された witness metadata・公開は保留し、caller 所有の source/snapshot freshness は reuse receipt にならない。

## Source discharge metadata

`check_source_registration_discharge_metadata` は既存 discharge facade と同じ六つの source/snapshot/target/policy 入力を取り、既存 Result 内で `(KernelCheckResult, PolicyDecision, Option<KernelAcceptanceMetadata>)` を返す。fresh 再構成・元 direct-premise association・通常 encoding/scheduling/parsing/checking・BuiltinDischarge 評価を一つの pipeline で行い、両 facade は [payload producer](#source-discharge-witness-projection) を使い、不要な tuple 値を捨てて公開 signature と既存結果/error を保つ。
実際の DischargedBuiltin、untainted Accepted ProofObligation、既存 accepted identity、AssertFalseForRefutation の場合だけ plain metadata を作る。同じ呼出しの元 handoff/check/current policy を結び、不足・未対応の関連付けは fallback hash や分類昇格なしの None。caller evidence/result/receipt は受け取らない。
全 hash ref は Interface。[VC component profile](../../mizar-vc/ja/kernel_evidence_handoff.md#local-evidence-component-hashes) を profile/formula/substitution/provenance digest に複写し、family は `mizar-vc/local-evidence-component/{profile,formula,substitutions,provenance}`、version は 1.0。target binding は `handoff.canonical_hash()` を `mizar-vc/kernel-evidence-handoff-canonical` 1.0 で複写し、別 context identity と goal-polarity 検査は既存通常 checker が所有する。
policy は `policy_fingerprint().hash()` と family `mizar-proof/verifier-policy`、version `(policy.schema_version(),0)`。accepted result は BuiltinDischarge KernelPolicyInput の `accepted_evidence_hash()` と `mizar-proof/kernel-accepted-evidence` 1.0。checker/evidence schema version は `(policy.checker_schema_version(),0)` と `(envelope.schema_version(),0)`。checker 値は設定された policy requirement であり、checker 実装や実行中 toolchain version の認証ではない。対応 direct handoff は実 empty substitutions・import 不在なので formula_context_hash は None。
raw data association であり trusted witness token・composite obligation/current-source receipt・committed reachability ではない。Artifact の既存 Builtin/version guard、source freshness 所有、staging/publication/reuse、全 Proof23/Artifact25 は不変または保留する。

## Source discharge selection

`select_source_registration_discharge_candidate(&SurfaceResolvedArena, &TypedArena, &SymbolEnv, BuildSnapshotId, VcId, &VerifierPolicy) -> Result<ProofSelection, String>` は同じ immutable 入力で check_source_registration_discharge_metadata を一度呼ぶ。既存 checker/metadata/ATP-selection/活性化の body/error を保ち、caller の result/evidence/metadata を受けず、source/Core/VC/discharge/handoff/check pipeline を再実行しない。
fresh Kernel result の元 target sort_bytes を obligation identity とし、CandidateSourceId は `source-registration-discharge-v1:` と、その bytes の lowercase hex（既存 algorithm/length framing を含む）を連結する。これは安定 candidate 名で、新たな VC/encoded-input hash ではない。ATP translator 用に再分類せず、元 Open target binding を保つ。
実 DischargedBuiltin decision と返却 metadata がある場合だけ、その result と BuiltinDischarge origin で from_trusted_kernel_input を使い、metadata/trusted evidence 不在は error とする。実 metadata の provenance digest を保つ。他の実 decision は untrusted candidate とし evidence/provenance を作らず、上流 error は selection を返さない。実 target、None encoded input、供給 policy を既存 selector と [input-availability 規則](selection.md#encoded-input-availability) に渡す。
既存 ProofSelection だけを返し、実 class/accepted evidence identity/polarity を保つ。deterministic_discharge_hash や witness metadata を作らず、汎用 Core fingerprint/discharge hash は取得不能、dependency slice は incomplete、built-in 公開は unsupported のまま、trusted cache reuse は成立しない。現 source/snapshot freshness は caller 所有で、活性化・lease・status projection・witness staging・artifact commit は加えない。
試験は original/renamed direct type leaf、単一/複数登録、snapshot、既存 policy/schema/profile variant を使い、実 fresh checker/metadata から独立構成した selection と比較する。未対応・foreign・mixed-input failure、無関係の偽定義があっても有効な direct leaf、実 false-correctness candidate 生成/Kernel 拒否を保つ。現在の source producer が error を返す箇所に rejected built-in check を捏造しない。既存 ATP selection/proof/metadata 試験の意図を保つ。

## Source discharge witness projection

`project_source_registration_discharge_witness(&SurfaceResolvedArena, &TypedArena, &SymbolEnv, BuildSnapshotId, VcId, &VerifierPolicy) -> Result<(ProofSelection, ProofWitnessRef, Vec<u8>), String>` は未公開の direct-premise built-in projection だけを新たに生成する。
同一呼出しで parse・検査した Formula v1 の正確な bytes を保持する。private payload checker は既存 source/Core/VC/discharge/handoff/check/metadata の body/error を保ち、compatibility check、raw metadata facade、この projection の３箇所が呼ぶ。既存 selection は raw metadata を一度だけ呼び、ATP 検査と活性化は変えない。
実 DischargedBuiltin、同一呼出し metadata、既存 trusted Kernel input を要求する。source-selection facade と同じ元 target の candidate 名・provenance・None encoding selection を、producer 再実行や caller evidence なしで構成する。他の結果は既存 invalid_source_candidate family とし、上流 error はそのまま返す。

２つの新 producer-owned reference は Artifact の CanonicalHashDomain、excluded path なし、version 1.0 を使う。payload reference は class Artifact、family `mizar-proof/source-registration-discharge-witness-payload` とし、正確な検査済み bytes の lowercase hex を含む CanonicalJson string を hash する。既存 witness-store payload hash とは別の content profile である。
obligation reference は class Interface、family `mizar-proof/source-registration-discharge-obligation` とし、次の正確な key を持つ CanonicalJson object を hash する:
`original_target_bytes`（元 Kernel target sort_bytes の lowercase hex）、`kernel_profile_fingerprint`、`verifier_policy_fingerprint`、`checker_schema_version`、`evidence_schema_version`、`target_binding_hash`、`formula_evidence_hash`、`substitution_evidence_hash`、`provenance_hash`、`formula_context_hash`、`accepted_result_hash`、`accepted_goal_polarity`、`witness_artifact_hash`。
metadata hash は完全な artifact-hash string、２つの version は canonical string、不在の imported context は null、実 accepted polarity は stable string、真正な payload hash は完全な artifact-hash string として複写する。汎用 VC/discharge/dependency fingerprint で代用しない。

obligation_id は選択された source-target 名、witness_path は `proof-witnesses/source-registration-discharge-v1/<lowercase-payload-digest>.bin` とする。lowercase hex は新 producer の実３箇所（target、payload、path）だけで共有し、既存 selector の書式/error は変えない。この metadata/hash から [明示的 built-in reference](../../mizar-artifact/ja/proof_witness.md#built-in-reference-profile) を構成し、Artifact value writer で構造検証して selection・bytes とともに返す。
この tuple は TrustedKernelWitnessMetadata、ProofWitnessDraft、file I/O、committed-publication proof、accepted witness status projection、selected witness/discharge reuse hash を生成しない。既存 unsupported publication、incomplete dependency slice、保守的な汎用 identity と caller 所有の現 source/snapshot freshness を保つ。上位 artifact/manifest 統合と完全な Proof23/Artifact25 は保留する。
試験は新鮮な単一/複数 source leaf、改名 source、snapshot/policy、再構成した正確な通常 payload/raw metadata、canonical round-trip、hash/path 入力と上流拒否を使う。実 false-correctness Kernel 拒否と既存 check/metadata/selection の意図を保ち、Artifact の shape mutation を source 受理/公開の証拠に数えない。

## Source existential-registration proof

`prove_source_existential_registration(&SurfaceResolvedArena, &TypedArena, &SymbolEnv,
BuildSnapshotId, &VerifierPolicy) -> Result<RegistrationDatabase, String>` は
[checker](../../mizar-checker/ja/registration_resolution.md#source-existential-registration-proof)・Core・VC・両
handoff を新たに実行する。[VC local transport](../../mizar-vc/ja/kernel_evidence_handoff.md#local-binary-transport) が実 field を frame 化し、Proof は全 term・置換・provenance・context identity・AssertFalseForRefutation
を含む六 section Formula v1 を構造検査し、この段階では受理を検査しない。
ATP の [未信頼 name writer](../../mizar-atp/ja/backend.md#closed-choice-reflexivity-names) が、その parsed 入力から Formula v2 の名前を供給する。
既存 parser と check_kernel_evidence を両 leaf
に自ら適用し、caller Accepted・result summary・cached handoff・diagnostic/hash text を代用しない。

検査前に既存 UncheckedBuiltinDischarge の実行許可を要求し、結果に KernelEvidenceOrigin::BuiltinDischarge を用いる。両 fresh
Accepted・untainted・ProofObligation 結果が既存 KernelPolicyInput/ProofPolicyEvaluator の真正な
DischargedBuiltin を満たす必要がある。同じ認証済み SymbolEnv と checker validation から関数内で既存
ActivationInput/RegistrationDatabase を構築し、全 pattern・正当性 owner・親/nonempty/leaf
関係・source/module/snapshot・handoff hash・現 policy を結ぶ。失敗で活性化や pending 入力変更は起こらず、kernel receipt や trust
policy は追加しない。

返却 database は当該呼出しの module-local 出力であり transferable proof receipt ではない。後続 consumer は同じ認証済み source
orchestration 内で producer を使い、任意の public database を証明権威にしない。item-order visibility と非遡及性を保ち、限定 C3 widening runner はその場で fresh local 出力を消費できる。
限定 C13 consumer は [harness orchestration](../../mizar-test/ja/harness.md#attributed-ambiguity-admission) が所有し、限定 C5 narrower-loci consumer も同一 immutable 入力で単一登録出力を直ちに消費できる。広域 Task274・artifact publication・cluster/reduction 効果は保留する。

## Three-registration source-proof extension

実装した追加 source profile は既存 prove_source_existential_registration signature のまま C13 の記述された全登録3件について checker/Core/VC 全 graph を新たに構成する。
現在の policy による通常 Formula-v2 leaf 6件の検査が全て成功してから、同一環境の full pattern 3件だけを含む database を構成する。失敗時に部分 activation や先行 pending 入力の変更を返さない。
各 association は各自の registration・parent・独立 nonempty seed・両 leaf・実定義依存・handoff hash・source/module/snapshot を結ぶ。callback・呼出側 receipt・別登録の受理で代替しない。
単一登録の挙動と database の非譲渡性を維持する。この prerequisite は C13 overload/属性付き consumer、C3 widening、汎用 Task274/import、cluster closure、publication を実行しない。
