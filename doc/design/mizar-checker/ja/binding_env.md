# mizar-checker: Binding Environment

> 正本は英語です。英語版: [../en/binding_env.md](../en/binding_env.md)。

## 目的

`binding_env` は、resolver output と型検査の間で使う checker-owned な binding
および local-context layer を定める。これは次を精緻化する。

- [architecture 04](../../architecture/ja/04.type_and_registration_resolution.md)
  Step 1「Build the Type Checking Context」;
- [architecture 04](../../architecture/ja/04.type_and_registration_resolution.md)
  「Local Type Contexts Are Layered」;
- [architecture 16](../../architecture/ja/16.substitution_and_binding.md)
  の binder identity、alpha-equivalence、capture avoidance、
  definition-time closure;
- [`typed_ast.md`](./typed_ast.md) の `LocalTypeContextTable`。

task 4 は仕様のみの task である。Rust source、実行可能 test、言語意味論、
type inference、registration activation、overload selection、proof behavior は
追加しない。task 5 がここで定義する data structure と context builder を実装する。

## 境界

`binding_env` が所有するもの:

- local variable、binder、reserved variable、local abbreviation、generated
  recovery binding の checker-local binding identity;
- `ResolvedAst` と resolver-owned `SymbolEnv` 上の決定的な layered context
  construction;
- normalized type を付ける前に使う local lookup order と shadowing rule;
- resolver lexical scope から `TypedAst` local context snapshot への bridge;
- task 5 の binding/context diagnostic と deterministic debug rendering。

`binding_env` が所有しないもの:

- parser または resolver の name lookup、label lookup、import/export validation、
  symbol allocation;
- type-expression normalization や attributed type の existence check;
- final type fact、coercion、registration closure、overload root selection、
  inserted view;
- substitution execution、abbreviation expansion replay、VC generation、proof
  search、proof acceptance、kernel replay;
- checker diagnostic code-space が external planning gate のままである間の
  public diagnostic-code allocation。

## 入力と出力

task 5 は、その時点で利用可能な resolver payload から `BindingEnv` を構築する。

- 1 つの source-shaped `ResolvedAst` snapshot;
- その resolver-owned `SymbolEnv`;
- それらの payload が存在する場合の、resolver/source-walk payload が供給する
  explicit local binding record;
- 利用可能な場合の dependency module summary（read-only input）;
- recovery を制御する checker configuration。ただし semantic inference はしない。

現在の resolver surface は `LocalTermScope`、`LocalTermBinding`、
`NameRefEntry::resolution()`、definition shell binder、`SymbolEnv` を公開しているが、
AST 全体の complete local binding declaration table、use-site scope、use-site
ordinal、reserve payload、closure replay 用 captured-free-variable payload は公開していない。
したがって task 5 は、利用可能な payload に対する binding-env data layer、validation、
deterministic rendering、module-level shell を実装しなければならない。欠けている
local source-walk payload または closure payload は、raw syntax から再構築せず、
`external_dependency_gap` diagnostic として記録する。

出力は checker-local snapshot である。

```rust
struct BindingEnv {
    source_id: SourceId,
    module_id: ModuleId,
    contexts: BindingContextTable,
    bindings: BindingTable,
    diagnostics: BindingDiagnosticTable,
}
```

`BindingEnv` は serialized artifact ではない。後続の type-checking task はこれを
消費して `TypedAst::contexts()` を埋め、`BindingTypeRef` entry を付ける。

- global declaration と imported symbol は resolver の `SymbolId` で参照する。
- local typed site は、対応する typed node または role が存在した後にだけ
  `TypedSiteRef` へ写像する。
- fact と assumption は後続の type-checking task が挿入する。binding builder
  自身は挿入しない。

task 5 は direct `mizar-syntax` dependency を追加してはならず、binding construct を
reverse-engineer するために `ResolvedNode::kind()` を inspection してはならない。
binding に必要な source-shape role は resolver-owned projection から到着するか、
external dependency gap として報告されなければならない。

## Context Graph

`BindingContextTable` は module context を root とする決定的な forest である。
各 context は構築後 immutable である。

```rust
struct BindingContext {
    id: BindingContextId,
    owner: BindingContextOwner,
    parent: Option<BindingContextId>,
    layer: BindingContextLayer,
    lexical_scope: Option<LocalTermScope>,
    bindings: Vec<BindingId>,
    visible_bindings: Vec<BindingId>,
    recovery: BindingContextRecovery,
}

enum BindingContextLayer {
    Module,
    Declaration,
    Proof,
    Block,
    Expression,
}
```

layer の意味は architecture 04 に従う。

| Layer | 含むもの | lifetime |
|---|---|---|
| `Module` | imported signature、exported declaration、built-in、top-level reserved variable | module 全体 |
| `Declaration` | declaration parameter、definition-local binder、declaration assumption | 現在の item |
| `Proof` | thesis-local binder、assumption、proof-local declaration、label-related fact | 現在の proof block |
| `Block` | `let`、`given`、`consider`、`reconsider`、statement-local binding、local abbreviation | lexical block または statement frame |
| `Expression` | expected-type / expected-formula mode、coercion context、一時 generated binder | 現在の expression/formula |

必須 invariant:

- context id は等価な resolver input に対して dense かつ deterministic である。
- `context#0` は `BindingContextOwner::Module` を持つ唯一の module root context
  であり、それ以外の context は parent を持たなければならない。
- parent link は acyclic chain を形成する。
- child context は外側の visible binding を読めるが、自分の `bindings` にだけ
  書き込める。
- `visible_bindings` は deterministic `BindingId` で sort する。semantic lookup
  priority は lookup 時に scope depth、visibility ordinal、declaration range から
  計算する。
- context を出るときは、その source construct が escape を許す binding と
  後続 fact だけを freeze する。
- recovered context は明示的であり、欠けている source binder を捏造してはならない。

## Binding Table

`BindingTable` は local checker binding を保存する。resolver symbol は
`SymbolEnv` に残し、source construct が local checker binding を導入する場合を
除き、この table にコピーしない。

```rust
struct BindingEntry {
    id: BindingId,
    spelling: String,
    kind: BindingKind,
    identity: BinderIdentity,
    owner_context: BindingContextId,
    declaration_range: SourceRange,
    visible_after_ordinal: usize,
    type_site: BindingTypeSite,
    status: BindingStatus,
    captured: CapturedFreeVariables,
    diagnostics: Vec<BindingDiagnosticId>,
    recovery: BindingRecoveryState,
}

enum BindingKind {
    QuantifierBinder,
    DefinitionParameter,
    LocalAbbreviation,
    ReservedVariable,
    LetBinding,
    Generated,
}
```

`spelling` は candidate binding を事前に絞り込む source key であり、diagnostic 表示にも使う。
candidate を選択した後の semantic equality、alpha-equivalence、capture check は
`BinderIdentity` を使う。

`type_site` は、後続の type-checking task が binding の型を付ける、または発見する
場所を記録する。resolver syntax または将来の typed site を指してよいが、task 5
はその型を normalize してはならない。

`CapturedFreeVariables` は `set`、`deffunc`、`defpred` closure に使う。task 5 は
resolver が公開している captured id を記録する。resolver がまだ十分な closure
payload を公開していない場合、builder は `external_dependency_gap` diagnostic を
記録し、captured variable を捏造せず abbreviation を non-expandable に保つ。

## Binder Identity

architecture 16 が権威である。source display name だけで equality、
alpha-equivalence、capture を判断してはならない。

```rust
enum BinderIdentity {
    ResolverLocal {
        scope: LocalTermScope,
        ordinal: usize,
        declaration_range: SourceRange,
    },
    DefinitionShell {
        symbol: SymbolId,
        shell: ResolverShellId,
    },
    ReservedVariable {
        spelling: String,
        declaration_range: SourceRange,
    },
    Generated {
        context: BindingContextId,
        counter: u32,
    },
}
```

必須 invariant:

- 同じ display spelling を持つ source variable でも、scope や ordinal が違えば
  identity は異なる。
- `LocalTermScope` は `mizar-resolve` が公開する resolver-owned lexical scope key
  である。task 5 は内部 wrapper を使ってよいが、boundary value は resolver scope
  と visibility ordinal へ trace できなければならない。
- shadowing は新しい binding id を作り、shadowed binding を変更しない。
- generated identity は owning context と counter から決定的に作る。
- alpha-equivalence と capture check は `spelling` ではなく `BinderIdentity` を
  使う。
- resolver identity payload が欠けている場合、textual matching で修復せず
  external dependency gap として報告する。

## Lookup Rules

local lookup は決定的である。

1. active context の `visible_bindings` snapshot だけを探索する。この snapshot は
   builder が選んだ under-approximation boundary であり、lookup は parent を歩いて
   省略された ancestor binding を回収してはならない。
2. その snapshot 内では、まず resolver local-binding key が use-site key と一致する
   binding だけに candidate を絞る。source local term では、resolver scope data が
   公開する use-site spelling を含む。
3. 一致した candidate のうち、`visible_after_ordinal` が use-site ordinal より
   厳密に前の binding だけを考慮する。
4. visible binding を semantic priority で partition する: use-site scope を含む
   最も深い lexical scope、その後最大 visibility ordinal、その後 source range。
5. 同じ spelling の resolver-local candidate が visible だが、その scope を比較するだけの
   lexical payload を use site が持たない場合、別の textual candidate を選択しない。
   抽出済み resolver `NameResolution` が利用可能ならそれを消費し、なければ
   `external_dependency_gap` の missing-payload result を返す。
6. 最良 partition に複数の binding がある場合、`AmbiguousLocalBinding`
   diagnostic draft を持つ degraded ambiguity result を返し、任意に 1 つを
   選ばない。
7. それ以外の場合は、最良 partition にある唯一の binding を選ぶ。
8. local binding が一致せず、use site に resolver `NameRefEntry` がある場合は、その
   entry の `NameResolution` を消費する。
   `BindingLookupSite` は抽出済みの `NameResolution` を保存する。checker は
   resolver-owned な `ReferenceSite` や `ResolvedNodeId` value を構築・永続化しない。
9. `SymbolEnv` は resolver outcome がすでに参照している `SymbolId` を inspect する
   ためだけに使う。checker は symbol index を呼んで global lookup をやり直したり、
   広げたりしてはならない。
10. lexical payload が visible local binding の不一致を判断するのに十分で、resolver
   outcome が渡されていない場合は `Unresolved` を返す。
11. local binding payload も resolver name-reference outcome も利用できない場合は、
   fallback を捏造せず、`external_dependency_gap` diagnostic draft を持つ
   degraded result を返す。

task 5 は lookup を pure に保つ。`BindingEnv::lookup()` は local、resolver、
ambiguous、forward-reference、missing-payload、unresolved の result state を返す。
ambiguity、forward-reference、missing-payload result は diagnostic draft を持つ。
builder または後続 semantic task は、affected site を materialize するときにその
draft を `BindingDiagnosticTable` に記録する。

`BindingId` は semantic lookup priority ではない。ambiguity がすでに reject された
後に限り、deterministic storage、iteration、rendering の tie-breaker として使って
よい。

name/key filter は lookup precondition であり、semantic equality ではない。binding
選択後の equality、alpha-equivalence、capture check は `BinderIdentity` を使う。
display spelling は diagnostic metadata である。

これらの rule は、現在の resolver local-binding ordering の semantic 部分、つまり
scope depth、visibility ordinal、declaration range を意図的に鏡映する。resolver
spelling と stable-id order は deterministic storage order の参考にしてよいが、
semantic ambiguity を黙って解決してはならない。

local binding の forward reference は不正である。binding occurrence は、宣言が
まだ parse/type されている間は自分自身に解決されない。

## Reserved Variables

top-level `reserve` declaration は module context に `ReservedVariable` binding を
導入する。これは declaration ordinal の後でだけ visible になり、後続の同じ spelling
の occurrence に default type site を与える。

reserved-variable rule:

- task 5 は explicit resolver/source-walk payload からだけ reserved binding を記録
  する。現在の `SymbolEnv` は reserve payload を公開していない。
- task 5 validation は non-module context が所有する `ReservedVariable` binding を
  reject する。
- nested `reserve` declaration は、resolver/source support がより狭い legal scope
  を証明するまで recovery case である。
- reserved variable は witness ではなく、それ自体では type fact を作らない。
- 同じ spelling の local binder は、別の `BinderIdentity` によって reserved
  variable を shadow する。
- reserved type expression の normalization は後続の type-checking task が行う。

## Binder And Closure Rules

quantifier、`for`、`ex`、`given`、definition parameter、binder を導入する source
construct は `QuantifierBinder` または `DefinitionParameter` entry を作る。それらの
body context はその binding を含み、後続 substitution work のために body の
free-variable set から取り除く。

local abbreviation（`set`、`deffunc`、`defpred`）は definition-time closure metadata
を持つ `LocalAbbreviation` entry を作る。

- captured free variable は `BinderIdentity` として保存する。
- definition 後の shadowing は closure を変更しない。
- expansion と capture-avoiding substitution は後続 semantic task に延期するが、
  task 5 はそのための identity metadata を保持しなければならない。
- deterministic closure metadata を収集できない場合、abbreviation は degraded
  diagnostic state としてだけ保持する。

`binding_env` は normalized binder path を計算・保存してよいが、substitution replay
を実行してはならない。replay は architecture 16 の pure function のままである。

## Diagnostics And Recovery

`BindingDiagnosticTable` は stable message key を持つ checker-local diagnostic を
記録する。id-order iterator は決定的な insertion order を保つ。`canonical_iter()` は
source range、class、message key、その後 id で sort した diagnostic を rendering と
query に使う。

必須 diagnostic class:

- 同一 lexical scope 内の duplicate local binding;
- visible になる前に使われた local binding;
- unsupported または ambiguous な binding source shape;
- resolver/source-walk integration からの local binding table、use-site
  scope/ordinal、reserve payload、closure payload の欠落;
- resolver identity または closure payload の欠落;
- illegal nested `reserve`;
- malformed source 後の recovered context boundary。

recovery は under-approximate しなければならない。異なる variable を capture したり
後続 proof obligation を unsound にしたりする identity を捏造するより、binding を
省略して diagnostic を出す方を優先する。

## Deterministic Debug Rendering

task 5 は versioned header を持つ deterministic binding-env debug rendering を
提供しなければならない。

```text
binding-env-debug-v1
```

rendering は module id、context graph、binding table、lookup priority key、
diagnostic、external dependency gap を stable order で含める。memory address、
host path、hash-map iteration order、`VcId`、proof witness、verifier status、final
overload information を含めてはならない。

## Public Enum Policy

task 31 は frontend task-25 の public-enum decision procedure をこの module に適用する。
`binding_env` の public checker-owned enum はすべて forward-compatible API surface であり、
`#[non_exhaustive]` を維持しなければならない。downstream consumer は wildcard または
fallback arm を保持する。checker 内部の match は、仕様化済み behavior を実装するために
現在表現されている variant へ exhaustive のままにしてよい。

| enum | decision |
|---|---|
| `BindingContextOwner` | 前方互換; context owner はより豊かな source-to-checker extraction とともに増える可能性がある。 |
| `BindingContextLayer` | 前方互換; context layer category は statement、proof、definition scope とともに増える可能性がある。 |
| `BindingContextRecovery` | 前方互換; context recovery state は partial binding recovery とともに増える可能性がある。 |
| `BindingKind` | 前方互換; binding form はより多くの Mizar declaration extraction とともに増える可能性がある。 |
| `BinderIdentity` | 前方互換; binder identity payload は closure と substitution evidence とともに増える可能性がある。 |
| `BindingTypeSite` | 前方互換; binding type reference は追加の checker-owned anchor を得る可能性がある。 |
| `BindingStatus` | 前方互換; binding status は deferred/external dependency state とともに増える可能性がある。 |
| `BindingRecoveryState` | 前方互換; binding recovery state はより豊かな resolver payload とともに増える可能性がある。 |
| `BindingDiagnosticClass` | 前方互換; diagnostic class は public checker diagnostic code が割り当てられる前に増える可能性がある。 |
| `BindingDiagnosticSeverity` | 前方互換; diagnostic severity policy は IDE/artifact consumer とともに増える可能性がある。 |
| `BindingDiagnosticRecovery` | 前方互換; diagnostic recovery state は partial binding policy とともに増える可能性がある。 |
| `BindingLookupResult` | 前方互換; lookup result は追加の ambiguity と external-gap handling とともに増える可能性がある。 |
| `BindingEnvError` | 前方互換; binding-env construction error は新しい validation case を得る可能性がある。 |

この module が所有する exhaustive public enum exception はない。

## Task 5 の予定テスト

task 5 は Rust test で次を覆わなければならない。

- context、binding、diagnostic、debug text の deterministic dense id;
- module、declaration、proof、block、expression layer creation;
- shadowing を含む nested layer の lookup order;
- global `SymbolEnv` lookup をやり直さず、local lookup から既存の
  `NameRefEntry::resolution()` へ fallback すること;
- `visible_after_ordinal` より前の local forward reference がないこと;
- `reserve` declaration が declaration 後に visible になり、local binder に
  shadow されること;
- binder identity equality が display spelling から独立していること;
- duplicate same-scope binding diagnostic;
- recovered/unsupported binding shape が binding を捏造せず under-approximate
  すること;
- resolver が公開する payload に対する definition-time closure metadata と、
  payload 欠落時の external-gap diagnostic;
- 現在の resolver payload が local binding/use-site/reserve/closure extraction data を欠く
  場合の external-gap diagnostic と deterministic module-shell output;
- public `module_shell(&ResolvedAst, &SymbolEnv)` signature と syntax-free な
  module-match seam;
- deterministic iteration と rendering;
- binding-env data shape が `VcId`、proof witness、verifier status、
  active registration state、final overload root、inserted overload-disambiguating
  `qua` view、resolver-owned な `ReferenceSite` value、resolver-owned な
  `ResolvedNodeId` value を保存しない boundary guard。

task 5 では task-local Rust test が executable scope を cover するため、`.miz`
checker-stage fixture は不要である。最初の active `type_elaboration` corpus runner は
引き続き task 12 が所有する。

## task 4 の分類

| Class | Finding | Action |
|---|---|---|
| `spec_gap` | task-4 spec をブロックするものはない。architecture 04 が layered-context responsibility を与え、architecture 16 が binder/capture authority を与える。 | docs-only spec task として進める。 |
| `test_gap` | task 5 が実装を所有するため、`binding_env` Rust test はまだ存在しない。 | この spec が task-5 必須 test を記録する。task 4 では実行可能 test を追加しない。 |
| `design_drift` | architecture 04 は checker `TypeContext` と呼び、`typed_ast.md` は immutable `LocalTypeContextTable` snapshot を保存する。 | mutable/context-building `BindingEnv` と後続 `TypedAst` snapshot を分け、bridge を定義する。 |
| `source_drift` | `src/binding_env.rs` source はまだ存在しない。 | task 5 前の期待状態。task 4 では source repair しない。 |
| `external_dependency_gap` | 現在の resolver data は `LocalTermScope`、`LocalTermBinding` type、`NameRefEntry::resolution()`、definition shell binder、`SymbolEnv` を公開しているが、AST 全体の complete local binding table、use-site scope/ordinal table、reserve payload、substitution replay 全体に必要な captured-free-variable payload は公開していない。 | task 5 は利用可能な binding-env data layer と module shell を実装してよい。local extraction、reserve payload、closure payload、binder payload の欠落は external dependency gap として記録し、direct `mizar-syntax` dependency を追加したり raw syntax から binding を再構築したりしない。 |
| `deferred` | type normalization、local type fact、registration activation、overload resolution、abbreviation expansion、substitution replay、proof/VC behavior は task 4 の外である。 | task 4 と task 5 は binding/context construction だけに集中する。 |

## task 5 implementation classification

| class | finding | action |
|---|---|---|
| `spec_gap` | data layer、explicit-payload lookup、module shell、diagnostic、deterministic rendering について、task 5 を block する spec gap は残っていない。 | task 5 の review、verification、commit 後に task 6 へ進む。 |
| `test_gap` | task 5 は context layer、lookup priority、forward-reference handling、reserved-variable shadowing、resolver-resolution fallback、closure identity metadata、diagnostic、deterministic ordering、module shell gap、public module-shell signature、boundary guard の Rust unit test を追加する。active `.miz` checker-stage coverage はまだ存在しない。 | Rust test は task 5 の executable scope を覆う。完全に構築された `ResolvedAst` fixture は、resolver が syntax-free fixture を公開するまで checker 外部に残る。active `type_elaboration` corpus coverage は引き続き task 12 が所有する。 |
| `design_drift` | architecture 04 は checker `TypeContext` と呼ぶが、実装は本 task を `BindingEnv` として保ち、後続で `TypedAst::contexts()` へ bridge する。 | task 5 の code drift は残っていない。bridge は type-checking task へ延期する。 |
| `source_drift` | `src/binding_env.rs` が存在し、文書化済み `binding_env` module として公開されている。 | task 5 で解決済み。 |
| `external_dependency_gap` | resolver は引き続き AST 全体の local binding table、use-site scope/ordinal table、reserve payload、captured-free-variable payload、checker-owned test 用 syntax-free empty `ResolvedAst` fixture を公開していない。 | task 5 は module-shell external-gap diagnostic を記録し、利用可能な explicit binding payload を受け取り、direct `mizar-syntax` dependency を追加せず public module-shell signature を type-check する。完全な source extraction と closure replay には、後続 resolver/source-walk integration が不足 payload と fixture を提供する必要がある。 |
| `deferred` | type normalization、local type fact、registration activation、overload resolution、abbreviation expansion、substitution replay、VC generation、proof acceptance、kernel replay は task 5 の外に残る。 | 後続 checker task と downstream crate が扱う。 |

## Task 248 source-context producer integration

Task 248 は syntax ownership を checker へ移さず、最初の bounded real source walk
を供給する。`mizar-test` は reserve shell 1件と definition-block shell 1件を
resolver `DeclarationShellSet` に照合し、opaque shell id、ordered item/binding
record、range、typed site、`LocalTermScope`、`LocalTermBinding` だけを
`SourceBindingContextProducer` へ渡す。producer は module context 1件と declaration
context 1件を構築し、same-spelling reserve/parameter の distinct identity と、visible
reserve への parameter の structural shadow link を保持する。

complete transaction は `SourceBindingContextHandoff` に保持し、
`LocalTypeContextTable` とpairにする。unsupported visibility、stale/reordered
identity/provenance、duplicate/partial row、bindingをclaimするrecovered shellはpublication
前にfailする。bindingを持たないsupported recovered shellはexplicit empty recovered
contextとinternal diagnostic 1件を生成するがincompleteのままで、`TypedAst`へ入れない。
これはexact Task-248 MC-G011/MC-G016 sliceだけをcloseする。term-use lookupと後続
proof/closure contextはTasks 252/257/258/269/270/272が所有し続ける。

## Task 257A source-formula context addendum

Task 257Aはsyntax-free quantified-formula body context 1件のため
`BindingContextOwner::SourceFormula { source_range }`を追加する。
`BindingEnv::try_new`はowner rangeがnonemptyでenvironment sourceに属することを
authenticateする。bounded producerはexact normal `1/0/4` module shellを
`2/1/4`へextendする。context 1はcontext 0のexpression childで、
resolver-shaped `QuantifierBinder` 1件をownしvisible bindingとして公開する。
context ownerはsource provenanceだけを記録し、semantic formula result、
accepted fact、theorem ownerをpublishしない。

## Task 258A reserved-theorem consumer

Task 258AはTask-48由来normal module environment 1件をconsumeする。context
0にactive/visibleな`ReservedVariable` binding 0 (`x`)があり、identifier
declarationはreserve item `0..18`内の`8..9`、written type siteは`14..17`、
theoremでのfirst use ordinalは1。
statement contextはauthenticated visible binding IDとreserved-type-guard
associationだけをcopyし、environment mutation、theorem/proof context追加、
fact publication、本module public API extensionは行わない。

## Task 258B1 source-statement proof contexts

Task 258B1はexact non-exhaustive owner variantを追加する。

```rust
BindingContextOwner::SourceStatement { source_range: SourceRange }
```

`BindingEnv::try_new`は各rangeがnonemptyかつenvironment sourceに属することを
要求する。deterministic debugはexact
`source-statement(<start>..<end>)`としてrenderし、existing owner/pre-B1
environmentのdebug byteは不変。

sole admitted Task-258B1 environmentは`3/1/0`。context 0はunchanged normal
module context、lexical scopeなし、reserved binding 0をownし`[0]`をexpose。
context 1はcontext 0のnormal proof child、owner range `69..137`、lexical
scope `[0]`、bindingをownせず`[0]`をexpose。context 2はcontext 1のnormal
proof child、owner range `86..113`、lexical scope `[0,0]`、bindingをownせず
`[0]`をexpose。binding 0と全Task-48 reserve identity/range/type/visibility
fieldは不変。

variantはsource topologyだけを記録し、proof-local binding、capture、
substitution、fact、diagnostic、goal、proof meaningを追加しない。
Task-258B1 checker matrixはexact owner/range/scope/debug、wrong-source/
empty range、parent/layer/scope/visibility mutation、Tasks 252/258への
fingerprint propagation、rollback、Task-258A byte compatibilityをcoverする。
本documentation prerequisiteはsource/testを変更しない。

### Task 258B1 implementation status

frozen `BindingContextOwner::SourceStatement { source_range }` variantを
実装した。exact outer/nested proof context `69..137` / `86..113`がmodule
baseを`3/1/0`へ拡張し、reserved binding、scope `[0]` / `[0, 0]`、
deterministic debug/fingerprint byteを保持する。empty/foreign rangeやprofile
substitutionはpublication前にfailする。Task-258Aのone-context byteは不変。

### Task 258B2 frozen assumption-context extension

Task 258B2は`source_statement.md`でfreezeしたexact 113-byte source、すなわち
module reserve 1件と、proof内にunlabeled assumption `assume x = x;`、続いて
`thus x = x;`を持つtheorem 1件だけを対象とする。Task-48 environment profileは
exact `2/1/0`。context 0はunchanged module contextでreserved binding 0をownする。
context 1はそのnormal proof childで、owner
`SourceStatement { source_range: 72..111 }`、lexical scope `[0]`、bindingを
ownせず`[0]`をexposeする。nested proof contextはない。

extensionはsource topologyだけを記録する。assumptionをbinding、premise、
fact、checked formula、goal、accepted theorem、proof resultにはしない。
empty/foreign range、non-proof owner、異なるparent/scope、別binding count、
`2/1/0`以外のprofileはpublication前にfailする。本documentation prerequisiteは
source/testを変更せず、既存Task-258A/Task-258B1 profileとbyteも不変。

### Task 258B2 implementation closure

implementationはexisting `SourceStatement` ownerをexactにreuseする。module
context 0とproof context 1 (`72..111`)、parent 0、proof layer、scope `[0]`、
local bindingなし、visible reserved binding `[0]`である。`binding_env.rs`は
本task外でbyte-for-byte不変。mutation testはexact `2/1/0` fingerprintを
authenticateし、cross-profile lower environmentをatomicにrejectする。

## Task 258B3 frozen proof context

witness profileは`binding_env.rs`を変更せずpublic Task-48 modelをreuseする。
module context 0とsource range `69..102`がownするproof context 1をrequireし、
parent 0、proof layer、scope `[0]`、local bindingなし、visible binding
`[0]`、normal recovery。reserved binding 0は`8..9`の`x`、source type site
`14..17`を保持する。

Task-252 termsはcontexts `0,0,1,1,1`を使い、witness rowはdirect
`BindingContextId(1)`をstoreする。theorem/conclusion
`SourceStatementContextId`とはassociationしない。witness validationは
primary term/reference 2を介してbinding 0、scope `[0]`、stored use ordinal
1をreauthenticateする。foreign context/scope/bindingまたはB1/B2 binding
fingerprintはwitness-row validation前にdependency failureとなる。

implemented B3 routeはexact two-context environmentをconstructし、checkerは
witness primary term 2を介してdirect proof contextをrevalidateする。
binding row/binding-environment APIは変更しない。

## Task 258B3N named-witness boundary

B3Nはexactly two contexts、reserved binding 1件、diagnostic 0件を維持する。
proof context 1は`68..105`をcoverし、lexical scope `[0]`、empty owned
binding list、visible binding `[0]`を持つ。named token `y`はnew
witness-name tableでtransportし、`BindingId`ではない。Task 269だけがlater
local binding、RHS link、capture-by-resolved-binding abbreviation replay、
context transitionを保持する。Task 270は`deffunc`/`defpred` closureに
限定したまま。

## Task 258B3N 実装結果

implemented routeはexactly two contextsとsole reserved bindingをrevalidate
する。token `y`は`SourceStatementWitnessNameTable`だけに存在し、新しい
`BindingId`、owned/visible binding、capture、context transitionをpublish
しない。これらのeffectはTask 269が保持する。

## Task 258B3M1 mixed-witness boundary

B3M1はmodule/proof contexts `0/1`、reserved binding 0、no diagnosticを
維持する。両witness primary termsはproof scope `[0]`のbinding 0をresolve
する。token `y`はwitness-name syntaxだけでbinding environmentへ入らず、
second unnamed rowもbindingを作らない。将来の`y` binding、RHS link、
abbreviation replay、context transitionはTask 269が保持する。binding API /
fingerprint grammarは変更しない。

## Task 258B3M1 implementation result

implementationはmodule/proof contexts `0/1`、reserved binding 0、visible
`[0]`、no diagnosticをexactに維持する。token `y`とsecond unnamed witnessは
binding/resolver-owned symbolを作らない。future witness-name bindingと
abbreviation replayは引き続きTask 269が所有する。

## Task 258B3M2A unnamed-numeral boundary

B3M2Aはmodule/proof contexts `0/1`、reserved binding 0、visible scope
`[0]`、no diagnosticをexactに維持する。numeral primary term 2には
reference rowがなく、witness row 0にはname row、owned binding、
resolver symbolがない。binding、abbreviation、capture、context transitionは
作らない。したがってTask 269にB3M2A workはなく、後続witness typing /
existential effectはTask 272が保持する。binding API/fingerprint grammarは
変更しない。

## Task 258B3M2A implementation result

implemented profileはmodule/proof contexts `0/1`、reserved binding 0、
visible `[0]`、diagnostics 0をexactにrevalidateする。numeral term 2と
unnamed witnessはreference、binding、resolver-owned symbol、capture、
abbreviation、context transitionを作らない。したがって本sliceではTask
269はno-opのままで、binding API/fingerprintは変更していない。

## Task 258B3M2B1 frozen binding boundary

exact parenthesized witnessはmodule/proof contexts `0/1`、reserved binding
0、proof scope `[0]`、diagnostics 0をreuseする。outer term 2にreferenceは
なく、child term 3だけがbinding 0をresolveする。references
`0/1/2/3/4`はterms `0/1/3/4/5`、use ordinal 1、scopes
`[]/[]/[0]/[0]/[0]`。unnamed witnessはbinding/capture/abbreviation/
resolver symbol/context transitionを作らず、Task 269はno-op。witness
typing、existential matching/substitution、remaining-goal、proof effectは
Task 272が保持し、binding API/fingerprint grammarを変更しない。

## Task 258B3M2B1 implementation result

implemented profileはmodule/proof contexts `0/1`、reserved binding 0、
visible proof scope `[0]`、diagnostics 0をrevalidateする。outer
parenthesized term 2はreferenceを作らず、child term 3だけがbinding 0を
use ordinal 1でresolveする。unnamed witnessはbinding、resolver symbol、
capture、abbreviation、context transitionを追加しない。Task 269はno-opで、
binding API/fingerprintは不変。

## Task 258B3M2B2A frozen binding boundary

nested-parentheses prerequisiteはmodule/proof contexts `0/1`、reserved
binding 0、proof scope `[0]`、diagnostics 0をreuseする。outer wrapper
term 2とinner wrapper term 3はreferenceを作らず、variable term 4だけが
binding 0をuse ordinal 1でresolveする。unnamed outer witnessはbinding、
capture、abbreviation、symbol、context transitionを追加しない。Task 269は
no-opで、binding API/table/fingerprintは変更しない。

## Task 258B3M2B2A implementation result

implemented profileはfrozen `2/1/0` environmentをbyte-for-byte reuseする。
module context 0、proof context 1 `82..119`、reserved binding 0、proof
scope `[0]`、diagnostic 0。proof-local referenceはleaf term 4だけで、
both wrappersはreference-free、unnamed witnessはbinding/symbolを追加しない。
binding source/public API changeなし。

## Task 258B5A frozen nested-proof binding boundary

private routeはone reserved bindingとexact four normal contexts、
diagnostic 0をreuseする。context 0はmodule。context 1はouter proof
`87..183`、parent 0、lexical scope `[0]`。context 2はfirst descendant
proof `104..131`、parent 1、scope `[0,0]`。context 3はsecond descendant
proof `146..178`、parent 1、scope `[0,1]`。全contextはbinding 0だけを
exposeする。

Task-252 term-reference contextsは`0,0,1,1,2,2,1,1,3,3`で、ten usesは
すべてbinding 0をproducer-stored use ordinal 1でselectする。proof labelは
BindingEnv bindingではなくresolver provenanceである。B5Aはbinding
producer/row/fingerprint/scope rule/diagnostic/source/public APIを変更しない。

## Task 258B5B frozen import-proof binding boundary

imported-citation routeはreserved binding 1件とexact two normal contexts、
diagnostic 0をreuseする。context 0はmodule、context 1はproof
`114..144`、parent 0、lexical scope `[0]`。both contextはbinding 0だけを
exposeする。Task-252 term-reference contextsは`0,0,1,1`で、four usesは
existing producer-stored use ordinalによりbinding 0をselectする。

imported theorem `Ref`はBindingEnv binding/statement factではなくresolver
label provenanceである。separate import-summary prerequisiteとlater upper
implementationはbinding source/row/fingerprint/scope rule/diagnostic/
BindingEnv APIを変更しない。

## Task 258B5C frozen unresolved-label binding boundary

two B5C negativesはchecker binding transportより前のresolver
declaration-symbol handlingでstopする。各raw resolver environmentの
`1/0/1/1/0`はBindingEnv profileではない。R-032Aがvalidated structural
arenaを先に提供し、R-032Bはproof scope `[0]`、`[0,0]`、`[0,1]`をderiveし
one `UnresolvedLabelRef`を返す一方、
`BindingContextId`、`BindingId`、visible binding、statement fact、checker
binding fingerprintをconstructしない。

active runnerはresolver-owned failureをconsumeし、checker経由にrouteする
binding contextをsynthesizeしてはならない。したがってB5CはBindingEnv
source/row/public API/diagnostic/scope rule/testを変更しない。Tasks 252/253と
B1/B5A/B5Bはexactかつdisjointのまま。

R-032B module-global one-based statement counter、completion maximum、
canonical `proof-step-v1` originはresolver label dataで、BindingEnv ordinal/
fingerprintではない。source-byte runner selectionとprivate
`proof_scope_input`/`proof_scope_confinement` detailもBindingEnv consumerを
追加しない。current documentation transactionは48 design filesだけ。

R-032B default-deny edge tableはexact
`Root -> CompilationUnit -> ItemList -> direct TheoremItem -> direct
ProofBlock`から始まり、Root/CompilationUnitは各exact-one normal structural
child、ItemListはdirect normal theorem childだけをscanする。other item
child、formula/token/wrapper、unsupported/
recovered/malformed、qualified/grouped/bulk、template formにordinal/descentを
与えず、いずれもBindingEnv contextを作れない。同様にenv/module、
projection namespace/module/contribution、exact id-0 LocalSource record/
source-id mutationはrunner input authenticationだけである。sole
`proof_scope_input` outputはこのcrateへ入らない。

## Task 248 Two-Parameter Profile-B Binding Boundary

Profile Bはbinding APIを追加しない。existing producerはone normal definition
item/two ordered `DefinitionParameter` inputを受け、definition context 1に
binding ids 0/1をpublishする。module context 0はempty、definition context 1は
parent 0、scope `[0]`、bindings/visible bindings `[0,1]`を持つ。両rowは
ordinals 0/1、exact declaration/type range、empty capture、no shadowのactive
resolver-local identityである。declaration 1のpredecessor 0はsource orderだけを
記録する。

same-scope duplicate、recovery、reserve hybrid、extra item/binding、stale
scope/ordinal/range/site provenanceはfail-closedのまま。Profile Aのreserve/local
shadowとrecovered-empty branchは不変。Task 259はこのhandoffをconsumeするだけで
reconstructできない。

## Task 248P property context binding reuse

Profile Cは`BindingEnv` enum/constructorを追加しない。normal property parameter 1件は
`BindingKind::DefinitionParameter`、resolver-local identity、active status、source
written-type site、empty capture/diagnostic listをreuseし、empty module contextをparentと
するproperty-shell-owned declaration contextに属する。visible/binding listはいずれも
binding zeroだけでshadowはない。recovered zero-binding branchはexisting recovered
context + recovery diagnostic 1件をreuseする。Profile A/B row/error precedenceは
byte-identicalで、Task 264はcontextをconsumeできるが`BindingEnv`内でreconstructしたり
property semanticsを追加できない。

## Task 248P active property context binding

implemented Profile Cはfrozen active definition-parameter binding/declaration context、
またはrecovered empty context + recovery diagnostic 1件だけをpublishする。
`BindingEnv` type/constructor/lookup/semantic payloadは変更せず、Profile A/Bのprior byteと
validation precedenceを保存する。

## Task 269A frozen named-witness binding transition

exact Task-258B3N base environment `2/1/0`はimmutable input。Task 269Aは
context 1を`bindings=[1]`/`visible_bindings=[0,1]`として再構成し、exact
`y` binding 1を`LocalAbbreviation`、resolver-local scope `[0]`、declaration
`81..82`、visible-after 1、missing type site、active、empty capture/diagnostic、
normal recoveryでappendする。context 0/binding 0はbyte-identical、diagnosticは
emptyのまま。

ordinal 1 lookupは`y`を見ず、同scopeのlater lookupはbinding 1を見る。この
transitionはdefinition-site identityだけを記録する。later-use/capture replayは
Task 269B+、witness typing/goal effectはTask 272が保持する。

## Task 269A active named-witness binding transition

implemented producerはexact `2/2/0`を再構成し、construction/installの両方で
ordinal lookup 2件をreplayする。checker/runner corruption matrixはlocal
provenance、row link、全51 node、final fingerprintがfail closedすることを確認する。
cfg(test)-only mutable-row seamはproduction APIを変えずinstalled
spelling/scope/range/ordinal precedenceを検証する。later-use/capture/typing behaviorは
追加しない。

## Task 269B frozen B3M1 transition

same transitionをTask-258B3M1のresolver-local `y`、scope `[0]`、range
`84..85`、visible-after 1にもadmitする。named witness0だけをbinding1にlinkし、
unnamed witness1はbindingを作らない。`2/1/0 -> 2/2/0`、definition-site
forward、later same-scope binding1を保存する。later-use/capture executionと
type/goal/proof effectはdeferする。

## Task 269B active B3M1 transition

implemented transitionはrange `84..85`でexact `2/1/0 -> 2/2/0`を再現する。
context1はbinding1だけをownし`[0,1]`をvisibleにし、binding2は存在しないため
unnamed siblingにbinding effectがない。ordinal1 lookupはforward、ordinal2は
binding1をresolveする。rowは`BindingTypeSite::Missing`を保持してtypeをinferせず、
captureはempty、fact、obligation、proof/goal effectはabsentのまま。

## Task 269CP no-binding lower boundary

isolated proof-`let` prerequisiteはresolver-shaped local
`y@[0],71..72,visible-after=1`をauthenticateするが、`BindingEnv`をmutateせず、
`LetBinding`をallocateせず、type siteやproof/block contextを選ばない。これらは
Task 269C ownerで、private lower projectionをactive bindingと扱うのは
boundary violationである。

## Task 269C frozen `LetBinding` transaction

checkerはTask-269CP syntax-free projectionとexisting reserve bridgeのexact base
`1/1/0`をconsumeする。reserved `x` binding0をvalidateしproof context1とbinding1だけを
appendする。resultはexact `2/2/0`。`y`、`LetBinding`、resolver-local scope `[0]`、
range `71..72`、visible-after1、missing type、active、uncaptured、diagnostic-free、normal。
context1は`SourceStatement(59..98)`、proof layer、parent0、owned `[1]`、visible
`[0,1]`。definition-site ordinal1はforward、synthetic ordinal2はbinding1をresolveする。
actual use/capture row/source typeはclaimせず、全base/final row/debug fingerprintをfail
closedにvalidateする。

## Task 269C active `LetBinding` transaction

implemented producerとinstaller 2段はこのexact transitionとlookup oracle 2件を
enforceする。environment/context/binding/declaration link/fingerprintのcorruptionは
transactional failし、final bindingはmissing typeのままでreal use/captureまたは
semantic effectを持たない。

## Task 269CT typed binding overlay

Task 269Cはimmutable missing-type dependencyのまま。269CTはseparate exact `2/2/0`
typed overlayを作り、binding 0はSource `14..17`、proof-local binding 1だけSource
`76..79`となる。context/identity/lookup/capture/diagnostic/non-type fieldは不変。
overlayはnew compositeからだけ参照し、use/capture、guard、fact、goal、proof、obligationを
publishしない。

## Task 269CT implemented overlay

implementationはdependency environmentをsorting/repairなしでexact reconstructする。context、
binding 2件、identity、lookup field、capture、diagnostic、recoveryは、binding 1のtype site
`Missing -> Source(76..79)`だけを除きTask 269Cとequal。validation/testはcomplete
`2/2/0` payloadをauthenticateし、binding corruptionをsource-type/availabilityより先にfail。

## Task 269GP no-binding lower boundary

private lower outputは`y` token spelling/rangeだけをcarryし、resolver-shaped local
identity、BindingEnv row、lookupを作らない。Chapter-4/16 `given` scope矛盾はhuman
canonical reconciliationまで269G/269GTをblockする。

implemented 269GPもこのboundaryを保持し、focused testsは全binding-shaped publicationを
rejectする。existing binding environment APIは不変。

## Task 269GS resolved scope input

canonical scopeはlater binding consumerに十分となった。`given` bindingはdeclarationの
`such that` conditionをcoverし、後続visibilityは最内のenclosing proof/reasoning blockと
ともに終了し、shadowされない限りnested childへ継承され、parent/sibling blockには存在しない。
Task269GSは`BindingEnv`を変更しない。
Task269Gがcondition/proof factを追加せず、exact scope ID、ordinal、lookup/replay、restore、
testsを別途freezeする。

## Task 269G frozen `GivenWitness` environment

`BindingKind::GivenWitness`を`LetBinding`直後/`Generated`直前、stable debug key
`given_witness`で追加し、exact proof-context bindingをcontext/binding `1/1`、scope
`[0]`、source/visible-after `1/1`、missing typeで追加。forward-before、same-condition/
later/child可視、parent/sibling exclusion、child shadow、outer restoreをlookup testする。
handoffにはreal context 0/1だけ。test-derived context 2/3/4はowner key
`task269g-unshadowed-child` / `task269g-shadow-child` / `task269g-sibling`のnormal `Block`、
binding 2はscope `[0,1]`、ordinal/range `2/109..110`、owner 3、missing type、active、empty
capture/diagnosticsのtest-only `y`/`GivenWitness` shadow。condition/fact/capture/type/proof rowなし。
missing source typeはTask269GT。

## Task 269G active `GivenWitness` transaction

producer/installer 2件はfrozen transitionとexact declaration-context forward/local lookupを
実装する。separate checker-only synthetic matrixが、witnessは対応block/nested childで
shadowされない限りvisible、shadow終了後restore、parent/siblingではabsentと証明するが、
synthetic contextはproduction handoff rowではない。environment/declaration/binding/fingerprint
corruptionはtransactional failure。installed rowはmissing typeを保持しcondition/fact/capture/
obligation/proof effectを作らない。

## Task 269GT frozen type overlay

new compositeはTask269G environmentをbinding 1の`Missing -> Source(84..87)`以外
byte-for-byte preserve。binding 0は`Source(14..17)`、context/identity/lookup/scope/status/
capture/diagnostic/cardinality `2/2/0`不変。immutable dependencyをmutateせずtyped snapshotをown。

### Task 269GT implemented overlay

immutable Task269G environmentはGiven rowのtype missingを含む`2/2/0`のまま。Task269GTはsort/repairせずcopyし、binding 1だけを`Source(84..87)`へ変更する。binding 0、両context、resolver identity、block-local inheritance/shadowing/restoration、capture、diagnostic、全non-type fieldはexactに不変。

## Task 269GUP new-source binding profile

sibling transactionは独立に`1/1/0 -> 2/2/0`をbuild。context 1 owner `62..126`、binding 1は
`GivenWitness`、scope `[0]`、ordinal 1、`76..77`、type `Missing`。ordinal 2はこのenv自身の
`BindingId(1)`をselect。capture/diagnostic empty、parent/sibling exclusionとchild inheritance/
shadow restorationをtest。
### Task 269GUP binding profile 実装状況

凍結済みの6ファイル transactionとchecker/runner各4件の正確なtestを実装した。libraryは`502/564`、checker/runner productionは`30/172531`と`37/74826`で、path hashは不変、content hashは`e0342952a01a0b379cf7b06ad243cd40a1656e940480196323cf43fbe7d8f7c5` / `8fe7c8c0b7e855e5113f3830873e133f42c8048a3272055e2fddd5ebd9cbb1bc`である。

閉じるのはdormant private lexical-binding evidenceだけで、active corpus、trace、type、term/use、condition/fact、goal/proof、obligation、diagnostic、CLIのcreditは0のままである。次はTask 269GUPTであり、Task 269GU、capture、Task 270は引き続きdeferする。

## Task 269GUPT frozen binding overlay boundary

immutable GUP handoffのexact `2/2/0` envをcopyし、binding 1だけを`Missing -> Source(84..87)`へ変更、binding 0は`Source(14..17)`を保持する。context/identity/lookup lifetime/capture/diagnostic/全non-type fieldは不変で、dependencyとold G/GT envをmutateしない。occurrence/guard/fact/capture/semantic binding stateは追加しない。

### Task 269GUPT implemented overlay

exact copied overlayとcorruption/replay/ownership/same-identity cross-family
matrixを実装しpass。immutable GUP dependencyと全non-type binding stateは不変で、
semantic binding creditなし。

## Task 269GU lookup consumer凍結

GUはimmutable GUPT `2/2/0` envをreuseする。later `y` 2 rowはcontext 1、scope
`[0]`、derived use ordinal 2で`GivenWitness` `BindingId(1)`へresolve必須。private
exact source-term profileだけがこれを`Variable`としてadmitする。generic admission、
context/capture/diagnostic/type/scope lifetime、GUP/GUPT byteは不変で、parent/
sibling visibilityはexcludeする。

### Task 269GU implemented binding use

term producerはimmutable GUPT environmentをconsumeし、later reference 2件を
binding 1/use ordinal 2へauthenticate。binding row/contextは追加せず、condition/
descendant occurrence、capture/exportはまだtransportしない。

## Task 269GCP no-binding lower boundary

exact condition sourceはfuture binding consumerの必要性だけを示す。GCPはcontext、
`BindingId`、lookup、captured identity、diagnosticを作らずGUP/GUPT/GU byteを不変に
する。Task269GCがgeneric source-order lookupを緩めずcondition occurrenceから
見えるdistinct exact environmentを構築する。

### Task 269GCP implemented no-binding boundary

implemented lower rowはwitness declaration siteを保持するが、binding context/
ID/lookup/lifetime/diagnosticを作らない。GUP/GUPT/GUはbyte-identical。user-confirmed
innermost-block lifetimeはnext distinct Task269GC binding handoffのownerである。

## Task 269GC frozen binding environment

GCはunchanged `GivenWitness`/common rowをreuseし`binding_env.rs`を変更しない。
reserve base `1/1/0 -> 2/2/0`、`SourceStatement(68..132)` proof context scope
`[0]`、normal active missing-type `y@82..83` 1件だけ。own condition、subsequent、
child inheritance、shadow/restore、parent/sibling exclusionをfreezeし、occurrence/
fact/condition lifetime/capture/diagnostic/type rowは作らない。

### Task 269GC implemented binding environment

exact `1/1/0 -> 2/2/0` transactionとcomplete lexical lookup matrixをfrozen
checker ownerへ実装し、`binding_env.rs`は不変。checker/runner各4 testsがown-
condition、subsequent、inheritance、shadow/restore、parent/sibling boundaryを
cover。type/occurrence/fact/capture/diagnostic/semantic creditはfrozenどおりdefer。

## Task 269GCT frozen type overlay

validated GC `2/2/0` envをcloneしbinding 1のtype siteだけ`Missing ->
Source(90..93)`、binding 0は`Source(14..17)`。context/visible order/lookup、
identity/kind/status/range/ordinal/scope/capture/recovery/empty diagnosticは不変で
binding/context追加なし。`binding_env.rs`変更、fact/condition/guard/capture/
obligation semanticsなし。

### Task 269GCT implementation status

documentation prerequisite `b43081161b31fcc4bc23ac2fd42c5c42e772ab78`後、
exact 7-file implementationとchecker 4件/private runner 4件のtestを実装した。
new public checker familyは
`SourceProofLocalGivenConditionType{Handoff,Producer,Error}`で、Typed/Resolvedは
same boxed compositeをatomicにownする。libraryは`518/584`。checker productionは
`30/179612`、unchanged path hash
`c89f43f6abebf7ebeb3ac9394ecd8ea3186ad28934c75526d2cc0b85a66ebad5`、
content hash
`8078ee6235c8ca52ce8cdba0be9a347231260d3421c54625a3fc96cf395c9718`。
runner productionは`37/77159`、unchanged path hash
`1f9e2c9c6589412d832eb92015d913c1b2e0f1309cba9c5c991e08b04d67a73d`、
content hash
`5b0e68f35d37fcf843f7cb64885f09bfa9dd5423c17506713e096811a5ddf689`。
raw/normalized test-list hashはchecker
`6d10b524115a209f198bc5085a726bc1fcc6f92dc3e25a8056e29975b708b656` /
`502f7535a34b9d2224c67e6db15f4eaf45f05eec2a2fe4c914704ecf162d89b2`、
runner
`d599bd69654d000f44858942cec771742d8c3c9e0d2ca459d7fecc84d76752c9` /
`bc3cdabbc6424b0f01d817ed323dd823ff57d1d8d4261220dc3d9c37d9004a61`。

canonical spec、`.miz`、fixture/sidecar/expectation、trace row/status/backlink、
metadata/diagnostic/public dispatch/CLI byte/active result/semantic creditは
変更しない。condition occurrence 2件とwider semantic effectはGCU ownerのまま。
test sufficiency、implementation、source/docs、final-qualityのindependent
reviewは**NO FINDINGS**。全9 hard gatesはscore capなしの`100/100`でPASSし、
focused/crate suite、lint policy、format、Clippy、workspace test、metadata、
全5 CLI、count/hash oracle、diff checkもPASS。dedicated implementation
commit `d6fb0ed28ced4d4706a1793b3aedd2a20eea0749`を完了。

## Task 269GCU frozen reference lookup

GCT `2/2/0` envはmutateしない。両termはcontext1/use ordinal2でbinding1
`GivenWitness`、scope `[0]`、declaration `82..83`、type
`Source(90..93)`へunique lookupする。GCU private profileだけがこのVariable
referenceをadmitし、context/binding/scope/capture/diagnostic/fingerprintは不変。

### Task 269GCU implementation status

documentation prerequisite `15f47a837bc2f52d4cd30e8a4dcb86c16f2961d3`
の後、frozen implementation 7 files、`cfg(test)`-only predecessor
ownership-sentinel support 1 file、checker/private runner各4 testが存在する。
support seamはreviewで判明したTask-269A both-order `test_gap`だけを閉じ、
production API/behaviorを変更しない。public familyは
`SourceProofLocalGivenConditionUseTerm{Handoff,Producer,Error}`であり、Typedと
Resolvedは同じboxed compositeをatomicに所有する。libraryは`522/588`。
checker productionは`30/181154`、unchanged path hash
`c89f43f6abebf7ebeb3ac9394ecd8ea3186ad28934c75526d2cc0b85a66ebad5`、
content hash
`f9901821c2242bfe66321c57982b54b78425c7940c5a7c47c93c43a8c2c035dc`。
runner productionは`37/77435`、unchanged path hash
`1f9e2c9c6589412d832eb92015d913c1b2e0f1309cba9c5c991e08b04d67a73d`、
content hash
`0651af8339c147d04f88be237f8f49fc716b7da3ff90238be50a9527e89992b7`。
raw/normalized test-list hashはchecker
`d453ca1e8a7cf9870f14a0f933451ca201c19cc8c8367d51767c40a941766f82` /
`7cd84f6cd8e6d1070b39be9e5f1031512cc2c1b664829f10d337f1b67bcb74b3`、
runner
`7a99bcbb35838b6c1df31dec7b7c70d9c569df86bdc6f5c68d72f41578be2a9e` /
`e49dac17564f330ad5c73018538bf5736720e47f4833709c1b9d36622208888a`
である。

implementationが閉じるのはfrozen own-condition内の2つの`y` term/reference
occurrenceだけである。authoritative block-scope decisionにより`given`
bindingはinnermost blockの残余とdescendant blockでinner shadowingを除き有効
だが、descendant use/capture implementationは別successorに残る。canonical
specification、`.miz`、fixture、sidecar、expectation、trace row/status/
backlink、metadata、diagnostic、public dispatch、CLI byte、active result、
semantic creditは変更しない。equality/formula/fact、guard、goal、proof/
obligation/acceptance、export/capture enforcement、downstream IR、Task 270は
deferredのままである。test-sufficiency、implementation、source/docsの
independent reviewは**NO FINDINGS**。final read-only qualityも**NO FINDINGS**、
全9 hard gatesはscore capなしの`100/100`でPASS。focused/full measured
gateもPASSし、exact stagingとimplementation commit f984ae683419944493c07723e9950a9101a46502 が完了した。

## Task 269SDP BindingEnv deferral

SDPは`BindingEnv`をinstallせず、spellingからwinnerを作らない。Given row、
descendant context、inherited `y`、LocalAbbreviation `z/q`は全て後続。
captureはcanonical `set` reconciliationまでblockedである。
