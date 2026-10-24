# Module: problem

> 正本は英語です。英語版:
> [../en/problem.md](../en/problem.md)。

## 目的

`problem` module は、pipeline phase 13 の backend-neutral ATP problem data model
を所有する。これは、concrete TPTP、SMT-LIB、backend process、portfolio policy
より前に、validated VC input から生成される immutable `AtpProblem` shape を定義する。

`AtpProblem` は untrusted な candidate-production input である。kernel evidence、
proof witness、SAT problem、accepted proof material ではない。formula/substitution
evidence を検査し、instantiated formula を導出し、deterministic SAT material を構築し、
VC を受理できる component は引き続き kernel だけである。

## 境界規則

problem layer は ATP backend に必要な formula、declaration、type context、
encoded property、provenance、target binding を記述してよい。ただし次を行っては
ならない:

- ATP backend、SAT solver、`mizar-kernel` を実行する;
- premise を追加選択する、substitution を発明する、binder を修復する、overload を
  解決する、cluster を探索する、implicit coercion を挿入する、fallback inference を行う;
- TPTP text、SMT-LIB text、DIMACS、SAT clause、caller-supplied instantiated formula、
  backend proof method、backend log、backend `used_axioms`、SMT proof object、
  MiniSAT-compatible resolution trace、legacy certificate を trusted field として含める;
- backend result を accepted proof status として分類する;
- proof witness、cache entry、artifact proof status を発行する。

Backend proof trace と backend success report は、後続の untrusted candidate production
における diagnostic または extraction input にすぎない。`AtpProblem` を通じて trusted
acceptance material にはならない。

## Conceptual shape

task 3 は backend-neutral data model の具体的な Rust name を公開し、module は次の
conceptual shape を表現する:

```text
AtpProblem
  problem_id
  vc_id
  target_binding
  logic_profile
  expected_result
  declarations
  axioms
  conjecture
  type_context
  properties
  symbol_map
  provenance
  diagnostics?
```

`diagnostics` は任意の producer-side note であり、proof acceptance には決して参加しない。
deterministic debug rendering に含める場合でも、その rendering は non-semantic と
明示しなければならない。

## field 要件

| Field | 要件 |
|---|---|
| `problem_id` | problem content、選択された backend-neutral profile、snapshot-local な `vc_id` collation component の deterministic identity。source edit をまたぐ proof-reuse identity ではなく、kernel evidence でもない。 |
| `vc_id` | snapshot-local な VC ordering と collation id。task-3 semantic identity は同一 snapshot 内の同内容 problem を区別するためこれを含むが、これ単体を stable target binding として使ってはならない。 |
| `target_binding` | validated `VcIr` / VC handoff input から導出される stable target fingerprint と producer binding。task 3 は target binding の不足または構造的 invalid を reject する。不一致の比較は、binding の両側を持つ translator / handoff task に属する。 |
| `logic_profile` | backend-neutral capability profile: first-order fragment、equality support、quantifier policy、sort/type strategy、property-native capability、concrete-format eligibility。backend proof method を記録してはならない。 |
| `expected_result` | validity-checking polarity。現在の trusted success contract は `Unsat` である。premise と negated goal が、選択 encoding の下で unsatisfiable でなければならない。`Sat`、`Unknown`、timeout、crash、backend error は VC を証明しない。 |
| `declarations` | formula layer に必要な symbol、sort、function、predicate、generated-binder declaration。backend-visible declaration は provenance、一意な symbol、candidate evidence に影響し得る formula / property / type-guard reference が要求する kind / arity を持たなければならない。そのような reference は declaration と `symbol_map` row の両方を通じて解決されなければならない。`symbol_map` binding は encoding と diagnostics を支えるが、provenance の代替ではない。 |
| `axioms` | prior VC phase が既に materialize した premise formula の deterministic order。axiom は candidate-search input であり、trusted `used_axioms` ではない。 |
| `conjecture` | target goal formula。axiom ではなく、`used_axioms` source にもならない。 |
| `type_context` | sound backend encoding に必要な soft-type / sort context。sort encoding は、必要な mode、attribute、subtype、coercion、guard、intersection-like fact を消してはならない。 |
| `properties` | explicit axiom formula または `logic_profile` が選択した backend-native declaration として encode された definitional property。native property declaration も encoded id と provenance を必要とする。 |
| `symbol_map` | backend-safe symbol と canonical Mizar/core/generated identity の deterministic map。encoding と diagnostics のためのものであり、proof acceptance ではない。 |
| `provenance` | candidate evidence に影響し得る backend-visible formula、native property declaration、type guard、generated declaration ごとの完全な source binding。 |

## Logic profile

`logic_profile` は concrete encoding の前に選択され、Mizar-side translation capability
だけを記録する。次を記録してよい:

- problem が FOF、TFF-like typed first-order structure、uninterpreted symbol を持つ
  SMT-LIB、または後で明示的に仕様化される fragment のどれを使うか;
- equality、quantifier、finite sort encoding、native property declaration を許すか;
- soft type を backend sort、explicit guard predicate、またはその両方で表すか;
- どの concrete encoder が problem を消費できるか。

backend success policy、portfolio priority、proof method、solver seed、timeout、
command-line flag、process environment を encode してはならない。これらは backend
および portfolio spec に属する。

## Formula と declaration model

problem model は backend-neutral である。formula は selected `logic_profile` 内の
structured term、atom、equality、connective、quantifier として表現し、backend text
として表してはならない。concrete encoder は後で同じ problem を、source problem
identity を変えずに TPTP または SMT-LIB へ lower できる。

Declaration は canonical source identity と generated-binder identity から導出される
deterministic key で順序付ける。map iteration、pointer address、source range 単体、
display spelling 単体、backend output order で順序付けてはならない。

Rust data model は declaration、formula、property、provenance row、type guard に
caller-supplied な dense identifier を使う。これらの dense id は既に canonical な key
であり、producer と後続 translator task は、item の canonical source identity、
generated-binder identity、または stable generated-fact identity から決定的に導出しなければ
ならない。problem layer は一意性を検証し、これらの id で sort するが、traversal order、
backend order、map iteration、display spelling に基づく id を canonical として承認しない。

選択された profile で表現できない formula は、その profile では unsupported と分類する。
translator は後続 task で別の explicit profile を試してよいが、problem layer は formula
を黙って近似したり、必要な fact を落としたりしてはならない。

## Provenance

candidate に影響し得る backend-visible item はすべて provenance を持たなければならない。
許可される source class は次のとおり:

- validated `VcIr` 由来の local hypothesis;
- explicit に materialize された cited premise;
- stable payload を持つ generated VC fact;
- stable package/module/item identity、statement fingerprint、required proof-status
  requirement、kernel context requirement が利用可能な imported axiom / theorem;
- explicit formula payload と source binding が既に存在する checker-owned fact;
- type fact と encoded definitional property。

trace-only record、backend log、backend proof trace、backend-reported used axiom、
legacy certificate object は trusted acceptance の provenance ではない。Mizar-side trace
が fact の存在理由を説明できるのは、その fact が explicit formula/provenance input として
既に materialize された後だけである。trace 自体は formula ではない。

## Expected result と failure semantics

task-2 の problem shape はすべて、成功する validity contract として
`ExpectedBackendResult::Unsat` を使う。problem は goal を conjecture、negated
conjecture、または negated goal の SMT assertion として提示する profile に消費されて
よいが、記録される contract は、backend success が premise と negated goal の
unsatisfiability に対応しなければならない、というものである。

必須の target binding、formula payload、provenance、symbol mapping、declaration、
type context、または `logic_profile` field 自体が不足または invalid な場合、
construction は fail closed する。VC handoff input に対する target-binding mismatch は、
problem data shape が比較対象となる第二の expected binding を持たないため、
translator / handoff task に deferred する。valid な Mizar-side input であっても
selected profile が扱えない formula feature は、その profile では unsupported/open-status
outcome として分類する。ATP unavailability、unsupported profile、timeout、unknown、
crash、backend error は VC を open のままにするか diagnostic を生成するだけで、
accepted proof status を作らない。

## 決定性

同等の validated input は、byte-identical な deterministic debug rendering と stable
problem identity を生成しなければならない。deterministic ordering は次に適用する:

- declaration;
- axiom;
- generated type guard;
- encoded property;
- symbol-map row;
- provenance row;
- diagnostic が render される場合の diagnostic row。

backend completion order、wall-clock time、random state、process id、stdout、stderr、
backend-reported proof order は semantic identity から除外する。

## Structured local ranking

`AtpProblem::rank_axioms()` は `Result<(&'static str, Vec<&AtpFormula>), String>` として設定 `symbol-overlap-v1` と元の全 axiom の順位を返す。構造化 goal/candidate tree の named predicate/function を重複なしで集め、入れ子・零引数 function を含める。variable・binder・sort・equality・論理演算子は除外し、tree 不足は text/hash から復元せず error とする。
候補 N 件で共有 symbol ごとに N-df(s)+1 を足す。df(s) はその symbol を含む候補数で、反復で増やさない。u128 の和を降順、同点は一意の formula ID 昇順に並べる。希少性は入力集合内だけを表し、元 row の provenance と不変 input 対応を保って conjecture 追加・前提削除を行わない。
未検証 candidate-production の前提部分であり、source projection は既存の真正な入力を使う。library facts・実 budget/citation・刈り込み・提案/retry・未解決 label・公開・証明受理は担当の依存待ちに留める。library 全体の希少性や source-currentness receipt は得られない。

## Observed Ground Terms
`AtpProblem::observed_ground_terms(&self) -> Result<BTreeSet<&AtpTerm>, AtpProblemError>` は既存の変数を含まない term subtree への参照を返し、既存 AtpTerm Ord（function 名、再帰 argument）で重複排除・整列する。新 type/field/clone/identity/正規化/profile を作らない。
既存 axiom、type-context guard、axiom-form property、conjecture を全て辿る。全論理・量化 body、atom argument、equality 両辺を辿り、Function は全 descendant が Function の場合だけ closed とする。Variable と variable を含む compound は除くが、全 sibling を訪問し closed descendant は残す。同値 subtree は value でまとめ、参照は元の immutable problem 内を指す。pointer-address 順や重複の代表位置を約束しない。
binder/sort/predicate 名や未使用 declaration を term にせず、代入、既定 constant、新たな組合せ、text/debug-render parsing を行わない。観測対象が空なら空集合を返す。axiom/conjecture tree 不在は既存 MissingFormulaPayload、NativeDeclaration property は既存 UnsupportedProfileFeature { feature: "native property ground-term input" } とし、部分結果や fallback を返さない。現 concrete encoder も native property route を拒否する。
追加三系列は独立定数による全 section/論理/groundness/重複/順序/参照、空観測と妥当な native-property 拒否、元の choice term を期待する実 single/aggregate source-registration の元・改名・否定ケースと入力/encoding 不変性を扱う。既存 validated fixture builder と実 pipeline helper を再利用し、constructor/encoder 全 matrix を複製しない。
有限の入力 query だけを ATP30 前提として実装する。新 term 生成、代入、equality name、policy search limit、SAT/backend、candidate/evidence 生成、受理/公開/cache、Steps6–8 は後続 owner に残す。private recursive walker は各々三個以上の実 call site がある場合だけ最大二個を認め、既存 method/helper body は全て保つ。

## public enum forward compatibility

task 22 は frontend task 25 の方針を `problem` module に適用する。この module が所有する
public enum はすべて downstream crate 向けに `#[non_exhaustive]` とする:
`LogicFragment`、`EqualitySupport`、`QuantifierPolicy`、`SoftTypeStrategy`、
`NativePropertySupport`、`ConcreteFormat`、`ExpectedBackendResult`、
`AtpDeclarationKind`、`AtpFormulaTree`、`AtpTerm`、`PropertyEncoding`、
`AtpSymbolSource`、`AtpSourceRef`、`AtpProblemError`。

Public enum inventory: `LogicFragment`, `EqualitySupport`, `QuantifierPolicy`, `SoftTypeStrategy`, `NativePropertySupport`, `ConcreteFormat`, `ExpectedBackendResult`, `AtpDeclarationKind`, `AtpFormulaTree`, `AtpTerm`, `PropertyEncoding`, `AtpSymbolSource`, `AtpSourceRef`, `AtpProblemError`.

将来 variant は、source が使う前に仕様化しなければならない。`mizar-atp` 内部では、problem
identity、evidence provenance、backend-visible syntax、proof status に影響する match は、
paired spec が意図的 fallback を記録しない限り、明示的に保ち fail closed しなければならない。
`ExpectedBackendResult` は `Unsat` が task 3 唯一の success contract であっても外部向けには
non-exhaustive のままとする。将来の success contract には、paired spec、test、
acceptance-boundary review が必要である。

## Kernel evidence との関係

`AtpProblem` は、後続 ATP task が formula/substitution evidence candidate を生産する助けに
なってよいが、それ自体は kernel が受理する evidence ではない。candidate evidence は
`mizar-kernel` schema と互換な formula、substitution、provenance、target binding record から
構築しなければならない。instantiated formula と SAT problem は、checking 中に kernel だけが
導出する。

## gap 分類

- resolved `source_drift`: task 3 はこの spec に従って Rust data shape、deterministic
  debug rendering、fail-closed validation、construction test を実装する。
- `deferred`: translator、property encoding、concrete encoder、backend runner、
  portfolio behavior は、それぞれ独自の module spec を必要とする。
- `external_dependency_gap`: full imported-fact context と downstream proof/cache/artifact
  policy は、所有 crate が存在して stable contract を定義するまで、この crate の外に残る。

## task-3 test coverage

task 3 は Rust coverage を追加し、次を確認する:

- minimal / populated `AtpProblem` value の構築;
- shuffled input の下での deterministic debug rendering と stable ordering;
- declaration、axiom、property、type guard、conjecture ごとの provenance completeness;
- target binding、formula payload、provenance、symbol-map row、declaration、
  type-context binding の不足、および duplicate id に対する fail-closed construction
  rejection;
- 必要な fact を黙って近似または削除しない、profile limitation の unsupported classification;
- `ExpectedBackendResult::Unsat` が task 3 で唯一の polarity であること。non-`Unsat`
  success contract の rejection または表現不能性と、polarity field の stable
  rendering / identity coverage を含む;
- public problem API に backend text、SAT clause、instantiated formula、proof method、
  backend log、legacy certificate、accepted-proof status field が存在しないこと。

## Scoped guard problems

SoftTypeStrategy::ScopedGuardPredicates は閉じた局所 sequent の初期表現を明示し、大域 GuardPredicates と区別する。旧 mode・検証・MissingTypeContextBinding を保つ。
通常の declaration/provenance/formula 検証後、type-context 検証前に FirstOrder/Supported equality、空 axioms/properties/type-context guards、sort なしの Forall binder 一つを要求する。body は Implies(単項 Atom(Variable(binder)), Equality(Variable(binder),Variable(binder))) に限り、他の variable/term/connective shape は拒否する。構造拒否は既存 UnsupportedProfileFeature の feature scoped-guard sequent、先行検証 error は従来どおり。
この限定構文検査は Core/source role/predicate 名の解釈、source proof seal、新 visitor を必要としない。通常の declaration signature と encoder の generated-binder source-kind/free-variable 検査を保つ。antecedent に局所 guard を置き、goal と同じ binder で束縛し、大域 guard 断言・parameter 定数化・guard 消去はしない。
真正な限定統合は [source theorem producer](./translator.md#source-theorem-scoped-sequent) が担う。広い guarded sequent は未完了。直接の problem data と構築成功は受理・生成元・currentness の権限ではない。
