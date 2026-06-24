# Module: generator

> 正本は英語です。英語版:
> [../en/generator.md](../en/generator.md)。

## 目的

この module は `mizar-vc` の phase-11 VC generation を仕様化する。

generator は task 4 の `SeedIntakeTable`、immutable な `CoreIr`、immutable な
`ControlFlowIr`、source/core provenance、proof hint、verifier policy input を
消費する。後続 task 8 が normalize、classify、seed へ map back し、deterministic
`VcId` を割り当てる generation candidate を生成する。source syntax の parse、missing
checker payload の復元、VC discharge、ATP backend 呼び出し、proof validation、
artifact publication、registration acceptance は行わない。

Task 5 は仕様のみである。theorem/definition と registration-style correctness VC の
Rust source は task 6、algorithm VC generation は task 7、normalization、classification、
`VcId` assignment は task 8 が担当する。

## この仕様の gap 分類

| ID | 分類 | 証拠 | 扱い |
|---|---|---|---|
| GEN-G001 | `spec_gap` | task 5 より前に `generator.md` は存在せず、tasks 6-8 には generation contract が必要である。 | この task は Rust source なしで英語/日本語 generator spec を作成する。 |
| GEN-G002 | `external_dependency_gap` | `mizar-core` は現在、registration、redefinition、reduction correctness をすべての registration-style condition 向けの dedicated payload ではなく、利用可能な definition/checker seed と provenance を通じて運ぶ。 | 既に存在する explicit core/checker seed と provenance からのみ registration-style correctness VC を生成する。dedicated payload が欠けている場合は fabricated obligation ではなく `DeferredExternal` または no-VC record とする。 |
| GEN-G003 | `test_gap` / `source_drift` | tasks 6-8 より前に `src/generator.rs` や generator tests は存在しない。 | この仕様は implementation task の behavior と test obligation を名付ける。task 5 は Rust source を変更しない。 |
| GEN-G004 | `external_dependency_gap` | active source-derived `proof_verification` `.miz` runner support と extraction seam はまだ利用不能である。 | tasks 6-8 では explicit core/control-flow payload 上の Rust fixture を使い、source-derived corpus activation は task 15 まで deferred に保つ。 |
| GEN-G005 | `external_dependency_gap` | `ObligationSeed` は first-class theorem status dependency metadata や dedicated registration/redefinition/reduction correctness payload field をまだ露出していない。 | Task 6 は upstream fixture が供給する namespaced explicit `CoreProvenance` marker だけを保持する。marker がない場合は seed kind / status に従って通常 candidate または visible no-candidate record になり、label、generic path、source text からこれらの semantics を推測してはならない。 |
| GEN-G006 | `external_dependency_gap` | 現在の `ObligationSeedHandoff` は contract、assertion、invariant obligations 用の flow-site metadata と goal formula を持つが、call-precondition、branch、match、range-loop、collection-loop、term-only termination、ghost-erasure obligations 用の generated formula schema はまだ公開していない。 | Task 7 は `ControlFlowObligationSite` metadata と goal formula を持つ explicit flow-derived seed row だけから candidate を生成する。欠けている algorithm payload family は fabricated VC ではなく visible no-candidate/deferred record に保つ。 |
| GEN-G007 | `source_drift` / `test_gap` | task 7 後の Rust source には task-6/task-7 の pre-normalized candidate はあるが、final `VcSet` normalizer、documented `VcKind` ordering rank、dense `VcId` assignment、normalizer tests はない。 | Task 8 は normalizer を実装し、下記の stable classification order を文書化し、dense id、seed accounting、duplicate rejection、deferred status preservation、expanded-mapping validation、stable rendering input の Rust test を追加する。 |
| GEN-G008 | `deferred` | Status transition、deterministic discharge、dependency slice、ATP translation、kernel/proof/cache/corpus integration、source-derived corpus runner activation は後続 module spec または未提供 external seam に依存する。 | Task 8 はそれらを out of scope として記録し、discharge、transition、slice、translation、publish、external integration record の fabricate を行わず existing status を保持しなければならない。 |

Task 5 では `doc/spec`、`.miz` fixture、expectation、traceability metadata を変更しない。
この文書は既存 architecture/spec requirement を精緻化するものであり、新しい language
semantics は導入しない。

## 入力と出力

必須入力:

- すべての handoff row が 1 回 intake-accounted された validated `SeedIntakeTable`;
- immutable `CoreIr` の proof skeleton、theorem proposition、definition correctness
  payload、proof hint、formula、source map、provenance;
- immutable `ControlFlowIr` の algorithm context、structured exit、contract site、
  loop metadata、ghost/runtime fact、termination metadata;
- upstream payload に含まれる場合の explicit registration、redefinition、reduction、
  checker、cluster、reduction-trace reference;
- status、premise restriction、local unfolding、computation request、open/assumed
  handling に影響する verifier policy input。

Generator output は normalize 前の candidate set である:

- stable seed reference、owner reference、generated formula reference、local context、
  symbolic premise ref、proof hint、source/core provenance、anchor ingredient を持つ
  candidate VC record;
- split、instantiated、schema-created formula 用の VC-local generated formula table;
- zero、one、expanded candidate cardinality を明示する seed-to-candidate bookkeeping;
- task 8 が消費する deterministic sort key と classification ingredient。

candidate set は final `VcIr` ではない。concrete `VcId`、final canonical ordering、
final `SeedAccountingTable`、discharge evidence、dependency slice、ATP/backend text を
外へ出してはならない。

## 全体 generation 規則

generator がしなければならないこと:

- explicit な `CoreIr`、`ControlFlowIr`、seed、provenance、policy payload だけを消費する;
- task 4 の handoff order を input evidence として保持しつつ、task 8 用に stable canonical
  sort key を提供する;
- required formula、source、owner、context、provenance data が欠ける場合は fail closed する;
- skipped、invalid、deferred、policy-open、missing-payload case を visible
  candidate/status/accounting record として表す;
- `NeedsAtp` 行き obligation は full local context と proof hint を持つ concrete candidate
  として保つ;
- generated formula を provenance と generation schema version 付きで VC-local table に置く;
- `CoreIr`、`ControlFlowIr`、seed handoff、upstream source map を mutate しない;
- hash-map iteration、worker completion order、backend availability、local absolute path、
  timing に依存しない。

generator がしてはならないこと:

- core/control-flow payload がない theorem、definition、registration、reduction、
  algorithm obligation を raw syntax から推測する;
- registration activation を proof acceptance に変える;
- unavailable registration、redefinition、reduction correctness condition を downstream
  resolution の fallback として利用可能にする;
- local context、disabled seed、policy-open obligation、deferred external gap を silent drop する;
- ATP encoding、backend process option、proof certificate、cache hit、kernel acceptance、
  artifact schema を選ぶ。

## Local Context Construction

すべての candidate VC は self-contained local context を持つ。ATP translation は source syntax
を読んだり semantic context を再構築したりせずに candidate を encode できなければならない。

obligation site で明示的に利用可能な場合に必要な context ingredient:

- binder declaration、normalized binder role、type predicate、sethood fact、
  non-emptiness fact;
- proof assumption、current thesis fact、local label、diffuse-proof fact、
  contradiction assumption、scoped witness;
- `by` justification、grouped citation、bulk citation、local proof label 由来の symbolic
  citation;
- checker fact、generated fact、`qua` evidence、registration trace、cluster trace、
  reduction trace;
- `ControlFlowIr` 由来の algorithm path condition、branch fact、loop invariant、
  hidden immutable loop metadata、post-havoc fact、structured exit context;
- definition opacity input、permitted local unfolding request、reduction policy input、
  `by computation` request;
- status、dispatch、premise limit、computation limit に影響する verifier policy input。

context entry は source/core provenance と canonical context sort key を持つ。key は insertion
order に依存せず、task 8 が local-context fingerprint を計算できる程度に stable でなければならない。
unknown または incomplete context は empty context として扱わない。candidate を conservative、
deferred、または diagnostic 付き error にしなければならない。

## Step 3: Theorem And Definition VCs

Theorem、lemma、proof-step、terminal-proof candidate は `CoreIr` 内の explicit proof skeleton と
terminal goal から生成する。

Generation rules:

- core proof context から theorem binder と type predicate を instantiate する;
- terminal proof obligation の goal として current `thesis` を保持する;
- proof assumption と scoped witness は owning block 内だけに保持する;
- `by` justification 由来の cited premise を symbolic `PremiseRef` として保持する;
- clean、non-clean、open、assumed、conditional theorem check 用に theorem status dependency を保持する;
- proof hint、local unfold request、computation request は backend dispatch decision ではなく
  symbolic hint として記録する。

Definition correctness candidate は explicit core definition-correctness payload からのみ生成する。
supported correctness family は existence、uniqueness、coherence、compatibility、consistency、
reducibility と、upstream seed がその condition を名付ける場合の definition-specific
sethood / non-emptiness obligation を含む。generated formula は definition owner、
correctness kind、source/core provenance、seed reference、generation schema を持たなければならない。

ordinary theorem、proof-step、definition-correctness、checker-initial の eligible seed は、この
仕様または後続 owning spec が explicit expansion schema を定義しない限り 1 candidate を生成する。
conjunction split、schema instantiation、generated helper formula は VC-local generated formula
table と task-8 seed mapping に表現しなければならない。

## Generated Core Obligation VCs

Generated core obligation candidate は explicit core seed kind からのみ生成する。必須 family:

- generated non-emptiness obligation;
- generated sethood obligation;
- generated Fraenkel membership axiom。

これらの candidate は、seed が core formula を名付けている場合はそれを使い、generator が
explicit core payload から schema を instantiate する必要がある場合は VC-local generated
formula を使う。candidate は seed kind、owner、source/core provenance、local proof path、
semantic origin、generated-formula schema を保持しなければならない。generated-obligation
payload が欠けている場合は `DeferredExternal` または visible no-VC mapping とする。generator は
それらを source syntax から再構築してはならない。

各 generated core obligation seed は、後続 owning spec が explicit expansion schema を定義しない
限り 1 candidate を生成する。Task 8 は concrete seed-to-VC mapping を記録し、duplicate ownership
を deterministic に拒否する。

## Registration-Style Correctness VCs

Registration-style correctness は explicit core/checker payload が存在する場合に限り、
registration、redefinition、reduction condition を扱う。generator は registration activation、
inferred attribute、raw source syntax から欠けている correctness obligation を合成してはならない。

explicit payload が存在する場合、candidate kind と provenance は次を区別しなければならない:

- existential cluster existence;
- conditional / functorial cluster coherence;
- core/checker seed が運ぶ redefinition compatibility または coherence;
- reduction reducibility と、upstream data が露出している場合の strict simplification-order
  side condition;
- registration-style owner を provenance が識別するが正確な将来 seed kind がまだ安定していない
  checker-initial carried obligation。

registration-style candidate の規則:

- registration label/FQN、owner item、activation boundary、correctness condition は provenance
  であり、proof acceptance ではない;
- pending または failed correctness は downstream resolution に unavailable のままでなければならない;
- unavailable dedicated registration/redefinition/reduction payload は concrete reason 付きの
  `DeferredExternal` または visible no-VC mapping として記録する;
- reduction trace は upstream payload が applied rule を明示的に記録する場合に限り context または
  premise evidence として参照できる。

## Step 4: Algorithm VCs

Algorithm candidate は `ControlFlowIr` とそこから準備された algorithm seed row から生成する。
generator は language spec の structured Hoare-style schema に従い、source text から control flow
を再構築してはならない。

完全な algorithm VC model は次の candidate family を含む。

- 各 algorithm call の call precondition;
- すべての return edge と algorithm contract exit の postcondition;
- assertion obligation;
- branch / match obligation。case context と、存在する場合の exhaustiveness evidence を含む;
- while-loop invariant entry、preservation、break、continue、exit、decreasing-measure obligation;
- `to` と `downto` の両方に対する range-loop positive-step、bound、hidden-index、
  invariant-entry、invariant-preservation obligation;
- collection-loop finiteness、processed-set、invariant-establishment、
  invariant-maintenance、order-independence obligation;
- call graph component と decreasing measure ごとに grouping された recursive /
  mutually recursive termination obligation;
- caller-side postcondition が termination evidence を要求する場合の partial-termination obligation;
- ghost-only Pick site を含むすべての set-based / type-based Pick site 向けの Pick
  non-emptiness obligation;
- ghost-erasure safety obligation と ghost-only `Pick` erasure record。

Algorithm context が保持しなければならないもの:

- assignment の pre-state から得られる old-state assignment fact;
- resolved lvalue path 由来の field-update alias identity;
- may-write location を forget / freshen しつつ immutable hidden loop value を保持する post-havoc loop context;
- loop の normal-exit `not C` fact を加えない break exit;
- measure が存在する場合に invariant と decreasing check を持つ continue exit;
- range-loop hidden value `a0`、`b0`、`s0`、`S0`、hidden exit index;
- explicit non-emptiness / type-inhabitation obligation を持つ fresh site-local logical
  value としての Pick binding;
- logical verification context 内だけの ghost fact。runtime dependency にはしない。

Task 7 は、現在の flow-derived handoff row が explicit `ControlFlowObligationSite` metadata と
goal formula を持つ subset、つまり requires、ensures、assertions、supported
loop-invariant phases だけの concrete generator implementation を所有する。残りの listed
family は、upstream payload と generated formula schema が明示的に公開されるまで visible
no-candidate/deferred record に保つ。

## Controlled Definition Unfolding

generator は `CoreIr` と proof hint からの explicit policy を通じてのみ definition unfold または
simplify してよい。

許可される入力:

- definition opacity / transparency metadata;
- proof hint 由来の local unfold request;
- resolution trace または checker payload に明示された reduction registration;
- discharge/computation policy が limit を固定するまで symbolic に保たれる computation request。

Unfolding output は source/core provenance 付きの generated formula または premise ref として表す。
explicit policy が unfolding を許さない場合、generator は opaque boundary を保持しなければならない。
traversal order、definition body の global availability、ATP convenience を使って goal を強めてはならない。

## Step 5: Normalization And Classification Handoff

Task 8 が final canonicalization、classification、`VcId` assignment を所有する。generator は
その task に必要な入力を供給する:

- module、owner、source/core provenance、seed canonical key、generation schema、expansion index
  に基づく stable candidate sort key;
- upstream API が提供する場合の normalized binder と local-context ingredient;
- explicit `VcKind` classification ingredient と priority hint;
- stable reference で sort された symbolic premise、または sorting input が incomplete な場合の
  conservative marker;
- generated formula provenance と schema version;
- candidate なし、1 candidate、dense zero-based expansion index を持つ expanded candidate という
  seed-to-candidate cardinality;
- required anchor ingredient が unavailable な場合の incomplete-anchor marker。

Task 8 はこれらの candidate を canonical `VcIr` に変換し、dense within-snapshot `VcId` を割り当て、
final `SeedAccountingTable` を作成し、duplicate を deterministic に拒否し、fingerprint input を
準備する。candidate key、source range、将来の `VcId` の一致は編集横断 proof reuse ではない。

## Task 6 Implementation Slice

Task 6 は最初の source module `src/generator.rs` を、explicit core seed family だけに対して
実装する。public surface は validated `SeedIntakeTable` と対応する `ObligationSeedHandoff`
から構築される normalize 前の `CoreGenerationCandidateSet` である。

Task 6 candidate が保持するもの:

- handoff id、seed origin、seed status、stable candidate sort key、schema version;
- 選択された `VcKind`、source reference、owner、local proof path、label、semantic origin、
  local context、symbolic premise、proof hint、goal formula、open status、provenance、
  incomplete anchor;
- skipped、deferred、error、missing-goal、後続 generator task が所有する seed kind 向けの
  no-candidate record。

candidate を生成する前に、Task 6 は同じ `ObligationSeedHandoff` から seed-intake table を
再計算し、供給された `SeedIntakeTable` と完全に一致しない場合は要求を拒否する。これにより
stale、partial、reordered、その他 mismatch のある intake が obligation を silent omission
したり復活させたりすることを防ぐ。

Task 6 が対応する active seed kind:

- `TheoremProof`;
- `DefinitionCorrectness`;
- `CheckerInitial`;
- `GeneratedNonEmptiness`;
- `GeneratedSethood`;
- `FraenkelMembershipAxiom`。

`AlgorithmContract`、`AlgorithmTermination`、`GhostErasure` の active seed は、task 7 まで
visible `DeferredExternal` no-candidate record として表す。Task 6 は final `VcId` の割り当て、
final `VcSet` の構築、algorithm VC generation、`Open` / visible no-candidate record を超える
status transition、dependency slice 計算、obligation discharge、ATP 呼び出し、corpus fixture
activation を行わない。

Task 6 の stable candidate sort key は module、generation schema、owner、seed canonical key、
source、core provenance、dense expansion index、handoff id、candidate kind から構築する。
Context entry は dense context id を割り当てる前に core formula id で sort する。元の context
insertion order は duplicate formula reference の tie-breaker としてのみ使う。

Registration-style correctness は `vc-registration-style:registration`、
`vc-registration-style:redefinition`、`vc-registration-style:reduction`、
`vc-registration-style:explicit-core-seed` のような namespaced explicit `CoreProvenance`
marker だけから検出する。label、generic local path、semantic-origin text だけでは seed を
registration-style correctness と分類しない。それ以外の `DefinitionCorrectness` と
`CheckerInitial` は通常の core/checker candidate または visible no-candidate record のままにする。

`vc-theorem-status:clean`、`vc-theorem-status:non-clean`、`vc-theorem-status:open`、
`vc-theorem-status:assumed`、`vc-theorem-status:conditional` のような explicit theorem status
dependency marker は、candidate local context の `VerifierPolicyInput` entry として保持する。
theorem-status payload がない場合、label、path、semantic origin、source text から捏造しない。

Task 6 は upstream provenance が explicit `vc-proof-goal:terminal` marker を含む場合に限り
terminal proof goal candidate を出す。それ以外の `TheoremProof` seed は proof-step candidate
のままである。Task 6 は explicit `vc-unfold:*` provenance marker が local unfolding を許す場合に
限り、local definition unfold request を作成する。

## Task 7 Implementation Slice

Task 7 は explicit flow-derived handoff row から algorithm candidate を生成するよう
`src/generator.rs` を拡張する。入力は immutable `ControlFlowOutput` を含めてもよく、
generator は flow-derived seed の `ControlFlowObligationSite` が参照先 `ControlFlowIr`
に属することを検証し、明示 CFG metadata から loop-invariant phase を分類する。raw source
text、label、generic semantic-origin string は algorithm payload ではない。

Task 7 は task-7 seed-intake rule が row を eligible としたうえで、次の条件がすべて
成り立つ場合に限り、deferred flow-derived seed から open candidate を生成してよい。

- row origin が `FlowDerived` である。
- entry が `ControlFlowObligationSite` を持つ。
- 参照先 `ControlFlowIr` が渡された `ControlFlowOutput` に存在する。
- seed kind と site kind が task-7 owned algorithm family を形成する。
- seed が explicit goal formula を持つ。

Seed intake は元の deferred seed status を bookkeeping に保持しつつ、goal-bearing flow row に
eligible one-candidate mapping を使う。生成 candidate には `VcStatus::Open` を使う。Task 8 は
final seed-to-VC mapping を所有し、この status transition を auditable にしなければならない。

Task 7 は explicit goal を持つ次の site mapping を support する。

- `Requires` -> `AlgorithmPrecondition`
- `Ensures` -> `AlgorithmPostcondition`
- `AlgorithmAssertion` と `StatementAssertion` -> `AlgorithmAssertion`
- `AlgorithmInvariant` -> `LoopInvariant { phase: Entry }`
- `LoopInvariant` -> 渡された `ControlFlowIr` が phase を区別するのに必要な loop header
  または exit kind を公開している場合、`Entry`、`Preservation`、`Break`、`Continue` の
  いずれかの `LoopInvariant`

Task 7 は次の case を visible no-candidate record として記録する。

- `ControlFlowObligationSite` を持たない flow-derived algorithm row。
- 参照先 `ControlFlowIr` が入力に存在しない row。
- explicit goal formula を持たない algorithm row。現在の `TerminationMeasure`、
  `PartialTermination`、`GhostPick`、`GhostAssignment` row を含む。
- spec が名前を挙げているが explicit handoff payload としてまだ存在しない
  call-precondition、branch、match、range-loop、collection-loop、Pick non-emptiness、
  ghost-erasure proof family。

Task 7 の algorithm context entry は symbolic かつ conservative である。site kind、ordinal、
statement、block、loop id、exit id、local id、assignment-effect id、対応する flow id などの
explicit site metadata は `VerifierPolicyInput` record または metadata-only local-context entry
として記録してよい。ただし、それらの formula が handoff または `ControlFlowIr` に存在しない
場合、path condition、old-state assignment fact、alias identity、post-havoc fact、hidden range
value、branch fact、ghost runtime fact を発明してはならない。

Task 7 は Rust fixture が `ControlFlowIr` に必要な `SymbolId` を構築できるよう、test-only
`mizar-resolve` dev-dependency を追加してよい。production `mizar-vc` code は
`mizar-core` と `mizar-session` input に限定される。

## Task 8 Implementation Slice

Task 8 は task-6/task-7 の pre-normalized candidate set を final `VcSet` data に
normalize する。`VcId` は current build snapshot 内でのみ dense に割り当て、concrete VC は
stable classification rank、candidate sort key、handoff id の順で ordering し、final
seed accounting row は handoff id 順に構築する。

Task-8 の `VcKind` classification rank:

1. `TheoremProofStep`
2. `TerminalProofGoal`
3. `DefinitionCorrectness`
4. `RegistrationStyleCorrectness`: `Registration`, `Redefinition`, `Reduction`,
   `ExplicitCoreSeed` の順
5. `CheckerInitial`
6. `GeneratedNonEmptiness`
7. `GeneratedSethood`
8. `FraenkelMembershipAxiom`
9. `AlgorithmPrecondition`
10. `AlgorithmPostcondition`
11. `CallPrecondition`
12. `AlgorithmAssertion`
13. `LoopInvariant`: `Entry`, `Preservation`, `Break`, `Continue`, `Exit` の順
14. `RangeLoop`: `PositiveStep`, `RangeBound`, `HiddenIndex` の順
15. `CollectionLoop`: `Finiteness`, `OrderIndependence` の順
16. `Termination`
17. `PartialTermination`
18. `GhostErasureSafety`
19. `PolicyDeferredTraceability`

将来の `VcKind` variant は、generator が emit する前に owning spec task によって末尾に追加する。
Task 8 は既存 candidate sort key を stable tie-breaker と duplicate-detection key として扱い、
kind classification semantics の唯一の供給元にはしない。

Task 8 が support するもの:

- concrete task-6/task-7 candidate ごとの `One` mapping。
- visible no-candidate record ごとの `NoConcreteVc` mapping。
- 既存 `VcSet` の `Expanded` mapping shape の validation。ただし explicit expansion schema
  が導入されるまでは、task-8 generator family は expanded candidate を作らない。

Normalization は duplicate candidate sort key と duplicate seed ownership を決定的に拒否する。
source reference、local context、symbolic premise、proof hint、status、provenance、
incomplete anchor、intake/generation が記録した元の `seed_status` を保持しなければならない。
normalization provenance は追加してよいが、VC discharge、policy status transition、
dependency slice 計算、ATP 呼び出し、corpus fixture activation、新しい algorithm payload family
の追加は行ってはならない。

## Task 20 reuse-identity wiring

Task 20 は milestone 内の generated-obligation reuse identity wiring を完成させる。
concrete generated candidate ごとに、generator は stable な source-shape、canonical-goal、
canonical-context hash marker を持つ `ObligationAnchor` を構築しなければならない。
generator-owned source-shape hash は source-shaped provenance が利用可能な場合に available
になり、stable な source-shaped ingredient を使い、`VcId`、source range、`SourceId`、handoff id、
candidate sort key、dense owner row id を除外する。Canonical goal/context hash marker は、
参照される formula と context entry の stable payload を `mizar-vc` が持つ場合だけ available
になる。現在の CoreFormulaId-only generator payload は raw upstream row id を hash せず、
incomplete / conservative-unknown のままにする。

Task 20 は cache reuse をそれ自体では許可しない。後続 consumer のために、complete
`ObligationAnchor`、canonical VC fingerprint、local-context fingerprint、dependency-slice
fingerprint、compatible verifier-policy fingerprint、選択された proof-evidence hash という
stable candidate ingredient を供給するだけである。`VcId`、source range、anchor の一致だけでは
引き続き不十分である。

## Planned Tests

Task 6 が追加すべき Rust coverage:

- explicit local context と symbolic citation を持つ theorem/proof-step terminal goal;
- clean、non-clean、open、assumed、conditional theorem status dependency の保持;
- 利用可能な existence、uniqueness、coherence、compatibility、consistency、reducibility payload
  と sethood / non-emptiness payload 向けの definition correctness candidate;
- generated non-emptiness、generated sethood、Fraenkel membership axiom seed 向けの
  generated core obligation candidate;
- explicit checker/core payload が利用可能な場合の registration-style correctness candidate;
- unavailable registration-style payload が fabricated candidate ではなく deferred/no-VC として記録されること;
- proof hint と local unfold request が symbolic に保持されること。

Task 7 が追加すべき Rust coverage:

- explicit flow-derived site からの goal-bearing algorithm precondition、postcondition、
  assertion、loop-invariant candidate;
- unavailable call-precondition、branch、match、range-loop、collection-loop、term-only
  termination、partial-termination、Pick non-emptiness、ghost-erasure payload family の
  visible no-candidate/deferred record;
- explicit flow-site metadata の conservative symbolic context preservation。loop
  header/backedge と break/continue exit classification を含むが、payload に存在しない
  old-state assignment fact、field-update alias identity、post-havoc fact、range-loop hidden
  metadata、branch fact、Pick fact は発明しないこと;
- traversal helper の map iteration に依存しない algorithm candidate の deterministic ordering。

Task 8 が追加すべき Rust coverage:

- deterministic candidate normalization と dense `VcId` assignment;
- no-VC と one-VC mapping を含む complete seed-to-VC accounting、および既存
  expanded-mapping contract の validation coverage;
- duplicate candidate または seed ownership rejection;
- local context、generated formula、incomplete anchor の stable rendering/fingerprinting input。

Task 5 では active `.miz` proof-verification fixture を追加しない。runner support と
source-derived extraction seam が external gap のままであるため。

## public enum policy

task 17 は `generator` の public enum をすべて downstream forward-compatible API surface
として分類する。後続の candidate、normalization、handoff validation error を downstream の
exhaustive match を壊さず追加できるよう、各 enum は `#[non_exhaustive]` を維持しなければならない。

| public enum | decision |
|---|---|
| `GeneratorError` | `#[non_exhaustive]` downstream forward-compatible surface。 |

この module が所有する exhaustive public enum exception はない。現在の variant を意図的に
列挙する `mizar-vc` 内部 match は exhaustive のままでよい。
