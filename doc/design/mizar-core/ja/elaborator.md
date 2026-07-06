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
  error placeholder 用 generated origin。algorithm `pick` site は generated symbol ではなく、
  local statement-shell binder として表現する。
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
- explicit な template type-parameter inhabitation assumption と、checker
  existential-gate evaluation 由来の template type-actual gate result。
- type、predicate、functor parameter に対する explicit scheme-actual
  validation row。directional widening evidence、skipped guard obligation
  seed、substitution-composition metadata を含む。
- checker initial obligation と deferred evidence row。

出力:

- `CoreTypePredicate` application。
- core assumption と guard formula。
- view explanation provenance。
- schema parameter inhabitation assumption と template type-actual gate record。
- scheme-actual validation record と skipped guard obligation seed reference。
- carried obligation seed reference。

消去規則:

- declared binder type `x be T` は `x` の core binder と
  `TypePred { subject: x, ty: pred(T) }` guard/assumption になる。
- formula assertion `x is T` は同じ `TypePred` formula になる。
- attribute chain は deterministic predicate order の明示的 predicate fact conjunction に lower する。
  negative polarity は `Not(TypePred)` になる。
- mode/radix expansion は checker-normalized type head を通じて lower し、type syntax を再構成しない。
- source-written `qua` と inserted view は、この step では variable-subject fact の消去に
  必要な view provenance、任意の checker-owned reduct metadata、既に確立済みの type
  fact だけへ lower する。この step は term を作らない。Step 2 fact の subject は変数に
  限られるため、reduct term 上の view-specific attribute fact は Step 3 の formula seed である。
  `qua` view は新しい proof step ではない。
- template type parameter `T` は explicit checker-owned seed から schema context
  assumption `Exists { binders: [witness], body: TypePred(witness, is_T) }`
  へ lower する。seed は witness binder identity、role、source name、predicate、
  source、provenance を提供し、elaborator は source syntax から witness variable を
  作らない。
- template type actual は §17.3.4 の checker existential-gate result を保持する。
  `Satisfied` gate は inhabitation を正当化した checker registration id、base evidence
  pair、または guard fact を保存する。非 satisfied status(`MissingExistential`,
  `BlockedGuard`, `InvalidCandidate`, `DegradedRecovery`)は core diagnostic と
  checker-diagnostic backref だけを生成する。actual 側の existential axiom や proof
  obligation は生成しない。
- template type-parameter sethood row は、template parameter と checker-normalized
  sethood evidence key により §13.4.2/§18.10.2 の sethood 判定を保持する。
  `Accepted` row は `BoundInherited` または `ConstraintSupplied` evidence から来なければ
  ならず、明示的 checker fact を持たなければならない。`BareParameter` row は
  diagnostic-only の `Missing` evidence であり、裸の template type parameter 上の
  Fraenkel generator に対する規範的な拒否であって sethood source ではない。
  重複または矛盾する row は lower 前に fail closed する。`DegradedRecovery` row も
  diagnostic-only で、fact を持たず、参照された場合も Fraenkel sethood gate を
  満たせない。
- scheme actual は Chapter 18 instantiation に対する explicit checker validation を
  保持する。predicate / functor actual row は §18.10.4 が要求する directional
  widening evidence を記録する: schema domain type は actual の宣言 parameter type
  へ widening し、functor result type は schema codomain へ widening する。
  accepted functor row は traceability として `Skipped` checker-initial guard
  obligation seed を持つ。この guard は core assumption、active VC、または
  instantiated functor axiom ではない。result widening failure、partial algorithm、
  void algorithm、role mismatch、arity mismatch などの rejected row は diagnostic-only
  か、accepted evidence を lower する前に fail closed する。inner scheme actual として
  使われる enclosing template parameter は checker substitution-composition metadata
  だけを記録し、elaborator は新しい predicate/function/type symbol を作らず、
  proof-local `defpred`/`deffunc` closure を source から展開しない。
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
- Fraenkel generator が template type parameter 上を動く場合の、Step 2 から来る
  lowered template type-parameter sethood record。
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
- source-written / inserted no-reduct `qua` は underlying term と view provenance / type
  fact へ lower し、implicit cast node を作らない。
- source-written / inserted reduct `qua` は checker-owned `QuaPathKey` と明示的 view
  functor の順序付き list を持つ。Step 3 は、それらの functor を lowered base term へ
  順に入れ子に適用し、最後の view term を返す。attribute atom、`TypePred` formula、
  selector、bounded-template actual formula、明示的 exact-instance extensionality guard は
  `Qua` seed id を参照することでその view term を対象にする。core は
  `exact_Magma(view_path(x))` のような明示的 exact-instance guard formula を保持する。
  source-derived extensionality axiom を合成せず、checker payload が提供していない view
  path を推論しない。
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
- Fraenkel generator が template type parameter 上を動く場合、Step 3 は parameter と
  normalized evidence key により Step 2 の `TemplateTypeParameterSethood` row を参照する
  構造化された `TemplateFraenkelSethoodEvidenceSeed` を受け取らなければならない。
  参照先 row は accepted で、`BoundInherited` または `ConstraintSupplied` source を使い、
  normalized type key が一致し、checker evidence を持たなければならない。row の欠落、
  重複、誤った status/source、normalized-key 不一致、空 evidence は fail closed する。
  `BareParameter`/`Missing` row は意図的に raw sethood provenance を消し、
  missing-sethood error/deferred path へ lower する。
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

- Step 1 で準備した `CoreItem` row に対する stable な definition-to-item mapping と
  status/diagnostic delta。
- `CoreDefinitionTable` row。
- correctness obligation seed。
- generated origin use に結び付いた generated dependency record。

規則:

- definition boundary は stable で、body lowering より前に登録する。
- resolved declaration 由来の visibility と export metadata は、生成される `CoreItem` row と
  dependency summary に保存する。
- elaborator は expansion policy（`Opaque`、`Transparent`、`Reducible`、`Computable`）を記録するが、
  definition を eager inline しない。
- formal binder は binder-scope rule の下で normalize された type guard を含む。
- `set`、`deffunc`、`defpred` の使用は、definition-time closure から capture-avoiding substitution で
  展開する。この処理は binder-normalization の closure 機構を使う。captured free variable は
  display name ではなく stable id である。task 11 はすでに lowered された term/formula body を受け取る。
  closure use payload の source-to-checker extraction は、外部 checker payload が利用可能になるまで
  deferred のままである。
- conditional definition は ordered guarded branch に lower する。`otherwise` は checker-owned guard と、
  それが除外する prior guard の ordered list として表す。mizar-core はその payload を記録し、
  否定を合成または証明しない。coverage 欠落は明示的 obligation/diagnostic のままにする。
- existence、uniqueness、coherence、compatibility、reducibility、coverage check は obligation seed になる。
  seed には owning `CoreDefinition` と関連 body node への back-reference を入れる。この crate では accept しない。
- prerequisite が失敗した item は structured diagnostic 付きで skipped または error にする。
- algorithm-backed definition body は Step 4 では lower しない。`DefinitionBody::Unavailable` と
  skipped/error status delta を生成し、Step 6 まで deferred とする。
- generated dependency は、その dependency id が存在し、definition body の generated origin use record
  から到達可能な場合にだけ受け付ける。formula body 経由で到達する term も含める。

task 11 の test は definition boundary order、expansion policy、formal guard、
conditional branch、correctness seed emission、generated dependency、skipped/error item preservation、
local-abbreviation closure-use payload extraction の deferred boundary を扱う。task 11 実装 test は
Step 4 の seed boundary として、expansion policy、formal guard、checker-owned `otherwise` record、
definition correctness seed、formula body 経由で到達する generated dependency、skipped/error body、
invalid boundary input、deferred/existing correctness metadata を覆う。closure expansion 自体は
`binder_normalization` test で扱い、明示 closure-use body を checker payload から供給できるようになるまでは
source-extraction handoff とする。
stable choice を含む exported definition は、unfolding 時に definition-owned generated choice symbol を
再利用しなければならない。use site で fresh choice symbol を再生成してはならない。

## Step 5: proof-skeleton lowering

task 12 がこの section を実装する。

入力:

- theorem / lemma proposition。
- 利用可能な checker proof skeleton payload。
- proof label と citation。
- lowered formula と binder context。
- 先行 step の generated-origin output と definition output。

出力:

- `CoreProofTable` row。
- core proof node。
- theorem/proof status metadata。
- terminal proof goal obligation seed。
- proof diagnostic と、proof node / terminal obligation 用 source-map entry。

規則:

- `thesis` は current core formula に置き換える。magic identifier として保存しない。task 12 は
  checker-owned thesis reference を、skeleton payload が供給する proposition/current-goal
  `CoreFormulaId` へ lower し、明示的な current-goal transition を proof node として記録する。
- introduced variable は type guard 付き `CoreBinder` になる。
- assumption と labeled step は source/core provenance を持つ明示的 proof node になる。
- sequential proof block は順序を保存し、同じ proof path の earlier step label を後続 node へ渡す。
  branch child は label scope を分離し、sibling case の label は漏らさない。
- citation は semantic input に既にある label、canonical symbol、generated origin を参照する。
  mizar-core は symbol citation が proof-like な current-module item または dependency summary
  （`Theorem`、`Lemma`、`Scheme`）であることを検証し、functor や mode など proof でない item kind は
  reject する。また generated citation が既存 generated origin であることを検証するが、premise
  selection は行わない。
- label citation は proof skeleton に local である。labeled assumption または step は、その node を
  lower した後に label を導入する。citation は同じ proof path の earlier label だけを参照できる。
  同一 proof 内の duplicate label、forward label、sibling branch 由来の label、malformed label payload、
  存在しない cited symbol/generated origin は proof row を emit する前の proof-seed validation error として
  reject する。
- cases、suppose、now などの branching proof form は構造を保存し、open proof leaf に terminal goal
  seed を生成する。
- `open`、`assumed`、`conditional`、`error` status を記録する。このいずれも `mizar-core` で theorem を
  証明しない。task 12 は checker-owned skeleton status が `CoreProofStatus` へ map できることだけを
  検証する。item/proof status delta は downstream phase 用 metadata のままである。`Open` と
  `Conditional` proof は active terminal obligation を生成してよい。`Assumed` proof は terminal proof
  acceptance なしの assumed proof skeleton を記録する。`Error` proof は error root と diagnostic を使う。
  acceptance、discharge、dependency soundness check は external/deferred のままである。
- 明示的な malformed / missing proof skeleton payload は `MalformedProofSkeleton` diagnostic と error
  proof node を生成する。
- terminal proof goal は `ObligationSeedKind::TheoremProof` seed になる。seed には owning
  `CoreProof`、terminal proof node、theorem item、goal formula、周辺 proof formula への back-reference を
  入れる。この crate ではこれらを accept しない。
- citation 付きの justified conclusion step は `Step` node へ lower する。checker payload が既に
  justified と mark している場合、それ自体は terminal obligation ではない。open proof leaf と omitted
  justification payload は checker extraction が明示 terminal-goal seed として表現しなければならず、
  mizar-core は source text を scan して合成しない。明示 terminal-goal seed は `TerminalGoal` node と
  `TheoremProof` obligation へ lower する。terminal obligation の `core_refs` には owning proof、
  terminal node、theorem item、goal formula、active path formula、local に参照できる cited generated origin
  または proof-like symbol を `Generated` / `Item` reference として含める。元の citation list は durable
  terminal proof node と terminal-obligation citation record に保存し、label citation と external
  proof-like symbol citation は downstream VC/proof phase が使える symbolic citation として残す。external
  symbol のために fabricated core ref を作らない。
- proof skeleton lowering は VC を作らず、proof search や kernel 呼び出しを行わず、artifact schema id も
  割り当てない。

task 12 の test は thesis replacement と current-goal transition、sequential label propagation、
introduced binder、assumption、label/citation、branch kind、terminal goal seed、theorem status、
malformed proof skeleton diagnostic を扱う。theorem / lemma proposition fixture は、generated symbol が
theorem/lemma proposition context に owned され、proof-skeleton lowering を通して保存される stable choice を
含めなければならない。invalid citation、missing または wrong-owner proof item、active path formula、
external dependency citation、terminal-goal obligation back-reference も test で扱う。

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
- algorithm diagnostic と local `Pick` statement-shell binding。

task 13 の分類:

- `external_dependency_gap`: mizar-parser task 32-34 coverage を含む full algorithm statement payload の
  source-to-checker extraction はこの task の外に残る。test は checker-owned な明示 Rust fixture を使う。
- `deferred`: CFG construction、control-flow diagnostic、use-before-assignment、reachability、
  VC generation、proof/kernel handoff は phase 10 以降の task に残す。
- `design_drift`: generated pick origin と表現していた以前の文言をここで修正する。実行可能な `the`
  site は reusable stable-choice/generated symbol ではなく、local `Pick` statement に lower する。

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
- missing / malformed algorithm payload は `AlgorithmShell` diagnostic と error statement shell になり、
  valid executable body にはならない。

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

## public enum policy

task 21 は `elaborator` の public enum をすべて downstream forward-compatible API
surface として分類する。将来の context、lowering、seed、evidence、diagnostic、proof、
algorithm payload category を下流 crate の exhaustive match を壊さずに追加できるよう、
各 enum は `#[non_exhaustive]` を維持しなければならない。

| public enum | decision |
|---|---|
| `CoreContextError` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `DefinitionBoundaryKind` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `DefinitionBoundaryStatus` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `CheckerSiteKind` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `CheckerSiteSeverity` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `ElaborationWorkItemKind` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `ElaborationWorkStatus` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `TypeAndFactLoweringError` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `ViewExplanationKind` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `MissingEvidenceKind` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `TemplateSchemeParameterKind` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `TemplateSchemeActualKind` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `TemplateSchemeActualStatus` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `TemplateWideningEvidenceStatus` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `TemplateTypeParameterSethoodSource` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `TemplateTypeParameterSethoodStatus` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `TermAndFormulaLoweringError` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `TemplateSethoodRecordErrorKind` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `CoreTermSeedKind` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `CoreFormulaSeedKind` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `FraenkelMembershipObligationSeed` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `GeneratedOriginReuseSource` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `DefinitionLoweringError` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `DefinitionBodySeed` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `DefinitionGuardSeed` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `DefinitionCorrectnessSeed` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `ProofLoweringError` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `ProofSkeletonSeed` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `ProofNodeSeed` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `ProofFormulaRef` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `AlgorithmLoweringError` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `AlgorithmPayloadSeed` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `AlgorithmStmtSeed` | `#[non_exhaustive]` downstream forward-compatible surface。 |

この module が所有する exhaustive public enum exception はない。現在の variant を意図的に
列挙する `mizar-core` 内部 match は exhaustive のままでよい。

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
