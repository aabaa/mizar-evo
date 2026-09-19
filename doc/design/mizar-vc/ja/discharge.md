# Module: discharge

`failed_functorial_coherence` は Core・VC・ソース・証明の対応と、前提ラベル・proof hint・前提の不在を照合し、既存の否定反射等式による関数型登録の失敗、または限定された簡約の失敗を観測する。
簡約では実際の透過的な恒等関数適用と同じ束縛の単元集合から `forall X:set. X={X}` が得られ、ガードは bare set のみ、追加仮定は存在しないことを要する。builtin の空集合とその空でない単元集合がこの全称等式を反証する（仕様 §§13.4.1,13.8.6,17.3.4,17.9.4）。
内部の意味上の証人は実際に失敗した VcId の観測にのみ用い、ソースの証人・反例成果物・受理済み事実・kernel 根拠を生成しない。反射的な右辺は失敗にならず、未対応または NeedsAtp の目標も失敗とは扱わない。

`failed_source_algorithm_assertion(&CoreIr, &VcSet) -> Result<Option<VcId>, String>` は source algorithm generator の完全な結果を再生成・照合し、実際の不変 object 仮引数一つに対する Open な assertion `not a=a` を観測する。元の Core Not/Equals graph、両出現、変数・書込み世代、FlowDerived algorithm/statement 所有者、唯一の整合した仮引数型ガード文脈を認証する。
この矛盾だけが実 VcId を返し、通常の等式は失敗にせず open に留める。局所変数・書込み・契約・未証明 assertion 仮定・hint・偽の前提・無関係なエラー・改変した accounting は assertion 失敗の根拠にならない。status 変更、一般的な Unknown/NeedsAtp から失敗への変換、受理証拠、Task274、terminating 昇格は導入しない。

> 正本は英語です。英語版:
> [../en/discharge.md](../en/discharge.md)。

## 目的

この module は `mizar-vc` phase 12 の deterministic pre-ATP discharge を仕様化する。
generation、normalization、status-policy projection 後の canonical `VcSet` data を消費し、
各 concrete VC を deterministic な Mizar 側 rule で discharged、明示的な policy status のまま、
または `NeedsAtp` として変更なしで転送、のいずれかに分類する。

Task 10 は仕様のみである。Rust discharge code は task 11 と task 12 が実装する。
この文書は architecture 07 と 08 を精緻化するものであり、language semantics、`.miz`
fixture、expectation、`doc/spec`、traceability metadata は変更しない。

## 責務

この module が所有するもの:

- deterministic pre-ATP discharge rule selection;
- replayable で untrusted な discharge evidence record;
- discharged VC と not-discharged VC の human-readable explanation;
- computation-limit と definition-unfolding policy boundary;
- full context と `NeedsAtp` status を持つ ATP-bound VC の保持。

範囲外:

- ATP translation、ATP portfolio scheduling、backend configuration;
- kernel proof acceptance、certificate validation、proof artifact publication;
- proof/cache reuse decision と dependency-slice fingerprint;
- source syntax reconstruction、新しい VC generation、新しい algorithm payload family;
- explicit upstream trace data が欠ける registration、cluster、reduction、computation result の受理。

## この仕様の gap 分類

| ID | 分類 | 証拠 | 扱い |
|---|---|---|---|
| DIS-G001 | `spec_gap` | task 10 より前に `discharge.md` は存在せず、tasks 11-12 には phase-12 contract が必要である。 | Task 10 は英語/日本語 discharge spec だけを追加する。 |
| DIS-G002 | `source_drift` / `test_gap` | task 11 より前には `src/discharge.rs`、`pub mod discharge`、lint-policy coverage、task-11 discharge API、focused engine tests が存在しなかった。 | Task 11 は `VcIr` に既に表現された explicit class だけを対象に、これらの source/module/test surface と最小の stable `DischargeEvidenceRef` を追加する。Task 12 が replayable evidence と explanation serialization を拡張する。 |
| DIS-G003 | `external_dependency_gap` / `deferred` | Task 10 は当初 ATP/kernel/proof/cache/corpus consumer を unwired と扱っていた。`mizar-kernel` は現在 corrected formula/substitution evidence checking を所有するが、`mizar-vc` にはまだ task-25 handoff builder と downstream ATP/proof/cache/artifact consumer がない。 | prover-independent status、untrusted evidence、deferred downstream integration point だけを記録する。Discharge rule name や evidence hash は trusted kernel acceptance material ではない。 |
| DIS-G004 | `external_dependency_gap` | 一部の type、cluster、registration、reduction、computation trace は、すべての VC について explicit upstream payload としてまだ利用できない場合がある。 | Discharge は `VcIr` に既に存在する explicit fact、premise ref、proof hint、policy input だけを使う。trace が欠ける場合は silent discharge ではなく explanation 付きで VC を `NeedsAtp` または deferred に保つ。 |
| DIS-G005 | `deferred` | artifact serialization、dependency-slice fingerprint、corpus fixture、VC kernel-evidence handoff / reuse integration、proof/cache validation は後続 task または crate が所有する。 | Task 10 は必要な shape と invariant を定義する。task 11 は engine default limit を記録する。task 12 は `DischargeOutput` 上の in-memory replayable evidence/explanation record を追加し、artifact/dependency/downstream consumer work は deferred のままにする。 |

## 入力と出力

必須入力:

- stable `VcId`、seed accounting、local context、premise、goal、proof hint、anchor、status を持つ validated `VcSet`;
- context entry または premise ref として既に存在する explicit local type、sethood、non-emptiness、cluster、registration、reduction、checker fact;
- `ProofHint` の definition unfold request と computation hint;
- discharge、computation limit、ATP dispatch を制御する verifier policy input。

必須出力:

- 入力 `VcSet` と同じ VC order と seed accounting;
- deterministic rule が replayable evidence を生成した場合だけの `Discharged` status;
- external search が必要な VC の `NeedsAtp` status;
- 保持された `PolicyOpen`、`AssumedByPolicy`、skipped、deferred、error status;
- discharged VC、および rule、trace、limit が利用不能で `NeedsAtp` または deferred のままの VC に対する deterministic explanation。

Discharge は VC を削除したり、goal を弱めたり、local context を落としたり、seed accounting を書き換えたり、
VC を並べ替えたり、ATP-bound obligation を missing record に置き換えたりしてはならない。

## Supported Discharge Classes

Tasks 11-12 は、必要な fact と trace がすべて explicit な場合だけ discharge してよい:

- canonical goal と local premise 上の syntactic tautology / contradiction check;
- reflexivity、direct equality normalization、alpha-stable formula identity check;
- local context に既に存在する type、sethood、non-emptiness fact;
- explicit replayable trace または premise reference と、goal に結びついた explicit generated/local fact を持つ cluster、registration、reduction fact;
- VC の unfold request と policy input が許し、reduction boundary 後の goal に結びついた explicit generated/local fact を持つ definitional reduction;
- requested computation、limit policy、goal に結びついた explicit computation result fact が存在する場合の bounded `by computation` または verification-time computation。

Unsupported または unavailable な class は negative evidence を生成してはならない。既存 status と発見した理由に従って、
VC を `NeedsAtp`、`PolicyOpen`、`AssumedByPolicy`、`DeferredExternal`、または `Error` に保つ。

## Limit Model

computation-based discharge はすべて deterministic limit tuple によって制御される:

- limit source を表す policy key;
- gas または step budget;
- optional な wall-clock-independent fuel class;
- nondeterministic timing の source ではなく stable policy identifier としてだけ使う timeout label;
- その VC で active な definition-unfolding / reduction policy。

Task 10 はこの shape を固定するが、numeric default は選ばない。Task 11 は engine default
policy key を `task-11-computation-step-limit`、`max_steps = 64` とする。呼び出し側は別の
deterministic `DischargePolicy` を与えてよい。`LimitPolicy` computation hint は active policy
key と一致しなければならず、`ByComputation` は active policy を直接使う。limit exceeded は
failed proof ではない。VC は stable explanation 付きで `NeedsAtp` または deferred のままになり、
`Discharged` evidence は持たない。

## Evidence And Explanations

Discharge evidence は untrusted production evidence である。pre-ATP status、diagnostics、
後続 proof/cache decision を支えてよいが、kernel-accepted proof ではない。

Task 11 は `VcIr` に既に存在する最小の status evidence だけを保存する:
`DischargeEvidenceRef` 内の deterministic rule name/version と stable evidence hash である。
選択された rule は、保持された `VcIr` input、すなわち local context entry、premise ref、
proof-hint citation、unfold request、computation hint、policy input、generated formula ref、
変更されない goal だけに依存してよい。trace、unfold、computation marker だけでは discharge
evidence にならない。同じ goal に対する explicit generated/local fact と結びついていなければならない。

Task 12 はこれを `DischargeOutput` から公開される in-memory replayable evidence record へ拡張する。
それは deterministic に render/clone でき、同じ pass が生成する stable な not-discharged explanation
と対応しなければならない。この record は次を含まなければならない:

- discharged された `VcId`;
- deterministic rule name と version;
- rule が使用した input formula ref、local context ref、premise ref、generated formula ref;
- 関連する policy key、unfold request、computation hint、limit tuple;
- rule が使用した cluster、registration、reduction、computation trace すべてに対する replay data または premise ref;
- dependency-slice と artifact record に使える stable evidence hash。

`DischargeOutput` 内の `Discharged` VC はすべて、VC status の evidence と一致する evidence record
を持たなければならない。`NeedsAtp`、`PolicyOpen`、`AssumedByPolicy`、skipped、deferred、error
のままの VC は explanation を持つが、discharged evidence として扱ってはならない。artifact
serialization、persistence、kernel-side replay validation は task 12 の範囲外に残す。

入力 `VcSet` に既に `VcStatus::Discharged` が含まれる場合、task 12 はその status を保持し、
欠けている replay data を捏造するのではなく preserved-evidence record を記録する。この preserved
record は VC、既存の rule name、evidence hash を識別し、evidence source が pre-existing input
status であること、および詳細 replay data をこの pass が再構築していないことを説明しなければならない。
この record は status preservation と diagnostics のためだけに有効であり、新しく生成された proof
evidence ではない。

Task 20 は newly produced deterministic evidence hash を cross-edit stable にする。hash input は
deterministic rule と canonical VC / local-context fingerprint であり、snapshot-local な `VcId`
ではない。Preserved / pre-existing discharged status record は proof evidence を合成せず、
cross-edit proof reuse key の対象にならない。どちらかの canonical fingerprint が利用不可なら、
evidence hash marker は conservative unknown になり、その VC は Task 20 の proof-reuse
candidate key の対象にならない。

unavailable trace marker は not-discharged explanation、または selected rule が使用しなかった
trace class に対してのみ現れる。

各 not-discharged VC は `needs_atp`、`policy_open`、`assumed_by_policy`、`missing_trace`、
`limit_exceeded`、`unsupported_rule`、`deferred_external`、`error` などの explanation category
を記録しなければならない。これらの category は diagnostic data にすぎず、VC を消してはならない。

## Status Interaction

Discharge は evidence が存在する場合に限り、`Open` または `NeedsAtp` VC を `Discharged` に変えてよい。
deterministic rule が適用できない場合、それらを `NeedsAtp` のままにしてよい。Policy status は明示的に残す:

- `PolicyOpen` は discharge evidence ではなく ATP に送られない;
- `AssumedByPolicy` は受理された assumption marker であり、proof evidence ではない;
- skipped、deferred、error status は可視のままで dispatch されない;
- `Discharged` evidence は kernel proof acceptance ではない。

ATP translation の対象になるのは `NeedsAtp` status を持つ canonical VC だけである。それらは source ref、
local context、premise、proof hint、anchor、seed accounting、元の goal を保持しなければならない。

## Determinism And Ordering

Discharge order は ascending `VcId` である。Rule selection は hash-map iteration、worker completion
order、local absolute path、backend availability、wall-clock time、nondeterministic resource
measurement に依存してはならない。

Discharged、policy-status、deferred/error、`NeedsAtp` output list は入力 VC order を保持する。
同じ source range を持つ diagnostic は `VcId`、stable rule name、stable diagnostic category の順に並べる。

## Planned Tests

Task 11 は Rust coverage として次を追加しなければならない:

- tautology、contradiction、reflexivity、direct equality-normalization fixture の stable discharge;
- global inference なしで explicit type/sethood/non-emptiness fact による discharge;
- replayable trace または premise ref が存在する場合の explicit cluster、registration、reduction trace による discharge;
- unfold policy が許す場合だけの definitional reduction discharge;
- computation input と limit policy が explicit で limit を超えていない場合の bounded computation discharge;
- limit exceeded computation が wrong answer ではなく stable explanation 付きで `NeedsAtp` または deferred になること;
- no-erase ATP boundary: unsupported rule が full VC context と `NeedsAtp` status を保持すること;
- repeated run をまたいだ deterministic output order。

Task 12 は Rust coverage として次を追加しなければならない:

- 各 discharged VC の evidence record;
- deterministic に render される evidence hash と explanation category;
- discharge evidence として扱われない policy status;
- missing trace data が discharge ではなく fail closed になること;
- discharge をまたいだ seed accounting、anchor、proof hint、generated formula ref の保持。

## public enum policy

task 17 は `discharge` の public enum をすべて downstream forward-compatible API surface
として分類する。後続の evidence source、replay mode、explanation category、deterministic
discharge rule を downstream の exhaustive match を壊さず追加できるよう、各 enum は
`#[non_exhaustive]` を維持しなければならない。

| public enum | decision |
|---|---|
| `DischargeEvidenceSource` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `DischargeEvidenceReplay` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `DischargeExplanationCategory` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `DischargeRule` | `#[non_exhaustive]` downstream forward-compatible surface。 |

この module が所有する exhaustive public enum exception はない。現在の variant を意図的に
列挙する `mizar-vc` 内部 match は exhaustive のままでよい。

## 限定 Pick 事後条件失敗

既存 `failed_source_algorithm_assertion` の signature を限定 return-Pick 事後条件にも使い、assertion 分岐を保持する。generator 結果全体の厳密な replay、実 runtime set Pick、真正な inhabitation 義務、完全な object(a), set(p) 文脈だけが Open 目標 p=a の判定を許す。仕様の集合論的基礎で a={}、p={{}} とすると、両者は set かつ object であり、{} は要素を持たず {{}} は {} を含むため外延性から p≠a が従う。この許された入力・選択の組は Pick 契約の全称保証を反証し、全実行の失敗を主張しない。有限宇宙モデルでも異なる ID からの推論でもない。実際の事後条件 VcId のみを返し、両 VC は Open に保つ。追加仮定・hint、目標/accounting の改変、stable choice、未対応入力、timeout、NeedsAtp は失敗を成立させない。改名 replay、return-a と反射等式 ensures の非失敗、整合的な source/Core/VC/context/accounting 破損の拒否を検査し、汎用反例 interface や kernel acceptance は追加しない。
