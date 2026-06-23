# Resolved Typed AST

## 目的

`ResolvedTypedAst` は、elaboration の前に置かれる最終の checker-owned な
source-shaped semantic AST である。phase 6 の typed source shape、phase 7 の
fact/trace、phase 8 の overload selection を 1 つの immutable layer に射影し、
LSP、artifact、VC generation、elaboration が name resolution、type checking、
registration closure、overload resolution を再実行せずに消費できるようにする。

本書は task 28 の data shape を仕様化する。assembly、artifact emission、proof
acceptance、source-to-checker extraction は実装しない。

## 参照

- [architecture 01](../../architecture/ja/01.ir_layers.md) は
  `ResolvedTypedAst` を最終の source-shaped semantic AST と定義する。
- [architecture 05](../../architecture/ja/05.overload_resolution.md) Step 6 は
  overload から `ResolvedTypedAst` への境界を定義する。
- [typed_ast.md](./typed_ast.md) は source-shaped typed arena と partial typing
  model を定義する。
- [type_checker.md](./type_checker.md) は normalized type、coercion candidate、
  type fact、initial obligation を定義する。
- [cluster_trace.md](./cluster_trace.md) は replayable cluster/reduction trace
  material を定義する。
- [overload_resolution.md](./overload_resolution.md) は overload result、
  inserted view、failed-site preservation semantics を定義する。

## 責務

`ResolvedTypedAst` が所有するもの:

- source-shaped resolved node arena;
- 各 projected expression / overload site の final expression/type metadata;
- failed record を含む final overload resolution record;
- 後続 phase が観測しなければならない inserted `qua` / coercion view metadata;
- 各 projected expression で可視な final cluster/type fact;
- LSP と artifact に必要な diagnostic と recovery metadata;
- `TypedAst` node と source range へ戻る deterministic source map。

`ResolvedTypedAst` が所有しないもの:

- lower された logical clause または kernel term;
- VC 固有の local proof context;
- ATP premise または proof search result;
- artifact serialization schema または cache reader;
- source walking、resolver-shell parsing、missing checker payload の捏造。

## 入力

task 28 assembly は explicit checker-owned output を消費する。

- `TypedAst` node、status、local context、typed-site reference;
- phase 6 の final `TypeFactTable` / type-fact query output;
- accepted cluster closure fact と replayable resolution trace;
- site owner、source range、candidate table、graph id を提供する overload
  collection、viability、specificity graph output;
- inserted-view kind、reason、evidence、path を含む
  `OverloadSelectionOutput` の selected overload result と inserted view;
- 先行 phase が生成した checker-local diagnostic。

不足する source-derived input は `external_dependency_gap` record である。不足する
checker-owned precursor table は task 28 assembly blocker であり、raw syntax を scan する許可では
ない。assembly は raw syntax や opaque resolver shell を調べて、いずれの gap も補完してはならない。

## データ形状

public data layer は assembled output 内で local な dense id を保つべきである。

```rust
struct ResolvedTypedAst {
    source_id: SourceId,
    module_id: ModuleId,
    nodes: ResolvedTypedArena,
    expr_metadata: ExpressionMetadataTable,
    resolved_overloads: OverloadResolutionTable,
    inserted_coercions: CoercionInsertionTable,
    cluster_facts: ClusterFactTable,
    diagnostics: ResolvedTypedDiagnosticTable,
}
```

### Resolved Node

```rust
struct ResolvedTypedNode {
    id: ResolvedTypedNodeId,
    typed_node: TypedNodeId,
    source_range: SourceRange,
    kind: ResolvedTypedNodeKind,
    final_type: Option<NormalizedTypeId>,
    metadata: Option<ExpressionMetadataId>,
    diagnostics: Vec<ResolvedTypedDiagnosticId>,
    recovery: ResolvedNodeRecovery,
}

enum ResolvedTypedNodeKind {
    SourcePreserved { role: SourceNodeRole },
    ResolvedUse { symbol: SymbolId },
    FailedOverload { result: OverloadResolutionId },
    Degraded { reason: ResolvedNodeRecoveryReason },
}
```

arena は source shape を保持する。failed overload site は failed overload result id
を持つ node として残り、successful `ResolvedUse` に書き換えてはならない。

### Expression Metadata

```rust
struct ExpressionMetadata {
    id: ExpressionMetadataId,
    expr: ExprId,
    typed_site: TypedSiteRef,
    source_range: SourceRange,
    final_type: Option<NormalizedTypeId>,
    visible_facts: Vec<TypeFactId>,
    cluster_facts: Vec<ClusterFactId>,
    overload: Option<OverloadResolutionId>,
    inserted_views: Vec<CoercionInsertionId>,
    local_context: Option<LocalTypeContextId>,
    diagnostics: Vec<ResolvedTypedDiagnosticId>,
}
```

expression metadata は hover、`@show_resolution`、artifact summary、downstream
elaboration の安定 lookup surface である。先行 phase が生成した id を記録し、fact や
overload choice を再計算しない。

`ExprId` は expression metadata の source-file identity である。
`ExpressionMetadataTable` は deterministic な `ExprId` から `ExpressionMetadataId` への lookup
を提供しなければならない。`ExpressionMetadataId` はこの `ResolvedTypedAst` 内の dense row id
にすぎない。task 28 の test は `ExprId` lookup を assert し、table insertion order を expression
identity として扱ってはならない。

### Overload Resolution Table

```rust
struct OverloadResolutionRecord {
    id: OverloadResolutionId,
    site: OverloadSiteId,
    typed_site: TypedSiteRef,
    source_range: SourceRange,
    status: OverloadResolutionStatus,
    candidates: Vec<OverloadCandidateId>,
    specificity_graph: Option<SpecificityGraphId>,
    diagnostics: Vec<ResolvedTypedDiagnosticId>,
}

enum OverloadResolutionStatus {
    Resolved {
        root: OverloadCandidateId,
        active_refinements: Vec<OverloadCandidateId>,
        exposed_result: Option<ExposedResultPayload>,
        inserted_views: Vec<CoercionInsertionId>,
    },
    NoMatch { rejected: Vec<OverloadCandidateId> },
    Ambiguous { candidates: Vec<OverloadCandidateId> },
    IncompatibleRefinementJoin {
        root: OverloadCandidateId,
        refinements: Vec<OverloadCandidateId>,
        reason: RefinementJoinFailure,
    },
    Blocked { reason: OverloadBlockedReason },
}
```

resolved record は task 26 selection output の射影である。failed record は first-class
metadata であり、有効な elaboration input ではない。

### Coercion Insertion Table

```rust
struct CoercionInsertion {
    id: CoercionInsertionId,
    typed_site: TypedSiteRef,
    source_range: SourceRange,
    target: NormalizedTypeId,
    selected_candidate: Option<OverloadCandidateId>,
    source: CoercionInsertionSource,
    reason: InsertedViewReasonKey,
    evidence_facts: Vec<TypeFactId>,
    path: Option<QuaPathKey>,
}

enum CoercionInsertionSource {
    SourceQua,
    InsertedWidening,
}
```

この table は semantic view metadata を記録する。source edit list ではない。failed または
blocked overload site は inserted coercion を生成してはならない。`path` は source-`qua` と
inserted inheritance path の single source of truth である。top-level の `reason` は
source-written view と inserted view の両方について task 26 の inserted-view reason を保持する。
`source` enum は view が source-written か inserted かを記録し、path payload や reason payload を
重複して持ってはならない。

### Cluster Fact Table

`ResolvedTypedAst.cluster_facts` は checker-owned
`cluster_trace::ClusterFactTable` の row と provenance schema を再利用する。同じ名前の
別 row shape を定義してはならない。expression metadata は既存の `ClusterFactId` を参照してよく、
それらの id 上に deterministic な per-expression index を構築してよいが、fact fingerprint、
source/attribute/generated-type payload、`ClusterFactProvenance` は `cluster_trace` が所有する。

assembly は registration firing、cluster closure、reduction replay、cluster fact から新しい
`TypeFactId` への変換を行わない。artifact 向けの cluster fact projection が将来必要な場合は、
別の schema task とする。

## Failure And Recovery

recoverable failure は明示的に表現する。

- failed overload site は `OverloadResolutionStatus` record を保持する;
- partial / degraded typed node は元の typed status と diagnostic を保持する;
- missing external payload は diagnostic または failed record のまま残す;
- failed site を successful resolved use に変換してはならない;
- downstream elaboration は failed node を skip または degrade しなければならない。

## 決定性

assembly は deterministic でなければならない。

- id は canonical source order に従う dense id である;
- overload record は site/source order で sort する;
- inserted coercion は typed site、target、source、stable reason で sort する;
- reused `cluster_trace::ClusterFactTable` は自身の canonical ordering を保持する;
  per-expression cluster fact reference / index は owning `TypedSiteRef`、`ClusterFactId`、
  existing provenance で sort する;
- equivalent input は byte-identical debug rendering を生成する。

## task 28 の planned tests

task 28 は Rust coverage を追加すべきである。

- explicit `TypedAst` と checker output からの source-shaped assembly;
- `TypedSiteRef` / expression id による metadata lookup;
- active refinement と inserted view を含む resolved overload projection;
- `NoMatch`、`Ambiguous`、incompatible refinement join、blocked status の
  failed overload site preservation;
- failed overload site が inserted coercion record を生成しないこと;
- equivalent input ordering に対する deterministic debug rendering;
- cluster fact id reference と既存 cluster-trace provenance の保持。

## Deferred And External Gaps

task 27 後も以下は deferred のままである。

- task 28 の `ResolvedTypedAst` assembly の Rust implementation;
- task 26 selection payload と source expression metadata の AST-wide
  source-to-checker extraction;
- artifact emission/reuse と stable artifact schema;
- public diagnostic-code allocation;
- final overload / cluster projection の active `.miz` semantic fixture。

これらの gap は task 28 で fabricated success record、raw syntax scan、artifact-like
side output を許可しない。
