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
- sethood と narrowing claim の checker-local `InitialObligation`;
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
    args: Vec<TypeArgumentRef>,
    attributes: AttributeSet,
    source: TypeSource,
    status: NormalizedTypeStatus,
}

enum TypeHeadRef {
    BuiltinObject,
    BuiltinSet,
    Mode(SymbolId),
    Structure(SymbolId),
    Error(TypeDiagnosticId),
}

struct AttributeSet {
    positive: Vec<AttributeInstance>,
    negative: Vec<AttributeInstance>,
}

struct AttributeInstance {
    symbol: SymbolId,
    args: Vec<TypeArgumentRef>,
    source_range: SourceRange,
}
```

canonical type key は head kind、canonical `SymbolId`、normalized argument key、
attribute key の順で並ぶ。source spelling と range は diagnostic のために保持するが、
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

- `ResolvedAst` から得る resolved type-expression site;
- `SymbolEnv` が公開する type-head、attribute、mode、structure、parameter signature;
- `BindingEnv` と先行 normalized parameter site から得る local binding/type context。

出力:

- deterministic key order の `NormalizedType` entry;
- type-expression site の `TypeEntry`;
- unknown head、wrong arity、wrong type-argument kind、contradictory
  attribute、unsupported syntax payload、missing external resolver payload の
  diagnostic。

規則:

1. type head は resolver/symbol payload だけで resolve する。checker は name lookup を
   やり直したり raw syntax node kind を inspect してはならない。
2. type argument を左から右へ正規化し、その canonical type key を parent type で使う。
3. `non A` を negative set、`A` を positive set に保存して attribute polarity を正規化する。
4. attribute を canonical symbol id、normalized arguments、polarity、source range の順で
   sort する。この順序は storage/rendering 用だけである。
5. resolver が defining radix と attribute payload を公開する場合だけ mode definition を
   unfold する。payload がない場合は `external_dependency_gap` を記録し、mode symbol
   headed の degraded type を保持する。
6. すべての diagnostic と debug rendering のために source spelling と source range を保持する。
7. task 7 では cluster closure を使って normalized type を「修復」しない。cluster closure は
   phase 7 であり、後続 registration task が所有する。

## Task 8: declaration and local-binding checking

task 8 がこの section を実装する。

declaration checking は binding に normalized type を付け、対応する
`LocalTypeContext` snapshot へ local fact を導入する。

必須 behavior:

- `let`、definition parameter、quantified variable、`given`、`consider`、`take`
  binder は `BindingId` と `TypedSiteRef` に link された `TypeEntry` を受け取る;
- reserved variable は、その occurrence が local binding に shadow されない場合だけ
  default type site を提供する;
- `set` declaration は right-hand side type を infer し、`BindingEnv` の definition-time
  closure metadata を持つ abbreviation binding を保存する;
- `deffunc` と `defpred` の formal は local definition parameter として check し、
  body は definition-time context の下で check する;
- `reconsider x as T` は current site で existing binding の type view を更新し、
  `reconsider y = t as T` は新しい local binding を導入する;
- attributed / constrained type を持つ declaration は、必要な evidence が known fact
  として既にない場合、sethood または existence `InitialObligation` を発行する;
- `such that`、`given`、assumption-like clause は、導入元 context にだけ `Assumed`
  fact を追加する。

invalid declaration は explicit diagnostic と partial entry を生成しなければならない。
known fact を捏造したり、registration を silently activate したり、source-shaped typed site
を drop してはならない。

## Task 9: term and formula type inference

task 9 がこの section を実装する。

term inference は各 typed term site に `TypeEntry` を記録する。formula inference は
well-formedness と formula structure が導入する type fact を記録する。

term rules:

- variable reference は `BindingEnv` lookup result を消費し、selected binding または
  resolver symbol を typed site に付ける;
- `it` は current result type を提供する definition/property context 内でだけ valid である;
- numeral は resolver が公開する built-in numeric type payload を受け取るか、payload が
  欠ける場合は degraded external-gap type を受け取る;
- functor application は、final overload root selection が phase 6 で決定的でない場合、
  candidate group を保持してよい;
- selector access は、current type view 上で selected field/property が visible かを check し、
  overload resolution が choice を完了する必要がある場合は candidate group を記録する;
- structure constructor は resolver-exposed structure signature に対して field coverage と
  field value type を check する;
- set enumeration / set comprehension は set-like type を生成し、spec chapter 13 が要求する
  場合 generator domain の sethood obligation を作る;
- `the T` は choice-like typed term と `T` の non-emptiness obligation を記録するが、
  proof-owned id は割り当てない;
- source-written `qua` は `SourceQua` coercion candidate を作り、後続 checking が使う
  type view だけを変える。

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

必須 behavior:

- widening candidate が proof-free であるのは、known type fact、built-in radix widening、
  structure inheritance payload、または task-scoped seam を通じて利用可能な already
  activated dependency summary に支えられる場合だけである;
- source-written `qua` は statically checkable upcast または compatible view に対してだけ
  valid であり、narrowing proof として使ってはならない;
- より specific な type への narrowing は、後続 task が local discharge rule を明示しない限り
  `InitialObligation` を作る;
- `reconsider` は target type が known fact で既に支えられていない場合、existing-binding
  form と new-binding form の両方で narrowing obligation を作る;
- sethood と non-emptiness requirement は source assumption と deterministic local id を持つ
  `InitialObligation` を作る;
- failed / unsupported coercion は diagnostic 付きの `Blocked` または `Rejected` entry として残る。

`InitialObligationId` が phase-6 boundary である。task 10 は `VcId`、
`ObligationAnchor`、prover status、proof witness、accepted verifier status を割り当ててはならない。

## Task 11: type facts and queries

task 11 がこの section を実装する。

type fact は declaration checking、inference、coercion checking、後続
registration/overload phase が共有する local currency である。

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
- fact key は subject、predicate、polarity、provenance class、assumption visibility を制御する
  context を含む;
- contradictory fact は insertion order で解決せず、diagnostic と explicit status を作る。

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

## task 7-11 の予定テスト

task 7 は Rust test で次を覆う。

- attribute sorting、deduplication、polarity、contradiction diagnostic;
- built-in singleton head、radix head のまま残る structure head、recursive
  type-argument normalization;
- resolver payload がある場合の mode unfolding idempotence;
- signature payload 欠落時の degraded mode/type entry;
- unknown / ambiguous head、wrong arity/kind diagnostic、source-range
  preservation;
- deterministic normalized type id;
- type normalization が cluster closure で degraded type を repair しない guard。

task 8 は Rust test で次を覆う。

- `let`、quantified binder、definition parameter、reserved variable、`set`、
  `deffunc`、`defpred`、`reconsider` の binding type attachment;
- reserved-variable shadowing と `set` / `deffunc` / `defpred` の definition-time
  closure metadata;
- `reconsider` の両形式と constrained declaration の obligation emission;
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
| `source_drift` | `src/type_checker.rs` はまだ存在しない。 | task 7 が作成する想定である。task 6 では source repair しない。 |
| `external_dependency_gap` | 複数の実装 seam は mode unfolding、structure field、attribute、functor/predicate candidate、built-in、dependency activated summary の resolver-exposed signature payload に依存する。public checker diagnostic code も未割り当てである。 | 実装 task は公開済み resolver/artifact payload だけを消費する。不足 payload は external dependency gap または degraded diagnostic とし、direct raw-syntax reconstruction を追加しない。 |
| `deferred` | registration closure、reduction normalization、final overload selection、inserted overload-disambiguating `qua` view、VC generation、proof acceptance、kernel replay、artifact publication は task 6 と phase 6 の外に残る。 | 後続 checker task と downstream crate task がこれらの境界を所有する。 |
