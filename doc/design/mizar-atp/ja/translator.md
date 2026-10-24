# Module: translator

> 正本は英語です。英語版:
> [../en/translator.md](../en/translator.md)。

## 目的

`translator` module は、validated `mizar-vc` `VcIr` obligation から
backend-neutral `AtpProblem` value への deterministic translation を所有する。
translation は依然として candidate-production setup である。concrete TPTP、
SMT-LIB encoder、backend process、portfolio policy、kernel check より前に、既に
利用可能な VC formula、declaration、soft-type fact、provenance、target binding を
problem layer に materialize する。

translator は VC を証明しない。ATP backend、SAT solver、`mizar-kernel` を呼ばず、
trusted acceptance material を生産しない。

## 境界規則

translator が行ってよいこと:

- validated `VcSet`、`VcStatus::NeedsAtp` の 1 つの `VcIr`、対応する VC
  kernel-evidence handoff target binding を消費する。
- caller-supplied profile policy と VC shape から explicit な backend-neutral
  `LogicProfile` を選ぶ。
- VC formula reference を structured `AtpFormulaTree` payload に変換する。
- 既に利用可能な local context fact、explicit `PremiseRef` entry、generated VC fact、
  imported fact payload、type fact、encoded property input だけを materialize する。
- canonical dense id、declaration、symbol-map row、provenance row、non-semantic
  diagnostic を導出する。

translator が行ってはならないこと:

- `VcIr` / handoff input の外から premise を追加選択する。
- substitution を発明する、binder を修復する、overload resolution を行う、cluster を
  探索する、implicit coercion を挿入する、fallback policy で definition を unfold
  する、unsupported formula を近似する。
- proof hint、backend log、backend proof trace、backend-reported `used_axioms`、
  resolution trace、SMT proof object、instantiated formula、SAT problem を trusted
  problem field として扱う。
- backend output を accepted proof status として分類する、または proof witness、
  cache entry、artifact status を発行する。

## 入力

Translator construction は概念的に次の immutable input bundle を消費する:

```text
AtpTranslationInput
  vc_set
  vc
  kernel_handoff
  logic_profile
  formula_projection_table
  declaration_projection_table
  soft_type_projection_table
  imported_formula_payloads?
  property_inputs?
  diagnostics_policy?
```

`vc_set` と `vc` は ATP 対象の `NeedsAtp` obligation を識別する。`kernel_handoff` は、
後続の kernel evidence candidate が一致しなければならない validated target fingerprint と
formula/provenance material を供給する。`logic_profile` は explicit である。translator は
VC shape に対して profile を unsupported として reject してよいが、translation を成功させる
ために profile を黙って変更してはならない。

structured formula / declaration / soft-type projection table は、`AtpFormulaTree` payload、
declaration kind / arity、binder / sort relationship、soft-type guard の唯一の source である。
これらはより前の deterministic phase が `mizar-core` / `mizar-vc` owned data から生成し、
`VcFormulaRef`、context identity、declaration identity、または provenance identity で
key 付けしなければならない。kernel handoff の formula fingerprint と payload byte は
agreement check と provenance anchor であり、translator が ATP formula tree や declaration を
復元するために parse してはならない。materialized formula row について、`AtpProblem`
provenance payload は deterministic な handoff-derived anchor であり、caller-supplied
projection payload byte そのものではない。

structured projection の欠落は、実装される translator task では fail-closed producer input error
である。translator は opaque kernel formula byte、debug rendering、backend text、display name、
source text、name resolution、overload resolution、implicit-coercion insertion、cluster search、
substitution invention から formula、declaration、type guard を復元してはならない。

source `VcIr` は `NeedsAtp` status でなければならない。`Open`、`Discharged`、
`AssumedByPolicy`、`PolicyOpen`、`SkippedDueToInvalidInput`、`DeferredExternal`、`Error`
obligation は、task-5/6 translator source によって ATP problem に変換されない。それらは
deterministic discharge、policy、diagnostic、または後続 external integration の所有である。

## target binding

translator は、供給された VC handoff が同じ `VcSet` 内の同じ `VcId` を対象にしていることを
要求しなければならない。不一致は `AtpProblem` construction の前に fail closed する。

結果の `AtpTargetBinding` は、handoff target fingerprint と VC/handoff source を名付ける
stable producer binding から導出される。snapshot-local な `VcId` は collation のために
`AtpProblem` semantic identity に参加してよいが、それ自体は stable target binding でも
proof-reuse key でもない。

## deterministic translation order

同等の validated input は byte-identical な `AtpProblem` debug rendering と同じ
problem id を生成しなければならない。`AtpProblem` construction の前に、次の決定的順序を
適用する:

- profile selection は caller-explicit であり、stable profile name を持つ。
- declaration は canonical source / generated identity、次に declaration kind と arity
  で key 付けする。
- generated binder declaration は display spelling や traversal order ではなく、
  canonical binder context で key 付けする。
- premise formula は、source class と stable source binding を検証した後の canonical sorted
  `VcIr` premise order に従う。duplicate premise reference、duplicate source / formula
  identity、または repeated imported source tuple は problem construction の前に fail closed
  する。
- type guard と soft-type fact は canonical context / source identity に従う。
- encoded property は stable property identity と target symbol に従う。
- symbol-map row と provenance row は canonical key に従う。
- diagnostic は debug output のためだけに sort し、semantic identity には参加しない。

translator は map iteration、pointer address、backend output order、source range 単体、
display spelling 単体、wall-clock time、random state、process id を ordering key として
使ってはならない。

## formula materialization

`VcFormulaRef::Core` と `VcFormulaRef::Generated` は、structured formula projection table
からのみ、backend text ではない structured `AtpFormulaTree` value に lower しなければならない。
translator は次を lower してよい:

- selected `LogicProfile` が support する constant、atom、equality、connective、
  quantifier。
- conjunction、split goal、negation、implication、quantified wrapper などの generated
  formula shape。ただし参照 payload がすべて存在する場合に限る。
- diagnostic / error formula reference は fail-closed translation error としてだけ扱い、
  何かを証明する axiom にしない。

参照 formula payload が missing、malformed、selected profile で unsupported、または
alpha-repair / substitution invention を必要とする場合、translator は failure class に従って
unsupported/open translation result または fail-closed error を返す。formula を落とす、
`true` に置き換える、近似する、obligation を accepted status に移すことはしない。

## premise materialization

`vc.premises` は task-6 axiom translation の authoritative premise list である。各 premise は、
参照 formula payload と provenance が存在し、selected profile に対して valid な場合だけ
axiom になる。

translator は duplicate premise reference、duplicate source / formula identity、repeated
imported source tuple を、黙って coalesce するのではなく reject する。duplicate premise は
provenance と後続 backend `used_axioms` identity を曖昧にするため、fail-closed producer input
error である。

許可される premise source:

- explicit formula payload を持つ `PremiseRef::LocalContext` entry。
- `PremiseRef::GeneratedFact` が参照する generated VC fact。
- imported payload、statement fingerprint、required proof status、formula context
  requirement が VC/handoff input から供給される imported fact。ATP projection symbol は
  imported source tuple の代替ではない。異なる imported symbol から来た場合でも、
  package/module/item/status/statement tuple の重複は reject する。

task-29 post-audit kernel contract では、非 import premise projection は VC handoff の
source identity を保持しなければならない。local hypothesis、cited premise、generated
VC fact の projection source binding が ATP producer により relabel されている場合、
translator はそれを新しい identity に修復せず、`AtpProblem` や backend candidate の
construction 前に fail closed する。Imported fact projection は引き続き VC handoff 経由で
供給された kernel-validated statement projection payload に依存する。ATP translator はその
payload を symbol label、backend axiom label、backend `used_axioms` record で置き換えない。

checker-owned fact と type predicate は future conditional な premise family である。
task-6 translator source は、VC handoff が対応する explicit source class/projection を公開する
までそれらを fail-closed に保つ。generic な policy-builtin bucket だけでは、ATP translator が
checker-owned または type-predicate provenance を名乗るには不十分である。

unsupported premise reference、conservative unknown marker、policy assumption、
trace-only record、cluster/reduction trace label、definition unfolding policy marker、
proof-hint citation は、それ自体では premise ではない。より前の VC phase が explicit
formula/provenance payload を materialize し、その payload を `vc.premises` または handoff
data に置いた後に限り axiom になり得る。

Proof hint は後続の backend/profile/portfolio choice を制約してよいが、translator が
`vc.premises` を追加、削除、prune する許可にはならない。`Only` / `Exclude` restriction は、
より前の immutable VC phase が既に `vc.premises` に反映していない限り、
diagnostic / profile / portfolio metadata のままである。task-5/6 translator が proof-hint
restriction と immutable premise list を整合できないと観測した場合、diagnostic または
unsupported/open translation outcome を出してよいが、premise set を mutate してはならない。

## Planned Premise Selection Boundary

予算に応じた選択は [VC 所有の境界](../../mizar-vc/ja/vc_ir.md#planned-pretranslation-selection-assembly) で組み立てる。translator は渡された immutable input を消費し、source premise を追加・削除・prune しない。

## goal と polarity

VC goal は `AtpProblem.conjecture` formula に変換される。translator は task-5/6 の全 problem に
`ExpectedBackendResult::Unsat` を記録する。task-6 source は、対応する VC kernel handoff の
final-goal polarity が `AssertFalseForRefutation` であることも要求する。他の handoff polarity は、
`AtpProblem` construction の前に fail closed する。goal projection provenance binding と
projection source identity は final-goal source binding `goal:1` を使わなければならない。
premise-style generated binding、checker-owned binding、local binding は conjecture には使えない。
Task 29 は `AssertTrueForConsistency` を要求する proof-obligation handoff をこの boundary の
non-candidate condition として扱う。ATP は proof obligation を consistency check として
解釈し直さない。concrete encoder は後で goal を TPTP
conjecture、negated TPTP conjecture、または negated goal の SMT assertion として提示してよいが、
backend-neutral problem contract は、successful backend result が premise と negated goal の
unsatisfiability に対応しなければならない、というままである。

goal を `axioms`、backend-reported `used_axioms`、trusted acceptance material にコピーしては
ならない。

## declaration と symbol map

formula、property、generated binder、type guard、native property declaration が使う
backend-visible symbol はすべて次を持たなければならない:

- 一意な `AtpDeclaration`。
- 各 use と一致する kind / arity。
- backend-safe symbol と canonical Mizar/core/generated identity を結び付ける
  `AtpSymbolMapEntry`。
- candidate evidence に影響し得る generated declaration または fact の provenance。

concrete encoder の name mangling は TPTP / SMT-LIB encoder spec に deferred する。
translator は backend-neutral canonical symbol と、後続 diagnostic / candidate-evidence extraction
を traceable にする mapping だけを所有する。
declaration kind、arity、binder / sort relationship、canonical source identity は、display name や
backend text ではなく structured declaration projection table から来る。

duplicate declaration、missing declaration、missing symbol-map row、kind/arity mismatch、
noncanonical dense id derivation は problem construction の前に fail closed する。

## soft-type preservation

local context、generated type obligation、type predicate、non-emptiness fact、sethood fact、
subtype/coercion fact、guard、intersection-like fact 由来の soft type information は、sort-only
encoding の選択によって消してはならない。

translator は、selected `LogicProfile` が関連する type information を lossless に表現すると
記録している場合だけ backend sort を使ってよい。sort で表現されない soft-type fact は、
guard predicate、explicit axiom、または provenance 付き type-context entry として残さなければ
ならない。preservation を selected profile で表現できない場合、その profile では translation は
unsupported/open である。

soft-type guard と preservation requirement は structured soft-type projection table から来る。
translator は symbol spelling、backend sort convention、kernel handoff byte からそれらを推論しては
ならない。

## provenance

candidate に影響し得る declaration、axiom、conjecture、type guard、encoded property、
native property declaration はすべて、VC/handoff input 由来の provenance を持たなければならない。
provenance record は source binding と stable projection payload を持つ。後続 producer task は、
owner が fingerprint field を公開したときにそれらの payload を fingerprint に refine してよい。
backend log や trace explanation ではない。

imported fact は package/module/item identity、statement fingerprint、required proof status、
formula context requirement を必要とする。task-6 source では formula-context requirement は
VC handoff formula-context fingerprint に binding される。imported context の missing または
mismatch は fail closed する。

## failure semantics

translator failure は producer-side open-status または fail-closed diagnostic であり、proof result ではない。

- malformed または missing の required VC/handoff payload は fail closed する。
- missing structured formula / declaration / soft-type projection は fail closed する。
- stale target binding、wrong VC、duplicate id、duplicate symbol、duplicate premise identity、
  provenance gap は fail closed する。
- unsupported formula/profile feature は、その profile に対する unsupported/open translation outcome を生む。
- profile unavailable、disabled ATP policy、backend configuration absence は後続 backend / portfolio task
  のために記録され、accepted proof status を構築しない。

## public enum forward compatibility

task 22 は frontend task 25 の方針を `translator` module に適用する。この module が所有する
public enum は downstream crate 向けに `#[non_exhaustive]` とする:
`AtpSymbolSourceProjection`、`AtpSoftTypeRepresentation`、`AtpFormulaProjectionTarget`、
`AtpTranslationError`。

Public enum inventory: `AtpSymbolSourceProjection`, `AtpSoftTypeRepresentation`, `AtpFormulaProjectionTarget`, `AtpTranslationError`.

将来 variant は、source が依存する前に仕様化しなければならない。`mizar-atp` 内部では、
premise selection、formula provenance、handoff agreement、backend-visible formula、proof
status に影響する match は、paired spec が意図的 fallback を記録しない限り、明示的に保ち
fail closed しなければならない。

## gap 分類

- resolved `deferred`: task 4 が translator ownership と boundary を仕様化する。
- resolved `source_drift`: task 5 は declaration / soft-type payload 用の Rust projection input
  struct を定義し、target/status handoff gate、deterministic declaration / symbol-map /
  type-context translation、final `AtpProblem` construction を伴わない type-guard signature
  validation を実装する。
- resolved `source_drift`: task 6 は VC formula ref と imported fact 用の structured Rust
  formula projection target を定義し、deterministic axiom/conjecture materialization を実装し、
  `ExpectedBackendResult::Unsat` を記録し、final-goal handoff polarity が
  `AssertFalseForRefutation` であることを検査し、duplicate premise ref と duplicate resolved
  formula/source identity を reject し、projection fingerprint / provenance payload を対応する
  VC kernel handoff と照合する。この照合では handoff formula byte を parse しない。
- `deferred`: checker-owned と type-predicate premise materialization は、VC handoff が対応する
  explicit source class/projection を公開するまで fail-closed のままである。`mizar-atp` は
  placeholder source class を発明してはならない。
- `deferred`: property encoding、concrete encoder、backend runner、portfolio、candidate evidence
  extraction はそれぞれの module spec / task に残る。
- `external_dependency_gap`: proof-policy winner selection、witness publication、cache promotion は、
  owner crate が stable contract を定義するまで `mizar-atp` の外に残る。

## planned tests

task 5/6 implementation は合わせて Rust coverage を追加し、次を確認する:

- non-`NeedsAtp` VC と stale/mismatched target handoff の rejection。
- missing structured formula / declaration / soft-type projection が fail closed すること。
- malformed formula / declaration / soft-type projection が required shape、declaration
  kind/arity、provenance invariant に違反する場合に fail closed すること。
- unsupported formula/profile feature と alpha-repair または substitution invention を必要とする
  formula が、explicit profile に対する unsupported/open outcome になり、profile を黙って
  切り替えないこと。
- shuffled equivalent local context / generated formula input に対する deterministic declaration と
  symbol-map output。
- duplicate、missing、kind/arity-mismatched declaration の fail-closed。
- duplicate premise reference または duplicate source / formula identity が fail closed すること。
- proof hint と premise restriction がそれ自体で premise を追加、削除、prune しないこと。
- required proof status、statement fingerprint、formula-context requirement が missing の
  imported fact が fail closed すること。
- local context、generated、imported fact の premise-order determinism と provenance completeness。
  checker-owned と type fact は、VC handoff が対応する explicit source class を公開するまで
  fail-closed のままにする。
- soft-type fact が lossless な場合だけ sort として表現され、それ以外は guard / axiom /
  type-context entry または unsupported profile result として残ること。
- goal/conjecture polarity が常に `ExpectedBackendResult::Unsat` を記録すること。
- translator public API と debug rendering に prohibited backend/kernel/SAT/proof-acceptance
  material が存在しないこと。

## Source registration projections

`translate_source_existential_registration(&CoreIr, &VcSet, VcId, LogicProfile) -> Result<AtpProblem, String>` は既存の単一・3登録 profile の実 structured projection を供給する。選択 VC の `NeedsAtp` と source handoff による全 generator/default-discharge replay を要求し、最終検証と構築は `translate_problem` に委ねる。
実 Core term/predicate/equality/builtin-set guard と VC Ref/Not/And/Or 木を読む。選択 premise/goal を全て保持し、未対応 node は fail closed。attribute predicate と閉じた stable-choice function は完全な Core symbol identity と役割別 prefix、builtin set は別の unary predicate で命名する。Debug は既存 identity/provenance の命名だけに使い、式内容や numeric kernel name を復元しない。
共有 definition formal は検証済みの実 Core parameter で、existential/nonempty binder と区別する。出現を構造的に追跡し、その formal を含む premise だけ一つの unsorted ATP binder で全称閉包し、実 type guard を保持する。ground premise は変更せず、自由変数の conjecture は拒否する。これは [§§9.9.3–9.9.4](../../../spec/en/09.predicates.md#993-predicate-definitions-as-bi-conditionals) の引数付き引用 definition の表現であり、premise や任意 substitution を追加しない。
Declaration kind/arity/source binding は実 definition/origin/guard から得る。式 row は実 producer ref と正確な handoff fingerprint/provenance anchor を保持し、bytes は一致確認の metadata のみとする。caller は指定 profile を供給する。この操作は一致する producer-owned Core/VC input から structured projection を構築し、選択 premise/status を保持する。指定 profile と既存 encoder gate を保持する。source 入口は GuardPredicates と Fof/Tptp または SmtLibUninterpreted/SmtLib の対応 format flag を要求し、他の fragment/soft-type の組合せは局所的に拒否する。追加 format flag は保持し、equality/quantifier の要件は既存 tree validation に委ねる。
型 context の GuardFormula は選択された ground generated choice-type premise そのもので、実 handoff source/provenance anchor を保持する。同じ選択 axiom を残し、この必要な表現は別の fact を追加しない。独立した formal 型 guard の全称主張や架空の tautology を出さない。各 concrete encoder は GuardPredicates を含む対応する caller profile で検証し、reprofile しない。
テストは実 Frontend/Resolver/Checker/Core/VC/default discharge と両既存 encoder を通す。一般 source extraction、active ATP runner、instance search、汎用 evidence transport、backend 実行、受理は deferred のままとする。

## Source theorem scoped sequent

translate_source_theorem_sequent(core: &CoreIr, original: &VcSet, vc_id: VcId, discharge: &DischargeOutput, policy: &DischargePolicy, logic_profile: LogicProfile) -> Result<AtpProblem, String> は最初に [configured VC handoff](../../mizar-vc/ja/kernel_evidence_handoff.md#source-theorem-configured-discharge-handoff) を呼び、その error を保つ。inline producer 一つを加え、Checker/parser/backend/Kernel 実行や全 profile validator の複製はしない。
次に ScopedGuardPredicates/FirstOrder/Supported equality と、Fof+Tptp flag または SmtLibUninterpreted+SmtLib flag を要求する。追加 format/native-property capability を含む要求 profile 全体を保ち、変更しない。新 profile/projection 拒否は内部 theorem.atp.invalid_projection、既存 target/problem constructor error は Display String とする。
汎用 translate_declarations では VC lookup/status/handoff 一致検査後、target/projection 構築前に新 mode を既存 UnsupportedProfileFeature(scoped-guard generic translation) で明示拒否する。translate_problem もその入口を通る。helper の新 mode arm も明示 error とする。旧 mode・汎用 premise materialization・source registration を保つ。

実 processed set の単一 Local ProofAssumption/premise と goal、その Core TypePred(set)/Equals/Var tree、および新鮮な handoff の producer ref/source/provenance-ID/fingerprint/target の一致を消費する。opaque byte の解析や元 universal binder の射影はしない。実共有 proof-let CoreVar を表現 local:0 に対応付け、predicate type:set は MizarSymbol(builtin-type:set)、binder local:0 は GeneratedBinder(proof-let:<実 CoreVar index>) とする。関連は範囲付き data であり大域 identity や生成元証明ではない。
既存 AtpProblemParts を [限定 problem mode](./problem.md#scoped-guard-problems) で直接作る。declaration0 Predicate/arity1/provenance0、declaration1 GeneratedBinder/arity0/provenance1、conjecture formula0/provenance2、external axioms/type guards/properties/diagnostics は空、期待結果は Unsat。tree は厳密に Forall([local:0, sort なし], Implies(実 guard(local:0), 実 equality(local:0,local:0)))。既存 target_binding helper を実 processed handoff に使う。

消費される provenance は三 row。0 LocalHypothesis(local-context:<実 context index+1>) は predicate declaration、1 CheckerOwnedFact(proof-let:<実 CoreVar index>) は生成 declaration metadata、2 GeneratedVcFact(goal:1) は合成 conjecture に使う。binder row は admitted Core declaration を記述し、受理 premise ではない。external axiom が空でも実 source premise 一つを保持し、pruning/budget は主張しない。
各 payload は LF なしの厳密な UTF-8 mizar-atp/source-theorem-sequent/v1;source=H;module=H;binder=D;target=D:H;handoff=H;context=H;guard-ref=H;goal-ref=H;guard-fingerprint=D:H;goal-fingerprint=D:H;guard-provenance=D:H;goal-provenance=D:H;role=R。H は小文字 hex。source/module/ref は実 Debug UTF-8、target/formula は実 fingerprint digest、handoff/context は実 canonical/context hash、provenance は対応する実 opaque payload を符号化する。D は箇所に応じ実 CoreVar index/algorithm id/provenance id の符号なし十進数、R は row0/1/2 に guard/binder/closure。順序と両 primitive の関連を保ち、raw goal fingerprint を閉じた式の fingerprint と称しない。
新 hash/type/field/helper/codec、架空 guard/constant/source evidence は加えない。全 discharge 一致は値の関連付けであり、異なる policy の真正な等出力を許す。currentness、candidate search/backend/Proof/Artifact/cache/services、汎用 ATP 完了は未完了。Kernel evidence の元 local formula/target を置換しない。

既存 problem-owner に shape/refusal 一 family、runner/proof_verification.rs に真正 source 二 family を追記する。Rust 前に独立した完全 tree/declaration/payload grammar と完全 TPTP/SMT text を固定し、実両 encoder/EncodedBackendProblem の target/label/symbol/source、ground term なし、反復・入力不変、実 source/owner/alpha/trivia/SourceId/snapshot/schema/profile/policy variant を検査する。因果的 upstream/foreign/discharge/absent/profile 拒否、等出力 policy、empty/GuardFormula projection での汎用入口拒否を検査し、旧 test/mode/registration を保つ。backend 成功や新 proof 受理は主張しない。
