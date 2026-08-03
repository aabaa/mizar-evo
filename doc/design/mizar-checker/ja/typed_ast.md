# mizar-checker: TypedAst

> 正本は英語です。英語版: [../en/typed_ast.md](../en/typed_ast.md)。

## 目的

`TypedAst` は、registration 閉包と最終的なオーバーロード解決が完了する前に、
型検査が生成する checker 所有の source-shaped な意味論スナップショットで
ある。これは次を精緻化する。

- [architecture 01](../../architecture/ja/01.ir_layers.md) の `TypedAst`
  所有関係
- [architecture 04](../../architecture/ja/04.type_and_registration_resolution.md)
  の phase 6 と `Typed AST` interface
- checker [todo.md](./todo.md) task 2

この文書は、checker 実装タスクが使う論理的なデータ形状を定める。task 3 が
物理的な arena 表現の決定を記録し、type inference、registration firing、
overload selection、言語意味論、証明挙動を追加せずにこれらの構造を実装する。

## 境界

`TypedAst` が所有するもの:

- 1 つの解決済みソースモジュールに対する typed node arena
- resolver node への source-shaped なリンクと checker の recovery state
- syntax-free resolver-shell projection から生成された optional immutable
  source-item/declaration/`BindingEnv` handoff
- typed site を解釈するために必要な immutable local type context snapshot
- checker の型情報を受け取る expression、formula、declaration、binding site
  の `TypeTable` entry
- declared、assumed、inferred、built-in、obligation-backed の型事実を表す
  `TypeFactTable` entry
- widening、narrowing、source-written `qua` の coercion candidate を表す
  `CoercionTable` entry
- `InitialObligationId` で識別される checker-local な `InitialObligation`
- typed snapshot の決定的な diagnostic と debug rendering

`TypedAst` が所有しないもの:

- name lookup、label lookup、import/export validation、resolver の symbol
  allocation
- final ordinary overload root selection、active refinement joining、または
  overload の曖昧性解消のために挿入される `qua` view
- registration activation、cluster closure、reduction normalization、または
  正準 `ResolutionTrace` schema
- `VcId`、`ObligationAnchor`、VC generation、ATP search、proof acceptance、
  kernel replay
- stable artifact schema publication や cache storage

この層で許される obligation identity は `InitialObligationId` だけである。
checker は `TypedAst` の構築中に `VcId` を割り当てたり、保存したり、導出
したりしてはならない。

## トップレベル形状

論理的なトップレベル形状は次のとおりである。

```rust
struct TypedAst {
    source_id: SourceId,
    module_id: ModuleId,
    resolved_root: Option<ResolvedNodeId>,
    source_context: Option<SourceBindingContextHandoff>,
    source_type: Option<SourceTypeApplicationHandoff>,
    source_attribute: Option<SourceAttributeHandoff>,
    nodes: TypedNodeArena,
    root: Option<TypedNodeId>,
    contexts: LocalTypeContextTable,
    types: TypeTable,
    facts: TypeFactTable,
    coercions: CoercionTable,
    initial_obligations: InitialObligationTable,
    diagnostics: TypeDiagnosticTable,
}
```

`source_id` と `module_id` は `ResolvedAst` から来る。これは source-map と
module-boundary の検査のために保存されるのであって、証明 identity や
artifact identity ではない。前提となる resolution が、source-shaped な
checker shell を構築できる前に失敗した場合、`resolved_root` と `root` は
存在しないことがある。recoverable な resolver error または type error の後に
十分な source shape が残っている場合、checker は subtree を黙って落とすの
ではなく、recovered typed shell を割り当てるべきである。

`TypedAst` 内のすべての id は typed snapshot に局所的である。同等の
`ResolvedAst`、`SymbolEnv`、dependency summary、checker configuration に対して
決定的でなければならないが、安定した public artifact identity でも
proof-reuse identity でもない。

Task 248 は complete source-item/binding-context handoff の唯一の owner として
`source_context` を追加する。matching `LocalTypeContextTable` と同時にだけ install
し、source/module、typed-site、binding、context link を transactional に validate
する。recovered-empty producer result は incomplete で install できない。この field
がない場合、deterministic debug output は Task-248 以前と byte-identical である。

Task 249はvalidated flat type-head/application/argument handoffのsole ownerとして
`source_type`を追加する。install時に全expression/head/term/`qua` typed siteを
attached arenaへ再照合し、same-source range containmentとexact recoveryを要求する。
producerがbinding/symbol environmentを既にauthenticateしており、`TypedAst`は
それらをreplace/reconstructできない。このfield absent時はconditional renderingが
existing debug byteを維持する。

Task 250はvalidated raw chain/attribute/qualifier/argument-group/actual handoffの
sole ownerとして`source_attribute`を追加する。install時にsource/module identity、
exact Task-249 expression association、各attached arena siteのrange/recovery stateを
再検証する。dense parent/order link、polarity、qualifier/symbol provenance、
punctuation、actual kind/origin、compositional spelling consistencyはproducer-time
invariantであり、immutable handoffが保持する。installはそれらをreconstructまたは
re-authenticateしない。partial/recovered bundleはinstallしない。field absent時は
conditional renderingがexisting debug byteを維持する。

## Node Arena

`TypedNodeArena` は、局所的に安定した `TypedNodeId` を持つ source-shaped な
`TypedNode` を保存する。

必須の node data:

- 元になった resolved node shape に対応する source-shaped kind
- source range または generated/recovered anchor
- source order の 0 個以上の child `TypedNodeId`
- node が resolver syntax から来た場合の、元 `ResolvedNodeId` への必須リンク
- node-local な type、fact、coercion、diagnostic、initial-obligation entry への
  optional table key
- successful、assumed、unknown、error、skipped typing を区別する `TypingState`
- typed node が degraded shell である場合の recovery metadata

arena invariant:

- すべての child id は同じ arena に割り当てられた node を参照する。
- parent/child edge は acyclic である。
- child order は決定的かつ source-shaped である。
- 同等の入力を検査すると同じ id と順序が生成される。
- unsupported だが recoverable な source shape は、`ResolvedAst` が十分な
  shape を保持している場合 degraded typed shell として表現される。
- arena id を `VcId`、`ObligationAnchor`、artifact id、cross-edit proof-reuse
  identity として使ってはならない。

task 3 の決定: `TypedAst` は、source-shaped role を `TypedNodeKind` が持つ
`TypedNode` record の同質 arena を使う。この arena は insertion order で dense な
local `TypedNodeId` を割り当て、`TypedAst` を受理する前に child link と acyclicity
を検証する。これは、共有 id 抽象が source-shaped traversal を所有し、node-specific
meaning は node kind payload または side table に置く、現在の `mizar-syntax`
compatibility view と `mizar-resolve` arena style を鏡映する。

`TypedNodeKind` は checker-local な source-shape projection である。task 3 は、
parser node kind を保存するためだけに direct `mizar-syntax` dependency を追加して
はならない。typed node が resolved source node を鏡映する場合、stable な
checker-local kind name と元の `ResolvedNodeId` を記録する。後で `mizar-resolve`
が projection を公開する場合、後続 task がそれを追加してよい。unsupported または
generated checker shell は raw parser vocabulary ではなく明示的な checker-local
kind name を用いる。

typed node struct は、後続 task が id stability、side-table ownership、deterministic
debug rendering を変えずに具体的な複雑さを減らせることを示した場合にだけ、
将来の refactor 候補として残る。

## LocalTypeContextTable

`LocalTypeContextTable` は、typed site で見えている checker-local context の
immutable snapshot を保存する。これは、architecture 01 が `TypedAst` は local
type context を所有すると述べる点と、`binding_env.md` が context construction を
指定するというタスク分割を整合させる。

```rust
struct LocalTypeContext {
    id: LocalTypeContextId,
    owner: TypedSiteRef,
    parent: Option<LocalTypeContextId>,
    layer: TypeContextLayer,
    bindings: Vec<BindingTypeRef>,
    introduced_assumptions: Vec<TypeFactId>,
    visible_facts: Vec<TypeFactId>,
    recovery: ContextRecoveryState,
}
```

必須 invariant:

- context entry は mutable な checker `TypeContext` ではなく immutable snapshot
  である。
- parent link は acyclic な layer chain を形成する。
- binding は name lookup をやり直さず、resolver 所有 symbol または typed binding
  site を参照する。
- `introduced_assumptions` はこの context layer が導入した
  `FactStatus::Assumed` fact を記録する。
- visible fact list は決定的に sort され、その context で消費可能な status の
  fact だけを含んでよい。
- `Assumed` fact は、current context の `introduced_assumptions` にあるか、
  visibility が残っている ancestor context にある場合だけ消費できる。
- recovered context は明示的であり、後続 phase が degraded assumption を
  verified evidence として扱うことを避けられる。

詳細な lookup、layer-building、binder-identity rule は task 4 と 5 の
`binding_env.md` が指定する。task 2 は storage shape だけを予約する。

## TypeTable

`TypeTable` は、typed site に付く型情報の checker 内の正準テーブルである。

```rust
struct TypeEntry {
    id: TypeEntryId,
    owner: TypedSiteRef,
    expected: Option<NormalizedTypeId>,
    actual: TypeEntryActual,
    status: TypeStatus,
    provenance: TypeProvenance,
}

enum TypeStatus {
    Known,
    Assumed,
    Unknown,
    Error,
    Skipped,
}
```

task 3 は `TypeStatus::is_available_for_handoff()` を status predicate としてだけ
公開する。`Known` と `Assumed` は provenance とともに後続へ渡してよいが、
`Unknown`、`Error`、`Skipped` は明示的な partial-typing record として残る。

`TypedSiteRef` は、typed node または binding site、expression result、
formula result、type expression、candidate argument などの安定した sub-node role
への source-local な参照である。raw surface syntax を指してはならない。
resolver 所有の id は、所有 typed node の resolver link を通してのみ参照してよい。
typed site order は、所有する `TypedNodeId`、whole-node entry、role entry、
安定 role key の順で並べる。

`TypeEntryActual` は、その site で分かっている normalized type、final overload
root が未確定の candidate set、または error 後に型がない状態を記録する。
`Error`、`Unknown`、`Skipped` status の table entry は明示状態であり、
成功した型の捏造ではない。handoff 可能な `Known` または `Assumed` entry は、
known normalized type または candidate set のどちらかを持たなければならない。
`Absent` は partial、error、skipped typing state 専用である。Recovery
provenance は存在する `TypeDiagnosticId` を参照しなければならない。

必須 invariant:

- 各 typed site は高々 1 つの primary `TypeEntry` を持つ。
- 補助的な expected-type constraint は、source traversal order だけに保存する
  のではなく、primary entry からリンクされる。
- normalized type id は正準 type key から決定的に割り当てられる。
- unresolved overload candidate は、あり得ない arity、kind、mandatory type
  constraint によって filter されてよいが、final root selection は `TypedAst`
  内で完了として表現しない。
- query と debug rendering の順序は typed site order、その後 table id である。

## TypeFactTable

`TypeFactTable` は、phase 6 と後続の registration/overload 作業が消費する
事実を保存する。

```rust
struct TypeFact {
    id: TypeFactId,
    subject: TypedSubjectRef,
    predicate: TypePredicateRef,
    polarity: Polarity,
    provenance: FactProvenance,
    status: FactStatus,
}

enum FactProvenance {
    Declared(SourceRange),
    Assumed(TypeAssumptionId),
    Inferred(TypeRuleId),
    Obligation(InitialObligationId),
    Builtin(BuiltinRuleId),
    Registration(ResolutionStepId),
}

enum FactStatus {
    Known,
    Assumed,
    PendingObligation,
    Degraded,
    Rejected,
}
```

`Registration` provenance は、registration closure 後に生成される enriched fact
table のために予約される。phase 6 は table shape を共有するために variant を
定義してよいが、phase 7 が対応する `ResolutionTrace` step を記録する前に
cluster-derived fact を作り出してはならない。

`FactStatus` は消費可否を制御する。

- `Known` fact は active checker evidence として消費してよい。
- `Assumed` fact は、その assumption を導入した local context の中でだけ消費して
  よく、assumption として印を残さなければならない。
- `PendingObligation` fact は、proof handoff が `InitialObligationId` で表される
  claim を説明するが、verified evidence ではない。
- `Degraded` fact は diagnostic または recovery metadata 専用である。
- `Rejected` fact は diagnostic を説明するためだけに保持され、消費または export
  できない。

task 3 は `FactStatus::is_unconditionally_consumable()` を `Known` case にだけ
公開する。Assumed fact は visible になる前に local-context introduction を
必要とし続ける。

必須 invariant:

- fact は canonical subject、predicate、polarity、provenance key によって
  重複排除される。
- `Obligation` provenance は存在する `InitialObligationId` を参照しなければ
  ならない。
- 矛盾する fact は、hash や traversal の偶然で解決するのではなく、
  diagnostic と status によって記録される。
- error node から導かれた invalid fact は local degraded metadata として残って
  よいが、verified metadata として export したり active evidence として消費
  したりしてはならない。
- recoverable assumption のもとで生成された fact は、完全に known な fact と
  区別できる。
- 決定的 query は canonical fact key、その後 `TypeFactId` で並ぶ。

## CoercionTable

`CoercionTable` は checker が見つけた coercion candidate を記録する。これは
最終的な implicit view が `ResolvedTypedAst` に挿入されたことを意味しない。

```rust
struct CoercionEntry {
    id: CoercionId,
    site: TypedSiteRef,
    from: Option<NormalizedTypeId>,
    to: NormalizedTypeId,
    kind: CoercionKind,
    status: CoercionStatus,
    supporting_facts: Vec<TypeFactId>,
    obligation: Option<InitialObligationId>,
    provenance: CoercionProvenance,
}

enum CoercionKind {
    Widening,
    Narrowing,
    SourceQua,
}

enum CoercionStatus {
    Candidate,
    RequiresObligation,
    Blocked,
    Rejected,
}

enum CoercionProvenance {
    WideningRule(TypeRuleId),
    NarrowingClaim(SourceRange),
    SourceQua(SourceRange),
    Recovery(TypeDiagnosticId),
}
```

task 3 は `CoercionStatus::is_available_for_handoff()` を公開し、後続 phase が
renderer text から推測せずに `Candidate` と `RequiresObligation` を
`Blocked` / `Rejected` から区別できるようにする。Recovery provenance は存在する
`TypeDiagnosticId` を参照しなければならない。

必須挙動:

- widening candidate は、記録済み type fact によって正当化される proof-free な
  semantic view でなければならず、その根拠は `supporting_facts` に保存する。
- narrowing candidate は、task 10 の known-fact support または後続 spec により
  VC generation なしで局所的に discharge できると示されない限り、
  `InitialObligationId` を必要とする。
- `Candidate` entry は、参照する fact と type の status に従って後続 phase から
  利用可能であり、provenance も保持する。
- `RequiresObligation` entry は `InitialObligationId` を持ち、verified coercion
  ではない。
- `Blocked` と `Rejected` entry は diagnostic/recovery record 専用である。
- source-written `qua` expression は source view として保持され、candidate
  constraint に寄与してよいが、task 2 は overload-root disambiguation を
  指定しない。
- final overload-driven inserted `qua` view は `TypedAst` ではなく
  `ResolvedTypedAst` に属する。
- candidate ordering は site order、kind、target type、provenance によって
  決定的である。provenance key が同じ場合は `supporting_facts` order が
  tie-breaker になる。これらの key も同一の場合に限り、source type と
  `CoercionId` を決定的な最終 tie-breaker として使う。

## InitialObligation

`InitialObligationTable` は、VC generation より前に作られる checker-local な
obligation を保存する。

```rust
struct InitialObligation {
    id: InitialObligationId,
    kind: InitialObligationKind,
    owner: TypedSiteRef,
    source_range: SourceRange,
    assumptions: Vec<TypeFactId>,
    goal: InitialObligationGoal,
    provenance: InitialObligationProvenance,
    status: InitialObligationStatus,
}

enum InitialObligationStatus {
    Pending,
    Blocked,
    Invalidated,
}
```

task 3 は `InitialObligationStatus::is_available_for_handoff()` を `Pending`
obligation にだけ公開する。`Blocked` と `Invalidated` obligation は、所有する
後続 task が変更するまで diagnostic state として残る。

必須 obligation kind:

- type expression と witness を導入する構文が必要とする sethood obligation
- `the T` のような choice term の non-emptiness obligation
- `reconsider` や不正または非自明な narrowing claim の narrowing obligation
- registration validation task が table を精緻化した後の registration
  correctness obligation

必須 invariant:

- `InitialObligationId` は `TypedAst` snapshot 内で決定的である。
- id は source order で割り当てられ、同じ site に複数 obligation がある場合は
  決定的な tie-breaker を用いる。
- table は後続で VC generation input へ変換できるだけの assumption と source
  provenance を保持する。
- `Pending` obligation は、後続の proof-owned VC generation に渡せる。
- `Blocked` obligation は、前提となる type または resolver data が degraded で
  ある場合に diagnostic のために保持される。
- `Invalidated` obligation は handoff できず、local error を説明するためだけに
  保持される。
- どの field も `VcId`、`ObligationAnchor`、prover result、proof witness、
  accepted verifier status を保存しない。
- 後続の VC generation が、proof-owned boundary で initial obligation を
  `VcId` へ写像する。

## TypeDiagnosticTable

`TypeDiagnosticTable` は、type data shape と recovery の checker-local diagnostic
record を保存する。dedicated diagnostic code-space が external planning gate として
残る間、public diagnostic code は割り当てない。

```rust
struct TypeDiagnostic {
    id: TypeDiagnosticId,
    owner: Option<TypedSiteRef>,
    source_range: SourceRange,
    class: TypeDiagnosticClass,
    severity: TypeDiagnosticSeverity,
    message_key: String,
    recovery: DiagnosticRecoveryState,
}
```

必須 invariant:

- `TypeDiagnosticId` は `TypedAst` snapshot に局所的である。
- `message_key` は stable crate-internal key であり、public diagnostic code では
  ない。
- diagnostic は source range、class、message key、その後 id で sort される。
- diagnostic record は degraded type、fact、coercion、context、initial obligation
  を説明してよいが、proof evidence ではない。
- diagnostic field は verifier status、proof witness、`VcId` を保存しない。

## エラー後の部分型付け

十分な source shape が残る場合、型検査は recoverable な resolver error または
type error の後も継続すべきである。

recovery contract:

- unresolved name、ambiguous name、failed type expression、impossible overload
  candidate、invalid coercion は明示的な degraded table entry を生成する。
- 後続 phase を動かすために `Known` entry を捏造してはならない。
- `Assumed` entry は recovery を可能にした assumption を記録しなければならない。
- `Unknown`、`Error`、`Skipped` entry は registration、overload、diagnostic、
  debug rendering から見える。
- degraded site に付く fact と coercion は、verified evidence として消費され
  ないように degraded status または diagnostic を持たなければならない。
- diagnostic は安定した secondary key を伴って決定的な source order で出力される。

後続 phase は type、fact、coercion を消費する前に status を検査しなければ
ならない。registration resolution は invalid fact から registration を発火して
はならない。overload resolution は failed site を保持してよいが、成功した
core term として elaborate してはならない。

## 決定的 Debug Rendering

task 3 は、exact な `typed-ast-debug-v1` header を持つ決定的な debug rendering
として `TypedAst::debug_text()` を提供しなければならない。rendering contract:

- top-level id、arena node、type entry、fact、coercion、initial obligation、
  diagnostic を安定順に render する。
- source reference は memory address や host path ではなく、source-local range
  または resolver/typed id として render する。
- map と set は canonical key order で render する。
- degraded status を明示的に含める。
- hash-map iteration order や allocation address に依存しない。

debug format はテストとレビューの補助であり、stable public artifact schema では
ない。

## Public Enum Policy

task 31 は frontend task-25 の public-enum decision procedure をこの module に適用する。
`typed_ast` の public checker-owned enum はすべて forward-compatible API surface であり、
`#[non_exhaustive]` を維持しなければならない。downstream consumer は wildcard または
fallback arm を保持する。checker 内部の match は、仕様化済み behavior を実装するために
現在表現されている variant へ exhaustive のままにしてよい。

| enum | decision |
|---|---|
| `TypingState` | 前方互換; phase-6 node typing state は recovery と handoff state の精緻化に伴い増える可能性がある。 |
| `NodeRecoveryState` | 前方互換; node recovery category は parser/checker recovery integration に伴い増える可能性がある。 |
| `TypedArenaError` | 前方互換; arena validation failure は新しい structural check を追加する可能性がある。 |
| `TypedSiteRef` | 前方互換; typed-site ownership は追加の checker-owned role を得る可能性がある。 |
| `TypeContextLayer` | 前方互換; local context layer は statement/proof extraction が入るにつれて増える可能性がある。 |
| `BindingTypeRef` | 前方互換; binding type reference は追加の checker-owned anchor を得る可能性がある。 |
| `ContextRecoveryState` | 前方互換; context recovery category はより豊かな partial checking とともに増える可能性がある。 |
| `TypeStatus` | 前方互換; type availability state は downstream handoff policy の精緻化に伴い増える可能性がある。 |
| `TypeEntryActual` | 前方互換; type-entry actual payload は後続 checker phase とともに増える可能性がある。 |
| `TypeProvenance` | 前方互換; type provenance は追加の checker-owned evidence class を得る可能性がある。 |
| `Polarity` | 前方互換; checker がより豊かな logical qualifier を記録する場合、predicate polarity は増える可能性がある。 |
| `FactProvenance` | 前方互換; fact provenance は proof、registration、artifact input とともに増える可能性がある。 |
| `FactStatus` | 前方互換; fact consumption state は obligation と artifact flow の成熟に伴い増える可能性がある。 |
| `CoercionKind` | 前方互換; coercion category は source と inserted-view handling に伴い増える可能性がある。 |
| `CoercionStatus` | 前方互換; coercion state は proof/artifact validation の接続に伴い増える可能性がある。 |
| `CoercionProvenance` | 前方互換; coercion provenance は追加の evidence source を得る可能性がある。 |
| `InitialObligationKind` | 前方互換; initial obligation category は VC と proof integration に伴い増える可能性がある。 |
| `InitialObligationStatus` | 前方互換; obligation status は proof/artifact handoff の接続に伴い増える可能性がある。 |
| `TypeDiagnosticClass` | 前方互換; diagnostic class は public checker diagnostic code が割り当てられる前に増える可能性がある。 |
| `TypeDiagnosticSeverity` | 前方互換; diagnostic severity policy は IDE/artifact consumer とともに増える可能性がある。 |
| `DiagnosticRecoveryState` | 前方互換; diagnostic recovery state は partial-checking policy に伴い増える可能性がある。 |
| `TypedAstError` | 前方互換; top-level typed-AST validation failure は新しい variant を得る可能性がある。 |

この module が所有する exhaustive public enum exception はない。

## task 3 の予定テスト

task 3 は次を覆う Rust test を追加しなければならない。

- 同等入力に対する `TypedNodeId`、`TypeEntryId`、`TypeFactId`、`CoercionId`、
  `InitialObligationId` allocation の決定性
- table insertion と query round-trip
- local context snapshot の insertion と query、決定的な context ordering、
  parent-chain validity、consumable status による visible-fact filtering、
  recovered-context marking
- fact deduplication と deterministic query ordering
- `Known` と `Assumed` type entry、consumable / pending / degraded / rejected
  fact、blocked / rejected coercion、handoff してはならない blocked /
  invalidated obligation の status consumption rule
- coercion candidate ordering と obligation link
- `Unknown`、`Error`、`Skipped` status に対する partial typing entry
- `TypedAst` data shape が `VcId`、`ObligationAnchor`、proof witness、prover
  result、accepted verifier status を保存しないことの boundary guard
- final overload root、active refinement、overload の曖昧性解消のために挿入
  される `qua` view が `TypedAst` に存在しないことの boundary guard
- deterministic debug rendering

task 2 では実行可能な checker semantics がまだ存在しないため、`.miz` の
checker-stage fixture は不要である。最初の active `type_elaboration` corpus runner
と traceability entry は task 12 が所有する。

現在の source-derived runner note: `mizar-test` type-elaboration runner は bounded
reserve-only bare-builtin declaration pass bridge のために explicit checker-owned
`TypedAst` node を構築してよい。各 reserve binding は declaration node と binding 固有の
type-expression node を持つ。複数 binding が同じ source type range を共有しても、
distinct `TypedSiteRef` owner を使う。same-module attributed builtin reserve head と
local-mode reserve head は active fail slice のみである。active runner は stable
diagnostic key を集めるために同じ checker-owned assembly helper を使ってよいが、これらの
slice は successful `TypedAst` readiness payload として credit しない。これは `TypedAst` を
checker-owned payload surface のまま保つためのものであり、`mizar-checker` に raw syntax
walking、general declaration extraction、CoreIr、ControlFlowIr、VC payload、proof evidence
を許可しない。

## task 2 の分類

| Class | Finding | Action |
|---|---|---|
| `spec_gap` | `TypedAst` の data-shape boundary については見つかっていない。architecture 01 と 04 はこの docs-only task に十分な authority を与える。 | この spec の review と commit 後、task 3 へ進む。 |
| `test_gap` | checker semantic fixture directory と `type_elaboration` runner はまだ存在しない。task 3 は proof-owned id と final overload/view field に対する明示的な boundary guard も必要とする。 | task 3 が Rust data-shape と boundary test を追加し、task 12 が active corpus coverage を追加する。 |
| `design_drift` | architecture 01 は `TypedAst` が local type context を所有すると述べる一方、`todo.md` は context construction を `binding_env.md` に割り当てている。さらに architecture 01 は coercion side table を `CoercionTable` と呼び、architecture 04 の例は `CoercionCandidateTable` を使っている。 | この spec は `LocalTypeContextTable` storage を予約しつつ construction rule を task 4-5 に延期することで context split を解決する。checker module 名を `CoercionTable` として標準化し、それが candidate entry だけを保存することを明記する。task 2 では architecture rename は行わない。 |
| `source_drift` | なし。task 1 は crate scaffolding だけを導入し、checker semantic source はない。 | task 2 では source repair は不要。 |
| `external_dependency_gap` | task 2 をブロックするものはない。後続 task は resolver payload、diagnostic code ownership、artifact summary、proof acceptance input に依存し続ける。 | 所有する実装タスクで再評価する。欠けている外部データを捏造しない。 |
| `deferred` | typed arena については task 3 で解決済み: dense local id を持つ同質な `TypedNodeKind` arena を使う。後続 semantic task はそれぞれの external dependency gate を所有し続ける。 | 将来の表現 refactor は behavior-preserving かつ task-scoped に保つ。 |

## task 3 の分類

| Class | Finding | Action |
|---|---|---|
| `spec_gap` | task 3 が checker-local node-kind projection、diagnostic table shape、context assumption link を追加した後、data-shape implementation をブロックするものはない。 | 文書化済み data shape と deterministic rendering だけを実装する。 |
| `test_gap` | task 2 は id、table、context、status、proof-boundary guard、final-overload-field 不在、rendering の Rust coverage 欠落を記録した。 | task 3 の Rust unit test で解決済み。`.miz` semantic fixture は task 12 のまま。 |
| `design_drift` | task-1 lint guard は crate が public semantic API を公開しないと記述し、TODO decision もこの task 前は arena representation を open と述べていた。 | task 3 で解決済み: guard は文書化済み `typed_ast` API だけを許し、TODO decision text も arena decision を記録する。 |
| `source_drift` | task 3 前は source に `typed_ast` module がなく、task 2 がそれを仕様化していた。 | task 3 で解決済み: `src/typed_ast.rs` を追加し、`lib.rs` から文書化済み module だけを公開する。 |
| `external_dependency_gap` | public checker diagnostic code ownership は未確定のままであり、resolver は後でより豊かな source-kind projection を公開する可能性がある。どちらも task 3 はブロックしない。 | diagnostic は stable `message_key` を持つ crate-internal record に保つ。node-kind storage のための direct `mizar-syntax` dependency は追加しない。 |
| `deferred` | task-3 decision 後、typed arena の物理表現について残る deferred はない。type inference、binding construction、registration firing、overload resolution、public diagnostics、artifact、proof acceptance は後続 task が所有する。 | task 3 は data-only に保つ。 |

## Task 251 ownership addendum

`TypedAst`はoptional immutable `SourceEvidenceHandoff`をownする。
`with_source_evidence`はreplacementをrejectし、installation前にhandoff
source/moduleとreferenced factをexisting typed payloadへauthenticateする。この
追加はsyntax-freeで、evidence truth、accepted fact、proof status、downstream
IRをtyped arenaへ追加しない。

## Task 252 ownership addendum

`TypedAst`はoptional immutable `SourcePrimaryTermHandoff`をownする。
`with_source_term`はreplacementをrejectし、installation前にhandoff
source/moduleとreferenced arena nodeすべてをauthenticateする。この追加は
syntax-freeで、normalized type、semantic term/formula、accepted fact、proof
status、downstream IRをtyped arenaへ追加しない。

## Task 253 ownership addendum

`TypedAst`はoptional immutable `SourceFunctorApplicationHandoff`をownする。
`with_source_application`はone-shotで、Task-252 handoffが先にinstall済みである
ことを要求し、そのexact deterministic debug fingerprintを比較して全referenced
primary rootをinstallation前にrevalidateする。Task-254 handoffが既にpresentなら
fieldをcommitする前にnew Task-253 ownership graphに対してそのhandoffも
revalidateし、install順によるshared primary、reverse containment、partial overlap、
unowned contained applicationを許さない。equivalent Task-252 cloneはacceptし、
replacementとnon-equivalent same-source/module substitutionはatomicにfailする。
signature、result type、candidate selection、definition behavior、semantic
term/formula、fact、proof、downstream IRは追加しない。

## Task 254 ownership addendum

`TypedAst`はoptional immutable `SourceStructureHandoff`をownする。
`with_source_structure`はone-shotで、Task 252とtargetされるTask-253 dependencyの
先行installを要求し、exact deterministic fingerprint、全Task-252/253/254
target、term/member/FieldUpdate/wrapper arena siteをinstallation前にrevalidate
し、producer-validated direct written partitionを保持する。Task-254 application fingerprint
absent時のunrelated install済みTask-253 handoffはtarget/rangeがTask 254と
disjointな場合だけ共存できる。replacement、wrong-key input、non-root/reverse
Task-253 ownership、shared Task-253 argument primary、non-equivalent dependency
substitutionはatomicにfailする。structure signature、member/view identity、
result type、semantic constructor/selector/update、fact、proof、downstream IRは
追加しない。

## Task 255 ownership addendum

`TypedAst`はoptional immutable `SourceSetTermHandoff`をownする。
`with_source_set_term`はone-shotで、Task 252と全targeted Task-253/254 dependencyの
先行installを要求し、exact deterministic fingerprint、全Task-252/253/254/255
target、arena site、canonical spelling、nearest-family ownershipをinstallation前に
revalidateする。`with_source_application`/`with_source_structure`もfield commit前に
installed Task-255 handoffをrevalidateし、どちらのinstall順でも同じpartitionを
保つ。fingerprint absent時のunrelated optional handoffはoccurrenceがrange-disjoint
な場合だけ共存できる。replacement、missing dependency、non-root/reverse
ownership、overlap、non-equivalent dependency substitutionはatomicにfailする。
comprehension binding/capture、formula、sethood/nonemptiness/widening result、
semantic term/type、fact、proof、downstream IRは追加しない。

## Task 256 ownership addendum

`TypedAst`はoptional immutable `SourceAtomicFormulaHandoff`を所有する。
`with_source_atomic_formula`はone-shotで、Task 252とtargeted Task-253/254/255
handoffの先行installを要求し、exact deterministic fingerprint、formula site、
provenance、request、nearest-family target partitionを再検証する。later
Task-253/254/255 installerはfield commit前にinstalled Task-256 handoffを再検証し、
install orderによるownership/fingerprint bypassを許さない。replacement、
missing/non-equivalent dependency、non-root target、overlap、arena/provenance driftは
atomicにfailする。candidate selection、expected-type answer、assertion fact/truth、
formula result、theorem acceptance、proof、downstream IRは追加しない。

## Task 257A ownership addendum

`TypedAst`はoptional immutable `SourceCompositeFormulaHandoff` 1件をownする。
`with_source_composite_formula`はone-shotでTask-248 source-context handoffとの
coexistenceをrejectし、publication前にsource/module identity、complete typed
arena、exact extended `BindingEnv`、dense table 7個を再検証する。borrowed getterが
syntax-free transportを公開する。field absent時はlegacy ASTのexact debug bytesを
保持する。handoffはunresolved source intentだけを持ち、formula truth、type
answer、fact、theorem owner、proof、acceptanceを作らない。

## Task 257B1 Ownership Addendum

`TypedAst`はoptional immutable `SourceFormulaCompositionHandoff`もownする。
combined `with_source_formula_composition` installerはpreinstalled Task-252
primary term/Task-256 atomic formulaを要求し、第2 composite profileとcompositionを
同時にvalidate/publishする。Task-248 source-context coexistence、installed
Task-257A profile、dependency drift、partial publicationをrejectする。legacy
composite installerは第2 profileを引き続きrejectする。

Task 257B2は同じcombined installerをexact third composite profile +
composition `8/0`へ拡張する。exact installed Task-252/256 dependenciesを必須とし、
missing/stale fingerprint、Task-248 coexistence、existing A/B1/B2 ownership、
partial publicationをrejectする。legacy composite-only installerは
Task-257A-onlyのまま。

## Task 257B3 Frozen Ownership Addendum

combined installerはTask-48-derived one-reserve baseとexact Task-252/256
dependency上のexact fourth composite profile/`3/6` compositionだけをadmitする。
Task-248 reserve-plus-definition profileはこのconsumerでないため
`source_context()`はabsentのまま。reserve-default provenance、nested context、
shadow、owning atomic edge、lookup replayを認証し、existing A/B1/B2/B3
ownershipまたはpartial publicationをrejectする。legacy composite-only
installerはA-onlyのまま。

B3 combined installationとduplicate/collision rollback pathはexecutableに
なった。Task-248 exclusionとlegacy installerは不変。

## Task 257C1 frozen ownership addendum

caller/pipelineがTask 252を先にinstallし、その後
`TypedAst::with_source_atomic_formula`がextended Task-256 handoffだけを
atomicにvalidate/publishする。exact chain transactionはshared boundary edge
1件、imported candidate 2件、candidate request 2件を含む`3/0/3`と
`1/0/2/2/2/0/0/3/2`をauthenticateする。missing/stale dependency
fingerprint、old-family collision、partial publication、segment corruptionは
fail closed。existing Task-256/Task-257A/B1/B2/B3 installer/bytesはexclusive
かつ不変。

Task 257C1はexisting one-shot `with_source_atomic_formula` pathで実装済み。
successful install/subsequent clone revalidationは9 tableすべてを保持し、
tested partial/cross-profile mutationはatomicかつfail-closedである。

## Task 255C1 frozen ownership addendum

existing one-shot `with_source_set_term` pathはcomplete Task-252 `4/0/4`と
Task-253 `1/0/1/2/2` dependencyのinstall後だけexact seven-table
Task-255C1 profileをadmitする。colon/direct condition-wrapper anchor、
condition内lower-family exclusion、両fingerprintをrevalidateする。condition
operandはTask-255 edgeなしでimmutable Task-252 handoffに残る。failureはcondition
rowをpublishせず、既存field/debug byteをすべて保持する。

## Task 255C1 installation result

`with_source_set_term`はTask 252/253後にauthenticated seven-table handoffを
installする。condition/dependency revalidation failureは何もpublishせず、
unchanged base objectからlater valid installが成功する。legacy condition-empty
objectはbyte-identicalのままである。

## Task 257C2 frozen ownership addendum

`TypedAst`はTask-252/253/255/256 install後にoptional
`SourceConditionFormulaCompositionHandoff` 1件を次のexact surfaceで公開する。

```rust
pub const fn source_condition_formula_composition(
    &self,
) -> Option<&SourceConditionFormulaCompositionHandoff>;

pub fn with_source_condition_formula_composition(
    self,
    composition: SourceConditionFormulaCompositionHandoff,
) -> Result<Self, TypedAstError>;

// New #[non_exhaustive] TypedAstError variant:
InvalidSourceConditionFormulaComposition,
```

installerはlower fingerprint 4件、direct condition-wrapper/equality relation、
exact operand ownership、association row 1件をreauthenticateする。missing/
substituted dependency、existing Task-257 composite/Task-257B composition、
second condition-composition handoffをdedicated error variantでatomicに
rejectする。existing Task-257A/combined Task-257B installerのsignatureと
successful legacy behaviorは不変だが、C2 install済みをexisting error
variantでrejectするreciprocal fail-closed checkを両方へ追加する。testsは
A/B-before-C2とC2-before-A/Bをrollback込みでcoverする。frozen
pre-Task-256C1 baselineでは、C2 installerはseparate lower prerequisiteが
unrelated overlap guardをweakenせず、authenticated Task-255 condition
containmentをset/atomic両installation orderでpassさせるまでimplementでき
なかった。Task 256C1は両orderをpassし、C2 installerは現在実装済みで、
両lower installation orderとreciprocal A/B/C2 exclusion 4 orderを
byte-identical rollback付きでpassする。absent-handoff debug byteは不変で、
semantic tableをpopulateしない。

## Task 256C1 frozen installation revalidation

`TypedAst` API/production implementationは変更しない。existing symmetric
revalidationがcontractである。Task 255→Task 256ではinstalled set handoffに
対してatomic handoffをvalidateし、Task 256→Task 255ではincoming set handoffに
対してinstalled atomic handoffをrevalidateする。private Task-256 validator
fix後、両orderはauthenticated equality-condition containerだけをacceptし、
immutable handoff/full debug outputはbyte-identicalになる。

invalid overlapはexisting `InvalidSourceAtomicFormula`/
`InvalidSourceSetTerm` variantでatomicにfailし、fieldをpublishせず、
unchanged baseからvalid replayできる。final resolved revalidation/clone
ownershipは不変。

## Task 256C1 implementation result

existing symmetric installerはexact authenticated condition/equality relationを
両orderでpassする。substituted validation contextはexisting order-specific
errorでfailし、fieldをpublishせず、unchanged baseからreplayできる。equal full
debug outputによりinstallation orderがstateを追加しないことを確認した。
`TypedAst` source/APIは変更していない。

## Task 257C3 frozen ownership

later implementationはoptional `SourcePredicateChainCompositionHandoff`、
accessor、one-shot installer、debug projection、
`InvalidSourcePredicateChainComposition`を追加する。exact Task-252/256
handoffを要求し、Task-257A/B/C2 ownershipと全installation orderで
reciprocally exclusive。failureは何もpublishせずbyte-identical replayを
保持する。本documentation prerequisiteは`TypedAst` source/executable APIを
変更しない。

```rust
pub const fn source_predicate_chain_composition(
    &self,
) -> Option<&SourcePredicateChainCompositionHandoff>;

pub fn with_source_predicate_chain_composition(
    self,
    composition: SourcePredicateChainCompositionHandoff,
) -> Result<Self, TypedAstError>;
```

C3-after-A/B/C2は`InvalidSourcePredicateChainComposition`。A/B/C2-after-C3は
順に`InvalidSourceCompositeFormula`、`InvalidSourceFormulaComposition`、
`InvalidSourceConditionFormulaComposition`にfailする。directional 6 pathは
すべてatomic/replayable。optional C3 debug chunkはTask-252 source-term、
Task-256 source-atomic-formula、mutually exclusive A/B/C2 slotの後、
node/table section直前。

## Task 257C3 implementation result

optional field/accessor/one-shot installer/dedicated error/debug projectionを
実装した。installerはexact Task-252/256 dependencyを要求し、publication前に
duplicate/A/B/C2 occupancyをrejectする。reciprocal test-only occupancy
mutationはotherwise-valid attempted installで全6 directional guardを直接
exerciseし、lower dependency mismatchがownership contractを隠さない。
failureはbase debug byte/valid replayを保持する。

## Task 258A frozen source-statement ownership

later Task-258A implementationはoptional `SourceStatementHandoff`、read-only
accessor、one-shot installer、debug projection、`InvalidSourceStatement`を
追加する。exact Task-252/256 handoffが先にinstallされ、frozen `MT10-FS`
smoke profileでは他lower/Task-257 ownerはabsent。

```rust
pub const fn source_statement(&self) -> Option<&SourceStatementHandoff>;

pub fn with_source_statement(
    self,
    statement: SourceStatementHandoff,
) -> Result<Self, TypedAstError>;
```

installerはsource/module、lower fingerprint 2件、handoff内のresolver-
authenticated owner、全`1/1/1/1/1` row、arena site/range、binding
environmentとfingerprint、visibility、reference use、formula targetを
publication前にrevalidateする。exact `BindingEnv`はhandoff-owned。
duplicate/missing lower/stale/substituted binding/corruptはatomic failし、
byte-identical state/valid replayを保持。

Task 248/Task 258Aはexclusive。productionが公開するのはTask-248
constructor-first directionだけで、`source_context`後の
`with_source_statement`は`InvalidSourceStatement`。Task 248に
post-construction installerはなく本taskも追加しない。exact reverse test
oracleはcanonical English blockの`#[cfg(test)]`
`with_source_context_for_test`でsame private validationを呼び、
`InvalidSourceContext`。各rejectionはfirst ownerのexact debugとvalid replayを
保持する。separate `inject_source_statement_for_test(&mut self,
SourceStatementHandoff)` bypassはfinal-assembly coexistence rejectionの準備
だけに使いproduction construction pathではない。debug chunkは全Task-257 owner
slotの後、node/table sectionの前。`facts`/existing semantic tableはempty。
本documentation commitは`TypedAst` source/APIを変更しない。

### Task 258A implementation result

optional handoff、read-only accessor、one-shot installer、debug chunk、
dedicated `InvalidSourceStatement` pathを実装した。installationはfrozen
source-family exclusionとTask-252/256 dependencyに加え、generic typed
projection (`resolved_root`、context、type、fact、coercion、initial
obligation、diagnostic)がemptyであることを要求する。coexistence failureは
prior valueをmutateせず、valid replayはdeterministic。

## Task 258B1 frozen combined statement ownership

Task 258B1はTask-258A field/accessor/installerを不変に保ち、second optional
fieldと次のexact public APIを追加する。

```rust
pub const fn source_statement_references(
    &self,
) -> Option<&SourceStatementReferenceHandoff>;

pub fn with_source_statement_references(
    self,
    statements: SourceStatementHandoff,
    references: SourceStatementReferenceHandoff,
) -> Result<Self, TypedAstError>;
```

existing `with_source_statement`はTask-258A-onlyのまま。combined installer
だけがB1 `1/4/4/4/4 + 1/1` pairをadmitする。fresh statement/reference
slot、exact installed Task-252/256 handoff、handoff-owned `3/1/0`
environment、arena 1件、matching statement fingerprint、replay-
authenticated resolver projection/reference/result、sole resolved
`Label(0)` node 68/table-site parityを持つretained 77-node/root-76 resolver
ASTを要求する。全failureは`TypedAstError::InvalidSourceStatement`で、
両fieldはvalidation完了後だけpublishされる。

Task-248、全Task-257 owner、Task-258A、generic typed semantic table、
referencesなしbase、matching baseなしreferences、duplicate installation、
全opposite install orderはatomic failする。debugはbase statement chunkの直後、
node/table前にreference chunkを出す。Task-258Aにはsecond chunkがなくbyteは
identical。checked formula、fact、statement semantic、proof、goal、
diagnostic、accepted theoremを作らない。本prerequisiteは`TypedAst` sourceを
変更しない。

### Task 258B1 implementation status

`TypedAst::with_source_statement_references`はexact B1 base/reference pairを
validateしてatomicにinstallする。legacy statement installerはTask 258Aだけを
acceptし、既存payload ownerはpairをrejectし、validation failure後もoriginal
valueをreuseできる。accessor、clone、debug orderはsemantic outputなしで
frozen contractを保持する。

### Task 258B2 frozen typed ownership

Task 258B2はbase-only
`TypedAst::with_source_statement(SourceStatementHandoff)` pathを再利用する。
このinstallerはexact Task-258A profileまたはexact Task-258B2 profileをadmit
でき、Task 258B1は引き続き`with_source_statement_references`を必要とする。
B2ではfrozen 113-byte source identity、Task-48 `2/1/0`、Task-252
`6/6/0`、Task-256 `3/0/0/0/0/0/0/6/6`、statement profile
`1/3/3/3/3`を、sole proof contextとtheorem/assumption/conclusion rowを含めて
revalidateする。

B2 handoffはreference associationを持たない。reference handoff、Task-248/
Task-257 payload、duplicate install、source/profile mismatch、semantic-stage
inputは`TypedAstError::InvalidSourceStatement`としてatomicにfailする。
successful installはsyntax-free source tableとlower-stage provenanceだけを
ownし、fact、accepted premise、checked formula、statement semantic、proof、
goal、diagnostic、theorem resultを作らない。本prerequisiteは`TypedAst`
source/test/existing debug byteを変更しない。

### Task 258B2 implementation closure

`TypedAst::with_source_statement`はexact Task-258AまたはTask-258B2 base
profileだけをadmitする。Task-258B1はpair-onlyのまま。Task-248、
Task-257A/B/C2/C3、Task-258 cross-profile hybrid、occupied semantic table、
foreign-first/statement-firstの両ownership orderはpartial mutationなしで
failする。clone/debug orderもstableである。

### Task 258B3 frozen paired installation

`TypedAst`は
`source_statement_witnesses: Option<SourceStatementWitnessHandoff>`、
`source_statement_witnesses()`、
`with_source_statement_witnesses(statements, witnesses)`を追加する。paired
installerはinstalled Task-252/256 lower value、exact B3 base/witness
fingerprint、shared 49-node arena、empty reference/foreign source
family/semantic tableをrequireし、全validation success後だけ両halfをpublish。

`with_source_statement`はA/B2-only、
`with_source_statement_references`はB1-onlyを維持する。B3 baseはproducer
resultとしてexistできるがstandalone install不可。orphan/stale/
cross-profile、Task-248/257、B1-reference、両order ownership conflictは
mutationなしでrollbackする。debugはstable witness chunkをbase chunk直後に
appendし、prior profile bytesは不変。

### Task 258B3 paired installation result

`source_statement_witnesses()`と
`with_source_statement_witnesses(statements, witnesses)`をimplementした。
base-only/reference-paired installerはB3をrejectし、B3 installerは両halfを
validate後にatomic publishする。cross-family/両order rollback testはPASSし、
debugはbase、witness、nodes順。

## Task 258B3N planned paired ownership

existing paired installerをB3Nのsole publication pathとして維持する。
base、witness、dense name table、exact 51-node arena、B3N profileをvalidate
してからatomicにinstallする。existing B3はempty name tableでvalidなまま。
B3/B3N hybrid、repeated install、cross-family order、semantic coexistenceは
partial publicationなしでrollbackしなければならない。

## Task 258B3N 実装結果

paired installerはauthenticated B3またはB3N bundleをacceptし、exact B3N
dense name tableと51-node arenaを含めてvalidateする。base-only、
repeated、B3/B3N hybrid、reference、Task-248/257、semantic-table orderは
atomicにfailし、successful debug orderはbase、witness/name、nodesを維持する。

## Task 258B3M1 planned paired ownership

existing paired installerだけをpublication pathとする。exact B3M1 base +
`2 witnesses / 1 name`をacceptし、56-node arenaと両witness/name linkを
verifyしてboth halvesをatomicにpublishする。B3/B3N bytesは不変。repeated
install、cross-profile halves、reference/Task-248/257/other-258 familyの
both order、semantic coexistenceはpartial ownershipなしでrollbackする。

## Task 258B3M1 implementation result

existing paired installerはexact authenticated B3M1 base +
`2 witnesses / 1 name`だけをrecognizeする。both halvesのpublish前に
6-term lower profile、56-node arena、statement/primary fingerprints、
dense ordinals、name linkをrevalidateする。全cross-family/repeated orderは
partial ownershipなしで`InvalidSourceStatement`を返す。

## Task 258B3M2A planned paired ownership

existing paired installerだけをpublication pathとする。exact B3M2A base +
`1 witness / 0 names`だけをacceptし、49-node arena、Task-252 terms 5件 /
references 4件 / numeric request 0、両equality exclusion、fingerprints、
source order `[0,1,2]`をauthenticateしてboth halvesをatomicにpublishする。
B3/B3N/B3M1 bytesは不変。standalone/repeated install、profile hybrid、
reference/numeric-request corruption、Task-248/257/other-258 familyのboth
order、semantic coexistenceはpartial ownershipなしでrollbackする。

## Task 258B3M2A implementation result

existing paired installerはexact B3M2A baseと`1 witness / 0 names`
transactionをacceptし、both tablesをatomicにpublishする。standalone、
repeated、cross-profile、Task-248/257/other-258、corrupted dependency、
reverse-order attemptはpartial ownershipなしでfailする。public typed-AST
method、field、debug grammarは変更していない。

## Task 258B3M2B1 frozen typed ownership

existing paired installerはone exact B3M2B1 base + `1 witness / 0 names`
transactionに十分。53-node arena、five-root/six-primary、parenthesized term
2 / child 3、child-only referenceをrevalidateしてatomic publishする。
Task-252 terms 6件、references 5件、outer/inner parent edge、both equality
exclusions、fingerprints、source order `[0,1,2]`をauthenticateし、witness
0はouter term 2、reference 2はinner term 3をtargetする。standalone、
repeated、prior-profile、B3M2A、Tasks248/253–257/other-258、corrupted
dependency、reference/parent corruption、semantic coexistence、reverse
orderはpartial ownershipなしでfail。public method/field/enum/debug
grammarを追加しない。

## Task 258B3M2B1 implementation result

existing paired installerはexact B3M2B1 baseと`1 witness / 0 names`をaccept
し、両tablesをatomic publishする。53-node arena、five-root/six-primary
map、parenthesized wrapper/child edge、five references、both equality
exclusions、fingerprints、`[0,1,2]` source orderをrevalidateする。
standalone、repeated、cross-profile、Tasks 253–255 occupied/reverse-order、
corrupted dependency、semantic-coexisting attemptはpartial ownershipなしで
failする。public typed-AST API/debug grammarは不変。

## Task 258B3M2B2A frozen typed ownership

future paired installerはauthenticated B3M2B2A base + `1 witness / 0
names`だけをacceptできる。57-node arena、five-root/seven-primary、
parent chain `2 -> 3 -> 4`、five refs、Task-256からthree witness-subtree
termsのexclusion、fingerprints、source order `[0,1,2]`をatomic publish前に
revalidateする。standalone、hybrid、repeated、stale、cross-family、
reverse-order、semantic-coexisting stateはreject。prerequisiteではpublic
typed-AST API/debug grammarを変更しない。

## Task 258B3M2B2A implementation result

paired installerはexact B3M2B2A base/witness profileをacceptし、
dependencies、both wrapper links、five refs、complete Task-256 subtree
exclusion、fingerprints、source order `[0,1,2]`をatomic publication前に
revalidateする。standalone、hybrid、repeated、stale、cross-family、
reversed-order、semantic-coexisting statesはreject。public typed-AST
API/debug grammar changeなし。

## Task 258B3M2B2B1A atomic typed ownership result

`TypedAst::with_source_application_statement_witnesses`だけがB1A publish
pathである。exact authenticated Task-253 application、Task-258 base
statement、unnamed `Application(0)` witness 1件をone three-handoff
transactionとして受ける。全63-node arena、Task-252 `6/4/2`、Task-253
`1/0/1/2/2`、Task-256 equality-only exclusion、resolver-owner
fingerprints、statement/witness source order、witness-to-application
fingerprintを全table publish前にrevalidateする。

既存のstandalone Task-253 applicationは引き続きvalidである。
application-first stateにseparate B1A statement-witness installerを続ける
場合、application + statementのみ、application + witnessのみ、
statement-first、witness-only、hybrid、stale、substituted、repeat、
reverse-order、Tasks-253/254/255 coexistence、semantic coexistenceのB1A
publication attemptは全てoriginal `TypedAst`を変更せずfailする。legacy
application-free statement profile/debug bytesは維持する。successful B1A
installはtype、expression-semantic、proof、goal ownershipを追加しない。

## Task 258B3M2B2B1B1 frozen atomic typed ownership

existing `TypedAst::with_source_application_statement_witnesses` entry
pointはB1A/B1B1をtwo exact profilesとしてenumerateする。B1B1は67-node
wrapped profileで、Task-252 `6/4/2`、Task-253 `1/1/1/2/2`、Task-256
equality-only edges `[0,1]` / `[4,5]`、base statement `1/2/2/2/2`、
one unnamed `Application(0)` witness/no names。wrapper 0はauthenticated
Task-253 containmentでwitness targetではない。

installerはthree handoffs publish前にcomplete source/module/arena
identity、local theorem/imported application provenance、lower
fingerprints 2件、base/witness rows、wrapper-to-application containment、
witness-to-application edgeをrevalidateする。B1Aはbyte-identical
API/debug behaviorを持つseparate 63-node unwrapped profileのままで、
一方をbroadenして他方を推定しない。

application + statement/witness片方だけ、orphan statement/witness pair、
B1A/B1B1 hybrids、wrapper/application substitution、stale fingerprints、
partial/reverse/repeated installation、別Task-258 family、Tasks-254/255
coexistence、semantic coexistenceはoriginal `TypedAst`不変でrejectする。
new public typed-AST API/debug grammar/type/semantic/proof/goal ownerは
authorizeしない。

## Task 258B3M2B2B1B1 typed installation result

typed installationはB1B1をB1Aの内部ではなく隣にenumerateし、exact
application/statement/witness bundleをatomically publishする。frozen
partial、hybrid、substituted、stale、reverse、repeated、coexistence
failuresはoriginal ASTを不変に保つ。`typed_ast.rs`は4,743 lines。
public installer/semantic/type ownerは変更していない。

## Task 258B3M2B2B2A frozen atomic installer

`with_source_structure_statement_witnesses`だけがこのprofileのfuture
atomic entry point。already-installed Task-252/256 dataに対してexact
structure/statement/witness transactionsをvalidateし、Task 256をdirect
Structure target/fingerprintなしのまま`Some(&structure)`でrevalidateして
threeをtogether publishする。existing `with_source_structure`はauthorized
pre-statement structure/atomic coexistenceを維持するが、
`with_source_statement_witnesses`とともにexact B2A statement bundleを
partially assembleできない。application installersはstructure targetを、
new installerはapplication/legacy targetsをrejectする。全failureは
original ASTをunchangedに保つ。

## Task 258B3M2B2B2A atomic installation result

`with_source_structure_statement_witnesses`はauthenticated B2A
structure/statement/witness tripleだけをatomically installする。mutation前に
Task-252/254/256と全statement/witness fingerprintsをrevalidateし、Task 256は
direct structure target/fingerprintなしの`Some(&structure)`でvalidate。
application/legacy/partial/repeated/stale/reverse/family-coexisting bundleは
original AST unchangedでrejectする。

`typed_ast.rs`は4,829 lines。existing installersはcompatibleで、active
route、semantic/type/proof owner、coverage creditは追加していない。

## Task 258B3M2B2B2B frozen atomic sibling

existing combined structure-statement installerはB2A/B2Bをexact siblings
2種としてenumerateし、genericな
`application = None`/`structure = Some` statement bundleをadmitしない。
B2Aは上でfreeze済みのconstructor-witness bundle。B2Bは79-node
selector bundleで、Task-254 terms `0/1`、witness target
`Structure(0)`、selector base `Structure(1)`を持つ。exact Task-252 roots、
Task-254 rows、Task-256 equality rows、Task-258 base rows、全fingerprintsを
revalidateしてからstructure/statement/witness tablesをtogether
publishする。

Task 256は`BuiltinPredicateApplication` nodes `51/70`をownする。
nodes `52/71`はunowned formula containersで、ownership mapへsubstitute
できない。B2A/B2B row/target/ownership/fingerprint hybridはoriginal AST
unchangedでrejectする。このprerequisiteはpublic API、installer、debug
surface、active route、semantic tableを追加しない。

## Task 258B3M2B2B2B atomic installation result

`with_source_structure_statement_witnesses`はB2BをB2A besideにenumerateし、
exact authenticated 79-node selector bundleだけをacceptする。publication
前にTask-252/254/256、Task-258 base/witness rows、全fingerprints、
witness target `Structure(0)`、selector base `Structure(1)`、Task-256
ownership `51/70`、unowned containers `52/71`をrevalidateする。

generic structure admission、B2A/B2B hybrid、stale/swapped
fingerprint/target、application coexistence、partial/reverse/repeated
installationはoriginal AST unchanged。`typed_ast.rs`は4,830 lines。
public installer、debug surface、semantic/type/proof/goal owner、corpus
active route、coverage creditは変更していない。

## Task 258B3M2B2B2C frozen atomic sibling

`with_source_structure_statement_witnesses`がsole atomic entry pointのまま。
full exact source/arena/provenance/profileでB2CをB2A/B2B besideにenumerate。
publication前にTask252 `7/4/3`、Task254 `2/0/1/3/1/4/9`、Task256
`Primary(0/1)`/`Primary(5/6)` equalities、Task258 base `1/2/2/2/2`、
witness `1/0`、structure fingerprint、target `Structure(0)`をrevalidate。

all checks後だけ3 handoffsをtogether publish。B2A/B2B/B2C row/ownership/
fingerprint/target hybrids、application coexistence、stale/reverse/partial/
repeated、container substitutionはoriginal AST unchangedでreject。
public installer/debug schema/type/semantic/proof/goal/fixture/trace/active
root dispatch追加なし。implementation/atomicity testsはopen。

## Task 258B3M2B2B2C implemented atomic sibling

`with_source_structure_statement_witnesses`はB2A/B2Bと並ぶexact B2C profileを
admitし、existing structure-aware validation/publication pathをreuseする。
common option shapeだけではselectせず、source/arena/lower tables/ownership/
fingerprint/statement/witnessを全て先にrevalidateする。failure時original
TypedAstはunchanged。

frozen atomicity、hybrid/order、replay、rollback matricesはPASS。public
installer/field/schema/semantic payload追加なし。final source/docs/quality
reviewsはpending。

## Task 258B3M2B2B2C broad atomic-install verification

broad fmt/Clippy/crate/workspace gates、focused `4/4`/`5/5`、sibling
`12/12`/`21/21` suitesはunchanged counts/hashesでPASS。atomic publication/
rollbackはimplemented private siblingに限定され、public/semantic expansion
なし。independent final source/docs/quality reviews、commit、post-commit
inventoryはpending。

## Task 258B3M2B2B2C final atomic-install review status

independent final source/docs consistency/final qualityは**NO FINDINGS**。
全9 hard gates PASS、valid `98/100`。atomicity evidence/counts/hashes/
public/semantic boundariesはunchanged。pendingはcached-diff/staging audit、
implementation commit、post-commit inventory/fresh-next-task gatesだけ。

## Task 258B3M2B2B3A frozen Typed-AST installer

`TypedAst`はexactly以下だけを追加：

```rust
pub fn with_source_set_term_statement_witnesses(
    mut self,
    set_terms: SourceSetTermHandoff,
    statements: SourceStatementHandoff,
    witnesses: SourceStatementWitnessHandoff,
) -> Result<Self, TypedAstError>;
```

set-term table、Task258 base owners/contexts、B3A witness1をpublication前に
atomically authenticate。tupleはapplication `None`、structure `None`、
set `Some`。legacy `None/None/None`、application `Some/None/None`、
structure `None/Some/None`はexisting installerだけでaccepted、
他のtupleとhybrid/reorder/duplicate/stale dependency/partition violation/
invalid targetは`TypedAstError::InvalidSourceStatement`でimmediate replay
可能にfail closed。error variant/displayは追加・変更せず、lower handoff
`SourceStatementWitnessError`はこのpublic layerからescapeしない。

installerはwitness1/names0とsole witness-to-`SetTerm(0)`だけpublish。
existing installers/family composition/literal debug/semantics/routes不変。
final clone revalidationは`ResolvedTypedAst` ownershipで、set-term producer
editなし。

## Task 258B3M2B2B3A implemented typed installation closure

`with_source_set_term_statement_witnesses`はpublish前にexact empty
semantic/competing-family precondition、Task-255 set handoff、set-aware
Task-256 atomic handoff、Task-258 statement profile、B3A witnessをvalidateし、
その後set/statement/witnessをatomically publishする。全mutation/family-order
failureはprior `TypedAst`をunchangedに保ちexact replay可能で、errorはfrozen
`InvalidSourceStatement` boundaryへmapする。2回目のsource/documentation
consistency repeatとfinal documentation/boundary rereadは
**NO FINDINGS**で、crate plans記載のparent final verificationはexact
`39`-file scopeを含めPASS。independent final read-only quality reviewは
**NO FINDINGS**。全9 hard gates PASS、score capなし、valid `98/100`
（`20/20/15/14/10/10/5/4`）。記載済みsemantic/coverage deferralsは
unchanged residual risk。pendingはdedicated implementation commit、
postcommit invariant verification、fresh next-task inventoryだけ。

## Task 258B3M2B2B3B frozen atomic installation boundary

B3Bはexact 118-byte empty-enumeration profileに
`with_source_set_term_statement_witnesses`をreuseする。typed
installationはauthenticated Task-252 references、zero-edge Task-255
handoff、Task-256 formula rows、Task-258 base、unnamed SetTerm witness
1件だけをatomically publishできる。lower-stage failure precedenceと全
legacy/application/structure/B3A tuplesはunchangedのままである。partial
state、stale fingerprints、family hybrids、nonempty semantic/proof/goal
tableはいずれもfail closedする。public API/error variantは追加しない。

## Task 258B3M2B2B3B implemented atomic installation boundary

typed installationはauthenticated base rows、zero-edge Task-255 handoff、
set-only fingerprint、unnamed witness 1件だけをatomicにpublishする。
partial state、resolver mutations、stale fingerprint、B3A/B3Bおよび他
family hybridsをfrozen matrixでrejectする。B3A public SetTerm-aware APIを
reuseし、new public schema/error/debug grammarまたはsemantic stateは
追加しない。post-auth injectionとstage-prefix/non-generic-guard
assertionsもatomic failure/replayをpreserveし、全test-sufficiency repeatsと
final implementation repeatは**NO FINDINGS**である。

## Task 258B3M2B2B3C frozen atomic installation

B3Cはexisting `build_with_set_term`/
`validate_installation_with_set_term` pathでTask-48/252/255/256/258と
SetTerm witness 1件をatomic installする。choice target/type-site/request
authenticationはwitness publicationより先で、mutation/stale replay/family
hybrid failure後もtyped ASTはunchanged。このprerequisiteはtyped-AST
source/public schema/semantic table/debug bytesを変更しない。

## Task 258B3M2B2B3C implemented typed-AST installation

`TypedAst`はauthenticated B3C source-set/statement/witness bundleだけを
existing atomic transactionでinstallする。exact choice set fingerprintを
preserveし、complete dependency tupleを再validateし、stale/reordered/
hybrid/generic-guard stateでrollbackする。B3A/B3Bは両family orderで
independently installableのまま。public schema/error text/debug grammar/
dependency/semantic tableは変更しない。private dormant runner selectorは
このtyped/final ownerの外にあり、existing active-corpus outcomeを変更しない。

## Task 258B3M2B2B3D frozen atomic installation

future installerはexact Task-255 qua handoff、statement base、unnamed
SetTerm witness 1件をatomically combineし、existing set-only
fingerprint tupleをvalidateして、stale/reordered/hybrid/generic-guard
stateではrollbackする。B3A/B3B/B3Cは全family orderでindependently
installableのままである。public schema、error text、debug grammar、
dependency、semantic table、active routeは変更しない。

## Task 258B3M2B2B3D implemented atomic installation

typed installerはauthenticated Task-255 qua handoff、Task-258 base、
unnamed `SetTerm(0)` witnessをexisting set-only fingerprint APIでatomically
installする。stale/reordered/hybrid/generic-guard stateではpartial
publicationなしにrollbackし、B3A/B3B/B3C/B3Dの全family ordersを
independentにpreserveする。typed moduleは`4933` lines。public schema、
error text、debug grammar、dependency、active route、semantic tablesは
unchangedで、typed/final test matrixはPASSする。independent implementation
reviewは**NO FINDINGS**。bounded wording/status remediation後のsource/docs
consistencyとboundary repeatも**NO FINDINGS**、full final verificationも
PASS。

independent final read-only quality reviewは**NO FINDINGS**、全9 hard
gates PASS、score capなし、valid `100/100`
（`20/20/15/15/10/10/5/5`）。CLI `23/0` warnings/errorsとlarge
repeated-test diff review volumeはnonblocking residual。staging/cached
diff、commit、post-commit/fresh-nextだけがpending。

## Task 258B3M2B2B3E frozen atomic installation

future `TypedAst` installはexact B3E tupleをone atomic transactionとして
acceptする。authenticated inputは139-byte/60-node source fingerprint、
Task-48 `2/1/0`、Task-252 `5/4/1`、empty Tasks 253/254、
Task-255 `1/0/1/1/0/1/2`、Task-256
`2/0/0/0/0/0/0/4/4`、Task-258 `1/2/2/2/2`、one unnamed
SetTerm witnessである。

installationはTask-252 `{32,34,38,47,49}`、Task-255
`{16,40,41,43}`、Task-256 `{36,51}`、Task-258 `{54,56}`、B3E
`{45,46}`のowner partitionと、
`ComprehensionMapper -> Primary(2)`および
`Witness(0) -> SetTerm(0)`をrevalidateする。generator segment `42`は
unownedであり、binding/reference/condition rowをfabricateしない。

wrong generator/type-site/condition/edge/request、reordered
`GeneratorSethood`/`ResultType`、stale source/lower fingerprints、
partial family state、hybrid/non-generic inputはpublication前にrollback
する。B3A/B3B/B3C/B3D/B3Eの全`120` ordersをindependent exact familiesと
して保持する。semantic tablesはemptyで、generator capture、typing、
goal/proof semanticsをinstallしない。future checker implementationは
existing set-aware installerをreuseし、public schema/error/debug
grammarを変えない。documentation-only atomic-boundary reviewは
**NO FINDINGS**である。future implementation/atomicity reviewはseparate
implementation taskに残す。

## Task 258B3M2B2B3E implemented atomic installation inventory

`TypedAst`はbinding/primary/shared-arena/set/atomic/statement/witness
fingerprintが全てmatchしたexact B3E tupleだけをinstallする。condition-free
comprehension 1、generator/type site 1、condition 0、unnamed set witness 1を
atomicにpublishし、partial/extra/stale/hybridではoriginal ASTを保持する。

private allowlistでownerは4,933から4,934 linesへ増える。全five familiesは
120 ordersでindependent。public typed-AST API、semantic/proof state、
diagnostic/debug grammarはunchangedである。
final source/docs consistencyとindependent qualityは**NO FINDINGS**、
full verificationと全9 gatesはvalid `100/100`でPASSした。staging/
post-commit gatesはimplementation commit
`e4479691db3b0a8785bb16e94d386bd71a394274`でcloseし、fresh inventoryは
Task 258B4Aをselectした。

## Task 258B4A frozen paired installation

new installation pathはexactに
`with_source_formula_composition_statement(mut self, composite:
SourceCompositeFormulaHandoff, composition:
SourceFormulaCompositionHandoff, statement: SourceStatementHandoff) ->
Result<Self, TypedAstError>`である。この順でexact Task-257B1
composite/formula-composition handoffとB4A statementをconsumeし、全lower
fingerprintとstatementのoptional composite/composition fingerprintを
revalidateして、3件すべてをatomically publishする。existing lower-only
`with_source_formula_composition` behaviorとdebug bytesはunchanged。

transactionはprivate double-LF routeとexact `Composite(0)`
statement/candidate pairだけをacceptする。atomic Task-258 statement
families、duplicate owner、stale arena、reordered/hybrid lower handoff、
partial tupleは`TypedAstError::InvalidSourceStatement`でfailし、stateを
byte-identicalに保ってreplayを許す。semantic tableとlower root-ownership
rowは変更しない。
repeated read-only documentation reviewは**NO FINDINGS**である。
independent final qualityは全9 hard gatesをcapなし、valid `100/100`で
PASSした。remainingはstaging、commit、post-commit inventoryだけである。

## Task 258B4A implemented paired installation

`with_source_formula_composition_statement`はexact Task-257B1
composite/composition pairとTask-258 B4A statementをone transactionとして
revalidate/installする。mutation前に両optional statement fingerprints、
`Composite(0)` links、source identity、lower fingerprints、table profiles、
family exclusivityが一致しなければならない。atomic-statement families、
duplicate owners、stale/reordered/partial handoffs、rooted/relocated lower
near misses、cross-family hybridsは`InvalidSourceStatement`を返し、ASTを
byte-identicalに保ってclean replayを許す。lower-only installerと全semantic
tableはunchangedである。

## Task 258B4B frozen paired installation

`with_source_formula_composition_statement`はpublic signatureをreuseする。
exact Task-257B2 composite/composition handoffs、B4B statement handoff、
Task-252/256 state、source/module identity、fingerprints、rootless 124-node
arena、empty incompatible familiesがすべて一致するときだけB4Bをpublish
できる。B4AはTask-257B1 statement profileだけのままである。existing B4A
crate-private predicateはshared cardinalitiesからexact B4A identityへ先に
narrowしなければならず、new exact B4B predicateとinterchangeableではない。

installerは全B4A/B4B pairing hybrid、duplicate/partial state、active
atomic statement family、Task-248 context、semantic table、rooted arena、
relocated owned site、stale fingerprintをmutation前にrejectする。全
installation ordersはatomic/replayableである。lower-only
`with_source_formula_composition`はunchangedのままで、B1、B2、B3をlower
transportとしてだけ引き続きacceptする。

## Task 258B4C frozen paired installation

`with_source_formula_composition_statement`はpublic signatureを維持し、
matched Task-257B3 composite/composition handoffとexact B4C statementに
限りB4Cをinstallできる。raw source、Surface、resolverのauthenticationは
runner selectorと`SourceStatementProducer`が所有する。installerは
mutation前に、その結果得られたproducer-authenticated statement
owner/context/candidate rowsとhandoff identity、matched lower
fingerprints、rootless 66-node lower arena、binding `4/4/0`、primary
`6/6/0`、atomic `3/0/0/0/0/0/0/6/6`、composite
`3/0/1/3/3/2/6`、composition `3/6`、24-site lower ownership
partition、upper `1/1/1/0/1`をrevalidateする。

statement/candidateは両方`Composite(0)`をtargetにし、statement context 0は
reserved binding `[0]`だけをexposeし、input factsはemptyでなければならない。
installerはB1/B4A、B2/B4B、B3/B4Cだけをrecognizeする。cross-pairing、
duplicate/partial state、stale fingerprint、rooted/relocated arena、
altered ownership、active atomic statement family、semantic-table
coexistence、lower-selector mismatchはexisting
`TypedAstError::InvalidSourceStatement`でfailし、ASTをbyte-identicalに
保ってreplayを許す。

mandatory lower-selector compatibility prerequisiteはinstaller変更前の
separate logical task/commitである。B4C transactionはpublic installer、
error variant、debug grammar、fact、theorem acceptance、proof、semantic
tableを追加せず、lower-only installerはunchangedである。

## Task 258B4C 実装済み Paired Installation

既存 paired installer は exact B3/B4C transaction のみを admit する。
binding `4/4/0`、primary `6/6/0`、atomic `3/0/0/0/0/0/0/6/6`、
composite `3/0/1/3/3/2/6`、composition `3/6`、upper `1/1/1/0/1`、
全 fingerprint、rootless 66-node arena を再検証する。全 anchor と recovery
state は exact で、24 node は lower-owned、statement-owned は theorem node
62 のみ、41 node は unowned のままである。

cross-pairing、duplicate/partial または atomic state、Task-248 occupancy、
stale fingerprint、rooted/relocated arena、altered ownership は mutation
前に既存 error で fail し、deterministic replay を許す。public installer
と semantic table は追加しない。

## Task 258B5A frozen paired installation

existing reference installerはprivateにgeneralizeし、unchanged B1
same-scope profileとB5A ancestor/descendant profileのexact two
authenticated transactionだけをacceptできる。B5Aはexact base
`1/5/5/5/5`、reference `1/1`、lower handoff、全fingerprint、
93-node/root-92 arena、20-owned/73-unowned partitionを要求する。

label/citationはscope `[0]`/`[0,1]`、statement ordinal 1/4、
private/local-only contribution 0、exact range/origin、node 82のmatching
resolver keyを維持する。cross-pair、partial/duplicate install、stale replay、
relocation/recovery、wrong scope/provenance、Task-248/other-family
occupancy、altered ownershipはmutation前にfailする。semantic tableを追加せず、
B1 debug output/public signatureを変更しない。

## Task 258B5A implemented paired installation

private paired installerはunchanged B1 same-scope transactionまたはexact B5A
ancestor/descendant transactionだけをadmitする。B5A installationはbase
`1/5/5/5/5`、reference `1/1`、全lower handoff/fingerprint、全93-node arena
identity、`20/73` ownership partition、label scope `[0]`、citation scope
`[0,1]`、statement ordinal `1/4`、contribution 0、resolver key node 82を
再検証する。

duplicate/partial state、B1/B5A cross-pair、stale fingerprint、
relocation/recovery、wrong range/origin/scope/contribution/key、Task-248/
other family、altered ownershipはmutation前にfailする。failed installation
後もtyped ASTはreplay可能。public installer、error variant、semantic table、
B1 debug changeを追加しない。

## Task 258B5B frozen imported-target installation

separate lower-stage opt-in label prerequisite後、paired installationはthird
exact reference profileとしてB5B base `1/2/2/2/2`、local labels/citations
`0/1`、57-node/root-56 arena、`8/49` ownershipをadmitする。citation targetは
`SourceStatementCitationTarget::Imported`、kindは`SimpleImported`、singular
projectionはimported/public/exported theorem `Ref`。

mandatory local `SourceStatementLabelId`はresolverにないrowをfabricateする
ためpublic target enumが必要。existing B1/B5A citationはdebug bytes/behaviorを
変えず`Local(id)`になる。installerはB1/B5A/B5B base/reference fingerprintを
exact matchし、B5A-localとB5B-imported stateを含む全cross-pairをrejectする。

duplicate/partial installation、absent/extra local label、wrong import
visibility/export/kind/module/namespace/contribution/origin/anchor/range/
path、recovered/relocated row、wrong node 48 key、Task-248/other-family
occupancy、altered ownershipはmutation前にfailしreplay可能なままにする。
semantic table/public runner-facing schemaを追加しない。

## Task 258B5B implemented imported-target installation

typed installationはmutually exclusiveなB1 local、B5A local、B5B imported
のexact three reference profilesをadmitする。B5Bはfrozen
57-node/root-56 resolver fingerprint、Task-258 `1/2/2/2/2 + 0/1`、
`8/49` ownership、local label 0、one
`SourceStatementCitationTarget::Imported`、one `SimpleImported` citationを
要求する。B1/B5Aは`Local(id)`をconstructしprior bytesをpreserveする。

installerはmutation前に全base/reference fingerprint、resolver node kind/
key、import/projection/reference provenance、citation row、owned-node
partitionを再検証する。全B1/B5A/B5B cross-pair、partial/duplicate
installation、occupied semantic stateをatomically reject。public installer、
semantic table、runner-facing schema、diagnosticを追加しない。

## Task 258B5C frozen typed-installation exclusion

B5Cはunresolved resolver resultで終了し、source-statement reference
handoffを満たさない。そのため`TypedAst`はB5C base/reference profile、
label/citation row、binding context、owned Surface node、checked formula、
fact、proof、semantic tableをinstallしない。unresolved resultをB1/B5A/B5B
near matchとしてinterpretしてはならない。

later active declaration-symbol runner taskはvalidated R-032A/B outputを
直接consumeする。
existing B1/B5A/B5B installation predicates、mutation atomicity/replay、
debug bytes、public installers、error variantsはunchanged。

R-032B closed edge tableはexact
`Root -> CompilationUnit -> ItemList -> direct TheoremItem -> direct
ProofBlock`、Root/CompilationUnit exact-one normal structural child、
direct-normal theorem scanから始まる。no-ordinal/no-descent default denyに
より、全excluded/relocated/mixed formはtyped installationへ到達しない。
positive edge/negative relocation/mixed-list testはlower-stage testのまま。

runnerのindependent env/projection/contribution provenance mutationはすべて
`proof_scope_input`で停止し、structurally coherentなmutationでも
confinement/`TypedAst` rowになれない。source bytes+exact normal ASTだけが
selectorで、48-file scopeはunchanged。

## Task 259 Frozen Typed-AST Transaction

future Task-259 projectionはauthenticated baseline
`InitialObligationTable`のclone、`SourcePredicateDefinitionHandoff`、
そのcloneをpreserveしてexactly one `PredicatePropertyCorrectness` rowを
appendしたcompleted tableを含む。handoffはsource/module identityと
`SourceBindingContextHandoff`、`SourceTypeApplicationHandoff`、
`SourcePrimaryTermHandoff`、`SourceAtomicFormulaHandoff`のexact debug
fingerprintをstoreする。

`TypedAst::with_source_predicate_definition`はone-shotでhandoffとobligation
tableをatomically publishする。four lower handoffをrequireし、全fingerprint、
dense target、source site/range/context、predicate resolver identity、
correctness link、obligation owner/kind/range/assumptions/goal/provenance/
statusをrevalidateし、partial/stale transactionをすべてrejectする。さらに
current obligation tableがretained baselineとexactly equalであることもrequire
し、不一致ならwhole projectionをrejectする。failureはTask-259 rowも
obligation linkageも残さない。`TypedAstParts`はTask-259 fieldを追加せず
alternate install pathにならない。`ResolvedTypedAst`は同じhandoff、
obligation、ID、order、fingerprint、debug bytesをreconstructせず
revalidate/clone-preserveする。

frozen exact transactionのtable cardinalityは`1/2/1/1/1`で、
available-for-handoff `Pending` obligationはexactly oneである。type fact、
coercion、diagnostic、`VcId`、proof status、accepted result、axiom、IR node
は追加しない。guardはobligation assumptionではなくsource-formula linkの
ままである。future semantic consumerはここで凍結したopaque stringからFOL
goalをinferしてはならない。

## Task 248 Two-Parameter Profile-B Typed Installation

lower implementationはinstallerを追加しない。existing
`TypedAstParts::source_context` pathでlocal contextをinstallする既存projectionを
返す。projection前にprivate runnerはfour caller siteをone shared
`TypedArena`に対してvalidateする。module siteはcontext 0のroot、
definition/two declaration siteはexact range/context 1、全nodeはnormalでsiteは
distinctである。

existing typed validationはanchor、context、root ownership、item/declaration
site、linkを独立にrecheckする。stale/duplicate siteはpartial handoffなしでfailする。
Profile A installation/debug/recoveryと全type/fact/coercion/obligation/diagnostic
tableはbyte-compatibleのまま。Task-259 installationはlater separate
transactionである。

## Task 260 Typed Functor-Definition Installation

`TypedAst`はoptional `SourceFunctorDefinitionHandoff` 1件を追加します。one-shot
installerは全lower handoff/fingerprintがexactで、initial-obligation tableがcaller
baseline plus means rows 2件の場合だけproducer projectionをacceptし、handoffと
complete obligation tableをatomic installします。

Task-259/260はseparate optionalですがTask 260ではmutually exclusiveです。
Task-260 installerはcurrent Task-259 handoffまたは
`PredicatePropertyCorrectness` baselineをrejectし、final assemblyも両handoffを
togetherにrejectします。cross-family install-order promiseもTask-259
compatibility editもありません。Task-260 pathはTask-259 stateをreplaceせず
fact/type/coercion/diagnostic/proof/acceptanceを作りません。

producer/installerはpre-existing `FunctorExistence` / `FunctorUniqueness`
baselineもrejectします。handoffありではlinked final row 2件だけを許可し、
handoffなしではどちらのkindもorphanとしてfinal assemblyがrejectします。

## Task 249R definition-return ownership addendum

Task 249Rは`TypedAstParts` field/install methodを追加しない。existing
`SourceTypeApplicationHandoff`を`TypedAst::try_new`前にextendし、
`validate_source_type`がdefinition ownerとappended return expressionをowned arenaへ
再照合する。同じoptional `source_type` fieldだけがownerである。empty-return
legacy debug byteは不変で、Task-260 profileはcombined `2/4/0/2` handoffを持つ。

## Task 249M mode-RHS ownership addendum

Task 249Mは`TypedAstParts` field/installerを追加せず、existing source-type
handoffを`TypedAst::try_new`前にextendする。source-type validationはexact owner/
appended expression/head、Task 249R mutual exclusion、arena identityをrecheckする。
same optional fieldがsole ownerで、legacy/Task-249R debug byteは不変、Task-262
lower profileはsemantic outputなしのcombined `2/3/0/0/1` handoffである。

## Task 249M active typed ownership

standalone mode-RHS extensionはinstallation前に実装済みである。
`TypedAst::try_new`はexisting optional handoffでrowとsource-type expression 3件を
revalidateし、new field/installerを追加しない。exact testはtype/fact/
coercion/obligation/diagnostic tableをemptyに保つ。

## Task 262 active mode-definition transaction

`TypedAst`は`with_source_mode_definition`だけからinstallされるoptional
`SourceModeDefinitionHandoff` 1件をownする。installerはcommitted Task-248 source
contextとcombined Task-249/249M source-type handoffをrequireし、producerがretain
したbaselineをcurrent obligation tableと比較して、six-table handoffとexact
one-row `Sethood` suffixをatomicにpublishする。`TypedAstParts`はunchangedで、
alternate install pathではない。

transactionはprior Task-259/260/261/262 owner、sibling-only obligation kind、
stale lower fingerprint、`source.definition.mode` domainのorphan goal/provenance
をrejectする。unrelated baseline `Sethood` rowはbyte-preserveする。pending rowと
unresolved RHS-inhabitation requestはgoal/guard composition、proof、discharge、
acceptance、fact、IR、VC semanticsを与えない。
## Task 249S standalone member-type ownership addendum

Task 249Sは`TypedAstParts` field/install pathを追加しない。existing optional
`source_type: Option<SourceTypeApplicationHandoff>`がexact standalone
`0/4/0/0/0/4` valueを所有する。installationはmember owner/expression/head site
4件を`TypedArena`へ再照合し、欠落/corruptionは`InvalidSourceType`でfailする。
type/fact/coercion/initial obligation/diagnostic/contextと全Task-263 upper fieldは
empty/absentのまま。

## Task 249S active Typed ownership result

existing optional `source_type` fieldとone-shot installation pathがsole
ownerである。install時にexact member/expression table、source/module identity、
arena site 12件、sibling profileとのmutual exclusionをrevalidateする。
semantic tableとTask-263 upper fieldは全てempty/absentのままである。

## Task 263 frozen Typed ownership

future `with_source_structure_definition`はone-shot compare-and-swap transactionで
ある。exact Task-249S source-type fingerprint、unchanged baseline/final obligation
pair、valid `2/4/1/2/0` rows、empty derived coherence tableを認証してからoptional
handoffをpublishする。同名getterと
`TypedAstError::InvalidSourceStructureDefinition`だけを追加し、`TypedAstParts`に
replacement pathはない。

installはTasks 259--262 definition-family occupancyを両observable orderで拒否し、
types/facts/coercions/diagnostics/obligation rowを変更しない。Task 259 correctness
transactionとmixed predicate/functor boundaryはindependentのままである。

handoffはcomplete baseline tableをprivate clone-retainする。installはcurrent ==
projection baseline == private snapshot == projection finalを要求する。exact runner
baselineはempty、checker testsはnonempty unrelated baselineもpreserveする。
same-length snapshot corruptionはtransactional failureで、snapshot getter/stable-
debug serializationはない。

## Task 263 active typed ownership

`TypedAst::with_source_structure_definition`はprojectionをone-shot installし、baseline/
current/final obligation tableのbyte equalityを要求し、全frozen dependency/rowを
revalidateし、Task-259--262 definition-family ownerを両installation orderでrejectする。
failureはoriginal typed valueを変更しない。

## Task 248P property context typed ownership

Task 248Pはtyped owner field/installation methodを追加しない。complete Profile C
handoffはexisting one-shot source-context installationを使い、source/module/root、
local-context、item/declaration site、context-linkをrevalidateする。recovered incomplete
branchはinstall不能のまま。property payload/type result/fact/obligation/diagnostic tableを
作らず、Profile A/B debug/installation byteは不変である。

## Task 248P active typed ownership

complete Profile Cはexisting one-shot source-context fieldへinstallされ、deterministicに
replayする。exact item/declaration/binding/binding-context/local-context/context-link/
provenance rowをrevalidateし、frozen testでは全type/fact/diagnostic/initial-obligation table
がemptyである。typed owner field/installation APIは追加しない。

## Task 264 frozen typed ownership

TypedAstはprivate optional property handoff、getter、consuming one-shot
`with_source_property_implementation`を追加する。Projection baselineとcurrent
obligationsを照合し、Meansはproperty existence/uniqueness two rows、Equalsはzero
をhandoffとatomic installする。Replacement/half publication/Task259 coexistenceを
`InvalidSourcePropertyImplementation`でrejectする。TypedAstPartsへpublic fieldは
追加せず、facts/coercions/diagnostics/types/proof/acceptance/IRは不変である。

## Task 249PI typed ownership boundary

Task 249PIは`TypedAstParts` field/installer/getter/serializerを追加しない。exact combined
source-type handoffはexisting optional `source_type` owner/one-shot installationを使い、
installationがprofile shapeとparameter/member/expression/head arena identityを再validate
する。types/local context/facts/coercions/initial obligations/diagnostics/proof/acceptance/
IRは不変。Task264がseparate frozen handoffからlower fingerprint/member ID 1をconsumeする。
