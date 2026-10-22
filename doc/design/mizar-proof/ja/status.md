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

## Source theorem canonical inputs

`source_theorem_canonical_inputs(check: &mizar_checker::type_checker::SourceTheoremCheck<'_>, snapshot: BuildSnapshotId, vc_id: VcId) -> Result<(CanonicalVcFingerprint, Hash, LocalContextFingerprint, DependencySlice), String>` は既存 status.rs の単一 inline public 関数で、VC・goal・local context・元の owned slice 全体を既存型だけで返す。caller graph・opaque map/bytes・identity・policy・evidence・accepted result は受け取らない。
最初に不変の lower_source_theorem_skeletons(check, None) を呼び、実 error を保ち、この fresh local Core を保持する。
全ての実 Core obligation seed から、各 context の copied id と続く optional goal を indexing・deduplication・fallback なしで集める。同一 borrowed check・Core・roots で [Core56 source_theorem_formula_payloads](../../mizar-core/ja/elaborator.md#source-theorem-formula-payloads) を呼び、Lemma/computation 拒否を含む完全 profile 検証と error を保ち、validator を複写しない。
同じ Core、caller snapshot、generation schema source-theorem-local-v1、VC schema vc-v1 で [generate_source_theorem_vcs](../../mizar-vc/ja/generator.md#source-theorem-vc-generation) を呼び、周囲の全 operation/validation/順序/error を保つ。
元 Open graph の [VC70 source_theorem_canonical_components](../../mizar-vc/ja/vc_ir.md#source-theorem-canonical-components)(vc_id, &core, &formulas) をそのままの local map で呼ぶ。全 VC69 replay/VC70 error と既存 content grammar/hash を不変に保ち、handoff hash の再ラベル・値の再 encode をしない。
その後だけ、同じ元 graph 全体に discharge_output None で既存 [try_compute_dependency_slices](../../mizar-vc/ja/dependency_slice.md) を適用し、typed error は Display で保つ。実 slices から vc_id を選び、DependencySlice 全体を clone するか正確に internal theorem.proof.invalid_source_canonical_inputs を返す。全処理成功後だけ four-tuple を atomic に返す。
実 Open/TheoremProofStep・entries・unknowns・IncompleteUncacheable/requires_cache_miss・元 collector fingerprint を保つ。汎用 Core fingerprint は None、元 anchor は incomplete/不変のまま。canonical content で保守的 coverage を代用せず、取得不能 Core/owner/syntax content を partial fingerprint へ要求しない。
全入力は不変。VcId はこの呼出しの graph を選び、同じ数値の foreign handle は区別できない。同一呼出しの実 producer 対応であり execution token・currentness 権限ではない。source/snapshot currentness は caller 所有で、skeleton checking は proof acceptance ではない。
default discharge・Kernel/ATP/policy 実行・Complete marker・cache lookup・source 活性化・trusted metadata・witness/publication は作らない。source owner/declaration syntax、完全 anchor/ProofObligationIdentity、合成 phase dependencies、fresh normalized acceptance origin/classification/evidence/witness、通常 ATP projection、Driver/runtime、Steps6–8 完了は別に残す。
既存 term_proof_tests に source_theorem_canonical_inputs_* family を正確に二つ追加し、with_step5c14_theorem_check を helper/extraction 変更なしで使う。family 一は元 T・別 bound/local 名・trivia・実 owner U・snapshots777/778 を facade に渡す。三 component を Rust 前に doc/ 外で準備/review した独立固定 schema literal hash と実 lower VC70 出力へ比較する。同 schema の元汎用 graph を独立再生成し None collector の全 getter/entry/unknown/status/fingerprint と比較し、Open TheoremProofStep・非空 unknown/cache miss・汎用 fingerprint None・元 incomplete anchor 不変も要求する。反復同値と borrowed check 不変性を保ち、中立編集/snapshot は component を保つが partial slice fingerprint の変化/安定は仮定しない。
family 二は valid source の absent VcId(usize::MAX) を既存 genuine generate_core_vcs/VC69 missing-target error と直接比較する。実 proofless Open は valid/absent 選択より前の lowerer status_unsupported を保つ。実 Lemma と computation は lower 成功後、選択前の実 Core56 profile error を保ち、computation の空 context で安全な root 収集を試す。全 error oracle を実 lowerer/producer から直接得て各 API に到達する。Core56/VC69/VC70 matrix を再利用し、旧 source/test/helper/final-brace bytes・fixture/expectation を保つ。成功 lower 捏造・foreign-byte 認証・合成 collector failure・Accepted/Hit/Complete control を加えない。

## Source registration candidate anchors

`source_registration_candidate_anchor(&SurfaceResolvedArena, &TypedArena, &SymbolEnv, BuildSnapshotId, VcId) -> Result<(ObligationAnchor, CanonicalVcFingerprint, Hash, Hash), String>` は現在の選択 leaf の候補 anchor・VC 全体 fingerprint・goal・可視 context hash をこの順で既存型により返す。caller graph／opaque payload／identity／accepted flag は受け取らず、source／snapshot の現在性は caller が所有する。
既存 checker、Core lowering、source VC generator を fresh 実行し、Proof の既存 generation schema source-existential-registration-v1 と VC schema vc-v1 を使う。[source canonical components](#source-canonical-components) と同様に、実 arena owner、全 graph の Ref/Not/And/Or からの Core root 集合、Core formula、全 canonical VC map を得る。上流 error を維持し、欠損／未対応走査・対応には内部 `registration.proof.invalid_source_anchor` を使う。既存 facade を変更せず、2か所用 shared pipeline helper を作らず inline で結合する。
次に同じ source/env で [Resolver syntax producer](../../mizar-resolve/ja/symbols.md#source-registration-syntax-payloads) を呼ぶ。各実 Core Registration item の SymbolId を未変更の局所 owner／syntax map に結び、その現在の CoreItemId に値を対応付け、全 syntax の完全一致を要求する。再解析・origin/label の捏造・caller opaque map・下流再符号化は行わない。同じ Core／formula／syntax／owner で [VC candidate-anchor producer](../../mizar-vc/ja/vc_ir.md#source-registration-candidate-anchor-hashes) を全実行後、選択 VcId が両全 map にあることを要求し、不在なら内部 error とする。続けて実 goal/context を得る。
既存 Proof ObligationAnchor の文字列は正確に `source-registration-candidate-anchor-v1:` と全32 hash bytes の小文字 hex とする。VC 所有 profile の可逆表示であり、Proof の hash／encoder や identity 代用を作らない。既存の非空 constructor 成功後にだけ返す。入力・元 anchor は不変とし、default discharge・Kernel・policy 評価・trusted metadata・Complete marker・依存／status mutation は行わない。
同一呼出しで実 producer を結ぶ best-effort cross-edit 候補であり、一致だけで reuse は許可しない。architecture22 の VC 全体・local context・完全依存・互換 policy・検証 evidence の要求を保つ。完全な構造化 anchor／Proof identity・source 活性化・受理 metadata・witness・公開・runtime 結合は保留する。未証明・偽の source でも候補を持ち得る。Proof23/Artifact25/17 と Steps6–8 は未完了のままとする。
共有 integration test の所有者は [VC profile](../../mizar-vc/ja/vc_ir.md#source-registration-candidate-anchor-hashes) とする。

## Source dependency slices

`source_registration_dependency_slice(&SurfaceResolvedArena, &TypedArena, &SymbolEnv, BuildSnapshotId, VcId) -> Result<DependencySlice, String>` は既存の所有された [VC-local slice](../../mizar-vc/ja/dependency_slice.md) を返し、entries、unknowns、completeness、status、fingerprint を保持する。呼び出し側の Core/VC graph、discharge output、payload、identity、policy、accepted flag は入力にしない。
変更しない source proof checker、Core lowering、source VC generator を順に新規実行し、既存 Proof generation schema source-existential-registration-v1 と VC schema vc-v1 を使用する。上流のエラーをそのまま伝播し、元の Open graph 全体に既存 try_compute_dependency_slices を discharge_output None で適用する。下流エラーは Display の文字列で伝播する。その後だけ VcId を選択して既存 slice を clone し、不在なら内部 registration.proof.invalid_source_dependency_slice を返す。不変入力と既存 facade の本体を保ち、小さな処理をインラインにし、helper、type、field、adapter、dependency、hash profile を追加しない。
VcId は今回の graph に対する検索 handle であり、来歴や現在性の証拠ではない。別 graph の数値が同じ handle は区別できない。source/snapshot の現在性は呼び出し側が担う。default discharge、Kernel check、policy 評価、status/anchor の変更、trusted metadata、witness、publication を行わない。
汎用 Core/Anchor unknown marker と実際の IncompleteUncacheable/cache-miss 判定を保持し、欠落を依存なしと解釈しない。この raw VC-local fingerprint は architecture22 の phase footprint 合成でも、別の source canonical-content profile でもない。完全な六項目 Proof identity、完全依存、受理/reuse、Proof23/Artifact25/17/Steps6–8 は別の課題として維持する。
既存 source setup を再利用する追加の実ソース三系列で、Proof schema による元の下流 producer と single/aggregate leaf の full-slice 一致、neutral/participating source・snapshot 変更でも実際の unknown/cache miss を保持すること、不在 handle・実際の混合/不正/未対応入力・選択前の上流エラー・入力不変性を確認する。不完全 hash が取得不能な内容の変更ごとに変わるとは仮定しない。既存 fixture/expectation/test を保ち、下流 slice matrix を複製せず、架空の下流エラーや受理済み control を作らない。

## Source original obligation identity

`source_registration_obligation_identity(&SurfaceResolvedArena, &TypedArena, &SymbolEnv, BuildSnapshotId, VcId) -> Result<(ProofObligationIdentity, DependencySlice), String>` は raw な限定データと元 Open slice 全体を返す。同じ不変引数で [candidate anchor](#source-registration-candidate-anchors)、[original slice](#source-dependency-slices) の順に既存 producer を呼び、全成功後だけ構築する。上流 error はそのまま伝播し、caller graph・identity・policy・evidence・受理結果は入力しない。
obligation_id は元 anchor 文字列そのものとし、anchor、正規 VC 全体 hash、可視 context hash、実 VC-local slice fingerprint を既存 identity field に写す。obligation fingerprint は Artifact CanonicalHashDomain の Interface family `mizar-proof/source-registration-original-obligation`、version 1.0、除外 path なしで計算する。CanonicalJson object の key は正確に `obligation_anchor`（元文字列）、`vc_fingerprint`、`canonical_goal_hash`、`local_context_fingerprint`、`vc_local_dependency_slice_fingerprint`（hash は全32 byte の小文字 hex）とする。既存の非空 identity constructor と hex helper を使い、constructor error は Display で伝播する。他 profile・既存 facade 本体は変更しない。
6 field 値とともに元の所有 slice 全体を未変更で返し、Open status、entry、unknown、IncompleteUncacheable、必須 cache miss を保持する。この field は VC-local であり architecture22 の合成依存ではない。同値の数値 handle でも現在の入力 graph を選び、この readout は同じ不変 source 引数に結び付く。source/snapshot freshness は caller が所有する。default discharge・Kernel/policy 検査・受理状態・trusted token・status/witness 公開・cache lookup は行わない。
この original-source profile は candidate/payload 依存 witness profile と独立で、汎用合成 identity や reuse/受理 receipt ではない。trusted witness/reuse には完全 phase footprint と同じ検査内の元 source identity・正確な検査済み payload の consumer enforcement が必要で、Proof23/Artifact25/17 と Steps6–8 は未完了とする。
実 source の追加3系列で既存 producer と独立した literal canonical object/domain による全 field/full slice 一致、中立・関与編集/snapshot/現在 handle、選択前上流 error と入力不変性を確認する。不完全 slice は取得不能内容の全変更で hash が変わるとは要求しない。未証明・偽 source も raw identity を持ち得る。既存 test/fixture/expectation は保つ。

## Source candidate checking

`check_source_registration_candidate(&SurfaceResolvedArena, &TypedArena, &SymbolEnv, BuildSnapshotId, VcId, (LogicProfile, ConcreteFormat), &VerifierPolicy) -> Result<(KernelCheckResult, PolicyDecision), String>` は leaf 1件の fresh 検査を所有し、登録活性化や公開を行わない。encoding tuple は既存の明示 profile/format をまとめるだけで型を追加しない。source checker と Core lowering を再実行し、既存 source/VC schema で元 graph を生成して実 default Discharge を適用する。選択 NeedsAtp leaf に [実 source candidate factory](../../mizar-atp/ja/backend.md#source-registration-candidate-inputs) を使う。未対応・foreign input、不正・Discharged target、実行許可のない policy では checked result や fallback なしで error とする。
candidate の payload kind、完全な target、input/provenance hash、label、symbol mapping を局所的に生成した encoded container と比較する。この比較から UncheckedFormulaSubstitution の encoded_problem_matches を得て、既存 FormulaSubstitution 検査実行許可を要求する。caller の一致 boolean、encoded/candidate pair、context、Accepted result、hash は入力しない。backend observation は作らない。
fresh Core/default-discharge graph 全体から同じ認証済み source handoff を独立再構成する。実 target/profile を期待値とし、schema 2 を明示許可して返却 v2 byte を parse し、この別 handoff の完全な identity/producer entry から通常の [Kernel context](../../mizar-kernel/ja/checker.md) を構築する。import・未対応 context class は空 context の捏造で補わず拒否する。既存 default policy/limit と ProofObligation で通常 check_kernel_evidence を呼び、legacy audit・過去 result を使わない。
実 result を KernelPolicyInput::from_kernel_result と明示 AtpFormulaSubstitution origin で包み、既存 evaluator に渡す。実 PolicyDecision と KernelCheckResult を返し、偽の goal の実拒否 detail も保つ。通常 Accepted/untainted proof check は既存 policy により KernelVerified となり、拒否を手動昇格したり Open/Builtin/external evidence へ置換しない。policy criteria/fingerprint を保つ。
返却値はこの呼出しの検査出力であって selection・source 活性化・witness 公開・artifact status・cache reuse の移転可能な権限ではない。caller が現 source/snapshot freshness を所有する。元 Open/Builtin facade の公開動作を保つ。汎用 by/built-in integration・探索 budget・import context・witness/artifact/cache 公開と全 milestone は保留する。

## Source screened candidate attempt

`try_check_source_registration_candidate(&SurfaceResolvedArena, &TypedArena, &SymbolEnv, BuildSnapshotId, VcId, (LogicProfile, ConcreteFormat), &VerifierPolicy) -> Result<Option<(KernelCheckResult, PolicyDecision)>, String>` は限定した一回の source attempt を明示選択する。通常 checker と同じ実 source 入力・policy を受け、caller Core/VC/evidence pair/context・Accepted flag・上限 override は受けない。
実 source Checker/Core/元 VC/default Discharge の prefix を先頭に保つ。既存 pair 生成位置で、その実 Core/VcSet/選択 VcId、明示 profile/format、policy.instance_search_limits() を ATP の [screen_source_registration_candidate](../../mizar-atp/ja/backend.md#source-candidate-sat-screening) に渡す。下位 producer/profile/全 graph/target error を任意上限の admission より先に保ち、None/zero の読取りだけで実 screen を省略しない。ATP 所有の admission count は先行する source 再構成/extraction 作業全体の budget ではない。
実 screen None（未設定、zero/実 count 超過、SAT-not-Unsat）は post-pair Kernel scheduling permission より前に Ok(None) を返す。有効な下位入力で admission 不在かつ Kernel permission 拒否なら None、実 malformed/foreign/unsupported 入力なら先に error とする。不在時に旧 fallback・通常 Kernel check・acceptance metadata・selection は行わない。screen の純粋 SAT query は既に実行されている場合がある。
実 Some(encoded, candidate) の正確な pair を、既存 payload-kind/全 container 対応、scheduling permission、独立再構成した元 handoff、実 v2 parse、通常 check_source_kernel_handoff、ATP-origin KernelPolicyInput/evaluator tail の順に渡す。default ProofObligation policy/Kernel limit と元 permission error を保つ。pair の破棄・再取得、第二 factory/check pipeline、candidate 由来 context、受理 data の捏造は禁止する。
共有 tail は check/metadata body 全体を保ち、元五値を Some で包む。None は実 screen からだけ返る。この facade は既存 [private payload checker](#source-candidate-witness-projection) に true を渡し、同じ実行の実 result/decision だけへ写し、他の内部出力は互換 checker と同様に捨てる。実 Kernel 拒否は Some（実拒否 result、実 decision）のままとし、昇格・置換しない。
None はこの明示 attempt の不在であり、検査済み拒否や全探索 exhaustion ではない。有無それ自体は selection・witness/publication・reuse・source 活性化・runtime 使用の移転可能な権限を与えない。受理・分類は実際の通常 Kernel result と既存 Proof policy だけに由来する。source/snapshot freshness は caller 所有、policy identity と旧公開動作は不変。全 ATP30/31・Proof23・Steps6–8、探索/列挙/累積制限適用、backend 実行、完全 footprint と trusted publication は保留する。
試験設計は既存 helper と source_candidate_check_bytes による実 source 三系列の追記だけとし、旧 runner/tests.rs prefix 全体、helper/oracle/fixture、source_registration_instance_search_limits_preserve_fresh_check_results を保持する。None/zero・実データ由来 exact/MAX/代表 underflow と有効な旧 check の対比、単一/aggregate/renamed/negative source・両 format の screen と fresh 通常 Kernel/policy 結果、偽定義依存の不在と独立 true sibling、upstream/foreign/wrong/Discharged/未対応 profile・scheduling/複合不在順序・source/policy 不変性を扱う。実 ground pool、parse 済み v2 substitution、equality name、一 query から count を導く。下位 coordinate matrix を再利用し SAT/Kernel matrix や架空 Accepted control を複製しない。
成功時の screen pair と通常 pair は等しいため、実行時の結果/byte 一致で示せるのは admission と fresh 結果の互換性まで。独立 code review で screen 返却 pair が破棄・再取得なしに共有 tail へ届くことを追跡する。observer/counter/injected writer や静的な実装写し試験は追加しない。

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
private check_source_registration_candidate_payload の引数は七つのまま、内部 encoding tuple だけを (LogicProfile, ConcreteFormat, bool) に拡張する。false は通常 factory、true は [screened attempt](#source-screened-candidate-attempt) を使う。返却型は Result<Option<(KernelCheckResult, PolicyDecision, Option<KernelAcceptanceMetadata>, BackendCandidateEvidence, VcKernelEvidenceHandoff)>, String> とし、通常生成は必ず Some。四 caller は互換 checker、raw metadata facade、この投影、screened attempt。旧三 caller は false を渡し、内部 Some を除去する。不在は既存 registration.proof.invalid_source_candidate とし、その後は従来の mapping/destructuring と不要出力の破棄を保つ。旧公開 signature・値・check/error 順序・policy None/zero 挙動・selector の独立再構成・全 Builtin body を保ち、純粋 hex renderer を改名せず再利用する。type/field/adapter/dependency/lower-stage 変更は加えない。
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
この tuple は TrustedKernelWitnessMetadata、ProofWitnessDraft、file I/O、committed-publication proof、accepted witness status projection、selected witness/discharge reuse hash を生成しない。既存 unsupported publication、incomplete dependency slice、保守的な汎用 identity と caller 所有の現 source/snapshot freshness を保つ。完全な source producer による artifact 組立、Proof witness staging／publication 統合と完全な Proof23/Artifact25 は保留する。
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

## Source theorem original obligation identity

`source_theorem_obligation_identity(source: &mizar_resolve::resolved_ast::SurfaceResolvedArena, check: &mizar_checker::type_checker::SourceTheoremCheck<'_>, symbols: &mizar_resolve::env::SymbolEnv, snapshot: mizar_session::BuildSnapshotId, vc_id: VcId) -> Result<(ProofObligationIdentity, mizar_vc::dependency_slice::DependencySlice), String>` を `crates/mizar-proof/src/status.rs` に一つの inline function として追加する。
最初に未変更 SourceVariableSemanticsChecker::check_theorem_skeletons(source, check.typed_ast(), check.scope(), symbols, check.labels(), check.resolved_labels(), None) を呼び、実 error をそのまま保つ。None は既存 ordinary profile で、algorithm-backed receipt 再構成や新 scope/label/現在性 seal ではない。
borrowed slice のまま check.owners() == fresh.owners() を要求し、不一致は内部 `theorem.proof.invalid_source_identity`。owner fact の clone/relabel や caller owner/hash/map を受け入れない。
次に未変更 [source_theorem_canonical_inputs](#source-theorem-canonical-inputs)(&fresh, snapshot, vc_id) を呼び、syntax/owner 作業より前の上流 error と正確な (vc, goal, context, 元 slice 全体) を保つ。実 lower-first Core/VC pipeline と元 Open graph 全体の None collector を再利用し、setup 複写・schema/status/unknown/completeness 変更・entry 削除・discharge 捏造をしない。
続いて同じ operands で未変更 [Resolver syntax producer](../../mizar-resolve/ja/symbols.md#source-theorem-syntax-payloads) source_theorem_syntax_payloads_from_arena(source, symbols) を呼び error を保つ。真正 fresh owner 一つとその実 owner.symbol をキーにする syntax row ちょうど一つを要求し、spelling/id fallback は使わない。role Theorem、status Unmodified、recovery Normal、検証済み public/private visibility を要求し、後段不一致は同じ内部 identity error として先行 ordinary profile を広げない。
実 owner から既存 CanonicalJson の STRING field 七つだけを作る。package_id = owner.symbol.module().package().as_str()、module_path = owner.symbol.module().path().as_str()、namespace = whole-env replay が確立した同じ module-path text、spelling = owner.spelling、role = role 検証後の "theorem"、status = status 検証後の "unmodified"、visibility = 実検証済み visibility。末尾 LF を含む既存 canonical_json_bytes を使い、別 owner 表現/encoding helper は作らない。
ID・FQN・contribution/site・range・provenance path は対応付けだけで owner content から除外する。実 owner JSON bytes、借用した未変更の同一 source syntax、未変更 canonical triple を [VC raw candidate 関数](../../mizar-vc/ja/vc_ir.md#source-theorem-candidate-anchor-hash) に渡す。hash grammar は VC が所有する。
厳密な `source-theorem-candidate-anchor-v1:` と candidate hash 全32 bytes の既存 source_discharge_hex から既存 ObligationAnchor を作る。可逆 presentation であって Proof 独自の anchor hash ではなく、constructor error は to_string のまま保つ。
既存 CanonicalJson の STRING key 五つだけを作る。obligation_anchor = candidate string、vc_fingerprint = vc.hash() の32-byte lowercase hex、canonical_goal_hash = goal の同 hex、local_context_fingerprint = context.hash() の同 hex、vc_local_dependency_slice_fingerprint = 元 slice.fingerprint().hash() の同 hex。
既存 CanonicalHashDomain(HashClass::Interface, "mizar-proof/source-theorem-original-obligation", SchemaVersion::new(1, 0)) で excluded path 無しに object を hash する。既存 ProofObligationIdentity::new(candidate.as_str().to_owned(), candidate, obligation_fingerprint, vc.hash(), context.hash(), original slice.fingerprint().hash()) を使い constructor error は to_string のまま保つ。obligation_id と anchor は意図的に同じ。identity と所有する元 slice 全体を不可分・変更無しで返す。
実 producer を結び付けるだけで acceptance/reuse/publication 権限は与えない。source/snapshot/filesystem 現在性は caller 責任で、foreign numeric id 一致は認証にならない。元 anchor、不完全 footprint/cache miss、acceptance origin/evidence/witness、ordinary ATP、Driver/runtime/cache、広域 Steps6–8 の保留を保つ。
test plumbing は既存 source_registration_owner_inputs を pub(super) にし、dotted module を親 directory のある階層 source path へ写し、単一 component の path と producer operand を保つ。既存 scope/type/label/check assembly だけを term_proof_tests の private with_source_theorem_check(ast, source, symbols, consume) に抽出する。元 wrapper の source 準備・parameter は完全に保つ。wrapper と新三系列が helper を使い、他の既存 bytes/test intent は保つ。
source_theorem_obligation_identity_* は exact_genuine_fields_and_slices、neutral_allocation_and_participating_edits、preserve_upstream_errors_and_inputs の三系列だけ。真正 arbitrary-source Frontend/parser/collector と同一 arena/env/check を使い、最初に VcId::new(0)、snapshot 777/778 の実旧 canonical inputs を得る。実装前の独立 normalized-owner/syntax/Core/VC/candidate literal と実旧 slice fingerprint による別記述五-key object で全六 getter、正確な anchor/id、slice 全体/entry/unknown/status/kind/completeness/fingerprint/cache miss、反復性、raw 空 owner/syntax error、入力/先行結果不変を検証する。
中立 Unicode trivia/whitespace と実 SourceId skip7 は candidate を保ち実旧 full slice を保持するが、full identity/slice の snapshot 安定は推論しない。真正 package/module（dotted path 含む）・owner label・binder・proof-local spelling edit は該当内容に参加する。Resolver private-neutrality と下位 profile matrix を再利用し、成功する private wrapper、並替え/先行 Lemma receipt、意味同値を捏造しない。
真正 foreign/augmented/recovered source/env の新 error を実既存 checker replay と比較し、不在 VcId も併用して順序を検出する。ordinary Lemma/open・multiowner check が真正に構築可能なら実旧 canonical-input error と比較し、valid source の不在選択も旧 error を保つ。全 asserted API を実行し、架空の正常 AST/owner/check、Accepted/Hit/Complete、placeholder dependency は使わない。

## Source theorem reflexivity check

`check_source_theorem_reflexivity(source: &SurfaceResolvedArena, check: &SourceTheoremCheck<'_>, symbols: &SymbolEnv, snapshot: BuildSnapshotId, vc_id: VcId, policy: &VerifierPolicy) -> Result<(KernelCheckResult, PolicyDecision), String>` は既存 status.rs の互換 facade とし、従来の二-field result と error を保つ。
private check_source_theorem_reflexivity_payload は未変更の下記 pipeline と private 第七引数 include_metadata bool を持つ。必要な五つの直接 caller は互換 check/selection/used-axioms status (false)、raw metadata/wire と witness projection (true) とし、(KernelCheckResult, PolicyDecision, VcKernelEvidenceHandoff, Vec<u8>, Option<KernelAcceptanceMetadata>) を返す。未変更の実 check/evaluator decision の後の true 分岐だけへ既存 guarded metadata mapping と local import を移し、先行 BuiltinDischarge origin で実 KernelPolicyInput を再導出して実 parsed.schema_version() を使う。false 分岐に component/metadata/hash 作業・失敗・Kernel 呼出し・parse・encode を加えない。
最初に指定 source/env、check.typed_ast()/scope()/labels()/resolved_labels()、None で通常 SourceVariableSemanticsChecker::check_theorem_skeletons を replay して error を保つ。その後 borrowed check.owners() == fresh.owners() を要求し、不一致は内部 theorem.proof.invalid_source_evidence。
fresh check を None で lower し、source-theorem-local-v1/vc-v1 と caller snapshot で [raw graph API](../../mizar-vc/ja/generator.md#source-theorem-vc-generation)、同じ Core/graph/selected id で未変更 build_source_theorem_kernel_handoff を呼ぶ。元 Open graph/anchor/status と VC69 whole replay/profile validation を保ち、不要な metadata のため default discharge/Core56/canonical inputs を呼ばない。
実 original handoff envelope の target/profile から通常 schema1 encode_local_evidence(None) と FormulaEvidenceParseContext::v1 parse を行う。未変更 ATP43 encode_local_variable_reflexivity_names を実 parsed variable-equality goal に適用し、同じ handoff からその names で schema2 を encode、同じ target/profile parse context で schema2 を明示許可して通常 parse する。
この手続は決定的 local-reflexivity BuiltinDischarge evidence を意図的に構築し、origin を結果より前に明示する。KernelAccepted/generic Unknown から推定しない。ATP encoded problem/encoded_problem_matches operand は存在しない。
実 stable wire 構築後、Kernel より前に既存 ProofPolicyEvaluator の UncheckedBuiltinDischarge scheduling を要求し、拒否は内部 theorem.proof.kernel_policy_rejected。実 original handoff/parsed schema2 で未変更 check_source_kernel_handoff を再利用し、registration.proof.context error を保つ。
実 result を KernelEvidenceOrigin::BuiltinDischarge と共に KernelPolicyInput::from_kernel_result、既存 evaluator.evaluate_candidate へ渡し、真正 rejection を含む実 result/decision を返す。checker/lower/raw graph/VC69/encode/name の String error は直接保ち、parser error だけ Debug を theorem.proof.parse:{error:?} に写す。自身の profile/owner 不一致は theorem.proof.invalid_source_evidence とし、public diagnostic registry/emission は変更しない。
既存 term_proof_tests に真正 source 三系列だけを追加し、source_registration_owner_inputs/with_source_theorem_check の旧 fixture/helper intent を保って再利用する。通常 formatting と local import alias を使う。
第一系列は original/renamed/trivia/allocation/module/snapshot、実 normal KernelAccepted/DischargedBuiltin、実 fixed schema の独立 generic graph 構築による正確な original target、ProofObligation kind、taint/used-axioms、不変性/反復性を検証し、旧 canonical/identity/full-slice 結果を保つ。
第二系列は実 enabled/disabled/strict policy-format scheduling を API から検証し、真正 enabled 成功/disabled 拒否を要求する。caller boolean/架空 accepted input は使わない。
第三系列は foreign/augmented/recovered source/env の実 checker error が不在選択/policy より先に来ることを保ち、構築可能な真正 Lemma/open/multi/cited profile error を実 lower/VC69 と比較して旧 missing-selected-id error を保つ。既存 no-name/foreign-target/context refusal、lower/oracle matrix を再利用し、Kernel-context wrapper 複製/旧期待変更をしない。
入力は不変で filesystem/source/snapshot 現在性は caller 責任。identity metadata/witness/publication/cache/reuse 権限/full acceptance gate/広域 Steps6–8 closure を返さず、架空 DischargeOutput/Accepted/Hit/Complete、caller payload/hash/graph 代用、Kernel/policy/encoder/try_discharge 変更をしない。

## Source theorem reflexivity selection

`select_source_theorem_reflexivity(source: &SurfaceResolvedArena, check: &SourceTheoremCheck<'_>, symbols: &SymbolEnv, snapshot: BuildSnapshotId, vc_id: VcId, policy: &VerifierPolicy) -> Result<ProofSelection, String>` は既存 status.rs の一つの inline facade とし、[共通の真正検査 payload](#source-theorem-reflexivity-check) を同じ immutable 入力と private include_metadata=false で一度だけ呼んで実 result/decision を取り出し、上流エラーをそのまま返す。
結果取得より前に既存手続きの明示的 BuiltinDischarge origin を選び、受理結果から推測しない。選択対象は実結果の target.sort_bytes とし、snapshot-local Kernel target を theorem の六フィールド original identity や canonical VC と混同しない。
private select_source_theorem_reflexivity_result(&KernelCheckResult, owned crate::policy::PolicyDecision, prior explicit KernelEvidenceOrigin, &VerifierPolicy) -> Result<crate::selection::ProofSelection, String> が local import/error を含む下記の未変更 post-check assembly を所有する。その body だけを移し &result を result に正規化する。実 production 直接 caller はこの selector、[witness](#source-theorem-reflexivity-witness-projection)、[used-axioms status](#source-theorem-reflexivity-status-with-used-axioms) の三つを必須とし、それぞれ同じ実 result と検査前に選んだ origin を渡す。
CandidateSourceId は `source-theorem-reflexivity-v1:` と target 全バイトの lowercase hex を既存 source_discharge_hex で連結する可逆名であり、新しい hash profile ではない。constructor 失敗は内部 theorem.proof.invalid_source_selection とする。
実 decision が DischargedBuiltin の場合だけ、実 result と明示 origin から作る KernelPolicyInput を既存 ProofEvidenceCandidate::from_trusted_kernel_input に渡し、trusted evidence 不在も同じ内部エラーにする。他の実 decision は既存 ordinary candidate に保持し、信頼へ昇格しない。
この候補、実 target bytes、encoded input の None、現在 policy の clone を変更なしの ProofEvidenceSet/select_winner に渡し、実 selection 全体を返す。producer のない priority、provenance、deterministic discharge、witness hash を設定しない。
marker/class/evidence hash/polarity の検査と順序は既存 selection が単独所有する。実 builtin 選択でも ExternalDependencyGap と witness/discharge/provenance hash 不在を保つ。trusted used-axiom availability は実受理結果だけを参照し、記録を合成しない。
追加する source_theorem_reflexivity_selection_* は二系列だけとし、実 check と既存 selector から独立に構成した期待値との selection 全体比較、source/rename/allocation/package/dotted-module/snapshot/policy、受理 hash/polarity と欠落 metadata、反復と入力・先行結果の不変性を扱う。実 check エラー・無効 format・上流順序を保持し、既存三検査系列と Kernel/name/profile 行列を再利用する。架空の受理・拒否 source control や context wrapper を追加しない。
source/snapshot/filesystem currentness は caller 所有。これは fresh selection だけであり、original identity 関連付け、metadata/wire 保持、witness/status/publication/activation/cache/reuse、一般受理や Steps6–8 完了を与えない。既存 checker/policy/selector/test/helper の振る舞いを変えない。

## Source theorem reflexivity status projection

`project_source_theorem_reflexivity_status(source: &SurfaceResolvedArena, check: &SourceTheoremCheck<'_>, symbols: &SymbolEnv, snapshot: BuildSnapshotId, vc_id: VcId, policy: &VerifierPolicy) -> Result<(ProofStatusProjection, mizar_vc::dependency_slice::DependencySlice), String>` は同じ immutable source 入力だけを取る既存 status.rs の一つの inline facade とし、caller の selection/identity/result/hash は受け取らない。
まず [source_theorem_obligation_identity](#source-theorem-original-obligation-identity) から実六フィールド identity と元 slice 全体を保持し、次に同じ source/check/env/snapshot/選択 id/現在 policy で [select_source_theorem_reflexivity](#source-theorem-reflexivity-selection) を呼ぶ。両 String error をこの順で直接伝播する。
実選択 id と真正 selection を既存 VcProofSelection で関連付け、空 portfolio と一つの builtin 入力を変更なしの merge_artifact_proof_selections に渡す。owned 出力は一行だけとし、cardinality 不正は内部 theorem.proof.invalid_source_status、実 merge error は Display とする。caller flag や作られた Accepted outcome で代用しない。
実 merged selection/現在 policy の clone/実 original identity を変更なしの ProofStatusProjectionInput::new/project_status に渡し、optional setter は全て使わない。実 projection error を Display に写し、projection 全体と変更なしの元 slice 全体を返す。
実 normal Kernel result 由来 trusted evidence は既存 selection、class からの写像は既存 projection が所有する。真正 builtin の内部 Accepted でも ExternalDependencyGap(DischargedBuiltinWitnessUnsupported)、accepted witness id 不在、trusted-used-axiom ref/hash 不在、provenance/discharge/witness/encoded producer 不在を保つ。selection の availability から保持されていない axiom 記録や空リスト hash を作らない。
元 slice の Open status、非空 entries/unknowns、IncompleteUncacheable と requires_cache_miss を保ち、projection は fingerprint だけを受け取る。compatibility/discharge 不在では局所 reuse predicate も false であり、完全な phase footprint を証明しない。compatibility token/cache Complete environment/metadata/wire/witness/publication を作らない。
同じ immutable genuine producer 入力による通常の source 関連付けであり、retained-wire/同一 Kernel 検査 provenance、転送可能な acceptance/currentness token、異なる Kernel target と canonical VC/anchor/composite identity domain の比較ではない。source/snapshot/filesystem 現在性は caller 所有。
追加する真正 source 二系列は既存 producer の identity/selection/merge/projection から独立に組み立てた projection 全体と元 slice 全体を比較する。source/rename/trivia/allocation/package/dotted-module/snapshot/policy で六 identity fields、実 accepted class/gap/None fields/reuse miss、反復、入力と先行結果の不変性を保つ。disabled policy/不在選択/foreign/recovered source で実 identity-first と後続 selection error 順を保持し、既存 Kernel/name/profile/identity/selection 行列を再利用する。架空 control や新 context helper は追加しない。
既存 producer/policy/selector/merge/projector/test/helper/保護 artifact を変えない。全 source activation、Artifact status/publication、witness token/reachability、完全依存/reuse、通常 ATP/finder、Steps6–8 は未完のまま。

## Source theorem reflexivity status with used axioms

`project_source_theorem_reflexivity_used_axioms_status(source: &SurfaceResolvedArena, check: &SourceTheoremCheck<'_>, symbols: &SymbolEnv, snapshot: BuildSnapshotId, vc_id: VcId, policy: &VerifierPolicy) -> Result<(ProofStatusProjection, mizar_vc::dependency_slice::DependencySlice), String>` は既存 status.rs の追加 inline facade とし、caller の result/selection/hash/identity/list/decision/flag を取らない。
最初に同じ五 source 入力で未変更 [source_theorem_obligation_identity](#source-theorem-original-obligation-identity) を呼び、真正元六-field identity と元 slice 全体を保持する。明示 KernelEvidenceOrigin::BuiltinDischarge を選んでから include_metadata=false の [payload check](#source-theorem-reflexivity-check) を一度だけ呼ぶ。identity、check の String error を直接この順で伝播し、Kernel 再検査や context/handoff 再構築をしない。
実 decision.class DischargedBuiltin を要求し、同じ result、所有 decision、先行 origin、現在 policy で private [result selector](#source-theorem-reflexivity-selection) を呼び、その String error を直接保つ。selected_class DischargedBuiltin を要求する。自身の association/class/cardinality 不正は内部 theorem.proof.invalid_source_used_axioms_status とする。
同じ実 result から既存 TrustedUsedAxiomsRef::from_kernel_result(&result, origin) を導出し、実 factory error を Display/to_string に写す。raw field 複写、test constructor、caller list、新 hash、偽空 hash を使わない。既存 [trusted used-axioms 条件](#trusted-used_axioms) が引き続き権威を持つ。
空 portfolio と一つの実 VcProofSelection(vc_id, selection) を未変更 merge_artifact_proof_selections に渡し、Display error を保ち、出力一つを要求する。実 merged selection/現在 policy clone/元 identity を ProofStatusProjectionInput::new/project_status に渡し、optional setter は .with_trusted_used_axioms(actual_reference) だけを使う。projector Display error を保ち、projection 全体と元 slice 全体を atomic に返す。
現在 policy、trusted class、availability、同一 accepted-evidence hash の検証、実記録由来 reference/hash の保持、既存 reuse-validation hash の計算は既存 projector だけが所有する。reference は accepted evidence identity/hash/count を持ち、FQN、完全 import/source map、artifact 出力ではない。
[旧 status](#source-theorem-reflexivity-status-projection) は production byte 同一で、trusted-reference/hash None の出力全体を保つ。新真正 status も内部 Accepted/Builtin と ExternalDependencyGap(DischargedBuiltinWitnessUnsupported) を保ち、accepted witness id と encoded/provenance/witness/discharge/compatibility/explanation producer はない。元 identity/fingerprint を witness composite にしない。
元 slice 全体の Open、非空 entries/unknowns、IncompleteUncacheable、requires_cache_miss と局所 reuse false を保つ。実 imported 記録ゼロは通常 Kernel 結果から public factory 経由で得て、偽 list/hash で作らない。実 nonempty source import は未対応。source/snapshot/filesystem currentness は caller 所有とし、witness token/constructor visibility、Draft/stage/file I/O/publication/cache Complete/完全 footprint/activation、広域 Steps6–8 完了を追加しない。
既存 proof_verification.rs に真正二系列だけを追加し、旧 test/helper/import/fixture/.miz/expectation の全 byte と source_registration_owner_inputs/with_source_theorem_check を保つ。helper/context wrapper や架空 Accepted/Rejected/None/Hit/Complete control は作らない。
`source_theorem_reflexivity_used_axioms_status_matches_real_kernel_and_original_slice`: facade より先に真正旧 original identity/元 slice 全体、互換 result/decision、selection/旧 status 全体を得る。期待 reference は実 result と先行明示 Builtin origin から public factory だけで作り、実 used_axioms().len()、accepted hash/polarity/class/ProofObligation/no-taint を使う。実 ref setter と元 identity で独立 merge/project し、projection/slice/reference 全体、元六 identity domain/現在 policy、Some 実 ref/hash/evidence 対応、fail-closed field/miss、旧 None 出力全体を比較する。list/hash を入力せず、対応 profile の空記録を実観測として確認する。
真正七変種（original、binder-only rename、proof-local-only rename、Unicode trivia、実 SourceId 差を確認する skip7、package、dotted module）、snapshot777/778、original の六 policy（default、release requirekernel、developmentBuiltinOnly、profile-id、policy schema2、checker schema2）と他変種の default を使う。真正反復と source/check/env/現在 policy/先行結果の不変性を要求する。全 rename の wire/hash 変化を仮定せず、witness hash/metadata oracle や旧行列を複製しない。
`source_theorem_reflexivity_used_axioms_status_preserves_identity_first_errors`: 旧 check/無効 policy error が異なっても新 API unwrap_err と実 identity unwrap_err を最初に比較し、identity 成功後に実互換 check unwrap_err と比較する。original の disabled empty/formula-only/不在 id、構築可能な Lemma/open/multiple/cited、foreign/augmented/recovered source/env、複合 absent+disabled 順序を実行し、先行真正成功/入力を保つ。到達不能な accepted-metadata/factory/selection/None/rejected 分岐と nonempty import を明示し、下位 real-Kernel の nonempty/rejected/tainted/consistency/mismatch tests を source producer coverage とせず保持する。

## Source theorem reflexivity metadata and checked wire

`check_source_theorem_reflexivity_metadata(source: &SurfaceResolvedArena, check: &SourceTheoremCheck<'_>, symbols: &SymbolEnv, snapshot: BuildSnapshotId, vc_id: VcId, policy: &VerifierPolicy) -> Result<(KernelCheckResult, PolicyDecision, Option<KernelAcceptanceMetadata>, Vec<u8>), String>` は既存の不変 source 入力だけを取る。
[共通の真正 payload](#source-theorem-reflexivity-check) を private include_metadata=true で一度だけ呼び String error を保ち、実 result/decision/optional metadata/実際に parse・検査した schema2 bytes を未変更の公開四-tuple へ投影する。payload は実 result より前に明示的 BuiltinDischarge を選び、再検査・再 encode で対応を作らない。
metadata が Some になるのは実 DischargedBuiltin、Kernel Accepted、taint 無しの ProofObligation、実 AssertFalseForRefutation polarity、真正 accepted evidence hash、未変更 local_evidence_component_hashes の成功が揃う場合だけで、それ以外は None。guarded field map の所有元は本節のみとし、対応を未変更のまま private payload の true 分岐へ移す。互換 check/selection に新 hash 作業や失敗を加えない。
既存 Interface ArtifactHashRef を使う。実 v1 四 component を順に kernel_profile_fingerprint/formula_evidence_hash/substitution_evidence_hash/provenance_hash へ写し、mizar-vc/local-evidence-component/{profile,formula,substitutions,provenance} と schema1.0 を使う。文法は [VC component owner](../../mizar-vc/ja/kernel_evidence_handoff.md#local-evidence-component-hashes) が所有し、equality names を除くため検査済み schema2 wire 全体の hash ではない。
verifier_policy_fingerprint は mizar-proof/verifier-policy、schema(policy.schema_version(),0)、現在の policy_fingerprint().hash() とする。checker_schema_version は設定値 policy.checker_schema_version(),0 であり runtime checker/toolchain の認証ではない。evidence_schema_version は実 parsed version,0 を用い、v1 envelope version で代用しない。
target_binding_hash は mizar-vc/kernel-evidence-handoff-canonical、schema1.0、実 handoff.canonical_hash() とし、formula_context_hash は None。accepted_result_hash は mizar-proof/kernel-accepted-evidence、schema1.0、先行する明示 origin による実 accepted evidence hash のみ。新 hash profile や target/context/composite 代用を追加しない。
実 result/decision 全体、任意の plain metadata、実際に検査した所有 bytes を返す。caller が複写できる metadata/bytes は opaque trusted witness metadata、commit publication、source currentness、転送可能な受理 token を作らない。本 raw facade は引き続き元 identity や別 context/composite binding を返さず、それらの対応は [witness 投影](#source-theorem-reflexivity-witness-projection) が所有する。完全 footprint、trusted witness/discharge/publication/reuse は保留し、既存 status は不完全な元 slice 全体と miss を保つ。
追加する source_theorem_reflexivity_metadata_* は真正 source/check/env helper を変更せず使う二系列だけ。既存 original handoff、通常 v1 parse、実 reflexivity names と schema2 wire/parse を新 facade より先に独立構築する。全 metadata field を既存 component/現在 policy/実 result producer と比較し、返す bytes 全体と実互換 result/decision、代表 source/allocation/package/dotted-module/snapshot/policy、反復、入力・先行結果の不変性を検証する。
実 upstream profile/source/env/不在選択/無効 policy の error を既存互換 check と比較し、旧 selection/status/normal-Kernel/name/identity 行列を全て保つ。架空の rejected/None-metadata control、Kernel-context wrapper 複写、旧 test intent 変更、別実行の一致による provenance 主張で coverage を作らない。
旧 checker/selection の戻り型、policy/Kernel/VC/ATP/Artifact の振る舞い、保護 artifact、source activation、通常 finder/runtime/cache、広域 Steps6–8 完了を変えない。

## Source theorem reflexivity witness projection

`project_source_theorem_reflexivity_witness(source: &SurfaceResolvedArena, check: &SourceTheoremCheck<'_>, symbols: &SymbolEnv, snapshot: BuildSnapshotId, vc_id: VcId, policy: &VerifierPolicy) -> Result<(ProofSelection, mizar_artifact::proof_witness::ProofWitnessRef, Vec<u8>), String>` は既存 status.rs の一つの inline public facade で、同じ六つの不変 source 入力だけを取る。caller の result/wire/hash/identity/decision/flag は取らない。
まず policy を除く同一の五 source 入力で [source_theorem_obligation_identity](#source-theorem-original-obligation-identity) を呼び真正 identity を保持する。未使用の元 slice 全体だけを未変更で破棄し、戻り field や Complete 主張を追加しない。次に実 result より前に明示 BuiltinDischarge origin を選び、include_metadata=true の [実 payload check](#source-theorem-reflexivity-check) を一度だけ呼ぶ。identity、payload の String error をこの順で直接伝播する。
実 DischargedBuiltin decision と同一呼出しの metadata Some を要求する。実 result と先行 origin から実 accepted polarity/KernelPolicyInput を導出し、既存 trusted candidate factory Some と既存 selection の selected class DischargedBuiltin を要求する。association、metadata 欠落、trusted factory、selection class、structural writer の失敗は internal theorem.proof.invalid_source_witness。
private [共通 result selector](#source-theorem-reflexivity-selection) へこの実 result、所有 decision、先行 origin、現在 policy を渡し、その失敗を既存 invalid_source_witness に写す。先行 guard/error 順序と optional field 欠落を含む selection 全体を保つ。未変更 composite 用 target_hex だけをこの実 result から再計算し、raw metadata は実 provenance を別に保持する。
二つの producer-owned profile は既存 CanonicalHashDomain/CanonicalJson、version1.0、excluded path 無しとする。codec/hash primitive/type/field/adapter/dependency を追加しない。
Artifact family `mizar-proof/source-theorem-reflexivity-witness-payload` は同一呼出しで実検査した schema2 の全 byte（equality names を含む）の lowercase hex を一つの CanonicalJson STRING として hash する。v1 component や witness-store payload profile とは別。
Interface family `mizar-proof/source-theorem-reflexivity-obligation` は次の厳密な 15 key の CanonicalJson OBJECT を hash する:
`original_obligation_fingerprint`, `checked_target_bytes`, `local_context_identity_hash`, `kernel_profile_fingerprint`, `verifier_policy_fingerprint`, `checker_schema_version`, `evidence_schema_version`, `target_binding_hash`, `formula_evidence_hash`, `substitution_evidence_hash`, `provenance_hash`, `formula_context_hash`, `accepted_result_hash`, `accepted_goal_polarity`, `witness_artifact_hash`。
元 identity.obligation_fingerprint()、実 Kernel target.sort_bytes の全 byte、同一実 handoff.context_identity_hash() は lowercase-hex STRING。旧 identity は既に anchor/canonical VC/goal/元 context/slice fingerprint と id==anchor を結ぶため、六 field をこの object に複写しない。
全 metadata hash reference と full witness hash は完全な to_artifact_hash_string STRING、両 version は canonical schema-version STRING、imported formula_context_hash は null、実 accepted polarity.as_str() は STRING とする。guard/field の唯一の所有元は [raw metadata 対応表](#source-theorem-reflexivity-metadata-and-checked-wire) であり、実 schema2 version は envelope1 や runtime checker 認証ではない。
handoff.context_identity_hash() は [独立 VC context owner](../../mizar-vc/ja/kernel_evidence_handoff.md#conceptual-handoff-shape) が所有する。target_binding_hash/canonical binding/元 context/v1 component hash で代用せず、context のための第二 handoff/check を構築しない。元 identity/id/fingerprint、Kernel target、canonical binding、handoff context、新 composite は別 domain のままで、反復一致は実行 receipt ではない。
既存 ProofWitnessRef を [Artifact built-in profile](../../mizar-artifact/ja/proof_witness.md#built-in-reference-profile) により構築する。schema3.0、実元 identity.obligation_id()、新 Interface composite obligation_fingerprint、DischargedBuiltin/FormulaSubstitutionKernelEvidence、full payload witness_artifact_hash、未変更の同一呼出し raw metadata を使う。
path は `proof-witnesses/source-theorem-reflexivity-v1/<lowercasepayload digest>.bin`。既存 builtin_proof_witness_ref_json で検証して、実 selection/reference/実 wire の全体を atomic に返す。codec は変更しない。
通常の caller-copyable reference は未公開データであり、TrustedKernelWitnessMetadata/CommittedWitnessPublicationProof、draft/stage/file write、selected witness/discharge、reuse/currentness、runtime checker 認証、CompleteSlice/phase footprint、activation/publication を作らない。既存 unsupported Builtin status、不完全な元 slice 全体/cache miss、通常 ref2/enclosing profile、publication guard は未変更。
他の全 production byte、公開出力/error、真正 check/evaluator 順序、通常 Kernel/policy/registration/VC/ATP/Artifact body と、既存 Kernel helper の実直接四 caller・raw VC generator の実三 caller を保つ。三 caller の共通 selection assembly だけを移し、新 field、fixture、source-handoff 再構築を追加しない。
既存 proof_verification.rs に真正三系列だけを追加し、旧 helper/test/import/fixture/.miz/expectation の全 byte を保持する。未変更 source_registration_owner_inputs/with_source_theorem_check を再利用し、新 helper/context wrapper を作らない。
`source_theorem_reflexivity_witness_matches_original_identity_and_checked_wire`: facade より前に真正元六-field identity/不完全 slice 全体と旧公開 raw check/result/decision を独立取得し、実元 Core/raw VC/handoff、v1 parse、実 ATP names、厳密 v2 wire/parse/context、policy input/polarity/trusted factory/旧 selection、既存 CanonicalJson/Artifact field による期待 15-key composite/full-wire hash/reference を独立構築する。selection/reference/bytes 全体、全旧 identity field/domain/anchor id、metadata/schema2/names/context/target/accepted hash、旧 selector None field を比較する。builtin ref3 writer/reader round-trip と任意の通常 ref2 writer rejection 一件のみで、enclosing/artifact/manifest fixture は加えない。
真正 source 七 variant（original、binder-only rename、proof-local-only rename、Unicode、実 SourceId skip7 と差異 assertion、package、dotted module）、original に代表六 policy/他は default、snapshots777/778 を使う。全 rename が wire を変えるとは仮定しない。反復と source/check/env/policy/先行結果/witness の不変性、旧 metadata/check/selector/status を保ち、facade が触れない独立 oracle local の無意味な clone/assert を省く。
`source_theorem_reflexivity_witness_hashes_bind_all_participants`: facade より前に真正旧 product から厳密 15-key 期待 object と期待 payload hash を独立構築する。全 15 単一参加要素の structural CanonicalJson/hash 文法 mutation で composite hash が変わり、実検査 wire/name byte の変更で payload hash が変わる。同一 handoff の別 context と full schema2 wire を使い、v1 component/canonical binding/imported None で代用しない。第一系列の行列を再利用し、setup 複写・架空 KernelAccepted/Rejected/None/Hit/Complete・偽 complete slice・API へ caller metadata/control を渡すことは避ける。mutation は構造 hash coverage のみを示す。
`source_theorem_reflexivity_witness_preserves_identity_first_errors`: 旧 check/無効 format が異なっても実 identity producer unwrap_err を最初に比較し、identity 成功後は実旧 raw-check unwrap_err と比較する。original の disabled empty/formula-only/不在 id、構築可能な Lemma/open/multi/cited、foreign/augmented/recovered source/env、複合 absent+disabled 順序、入力/先行実結果の不変性で真正 API を実行する。現 profile の真正成功は BuiltinAcceptedSome 又は upstream Err とし、association/false/None guard が到達不能なら control を捏造せず明示する。旧独立 normal Kernel/name/profile/identity/raw metadata/selection/status 行列を保つ。

## Source theorem reflexivity trusted witness metadata
### Source theorem trusted witness metadata API
`source_theorem_reflexivity_trusted_witness_metadata(source: &SurfaceResolvedArena, check: &SourceTheoremCheck<'_>, symbols: &SymbolEnv, snapshot: BuildSnapshotId, vc_id: VcId, policy: &VerifierPolicy) -> Result<crate::witness_store::TrustedKernelWitnessMetadata, String>` は既存 status.rs の compatibility facade であり、この六つの不変入力だけを取る。
### Source theorem trusted witness metadata invariants
同じ六つの不変入力で [同一検査 token/wire producer](#source-theorem-trusted-witness-metadata-with-wire) に一度だけ委譲し、wire だけを捨て token を保持する。元 raw-check 順序、class/error/binder 規則と opaque token 検証はその producer が所有し、再検査や caller metadata を受け入れない。
### Source theorem trusted witness metadata boundaries
既存の所有 opaque token だけを返す。元/composite/current-source identity、完全 wire/context receipt、Draft/stage/path/file/manifest/commit proof、selected witness/discharge と trusted-used-axiom reference/hash、status projection、CompleteSlice/完全 footprint、activation/runtime/service/cache/reuse/publication、広域 Steps6–8 権威は作らない。source/check/snapshot freshness は caller 所有とし、BuiltinWitnessUnsupported、元 Open/IncompleteUncacheable slice 全体と cache miss、acceptance/policy、通常/Builtin Artifact profile を保つ。旧 raw metadata facade は引き続き opaque token のない plain data を返す。
### Source theorem trusted witness metadata test design
既存 term_proof_tests/proof_verification.rs に source_theorem_reflexivity_trusted_witness_metadata_matches_fresh_kernel と source_theorem_reflexivity_trusted_witness_metadata_preserves_upstream_errors だけを追加し、未変更 source_registration_owner_inputs/with_source_theorem_check と旧全 byte を保つ。helper/top-level import/fixture は追加しない。positive call より先に実旧 tuple 全体を得て Accepted/ProofObligation/no taint/DischargedBuiltin/Some を確認し、先行 origin と public real-result factory で期待 KernelPolicyInput/TrustedKernelEvidence を導出する。kernel_acceptance/trusted_kernel_evidence 全体と class/origin/hash/polarity、token 全体の反復一致、渡した source/env/check/policy の不変性を検証する。 [旧 raw metadata 系列の source/snapshot/policy case](#source-theorem-reflexivity-metadata-and-checked-wire) を再利用し、実旧生成物と比較し、hash 不変性の仮定や下位行列の重複を避ける。
negative は最初に実旧 metadata facade unwrap_err を得て、完全に同じ入力の新 error と比較する。disabled empty/formula-only format、不在 VC と複合 absent+disabled 優先順、真正で構築可能な Lemma/open/multiple/cited、foreign/augmented/recovered source/env を扱い、渡した入力/先行真正成功を保つ。現真正経路は BuiltinAcceptedSome または upstream Err のため class/None/binder-mismatch guard は静的確認だけの場合があり、下位 constructor tests を保持する。架空 result/token/空 hash/None/Hit/Complete や caller control で coverage を作らない。

## Source theorem trusted witness metadata with wire
`source_theorem_reflexivity_trusted_witness_metadata_with_wire(source: &SurfaceResolvedArena, check: &SourceTheoremCheck<'_>, symbols: &SymbolEnv, snapshot: BuildSnapshotId, vc_id: VcId, policy: &VerifierPolicy) -> Result<(crate::witness_store::TrustedKernelWitnessMetadata, Vec<u8>), String>` を旧 token 専用 facade の doccomment 全体より前に inline で置く。compatibility facade は一度だけ委譲し、wire だけを捨て、API/token/error 順序と他の旧本体を保持する。
result より前に明示 BuiltinDischarge origin を選び、同じ六不変入力で未変更 [check_source_theorem_reflexivity_metadata](#source-theorem-reflexivity-metadata-and-checked-wire) を最初に一度だけ呼ぶ。String error/result/decision/metadata/wire を保ち、実 DischargedBuiltin と Some metadata を要求し、それ以外は theorem.proof.invalid_source_trusted_witness_metadata とする。実 result/先行 origin から既存 KernelPolicyInput::from_kernel_result を作り、同一実行 metadata を未変更 crate-private TrustedKernelWitnessMetadata::from_kernel_policy_input で結び、error.to_string() と全検証を保つ。この token と同じ実行の検査済み wire 全 byte を返す。再検査、private payload 呼出し、再構築、caller evidence/result/metadata/bytes を使わない。
既存型のみとし、helper/field/adapter/codec/hash/dependency や constructor 可視性を変えない。tuple は元/composite/current-source identity、独立 context/currentness receipt、Draft/stage/path/file/manifest/commit proof、status/selected-witness/discharge/used-axiom projection、完全 dependency/phase footprint、activation/runtime/service/cache/reuse/publication、Steps6–8 完了を作らない。source/check/snapshot freshness は caller 所有。acceptance/policy、BuiltinWitnessUnsupported、元 Open/IncompleteUncacheable と Artifact profile は未変更で、source-projection と generic-staging の hash を区別する。
### Trusted witness wire tests
既存 proof_verification.rs の term_proof_tests の最終 brace より前に source_theorem_reflexivity_trusted_witness_metadata_with_wire_matches_same_check と source_theorem_reflexivity_trusted_witness_metadata_with_wire_preserves_errors だけを挿入する。旧全 byte/import/helper/fixture/test を保持し、source_registration_owner_inputs/with_source_theorem_check と local import のみ使う。[既存 token test design](#source-theorem-trusted-witness-metadata-test-design) と raw owner の七 source variant、二 snapshot、original のみ六 policy/他 default を再利用し、下位完全行列や全 rename による wire 変更の仮定を追加しない。
先行 BuiltinDischarge origin と実 raw metadata Result を最初に取得し、Accepted/ProofObligation/no taint/DischargedBuiltin/Some を確認する。public real-result factory で期待 KernelPolicyInput/TrustedKernelEvidence を導出し、token metadata/evidence 全体と wire 全 byte、通常 v2 parse と metadata evidence version を比較する。旧 token 専用 Result 全体との比較、tuple 反復、実際に渡す source/env/check/policy と先行結果の不変性を確認する。
リンク先の真正 unsupported/foreign/augmented/recovered/disabled/absent と複合優先順 case は、最初に raw metadata unwrap_err を取得し、同じ入力の新 error と compatibility error を比較する。先行真正成功と入力を保つ。class/None/binder fallback guard が到達不能なら静的確認と既存下位 constructor test で扱う。架空 result/token/metadata/wire/Hit/Accepted/Complete や caller control を positive coverage に使わない。

## Source theorem witness file candidate
`source_theorem_reflexivity_witness_file_candidate` は[元ソース取得](#original-source-witness-draft-acquisition)と同じ5つの借用入力から `Result<((ProofStatusProjection, DependencySlice, ProofWitnessDraft), CanonicalJson, ProofWitnessRef), String>` を返す。最初に未変更の取得を一度呼び、エラーと所有済み triple 全体を保持する。呼出側の draft/check/result/token/bytes は受け取らない。kernel 合成は Status が所有し、Witness Store は kernel を呼ばない。
同じ draft の元 id/fingerprint、実 wire family/version/bytes と accepted metadata 全体から、別の[witnessファイル](./witness_store.md#source-theorem-witness-file)を作る。既存 private source_discharge_hex と typed hash encoding を再利用する。新 Artifact domain で全体を除外なしに hash し、実 path、元 identity、metadata、新 hash の明示 reference3/DischargedBuiltin/formula-substitution 候補を未変更 Artifact builtin encoder で検証する。元 triple・file・ref を I/O/stage なしに一括で返す。
旧 draft/stage と raw STRING/composite の hash は不変かつ別である。ref は普通の構造データであり committed receipt/token ではない。元 Open/IncompleteUncacheable、Builtin stage reference=None、publish guard を保ち、currentness・file replay の受理・全 footprint・used-axioms・reuse/publication の達成を与えない。

## Original-source witness draft acquisition
`source_theorem_reflexivity_witness_draft_from_original(source: &SurfaceResolvedArena, symbols: &SymbolEnv, snapshot: &BuildSnapshot, policy: &VerifierPolicy, witness_path: &str)` は既存の所有済み `(ProofStatusProjection, DependencySlice, ProofWitnessDraft)` または String を返す。Proof の既存依存で合成し、Syntax 公開、依存境界の迂回、呼出側の target/check/result/token、新しい表現を導入しない。
最初に Resolver の[元ソース proof 入力生成](../../mizar-resolve/ja/symbols.md#original-source-proof-input-acquisition)、次にその scope で Checker の[実 equality 生成](../../mizar-checker/ja/type_checker.md#original-source-theorem-equality-producer)を呼ぶ。実際の推論結果と TypedAst 搬送は別であり、完全な typed phase の達成にはしない。その搬送、scope、実 label collection/resolution 全体を、変更しない SourceVariableSemanticsChecker::check_theorem_skeletons に None とともに渡す。
その check を既存 lower_source_theorem_skeletons(None) で下げ、snapshot.id と既存 generation/schema `source-theorem-local-v1`/`vc-v1` で実 VC 集合全体を生成し、ちょうど1件を要求する。0件または複数件は既存 `theorem.proof.invalid_source_witness_draft` を返す。フィルタ、最小値選択、架空 id、下位 profile 拡張を行わない。実際の唯一の VC id と同じ source/check/env/snapshot/policy/path を変更しない[7入力 draft 生成](#source-theorem-witness-draft)へ渡し、I/O なしに triple 全体を返す。この順序で元の String エラーを伝播し、既存 draft の identity/check/policy/path 優先順も保つ。
下位 profile、recovery、型、label、診断、証明・受理方針を保つ。元の Open/IncompleteUncacheable、BuiltinWitnessUnsupported、内部 Builtin stage reference=None と publication guard は不変。snapshot/source 所属、currentness、確定 dependency 認証は呼出側の責任であり、渡した metadata は runtime 認証ではない。full footprint、publication、Hit、CompleteSlice、Steps6–8 の完了にはしない。
### Original-source witness draft tests
既存 term_proof_tests の最後の括弧直前に `original_source_witness_draft_matches_genuine_pipeline` と `original_source_witness_draft_preserves_errors` だけを追加する。古い Rust/import/helper/test/fixture の全 byte を保ち、新 helper/type/adapter は作らない。成功入力は実 SnapshotRegistry Disk/OpenBuffer capture、source_unit_from_loaded/Frontend::run_loaded、元の resolver arena/environment を使い、実 snapshot/source 変化と purity/replay を検査する。
新しい成功呼出しの前に、変更しない with_source_theorem_check を独立した既存の実生成 control として使い、実 Core/VC 集合がちょうど1件であることを確認し、その実 id で既存 draft 生成を呼ぶ。返る triple 全体、元の status/conservative slice と capture provenance を比較する。架空の正の VC、Accepted、token、complete slice を渡さず、下位推論や既存 draft hash 行列を複製しない。
負例は構成可能な各段階の実エラーを比較する。元・別 source・拡張 environment と recovery/unresolved（複合時の優先順を含む）、未対応 equality profile、実際に型付けできる異変数 source の既存 seal 拒否、不変の lemma-reference fixture から実生成した2件の VC 全体の拒否、単独 lemma の既存 identity 拒否を扱う。下流拒否の前に必要な実下位成功を確立する。有効な唯一の target で既存 draft の policy 無効/path エラーと優先順を比較する。0 target など構成不能な guard は静的に確認し、到達のために中間構造を捏造しない。

## Source theorem witness draft
`source_theorem_reflexivity_witness_draft(source: &SurfaceResolvedArena, check: &SourceTheoremCheck<'_>, symbols: &SymbolEnv, snapshot: &mizar_session::BuildSnapshot, vc_id: VcId, policy: &VerifierPolicy, witness_path: &str) -> Result<(ProofStatusProjection, mizar_vc::dependency_slice::DependencySlice, crate::witness_store::ProofWitnessDraft), String>` を status.rs の既存 status 射影の doccomment 全体の直前へ inline で追加します。入力はこの七つの不変 operand だけで、既存 API・本体・import・下位 producer は変更しません。
最初に snapshot.id で[元の identity と conservative slice 全体](#source-theorem-original-obligation-identity)を取得します。明示的な BuiltinDischarge origin を決めてから、同じ source/check/env/id/vc/policy で既存の[raw metadata 検査](#source-theorem-reflexivity-metadata-and-checked-wire)を一度だけ呼び、identity、raw check の順に String error をそのまま伝播します。実際の DischargedBuiltin decision と Some metadata を要求し、それ以外は theorem.proof.invalid_source_witness_draft です。呼出側の result・decision・token・bytes は受け取りません。
同じ result/decision/origin/policy を既存 private selector に渡し、既存 single-VC merge と元の identity による project_status、同じ result/origin/metadata による既存 crate-private binder を使います。slice・projection 全体を保持し、既存 typed error は to_string で返します。実際の projected candidate がなければ invalid_source_witness_draft です。再検査・trusted marker 再構築・raw composite identity・重複 class guard・任意 reuse field 設定は追加しません。
既存 ProofWitnessProvenance は[保持済み snapshot・依存参照 digest](../../mizar-session/ja/snapshot.md#captured-metadata-fingerprints)、実際の projected candidate、先に決めた origin、元の projected VC/slice fingerprint、現行 policy fingerprint、同じ検査の checker/evidence schema と accepted-result digest から作ります。producer identity は `mizar-proof/source-theorem-reflexivity-witness-draft/v1` です。projection/provenance 両方の trusted-used-axioms 参照と advisory provenance は未設定のままで、全 used-axiom 記録は別作業です。
既存 ProofWitnessDraft::new に元の projected obligation id/anchor、Interface ArtifactHashRef の family `mizar-proof/source-theorem-original-obligation`・version1.0・元の obligation digest、実際の projection/token、同じ検査の wire 全体を渡します。payload schema family は `mizar-proof/source-theorem-reflexivity-checked-wire`、version は実際の evidence_schema_version、既存 canonical encoder に基づき canonical_bytes_required=true です。witness_path をそのまま既存 validator に渡し、error を伝播します。[Witness Store の hash・検証](./witness_store.md#stable-hashing)が引き続き唯一の所有元で、OBJECT envelope は raw STRING/composite profile と別です。
projection/slice/draft 全体を一括で返し、stage・I/O は行いません。snapshot/source/check の対応と現在性は呼出側が所有し、BuildSnapshot と記録済み依存参照は metadata であって registry/native capture・committed dependency の認証ではありません。元の Open/IncompleteUncacheable、公開 BuiltinWitnessUnsupported、内部 Builtin stage reference=None、publish guard、selected witness/discharge hash と G004 は維持します。公開・Hit・CompleteSlice・全 footprint・runtime/service・Steps6–8 完了は与えません。
### Source theorem witness draft tests
既存 proof_verification.rs の term_proof_tests 最終 brace の直前に source_theorem_reflexivity_witness_draft_matches_captured_inputs と source_theorem_reflexivity_witness_draft_preserves_errors だけを追加し、既存 bytes/helper/import/fixture を維持します。実際の native Disk/OpenBuffer の SnapshotRegistry::load_snapshot、source_unit_from_loaded/Frontend::run_loaded と既存 parser/import provider、実際の resolver arena/shell/signature collection を inline で使い、変更しない with_source_theorem_check を再利用します。新しい helper/type/context wrapper/dependency や架空 snapshot で positive provenance を作りません。
新しい成功呼出の前に、既存 API の実際の元 identity/slice 全体、status projection、trusted token/wire を取得し、既存 constructor で期待 Draft 全体を独立に組み立てます。返却 triple 全体、元の domain/field、実際の accepted metadata/wire/policy/candidate と保持済み capture digest を比較します。native origin/source/snapshot と有効 policy の変種、再実行、実際に受け取った入力の不変性を検証します。捕捉した空参照は incomplete のままで、非空参照を与える場合も明示的に metadata だけです。raw file hash を Build の typed lock/config profile や runtime attestation と同一視しません。
実際の identity error を最優先し、identity 成功後は既存 raw-check error を比較します。absent VC/disabled policy の組合せ、構成可能な non-theorem/open/multiple/cited・foreign/recovered source/env を含みます。不正 path は既存の実際の生成物による Draft constructor の error と比較します。構成不可能な防御的 association/selection guard は静的にレビューし、Accepted/Rejected/None metadata・complete slice・公開 receipt を捏造しません。既存の独立した下位行列と受理方針は保持します。

## Source algorithm return reflexivity

`check_source_algorithm_return_reflexivity(source: &SurfaceResolvedArena, nodes: &TypedArena, symbols: &SymbolEnv, snapshot: BuildSnapshotId, vc_id: VcId, policy: &VerifierPolicy) -> Result<(KernelCheckResult, PolicyDecision), String>` を既存 status.rs の theorem reflexivity の完全な doccomment/function 直前へ一つ追加する。caller Core/check/VC/evidence/target/result/flag は受け取らない。
FIRST は既存 check_source_algorithm_types(source, nodes, symbols)、その fresh check の lower_source_algorithms、既存 generate_source_algorithm_postconditions の順。GenerationSchemaVersion("mizar-vc-generation-step5c14-return-v1") と VcSchemaVersion("mizar-vc-vcset-step5c14-return-v1") を固定し、各上流 String をその順で保持する。
その実 Core、全 raw graph、選択 id、generation schema を既存 [build_source_algorithm_return_kernel_handoff](../../mizar-vc/ja/kernel_evidence_handoff.md#source-algorithm-return-local-handoff) へ渡す。object 引数一つの Return/ensures 代入後 reflexivity、guard、policy、normal manifest/wire、provenance、上流優先 replay は同 owner の責任とし、再実装・拡張しない。
実 handoff を names なしの v1 に encode し、その実 target/profile の FormulaEvidenceParseContext::v1 で parse する。ClauseTautologyPolicy は既存 theorem と同じく実 profile から導き、未対応値は内部 algorithm.proof.invalid_source_evidence。
names は parse 済み v1 に既存 mizar_atp::backend::encode_local_variable_reflexivity_names を適用して得る。同じ handoff をその names で encode し、同じ parse context へ schema2 を許可して正確な bytes を parse する。caller names、供給 axiom formula、架空 SAT clauses、二回目の handoff/check 再構築は禁止。
KernelEvidenceOrigin::BuiltinDischarge を検査前に選ぶ。現 policy clone の既存 ProofPolicyEvaluator.can_schedule_kernel_check(UncheckedBuiltinDischarge { has_stable_kernel_representation: true }) を使用し、不許可なら Kernel 前に内部 algorithm.proof.kernel_policy_rejected。
その同じ handoff/parse 済み v2 と内部 algorithm.proof.invalid_source_evidence で既存 private check_source_kernel_handoff を一回呼ぶ。同 helper だけが元 context と normal ProofObligation Kernel を構成する。実 result と先に選んだ origin の KernelPolicyInput::from_kernel_result を同 evaluator へ渡し、実 result/decision 全体を返す。
handoff、ATP-name、private helper の String は直接保持し、両 parser error は内部 algorithm.proof.parse:{error:?} とする。public diagnostic ではない。policy/Kernel/ATP/checker/lowerer/generator/handoff/helper 本体、alias/type/field/adapter/dependency/export/並行表現を変更せず、既存段階を inline で呼ぶだけとする。
実 accepted・untainted ProofObligation のみ既存 DischargedBuiltin 分類を得る。raw Open graph、不完全 anchor、Deferred termination accounting、既存 acceptance/cache 挙動を保持し、return reflexivity や terminating 宣言を E4・algorithm 全体の受理へ昇格しない。
source/snapshot/filesystem currentness は caller 所有。元 obligation identity、完全 context/dependency/footprint、選択 witness/status metadata、trusted witness/token、Artifact/staging/publication、cache/reuse/Hit、広い algorithm/service/LSP/Steps6–8 完了を与えない。新規 file I/O、backend execution、直接 SAT、架空 producer/Core/check/VC/receipt/Accepted を禁止する。

### Source algorithm return reflexivity tests

既存 runner/proof_verification.rs の最終 term_proof_tests brace 前へ source_algorithm_return_reflexivity_checks_genuine_targets と source_algorithm_return_reflexivity_preserves_upstream_and_policy_errors のみ追加する。旧 test/helper/import/fixture/MIZ/expectation 全 byte を保護し、既存 source_registration_inputs/step5c8_test_frontend と [return fixture](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz) を再利用する。旧 harness route は vc_generation のまま。
API より前に実 check/Core/raw graph/VC74 handoff、v1 parse、実 name encoder、v2 wire/parse を独立構成し、doc 外で Rust 前に固定した literal normal grammar vector と name を比較する。実 handoff から元 Kernel context を直接 inline 構成し、実 normal Kernel と明示 Builtin origin の既存 evaluator を実行する。result/decision 全体、target/accepted hash/polarity/ProofObligation/no-taint/実 used_axioms、反復性・純粋性を比較し、構築 Accepted control/helper wrapper を使わない。
原文、parameter rename、Unicode trivia、algorithm owner rename を baseline で扱う。別 snapshot と default/development/interactive policy の変更は原文のみに独立の case とし、直積にしない。実 raw Open/Deferred/不完全 anchor/accounting と source/nodes/symbols/policy/先行実 result を保持し、編集間 hash 不変は仮定しない。
absent id、実 zero-VC と広い profile 一つは既存 handoff error、no-ensures 二 return Core は既存 generator String、object result の bare-return 実 source は既存 checker String を absent-id/disabled-policy より先に返す。valid source/handoff 後の empty・formula-only format は scheduling error、absent target は disabled policy より先。既存 lower profile/normal grammar/name/context/policy の拒否 matrix を再利用し、架空 producer や checker test の重複は禁止。

## Source algorithm return canonical inputs

`source_algorithm_return_canonical_inputs(check: &SourceAlgorithmCheck<'_>, snapshot: BuildSnapshotId, vc_id: VcId) -> Result<(CanonicalVcFingerprint, Hash, LocalContextFingerprint, DependencySlice), String>` を既存 status.rs の source_theorem_canonical_inputs の完全な doccomments/function 前へ一つ追加する。実 opaque source check と通常の snapshot/選択だけを受け、caller Core/maps/VC/hashes/slice/result/flag は受けない。
FIRST は lower_source_algorithms(check)、続いてその実 Core と snapshot、固定 GenerationSchemaVersion("mizar-vc-generation-step5c14-return-v1") / VcSchemaVersion("mizar-vc-vcset-step5c14-return-v1") の既存 generate_source_algorithm_postconditions。guard 選択・target lookup 前に lower、generator の String を順に保持する。
生成後、その Core の全 algorithm parameter の実 optional ty_guard root を順序どおり集め、架空 id や早い検証 error を導入しない。単一引数 return profile の絞込みは既存 raw generator/profile と後段 component owner に任せ、別 grammar/checker/profile を実装しない。
同じ check、実 Core、集めた root で既存 [source_algorithm_formula_payloads](../../mizar-core/ja/elaborator.md#source-algorithm-formula-payloads) を呼ぶ。実 fresh Core 全体、全 available formula、選択 Core58 content の認証と encode は同 owner のみが行う。String を保持し、caller label、Debug/syntax byte、架空 map、normal Kernel formula byte を代用しない。
実 raw VcSet.[source_algorithm_return_canonical_components](../../mizar-vc/ja/vc_ir.md#source-algorithm-return-canonical-components) を選択 id、同じ Core/generation schema、実 formula map で呼ぶ。正確な三値と String を保持し、whole replay、実 guard/代入後 goal/context/policy、既存 VC/goal/context hash domain は同 owner の責務とする。generic fingerprint None を fallback にしない。
続いて同じ元 VcSet 全体と discharge_output None で既存 [try_compute_dependency_slices](../../mizar-vc/ja/dependency_slice.md) を呼ぶ。Display error を保持し、slice_for(選択 id).cloned() を選ぶ。欠落のみ内部 algorithm.proof.invalid_source_canonical_inputs とし、正確な三値と元 slice 全体を一括で返す。
順序は lower、raw generation、実 formula payload、specialized components、raw slice producer/選択。mutation、切詰め・再構築、default/None/error 抑制、旧出力変更、新 hash/codec/type/field/helper/adapter/alias/dependency/export/並行表現を禁止する。raw data の合成であり、source check 再実行や別 Kernel check ではない。
positive は実 object 引数/result 一つ、単独 Return/ensures、選択 raw Open postcondition のまま。元 goal/context anchor の不完全性、Deferred termination accounting、slice の非空 entries/unknowns、IncompleteUncacheable と requires_cache_miss を保持し、specialized hash で旧 anchor や全 footprint を完成させない。
source check は実 Checker が供給し、source/snapshot/filesystem currentness は caller 所有。元 obligation identity/candidate anchor を作らず、Proof acceptance/policy/selection、ATP、Kernel、witness、Artifact/staging/publication、cache を呼ばない。広い algorithm/E4/termination/全体受理、完全 context/dependencies/footprints、trusted witness/reuse/Hit/services/LSP/Steps6–8 は未完。架空 producer/Core/check/map/Accepted/Complete を禁止する。

### Source algorithm return canonical inputs tests

既存 proof_verification.rs の最終 term_proof_tests brace 前へ source_algorithm_return_canonical_inputs_preserve_components_and_slice と source_algorithm_return_canonical_inputs_preserve_upstream_order のみ追加する。旧 test/helper/import/fixture/MIZ/expectation 全 byte を保持し、既存 source_registration_inputs/step5c8_test_frontend と [return fixture](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz) の実 Checker を使用する。旧 harness route は vc_generation のまま。
API より前に実 check を lower し、固定 schema の元 raw graph を生成する。実 Core payload factory へ [実 parameter guard] を要求し、既存 VC73 components と旧 raw slice producer を呼ぶ。slice の全 entry/unknown/fingerprint を含む四値全体、選択 id と raw Open/Deferred/不完全 anchor、generic fingerprint None、slice 非空/IncompleteUncacheable/requires_cache_miss、反復性と source/nodes/symbols/check/Core/raw graph/map/先行実 result の不変を比較する。架空 guard map/slice/identity/accepted control を使わない。
原文、parameter rename、Unicode trivia、algorithm owner rename を baseline で扱い、別 snapshot は原文のみの独立 case とする。編集間・slice 全体の snapshot hash 不変を仮定せず、旧 byte/hash/schema/profile matrix をコピーしない。期待値は実既存 producer から得て、独立した下位 oracle は変更しない。
実 absent id、zero-VC、広い raw profile 一つは前段成功後の旧 pipeline error と比較する。実 no-ensures 二 return Core は absent id でも formula 射影前の実 raw generator String が先になることを示す。旧 error を一括で保持し、先行成功/入力を変えない。opaque-check/lower/formula 固有 failure や到達不能な missing slice は既存下位 test/static review に任せ、架空 check/Core/map、新 helper、下位 matrix 重複を作らない。

## Source algorithm return obligation identity

`source_algorithm_return_obligation_identity(source: &SurfaceResolvedArena, check: &SourceAlgorithmCheck<'_>, symbols: &SymbolEnv, snapshot: BuildSnapshotId, vc_id: VcId) -> Result<(ProofObligationIdentity, DependencySlice), String>` は status.rs の既存 return-canonical-input 全 doccomment/function の直前に置く単一 inline facade とする。
最初に既存 check_source_algorithm_types(source, check.typed(), symbols) を呼び、実際の String を保持する。以後はこの fresh opaque check を使用し、algorithm() の対応が入力 check と一致することを要求する。新しい対応拒否は内部 algorithm.proof.invalid_source_identity であり公開診断ではない。
続いて既存 source_algorithm_return_canonical_inputs(&fresh, snapshot, vc_id) を呼ぶ。実際の triple と元の raw slice 全体を保持し、producer/profile/target のエラーを後続の構文・identity 処理より先に伝播する。discharge 結果、書換 graph、呼出側 map/hash、代替 slice は受け取らない。
実際の source_algorithm_syntax_payloads_from_arena(source, symbols) から fresh.algorithm().1 の唯一の行を要求し、fresh check を lower して source_algorithm_core_semantic_payload(&fresh, &core) を呼ぶ。下位エラーを保持する。構文は元の terminating/header token、Core は検査済み guard/contract/body を担う。
fresh symbol に対応する実際の SymbolKind::Algorithm entry を唯一選び、実 module/namespace/primary spelling と Public visibility を使う。欠落・重複・非対応 owner は内部 identity 拒否とする。id/name 単独、Debug、opaque signature、新 producer から owner を作らない。
既存 CanonicalJson の STRING 六項目を構築する: entry.symbol の module から package_id/module_path、entry.namespace() から namespace、entry.primary_spelling() から spelling、実 enum 検査後の kind = algorithm と visibility = public。source id/range/allocation/local/FQN/contribution 座標を新しい owner 項目にしない。
canonical_json_bytes(owner)、変更しない構文/Core 全 bytes、元の VC/goal/context triple を [VC candidate producer](../../mizar-vc/ja/vc_ir.md#source-algorithm-return-candidate-anchor-hash) に一度だけ渡す。grammar/hash は VC、実 same-source operands は Proof が所有する。既存 ObligationAnchor は source-algorithm-return-candidate-anchor-v1: と返却 hash 全32byte の既存 source_discharge_hex から作る。
二つ目の既存 CanonicalJson は STRING 五項目のみ: obligation_anchor = candidate 文字列、vc_fingerprint = hex(vc.hash())、canonical_goal_hash = hex(goal)、local_context_fingerprint = hex(context.hash())、vc_local_dependency_slice_fingerprint = hex(元 slice.fingerprint().hash())。hex は全 bytes の小文字表現とする。
既存 CanonicalHashDomain(HashClass::Interface, mizar-proof/source-algorithm-return-original-obligation, SchemaVersion::new(1, 0)) を除外 path なしで使う。既存 ProofObligationIdentity::new(candidate文字列, candidate, このhash, vc.hash(), context.hash(), 元slice.fingerprint().hash()) を呼び、object/constructor エラーは to_string を保持する。identity と元 slice 全体を変更せず atomic に返す。
raw identity は受理、完全な source/context/dependency footprint、snapshot/filesystem currentness、terminating promotion/E4、algorithm 全体の検証、trusted witness、Artifact/publication、cache/reuse/Hit、services/LSP、Steps6–8 完了を与えない。raw Open/Deferred/incomplete anchors、非空 unknowns、IncompleteUncacheable/requires_cache_miss を保持し Accepted/Complete を作らない。
### Source algorithm return identity tests
既存 term_proof_tests 最終 brace の前に private with_source_algorithm_identity_inputs と三 family source_algorithm_return_obligation_identity_preserves_genuine_fields_and_slice、source_algorithm_return_obligation_identity_preserves_source_associations、source_algorithm_return_obligation_identity_preserves_upstream_errors_and_inputs だけを追加する。helper は三 family に実 call site を持ち、既存 source_registration_owner_inputs と runner.rs の neutral TypedArena projection を再利用して実 Checker を呼ぶ。架空の positive AST/env/check や旧 helper/test 変更は禁止する。
新 API より前に実 fresh check/Core 全 payload/Resolver syntax/current owner/旧 canonical-input tuple を得る。変更しない return fixture と terminating 除去 source について Rust 前に独立固定した candidate/final fingerprint、identity 六 getter、candidate/id 表示、slice 全体、反復性と入力・既存結果の不変性を照合する。raw 不完全性と cache miss を保持し、edit/snapshot 間の slice 全体の安定性を要求しない。
実再構築した Unicode trivia、実 SourceId 再配置、階層 path を含む package/module、owner/parameter 編集を使う。新 API 前に旧 operands を確認する: trivia/再配置は candidate 内容を保ち、terminating 除去と引数名変更は candidate operands 中で declaration syntax だけを変えて candidate を変える。package/module/owner も参加する。別 target snapshot は元 source のみに適用し identity 全体の安定性を要求しない。
実 foreign/augmented source-env では先に実 fresh Checker エラーを得て、未存在選択との組合せでも完全伝播を要求する。fresh check 成功後の absent-id/zero-VC/実 broader profile の存在 target について旧 canonical-input エラーと比較する。実 no-ensures two-return Core の generator エラーを先行させる。既存下位拒否 matrix/static review を再利用し架空 opaque check や重複 test を作らない。
実 baseline owner/syntax/Core/triple で raw VC producer の固定全 hash、三 empty-operand 拒否、反復性と不変性も検査する。raw hash の摂動は非信頼入力であり source receipt/意味的受理ではない。追加 helper/type/adapter/codec/dependency、corpus/expectation/trace/coverage credit の変更は禁止する。
