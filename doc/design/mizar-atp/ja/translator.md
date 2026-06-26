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
復元するために parse してはならない。

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
  `VcIr` premise order に従う。duplicate premise reference または duplicate source / formula
  identity は problem construction の前に fail closed する。
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

translator は duplicate premise reference と duplicate source / formula identity を、黙って
coalesce するのではなく reject する。duplicate premise は provenance と後続 backend
`used_axioms` identity を曖昧にするため、fail-closed producer input error である。

許可される premise source:

- explicit formula payload を持つ `PremiseRef::LocalContext` entry。
- `PremiseRef::GeneratedFact` が参照する generated VC fact。
- VC または handoff input に explicit formula payload / provenance を既に持つ
  checker-owned fact、type predicate、generated fact。
- imported payload、statement fingerprint、required proof status、formula context
  requirement が VC/handoff input から供給される imported fact。

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

## goal と polarity

VC goal は `AtpProblem.conjecture` formula に変換される。translator は task-5/6 の全 problem に
`ExpectedBackendResult::Unsat` を記録する。concrete encoder は後で goal を TPTP
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
formula context requirement を必要とする。imported context の missing または mismatch は
fail closed する。

## failure semantics

translator failure は producer-side open-status または fail-closed diagnostic であり、proof result ではない。

- malformed または missing の required VC/handoff payload は fail closed する。
- missing structured formula / declaration / soft-type projection は fail closed する。
- stale target binding、wrong VC、duplicate id、duplicate symbol、duplicate premise identity、
  provenance gap は fail closed する。
- unsupported formula/profile feature は、その profile に対する unsupported/open translation outcome を生む。
- profile unavailable、disabled ATP policy、backend configuration absence は後続 backend / portfolio task
  のために記録され、accepted proof status を構築しない。

## gap 分類

- resolved `deferred`: task 4 が translator ownership と boundary を仕様化する。
- resolved `source_drift`: task 5 は declaration / soft-type payload 用の Rust projection input
  struct を定義し、target/status handoff gate、deterministic declaration / symbol-map /
  type-context translation、final `AtpProblem` construction を伴わない type-guard signature
  validation を実装する。
- `deferred`: task 6 が axiom / conjecture translation を実装する。
- `deferred`: property encoding、concrete encoder、backend runner、portfolio、candidate evidence
  extraction はそれぞれの module spec / task に残る。
- `external_dependency_gap`: proof-policy winner selection、witness publication、cache promotion は、
  owner crate が stable contract を定義するまで `mizar-atp` の外に残る。

## planned tests

task 5/6 implementation は合わせて Rust coverage を追加し、次を確認しなければならない:

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
- local context、generated、imported、checker-owned、type fact の premise-order determinism と
  provenance completeness。
- soft-type fact が lossless な場合だけ sort として表現され、それ以外は guard / axiom /
  type-context entry または unsupported profile result として残ること。
- goal/conjecture polarity が常に `ExpectedBackendResult::Unsat` を記録すること。
- translator public API と debug rendering に prohibited backend/kernel/SAT/proof-acceptance
  material が存在しないこと。
