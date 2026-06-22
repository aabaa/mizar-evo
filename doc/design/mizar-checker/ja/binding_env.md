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

task 5 は次から `BindingEnv` を構築する。

- 1 つの source-shaped `ResolvedAst` snapshot;
- その resolver-owned `SymbolEnv`;
- 利用可能な場合の dependency module summary（read-only input）;
- recovery を制御する checker configuration。ただし semantic inference はしない。

出力は checker-local snapshot である。

```rust
struct BindingEnv {
    module: ModuleId,
    contexts: BindingContextTable,
    bindings: BindingTable,
    diagnostics: BindingEnvDiagnosticTable,
}
```

`BindingEnv` は serialized artifact ではない。後続の type-checking task はこれを
消費して `TypedAst::contexts()` を埋め、`BindingTypeRef` entry を付ける。

- global declaration と imported symbol は resolver の `SymbolId` で参照する。
- local typed site は、対応する typed node または role が存在した後にだけ
  `TypedSiteRef` へ写像する。
- fact と assumption は後続の type-checking task が挿入する。binding builder
  自身は挿入しない。

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
- parent link は acyclic chain を形成する。
- child context は外側の visible binding を読めるが、自分の `bindings` にだけ
  書き込める。
- `visible_bindings` は binding lookup priority、その後 `BindingId` で sort する。
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
    spelling: Option<String>,
    identity: BinderIdentity,
    kind: BindingKind,
    owner_context: BindingContextId,
    declaration_range: SourceRange,
    visible_after_ordinal: usize,
    type_site: Option<BindingTypeSite>,
    capture: CapturedFreeVariables,
    status: BindingStatus,
}

enum BindingKind {
    Value,
    ReservedVariable,
    QuantifierBinder,
    DefinitionParameter,
    LocalAbbreviation,
    GeneratedRecovery,
}
```

`spelling` は diagnostic 専用である。semantic equality と lookup identity は
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

1. active context とその ancestor を内側から外側へ探索する。
2. context 内では、まず resolver local-binding key が use-site key と一致する
   binding だけに candidate を絞る。source local term では、resolver scope data が
   公開する use-site spelling を含む。
3. 一致した candidate のうち、`visible_after_ordinal` が use-site ordinal より
   厳密に前の binding だけを考慮する。
4. visible binding を semantic priority で partition する: use-site scope を含む
   最も深い lexical scope、その後最大 visibility ordinal、その後 source range。
5. 最良 partition に複数の binding がある場合、degraded diagnostic を出し、任意に
   1 つを選ばない。
6. それ以外の場合は、最良 partition にある唯一の binding を選ぶ。
7. local binding が一致しない場合だけ、resolver-owned `SymbolEnv` candidate set
   に fallback する。checker は global name lookup をやり直してはならない。

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

`BindingEnvDiagnosticTable` は stable message key を持つ checker-local diagnostic を
記録する。diagnostic は source range、class、message key、その後 id で sort する。

必須 diagnostic class:

- 同一 lexical scope 内の duplicate local binding;
- visible になる前に使われた local binding;
- unsupported または ambiguous な binding source shape;
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

## Task 5 の予定テスト

task 5 は Rust test で次を覆わなければならない。

- context、binding、diagnostic、debug text の deterministic dense id;
- module、declaration、proof、block、expression layer creation;
- shadowing を含む nested layer の lookup order;
- `visible_after_ordinal` より前の local forward reference がないこと;
- `reserve` declaration が declaration 後に visible になり、local binder に
  shadow されること;
- binder identity equality が display spelling から独立していること;
- duplicate same-scope binding diagnostic;
- recovered/unsupported binding shape が binding を捏造せず under-approximate
  すること;
- resolver が公開する payload に対する definition-time closure metadata と、
  payload 欠落時の external-gap diagnostic;
- deterministic iteration と rendering;
- binding-env data shape が `VcId`、proof witness、verifier status、
  active registration state、final overload root、inserted overload-disambiguating
  `qua` view を保存しない boundary guard。

task 4 は documentation-only なので `.miz` checker-stage fixture は不要である。最初の
active `type_elaboration` corpus runner は引き続き task 12 が所有する。

## task 4 の分類

| Class | Finding | Action |
|---|---|---|
| `spec_gap` | task-4 spec をブロックするものはない。architecture 04 が layered-context responsibility を与え、architecture 16 が binder/capture authority を与える。 | docs-only spec task として進める。 |
| `test_gap` | task 5 が実装を所有するため、`binding_env` Rust test はまだ存在しない。 | この spec が task-5 必須 test を記録する。task 4 では実行可能 test を追加しない。 |
| `design_drift` | architecture 04 は checker `TypeContext` と呼び、`typed_ast.md` は immutable `LocalTypeContextTable` snapshot を保存する。 | mutable/context-building `BindingEnv` と後続 `TypedAst` snapshot を分け、bridge を定義する。 |
| `source_drift` | `src/binding_env.rs` source はまだ存在しない。 | task 5 前の期待状態。task 4 では source repair しない。 |
| `external_dependency_gap` | 現在の resolver data は `LocalTermScope`、visibility ordinal、definition shell binder、`SymbolEnv` を公開しているが、substitution replay 全体に必要な richer stable binder id と captured-free-variable payload は未完の可能性がある。 | task 5 は利用可能な context skeleton を実装してよいが、closure または binder payload 欠落は external dependency gap として記録する。 |
| `deferred` | type normalization、local type fact、registration activation、overload resolution、abbreviation expansion、substitution replay、proof/VC behavior は task 4 の外である。 | task 4 と task 5 は binding/context construction だけに集中する。 |
