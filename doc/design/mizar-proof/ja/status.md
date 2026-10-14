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

## Source candidate checking

`check_source_registration_candidate(&SurfaceResolvedArena, &TypedArena, &SymbolEnv, BuildSnapshotId, VcId, (LogicProfile, ConcreteFormat), &VerifierPolicy) -> Result<(KernelCheckResult, PolicyDecision), String>` は leaf 1件の fresh 検査を所有し、登録活性化や公開を行わない。encoding tuple は既存の明示 profile/format をまとめるだけで型を追加しない。source checker と Core lowering を再実行し、既存 source/VC schema で元 graph を生成して実 default Discharge を適用する。選択 NeedsAtp leaf に [実 source candidate factory](../../mizar-atp/ja/backend.md#source-registration-candidate-inputs) を使う。未対応・foreign input、不正・Discharged target、実行許可のない policy では checked result や fallback なしで error とする。
candidate の payload kind、完全な target、input/provenance hash、label、symbol mapping を局所的に生成した encoded container と比較する。この比較から UncheckedFormulaSubstitution の encoded_problem_matches を得て、既存 FormulaSubstitution 検査実行許可を要求する。caller の一致 boolean、encoded/candidate pair、context、Accepted result、hash は入力しない。backend observation は作らない。
fresh Core/default-discharge graph 全体から同じ認証済み source handoff を独立再構成する。実 target/profile を期待値とし、schema 2 を明示許可して返却 v2 byte を parse し、この別 handoff の完全な identity/producer entry から通常の [Kernel context](../../mizar-kernel/ja/checker.md) を構築する。import・未対応 context class は空 context の捏造で補わず拒否する。既存 default policy/limit と ProofObligation で通常 check_kernel_evidence を呼び、legacy audit・過去 result を使わない。
実 result を KernelPolicyInput::from_kernel_result と明示 AtpFormulaSubstitution origin で包み、既存 evaluator に渡す。実 PolicyDecision と KernelCheckResult を返し、偽の goal の実拒否 detail も保つ。通常 Accepted/untainted proof check は既存 policy により KernelVerified となり、拒否を手動昇格したり Open/Builtin/external evidence へ置換しない。policy criteria/fingerprint を保つ。
返却値はこの呼出しの検査出力であって selection・source 活性化・witness 公開・artifact status・cache reuse の移転可能な権限ではない。caller が現 source/snapshot freshness を所有する。元 Open/Builtin facade の公開動作を保つ。汎用 by/built-in integration・探索 budget・import context・witness/artifact/cache 公開と全 milestone は保留する。

## Source candidate selection

`select_source_registration_candidate(&SurfaceResolvedArena, &TypedArena, &SymbolEnv, BuildSnapshotId, VcId, (LogicProfile, ConcreteFormat), &VerifierPolicy) -> Result<ProofSelection, String>` は同一 source 内の限定 orchestration を所有する。同じ immutable input と供給 policy で check_source_registration_candidate を新たに呼び、caller check tuple・Accepted flag・evidence pair を受け取らない。元 checking/Builtin facade の公開動作を保つ。
同じ input から既存 source checker/Core/original VC/default Discharge producer を独立に再実行し、実 source candidate factory の encoded container と candidate を取得する。既存 schema/API を使い、hash/codec/parser/check 実装を追加しない。fresh Kernel target の algorithm/digest と実 encoded target fingerprint の一致を selection 前に要求する。現 source/snapshot freshness は caller 所有で、局所対応から runtime lease や source reuse receipt は得られない。
実 candidate_id を CandidateSourceId::new に渡す。実 current decision が KernelVerified の場合だけ、この fresh result と AtpFormulaSubstitution origin による ProofEvidenceCandidate::from_trusted_kernel_input を使い、trusted evidence を得られなければ error とする。他は実 decision を ProofEvidenceCandidate::new で保持し、trusted marker・架空 evidence hash を加えない。両者に実 candidate provenance hash を運び、backend/profile/format priority・witness hash を捏造しない。
fresh target の既存 TargetVcFingerprint::sort_bytes identity、実 encoded input_hash、供給 immutable policy の clone で ProofEvidenceSet を構築し、この束縛 candidate 一件を select_winner に渡す。順序・拒否診断・受理 evidence identity/polarity・未公開 reuse metadata は既存 selection が所有する。binding/producer/check/scheduling error は selection なしの Err。実拒否は既存 non-trusted rejected selection とし、別 leaf の受理や Open/Builtin fallback に置換しない。
試験は original/renamed の単一・三登録 source と両 encoding を、実 producer から独立構成した既存 selector 結果へ比較する。false definition の依存 leaf は拒否し、独立 leaf identity を保つ。fresh-source/policy/target/format の失敗で trusted selection を返さず、FormulaSubstitution/Builtin permission を区別する。checking 試験と元 Builtin 挙動は保つ。
返すものはこの呼出しの既存 ProofSelection のみで、selected witness・committed publication を作らない。activation・artifact projection・trusted cache reuse・現 source validity の移転可能な権限ではない。汎用 by/built-in 受理・finder/backend 実行・対応済み witness/obligation metadata・Artifact publication・全 Steps 6–8 は保留する。

## Source discharge checking

`check_source_registration_discharge_candidate(&SurfaceResolvedArena, &TypedArena, &SymbolEnv, BuildSnapshotId, VcId, &VerifierPolicy) -> Result<(KernelCheckResult, PolicyDecision), String>` は元 source graph と実際の既定 Discharge を新たに構成し、VC の関連付けから選択された元 direct-premise handoff を得る。通常のローカル証拠を encode し、既存 BuiltinKernelEvidence の scheduling 許可を要求して元 target/profile で parse・通常の proof-obligation 検査を行う。実結果を BuiltinDischarge origin と供給 policy で分類し、caller 証拠・結果・receipt は受け取らない。
context 組立と通常 Kernel 呼出しだけを既存 ATP 検査・元 built-in 活性化と private に共有する。既存 parser/schema/instance 選択・error・scheduling・origin・policy・活性化の動作を保つ。未対応・foreign 入力や許可されない format は拒否し、偽定義の direct leaf も不変の premise/goal を証明する。汎用受理・selection・信頼された witness metadata・公開は保留し、caller 所有の source/snapshot freshness は reuse receipt にならない。

## Source discharge metadata

`check_source_registration_discharge_metadata` は既存 discharge facade と同じ六つの source/snapshot/target/policy 入力を取り、既存 Result 内で `(KernelCheckResult, PolicyDecision, Option<KernelAcceptanceMetadata>)` を返す。fresh 再構成・元 direct-premise association・通常 encoding/scheduling/parsing/checking・BuiltinDischarge 評価を一つの pipeline で行い、旧 facade は委譲して metadata だけを捨て、signature と既存結果/error を保つ。
実際の DischargedBuiltin、untainted Accepted ProofObligation、既存 accepted identity、AssertFalseForRefutation の場合だけ plain metadata を作る。同じ呼出しの元 handoff/check/current policy を結び、不足・未対応の関連付けは fallback hash や分類昇格なしの None。caller evidence/result/receipt は受け取らない。
全 hash ref は Interface。[VC component profile](../../mizar-vc/ja/kernel_evidence_handoff.md#local-evidence-component-hashes) を profile/formula/substitution/provenance digest に複写し、family は `mizar-vc/local-evidence-component/{profile,formula,substitutions,provenance}`、version は 1.0。target binding は `handoff.canonical_hash()` を `mizar-vc/kernel-evidence-handoff-canonical` 1.0 で複写し、別 context identity と goal-polarity 検査は既存通常 checker が所有する。
policy は `policy_fingerprint().hash()` と family `mizar-proof/verifier-policy`、version `(policy.schema_version(),0)`。accepted result は BuiltinDischarge KernelPolicyInput の `accepted_evidence_hash()` と `mizar-proof/kernel-accepted-evidence` 1.0。checker/evidence schema version は `(policy.checker_schema_version(),0)` と `(envelope.schema_version(),0)`。checker 値は設定された policy requirement であり、checker 実装や実行中 toolchain version の認証ではない。対応 direct handoff は実 empty substitutions・import 不在なので formula_context_hash は None。
raw data association であり trusted witness token・composite obligation/current-source receipt・committed reachability ではない。Artifact の既存 Builtin/version guard、source freshness 所有、selection/staging/publication/reuse、全 Proof23/Artifact25 は不変または保留する。

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
