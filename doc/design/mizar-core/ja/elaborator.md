# Elaborator

`lower_source_functorial_registration` は checker が封印した SourceRegistrationCheck のみから既存 Core の定義と全ガード付き関数型・簡約目標を生成し、SymbolEnv 参照、名前・型の再検査、要求キーの解析や受理済み効果を追加しない。
簡約経路は恒等関数の入れ子適用、単元集合または変数の右辺、定義と登録の別々の束縛、および目標に結び付く登録所有の実際の `thus thesis` 証明を保持し、ガードは追加仮定のない bare set に限定する。
両方の無引用経路は登録の識別情報を item/path に保持し、seed の前提ラベルを付けない。実際の reducibility seed に `vc-registration-style:reduction` を保持し、既存の `RegistrationStyleCorrectness` / `Reduction` VC family を選ぶ。

`lower_source_theorem_skeletons` は checker の不透明な `SourceTheoremCheck` と任意の `&SourceAlgorithmCheck` を受け、既存の source owner・
formula identity を Core の item・formula・proof seed で lower する。source range、
binder 対応、引用先、証明義務 provenance を保持する。修飾なしの証明付き owner だけを対象とし、
open/assumed の登録は checker/policy に留める。resolver 環境/Visibility を受け取らず、
Task-31 例外や汎用 checked-proof installation は拡張しない。 seal の照合は checker 所有の変数・等式 kind 対応と任意 resolved link の規約を保ち、node ID・子辺・anchor・recovery は完全一致させる。両 seal が一致する場合は厳密な void algorithm と claim theorem を一つの Core context で lower し、theorem item dependency と seed CoreNodeRef に実 algorithm を残す。義務 source は claim wrapper、theorem/proof 行は各 source と全称命題を保持する。seal なしでは claim wrapper を認めず、空実行 tuple や target を創作しない。

限定 computation theorem では `CoreTermSeedKind::Numeral(String)` が認証済み source zero の各出現を別々の `CoreTermKind::Numeral` に lower し、不正な桁列 seed は `TermAndFormulaLoweringError::InvalidNumeral` で拒否する。
既存 terminal lowering が実等式の義務と backref を作り、Core 検証前に node を認証済み steps 表記と computation justification source を持つ `ComputationGoal` にする。
theorem は `PendingAutomaticProof` のままで、途中の証明事実・引用・新しい proof seed 形状・raw source 解析は追加しない。

`normalize_source_membership_proof` は検査済みの借用 source transaction だけを受け取り、proof-local 束縛を対応させる。
列挙集合への所属、または引用された直接上界からの恒等 mapper への所属を簡約し、generic IR・空の VC receipt・kernel evidence は作らない。

> 正本は英語です。英語版:
> [../en/elaborator.md](../en/elaborator.md)。

Step 5C.2 の限定 structure normalizer は generic `CoreIr` と Task-180 VC behavior を変更しません。

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

### Core-33 C4C8 capture-context prerequisite

[Task CORE-FRAENKEL-CAPTURE-CONTEXT-33C4C8](../../task_contracts/ja/CORE-FRAENKEL-CAPTURE-CONTEXT-33C4C8.md)は
standalone immutable Step-1 handoff 1個を追加する。Existing Task33C checker receiptと
already prepared `CoreContext`をby-valueでconsumeし、retained whole owner `SymbolId`を
existing valid functor itemへmapし、captured outer generator identity 2件だけをfresh free
term `CoreVarId`としてinstallする。Allocationはchecker-authenticated private capture orderの
checked max-plus-oneであり、local inner generatorは除外する。Handoffはexact source/checker
provenanceとempty type factをretainedする。

このAPIはzero-semantic Core-33 association boundaryである。`CoreContextInput`、Typed、
Resolvedを拡張せず、parameter、argument、generated origin、term、formula、type evidence、
diagnostic、active routeを作らない。Core 35は該当するCore-33 local-binder/Core-34
prerequisite後にのみconsumeでき、allocate/infer/reorder/repairしてはならない。

### Core-33 standalone source local-binder prerequisite

[Task CORE-SOURCE-LOCAL-BINDER-CONTEXT-33LB](../../task_contracts/ja/CORE-SOURCE-LOCAL-BINDER-CONTEXT-33LB.md)は
separate immutable Step-1 handoffを追加する。Already prepared `CoreContext`とcomplete checker
`BindingEnv` 1個をby-valueでconsumeする。Admitされたreserve/defaultまたは
definition-parameter bindingごとに、Coreはchecked existing maximumより上へfresh free term
variableをallocateし、exact declaration range/checker provenance/empty type factをinstallし、
checker binding orderでexplicit `BindingId`-to-`CoreVarId` rowを記録する。

Handoffはcomplete `BindingEnv`をretainするため、context parentage、local scope、visibility、
identity、source provenanceはchecker ownerのままで、name/range joinへ依存しない。
Default-deny oracleはunsupported/recovered/partial binding stateおよびmissing/extra/duplicate/
reordered/stale/mismatched Core associationをrejectする。Source itemをreconstructせず、
`CoreContextInput`、`CoreContext`、`CoreIr`、Typed、Resolvedを拡張しない。Zero-semantic/
zero-creditであり、general Core-33 item associationと`MT10-CIR-TE`はdeferredのまま。

### Core-33 Task259 predicate item context prerequisite

[Task CORE-SOURCE-PREDICATE-ITEM-CONTEXT-33I259](../../task_contracts/ja/CORE-SOURCE-PREDICATE-ITEM-CONTEXT-33I259.md)は
33LB後のfirst nonempty standalone item associationを追加する。Complete 33LB handoff、exact
Task248 Profile-B source context、active Task259 predicate ownerをby-valueでconsumeする。
Predicate definitionのexact `BindingContextId`から`SourceContextLink`とcontaining
`SourceItemId` 1件を選び、retained whole `SymbolId`からexisting valid public predicate
`CoreItemId` 1件を選ぶ。Associationは4 identity全てをretainする。

Admit profileはpredicate/Core item/pending definitional boundary各1件だけ。Orderはchecker
definition table、range/provenanceはidentity-selected itemのvalidation専用でjoin keyではない。
Missing/extra/duplicate/reordered/stale/mismatch/recovered/diagnostic/partial/orphanはfail
closed。Field/installerを追加せず、one definition blockをmultiple semantic ownerへgeneralizeしない。

### Core-33 Task260 functor item context

完了した [CORE-SOURCE-FUNCTOR-ITEM-CONTEXT-33I260 contract](../../task_contracts/ja/CORE-SOURCE-FUNCTOR-ITEM-CONTEXT-33I260.md) が、Task260固有二行 Core-context association、immutable handoff、error surface、producer boundary と詳細 invariant を所有する。Implementation/verification/exact task-only commit `f8e9fc21` は complete。下記 public enum inventory に `SourceFunctorCoreContextError` を含める。これは zero-semantic/zero-credit の private-probe slice であり、production install、`CoreIr`、Core34--36、`MT10-CIR-TE`、diagnostic、Task277B は deferred のままである。

### Core-33 Task261 attribute item context

Freeze済みの [CORE-SOURCE-ATTRIBUTE-ITEM-CONTEXT-33I261 contract](../../task_contracts/ja/CORE-SOURCE-ATTRIBUTE-ITEM-CONTEXT-33I261.md) が、immutable `SourceAttributeCoreItemAssociation` row/table、`SourceAttributeCoreContextHandoff`、non-exhaustive error、producerとdurable invariantを所有する。Producerはcomplete 33LB、Task248 source context、Task261 checker handoffをby-valueでretainし、complete validation後だけpublishする。Admitするmappingはwhole `SymbolId`で選んだordinary public `Attribute` item 1件と、`SourceAttributeDefinitionId`をkeyとするsource-ordered association row 1件であり、source order、inner definition range、versioned provenanceはjoin keyではなくvalidation対象である。Implementation/verification/exact task-only commit `4c6ecafc2a9bee7a4eb6e3f27336733fc672bd57` はcomplete。これはzero-semantic/zero-credit・private-probe-onlyであり、generic definition-family API、`CoreContextInput`/Typed/Resolved/CoreIr slot、installer、production route、Core34--36 semantics、`GeneratedOrigin`、`MT10-CIR-TE`、diagnostic、Task277B readinessを追加しない。

### Core-33 Task262 mode item context

Freeze済みの [CORE-SOURCE-MODE-ITEM-CONTEXT-33I262 contract](../../task_contracts/ja/CORE-SOURCE-MODE-ITEM-CONTEXT-33I262.md) が、immutable `SourceModeCoreItemAssociation` row/table、`SourceModeCoreContextHandoff`、non-exhaustive error、producerとdurable invariantを所有する。Producerはcomplete 33LB、Task248 source context、Task262 checker handoff、association tableをby-valueでretainし、complete postvalidation後だけpublishする。Admitするmappingはwhole `SymbolId`で選んだvalid public `Mode` item 1件と、`SourceModeDefinitionId`をkeyとするsource-ordered association row 1件であり、inner rangeとversioned provenanceはjoin keyではなくvalidation対象である。Error precedenceは`EnvironmentMismatch`、`InvalidSourceBindingContext`、`InvalidCheckerOwner`、`InvalidCoreContext`、`InvalidItemAssociation`。Build signatureは`SourceModeCoreContextProducer::build(SourceBindingCoreContextHandoff, SourceBindingContextHandoff, SourceModeDefinitionHandoff) -> Result<SourceModeCoreContextHandoff, SourceModeCoreContextError>`。Implementation/verification complete、documentation repair後final review no findings。これはzero-semantic/zero-credit・private-probe-onlyであり、RHS、inhabitation、`sethood`、computation justification、pending obligation、generic API/slot、installer、production route、Core34--36 semantics、`GeneratedOrigin`、`MT10-CIR-TE`、diagnostic、Task277Bはdeferred/not-ready。

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

### Task 267 exact Task-180 adapter boundary

Core Task 31 は narrow `lower_exact_task180_handoff` adapter を追加し、Task 267
はその仕様だけを定める。adapter は complete Task-268 singleton checker bundle
を preflight し、generic lowering input をすべて local unpublished state で作り、
exact identity/status/cardinality predicate が通った後だけ
`lower_proof_inputs` を呼ぶ。terminal `ProofFormulaRef` は
`Formula(CoreFormulaId(0))` であり、`Thesis` ではない。

production signature は public/borrowed とする。

```rust
pub fn lower_exact_task180_handoff(
    resolved: &ResolvedTypedAst,
) -> Result<CoreIr, ExactTask180LoweringError>;
```

`ExactTask180LoweringError` は public non-exhaustive typed error で、checker-bundle
preflight、generic lowering、exact provenance enrichment、final projection
validation をそれぞれ区別する `InvalidCheckerBundle`、`GenericLowering`、
`ProvenanceEnrichment`、`InvalidProjection` variantを持ち、partially lowered
output を所有しない。adapter は
`core_ir.md` の strict input allowlist を検査後に private core-owned draft を
extractし、free-standing proof-table inputを公開したりchecker table mutatorを
追加したりしない。private `cfg(test)` failure selectorは4 categoryを強制してよいが、
public APIには含めない。

このboundaryで許すresolver-environment metadataは、checker-authenticated済みの
`Visibility::Public`/`ExportStatus::Exported` fieldのtyped comparisonだけである。
`SymbolEnv`を許可せず、name resolutionを行わず、existing frontend/downstream
boundary guardを弱めない。

generic `CheckerOwnedProvenance` rule は変更しない。terminal
`CoreSourceRef` は source provenance に exact `ProofSkeleton(K)` をあらかじめ
持って generic lowering に入り、`CheckerOwnedProvenance` は `Checker(T)` だけ
を持つ。これにより `ProofSkeleton` を checker-owned input として許可せず、
required terminal-node/seed source を生成する。generic lowering 後、exact
adapter は sole final `ObligationSeed.provenance` に `ProofSkeleton(K)` を追加し、
sort/dedup して exact phase order `[Checker(T), ProofSkeleton(K)]` にし、count、
dense id、relation、source/source map、provenance key、empty field、status の
complete Task-267 postcondition を検証する。

preflight、generic lowering、enrichment、postcondition のいずれかが fail すれば
`Err` を返し local value を破棄する。`CoreIr`、error proof、partial item、partial
source map は publish しない。generic proof caller と accepted provenance phase
は不変である。この adapter は formula の proof/discharge、public theorem の
verified premise 化、VC build、exact Task-180 source 外への generalization を
行わない。

traceabilityはexisting contradiction sidecarとcommitted CoreIr baselineに裏付けられる
new exact Task-31 snapshot requirementを追加する。idは
`spec.en.mizar_core.core_ir.task180_type_elaboration_snapshot`、stageは
`type_elaboration`、statusは`covered`、coverageは`snapshot`、sole test/backlinkは
existing contradiction expectation pathとし、sidecarはそのidを`spec_refs`へ追加する。
older broad CoreIr/
type-elaboration snapshot requirementはCore Tasks 33-47に割り当てた全non-Task-180 familyについて
empty tests/deferredのままとし、Task 31はそのbroad rowをpromoteしない。
`spec_coverage_audit.md`はこのexact-only splitを記録する。

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
| `SourceNestedFraenkelCaptureCoreContextError` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `SourceBindingCoreContextError` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `SourcePredicateCoreContextError` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `SourceFunctorCoreContextError` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `SourceAttributeCoreContextError` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `SourceModeCoreContextError` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `SourceStructureCoreContextError` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `SourceStructureCoreNormalizationError` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `SourcePropertyCarrierCoreContextError` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `SourcePropertySelectorTypeContextError` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `SourcePropertyParameterCoreContextError` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `SourcePropertyEqualsSelectorTermSeedError` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `SourcePropertyEqualsSelectorTermLoweringError` | `#[non_exhaustive]` downstream forward-compatible surface。 |
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
| `ExactTask180LoweringError` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `AlgorithmLoweringError` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `AlgorithmPayloadSeed` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `AlgorithmStmtSeed` | `#[non_exhaustive]` downstream forward-compatible surface。 |

この module が所有する exhaustive public enum exception はない。現在の variant を意図的に
列挙する `mizar-core` 内部 match は exhaustive のままでよい。

## Task-32 source-derived follow-up

[source_family_decomposition.md](./source_family_decomposition.md) はCore Tasks
33-47と`MT10-CIR-*` consumerを割り当てる。Algorithm Tasks 42-47はbounded joint
vertical taskであり、ASTは`mizar-test`だけが読み、checkerはsyntax-free final
projectionをemitし、このcrateがlowerする。Core 34はreusable conversion/evidence
API、Core 37はsource `reconsider`を所有する。Core 46はcall/result substitution
requestだけを搬送し、substitution result/VCはdownstreamに残す。

## Task 33I263 standalone structure item context

Canonical [Task 33I263 contract](../../task_contracts/ja/CORE-SOURCE-STRUCTURE-ITEM-CONTEXT-33I263.md)
がstandalone `SourceStructureCoreContextProducer`、immutable two-row
definition/Core-item association、handoff、typed failure surfaceをownする。
Prepared `CoreContext`とexact Task263 checker handoffだけをconsumeし、public valid
`Structure` 2件をauthenticateしてdirect inheritanceをDerivedからBaseへのsole local
dependencyとしてmaterializeする。

Task248/33LBはinputではない。Member/constructor/mapping/view/coherence/
obligation/acceptance artifactはvalidation-onlyまたはdeferred。Generic adapter、
context/IR slot、installer、production routeは追加しない。

## Task 33I264 Task264 carrier item context

Canonical [Task 33I264 contract](../../task_contracts/ja/CORE-SOURCE-PROPERTY-CARRIER-ITEM-CONTEXT-33I264.md)
がsame-source Task264 carrier向けseparate singleton associationをownする。Task263の
different-source two-row APIはreuse/generalizeしない。

Durable public surfaceは`SourcePropertyCarrierCoreContextHandoff`、non-exhaustive
`SourcePropertyCarrierCoreContextError`、
`SourcePropertyCarrierCoreContextProducer`だけ。Handoffはprepared `CoreContext`、
complete `SourcePropertyImplementationHandoff`、private `CoreItemId` 1件をby-value
retainする。Getterは`source_id()`、`module_id()`、`context()`、`checker_owner()`、
`carrier_item()`、`debug_text()`で、public association row/tableはない。Producer entryは
`build(CoreContext, SourcePropertyImplementationHandoff)`だけである。

Producerはimmutable three-role receipt、target/property link、source/module、one-item Core
shapeをrecheckする。Exact whole-symbol lookupがrange `13..101`、dependency/diagnosticなし、
checker provenance `source-property-carrier-core-item-v1.structure`のpublic valid
`Structure` item 1件を選ぶ。Item source、pending `DefinitionalItem`、pending worklistは
各1件で、binder/checker site/dependency summary/generated origin/other Core/source-map
domainはempty。Environment→checker→Core→scalar association順にvalidateし、complete
postvalidation後だけpublishする。

Exact no-final-LF debugは
`source-property-carrier-core-item-context-v1|module=<package>.<path>|carrier=<whole-fqn>:0:0|item=<id>`。
Property-implementation shell/field/propertyにはCore itemを作らず、selector/type ownerは
Core34に残す。Dependency/item-kind/semantic lowering/installer/production route変更なし。

## Task 34I264 Task264 selector/type context

Canonical [Task 34I264 contract](../../task_contracts/ja/CORE-SOURCE-PROPERTY-SELECTOR-TYPE-CONTEXT-34I264.md)
がseparate immutable Core34 authentication contextをownする。Complete Task33I264
handoffとexact Task249PI `SourceTypeApplicationHandoff`をconsumeし、retained
fingerprintをcomplete typed rowへreplay validateして、checker targetの`marker`
propertyからexplicit return member/root `1/2`へのsole authenticated edgeをderiveする。
Lower handoffは`carrier` field→member 0をauthenticateしないためfield associationなし。

Durable public surfaceはassociation、handoff、non-exhaustive error、unit
producer。両complete inputをby-value retainし、environment→carrier→source type→
association順のcomplete postvalidation後だけpublishする。
Associationはwhole `SymbolId`、member id、root idだけをexposeする。Typed
definition/contribution identityはretained checker owner内でvalidateし、resolver-
environment typeをCore public boundaryへ入れない。

Selector item/`CoreDefinition` row/body、`CoreTypePredicate`、normalized type、fact/
coercion/view/binder/term/formula/diagnostic/semanticsはpublishしない。

## Task IR264 Authenticated Property Definition Owner

Frozen [IR264 contract](../../task_contracts/ja/CORE-STRUCTURE-PROPERTY-DEFINITION-OWNER-IR264.md)
は`SourcePropertySelectorTypeContextHandoff::definition_owner`を追加する。Inherent
implementationはproperty-form owner fieldをprivateにownする`core_ir.rs`内にあり、他moduleは
raw valueからownerをmintできない。Validated source/module/carrier item/sole `marker` symbol
だけを`CoreDefinitionOwner`へcopyし、field-owner constructor/mutatorは公開しない。

Private Task264 testはmeans/equals handoffがcarrier item 0とauthenticated property symbolだけを
返すことを証明する。Zero-credit owner valueであり、selector item、`CoreDefinition` row/body、
normalized type、term/formula/obligation/production route/semantic resultではない。
Core35/36はseparateのまま。

## Task 34D264 Task264 domain/return type input

Canonical [Task34D264 contract](../../task_contracts/ja/CORE-SOURCE-PROPERTY-DOMAIN-RETURN-TYPE-34D264.md)
はcomplete Task34I264 handoffへprivate immutable
`SourcePropertyDomainTypeAssociation` 1件を追加する。Accessorはchecker parameter
binding、written-type application、domain root、existing Task33I264 carrier itemだけを
公開する。Existing selector associationはreturn sideのままで、`marker`とreturn
member/root `1/2`だけを公開する。

Constructionはparameter 0からbinding/application/root `0/0/0`を辿り、property target
0のsubjectがsame bindingであること、whole `Task264Carrier` symbol/contributionのreplay、
retained carrier item 0へのexact registry lookupを要求する。Postvalidationはpublish前に
全linkを再検証する。Existing v1 debug byteは不変。Normalized type、predicate/guard、
binder、field/member association、term/formula/definition/obligation/route/semanticsは作らない。

## Task 33P264 Task264 parameter Core context

Canonical [Task33P264 contract](../../task_contracts/ja/CORE-SOURCE-PROPERTY-PARAMETER-CONTEXT-33P264.md)
はchecker property parameter 0/binding 0からexisting Task33LB free term Core variableへの
private-field association 1件を追加する。Producerはcomplete Task34I264/34D264 selector
contextとexact Task264 source contextをconsumeし、retained carrier context/binding envの
exact clone上でexisting `SourceBindingCoreContextProducer`を呼ぶ。

Handoffはcomplete input 2件、complete derived 33LB handoff、association `0/0/0`をretain。
Replayは33LB valueをdeterministically rebuildしてcomplete compareする。Result contextは
carrier item 0を維持し、free term variable 0/checker declaration source/empty type-factだけを
追加する。Binder frame、normalized type、term/formula/`it`/definition/route/diagnostic/
obligation/semantic creditは作らない。

## Task 35E264 Task264 equals selector term seeds

Canonical [Task35E264 contract](../../task_contracts/ja/CORE-SOURCE-PROPERTY-EQUALS-SELECTOR-SEEDS-35E264.md)
はstandalone Core35-input handoffを追加する。Complete Task33P264 parameter context、
Task264D equals-selector identity、IR264 property definition ownerをretainし、exact two
immutable local seedだけをpublishする。Seed 0はauthenticated `M` useの
`Var(CoreVarId(0))`、seed 1は`M.carrier`のwhole carrier-field symbolを使う
`Select { base: seed 0 }`。

Generic term/formula loweringはcallしない。Generic inputはordinary `CoreItemId`を要求するが、
IR264 property ownerは意図的にnon-itemである。Carrier item 0はowner anchorとしてretainする
だけでproperty ownerに代用しない。Seed-local id/use-site source/checker provenanceはlater
owner-aware lowerer向けinputであり、`CoreTermId`/term table/source map/formula/definition body/
route/semantic creditはpublishしない。

## Task 35L264 Task264 equals selector term lowering

Implemented [Task35L264 contract](../../task_contracts/ja/CORE-SOURCE-PROPERTY-EQUALS-SELECTOR-LOWERING-35L264.md)
はcomplete Task35E264 capability上のspecialized pure-term lowerer 1件をownする。
Authenticated non-item property ownerをdelegateし、local dense term tableへterm 0
`Var(CoreVarId(0))`、term 1
`Select { selector: <whole carrier-field SymbolId>, base: term 0 }`をpublishする。
Seed/term/root associationは`0->0`、`1->1`、root 1。

各`CoreTerm.source`はTask35E264 checker provenanceをexact direct sourceへmergeする。
Term-only `CoreSourceMap`はsame term source 2件だけを持つ。Table/idはunattached local
lowering resultで、CoreIr install/definition-body attachment/carrier item owner substitutionはない。

Generic term/formula loweringはcall/changeしない。同pathはgenerated origin/diagnostic/
obligation向けordinary item ownerをrequireし続ける。Formula/generated origin/diagnostic/
obligation/definition/route/acceptance/fact/coverage creditなし。

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

## Static source algorithm lowering

`lower_source_algorithms(&SourceAlgorithmCheck) -> Result<CoreIr, String>` は封印された静的 profile または平坦な object 状態 profile を消費する。実際の仮引数・局所変数・result 同一性、記述された return 型、var/const/ghost role、初期化子、文順序と checker 由来を既存 lowering で保持する。直接変数への書込みは検査済み代入先 BindingId から型付き AssignLocal seed にし、同じ algorithm の束縛を要求する。opaque CorePlace 文字列に束縛同一性を符号化しない。ソース所有の式 map は header 等式だけを CoreContractSet.ensures に、本文 assertion を各 Assert seed に渡し、契約 result 束縛を既存の独立した暗黙 result slot に対応させる。prefix 否定等式は既存 Equals と Not(child) に下げ、両 checked formula の同一性・親子辺・範囲・由来を保持し、Assert は外側を参照する。source map と空の Core obligation 表の境界を維持し、配置と静的代入診断は CFG、値の世代と具体的な式は VC が所有する。生の resolver 権限、algorithm 承認、terminating 昇格を導入しない。

## Source existential-registration proof

`lower_source_existential_registration(&SourceRegistrationCheck) -> Result<CoreIr, String>` は
[checker
seal](../../mizar-checker/ja/registration_resolution.md#source-existential-registration-proof)
を消費し、元の属性付き Exists とガード付き定義を保持する。認証済みソース引用は proof Symbol ではなく正規化された `Label("definition:<attribute FQN>")` として Step と Terminal に渡す。定義 formal・存在束縛・choice を区別する。別の active GeneratedNonEmptiness seed
は新しい local binder と厳密な `Exists z. is_set(z)` だけを持ち、追加 conjunct/前提はない。

proof 所有 StableChoice は実 bare-set 正規化 key・generated functor・空 params を持つ。両実出現はその functor の空引数適用と別々の実
range であり、空集合に置換しない。既存 CurrentGoal/Step/Sequence/TerminalGoal と型付き CoreNodeRef が両 seed・origin・term・型
step・引用終端を結ぶ。全 formula/term/source map・順序・到達性・所有者を検証し、provenance 文字列を gate receipt としない。Take DTO・raw
environment・checker 再評価・受理状態は追加しない。

## Three-registration source-proof extension

実装した集約 profile は既存 lower_source_existential_registration signature で実定義2件と元の存在目標3件を保持する。単一登録の graph は変えない。
各親は独立 stable-choice origin、実出現 range 2件、active な bare-set nonempty seed を持つ。formal・存在・nonempty 束縛の実同一性と capture 境界を保つ。
連言登録は同じ証人上の両属性述語、両定義の正確な canonical citation、型付き definition/backreference と source map を保持し、一方の conjunct から他方を推測しない。
既存 table に全証明を格納し、新しい公開 record・callback・部分ソースを追加しない。[VC generation](../../mizar-vc/ja/generator.md#three-registration-source-proof-extension) が親ごとの展開を所有し、Core は受理しない。

## Flat snapshot lowering

実装済みの `lower_source_algorithms` 拡張は seal の名前・BindingId を `AlgorithmStmtSeed::Snapshot { name, captures, source, provenance }` で [既存 statement table](./core_ir.md#flat-snapshot-statement) へ渡す。source node を漏れなく対応させ、捕捉宣言を同じ algorithm の実変数割当で変換する。snapshot は binder・項・式・証明 seed を追加しない。

## Source phrase predicate theorem

`lower_source_theorem_skeletons` の厳密な phrase 分岐は checker が封印した source receipt のみを消費し、resolver 環境や型の再検査を持ち込まない。
実際の predicate item/symbol、順序付き仮引数 binder、等式本体の `CoreDefinition` と `ExpansionPolicy::Transparent` を保持し、この宣言は correctness や accepted-proof receipt を持たない。
定義由来の context は既存 Core formula 種別と実際の仮引数 identity による `forall x,y ((set(x) & set(y)) -> (P(x,y) iff x=y))` とする。
定理命題は `forall q (set(q) -> P(q,q))` を保持し、別の局所 binder `l` の終端は実際の goal `P(l,l)` と順序付き context `[set(l), guarded definition]` を持つ。
出現と宣言の対応で producer binding と定理 scope を結び、数値 ID を混同しない。source map、definition/formula/term backref、依存と証明 provenance を完全に保つ。
Active TheoremProof seed は一つで、定理は `PendingAutomaticProof` のままとし、定理 fact、証明 discharge、大域的な定義 installation、Task274 acceptance、一般 predicate unfolding は行わない。

## 限定 symbolic predicate theorem

既存の封印済み `lower_source_theorem_skeletons` は実 body `forall z (object(z) -> (z in X -> z in Y))` を guard 付き predicate 双条件内に lowering する。既存 Forall/Implies/Iff と primitive Membership seed を使い、formal 二つ・body 束縛子・theorem 束縛子・proof-local 束縛子の五宣言を区別する。source/formula/term map と definition 所有 provenance で両 membership 辺と capture 境界を認証し、body オペランドと宣言の zip から formal を推測しない。実 user-symbol predicate goal、順序付き local-set-guard/definition 文脈、未解決 terminal proof の種類は維持する。unfolding・correctness 証明・受理済み事実を付与しない。

## 限定 while lowering

`lower_source_algorithms` は既存 seal から実際の条件・invariant・空の decreasing と入れ子 `AssignLocal` を持つ既存 `AlgorithmStmtSeed::While` を生成する。注釈は assert やアルゴリズム全体の invariant に変換しない。入れ子ソースノードを過不足なく認証し、式・文・所有者の対応を保つ。並行構文表現や証明を生成しない。

## 限定 means-prefix functor theorem

封印済み source-theorem 経路は実 user-functor 適用と Opaque な式定義を保持する。source `it` を適用に置換するが、その等式を前提として公開しない。既存 definition-correctness seed は guard 付き存在 `forall x:set exists r:set r=x` と一意性 `forall x:set forall r1,r2:set ((r1=x & r2=x) -> r1=r2)` を別の生成結果束縛子・節由来で表す。定理は `forall q:set F(q)=q` のまま、別の局所束縛子は set guard だけの下で `F(l)=l` を終端とする。二つの active definition seed と PendingAutomaticProof 定理に属する一つの Active TheoremProof seed の所有者・source map・backref を保持する。定義登録・unfolding・受理済み事実・新 Core 表現は追加しない。

## 限定 return Pick lowering

封印済み `return the set` は既存 runtime `Pick` 一つと、その新しい hidden binder を返す `Return` に lower し、実際の set predicate を保持する。Pick のソースは所有 return 範囲内の choice 出現とし、この組だけは平坦な非重複範囲検査に代えて認証済み包含関係を検査する。別の新しい論理 binder で `exists q. is_set(q)` を構築し、この Pick と algorithm への既存 Core 参照および完全な source map を持つ Active `GeneratedNonEmptiness` seed にする。既存の項・式・seed を使い、hidden binder をソース契約に公開せず、StableChoice origin・証人・受理済み inhabitation 証明・停止性昇格を生成しない。

## 限定 equals-infix functor theorem

既存 sealed 経路は実仮引数参照 RHS を、二つの set 仮引数を持つ Opaque な term 定義として保存する。§16.6.2 に従い、実 RHS t と宣言結果型による `forall x,y ((set(x) & set(y)) -> set(t))` を一つの Active DefinitionCorrectness seed にし、節・source provenance と body/goal の完全な backref を保持する。定理 `forall q1,q2:set F(q1,q2)=q1` と二局所 IntroduceBinder の連鎖を保ち、set guard のみの下で `F(l1,l2)=l1` を終端とする。一つの Active TheoremProof seed は PendingAutomaticProof 定理に属する。演算子・引数順序と六つの別々の束縛同一性を保つ。未解決の定義等式・coherence goal・静的結果型を受理済み前提にせず、unfolding・登録・新 Core 表現を追加しない。

## Source registration formula payloads

`source_registration_formula_payloads(check: &SourceRegistrationCheck<'_>, core: &CoreIr, owners: &BTreeMap<SymbolId, Vec<u8>>, roots: &[CoreFormulaId]) -> Result<BTreeMap<CoreFormulaId, Vec<u8>>, String>` は変更しない lower_source_existential_registration を再実行して Core 全体の一致を要求し、その error を保つ。他の不足は内部 error unsupported source registration formula payload とする。既存の一件・三件 registration profile のみを対象とし、実 Attribute/Registration item と owner key の完全対応、非空で相互に異なる value、一意かつ解決できる root を要求する。失敗は部分 map なしの Err、空 root は検証後の空 map。owner bytes は同じ真正な source に対する未変更の [Resolver 結果](../../mizar-resolve/ja/symbols.md#source-registration-owner-payloads) を caller が渡す責任を持ち、この検査は由来や freshness を認証しない。
実 definition param の共有 formal と role から既存 BinderContext に Free/Term を登録し、各 root に変更しない normalize_core_formula/canonical_formula を用いる。失敗は map 全体を保留する。bound variable は canonical de Bruijn index、唯一の実 shared Free formal は slot zero とし、旧 canonical/normalized 構造と alpha-equivalence を変えない。attribute は実 predicate SymbolId の opaque owner bytes を使い、引数なし choice Apply は実 StableChoice origin の functor と registration item の SymbolId を結んで owner bytes と実 semantic key を使う。suffix の解析を根拠にしない。
value は UTF-8 mizar-core/source-registration-formula/v1 と zero、package・module path・shared-formal role の順に checked big-endian u32 UTF-8 byte length と bytes を持つ。formula tag は 1 Atom、2 Equals、3 TypePred、4 Not、5 And、6 Exists。term は 1 Bound と u32 index、2 shared Free formal と u32 zero、3 choice と length-framed owner bytes/semantic key。Atom は framed attribute owner・u32 引数数・順序付き term、Equals は左右、TypePred は framed type spelling と subject、Not は child、And は u32 個数と順序付き child。Exists は u32 binder 数・各 framed role と zero guard marker・body を書く。この profile の binder guard はなく、他 shape/guard/variable は全体 Err とする。公開診断は追加しない。
これは明示的な入力責任の下で選択 root の未 hash Core 内容を返すだけであり、source/row/contribution/structural ordinal、Free CoreVarId、generated functor の一時 SymbolId、binder 表示名、range、debug/provenance text を bytes に含めない。既存 ID、body/child 順序、kernel evidence naming は維持する。canonical VC/context hash、完全 anchor/dependency、accepted evidence、reuse key、実行 receipt ではなく、VC の generated formula/provenance/full fingerprint と真正入力を結ぶ将来の source factory は別の責任に残る。
テストは実 single/aggregate source、現在の root 対応、六 shape と shared-formal/bound/choice role の独立 framed 期待値、中立編集と実 allocator relocation、module/owner/body の変更、foreign seal/Core と欠落・重複・未対応の拒否を扱う。

## Source theorem formula payloads

`source_theorem_formula_payloads(check: &mizar_checker::type_checker::SourceTheoremCheck<'_>, core: &CoreIr, roots: &[CoreFormulaId]) -> Result<BTreeMap<CoreFormulaId, Vec<u8>>, String>` は既存 canonical node から限定した unhashed Core 選択内容を生成する。新表現、汎用 serializer、envelope、Kernel codec は作らない。
最初に未変更の `lower_source_theorem_skeletons(check, None)` を replay して Core 全体の一致を要求する。lowering error は保持し、それ以降の profile/root/normalization/canonicalization/framing failure は厳密な内部 error `unsupported source theorem formula payload` で全体を拒否する。部分出力や公開診断を作らず、入力は不変。
Core item は依存・診断のない Valid `Theorem` の一つだけで Lemma を除く。その item が所有する `PendingAutomaticProof` proof 一つは、node 診断なしの `IntroduceBinder`、引用なし `TerminalGoal` の二 node だけを持つ。
同じ item 所有の Active `TheoremProof` seed 一つが終端 obligation と一致し、実 goal が存在し、context は厳密に `[local.ty_guard]`、label・診断はない。guard/local/goal は replay した proof/seed からのみ導出し、VC 検証や全到達 table の走査を重複しない。
実 proposition は元の quantifier binder 一つの `Forall` で、導入 local とは別の identity、実 role は `quantifier`/`proof-let` とする。definition、algorithm、algorithm statement、generated origin、Core 診断はない。
両実 guard/goal を常に未変更の `normalize_core_formula`/`canonical_formula`、実 source reference、実 local をその role で Free/Term 宣言した `BinderContext` により処理する。[汎用 binder identity](./binder_normalization.md#alpha-equivalence) は変えない。
canonical shape は `TypePred(set, Var(Free(actualLocal)))` と `Equals(Var(Free(actualLocal)), Var(Free(actualLocal)))` だけを認める。slot zero 出力前に等式両項が認証済み実 local を指すことを要求する。実 guard/goal が不一致ならその root が未要求でも拒否し、どちらも元の量化変数を使えない。
選択可能なのは互いに異なる実 guard/goal id 二つだけで、要求 root は重複なく存在しなければならない。元の bound guard/body、proposition、未知・余分な root は拒否する。部分集合・順序変更を認め、空 roots も全 profile と両 shape の検証後だけ空 map を返す。
payload は UTF-8 `mizar-core/source-theorem-formula/v1` と zero、続いて package/module path/実 local role を各 checked big-endian u32 UTF-8 byte length と bytes で書く。formula tag は登録内容と同じ意味で Equals 2 の後に左右、TypePred 3 の後に長さ付き実 type spelling と subject を書く。各許容 term は tag 2 と big-endian u32 zero。
この単一 local 投影は汎用 free-variable identity/alpha-equivalence を変えない。dense id、range、source/display/debug/provenance text、hash は bytes に含めず、実 role/type/child 順は保つ。bytes は source currentness、VC/context identity、anchor/dependency、witness/reuse、proof/acceptance 権限を与えず、VC が formula/provenance/fingerprint を所有する。
実 receipt を callback 内に保つ `source_theorem_formula_payloads_*` test family は正例・拒否例の二つだけ。正例は元/alpha/trivia source、元/local binder の区別、独立 framing の厳密 bytes/実 key、反復・root 順序/部分集合/空、purity と neutral edit を検証し、Core id/status と normalizer free variable を保つ。allocator relocation は実際に利用可能な場合だけ使う。
拒否例は実 receipt に foreign/mutated Core、重複/未知/元量化 root、実 Lemma と利用可能な computation/richer profile を組み合わせ、空 roots でも検証を迂回しない。既存 malformed-normalizer coverage を再利用し、matrix 複製、seal 捏造、accepted/completed/cache control は追加しない。
必要な private `with_step5c14_theorem_check<T>(text: &str, consume: impl FnOnce(&SourceTheoremCheck<'_>) -> Result<T, String>) -> Result<T, String>` は既存 parse/resolve/scope/typed/owner/labels/check pipeline を機械的に保ち、末尾だけ `consume(&check)` にする。実 call site 三つは signature 不変の `step5c14_computation_core` lowering wrapper と新二 family callback。旧 helper の signature/末尾以外の bytes と helper/最終 module brace 外の旧 code を保ってから test を追記する。pipeline 複製、所有自己参照、production route は追加しない。

## Source theorem Core semantic payload

`source_theorem_core_semantic_payload(check: &mizar_checker::type_checker::SourceTheoremCheck<'_>, core: &CoreIr) -> Result<Vec<u8>, String>` は以下の限定した単一 local 定理 Core の未 hash canonical 内容を返す。lossless source/storage 再構築は別に所有する。
実 seed の context/goal id を早期失敗なく集め、未変更の `source_theorem_formula_payloads` を一度だけ呼ぶ。その実 error String をそのまま伝播し、真正な Core 全体 replay と旧 profile の検査を先行させる。追加 profile・normalization・framing の不足は厳密な内部 error `unsupported source theorem core semantic payload` とし、部分出力・入力変更・公開診断は作らない。
旧操作の item/proof/seed/local profile 全体を保持する。さらに term 六行・formula 五行を要求し、互いに異なる実 proposition・元 guard/body・local guard/goal と六つの operand/subject id が全行を覆う。余分・未到達の行は拒否する。seed `core_refs` は実 item・proof・terminal node・local guard・goal の参照を canonical 順に並べた集合と厳密に一致し、余分な参照を持たない。
実 `proof.proposition` 全体をその source reference と空 `BinderContext` により normalize/canonicalize する。`quantifier` binder 一つの `Forall`、存在する `TypePred(set, Bound(0))` guard、順序付き body `Equals(Bound(0), Bound(0))` を要求する。元/local の宣言型一致と bound-to-local 置換は既存 opaque source check が所有し、不適合 source の receipt は捏造しない。汎用 binder/free-variable identity は変えない。
出力は UTF-8 `mizar-core/source-theorem-core-semantics/v1` と zero に続き、checked little-endian u64 byte length と bytes の十二 field を次の順に持つ: Core module package/path、item SymbolId module package/path・local・FQN、実 visibility、元 canonical tree、実 local guard payload、実 local goal payload、seed local_path、seed semantic_origin。全加算を検査し、32 MiB を超える出力は拒否する。
元 tree は formula tag 7 Forall、big-endian u32 binder 数、framed 実 role、guard-present byte 1、tag 3 TypePred、framed 実 type、subject term、tag 2 Equals と順序付き左右 term を持つ。ここで frame は checked big-endian u32 UTF-8 byte length と bytes、各実 term は tag 1 Bound と big-endian u32 index。検査した実 canonical value だけを出力し、local payload は旧 domain/grammar を保つ。
versioned profile は検査済み Valid Theorem・PendingAutomaticProof・Active TheoremProof、IntroduceBinder から引用なし TerminalGoal への graph、owner/context/ref 対応、依存・label・診断・他 table の不在を表す。別 status/graph は表現しない。実 SymbolId 各成分を解析・置換せず保持する。dense row/variable id、binder 表示名、source range/map/provenance は内容から除き、storage/source 対応に保持する。
汎用 Core codec、theorem-interface/proof-body fingerprint policy ではない。hash、完全 elaboration/dependency footprint、currentness、phase completion、artifact publication、reuse、proof acceptance、Steps 6–8 の達成を与えず、footprint 不足は [dependency contract](../../architecture/ja/18.dependency_fingerprint.md#依存スライス) に従い uncacheable のままとする。
新 `source_theorem_core_semantic_payload_*` callback test family は二つだけで、旧 source-check helper は変更しない。新 API 前に真正 lowering・全 row/graph/status 対応・元 canonical shape・独立した旧 local payload 期待値を確認する。別に framing した全出力の厳密期待値、反復、引数不変性を検査する。同じ owner の bound/local alpha rename と trivia は Core が変わっても bytes を保ち、実 theorem owner の改名は bytes を変える。拒否例は foreign Core/check、Lemma、両方 object の guard、open status の真正な旧 error を先に確認し、そのままの error を要求する。元/local 不整合は既存拒否 test が所有し、正例 receipt 捏造・matrix 複製・新 helper は行わない。

## Source algorithm formula payloads
`source_algorithm_formula_payloads(check: &mizar_checker::type_checker::SourceAlgorithmCheck<'_>, core: &CoreIr, roots: &[CoreFormulaId]) -> Result<BTreeMap<CoreFormulaId, Vec<u8>>, String>` は選択した論理式の未 hash 内容だけを返し、Core 全体の内容や storage とは別とする。既存 source_theorem_formula_payloads の doc comment/関数の直前に置き、旧 bytes/API は保持する。
実 [lower_source_algorithms](#static-source-algorithm-lowering) を最初に呼び、その error String をそのまま伝播する。その後 Core 全体の一致、Valid Algorithm item 一つと対応する algorithm 一つ、item 依存/診断・algorithm 診断/ghost_effects・Core 診断・definition/proof/proof node/generated origin の不在を要求する。追加 profile/選択/normalization/framing の失敗は厳密な内部 `unsupported source algorithm formula payload` とし、部分 map・入力変更・公開診断を作らない。
既存 static/flat/while/[Pick](#限定-return-pick-lowering) profile を保持する。requires/invariants/assertions/decreasing の contract vector は空、実 ensures の順序は保持する。statement は Snapshot/Let/AssignLocal/Assert/空 decreasing の While/Return/Break/Pick のみで、producer の変更や構文走査をしない。実 choice の Active 非空性と flow 所有の deferred termination は保持する。
利用可能な root は実 parameter/result/Let/Pick の存在する guard、順序付き ensures、Assert 式、While condition/invariant、Pick witness guard、obligation context/goal を実 id で重複排除した集合とする。要求 root の重複と集合外 id を拒否し、内部専用 Not child/quantifier body も含む。選択が空でも全利用可能 root を再帰検査し、要求した実 key だけを返す。
実 params、必須 result、実 statement-table 順の Let/Pick binder から重複しない slot を作る。guard の存在と実 role を要求し、parameter/local:var/local:const は Free、result/pick は Generated とする。role/class を保持し、既存 BinderContext に Term 変数として宣言し、実 CoreVarId を checked u32 slot 順序へ対応付ける。論理量化 binder を global free にしない。
既存 [normalize_core_formula/canonical_formula](./binder_normalization.md#alpha-equivalence) をその context と実 root の source で使う。Bound は de Bruijn index、Free/Generated は同じ class の実 slot に結合する。Exists traversal が論理 binder を所有し、汎用 variable identity・normalization・canonical 構造は変更しない。
各値は UTF-8 `mizar-core/source-algorithm-formula/v1` と zero、次いで七つの frame: Core module package/path、item SymbolId module package/path/local/FQN、実 visibility。frame は checked big-endian u32 byte length と bytes。その後 checked big-endian u32 slot 数、各 slot の class byte (1 Free,2 Generated) と framed 実 role、実 canonical tree を置く。
formula tag は 2 Equals と順序付き左右 term、3 TypePred と framed 実 type/subject、4 Not と child、6 Exists と checked big-endian u32 binder 数・各 framed 実 term-binder role/guard 不在 byte0・body。term は tag1 Bound と big-endian u32 index、tag2 Free または tag4 Generated と big-endian u32 実 slot。他の canonical shape/role/guard/variable は拒否する。
全 length/count 変換と加算を buffer 拡張前に検査し、prefix/value と選択 value 長の総和をそれぞれ32MiB 以下とする。失敗は結果全体を拒否する。dense row/variable id、source/表示名、range/map/provenance、source text、Debug/hash は内容に含めず、map key だけが Core 内 id を保持する。
closed VC/context identity、algorithm 全体、header/termination policy、runtime/interface fingerprint、dependency footprint、acceptance は与えない。不足 footprint は [uncacheable](../../architecture/ja/18.dependency_fingerprint.md#依存スライス) のままで、publication/full services/nonempty phase10/Proof/Artifact/cache/Hit/reuse/LSP/Steps6–8 は別とする。
### Source algorithm formula payload tests
既存 mizar-test runner/proof_verification test module の末尾 brace 前に `source_algorithm_formula_payloads_preserve_actual_content` と `source_algorithm_formula_payloads_preserve_refusals` だけを追加し、旧 helper/test を保持する。既存の真正 parse/resolver/original-input/neutral-typed/opaque-check/lower による setup を inline に置き、producer 複製・新依存・正例 receipt/Core 捏造をしない。
未変更の [while](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_while_invariant_001.miz)、[var/const/assert](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_var_const_assert_001.miz)、[ghost/snapshot](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ghost_snapshot_001.miz)、[choice](../../../../tests/miz/fail/algorithms/fail_proof_verification_algorithm_ensures_unprovable_001.miz) を使う。API 前に実 Core/role/entry root、非空 CFG/handoff、deferred termination と choice の Active 非空性を証明 credit なしに確認し、literal expected canonical value を独立 framing して map 全体を比較する。
反復・root 順序変更/部分/空・不変性、Core が変わる同 owner の alpha/trivia での bytes 一致、真正に受理される owner/式/operand 順序変更での対応 bytes 変化を検査する。式を変えない return 式だけの編集では選択内容を保ち、algorithm 全体との混同を防ぐ。量化 choice goal は Bound(0) とし、内部 body root は拒否する。
拒否例は真正 foreign Core/check、重複/不明/内部専用 root、空選択でも行う検証とする。未変更 [claim fixture](../../../../tests/miz/pass/algorithms/pass_proof_verification_claim_block_theorem_001.miz) の真正 empty-interface seal から実 lower error を先に確認し、別の真正 Core を渡して同じ error 伝播を要求する。到達不能な size/width branch は静的 review とし、偽状態・race・accepted/completed control・旧 matrix 複製・期待値/trace/coverage 変更を行わない。

## Source algorithm Core semantic payload
`source_algorithm_core_semantic_payload(check: &mizar_checker::type_checker::SourceAlgorithmCheck<'_>, core: &CoreIr) -> Result<Vec<u8>, String>` は既存の単一 parameter の限定 algorithm Core 全体の意味内容を返し、lossless source storage や実行/interface 全体の identity とは別とする。
実 entry guard/ensures/statement formula/seed context・goal を入力拒否なしに収集・重複排除し、[source_algorithm_formula_payloads](#source-algorithm-formula-payloads) を最初に呼ぶ。その lower/profile error はそのまま伝播する。replay/Core 全体一致と既存 profile の認証後、追加の全体内容検証は厳密な内部 `unsupported source algorithm core semantic payload` で原子的に失敗し、入力変更・公開診断を作らない。
選択操作の全検査を保持し、実 parameter は一つ、実 result と params/result/statement-table Let/Pick 順の一意 runtime slot を要求する。内容から除く field は source 対応か検査済み固定 profile のみ: Valid Algorithm item、symbol/owner 一致、空の依存/診断/除外 table/ghost effect と ensures 以外の contract vector、空の While decreasing。構文/綴り解釈・normalizer 複製・別 payload 表現は作らない。
各 guard/contract/assert/while/witness/goal は返された実 formula value を使う。七つの module/symbol/visibility と全 runtime role/class はその値が保持し、identity prefix は重複させない。実 operand は既存 [normalize_core_term/canonical_term](./binder_normalization.md#alpha-equivalence)、実 BinderContext/term source で正規化し、同じ実 slot/class の Free/Generated 変数だけを認める。論理 Exists binder を global slot にしない。
順序付き algorithm root と While child を preorder 走査する。statement の重複/cycle/別 owner を拒否し、訪問 id が実 table 全体と等しいことを要求する。実 statement id は checked preorder ordinal に対応付ける。下記の全 field、ghost flag、optional、capture 順序、意味上の Snapshot 名を保持し、binder 表示名・source id/range/map/provenance・dense 割当順は source/storage 対応とする。
実 entry root から Equals/TypePred/Not/Exists を辿って全 formula row、その実 term occurrence と program operand を対応付ける。内部量化 body を scope 外で正規化しない。未参照 term は真正 header parameter Var 一つだけで、その実 parameter slot へ canonicalize されることを要求し、他は全て到達済みとする。隠れた table 末尾や構文/hash 代替で内容を補わない。
出力は UTF-8 `mizar-core/source-algorithm-core-semantics/v1` と zero。以下の count/length/index/slot は checked big-endian u32、frame は length と bytes。拡張前に変換/加算を検査し、全体を32MiB 以下とする。次に parameter 数と順序付き guard-value frame、必須 result guard-value frame、ensures 数と順序付き formula-value frame、root statement 数と preorder node を置く。既存選択値はそのまま frame にする。
statement tag/field は1 Snapshot(name frame,capture 数,順序付き slot)、2 Let(slot,guard frame,ghost byte,optional initializer)、3 AssignLocal(target slot,value term)、4 Assert(formula frame)、5 While(condition frame,invariant 数と frame,child 数と child)、6 Return(optional value)、7 Break、8 Pick(slot,guard frame,optional witness formula frame,ghost byte)。optional は byte0 不在/1 と値、bool は0/1、term は tag2 Free/4 Generated と実 slot。他の kind/shape/結合は拒否する。
最後に seed 数0/1。Pick なしなら seed なし、実 Pick 一つならこの item 所有の実 GeneratedNonEmptiness Active seed 一つ、実 Exists goal、空 context/label/diagnostic、生成元の順序を保持した厳密な typed ref [Item, Algorithm, Pick-statement, goal-Formula] を要求する。固定状態はこの version profile に含意する。seed は実 Pick preorder ordinal、framed 選択 goal 値、framed 実 local_path、framed 実 semantic_origin の順とし、他の seed/Pick 対応や状態は拒否する。
実 choice 非空性は Active、別の flow 所有 termination は Deferred のままとする。Core は header terminating/promotion policy を持たず、実行/interface fingerprint・完全 footprint・closed VC/context identity・受理は与えない。不足 footprint は [uncacheable](../../architecture/ja/18.dependency_fingerprint.md#依存スライス) のままで、coherent publication/full service/nonempty phase10/runtime/Proof/Artifact/cache/reuse/Hit/LSP/Steps6–8 は別とする。
### Source algorithm Core semantic payload tests
既存 mizar-test runner/proof_verification test module 末尾 brace 前に `source_algorithm_core_semantic_payload_preserves_actual_program` と `source_algorithm_core_semantic_payload_preserves_refusals` だけを追加する。旧 helper/test は全保持し、既存真正 parse/resolver/original-input/neutral-typed/opaque-check/lower を inline setup で使い、正例 Core/check/status 捏造・新依存は行わない。
未変更 [while](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_while_invariant_001.miz)、[var/const/assert](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_var_const_assert_001.miz)、[ghost/snapshot](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ghost_snapshot_001.miz)、[choice](../../../../tests/miz/fail/algorithms/fail_proof_verification_algorithm_ensures_unprovable_001.miz) を使う。API 前に実 signature/slot/guard/program と全 row/seed 対応、非空 CFG/handoff、実 choice Active/flow Deferred を確認する。旧 formula/term の literal 値と program/signature/seed 全出力を独立 framing し、scoped Bound(0) を含め、新 writer 自体を oracle にしない。
反復/不変性、同 owner alpha/trivia で Core が変わっても内容が等しいことを検査する。真正 owner/formula 変更で内容が変わり、真正 return operand/assignment operand/statement 順序/Snapshot 名/ghost flag control は実際の意味差を保持する。特に return のみ変更なら選択 formula 値は等しく全体内容は変わる。旧 root 選択 matrix は複製しない。
拒否は真正 foreign Core/check と未変更 [claim fixture](../../../../tests/miz/pass/algorithms/pass_proof_verification_claim_block_theorem_001.miz) を使い、実 lower error を先に確認して別の真正 Core でも同じ伝播を要求する。到達不能 size/u32/profile branch は静的 review とし、偽 receipt/race/Accepted/Complete control や既存期待値/trace/coverage 変更は行わない。
