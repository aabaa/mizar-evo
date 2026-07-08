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
上書きする。task-83 bridge 外の broader imported structure は、non-`R` の
source-derived fixture が存在するまで deferred のままとする。bridge はこの import
summary を real imported module AST extraction と扱ってはならず、base-shape /
constructor-witness evidence、positive structure elaboration、
CoreIr/ControlFlowIr/VC/proof payload を捏造してはならない。task 83 は imported
structure provenance bridge を記録する。documented `parser.type_fixtures` import
summary 由来の `R` は imported structure type head として declaration checking
に到達し、base-shape / constructor-witness evidence がまだ存在しないため
`type_elaboration.checker.checker.declaration.deferred.evidence_query` で fail
closed する。これは positive imported structure elaboration や imported module
AST extraction を credit しない。
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
`AttributeInput` として渡すことで同じ boundary を上書きする。task-84 / task-85
bridge 外の broader imported attribute は、source-derived fixture と payload producer
が存在するまで deferred のままとする。active runner sidecar は positive
`empty set` と builtin `object` 上の imported `empty` を external
extraction-gap boundary に固定する。bridge はこの import summary を real
imported module AST extraction と扱ってはならず、attributed-type evidence、positive
attributed type elaboration、positive `empty set`、non-`set` head 上の imported
`empty`、CoreIr/ControlFlowIr/VC/proof payload を捏造してはならない。
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
上書きする。positive `empty set` と `object` 上の imported attribute は active
external-gap boundary fixture のままである。positive `empty set`、`object` や
local symbol head 上の imported attribute、imported module AST extraction、attributed-type evidence、positive
imported attributed type elaboration、structure-qualified owner provenance、
attribute argument、downstream payload は credit しない。
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
source は active type-elaboration runner まで到達できるが、checker-owned
theorem/formula payload extraction、local proof context、recorded fact、dedicated
`formula_statement` runner が存在するまでは
`type_elaboration.external_dependency.ast_payload_extraction` に残す。この boundary は
theorem acceptance、formula fact、proof skeleton、CoreIr、ControlFlowIr、VC、
proof payload を credit しない。
task 87 は term を含む theorem formula について同じ boundary を記録する:
`theorem TermFormulaPayloadBoundary: 1 = 1;` は Chapter 13 の numeral term と
Chapter 14 の builtin equality surface を伴って parser / resolver 実行まで到達するが、
real term/formula payload extraction、term inference、formula checking、recorded fact、
theorem acceptance、dedicated `formula_statement` runner、CoreIr、ControlFlowIr、VC、
proof payload がまだ存在しないため
`type_elaboration.external_dependency.ast_payload_extraction` に残す。
task 88 は対応する proof-block boundary を記録する:
`theorem ProofSkeletonPayloadBoundary: thesis proof thus thesis; end;` のような
theorem は Chapter 16 の proof block と Chapter 15 の conclusion statement を伴って
parser / resolver 実行まで到達するが、real proof skeleton payload extraction、
local proof context、formula payload extraction、recorded fact、theorem acceptance、
dedicated `formula_statement` runner、CoreIr、ControlFlowIr、VC、proof payload が
まだ存在しないため `type_elaboration.external_dependency.ast_payload_extraction` に残す。
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
- set enumeration / set comprehension は set-like type を生成し、spec chapter 13 が要求する
  場合 generator domain の deferred sethood requirement を記録する;
- `the T` は choice-like typed term と `T` の deferred non-emptiness requirement を記録するが、
  proof-owned id は割り当てない;
- source-written `qua` は後続 checking が必要とする source view と deferred source-`qua`
  requirement を記録する。`SourceQua` coercion candidate を作るのは task 10 である。

formula rules:

- predicate application は candidate argument type を check するが、final root selection が
  ambiguous な場合は phase 8 のため candidate group を保持する;
- built-in `=`、`<>`、`in` form は term well-formedness を check し、appropriate
  expected-type constraint を追加する;
- type / attribute assertion は subject term が normalized asserted type または
  attribute chain に admissible かを check する;
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
