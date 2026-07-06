# mizar-checker: overload resolution

> canonical language: English. 英語版:
> [../en/overload_resolution.md](../en/overload_resolution.md).

## purpose

`overload_resolution` は、type checking と registration resolution が typed site、
candidate group、recorded fact、coercion candidate、replayable registration trace を
生成した後の phase 8 checker behavior を仕様化する。これは以下を refine する。

- [architecture 05](../../architecture/ja/05.overload_resolution.md);
- [spec chapter 18](../../../spec/ja/18.templates.md) templates と template inference;
- [spec chapter 19](../../../spec/ja/19.overload_resolution.md) ordinary overload selection、
  `qua`、`redefine`;
- [`typed_ast.md`](./typed_ast.md) typed site、type fact、coercion candidate;
- [`registration_resolution.md`](./registration_resolution.md) activated registration と
  gate boundary;
- [`cluster_trace.md`](./cluster_trace.md) recorded closure / reduction trace fact。

task 21 は documentation-only である。Rust source、新しい language behavior、source
extraction、artifact writer、proof acceptance は追加しない。task 22 は explicit payload 用の
checker-local site/candidate collection data layer を実装する。task 23 はそれらの payload 上の
checker-local template expansion data layer を実装する。task 24 は explicit recorded-evidence
payload 上の checker-local viability filtering data layer を実装する。tasks 25-26 が残りの
named section を実装する。task 28 は後で `resolved_typed_ast.md` が仕様化する final
`ResolvedTypedAst` data shape を組み立てる。

## boundary

`overload_resolution` が所有するもの:

- checker-owned typed payload から final overload site を収集すること;
- declaration、root、template、provenance metadata を持つ concrete candidate record;
- ordinary candidate ordering より前の template expansion;
- 既に記録された type fact、coercion candidate、source-written `qua` 上の viability
  filtering;
- viable ordinary root 上の per-site specificity graph;
- unique ordinary root selection と same-root refinement join;
- 後続 phase に明示する必要がある proof-free widening view の insertion record;
- stable diagnostics と candidate list を持つ failed-site preservation。

`overload_resolution` が所有しないもの:

- name lookup、import visibility、declaration signature collection、global `SymbolEnv`
  scan;
- type-expression normalization、term/formula inference、raw syntax からの新しい
  candidate discovery;
- cluster closure、registration activation、reduction replay、新しい fact derivation;
- `VcId`、proof discharge、accepted verifier status、kernel replay;
- return-type-based ordinary overload selection;
- checker diagnostic code-space が外部のままである間の stable public diagnostic-code
  allocation。

## inputs and outputs

この phase は checker-owned payload を消費する。

```rust
struct OverloadResolutionInput {
    typed_ast: TypedAst,
    sites: Vec<OverloadSiteInput>,
    candidates: Vec<OverloadCandidateInput>,
    facts: TypeFactTable,
    coercions: CoercionTable,
    trace: ResolutionTrace,
}
```

input は既に scope-filter 済みでなければならない。current-module candidate は use site より
前に visible であり、imported candidate は resolver に従って visible/exported であり、
same-signature declaration conflict は diagnostic または blocked candidate group として既に
表現されていなければならない。checker は diagnostic のために provenance を検証してよいが、
resolver global を検索して candidate set を広げてはならない。

この phase は次を生成する。

```rust
struct OverloadResolutionOutput {
    sites: OverloadSiteTable,
    candidates: OverloadCandidateTable,
    template_expansions: TemplateExpansionTable,
    viability: ViabilityTable,
    specificity: SpecificityGraphTable,
    results: OverloadResultTable,
    inserted_views: InsertedViewTable,
    diagnostics: OverloadDiagnosticTable,
}
```

output は task 28 が `ResolvedTypedAst` に射影するまで checker-local である。id は output 内で
deterministic だが、proof id、artifact id、cross-edit cache key ではない。

## site and candidate collection

overload site は、final semantic definition がまだ選択されていない typed use である。

```rust
struct OverloadSiteInput {
    key: OverloadSiteKey,
    owner: TypedSiteRef,
    source_range: SourceRange,
    kind: OverloadSiteKind,
    name: OverloadNameKey,
    arguments: Vec<TypedSiteRef>,
    expected: Option<NormalizedTypeId>,
    source_qua: Vec<SourceQuaView>,
    recovery: OverloadSiteRecovery,
}
```

site kind は functor、predicate、attribute、mode、selector、structure field、template
name、scheme/theorem application を含む。ただし resolver と parser が必要 payload を公開する
場合に限る。unsupported または missing source role は `external_dependency_gap` または
`deferred` として分類し、raw syntax から推測しない。

各 candidate は既に resolve された declaration identity を記録する。

```rust
struct OverloadCandidateInput {
    site: OverloadSiteKey,
    symbol: SymbolId,
    ordinary_root: SymbolId,
    declaration_kind: CandidateDeclarationKind,
    parameters: Vec<NormalizedTypeId>,
    result: Option<NormalizedTypeId>,
    origin: CandidateOrigin,
    template: Option<TemplateCandidatePayload>,
    coherence: Option<CoherenceStatus>,
    provenance: CandidateProvenance,
}

enum CoherenceStatus {
    Accepted,
    Pending,
    Rejected,
}
```

`OverloadSiteInput.key` は collection payload 用に caller が供給する stable key である。
`OverloadCandidateInput.site` はその key を参照する。output の `OverloadSiteId` は
canonical site sorting 後に割り当てられる collection-local id である。未知の site key を
参照する candidate は診断され、挿入されない。

task 22 では、site provenance を `owner`、`source_range`、`name`、`arguments`、
`source_qua`、`recovery` の組として扱う。candidate provenance は明示的な
`CandidateProvenance` payload である。どちらも deterministic debug rendering と diagnostic
のために保持する。collector は raw syntax や resolver global を scan して欠けた provenance を
創作しない。

`result` は metadata と same-root refinement join のために保持する。ordinary root selection には
決して参加しない。`redefine` candidate は refine する ordinary root を指さなければならない。
same-root redefinition は競合する root ではない。pending または rejected coherence は
diagnostic のために保持してよいが、accepted coherence だけが redefinition を active にできる。

site と candidate の canonical ordering は source id、source range、typed owner key、
declaration kind、ordinary root、symbol id、template instantiation key、import/source
provenance、explicit declaration order、最後の duplicate tie-break として candidate-local id を
使う。site 内の candidate は declaration kind、ordinary root、symbol id、template
instantiation key、provenance、explicit declaration order、candidate-local id で sort する。
hash-map iteration、worker order、import read order、diagnostic insertion order は rendered
output に影響してはならない。

### task 22 collection data layer

`src/overload_resolution.rs` は explicit な `OverloadSiteInput` と
`OverloadCandidateInput` vector に対する `OverloadCollectionOutput::collect` を実装する。
これは以下を行う。

- canonical sorting 後に deterministic local `OverloadSiteId` と
  `OverloadCandidateId` を割り当てる。
- site provenance、candidate provenance、source-written `qua` metadata、template payload
  metadata、coherence metadata を保持する。
- duplicate site key と、未知 site key への candidate link を検証する。
- duplicate key、unknown link、unsupported role、recovery state に対し、問題または関連する
  site / candidate input provenance を保持する stable checker-local diagnostic を発行する。
- unsupported な site / candidate role を deferred にする。
- scope/visibility filter 済み candidate set を供給された通りに保持する。

task 22 は意図的に `TypedAst` walk、`SymbolEnv` scan、opaque resolver shell parse、template
expansion、viability check、specificity graph 構築、root selection、view insertion、
`ResolvedTypedAst` projection を行わない。これらは tasks 23-28 または MC-G027 の
external/deferred work である。

## template expansion

template expansion は ordinary viability と specificity より前に実行する。

template payload を持つ candidate ごとに:

1. explicit template argument を template parameter list に照合して分類する。
2. argument が省略されている場合、caller が供給した omitted-argument inference payload を
   検証する。その evidence は task 23 より前に公開済みの exact argument type または
   mode-hierarchy constraint metadata でなければならない。
3. constrained template parameter を、caller が供給した explicit constraint evidence status と
   site で visible な fact に照らして検査する。
4. missing、contradictory、ambiguous、unsupported、または required constraint evidence が欠けた
   inference を reject する。
5. 成功した template を concrete candidate に instantiate し、tie-breaking と diagnostic 用に
   template origin を保持する。

task 23 は omitted-argument inference を `TypedAst`、fact table、cluster closure、resolver
global から計算しない。template parameter list、template argument、omitted-argument inference
payload、source-`qua` argument status、constraint-evidence status という explicit payload field を
検証する。omitted-argument template inference では cluster expansion を使わない。source-written
template `qua_arg` は typed/coercion payload が proof-free inheritance widening として公開する
場合だけ受け入れる。predicate、functor、scheme、algorithm template role は、parser、
resolver、checker-owned payload が具体的な role と parameter mapping を公開しない限り
deferred のままである。non-template candidate は、concrete parameter vector が otherwise
equivalent の場合に限って template-derived candidate に勝つ。template 宣言制約の
厳しさは、展開後の tie-breaker ではない。未解決の同値な template-derived root は
曖昧なまま残る。これは task 37 の Phase B overload-selection 決定だけを記録
する。mizar-core task 26 / template-audit F7 は、別個の Phase A 省略 template
argument 推論決定性規則を記録済みである。

template expansion diagnostic は skipped template candidate と reason を保持する。failed
template expansion は ordinary candidate にならない。
`such that` template constraint に visible accepted evidence が欠けている場合、candidate は
constraint-evidence reason 付きで reject または defer される。checker は overload resolution
中に新しい proof obligation を作成したり constraint を仮定したりしてはならない。parser/resolver
payload が constraint または label mapping を公開しない場合、その candidate は expand せず
MC-G027 の下で deferred とする。

### task 23 template expansion data layer

`TemplateExpansionOutput::expand` は `OverloadCollectionOutput` を消費し、task 22 が保持した
explicit `TemplateCandidatePayload` metadata だけを検証する。成功した template candidate は
template payload を外し、`CandidateOrigin::TemplateDerived` を保持した concrete
`OverloadCandidateTable` へ copy される。non-template candidate はそのまま copy される。
reject または defer された template candidate は concrete candidate table へ入らず、
`TemplateExpansionTable` に diagnostic、source candidate id、成功した substitution list、
stable failure reason とともに保持される。task 23 は normalized signature id を rewrite しない。
candidate の normalized parameter/result metadata は caller が供給した concrete signature であり、
expansion table はそれを concrete として公開する根拠になった template argument mapping を記録する。

task 23 は新しい site/fact state から omitted parameter を infer せず、cluster expansion、
viability check、specificity comparison、ordinary root selection、view insertion、
unsupported parser/resolver template role の合成を行わない。

## viability over recorded facts

concrete candidate は、すべての argument が phase 8 より前に既に記録された proof-free widening
で対応する parameter として view できる場合に viable である。

- identical normalized type;
- unique structure または mode inheritance upcast;
- 既存 `TypeFactId` と phase-7 closure によって正当化される attribute weakening;
- widening として検証済みの source-written `qua`;
- template expansion 後の explicit template argument viability。

viability checker は `TypeFactTable`、`CoercionTable`、`ResolutionTrace` を読んでよい。
新しい type inference、cluster fact derivation、registration activation、obligation creation、
proof acceptance を実行してはならない。fact が欠けている場合、candidate を stable な説明付きで
reject または block する。

candidate を viable にできるのは consumable evidence だけである。

- known fact は degraded、rejected、contradicted、obligation-pending でない場合に使える;
- assumed fact は overload site でその assumption を visible にする local context を通じる場合だけ
  使える;
- coercion record は available widening または source-written `qua` view を表す場合だけ使える。
  narrowing、missing evidence、blocked obligation には使えない;
- trace-derived fact は trace step が accepted であり current typed site から visible な場合だけ
  使える。

pending obligation、degraded recovery fact、rejected fact、out-of-scope assumption、
unaccepted registration evidence は candidate を viable にできない。

```rust
enum CandidateViabilityStatus {
    Viable { views: Vec<ArgumentViewPlan> },
    Rejected { reasons: Vec<CandidateRejection> },
    Blocked { reason: CandidateBlockedReason },
}
```

ambiguous inheritance path は、source `qua` が path を選択していない限り affected candidate を
block する。invalid narrowing は reject する。narrowing は `reconsider` と task-10 obligation
handling の責務であり、overload resolution の責務ではない。

### task 24 viability data layer

`CandidateViabilityOutput::filter` は `TemplateExpansionOutput` と、concrete candidate id で
key 付けされた explicit checker-owned viability payload を消費する。各 argument の payload は、
すでに記録済みの evidence kind と status を表す。つまり exact type match、consumable fact
widening、accepted widening/source-`qua` coercion、rejected narrowing、missing evidence、
pending/degraded/rejected fact state、out-of-scope assumption、ambiguous inheritance、
または external dependency deferral である。

task 24 はこれらの payload を検証するだけで、`TypeFactTable` から fact を derive したり、
cluster closure を実行したり、registration を activate したり、obligation を discharge したり、
`TypedAst` を walk したり、view を insert したりしない。すべての argument が accepted
proof-free evidence を持つ場合だけ、candidate は viable candidate table へ出力される。reject
または block された candidate はその table から除外し、stable な argument 別 reason と
diagnostic を持つ viability decision table に保持する。viability decision table は accepted
candidate を含むすべての concrete source candidate について 1 row を持つため、accepted view
plan と non-viable reason は同じ stable table から render される。既存 candidate diagnostic は、
viable candidate table に残る candidate についてだけ remap される。同じ concrete candidate
に対する duplicate viability payload は stable reason でその candidate を block し、unknown
candidate id を key にした payload は黙って消費せず diagnostic にする。

## specificity preorder

specificity は overload site ごとの graph として表す。global DAG ではない。node は ordinary
root ごとに group された viable concrete candidate である。edge `A -> B` は、root `A` がすべて
の argument position で root `B` 以上に specific であることを意味する。

comparison は normalized parameter type と recorded closure fact を使う。

- radix または structure compatibility は既知の mode/structure hierarchy に従う;
- attribute subsumption は既に記録された closure fact を使う;
- すべての argument position が at least as specific でなければならない;
- strictness は reverse edge が存在しない場合だけ成立する;
- mutual specificity は concrete normalized parameter vector が closure-equivalent
  であることを意味し、root が交換可能であることは意味しない;
- incomparable pair は incomparable のまま残す。

許可される tie-breaker は architecture 05 と spec 19 のものに限る。

- parameter vector が equivalent の場合、non-template は template-derived に勝つ;
- source-local declaration は、signature collection がその shadowing を既に accepted している
  場合だけ imported declaration を shadow できる;
- template 宣言制約の厳しさは、展開後の tie-breaker ではない;
- return type は決して tie-breaker ではない;
- same-root redefinition は tie-breaker ではない。

specificity graph rendering は deterministic である。edge は reason key を持ち、diagnostic と
`@show_resolution` が comparison の成功、失敗、blocked 理由を説明できるようにする。

### task 25 specificity graph data layer

`SpecificityGraphOutput::build` は `CandidateViabilityOutput` と、viable candidate id で key
付けされた explicit checker-owned pairwise comparison payload を消費する。各 payload は left
candidate が at least as specific か、right candidate が at least as specific か、両者が
equivalent か、その pair が incomparable か、または comparison が blocked/deferred かを記録する。
task 25 は overload site ごとに 1 graph、viable concrete candidate ごとに 1 node、observed または
missing same-site pair ごとの comparison row、accepted at-least-as-specific relation だけの
directed edge を記録する。

task 25 は closure fact の derivation、return type の参照、root selection、tie-breaker 適用、
refinement join、view insertion を行わない。missing comparison payload、duplicate pair payload、
cross-site pair payload、unknown candidate id は黙って消費せず stable reason で診断する。
incomparable pair は explicit comparison row のまま残り、edge を作らない。

## root selection and refinement joins

selected ordinary root は、allowed tie-breaker 適用後の per-site specificity graph における
unique maximal non-redefinition candidate として表現される。viable root がなければ site は
`NoMatch` になる。unrelated / incomparable であるか mutual specificity により equivalent
であるかに関わらず、複数の相異なる ordinary root が maximal のままなら site は
`Ambiguous` になる。
checker data-layer boundary では、「allowed tie-breaker 適用後」とは、pairwise comparison
payload が spec で許可された non-template/template または accepted local-shadow decision だけを
graph edge または equivalence として既に符号化している、という意味である。root selection は
新しい tie-breaker を適用せず、root の順序付けに return type を参照せず、未解決の tie は
`Ambiguous` のまま残さなければならない。
redefinition candidate は selected root candidate にはならない。graph が ordinary root について
redefinition だけを maximal に残す場合、その site は malformed / missing ordinary-root payload
として blocked になり、redefinition は refinement-only metadata のまま残る。
複数の same-root non-redefinition candidate が maximal のまま残る場合、たとえば
non-template/template tie-breaker が edge として符号化されていない場合、その site は
ambiguous ordinary-root payload として blocked になる。

root 選択後、checker は `ordinary_root` が selected root と同じで、coherence metadata が
`Accepted` の accepted viable redefinition を集める。

- active refinement がなければ root metadata を expose する;
- active refinement が 1 つならその refinement metadata を expose する;
- active refinement が複数なら result fact が compatible な場合に join する;
- joined fact が incompatible なら `IncompatibleRefinementJoin` を生成する。

refinement join は return/result metadata を使ってよい。ordinary root は既に選択済みだからである。
unrelated ordinary root 間の選択に result metadata を使ってはならない。pending または failed
coherence evidence は redefinition を diagnostic-visible に保つが、join には使えない。

join compatibility は spec 19 に従う。

- functor では、active result type の 1 つが他のすべての active result type 以上に specific
  なら、その strongest type を expose する;
- そうでない場合、同じ radix を持つ functor result type は mutually consistent な attribute の
  union を expose でき、その union は既に記録された active cluster fact で閉じる;
- incompatible result radix、contradictory attribute、unique joined type の欠如は
  `IncompatibleRefinementJoin` を生成する;
- predicate / attribute refinement は logical bookkeeping だけを join する。coherence により
  truth-value disagreement は起きないが、structure / attribute metadata が incompatible な場合は
  source position で順序付けせず diagnostic にする。

## `qua` view insertion

source-written `qua` view は保存する。checker は、selected candidate が後続 phase に明示すべき
proof-free widening を要求する場合、inserted view も記録してよい。

- unique inheritance path upcast;
- 既存 `TypeFactId` で正当化される attribute weakening;
- template expansion 後の explicit template-argument widening。

inserted view は semantic metadata であり source edit ではない。source argument、target
normalized type、reason、evidence fact または inheritance path、selected candidate、view が
source-written か inserted かを記録する。

inserted view は narrowing、missing fact、ambiguous path、または ambiguity 検出後の post-hoc
overload disambiguation には禁止される。複数 inheritance path が存在し source `qua` が選択して
いない場合、site または candidate は blocked のまま残る。

### task 26 selection and view data layer

`OverloadSelectionOutput::resolve` は `SpecificityGraphOutput` と、overload site で key 付けされた
explicit checker-owned refinement-join / inserted-view payload を消費する。per-site specificity
graph から additional tie-breaker なしで candidate maximality を計算し、unique maximal ordinary
root の non-redefinition candidate が存在する場合にそれを選択し、selection が成功できない場合は
`NoMatch` と `Ambiguous` の failed-site record を保持し、caller supplied refinement/view payload
が accepted coherence を持つ selected-root redefinition と proof-free widening evidence だけを
参照することを検証する。

task 26 は ordinary root 選択後に限り result metadata を使ってよい。caller が compatible evidence
を供給した場合、strongest result type と same-radix attribute-union payload を受け入れ、
incompatible join は `IncompatibleRefinementJoin` failed-site record にする。inserted view は
accepted widening/source-`qua` evidence に限って記録する。narrowing、ambiguous path、missing
fact、missing selection payload、non-selected root 用 payload は成功を捏造せず、stable failed
record と diagnostic を生成する。

## failed-site preservation

recoverable failure は explicit failed overload-site record を生成する。

```rust
enum OverloadResult {
    Resolved {
        site: OverloadSiteId,
        root: CandidateId,
        refinements: Vec<CandidateId>,
        exposed_result: Option<ExposedResultPayload>,
        inserted_views: Vec<InsertedViewId>,
    },
    NoMatch { site: OverloadSiteId, rejected: Vec<CandidateId> },
    Ambiguous { site: OverloadSiteId, candidates: Vec<CandidateId>, graph: SpecificityGraphId },
    IncompatibleRefinementJoin {
        site: OverloadSiteId,
        root: CandidateId,
        refinements: Vec<CandidateId>,
        reason: RefinementJoinFailure,
    },
    Blocked { site: OverloadSiteId, reason: OverloadBlockedReason },
}
```

failed site は candidate list、rejection reason、source span、利用可能な graph id を保持する。
fabricated successful definition へ書き換えてはならない。elaboration と proof phase は failed
site を core logic に lower せず、skip または degrade する。

## diagnostics and determinism

diagnostic は public diagnostic code allocation が存在するまで checker-local である。以下に
stable detail key を含めなければならない。

- no viable candidate;
- overload ambiguity;
- template inference failure または ambiguity;
- ambiguous inheritance path;
- invalid または narrowing `qua`;
- incompatible refinement join;
- pre-existing declaration conflict;
- missing checker-owned payload。

すべての diagnostic は primary source range、candidate order、declaration / import
provenance、expected/actual type summary、comparison reason、1 つの widening で site が一意に
なる場合の suggested `qua` target を保持する。

deterministic rendering requirements:

1. site は source id、range、owner、kind で sort する。
2. candidate は site 内で declaration kind、ordinary root、symbol id、template key、
   provenance、declaration source order、最後の duplicate tie-break で sort する。
3. template exclusion、viability rejection、specificity edge、active refinement、inserted
   view、diagnostic はすべて canonical order で render する。
4. equivalent input は byte-identical debug rendering を生成する。

## Public Enum Policy

task 31 は frontend task-25 の public-enum decision procedure をこの module に適用する。
`overload_resolution` の public checker-owned enum はすべて forward-compatible API
surface であり、`#[non_exhaustive]` を維持しなければならない。downstream consumer は
wildcard または fallback arm を保持する。checker 内部の match は、仕様化済み behavior を
実装するために現在表現されている variant へ exhaustive のままにしてよい。

| enum | decision |
|---|---|
| `OverloadSiteKind` | 前方互換; overload site role は source extraction の拡大に伴い増える可能性がある。 |
| `UnsupportedOverloadRole` | 前方互換; unsupported role category は parser/checker surface とともに増える可能性がある。 |
| `OverloadSiteRecovery` | 前方互換; site recovery state は source recovery integration とともに増える可能性がある。 |
| `CandidateDeclarationKind` | 前方互換; candidate declaration family は Mizar declaration とともに増える可能性がある。 |
| `CandidateOrigin` | 前方互換; candidate origin は summary、artifact、recovery source とともに増える可能性がある。 |
| `CoherenceStatus` | 前方互換; coherence state は proof/artifact status の接続に伴い増える可能性がある。 |
| `TemplateArgument` | 前方互換; template argument form は parser/template semantics とともに増える可能性がある。 |
| `TemplateQuaStatus` | 前方互換; template `qua` state は view evidence policy とともに増える可能性がある。 |
| `TemplateConstraintEvidenceStatus` | 前方互換; constraint evidence state は proof/artifact input とともに増える可能性がある。 |
| `CandidateScope` | 前方互換; candidate scope は dependency と summary source とともに増える可能性がある。 |
| `OverloadSiteStatus` | 前方互換; site state は recovery と deferred input とともに増える可能性がある。 |
| `OverloadCandidateStatus` | 前方互換; candidate state は collection と filtering をまたいで増える可能性がある。 |
| `TemplateSubstitutionSource` | 前方互換; substitution source は inference payload とともに増える可能性がある。 |
| `TemplateExpansionStatus` | 前方互換; expansion outcome は template semantics とともに増える可能性がある。 |
| `TemplateExpansionFailure` | 前方互換; expansion failure は新しい template check とともに増える可能性がある。 |
| `ArgumentViabilityEvidence` | 前方互換; viability evidence は fact、view、proof とともに増える可能性がある。 |
| `ViabilityFactStatus` | 前方互換; fact evidence state は fact-query policy とともに増える可能性がある。 |
| `ViabilityCoercionKind` | 前方互換; viability coercion category は view insertion とともに増える可能性がある。 |
| `ViabilityCoercionStatus` | 前方互換; viability coercion state は evidence handling とともに増える可能性がある。 |
| `CandidateViabilityStatus` | 前方互換; candidate viability state は rejection/blocking policy とともに増える可能性がある。 |
| `ArgumentViewKind` | 前方互換; argument view kind は追加 coercion form とともに増える可能性がある。 |
| `CandidateRejectionReason` | 前方互換; rejection reason は semantic check とともに増える可能性がある。 |
| `CandidateBlockedReasonKind` | 前方互換; blocked reason は external dependency とともに増える可能性がある。 |
| `SpecificityComparisonStatus` | 前方互換; comparison input status は specificity evidence とともに増える可能性がある。 |
| `SpecificityBlockedReasonKind` | 前方互換; blocked comparison reason は payload validation とともに増える可能性がある。 |
| `SpecificityComparisonOutcome` | 前方互換; comparison outcome は ordering policy とともに増える可能性がある。 |
| `SpecificityFailureReason` | 前方互換; specificity failure reason は graph validation とともに増える可能性がある。 |
| `RefinementJoinStatus` | 前方互換; refinement join state は result-type policy とともに増える可能性がある。 |
| `RefinementJoinFailure` | 前方互換; refinement join failure は compatibility check とともに増える可能性がある。 |
| `ExposedResultSource` | 前方互換; exposed-result source は selection policy とともに増える可能性がある。 |
| `InsertedViewKind` | 前方互換; inserted view kind は accepted coercion form とともに増える可能性がある。 |
| `InsertedViewStatus` | 前方互換; inserted view state は validation policy とともに増える可能性がある。 |
| `OverloadResultStatus` | 前方互換; overload result state は failed-site handling とともに増える可能性がある。 |
| `OverloadBlockedReason` | 前方互換; blocked overload reason は selection validation とともに増える可能性がある。 |
| `OverloadDiagnosticProvenance` | 前方互換; diagnostic provenance は追加 payload stage とともに増える可能性がある。 |
| `OverloadDiagnosticClass` | 前方互換; diagnostic class は public checker diagnostic code が割り当てられる前に増える可能性がある。 |
| `OverloadDiagnosticSeverity` | 前方互換; diagnostic severity policy は IDE/artifact consumer とともに増える可能性がある。 |
| `OverloadDiagnosticRecovery` | 前方互換; diagnostic recovery state は partial overload policy とともに増える可能性がある。 |

この module が所有する exhaustive public enum exception はない。

## deferred and external gaps

task 26 は以下を意図的に deferred のままにする。

- overload site、candidate、template payload、source `qua` path、viability evidence、
  specificity comparison evidence、selection/refinement payload、inserted-view evidence、
  scheme/theorem role の
  AST-wide source-to-checker extraction;
- MC-G006 が記録する unsupported template / scheme role の parser/resolver exposure;
- public diagnostic-code allocation;
- artifact emission/reuse と stable `ResolvedTypedAst` schema projection;
- active `.miz` overload semantic fixture。

これらの gap は implementation task で `test_gap`、`external_dependency_gap`、または
`deferred` として記録する。raw syntax scan、opaque resolver shell parse、candidate
fabrication、failed site の successful 扱いを許可するものではない。

## planned tests

task 22 candidate-site collection:

- supported site kind ごとの fixture;
- provenance と source range の保持;
- already-filtered scope/visibility input を global scan なしで保存すること;
- deterministic candidate order;
- duplicate site key、missing candidate site link、deferred unsupported role。

task 23 template expansion:

- explicit / omitted template argument case;
- inference 中に cluster expansion を使わないこと;
- constrained template evidence の accepted、missing、deferred case;
- source-`qua` accepted widening / rejected narrowing case;
- omitted-inference payload missing case;
- equivalent template-derived candidate に対する non-template priority;
- unsupported template/scheme role は deferred exclusion を生成すること;
- deterministic template expansion rendering。

task 24 viability:

- exact match、unique inheritance upcast、attribute weakening、source `qua`;
- known / context-visible / consumable evidence と、pending、degraded、rejected、
  out-of-scope evidence の対比;
- missing fact、invalid narrowing、ambiguous inheritance path;
- missing viability payload と deferred external dependency;
- rejection reason、blocked reason、view plan が stable であること;
- deterministic viability rendering。

task 25 specificity:

- comparable、equivalent、incomparable candidate pair;
- edge を作らない blocked / deferred comparison row;
- per-site graph rendering;
- missing、duplicate、unknown、cross-site comparison payload diagnostic;
- root selection、tie-breaker application、return-type-based ordering を行わないこと。

task 26 selection, refinement, and views:

- unique root selection、no-match、ambiguity、blocked path;
- missing、duplicate、unknown selection payload diagnostic;
- maximal set が zero または multiple non-redefinition root を持つ場合の missing /
  ambiguous ordinary-root candidate;
- blocked specificity graph の保存;
- graph の maximal root が unrelated のままなら追加 root-selection tie-breaker を行わないこと;
- active accepted same-root refinement;
- rejected/pending coherence redefinition を inactive refinement として拒否すること;
- strongest result type、same-radix attribute union、incompatible refinement join;
- inserted widening view;
- rejected narrowing または missing-evidence inserted view;
- non-selected-root refinement/view payload diagnostic;
- equivalent な selection/refinement/view payload ordering の deterministic rendering;
- failed site が explicit のままで successful output を seed できないこと。
