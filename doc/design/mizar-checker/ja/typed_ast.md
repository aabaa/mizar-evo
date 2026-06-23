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
