# Module: generator

unary descriptor は contract-free の範囲を保ち、保存された caller ensures があると後続の証明義務・既存 identity の生成前に拒否する。

限定 theorem bridge は既存 Core seed handoff・seed intake・candidate 正規化を使う。
lemma 引用の実際の終端証明義務と symbolic premise を保持し、VC を kernel 受理にしない。

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
| GEN-G004 | historical `external_dependency_gap`; exact exception resolved | Task 31 より前は active source-derived `proof_verification` `.miz` runner support と extraction seam が利用不能だった。 | tasks 6-8 は explicit core/control-flow payload 上の Rust fixture を使った。Task 31 は exact Task-180 source-derived exception を追加し、broader corpus activation は task 15 と source-VC decomposition のもとで deferred に保つ。 |
| GEN-G005 | `external_dependency_gap` | `ObligationSeed` は first-class theorem status dependency metadata や dedicated registration/redefinition/reduction correctness payload field をまだ露出していない。 | Task 6 は upstream fixture が供給する namespaced explicit `CoreProvenance` marker だけを保持する。marker がない場合は seed kind / status に従って通常 candidate または visible no-candidate record になり、label、generic path、source text からこれらの semantics を推測してはならない。 |
| GEN-G006 | `external_dependency_gap` | 現在の `ObligationSeedHandoff` は contract、assertion、invariant obligations 用の flow-site metadata と goal formula を持つが、call-precondition、branch、match、range-loop、collection-loop、term-derived termination schema、authenticated partial-call termination evidence、complete ghost-isolation metadata はまだ公開していない。 | Task 7 は `ControlFlowObligationSite` metadata と goal formula を持つ explicit flow-derived seed row だけから candidate を生成する。欠けている proof family/zero-VC/static integration payload は fabricated VC ではなく visible no-candidate/deferred record に保つ。 |
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
- explicit upstream handoff が運ぶ generated sethood obligation。これは existing
  task-6/task-8 payload/kind の compatibility であり、Task-30 source-derived generator
  の authority ではない;
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
- reduction `reducibility` equality のみ。simplification-order、size、variable-occurrence
  rule は invalid registration を reject する structural registration-time check で、proof VC
  にはならない;
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
- 順序付き branch/match context、`otherwise` と partial の暗黙的 fail。
  match-exhaustiveness VC と旧 `exhaustive` 構文は作らない;
- while-loop invariant entry、preservation、break、continue、decreasing-measure obligation。
  normal exit は successor context で、exit VC ではない;
- `to` と `downto` の両方に対する range-loop positive-step、invariant-entry、
  invariant-preservation obligation。bound/hidden index は context で standalone VC ではない;
- collection-loop finiteness、processed-set、invariant-establishment、
  invariant-maintenance、order-independence obligation;
- call graph component と decreasing measure ごとに grouping された recursive /
  mutually recursive termination obligation;
- partial body-call の継続には fresh `returns` と postcondition fact を保持する。
  terminating caller は正常返却証拠を別に要求する (§20.13.1);
- ghost-only Pick site を含むすべての set-based / type-based Pick site 向けの Pick
  non-emptiness obligation;
- ghost-isolation static check と ghost-only `Pick` erasure record。isolation は
  `GhostErasureSafety` proof VC を作らない。

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
- explicit handoff payload としてまだ存在しない call-precondition、branch、match、
  range-loop、collection-loop、Pick non-emptiness proof family。
- historical ghost-erasure row は spec-backed proof family ではなく、zero-VC/static-check
  integration data とする。

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
17. `PartialTermination`（existing validation/sort shape のみ。current canonical spec から
    source-generate しない）
18. `GhostErasureSafety`（existing validation/sort shape のみ。ghost isolation は static
    check で source-generated VC ではない）
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

## Task 30 source-derived integration contract

exhaustive contract は
[source_vc_decomposition.md](./source_vc_decomposition.md) を正本とする。VC Task 31 は
narrow Core-aware adapter で exact Task-180 direct terminal proof-node relation を検証してから
candidate を分類する。generic generator は現在 seed handoff しか見ず、real marker-free seed を
`TheoremProofStep` に分類するため、Task 31 は provenance を挿入・変更せず、認証済み構造を
`TerminalProofGoal` に分類しなければならない。

Tasks 32-55 は broader source-derived family をすべて所有する。特に declared entry
`requires` は callee-body context であり source-generated `AlgorithmPrecondition` proof VC
ではない。loop normal exit と range hidden/bound value は explicit canonical formula がない限り
context である。simplification-order/ghost isolation は static check で、source
`PartialTermination` / `GhostErasureSafety` VC は存在しない。
Task 30 は existing `GeneratedSethood` compatibility kind に source-derived family を
割り当てない。explicit mode-declaration sethood は `DefinitionCorrectness` を使い、
Fraenkel sethood は VC ではなく accepted prerequisite evidence である。
`GeneratedSethood` の source generation には将来の canonical authority と最初の real
source が必要である。
generated formula、substitution、binder、path、trace、old-state 表現は最初の real owning family
と共にだけ拡張し、empty infrastructure task を置かない。この canonical rule と異なる current
source behavior は relevant descendant に割り当てた `source_drift` であり、保持する authority
ではない。

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
- unavailable call-precondition、branch、match、range-loop、collection-loop、term-derived
  termination、Pick non-emptiness と、Task 30 が source-generated VC ではなく
  evidence-admission/zero-VC/static boundary と分類した historical partial/ghost row の
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

## VC Task 31 exact Task-180 adapter

Task 31 は private leaf `generator/task180.rs` に実装された public borrowed adapter
`generate_exact_task180_vc(ExactTask180VcInput)` を1つ追加する。input は immutable
`CoreIr`、target `BuildSnapshotId`、explicit generation/VC schema version を運ぶ。
adapter は Core package/module から length-framed UTF-8 form
`package=<byte-len>:<package>;module=<byte-len>:<module>` で `VcModuleRef` を導出し、
source path や debug text から module identity を推測しない。

adapter は exact Core Task-31 public/valid theorem、`False` formula、pending proof、
direct terminal node、active theorem seed、source map、reference、range、provenance を
atomic に検証する。empty control-flow output、singleton `ExistingCore` handoff、fresh に
再計算した `EligibleOneVc` intake を内部で構築・検証し、その後だけ existing candidate
builder/normalizer で1つの open `TerminalProofGoal` を返す。failure は
`ExactTask180VcError` であり、partial candidate set を公開しない。

これは generic marker-based terminal classification の `design_drift` を Task 180
だけで閉じる exact structural exception である。terminal marker は注入しない。
discharge、`NeedsAtp` transition、ATP/kernel/proof-policy execution、theorem acceptance、
fact publication、general theorem/definition/algorithm generation は行わない。

## public enum policy

task 17 は `generator` の public enum をすべて downstream forward-compatible API surface
として分類する。後続の candidate、normalization、handoff validation error を downstream の
exhaustive match を壊さず追加できるよう、各 enum は `#[non_exhaustive]` を維持しなければならない。

| public enum | decision |
|---|---|
| `ExactTask180VcError` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `GeneratorError` | `#[non_exhaustive]` downstream forward-compatible surface。 |

この module が所有する exhaustive public enum exception はない。現在の variant を意図的に
列挙する `mizar-vc` 内部 match は exhaustive のままでよい。

## Bounded source return postconditions

`generate_source_algorithm_postconditions(&CoreIr, BuildSnapshotId, &GenerationSchemaVersion, &VcSchemaVersion) -> Result<VcSet, String>` は object 仮引数一つの平坦な profile を検証し、CFG/handoff と既存 intake/normalization を導出する。不変の単一 return は既存 Core 項等式を保持する。初期化 var/const・型付き局所書込み・等式 assert は認証済み program-value 世代を使用し、書込み前に RHS を評価し、古い事実と独立記憶領域を保持して新しい等式・型ガードを追加する。prefix 否定一つの等式 assertion は既存 Not(Generated(ProgramEquals)) を使い、実変数・書込み世代と両ソース式の範囲を保持し、全行を計上する。ensures は等式のみとする。assertion 目標は直前の文脈を使用し、後続には同じ目標を assertion handoff への未証明依存としてのみ渡す。return は現在値で result を置換する。未加工の正規化文脈は非公開とし、ソース・所有者・map・由来・書込み順序・依存リンクを原子的に検証する。assert と ensures は open VC、終了メタデータは元の status の NoConcreteVc とし、obligation がなければ完全なゼロ VC を返す。未対応制御、ghost pick・未対応 ghost flow、不正・診断付き入力を拒否する。anchor と未解決依存は再利用不可に留め、discharge・承認・terminating 昇格を行わない。

## Bounded void-claim generation

`generate_source_void_claim(&CoreIr, BuildSnapshotId, &GenerationSchemaVersion, &VcSchemaVersion) -> Result<VcSet, String>` は先行する仮引数なし void algorithm の bare return、claim を source とする theorem seed、型付き algorithm 依存、実際の全称命題と proof-local 汎化・目標を認証する。空 interface は実行変数 binder や状態前提を追加しない。新鮮な CFG/handoff と既存 intake/normalization により open theorem-proof VC 一つと正直な終了メタデータ accounting を生成する。VC は ExistingCore seed 依存と theorem・terminal・algorithm の関連 source を保持し、所有者不整合、wrapper・依存の欠落、空でない実行 interface、不完全な source/map を拒否する。実行・定理受理・再利用適格性・Task274 証拠を推測せず、状態・snapshot claim は保留する。

## Source existential-registration proof

`generate_source_existential_registration(&CoreIr, BuildSnapshotId, &GenerationSchemaVersion,
&VcSchemaVersion) -> Result<VcSet, String>` は全 [Core
profile](../../mizar-core/ja/elaborator.md#source-existential-registration-proof)、正規定義ラベルと実定義・逆参照・ガード・本体・source の対応を検証する。ラベルだけを前提・証拠とせず、kernel の定義置換を引き続き要求する。元の登録 seed を実
capture-safe 束縛置換に従い、証人型と属性本体の二つの open leaf へ順序付き Expanded で結ぶ。

この source operation だけが別の active nonempty seed を NoConcreteVc(BuiltinSetInhabitation { origin })
に対応させ、実 Exists-set 目標と行を保持する。汎用 intake は active goal を VC 候補のままとする。独立 base inhabitation により choice
型事実を生成し、属性目標を仮定しない。seed/term/formula/leaf/source map/owner の不足・余剰を拒否し、公開 reason/ID や provenance text
を受理証拠にしない。

## Three-registration source-proof extension

実装した集約 profile は Core 全 graph を認証し、各元親を証人型と完全な属性目標の順序付き leaf に対応させる。既存 generator signature のまま Open VC 6件を生成する。
独立に再照合する active nonempty seed 3件と各自の BuiltinSetInhabitation accounting を保持し、汎用 intake は変えない。同じ型名や別の親の証明で gate を満たさない。
連言 leaf は両証人属性の既存 generated And と実際の両ガード付き定義前提を使う。canonical label だけを前提にせず、一方の conjunct や theorem の真偽で目標を置換しない。
各 VC を実 parent・proof・origin・型付き参照から対応付け、証拠の不足・余剰・親の混同を拒否する。元の単一登録出力と [handoff](./kernel_evidence_handoff.md#three-registration-source-proof-extension) 再照合を維持する。

## Bounded computation request generation

`generate_source_computation_request(&CoreIr, BuildSnapshotId, &GenerationSchemaVersion, &VcSchemaVersion) -> Result<VcSet, String>` は唯一の source 付き theorem、別々の zero 出現二つ、等式、終端 computation 要求、active theorem-proof seed/backref を検証する。
既存 intake/candidate/normalization で実 Core goal と厳密な seed accounting を保ち、正規化前に実要求由来の `ComputationHint::SymbolicRequest(ProofHintKey("by-computation(steps:<digits>)"))` を付ける。
steps 表記と source provenance を保持し、`0` は実行主張なしの無制限要求のままとする。VC は前提・discharge・policy 置換・反射等式受理なしで Open に留める。
Core goal hash は保守的なままで、dependency slicing は opaque computation dependency を残し、discharge/reuse evidence にはしない。source 経路は検証済み Core seed から active handoff を直接構築し、汎用 Core-aware handoff は要求を捨てず computation 終端を明示的に deferred にする。

## Flat snapshot postconditions

実装済みの snapshot 拡張は Core/CFG 捕捉を完全検証して既存 `generate_source_algorithm_postconditions` を使う。初期化 ghost var/const は既存 program-value 世代を使い、runtime sink の ghost 依存拒否を保つ。捕捉は状態・仮定を変更せず、通常の返却 postcondition は Open に留める。
全 handoff 行を保持し、ghost 初期化は既存 GhostErasure/NoConcreteVc、終了は元の未解決 status を残す。捕捉は証明や ghost-erasure 証明書ではなく、snapshot claim 置換・限定 while 以外の loop・限定 return Pick 以外の事後条件失敗は未対応とする。

## 限定 while 事後条件

`generate_source_algorithm_postconditions` は一ループ Core グラフと新規 CFG/handoff、ループ所有の入口・後退辺 site、実際の最終 return を検証する。認証済み本体の代入先から MayWrite を計算し、そのローカルだけを cutpoint で freshen して不変仮引数を保つ。初期化事実は入口だけに残す。入口・保存 invariant と return の Open VC を生成する。保存は havoc 後の invariant・guard に実際の旧状態代入を適用し、return は cutpoint invariant と guard の否定を使い、本体代入事実を使わない。この source generator 内で両 invariant handoff と具体化した出口要約を認証する。出口仮定は既存 LoopInvariantAvailable、未解決依存は PremiseRef::ConservativeUnknown とし、実 dependency slice を再利用不可に保つ。型付き依存リンクや単独 VcSet のソース認証は約束しない。本体 invariant は帰納仮定であり受理済み証明や自己依存ではない。全 handoff 行と未解決の termination metadata を保持し、decreasing VC・実行・discharge・昇格を追加しない。

## 限定 runtime Pick 事後条件

`generate_source_algorithm_postconditions` は、ソース由来 Pick/Return の完全な組、範囲包含、相異なる parameter/result/Pick/logical binder、bare-set predicate、一つの ExistingCore inhabitation seed、新規 CFG/handoff を認証する。選択値を仮定せず、実際の `exists q. is_set(q)` を Open `GeneratedNonEmptiness` VC として生成し、実 Pick 文で識別する p と厳密な object(a), set(p) 文脈から Open 事後条件を生成する。result を p で置換し、型互換性から p=a、ID の違いから不等性を導かない。未解決の algorithm termination accounting を保持し、builtin 抑制、StableChoice origin、Discharged 根拠、汎用 CFG 再設計を行わない。意味上の認証は結果全体の replay が所有し、依存・再利用は保守的に扱う。

## Bounded unary child-request descriptor

`generate_source_unary_child_call_descriptor(source, typed, symbols, core, request)` は [限定 Core producer](../../mizar-core/ja/elaborator.md#static-source-algorithm-lowering) を fresh Checker 認証と Core/request 全体の再生成一致の後で消費する。上流エラーを保持し、記述子の拒否は `algorithms.unary_child.invalid_request`。既存 ID の組 `((formal, actual), (formal_guard, actual_guard), (callee_result, caller_result))` を不可分に返す。元の atomic object guard は捕獲のない入力対応を表し、結果スロットは対応情報だけを表す。immutable Core/request が元の Apply/Return/source 関係を保持する。

この記述子は schedulable E4 graph、Kernel/Proof 検査、evidence/adoption と意味的 credit を除外する。[child 存在 producer](#bounded-unary-child-existential) が caller 入力の instantiation 検査を所有し、Proof adoption は別の責務とする。結果の等式や terminating 意図は evidence を与えない。range と syntax/environment 一般化は保留する。

## Bounded unary caller E2 request

`generate_source_unary_child_call_caller_e2_request(source, typed, symbols, core, request)` は既存 Checker と Core lowering で新規認証し、Core/request 全体の一致を確認して caller の `ensures result = actual` を要求する。上流 error を先に返し、局所拒否は `algorithms.unary_child.invalid_caller_e2_request` とする。既存 `(LocalContext, [VcGeneratedFormula; 4])` は元 caller object guard と、全 parameter/result/Var ID より fresh な result 宣言（加算検査）だけを保持する。policy inputs と provenance は空である。

行は正確な caller/flow/Return/actual の AlgorithmReturns、fresh result と actual の ProgramEquals、Implies、宣言 result の Forall である。ID は行番号、全行は SplitGoal、value の definition annotation は無い。`object(actual) |- forall r. (returns_caller(actual,r) -> r=actual)` を未証明のまま要求し、returns や ensures を前提に加えない。入力は不変。この request は checking、Proof authority、acceptance/status、currentness、complete footprints/cache、promotion/publication を除外する。旧 descriptor/E4/E1/child/direct-return/Proof 経路は clause を持つ unary pair を引き続き拒否する。詳細は [英語版](../en/generator.md#bounded-unary-caller-e2-request) を参照。

## Bounded unary child existential

`generate_source_unary_child_call_existential_request(source, typed, symbols, core, request)` は descriptor で Core/request 全体を認証する。既存型の組 `(LocalContext, [VcGeneratedFormula; 2], CoreTermId)` は元 caller object guard のみを CheckerFact とし、fresh result binder、行0の `AlgorithmReturns(callee, flow, callee Return, actual, r)`、行1の `Exists r`、actual witness を保持する。immutable Core/request は source・Apply・Return・formal/actual/result-slot 対応を保持する。VcSet・seed・scheduling・snapshot identity は作らない。

`check_source_unary_child_call_existential_result` は再生成した request を同じ呼出しの [Kernel 規則](../../mizar-kernel/ja/checker.md#unary-child-existential-instantiation) で検査し、変更しない request と既存の不透明な結果を不可分に返す。上流認証エラーが `algorithms.unary_child.invalid_existential_request` に先行する。元 direct-return API は維持する。Proof identity/current-policy/adoption、caller E4 受理、E1、一般置換、range、syntax/environment 一般化は延期し、evidence・status・promotion・publication・coverage credit は与えない。詳細は [英語版](../en/generator.md#bounded-unary-child-existential) を参照。

## Bounded unary caller E1 inhabitation request

`generate_source_unary_child_call_caller_e1_inhabitation_request(source, typed, symbols, core, request)` は既存の [child existential producer](#bounded-unary-child-existential) を先に呼び、新規認証全体と上流エラーを維持する。context、fresh binder、row metadata、存在量化 row、actual-input witness をそのまま保持し、row zero の shape だけを `ProgramTypePredicate(object(r))` に置換する。唯一の caller object-input guard の下で、normal return や requires/ensures を仮定せず、全 typed input に `exists r. object(r)` を要求する。局所 shape 拒否は `algorithms.unary_child.invalid_existential_request` を維持する。

この tuple は checking、native adoption、identity、status、acceptance、currentness、cache、promotion、publication を与えない。E1 typing/single-valuedness、E2、algorithm 全体の credit は別であり、E4 と direct-return API は変更しない。

`check_source_unary_child_call_caller_e1_inhabitation_result` は同じ入力で先に request producer を呼び、上流エラーを維持する。fresh な宣言 binder、atomic object body、actual-input witness、元 guard を既存の [Kernel introduction rule](../../mizar-kernel/ja/checker.md#限定-e1-inhabitation-の存在導入) に一度渡し、元 guard を child obligation と唯一の context の両方に使う。成功時だけ変更しない tuple と同じ Core に束縛された opaque `ObjectInhabitationCheck` を返し、局所拒否は `algorithms.unary_child.invalid_existential_request` を維持する。この checking は caller object inhabitation のみを与え、上記の後続権限は別途必要である。

## Bounded unary caller E1 typing request

`generate_source_unary_child_call_caller_e1_typing_request(source, typed, symbols, core, request)` は既存 [caller E4 producer](#bounded-unary-caller-e4-descriptor) を一度先に呼び、新規認証全体と上流エラーを維持する。既存の `(LocalContext, [VcGeneratedFormula; 4])` は元 context、caller returns 行、fresh binder を保持する。行1は object(result)、行2は returns から object への含意、行3は元 result binder の全称量化となる。元行の metadata を保持し、追加行は空 provenance の SplitGoal とする。局所 shape 拒否は `algorithms.unary_child.invalid_caller_e4_request` を維持する。

元 caller object-input guard のみの下で `forall r. returns_caller(actual,r) -> object(r)` を要求し、return/requires/ensures を context fact にしない。request producer 自体は checking を与えない。

`check_source_unary_child_call_caller_e1_typing_request_with_schema` は同じ入力で既存 producer を先に呼び、上流エラーを維持する。正確な caller target、宣言 result binder、atomic object conclusion、唯一の元 guard、再構築した callee flow を [Kernel typing schema reporting](../../mizar-kernel/ja/checker.md#bounded-unary-caller-e1-typing) に一度渡す。成功時だけ tuple 全体を変更せず実際の同一呼出しの Kernel schema と組にして返し、局所拒否は `algorithms.unary_child.invalid_caller_e4_request` を維持する。旧 `check_source_unary_child_call_caller_e1_typing_request` は委譲して scalar だけを捨てる。要求された schema は受け取らず刻印しない。Proof schema association、policy/native adoption、single-valuedness、E2、全体受理、status、currentness、complete footprint/cache、promotion、publication は与えない。raw identity の unknown/incomplete/cache-miss と child/E4/inhabitation/direct-return 経路は変更しない。詳細は [英語版](../en/generator.md#bounded-unary-caller-e1-typing-request) を参照。

## Bounded unary caller E1 single-valuedness request

`generate_source_unary_child_call_caller_e1_single_valuedness_request(source, typed, symbols, core, request)` は未変更の [caller E4 request](#bounded-unary-caller-e4-descriptor) を同一入力で一度再生成し、認証と上流 error を保つ。既存の `(LocalContext, [VcGeneratedFormula; 6])` は元 context の prefix、policy inputs、caller returns 行を保持する。次の context ID と、元の fresh result より一つ大きい変数（加算を検査）で result 宣言を追加し、両変数を認証済み Core 変数より fresh にする。existential 行の metadata を再利用し、result だけを新変数に替えた同一 caller/flow/Return/input relation を置く。順序付き二宣言に対する And、ProgramEquals、Implies、Forall 行を追加する。ID は行番号、追加行は SplitGoal・空 provenance とし、局所 shape/overflow/context 拒否は `algorithms.unary_child.invalid_caller_e4_request` を再利用する。

元の caller object guard のみの下で `forall r,s. (returns_caller(actual,r) & returns_caller(actual,s)) -> r=s` を要求する。returns は前件に留まり、E1/E3・requires/ensures・child returns・function equation を前提や証明にしない。checking、Kernel/Proof authority、identity、policy/native adoption、E2、whole acceptance/status、currentness、complete footprints/cache、promotion/publication は与えず、既存経路を維持する。

`check_source_unary_child_call_caller_e1_single_valuedness_request_with_schema` は同じ五入力で未変更 producer を最初に一度呼び、認証と上流 error を保つ。両 caller target、両宣言 binder、順序付き等式、唯一の元 Core guard、再構築した callee flow を [Kernel schema reporting](../../mizar-kernel/ja/checker.md#bounded-unary-caller-e1-single-valuedness) に一度渡す。成功時のみ元の六行 tuple 全体と実際の同一呼出しの Kernel schema を返し、局所 shape/flow または Kernel 拒否は `algorithms.unary_child.invalid_caller_e4_request` を再利用する。旧 `check_source_unary_child_call_caller_e1_single_valuedness_request` は委譲して scalar だけを捨てる。要求 schema を受け取らず刻印しない。Proof association、identity、policy/native や後続 authority は与えない。

## Bounded unary caller E4 descriptor

`generate_source_unary_child_call_caller_e4_request(source, typed, symbols, core, request)` は既存 child existential を先に再生成し、source/Core/request 全体の新規認証と上流エラーを維持する。既存の `(LocalContext, [VcGeneratedFormula; 2])` を返し、caller object guard と fresh result binder を共有する。row zero は caller の元 CFG flow・Return・actual を結ぶ `AlgorithmReturns`、row one はその結果の存在量化である。不変の Core/request が Return→Apply→callee/actual、formal/result-slot/source の対応を保持する。child witness、VcSet、seed、scheduling identity は生成しない。

`check_source_unary_child_call_caller_e4_result(source, typed, symbols, core, request, child)` はこの descriptor を先に再生成し、明示的 child evidence の検査より前に上流エラーを維持する。正確な caller relation と元 guard を [Kernel composition 規則](../../mizar-kernel/ja/checker.md#bounded-unary-caller-operational-composition) に渡し、変更しない descriptor と既存の不透明な結果を不可分に返す。局所拒否は `algorithms.unary_child.invalid_caller_e4_request`。同値な clone Core でも、その arena で新たに検査した child が必要である。

この descriptor 単独では checked operational composition と normal-return evidence は与えない。受動的な [Proof caller identity](../../mizar-proof/ja/status.md#arena-local-caller-e4-association) は current-policy/adoption、E1/E2、全体受理の権限を与えず、child identity は caller target の権限にならない。unknown dependency、cache miss、caller 所有の currentness は維持する。一般 evidence・status・publication・complete footprint・profile 拡大は含まない。詳細は [英語版](../en/generator.md#bounded-unary-caller-e4-descriptor) を参照。

## Bounded direct-return E1/E4 producer design

`generate_source_algorithm_direct_return_requests(source, check, symbols, core, snapshot, generation_schema, vc_schema)` は、変更しない [idalgo source](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz) の要求専用経路である。immutable object parameter、object result、equality ensures、唯一の `return a` に限定する。[fixture expectation](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.expect.toml) と既存 postcondition generator は維持する。call、loop、Pick、recursion、group、syntax input、premise exposure は範囲外。

**真正入力と関係。** 直接依存する Checker/Resolver が source typing と resolved symbol の認証を、Core が lowering と CFG を所有する。同じ immutable resolved source、typed tree、symbol environment を再検査し、完全な Core を再構築して比較した上で、完全な CFG/handoff を生成・検証する。algorithm symbol、認証済み terminating intent、順序付き object parameter/result guard、equality ensures、実 parameter return reference、唯一の statement、source map、provenance を要求する。requires 省略は P=⊤。ID、名前、metadata、等式だけでは認証しない。関係 atom は当該 algorithm、flow、return statement、実 immutable input、別に束縛された候補 result を参照し、この source body の正常実行とその結果を意味する。未検証の operational request であり、等式の別名、関数等式、検証済み証拠ではない。

**独立した四つの Open goal。** object(a) だけを前提とし、capture のない明示的な result binder を使う。E1 typing は `forall r. returns(a,r) -> object(r)`、E1 single-valuedness は `forall r,s. (returns(a,r) & returns(a,s)) -> r=s`、E1 全 typed input inhabitation は `exists r. object(r)`、独立した E4 は `exists r. returns(a,r)`。実 return a は将来の witness 候補であり、生成済み証明ではない。inhabitation は return/P を、単一値性は E1/E3/ensures を、E4 は宣言意図・関数等式・E2 continuation・受理済み postcondition を仮定しない。

新 VcSet は元 postcondition を保持し、その source seed を四要求へ個別に展開する。goal のない PartialTermination 行は Deferred のまま。真正な再生成で source/type/body/return/CFG association、snapshot、generation/VC schema を保持する。binder declaration は premise ではない。意味認証には結果全体の再生成が必要であり、独立した VcSet 構築は認証を与えない。未完成 dependency footprint は conservative/reuse-ineligible のまま。proof policy や checker-schema receipt を捏造せず、adoption は実 checker/policy も束縛する。

**Proof 境界。** 真正 Kernel check、policy、義務別 identity、原子的 replay/adoption は Proof が所有する。[既存 source identity](../../mizar-proof/ja/status.md#source-algorithm-return-obligation-identity) と [return reflexivity result](../../mizar-proof/ja/status.md#source-algorithm-return-reflexivity-status) は旧 postcondition だけを扱う。Open/missing/rejected/stale/cross-bound E1 は記号導入を block し、E4 は独立検証まで未取得。E1 は E4 を、E4 は E1/E2 を補わない。status promotion、aggregate receipt、acceptance、coverage、E5 execution、import を与えない。

**Producer 行列（五ケース）。** 真正 source/check/Core/CFG を使い、全 formula/context/seed accounting/provenance を比較する。

| Case | Producer の結果と独立した proof control |
|---|---|
| 変更しない fixture | 全出力を実 upstream data と比較し、元 postcondition、Deferred termination、別々の四 Open goal を保持する。 |
| E1 typing | parameter/result typing の欠落・cross-binding を拒否し、typing 自体は証明を要する Open goal。 |
| E1 single-valuedness | 別々の量化 result が同じ relation/input を共有し、ensures/E3 は premise/proof にならない。 |
| E1 inhabitation | 全 typed input で object result が存在する goal。return/P 仮定や precondition 下だけの witness 一般化はない。 |
| E4 と isolation | 独立 return existential。stale/cross-source/check/Core/CFG、schema 変更は拒否または異なる再生成結果となる。欠落・差替え proof、宣言、旧 postcondition、continuation、stopping-only evidence は adoption を許可せず、この producer は adoption を行わない。 |

### 元の E1 typing descriptor

`generate_source_algorithm_e1_typing_request(source, check, symbols, core, snapshot, generation_schema, vc_schema, vc_id)` は authentic direct-return graph を再生成してから Open E1 typing のみを選ぶ。元の単一 Forall result binder、含意、AlgorithmReturns 前件の正確な algorithm/CFG/return/input 関連、atomic object 結論、唯一の CheckerFact 入力 guard/premise を要求する。source/Core/CFG 認証は変更しない producer が所有し、その error は選択拒否に先行する。

`(VcSet, ContextEntryId, VcFormulaRef, VcFormulaRef, VcFormulaRef, DependencySlice)` として、変更しない graph 全体、元 binder、前件、結論、Core typed-input guard、元の raw slice 全体を返す。元の式への参照であり、置換、子の scheduling、checking、proof credit は与えない。Open4/Deferred、unknown/incomplete/cache-miss、呼出側 currentness を保つ。環境解釈、single-valuedness、adoption、symbol introduction、promotion、reuse、acceptance/coverage は伴わず、formula-cycle hardening は別件のままとする。

`check_source_algorithm_e1_typing_request` は先にその記述子を再生成し、上流 error と選択拒否を保つ。元の binder、returns 前件、object 結論、Core guard のみを Kernel の[限定型付け規則](../../mizar-kernel/ja/checker.md#bounded-original-e1-typing)へ射影し、成功時には変更しない六要素 tuple 全体を返し、失敗時は拒否する。E4 converse、native evidence、Proof adoption、status 変更は与えず、記述子の Open4/Deferred とその他の除外事項を保つ。

`check_source_algorithm_e1_typing_request_with_schema` は同じ記述子を先に認証し、その呼出しの検査成功時だけ Kernel 所有の `u16` スキーマを取得し、`(変更しない六要素 tuple, schema)` を一体で返す。検査失敗は `algorithms.direct_return.invalid_e1_typing_check` を保つ。記述子のみの API はこれに委譲してスキーマだけを除く。この scalar は Proof policy eligibility/adoption を与えない。

### 元の E1 single-valuedness descriptor

`generate_source_algorithm_e1_single_valuedness_request(source, check, symbols, core, snapshot, generation_schema, vc_schema, vc_id)` は authentic direct-return graph を再生成してから Open single-valuedness のみを選ぶ。元の順序付き相異なる二つの Forall result 宣言、同一 algorithm/CFG/return/input を共有するちょうど二つの AlgorithmReturns atom を前件とする含意、対応する ProgramEquals 結論、唯一の CheckerFact 入力 guard/premise を要求する。producer error は選択拒否 (`algorithms.direct_return.invalid_e1_single_valuedness_request`) に先行する。

`(VcSet, [ContextEntryId; 2], [VcFormulaRef; 2], VcFormulaRef, VcFormulaRef, DependencySlice)` として、変更しない graph、元 binder、元 returns atom、等式結論、Core guard、元の raw slice 全体を返す。native typing result は不要。元の式の射影であり、置換、checking、新たな proof premise、credit を与えない。Open4/Deferred、unknown/incomplete/cache-miss、呼出側 currentness を保ち、環境解釈、adoption、symbol introduction、acceptance は範囲外。formula-cycle hardening は別件のままとする。

`check_source_algorithm_e1_single_valuedness_request` は先にこの descriptor を認証し、producer/選択 error を保つ。元の順序付き binder 宣言、二つの returns atom、等式、Core guard を Kernel の[限定 single-valuedness rule](../../mizar-kernel/ja/checker.md#bounded-original-e1-single-valuedness)へ射影する。成功時のみ six-tuple/raw slice 全体を変更せず返し、局所拒否は `algorithms.direct_return.invalid_e1_single_valuedness_check` とする。native typing prerequisite、Proof association/adoption、status 変更、symbol introduction、acceptance credit は与えず、descriptor の除外事項を保つ。

`check_source_algorithm_e1_single_valuedness_request_with_schema` は同じ descriptor を先に認証し、その呼出しの検査成功時だけ Kernel 所有の `u16` スキーマを取得し、`(変更しない六要素 tuple, schema)` を一体で返す。検査失敗は `algorithms.direct_return.invalid_e1_single_valuedness_check` を保つ。descriptor のみの API はこれに委譲してスキーマだけを除く。この scalar は native typing prerequisite や Proof association/policy/adoption を与えない。

### E1 inhabitation witness-reduction request

`generate_source_algorithm_e1_inhabitation_witness_request(source, check, symbols, core, snapshot, generation_schema, vc_schema, vc_id)` は既存 producer で四要求の authentic graph 全体を再生成し、元の error を保つ。Open E1-inhabitation existential のみを選び、変更しない `VcSet`、実 return の `CoreTermId`、元の binder `ContextEntryId`、置換後の子 `VcFormulaRef::Core(parameter.ty_guard)`、親の raw `DependencySlice` 全体を返す。選択 ID と元 set 全体で goal/context/anchor/fingerprint を保持する。Proof identity の権限は既存 Proof route に残る。

binder と term の組は object(r) の fresh r を実 immutable a に置換する記述である。単一 existential binder、object body、parameter との変数の相違を要求する。atomic body に nested binder はなく capture は生じない。子は元の typed-input context における authentic object(a) guard の要求記述であり、新たな実行可能 VC や検証済み前提ではない。親の式・provenance・Open 四要求・Deferred termination・incomplete/reuse-ineligible slice を維持する。generic substitution、Kernel wire/evidence、checking、existential introduction 検証、Proof adoption、symbol introduction、promotion、publication、acceptance/coverage credit は与えない。

### E4 operational-reduction request

`generate_source_algorithm_e4_operational_reduction_request(source, check, symbols, core, snapshot, generation_schema, vc_schema, vc_id)` は authentic direct-return graph 全体を再生成した後、その Open E4 existential のみを選ぶ。既存型の tuple として、変更しない `VcSet`、実 return の `CoreTermId`、元の existential `ContextEntryId`、独立した置換後の `VcGeneratedFormulaShape::AlgorithmReturns` と残余 `VcGeneratedFormulaShape::True`、親の raw `DependencySlice` 全体を返す。選択した元の goal/context/anchor と Proof identity は変えない。

単一の fresh existential binder と、元 relation の正確な algorithm・CFG・唯一の return statement・immutable input を要求する。candidate result を、その statement が実際に返す parameter に置換する。atomic relation に nested binder はない。`returns(a,a)` から `True` への組は、immutable variable の評価がその input を返し、唯一の `Return` がその値で正常終了するという具体的な operational step の要求である。未検証の reduction proposal であり、検証済み実行や正常復帰の fact ではない。残余 `True` は operation や existential introduction の証明を与えない。equality alias、premise、relation interpretation、graph rewrite、新たな実行可能 VC、evidence、checking、adoption、symbol introduction、status promotion、publication、acceptance/coverage credit は導入しない。四要求は Open、termination は Deferred、raw slice は incomplete/reuse-ineligible のままとする。call・他の body・任意の candidate result の reduction は対象外とする。


### E4 authenticated validator handoff

`generate_source_algorithm_e4_validator_handoff_request` は operational-reduction producer の入力と借用した `[(CoreVarId, CoreFormulaId, T)]` environment を受け取る。environment を調べる前に authentic E4 descriptor を再生成する。認証済み immutable parameter と guard に一致する単一 binding のみを許し、不正な binding には handoff を返さない。既存 tuple だけで `(original_descriptor, (core, algorithm_id, environment), (statement_id, returned_term_id, &value))` を返す。raw slice は選択した target ID を保持する。元の graph・existential binder・raw slice・独立した shape 全体を保持し、渡された Core・environment・opaque value を複製・解釈せずそのまま借用する。

入力 tuple は Kernel の[条件付き validator](../../mizar-kernel/ja/checker.md#bounded-varsole-return-local-validation) への正確な要求であり、出力 tuple は期待する局所的 association であって検証済み結果ではない。この関数は Kernel を呼ばない。初期化と parameter guard の意味的充足は呼出側の明示的前提であり、guard ID は association のみを示す。environment premise は VC context に追加しない。Kernel checking・元の E4 proof・existential introduction・result typing/contracts・Proof identity/currentness/policy binding・adoption は別途必要である。四要求は Open、termination は Deferred のままで、evidence・symbol・status・publication・acceptance/coverage credit を与えない。

### E4 checked local request

`check_source_algorithm_e4_local_request` は handoff と同じ引数を受け、最初にそれを呼んで上流エラーを保持する。返された入力 tuple そのものを変更しない Kernel `check_sole_return_var` に渡す。実際の statement/term ID と借用値のポインタ同一性が期待出力に一致することを要求し、拒否時はエラー、成功時だけ変更しない元 descriptor を返す。任意の T に Eq/Clone を要求せず、値を解釈・複製しない。

これは handoff の呼出側前提に基づく条件付き局所検査であり、意味的 environment association や元の AlgorithmReturns proof は与えない。E4 existential introduction、Proof identity/current-policy binding、adoption は延期のままである。descriptor/raw slice と Open4/Deferred は変わらず、汎用 evidence/status/reuse や広い credit は与えない。

### E4 checked instantiated child

`check_source_algorithm_e4_instantiated_request` は operational-reduction producer と同じ
引数を受け、最初に呼んでエラーを保持する。元 E4 存在量化の result binder、単一の
型付き入力 Core CheckerFact と premise 対応を認証し、置換済み AlgorithmReturns
tuple 全体・実際の witness・元仮定を Kernel の symbolic child rule に渡す。
成功時だけ変更しない元 descriptor を返し、拒否は atomic。型付き入力仮定の下の
`returns(a,a)` のみを検査し、opaque 実行値や既存 local consumer の前提は変えない。
この child consumer では E4 存在導入・意味的 environment 対応・Proof identity/current-policy binding/adoption は延期。
raw unknown・incomplete/cache miss・Open4/Deferred と credit の除外を維持する。
詳細は [英語版](../en/generator.md#e4-checked-instantiated-child) を参照。

### E4 checked original existential

`check_source_algorithm_e4_existential_result` は operational-reduction producer と同じ
引数を受け、最初に呼んでエラーを保持する。単一の元 existential binder、元の
AlgorithmReturns body、型付き入力 Core CheckerFact と正確な premise 対応を認証し、
元 tuple・変更しない binder・実際の witness・元仮定を Kernel の存在導入 rule に渡す。
成功時だけ変更しない descriptor と同じ認証呼出しで生成した真正な Kernel
`AlgorithmReturnsExistentialCheck` を返す。`check_source_algorithm_e4_existential_request`
は委譲し result のみを捨てて descriptor-only 動作を維持する。先行 boolean や detached child/residual は
証明にならず、既存 child consumer は独立する。[Proof 元 identity 関連付け](../../mizar-proof/ja/status.md#original-e4-request-identity-association) は独立する。Proof が限定的な [current-policy 消費](../../mizar-proof/ja/status.md#original-e4-current-policy-consumption) と [native 元 E4 adoption](../../mizar-proof/ja/status.md#native-original-e4-adoption) を所有する。意味的 environment 対応、別の E1 義務、広範な body 解釈は延期する。
raw unknown・incomplete/cache miss・Open4/Deferred と evidence・symbol・promotion・
publication・acceptance/coverage の除外を維持する。
詳細は [英語版](../en/generator.md#e4-checked-original-existential) を参照。

## Reflected Algorithm Obligations

§§20.7/20.13.1 は E1 の型/inhabitation/単一値性と検証済み E2 部分正当性を要求する。E4 正常復帰は terminating/昇格の文脈に適用し、すべての部分定義には要求しない。
C/文脈を含む完全な core/CFG 入力を消費し、Adm/InputOK と適合環境ガードを保つ。宣言を証明にしない。
子呼出の前提/正常復帰、失敗、範囲、invariant、選択された decreasing measure を義務に含める。
テストは許される文脈で guarded な open syntax を含む E4 のない guarded 部分正当性を受理する。全入力の結果 inhabitation/単一値性の欠落、継続だけによる子復帰、合法な partial match 終端省略に対する暗黙失敗 handoff の欠落、未検証 promotion/方程式ラベルを拒否する。選択 measure、子 requires/復帰、明示/暗黙失敗の対照例は完全なガードを保つ。
既存の symbolic computation request は Open な転送に留まり、E5 実行と公開は実際の生成側が所有する。

## Reflected Formation Requests

暫定 request から source-authenticated conditional library/all-input Binder formation goal を生成し、算法 E1/E2/E4 とは独立させる。有限 KΓ/Adm equivalence、range/construction/uniqueness、正確な premise と UID/context/C binding を保持する。Open goal は formation を認証しない。Proof acceptance は対応する formation-dependent checked/runtime consumer より先に Checker に戻す。

### Formation dependency inventory and request-only design

これは文書上の設計であり request format の実装ではない。適用条件は [§13.10](../../../spec/en/13.term_expression.md#1310-function-binder--) と [§20.5.3](../../../spec/en/20.algorithm_and_verification.md#2053-reflected-syntax) に従う。選択された操作に規定された前提だけを要求する。structural retrieval/matching と T7 は現在の数学的値のない open Var も扱えるが、formation/meaning には固有の条件が必要。構造アクセスも空虚な全称命題も domain element/compatible environment を供給しない。
正確な参照、条件付き premise、未実装の D1–D3 certification route は [pinned library inventory](../../mizar-checker/en/typed_ast.md#pinned-library-inventory) が所有する。各適用 use は実際に instance 化した proof/premise を resolved declaration、元の type/attribute、source/tree、順序付き UID、保持 context/C に結び付ける。library/scheme の名前や無関係な受理 instance では不足する。

| Request dependency | 必要な真正入力と条件付き義務 | 未実装の production route |
|---|---|---|
| 選択した分類/use | resolved/desugared term/formula、完全な明示/暗黙 value argument、signature/application guard、opaque saved original/free UID、元の type/view premise。R1 correspondence と R2 insertion context は別。 | AST-PARSE/resolved reflected record と AST-TYPE formation intake。 |
| Fresh Binder domain/codomain | resolution 前に構築 mode を固定し、outer declaration type と期待する認証済み Function/PartFunc または exact carrier から fresh UID に依存しない A/B を決める。全入力が head type を満たし、個体の仮定や image/carrier を捏造しない。 | 認証済み type/carrier 入力と per-use compatibility request 生成。 |
| Multi-head Binder | 順序付き相異なる head、outer context 内の left-nested Pn/Dn factor、認証済み pair/product law と有限 decomposition。全 head を同時に拡張し、任意の subset-product/dependent-head へ一般化しない。 | library binary foundation/derivation と reflected product/decomposition request。 |
| Body/range/view | 実際の A の全 packed input について extended-context compatibility、body typing/application、使用 premise 付き E(u)∈B。依存する元の型はその環境で解釈し、固定 view は all-input proof を要する。restriction は全体構築後。 | all-input formation goal と一致する検証済み instance。 |
| Construction/uniqueness | 実際の well-formed な認証済み scheme instance、exact dom=A、rng⊆B、pointwise value/uniqueness、B={}⇒A={} を含む。PartFunc expectation でも新規 Binder の実際の A を縮めない。 | 適用 scheme-premise 検証と per-use construction/uniqueness evidence。 |
| Empty domain | 対応する syntax/type と認証済み構築は必須。空の A は現在の head/body value や body environment を作らず空関数を与える。A={} から factor を推論しない。 | 同じ formation route を通る真正な empty-domain 検証。 |
| Capture/reconstruction | 保存した実際の A/input type、完全な順序付き capture group、outer/extended context、capture-specific C/default B。Function は A=A0、PartFunc は A⊆A0 を要求し A を保持。明示 B' でも range proof を保持し、Var equality から C を同一視しない。R2 は free UID を保持し、衝突する inner binder と依存参照だけを rename する。 | 真正な capture/C correspondence、per-use reconstruction goal と adoption。 |
| 有限 KΓ/Adm | classification row ごとに使用した type/attribute/dependency/application/view 条件だけを記録し、AdmΓ(ρ) iff ∧KΓ(ρ) を証明。量化制限/local implication guard を保持し、有限 formula を列挙せず全 ambient assumption に置換しない。 | reflected condition-record producer と equivalence 検証。 |
| M1–M3 | 条件付き node meaning は implicit value/signature/guard を含む。pure R1 correspondence、mixed R2 meaning、保存した opaque term/formula/FV context は実際の基礎証明が必要。boolean term と truth condition は別。 | conditional foundation request と Proof 検証。 |
| M4 | 上記 all-input construction により internal extended environment で Binder meaning を与え、空 A も含む。新 kernel lambda rule や無条件の public val_at は作らない。 | conditional valuation と結び付いた per-use Binder evidence。 |
| M5–M7 | M5 は現在の Adm を保持し updated compatibility equivalence を証明し、v は update 前に解釈。M6 はさらに local valid_at と UID-independent return type または正当な view が必要。M7 は両環境の compatibility と変更 UID 以外の一致、および構造上の non-occurrence を要求し逆は認めない。UID substitution は捕獲を避け type/domain 参照を含み挿入 term に再入しない。 | guard 付き update/view/non-occurrence/substitution request と受理 instance。valid_at 展開だけでは条件の証明にならない。 |

実際の seam と責務:
- Checker input: [syntax Binder node](../../../../crates/mizar-syntax/src/ast.rs)、[resolver collect_declarations](../../../../crates/mizar-resolve/src/names.rs)、[Checker source-context installation](../../../../crates/mizar-checker/src/type_checker.rs) は限定された数学的 Binder の shape/head/context metadata を供給する。resolver は algorithm/match と明示 expr-annotation host を拒否し、これは真正な reflected classification/all-input formation ではない。[BindingEnv/BindingContextOwner/BinderIdentity](../../../../crates/mizar-checker/src/binding_env.rs) は identity を保持するが KΓ/compatible valuation environment は作らない。[SourceEvidenceRequestKind](../../../../crates/mizar-checker/src/source_evidence.rs) に reflected formation payload はない。
- Core transport: [provisional formation boundary](../../mizar-core/en/core_ir.md#provisional-formation-transport) は真正な暫定 formula/structure と正確な入力の zero-credit 保存を所有する。[Core IR](../../../../crates/mizar-core/src/core_ir.rs)/[elaborator](../../../../crates/mizar-core/src/elaborator.rs) に reflected formation transport はなく、既存 checked lowering で代用したり暫定 request を実行したりしない。
- VC requests: [generator](../../../../crates/mizar-vc/src/generator.rs) には source-theorem/symbolic-computation/algorithm-postcondition 生成があるが reflected formation goal はない。既存 Open request は義務を discharge しない。将来の request は表の正確な条件付き meaning を E1/E2/E4 と独立に保持し Proof に渡す。受理済み formation evidence を先に要求する必要はない。
- Proof acceptance: [formation evidence policy](../../mizar-proof/en/policy.md#formation-evidence-boundary) が真正な library/per-use 検証を所有する。[ProofStatusProjectionInput/ProjectedProofStatus](../../../../crates/mizar-proof/src/status.rs) と [Kernel imported/formula evidence](../../../../crates/mizar-kernel/src/checker.rs) は供給済み evidence を消費/投影するだけで reflected formation authority は作らない。
- Checker adoption: [formation boundary](../../mizar-checker/en/typed_ast.md#reflected-syntax-formation) は元 request、正確な premise、現在の source/tree、library version/content/declaration、type/UID/context/C に一致する真正な受理 evidence の re-entry を所有する。Open/rejected/missing、stale、wrong-premise、cross-context/swapped evidence は adoption を block する。production adoption join はなく、再生成時も requested meaning を弱めない。adoption 後にのみ final checked Core handoff が可能。runtime execution、algorithm-contract acceptance、public-fact import は別 producer を必要とする。

既存の [Binder parser test](../../../../crates/mizar-test/src/runner/tests/binder.rs)、[resolver binding test](../../../../crates/mizar-resolve/src/labels/tests.rs)、[Kernel evidence test](../../../../crates/mizar-kernel/src/formula_evidence/tests.rs)、[Proof status test](../../../../crates/mizar-proof/src/status/tests.rs) は syntax/identity または汎用 evidence control を検査するが、この request→proof→adoption route は検査しない。真正な accepted と Open/failed の all-input/empty-domain formation、used-only KΓ/equivalence と conditional M1–M7、wrong/missing premise、stale UID/context/C、capture collision、same-name library impostor、swapped-evidence rejection は真正 producer まで延期。formation だけで E1/E2/E4 を満たしたり T4/T6 import を許可したりしない。この inventory は executable coverage/acceptance を供給せず、未解決の foundational/acceptance meaning は人間が所有する。

## Full Computation Closure Admission

ソース所有の R6-A/B 閉包・分類準備は R3 に先行できるが、VC58 の同一パッケージ Open 目標を含む R6-C 一意性目標生成は R3 の検証済み export を前提とする。

将来の computation admission は真正な local/検証済み exported body/dependency から、内部 private helper を含め direct/transitive に評価する全算法を対象とする。non-ghost Pick/unordered set iteration を含む各 value-returning callee に type/precondition guard と完全な ensures uniqueness goal (§20.9.2) を保持するか、t/φ に result が現れない正確な result=t/result=true iff φ exemption を確認する。procedure 自体はこの obligation の対象外。非適用の value-returning control は non-ghost Pick と unordered set loop の両方を持たない。ordered sequence と non-ghost Pick、ghost Pick と unordered set loop の組合せは引き続き要件を発生させる。不足する closure/classification/contract input は admission を block する。テストは各適用境界、imported/transitive nondeterministic callee、真正な proof/exemption と Open/failed goal、古い contract/group/call/dependency、偽 exemption を explicit computation/ground-evaluation discharge の両方で点検する。

## Source theorem VC generation

`generate_source_theorem_vcs(&CoreIr, BuildSnapshotId, &GenerationSchemaVersion, &VcSchemaVersion) -> Result<VcSet, String>` は generator.rs の既存 generic raw generation block だけを所有する。
build_control_flow_ir、build_obligation_seed_handoff、SeedIntakeTable::try_from_handoff の順で、既存 validation と Display error text を保つ。
実 Core package/path の UTF-8 byte 長と値を `package={}:{};module={}:{}` に frame した VcModuleRef を使い、指定 generation schema、実 intake/handoff/Some(flow) で CoreGenerationCandidateSet::try_from_seed_intake を呼ぶ。
続いて指定 VC schema/snapshot、実 core.source_id() と同じ candidates で try_normalize し、Display error と元 graph 全体を保つ。
同一 task の直接 production caller は Proof canonical inputs、VC theorem-handoff replay、fresh Proof theorem reflexivity check の三つ。registration の専用 generator と独立 test 構築は変更しない。
最初の caller は lower/roots/Core56 を generation より前に保つ。replay は selected lookup を前に、whole equality/profile check を後に保ち、実 selected anchor generation schema と set VC schema/snapshot を使う。周囲の旧 statement、validation、構造、順序、error は全て保つ。
これは theorem-profile validator、source seal、acceptance/現在性 API ではない。type/field/adapter/helper/dependency/encoder/receipt/validation を追加せず、anchor/status/schema と generic behavior 全てを保つ。

## E1 inhabitation witness-introduction check

`check_source_algorithm_e1_inhabitation_witness` は元の記述子を再生成し、元の
existential・result binder・原子 object 本体・単一 CheckerFact とその premise を照合する。
元の型付き context と child を Kernel の限定規則へ渡し、成功時のみ未変更の記述子を返す。
上流 error の優先順序、raw slice/unknown、Open4/Deferred は保持する。
証拠、Proof binding/adoption、状態変更、symbol、promotion、publication、acceptance/coverage credit はこの API から与えない。
[元 E4 checking](#e4-checked-original-existential) は独立する。詳細は [英語版](../en/generator.md#e1-inhabitation-witness-introduction-check) を参照。

`check_source_algorithm_e1_inhabitation_result` は同じ再認証を行い、未変更の記述子と
Kernel の不透明な借用結果を組にして返す。元 binder/body/context を結果生成時に検査する。
既存 API はこの経路へ委譲する。Core のみ借用し、元の全要求・選択 ID・witness/child・
raw slice が元要求との対応を保持する。この API は Proof identity/policy 消費と adoption を行わない。
実装済みの [限定 native E1 consumption](../../mizar-proof/ja/status.md#bounded-native-e1-result-consumption) と [inhabitation adoption](../../mizar-proof/ja/status.md#native-e1-inhabitation-adoption) は Proof 所有。currentness は caller 所有のまま。詳細は同じ英語節を参照。
