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
