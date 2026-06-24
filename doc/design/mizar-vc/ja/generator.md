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

必須 candidate family:

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

Task 7 がこれらの concrete generator implementation と tests を所有する。old-state assignment、
field-update alias identity、`not C` なしの `break` exit、`continue` decreasing check、
`downto` と `step` range loop、ghost-only `Pick` erasure の audit fixture を含む。

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

- call-precondition、return-postcondition、assertion、branch、match、loop、range-loop、
  collection-loop、termination、partial-termination、ghost-erasure candidate;
- old-state assignment、field-update alias identity、post-havoc loop、break/continue exit、
  range-loop hidden metadata、Pick non-emptiness、ghost-only `Pick` erasure の context preservation;
- traversal helper の map iteration に依存しない algorithm candidate の deterministic ordering。

Task 8 が追加すべき Rust coverage:

- deterministic candidate normalization と dense `VcId` assignment;
- no-VC、one-VC、expanded mapping を含む complete seed-to-VC accounting;
- duplicate candidate または seed ownership rejection;
- local context、generated formula、incomplete anchor の stable rendering/fingerprinting input。

Task 5 では active `.miz` proof-verification fixture を追加しない。runner support と
source-derived extraction seam が external gap のままであるため。
