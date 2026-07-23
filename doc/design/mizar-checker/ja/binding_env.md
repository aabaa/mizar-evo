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
