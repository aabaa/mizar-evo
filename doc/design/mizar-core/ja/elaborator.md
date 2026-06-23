# Elaborator

> 正本は英語です。英語版:
> [../en/elaborator.md](../en/elaborator.md)。

状態: `mizar-core` task 7 のモジュール仕様。この文書は task 8-13 の
task-local design であり、[architecture 06](../../architecture/ja/06.elaboration_and_core_ir.md)
を精緻化し、出力 contract として [core_ir.md](./core_ir.md) と
[binder_normalization.md](./binder_normalization.md) を使う。

## 目的

`elaborator` は checker が所有する `ResolvedTypedAst` material を backend-neutral な
`CoreIr` へ lower する。これは source-shaped semantic boundary の最後である。入力は
source expression metadata、overload record、inserted view、type fact、diagnostic、
recovery status をまだ保持する。出力は canonical core symbol、binder-normalized
term/formula、明示的 type predicate、definition boundary、proof skeleton、algorithm shell、
source map、obligation seed を使う。

この module は raw syntax を inspect してはならず、resolver/checker/registration closure/
overload selection を再実行してはならない。proof search、CFG 構築、`VcId` 割当、
proof acceptance、artifact/kernel schema の発明も行わない。

## 参照

- [architecture 06](../../architecture/ja/06.elaboration_and_core_ir.md): phase 9 の
  Step 1 から Step 6。
- [architecture 05](../../architecture/ja/05.overload_resolution.md): overload selection、
  inserted `qua` view、`ResolvedTypedAst` 境界。
- [architecture 08](../../architecture/ja/08.reasoning_boundary.md): ATP/proof dispatch 前の
  Mizar-side semantic processing。
- [architecture 16](../../architecture/ja/16.substitution_and_binding.md): binder identity、
  capture avoidance、deterministic replay。
- [mizar-checker resolved typed AST](../../mizar-checker/ja/resolved_typed_ast.md):
  source-shaped final semantic input。
- [mizar-checker source/spec audit](../../mizar-checker/ja/source_spec_audit.md) と
  [crate exit report](../../mizar-checker/ja/crate_exit_report.md): upstream payload gap と
  完了済み checker boundary。
- [core_ir.md](./core_ir.md): core data shape、generated origin、diagnostic、source map、
  definition、proof node、algorithm、obligation seed。
- [binder_normalization.md](./binder_normalization.md): normalized binder representation、
  guard、generated semantic record、substitution、alpha-equivalence。

## 責務

`elaborator` が所有するもの:

- checker が提供した semantic record が core lowering に十分か検証すること。
- current-module core context を準備すること。canonical item id、definition boundary、
  local binder context、source-map builder、generated-origin registry、dependency summary を
  含む。
- checker の normalized type と type fact を明示的な core type predicate と assumption に
  lower すること。
- resolved term/formula を binder-normalized core node に lower すること。
- source-written / inserted `qua` view を明示的な provenance と既に確立済みの type fact として
  記録すること。
- structure、mode、attribute、predicate、functor、theorem、scheme、registration、
  reduction、algorithm を core item shell に lower すること。
- definition expansion policy、definition-time closure、formal type guard、
  generated-origin dependency、correctness obligation seed を記録すること。
- proof skeleton を core proof tree に lower し、thesis replacement、assumption、label、
  citation、terminal goal seed を明示すること。
- algorithm body は contract、ghost/runtime metadata、termination term を持つ
  core algorithm statement shell としてだけ lower すること。
- 失敗した semantic site を明示的 diagnostic と error/skipped node として保存すること。

`elaborator` が所有しないもの:

- source-to-checker extraction や AST walking。
- name resolution、type checking、cluster saturation、registration activation、
  overload candidate selection、template inference。
- proof acceptance、ATP dispatch、kernel checking、certificate production。
- `VcId` assignment、VC slicing、control-flow graph construction。
- artifact serialization、cache reuse anchor、public diagnostic-code registry allocation、
  stable external schema publication。

## 入力 contract

task 8-13 の実装は、明示的な checker-owned input だけを消費する。

- `ResolvedTypedAst` node、expression metadata、overload record、inserted coercion/view、
  candidate summary、diagnostic、recovery state。
- `ResolvedTypedAst` が参照する checker type/fact row。
- checker が既に materialize した accepted cluster/reduction trace row。
- resolver/checker output の canonical `SymbolId`。
- caller が提供する dependency core summary。
- `ResolvedTypedAst` 由来の caller-owned source-map span。

不足している checker payload を source inspection で埋めてはならない。upstream extraction または
利用できない external/downstream crate が必要なら `external_dependency_gap` として分類する。
`mizar-core` が所有するが後続 task で予定されている作業だけを `deferred` として分類する。

## 出力 contract

成功した elaboration は次を持つ `CoreIr` を生成する。

- deterministic な current-module item order。
- 後で diagnostic、obligation、metadata、snapshot、artifact、または source-mapped downstream record を
  生成し得るすべての core node の明示的 source map。
- `binder_normalization.md` を満たす binder を持つ core term/formula。
- expansion policy と correctness seed を持つ definition record。
- accepted proof status ではなく terminal goal seed を持つ proof record。
- CFG block ではなく statement shell を持つ algorithm record。
- core shape が必要とする local abbreviation、comprehension、type predicate、
  error placeholder、algorithm pick 用 generated origin。
- unsupported / malformed lowering 用の structured diagnostic。

失敗した semantic site は明示的な `Error` node または skipped/error item のままにする。
failed overload、missing type fact、unsupported source form、malformed proof skeleton、
unavailable algorithm metadata は、valid core term/formula になってはならない。

## gap 分類

| ID | 分類 | 内容 | task 7 の判断 |
|---|---|---|---|
| ELAB-G001 | `spec_gap` | task 7 前に `elaborator.md` は存在しない。 | この文書が task 8-13 の module-spec gap を閉じる。 |
| ELAB-G002 | `test_gap` | task 8 前に `src/elaborator.rs` test は存在しない。 | task 8-13 が section ごとの Rust fixture を追加する。 |
| ELAB-G003 | `external_dependency_gap` | full semantic payload の source-to-checker extraction は checker closeout で deferred のまま。 | 明示的な `ResolvedTypedAst` fixture を使い、source scan や payload fabrication をしない。 |
| ELAB-G004 | `external_dependency_gap` | artifact schema と cache reuse anchor は `mizar-core` 外の責務。 | provenance と dependency summary だけを保存する。 |
| ELAB-G005 | `external_dependency_gap` | proof acceptance、VC generation、kernel checking、certificate schema は downstream crate の責務。 | obligation seed と proof skeleton だけを emit する。 |
| ELAB-G006 | `deferred` | source-derived `.miz` core snapshot には mizar-test stage support が必要。 | staged source-to-core snapshot ができるまで Rust fixture を使う。 |
| ELAB-G007 | `source_drift` | task 8 前の source には `elaborator` module がなかった。 | task 8 が module を導入済みであり、task 9-13 が残る実装 slice を閉じる。 |

## Step 1: core context preparation

task 8 がこの section を実装する。

入力:

- `ResolvedTypedAst`。
- caller が提供する dependency core summary。
- resolver/checker の canonical symbol identity。
- checker diagnostic と recovery record。

出力:

- `CoreContext`。
- current-module item registry。
- definition boundary registry。
- generated-origin registry。
- binder / variable metadata context。
- source-map / diagnostic builder。
- deterministic elaboration worklist。

規則:

- source-order diagnostic を記録した後、deterministic source/module order で
  `CoreItemId` を割り当てる。
- item と reference identity には raw source spelling ではなく canonical `SymbolId` を保存する。
- dependency summary は read-only input として扱う。必要な reference の summary が欠ける場合は
  `UnsupportedLowering` または `UnresolvedSemanticInput` diagnostic であり、source を inspect する
  理由にはならない。
- body lowering より前に definition boundary を初期化し、recursive / mutually dependent
  reference を eager inlining なしに表現できるようにする。
- checker-resolved binder id、role、sort、type fact から binder context を作る。display name は
  diagnostic-only である。
- failed `ResolvedTypedAst` site は error/skipped worklist state として保存する。

task 8 の test は deterministic item id、dependency-summary absence、canonical id が raw spelling
を使わないこと、failed overload/worklist preservation、source-map builder initialization を扱う。

## Step 2: type and fact lowering

task 9 がこの section を実装する。

入力:

- normalized checker type row。
- visible type fact と cluster fact。
- inserted view と source-written `qua` metadata。
- checker initial obligation と deferred evidence row。

出力:

- `CoreTypePredicate` application。
- core assumption と guard formula。
- view explanation provenance。
- carried obligation seed reference。

消去規則:

- declared binder type `x be T` は `x` の core binder と
  `TypePred { subject: x, ty: pred(T) }` guard/assumption になる。
- formula assertion `x is T` は同じ `TypePred` formula になる。
- attribute chain は deterministic predicate order の明示的 predicate fact conjunction に lower する。
  negative polarity は `Not(TypePred)` になる。
- mode/radix expansion は checker-normalized type head を通じて lower し、type syntax を再構成しない。
- source-written `qua` と inserted view は、この step では variable-subject fact の消去に必要な
  view provenance と既に確立済みの type fact だけへ lower する。underlying term は Step 3 の
  lowering 責務として残る。新しい proof step ではない。
- reconsider/narrowing payload は fresh/narrowed core binding と、checker が提供した場合の
  carried obligation seed になる。
- sethood、non-emptiness、coercion、cluster evidence が欠けている場合は diagnostic/error node
  または deferred seed であり、elaborator が証明してはならない。

task 9 の test は各 erasure rule、positive/negative attribute polarity、inserted/source `qua`、
missing evidence diagnostic、deterministic conjunction ordering を扱う。

## Step 3: term and formula lowering

task 10 がこの section を実装する。

入力:

- resolved expression metadata と overload resolution record。
- Step 2 の lowered type predicate/fact。
- Step 1 の binder context。
- generated-origin registry。

出力:

- `CoreTermTable` row。
- `CoreFormulaTable` row。
- validation 用に merge された generated-origin table、新たに必要になった
  `GeneratedOriginTable` delta row、そして Step 1 registry または現在の lowering delta に
  既に存在する generated origin への参照。
- generated application term をその `GeneratedOriginId`、generated functor symbol、lowered argument に
  結びつける generated-origin use record。
- checker evidence が Step 2 で既に carry されていない Fraenkel membership/sethood
  obligation seed row。
- expression source-map entry。
- failed site の明示的 diagnostic。

term 規則:

- variable は stable checker/resolver `CoreVarId` で lower し、その後
  `binder_normalization` を通じて normalize する。
- constant と selected functor root は canonical `SymbolId` に lower する。
- application は selected root と lowered argument を持つ `CoreTermKind::Apply` へ lower する。
- selector、tuple、set enumeration は明示的 core node へ lower する。
- source-written / inserted `qua` は underlying term と view provenance / type fact へ lower し、
  implicit cast node を作らない。
- stable choice term は generated choice symbol を functor とする通常の `Apply` node へ lower する。
  stable choice に `CoreTermKind::Generated` を使ってはならない。
- stable choice generated symbol は owning core item/proof または definition context、
  alpha-normalized target type、明示的 free parameter により key 付ける。同じ owner/key pair は同じ
  generated symbol を再利用する。
- Step 3 は `(owner, kind, key)` をまず Step 1 の generated-origin registry で検索し、
  次に現在の lowering delta で検索する。存在する場合は既存の `GeneratedOriginId` への参照を
  記録し、存在しない場合だけその key に対してちょうど 1 つの新しい generated-origin delta row を
  出力する。
- 再利用される generated origin は現在の seed と同じ normalized parameter payload を持たなければ
  ならない。異なる場合は malformed checker input とし、有効な generated application へ lower しない。
- `Apply` term に選択された generated functor は generated-origin use に記録された functor と一致しなければ
  ならず、かつ generated origin に記録された functor とも一致しなければならない。registry functor が
  欠けている場合や通常 functor などと不一致なら malformed checker input とする。
- Fraenkel comprehension は alpha-normalized generator/mapper/predicate key と explicit free-parameter
  payload を持つ generated set-valued symbol へ lower する。
- Fraenkel comprehension は必要な sethood / membership evidence を generated origin の checker
  provenance、または明示的で active な `ObligationSeedKind::FraenkelMembershipAxiom` handoff row として
  保持しなければならない。membership axiom が以前の checker slice によって既に carry されている
  場合は、Step 3 が明示的な already-carried marker を記録する。evidence が欠ける場合は error term と、
  checker が deferred `ObligationSeedKind::GeneratedSethood` seed を提供するならその deferred seed へ
  lower し、有効な set term を捏造しない。
- algorithm statement の pick は Step 6 の `CoreAlgorithmStmtKind::Pick` へ lower し、
  shared stable choice symbol を使わない。

formula 規則:

- predicate application、equality、type assertion、attribute、connective、quantifier は明示的な
  `CoreFormulaKind` row に lower する。
- `contradiction` は `CoreFormulaKind::False` へ lower する。checker 由来の tautology または
  synthetic guard constant は `CoreFormulaKind::True` へ lower してよい。`thesis` は Step 5 の
  proof-skeleton lowering の責務として残す。
- quantifier binder は左から右へ処理する。binder の guard は prior binder と自身を見られるが、
  later binder は見られない。この検査は caller-supplied summary metadata だけでなく、実際の
  guard term/formula seed graph から導出する。
- failed overload site、missing selected root、malformed type evidence、unsupported surface form は
  `Error(CoreDiagnosticId)` へ lower し、valid logical node にしてはならない。
- generated term semantic record は owner、kind、key、normalized params、normalized arguments を含む。
  evidence/source text は provenance-only である。

task 10 の test は variable、constant、application、selector、tuple、set enumeration、predicate、
equality、type predicate、connective、quantifier guard、inserted/source `qua`、stable choice、
Fraenkel comprehension、failed overload preservation、generated-key determinism を扱う。
stable-choice fixture は `the T` が `CoreTermKind::Generated` ではなく通常の
`Apply(choice_T(params))` generated symbol に lower されることを assert しなければならない。
Fraenkel fixture は required sethood evidence が明示的 provenance または obligation input として
保存されること、そして sethood evidence が欠ける場合は fabricated valid set term ではなく
error/deferred seed のままになることを assert しなければならない。

## Step 4: definition lowering

task 11 がこの section を実装する。

入力:

- resolved declaration と signature。
- lowered term/formula。
- checker correctness/deferred obligation metadata。
- dependency / visibility summary。

出力:

- structure、mode、attribute、predicate、functor、theorem、scheme、registration、reduction、
  generated definition、algorithm 用 `CoreItem` row。
- `CoreDefinitionTable` row。
- correctness obligation seed。
- generated dependency。

規則:

- definition boundary は stable で、body lowering より前に登録する。
- resolved declaration 由来の visibility と export metadata は、生成される `CoreItem` row と
  dependency summary に保存する。
- elaborator は expansion policy（`Opaque`、`Transparent`、`Reducible`、`Computable`）を記録するが、
  definition を eager inline しない。
- formal binder は binder-scope rule の下で normalize された type guard を含む。
- `set`、`deffunc`、`defpred` の使用は、definition-time closure から capture-avoiding substitution で
  展開する。captured free variable は display name ではなく stable id である。
- conditional definition は ordered guarded branch に lower する。`otherwise` は prior guard の否定として
  表現し、coverage 欠落は明示的 obligation/diagnostic のままにする。
- existence、uniqueness、coherence、compatibility、reducibility、coverage check は obligation seed になる。
  この crate では accept しない。
- prerequisite が失敗した item は structured diagnostic 付きで skipped または error にする。

task 11 の test は definition boundary order、expansion policy、formal guard、
local abbreviation closure expansion、conditional branch、correctness seed emission、
generated dependency、skipped/error item preservation を扱う。stable choice を含む exported
definition は、unfolding 時に definition-owned generated choice symbol を再利用しなければならない。
use site で fresh choice symbol を再生成してはならない。

## Step 5: proof-skeleton lowering

task 12 がこの section を実装する。

入力:

- theorem / lemma proposition。
- 利用可能な checker proof skeleton payload。
- proof label と citation。
- lowered formula と binder context。

出力:

- `CoreProofTable` row。
- core proof node。
- theorem/proof status metadata。
- terminal proof goal obligation seed。

規則:

- `thesis` は current core formula に置き換える。magic identifier として保存しない。
- introduced variable は type guard 付き `CoreBinder` になる。
- assumption と labeled step は source/core provenance を持つ明示的 proof node になる。
- citation は semantic input に既にある label、canonical symbol、generated origin を参照する。
- cases、suppose、now などの branching proof form は構造を保存し、open proof leaf に terminal goal
  seed を生成する。
- `open`、`assumed`、`conditional`、`error` status を記録する。このいずれも `mizar-core` で theorem を
  証明しない。
- malformed / missing proof skeleton payload は `MalformedProofSkeleton` diagnostic と error proof node を
  生成する。

task 12 の test は thesis replacement、introduced binder、assumption、label/citation、branch kind、
terminal goal seed、theorem status、malformed proof skeleton diagnostic を扱う。theorem / lemma
proposition fixture は、generated symbol が theorem/lemma proposition context に owned され、
proof-skeleton lowering を通して保存される stable choice を含めなければならない。

## Step 6: algorithm-shell lowering

task 13 がこの section を実装する。

入力:

- resolved algorithm declaration。
- lowered contract formula と termination term。
- ghost/runtime metadata。
- source-shaped resolved statement payload。

出力:

- `CoreAlgorithmTable` row。
- `CoreAlgorithmStmtTable` statement shell。
- contract set。
- algorithm diagnostic と generated pick origin。

規則:

- parameter と optional result binder は role、guard、source map とともに記録する。
- `requires`、`ensures`、`assert`、`invariant`、`decreasing` clause は phase 9 で core formula/term へ
  lower する。
- statement order と nesting は保存するが、basic block と CFG edge は `control_flow.md` に deferred する。
- nested statement を含むすべての `CoreAlgorithmStmt` row は、containing algorithm を owner として記録する。
  phase 10 はこの owner relation を信頼してよい。
- assignment は resolved target place と lowered value term を保存する。
- `if`、`while`、`match`、`return`、`break`、`continue` は phase 10 用の statement shell のままにする。
- algorithm `pick` statement は local binder と witness type fact を `CoreAlgorithmStmtKind::Pick` として
  記録する。stable choice symbol を再利用しない。
- ghost/runtime classification は metadata として保存する。elaborator は extraction-oriented erasure を
  実行しない。

task 13 の test は parameter/result、contract、assertion、invariant、decreasing term、statement
nesting/order、pick binding、ghost/runtime metadata、missing algorithm payload diagnostic を扱う。
executable algorithm statement の `the` occurrence は shared stable choice symbol ではなく local
`Pick` binding に lower しなければならない。ghost-only `Pick` site は後続 phase 10/11 の erasure と
checking のため、ghost metadata として mark されたままにしなければならない。

## diagnostics

elaboration diagnostic は deterministic order の `CoreDiagnostic` row を使う。local class は
`core_ir.md` の次のものを使う。

- `UnresolvedSemanticInput`。
- `InvalidErasure`。
- `UnsupportedLowering`。
- `MalformedProofSkeleton`。
- `SourceMapping`。
- `AlgorithmShell`。

diagnostic は `ResolvedTypedAst` 由来の source provenance、overload result や inserted view id などの
semantic provenance、存在する場合は owning core node を含む。public diagnostic-code allocation は
`external_dependency_gap` のままであり、task 7 は registry code を割り当てない。

## determinism

elaboration は worker count、map iteration order、diagnostic rendering に依存せず deterministic でなければ
ならない。

- item worklist は canonical source/module order で sort する。
- generated origin key は alpha-normalized semantic record を使う。
- erasure 由来の conjunction は stable predicate ordering を使う。
- source-map provenance は phase と source range で sort する。
- diagnostic は stable message key を持ち source/core order で emit する。
- skipped/error node は traversal order で保存する。

## 禁止事項

`elaborator` は以下をしてはならない。

- checker payload gap を埋めるために raw source syntax を inspect する。
- semantic identity に source display name を使う。
- name resolution、type checking、registration activation、cluster closure、overload selection、
  template inference を再実行する。
- failed semantic site を valid core term/formula に変える。
- すべての definition を eager inline する。
- `VcId` を割り当てる、CFG を構築する、proof search を実行する、proof acceptance を記録する、
  kernel を呼ぶ、artifact schema を emit する、cache/proof reuse anchor を発明する。
