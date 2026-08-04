# Resolved Typed AST

> 正本は英語です。英語版:
> [../en/resolved_typed_ast.md](../en/resolved_typed_ast.md)。

## 目的

`ResolvedTypedAst` は、elaboration の前に置かれる最終の checker-owned な
source-shaped semantic AST である。phase 6 の typed source shape、phase 7 の
cluster fact row / provenance id、phase 8 の overload selection を 1 つの immutable
layer に射影し、LSP、artifact、VC generation、elaboration が name resolution、type
checking、registration closure、overload resolution を再実行せずに消費できるようにする。

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
- `TypedAst` が既に所有・validate した optional complete
  source/binding-context handoff;
- phase 6 の final `TypeFactTable` / type-fact query output;
- existing provenance id を持つ accepted cluster closure fact row;
- site owner、source range、filter 前と viable の candidate table、rejection / blocking reason、
  graph id を提供する overload collection、template expansion、viability、specificity graph output;
- inserted-view kind、reason、evidence、path を含む
  `OverloadSelectionOutput` の selected overload result と inserted view;
- 先行 phase が生成した checker-local diagnostic;
- stable source expression id を `TypedSiteRef` owner と計算済み cluster fact
  reference に対応付ける caller-supplied `ExpressionMetadataInput` row;
- checker table からまだ推論できない source-preserved / resolved-use / degraded
  node role のための optional `ResolvedNodeKindHint` row。

不足する source-derived input は `external_dependency_gap` record である。不足する
checker-owned precursor table は task 28 assembly blocker であり、raw syntax を scan する許可では
ない。assembly は raw syntax や opaque resolver shell を調べて、いずれの gap も補完してはならない。

Expression metadata input は dense id を割り当てる前に expression id で canonicalize
される。site-based lookup と resolved-node attachment を曖昧にしないため、duplicate
expression id または duplicate `TypedSiteRef` owner は assembly error である。
`ExpressionMetadataInput` row がない site は task 28 では expression metadata entry を持たない。
全 source expression id の AST-wide extraction は deferred の source-to-checker integration task
のままである。

現在の source-derived runner note: `mizar-test` type-elaboration runner は bounded
reserve-only bare-builtin declaration pass bridge のために real
`ExpressionMetadataInput` row を供給する。Reserve declaration node と binding 固有の
type-expression node は、declaration checking が成功した場合に final type を持つ
source-preserved `ResolvedTypedAst` node になる。same-module attributed builtin reserve
head と local-mode reserve head は active fail slice のみである。active runner は stable
diagnostic key を集めるために同じ assembly helper を使ってよいが、diagnostic-free
bare-builtin output だけを `ResolvedTypedAst` readiness として credit する。active runner は
さらに、successful bare-builtin の real `ResolvedTypedAst` payload を `mizar-core` の
`ResolvedTypedAstSummary::from_ast` に渡し、summary-readiness を確認する。これは
`mizar-core` lowering を実行したり、artifact を publish したり、public diagnostic を
割り当てたり、CoreIr / ControlFlowIr / VC / proof corpus row を昇格したりするものではない。

## データ形状

public data layer は assembled output 内で local な dense id を保つべきである。

```rust
struct ResolvedTypedAst {
    source_id: SourceId,
    module_id: ModuleId,
    source_context: Option<SourceBindingContextHandoff>,
    source_type: Option<SourceTypeApplicationHandoff>,
    source_attribute: Option<SourceAttributeHandoff>,
    nodes: ResolvedTypedArena,
    expr_metadata: ExpressionMetadataTable,
    collection_candidates: OverloadCandidateSummaryTable,
    expanded_candidates: OverloadCandidateSummaryTable,
    template_expansions: TemplateExpansionSummaryTable,
    viable_candidates: OverloadCandidateSummaryTable,
    viability_decisions: CandidateViabilitySummaryTable,
    specificity_graphs: ResolvedSpecificityGraphTable,
    resolved_overloads: OverloadResolutionTable,
    inserted_coercions: CoercionInsertionTable,
    cluster_facts: ClusterFactTable,
    diagnostics: ResolvedTypedDiagnosticTable,
    checked_formulas: CheckedFormulaTable,
    statement_semantics: StatementSemanticTable,
}
```

Task 248 は差し替え可能な別 source-context assembler input を許さない。assembly
は supplied `TypedAst` からだけ `source_context` を clone するため、final layer は
checker-owned source-item、declaration、binding、local-context link から乖離できない。
absent 時は legacy debug byte を維持し、present 時は deterministic nonempty
handoff を render する。

Task 249は`source_type`にも同じruleを適用する。assemblyはimmutable handoffを
`TypedAst`からだけcloneし、independent source-type inputを受け取らないため、
authenticated済みflat application/expression/argument tableから乖離できない。
absent時はlegacy debug byteを維持する。

Task 250は`source_attribute`にも同じclone-only ruleを適用する。assemblyは
independent attribute-chain inputを受け取らず、immutable handoffを`TypedAst`から
だけcopyするため、final layerはauthenticated chain、polarity、qualifier、group、
actual、provenance、Task-249 association tableから乖離できない。absent時はlegacy
debug byteを維持する。

### Resolved Node

```rust
struct ResolvedTypedNode {
    id: ResolvedTypedNodeId,
    typed_node: TypedNodeId,
    source_range: SourceRange,
    children: Vec<ResolvedTypedNodeId>,
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

`final_type` は final semantic precedence に従って解決する。expression が successful overload
result を持つ場合、assembly はまず `exposed_result.result` が存在すればそれを使い、次に
selected root candidate の result type があればそれを使う。successful overload result がない場合は、
`TypedAst` の handoff-available な `TypeEntryActual::Known` type を使う。open な
`TypeEntryActual::CandidateSet` entry はそれだけでは final type ではない。successful overload result で
解決されない場合、`final_type` は `None` のままとし、failed / open state は diagnostic と overload
metadata を通じて可視のままにする。

### Overload Candidate And Graph Summary

`ResolvedTypedAst` は `@show_resolution`、diagnostic、artifact、downstream elaboration に必要な
candidate summary と specificity graph summary をコピーする。後続 consumer が task 22 から task 25 の
precursor output を保持していることを要求しない。

candidate id は owning predecessor table 内で dense なので、task 28 は 3 つの明示的な candidate
namespace を保持する。`collection_candidates` は task 22 collection table をコピーする。
`expanded_candidates` は non-template candidate と instantiated template candidate を含む、viability
decision 用の task 23 template-expansion candidate table をコピーする。`viable_candidates` は
specificity graph、overload selection、inserted view が使う viability / specificity candidate table をコピーする。
`TemplateExpansionSummary` は collection の `source_candidate` id から optional な expanded
`instantiated_candidate` id への明示的な橋渡しである。`CandidateViabilitySummary` は expanded の
`source_candidate` id から optional な viable `output_candidate` id への明示的な橋渡しである。
`OverloadResolutionRecord`、`ResolvedSpecificityGraph`、`CoercionInsertion` の candidate reference は
すべて viable namespace を使う。

```rust
struct OverloadCandidateSummary {
    candidate: OverloadCandidateId,
    site: OverloadSiteId,
    symbol: SymbolId,
    ordinary_root: SymbolId,
    declaration_kind: CandidateDeclarationKind,
    parameters: Vec<NormalizedTypeId>,
    result: Option<NormalizedTypeId>,
    origin: CandidateOrigin,
    template: Option<TemplateCandidatePayload>,
    coherence: Option<CoherenceStatus>,
    provenance: CandidateProvenance,
    status: OverloadCandidateStatus,
    diagnostics: Vec<ResolvedTypedDiagnosticId>,
}

struct TemplateExpansionSummary {
    id: TemplateExpansionId,
    source_candidate: OverloadCandidateId,
    site: OverloadSiteId,
    template: SymbolId,
    instantiation_key: TemplateInstantiationKey,
    substitutions: Vec<TemplateSubstitution>,
    instantiated_candidate: Option<OverloadCandidateId>,
    status: TemplateExpansionStatus,
    diagnostics: Vec<ResolvedTypedDiagnosticId>,
}

struct CandidateViabilitySummary {
    id: CandidateViabilityId,
    source_candidate: OverloadCandidateId,
    site: OverloadSiteId,
    output_candidate: Option<OverloadCandidateId>,
    status: CandidateViabilityStatus,
    diagnostics: Vec<ResolvedTypedDiagnosticId>,
}

struct ResolvedSpecificityComparison {
    id: SpecificityComparisonId,
    left: OverloadCandidateId,
    right: OverloadCandidateId,
    status: SpecificityComparisonOutcome,
    reasons: Vec<SpecificityReasonKey>,
    diagnostics: Vec<ResolvedTypedDiagnosticId>,
}

struct ResolvedSpecificityGraph {
    graph: SpecificityGraphId,
    site: OverloadSiteId,
    nodes: Vec<SpecificityNode>,
    comparisons: Vec<ResolvedSpecificityComparison>,
    edges: Vec<SpecificityEdge>,
    diagnostics: Vec<ResolvedTypedDiagnosticId>,
}
```

これらの summary は candidate status、template payload、coherence status、candidate provenance の
declaration span / import provenance、template substitution / skipped-template status、failed /
no-match site 向け viability rejection / blocking reason、graph の stable comparison evidence を保持する。
すべての diagnostic reference は `ResolvedTypedDiagnosticId` に変換する。これは copied metadata であり、
2 つ目の overload-resolution engine ではない。

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
別の schema task とする。task 28 は reused fact row の一部として
`ClusterFactProvenance::TraceStep` id を保持するが、full trace step payload の validation や storage は行わない。

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

## Public Enum Policy

task 31 は frontend task-25 の public-enum decision procedure をこの module に適用する。
`resolved_typed_ast` の public checker-owned enum はすべて forward-compatible API surface
であり、`#[non_exhaustive]` を維持しなければならない。downstream consumer は wildcard
または fallback arm を保持する。checker 内部の match は、仕様化済み behavior を実装するために
現在表現されている variant へ exhaustive のままにしてよい。

| enum | decision |
|---|---|
| `ResolvedNodeKindHintKind` | 前方互換; source-shaped node hint は downstream presentation need とともに増える可能性がある。 |
| `ResolvedTypedNodeKind` | 前方互換; resolved node category は後続 source-shaped projection とともに増える可能性がある。 |
| `ResolvedNodeRecovery` | 前方互換; node recovery state は partial assembly policy とともに増える可能性がある。 |
| `ResolvedNodeRecoveryReason` | 前方互換; recovery reason は source extraction と failed-site handling の拡大に伴い増える可能性がある。 |
| `OverloadResolutionStatus` | 前方互換; projected overload status は phase-8 result handling とともに増える可能性がある。 |
| `CoercionInsertionSource` | 前方互換; insertion source は accepted coercion/view form とともに増える可能性がある。 |
| `ResolvedTypedDiagnosticSource` | 前方互換; diagnostic source は追加 projection stage とともに増える可能性がある。 |
| `ResolvedTypedDiagnosticSeverity` | 前方互換; diagnostic severity policy は IDE/artifact consumer とともに増える可能性がある。 |
| `CandidateSummaryNamespace` | 前方互換; candidate-summary namespace は追加 overload table とともに増える可能性がある。 |
| `ResolvedTypedAstError` | 前方互換; assembly validation error は新しい projection invariant とともに増える可能性がある。 |
| `TheoremPolicyIntent` | 前方互換; declaration-policy intent は明示的にsupportするtheorem modifierとともに増える可能性がある。 |
| `TheoremJustificationIntent` | 前方互換; justification intent は明示的にextractするwritten proof formとともに増える可能性がある。 |
| `CheckedProofStatus` | 前方互換; checker-owned proof processing stateはacceptanceを意味せず拡張され得る。 |
| `CheckedProofNodeKind` | 前方互換; checked proof skeleton nodeはchecker Task 247 descendantで拡張され得る。 |
| `CheckedCitation` | 前方互換のempty carrier; citation variantはchecker Task 247 descendantまでdeferred。 |
| `CheckedProofLabel` | 前方互換のempty carrier; proof-label variantはchecker Task 247 descendantまでdeferred。 |

この module が所有する exhaustive public enum exception はない。

## task 28 の planned tests

task 28 は Rust coverage を追加すべきである。

- explicit `TypedAst` と checker output からの source-shaped assembly;
- `TypedSiteRef` / expression id による metadata lookup;
- successful overload result が open candidate set より優先される final-type precedence;
- rejected expanded candidate により viable output id がずれる case を含む、collection、expanded、
  viable candidate namespace の分離;
- instantiated / rejected / deferred template の template expansion summary;
- active refinement と inserted view を含む resolved overload projection;
- `NoMatch`、`Ambiguous`、incompatible refinement join、blocked status の
  failed overload site preservation;
- failed overload site が inserted coercion record を生成しないこと;
- equivalent input ordering に対する deterministic debug rendering;
- cluster fact id reference と既存 cluster-trace provenance の保持。

## Deferred And External Gaps

task 28 後も以下は deferred のままである。

- task 26 selection payload と source expression metadata の AST-wide
  source-to-checker extraction;
- artifact emission/reuse と stable artifact schema;
- full `ResolutionTrace` artifact projection / validation;
- public diagnostic-code allocation;
- final overload / cluster projection の active `.miz` semantic fixture。

これらの gap は task 28 で fabricated success record、raw syntax scan、artifact-like
side output を許可しない。

## Task 266 exact statement-semantic projection

Task 266 は `ResolvedTypedAstInputs` に optional な syntax-free predecessor
bundle を追加する。

```rust
struct StatementSemanticInputs<'a> {
    owner: &'a CheckedStatementOwner,
    binding_env: &'a BindingEnv,
    term_formula: &'a TermFormulaInferenceOutput,
    rows: Vec<StatementSemanticInput>,
}

struct StatementSemanticInput {
    owner: SymbolId,
    owner_node: TypedNodeId,
    formula: CheckedFormulaId,
    formula_node: TypedNodeId,
}
```

bundle がない既存 assembly は checked-formula/statement-semantic table を
empty にし、debug output を byte-stable に保つ。Task 180 の bundle は source
order どおりの row 1件だけを持つ。assembly は既存 checked formula table を
copy し、resolver theorem symbol、theorem typed-node identity、owner range/
`SemanticOrigin`、existing checked formula identity/site、compact final tree用の
separate formula typed-node identity を保持する dense `StatementSemantic` 1件を
出力する。

受理する typed tree は module root -> theorem owner -> formula の3 node
だけで、全 node は normal/successfully typed である。owner range は validated
theorem range と一致し、formula range/recovery は checked formula と一致し、
root は owner を包含し formula は owner の strict interior にある。
`TypedAst`、`BindingEnv`、inference output、checked owner の source/module は
一致する。inference output は normal `Checked`
`FormulaKind::Contradiction` 1件だけを持ち、term/type entry/normalized type/
candidate/fact/diagnostic/asserted type/expected constraint/deferred reason は
禁止する。

supplied bundle の row 欠落、non-singleton、duplicate owner/formula、reorder、
unknown、recovered、deferred、cross-source/module、tree/range/provenance/
owner/formula mismatch は fail closed とする。real resolver theorem owner の
validation は `type_checker` が所有し、本 module は `SymbolEnv` や raw syntax
を scan しない。この projection は truth/fact/theorem acceptance/proof/
terminal goal/CoreIr/ControlFlowIr/VC semantics を追加しない。

## Task 267 justification 省略 proof-handoff contract

Task 267 は Task 268 が実装する target contract を確定する。それ自体では
current Rust surface を変更しない。exact source syntax を分類するのは
`mizar-test` だけである。対象は unrecovered、status annotation なしの ordinary
theorem 1件、contradiction formula child 1件、justification node なしである。
この確認済み構文事実を明示的な `Unmodified` と `Omitted` intent に変換する。
checker assembly と core lowering は missing row、absent optional field、raw
syntax から intent を推測してはならない。

Task 268 は次の exact shape の syntax-free input row 1件を追加する。

```rust
struct StatementProofIntentInput {
    id: StatementProofIntentId,
    source_order: usize,
    statement: StatementSemanticId,
    source_id: SourceId,
    module_id: ModuleId,
    owner: SymbolId,
    owner_node: TypedNodeId,
    owner_range: SourceRange,
    owner_origin: SemanticOrigin,
    owner_visibility: Visibility,
    owner_export_status: ExportStatus,
    formula: CheckedFormulaId,
    formula_site: TypedSiteRef,
    formula_node: TypedNodeId,
    formula_range: SourceRange,
    recovery: NodeRecoveryState,
    policy: TheoremPolicyIntent,
    justification: TheoremJustificationIntent,
}

enum TheoremPolicyIntent { Unmodified }
enum TheoremJustificationIntent { Omitted }
```

本sectionのnew enum 6個（`TheoremPolicyIntent`、
`TheoremJustificationIntent`、`CheckedProofStatus`、`CheckedProofNodeKind`、
empty carrierの`CheckedCitation`と`CheckedProofLabel`）は上記module policyに従う
public `#[non_exhaustive]` surfaceである。Task 268はcurrent-source policy tableへ6 rowを
same implementation commitで追加し、Task 267はunimplemented enumをlint-guarded
tableへ入れない。rowは`StatementSemanticInputs`から推測せず、
separate optional top-level bundleで供給する。

```rust
struct StatementProofInputs<'a> {
    pub owner: &'a CheckedStatementOwner,
    pub rows: Vec<StatementProofIntentInput>,
}

struct ResolvedTypedAstInputs<'a> {
    // existing fields unchanged
    statement_semantics: Option<StatementSemanticInputs<'a>>,
    statement_proofs: Option<StatementProofInputs<'a>>,
}
```

Task 268はexisting struct-literal callerを`None`で更新する。supplied proof bundle
はsame assembly callのstatement bundleと、そのauthenticated
`CheckedStatementOwner`を要求する。supplied exact Task-180 statement bundleも
proof bundleを要求する。両bundleはどちらのtable familyもpublishする前にlocal
stateで一緒にvalidateする。

`StatementProofInputs`と`StatementProofIntentInput`はpublic fieldを持つpublic
input structである。`StatementProofIntentId`はpublic dense idで、`mizar-test`が
syntax-free rowをconstructできる`new(index)`/`index()`を公開する。Task-268
extensionは`CheckedStatementOwner`にpublic `visibility()`/`export_status()` getter
を追加し、stored fieldはprivateのままにする。

exact Task-180 source では `id`、`source_order`、`statement` はすべて dense
index 0 である。`formula_site` は既存 Task-266 `TypedSiteRef::Node`、
`formula_node` は別の compact final-tree node であり、real site を置換・再構築
してはならない。visibility/export は明示的 resolver fact
`Public`/`Exported`、recovery は `Normal`、policy/justification は上記2 variant
である。source、module、owner、owner node/range/origin、formula
id/site/node/range、recovery は Task-266 statement/checked-formula dataおよび
authenticated owner bundleと一致しなければならない。Task 268は
`CheckedStatementOwner`をresolver entryのvisibility/export status保持へ拡張し、
proof-intent rowへcopyする前に独立validationする。`Exported` は resolver name
visibility だけを表し、proof
acceptance ではない。

accepted output は all-or-none の singleton dense table 3個である。

```rust
struct CheckedProof {
    id: CheckedProofId,
    source_order: usize,
    statement: StatementSemanticId,
    owner: SymbolId,
    owner_node: TypedNodeId,
    owner_visibility: Visibility,
    owner_export_status: ExportStatus,
    proposition: CheckedFormulaId,
    policy: TheoremPolicyIntent,
    justification: TheoremJustificationIntent,
    root: CheckedProofNodeId,
    status: CheckedProofStatus,
    source_range: SourceRange,
    owner_origin: SemanticOrigin,
}

enum CheckedProofStatus { PendingAutomaticProof }

struct CheckedProofNode {
    id: CheckedProofNodeId,
    proof: CheckedProofId,
    kind: CheckedProofNodeKind,
    source_range: SourceRange,
    recovery: NodeRecoveryState,
}

enum CheckedProofNodeKind {
    TerminalGoal(CheckedTerminalGoalId),
}

struct CheckedTerminalGoal {
    id: CheckedTerminalGoalId,
    proof: CheckedProofId,
    node: CheckedProofNodeId,
    statement: StatementSemanticId,
    owner: SymbolId,
    formula: CheckedFormulaId,
    formula_site: TypedSiteRef,
    formula_node: TypedNodeId,
    source_range: SourceRange,
    recovery: NodeRecoveryState,
    citations: Vec<CheckedCitation>,
    active_context: Vec<CheckedFormulaId>,
    local_path: String,
    label: Option<CheckedProofLabel>,
}

#[non_exhaustive]
pub enum CheckedCitation {}

#[non_exhaustive]
pub enum CheckedProofLabel {}
```

`CheckedCitation`と`CheckedProofLabel`はTask-267 variantを持たないため、exact
valueはempty vector/`None`だけである。empty public enumはworkspaceの
`deny(warnings)` policy下でlint-cleanであり、Task 268からconstructできない。
named forward-compatible carrierはsymbol/local label/generated-origin citation
semanticsをauthorizeしない。broader variantはChecker Task 247が所有する。

`ResolvedTypedAst`はpublic `CheckedProofTable`、`CheckedProofNodeTable`、
`CheckedTerminalGoalTable` typeのprivate fieldを所有し、`checked_proofs()`、
`checked_proof_nodes()`、`checked_terminal_goals()` getterを公開する。public
dense idはexisting statement idと同様に`new(index)`/`index()`を公開する。各table
は`get`、source-order `iter`、`len`、`is_empty`を公開し、mutationは
assembly-privateである。3 row structは`StatementSemantic`と同様のpublic
read-only fieldを持つ。legacy empty assemblyは3 empty tableを返す。

3 tableがnonemptyの場合、canonical `debug_text()`は全row fieldとcross-referenceを
含む3 tableすべてをdeterministic source/id orderでrenderする。3 tableすべてが
emptyの場合はnew proof sectionをemitせず、Task-266 legacy projectionと
byte-for-byte identicalのままとする。Task 268はexact nonempty rendering assertionと
empty-output byte-stability regressionの両方を所有する。

全 id と `source_order` は 0 である。proof source は owner range、single root
node と terminal goal は formula range を使う。root は直接
`TerminalGoal(CheckedTerminalGoalId(0))` であり、`CurrentGoal`、`Sequence`、
implicit `Thesis`、intermediate step、synthesized child はない。terminal row
は proof/node/statement 0、同じ owner/checked contradiction、real formula
site、separate compact node を参照する。recovery は normal、citation と active
context は empty、local path は exact nonempty `proof/0`、label はない。

`Unmodified` は declaration-policy axis である。`PendingAutomaticProof` は
automatic proof attempt が未実行であることを示す別の processing axis で、
`Open`、`Assumed`、`Conditional`、`Error`、published fact、proof evidence、
theorem acceptance、discharge ではない。statement/proof の片側欠落、
non-singleton、duplicate/non-dense id、nonzero source order/statement reference、
wrong root/cross-reference、role site、recovery、identity/range/provenance/status
mismatch は publication 前に fail する。この singleton contract の
“reordered” は explicit source-order/id/reference chain が exact
`0 -> 0 -> 0` でないことを表す。assembly は transactional で3 table 全部か
none を出力し、error/partial proof row に置き換えない。Task-266 statement
bundle と proof-intent bundle の両方を渡さない existing caller は legacy empty
behavior を維持する。Task 268 後は exact Task-180 statement bundle があるのに
proof-intent bundle がない場合は error である。

Task 268 が所有するのはこの producer と corruption tests だけである。broader
theorem/proof form、truth/fact、proof search、acceptance、CoreIr/ControlFlowIr/VC
generation、fixture/expectation change、Step 6/7 behavior は追加しない。Core Task
31 はこの explicit table を consume し、source scan で intent を復元しない。

## Task 268 implementation completion

Task 268はaccepted contractをexact Task-180 sourceだけに実装する。
`ResolvedTypedAst::assemble`はstatement/proof bundleを同時に要求し、explicit
`Unmodified`/`Omitted` rowをauthenticated ownerとchecked contradictionへ照合し、
3 singleton tableをlocal stateで構築する。publication前にprivate
postvalidationがcardinality、dense id、root/status、cross-reference、empty
carrier、`proof/0` metadataを検証する。全mismatchはpartial tableなしでerrorを返す。
construct可能なproof/node/goal invariantはcloned-table corruptionで検査する。
single-variant policy/justification/statusとempty citation/labelにはsafeなinvalid
valueがないためprivate predicateを独立testし、status rejectionは同じvalidator
coreへfalse status-matchを渡すseamでも検査する。

exact nonempty debug projectionはproof/node/terminal-goal tableの全fieldを
deterministicにrenderする。captured Task-266 empty-bundle stringがbyte-identical
legacy renderingをguardする。checkerへのraw syntax dependency、fact publication、
theorem acceptance/discharge、CoreIr/ControlFlowIr/VC payloadは追加しない。次の
dependency-authorized consumerはCore Task 31で、Steps 6/7はdeferredのままである。

## Task 251 final-handoff addendum

`ResolvedTypedAst`は`TypedAst`へinstall済みのoptional checker-owned
`SourceEvidenceHandoff`をclone-preserveし、borrowed `source_evidence()` getterを
exposeする。final assemblyはevidenceをrebuild/reinterpret/acceptせず、handoff
absent時のlegacy projectionは不変である。

## Task 252 final-handoff addendum

`ResolvedTypedAst`は`TypedAst`にinstallされたoptional checker-owned
`SourcePrimaryTermHandoff`をclone-preserveし、borrowed `source_term()` getterを
exposeする。final assemblyはsource termをrebuild/reinterpretせず、numeric typeを
選択せず、semantic resultを作らない。handoff absent時のlegacy projectionは
不変である。

## Task 253 final-handoff addendum

`ResolvedTypedAst`はexact Task-252 fingerprint/primary-edge associationを
revalidateし、`TypedAst`にinstallされたoptional checker-owned
`SourceFunctorApplicationHandoff`をclone-preserveする。borrowed
`source_application()` getterがimmutable handoffをexposeする。final assemblyは
dense IDのrebuild/retarget、candidate collect/select、signature/result type
resolve、semantic result作成を行わず、handoff absent時のlegacy projectionは不変で
ある。

## Task 254 final-handoff addendum

`ResolvedTypedAst`はexact Task-252/conditional Task-253 fingerprint、
root-only cross-family target、arena-site associationをrevalidateし、`TypedAst`に
installされたoptional checker-owned `SourceStructureHandoff`をclone-preserve
する。borrowed `source_structure()` getterがimmutable handoffをexposeする。
final assemblyはdense IDをrebuild/retargetせず、member/inheritance viewをresolve
せず、constructor coverageを決めず、selector/update resultやsemantic resultを
作らない。handoff absent時のlegacy projectionは不変である。

## Task 255 final-handoff addendum

`ResolvedTypedAst`はexact Task-252/conditional Task-253/254 fingerprint、
nearest-family cross-target partition、canonical spelling、arena-site associationを
revalidateし、`TypedAst`にinstallされたoptional checker-owned
`SourceSetTermHandoff`をclone-preserveする。borrowed `source_set_term()` getterが
immutable handoffをexposeする。final assemblyはdense IDをrebuild/retargetせず、
comprehension generatorをbindせず、conditionをresolveせず、sethood/
nonemptiness/wideningを決定せず、result type/semantic resultを作らない。
handoff absent時のlegacy projectionは不変である。

## Task 256 final-handoff addendum

`ResolvedTypedAst`はexact Task-252とconditional Task-253/254/255 fingerprint、
resolver provenance、request association、nearest-family target partition、
arena siteを再検証し、optional checker-owned `SourceAtomicFormulaHandoff`を
clone-preserveする。borrowed `source_atomic_formula()` getterがimmutable handoffを
公開する。final assemblyはdense IDのrebuild/retarget、predicate candidate選択、
expected-input request回答、assertion/formula判断、fact publish、theorem acceptanceを
行わず、handoff absent時のlegacy projectionは不変である。

## Task 257A final-handoff addendum

`ResolvedTypedAst::assemble`はoptional checker-owned
`SourceCompositeFormulaHandoff`をrevalidateしてclone-preserveし、raw sourceから
tree/binder/contextをrebuildしない。borrowed `source_composite_formula()` getterは
同じimmutable transactionを公開する。assemblyはinvalid source-context coexistence
またはdependency driftをrejectし、unresolved request 6件へ回答しない。

## Task 257B1 Final-Handoff Addendum

`ResolvedTypedAst::assemble`はTask-252/Task-256/第2 Task-257 fingerprintを
revalidateし、optional `SourceFormulaCompositionHandoff`のatomic-edge/
bound-use rowをrebuild/renumberせずclone-preserveする。borrowed
`source_formula_composition()` getterがimmutable handoffを公開する。absenceは
legacy byteを保持し、presenceはformula result/fact/truth、theorem acceptance、
proof、downstream IRを追加しない。

Task 257B2もこのboundaryをreuseし、final assemblyはexact third composite
profileと`8/0` composition fingerprintを再検証してclone-preserveする。
resolved resultにはchecked formula、statement semantics、accepted theorem、
proof、IR outputを追加しない。

## Task 257B3 Frozen Final-Handoff Addendum

final assemblyはTask-48 reserve provenance、Task-252/256 fingerprint、fourth
composite profile、nested binding environment、`3/6` owning-edge/use
associationを再検証してからhandoffをclone-preserveする。Task-248
source-context handoffのabsenceはexact profileの一部。checked formula、
witness、restriction result、closure、fact、theorem acceptance、proof、IR
outputをassembleしない。

B3 orphan rejection、exact revalidation、deterministic cloneはchecker/
runner executable testsでcover済み。

## Task 257C1 frozen final-handoff addendum

`ResolvedTypedAst::assemble`はextended Task-256 9-table validationを再実行し、
exact chain handoffをclone-preserveする。segmentをreconstructせず、重複した
imported candidateからwinnerをselectしない。orphan/shared-edge、
polarity-token、provenance、request、fingerprint、legacy-profile corruptionは
fail closed。checked formula、conjunction/negation result、predicate winner、
theorem acceptance、proof、IR outputをassembleしない。

implemented Task 257C1 runnerはtyped/resolved debug byte stabilityと
Task-252/Task-256両handoff equalityを確認する。resolutionはclone-onlyで、
semantic selectionを追加しない。

## Task 255C1 frozen resolution addendum

final assemblyは7番目のTask-255 table、colon/direct condition-wrapper arena
anchor、authenticated condition range、unchanged Task-252/253 fingerprintを
revalidateしてclone-preserveする。Task-256 inner equality/Task-257
compositionを構築せず、condition operandをretargetせず、dense IDをrebuildしない。

## Task 255C1 resolution result

final assemblyはrecursive condition boundaryを含むexact Task-252/253/255
objectをrevalidateしてclone-preserveする。本task由来のexpression metadata、
fact、diagnostic、Task-256 condition formula、Task-257 compositionはresolved
objectに存在しない。

## Task 257C2 frozen resolution addendum

final assemblyはexact Task-252/253/255/256 object後にoptional
`SourceConditionFormulaCompositionHandoff`をclone/revalidateする。
fingerprint 4件とcondition-to-atomic relationを比較し、ID/formula rowを
rebuildしない。missing/stale/substituted/reordered dependencyはdedicated
`ResolvedTypedAstError::InvalidSourceConditionFormulaComposition` variantで
final assemblyをfailさせる。exact accessorは次の通り。

```rust
pub const fn source_condition_formula_composition(
    &self,
) -> Option<&SourceConditionFormulaCompositionHandoff>;
```

frozen pre-Task-256C1 baselineでは、このprojectionはseparate lower taskが
authenticated condition containmentを両installation orderでvalidにするまで
gateされていた。Task 256C1は両orderをpassし、projectionは現在実装済みで、
dependency fingerprint 4件とsole associationをrevalidateしてimmutable
handoffをclone-preserveする。final assemblyはそのlower validationを
compensate/weakenしない。

checked formula、equality truth、fact、diagnostic、definition acceptance、
proof、IR outputはderiveしない。

## Task 257C3 frozen final projection

later final projectionはtyped ownership成功後、optional predicate-chain
composition handoffをrevalidate/clone-preserveする。matching accessor、
deterministic debug placement、`InvalidSourcePredicateChainComposition`を
追加するが、expression metadata、diagnostic、checked formula、truth、fact、
downstream semantic resultは追加しない。本documentation prerequisiteは
resolved source/outputを変更しない。

```rust
pub const fn source_predicate_chain_composition(
    &self,
) -> Option<&SourcePredicateChainCompositionHandoff>;
```

cloned C3 debug chunkはTask-252 source-term、Task-256 source-atomic-formula、
A/B/C2 slotの後、resolved node/table section直前のfinal mutually exclusive
formula-owner slotを占める。

## Task 257C3 implementation result

final assemblyはoptional handoffをcloned Task-252/256 dependency/arenaに対して
revalidateし、`Clone`でpreserveし、frozen slotへdeterministic debug chunkを
emitする。test-only stateでrequired lower handoffをremoveすると
`InvalidSourcePredicateChainComposition`でfailする。expression metadata、
diagnostic、truth、fact、proof、downstream IRはemptyのまま。

## Task 258A frozen final projection

final assemblyはexact Task-252/256 lower handoff後にoptional
`SourceStatementHandoff`をrevalidate/clone-preserveし、read-only
`source_statement()` accessorだけを追加する。missing/stale/substituted/
corruptなlower/owner/statement/context/fact relationは
`ResolvedTypedAstError::InvalidSourceStatement`。assemblyはIDをrebuildせず、
label resolve/equality check/existing `StatementSemanticInput`/
`StatementProofIntentInput`生成を行わない。Task-266/268 standalone-
contradiction checked tableはdisjoint/unchanged。expression metadata、
checked formula、statement semantics、proof/terminal goal、fact、diagnostic、
downstream IRはempty。本prerequisiteはresolved source/outputを変更しない。

revalidationはhandoff-owned exact `BindingEnv`とfingerprintを含む。
Task-248 `source_context`とTask-258A `source_statement`を同時に含む
test-injected typed inputはdeterministically `InvalidSourceStatement`。
final outputをpublishせずoriginal typed debugをbyte-identicalに保ち、validな
single-owner inputはいずれもreplayできる。

### Task 258A implementation result

final assemblyはowned binding environment、Task-252/256 fingerprint、arenaを
再検証してhandoffをclone-preserveする。output construction前にnonempty
typed semantic table、cluster fact、overload-stage output、expression input、
statement semantics/proof、diagnosticをrejectする。empty node hintとexact
complete `source.statement.transport` source-preserved hint setだけはsyntax
nodeをpreserveするためadmitし、他のnonempty hint setは
`InvalidSourceStatement`となる。

## Task 258B1 frozen final statement projection

`ResolvedTypedAst`はoptional `SourceStatementReferenceHandoff` cloneとexact
accessorを追加する。

```rust
pub const fn source_statement_references(
    &self,
) -> Option<&SourceStatementReferenceHandoff>;
```

assemblyはtyped ownership後のcomplete B1 base/reference pairだけをadmit。
source/module、`3/1/0` environment、Task-252/256 fingerprint、shared
arena/statement topology、sole resolved `Label(0)` node 68を持つstored
77-node/root-76 `ResolvedAst`、projection/reference/result replay、
両handoff fingerprint、全rowをrevalidateする。片方欠落、stale/substituted
provenance、Task-248/257/258A coexistence、nonempty semantic-stage inputは
publication前に`ResolvedTypedAstError::InvalidSourceStatement`。

final debugはbase statement chunk直後、resolved node/table前にreference
chunkを置く。Cloneは両exact handoffとretained resolver ASTをpreserve。
Task-258Aはreference fieldが
absentなのでoutput byte-identical。final assemblyでname resolutionをinfer
せず、fact、checked formula、statement semantic、proof、goal、diagnostic、
downstream IR/VC outputを作らない。本prerequisiteはresolved sourceを変更しない。

### Task 258B1 implementation status

final assemblyはB1 base/reference pairを一緒にclone/revalidateし、orphan、
missing、stale、cross-profile halfを`InvalidSourceStatement`としてrejectする。
reference debug chunkはbase chunk直後、resolved node前に出る。statement
semantic、checked-formula、proof、goal、diagnostic、downstream tableはすべて
emptyのまま。

### Task 258B2 frozen final ownership

final assemblyはtyped ownership後のTask-258B2 base-only handoffをadmitする。
exact 113-byte sourceと、frozen Task-48 `2/1/0`、Task-252 `6/6/0`、
Task-256 `3/0/0/0/0/0/0/6/6`、statement `1/3/3/3/3` profileを
revalidateする。retained resolver provenanceはorigin path `[2, 1]`、
contribution 0のpublic/exported theorem 1件を持ち、proof-step label、
citation、reference keyを持たない。

source/profile substitution、reference half、competing payload owner、
nonempty semantic-stage inputはpublication前にfailする。Clone/debugはbase
handoffをexactに保持する。final assemblyはnew resolutionをinferせず、
accepted premise、fact、checked formula、statement semantic、proof、goal、
diagnostic、theorem result、downstream IR/VC outputをpublishしない。本prerequisite
はresolved source/testを変更しない。

### Task 258B2 implementation closure

final assemblyはreference associationなしのexact base-only B2 handoffを
revalidateしてclone-preserveする。cluster fact、statement-proof input、
foreign source family、checked formula、statement semantic、proof、goal、
diagnostic、その他semantic outputはrejectしたままで、successful resultは
全該当tableをemptyに保つ。

### Task 258B3 frozen final ownership

final assemblyはtyped ownership後だけTask-258B3 statement/witness pairを
admitする。source/module identity、base/lower fingerprint、exact
`1/2/2/2/2` + one-row profile、shared 49-node arena、atomic edgeからのterm
2 exclusion、combined source order `[0,1,2]`をrevalidateする。resultはequal
`source_statement()`/`source_statement_witnesses()` cloneを公開し、
reference handoffはない。

orphan witness、standalone B3 base、stale fingerprint、B1/B2 hybrid、
foreign source owner、cluster fact、statement-proof input、nonempty semantic
outputは`InvalidSourceStatement`。final assemblyはresolver/witness semanticsを
inferせず、checked formula、fact、statement semantic、proof、goal、
diagnosticをemptyに保つ。本prerequisiteはresolved source/testを変更しない。

### Task 258B3 final ownership result

final assemblyはexact base/witness pairをclone-preserve/revalidateする。
standalone base、orphan witness、stale fingerprint、reference hybrid、
nonempty semantic coexistenceは`InvalidSourceStatement`。successful
assemblyは全semantic/proof/goal/diagnostic tableをemptyに保つ。

## Task 258B3N planned final ownership

final assemblyは`TypedAst`が既にownするauthenticated B3N base +
witness/name bundleだけをacceptする。B3/B3N hybrid、standalone half、
reference hybrid、stale name link/fingerprint、semantic/proof/goalとの
coexistenceは`InvalidSourceStatement`。successful B3N assemblyは
transport-onlyのままdense name tableをclone-preserveする。

## Task 258B3N 実装結果

final assemblyはB3N baseとwitness/name bundleをrevalidateして
clone-preserveする。orphan/standalone half、stale statement/primary
fingerprint、reference hybrid、nonempty typed table、expression/cluster
metadata、proof input、statement semanticsは`InvalidSourceStatement`でfail
し、success時は全semantic/proof/goal/diagnostic tableがemptyである。

## Task 258B3M1 planned final ownership

final assemblyはalready authenticated B3M1 base/witness pair、2 witness
rows、1 name row、exact fingerprint、56-node arenaだけをclone-preserve
できる。両row、shared source ordinal、dense within-`take` order、name link、
subtree ownershipをrevalidateする。B3/B3N/B3M1 hybrid、orphan half、
reference hybrid、stale dependency、nonempty semantic/proof/goal tableは
`InvalidSourceStatement`でfailする。semanticsはinferしない。

## Task 258B3M1 implementation result

final assemblyはauthenticated B3M1 pairをclone-preserveし、full
lower/base/witness/name dependency graphをrevalidateする。orphan、stale、
reference hybrid、B3/B3N/B3M1 cross-family、nonempty overload pipeline、
semantic、proof、goal inputsはすべて`InvalidSourceStatement`でfailする。
successful outputの全semantic/proof/goal tablesはempty。

## Task 258B3M2A planned final ownership

final assemblyはalready authenticated B3M2A base/witness pair、one
unnamed primary-numeral witness、0 names、exact Task-252 numeric request、
fingerprints、complete 49-node arenaだけをclone-preserveできる。row、
source partition `[0,1,2]`、subtree exclusions、全lower dependenciesを
revalidateする。standalone half、B3/B3N/B3M1/B3M2A hybrid、reference /
numeric-request corruption、stale dependency、nonempty semantic/proof/goal
tableは`InvalidSourceStatement`でfailする。successでもtype、existential
match、substitution、goal、proof effectはinferしない。

## Task 258B3M2A implementation result

final assemblyはauthenticated B3M2A base/witness pairだけを
clone-preserveする。exact one unnamed numeral witness、dependency
fingerprints、49-node arena、lower tablesをrevalidateし、standalone、
hybrid、stale、reference/numeric-request corruption、semantic coexistenceは
`InvalidSourceStatement`のまま。successful final handoffではexpression、
candidate、coercion、cluster、diagnostic、statement-semantic、proof、goal
tablesがempty。

## Task 258B3M2B1 frozen final ownership

final assemblyはauthenticated 53-node B3M2B1 base/witness pair、
five roots、six primary rows、one parenthesized witness/no namesだけを
clone-preserveする。standalone、hybrid、stale、parent/child/reference-map、
B3M2A、Tasks253–255、semantic coexistenceは
`InvalidSourceStatement`。source partition `[0,1,2]`、outer-wrapperへの
witness target、inner reference/parent edge、complete witness-subtree
exclusion、fingerprints、all lower dependencies、53-node arenaをrevalidate
する。successはexpression/candidate/coercion/cluster/diagnostic/
statement-semantic/proof/goal tablesをemptyにし、type/existential match/
substitution/goal/proof effectをinferしない。public final APIなし。

## Task 258B3M2B1 implementation result

final assemblyはauthenticated B3M2B1 base/witness pairだけを
clone-preserveする。exact 53-node arena、five-root/six-primary mapping、
wrapper/child edge、one unnamed outer-term witness、complete subtree
exclusions、dependency fingerprints、all lower tablesをrevalidateする。
standalone、hybrid、stale、parent/reference-corrupt、semantic-coexisting
stateは`InvalidSourceStatement`のまま。success時もsemantic/proof/goal
tablesはempty。

## Task 258B3M2B2A frozen final ownership

future final assemblyはauthenticated B3M2B2A base/witness pairだけを
clone-preserveし、57-node arena、five-root/seven-primary、two wrapper
links、one unnamed outer witness、subtree exclusions、fingerprints、全lower
tablesをrevalidateする。standalone、hybrid、stale、parent/reference
corrupt、family/semantic-coexisting stateは`InvalidSourceStatement`。
success時もsemantic/proof/goal tablesはempty。docs prerequisiteではfinal
public APIを変更しない。

## Task 258B3M2B2A implementation result

final assemblyは57-node arena、Task-252 parent/reference chain、Task-256
subtree exclusion、fingerprints、source orderを再検証後だけauthenticated
paired base/witness handoffをclone-preserveする。standalone、hybrid、
stale、corrupt、reversed、family/semantic coexistenceは
`InvalidSourceStatement`で、semantic/proof/goal tablesはempty。final public
API changeなし。

## Task 258B3M2B2B1A final bundle revalidation result

final assemblyはexact B1A application/statement/witness bundleをtyped-stage
authentication全体の独立repeat後だけclone-preserveする。63-node arena、
imported `parser.type_fixtures::++` application/resolver provenance、
Task-252 arguments/numeric requests、Task-256 equality-only exclusion、
base/witness profiles、lower fingerprints 2件、optional B1A application
fingerprintをrevalidateする。

既存のstandalone Task-253 applicationはvalidなfinal bundleのままである。
B1A application + statementのみ、B1A application + witnessのみ、orphan
statement/witness pair、application-free B1A hybrid、stale primary/
application fingerprint、substituted provenance、partial/reverse B1A family
install、semantic coexistenceは`InvalidSourceStatement`となり、cloneは
invalid stateをrepairしない。successful cloneはexpression-semantic、
candidate、coercion、cluster、diagnostic、statement-semantic、proof、goal
outputsをemptyに維持し、type、proof step、substitution、goal effectを
inferしない。

## Task 258B3M2B2B1B1 frozen final bundle revalidation

final assemblyはtyped-stage enumerationをmirrorする。existing
application/statement/witness bundleはexact B1Aまたはexact B1B1だけが
valid。B1B1は67-node arena、local theorem contribution/label、wrapped
Task-253 `1/1/1/2/2` application provenance/containment、Task-252
`6/4/2`、Task-256 equality-only exclusion、base `1/2/2/2/2`、one
unnamed `Application(0)` witness/no names、全fingerprintsをrevalidateする。
B1Aはunchanged bytesを持つindependent 63-node unwrapped profileのまま。

partial、orphaned、stale、substituted、B1A/B1B1-hybrid、reversed、
family/semantic coexistence stateは`InvalidSourceStatement`で、cloneは
repairできない。successful B1B1 cloneはthree handoffsをbyte-for-byte
preserveし、type-semantic、formula-semantic、proof、goal、overload、
coercion、obligation、cluster、diagnostic outputsをemptyに保つ。new
public final-AST API/semantic meaningは追加しない。

## Task 258B3M2B2B1B1 final revalidation result

final assemblyはexact B1B1 bundleをseparate private profileとしてacceptし、
frozen partial/hybrid/substitution casesを全てrejectする。clone
revalidationはthree handoffsをbyte-for-byte preserveし、deferred upper
tablesは全てempty。`resolved_typed_ast.rs`は7,225 lines。public final-AST/
semantic/proof/goal APIは変更していない。

## Task 258B3M2B2B2A frozen final-AST contract

ResolvedTypedAst public accessorは追加しない。future assemblyはexact
coexisting source-structure/source-statement/structure-target witness
handoffsをrequireし、statement/primary/structure fingerprintsと全lower
installations、equality-only Task 256をdirect structure fingerprintなしの
`Some(&structure)`でrevalidateし、three handoffsをbyte-for-byte cloneする。
current blanket structure+statement rejectionはexact B2A tripleだけrelax。
missing/orphan/partial/stale/application-structure hybrid/reverse/repeated
bundlesはatomically reject。全semantic/proof/goal/overload tablesはempty。

## Task 258B3M2B2B2A final revalidation result

final assemblyはexact coexisting structure/statement/structure-target
witness bundleだけをadmitする。equality-only Task 256の
`Some(&structure)` revalidationを含む全lower installation/fingerprintを
revalidateし、three handoffsをbyte-for-byte clone-preserveする。
missing/partial/stale/repeated/reversed/application-structure hybridは
`InvalidSourceStatement`のまま。

`resolved_typed_ast.rs`は7,241 lines。public final accessor、active route、
semantic/proof/goal owner、coverage creditは追加せず、upper semantic
tablesはempty。

## Task 258B3M2B2B2B frozen final-AST sibling

`ResolvedTypedAst` accessorは追加しない。final assemblyが
source-structure statementとcoexistできるのは、exact authenticated
siblings 2種だけ。B2Aは`Structure(0)`をtargetとする76-node
constructor-witness profile、B2Bは`Structure(0)`をtargetとし、その
selector baseが`Structure(1)`となる79-node selector-witness profile。
両方とも`application = None`、`structure = Some`なので、このoption
shapeではなくfull source/arena/ownership/lower-table/target/fingerprint
profileがsiblingをselectする。

B2B final assemblyはTask 252、Task 254、equality-only Task 256、Task 258
base rows、witness edge、全statement/primary/structure fingerprintsを
revalidateしてからauthenticated bundleをbyte-for-byte cloneする。
Task 256はformula application nodes `51/70`をownし、enclosing
`FormulaExpression` nodes `52/71`はunowned containersのまま。
B2A/B2B hybrid、swapped target、cross-profile fingerprint、partial/
repeated bundle、application coexistenceはatomically rejectする。
semantic/proof/goal/overload/theorem acceptance tablesはemptyのまま。

## Task 258B3M2B2B2B final revalidation result

final assemblyはB2BをB2Aのexact 79-node siblingとしてのみacceptする。
Task-48/252/254/256とTask-258 base profiles、structure fingerprint、
selector target `Structure(0)`、selector base `Structure(1)`、ownership
`51/70`、unowned containers `52/71`をrevalidateしてから、3 handoffsを
byte-for-byte clone-preserveする。

B2A/B2B hybrid、generic structure-plus-statement bundle、stale
fingerprint、swapped target、partial/repeated installation、application
coexistenceは`InvalidSourceStatement`のまま。semantic、proof、goal、
overload、theorem-acceptance outputsはすべてempty。
`resolved_typed_ast.rs`は7,244 linesで、public final-AST APIは変更して
いない。

## Task 258B3M2B2B2C frozen final-AST sibling

ResolvedTypedAst API追加なし。final assemblyはB2CをB2A/B2B besideのthird
exact structure-statement siblingとしてenumerateし、common
`application = None`/`structure = Some` shapeではなくcomplete
181-byte/86-node profileでselectする。Task252 `7/4/3`、Task254
`2/0/1/3/1/4/9`、Task256 equality pairs `Primary(0/1)`/
`Primary(5/6)`、Task258 base `1/2/2/2/2`、witness `1/0` target
`Structure(0)`をrevalidateする。

source/arena/provenance/ownership/rows/fingerprintsが全てPASSした後だけ
structure/statement/witness handoffsをbyte-for-byte cloneする。
B2A/B2B/B2C/application hybrids、stale fingerprints、swapped targets、
partial/reverse/repeated、subtree ownership substitutionはatomic reject。
semantic/proof/goal/overload/coercion/obligation/theorem outputsはempty。
implementation/final-clone testsはopen。

## Task 258B3M2B2B2C implemented final-AST sibling

final assemblyはB2A/B2Bと並べてB2Cをenumerateし、clone前にcomplete
source/arena、Tasks252/254/256/258、witness、structure fingerprint、
`Structure(0)` target contractをrevalidateする。public accessor/schema/
semantic output追加なし。hybrid/stale/swapped/partial/reverse/repeated/
subtree substitutionはatomic reject。

frozen checker final-clone testとrunner typed/final rollback testを含む
four-plus-five matrixはPASSし、final implementation reviewはfindingsなし。
final source/docsとquality reviewsはpending。

## Task 258B3M2B2B2C broad final-AST verification

broad fmt/Clippy/checker/runner/full workspace gates、focused `4/4`/`5/5`、
sibling `12/12`/`21/21` suitesはunchanged counts/hashesでPASS。final-AST
surface/semantic claim追加なし。independent final source/docs/quality
reviews、implementation commit、post-commit inventoryはpending。

## Task 258B3M2B2B2C final final-AST review status

independent final source/docs consistency/final qualityは**NO FINDINGS**。
全9 hard gates PASS、valid score `98/100`で、final-AST evidence/boundariesは
unchanged。pendingはcached-diff/staging audit、implementation commit、
post-commit inventory/fresh-next-task gatesだけ。

## Task 258B3M2B2B3A frozen final-AST contract

`ResolvedTypedAst`はapplication/structure fingerprints absent、set
fingerprint presentの`SetTerm(SourceSetTermId)`にexact allow/revalidate/
cloneだけを追加。precedenceはsource/AST、local resolver+label、
Tasks48/252/255/256/258 base、witness、atomic publication、final clone。
unsupported B3A statement/witness combinationとwitness/fingerprint
revalidation failureは`ResolvedTypedAstError::InvalidSourceStatement`。
precedence上先行するlower-stage mutationは`InvalidSourceSetTerm`、
`InvalidSourceAtomicFormula`を含む既存owner variantを保持。全failureは
partial stateなしでimmediate clean replay可能。lower
`SourceStatementWitnessError`はinternalのままで、final error variant/
display textを変更しない。

final cloneはwitness 1/names 0、`set-term#0`、existing optional debug fields
後のoptional set fingerprint、literal legacy bytes、empty semantic/proof/
goal/IRをpreserve。semantic result/public routeは追加しない。

## Task 258B3M2B2B3A implemented final-AST closure

`ResolvedTypedAst`はexact B3A statement + set-only witness tupleだけをallowし、
set/witness fingerprintsをrevalidateし、source set/statement/witness
handoffをcloneする。set/atomic defectは
`InvalidSourceSetTerm`/`InvalidSourceAtomicFormula` precedenceを保持し、
unsupported/stale upper combinationは`InvalidSourceStatement`のまま。
final clone/replay/debug/empty semantic/proof/goal surfacesのfrozen testsは
PASS。semantic result、error variant/text、public/active routeは追加しない。
2回目のsource/documentation consistency repeatとfinal documentation/
boundary rereadは**NO FINDINGS**で、crate plans記載のparent final
verificationはexact `39`-file scopeを含めPASS。independent final
read-only quality reviewは**NO FINDINGS**。全9 hard gates PASS、score
capなし、valid `98/100`（`20/20/15/14/10/10/5/4`）。記載済み
semantic/coverage deferralsはunchanged residual risk。pendingは
dedicated implementation commit、postcommit invariant verification、
fresh next-task inventoryだけ。

## Task 258B3M2B2B3B frozen final-clone boundary

final assemblyはexact B3B statement/witness profileをrecognizeし、
zero-edge Task-255 dependencyとset-only fingerprintをrevalidateして、
debug bytesまたはsemantic tablesを変更せずcloneしなければならない。
corrupt source ownership、label、Task-48/252/255/256/258 rows、witness
linkage、fingerprintはalready owning final errorでfailする。choice、
comprehension、`qua`、existential matching、proof acceptance、
Core/CFG/VC、その他すべてのsemanticsはabsentのままである。

## Task 258B3M2B2B3B implemented final-clone boundary

final assemblyはexact B3B source/resolver/lower rows、zero-edge
Task-255 handoff、set-only fingerprint、witness linkageを再検証してclone
する。corruption/hybrid/final-clone mutationsはfrozen checker/runner
matrixでfail closedを確認した。debug bytes、public API、semantic tables、
active routeはunchangedである。post-auth/stage-prefix guards後のfinal
implementation repeatは**NO FINDINGS**。source/documentation
consistency repeat、final documentation/boundary、independent qualityも
**NO FINDINGS**。全9 protocol hard gates PASS、score capなし、valid
`98/100`（`20/20/15/14/10/10/5/4`）である。

## Task 258B3M2B2B3C frozen final boundary

future B3C routeは`with_source_set_term_statement_witnesses`をreuseし、
exact Task-48/252/255/256/258/witness tupleを再検証してから
`ResolvedTypedAst`へcloneしなければならない。new graph edgeは
`Witness(0) -> SetTerm(0)`だけ。choice nonemptiness、stable choice
symbols、facts/proofsと全semantic tablesはemptyのまま。このdocumentation
taskはresolved-AST source/public APIを変更しない。

## Task 258B3M2B2B3C implemented final-AST closure

`ResolvedTypedAst`はexact B3C choice statement/witness/set-only tupleだけを
admitし、全Task-48/252/255/256/258とwitness fieldsをrevalidateして
authenticated source set/statement/witness stateをcloneする。lower
set/atomic error precedenceをpreserveし、stale/hybrid/non-generic-guard
upper stateはexisting statement errorでfailする。replayはdebug bytesと
empty semantic/proof/goal tablesをpreserveする。error/public route/
dependency/semantic resultは追加していない。

## Task 258B3M2B2B3D frozen final boundary

future final projectionはindependent exact B3D set-only fingerprint tuple
だけをacceptし、Task-48/252/255/256/258と
`SetTerm(0)` witness 1件をrevalidateしてauthenticated stateをcloneする。
stale/hybrid/family mixとwrong `QuaBase`/`QuaWidening` stateはexisting
errorsでfailする。schema、error、debug、dependency、active route、
semantic tableは変更しない。

## Task 258B3M2B2B3D implemented final-AST closure

`ResolvedTypedAst`はexact B3D qua statement/witness/set-only tupleだけを
admitし、Task-48/252/255/256/258とwitness linkage、set fingerprintを
revalidateしてsource-term/source-set-term/atomic/statement/witness stateを
cloneする。wrong `QuaBase`、request order、stale fingerprint、family
hybrid、occupied semantic tables、proof/expression coexistenceはexisting
owning errorsでfail closedする。lower set/atomic error precedence、
debug bytes、empty semantic/proof/goal tablesをpreserveし、schema/error/
public route/dependency/semantic resultは追加しない。frozen final/replay
testsとfield matricesはPASSし、test-sufficiency reviewは
**NO FINDINGS**。independent implementation reviewも**NO FINDINGS**。
bounded wording/status remediation後のsource/docs consistencyとboundary
repeatも**NO FINDINGS**。full workspace/CLI/count/hash final verificationは
PASS。

independent final read-only quality reviewは**NO FINDINGS**、全9 hard
gates PASS、no cap、valid `100/100`
（`20/20/15/15/10/10/5/5`）。CLI `23/0` warnings/errorsとlarge
repeated-test diff review volumeはnonblocking residual。staging/cached
diff、commit、post-commit/fresh-nextだけがpending。

## Task 258B3M2B2B3E frozen final boundary

future `ResolvedTypedAst` projectionはauthenticated exact B3E set-only tuple
だけをacceptする。tupleはfinal-LF 139-byte/60-node source fingerprint、
Task-255 `1/0/1/1/0/1/2` comprehension state、
`ComprehensionMapper -> Primary(2)`、ordered
`GeneratorSethood`/`ResultType`、Task-258 `1/2/2/2/2` base、
unnamed witness 1件と`Witness(0) -> SetTerm(0)`をclone-preservingに
revalidateする。

final owner partitionはTask-252 `{32,34,38,47,49}`、Task-255
`{16,40,41,43}`、Task-256 `{36,51}`、Task-258 `{54,56}`、B3E
`{45,46}`である。generator segment node `42`はunownedであり、final
projectionはこれをbinding/referenceまたは別owner rowとしてsynthesize
しない。stale/hybrid/family mixes、wrong generator/type/condition/edge/
request state、reordered requests、non-generic fallbackは既存errorを通じて
atomicにfailする。全five-family `120` ordersをindependentに保持する。

semantic tablesはemptyのままで、generator binding/capture、sethood/
result/numeric typing、goal/proof/fact/overload/Core/CFG/VC semanticsを
final projectionに追加しない。public API/error/debug grammar変更、
lower-stage prerequisite、active semantic creditはない。implementation
前のdocumentation-only final-boundary reviewは**NO FINDINGS**である。
future implementation reviewはseparate implementation taskに残す。

## Task 258B3M2B2B3E implemented final-AST inventory

`ResolvedTypedAst`はauthenticated exact B3E tupleだけをacceptし、
Task-48/252/255/256/258とone `SetTerm(0)` witnessをrevalidateして、
generator/type-site、set-term、atomic、statement、witness stateをcloneする。
wrong rows、partial/extra ownership、stale/hybrid/occupied semanticsはexisting
errorでfailし、replay/cloneはdebug bytesを保持する。

final ownerはprivate allowlistだけで7,270から7,272 linesへ増える。public
schema、error/debug、dependency、active route、semantic/proof/goal tableは
unchanged。implementation reviewは**NO FINDINGS**。bounded design
correction後のfinal source/docs consistencyも**NO FINDINGS**で、full
verificationはPASSした。independent final qualityは**NO FINDINGS**、
全9 gates PASS、valid `100/100`。staging/post-commit gatesは
implementation commit
`e4479691db3b0a8785bb16e94d386bd71a394274`でcloseし、fresh inventoryは
Task 258B4Aをselectした。

## Task 258B4A frozen final boundary

final constructionはoptional composite-formula/formula-composition statement
fingerprintsをinstalled Task-257B1 handoffに対してrevalidateし、exact
`Composite(0)` statement/candidate links、single-owner resolver provenance、
atomic statement familiesとのmutual exclusionを検証する。clone/debug outputは
complete syntax-free transactionをpreserveする。

contexts、types、facts、coercions、initial obligations、diagnostics、
theorem acceptance、proof stateはemptyのままである。stale、partial、
hybrid、cross-family tupleはfinal publication前に
`ResolvedTypedAstError::InvalidSourceStatement`でfailし、replay可能な
stateを保持する。repeated read-only documentation reviewは
**NO FINDINGS**である。independent final qualityは全9 hard gatesを
capなし、valid `100/100`でPASSした。remainingはstaging、commit、
post-commit inventoryだけである。

## Task 258B4A implemented final boundary

final constructionはclone前にinstalled composite、composition、statement
fingerprints、exact `Composite(0)` statement/candidate links、
resolver-backed owner、lower rootless-arena contract、atomic-family
exclusionをrevalidateする。statement corruptions 19件、absent-lower tuples
3件、stale/hybrid inputs、occupied incompatible stateは
`ResolvedTypedAstError::InvalidSourceStatement`でpublicationなしにfailし、
replayを許す。successful clone/debug bytesはstableで、contexts、types、
facts、coercions、obligations、diagnostics、theorem acceptance、proof stateは
emptyのままである。

## Task 258B4B frozen final boundary

final constructionがcomposite-statement familyをacceptできるのはmatched
B4A/Task-257B1またはB4B/Task-257B2 tupleとしてだけである。B4Bはclone前に
exact 124-node rootless lower arena、Task-252/256/257/B2
profiles/fingerprints、`0..165`/origin `[2,0]`のresolver owner 1件、upper
`1/1/1/0/1`、両`Composite(0)` linksをrevalidateする。final dispatchは
shared cardinality shapeではなくexact crate-private statement predicates
2件を使用する。B4B statementはB4AのTask-257B1 branchへ、またはその逆へ
決して入ってはならない。

全cross-profile hybrid、stale fingerprint、missing lower handoff、
rooted/relocated coherent near miss、atomic-statement coexistence、occupied
semantic stateは`ResolvedTypedAstError::InvalidSourceStatement`でpartial
outputなしにfailし、replayを許す。B4A bytesと全lower-owned error
precedenceはunchangedである。final-AST schema、debug grammar、semantic
tableは追加しない。

## Task 258B4C frozen final boundary

final constructionがB4Cをadmitできるのはexact Task-257B3 lower handoffsと
exact B4C statement identityが揃う場合だけである。private 139-byte
source、66-node/root-65 Surface identity、raw resolver origin `[2,1]`、
contribution 0 anchor `0..18`、enriched `1/1/1/1/0`のraw
authenticationはrunner selectorと`SourceStatementProducer`が所有する。
final constructionはclone前にtyped ASTが保持するproducer-authenticated
statement handoff rows/identity、matched lower fingerprints、rootless
lower arena、binding `4/4/0`、primary `6/6/0`、atomic
`3/0/0/0/0/0/0/6/6`、composite `3/0/1/3/3/2/6`、composition
`3/6`、lower partition 24 sites、upper `1/1/1/0/1`をrevalidateする。

statement 0/candidate 0は両方`Composite(0)`をtargetにし、context 0は
exactly `[0]`をexposeし、input factsはempty、theorem node 62だけが
upper-owned Surface nodeでなければならない。final dispatchはexact
B1/B4A、B2/B4B、B3/B4C pairingだけをrecognizeする。cross-family
hybrids、stale/partial handoffs、rooted/relocated arenas、altered
ownership、active atomic statement family、occupied semantic state、
lower-selector mismatchはexisting
`ResolvedTypedAstError::InvalidSourceStatement` boundaryでpublication前に
failし、replayを許す。

このfrozen pathはfinal-AST schema、public API、debug/error grammar、
fact、theorem-acceptance、proof、IR tableを追加しない。separately
committed lower-selector prerequisiteを完了してからfinal projectionを
implementする。

## Task 258B4C 実装済み Final Boundary

final assembly は typed AST が exact B3 lower handoff、B4C upper row、
matching fingerprint、rootless exact 66-node arena、`24/1/41` ownership を
保持する場合にだけ B4C を認識する。clone 前に全 anchor と normal recovery
state を検証する。partial/cross-family state、stale fingerprint、
relocation、atomic statement coexistence、occupied semantic table は
partial publication なしで fail し、deterministic replay を許す。

final AST は clone-preserving syntax/provenance projection のままである。
checked formula、statement semantics、proof、proof node、terminal goal は空で、
schema と public API は変更しない。

## Task 258B5A frozen final-assembly boundary

final assemblyはexact 93-node/root-92 arena、matched B5A base/reference
fingerprints、20/73 ownership、label scope `[0]`、citation scope `[0,1]`、
resolver sole keyed node 82 -> label key 0をtyped ASTが保持する場合だけ
B5Aを認識できる。preliminary resolverはkeyed nodeなし、final resolverは
exact one resolved idを持ち、diagnostic/name/import/export entryはない。

clone/replayはpublication前に全row/range/origin/ordinal/recovery state、
scope-prefix relation、empty semantic tableを再検証する。partial/
cross-profile install、stale fingerprint、relocation、recovery、wrong
contribution/keyed node、Task-248/other-family coexistence、occupied
semantic outputはatomically failする。B1 debug bytesとpublic APIは
unchangedである。

## Task 258B5A implemented final-assembly boundary

final assemblyはtyped ASTがexact matched base/reference profile、全dependency
fingerprint、93-node/root-92 arena、`20/73` ownership、label scope `[0]`、
citation scope `[0,1]`、resolver node 82からlabel key 0へのmappingを保持する
場合だけfrozen B5A transactionをrecognizeする。clone前に全resolver node
kind/range/child order/recovery state/origin/ordinal/contribution/
scope-prefix relationを再検証する。

unchanged B1 same-scope transactionとB5A ancestor/descendant transactionだけが
admitted reference profileである。partial、cross-profile、stale、
relocated、recovered、wrongly keyed、semantically occupied stateはpublication
前にfailし、replay可能なままにする。checked formula、accepted statement、
proof、proof node、terminal goal、fact、downstream IRはemptyのままで、
public schema/APIを変更しない。

## Task 258B5B frozen imported final-assembly boundary

final assemblyはlower opt-in environmentがexact `8/1/1/3/1`で、typed ASTが
matched base `1/2/2/2/2`、reference `0/1`、全dependency fingerprint、
57-node/root-56 arena、`8/49` ownershipを保持する場合だけB5Bをrecognize
できる。resolver node 48だけがkeyed resolved nodeで、replayはresolved
import 1、resolved label reference/id 1、export/name reference/diagnostic 0。

resolved import id 0はunkeyed `ImportAliasDecl` node 29がownし、range
`7..27`、spelling `import parser.type_fixtures;`、alias `None`、
`<package>::parser.type_fixtures`へresolveする。current-source/
current-module originはanchor `7..27`、path `[0]`、import edgeなし、
normal recovery。nodes 28/29/30はexact Surface identityのunkeyed
`NotApplicable` nodeを維持し、node 48だけがlabel key 0を持つ。imported
projection originは独立にcurrent source、declaring imported module、
anchor `7..27`、path `[1,0]`、import edgeなし、normal recovery。
reference originはcurrent source/current module、anchor `136..139`、
path `[48]`、import edgeなし、normal recovery。

immutable cloneはimported/public/exported theorem projection、
`target=Imported`、`SimpleImported`、scope `[0]`、全origin/module/
namespace/contribution/anchor/structural path/range/ordinal/node kind/child
order/recovery state/resolver keyを再検証する。exact B1/B5A local behaviorを
preserveし、全B1/B5A/B5B cross-pair、partial/stale/recovered state、
occupied semanticsをpublication前にrejectする。

checked formula、fact、accepted statement、proof、proof node、goal、status
propagation、downstream IRはemptyのまま。public checker surface changeは
crate planでfreezeしたcitation-target enum/field/getterとimported citation
kindだけ。

## Task 258B5B implemented imported final-assembly boundary

final assemblyはresolved import owner/range/spelling/alias/result、imported
projection origin/public-exported theorem identity、reference candidate
origin、node 48 kind/key、resolution result、citation target/kind/ordinal、
complete `8/49` ownership partitionをindependently再検証した後だけtyped
B5B installationをcloneする。stale/relocated/recovered/partial/
wrongly-keyed/B1-B5A-B5B cross-paired stateはpublication前にfailしvalid
replayを保持する。

cloneはbyte-identical B1/B5A local behaviorとexact B5B
`label_node=absent`/`source=imported` debug schemaをpreserveする。checked
formula、fact、accepted statement、proof、proof node、terminal goal、
diagnostic、status propagation、downstream IRはempty。public final-owner
schema/semantic APIを変更しない。

## Task 258B5C frozen final-assembly exclusion

両B5C candidateは`has_unresolved = true`の`UnresolvedLabelRef`へresolve
する。このためsource-statement handoffがtyped installation前にrejectし、
final assemblyがcloneするB5C stateはない。R-032A structural mirrorは
`NodeResolutionState::NotApplicable`/no keyのままでsemantic successでは
なく、`LabelResolution::Resolved` result、label/citation/reference DTO、
checked formula、fact、accepted statement、
proof node、goal、diagnostic carrier、downstream IR rowはない。

final assemblyはactive resolver failureをempty/partial checker profileへ
変換してはならない。B1/B5A/B5B clone validationと全existing debug byteは
unchangedで、two negative sourceの全Surface nodeはsyntax-ownedのまま。

default-deny R-032B traversalはexact
`Root -> CompilationUnit -> ItemList -> direct TheoremItem -> direct
ProofBlock`、Root/CompilationUnit exact-one normal structural child、
direct-normal theorem scanをrequireする。other item childとexcluded
formula/token/wrapper、
unsupported/recovered/malformed、qualified/grouped/bulk、template nodeに
collected row/ordinal/descentがなく、`ResolvedTypedAst` ownerにはなれない。
positive-edge/forbidden-relocation/mixed-list testはresolver-ownedのまま。

environment module、projection module/namespace/contribution、
contribution cardinality/id/kind/record module/source idのrunner mutationは
`proof_scope_input`で停止し、final assemblyへ到達したりauthenticated
confinementへreclassifyされたりできない。

## Task 259 Frozen Final Handoff

future final assemblyはone optional `SourcePredicateDefinitionHandoff`を
持つ。exact Task-248/249/252/256 lower handoffとatomically installed complete
obligation tableがある場合だけappearできる。`ResolvedTypedAst`はpublish前に
four fingerprint、five dense table、resolver-derived predicate origin、全
cross-family ID、sole `Pending` `PredicatePropertyCorrectness` rowを
revalidateする。

clone/debug renderingはexact ID、order、string、anchor、dependency bytesを
preserveする。final assemblyはhandoffをreplaceせず、guard/propertyを
reinterpretせず、justification subtreeをconsumeせず、FOL goalをconstruct
せず、pending obligationをfact/proof/VC/accepted definition/axiomへ変えない。

## Task 248 Two-Parameter Profile-B Final Clone

final assemblyはunchanged source-context clone pathを使う。typed validationが
exact `1/2/2/2/2/2/0` stateをretainしexisting handoff invariantをrevalidateした
場合だけProfile Bをacceptし、separately replaceable runner inputを追加しない。
clone/debug order/bytesはdeterministicで、invalid Profile-B stateをrepairまたは
partial publishできない。本lower profileはTask-249+、Task-259、fact、
obligation、proof、semantic rowを作らない。

## Task 260 Final Functor-Definition Ownership

`ResolvedTypedAst`はtyped-owned finalization pathだけからcloneされるoptional
immutable Task-260 handoff 1件を持ちます。five table、全lower fingerprint、
baseline-plus-appended obligation table、Task-259 handoffと
`PredicatePropertyCorrectness` baselineのrequired absenceをrevalidateします。
second final obligation input/getterやresolver reconstructionはありません。
clone/debug orderはdeterministicで、failureはrepair/renumber/partial publish
できません。

Task 260 absent時の`FunctorExistence` / `FunctorUniqueness`はorphanとして
rejectします。present時はcorrectness tableがlinkするfinal row 2件だけをrequireし、
additional functor-kind rowをrejectします。

## Task 249R definition-return clone addendum

`ResolvedTypedAst`はTask-249R input/fieldを追加しない。final assemblyは
`TypedAst` installationでalready revalidatedされたimmutable handoffをtrustし、
definition-return table/rootをclone-preserveするだけである。final debugは同じ
combined source-type fingerprintをexactly once含み、resolver payloadからreturn
rowをreconstructしない。

## Task 249M mode-RHS clone addendum

`ResolvedTypedAst`はTask-249M input/field/getterを追加しない。single typed-owned
source-type handoff内でrevalidated済みmode-RHS tableをclone-preserveするだけで、
combined fingerprintはexactly once。resolver spellingからRHSをreconstructせず、
inhabitation/expansion/sethood/acceptance/proof/fact semanticsを与えない。

## Task 249M active clone boundary

implemented mode-RHS tableはexisting typed-owned source-type handoff内に保持する。
`ResolvedTypedAst`はfield/inputを追加せずvalidated `2/3/0/0/1`
fingerprintをexactly once clone-preserveする。test 4件はfinal semantic result
tableがemptyであることを確認する。

## Task 262 active final mode-definition ownership

final assemblyはoptional Task-262 handoffをtyped ownerだけからclone-preserveし、
exact Task-248/249/249M fingerprint、dense table 6件、baseline count、property-to-
obligation link、single pending `Sethood` suffixをrevalidateする。
`ResolvedTypedAstInputs`はreplaceable Task-262 fieldを追加せず、新規public
projectionはread-only getterだけである。

older sibling installerが検出できないreverse install orderを含む全mixed
Task-259/260/261/262 stateをrejectする。handoffなしでは
`source.definition.mode` domainのgoal/provenanceをorphanとしてrejectし、unrelated
existing-kind `Sethood` rowは許可する。final assemblyはinhabitation requestへ
answerせず、acceptance、expansion/sethood fact、proof、Core、control flow、VCを
publishしない。
## Task 249S standalone member-type clone addendum

Task 249Sはfinal field/semantic routeを追加しない。`ResolvedTypedAst`はexisting
`source_type` fieldでvalidated `SourceTypeApplicationHandoff`をclone-preserveする
だけである。final assemblyはlower owner row 4件からexpression metadata、
candidate、coercion、fact、diagnostic、formula、statement、proof、obligation、
acceptance、Task-263 structure resultを生成しない。

## Task 249S active clone-preservation result

repeated final assemblyはexact immutable source-type handoffとdeterministic
debug fingerprintをvalue equalityで維持する。final field/installer、
semantic projection、diagnostic、obligation、fact、proof、acceptance outputは
追加しない。

## Task 269B active final replay increment

final assemblyはexisting getter/seven phasesでexact B3M1 handoffをclone-preserve
してrevalidateする。valid replayはdeterministicで、orphan/partial/stale/
cross-profile/arena-corrupt inputはexisting proof-local errorでfailする。proof、
goal、fact、obligation、diagnostic、downstream semantic tableはすべてemptyまたは
unchangedのまま。

## Task 263 frozen final ownership

final assemblyはfuture structure-definition handoffを`TypedAst`からcloneする場合だけ
取得する。complete source-type fingerprint、`2/4/1/2/0` graph、zero coherence、
unchanged obligationをrevalidateしてgetter 1件を公開する。
`ResolvedTypedAstInputs`にreplacement fieldはなく、new failureは
`ResolvedTypedAstError::InvalidSourceStructureDefinition`だけである。reverse mixed
Tasks-259--262 stateはfail-closedで、semantic table/accepted resultを生成しない。

final validationはcurrent complete obligation tableをfrozen countだけでなくprivate
byte-equal snapshotと比較するため、same-length changed rowもfailする。snapshot bytesは
stable `debug_text()`に入らず、replaceable final inputで迂回できない。

## Task 263 active final ownership

final assemblyはTask-263 handoff、complete unchanged obligation table、private
baseline snapshot、lower fingerprint、全row、zero-coherence profile、arena ownershipを
clone-preserve/revalidateする。mixed Tasks 259--263をrejectし、immutable source
transport以外のsemantic resultをpublishしない。

## Task 248P property context final ownership

Task 248Pはfinal input/assembler branchを追加しない。`ResolvedTypedAst`は`TypedAst`へ
install済みのProfile C handoffだけをclone-preserveし、existing source-context
validation/debug positionをreuseする。callerはhandoffをreplaceできず、recovered inputを
promoteできない。property payload/provenanceと全semantic resultはTask 264までabsent。

## Task 248P active final ownership boundary

implemented source-context handoffはexisting `TypedAst` clone pathだけでownedされる。
final input/assembler branch/getter/replacement path/debug field/semantic resultは追加せず、
recovered Profile Cはfinal assemblyへ到達不能のままである。

## Task 264 frozen final ownership

ResolvedTypedAstはread-only `source_property_implementation()`だけを追加する。
Finalは全lower fingerprintとstyle別obligation suffix（Means two、Equals zero）を
revalidateし、orphan/extra/mismatch、Task259 coexistenceを
`InvalidSourcePropertyImplementation`でrejectする。Transportをcloneするだけで
expr metadata/checked formula/fact/proof/diagnostic/acceptance/property value/IR/VCを
追加しない。DocsおよびTask249PI段階ではfinal ownerを追加しない。

## Task 249PI final ownership boundary

Task 249PIは`ResolvedTypedAst` field/constructor input/getter/serializerを追加しない。
existing `source_type` handoffをcomplete `source-type-application-debug-v1` fingerprintと
exact `1/3/0/0/0/2` profileのまま1回clone-preserveする。final installationはexisting
Typed source-type validationでlower driftをrejectする。property identity/return
association/initial obligation/diagnostic/fact/proof/acceptance/Task259 dataはseparate owner
までabsent。

## Task 249PI implemented final ownership

final assemblyはexact combined source-type handoffをclone-preserveし、修正した
orphan-member installation shapeをrejectする。final field/getter/semantic result/
Task259/264 ownershipは追加しない。

## Task 264 active final ownership

final assemblyは全frozen lower fingerprint/row/arena identity/style-specific obligation
suffixをreplayした後だけtyped property-implementation handoffをclone-preserveする。
orphan/extra property kindとTask259--263 sibling coexistenceを
`InvalidSourcePropertyImplementation`でrejectする。read-only getterはtransportだけを
publishし、expression metadata/checked formula/fact/diagnostic/proof/acceptance/
property value/Core/CFG/VCは不変である。

## Task 269A frozen final ownership

final assemblyはtyped `SourceProofLocalDeclarationHandoff`をone read-only
getter、`ResolvedTypedAstInputs` replacement fieldなしでclone-preserveする。
publish前に全lower/final fingerprintとbinding transitionを再計算する。orphan、
half-installed、stale、corrupt valueは`InvalidSourceProofLocalDeclaration`。

final ownerが公開するのはdefinition-site associationとextended binding
environmentだけ。expression metadata、overload/coercion/cluster、checked
formula、statement semantics、proof/goal、obligation、diagnostic、fact、Core、
CFG、VCはemptyまたは不変。

## Task 269A active final ownership

final assemblyはoptional handoffをclone-preserveし、publish前にexact lower/final
transactionをreplayする。top-level preflightはlegacy lower validatorより前にterm、
atomic formula、statement、witness、forbidden referenceのpresenceをcheckするため、
全half-installed injectionはdedicated final errorでfailする。orphan/same-length
stale injectionも同じerrorでfailし、valid clone/replayはdeterministic、全deferred
semantic tableはemptyのまま。

## Task 269B frozen final replay increment

final assemblyはsame getter/phase 7件でexact B3M1 bundleをreplayする。top preflightは
term/atomic/statement/witnessを要求しstatement referenceを禁止する。cross-profile/
stale/partialはexisting dedicated errorでreject。field/error/node role/semantic tableは
追加しない。

## Task 269CP final-owner exclusion

Task 269CPは`TypedAst`/`ResolvedTypedAst` field、getter、installer、debug、replay
phase、final cloneを追加しない。lower outputは`mizar-test`内に留まる。Task 269Cは
final ownerをseparately freezeし、このprerequisiteから推測してはならない。

## Task 269C frozen final owner

`ResolvedTypedAst`はempty node/semantic profileに対するcomplete revalidation後だけnew
binding-only handoffをclone-preserveする。read-only getterとdeterministic debug sectionを
追加するがexpression/candidate/formula/statement-semantic/proof/goal/obligation/fact/
diagnostic rowは追加しない。orphan/cross-family/stale/semantic-coexistenceをrejectし、
Task-269A/B final byteは不変。

## Task 269C active final ownership

final assemblyはcomplete replayとclone-preservationを実装した。dedicated testsは
orphan/stale/cross-family/nonempty semantic inputをrejectし、valid cloneはexact
handoffとempty semantic profileをpreserveする。

final optional handoffはstack-size stabilityのためprivate boxed storageとし、
freeze済みgetter/debug/clone/replay behaviorを維持する。

## Task 269CT final composite replay

`ResolvedTypedAst`はboxed optional `SourceProofLocalLetTypeHandoff`とconst getterを追加し、
authenticated typed ownerからだけcloneする。exact typed node 3件をsource-preserved role
`source.proof-local.let.type`でone-for-one replayし、direct source-type/Task-269C fieldと
resolved semantic tableはempty。malformed/duplicate/occupied/semantic-nonempty profileは
`InvalidSourceProofLocalLetType`。

## Task 269CT implemented final replay

final assemblyはarena construct前にcompositeをclone/revalidateし、exact 3-node Typed profileと
otherwise empty source/semantic/overload stateを要求する。Task-specific predicateは
`node_hints.is_empty()`を要求し、3 node全てを`source.proof-local.let.type`へone-for-one map。
statement-transport hint/nonempty expression metadataは
`InvalidSourceProofLocalLetType`でatomic fail。

## Task 269GP no-final-owner boundary

269GPは`ResolvedTypedAst` sibling/node hintをpublishしない。existing 269C/CT replayと
全final semantic exclusionはbyte-identical。

implemented runner routeもfinal owner/serializer変更なしをconfirmする。

## Task 269GS no-final-owner reconciliation

lexical scopeのresolveは`ResolvedTypedAst` field/final ownerをinstallしない。future 269G
contractがcanonical block ruleからbinding handoff/replayを決め、269GTはseparate type owner。
Task269GSでは全semantic table不変。

## Task 269G final owner

`ResolvedTypedAst`はvalidated `SourceProofLocalGivenBindingHandoff` 1件をread-only getterで
clone-preserveし、duplicate/stale/cross-family/semantic coexistenceを
`InvalidSourceProofLocalGivenBinding`でreject。node/semantic tableなし、debugはexisting
`let` binding/type slot後。

## Task 269G active final ownership

final assemblyはcomplete Given binding handoffをrevalidate/clone-preserveする。dedicated final-
replay testsはstale/cross-family/node-hint conflictをrejectし、valid replayはexact lexical
binding/empty semantic profileを保持する。Typed installer testsは別にnonempty semantic table
6 familyをfinal assembly前にreject。private boxed storageはfrozen getter/installer/clone/debug
contractを変えずaggregate stack sizeを保つ。

## Task 269GT frozen final owner

final assemblyはboxed `SourceProofLocalGivenTypeHandoff` 1件をclone/revalidateし、exact 3-node
arenaを`source.proof-local.given.type` roleでmap、stale/sibling/node-hint/semantic coexistenceを
reject。direct Given binding/generic source-type slotはcomposite ownershipのためempty。semantic
tableをpopulateしない。

### Task 269GT implemented final owner

Resolved assemblyは`SourceProofLocalGivenTypeHandoff`だけをcloneし、occupied/semantic inputをrejectし、3 typed nodesを`source.proof-local.given.type` roleへ1対1 mapする。direct source-type/Given-binding/Let ownerと全semantic tableは空。

## Task 269GUP final-owner exclusion

GUPは`ResolvedTypedAst` field/getter/error/node role/clone replay/final tableを追加せずassemblyは
byte-identical。GUPT/GUはGUP dependencyを順にconsumeした後だけownerをfreezeできる。
### Task 269GUP binding profile 実装状況

凍結済みの6ファイル transactionとchecker/runner各4件の正確なtestを実装した。libraryは`502/564`、checker/runner productionは`30/172531`と`37/74826`で、path hashは不変、content hashは`e0342952a01a0b379cf7b06ad243cd40a1656e940480196323cf43fbe7d8f7c5` / `8fe7c8c0b7e855e5113f3830873e133f42c8048a3272055e2fddd5ebd9cbb1bc`である。

閉じるのはdormant private lexical-binding evidenceだけで、active corpus、trace、type、term/use、condition/fact、goal/proof、obligation、diagnostic、CLIのcreditは0のままである。次はTask 269GUPTであり、Task 269GU、capture、Task 270は引き続きdeferする。
