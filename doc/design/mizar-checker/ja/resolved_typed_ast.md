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
