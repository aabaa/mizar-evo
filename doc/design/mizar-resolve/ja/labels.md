# Module: labels

> 正本は英語です。英語版: [../en/labels.md](../en/labels.md)。

状態: task R-017 は resolver-owned label-resolution contract を仕様化し、task
R-018 は `src/labels.rs` に theorem / lemma と proof-step の projection resolver
を実装した。専用 label scope family、proof-block nesting key、forward-reference
rejection、simple / qualified citation candidate、lowered grouped-item candidate、
`LabelIndex` population、`LabelRefTable` outcome、crate-local/internal conflict
diagnostic をカバーする。R-023 が追加したのは declaration-symbol corpus
collection だけで、production `SurfaceAst` proof-label declaration/reference
lowering ではない。bounded normal-source proof-step/simple-unqualified collector は
Checker Task 258B5C active confinement coverage より前の R-032 として計画する。
grouped shared-prefix container diagnostic と definition/registration label
extraction は R-032 の外である。

## 参照

この設計は resolver-owned label contract を次から導出する:

- architecture 03 "Label Resolution Is Scoped Separately from Item Resolution"。
- statement label、proof organization、justification form、scoping rule に関する
  spec chapter 15。
- theorem label、proof-block visibility、citation form に関する spec chapter 16。
- diagnostic payload requirement と現行 resolver-code `spec_gap` に関する
  spec chapter 22。
- architecture 22 の `ObligationAnchor` provenance requirement。
- resolver-local `resolved_ast.md`、`env.md`、`imports.md`、`names.md`、
  `declarations.md`。

## 目的

labels phase は import、declaration shell、namespace lookup が利用可能になった後、
proof checking、type checking、ATP dispatch、template instantiation、obligation
generation の前に、label declaration と citation use site を解決する。
source-shaped syntax と resolver-owned index を消費し、`ResolvedAst` には明示的な
label outcome を、`SymbolEnv` には可視 label projection を記録する。

入力:

- 現在の module の `SurfaceAst`。
- `imports.md` と `names.md` 由来の resolved import と namespace lookup behavior。
- `declarations.md` 由来の declaration shell。
- 利用可能な場合の source-backed fixture または summary 由来の module / dependency
  label projection。
- `mizar-syntax` が所有する syntax recovery marker と source range。

出力:

- represented theorem、definition、proof-step、registration label の declaration record。
- `LabelIndex` entry と可視 label projection。
- resolver が試みた citation use site の `LabelRefTable` entry。
- 明示的な unresolved / ambiguous label record。
- deterministic ordering を持つ crate-local/internal label diagnostic record。

## 境界

labels phase は次を行ってよい:

- label declaration を label scope family と source role で分類する。
- simple、qualified、grouped citation label を解決する。
- label visibility、duplicate-label conflict、forward-reference failure を判定する。
- 後続の `ObligationAnchor` label hint と dependency slice のために normalized
  provenance を保持する。

次は行ってはならない:

- theorem、proof step、definition correctness condition、registration condition を証明する。
- `ObligationAnchor` value や verification condition を生成する。
- ATP を実行する、premise を選択する、template argument を意味的に展開する。
- definition body、registration、proof statement を type-check する。
- ordinary name の overload winner を選択する。
- public user-facing resolver diagnostic code を創作する。

## Label Scope Family

label は ordinary symbol ではない。label declaration は resolver-owned family の
いずれかに属する。

| family | source | visibility surface | downstream consumer |
|---|---|---|---|
| theorem / lemma result | `theorem` item と `lemma` item | declaration 後の current module。public の場合は exported table | citation、artifact、ATP premise selection |
| definition | definition / redefinition label | defining item と source correctness-role provenance | checker、VC generation、diagnostics |
| proof step | labeled proposition、assumption、conclusion、case、`now` block、iterative equality chain | declaration 後の enclosing reasoning block と nested child block | proof justification と local context |
| registration | registration / reduction label | registration item と registration trace | checker、kernel replay、diagnostics |

期待される label family は use-site syntax から来る。`by` citation は local proof-step
label または module theorem / lemma label を参照できる。Definition と registration の
label reference は correctness site や registration trace site など、その family を
期待する syntax position でのみ解決する。ある use site が複数 family を正当に受け入れ、
visibility filtering 後も複数候補が残る場合、resolver は source order で選ばず
deterministic ambiguity を記録する。

## Proof-Block Scope

proof label は ordinary symbol namespace ではなく reasoning block に scope される。

- statement に付いた label は、その statement が完了した後にのみ可視になる。
- `now ... end` に付いた label は enclosing block に属し、その block が閉じた後にのみ
  可視になる。
- nested proof、case、suppose、diffuse reasoning block の内部で宣言された label は、
  その block と nested child block には可視だが、child block が閉じた後の enclosing
  block には可視ではない。
- enclosing proof label は、nested construct が別の module-level item を開始しない限り、
  nested child block 内で可視である。
- inner-scope label shadowing は spec chapter 15 により禁止される。current label scope から
  可視な label と同じ spelling の新しい label は shadowing declaration ではなく
  duplicate または conflict である。
- same-scope duplicate label は duplicate-label conflict である。

resolver は duplicate または conflicting label の後も module の残りを解決し続ける。
conflict は crate-local/internal diagnostic data として記録し、後続診断と editor
navigation のために十分な candidate provenance を保持する。

## Declaration Point And Forward References

label lookup は declaration point に依存する。

- label は、その declaring statement、item、block が完了した後にのみ可視になる。
- same proof block 内の後続 label への citation は unresolved である。
- theorem / lemma label は、その theorem / lemma item が完了した後にのみ後続 module item へ
  可視になる。同じ module 内の後続 theorem / lemma への citation は unresolved である。
- definition と registration の label は、enclosing item structure に従って resolver-visible な
  correctness-role / trace-provenance position で可視になるが、declaring syntax が収集される前には
  可視ではない。
- label 自身の declaration body からの self-reference は、後続 proof/checker phase が
  別の recursive rule を定義しない限り unresolved である。R-017 はそのような rule を
  定義しない。

forward-reference failure は、attempted spelling、use-site range、expected label family を
持つ明示的な `UnresolvedLabelRef` 風 outcome として表現する。label origin path は創作しない。

## Citation Lookup

simple unqualified citation lookup は label family ごとに行う。

1. current proof block chain で可視な proof-step label。
2. use site で可視な current-module theorem / lemma label。
3. resolved import と export を通じて可視になった imported public theorem / lemma label。

inner proof-label shadowing は禁止されるため、同じ spelling の proof-step candidate が複数あれば
conflict record である。family と visibility の filtering 後も unqualified citation に複数の
正当候補が残る場合、resolver は normalized origin path、kind、source range で sort した
candidate を持つ `AmbiguousLabelRef` を記録する。

qualified citation は namespace lookup と label lookup に分割される。

1. module prefix を `names.md` の namespace rule で解決する。
2. final label spelling を target module の exported label table で解決する。

citation prefix は namespace path に限られる。R-016 の local-term shadowing、
selector、`DeferredSelector` record に関する dot-chain finalization rule は、simple、
qualified、grouped、bulk citation prefix には適用しない。

grouped citation は各 grouped label に同じ resolved module prefix を使い、具体的な grouped
item ごとに 1 つの label-resolution outcome を生成する。R-018 は shared prefix がすでに
resolved または failed になった後の lowered per-item candidate を受け取る。完全な
`SurfaceAst` lowering は shared-prefix failure を 1 回記録し、各 grouped item に dependent
unresolved label outcome を付ける。R-023 はこの container walk を実装しておらず、
R-032 は simple unqualified proof-label citation だけに限定するため、grouped
shared-prefix lowering は別に authorize される後続 task に残る。

bulk citation（`module_path.*`）は individual label entry を創作する許可ではない。target
module の exported theorem / lemma label table が利用可能な場合、resolver は spec chapter 16
が要求する deterministic public theorem / lemma label set へ bulk citation を展開してよい。
その table が利用できない場合、resolver は citation container に unresolved
module-label-set dependency を記録し、synthetic `LabelRef` entry を創作しない。

citation に付いた template argument は後続 template / proof phase 用の use-site provenance
として運ぶ。R-017 と R-018 はそれらを検証、instantiate、type-check しない。

## Label Origin Path

`LabelOriginPath` は `LabelRef`、`LabelIndex`、dependency edge、後続の
`ObligationAnchor` label hint で使う resolver-owned stable identity である。これは proof
evidence ではなく、proof/checker-owned identity の代替にしてはならない。

canonical label-origin serialization は formatting と無関係な局所編集で安定する
構造を含む。canonical なのは framing / field order であり、identifier spelling は
exact parser token byte のままで case-fold / Unicode normalize しない。

- canonical `ModuleId` または module path。
- label family と primary spelling。
- defining item kind と source contribution。
- declaring statement、proof block、definition clause、registration clause への
  source-shaped structural path。
- proof label の場合は、enclosing theorem または proof owner と proof-block / local
  statement path。
- definition / registration label の場合は、checker-owned semantics なしに利用可能な範囲で
  source correctness-role または trace provenance。

Source range と `SurfaceNodeId` は diagnostics と editor navigation の provenance であり、
それ自体は canonical label identity ではない。

## Recovery And Diagnostics

recovered または malformed label syntax は、周辺の source shape がまだ表現されている場合、
unresolved または recovered label record として保持する。resolver は recovered proof /
declaration subtree で panic してはならない。
recovered label projection は degraded label-index fact として保持するが、
label-reference candidate set と duplicate / conflicting-label diagnostics から除外し、
parser recovery が semantic ambiguity や conflict report へ連鎖しないようにする。

diagnostic record は R-G001 が未解決の間 crate-local/internal に保つ。label diagnostic は
次を保持しなければならない。

- primary use-site または declaration range。
- duplicate / conflicting declaration range。
- expected label family。
- qualified citation の failed namespace または unresolved import dependency。
- ambiguity 用の deterministic candidate list。

本 module spec は public numeric resolver diagnostic code を割り当てない。

## Determinism

label collection と resolution は deterministic である。

- declaration traversal は stable source order に従う。
- table id は deterministic traversal から来る insertion-order id である。
- candidate list は `LabelOriginPath`、label kind、source range で sort する。
- diagnostic は primary source range、diagnostic class、stable origin path で sort する。
- debug rendering は normalized origin path を使い、raw hash-map order を使わない。

## 公開 enum の前方互換性

task R-026 は frontend task 25 の public-enum decision procedure をこの module に適用する。
`labels` が所有する公開 resolver enum はすべて forward-compatible API surface であり、
`#[non_exhaustive]` を維持しなければならない:

- `LabelProjectionSource`
- `LabelReferenceScope`
- `LabelDiagnosticKind`
- 計画済み `ProofLabelSourceCollectionError`

この module は exhaustive な公開 enum 例外を所有しない。下流 consumer は wildcard
または fallback arm を持たなければならない。resolver 内部の match は、仕様化済みの
挙動を実装する範囲で、現在表現されている variant に対して exhaustive でよい。

## Test Obligations

R-017 は documentation-only であったため executable test を追加しなかった。R-018 は次の
unit test を追加する。

- proof-block visibility と nested-block confinement。
- spec が禁止する inner-scope shadowing case を含む、visible scope をまたぐ duplicate /
  conflicting label。
- 後続 label への forward reference の拒否。
- parser が該当 syntax をすでに生成する範囲での simple、qualified、lowered grouped-item
  citation lookup。
- deterministic `LabelRefTable`、`LabelIndex`、diagnostic ordering。

R-023 は active declaration-symbol corpus coverage を導入したが、label-reference
corpus coverage や production proof-label source projection は導入していない。
残る active label-reference case は R-G007 `test_gap` である。R-032 は最初の bounded
Checker Task 258B5C inner-to-outer / sibling confinement increment の独立 lower
prerequisite である。

## R-032B frozen normal-source projection contract

### Authority と finding

canonical Chapter 15 §15.10 と Chapter 16 §§16.4.2/16.5.1 は proof label の
reasoning-block confinement と同じ proof 内の先行 proposition citation を定める。
既存 `LabelResolver` prefix rule は正しい。欠けた production `SurfaceAst`
projection/reference path は Medium `source_drift`、R-023 attribution は
`design_drift`、bare mapping callback は validated structural-lowering boundary
を越える `boundary_violation` である。R-032A が mapping prerequisite を先に
修復する。active case 欠如は R-G007 `test_gap`、public resolver code 欠如は
Low deferred R-G001 `spec_gap` のまま。

### Exact lowering contract

R-032B は、required node/edge がすべて direct、normal/unrecovered、
exact-shaped の場合だけ candidate/traversal を受理する。unlisted node kind/edge
の default は skip、すなわち row/ordinal/descent なし。semantic descendant は
collector input にしない。

exhaustive default-deny Surface edge table:

| parent | allowed direct child / inspection | effect | all-other action |
|---|---|---|---|
| `Root` | exactly one direct normal `CompilationUnit` structural child | compilation unit へ descend。direct token child は skip | other/additional/missing structural child は root unsupported |
| `CompilationUnit` | exactly one direct normal `ItemList` structural child | item list へ descend | any other direct child または additional/missing structural child は compilation unit unsupported |
| `ItemList` | direct normal `TheoremItem` だけ | supported theorem owner を source order で scan | `LemmaItem`、`VisibleItem`、`StatementItem`、definition、annotation、recovered item を含む他 item child は skip/no descent |
| `TheoremItem` | direct role/theorem-label/colon token を inspectし、exact normal label と exactly one direct `ProofBlock` justification を要求 | theorem owner/root scope を allocate し、その `ProofBlock` だけへ descend | formula/other token/wrapper/additional-or-missing proof/other child は no descent。required shape failure は owner unsupported |
| `ProofBlock` | direct `proof`/`end` boundary と recovered/malformed direct child 不在を validate。ordered direct child の `CompactStatement` / `ConclusionStatement` だけ受理 | accepted statement は module-global ordinal を consume し direct-child order で visit | 他 statement/wrapper/token は no descent/no ordinal。malformed/recovered boundary は proof owner unsupported |
| `CompactStatement` | direct `Proposition` の exact first identifier token + colon だけを inspect | exact shape の場合だけ proof-step projection 1件 | `FormulaExpression` / token へは descend しない。他 proposition child/shape は no projection |
| `CompactStatement` / `ConclusionStatement` | direct `ProofBlock`、direct `JustificationClause` | proof block は nested child scope を作り descend、justification は citation walk を許可 | proposition/formula/token/other child は no descent。`ConclusionStatement` proposition label は除外 |
| `JustificationClause` | exact first token が `by` の場合だけ direct `ReferenceList` | reference list へ descend | computation / other child/shape は no descent |
| `ReferenceList` | source order の direct `Reference` child | exact simple-reference sibling を visit、comma token は skip | `QualifiedReference` / `GroupedReference` / `BulkReference` / recovered / other child は no descent/no row |
| `Reference` | exactly one direct identifier token、`TemplateArguments` / other child なし | `LabelReferenceCandidate` 1件 | additional/missing/template/qualified/malformed/recovered shape は no row |

required shape/owner chain の recovery、missing/error node、malformed boundary、
non-direct edge は owner/edge unsupported とし、そこを通って descend しない。
statement ordinal を consume するのは successfully supported direct
`CompactStatement` / `ConclusionStatement` row だけ。

```rust
pub struct ProofLabelSourceCollector<'a> {
    // Private fields.
}

impl<'a> ProofLabelSourceCollector<'a> {
    pub fn new(
        ast: &'a SurfaceAst,
        module: &ModuleId,
        namespace: NamespacePath,
        contribution: SourceContributionId,
        resolved: &'a SurfaceResolvedArena,
    ) -> Result<Self, ProofLabelSourceCollectionError>;

    pub fn collect(
        &self,
    ) -> Result<ProofLabelSourceCollection, ProofLabelSourceCollectionError>;
}
```

collection accessor は `projections() -> &[LabelProjection]` と
`references() -> &[LabelReferenceCandidate]`。construction は
`resolved.validate_against(ast, module)` を実行し、すべての
`ResolvedNodeId` を validated `SurfaceResolvedArena` から得る。callback /
unmapped-reference side channel / fabricated id は存在しない。
collector は `'a` の下で `ast` / `resolved` borrow だけを store し、
`namespace` / `contribution` を own する。`module` は borrow/store しない。
`new` は validation にだけ使う。各 `collect` は
`resolved.validate_against(ast, resolved.module())` を再実行し、stored
constructor argument ではなく arena の validated canonical identity を使う。

`ProofLabelSourceCollection` は `Debug`, `Clone`, `PartialEq`, `Eq` を derive
し、`Copy` は要求しない。`ProofLabelSourceCollectionError` は `Debug` を
derive して `Display` / `std::error::Error` を実装し、`Clone` / `Eq` / `Copy`
は要求しない。

exact public error declaration:

```rust
#[derive(Debug)]
#[non_exhaustive]
pub enum ProofLabelSourceCollectionError {
    SurfaceArena(SurfaceResolvedArenaError),
    ScopeComponentOverflow { node: SurfaceNodeId },
    StructuralPathComponentOverflow { node: SurfaceNodeId },
}
```

downstream は wildcard arm を持つ。
scope/path `u32` conversion は checked で、unwrap、saturation、truncation、
panic を禁止する。

stable structure / provenance:

- exact `Root` -> `CompilationUnit` -> `ItemList` upper chain を traverse し、
  supported direct normal `TheoremItem` / direct `ProofBlock` root を item-list
  source order で scan する。root scope は `[0]`, `[1]`, ...。theorem owner は
  statement ordinal を消費しない。
- module-global one-based statement counter 1つを theorem ごとに reset しない。
  各 root で normal supported `CompactStatement` / `ConclusionStatement` を
  depth-first preorder し、この form が own する direct nested `ProofBlock`
  だけへ ordered child order で descend する。
- supported statement は label/citation なしでも1 ordinal を消費し、reference は
  owning supported statement ordinal を使う。unlisted wrapper/container と
  excluded/unsupported/recovered/malformed statement/subtree は consume せず
  descend もしない。
- nested scope は owner-relative proof-child component を append する。
  `visible_after_ordinal` は labelled `CompactStatement` 自身と own proof を含む
  subtree の最大 consumed ordinal。
- B5C inner case は `1..5`: A declaration `2`、own-proof `3`、completion `3`、
  following `4`、negative ref `5`。sibling case は `1..6`、negative ref `6`。
  multi-theorem test は global counter を保ち ordinal eligible にする。
- exact one-line serialized grammar（space/newline なし）:

```text
proof-step-v1|package=<n>:<package>|module=<n>:<module-path>|contribution=<u>|owner-kind=theorem|owner=<n>:<owner-label>|owner-occurrence=<u>|proof-path=<k>:<c0>,...|label=<n>:<label>|label-occurrence=<u>
```

  `<n>` は following spelling の UTF-8 byte length。`<u>` / `<k>` / `<ci>` は
  0 以外 leading zero なしの canonical unsigned decimal、`<k>` は
  comma-separated path component 数、各 `<ci>` は checked `u32`。empty path は
  exact `proof-path=0:`。length framing のため
  escaping は不要。package/module は canonical `ModuleId`。owner/label spelling
  は parser identifier token text byte-for-byte で case fold / Unicode transform
  なし。owner occurrence は同じ exact spelling の earlier/current supported
  normal top-level theorem owner 中 zero-based。label occurrence は declaring
  proof scope 内 same-spelling supported labelled compact statement 中 zero-based。
  proof-path は owner root-relative、root は empty。root visibility index は
  serialize しない。
- projection origin は exact label token anchor と
  `[theorem item, compact statement, label token]`、reference origin は exact
  reference anchor と
  `[theorem item, owning CompactStatement or ConclusionStatement, reference]`。
  B5C exact path は inner `[57,42,8]` / `[57,55,52]`、sibling
  `[67,47,8]` / `[67,63,60]`。
- richer table origin は R-032A arena node minimal `[surface_id]` origin と
  意図的に異なり、追加で validate する。

lemma/claim/definition/registration owner、top-level theorem label、
assumption/given/take/set/consider/reconsider/case/suppose/now/hereby/
iterative-equality とその他 statement label、qualified/grouped/bulk/template
citation、recovered/malformed shape、semantic descendant はすべて除外し、row を
emit しない。

### Ownership、test、exit

R-032B ownership は `crates/mizar-resolve/src/labels.rs`、
`crates/mizar-resolve/src/labels/tests.rs`、paired design record だけ。R-032A は
preceding commit の `resolved_ast.rs` / `resolved_ast/tests.rs` owner である。

test は `[0] -> [0,1]` success、`[0,0] -> [0]` inner-to-outer と
`[0,0] -> [0,1]` sibling unresolved、cross-theorem same-spelling nonconflict、
earlier theorem root `[0]` label が ordinal 上は eligible でも later theorem
root `[1]` から unresolved、deterministic root order、own-proof A
unresolved、same-block post-completion resolved、exact B5C ordinal/range/origin/
path/identity、inclusion/exclusion/recovery、wrong/stale map、overflow、collection
order、spelling/proof-topology/formatting/owner spelling/order mutation を覆う。
さらに exact `proof-step-v1` construction/byte equality、UTF-8 length、
empty/nonempty path、occurrence counter、全 field mutation、module-global ordinal
continuity、unlabelled/no-citation consumption、unlisted wrapper/excluded
non-consumption/no-descent を覆う。
各 table permitted edge に positive test 1件を置き、upper edge は
root-to-compilation-unit、compilation-unit-to-item-list、item-list-to-theorem を
別々に証明する。root positive は direct token sibling を含み、sole structural
`CompilationUnit` を変えず skip することも証明する。
missing/additional/wrong upper structural child と relocated/
wrapped alternative は拒否し、theorem が `Root` / `CompilationUnit` 直下または
`VisibleItem` 配下なら到達しない。その他 negative は parent relocation、
wrapper、formula token、computation、
qualified/grouped/bulk/template、unsupported proof owner、recovered/malformed
mutation が rejected edge より先へ row/ordinal/descent を出さないことを証明する。
mixed `ReferenceList` は exact simple `Reference` sibling だけを収集して comma と
unsupported sibling を skip する。各 table row の all-other action を representative
case で覆う exhaustive default-deny matrix を持つ。

後続 active consumer は private `mizar-test` `declaration_symbol` の
`declaration_symbol.label.proof_scope_confinement`。public checker handoff は
unresolved input を拒否するため除外する。historical pre-S-026 record は docs
prerequisite、R-032A、R-032B、active B5C の4 logical task/4 commit だった。
下記 effective order がその execution count を supersede する。各 fresh
inventory を挟み、docs prerequisite は production、fixture、sidecar、trace
status/count を変更しない。

後続 R-032A preflight は、valid disconnected node を含む complete dense
Surface id が既存 API だけでは得られないため、R-032A 前に別 mizar-syntax
S-026 documentation/implementation prerequisite を挿入した。R-032B label
contract と active B5C test intent は変わらない。effective order は S-026 docs、
S-026 implementation、R-032A、R-032B、active B5C で、commit 間に fresh
inventory を挟む。
