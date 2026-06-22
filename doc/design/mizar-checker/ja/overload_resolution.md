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
extraction、artifact writer、proof acceptance は追加しない。tasks 22-26 が以下の named
section を実装する。task 28 は後で `resolved_typed_ast.md` が仕様化する final
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
    owner: TypedSiteRef,
    source_range: SourceRange,
    kind: OverloadSiteKind,
    name: SymbolRef,
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
}

enum CoherenceStatus {
    Accepted,
    Pending,
    Rejected,
}
```

`result` は metadata と same-root refinement join のために保持する。ordinary root selection には
決して参加しない。`redefine` candidate は refine する ordinary root を指さなければならない。
same-root redefinition は競合する root ではない。pending または rejected coherence は
diagnostic のために保持してよいが、accepted coherence だけが redefinition を active にできる。

site と candidate の canonical ordering は source id、source range、typed owner key、
declaration kind、ordinary root、symbol id、template instantiation key、import/source
provenance、最後の duplicate tie-break として candidate-local id を使う。hash-map iteration、
worker order、import read order、diagnostic insertion order は rendered output に影響しては
ならない。

## template expansion

template expansion は ordinary viability と specificity より前に実行する。

template payload を持つ candidate ごとに:

1. explicit template argument を template parameter list に照合して分類する。
2. argument が省略されている場合、exact argument type と mode-hierarchy constraint だけから
   template parameter を infer する。
3. constrained template parameter を、caller が供給した explicit constraint evidence と site で
   visible な fact に照らして検査する。
4. missing、contradictory、ambiguous、unsupported、または required constraint evidence が欠けた
   inference を reject する。
5. 成功した template を concrete candidate に instantiate し、tie-breaking と diagnostic 用に
   template origin を保持する。

omitted-argument template inference では cluster expansion を使わない。source-written
template `qua_arg` は typed/coercion payload が proof-free inheritance widening として公開する
場合だけ受け入れる。predicate、functor、scheme、algorithm template role は、parser、
resolver、checker-owned payload が具体的な role と parameter mapping を公開しない限り
deferred のままである。non-template candidate は、concrete parameter vector が otherwise
equivalent の場合に限って template-derived candidate に勝つ。

template expansion diagnostic は skipped template candidate と reason を保持する。failed
template expansion は ordinary candidate にならない。
`such that` template constraint に visible accepted evidence が欠けている場合、candidate は
constraint-evidence reason 付きで block または reject される。checker は overload resolution
中に新しい proof obligation を作成したり constraint を仮定したりしてはならない。parser/resolver
payload が constraint または label mapping を公開しない場合、その candidate は expand せず
MC-G027 の下で deferred とする。

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

## specificity partial order

specificity は overload site ごとの graph として表す。global DAG ではない。node は ordinary
root ごとに group された viable concrete candidate である。edge `A -> B` は、root `A` がすべて
の argument position で root `B` 以上に specific であることを意味する。

comparison は normalized parameter type と recorded closure fact を使う。

- radix または structure compatibility は既知の mode/structure hierarchy に従う;
- attribute subsumption は既に記録された closure fact を使う;
- すべての argument position が at least as specific でなければならない;
- strictness は reverse edge が存在しない場合だけ成立する;
- incomparable pair は incomparable のまま残す。

許可される tie-breaker は architecture 05 と spec 19 のものに限る。

- parameter vector が equivalent の場合、non-template は template-derived に勝つ;
- source-local declaration は、signature collection がその shadowing を既に accepted している
  場合だけ imported declaration を shadow できる;
- return type は決して tie-breaker ではない;
- same-root redefinition は tie-breaker ではない。

specificity graph rendering は deterministic である。edge は reason key を持ち、diagnostic と
`@show_resolution` が comparison の成功、失敗、blocked 理由を説明できるようにする。

## root selection and refinement joins

selected ordinary root は、allowed tie-breaker 適用後の per-site specificity graph における
unique maximal root である。viable root がなければ site は `NoMatch` になる。複数の unrelated
root が maximal のままなら site は `Ambiguous` になる。

root 選択後、checker は `ordinary_root` が selected root と同じ accepted viable redefinition を
集める。

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
2. candidate は ordinary root、symbol id、template key、provenance、declaration source
   order で sort する。
3. template exclusion、viability rejection、specificity edge、active refinement、inserted
   view、diagnostic はすべて canonical order で render する。
4. equivalent input は byte-identical debug rendering を生成する。

## deferred and external gaps

task 21 は以下を意図的に deferred のままにする。

- tasks 22-26 の Rust implementation;
- overload site、candidate、template payload、source `qua` path、scheme/theorem role の
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
- deterministic candidate order。

task 23 template expansion:

- explicit / omitted template argument case;
- inference 中に cluster expansion を使わないこと;
- constrained template evidence の accepted、missing、deferred case;
- equivalent template-derived candidate に対する non-template priority;
- unsupported template/scheme role は deferred exclusion を生成すること。

task 24 viability:

- exact match、unique inheritance upcast、attribute weakening、source `qua`;
- known / context-visible / consumable evidence と、pending、degraded、rejected、
  out-of-scope evidence の対比;
- missing fact、invalid narrowing、ambiguous inheritance path;
- rejection reason と view plan が stable であること。

task 25 specificity:

- comparable、equivalent、incomparable candidate pair;
- per-site graph rendering;
- allowed tie-breaker だけを使うこと;
- return type が root ordering に参加しないこと。

task 26 selection, refinement, and views:

- unique root selection、no-match、ambiguity、blocked path;
- active accepted same-root refinement;
- strongest result type、same-radix attribute union、incompatible refinement join;
- inserted widening view;
- failed site が explicit のままで successful output を seed できないこと。
