# Module: labels

> 正本は英語です。英語版: [../en/labels.md](../en/labels.md)。

状態: task R-017 は resolver-owned label-resolution contract を仕様化し、task
R-018 は `src/labels.rs` に theorem / lemma と proof-step の projection resolver
を実装した。専用 label scope family、proof-block nesting key、forward-reference
rejection、simple / qualified citation candidate、lowered grouped-item candidate、
`LabelIndex` population、`LabelRefTable` outcome、crate-local/internal conflict
diagnostic をカバーする。driver pipeline への完全な `SurfaceAst` lowering、grouped
shared-prefix container diagnostic、semantic `.miz` traceability は R-G002 の下で
task R-023 に残る。definition / registration label extraction は、後続の symbol /
signature source role に律速される。

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
unresolved label outcome を付ける。この container-level source walk は R-023 の paired work に残る。

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

normalized label-origin path は、formatting と無関係な局所編集で安定するだけの構造を含む。

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

semantic `.miz` corpus coverage と traceability metadata は、既存の R-G002 `test_gap` の下で
task R-023 が導入する。
