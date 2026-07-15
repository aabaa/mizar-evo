# mizar-checker: Type Checker

> 正本は英語です。英語版:
> [../en/type_checker.md](../en/type_checker.md)。

## 目的

`type_checker` は resolver の名前解決と binding environment 構築の後に走る
phase-6 type checking / inference を仕様化する。これは次を精緻化する。

- [architecture 04](../../architecture/ja/04.type_and_registration_resolution.md)
  Step 2 "Normalize Type Expressions";
- [architecture 04](../../architecture/ja/04.type_and_registration_resolution.md)
  Step 3 "Check Declarations and Local Bindings";
- [architecture 04](../../architecture/ja/04.type_and_registration_resolution.md)
  Step 4 "Infer Terms and Formulas";
- [spec chapter 03](../../../spec/ja/03.type_system.md) の soft type、radix type、
  attribute、widening;
- [spec chapter 08](../../../spec/ja/08.type_inference.md) の conversion、`qua`、
  `reconsider`;
- [spec chapter 13](../../../spec/ja/13.term_expression.md) の term expression、
  set enumeration、set comprehension、choice term;
- [`typed_ast.md`](./typed_ast.md) の type / fact / coercion / obligation /
  diagnostic table;
- [`binding_env.md`](./binding_env.md) の local binding と context construction。

task 6 は specification-only task である。Rust source、active checker runner、
language behavior change、registration closure、final overload root selection、
VC generation、proof acceptance は追加しない。task 7-11 が以下の named section を
実装する。

## 境界

`type_checker` が所有するもの:

- source type expression から deterministic normalized type key への正規化;
- `BindingEnv` 上の declaration、local binding、binding type check;
- final overload root selection 前の term / formula type inference;
- expected-type constraint と未確定 typed candidate group;
- widening、source-written `qua`、narrowing coercion candidate;
- sethood、non-emptiness、narrowing claim の checker-local
  `InitialObligation`;
- type fact、local assumption、deterministic fact query;
- recoverable semantic error の partial typing と diagnostic recovery。

`type_checker` が所有しないもの:

- resolver name lookup、label lookup、import/export validation、symbol allocation;
- `binding_env` が所有済みの binding-context construction と binder identity rule;
- cluster saturation、registration activation、reduction normalization、
  canonical `ResolutionTrace` schema;
- final ordinary overload root selection、active refinement joining、inserted
  overload-disambiguating `qua` view;
- `binding_env` が提供する definition-time closure metadata の保存を超える
  `set`、`deffunc`、`defpred` body の expansion replay;
- `VcId`、`ObligationAnchor`、proof witness、prover result、accepted verifier
  status、kernel replay;
- checker diagnostic code-space が外部 planning gate である間の public
  diagnostic-code allocation。

## 入力と出力

type checker は次を消費する。

- resolver `ResolvedAst` 1 つ;
- 対応する resolver `SymbolEnv`;
- validated `BindingEnv`;
- recovery と feature gate を制御する checker-local configuration;
- 後続 task が task-scoped seam として公開した場合の dependency summary と
  activated registration summary。

type checker は checker-local `TypedAst` snapshot を生成する。

```rust
struct TypeCheckOutput {
    typed_ast: TypedAst,
    diagnostics: TypeDiagnosticTable,
}
```

論理実装は checking 中に mutable `TypeCheckState` を使ってよいが、accepted output は
`TypedAst` table で表現する。

- `LocalTypeContextTable`;
- `TypeTable`;
- `TypeFactTable`;
- `CoercionTable`;
- `InitialObligationTable`;
- `TypeDiagnosticTable`。

phase-6 output は stable artifact schema ではない。後続 artifact task は、独自仕様を
通じてのみ `TypedAst` から stable summary を投影できる。

## 正規化 type model

type checking は Mizar type を untyped object 上の soft predicate として扱う。
normalized type は radix/type-head key と canonical attribute set から成る。

```rust
struct NormalizedType {
    id: NormalizedTypeId,
    head: TypeHeadRef,
    args: Vec<NormalizedTypeId>,
    attributes: AttributeSet,
    source: TypeSource,
    status: NormalizedTypeStatus,
}

enum TypeHeadRef {
    BuiltinObject,
    BuiltinSet,
    Mode(SymbolId),
    Structure(SymbolId),
    Error(TypeHeadErrorKind),
}

enum TypeHeadErrorKind {
    Unknown,
    WrongKind,
    Ambiguous,
    Unsupported,
    Recovery,
}

struct AttributeSet {
    positive: Vec<AttributeInstance>,
    negative: Vec<AttributeInstance>,
}

struct AttributeInstance {
    symbol: SymbolId,
    args: Vec<NormalizedTypeId>,
    source_range: SourceRangeKey,
    spelling: String,
}
```

canonical type key は head kind、canonical `SymbolId`、normalized argument key、
attribute key の順で並ぶ。source range は diagnostic のために保持する。source spelling
と range は debug rendering 用に normalized record 上にも保持するが、どちらも
semantic equality を定義しない。

必須 invariant:

- built-in `object` と `set` head は canonical singleton head である;
- mode head は resolver が十分な signature payload を公開する場合、declared radix と
  attribute chain へ unfold する;
- structure head は radix head のままであり、黙って `set` と扱わない;
- 同じ polarity と argument を持つ duplicate attribute は 1 つに collapse する;
- 同じ attribute key の positive / negative occurrence は silently remove せず、
  diagnostic と degraded normalized type を作る;
- type argument は parent type key へ入る前に正規化する;
- signature、mode definition、structure、attribute payload の欠落は
  `external_dependency_gap` であり、raw parser syntax を inspect する理由にならない。

## Task 7: type-expression normalization

task 7 がこの section を実装する。

入力:

- typed site、source range、type-head symbol、type argument、attribute occurrence を
  識別する checker-owned resolved type-expression payload;
- `SymbolEnv` で validation する type-head、attribute、mode、structure identity;
- 後続 resolver / artifact task が payload を公開した場合の radix / attribute payload 用
  explicit mode-expansion provider。

出力:

- deterministic key order の `NormalizedTypeTable` entry を所有する task-local
  `TypeNormalizationOutput`;
- type-expression site の `TypeEntry`;
- unknown head、wrong arity、wrong symbol kind、contradictory attribute、
  unsupported checker-owned payload、missing explicit mode-expansion provider
  payload の diagnostic。

現在の resolver は type expression 用 typed site table を公開しないので、task 7 は
`ResolvedAst` を直接 walk しない。integration seam は上記の checker-owned payload である。
後続 resolver/source-walk task はその payload を `ResolvedAst` から埋めてもよいが、task 7 は
raw surface node kind から推測してはならない。

規則:

1. type head は resolver/symbol payload だけで resolve する。checker は name lookup を
   やり直したり raw syntax node kind を inspect してはならない。
2. type argument を左から右へ正規化し、その canonical type key を parent type で使う。
3. `non A` を negative set、`A` を positive set に保存して attribute polarity を正規化する。
4. attribute を canonical symbol id、normalized arguments、polarity、source range の順で
   sort する。この順序は storage/rendering 用だけである。
5. explicit mode-expansion provider が defining radix と attribute payload を供給する場合だけ
   mode definition を unfold する。payload がない場合は `external_dependency_gap` を記録し、
   mode symbol headed の degraded type を保持する。
6. すべての diagnostic について source range を保持する。normalized type と attribute
   record には debug rendering 用の source spelling と range を保持し、semantic に同等な
   type key には deterministic representative source を使う。
7. task 7 では cluster closure を使って normalized type を「修復」しない。cluster closure は
   phase 7 であり、後続 registration task が所有する。

## Task 8: declaration and local-binding checking

task 8 がこの section を実装する。

declaration checking は binding に normalized type を付け、対応する
`LocalTypeContext` snapshot へ local fact を導入する。

task 8 は checker-owned declaration payload を使う。この payload は検査する
`BindingEnv` の binding / context id、typed declaration site、optional explicit
type-expression payload、source range を識別する。現在の resolver は AST 全体の
declaration/type-site table、reserve default payload、right-hand-side term payload、
definition body payload を公開しないため、task 8 は raw syntax を walk してそれらを
再構成してはならない。

現在の source-derived producer seam: `type_checker` module は、upstream runner が
抽出した syntax-free な reserve-only payload を受け取り、builtin `set` /
`object` reserve declaration 用の checker-owned `BindingEnv` と
`DeclarationCheckingOutput` を構築してよい。successful pass slice では payload は
bare builtin `set` / `object` である。task 50 はさらに、attribute symbol がすでに
resolver `SymbolEnv` に存在する場合に限り、これら builtin head 上の
source-derived attribute payload を許可する。attributed reserve declaration は
`MissingEvidenceQuery` を付けられ、real existential / evidence-query seam が
存在するまで active fail case のままにする。task 51 はさらに、type argument を持たず
attribute も付かない、unique な same-module `LocalSource` `SymbolKind::Mode` entry に
解決された reserve type head を許可する。これら local-mode reserve declaration は
type normalization に到達し、real mode-expansion provider / extraction seam が存在する
まで `checker.type.external.mode_expansion_payload` で fail closed する。task 54 はその
local-mode slice に same-module source-derived attribute を持たせることを
許可する。supported real mode-expansion payload が存在しない場合や、同じ local mode が
bare reserve use と mixed の場合は `checker.type.external.mode_expansion_payload` で
fail closed し、fully expanded attributed type の existential evidence は主張しない。
task 55 は
bare local-mode reserve use だけに 1 つの real mode-expansion producer slice を許可する:
runner は、reserve type use より前に現れる unique / unrecovered / same-module /
no-argument `ModeDefinition` で、definition-local parameter / assumption context を持たず、
RHS が attribute や argument のない bare builtin `set` / `object` である場合に限り、
`ModeExpansion` を供給してよい。runner は同じ local mode head に attribute を付ける
reserve binding が source 内に 1 つでもある場合、その expansion を渡さないため、
task 54 の attributed local-mode fail-closed behavior は task 59 まで維持される。task 59 は、
同じ local mode が同じ bridge input 内で bare reserve head としても使われていない場合に限り、
attributed local-mode reserve head に対して同じ real direct bare-builtin RHS expansion を許可する。
expanded attributed declaration は、real attributed-type existential evidence がまだ無いため
`checker.declaration.deferred.evidence_query` に到達する。mixed bare/attributed use は
missing-expansion path に残る。task 56 はこの
producer を 1 つの source-derived chain edge だけ拡張する: bare local-mode reserve
head は、unique / preceding / same-module / no-argument mode definition の RHS が別の
bare same-module no-argument local mode であり、その dependency mode が task 55 の
bare builtin `set` / `object` expansion として受理済みで、dependency definition が chain
definition より前に現れ、両方の definition node が AST 上で一意に対応し、chain 内の
どちらの symbol にも attribute 付き reserve binding が無い場合だけ expand してよい。
forward、ambiguous、partial、imported、argument-bearing、parameterized、
contextual、cyclic、task-62 bare one-edge chain slice 外の attributed-structure RHS、
task-58/task-61 direct slice と task-63 bare one-edge chain slice 外の
attributed-RHS chain、task 64 外の attributed-root bare-builtin chain、task 65 外の
attributed-root structure-RHS chain、または attributed-builtin RHS mode definition は missing-expansion /
extraction-gap path に残り、checker-owned seam は expansion や existential evidence を
捏造してはならない。task 72 は bare builtin terminal の pass slice だけをもう 1 つの
source-derived dependency edge に拡張する: bare local-mode reserve head は、3 個すべての
mode definition が unique / unrecovered / same-module / no-argument /
definition-local-context-free / source-preceding である場合に限り、
`Outer -> Middle -> Base -> set` / `object` として expand してよい。active pass
fixture は既存の `TypedAst`、`ResolvedTypedAst`、summary-readiness、binder-only
`CoreContext` preparation path を通るが、新しい CoreIr / ControlFlowIr / VC / proof
payload は昇格しない。別の supported reserve binding によって middle dependency が
cache 済みである場合も含め、three-edge local-mode dependency chain は task 72 時点で
missing mode-expansion diagnostic に残していた。task 73 は同じ source-derived seam を
さらに 1 dependency edge 昇格し、4 個すべての mode definition が同じ
unique / same-module / no-argument / definition-local-context-free /
source-preceding 制約を満たす場合に `Outer -> Middle -> Inner -> Base -> set` /
`object` を pass させる。task 74 は temporary depth cap を structural な bare
builtin-terminal rule に置き換え、すべての mode definition が unique /
unrecovered / same-module / no-argument / definition-local-context-free /
source-preceding / argument-free / attribute-free で、terminal RHS が exactly
builtin `set` / `object` である場合、AST-bounded acyclic chain を pass させる。
producer は source mode definition 数と等しい AST-derived traversal budget を持つ。
この budget は resource guard であり semantic chain-length limit ではない。
structural guard を満たさない chain は missing-expansion / extraction-gap path に
残る。task 75 はこの family の forward-reference boundary を固定する。
reserve head が、その mode declaration item が active になる前に local mode を
名前参照する場合、lower-stage frontend/resolver processing は checker handoff の前に
`type_elaboration.lower_stage.frontend:malformed_type_expression` で type
expression を拒否する。source-derived runner は future declaration から後続の
`ModeExpansion` payload を捏造してはならない。task 76 は同じ active-range
boundary を same-module local structure にも適用する。reserve head が、その
structure declaration item が active になる前に local structure を名前参照する場合、
lower-stage frontend/resolver processing は checker handoff 前に
`type_elaboration.lower_stage.frontend:malformed_type_expression` で type
expression を拒否する。runner は future declaration から structure type-head
payload、successful reserve declaration、base-shape / constructor-witness
evidence query、CoreIr/ControlFlowIr/VC/proof payload を捏造してはならない。
task 77 は同じ boundary を same-module local attribute に適用する。
`marked set` のような reserve type expression が、その attribute declaration
item が active になる前に attribute を使う場合、lower-stage frontend/resolver
processing は checker handoff 前に
`type_elaboration.lower_stage.frontend:malformed_type_expression` で type
expression を拒否する。runner は future attribute declaration から
`AttributeInput`、attributed-type evidence query、successful reserve
declaration、CoreIr/ControlFlowIr/VC/proof payload を捏造してはならない。
task 78 は imported-structure 類似 case を checker payload ではなく external
extraction-gap boundary として historical に記録した。task 83 は documented
`parser.type_fixtures` `R` reserve head だけについて real imported
`SymbolKind::Structure` を checker type head として渡すことで、その boundary を
上書きする。task 97 は同じ narrow bridge を documented `TypeCaseStruct` fixture
に適用する。task-83 / task-97 fixture 外の broader imported structure は deferred
のままとする。bridge はこの import summary を real imported module AST extraction
と扱ってはならず、base-shape / constructor-witness evidence、positive structure
elaboration、CoreIr/ControlFlowIr/VC/proof payload を捏造してはならない。task 83
と task 97 は imported structure provenance bridge を記録する。documented
`parser.type_fixtures` import summary 由来の `R` と `TypeCaseStruct` は imported
structure type head として declaration checking に到達し、base-shape /
constructor-witness evidence がまだ存在しないため
`type_elaboration.checker.checker.declaration.deferred.evidence_query` で fail
closed する。これは positive imported structure elaboration や imported module AST
extraction を credit しない。
task 79 は元々 imported-mode 類似 case を同じ external extraction-gap boundary
として記録した。task 82 は documented `parser.type_fixtures` `TypeCaseMode`
reserve head だけについて real imported `SymbolKind::Mode` を checker type head
として渡すことで、その boundary を上書きする。task-82 bridge 外の imported
mode は引き続き `type_elaboration.external_dependency.ast_payload_extraction` に残る。
bridge はこの import summary を real imported module AST extraction と扱っては
ならず、`ModeExpansion` payload、positive mode elaboration、
CoreIr/ControlFlowIr/VC/proof payload を捏造してはならない。
task 80 は imported-attribute 類似 case を historical に同じ external
extraction-gap boundary として記録した。task 84 は documented
`parser.type_fixtures` `TypeCaseAttr` reserve attribute だけについて real
imported `SymbolKind::Attribute` を checker `AttributeInput` として渡すことで、
その boundary を上書きする。task 85 は既存 negative `empty`/builtin-`set`
fixture だけについて real imported `empty` attribute を negative checker
`AttributeInput` として渡すことで同じ boundary を上書きする。task 116 は既存
positive `empty`/builtin-`set` fixture だけについて同じ imported attribute を
positive checker `AttributeInput` として渡すことで同じ boundary を上書きする。
task 171 は既存 negative `empty`/builtin-`object` fixture だけについて real
imported attribute を negative checker `AttributeInput` として渡すことで同じ
boundary を上書きする。task-84 / task-85 / task-116 / task-171 bridge 外の
broader imported attribute は、
source-derived fixture と payload producer が存在するまで deferred のままとする。
bridge はこの import summary を real
imported module AST extraction と扱ってはならず、attributed-type evidence、positive
attributed type elaboration、builtin `object` 上の positive `empty`、symbol head
上の imported `empty`、
CoreIr/ControlFlowIr/VC/proof payload を捏造してはならない。
task 84 は imported-attribute provenance bridge を記録する。documented
`parser.type_fixtures` import summary 由来の `TypeCaseAttr` は builtin `set` 上の
imported attribute payload として declaration checking に到達し、attributed-type
existential/evidence payload がまだ存在しないため
`type_elaboration.checker.checker.declaration.deferred.evidence_query` で fail
closed する。これは positive imported attributed type elaboration、imported
module AST extraction、`empty` のような generic imported attribute、
structure-qualified attribute owner provenance、attribute argument を credit しない。
task 85 は次の imported-attribute provenance slice を記録する。既存の
`non empty set` fixture は、documented `parser.type_fixtures` imported attribute
`empty` を builtin `set` 上の negative checker `AttributeInput` として渡してよく、
同じ evidence-query diagnostic で fail closed する。これは broader task-80
payload gap を、その negative `empty` / builtin-`set` source shape についてだけ
上書きする。task 116 は matching positive imported-attribute provenance slice を
記録する。既存 `empty set` fixture は documented imported `empty` attribute を
builtin `set` 上の positive checker `AttributeInput` として渡してよく、同じ
evidence-query diagnostic で fail closed する。これは broader task-80 payload gap を、
その positive `empty` / builtin-`set` source shape についてだけ上書きする。
task 171 は matching negative builtin-object provenance slice を規定する。既存
`non empty object` fixture は documented imported `empty` attribute を builtin
`object` 上の negative checker `AttributeInput` として渡し、`ImportedSource`
provenance と written polarity を保持した後、同じ evidence-query diagnostic で
fail closed しなければならない。これは task-80 payload gap を exact source shape
だけについて上書きする。positive `empty object` と symbol head 上の imported
attribute は extraction gap のままである。tasks 85、116、171 は attribute
admissibility、attributed-type evidence / acceptance、imported module AST extraction、
structure-qualified owner provenance、attribute argument、downstream payload を
credit しない。
task 81 は same-module argument-bearing local attribute surface について同じ
extraction-gap boundary を記録する。`attr RankedDef: x is 2-ranked` のように
Chapter 6 の `param_prefix` 構文で書かれた declaration-site attribute と、
`ranked(2) set` のような Chapter 3/6 の use-site application は、real
lexer/parser/frontend producer seam によって active type-elaboration runner まで
運ばれ、resolver declaration-symbol projection は suffix を primary spelling として
記録し lexer-visible lexical summary として export しつつ、prefixed surface を
notation として保持する。runner は
`type_elaboration.external_dependency.ast_payload_extraction` を報告する。checker
payload extraction はまだ real term-argument provenance や
checker-owned `AttributeInput` argument payload を保持しないためである。bridge は
attribute argument、attributed-type evidence、positive parameterized attribute
elaboration、CoreIr/ControlFlowIr/VC/proof payload を捏造してはならない。
task 86 は theorem/formula extraction-gap boundary を記録する:
`theorem FormulaPayloadBoundary: thesis;` のような theorem formula だけを含む
source は active type-elaboration runner まで到達できる。task 115 はこの exact
source だけを supersede し、source-derived `thesis` formula constant site/range
を checker recovery `FormulaInput` として渡す。task 117 はその recovery marker
を real `FormulaKind::Thesis` checker payload に進め、missing formula payload
で fail closed する。この boundary は引き続き formula constant semantics、
child-formula graph payload、theorem acceptance、formula fact、proof skeleton、
local proof context、`formula_statement`、CoreIr、ControlFlowIr、VC、proof
payload を credit しない。
task 106 は task 87 の generic boundary のうち、builtin equality theorem formula
`theorem TermFormulaPayloadBoundary: 1 = 1;` の narrow slice を supersede する。
active runner は Chapter 13 の 2 つの numeral operand から real checker
`TermInput` を、Chapter 14 の equality formula から real checker `FormulaInput` を
module binding context の下で抽出し、その後
`type_elaboration.checker.checker.term.external.numeric_type_payload` と
`type_elaboration.checker.checker.formula.term.partial` で fail closed する。
numeric type payload、equality checking、recorded fact、theorem acceptance、
dedicated `formula_statement` runner、CoreIr、ControlFlowIr、VC、proof payload は
まだ deferred のままである。
task 98 は同じ boundary の imported predicate/functor variant を historical に記録した:
`theorem ImportedPredicateFunctorPayloadBoundary: 1 divides (1 ++ 2);` は documented
`parser.type_fixtures` surface を通じて parser / resolver 実行まで到達する。task
110 はこの exact source を supersede し、source-derived numeral `TermInput`、
imported `++` symbol reference を持つ functor-application `TermInput`、および
predicate-application `FormulaInput` を抽出する。runner は `divides` / `++` の
imported provenance を検証してから、missing numeric type payload、missing functor
signature payload、missing predicate signature payload、partial formula checking で
fail closed する。これは imported module AST extraction、semantic
predicate/functor signature payload、term inference、formula checking、recorded
fact、theorem acceptance、dedicated `formula_statement` runner、CoreIr、
ControlFlowIr、VC、proof payload を credit しない。
task 108 は task 100 の builtin membership generic boundary のうち exact formula
`theorem BuiltinMembershipPayloadBoundary: 1 in 1;` を supersede する。active
runner は Chapter 13 の 2 つの numeral operand から real source-derived checker
`TermInput` payload を、Chapter 14 の membership formula から real checker
`FormulaInput` payload を module binding context の下で抽出し、
`type_elaboration.checker.checker.term.external.numeric_type_payload` と
`type_elaboration.checker.checker.formula.term.partial` で fail closed する。
numeric type payload、membership operand expected-type construction/checking、
recorded fact、theorem acceptance、dedicated `formula_statement` runner、CoreIr、
ControlFlowIr、VC、proof payload はまだ存在しない。
task 107 は task 101 generic boundary の exact builtin inequality theorem formula
`theorem BuiltinInequalityPayloadBoundary: 1 <> 2;` を supersede する。active
runner は Chapter 13 の 2 つの numeral operand から real source-derived checker
`TermInput` payload を、Chapter 14 の inequality formula から real checker
`FormulaInput` payload を module binding context の下で抽出し、
`type_elaboration.checker.checker.term.external.numeric_type_payload` と
`type_elaboration.checker.checker.formula.term.partial` で fail closed する。
numeric type payload、inequality desugaring または equality semantic checking、
recorded fact、theorem acceptance、dedicated `formula_statement` runner、CoreIr、
ControlFlowIr、VC、proof payload はまだ存在しない。
task 118 は task 106/107/108 が共有する builtin-binary theorem producer を
厳密化し、direct theorem token が exact `theorem <label> : ;` である場合だけ
equality、membership、inequality config を選ぶ。status-prefixed または extra-token
を持つ theorem shape は
`type_elaboration.external_dependency.ast_payload_extraction` に残る。この
guard-only repair は新しい `.miz` sidecar coverage や spec coverage credit を追加しない。
task 119 は exact source
`reserve x for set; theorem ReservedVariableEqualityPayloadBoundary: x = x;`
について、diagnostic を出さない最初の source-derived identifier-term / equality
slice を追加する。runner は real reserve `BindingEnv` を再利用し、2 つの
identifier-term site を独立した `BindingEnv::lookup` call で解決する。lookup
ordinal は source binding range と 2 つの use range を 1 つの binding/use event
stream として source 順に sort して割り当てるため、exact fixture では ordinal 0
の reserve binding に続く ordinal 1 と 2 を導出し、shared synthetic use ordinal
を供給しない。記述された reserve type の
range、spelling、builtin `set` head を、左右 term result type と左右 equality
expected-type constraint の 4 つの distinct checker role site に投影する。
`TermFormulaChecker` は 2 つの variable term を `Inferred`、equality formula を
`Checked` として diagnostic/fact なしで記録する。active producer は pass を
報告する前に declaration/binding identity、両 lookup result、term/formula site と
status、expected-type range、4 つすべての role owner、normalized source
spelling/range/head、empty candidate/fact/deferred/diagnostic table を検証する。
不一致は stable
`type_elaboration.checker.reserved_variable_equality.invalid_payload` detail key を
報告する。ここで `Checked` は
source-derived term/type/formula payload の well-formedness だけを意味し、task
119 は implicit universal-closure node を materialize せず、theorem を証明・受理
せず、equality fact を記録せず、`formula_statement` を activate せず、proof、
CoreIr、ControlFlowIr、VC payload を生成しない。non-exact label、operand、reserve
binding/type、attributed type、operator、status/extra token、追加 reserve/theorem
item、source-order reversal、recovery、numeral-term shape は
`type_elaboration.external_dependency.ast_payload_extraction` に残る。
task 123 は exact distinct-binding sibling
`reserve x, y for set; theorem DistinctReservedVariableEqualityPayloadBoundary: x = y;`
を追加する。real multi-reserve handoff は、両 source binding が同じ記述上の
builtin `set` type range を指していても、checker binding 2 個を所有する。
source binding/use ordering は binding ordinal 0 と 1 の後に lookup ordinal 2 と
3 を導出し、independent `BindingEnv::lookup` call は operand を collapse せず
`BindingId(0)` と `BindingId(1)` に解決する。operand ごとの result/expected
role は対応する source binding provenance を保持し、checker は 2 `Inferred`
variable term と 1 fact-free `Checked` equality を記録する。production invariant
は distinct identity、shared type range、exact source shape、role ownership、empty
candidate/fact/deferred/diagnostic output を検証する。drift は
`type_elaboration.checker.distinct_reserved_variable_equality.invalid_payload` を
報告し、near-miss matrix と real frontend/resolver sidecar は separate reserve
item、reversed/same operand、wrong label/operator/type、extra binding/item、
status/recovery、numeral operand を extraction gap に残す。これは
type/well-formedness だけであり、implicit universal closure と quantifier order、
equality truth/fact、theorem acceptance、`formula_statement`、proof、CoreIr、
ControlFlowIr、VC は deferred のままである。
task 124 は exact multiple-declaration sibling
`reserve x for set; reserve y for set; theorem MultipleReserveDeclarationEqualityPayloadBoundary: x = y;`
を追加する。reserve producer は 2 binding identity と 2 distinct written type
range を保持する。operand ごとの 4 pre-normalization result/expected input は、
checker が同一の builtin `set` semantics を deterministic な最初の source
representative を持つ 1 normalized type に intern する前に、それぞれの range を
保持する。validation は original input を独立に検証するため、normalized interning
は duplicate type を捏造せず、written provenance も消去しない。exact shape guard、
task-specific invalid-payload key、near miss、real frontend/resolver sidecar が seam
を cover する。implicit closure/order、truth/fact、theorem acceptance、
`formula_statement`、proof、CoreIr、ControlFlowIr、VC は deferred のままである。
task 125 は exact heterogeneous membership sibling
`reserve x for object; reserve y for set; theorem HeterogeneousReserveMembershipPayloadBoundary: x in y;`
を追加する。generalized binary bridge は全 reserved operand を `set` と仮定せず、
binding-specific builtin shape を保持する。左 result input は source-derived
`object`、右 result と唯一の expected input は source-derived `set` である。
production validation は 2 normalized identity を要求し、右 result/expected role は
`set` identity を共有し、左 `object` identity は distinct のままにする。各 identity
は written declaration 由来の deterministic canonical source を保持する。exact
guard、task-specific invalid-payload key、near miss、real frontend/resolver sidecar
が seam を cover する。これは type/well-formedness だけであり、membership
truth/fact、object/set coercion evidence、implicit closure/order、theorem acceptance、
`formula_statement`、proof、CoreIr、ControlFlowIr、VC は deferred のままである。
task 126 は exact direct-local-mode equality sibling を追加する。4 raw
result/expected input は reserve の `LocalModeFormula` symbol/range を保持し、
`TermFormulaChecker` は exact AST-derived direct bare-`set` `ModeExpansion` を受け、
全 role を 1 builtin-set identity に normalize する。normalized canonical source は
expansion RHS であり、original mode provenance は raw input で review できる。exact
guard、invalid key、withheld-family near miss、real sidecar が bridge を guard する。
mode declaration checking/acceptance、inhabitation evidence、broader mode、
closure/order、truth/fact、theorem acceptance、proof、CoreIr、ControlFlowIr、VC は
deferred のままである。
task 127 は exact one-edge bare local-mode-chain equality sibling を追加する。active
source は `BaseModeFormula -> set` と
`ChainModeFormula -> BaseModeFormula` の separate definition block 2 個、outer-mode
reserve 1 個、`ChainedLocalModeReservedVariableEqualityPayloadBoundary: x = x;` を
持つ。runner は 4 raw outer-mode input を保持し、既存 recursive `TypeNormalizer`
は task-56 real expansion 2 個を消費し、terminal `set` RHS を 1 normalized
identity の canonical source として保持する。exact label/block structure/chain
link、invalid-link corruption、withheld-family near miss が route を guard する。mode
declaration acceptance/inhabitation、object terminal、longer-chain formula、
closure/order、truth/fact、theorem acceptance、proof、CoreIr、ControlFlowIr、VC は
deferred のままである。
task 128 は exact direct local-object-mode equality sibling に限定する。source
は task-55 `LocalObjectMode -> object` definition を再利用し、reserve 1 個と
`LocalObjectModeReservedVariableEqualityPayloadBoundary: x = x;` を追加し、4 raw
object-mode input を保持して既存 `TypeNormalizer` に real expansion RHS 由来の 1
builtin-object identity を要求する。exact block/label guard、invalid key、
withheld-family near miss、real sidecar が route を guard する。mode
declaration acceptance/inhabitation、closure/order、truth/fact、theorem acceptance、
proof、CoreIr、ControlFlowIr、VC は deferred のままである。
task 129 は exact one-edge local-object-mode-chain equality sibling に限定する。
source は task 56 の `ChainObjectMode -> BaseObjectMode -> object` producer を
再利用し、exact outer-mode reserve と
`ChainedLocalObjectModeReservedVariableEqualityPayloadBoundary: z = z;` を追加し、
4 raw outer-mode input を保持して recursive `TypeNormalizer` に両 real expansion
と terminal object-RHS provenance の消費を要求する。exact label/block
order/chain link、invalid-link corruption、withheld-family near miss、real sidecar
で route を保護する。mode declaration acceptance/inhabitation、
longer-chain formula、closure/order、truth/fact、theorem acceptance、proof、CoreIr、
ControlFlowIr、VC は deferred のままである。
task 130 は exact direct bare-set local-mode inequality に限定する。4 raw
`LocalModeInequality` input を保持し、real expansion を消費して RHS 起点の
builtin-set identity 1 個と fact-free pre-desugaring `Checked` inequality を
記録する。non-exact shape は fail closed とし、declaration acceptance、
desugaring、truth/fact、theorem/proof/Core/VC は deferred のままである。
task 131 はその exact inequality consumer を direct bare-object
`LocalObjectModeInequality -> object` producer に適用する。4 raw object-mode
input は記述された provenance を保持し、real expansion 1 本がそれらを RHS
起点の builtin-object identity 1 個へ normalize してから、2 `Inferred`
variable term と 1 fact-free pre-desugaring `Checked` inequality を記録する。
non-exact shape は fail closed とし、mode declaration acceptance/inhabitation、
desugaring、closure/order、truth/fact、theorem acceptance、proof、CoreIr、
ControlFlowIr、VC は deferred のままである。
task 132 は同じ pre-desugaring inequality consumer を exact one-edge bare-set
chain `ChainModeInequality -> BaseModeInequality -> set` に適用する。4 raw
outer-mode input は記述された provenance を保持し、real AST-derived expansion
2 本がそれらを terminal-RHS builtin-set identity 1 個へ normalize してから、2
`Inferred` variable term と 1 fact-free `Checked` inequality を記録する。missing
または non-exact link、object-terminal、direct、longer shape は fail closed とし、
mode declaration acceptance/inhabitation、desugaring、closure/order、truth/fact、
theorem acceptance、proof、CoreIr、ControlFlowIr、VC は deferred のままである。
task 133 はその consumer を exact one-edge bare-object chain
`ChainObjectModeInequality -> BaseObjectModeInequality -> object` に適用する。
4 raw outer-mode input は記述された provenance を保持し、real AST-derived
expansion 2 本が terminal-RHS builtin-object identity 1 個へ normalize してから、
2 `Inferred` variable term と 1 fact-free `Checked` inequality を記録する。
missing/non-exact link、set-terminal、direct、longer shape は fail closed とし、
declaration acceptance/inhabitation、desugaring、closure/order、truth/fact、theorem
acceptance、proof、CoreIr、ControlFlowIr、VC は deferred のままである。
task 134 は equality consumer を exact two-edge bare-set chain
`OuterTwoEdgeModeEquality -> MiddleTwoEdgeModeEquality -> BaseTwoEdgeModeEquality -> set`
に適用する。4 raw outer-mode input は記述された provenance を保持し、real
AST-derived expansion 3 本が terminal-RHS builtin-set identity 1 個へ normalize
してから、2 `Inferred` variable term と 1 fact-free `Checked` equality を記録する。
missing/non-exact link、object-terminal、direct、one-edge、longer shape は fail
closed とし、declaration acceptance/inhabitation、implicit closure/order、
truth/fact、theorem acceptance、proof、CoreIr、ControlFlowIr、VC は deferred の
ままである。
task 135 は同じ equality consumer を exact two-edge bare-object chain
`OuterTwoEdgeObjectModeEquality -> MiddleTwoEdgeObjectModeEquality -> BaseTwoEdgeObjectModeEquality -> object`
に適用する。4 raw outer-mode input は記述された provenance を保持し、real
AST-derived expansion 3 本が terminal-RHS builtin-object identity 1 個へ normalize
してから、2 `Inferred` variable term と 1 fact-free `Checked` equality を記録する。
missing/non-exact link、set-terminal、direct、one-edge、longer shape は fail
closed とし、declaration acceptance/inhabitation、implicit closure/order、
truth/fact、theorem acceptance、proof、CoreIr、ControlFlowIr、VC は deferred の
ままである。
task 136 は pre-desugaring inequality consumer を exact two-edge bare-set chain
`OuterTwoEdgeModeInequality -> MiddleTwoEdgeModeInequality -> BaseTwoEdgeModeInequality -> set`
に適用する。4 raw outer-mode input は記述された provenance を保持し、real
AST-derived expansion 3 本が terminal-RHS builtin-set identity 1 個へ normalize
してから、2 `Inferred` variable term と 1 fact-free pre-desugaring `Checked`
inequality を記録する。missing/non-exact link、object-terminal、direct、one-edge、
longer shape は fail closed とし、mode declaration acceptance/inhabitation、
inequality desugaring、implicit closure/order、truth/fact、theorem acceptance、
proof、CoreIr、ControlFlowIr、VC は deferred のままである。
task 137 は builtin-object pre-desugaring inequality consumer を exact two-edge
bare-object chain
`OuterTwoEdgeObjectModeInequality -> MiddleTwoEdgeObjectModeInequality -> BaseTwoEdgeObjectModeInequality -> object`
に適用する。4 raw outer-mode input は記述された provenance を保持し、real
AST-derived expansion 3 本が terminal-RHS builtin-object identity 1 個へ normalize
してから、2 `Inferred` variable term と 1 fact-free pre-desugaring `Checked`
inequality を記録する。missing/non-exact link、set-terminal、direct、one-edge、
longer shape は fail closed とし、declaration acceptance/inhabitation、inequality
desugaring、implicit closure/order、truth/fact、theorem acceptance、proof、CoreIr、
ControlFlowIr、VC は deferred のままである。
task 138 は normalized-reflexive type-assertion consumer を exact direct bare-set
mode source
`LocalModeTypeAssertion -> set; reserve x for LocalModeTypeAssertion; theorem ...: x is set;`
に適用する。raw subject result は記述された local-mode symbol/range、asserted
builtin `set` は独立した formula source node を保持し、real AST-derived expansion
1 本を `TermFormulaChecker` に渡す。両 input は definition RHS を canonical source
とする builtin-set identity 1 個へ normalize してから、1 `Inferred` variable term
と 1 fact-free `Checked` type assertion を記録する。missing/non-exact expansion と
formula-side local-mode asserted head は fail closed とし、mode declaration
acceptance/inhabitation、general reachability/widening/`qua`、truth/fact、theorem
acceptance、proof、CoreIr、ControlFlowIr、VC は deferred のままである。
task 139 exact direct-local-mode left reserved-variable membership checker
bridge は right-only expected-`set` membership consumer を
`LocalModeMembership -> set; reserve x for LocalModeMembership; reserve y for set; theorem ...: x in y;`
に適用する。raw left result は written local-mode symbol/range、right result と
sole expected-`set` input は独立した explicit reserve range を保持する。real
AST-derived expansion 1 本は left result を normalize し、right builtin-set role
2 個は直接 normalize され、3 role は earlier definition RHS を canonical source
とする builtin-set identity 1 個へ intern される。production validation は
`BindingId(0/1)`、2 `Inferred` variable、1 fact-free `Checked` membership、right
所有の expected constraint 1 個だけ、left expected input なしを要求する。
independent expansion と right expected-`set` corruption test は task-specific
invalid-payload key を報告する。missing/non-exact mode/reserve/formula は fail
closed とし、mode declaration acceptance/inhabitation、membership truth/fact、
implicit closure/order、theorem acceptance、proof、CoreIr、ControlFlowIr、VC は
deferred のままである。
task 140 exact direct local-object-mode left reserved-variable membership
checker bridge は task 125 の right-only expected-`set`、two-binding membership
consumer を
`LocalObjectModeMembership -> object; reserve x for LocalObjectModeMembership; reserve y for set; theorem ...: x in y;`
に適用する。raw left result は written local object-mode symbol/range、right
result と sole expected-`set` input は独立した explicit reserve range を保持する。
real AST-derived expansion 1 本は left result を definition RHS を canonical
source とする builtin-object identity 1 個へ normalize し、right role 2 個は
explicit reserve 起点の distinct builtin-set identity 1 個へ直接 normalize される。
production validation は `BindingId(0/1)`、2 `Inferred` variable、1 fact-free
`Checked` membership、right 所有の expected constraint 1 個だけ、left expected
input なし、object/set coercion なしを要求する。independent expansion と right
expected-`set` corruption test は task-specific invalid-payload key を報告する。
missing/non-exact mode/reserve/formula は fail closed とし、mode declaration
acceptance/inhabitation、membership truth/fact、implicit closure/order、theorem
acceptance、proof、CoreIr、ControlFlowIr、VC は deferred のままである。
task 141 exact one-edge local-mode-chain left reserved-variable membership
checker bridge は task 139 の right-only expected-`set`、two-binding membership
consumer を
`ChainModeMembership -> BaseModeMembership -> set; reserve x for ChainModeMembership; reserve y for set; theorem ...: x in y;`
に適用する。raw left result は written outer-mode symbol/range、right result と
sole expected-`set` input は独立した explicit reserve range を保持する。real
AST-derived expansion 2 本は left result を terminal `set` RHS を canonical
source とする builtin-set identity 1 個へ recursive に normalize し、right role
2 個は直接 normalize されて同じ identity へ intern する。production validation
は `BindingId(0/1)`、2 `Inferred` variable、1 fact-free `Checked` membership、
right 所有の expected constraint 1 個だけ、left expected input なしを要求する。
chain link 2 本と right expected-`set` projection の独立した corruption は
task-specific invalid-payload key を報告する。missing/non-exact
mode/reserve/formula は fail closed とし、mode declaration
acceptance/inhabitation、membership truth/fact、implicit closure/order、theorem
acceptance、proof、CoreIr、ControlFlowIr、VC は deferred のままである。
task 142 exact one-edge local-object-mode-chain left reserved-variable
membership checker bridge は同じ right-only expected-`set`、two-binding
membership consumer を
`ChainObjectModeMembership -> BaseObjectModeMembership -> object; reserve x for ChainObjectModeMembership; reserve y for set; theorem ...: x in y;`
に適用する。raw left result は written outer-mode symbol/range、right result と
sole expected-`set` input は独立した explicit reserve range を保持する。real
AST-derived expansion 2 本は left result を terminal `object` RHS を canonical
source とする builtin-object identity 1 個へ recursive に normalize し、right
role 2 個は直接 distinct explicit-reserve-anchored builtin-set identity 1 個へ
normalize される。production validation は `BindingId(0/1)`、2 `Inferred`
variable、1 fact-free `Checked` membership、right 所有の expected constraint 1
個だけ、left expected input なし、object/set coercion なしを要求する。chain
link 2 本と right expected-`set` projection の独立した corruption は
task-specific invalid-payload key を報告しなければならない。missing/non-exact
mode/reserve/formula は fail closed とし、mode declaration
acceptance/inhabitation、membership truth/fact、implicit closure/order、theorem
acceptance、proof、CoreIr、ControlFlowIr、VC は deferred のままである。
task 143 は exact two-edge set-terminal local-mode-chain left reserved-variable
membership bridge を規定する。source は unique、unrecovered、source-preceding、
no-argument/attribute definition 3 個
`OuterTwoEdgeModeMembership -> MiddleTwoEdgeModeMembership -> BaseTwoEdgeModeMembership -> set`、
outer mode の `x` と explicit `set` の `y` から成る ordered reserve、
`TwoEdgeLocalModeReservedVariableMembershipPayloadBoundary: x in y;` を持つ。
raw left result は outer symbol/range、right result と sole expected-set input は
独立した explicit reserve range を保持する。real Task 72 expansion 3 本が left
role を再帰的に normalize し、right builtin-set role は直接 normalize され、3
role すべてが terminal-RHS-anchored builtin-set identity 1 個へ intern する。
exact contract は `BindingId(0/1)`、2 `Inferred` term、1 fact-free `Checked`
membership、right-owned expected constraint 1 個だけ、left expected type なしを
要求する。各 definition label、両 chain radix、expansion entry 3 本、right
expected-set projection を独立に guard する。missing/non-exact definition、
reserve、formula は fail closed する。mode declaration acceptance/inhabitation、
membership truth/fact、implicit closure/order、theorem acceptance、proof、
CoreIr、ControlFlowIr、VC は deferred のままである。
task 144 は exact two-edge object-terminal local-mode-chain left
reserved-variable membership bridge を規定する。source は unique、unrecovered、
source-preceding、no-argument/attribute definition 3 個
`OuterTwoEdgeObjectModeMembership -> MiddleTwoEdgeObjectModeMembership -> BaseTwoEdgeObjectModeMembership -> object`、
outer mode の `x` と explicit `set` の `y` から成る ordered reserve、
`TwoEdgeLocalObjectModeReservedVariableMembershipPayloadBoundary: x in y;` を
持つ。raw left result は outer symbol/range、right result と sole expected-set
input は独立した explicit reserve range を保持する。real Task 72 expansion 3
本が left role を terminal-RHS-anchored builtin-object identity 1 個へ再帰的に
normalize し、right builtin-set role は直接 distinct explicit-reserve-anchored
identity 1 個へ normalize される。exact contract は `BindingId(0/1)`、2
`Inferred` term、1 fact-free `Checked` membership、right-owned expected
constraint 1 個だけ、left expected type なし、object/set coercion なしを要求する。
各 definition label、両 chain radix、expansion entry 3 本、right expected-set
projection を独立に guard する。missing/non-exact definition、reserve、formula
は fail closed する。mode declaration acceptance/inhabitation、membership
truth/fact、implicit closure/order、theorem acceptance、proof、CoreIr、
ControlFlowIr、VC は deferred のままである。
task 145 は exact direct bare-object local-mode reserved-variable
normalized-reflexive type assertion bridge
`LocalObjectModeTypeAssertion -> object; reserve x for LocalObjectModeTypeAssertion; theorem ...: x is object;`
を規定する。raw subject result は written local-mode symbol/range、asserted
builtin `object` は独立した formula source node を保持する。real Task 55
expansion 1 本を `TermFormulaChecker` に渡し、両 input を definition RHS を
canonical source とする builtin-object identity 1 個へ normalize してから、1
`Inferred` variable term と 1 fact-free `Checked` type assertion を記録する。
exact contract は `BindingId(0)` と source-order use ordinal 1 を要求する。
definition label と expansion entry を独立に guard し、missing/non-exact
definition、reserve、formula は fail closed する。mode declaration
acceptance/inhabitation、formula-side local-mode asserted-head extraction、
general reachability/widening/`qua`、object/set coercion、truth/fact、theorem
acceptance、proof、CoreIr、ControlFlowIr、VC は deferred のままである。
task 146 は exact one-edge bare-set local-mode-chain reserved-variable
normalized-reflexive type assertion bridge
`BaseModeTypeAssertion -> set; ChainModeTypeAssertion -> BaseModeTypeAssertion; reserve x for ChainModeTypeAssertion; theorem ...: x is set;`
を規定する。raw subject result は written outer-mode symbol/range、asserted
builtin `set` は独立した formula source node を保持する。real Task 56
expansion 2 本を `TermFormulaChecker` に渡し、両 input を terminal definition
RHS を canonical source とする builtin-set identity 1 個へ再帰的に normalize
してから 1 `Inferred` variable term と 1 fact-free `Checked` type assertion を
記録する。exact contract は `BindingId(0)` と source-order use ordinal 1 を
要求する。両 definition label、chain radix、expansion entry 2 本を独立に guard
し、missing/non-exact definition、reserve、formula は fail closed する。mode
declaration acceptance/inhabitation、formula-side local-mode asserted-head
extraction、general reachability/widening/`qua`、truth/fact、theorem acceptance、
proof、CoreIr、ControlFlowIr、VC は deferred のままである。
task 147 は exact one-edge bare-object local-mode-chain reserved-variable
normalized-reflexive type assertion bridge
`BaseObjectModeTypeAssertion -> object; ChainObjectModeTypeAssertion -> BaseObjectModeTypeAssertion; reserve x for ChainObjectModeTypeAssertion; theorem ...: x is object;`
を規定する。raw subject result は written outer-mode symbol/range、asserted
builtin `object` は独立した formula source node を保持する。real Task 56
expansion 2 本を `TermFormulaChecker` に渡し、両 input を terminal definition
RHS を canonical source とする builtin-object identity 1 個へ再帰的に
normalize してから 1 `Inferred` variable term と 1 fact-free `Checked` type
assertion を記録する。exact contract は `BindingId(0)` と source-order use
ordinal 1 を要求する。両 definition label、chain radix、expansion entry 2 本を
独立に guard し、missing/non-exact definition、reserve、formula は fail closed
する。mode declaration acceptance/inhabitation、formula-side local-mode
asserted-head extraction、general reachability/widening/`qua`、object/set
coercion、truth/fact、theorem acceptance、proof、CoreIr、ControlFlowIr、VC は
deferred のままである。
task 148 は exact two-edge bare-set local-mode-
chain reserved-variable normalized-reflexive type assertion bridge
`BaseTwoEdgeModeTypeAssertion -> set; MiddleTwoEdgeModeTypeAssertion -> BaseTwoEdgeModeTypeAssertion; OuterTwoEdgeModeTypeAssertion -> MiddleTwoEdgeModeTypeAssertion; reserve x for OuterTwoEdgeModeTypeAssertion; theorem ...: x is set;`
だけを規定する。raw subject result は written outer-mode symbol/range、
asserted builtin `set` は独立した formula source node を保持する。real task
72 expansion 3 本を `TermFormulaChecker` に渡し、両 input を terminal
definition RHS を canonical source とする builtin-set identity 1 個へ再帰的に
normalize してから 1 `Inferred` variable term と 1 fact-free `Checked` type
assertion を記録する。exact contract は `BindingId(0)` と source-order use
ordinal 1 を要求する。definition label 3 個、chain radix 2 個、expansion entry
3 本を独立に guard し、missing/non-exact definition、reserve、formula は fail
closed する。mode declaration acceptance/inhabitation、formula-side local-mode
asserted-head extraction、general reachability/widening/`qua`、truth/fact、
theorem acceptance、proof、CoreIr、ControlFlowIr、VC は deferred のままである。
task 149 は exact two-edge bare-object local-mode-chain reserved-variable
normalized-reflexive type assertion bridge
`BaseTwoEdgeObjectModeTypeAssertion -> object; MiddleTwoEdgeObjectModeTypeAssertion -> BaseTwoEdgeObjectModeTypeAssertion; OuterTwoEdgeObjectModeTypeAssertion -> MiddleTwoEdgeObjectModeTypeAssertion; reserve x for OuterTwoEdgeObjectModeTypeAssertion; theorem ...: x is object;`
だけを規定する。raw subject result は written outer-mode symbol/range、asserted
builtin `object` は独立した formula source node を保持する。real task 72
expansion 3 本を `TermFormulaChecker` に渡し、両 input を terminal definition
RHS を canonical source とする builtin-object identity 1 個へ再帰的に normalize
してから 1 `Inferred` variable term と 1 fact-free `Checked` type assertion を
記録する。exact contract は `BindingId(0)` と source-order use ordinal 1 を
要求する。definition label 3 個、chain radix 2 個、expansion entry 3 本を独立に
guard し、missing/non-exact definition、reserve、formula は fail closed する。
mode declaration acceptance/inhabitation、formula-side local-mode asserted-head
extraction、general reachability/widening/`qua`、object/set coercion、truth/fact、
theorem acceptance、proof、CoreIr、ControlFlowIr、VC は deferred のままである。
exact source guard、独立した definition/three-link corruption、real frontend/
resolver sidecar が production route を保護する。
task 150 は exact three-edge bare-set local-mode-chain reserved-variable
normalized-reflexive type assertion bridge
`BaseThreeEdgeModeTypeAssertion -> set; InnerThreeEdgeModeTypeAssertion -> BaseThreeEdgeModeTypeAssertion; MiddleThreeEdgeModeTypeAssertion -> InnerThreeEdgeModeTypeAssertion; OuterThreeEdgeModeTypeAssertion -> MiddleThreeEdgeModeTypeAssertion; reserve x for OuterThreeEdgeModeTypeAssertion; theorem ...: x is set;`
だけを規定する。raw subject result は written outer-mode symbol/range、asserted
builtin `set` は独立した formula source node を保持する。real task 73
expansion 4 本を `TermFormulaChecker` に渡し、両 input を terminal definition
RHS を canonical source とする builtin-set identity 1 個へ再帰的に normalize
してから 1 `Inferred` variable term と 1 fact-free `Checked` type assertion を
記録する。exact contract は `BindingId(0)` と source-order use ordinal 1 を
要求する。definition label 4 個、chain radix 3 個、expansion entry 4 本を独立に
guard し、missing/non-exact definition、reserve、formula は fail closed する。
mode declaration acceptance/inhabitation、formula-side local-mode asserted-head
extraction、general reachability/widening/`qua`、truth/fact、theorem acceptance、
proof、CoreIr、ControlFlowIr、VC は deferred のままである。exact source guard、
独立した definition/four-link corruption、real frontend/resolver sidecar で
active production route を保護する。
task 151 は exact three-edge bare-object local-mode-chain reserved-variable
normalized-reflexive type assertion bridge
`BaseThreeEdgeObjectModeTypeAssertion -> object; InnerThreeEdgeObjectModeTypeAssertion -> BaseThreeEdgeObjectModeTypeAssertion; MiddleThreeEdgeObjectModeTypeAssertion -> InnerThreeEdgeObjectModeTypeAssertion; OuterThreeEdgeObjectModeTypeAssertion -> MiddleThreeEdgeObjectModeTypeAssertion; reserve x for OuterThreeEdgeObjectModeTypeAssertion; theorem ...: x is object;`
だけを規定する。raw subject result は written outer-mode symbol/range、asserted
builtin `object` は独立した formula source node を保持する。real task 73
expansion 4 本を `TermFormulaChecker` に渡し、両 input を terminal definition
RHS を canonical source とする builtin-object identity 1 個へ再帰的に normalize
してから 1 `Inferred` variable term と 1 fact-free `Checked` type assertion を
記録する。exact contract は `BindingId(0)` と source-order use ordinal 1 を
要求する。definition label 4 個、chain radix 3 個、expansion entry 4 本を独立に
guard し、missing/non-exact definition、reserve、formula は fail closed する。
mode declaration acceptance/inhabitation、formula-side local-mode asserted-head
extraction、general reachability/widening/`qua`、object/set coercion、truth/fact、
theorem acceptance、proof、CoreIr、ControlFlowIr、VC は deferred のままである。
exact source guard、独立した definition/four-link corruption、real frontend/
resolver sidecar で active production route を保護する。
task 152 は exact four-edge bare-set local-mode-chain reserved-variable
normalized-reflexive type assertion bridge
`BaseFourEdgeModeTypeAssertion -> set; InnerFourEdgeModeTypeAssertion -> BaseFourEdgeModeTypeAssertion; MiddleFourEdgeModeTypeAssertion -> InnerFourEdgeModeTypeAssertion; OuterFourEdgeModeTypeAssertion -> MiddleFourEdgeModeTypeAssertion; TooDeepFourEdgeModeTypeAssertion -> OuterFourEdgeModeTypeAssertion; reserve x for TooDeepFourEdgeModeTypeAssertion; theorem ...: x is set;`
だけを規定する。raw subject result は written outermost-mode symbol/range、
asserted builtin `set` は独立した formula source node を保持する。real task 74
expansion 5 本を `TermFormulaChecker` に渡し、両 input を terminal definition
RHS を canonical source とする builtin-set identity 1 個へ再帰的に normalize
してから 1 `Inferred` variable term と 1 fact-free `Checked` type assertion を
記録する。exact contract は `BindingId(0)` と source-order use ordinal 1 を
要求する。definition label 5 個、chain radix 4 個、expansion entry 5 本を独立に
guard し、missing/non-exact definition、reserve、formula は fail closed する。
mode declaration acceptance/inhabitation、formula-side local-mode asserted-head
extraction、general reachability/widening/`qua`、truth/fact、theorem acceptance、
proof、CoreIr、ControlFlowIr、VC は deferred のままである。exact source guard、
独立した definition/five-link corruption、real frontend/resolver sidecar で
active production route を保護する。
task 153 は exact four-edge bare-object local-mode-chain reserved-variable
normalized-reflexive type assertion bridge
`BaseFourEdgeObjectModeTypeAssertion -> object; InnerFourEdgeObjectModeTypeAssertion -> BaseFourEdgeObjectModeTypeAssertion; MiddleFourEdgeObjectModeTypeAssertion -> InnerFourEdgeObjectModeTypeAssertion; OuterFourEdgeObjectModeTypeAssertion -> MiddleFourEdgeObjectModeTypeAssertion; TooDeepFourEdgeObjectModeTypeAssertion -> OuterFourEdgeObjectModeTypeAssertion; reserve x for TooDeepFourEdgeObjectModeTypeAssertion; theorem ...: x is object;`
だけを規定する。raw subject result は written outermost-mode symbol/range を、
asserted builtin `object` は独立した formula source node を保持する。task 74 の
real expansion 5 本を `TermFormulaChecker` へ渡し、両 input を terminal
definition RHS に canonical anchor された builtin-object identity 1 個へ再帰的に
normalize してから、1 `Inferred` term と 1 fact-free `Checked` type assertion
を記録する。`BindingId(0)` と source-order use ordinal 1 を要求する。definition
label 5 個、chain radix 4 個、expansion entry 5 本を独立 guard し、non-exact
definition/reserve/formula は fail closed とする。declaration acceptance/
inhabitation、formula-side local asserted-head、general reachability/widening/
`qua`、object/set coercion、truth/fact、theorem acceptance、proof/Core/
ControlFlow/VC は deferred のままである。exact source guard、independent
definition/five-link corruption、real frontend/resolver sidecar で active route
を保護する。
task 154 は exact three-edge bare-set local-mode-chain reserved-variable
equality bridge
`BaseThreeEdgeModeEquality -> set; InnerThreeEdgeModeEquality -> BaseThreeEdgeModeEquality; MiddleThreeEdgeModeEquality -> InnerThreeEdgeModeEquality; OuterThreeEdgeModeEquality -> MiddleThreeEdgeModeEquality; reserve z for OuterThreeEdgeModeEquality; theorem ...: z = z;`
だけを規定する。raw result/expected input 4 個は written outer-mode source を
保持し、両 identifier use は source-order ordinal 1、2 で独立に
`BindingId(0)` へ解決する。task 73 の real expansion 4 本は全 role を terminal
RHS に canonical anchor された builtin-set identity 1 個へ再帰的に normalize
してから、2 `Inferred` variable term と 1 fact/deferred-free `Checked` equality
を記録する。全 definition label、chain radix、expansion entry を独立に guard
し、non-exact definition/reserve/formula と withheld chain/terminal shape は
fail closed とする。これは equality type/well-formedness だけであり、mode
declaration acceptance/inhabitation、equality truth/fact、closure/order、theorem
acceptance、proof/Core/ControlFlow/VC は deferred のままである。exact source
guard、独立した definition/radix/expansion corruption matrix、production route、
real frontend/resolver sidecar が active bridge を保護する。
task 155 は exact three-edge bare-object local-mode-chain reserved-variable
equality bridge
`BaseThreeEdgeObjectModeEquality -> object; InnerThreeEdgeObjectModeEquality -> BaseThreeEdgeObjectModeEquality; MiddleThreeEdgeObjectModeEquality -> InnerThreeEdgeObjectModeEquality; OuterThreeEdgeObjectModeEquality -> MiddleThreeEdgeObjectModeEquality; reserve z for OuterThreeEdgeObjectModeEquality; theorem ...: z = z;`
だけを規定する。raw result/expected input 4 個は written outer-mode source を
保持し、両 identifier use は source-order ordinal 1、2 で独立に
`BindingId(0)` へ解決する。task 73 の real expansion 4 本は全 role を terminal
RHS に canonical anchor された builtin-object identity 1 個へ再帰的に normalize
してから、2 `Inferred` variable term と 1 fact/deferred-free `Checked` equality
を記録する。全 definition label、chain radix、expansion entry を独立に guard
し、non-exact definition/reserve/formula と withheld chain/terminal shape は
fail closed とする。これは equality type/well-formedness だけであり、mode
declaration acceptance/inhabitation、object/set coercion、equality truth/fact、
closure/order、theorem acceptance、proof/Core/ControlFlow/VC は deferred の
ままである。exact source guard、独立した definition/radix/expansion corruption
matrix、production route、real frontend/resolver sidecar が active bridge を
保護する。
task 156 は exact three-edge bare-set local-mode-chain reserved-variable
inequality bridge
`BaseThreeEdgeModeInequality -> set; InnerThreeEdgeModeInequality -> BaseThreeEdgeModeInequality; MiddleThreeEdgeModeInequality -> InnerThreeEdgeModeInequality; OuterThreeEdgeModeInequality -> MiddleThreeEdgeModeInequality; reserve z for OuterThreeEdgeModeInequality; theorem ...: z <> z;`
だけを規定する。raw result/expected input 4 個は written outer-mode source を
保持し、両 identifier use は source-order ordinal 1、2 で独立に
`BindingId(0)` へ解決する。task 73 の real expansion 4 本は全 role を terminal
RHS に canonical anchor された builtin-set identity 1 個へ再帰的に normalize
してから、2 `Inferred` variable term と 1 fact/deferred-free pre-desugaring
`Checked` inequality を記録する。全 definition label、chain radix、expansion
entry を独立に guard し、non-exact definition/reserve/formula と withheld
chain/terminal shape は fail closed とする。これは inequality type/well-
formedness だけであり、mode declaration acceptance/inhabitation、inequality
desugaring、truth/fact、closure/order、theorem acceptance、proof/Core/
ControlFlow/VC は deferred のままである。test-first source、trace contract、
exact source guard、独立した definition/radix/expansion corruption matrix、
production route、real frontend/resolver sidecar が active bridge を保護する。
task 157 は exact three-edge bare-object local-mode-chain reserved-variable
inequality bridge
`BaseThreeEdgeObjectModeInequality -> object; InnerThreeEdgeObjectModeInequality -> BaseThreeEdgeObjectModeInequality; MiddleThreeEdgeObjectModeInequality -> InnerThreeEdgeObjectModeInequality; OuterThreeEdgeObjectModeInequality -> MiddleThreeEdgeObjectModeInequality; reserve z for OuterThreeEdgeObjectModeInequality; theorem ...: z <> z;`
だけを規定する。raw result/expected input 4 個は written outer-mode source を
保持し、両 identifier use は source-order ordinal 1、2 で独立に
`BindingId(0)` へ解決する。task 73 の real expansion 4 本は全 role を terminal
RHS に canonical anchor された builtin-object identity 1 個へ再帰的に
normalize してから、2 `Inferred` variable term と 1 fact/deferred-free pre-
desugaring `Checked` inequality を記録する。全 definition label、chain radix、
expansion entry を独立に guard し、non-exact definition/reserve/formula と
withheld chain/terminal shape は fail closed とする。これは inequality type/
well-formedness だけであり、mode declaration acceptance/inhabitation、object/
set coercion、inequality desugaring、truth/fact、closure/order、theorem
acceptance、proof/Core/ControlFlow/VC は deferred のままである。source と
trace contract、production route、独立した corruption matrix、real frontend/
resolver sidecar が active bridge を保護する。
task 158 は exact three-edge bare-set local-mode-chain left reserved-variable
membership bridge
`BaseThreeEdgeModeMembership -> set; InnerThreeEdgeModeMembership -> BaseThreeEdgeModeMembership; MiddleThreeEdgeModeMembership -> InnerThreeEdgeModeMembership; OuterThreeEdgeModeMembership -> MiddleThreeEdgeModeMembership; reserve x for OuterThreeEdgeModeMembership; reserve y for set; theorem ...: x in y;`
だけを規定する。raw left result は written outer-mode source、right result と
sole expected input は独立した explicit-set reserve source を保持し、left
expected input は持たない。checker は `x/y` を source-order ordinal 2/3 で
独立に `BindingId(0/1)` へ解決する。task 73 の real expansion 4 本は 3 role
を terminal RHS に canonical anchor された builtin-set identity 1 個へ normalize
してから、2 `Inferred` variable term と exactly one right-owned expected-type
constraint を持つ 1 fact/deferred-free `Checked` membership を記録する。全
definition label/radix/expansion entry を独立に guard し、non-exact chain、
terminal、reserve、formula、withheld shape は fail closed とする。mode
declaration acceptance/inhabitation、membership truth/fact、implicit closure/
order、theorem acceptance、proof/Core/ControlFlow/VC、object-terminal behavior、
broader chain depth は deferred のままである。source/trace contract、
production route、独立した corruption matrix、real frontend/resolver sidecar
が active bridge を保護する。
task 159 は `reserve x, y for set; theorem
DistinctReservedVariableMembershipPayloadBoundary: x in y;` だけを規定する。
reserve item 1 個は shared written set range 1 個を持つ distinct
`BindingId(0/1)` を作り、independent lookup は ordinal 2/3 を使う。left/right
result と sole right expected role はその range を保持し、left expected role は
なく、checker は normalized builtin-set identity 1 個、2 `Inferred` variable、
right-owned constraint 1 個を持つ 1 fact/deferred-free `Checked` membership を
記録する。exact source と matched-output corruption guard を必須とする。
membership truth/fact、closure/order、theorem acceptance、proof/Core/VC、
separate declaration、broader shape は deferred のままである。source/trace
contract、production route、独立した corruption matrix、real frontend/resolver
sidecar が active bridge を保護する。
task 161 は source `reserve x for set; reserve y for set; theorem
MultipleReserveDeclarationInequalityPayloadBoundary: x <> y;` だけを規定する。
reserve item 2 個は distinct written set range を持つ distinct `BindingId(0/1)` と
ordinal 2/3 の独立 lookup を生成しなければならない。各 operand result/expected
role は binding 固有の range を保持しながら全 4 role を earlier `x` range に
canonical anchor された builtin-set identity 1 個へ normalize し、2 `Inferred`
variable と 2 ordered constraint を持つ 1 fact/deferred-free pre-desugaring
`Checked` inequality を生成する。exact source、route order、matched output、
corruption guard を必須とする。desugaring/truth/fact、closure/order、theorem
acceptance、proof/Core/VC、shared range、broader shape は deferred のままである。
source/trace contract、production route、corruption coverage、real sidecar が
active bridge を保護する。
task 162 は active source `reserve x for set; reserve y for set; theorem
MultipleReserveDeclarationMembershipPayloadBoundary: x in y;` だけを規定する。
reserve item 2 個は distinct written set range を持つ distinct `BindingId(0/1)` と
ordinal 2/3 の独立 lookup を生成しなければならない。left result は first range、
right result と sole right expected role は second range を保持し、left expected
role は持たない。3 role は earlier `x` range に canonical anchor された builtin-
set identity 1 個へ normalize されてから、2 `Inferred` variable と exactly one
right-owned constraint を持つ 1 fact/deferred-free `Checked` membership を生成する。
exact source、route order、matched output、corruption guard を必須とする。
membership truth/fact、closure/order、theorem acceptance、proof/Core/VC、shared
range、broader shape は deferred のままである。production route、独立した
corruption/near-miss coverage、real frontend/resolver sidecar が contract を実装し
guard するため active count は 113 件である。
task 163 は unique / unrecovered / same-module / argument-free / source-
preceding な mode definition 4 個が
`OuterThreeEdgeObjectModeMembership -> MiddleThreeEdgeObjectModeMembership ->
InnerThreeEdgeObjectModeMembership -> BaseThreeEdgeObjectModeMembership ->
object` を形成し、outer mode の `reserve x`、`reserve y for set`、
`ThreeEdgeLocalObjectModeReservedVariableMembershipPayloadBoundary: x in y;`
が続く active source だけを規定する。実装は ordinal 2/3 の
`BindingId(0/1)`、raw outer-mode left result、独立した explicit-set right result /
sole expected input、no left expected input を保持し、real AST-derived expansion
4 本をすべて消費して distinct terminal-object-RHS / explicit-set identity へ
normalize しなければならない。その後 2 `Inferred` variable と exactly one
right-owned constraint を持つ 1 fact/deferred-free `Checked` membership を要求する。
exact route-order、near-miss、matched-output、expansion corruption、real
frontend/resolver sidecar guard が active route を保護する。membership truth/fact、object/set
coercion、closure/order、theorem acceptance、proof/Core/VC、他の chain depth、
broader shape は deferred のままであり、active runner は 114 件である。
task 164 は unique / unrecovered / same-module / argument-free / source-
preceding な mode definition 5 個が `TooDeepFourEdgeModeMembership ->
OuterFourEdgeModeMembership -> MiddleFourEdgeModeMembership ->
InnerFourEdgeModeMembership -> BaseFourEdgeModeMembership -> set` を形成し、
outermost mode の `reserve x`、`reserve y for set`、
`FourEdgeLocalModeReservedVariableMembershipPayloadBoundary: x in y;` が続く
active source だけを規定する。実装は ordinal 2/3 の
`BindingId(0/1)`、raw outermost-mode left provenance、独立した explicit-set
right result/sole expected provenance、no left expected input を保持し、real
AST-derived expansion 5 本をすべて消費して 3 type role を terminal-set-RHS
identity 1 個へ normalize しなければならない。その後 2 `Inferred`
variable と exactly one right-owned constraint を持つ 1 fact/deferred-free
`Checked` membership を要求する。exact route-order、near-miss、matched-
output、expansion corruption、real frontend/resolver sidecar guard は必須で
ある。truth/fact、closure/order、theorem acceptance、proof/Core/VC、object-
terminal behavior、他 chain depth、broader shape は deferred のままである。
production route、full corruption/near-miss coverage、real sidecar が active
runner 115 を保護する。
task 165 は unique / unrecovered / same-module / argument-free / source-
preceding な mode definition 5 個が
`TooDeepFourEdgeObjectModeMembership -> OuterFourEdgeObjectModeMembership ->
MiddleFourEdgeObjectModeMembership -> InnerFourEdgeObjectModeMembership ->
BaseFourEdgeObjectModeMembership -> object` を成し、outermost mode の
`reserve x`、`reserve y for set`、
`FourEdgeLocalObjectModeReservedVariableMembershipPayloadBoundary: x in y;`
を続ける active source だけを規定する。実装は ordinal 2/3 で `x/y` を
`BindingId(0/1)` へ解決し、raw outermost-mode left provenance と独立した
explicit-set right result / sole expected provenance を保持し、left expected
input を持たず、real AST-derived expansion 5 本を消費して distinct terminal-
object-RHS / explicit-set identity へ normalize しなければならない。2
`Inferred` variable、exactly one right-owned constraint を持つ 1 fact/deferred-
free `Checked` membership を要求する。membership truth/fact、object/set
coercion、closure/order、theorem acceptance、proof/Core/VC、他 chain depth、
broader shape は deferred のままとする。fixture、expectation、trace backlink
6 件、exact route-order、near-miss、matched-output、expansion-corruption、real
frontend/resolver sidecar guard が active runner 116 を保護する。
task 166 は unique / unrecovered / same-module / argument-free / source-
preceding な mode definition 5 個が `TooDeepFourEdgeModeEquality ->
OuterFourEdgeModeEquality -> MiddleFourEdgeModeEquality ->
InnerFourEdgeModeEquality -> BaseFourEdgeModeEquality -> set` を成し、
outermost mode の `reserve z` と
`FourEdgeLocalModeReservedVariableEqualityPayloadBoundary: z = z;` を続ける
active source だけを規定する。実装は両 use を ordinal 1/2 の
`BindingId(0)` へ解決し、raw result/expected input 4 個を保持し、real
expansion 5 本を消費して全 role を terminal-set-RHS identity 1 個へ normalize
しなければならない。その後 2 `Inferred` variable、1 fact/deferred-free
`Checked` equality、ordered operand-owned expected constraint 2 個を要求する。
exact route-order、definition/link/depth ごとの
near miss、matched-output/expansion corruption、real frontend/resolver sidecar
を必須とする。declaration acceptance/inhabitation、equality truth/fact、
closure/order、theorem acceptance、proof/Core/VC、object-terminal behavior、
他 depth、broader shape は deferred のままである。fixture/expectation、trace
backlink 6 件、exact routing、full near-miss/corruption matrix、real frontend/
resolver sidecar が active runner 117 を保護する。
task 167 は unique / unrecovered / same-module / argument-free / source-
preceding な mode definition 5 個が
`TooDeepFourEdgeObjectModeEquality -> OuterFourEdgeObjectModeEquality ->
MiddleFourEdgeObjectModeEquality -> InnerFourEdgeObjectModeEquality ->
BaseFourEdgeObjectModeEquality -> object` を成し、outermost mode の
`reserve z` と
`FourEdgeLocalObjectModeReservedVariableEqualityPayloadBoundary: z = z;` を
続ける active source だけを規定する。実装は両 use を ordinal 1/2 の
`BindingId(0)` へ解決し、raw result/expected input 4 個を保持し、real
expansion 5 本を消費して全 role を terminal-object-RHS identity 1 個へ
normalize しなければならない。その後 2 `Inferred` variable、1 fact/
deferred-free `Checked` equality、ordered operand-owned expected constraint 2
個を object/set coercion なしで要求する。exact route-order、definition/link/
depth ごとの near miss、matched-output/expansion corruption、real frontend/
resolver sidecar を必須とする。declaration acceptance/inhabitation、equality
truth/fact、closure/order、theorem acceptance、proof/Core/VC、set-terminal
behavior、他 depth、broader shape は deferred のままである。fixture/
expectation、trace backlink 6 件、exact routing、full near-miss/corruption
matrix、real frontend/resolver sidecar が active runner 118 を保護する。
task 168 は unique / unrecovered / same-module / argument-free / source-
preceding な mode definition 5 個が
`TooDeepFourEdgeModeInequality -> OuterFourEdgeModeInequality ->
MiddleFourEdgeModeInequality -> InnerFourEdgeModeInequality ->
BaseFourEdgeModeInequality -> set` を成し、outermost mode の `reserve z` と
`FourEdgeLocalModeReservedVariableInequalityPayloadBoundary: z <> z;` を続ける
test-first source だけを規定する。実装は両 use を ordinal 1/2 の
`BindingId(0)` へ解決し、raw result/expected input 4 個を保持し、real
expansion 5 本を消費して全 role を terminal-set-RHS identity 1 個へ normalize
しなければならない。その後 2 `Inferred` variable、1 fact/deferred-free
pre-desugaring `Checked` inequality、ordered operand-owned expected constraint 2
個を要求する。exact route-order、near-miss、matched-output、expansion-
corruption、real sidecar guard を必須とする。declaration acceptance/
inhabitation、inequality desugaring/truth/fact、closure/order、theorem
acceptance、proof/Core/VC、object-terminal behavior、他 depth、broader shape は
deferred のままである。fixture/expectation、trace backlink 6 件、exact routing、
full near-miss/corruption coverage、real frontend/resolver sidecar が active
runner 119 を保護する。
task 169 は unique / unrecovered / same-module / argument-free / source-
preceding な mode definition 5 個が
`TooDeepFourEdgeObjectModeInequality -> OuterFourEdgeObjectModeInequality ->
MiddleFourEdgeObjectModeInequality -> InnerFourEdgeObjectModeInequality ->
BaseFourEdgeObjectModeInequality -> object` を成し、outermost mode の
`reserve z` と
`FourEdgeLocalObjectModeReservedVariableInequalityPayloadBoundary: z <> z;` を
続ける test-first source だけを規定する。実装は両 use を ordinal 1/2 の
`BindingId(0)` へ解決し、raw result/expected input 4 個を保持し、real
expansion 5 本を消費して全 role を terminal-object-RHS identity 1 個へ
normalize しなければならない。その後 2 `Inferred` variable、1 fact/
deferred-free pre-desugaring `Checked` inequality、ordered operand-owned
expected constraint 2 個を object/set coercion なしで要求する。exact route-
order、near-miss、matched-output、expansion-corruption、real sidecar guard を
必須とする。declaration acceptance/inhabitation、inequality desugaring/truth/
fact、closure/order、theorem acceptance、proof/Core/VC、set-terminal behavior、
他 depth、broader shape は deferred のままである。fixture/expectation、trace
backlink 6 件、exact routing、full near-miss/corruption coverage、real frontend/
resolver sidecar が active runner 120 を保護する。
task 172 が規定するのは、unique / unrecovered / same-module / argument-free /
source-preceding な mode definition 7 本が `ChainMode6 -> ChainMode5 ->
ChainMode4 -> ChainMode3 -> ChainMode2 -> ChainMode1 -> BaseMode -> set` を
形成し、その後に `reserve z for ChainMode6;` と
`LongLocalModeReservedVariableEqualityPayloadBoundary: z = z;` が続く exact
test-first source だけである。implementation は real AST-derived expansion
7 本すべてを消費し、両方の use を ordinal 1/2 の `BindingId(0)` に解決し、
raw `ChainMode6` result/expected input 4 個を保持して、全 role を terminal
`BaseMode` builtin-set RHS に anchor された identity 1 個へ normalize する。
2 `Inferred` variable、1 fact/deferred-free `Checked` equality、ordered
operand-owned expected constraint 2 個を要求する。exact route order、definition/
link/order/depth/recovery/context/parameterization ごとの near miss、matched-
output / expansion corruption、real frontend/resolver sidecar を必須とする。
declaration acceptance/inhabitation、equality truth/fact、closure/order、theorem
acceptance、formula-statement/proof/Core/ControlFlow/VC payload、imported/
attributed/argument-bearing または別 chain shape、general unbounded semantics
は deferred のままである。fixture、expectation、trace backlink 6 件、
production route、full near-miss/corruption matrix、real frontend/resolver
sidecar が active runner 121 を保護する。
task 173 は同じ definition 7 本と `reserve z for ChainMode6;` の後に
`LongLocalModeReservedVariableInequalityPayloadBoundary: z <> z;` だけが続く
exact sibling source を規定する。real expansion 7 本すべてと同じ raw
provenance、ordinal 1/2 の `BindingId(0)`、terminal `BaseMode` RHS identity、
2 `Inferred` variable、ordered operand-owned expected constraint 2 個を保持
してから、1 fact/deferred-free pre-desugaring `Checked` inequality を生成する。
task 172 と同じ exact/near-miss/corruption/real-sidecar guard breadth を必須と
する。inequality desugaring/truth/fact、acceptance、closure/order、theorem/
proof/Core/ControlFlow/VC、別 chain shape、general unbounded semantics は
deferred のままである。
fixture、expectation、backlink 6 件、exact routing、shared full guard matrix、
task 173 corruption test、real sidecar が active runner 122 を保護する。
task 174 は同じ definition 7 本、ordered `reserve x for ChainMode6;` と
`reserve y for set;`、その後に
`LongLocalModeReservedVariableMembershipPayloadBoundary: x in y;` だけが続く
exact membership sibling を規定する。production route は real expansion 7 本
すべてを消費し、raw `ChainMode6` left result と独立した explicit-set right
result/sole right expected input を保持する。ordinal 2/3 の `BindingId(0/1)`、
terminal `BaseMode` RHS builtin-set identity 1 個へ 3 role すべてを normalize
し、left expected input は存在させず、2 `Inferred` variable、right-owned
constraint 1 個、1 fact/deferred-free `Checked` membership を記録する。task 172
の full structural guard matrix と task 174 membership-specific matched-output
corruption/real-sidecar test を必須とする。membership truth/fact、acceptance、
closure/order、theorem/proof/Core/ControlFlow/VC、別 chain shape、general
unbounded semantics は deferred のままである。fixture、expectation、backlink
6 件、exact routing、shared full structural guard、membership-specific corruption
test、real sidecar が active runner 123 を保護する。
task 175 は同じ definition 7 本、`reserve x for ChainMode6;`、その後に
`LongLocalModeReservedVariableTypeAssertionPayloadBoundary: x is set;` だけが
続く exact normalized-reflexive type-assertion sibling を規定する。production
route は real expansion 7 本すべてを消費し、raw `ChainMode6` subject result と
独立した formula-side builtin-set asserted input を保持する。ordinal 1 の
`BindingId(0)`、terminal `BaseMode` RHS builtin-set identity 1 個へ両 role を
normalize し、general reachability を用いず 1 `Inferred` variable と 1 fact/
deferred-free `Checked` type assertion を記録する。task 172 の full structural
guard matrix と task 175 type-assertion-specific matched-output corruption/real-
sidecar test を必須とする。widening/`qua`、assertion truth/fact、acceptance、
closure/order、theorem/proof/Core/ControlFlow/VC、別 chain shape、general
unbounded semantics は deferred のままである。test-first fixture、expectation、
backlink 7 件、production routing、type-assertion-specific corruption coverage、
real sidecar が active runner 124 を保護する。
task 176 は
`LongLocalObjectModeReservedVariableEqualityPayloadBoundary: z = z;` で終わる
exact builtin-object-terminal definition chain 7 本の sibling だけを規定する。
production route は task 74 の AST-bounded source producer と task 167
の equality consumer を合成し、real expansion 7 本を消費して raw
`ChainObjectMode6` result/expected input 4 個、ordinal 1/2 の `BindingId(0)` を
保持し、全 role を terminal `BaseObjectMode` RHS builtin-object identity 1 個へ
normalize する。object/set coercion なしで 2 `Inferred` term、ordered operand-
owned constraint 2 個、1 fact/deferred-free `Checked` equality を記録する。
task 172 の shared seven-definition structural guard pattern、task 176 の object-
terminal/matched-output corruption、real-sidecar test を必須とする。truth/fact、
acceptance、closure/order、theorem/proof/Core/ControlFlow/VC、別 chain shape、
general semantics は deferred のままである。test-first fixture、expectation、
backlink 6 件、production routing、object-specific corruption coverage、real
sidecar が active runner 125 を保護する。
task 177 は
`LongLocalObjectModeReservedVariableInequalityPayloadBoundary: z <> z;` で終わる
matching exact builtin-object-terminal definition chain 7 本の sibling だけを規定
する。production route は task 74 の AST-bounded source producer と task 169 の
pre-desugaring inequality consumer を合成し、real expansion 7 本、raw
`ChainObjectMode6` result/expected input 4 個、ordinal 1/2 の `BindingId(0)` を
保持し、全 role を terminal `BaseObjectMode` RHS builtin-object identity 1 個へ
normalize する。object/set coercion なしで 2 `Inferred` term、
ordered operand-owned constraint 2 個、1 fact/deferred-free pre-desugaring
`Checked` inequality を記録する。task 172 の shared seven-definition structural
guard pattern、task 177 の object-terminal/matched-output corruption、real-sidecar
test を必須とする。inequality desugaring、truth/fact、acceptance、closure/order、
theorem/proof/Core/ControlFlow/VC、別 chain shape、general semantics は deferred
のままである。test-first fixture、expectation、backlink 6 件、production
routing、object-specific corruption coverage、real sidecar が active runner 126
を保護する。
task 178 は
`LongLocalObjectModeReservedVariableMembershipPayloadBoundary: x in y;` で終わる
matching exact builtin-object-terminal definition chain 7 本の left-membership
sibling だけを規定する。production route は task 74 の AST-bounded chain producer
と task 165 の object-left/set-right membership consumer を合成し、real expansion
7 本、raw `ChainObjectMode6` left result、独立した explicit-set right result/sole
right expected input、ordinal 2/3 の `BindingId(0/1)` を保持し、distinct terminal
`BaseObjectMode` RHS builtin-object identity と explicit-set identity へ normalize
しなければならない。left expected input なし、2 `Inferred` term、right-owned
constraint 1 個、object/set coercion を用いない 1 fact/deferred-free `Checked`
membership を記録する。task 172 の shared seven-definition structural guard
pattern、task 178 の membership/object-specific matched-output corruption、real-
sidecar test が route を保護する。truth/fact、acceptance、closure/order、theorem/proof/
Core/ControlFlow/VC、別 chain shape、general semantics は deferred のままである。
fixture、expectation、backlink 6 件、production routing、full guard、real sidecar が
active runner 127 を保護する。
task 179 は
`LongLocalObjectModeReservedVariableTypeAssertionPayloadBoundary: x is object;`
で終わる matching normalized-reflexive type-assertion sibling だけを規定する。
production route は task 74 の AST-bounded chain producer と task 153 の object-
normalizing type-assertion consumer を合成し、real expansion 7 本、raw
`ChainObjectMode6` subject result、独立した formula-side builtin-object asserted
input、ordinal 1 の `BindingId(0)` を保持し、両 role を terminal
`BaseObjectMode` RHS builtin-object identity 1 個へ normalize する。
general reachability と object/set coercion を用いず、1 `Inferred` term と 1
fact/deferred-free `Checked` type assertion を記録する。task 172 shared seven-
definition structural guard pattern と task 153 の real object consumer/source
near miss を再利用する。task 175 の matched-output guard は builtin-set asserted
head と raw `ChainObjectMode6` subject provenance corruption を reject するよう
適応し、real-sidecar test が route を保護する。widening/
`qua`、truth/fact、acceptance、closure/order、theorem/proof/Core/ControlFlow/VC、
別 chain shape、general semantics は deferred のままである。fixture、expectation、
trace reference 7 件、production routing、full guard、real sidecar が active
runner 128 を保護する。
task 180 は exact formula-leaf sibling
`theorem SourceDerivedContradictionConstantBoundary: contradiction;` を仕様化する。
source route は task 117 の contradiction-kind mapping と task 115 の standalone
theorem-shape validation pattern を再利用して standalone exact leaf extractor を
追加し、この leaf を synthetic missing-payload reason なしで既存
`FormulaKind::Contradiction` consumer へ渡す。結果は exact source
site/range と module-root context、term、asserted type、expected constraint、
candidate、fact、deferred reason、diagnostic を持たない 1 `Checked` formula を
保持しなければならない。この status は formula type/well-formedness checking
だけを意味する。falsehood/fact publication、theorem acceptance、proof-goal
closure、implicit closure/child graph extraction、`formula_statement` activation、
proof/CoreIr/ControlFlowIr/VC payload を主張しない。exact-source、near-miss/
corruption、real parser/resolver-sidecar guard が route を保護しなければならない。
task 181 は semantic promotion ではなく exactness repair である。reserve
source bridge が real imported `parser.type_fixtures` attribute を観測する場合、
既に credit された source 5 件、すなわち single-binding の positive
`TypeCaseAttr set`、positive または negative `empty set`、negative
`empty object` と、ordered mixed source `reserve x for set; reserve y for non
empty set;` だけを受理しなければならない。exact-source gate は各 attributed
binding の argument-free attribute 1 個と unrelated top-level item なしを要求
する。duplicate/mixed attribute、wrong polarity/head、exact mixed source 外の
multiple binding/item、extra definition は source extraction gap に残る。既存
fail-closed fixture 5 件の evidence-query expectation は保持する。この repair は
`.miz` case を追加せず、attributed type を accept せず、evidence を提供せず、
positive `empty object` を昇格しない。
task 182 は最初の formula-side local-mode asserted head を追加する。exact source
は `mode LocalModeAssertedHeadDef: LocalModeAssertedHead is set;` を含む
`definition` block、`reserve x for LocalModeAssertedHead;` 1 個、
`LocalModeAssertedHeadPayloadBoundary: x is
LocalModeAssertedHead;` だけを含む。producer は独立した raw type-expression
input 2 個を保持する。reserve-owned subject result と formula-owned
asserted type は distinct site/range を持ちながら同じ real local-mode symbol に
resolve する。real AST-derived mode expansion 1 個は known type entry 3 個すべてを
definition RHS に canonical anchor された builtin-set identity 1 個へ normalize
する。prepared type-assertion consumer は ordinal 1 を `BindingId(0)` に解決し、
1 `Inferred` variable と 1 fact/deferred-free normalized-reflexive `Checked`
formula を記録し、general reachability payload を使わない。exact-source、near-
miss、corruption、production-route、real parser/resolver-sidecar guard が builtin/
other-mode asserted head、attributed/argument-bearing head、object terminal、chain、
recovery、extra item、collapsed provenance を reject する。これは type/well-
formedness のみであり、mode declaration acceptance/inhabitation、widening/`qua`、
truth/fact、theorem acceptance、proof/CoreIr/ControlFlowIr/VC、他 asserted-head
family、general semantics は deferred のままである。
task 183 は direct object-terminal sibling を追加する。exact source は `mode
LocalObjectModeAssertedHeadDef: LocalObjectModeAssertedHead is object;` を含む
definition block 1 個、matching reserve 1 個、`LocalObjectModeAssertedHeadPayloadBoundary:
x is LocalObjectModeAssertedHead;` だけを含む。producer は distinct site/range と
同じ resolved mode symbol を持つ独立した raw reserve-subject/formula-side
asserted type-expression input を保持する。real AST-derived
expansion 1 個は known type entry 3 個すべてを definition RHS に canonical anchor
された builtin-object identity 1 個へ normalize する。prepared consumer は ordinal
1 を `BindingId(0)` に解決し、1 `Inferred` variable と 1 fact/deferred-free
normalized-reflexive `Checked` formula を記録し、general reachability と object/set
coercion を用いない。exact/near-miss/corruption/route-order/real frontend-resolver
sidecar guard は set terminal、builtin/other asserted head、chain、attribute、
argument、recovery、extra item、collapsed provenance を reject する。
declaration acceptance/inhabitation、truth/fact、theorem/proof/CoreIr/
ControlFlowIr/VC、他 asserted-head family、general semantics は deferred のままである。
task 184 は one-edge set-terminal
same-outer-mode asserted-head sibling だけを追加する。exact source は `mode
BaseModeAssertedHeadDef: BaseModeAssertedHead is set;` と `mode
ChainModeAssertedHeadDef: ChainModeAssertedHead is BaseModeAssertedHead;` を
含む ordered definition block 2 個、outer-mode reserve 1 個、
`ChainedLocalModeAssertedHeadPayloadBoundary: x is ChainModeAssertedHead;`
だけを含む。producer は distinct site/range と同じ resolved outer-mode symbol を
持つ独立した raw reserve-subject/formula-side asserted type-expression input を
保持する。real AST-derived expansion 2 個は known type entry 3 個
すべてを base definition RHS に canonical anchor された builtin-set identity 1 個へ
normalize する。prepared consumer は ordinal 1 を
`BindingId(0)` に解決し、1 `Inferred` variable と 1 fact/deferred-free
normalized-reflexive `Checked` formula を記録し、general reachability、widening、
`qua` を起動しない。exact/near-miss/corruption/route-order/real
frontend-resolver sidecar guard は wrong link/terminal/order/depth、builtin/base/
other asserted head、attribute、argument、recovery、extra item、collapsed
provenance を reject する。declaration acceptance/inhabitation、
truth/fact、closure/order、theorem/proof/CoreIr/ControlFlowIr/VC、object-terminal/
deeper asserted-head chain、general chain semantics は credit しない。
task 185 は one-edge object-terminal same-outer-mode asserted-head sibling だけを
追加する。exact source は `mode BaseObjectModeAssertedHeadDef:
BaseObjectModeAssertedHead is object;` と `mode ChainObjectModeAssertedHeadDef:
ChainObjectModeAssertedHead is BaseObjectModeAssertedHead;` を含む ordered
definition block 2 個、outer-mode reserve 1 個、
`ChainedLocalObjectModeAssertedHeadPayloadBoundary: x is
ChainObjectModeAssertedHead;` だけを含む。producer は distinct site/range と同じ
resolved outer-mode symbol を持つ独立した raw reserve-subject/formula-side
asserted type-expression input を保持する。real AST-derived
expansion 2 個は known type entry 3 個すべてを base definition RHS に canonical
anchor された builtin-object identity 1 個へ normalize する。
consumer は ordinal 1 を `BindingId(0)` に解決し、1 `Inferred`
variable と 1 fact/deferred-free normalized-reflexive `Checked` formula を記録し、
general reachability、widening、`qua`、object/set coercion を起動してはならない。
exact/near-miss/corruption/route-order/real frontend-resolver sidecar guard は wrong
link/terminal/order/depth、builtin/base/other asserted head、attribute、argument、
imported provenance、recovery、extra item、collapsed provenance、builtin-set output
corruption を reject する。shared trace backlink 5 個と dedicated row 1 個が
active runner 133 を保護する。declaration/attribute
acceptance、broader term/formula、child graph、truth/fact、closure/order、theorem/
proof/CoreIr/ControlFlowIr/VC、deeper/他 asserted-head chain、general chain
semantics は credit しない。module layout 更新は不要である。
task 186 は two-edge set-terminal same-outer-mode asserted-head slice だけを追加する。
exact source は `mode BaseTwoEdgeModeAssertedHeadDef:
BaseTwoEdgeModeAssertedHead is set;`、`mode MiddleTwoEdgeModeAssertedHeadDef:
MiddleTwoEdgeModeAssertedHead is BaseTwoEdgeModeAssertedHead;`、`mode
OuterTwoEdgeModeAssertedHeadDef: OuterTwoEdgeModeAssertedHead is
MiddleTwoEdgeModeAssertedHead;` を含む ordered definition block 3 個、outer-mode
reserve 1 個、`TwoEdgeLocalModeAssertedHeadPayloadBoundary: x is
OuterTwoEdgeModeAssertedHead;` だけを含む。producer は separate site/range と同じ
resolved outer-mode symbol を持つ distinct raw reserve-subject/formula-side
asserted type-expression input を保持する。real AST-derived expansion 3 個は known
type entry 3 個すべてを `BaseTwoEdgeModeAssertedHead` definition RHS に canonical
anchor された builtin-set identity 1 個へ normalize しなければならない。
consumer は ordinal 1 を `BindingId(0)` に解決し、1 `Inferred` variable と 1
fact/deferred-free normalized-reflexive `Checked` formula を記録し、reachability、
widening、`qua` を起動してはならない。exact/near-miss/corruption/route-order/
real frontend-resolver sidecar guard は各 missing expansion、wrong link/terminal/
order/depth、duplicate、forward/recovered/contextual/parameterized/argument-
bearing/attributed definition、direct/one-edge/deeper/object-terminal shape、non-
exact reserve/formula、wrong subject、builtin/base/middle/other/attributed/
argument-bearing asserted head、imported/ambiguous provenance、recovery、extra
item、collapsed provenance、builtin-object output corruption を reject する。
shared trace backlink 5 個と dedicated row 1 個が active runner 134 を保護する。
object-terminal/deeper/imported asserted head、declaration/attribute acceptance、
broader term/formula/child graph、truth/fact、closure/order、theorem/proof/CoreIr/
ControlFlowIr/VC、general chain semantics は credit しない。module layout 更新は
不要である。
task 187 は two-edge object-terminal same-outer-mode asserted-head slice だけを
追加する。exact source は ordered definition `mode
BaseTwoEdgeObjectModeAssertedHeadDef: BaseTwoEdgeObjectModeAssertedHead is
object;`、`mode MiddleTwoEdgeObjectModeAssertedHeadDef:
MiddleTwoEdgeObjectModeAssertedHead is BaseTwoEdgeObjectModeAssertedHead;`、
`mode OuterTwoEdgeObjectModeAssertedHeadDef: OuterTwoEdgeObjectModeAssertedHead
is MiddleTwoEdgeObjectModeAssertedHead;`、
outer-mode reserve 1 個、`TwoEdgeLocalObjectModeAssertedHeadPayloadBoundary: x is
OuterTwoEdgeObjectModeAssertedHead;` だけを含む。producer は同じ local outer
symbol 向けの distinct raw subject/asserted type-expression site/range を保持する。
real AST-derived expansion 3 個は known entry 3 個すべてを base definition RHS
に canonical anchor された builtin-object identity 1 個へ normalize する。
consumer は ordinal 1 を `BindingId(0)` に解決し、expected constraint、
reachability、widening、`qua`、object/set coercion を持たない 1 `Inferred`
variable と 1 fact/deferred-free normalized-reflexive `Checked` assertion を
記録する。exact/near-miss/corruption/route-order/real frontend-resolver sidecar
guard は missing/wrong/duplicate/reordered/forward/recovered/contextual/
parameterized/argument-bearing/attributed link または wrong label、set-terminal/
direct/one-edge/deeper shape、builtin/base/middle/other asserted head および
attributed/argument-bearing formula-side asserted head、imported Base/Middle/Outer、
imported/ambiguous asserted provenance、extra/collapsed provenance、`BuiltinSet`
output corruption を reject する。shared backlink 5 個と dedicated trace row 1 個
が active runner 135 を保護する。positive imported semantics、declaration/
attribute acceptance、broader term/formula/child graph、truth/fact、implicit
closure/order、theorem acceptance、proof/CoreIr/ControlFlowIr/VC、deeper asserted
head、general chain semantics は credit
しない。Step 5 は active、Steps 6/7 は deferred のまま。module layout 更新は
不要である。
task 160 は source
`reserve x, y for set; theorem DistinctReservedVariableInequalityPayloadBoundary: x <> y;`
だけを規定する。reserve item 1 個は shared written set range 1 個を持つ distinct
`BindingId(0/1)` と ordinal 2/3 の独立 lookup を生成しなければならない。両
operand result と operand-owned expected role 2 個はその range を保持して shared-
source builtin-set identity 1 個へ normalize し、2 `Inferred` variable と 2 ordered
constraint を持つ 1 fact/deferred-free pre-desugaring `Checked` inequality を生成する。
exact source、route order、matched output、corruption guard を必須とする。
inequality desugaring/truth/fact、closure/order、theorem acceptance、proof/Core/VC、
separate declaration、broader shape は deferred のままである。source/trace
contract、production route、独立した corruption coverage、real frontend/resolver
sidecar が active bridge を保護する。

task 188 は exact builtin-object reserved-variable
equality source `reserve x for object; theorem
ReservedObjectVariableEqualityPayloadBoundary: x = x;` だけを追加する。
production route は real source reserve handoff と既存 reserved-variable binary-
formula checker builder を再利用し、source-order use 2 個を ordinal 1/2 で
`BindingId(0)` に解決し、single written `object` range をすべて保持する distinct
result/expected role site 4 個を残さなければならない。checker output は written
reserve type に anchor された canonical known builtin-object identity 1 個、
`Inferred` variable term 2 個、operand-owned ordered expected constraint 2 個、
fact/deferred-free `Checked` equality 1 個を含まなければならない。exact/near-
miss test は別 label/operand/operator、status/recovery/extra item、multiple binding
または reserve item、`set`、mode、structure、attributed、argument-bearing reserve
head を reject する。task-local negative probe は binding/ordinal drift、
`BuiltinSet` head corruption、collapsed role site、wrong source projection、missing
expected input を reject し、positive count/status/constraint/canonical-source
assertion と shared formula-output validator が immutable checker output を固定する。
real frontend/resolver sidecar が active fixture を guard する。これは
type/well-formedness checking だけであり、object/set coercion、non-reflexive または
general object equality、truth/fact、implicit closure/order、theorem acceptance、
proof execution、CoreIr/ControlFlowIr/VC/proof payload は deferred のままである。

task 189 は exact builtin-object reserved-variable type assertion
`reserve x for object; theorem
ReservedObjectVariableTypeAssertionPayloadBoundary: x is object;` だけを追加する。
production route は real source reserve handoff と既存 reserved-variable type-
assertion builder を再利用し、subject を source-order ordinal 1 で
`BindingId(0)` に解決し、reserve-derived subject result と formula-side asserted
builtin `object` を distinct site/source range として保持しなければならない。
raw input 2 個はいずれも argument/attribute-free `BuiltinObject` であり、written
reserve type を canonical anchor とする known builtin-object identity 1 個へ
normalize しなければならない。immutable checker result は `Inferred` variable
1 個、known type entry 3 個、known normalized type 1 個、expected constraint/
candidate/fact/diagnostic/deferred reason 0 個、`Checked` `TypeAssertion` 1 個を
含まなければならない。exact/near-miss test は wrong label/subject、negation、
status/recovery/extra item、multiple binding/reserve、`set`、mode、structure、
attributed、argument-bearing、imported、ambiguous reserve/asserted head を reject
する。task-local corruption probe は binding/ordinal drift、raw input いずれかの
`BuiltinSet` 置換、collapsed/wrong source/site provenance を reject し、positive
count/status/constraint/canonical-source assertion と shared type-assertion output
validator が immutable checker table を固定する。real frontend/resolver sidecar
は active fixture を guard する。これは normalized-reflexive type/well-formedness checking
だけである。general reachability/widening/`qua`、object/set coercion、truth/fact、
implicit closure/order、theorem acceptance、proof execution、CoreIr/ControlFlowIr/
VC/proof payload は deferred のままとする。checker source と module layout の
変更は不要である。

task 190 は exact builtin-object reserved-variable inequality `reserve x for
object; theorem ReservedObjectVariableInequalityPayloadBoundary: x <> x;`
だけを追加する。production route は real source reserve handoff と既存の
reserved-variable binary-formula builder を再利用し、source-order ordinal 1/2
の両 use を `BindingId(0)` へ解決し、raw type input が written `object` range
1 個を保持する distinct result/expected role site 4 個を維持しなければ
ならない。すべての raw input は argument/attribute-free `BuiltinObject` で、
written reserve type を canonical anchor とする known builtin-object identity
1 個へ normalize しなければならない。immutable checker result は
`Inferred` variable 2 個、known type entry 6 個、known normalized type 1 個、
ordered operand-owned expected constraint 2 個、candidate/fact/diagnostic/
deferred reason 0 個、pre-desugaring `Checked` `Inequality` 1 個を含まなければ
ならない。exact/near-miss test は別 label/operand/operator、status/recovery/
extra item、multiple binding/reserve、`set`、mode、structure、attributed、
argument-bearing、imported、ambiguous reserve head を拒否する。task-local
corruption probe は binding/ordinal drift、`BuiltinSet` substitution、collapsed
role、wrong source projection、missing expected input を拒否し、positive count/
status/constraint/canonical-source assertion と shared binary-formula output
validator が immutable checker table を固定する。real frontend/resolver
sidecar は active fixture を保護する。これは type/well-formedness checking
のみであり、inequality desugaring/equality truth、object/set coercion、fact、
implicit closure/order、theorem acceptance、proof execution、CoreIr/
ControlFlowIr/VC/proof payload は deferred のままである。checker source または
module-layout change は不要であった。

task 191 は exact distinct-binding shared-builtin-object equality `reserve x,
y for object; theorem DistinctReservedObjectVariableEqualityPayloadBoundary:
x = y;` だけを追加する。production route は real one-item/two-binding
shared-range reserve handoff と既存 builtin-object binary-formula consumer を
合成し、source-order ordinal 2/3 の use を `BindingId(0)` と
`BindingId(1)` に解決し、raw type input が shared written `object` range 1 個
を保持する distinct result/expected role site 4 個を維持しなければならない。
raw input 4 個はいずれも argument/attribute-free `BuiltinObject` であり、
reserve range を canonical anchor とする known builtin-object identity 1 個へ
normalize する。immutable checker result は `Inferred` variable 2 個、known
type entry 6 個、known normalized type 1 個、operand-owned ordered expected
constraint 2 個、candidate/fact/diagnostic/deferred reason 0 個、`Checked`
`Equality` 1 個を含まなければならない。exact/near-miss test は別 label、
reversed/same/unknown operand、別 operator、status/recovery/extra item、single/
extra/separate reserve binding、`set`、mode、structure、attributed、argument-
bearing、imported、ambiguous reserve head を拒否する。task-local corruption
probe は collapsed binding identity、ordinal drift、`BuiltinSet` substitution、
collapsed role、誤った shared source projection、missing expected input を
拒否し、positive count/status/constraint/canonical-source assertion と shared
binary-formula output validator は immutable checker table を固定する。real
frontend/resolver sidecar は active fixture を guard しなければならない。
これは type/well-formedness checking のみである。equality truth、object/set
coercion、fact、implicit closure/order、theorem acceptance、proof execution、
CoreIr/ControlFlowIr/VC/proof payload は deferred のままとする。checker
source または module-layout change は不要であった。

task 192 は exact distinct-binding shared-builtin-object inequality `reserve x,
y for object; theorem DistinctReservedObjectVariableInequalityPayloadBoundary:
x <> y;` だけを対象とする。production route は real one-item/two-binding
shared-range builtin-object reserve handoff と既存 pre-desugaring inequality
consumer を合成し、source-order use を ordinal 2 と 3 で `BindingId(0)` と
`BindingId(1)` に解決し、raw type input が shared written `object` range 1 個を
保持する distinct result/expected role site 4 個を残さなければならない。raw
input 4 個はいずれも argument/attribute-free `BuiltinObject` であり、reserve
range を canonical anchor とする known builtin-object identity 1 個へ normalize
しなければならない。immutable checker result は `Inferred` variable 2 個、
known type entry 6 個、known normalized type 1 個、operand-owned ordered expected
constraint 2 個、candidate/fact/diagnostic/deferred reason 0 個、`Checked`
`Inequality` 1 個を含まなければならない。exact/near-miss test は wrong label、
reversed/same/isolated-left/unknown operand、wrong operator、status/recovery/
extra item、single/extra/separate reserve binding、`set`、mode、structure、
attributed、argument-bearing、imported、ambiguous reserve head を reject する。
task-local corruption probe は collapsed binding identity、ordinal drift、
`BuiltinSet` substitution、collapsed role、wrong shared source projection、
missing expected input を reject し、positive count/status/constraint/canonical-
source assertion と shared binary-formula output validator が immutable checker
table を固定する。real frontend/resolver sidecar が active fixture を guard
する。これは type/well-formedness checking のみであり、inequality desugaring/
equality truth、object/set coercion、fact、implicit closure/order、theorem
acceptance、proof execution、CoreIr/ControlFlowIr/VC/proof payload は deferred
のままである。checker source または module-layout change は不要であった。

task 193 は exact multiple-reserve-declaration builtin-object equality
`reserve x for object; reserve y for object; theorem
MultipleObjectReserveDeclarationEqualityPayloadBoundary: x = y;` だけを対象と
する。production route は Task 124 の real two-item/two-binding/distinct-
written-range reserve handoff と builtin-object equality consumer を合成し、
source-order use を ordinal 2 と 3 で `BindingId(0)` と `BindingId(1)` に解決し、
raw type input が binding ごとの distinct written `object` range 2 個を保持する
distinct result/expected role site 4 個を残す。raw input 4 個はいずれも
argument/attribute-free `BuiltinObject` であり、先行する `x` reserve range を
canonical anchor とする known builtin-object identity 1 個へ normalize する。
immutable checker result は `Inferred` variable 2 個、known type entry 6 個、
known normalized type 1 個、operand-owned ordered expected constraint 2 個、
candidate/fact/diagnostic/deferred reason 0 個、`Checked` `Equality` 1 個を含む。
exact/near-miss test は wrong label、reversed/same/isolated-unknown operand、
other operator、status/recovery/extra theorem、shared/reordered/mixed/extra
reserve item、numeral operand、attributed/argument-bearing/local/imported/
ambiguous mode/structure head を reject する。corruption probe は binding/
ordinal/range/role/head/raw-source/canonical-source/expected-input/module drift
を reject し、positive checker-table/canonical-source assertion と real
frontend/resolver sidecar が active fixture を guard する。これは type/well-
formedness checking のみであり、equality truth、object/set coercion、fact、
implicit closure/order、theorem acceptance、proof execution、CoreIr/
ControlFlowIr/VC/proof payload、shared-range shape、broader multiple-reserve
object shape は deferred のままである。checker source または module-layout
change は不要であった。

task 194 は exact multiple-reserve-declaration builtin-object inequality
`reserve x for object; reserve y for object; theorem
MultipleObjectReserveDeclarationInequalityPayloadBoundary: x <> y;` だけを対象と
する。production route は Task 193 の real ordered two-item/two-binding/
distinct-written-object-range handoff と pre-desugaring builtin-object
inequality consumer を合成する。source-order use を ordinal 2 と 3 で
`BindingId(0)` と `BindingId(1)` に解決し、raw type input が binding ごとの
written `object` range 2 個を保持する distinct result/expected role site 4 個を
残す。raw input 4 個はいずれも argument/attribute-free `BuiltinObject` であり、
先行する `x` reserve range を canonical anchor とする known builtin-object
identity 1 個へ normalize する。immutable checker result は `Inferred`
variable 2 個、known type entry 6 個、known normalized type 1 個、operand-owned
ordered expected constraint 2 個、candidate/fact/diagnostic/deferred reason 0
個、pre-desugaring `Checked` `Inequality` 1 個を含む。exact/near-miss test は
wrong label、reversed/same/isolated-unknown operand、other operator、status/
recovery/extra theorem、shared/reordered/mixed/extra reserve item、numeral
operand、attributed/argument-bearing/local/imported/ambiguous mode/structure
head を reject する。corruption probe は binding/ordinal/range/role/head/raw-
source/canonical-source/expected-input/module drift を reject し、positive
checker-table assertion、route isolation、real frontend/resolver sidecar が
active fixture を guard する。これは type/well-formedness checking のみで
あり、inequality desugaring/equality truth、object/set coercion、fact、implicit
closure/order、theorem acceptance、proof execution、CoreIr/ControlFlowIr/VC/
proof payload、shared-range shape、broader multiple-reserve object shape は
deferred のままである。checker source または module-layout change は不要で
あった。

task 195 は ordered definition 4 個 `Outer -> Middle -> Inner -> Base -> set`、
`OuterThreeEdgeModeAssertedHead` reserve 1 個、
`ThreeEdgeLocalModeAssertedHeadPayloadBoundary: x is
OuterThreeEdgeModeAssertedHead;` を持つ exact three-edge set-terminal same-
outer-mode asserted-head source だけを対象とする。production route は Task 73
の real four-expansion AST handoff と Task 186 の same-symbol formula-side
asserted-head consumer を合成する。ordinal 1 を `BindingId(0)` に解決し、outer
symbol の raw reserve-subject と formula asserted-type の独立した site/range を
保持し、local bare argument/attribute/context/recovery-free expansion 4 個だけを
消費する。known type entry 3 個は base definition RHS を canonical anchor と
する `BuiltinSet` identity 1 個へ normalize する。immutable result は
`Inferred` variable 1 個、expected constraint 0 個、candidate/fact/diagnostic/
deferred reason 0 個、normalized-reflexive `Checked` `TypeAssertion` 1 個を含む。
exact structural/provenance guard は shorter、deeper、object-terminal、malformed、
unrelated-local、imported、ambiguous asserted-head shape を reject し、独立した
corruption probe、positive output check、route isolation、real frontend/resolver
sidecar が active runner 143 を保護する。これは mode declaration acceptance、
reachability/widening/`qua`、assertion truth/fact、implicit closure/order、theorem
acceptance、broader term/formula/child-graph semantics、proof/CoreIr/
ControlFlowIr/VC、general chain semantics を確立しない。checker source または
module-layout change は不要であった。

task 196 は ordered definition 4 個 `Outer -> Middle -> Inner -> Base ->
object`、`OuterThreeEdgeObjectModeAssertedHead` reserve 1 個、
`ThreeEdgeLocalObjectModeAssertedHeadPayloadBoundary: x is
OuterThreeEdgeObjectModeAssertedHead;` を持つ exact three-edge object-terminal
same-outer-mode asserted-head source だけを対象とする。production route は
Tasks 73/151 の real four-expansion object-terminal AST handoff と Task 187 の
same-symbol formula-side asserted-head consumer を合成する。ordinal 1 を
`BindingId(0)` に解決し、outer symbol の raw reserve-subject と formula
asserted-type の独立した site/range を保持し、local bare argument/attribute/
context/recovery-free expansion 4 個だけを消費する。known type entry 3 個は
base definition RHS を canonical anchor とする `BuiltinObject` identity 1 個へ
normalize する。immutable result は `Inferred` variable 1 個、expected
constraint 0 個、candidate/fact/diagnostic/deferred reason 0 個、normalized-
reflexive `Checked` `TypeAssertion` 1 個を含み、object/set coercion はない。
exact structural/provenance guard は shorter、deeper、set-terminal、malformed、
unrelated-local、imported、ambiguous asserted-head shape を reject し、独立した
`BuiltinSet`/canonical-source corruption probe、positive output check、route
isolation、real frontend/resolver sidecar が active runner 144 を保護する。これは
mode declaration acceptance、reachability/widening/`qua`、assertion truth/fact、
implicit closure/order、theorem acceptance、broader term/formula/child-graph
semantics、proof/CoreIr/ControlFlowIr/VC、general chain semantics を確立しない。
checker source または module-layout change は不要であった。

task 197 は ordered definition 5 個 `TooDeep -> Outer -> Middle -> Inner ->
Base -> set`、`TooDeepFourEdgeModeAssertedHead` reserve 1 個、
`FourEdgeLocalModeAssertedHeadPayloadBoundary: x is
TooDeepFourEdgeModeAssertedHead;` を持つ exact four-edge set-terminal same-
outermost-mode asserted-head source だけを対象とする。production route は
Tasks 74/152 の real five-expansion AST handoff と Tasks 186/195 の same-symbol
formula-side asserted-head consumer を合成する。ordinal 1 を `BindingId(0)`
に解決し、outermost symbol の raw reserve-subject と formula asserted-type の
独立した site/range を保持し、local bare argument/attribute/context/recovery-
free expansion 5 個だけを消費する。known type entry 3 個は base definition
RHS を canonical anchor とする `BuiltinSet` identity 1 個へ normalize する。
immutable result は `Inferred` variable 1 個、expected constraint 0 個、
candidate/fact/diagnostic/deferred reason 0 個、normalized-reflexive `Checked`
`TypeAssertion` 1 個を含む。exact structural/provenance guard は shorter、
deeper、reordered、object-terminal、malformed、unrelated-local、imported、
ambiguous asserted-head shape を reject し、独立した `BuiltinObject`/
canonical-source corruption probe、positive output check、route isolation、
real frontend/resolver sidecar が active runner 145 を保護する。これは mode
declaration acceptance、reachability/widening/`qua`、assertion truth/fact、
implicit closure/order、theorem acceptance、broader term/formula/child-graph
semantics、proof/CoreIr/ControlFlowIr/VC、general chain semantics を確立しない。
checker source または module-layout change は不要であった。

task 198 は ordered definition 5 個 `TooDeep -> Outer -> Middle -> Inner ->
Base -> object`、`TooDeepFourEdgeObjectModeAssertedHead` reserve 1 個、
`FourEdgeLocalObjectModeAssertedHeadPayloadBoundary: x is
TooDeepFourEdgeObjectModeAssertedHead;` を持つ exact four-edge object-terminal
same-outermost-mode asserted-head source だけを対象とする。production route は
Tasks 74/153 の real five-expansion AST handoff と Tasks 187/196 の same-symbol
formula-side asserted-head consumer を合成する。ordinal 1 を `BindingId(0)` に
解決し、outermost symbol の raw reserve-subject と formula asserted-type の
独立した site/range を保持し、local bare argument/attribute/context/recovery-
free expansion 5 個だけを消費する。known type entry 3 個は base definition
RHS を canonical anchor とする `BuiltinObject` identity 1 個へ normalize する。
immutable result は `Inferred` variable 1 個、expected constraint 0 個、
candidate/fact/diagnostic/deferred reason 0 個、normalized-reflexive `Checked`
`TypeAssertion` 1 個を含み、object/set coercion はない。exact structural/
provenance guard は shorter、deeper、full-reordered、set-terminal、malformed、
unrelated-local、imported、ambiguous asserted-head shape を reject し、独立した
`BuiltinSet`/canonical-source corruption probe、positive output check、route
isolation、real frontend/resolver sidecar が active runner 146 を保護する。
これは mode declaration acceptance、reachability/widening/`qua`、assertion
truth/fact、implicit closure/order、theorem acceptance、broader term/formula/
child-graph semantics、proof/CoreIr/ControlFlowIr/VC、general chain semantics
を確立しない。checker source または module-layout change は不要であった。

task 199 は既存の ordered definition `ChainMode6 -> ChainMode5 -> ChainMode4
-> ChainMode3 -> ChainMode2 -> ChainMode1 -> BaseMode -> set`、`ChainMode6`
reserve 1 個、`LongLocalModeAssertedHeadPayloadBoundary: x is ChainMode6;` を
持つ exact seven-expansion set-terminal same-`ChainMode6` asserted-head source
だけを対象とする。production route は Tasks 74/175 の real seven-expansion
AST handoff と Tasks 186/195/197 の same-symbol formula-side asserted-head
consumer を合成する。ordinal 1 を `BindingId(0)` に解決し、同じ symbol の
raw reserve-subject と formula asserted-type の独立した site/range を保持し、
local bare argument/attribute/context/recovery-free expansion 7 個だけを消費
する。known type entry 3 個は `BaseModeDef` RHS を canonical anchor とする
`BuiltinSet` identity 1 個へ normalize する。immutable result は `Inferred`
variable 1 個、expected constraint/candidate/fact/diagnostic/deferred reason 0
個、normalized-reflexive `Checked` `TypeAssertion` 1 個を含む。full-reverse、
connected-eighth-edge、structural/provenance、mutable-corruption、positive-
output、route-isolation、real frontend/resolver-sidecar guard が active runner
147 を保護する。これは mode declaration acceptance、reachability/widening/
`qua`、assertion truth/fact、implicit closure/order、theorem acceptance、
broader term/formula/child-graph semantics、proof/CoreIr/ControlFlowIr/VC、
general unbounded chain semantics を確立しない。checker source または
module-layout change は不要であった。

task 200 は既存の ordered definition `ChainObjectMode6 -> ChainObjectMode5 ->
ChainObjectMode4 -> ChainObjectMode3 -> ChainObjectMode2 -> ChainObjectMode1 ->
BaseObjectMode -> object`、`ChainObjectMode6` reserve 1 個、
`LongLocalObjectModeAssertedHeadPayloadBoundary: x is ChainObjectMode6;` を
持つ exact seven-expansion object-terminal same-`ChainObjectMode6` asserted-
head source だけを対象とする。production route は Tasks 74/179 の real seven-
expansion AST handoff と Tasks 187/196/198 の same-symbol formula-side
asserted-head consumer、Task 199 の depth-matched set-terminal sibling を合成
する。ordinal 1 を `BindingId(0)` に解決し、同じ symbol の raw reserve-
subject と formula asserted-type の独立した site/range を保持し、local bare
argument/attribute/context/recovery-free expansion 7 個だけを消費する。known
type entry 3 個は `BaseObjectModeDef` RHS を canonical anchor とする
`BuiltinObject` identity 1 個へ normalize する。immutable result は
`Inferred` variable 1 個、expected constraint/candidate/fact/diagnostic/
deferred reason 0 個、object/set coercion のない normalized-reflexive
`Checked` `TypeAssertion` 1 個を含む。full-reverse、connected-eighth-edge、
structural/provenance、mutable-corruption、positive-output、route-isolation、
real frontend/resolver-sidecar guard が active runner 148 を保護する。これは
mode declaration acceptance、reachability/widening/`qua`、assertion truth/
fact、implicit closure/order、theorem acceptance、broader term/formula/child-
graph semantics、proof/CoreIr/ControlFlowIr/VC、general unbounded chain
semantics を確立しない。checker source または module-layout change は不要
であった。

task 120 は exact source
`reserve x for set; theorem ReservedVariableMembershipPayloadBoundary: x in x;`
だけについて、その real identifier-term seam を拡張する。同じ source-range event
ordering から distinct lookup ordinal 1 と 2 を導出し、2 回の独立した
`BindingEnv::lookup` call が両 identifier term を記述された reserve binding に
解決する。shared producer は builtin `set` を 2 つの result role と、既存 checker
membership contract に従う右 operand 所有の 1 つの expected-type role に投影する。
production validation は 2 つの `Inferred` variable term、1 つの `Checked`
`FormulaKind::Membership`、右 operand の exact single expected-type constraint、
source range/spelling/head を保つ 3 つの exact role owner、empty
candidate/fact/deferred/diagnostic output を要求する。matched source の
construction または invariant failure は
`type_elaboration.checker.reserved_variable_membership.invalid_payload` を報告する。
`Checked` は type/well-formedness だけを表し、task 120 は membership truth/fact、
implicit closure、theorem acceptance、`formula_statement`、proof、CoreIr、
ControlFlowIr、VC payload を生成しない。non-exact label/operator/operand/reserve
shape/item count/order/recovery/status token/numeral operand は extraction gap に残る。
task 121 は exact sibling
`reserve x for set; theorem ReservedVariableInequalityPayloadBoundary: x <> x;`
を追加する。shared producer は independently resolved な 2 つの known-`set`
variable、2 result role、checker output に link された 2 expected role を供給する。
1 つの fact-free `Checked` `FormulaKind::Inequality` は pre-desugaring
type/well-formedness だけを記録する。construction/invariant drift は
`type_elaboration.checker.reserved_variable_inequality.invalid_payload` を報告し、
real frontend/resolver active-sidecar test が payload を guard する。two-expected-type
contract の根拠は checker-owned API coverage と task 119 の real role producer で、
task 107 は expected type を持たない partial numeral bridge のままである。task 121
は `not equality` desugaring、truth/fact、implicit closure、theorem acceptance、
proof、CoreIr、ControlFlowIr、VC を実装しない。
task 109 は task 102 の exact builtin `set` portion を supersede する:
`theorem BuiltinTypeAssertionPayloadBoundary: 1 is set;` は Chapter 13 の numeral
term と Chapter 14 の builtin type-assertion form を通じて parser / resolver 実行まで
到達し、source-derived checker `TermInput`、`FormulaInput`、asserted builtin
`set` `TypeExpressionInput` payload を渡してから missing numeric type payload と
partial formula checking で fail する。より広い asserted type payload extraction、
term inference、type-assertion semantic checking、partial-term diagnostic を超える
formula checking、recorded fact、theorem acceptance、dedicated `formula_statement`
runner、CoreIr、ControlFlowIr、VC、proof payload はまだ存在しない。
task 122 はその formula-side asserted-type producer と task 119 の real
reserved-variable producer を exact source
`reserve x for set; theorem ReservedVariableTypeAssertionPayloadBoundary: x is set;`
について結合する。また checker-owned type-assertion admissibility gate を修正し、
exactly one ready subject と one normalized asserted type を要求する。semantic
normalized identity だけが `Checked` を維持し、known non-identical pair は widening
evidence を捏造せず、
`FormulaDeferredReason::MissingTypeAssertionReachabilityPayload` と
`checker.formula.external.type_assertion_reachability_payload` を持つ `Partial`
になる。missing asserted payload は `checker.formula.missing_asserted_type`
(`Partial`)、invalid subject arity は
`checker.formula.type_assertion.subject_arity` (`Error`) を使う。source runner は
normalization 前の reserve-derived subject result input と formula node-derived
asserted input を独立に保持し、distinct source anchor と empty argument/attribute
を検証してから、checker が両者を同じ reflexive builtin-`set` semantic id に
link することを要求する。1 `Inferred` variable と 1 fact-free `Checked` type
assertion を記録する。task 109 の numeric subject は exact existing two
diagnostics のまま partial である。general reachability/widening/`qua`、broader
asserted type/attribute、truth/fact、implicit closure、theorem acceptance、
`formula_statement`、proof、CoreIr、ControlFlowIr、VC は deferred のままである。
task 113 は task 103 の exact positive imported attribute assertion variant を
supersede する:
`import parser.type_fixtures; theorem ImportedAttributeAssertionPayloadBoundary: 1 is empty;`
は Chapter 13 の numeral term、documented imported `parser.type_fixtures` の
`empty` attribute、Chapter 14 の attribute-assertion form を通じて parser /
resolver 実行まで到達し、imported `empty` provenance を検証してから
source-derived checker `TermInput` / `FormulaInput` payload を渡し、missing
numeric type payload、missing formula / attribute semantic payload、partial
formula checking で fail closed する。これは imported module AST extraction、
imported attribute assertion attribute-chain semantic payload extraction、theorem
formula 向け checker `AttributeInput` payload extraction、term inference、
attribute admissibility/semantic checking、formula checking、recorded fact、
theorem acceptance、dedicated `formula_statement` runner、CoreIr、ControlFlowIr、
VC、proof payload を credit しない。
task 114 は同じ term/formula boundary の exact attribute-level `non empty`
imported attribute assertion variant について task 104 を supersede する:
`import parser.type_fixtures; theorem ImportedNonEmptyAttributeAssertionPayloadBoundary: 1 is non empty;`
は Chapter 13 の numeral term、documented imported `parser.type_fixtures` の
`empty` attribute、Chapter 6 の attribute negation/composition、Chapter 14 の
attribute-assertion form を通じて parser / resolver 実行まで到達し、direct `non`
surface と imported `empty` provenance を検証して source-derived checker
`TermInput` / `FormulaInput` payload を渡してから、missing numeric type payload、
missing formula / attribute semantic payload、partial formula checking で fail
closed する。これは imported module AST extraction、negated attribute-chain
semantic payload extraction、theorem formula 向け checker `AttributeInput`
payload extraction、term inference、negated attribute admissibility/semantic
checking、formula checking、recorded fact、theorem acceptance、dedicated
`formula_statement` runner、CoreIr、ControlFlowIr、VC、proof payload を credit
しない。
task 111 は task 105 のうち exact set-enumeration theorem bridge だけを
supersede する:
`theorem SetEnumerationPayloadBoundary: {1, 2} = {1, 2};` は Chapter 13 の
set-enumeration term operand と Chapter 14 の builtin equality を伴って parser /
resolver 実行まで到達し、4 つの numeral item term、2 つの set-enumeration
term、equality formula の real source-derived checker payload を渡してから、
missing numeric type payload、missing set-enumeration result-type
payload、partial formula checking で fail closed する。broader
set-enumeration term extraction、term inference、equality/formula checking、
recorded fact、theorem acceptance、dedicated `formula_statement` runner、CoreIr、
ControlFlowIr、VC、proof payload は主張しない。sethood は missing
set-enumeration payload ではなく、Chapter 13 では set comprehension の
generator domain だけに課される requirement である。
task 112 は task 99 のうち exact connective / quantifier theorem formula bridge
だけを supersede する:
`theorem FormulaConnectiveQuantifierPayloadBoundary: contradiction implies for x being set holds not contradiction;`
は Chapter 14 の implication、universal-quantifier、negation surface を通じて
parser / resolver 実行まで到達し、implication、quantified formula、negation の
real source-derived checker `FormulaInput` shell を渡す。checker は implication と
negation shell では `FormulaDeferredReason::MissingFormulaPayload`、quantified
shell では `FormulaDeferredReason::MissingQuantifierPayload` で fail closed
しなければならない。task 117 はこの exact source だけをさらに進め、2 つの
source-derived `contradiction` constant site/range を
`FormulaKind::Contradiction` checker payload として渡し、同じ missing formula
payload diagnostic に留める。formula constant semantic truth value、
child-formula graph payload、quantifier binder/context payload、formula checking、
recorded fact、theorem acceptance、dedicated `formula_statement` runner、CoreIr、
ControlFlowIr、VC、proof payload はまだ存在しない。この bridge は broader formula
extraction や accepted formula fact を credit しない。
task 88 は対応する proof-block boundary を記録する:
`theorem ProofSkeletonPayloadBoundary: thesis proof thus thesis; end;` のような
theorem は Chapter 16 の proof block と Chapter 15 の conclusion statement を伴って
parser / resolver 実行まで到達するが、real proof skeleton payload extraction、
local proof context、formula payload extraction、recorded fact、theorem acceptance、
dedicated `formula_statement` runner、CoreIr、ControlFlowIr、VC、proof payload が
まだ存在しないため `type_elaboration.external_dependency.ast_payload_extraction` に残す。
task 89 は statement-level proof-justification boundary を記録する:
labeled `A: thesis proof ... end;` statement と final `thus thesis proof ... end;`
conclusion を含む theorem proof は parser / resolver 実行まで到達するが、real
statement proof payload extraction、nested proof skeleton payload、local proof
context、formula payload extraction、label-reference semantic checking、recorded
fact、theorem acceptance、dedicated `formula_statement` runner、CoreIr、
ControlFlowIr、VC、proof payload がまだ存在しないため
`type_elaboration.external_dependency.ast_payload_extraction` に残す。
task 90 は predicate/functor definition boundary を記録する:
`pred DefinitionPredicatePayloadBoundary: x boundary_rel y means thesis;` と
`func DefinitionFunctorPayloadBoundary: boundary_func x -> set equals x;` を含む
definition block は parser / resolver 実行まで到達するが、real predicate/functor
definition declaration payload extraction、definition-local context、definiens
formula/term payload、overload payload、recorded fact、dedicated
`formula_statement` runner、CoreIr、ControlFlowIr、VC、proof payload がまだ存在しないため
`type_elaboration.external_dependency.ast_payload_extraction` に残す。
task 91 は attribute definition boundary を記録する:
`attr AttributePayloadBoundary: x is marked means thesis;` を含む definition
block は parser / resolver 実行まで到達するが、real attribute definition
declaration payload extraction、definition-local context、formula-definiens
payload、attributed-type evidence、recorded fact、dedicated
`formula_statement` runner、CoreIr、ControlFlowIr、VC、proof payload がまだ存在しないため
`type_elaboration.external_dependency.ast_payload_extraction` に残す。
task 92 は mode/structure definition boundary を記録する:
`struct DefinitionStructPayloadBoundary where ... end;` と
`mode DefinitionModePayloadBoundaryDef: DefinitionModePayloadBoundary is set;`
を含む definition block は parser / resolver 実行まで到達するが、real
mode/structure definition declaration payload extraction、mode expansion、
structure base-shape / constructor / selector evidence、definition-local context、
recorded fact、dedicated `formula_statement` runner、CoreIr、ControlFlowIr、VC、
proof payload がまだ存在しないため
`type_elaboration.external_dependency.ast_payload_extraction` に残す。
task 93 は proof-local declaration statement boundary を記録する:
`let`、`given`、`consider`、`set`、`reconsider` statement を含む theorem proof は
parser / resolver 実行まで到達するが、real proof-local declaration payload
extraction、local proof context、formula / term payload、RHS term inference、
reconsider coercion / obligation evidence、recorded fact、dedicated
`formula_statement` runner、CoreIr、ControlFlowIr、VC、proof payload がまだ
存在しないため `type_elaboration.external_dependency.ast_payload_extraction`
に残す。
task 94 は proof-local inline definition boundary を記録する: `deffunc` と
`defpred` statement を含む theorem proof は parser / resolver 実行まで到達するが、
real inline definition formal/body payload extraction、local abbreviation
expansion、term / formula body payload、guard evidence、recorded fact、theorem acceptance、dedicated
`formula_statement` runner、CoreIr、ControlFlowIr、VC、proof payload がまだ
存在しないため `type_elaboration.external_dependency.ast_payload_extraction`
に残す。
task 95 は registration block boundary を記録する: existential cluster と
conditional cluster を含む top-level registration block は parser / resolver
実行まで到達するが、real registration-item payload extraction、correctness-condition /
proof-obligation payload、accepted activation / evidence status、cluster /
reduction semantics、recorded fact、dedicated `formula_statement` または
`advanced_semantics` runner、CoreIr、ControlFlowIr、VC、proof payload がまだ
存在しないため `type_elaboration.external_dependency.ast_payload_extraction`
に残す。これは Chapter 17 semantic cluster / reduction row を credit しない。
task 96 は redefinition / notation boundary を記録する: top-level と
definition-local の synonym / antonym alias、および attribute、predicate、
functor redefinition declaration は documented `parser.type_fixtures` symbol を
使って parser / resolver 実行まで到達するが、real redefinition payload
extraction、notation alias relation payload、redefinition target inference、
coherence proof-obligation payload、overload candidate payload、recorded fact、
dedicated `formula_statement` または `advanced_semantics` runner、CoreIr、
ControlFlowIr、VC、proof payload がまだ存在しないため
`type_elaboration.external_dependency.ast_payload_extraction` に残す。これは
Chapter 11 alias semantic resolution や Chapter 19 overload / redefinition
semantics を credit しない。
task 82 は task 79 の imported-mode provenance 部分だけを昇格する:
documented `parser.type_fixtures` import summary 由来の `TypeCaseMode` のような
reserve head は、可視 resolver symbol が `SymbolKind::Mode` で
`ImportedSource` contribution を持つ場合に checker-owned
`TypeHeadInput::Symbol` として渡してよい。ただし real imported mode definition
または module-summary expansion payload は存在しないため、runner は
`ModeExpansion` payload を引き続き渡さず、active case は generic AST payload
extraction gap ではなく `checker.type.external.mode_expansion_payload` に到達する。
これは imported module AST extraction、arity checking、positive imported mode
elaboration、imported structure、imported attribute、CoreIr、ControlFlowIr、VC、
proof payload を主張しない。
task 57 はさらに、RHS が type argument を持たない same-module
local structure head である bare same-module no-argument local mode expansion を許可する。
この case は real `ModeExpansion` を消費するため missing mode-expansion payload diagnostic を
出してはならない。ただし expanded structure radix は real base-shape /
constructor-witness evidence extraction が存在するまで
`checker.declaration.deferred.evidence_query` で fail closed する。task 62 はさらに、
同じ structure-RHS diagnostic path を bare local-mode dependency edge 1 つだけに許可する。
unique / unrecovered / same-module / no-argument な terminal mode definition
`B is LocalStruct` と unique / unrecovered / same-module / no-argument な chain
definition `A is B` はどちらも reserve use より前に現れ、unique / unrecovered /
same-module structure definition は `B` より前に現れ、両方の mode definition は
definition-local context を持たず、runner は同じ `SurfaceAst` から real `B -> LocalStruct`
と `A -> B` expansion の両方を抽出しなければならない。expanded chain は real
base-shape / constructor-witness evidence extraction が存在するまで
`checker.declaration.deferred.evidence_query` で fail closed する。attributed root、
attributed/deeper chain、imported / argument-bearing symbol、contextual /
parameterized definition はこの slice の外に残る。task 63 はさらに、同じ
attributed-builtin RHS diagnostic path を bare local-mode dependency edge 1 つだけに
許可する。unique / unrecovered / same-module / no-argument な terminal attributed-builtin mode definition
（`B is marked set` または `B is marked object`）と unique / unrecovered / same-module / no-argument な
chain definition `A is B` はどちらも reserve use より前に現れ、`B` は `A` より前に
現れ、両方の mode definition は definition-local context を持たず、runner は同じ
`SurfaceAst` から real `B -> marked set` と `A -> B` expansion の両方を抽出しなければ
ならない。expanded chain は real attributed-type existential evidence extraction が
存在するまで `checker.declaration.deferred.evidence_query` で fail closed する。
attributed root、attributed/deeper chain、imported / argument-bearing attribute や
mode、contextual / parameterized definition はこの slice の外に残る。

task 64 は task-56 chain の attributed-root variant を 1 つだけ許可する:
attributed local-mode reserve head `marked A` は、`A` が bare reserve head としても
使われておらず、`B` が attributed reserve head として使われておらず、両方の mode
definition が unique / unrecovered / same-module / no-argument で definition-local
context を持たず、source order が `B -> A -> reserve` である場合に限り、real one-edge
chain `A -> B -> set` / `object` を消費してよい。checker は両方の real expansion と
reserve-head attribute を消費し、real attributed-type existential evidence extraction が
存在するまで `checker.declaration.deferred.evidence_query` で fail closed する。
one-edge dependency が local structure RHS に終端する attributed root は task 64 の外に
残るが task 65 で許可される。attributed builtin RHS terminal は task 64 の外に残るが
task 66 で許可される。deeper chain、mixed bare/attributed use、import、argument、
contextual / parameterized definition、positive attributed-type acceptance はこの slice の外に残る。

task 65 は task-64 attributed-root chain の structure-RHS counterpart を許可する:
attributed local-mode reserve head `marked A` は、`A` が bare reserve head としても
使われておらず、`B` が attributed reserve head として使われておらず、`B is LocalStruct`
と `A is B` が unique / unrecovered / same-module / no-argument で definition-local
context を持たず、same-module structure definition が unique / unrecovered で `B` より
前に現れ、source order が `LocalStruct -> B -> A -> reserve` である場合だけ、real
one-edge chain `A -> B -> LocalStruct` を消費してよい。checker は両方の real expansion と
reserve-head attribute を消費し、real structure base-shape / constructor-witness evidence と
full attributed-type existential evidence extraction が存在するまで
`checker.declaration.deferred.evidence_query` で fail closed する。attributed-builtin RHS
terminal、deeper chain、mixed bare/attributed use、attributed dependency、import、argument、
ambiguous symbol、contextual / parameterized definition、positive structure /
attributed-type acceptance、CoreIr、ControlFlowIr、VC、proof payload はこの slice の外に残るが、
task 66 は one-edge attributed-builtin RHS terminal を別途許可する。

task 66 は task-64/task-65 attributed-root chain の attributed-builtin-RHS counterpart を
許可する: attributed local-mode reserve head `marked A` は、`A` が bare reserve head としても
使われておらず、`B` が attributed reserve head として使われておらず、両方の mode
definition が unique / unrecovered / same-module / no-argument で definition-local
context を持たず、RHS attribute が argument-free same-module attribute symbol に resolve し、
source order が `B -> A -> reserve` である場合だけ、real one-edge chain
`A -> B -> marked set` または `A -> B -> marked object` を消費してよい。checker は両方の
real expansion、reserve-head attribute、terminal RHS attribute を消費し、real full
attributed-type existential evidence extraction が存在するまで
`checker.declaration.deferred.evidence_query` で fail closed する。deeper chain、mixed
bare/attributed use、attributed dependency、import、ambiguous symbol、attribute / mode
argument、contextual / parameterized definition、positive attributed-type acceptance、
CoreIr、ControlFlowIr、VC、proof payload はこの slice の外に残る。

task 67 は reserve type expression 内の structure-qualified attribute reference の
境界を記録する。`LocalStruct.marked LocalStruct` のような source expression は
Chapter 3 と 6 の type-expression syntax として有効で、Chapter 5 の local structure
declaration を使うが、現在の checker-owned `AttributeInput` payload は resolved
attribute symbol、polarity、argument、range、spelling だけを保持し、structure
qualifier や attribute-owner provenance を持たない。
そのため active runner は structure-qualified attribute reference を unqualified
same-module attribute payload に書き換えず、
`type_elaboration.external_dependency.ast_payload_extraction` に残さなければならない。
これは diagnostic boundary coverage に限る。real qualified attribute payload を
昇格せず、same-module no-argument unqualified attribute slice を変更せず、
existential evidence、CoreIr、ControlFlowIr、VC、proof payload も捏造しない。

task 68 は reserve type expression 内の argument-bearing mode head の境界を記録する。
`Element of a` のような source は Chapter 3 の type-expression syntax として有効で、
same-module mode surface に現れることができるが、現在の reserve source bridge は
argument-free local mode / structure head だけを許可し、real term / type-argument
provenance payload を持たない。そのため active runner は checker mode expansion、arity
matching、positive type elaboration へ進む前に、この source family を
`type_elaboration.external_dependency.ast_payload_extraction` に残さなければならない。
これは diagnostic boundary coverage に限る。mode argument を `TypeExpressionInput` に
昇格せず、term payload を捏造せず、CoreIr、ControlFlowIr、VC、proof payload も昇格しない。

task 69 は reserve type expression 内の argument-bearing structure head の対応する境界を
記録する。`LocalStruct of a` のような source は Chapter 3 と Chapter 5 の
type-expression syntax として有効で、`of` parameter surface を持つ same-module
structure declaration を指せるが、現在の reserve source bridge は引き続き
argument-free local mode / structure head だけを許可し、real term / type-argument
provenance payload を持たない。そのため active runner は
structure argument payload extraction、arity matching、base-shape /
constructor-witness evidence、positive structure type elaboration へ進む前に、この
source family を `type_elaboration.external_dependency.ast_payload_extraction` に残さなければ
ならない。これは diagnostic boundary coverage に限る。structure argument を
`TypeExpressionInput` に昇格せず、term payload を捏造せず、CoreIr、ControlFlowIr、VC、
proof payload も昇格しない。

task 70 は reserve type expression 内の local mode head について bracket-form の対応境界を
記録する。`Family[set]` のような source は第 3 章と第 7 章により bracket
type-argument syntax として有効で、same-module bracket-parameter mode declaration と並んで
現れ得るが、現在の reserve source bridge は引き続き argument-free local mode /
structure head だけを許可し、real bracket `type_arg_list` / `qua`-argument provenance
payload を持たない。そのため active runner は bracket type-argument payload extraction、
mode-head resolution、arity matching、mode expansion、positive type elaboration へ進む前に、
この source family を `type_elaboration.external_dependency.ast_payload_extraction` に残さなければならない。
これは diagnostic boundary coverage に限る。bracket argument を `TypeExpressionInput` に
昇格せず、`qua` payload や term payload を捏造せず、CoreIr、ControlFlowIr、VC、proof
payload も昇格しない。

task 71 は reserve type expression 内の local structure head について bracket-form の対応境界を
記録する。`LocalStruct[set]` のような source は第 3 章と第 5 章により bracket
type-argument syntax として有効で、same-module bracket-parameter structure declaration と並んで
現れ得るが、現在の reserve source bridge は引き続き argument-free local mode /
structure head だけを許可し、real bracket `type_arg_list` / `qua`-argument /
structure argument provenance payload を持たない。そのため active runner は bracket
type-argument payload extraction、structure-head resolution、arity matching、
base-shape / constructor-witness evidence、positive structure type elaboration へ進む前に、
この source family を `type_elaboration.external_dependency.ast_payload_extraction` に残さなければならない。
これは diagnostic boundary coverage に限る。bracket argument を `TypeExpressionInput` に
昇格せず、`qua` payload、term payload、base-shape payload、constructor evidence を捏造せず、
CoreIr、ControlFlowIr、VC、proof payload も昇格しない。

task 60 はさらに、
mode definition が unique / unrecovered / preceding / no-argument で definition-local
context を持たず、structure definition が unique / unrecovered / same-module で mode
definition より前に現れ、同じ mode が同じ bridge input 内で bare reserve head としても
使われていない場合に限り、この direct structure-RHS expansion を attributed local-mode
reserve head に許可する。expanded attributed structure type は real base-shape /
constructor-witness evidence と full attributed-type existential evidence が存在するまで
`checker.declaration.deferred.evidence_query` で fail closed する。mixed bare/attributed use、
dependency / chain、imported / argument-bearing symbol、attributed structure RHS はこの slice
の外に残る。task 58 はさらに、
RHS が attributed builtin `set` / `object` type である bare same-module no-argument
local mode expansion を許可する。この case も real `ModeExpansion` を消費するため
missing mode-expansion payload diagnostic を出してはならない。ただし expanded
attributed type は real attributed-type existential evidence extraction が存在するまで
`checker.declaration.deferred.evidence_query` で fail closed する。task 61 はさらに、
mode definition が unique / unrecovered / preceding / no-argument で definition-local
context を持たず、同じ mode が同じ bridge input 内で bare reserve head としても
使われていない場合に限り、この direct attributed-builtin RHS expansion を attributed
local-mode reserve head に許可する。expanded attributed type は real full attributed-type
existential evidence extraction が存在するまで `checker.declaration.deferred.evidence_query`
で fail closed する。mixed bare/attributed use、dependency / chain、imported /
argument-bearing symbol、structure RHS、attributed structure RHS はこの slice の外に残る。
task 52 はさらに、
type argument を持たず attribute も付かない、unique な same-module `LocalSource`
`SymbolKind::Structure` entry に解決された reserve type head を許可する。これら
local-structure reserve declaration は declaration checking に到達し、real
base-shape / constructor-witness evidence extraction が存在するまで
`checker.declaration.deferred.evidence_query` で fail closed する。task 53 はその
local-structure slice に same-module source-derived attribute を持たせることを許可する。
この場合も、第 17 章が full normalized attributed type について existential evidence を
要求するため、positive acceptance には bare-structure base-shape evidence だけでは不十分で、
`checker.declaration.deferred.evidence_query` で fail closed する。この payload は
source/module identity、reserve item source range、各 binding の spelling と
declaration range、対応済み type-expression の spelling / range / head、および
対応済み same-module attribute の symbol / range / polarity、対応済み same-module
local-mode expansion payload を含まなければならない。
この seam は runner が successful bare-builtin slice と対応済み local-mode expansion
slice 用の既存 `TypedAst` / `ResolvedTypedAst` readiness check を組み立てるための
deterministic typed-site id を公開するが、`mizar-checker` が `mizar-syntax` を import すること、raw syntax を scan
すること、non-reserve declaration を accept すること、imported symbol を捏造すること、
mode expansion や existential / base-shape evidence を捏造すること、local mode head に
imported または argument-bearing source attribute を付けること、argument-bearing symbol
head を accept すること、CoreIr / ControlFlowIr / VC / proof execution を主張することは
許可しない。

必須 behavior:

- `let`、definition parameter、quantified variable、`given`、`consider`、`take`
  binder は `BindingId` と `TypedSiteRef` に link された `TypeEntry` を受け取る;
- reserved variable は、explicit default type-site payload がその occurrence は local
  binding に shadow されていないと示す場合だけ、その payload を使う;
- `set` declaration は explicit normalized type が供給される場合はそれを attach し、
  payload がまだない場合は right-hand-side inference について deferred diagnostic を記録する;
- `deffunc` と `defpred` の formal は explicit formal payload が供給される場合に local
  definition parameter として check する。body checking は term / formula inference へ defer する;
- `reconsider x as T` は current site で existing binding の type view を更新し、
  `reconsider y = t as T` は新しい local binding を導入する;
- attributed / constrained type を持つ declaration は、後続 sethood / existence handling
  が必要であることを mark する。coercion と evidence check が存在してから、task 10 が対応する
  `InitialObligation` を発行する;
- checked declaration 上の `such that`、`given`、assumption-like clause は、導入元
  context にだけ `Assumed` fact を追加する。導入元 declaration が partial または error
  の場合、task 8 はそれらの assumption payload を explicit diagnostic とともに drop し、
  active evidence として公開しない。full fact query API は task 11 が所有する。

invalid declaration は explicit diagnostic と partial entry を生成しなければならない。
known fact を捏造したり、registration を silently activate したり、source-shaped typed site
を drop してはならない。

## Task 9: term and formula type inference

task 9 がこの section を実装する。

term inference は各 typed term site に `TypeEntry` を記録する。formula inference は
well-formedness と formula structure が導入する task-9-local fact を記録する。
task 11 は、完全な deterministic fact query API と、task 9 に必要な最小 inference record を
超えた fact recording の拡張に責任を持ち続ける。

task 9 は checker-owned な term/formula payload を使う。現在の resolver は
AST 全体の typed term/formula table、built-in numeric type payload、
functor/predicate の candidate signature、selector と structure field payload、
source `qua` の coercion evidence、sethood / non-emptiness の evidence query をまだ
公開していない。そのため task 9 は raw syntax を歩いてこれらの入力を再構築してはならない。
caller が渡す明示 payload、すなわち term/formula site、reference 用の binding id または
resolver symbol、明示 result / expected type expression、`it` 用の current-result type
payload、後続 overload phase が root selection を完了する必要がある unresolved candidate
group を消費する。不足している resolver/source payload は `external_dependency_gap` または
deferred diagnostic として分類する。

実装表面は `TermFormulaChecker` である。これは normalized type expression、checked term /
formula record、task-local な `OpenCandidateSetTable`、`TypeTable`、`TypeFactTable`、
diagnostic を含む `TermFormulaInferenceOutput` を生成する。`TypedAst` はすでに
`TypeEntryActual::CandidateSet(OpenCandidateSetId)` を予約しているが、candidate-set payload
table を最初に所有するのは task 9 の `type_checker` である。後続の overload と
`ResolvedTypedAst` task は、その table を消費または投影し、candidate id だけを final overload
decision として扱わない。

formula の well-formedness は checked-formula status と linked fact に記録し、成功した
term type を捏造しない。source-written `qua`、sethood check、non-emptiness check は task 9 で
deferred diagnostic と open type view を記録してよいが、`CoercionTable` entry と
`InitialObligation` を発行するのは task 10 だけである。

task 9 の分類:

- `external_dependency_gap`: AST 全体の term/formula extraction、numeric builtin、
  functor/predicate signature、selector/structure signature、source `qua` evidence の
  resolver/source payload 欠落。これらは degraded diagnostic と partial/skipped checked
  term または formula として記録する。
- `deferred`: task 10 が意図的に所有する sethood、non-emptiness、source-`qua` の
  coercion/obligation 発行。これらは deferred requirement marker または diagnostic として
  記録し、`CoercionTable` / `InitialObligationTable` entry としては記録しない。

term rules:

- variable reference は `BindingEnv` lookup result を消費し、selected binding または
  resolver symbol を typed site に付ける;
- `it` は current result type を提供する definition/property context 内でだけ valid である;
- numeral は resolver が公開する built-in numeric type payload を受け取るか、payload が
  欠ける場合は degraded external-gap type を受け取る;
- functor application は、final overload root selection が phase 6 で決定的でない場合、
  candidate group を保持してよい;
- selector access は、供給された result または candidate payload を記録し、selector/signature
  payload 欠落を MC-G017 として degrade する。field/property visibility validation は
  resolver-exposed selector signature を待つ;
- structure constructor は、供給された result payload を記録し、structure-field payload 欠落を
  MC-G017 として degrade する。field coverage と value-type checking は resolver-exposed
  structure signature を待つ;
- set enumeration は供給済みの set-like result type を記録し、result-type payload 欠落時は
  degrade する。Chapter 13 は enumeration に generator-domain sethood requirement を課さない;
- set comprehension も供給済み result type を記録する一方、外部 owner の generator-domain
  evidence 欠落は、generator ごとの payload ではなく、現行の粗い deferred sethood marker
  だけで表しうる;
- `the T` は choice-like typed term と `T` の deferred non-emptiness requirement を記録するが、
  proof-owned id は割り当てない;
- source-written `qua` は後続 checking が必要とする source view と deferred source-`qua`
  requirement を記録する。`SourceQua` coercion candidate を作るのは task 10 である。

formula rules:

- predicate application は candidate argument type を check するが、final root selection が
  ambiguous な場合は phase 8 のため candidate group を保持する;
- built-in `=`、`<>`、`in` form は term well-formedness を check し、appropriate
  expected-type constraint を追加する;
- type assertion は exactly one ready subject と normalized asserted type を要求し、
  normalized identity を reflexive admissible とする。known non-identical type は
  widening/`qua` evidence が得られるまで explicit external reachability payload gap
  上の partial に残す。attribute-assertion admissibility は real radix/parameter と
  attribute-chain semantic payload が得られるまで deferred のままにする;
- logical connective は formula type/well-formedness state を保持し、fact の結合は
  statement が所有する explicit assumption/conclusion rule を通じてのみ行う;
- quantified formula は `BindingEnv` を通じて binder context を作り、その下で body を check する。

site に matching typed candidate がないことを判断する十分な local payload がある場合、
checker は successful type を捏造せず `Unknown` または `Error` status を記録する。

## Task 10: coercion candidates and initial obligations

task 10 がこの section を実装する。

coercion entry は checker が見つけた candidate であり、final inserted view ではない。

task 10 は checker-owned な coercion と initial-obligation payload を使う。現在の
resolver/checker boundary は、AST 全体の coercion request table、active dependency-summary
fact database、structure inheritance graph、cluster-closure evidence、sethood evidence、
non-emptiness evidence、proof-query result をまだ公開していない。task 10 は明示された
source site、normalized source / target type payload、supporting fact id、obligation request
payload を消費する。不足する resolver/dependency payload は `external_dependency_gap` であり、
diagnostic 付きの blocked または rejected entry を生成する。raw syntax walk、registration
closure、proof search、fact の捏造によって修復してはならない。

実装表面は `CoercionObligationChecker` である。これは normalized type expression、
`TypeEntry`、`CoercionTable`、`InitialObligationTable`、必要なら obligation-backed fact、
diagnostic を含む `CoercionCheckingOutput` を生成する。supporting fact は明示的に渡され、
input fact table に consumable status で存在する場合だけ受け入れる。output は input fact を
保持し、coercion support id が handoff で有効なままになるよう local obligation fact を
追加する。built-in radix widening は checker-local builtin fact を追加してよい。
structure-inheritance と activated-summary evidence は supplied consumable supporting
fact id を必要とし、enum marker だけでは受け入れない。full fact query API は引き続き
task 11 が所有する。

必須 behavior:

- widening candidate が proof-free であるのは、known type fact、local builtin fact として
  記録された built-in radix widening、supplied fact で表された structure inheritance
  payload、または task-scoped seam を通じた supplied fact で表された already activated
  dependency summary に支えられる場合だけである;
- source-written `qua` は statically checkable upcast または compatible view に対してだけ
  valid であり、narrowing proof として使ってはならない;
- explicit な narrowing は、task 10 input が target type を既に支える `KnownFacts` evidence
  と consumable supporting fact id を supplied している場合を除き、`InitialObligation` を作る;
- task 47 は explicit payload 上で `reconsider` の explicit / omitted を区別する。
  `CoercionJustification::Explicit` は task-10 の obligation 経路を維持する。
  `CoercionJustification::Omitted` は、known local fact、built-in radix widening、
  structure inheritance、activated summary / cluster closure、static upcast、compatible view
  に由来する proof-free evidence が consumable fact に支えられる場合だけ受理する。evidence
  marker だけでは omitted form を discharge できない;
- omitted `reconsider` が proof-free evidence を欠く、または evidence が non-consumable なら、
  `type.narrowing_requires_proof` を emit し、rejected/degraded coercion を記録し、implicit
  obligation、proof search、hidden `by` は作らない;
- sethood と non-emptiness requirement は source assumption と deterministic local id を持つ
  `InitialObligation` を作る;
- failed / unsupported coercion は diagnostic 付きの `Blocked` または `Rejected` entry として残る。

`InitialObligationId` が phase-6 boundary である。task 10 は `VcId`、
`ObligationAnchor`、prover status、proof witness、accepted verifier status を割り当ててはならない。
context-sensitive な `Assumed` fact は omitted `reconsider` helper が直接 query しない。upstream
producer が fact-query / context boundary を使って consumable supporting fact id を先に供給する
必要がある。source-derived reconsider/coercion extraction は MC-G019/MC-G020 の下で deferred の
ままである。

## Task 11: type facts and queries

task 11 がこの section を実装する。

type fact は declaration checking、inference、coercion checking、後続
registration/overload phase が共有する local currency である。

task 11 は declaration checking、term/formula inference、coercion checking が既に記録した
fact を使う。source walker や新しい registration fact inference は追加しない。現在の
resolver/checker boundary は、AST 全体の statement/proof assumption table、theorem acceptance
payload、phase-7 `ResolutionTrace` fact をまだ公開していない。これらは後続 task の
`external_dependency_gap` として残る。

実装表面は `TypeFactQueryEngine` である。これは `TypeFactTable` と optional な
`LocalTypeContextTable` を消費し、`TypeFactQuery` / `TypeFactQueryOutput` を通じて
deterministic point query に答える。query output は explicit な `Satisfied`、`Missing`、
`Contradicted` status、canonical order の matched fact id、contradiction 用 query-local
diagnostic を持つ。context で active visible fact を列挙してよいが、新しい fact の導出や
table entry の書き換えはしない。

`TypeFactQuery` は subject、predicate、polarity、optional local context id で match する。
provenance は point-query matching には参加しない。provenance は canonical output ordering、
traceability、後続 explanation のために保持する。positive query は subject/predicate の
active positive fact が少なくとも 1 つ visible で、同じ subject/predicate の active negative
fact が visible でない場合に `Satisfied` になる。negative query は対称である。matching-polarity
fact も opposite-polarity fact も active visible でない場合、query は `Missing` になる。

contradiction は visibility/status filtering 後に、同じ subject と predicate に対して
opposite polarity の active visible fact があることを意味し、provenance は無視する。
contradicted query は両 polarity の active same subject/predicate fact id を canonical order で返し、
query-local diagnostic を発行する。underlying fact table は変更しない。

Assumed fact は、query が context id を supplied し、engine が `LocalTypeContextTable` を持ち、
その context が fact を consume できる場合だけ visible である。engine が context table を持たない、
query が context を省く、context id が missing、または fact がその context の visibility chain の外に
ある場合、`Assumed` fact はその query では inactive である。これにより、local context を選んでいない
registration、overload、dependency-summary consumer に assumption が漏れない。

fact source:

- `Declared`: binding declaration と type-expression site;
- `Assumed`: local assumption、`such that`、`given`、proof-context assumption;
- `Inferred`: built-in widening や selector result typing などの direct checker rule;
- `Obligation`: claim が `InitialObligationId` で表現される fact;
- `Builtin`: `object`、`set`、equality、membership に関する built-in fact;
- `Registration`: `ResolutionTrace` step を持つ後続 phase-7 closure のために予約する。

query rules:

- `Known` fact だけが unconditional に consumable である;
- `Assumed` fact は、`LocalTypeContextTable` に記録された introducing context または visible
  descendant でだけ consumable である;
- `PendingObligation`、`Degraded`、`Rejected` fact は active evidence ではない;
- `TypeFactTable` の semantic key は subject、predicate、polarity、provenance を含む。
  assumption visibility は insertion order ではなく `LocalTypeContextTable` の
  `introduced_assumptions` / `visible_facts` で制御する;
- contradictory active fact は insertion order で解決せず、query diagnostic と explicit な
  `Contradicted` status を作る。

phase 6 は後続 registration resolution が必要とする fact を記録してよいが、phase 7 が
対応 derivation を所有する前に `Registration` provenance や trace step を作ってはならない。

## partial typing and recovery

recoverable error は explicit partial state を残さなければならない。

- unresolved / ambiguous type head は degraded `NormalizedType` と `TypeEntry` diagnostic を作る;
- unresolved term、missing binding payload、missing signature payload、impossible candidate group は
  `Unknown`、`Error`、`Skipped` entry を作る;
- degraded site から派生した fact/coercion は `Degraded`、`Blocked`、`Rejected` status を持つ;
- diagnostic は primary source range と stable secondary key を保持する;
- 後続 phase は type、fact、coercion、obligation entry を消費する前に status predicate を
  check しなければならない。

recovery は under-approximation policy である。checker は fact を省略して diagnostic を
出してよいが、verified fact を捏造したり、registration を activate したり、obligation を
accepted と mark してはならない。

## diagnostics

public checker diagnostic code-space が割り当てられるまで、task-local diagnostic は stable
message key を使う。

必須 diagnostic class:

- unknown / ambiguous type head;
- unsupported または missing resolver signature payload;
- wrong type-argument arity / kind;
- contradictory attribute;
- uninhabited / unsupported attributed declaration;
- illegal declaration / local-binding type;
- invalid `qua` target または obligation なしの narrowing;
- failed sethood / non-emptiness requirement;
- term/formula kind mismatch;
- ambiguous / impossible candidate group;
- partial-typing recovery boundary。

diagnostic は proof evidence ではない。degraded table entry を説明してよいが、supporting
fact として使ってはならない。

## determinism

task 7-11 implementation は deterministic output を保たなければならない。

- normalized type id は canonical type key で割り当てる;
- declaration、term、formula、coercion、obligation、fact、diagnostic の iteration order は
  hash-map iteration に依存しない;
- candidate group は resolver candidate identity、mandatory type constraint、source range、
  stable local id の順で sort する;
- debug rendering は host path、memory address、nondeterministic map order を含めず
  `typed-ast-debug-v1` を拡張する;
- 同等の `ResolvedAst`、`SymbolEnv`、`BindingEnv`、dependency summary、checker configuration は
  同等の `TypedAst` table を生成する。

## Public Enum Policy

task 31 は frontend task-25 の public-enum decision procedure をこの module に適用する。
`type_checker` の public checker-owned enum はすべて forward-compatible API surface であり、
`#[non_exhaustive]` を維持しなければならない。downstream consumer は wildcard または
fallback arm を保持する。checker 内部の match は、仕様化済み behavior を実装するために
現在表現されている variant へ exhaustive のままにしてよい。

| enum | decision |
|---|---|
| `CoercionRequestKind` | 前方互換; coercion request category は後続の view と obligation form とともに増える可能性がある。 |
| `CoercionJustification` | 前方互換; proof-block と artifact-backed evidence payload が着地すると justification class は増える可能性がある。 |
| `CoercionEvidence` | 前方互換; coercion evidence は proof、registration、artifact source とともに増える可能性がある。 |
| `CoercionDeferredReason` | 前方互換; deferred coercion reason は external payload gap が閉じるにつれて増える可能性がある。 |
| `InitialRequirementKind` | 前方互換; initial requirement category は VC/proof integration とともに増える可能性がある。 |
| `TypeFactQueryStatus` | 前方互換; fact query outcome は contradiction と evidence policy の成熟に伴い増える可能性がある。 |
| `TermKind` | 前方互換; term category は source-to-checker extraction とともに増える可能性がある。 |
| `TermReference` | 前方互換; term reference は追加の checker-owned identity anchor を得る可能性がある。 |
| `TermDeferredReason` | 前方互換; deferred term reason は source payload が入るにつれて増える可能性がある。 |
| `FormulaKind` | 前方互換; formula category は statement/proof extraction とともに増える可能性がある。 |
| `FormulaDeferredReason` | 前方互換; deferred formula reason は source payload が入るにつれて増える可能性がある。 |
| `CandidateIdentity` | 前方互換; open candidate identity はより豊かな overload extraction とともに増える可能性がある。 |
| `CandidateSetKind` | 前方互換; candidate-set category は後続 overload phase とともに増える可能性がある。 |
| `CandidateSetStatus` | 前方互換; candidate-set state は deferred と failed-site handling とともに増える可能性がある。 |
| `CandidateStatus` | 前方互換; candidate state は evidence と recovery handling とともに増える可能性がある。 |
| `TermStatus` | 前方互換; checked-term state は partial inference policy とともに増える可能性がある。 |
| `FormulaStatus` | 前方互換; checked-formula state は partial inference policy とともに増える可能性がある。 |
| `DeclarationKind` | 前方互換; declaration kind はより多くの Mizar binding form とともに増える可能性がある。 |
| `DeclarationDeferredReason` | 前方互換; deferred declaration reason は extraction gap が閉じるにつれて増える可能性がある。 |
| `DeclarationStatus` | 前方互換; declaration state は local recovery と handoff policy とともに増える可能性がある。 |
| `TypeHeadInput` | 前方互換; input type head は resolver と built-in payload とともに増える可能性がある。 |
| `AttributePolarity` | 前方互換; type predicate がより豊かな qualifier を得る場合、attribute polarity は増える可能性がある。 |
| `TypeHeadRef` | 前方互換; normalized type head は structure、mode、built-in とともに増える可能性がある。 |
| `TypeHeadErrorKind` | 前方互換; type-head error category は resolver diagnostic とともに増える可能性がある。 |
| `NormalizedTypeStatus` | 前方互換; normalized type state は recovery と artifact handoff policy とともに増える可能性がある。 |

この module が所有する exhaustive public enum exception はない。

## task 7-11 の予定テスト

task 7 は Rust test で次を覆う。

- attribute sorting、deduplication、polarity、contradiction diagnostic;
- built-in singleton head、radix head のまま残る structure head、recursive
  type-argument normalization;
- explicit mode-expansion provider が payload を供給する場合の mode unfolding idempotence;
- signature payload 欠落時の degraded mode/type entry;
- unknown / ambiguous head、wrong arity/kind diagnostic、source-range
  preservation;
- deterministic normalized type id;
- type normalization が cluster closure で degraded type を repair しない guard。

task 8 は Rust test で次を覆う。

- `let`、quantified binder、definition parameter、reserved variable、`set`、
  `deffunc`、`defpred`、`reconsider` の binding type attachment;
- explicit reserved-variable default payload handling と reserve / source payload
  欠落時の deferred diagnostic;
- RHS または body payload がまだない場合の `set`、`deffunc`、`defpred` deferred diagnostic;
- `reconsider` の両形式と constrained declaration の deferred obligation diagnostic;
- local assumption visibility と context snapshot update;
- invalid declaration 後の partial entry;
- local-context、type-entry、diagnostic、debug-rendering order の determinism。

task 9 は Rust test で次を覆う。

- variable、numeral、selector、structure、set-expression、choice、`qua`、
  parenthesized term site;
- `it` validity と built-in `=` / `<>` / `in` expected-type constraint;
- predicate application、type assertion、attribute assertion、connective、
  quantified formula;
- overload resolution のため open に残る candidate group;
- sorted candidate group と deterministic term/formula/diagnostic rendering;
- recovery が successful type を捏造しない rule を含む unknown/error/skipped
  partial typing。

task 10 は Rust test で次を覆う。

- widening、source `qua`、narrowing coercion candidate;
- narrowing proof にならない invalid `qua` target;
- `Blocked` または `Rejected` のまま残る failed / unsupported coercion;
- sethood / non-emptiness initial obligation;
- `reconsider` obligation の source range と assumption list;
- coercion、obligation、diagnostic、debug-rendering order の determinism;
- `VcId` や proof-owned status を割り当てない boundary guard。

task 11 は Rust test で次を覆う。

- fact deduplication と canonical query order;
- `Known`、`Assumed`、`PendingObligation`、`Degraded`、`Rejected` の consumability rule;
- contradiction diagnostic;
- phase 7 trace ownership 前に `Registration` provenance が存在しないこと。

task 6 は documentation-only なので `.miz` checker-stage fixture は不要である。
最初の active `type_elaboration` corpus runner と traceability metadata は引き続き
task 12 が所有する。

## task 6 の分類

| class | finding | action |
|---|---|---|
| `spec_gap` | named phase-6 section について task 6 を block する spec gap は残っていない。chapter 03、chapter 08、chapter 13、architecture 04 が normalization、declaration checking、term-expression inference、coercion candidate、fact、recovery に十分な authority を与える。 | task 6 の review、verification、commit 後に task 7 へ進む。 |
| `test_gap` | active checker-stage `.miz` coverage と `type_elaboration` runner はまだ存在しない。 | task 7-11 が task-local Rust test を追加する。task 12 が active corpus coverage と traceability metadata を所有する。 |
| `design_drift` | architecture 04 の例は broad `TypeContext` と `CoercionCandidateTable` 名を使うが、既存 checker docs は `BindingEnv`、immutable `LocalTypeContextTable`、`CoercionTable` を使う。 | 本 spec は refined checker module split を保ち、`CoercionTable` entry を後続 phase が解決するまで candidate として扱う。 |
| `source_drift` | task 6 時点では `src/type_checker.rs` はまだ存在しなかった。 | task 7 が module を作成し `lib.rs` から export したことで解決済みである。task 6 では source repair しなかった。 |
| `external_dependency_gap` | 複数の実装 seam は mode unfolding、structure field、attribute、functor/predicate candidate、built-in、dependency activated summary の resolver-exposed signature payload に依存する。public checker diagnostic code も未割り当てである。 | 実装 task は公開済み resolver/artifact payload だけを消費する。不足 payload は external dependency gap または degraded diagnostic とし、direct raw-syntax reconstruction を追加しない。 |
| `deferred` | registration closure、reduction normalization、final overload selection、inserted overload-disambiguating `qua` view、VC generation、proof acceptance、kernel replay、artifact publication は task 6 と phase 6 の外に残る。 | 後続 checker task と downstream crate task がこれらの境界を所有する。 |


## task 201 exact one-edge formula-side immediate-radix local-mode asserted-head checker bridge

task 201 は `BaseModeRadixAssertedHeadDef` が `BaseModeRadixAssertedHead` を builtin `set` として定義し、`OuterModeRadixAssertedHeadDef` が `OuterModeRadixAssertedHead` をその base mode として定義し、outer mode の `x` reserve 1 個と `ChainedLocalModeRadixAssertedHeadPayloadBoundary: x is BaseModeRadixAssertedHead;` を持つ exact source だけを対象とする。runner は Tasks 56/146 の real AST-derived expansion 2 個と Task 184 の formula-side local asserted-type consumer を合成する。closed `Builtin` / `SameMode` / `BindingImmediateRadix` relation は既存 route をすべて保ち、outer binding expansion の real immediate radix である resolved base symbol だけを受理する。

route は distinct raw Outer/Base symbol、site、range を保持し、ordinal 1 を `BindingId(0)` に解決し、expansion 2 個だけを消費し、known entry 3 個を `BaseModeRadixAssertedHeadDef` RHS anchor の `BuiltinSet` identity 1 個へ normalize する。immutable output は `Inferred` variable 1 個、expected constraint/candidate/fact/diagnostic/deferred reason 0 個、normalized-reflexive `Checked` `TypeAssertion` 1 個を持つ。exact structural/provenance、independent payload corruption、Task 146/184 isolation、real frontend/resolver-sidecar guard が active runner 149 を保護する。object-terminal、deeper、unrelated、imported、attributed、argument-bearing asserted head、general reachability/widening/`qua`、declaration/theorem acceptance、truth/fact、closure/order、broader child-graph semantics、proof/CoreIr/ControlFlowIr/VC、general chain semantics は deferred のままとする。checker source または module-layout change は不要であった。


## task 202 exact one-edge object-terminal formula-side immediate-radix local-mode asserted-head checker bridge

task 202 は `BaseObjectModeRadixAssertedHead -> object`、`OuterObjectModeRadixAssertedHead -> BaseObjectModeRadixAssertedHead`、outer-mode `x` reserve 1 個、`ChainedLocalObjectModeRadixAssertedHeadPayloadBoundary: x is BaseObjectModeRadixAssertedHead;` だけを対象とする。Tasks 56/147 の real two-expansion object normalization、Task 185 の object formula consumer、Task 201 の closed `BindingImmediateRadix` relation を合成し、relation または既存 route は変更しない。

route は distinct Outer/Base resolved symbol/site/range を保持し、ordinal 1 を `BindingId(0)` に解決し、expansion 2 個だけを消費し、known entry 3 個を `BaseObjectModeRadixAssertedHeadDef` RHS anchor の `BuiltinObject` 1 個へ normalize する。immutable output は `Inferred` variable 1 個、expected constraint/candidate/fact/diagnostic/deferred reason 0 個、object/set coercion のない `Checked` `TypeAssertion` 1 個を持つ。exact structural/provenance/corruption、real Tasks 147/185/201 bidirectional isolation、frontend/resolver-sidecar guard が active runner 150 を保護する。Task 201 以外の追加 set-terminal/deeper/unrelated/imported/attributed/argument-bearing head、reachability/widening/`qua`、acceptance/truth/fact、child graph、proof/CoreIr/ControlFlowIr/VC、general chain は deferred のままとする。checker source または module-layout change は不要であった。


## task 203 exact two-edge set-terminal formula-side immediate-radix local-mode asserted-head checker bridge

task 203 は `BaseTwoEdgeModeRadixAssertedHead -> set`、`MiddleTwoEdgeModeRadixAssertedHead -> BaseTwoEdgeModeRadixAssertedHead`、`OuterTwoEdgeModeRadixAssertedHead -> MiddleTwoEdgeModeRadixAssertedHead` という exact local definition 3 個、outer-mode `x` reserve 1 個、`TwoEdgeLocalModeRadixAssertedHeadPayloadBoundary: x is MiddleTwoEdgeModeRadixAssertedHead;` だけを対象とする。Task 72 の real three-expansion set producer、Task 186 の formula consumer、Tasks 201/202 の変更しない closed `BindingImmediateRadix` relation を合成する。real immediate Outer-to-Middle edge だけを受理し、two-hop reachability は導入しない。

route は distinct Outer-subject/Middle-asserted resolved symbol/site/range を保持し、ordinal 1 を `BindingId(0)` に解決し、expansion 3 個だけを消費し、known entry 3 個を `BaseTwoEdgeModeRadixAssertedHeadDef` RHS anchor の `BuiltinSet` identity へ normalize する。immutable output は `Inferred` variable 1 個、expected constraint/candidate/fact/diagnostic/deferred reason 0 個、normalized-reflexive `Checked` `TypeAssertion` 1 個を持つ。exact structural/provenance/corruption check、全 nonidentity definition reordering、duplicate/spelling/imported/ambiguous/deeper near miss、real Tasks 122/148/149/186/187/201/202 bidirectional isolation、frontend/resolver sidecar が active runner 151 を保護する。両 link をまたぐ Base assertion、object-terminal sibling、broader head/semantics、proof/CoreIr/ControlFlowIr/VC、general chain behavior は deferred のままとする。checker source または module-layout change は不要であった。


## task 204 exact two-edge object-terminal formula-side immediate-radix local-mode asserted-head checker bridge

task 204 は `BaseTwoEdgeObjectModeRadixAssertedHead -> object`、`MiddleTwoEdgeObjectModeRadixAssertedHead -> BaseTwoEdgeObjectModeRadixAssertedHead`、`OuterTwoEdgeObjectModeRadixAssertedHead -> MiddleTwoEdgeObjectModeRadixAssertedHead` という exact local definition 3 個、outer-mode `x` reserve 1 個、`TwoEdgeLocalObjectModeRadixAssertedHeadPayloadBoundary: x is MiddleTwoEdgeObjectModeRadixAssertedHead;` だけを対象とする。Task 72 の real three-expansion object producer、Task 187 の formula consumer、Tasks 202/203 の変更しない closed `BindingImmediateRadix` relation を合成する。real immediate Outer-to-Middle edge だけを受理し、two-hop reachability は導入しない。

route は distinct Outer-subject/Middle-asserted resolved symbol/site/range を保持し、ordinal 1 を `BindingId(0)` に解決し、expansion 3 個だけを消費し、known entry 3 個を `BaseTwoEdgeObjectModeRadixAssertedHeadDef` RHS anchor の `BuiltinObject` identity へ normalize する。immutable output は `Inferred` variable 1 個、expected constraint/candidate/fact/diagnostic/deferred reason 0 個、object/set coercion のない normalized-reflexive `Checked` `TypeAssertion` 1 個を持つ。exact structural/provenance/corruption check、全 nonidentity definition order、duplicate/spelling/imported/ambiguous/deeper near miss、real Tasks 189/145/147/149/187/202 および set-terminal Tasks 148/186/203 bidirectional isolation、frontend/resolver sidecar が active runner 152 を保護する。両 link をまたぐ Base assertion、broader head/semantics、proof/CoreIr/ControlFlowIr/VC、general chain behavior は deferred のままとする。checker source または module-layout change は不要であった。

## task 205 exact three-edge set-terminal formula-side immediate-radix local-mode asserted-head checker bridge

task 205 は `BaseThreeEdgeModeRadixAssertedHead -> set`、`InnerThreeEdgeModeRadixAssertedHead -> BaseThreeEdgeModeRadixAssertedHead`、`MiddleThreeEdgeModeRadixAssertedHead -> InnerThreeEdgeModeRadixAssertedHead`、`OuterThreeEdgeModeRadixAssertedHead -> MiddleThreeEdgeModeRadixAssertedHead` という exact local definition 4 個、outer-mode `x` reserve 1 個、`ThreeEdgeLocalModeRadixAssertedHeadPayloadBoundary: x is MiddleThreeEdgeModeRadixAssertedHead;` だけを対象とする。Task 73 の real four-expansion set-terminal producer、Task 195 の formula consumer、Tasks 201/203/204 の変更しない closed `BindingImmediateRadix` relation を合成する。real immediate Outer-to-Middle edge だけを受理し、multi-hop Inner/Base reachability は導入しない。

route は distinct Outer-subject/Middle-asserted resolved symbol/site/range を保持し、ordinal 1 を `BindingId(0)` に解決し、expansion 4 個だけを消費し、known entry 3 個を `BaseThreeEdgeModeRadixAssertedHeadDef` RHS anchor の `BuiltinSet` identity へ normalize する。immutable output は `Inferred` variable 1 個、expected constraint/candidate/fact/diagnostic/deferred reason 0 個、normalized-reflexive `Checked` `TypeAssertion` 1 個を持つ。exact structural/provenance/corruption check、全 23 nonidentity definition order、missing/duplicate/label/spelling/radix および imported/ambiguous/deeper/multi-hop near miss、set Tasks 122/138/146/148/150/195/201/203 および object Tasks 189/145/147/149/151/196/202/204 との bidirectional isolation、frontend/resolver sidecar が active runner 153 を保護する。multi-hop Inner/Base assertion、matching object sibling、broader head/semantics、proof/CoreIr/ControlFlowIr/VC、general chain behavior は deferred のままとする。checker source または module-layout change は不要であった。

## task 206 exact three-edge object-terminal formula-side immediate-radix local-mode asserted-head checker bridge

task 206 は `BaseThreeEdgeObjectModeRadixAssertedHead -> object`、`InnerThreeEdgeObjectModeRadixAssertedHead -> BaseThreeEdgeObjectModeRadixAssertedHead`、`MiddleThreeEdgeObjectModeRadixAssertedHead -> InnerThreeEdgeObjectModeRadixAssertedHead`、`OuterThreeEdgeObjectModeRadixAssertedHead -> MiddleThreeEdgeObjectModeRadixAssertedHead` という exact local definition 4 個、outer-mode `x` reserve 1 個、`ThreeEdgeLocalObjectModeRadixAssertedHeadPayloadBoundary: x is MiddleThreeEdgeObjectModeRadixAssertedHead;` だけを対象とする。Task 73 の real four-expansion object-terminal producer、Task 196 の formula consumer、Tasks 201/204/205 の変更しない closed `BindingImmediateRadix` relation を合成する。real immediate Outer-to-Middle edge だけを受理し、multi-hop Inner/Base reachability または object/set coercion は導入しない。

route は distinct Outer-subject/Middle-asserted resolved symbol/site/range を保持し、ordinal 1 を `BindingId(0)` に解決し、expansion 4 個だけを消費し、known entry 3 個を `BaseThreeEdgeObjectModeRadixAssertedHeadDef` RHS anchor の `BuiltinObject` identity へ normalize する。immutable output は `Inferred` variable 1 個、expected constraint/candidate/fact/diagnostic/deferred reason 0 個、normalized-reflexive `Checked` `TypeAssertion` 1 個を持つ。exact structural/provenance/corruption check、全 23 nonidentity definition order、各 definition の missing/duplicate/label/spelling/radix および imported/ambiguous/deeper/multi-hop/local-other near miss、set Tasks 122/138/146/148/150/195/201/203/205 および object Tasks 189/145/147/149/151/196/202/204 との bidirectional isolation、frontend/resolver sidecar が active runner 154 を保護する。multi-hop Inner/Base assertion、broader head/semantics、proof/CoreIr/ControlFlowIr/VC、general chain behavior は deferred のままとする。checker source または module-layout change は不要であった。

## task 207 exact four-edge set-terminal formula-side immediate-radix local-mode asserted-head checker bridge

task 207 は ordered local definition 5 個 `BaseFourEdgeModeRadixAssertedHead -> set`、`InnerFourEdgeModeRadixAssertedHead -> BaseFourEdgeModeRadixAssertedHead`、`MiddleFourEdgeModeRadixAssertedHead -> InnerFourEdgeModeRadixAssertedHead`、`OuterFourEdgeModeRadixAssertedHead -> MiddleFourEdgeModeRadixAssertedHead`、`TooDeepFourEdgeModeRadixAssertedHead -> OuterFourEdgeModeRadixAssertedHead`、TooDeep-mode `x` reserve 1 個、`FourEdgeLocalModeRadixAssertedHeadPayloadBoundary: x is OuterFourEdgeModeRadixAssertedHead;` だけを対象とする。Task 74 の real five-expansion set-terminal producer、Task 197 の formula consumer、Tasks 201/203/205/206 の変更しない closed `BindingImmediateRadix` relation を合成する。real immediate TooDeep-to-Outer edge だけを受理し、multi-hop Middle/Inner/Base reachability は導入しない。

route は distinct TooDeep-subject/Outer-asserted resolved symbol/site/range を保持し、ordinal 1 を `BindingId(0)` に解決し、expansion 5 個だけを消費し、known entry 3 個を `BaseFourEdgeModeRadixAssertedHeadDef` RHS anchor の `BuiltinSet` identity へ normalize する。immutable output は `Inferred` variable 1 個、expected constraint/candidate/fact/diagnostic/deferred reason 0 個、normalized-reflexive `Checked` `TypeAssertion` 1 個を持つ。exact structural/provenance/corruption check、全 119 nonidentity definition order、各 definition/asserted-head near miss、全 symbol の imported/ambiguous check、declared set/object owner route 20 件との bidirectional isolation、frontend/resolver sidecar が active runner 155 を保護する。multi-hop Middle/Inner/Base assertion、matching object sibling、broader head/semantics、proof/CoreIr/ControlFlowIr/VC、general chain behavior は deferred のままとする。checker source または module-layout change は不要であった。

## task 208 exact four-edge object-terminal formula-side immediate-radix local-mode asserted-head checker bridge

task 208 は ordered local definition 5 個 `BaseFourEdgeObjectModeRadixAssertedHead -> object`、`InnerFourEdgeObjectModeRadixAssertedHead -> BaseFourEdgeObjectModeRadixAssertedHead`、`MiddleFourEdgeObjectModeRadixAssertedHead -> InnerFourEdgeObjectModeRadixAssertedHead`、`OuterFourEdgeObjectModeRadixAssertedHead -> MiddleFourEdgeObjectModeRadixAssertedHead`、`TooDeepFourEdgeObjectModeRadixAssertedHead -> OuterFourEdgeObjectModeRadixAssertedHead`、TooDeep-mode `x` reserve 1 個、`FourEdgeLocalObjectModeRadixAssertedHeadPayloadBoundary: x is OuterFourEdgeObjectModeRadixAssertedHead;` だけを対象とする。Tasks 74/153 の real five-expansion object-terminal producer、Task 198 の formula consumer、Tasks 202/204/206/207 の変更しない closed `BindingImmediateRadix` relation を合成する。real immediate TooDeep-to-Outer edge だけを受理し、multi-hop Middle/Inner/Base reachability または object/set coercion は導入しない。

route は distinct TooDeep-subject/Outer-asserted resolved symbol/site/range を保持し、ordinal 1 を `BindingId(0)` に解決し、expansion 5 個だけを消費し、known entry 3 個を `BaseFourEdgeObjectModeRadixAssertedHeadDef` RHS anchor の `BuiltinObject` identity へ normalize する。immutable output は `Inferred` variable 1 個、expected constraint/candidate/fact/diagnostic/deferred reason 0 個、normalized-reflexive `Checked` `TypeAssertion` 1 個を持つ。exact/corruption/all-119-orders/per-definition/reserve/formula/asserted-head/deeper/provenance guard、unrelated-import positive、全 symbol の imported/ambiguous check、expansion removal/corruption check、declared owner route 21 件との bidirectional isolation、immutable output、frontend/resolver sidecar が active runner 156 を保護する。multi-hop Middle/Inner/Base assertion、broader head/semantics、proof/CoreIr/ControlFlowIr/VC、general chain behavior は deferred のままとする。checker source または module-layout change は不要であった。

## task 209 exact seven-expansion set-terminal formula-side immediate-radix local-mode asserted-head checker bridge

task 209 は ordered `BaseMode -> set`、`ChainMode1 -> BaseMode` から `ChainMode6 -> ChainMode5`、ChainMode6 reserve 1 個、`LongLocalModeRadixAssertedHeadPayloadBoundary: x is ChainMode5;` だけを対象とする。Task 74 は real AST-derived expansion 7 個、Task 199 は formula consumer、Task 175 は builtin asserted-type sibling と reusable output guard を提供する。変更しない closed `BindingImmediateRadix` relation は real bare `ChainMode6 -> ChainMode5` immediate edge だけを受理し、multi-hop reachability を追加しない。

route は distinct ChainMode6-subject/ChainMode5-asserted resolved symbol/site/range を保持し、ordinal 1 を `BindingId(0)` に解決し、expansion 7 個を消費し、known entry 3 個を `BaseModeDef` RHS anchor の `BuiltinSet` identity 1 個へ normalize する。immutable output は `Inferred` variable 1 個、normalized-reflexive `Checked` `TypeAssertion` 1 個、constraint/candidate/fact/diagnostic/deferred reason 0 個を持つ。全 5,039 nonidentity order、宣言済み finite source/provenance/corruption matrix、Task 209 実装前の owner route 34 件すべてとの bidirectional isolation、immutable output、real frontend/resolver sidecar が active runner 157 を保護する。multi-hop ChainMode4 から BaseMode、object sibling、imported-positive/attributed/argument-bearing/parameterized/contextual shape、widening/`qua`、acceptance、truth/fact、proof/CoreIr/ControlFlowIr/VC、general chain semantics は deferred のままとする。checker source または module-layout change は不要であった。

## task 210 exact seven-expansion object-terminal formula-side immediate-radix local-mode asserted-head checker bridge

task 210 は ordered `BaseObjectMode -> object`、`ChainObjectMode1 -> BaseObjectMode` から `ChainObjectMode6 -> ChainObjectMode5`、ChainObjectMode6 reserve 1 個、`LongLocalObjectModeRadixAssertedHeadPayloadBoundary: x is ChainObjectMode5;` だけを対象とする。Task 74 は real AST-derived object-terminal expansion 7 個、Task 200 は formula consumer、Task 179 は builtin-object asserted-type sibling と reusable output guard、Task 209 は matching set-terminal immediate-radix sibling を提供する。変更しない closed `BindingImmediateRadix` relation は real bare `ChainObjectMode6 -> ChainObjectMode5` immediate edge だけを受理し、multi-hop reachability または object/set coercion を追加しない。

route は distinct ChainObjectMode6-subject/ChainObjectMode5-asserted resolved symbol/site/range を保持し、ordinal 1 を `BindingId(0)` に解決し、expansion 7 個を消費し、known entry 3 個を `BaseObjectModeDef` RHS anchor の `BuiltinObject` identity 1 個へ normalize する。immutable output は `Inferred` variable 1 個、normalized-reflexive `Checked` `TypeAssertion` 1 個、constraint/candidate/fact/diagnostic/deferred reason 0 個を持つ。全 5,039 nonidentity order、宣言済み finite source/provenance/corruption matrix、Task 210 実装前の owner route 35 件すべてとの bidirectional isolation、immutable output、real frontend/resolver sidecar が active runner 158 を保護する。multi-hop ChainObjectMode4 から BaseObjectMode、imported-positive/attributed/argument-bearing/parameterized/contextual shape、widening/`qua`、object/set coercion、acceptance、truth/fact、proof/CoreIr/ControlFlowIr/VC、general chain semantics は deferred のままとする。checker source または module-layout change は不要であった。

## task 211 exact two-edge set-terminal formula-side two-hop local-mode asserted-head checker bridge

task 211 は `BaseTwoHopModeAssertedHead -> set`、`MiddleTwoHopModeAssertedHead -> BaseTwoHopModeAssertedHead`、`OuterTwoHopModeAssertedHead -> MiddleTwoHopModeAssertedHead`、Outer reserve 1 個、`TwoEdgeLocalModeTwoHopAssertedHeadPayloadBoundary: x is BaseTwoHopModeAssertedHead;` だけを対象とする。Task 72 は real AST-derived expansion 3 個を供給し、既存 reserved-variable formula bridge は raw Outer subject と独立に resolve された Base asserted type を checker へ渡す。独立した closed `BindingTwoHopRadix` relation は pairwise-distinct symbol を持つ actual bare link 2 本と exact Base-to-set terminal を検証し、generic terminal traversal だけを relation evidence にしない。

route は distinct Outer-subject/Base-asserted resolved symbol/site/range を保持し、ordinal 1 を `BindingId(0)` に解決し、expansion 3 個を消費し、known entry 3 個を `BaseTwoHopModeAssertedHeadDef` RHS anchor の `BuiltinSet` identity 1 個へ normalize する。immutable output は `Inferred` variable 1 個、normalized-reflexive `Checked` `TypeAssertion` 1 個、constraint/candidate/fact/diagnostic/deferred reason 0 個を持つ。全5 nonidentity definition order、finite structural/provenance/corruption matrix、既存 owner 36 件すべてとの bidirectional isolation、immutable output、real frontend/resolver sidecar が active runner 159 を保護する。object sibling、他 distance、imported-positive/attributed/argument-bearing/parameterized/contextual shape、widening/`qua`、general reachability、acceptance、truth/fact、proof/CoreIr/ControlFlowIr/VC、general chain semantics は deferred のままである。checker source/module-layout change は不要であった。

## task 212 exact two-edge object-terminal formula-side two-hop local-mode asserted-head checker bridge

task 212 は `BaseTwoHopObjectModeAssertedHead -> object`、`MiddleTwoHopObjectModeAssertedHead -> BaseTwoHopObjectModeAssertedHead`、`OuterTwoHopObjectModeAssertedHead -> MiddleTwoHopObjectModeAssertedHead`、Outer reserve 1 個、`TwoEdgeLocalObjectModeTwoHopAssertedHeadPayloadBoundary: x is BaseTwoHopObjectModeAssertedHead;` だけを対象とする。Task 72 は real AST-derived object expansion 3 個を供給し、既存 reserved-variable formula bridge は raw Outer subject と独立に resolve された Base asserted type を checker へ渡す。Task 211 の closed `BindingTwoHopRadix` relation は pairwise-distinct symbol を持つ actual bare link 2 本と exact Base-to-object terminal を検証し、generic terminal traversal だけを relation evidence にしない。

route は distinct Outer-subject/Base-asserted resolved symbol/site/range を保持し、ordinal 1 を `BindingId(0)` に解決し、expansion 3 個を消費し、known entry 3 個を `BaseTwoHopObjectModeAssertedHeadDef` RHS anchor の `BuiltinObject` identity 1 個へ normalize する。immutable output は object/set coercion なしで `Inferred` variable 1 個、normalized-reflexive `Checked` `TypeAssertion` 1 個、constraint/candidate/fact/diagnostic/deferred reason 0 個を持つ。全5 nonidentity definition order、finite structural/provenance/corruption matrix、既存 owner 37 件すべてとの bidirectional isolation、immutable output、real frontend/resolver sidecar が active runner 160 を保護する。他 distance、imported-positive/attributed/argument-bearing/parameterized/contextual shape、widening/`qua`、general reachability、object/set coercion、acceptance、truth/fact、proof/CoreIr/ControlFlowIr/VC、general chain semantics は deferred のままである。checker source/module-layout change は不要であった。

## task 213 exact three-edge set-terminal formula-side two-hop local-mode asserted-head checker bridge

task 213 は `BaseThreeEdgeModeTwoHopAssertedHead -> set`、`InnerThreeEdgeModeTwoHopAssertedHead -> BaseThreeEdgeModeTwoHopAssertedHead`、`MiddleThreeEdgeModeTwoHopAssertedHead -> InnerThreeEdgeModeTwoHopAssertedHead`、`OuterThreeEdgeModeTwoHopAssertedHead -> MiddleThreeEdgeModeTwoHopAssertedHead`、Outer reserve 1 個、`ThreeEdgeLocalModeTwoHopAssertedHeadPayloadBoundary: x is InnerThreeEdgeModeTwoHopAssertedHead;` だけを対象とする。Task 73 は real AST-derived expansion 4 個を供給し、Task 195 の reserved-variable formula bridge は raw Outer subject と独立に resolve された Inner asserted type を checker へ渡す。refine した `BindingTwoHopRadix` relation は actual bare Outer-to-Middle/Middle-to-Inner link を pairwise-distinct symbol とともに直接検証し続け、残る Inner-to-Base-to-set tail だけを cycle-safe terminal traversal で検証する。この traversal は relation evidence ではない。

route は distinct Outer-subject/Inner-asserted resolved symbol/site/range を保持し、ordinal 1 を `BindingId(0)` に解決し、expansion 4 個を消費し、known entry 3 個を `BaseThreeEdgeModeTwoHopAssertedHeadDef` RHS anchor の `BuiltinSet` identity 1 個へ normalize する。immutable output は `Inferred` variable 1 個、normalized-reflexive `Checked` `TypeAssertion` 1 個、constraint/candidate/fact/diagnostic/deferred reason 0 個を持つ。全23 nonidentity definition order、finite structural/provenance/corruption matrix、Task 211/212 focused regression、既存 owner 38 件すべてとの bidirectional isolation、immutable output、real frontend/resolver sidecar が active runner 161 を保護する。object sibling、Base/full-distance assertion、deeper/imported-positive/attributed/argument-bearing/parameterized/contextual shape、widening/`qua`、general reachability、acceptance、truth/fact、proof/CoreIr/ControlFlowIr/VC、general chain semantics は deferred のままである。checker source/module-layout change は不要であった。

## task 214 exact three-edge object-terminal formula-side two-hop local-mode asserted-head checker bridge

task 214 は `BaseThreeEdgeObjectModeTwoHopAssertedHead -> object`、`InnerThreeEdgeObjectModeTwoHopAssertedHead -> BaseThreeEdgeObjectModeTwoHopAssertedHead`、`MiddleThreeEdgeObjectModeTwoHopAssertedHead -> InnerThreeEdgeObjectModeTwoHopAssertedHead`、`OuterThreeEdgeObjectModeTwoHopAssertedHead -> MiddleThreeEdgeObjectModeTwoHopAssertedHead`、Outer reserve 1 個、`ThreeEdgeLocalObjectModeTwoHopAssertedHeadPayloadBoundary: x is InnerThreeEdgeObjectModeTwoHopAssertedHead;` だけを対象とする。Task 73 は real AST-derived object expansion 4 個を供給し、Task 196 の formula bridge は raw Outer subject と独立に resolve された Inner asserted type を checker へ渡す。変更しない `BindingTwoHopRadix` relation は pairwise-distinct な Outer-to-Middle/Middle-to-Inner bare link を直接検証し、残る Inner-to-Base-to-object tail は cycle-safe terminal normalization だけで検証して relation evidence にはしない。

route は distinct Outer-subject/Inner-asserted symbol/site/range を保持し、ordinal 1 を `BindingId(0)` に解決し、expansion 4 個を消費し、known entry 3 個を `BaseThreeEdgeObjectModeTwoHopAssertedHeadDef` RHS anchor の `BuiltinObject` identity 1 個へ normalize する。immutable output は object/set coercion なしで `Inferred` variable 1 個、normalized-reflexive `Checked` `TypeAssertion` 1 個、constraint/candidate/fact/diagnostic/deferred reason 0 個を持つ。全23 nonidentity definition order、finite structural/provenance/corruption matrix、Task 211/212/213 focused regression、既存 owner 39 件すべてとの bidirectional isolation、immutable output、real frontend/resolver sidecar が active runner 162 を保護する。Base/full-distance assertion、deeper/imported-positive/attributed/argument-bearing/parameterized/contextual shape、widening/`qua`、general reachability、acceptance、truth/fact、proof/CoreIr/ControlFlowIr/VC、object/set coercion、general chain semantics は deferred のままである。checker source/module-layout change は不要であった。

## task 215 exact four-edge set-terminal formula-side two-hop local-mode asserted-head checker bridge

task 215 は `BaseFourEdgeModeTwoHopAssertedHead -> set`、`InnerFourEdgeModeTwoHopAssertedHead -> BaseFourEdgeModeTwoHopAssertedHead`、`MiddleFourEdgeModeTwoHopAssertedHead -> InnerFourEdgeModeTwoHopAssertedHead`、`OuterFourEdgeModeTwoHopAssertedHead -> MiddleFourEdgeModeTwoHopAssertedHead`、`TooDeepFourEdgeModeTwoHopAssertedHead -> OuterFourEdgeModeTwoHopAssertedHead`、TooDeep reserve 1 個、`FourEdgeLocalModeTwoHopAssertedHeadPayloadBoundary: x is MiddleFourEdgeModeTwoHopAssertedHead;` だけを対象とする。Task 74 は real AST-derived set expansion 5 個を供給し、Task 197 の formula bridge は raw TooDeep subject と独立に resolve された Middle asserted type を checker へ渡す。byte-for-byte 変更しない `BindingTwoHopRadix` relation は pairwise-distinct な TooDeep-to-Outer/Outer-to-Middle bare link を直接検証し、残る Middle-to-Inner-to-Base-to-set tail は cycle-safe terminal normalization だけで検証して relation evidence にはしない。

route は distinct TooDeep-subject/Middle-asserted symbol/site/range を保持し、ordinal 1 を `BindingId(0)` に解決し、expansion 5 個を消費し、known entry 3 個を `BaseFourEdgeModeTwoHopAssertedHeadDef` RHS anchor の `BuiltinSet` identity 1 個へ normalize する。immutable output は object/set coercion なしで `Inferred` variable 1 個、normalized-reflexive `Checked` `TypeAssertion` 1 個、constraint/candidate/fact/diagnostic/deferred reason 0 個を持つ。全119 nonidentity definition order、finite structural/provenance/corruption matrix、Tasks 211-214 focused regression、既存 owner 40 件すべてとの bidirectional isolation、immutable output、real frontend/resolver sidecar が active runner 163 を保護する。test-first fixture 1 件と shared 5 + dedicated 1 backlink により、既存 expectation を変更せず 378 cases、342 requirements、type-elaboration coverage 210/198、pass/fail 194/184 を持つ。object sibling、Inner three-hop/Base full-distance assertion、deeper/imported-positive/attributed/argument-bearing/parameterized/contextual shape、widening/`qua`、general reachability、acceptance、truth/fact、proof/CoreIr/ControlFlowIr/VC、object/set coercion、general chain semantics は deferred のままである。checker source/module-layout change は不要であった。

## task 216 exact four-edge object-terminal formula-side two-hop local-mode asserted-head checker bridge

task 216 は `BaseFourEdgeObjectModeTwoHopAssertedHead -> object`、`InnerFourEdgeObjectModeTwoHopAssertedHead -> BaseFourEdgeObjectModeTwoHopAssertedHead`、`MiddleFourEdgeObjectModeTwoHopAssertedHead -> InnerFourEdgeObjectModeTwoHopAssertedHead`、`OuterFourEdgeObjectModeTwoHopAssertedHead -> MiddleFourEdgeObjectModeTwoHopAssertedHead`、`TooDeepFourEdgeObjectModeTwoHopAssertedHead -> OuterFourEdgeObjectModeTwoHopAssertedHead`、TooDeep reserve 1 個、`FourEdgeLocalObjectModeTwoHopAssertedHeadPayloadBoundary: x is MiddleFourEdgeObjectModeTwoHopAssertedHead;` だけを対象とする。Task 74 は real AST-derived object expansion 5 個を供給し、Task 198 の formula bridge は raw TooDeep subject と独立に resolve された Middle asserted type を checker へ渡す。byte-for-byte 変更しない `BindingTwoHopRadix` relation は pairwise-distinct な TooDeep-to-Outer/Outer-to-Middle bare link を直接検証し、残る Middle-to-Inner-to-Base-to-object tail は cycle-safe terminal normalization だけで検証して relation evidence にはしない。

route は distinct TooDeep-subject/Middle-asserted symbol/site/range を保持し、ordinal 1 を `BindingId(0)` に解決し、expansion 5 個を消費し、known entry 3 個を `BaseFourEdgeObjectModeTwoHopAssertedHeadDef` RHS anchor の `BuiltinObject` identity 1 個へ normalize する。immutable output は object/set coercion なしで `Inferred` variable 1 個、normalized-reflexive `Checked` `TypeAssertion` 1 個、constraint/candidate/fact/diagnostic/deferred reason 0 個を持つ。全119 nonidentity definition order、finite structural/provenance/corruption matrix、Tasks 211-215 focused regression、既存 owner 41 件すべてとの bidirectional isolation、immutable output、real frontend/resolver sidecar が active runner 164 を保護する。test-first fixture 1 件と shared 5 + dedicated 1 backlink により、既存 expectation を変更せず 379 cases、343 requirements、type-elaboration coverage 211/199、pass/fail 195/184 を持つ。Inner three-hop/Base full-distance assertion、deeper/imported-positive/attributed/argument-bearing/parameterized/contextual shape、widening/`qua`、general reachability、acceptance、truth/fact、proof/CoreIr/ControlFlowIr/VC、object/set coercion、general chain semantics は deferred のままである。checker source/module-layout change は不要であった。

## task 217 exact three-edge set-terminal formula-side three-hop local-mode asserted-head checker bridge

task 217 は `BaseThreeEdgeModeThreeHopAssertedHead -> set`、`InnerThreeEdgeModeThreeHopAssertedHead -> BaseThreeEdgeModeThreeHopAssertedHead`、`MiddleThreeEdgeModeThreeHopAssertedHead -> InnerThreeEdgeModeThreeHopAssertedHead`、`OuterThreeEdgeModeThreeHopAssertedHead -> MiddleThreeEdgeModeThreeHopAssertedHead`、Outer reserve 1 個、`ThreeEdgeLocalModeThreeHopAssertedHeadPayloadBoundary: x is BaseThreeEdgeModeThreeHopAssertedHead;` だけを対象とする。Task 73 は real AST-derived set expansion 4 個、Task 195 は real formula/checker consumer を供給する。新しい closed `BindingThreeHopRadix` relation は pairwise-distinct な Outer-to-Middle、Middle-to-Inner、Inner-to-Base bare link を直接検証し、Base-to-set は cycle-safe terminal normalization のみに使い relation evidence にはしない。

route は distinct Outer-subject/Base-asserted symbol/site/range を保持し、ordinal 1 を `BindingId(0)` に解決し、expansion 4 個を消費し、known entry 3 個を `BaseThreeEdgeModeThreeHopAssertedHeadDef` RHS anchor の `BuiltinSet` identity 1 個へ normalize した後、inferred variable 1 個と constraint/fact/candidate/diagnostic/deferred 0 個の checked type assertion 1 個を生成する。全23 nonidentity definition order、finite structural/provenance/corruption matrix、Tasks 211-216 focused regression、先行 owner 42 件との bidirectional isolation、immutable output、real frontend/resolver sidecar が active runner 165 を保護する。test-first fixture 1 件と shared 5 + dedicated 1 backlink により、既存 expectation を変更せず 380 cases、344 requirements、type-elaboration coverage 212/200、pass/fail 196/184 を持つ。object sibling、他 depth、generic reachability/widening/`qua`、declaration/theorem acceptance、truth/fact、proof/CoreIr/ControlFlowIr/VC、general chain semantics は deferred のままである。checker source/module-layout change は不要であった。

## task 218 exact three-edge object-terminal formula-side three-hop local-mode asserted-head checker bridge

task 218 は `BaseThreeEdgeObjectModeThreeHopAssertedHead -> object`、`InnerThreeEdgeObjectModeThreeHopAssertedHead -> BaseThreeEdgeObjectModeThreeHopAssertedHead`、`MiddleThreeEdgeObjectModeThreeHopAssertedHead -> InnerThreeEdgeObjectModeThreeHopAssertedHead`、`OuterThreeEdgeObjectModeThreeHopAssertedHead -> MiddleThreeEdgeObjectModeThreeHopAssertedHead`、Outer reserve 1 個、`ThreeEdgeLocalObjectModeThreeHopAssertedHeadPayloadBoundary: x is BaseThreeEdgeObjectModeThreeHopAssertedHead;` だけを対象とする。Task 73 は real AST-derived object expansion 4 個、Task 196 は real formula/checker consumer を供給する。byte-for-byte 変更しない `BindingThreeHopRadix` relation は pairwise-distinct な Outer-to-Middle、Middle-to-Inner、Inner-to-Base bare link を直接検証し、Base-to-object は cycle-safe terminal normalization のみに使い relation evidence にはしない。

route は distinct Outer-subject/Base-asserted symbol/site/range を保持し、ordinal 1 を `BindingId(0)` に解決し、expansion 4 個を消費し、known entry 3 個を `BaseThreeEdgeObjectModeThreeHopAssertedHeadDef` RHS anchor の `BuiltinObject` identity 1 個へ normalize した後、object/set coercion なしで inferred variable 1 個と constraint/fact/candidate/diagnostic/deferred 0 個の checked type assertion 1 個を生成する。全23 nonidentity definition order、finite structural/provenance/corruption matrix、Tasks 211-217 focused regression、先行 owner 43 件との bidirectional isolation、immutable output、real frontend/resolver sidecar が active runner 166 を保護する。test-first fixture 1 件と shared 5 + dedicated 1 backlink により、既存 expectation を変更せず 381 cases、345 requirements、type-elaboration coverage 213/201、pass/fail 197/184 を持つ。他 depth、generic reachability/widening/`qua`、declaration/theorem acceptance、truth/fact、proof/CoreIr/ControlFlowIr/VC、object/set coercion、general chain semantics は deferred のままである。checker source/module-layout change は不要であった。

## task 219 exact four-edge set-terminal formula-side three-hop local-mode asserted-head checker bridge

task 219 は `BaseFourEdgeModeThreeHopAssertedHead -> set`、`InnerFourEdgeModeThreeHopAssertedHead -> BaseFourEdgeModeThreeHopAssertedHead`、`MiddleFourEdgeModeThreeHopAssertedHead -> InnerFourEdgeModeThreeHopAssertedHead`、`OuterFourEdgeModeThreeHopAssertedHead -> MiddleFourEdgeModeThreeHopAssertedHead`、`TooDeepFourEdgeModeThreeHopAssertedHead -> OuterFourEdgeModeThreeHopAssertedHead`、TooDeep reserve 1 個、`FourEdgeLocalModeThreeHopAssertedHeadPayloadBoundary: x is InnerFourEdgeModeThreeHopAssertedHead;` だけを対象とする。Task 74 は real AST-derived set expansion 5 個、Task 197 は real formula/checker consumer を供給する。byte-for-byte 変更しない `BindingThreeHopRadix` relation は pairwise-distinct な TooDeep-to-Outer、Outer-to-Middle、Middle-to-Inner bare link を直接検証し、Inner-to-Base-to-set tail は cycle-safe terminal normalization のみに使い relation evidence にはしない。

route は distinct TooDeep-subject/Inner-asserted symbol/site/range を保持し、ordinal 1 を `BindingId(0)` に解決し、expansion 5 個を消費し、known entry 3 個を `BaseFourEdgeModeThreeHopAssertedHeadDef` RHS anchor の `BuiltinSet` identity 1 個へ normalize した後、inferred variable 1 個と constraint/fact/candidate/diagnostic/deferred 0 個の checked type assertion 1 個を生成する。全119 nonidentity definition order、unconnected unsupported deeper asserted head と actual connected sixth-definition/sixth-edge asserted head の独立 guard を含む finite structural/provenance/corruption matrix、Task 207 と Tasks 211-218 focused regression、先行 owner 44 件との bidirectional isolation、immutable output、real frontend/resolver sidecar が active runner 167 を保護する。active fixture 1 件と shared 5 + dedicated 1 backlink により、既存 expectation を変更せず 382 cases、346 requirements、type-elaboration coverage 214/202、pass/fail 198/184 を計上する。object sibling、Base full-distance assertion、generic reachability/widening/`qua`、declaration/theorem acceptance、truth/fact、proof/CoreIr/ControlFlowIr/VC、general chain semantics は deferred のままである。checker source/module-layout change は不要であった。

## task 220 exact four-edge object-terminal formula-side three-hop local-mode asserted-head checker bridge

task 220 は `BaseFourEdgeObjectModeThreeHopAssertedHead -> object`、`InnerFourEdgeObjectModeThreeHopAssertedHead -> BaseFourEdgeObjectModeThreeHopAssertedHead`、`MiddleFourEdgeObjectModeThreeHopAssertedHead -> InnerFourEdgeObjectModeThreeHopAssertedHead`、`OuterFourEdgeObjectModeThreeHopAssertedHead -> MiddleFourEdgeObjectModeThreeHopAssertedHead`、`TooDeepFourEdgeObjectModeThreeHopAssertedHead -> OuterFourEdgeObjectModeThreeHopAssertedHead`、TooDeep reserve 1 個、`FourEdgeLocalObjectModeThreeHopAssertedHeadPayloadBoundary: x is InnerFourEdgeObjectModeThreeHopAssertedHead;` だけを対象とする。Task 74 は real AST-derived object expansion 5 個、Task 198 は real formula/checker consumer を供給し、Task 208 は immediate-edge sibling を guard する。byte-for-byte 変更しない `BindingThreeHopRadix` relation は pairwise-distinct な TooDeep-to-Outer、Outer-to-Middle、Middle-to-Inner bare link を直接検証し、Inner-to-Base-to-object tail は cycle-safe terminal normalization のみに使って relation evidence にはしない。

route は distinct TooDeep-subject/Inner-asserted symbol/site/range を保持し、ordinal 1 を `BindingId(0)` に解決し、expansion 5 個を消費し、known entry 3 個を `BaseFourEdgeObjectModeThreeHopAssertedHeadDef` RHS anchor の `BuiltinObject` identity 1 個へ normalize した後、inferred variable 1 個と object/set coercion なしの constraint/fact/candidate/diagnostic/deferred 0 個の checked type assertion 1 個を生成する。全119 nonidentity definition order、unconnected unsupported deeper asserted head と actual connected sixth-definition/sixth-edge asserted head の独立 guard を含む finite structural/provenance/corruption matrix、Tasks 208 と 211-219 focused regression、先行 owner 45 件との bidirectional isolation、immutable output、real frontend/resolver sidecar が active runner 168 を保護する。active fixture 1 件と shared 5 + dedicated 1 backlink により、既存 expectation を変更せず 383 cases、347 requirements、type-elaboration coverage 215/203、pass/fail 199/184 を計上する。Base full-distance assertion、generic reachability/widening/`qua`、declaration/theorem acceptance、truth/fact、proof/CoreIr/ControlFlowIr/VC、object/set coercion、general chain semantics は deferred のままである。checker source/module-layout change は不要であった。

## task 221 exact four-edge set-terminal formula-side four-hop local-mode asserted-head checker bridge

task 221 は `BaseFourEdgeModeFourHopAssertedHead -> set`、`InnerFourEdgeModeFourHopAssertedHead -> BaseFourEdgeModeFourHopAssertedHead`、`MiddleFourEdgeModeFourHopAssertedHead -> InnerFourEdgeModeFourHopAssertedHead`、`OuterFourEdgeModeFourHopAssertedHead -> MiddleFourEdgeModeFourHopAssertedHead`、`TooDeepFourEdgeModeFourHopAssertedHead -> OuterFourEdgeModeFourHopAssertedHead`、TooDeep reserve 1 個、`FourEdgeLocalModeFourHopAssertedHeadPayloadBoundary: x is BaseFourEdgeModeFourHopAssertedHead;` だけを対象とする。Task 74 は real AST-derived set expansion 5 個、Task 197 は real formula/checker consumer を供給する。active closed `BindingFourHopRadix` relation は pairwise-distinct な TooDeep-to-Outer、Outer-to-Middle、Middle-to-Inner、Inner-to-Base bare link を直接検証し、Base-to-set は cycle-safe terminal normalization のみに使って relation evidence にはしない。

active route は distinct TooDeep-subject/Base-asserted symbol/site/range、ordinal 1 / `BindingId(0)`、expansion 5 個、`BaseFourEdgeModeFourHopAssertedHeadDef` RHS anchor の `BuiltinSet` identity 1 個へ normalize する known entry 3 個、inferred variable 1 個、constraint/fact/candidate/diagnostic/deferred 0 個の checked type assertion 1 個を保持する。finite contract は全119 nonidentity definition order、全 definition structural variant、四つの relation link と terminal corruption を含む完全な reserve/formula/head/provenance/removal/corruption matrix、unconnected-deeper と actual connected fifth-link の独立 guard、Task 207 と Tasks 211-220 focused regression、先行 owner 46 件との bidirectional isolation、immutable output、real frontend/resolver sidecar を網羅する。active fixture 1 件と shared 5 + dedicated 1 backlink は、既存 expectation を変更せず 384 cases、348 requirements、type-elaboration coverage 216/204、pass/fail 200/184 内の active runner 169 を計上する。object sibling、longer chain、imported-positive definition、attributed/argument-bearing shape、generic reachability/widening/`qua`、acceptance、truth/fact、proof/CoreIr/ControlFlowIr/VC、general chain semantics は deferred のままとする。checker source/module layout は変更しなかった。

## task 222 exact four-edge object-terminal formula-side four-hop local-mode asserted-head checker bridge

task 222 は `BaseFourEdgeObjectModeFourHopAssertedHead -> object`、`InnerFourEdgeObjectModeFourHopAssertedHead -> BaseFourEdgeObjectModeFourHopAssertedHead`、`MiddleFourEdgeObjectModeFourHopAssertedHead -> InnerFourEdgeObjectModeFourHopAssertedHead`、`OuterFourEdgeObjectModeFourHopAssertedHead -> MiddleFourEdgeObjectModeFourHopAssertedHead`、`TooDeepFourEdgeObjectModeFourHopAssertedHead -> OuterFourEdgeObjectModeFourHopAssertedHead`、TooDeep reserve 1 個、`FourEdgeLocalObjectModeFourHopAssertedHeadPayloadBoundary: x is BaseFourEdgeObjectModeFourHopAssertedHead;` だけを対象とする。Task 74 は real AST-derived object expansion 5 個、Task 198 は real formula/checker consumer を供給する。変更しない closed `BindingFourHopRadix` relation は pairwise-distinct な TooDeep-to-Outer、Outer-to-Middle、Middle-to-Inner、Inner-to-Base bare link を直接検証し、Base-to-object は cycle-safe terminal normalization のみに使って relation evidence にはしない。

active route は distinct TooDeep-subject/Base-asserted symbol/site/range、ordinal 1 / `BindingId(0)`、expansion 5 個、`BaseFourEdgeObjectModeFourHopAssertedHeadDef` RHS anchor の `BuiltinObject` identity 1 個へ normalize する known entry 3 個、inferred variable 1 個、constraint/fact/candidate/diagnostic/deferred 0 個の checked type assertion 1 個を object/set coercion なしで保持する。finite contract は全119 nonidentity definition order、全 definition structural variant、四つの relation link と `BuiltinSet` terminal corruption を含む完全な reserve/formula/head/provenance/removal/corruption matrix、unconnected-deeper と actual connected fifth-link の独立 guard、Task 208 と Tasks 211-221 focused regression、先行 owner 47 件との bidirectional isolation、immutable output、real frontend/resolver sidecar を網羅する。active fixture 1 件と shared 5 + dedicated 1 backlink は、既存 expectation を変更せず 385 cases、349 requirements、type-elaboration coverage 217/205、pass/fail 201/184 内の active runner 170 を計上する。relevant-crate と workspace verification は成功した。longer chain、imported-positive definition、attributed/argument-bearing shape、generic reachability/widening/`qua`、object/set coercion、acceptance、truth/fact、proof/CoreIr/ControlFlowIr/VC、general chain semantics は deferred のままとする。checker source/module-layout change は不要であった。

## task 223 exact transparent parenthesized reserved-variable equality checker bridge

task 223 は `reserve x for set;` と、それに続く `ParenthesizedReservedVariableEqualityPayloadBoundary: (x) = x;` だけを対象とする。Chapter 13 は parenthesized expression が inner expression の型を保持し、FOL encoding を変えないことを要求する。active route は direct `(` / `)` token、nested `TermExpression` 1 個、identifier `TermReference` 1 個を持つ real unrecovered `ParenthesizedTerm` 1 個を消費する。Task 119 は real reserve extraction、`BindingEnv`、builtin-set type projection、equality consumer を供給する。

active route は source payload metadata で独立 wrapper/inner-reference/direct-right site/range を保持し、inner/right `x` use を source-order ordinal 1/2 で `BindingId(0)` へ解決し、inner reference の reserve-derived builtin-set type/value を既存 equality path へ再利用して wrapper を透明に lower する。別個の parenthesis type/axiom/fact/FOL node/synthetic child payload は許可しない。finite contract は direct/right/both/nested/empty/non-identifier/recovered/malformed variant と non-exact reserve/theorem/formula shape を reject し、独立 provenance/lookup/type-input/matched-output/immutable-output corruption を網羅し、real frontend/resolver sidecar とともに先行 reserved-variable binary-formula owner 52 件を双方向に isolate する。active fixture 1 件と shared 4 + dedicated 1 backlink は、既存 expectation を変更せず 386 cases、350 requirements、type-elaboration coverage 218/206、pass/fail 202/184 内の active runner 171 を計上する。focused、relevant-crate、workspace verification は成功した。arbitrary nesting/operand/precedence、formula grouping、implicit-closure materialization、equality truth/fact、theorem acceptance、proof/CoreIr/ControlFlowIr/VC、general child graph、broader term/formula semantics は deferred のままとする。checker source/module-layout change は不要であった。

## task 224 exact seven-expansion set-terminal formula-side two-hop local-mode asserted-head checker bridge

task 224 は ordered bare definition 7 個 `BaseMode -> set`、`ChainMode1 -> BaseMode` から `ChainMode6 -> ChainMode5`、`ChainMode6` reserve 1 個、`LongLocalModeTwoHopAssertedHeadPayloadBoundary: x is ChainMode4` だけを対象とする。Task 74 は real AST-derived expansion、Task 199 は real formula/checker consumer、Task 211 は byte-for-byte unchanged closed `BindingTwoHopRadix` を供給し、Task 209 は immediate-edge sibling guard のみに使う。

active route は distinct `ChainMode6` subject/`ChainMode4` asserted symbol/site/range、ordinal 1 / `BindingId(0)`、exact expansion 7 個を保持し、pairwise-distinct bare `ChainMode6 -> ChainMode5 -> ChainMode4` link を直接検証する。残る tail は cycle-safe terminal normalization のみである。known entry 3 個は BaseModeDef-RHS `BuiltinSet` identity 1 個へ normalize し、inferred variable 1 個と constraint/candidate/fact/diagnostic/deferred 0 個の checked type assertion 1 個を生成する。全5,039 nonidentity order、finite structural/provenance/corruption coverage、先行 owner 48 件、immutable output、focused sibling、real sidecar が route を保護する。focused、relevant-crate、workspace verification は成功した。generic reachability、broader chain、proof/CoreIr/ControlFlowIr/VC、downstream payload は deferred のままとし、checker source/module-layout change は不要であった。

## task 225 exact seven-expansion object-terminal formula-side two-hop local-mode asserted-head checker bridge

task 225 は ordered bare definition 7 個 `BaseObjectMode -> object`、`ChainObjectMode1 -> BaseObjectMode` から `ChainObjectMode6 -> ChainObjectMode5`、`ChainObjectMode6` reserve 1 個、`LongLocalObjectModeTwoHopAssertedHeadPayloadBoundary: x is ChainObjectMode4` だけを対象とする。Task 74 は real AST-derived object expansion、Task 200 は real formula/checker consumer、Task 211 は byte-for-byte unchanged closed `BindingTwoHopRadix` を供給し、Task 210 は immediate-edge sibling、Task 224 は set-terminal two-hop sibling とする。

active route は distinct `ChainObjectMode6` subject/`ChainObjectMode4` asserted symbol/site/range、ordinal 1 / `BindingId(0)`、exact expansion 7 個を保持し、pairwise-distinct bare `ChainObjectMode6 -> ChainObjectMode5 -> ChainObjectMode4` link を直接検証する。残る tail は cycle-safe object-terminal normalization のみである。known entry 3 個は BaseObjectModeDef-RHS `BuiltinObject` identity 1 個へ normalize し、inferred variable 1 個と constraint/candidate/fact/diagnostic/deferred 0 個の checked type assertion 1 個を object/set coercion なしで生成する。全5,039 nonidentity order、finite structural/provenance/corruption coverage、先行 owner 49 件、immutable output、focused sibling、real sidecar が route を保護する。focused、relevant-crate、workspace verification は成功した。generic reachability、broader chain、proof/CoreIr/ControlFlowIr/VC、downstream payload は deferred のままとし、checker source/module-layout change は不要であった。

## task 226 exact seven-expansion set-terminal formula-side three-hop local-mode asserted-head checker bridge

task 226 は ordered bare definition 7 個 `BaseMode -> set`、`ChainMode1 -> BaseMode` から `ChainMode6 -> ChainMode5`、`ChainMode6` reserve 1 個、`LongLocalModeThreeHopAssertedHeadPayloadBoundary: x is ChainMode3` だけを対象とする。Task 74 は real AST-derived set expansion、Task 199 は real formula/checker consumer、Task 217 は byte-for-byte unchanged closed `BindingThreeHopRadix` を供給する。Task 219 は set-terminal three-hop longer-tail sibling、Tasks 209/224 は immediate/two-hop long-chain guard とする。

active route は distinct `ChainMode6` subject/`ChainMode3` asserted symbol/site/range、ordinal 1 / `BindingId(0)`、exact expansion 7 個を保持し、pairwise-distinct bare `ChainMode6 -> ChainMode5 -> ChainMode4 -> ChainMode3` link を直接検証する。残る tail は cycle-safe set-terminal normalization のみである。known entry 3 個は BaseModeDef-RHS `BuiltinSet` identity 1 個へ normalize し、inferred variable 1 個と constraint/candidate/fact/diagnostic/deferred 0 個の checked type assertion 1 個を生成する。全5,039 nonidentity order、finite structural/provenance/corruption coverage、先行 owner 50 件、immutable output、focused sibling、real sidecar が route を保護する。focused、relevant-crate、workspace verification は成功した。generic reachability、object-terminal/broader chain、proof/CoreIr/ControlFlowIr/VC、downstream payload は deferred のままとし、checker source/module-layout change は不要であった。

## task 227 active exact seven-expansion object-terminal formula-side three-hop local-mode asserted-head checker bridge

task 227 は ordered bare definition 7 個 `BaseObjectMode -> object`、`ChainObjectMode1 -> BaseObjectMode` から `ChainObjectMode6 -> ChainObjectMode5`、`ChainObjectMode6` reserve 1 個、`LongLocalObjectModeThreeHopAssertedHeadPayloadBoundary: x is ChainObjectMode3` だけを対象とする。Task 74 は real AST-derived object expansion、Task 200 は real formula/checker consumer、Task 217 は byte-for-byte unchanged closed `BindingThreeHopRadix` を供給する。Task 220 は object-terminal three-hop longer-tail sibling、Task 226 は depth-matched set sibling、Tasks 210/225 は immediate/two-hop object long-chain guard とする。

active route は distinct `ChainObjectMode6` subject/`ChainObjectMode3` asserted symbol/site/range、ordinal 1 / `BindingId(0)`、exact expansion 7 個を保持し、pairwise-distinct bare `ChainObjectMode6 -> ChainObjectMode5 -> ChainObjectMode4 -> ChainObjectMode3` link を直接検証する。残る tail は cycle-safe object-terminal normalization のみに使う。known entry 3 個は BaseObjectModeDef-RHS `BuiltinObject` identity 1 個へ normalize し、inferred variable 1 個と object/set coercion なしの constraint/candidate/fact/diagnostic/deferred 0 個の checked type assertion 1 個を生成する。全5,039 nonidentity order、finite structural/provenance/corruption coverage、先行 owner 51 件、immutable output、focused sibling、real sidecar が route を保護する。focused、relevant-crate、workspace verification は成功した。generic reachability、set-terminal/broader chain、proof/CoreIr/ControlFlowIr/VC、downstream payload は deferred のままとし、checker source/module-layout change は不要であった。

## task 228 exact seven-expansion set-terminal formula-side four-hop local-mode asserted-head checker bridge

task 228 は ordered bare definition 7 個 `BaseMode -> set`、`ChainMode1 -> BaseMode` から `ChainMode6 -> ChainMode5`、`ChainMode6` reserve 1 個、`LongLocalModeFourHopAssertedHeadPayloadBoundary: x is ChainMode2` だけを対象とする。Task 74 は real AST-derived set expansion、Task 199 は real formula/checker consumer、Task 221 は byte-for-byte unchanged closed `BindingFourHopRadix` を供給する。Tasks 224/226 は two/three-hop long-chain guard、Task 222 は object-terminal relation guard、Task 227 は latest terminal guard とする。

active route は distinct `ChainMode6` subject/`ChainMode2` asserted symbol/site/range、ordinal 1 / `BindingId(0)`、exact expansion 7 個を保持し、pairwise-distinct bare `ChainMode6 -> ChainMode5 -> ChainMode4 -> ChainMode3 -> ChainMode2` link を直接検証する。残る tail は cycle-safe set-terminal normalization のみに使う。known entry 3 個は BaseModeDef-RHS `BuiltinSet` identity 1 個へ normalize し、inferred variable 1 個と constraint/candidate/fact/diagnostic/deferred 0 個の checked type assertion 1 個を生成する。全5,039 nonidentity order、finite structural/provenance/corruption coverage、先行 owner 52 件、immutable output、focused sibling、real sidecar が route を保護する。focused、relevant-crate、workspace verification は成功した。generic reachability、object-terminal/broader chain、proof/CoreIr/ControlFlowIr/VC、downstream payload は deferred のままとし、checker source/module-layout change は不要であった。
