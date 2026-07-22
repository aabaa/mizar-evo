# STEP 5 Source-Derived Core/Control-Flow Family Decomposition

> Canonical language: English. 英語 canonical:
> [../en/source_family_decomposition.md](../en/source_family_decomposition.md)。

この文書は mizar-core Task 32 の accepted output である。残るすべての
source-derived `CoreIr` / `ControlFlowIr` family を bounded follow-up task へ
分解する。これは task/dependency authority だけを変更し、language semantics、
Rust behavior、`.miz`、expectation、trace status、test list、coverage credit は
変更しない。

## Authority And Baseline

inventory は repository authority order、すなわち `doc/spec/en/`、既存 `.miz`、
`tests/coverage/spec_trace.toml`、既存 expectation、design、non-normative source
evidence の順に従う。Accepted checker input は Task 247 の
[payload-family decomposition](../../mizar-checker/ja/payload_family_decomposition.md)
である。Core Task 31 は exact Task-180 exception のままで、broad non-Task-180
`CoreIr` row と全 `ControlFlowIr` row は deferred のままにする。

Task 32 は clean-entry behavior oracle、すなわち active runner parse 96 /
declaration 4 / type elaboration 188、plan 403/368、type 236/224、pass/fail
219/184、`mizar-test` unit test 272、production path 17 を保持する。Task-247
production layout は 19,803 lines、path hash
`b36d96fed3207b415c95de27be11ade57654c6573a2f0637aa2d0a3d56aca01d`、
content hash
`5f9e716169964a861b71576957c05e2dc2538b5e0ff9d1025ef51a4bea6aa306`
である。Task 32 は documentation と deferred-reason text だけを変更するため、
runner/test-list/production-source oracle は不変でなければならない。

## Descendant Task 共通 Contract

下表の各 row は 1 nonempty logical task / 1 commit である。編集前に exact
canonical section、source family、upstream syntax-free payload、consumer、
visibility、forbidden scope、tests、coverage impact、exit criteria を固定する。

各 source-derived Core task は、特記がない限り次を満たす。

- `.miz` AST inspection は `mizar-test` に残す。
- `mizar-core` で raw syntax や resolver result を再構築せず、checker-owned
  syntax-free/source-ordered final projection を消費する。
- owner identity、dense order、source range、recovery、predecessor link、
  versioned provenance を transactionally 保持する。
- 既存 generic Core lowering API を再利用し、Task 31 exact adapter を一般化しない。
- missing、duplicate、reordered、recovered、cross-owner/module、stale
  provenance、wrong-role、partial、orphan payload は fail closed にする。
- owning task の prepared consumer で deterministic rerun と full immutable
  IR comparison を行う。
- 最小の spec-derived real-source positive case と corruption matrix を追加し、
  implementation に合わせる目的で既存 expectation を変更しない。
- rejection/diagnosticを所有するtaskは、新しくclaimする各diagnostic familyについて
  最小のspec-derived real-source negative caseを追加する。payload corruptionは追加の
  boundary coverageであり、runnerでsemantic rejectionを実行する代替ではない。
- trace credit は実行された exact slice だけを昇格する。

algorithm task は同じ logical task 内で、`mizar-test` AST extraction から
syntax-free `mizar-checker` final projection、`mizar-core` lowering までの narrow
joint path も所有する。Core Task 32 がこの joint split の canonical authority
であり、新しい checker task number を意味しない。Fresh inventory で具体的な
parser/resolver semantic identity が欠けていると判明した slice は、identity を
推測したり general unnamed gate を作ったりせず、その具体的 authority decision
まで停止する。

全 task は proof search/acceptance、registration status fabrication、artifact/schema
invention、public diagnostic code allocation、`VcId` / `ObligationAnchor`、VC、
kernel、MVM execution、code extraction、Steps 6/7 promotion を禁止する。

## Accepted CoreIr Task Graph

表の task ID は `mizar-core` に属し、checker task number は prerequisite だけを
表す。

| Core task | Bounded source-derived family | Dependencies / prepared consumer | Exit boundary |
|---|---|---|---|
| 33 | Core context、item shell、binder、declaration order、local scope、visibility、reserve/default context、source map、checker provenance。 | Checker 248、`MT10-CIR-TE`。 | Context/item identity のみ。type result、RHS、proof、registration activation、algorithm body は扱わない。 |
| 34 | Type/attribute/evidence/coercion/view lowering。written head/argument、attribute chain、accepted/missing evidence ref、normalized type、widening/narrowing request result、sethood/non-emptiness record、authenticated reduct/view path。 | Checker 249-251、Core 33、既存 Core 27-30 API、`MT10-CIR-TE`。 | Reusable conversion/evidence/view lowering のみ。source-level proof-local `reconsider` は Core 37。evidence/path/fact を推測しない。 |
| 35 | Primary/application/structure/set/choice/`qua` term と atomic/composite formula、complete child graph、binder link、generated choice/comprehension origin、source identity。 | Checker 252-257、Core 33-34、`MT10-CIR-TE`。 | truth、theorem acceptance、proof closure、implicit sethood/view、algorithm `Pick` は扱わない。 |
| 36 | Predicate/functor/attribute/mode/structure/property definition shell、parameter/guard/definiens、expansion boundary、correctness-condition/initial-obligation ref。 | Checker 259-264、Core 33-35、property syntax の parser 48、`MT10-CIR-AS`。 | correctness proof/discharge、accepted property、recursive unfolding、overload winner、axiom publication は扱わない。 |
| 37 | Statement/theorem shell、assumption/conclusion、proof-local declaration/closure、source-level `reconsider`、non-Task-180 proof skeleton/citation/case/pending-blocked/thesis/terminal goal。 | Checker 258/269-272、Core 33-35、Core 34 conversion API、parser 47、`MT10-CIR-FS`。 | Task-180 は Core 31。proof search、implicit closure、theorem fact/acceptance/discharge/verified premise は扱わない。 |
| 38 | Direct template-role Core metadata、authenticated substitution request、ordinary/template overload result、redefinition/notation root、coherence/refinement input、exposed view。 | Checker 277-279、Core 33-36、`MT10-CIR-AS`。 | Direct role のみ。missing scheme/theorem role は Core 41/Gate S1。substitution result、guessed root、accepted coherence、fresh scheme symbol は作らない。 |
| 39 | Existential/conditional/functorial/reduction registration の pending shell と correctness/initial-obligation intake。 | Checker 273、Core 33-37、`MT10-CIR-AS`。 | Pending intake のみ。accepted status、activation、closure/rewrite/trace、artifact、`VcId`、discharge は扱わない。 |
| 40 | **Blocked-reserved:** authenticated accepted-registration activation と source-derived cluster/reduction trace lowering。 | Checker 274-276、Core 34-35/39、Gate A1、MC-G004、`MT10-CIR-AS`。 | Canonical accepted verifier/artifact-status producer、schema、authentication、corruption test が命名されるまで実行不可。order/local check/obligation request から `Accepted` を作らない。 |
| 41 | **Blocked-reserved:** direct parser/syntax role にない scheme/theorem-role-dependent Core slice。 | Checker Gate S1、該当 Core 33-38、`MT10-CIR-AS`。 | Canonical parser/syntax/resolver owner が missing role を命名するまで実行不可。Task 277/Core 38 は補完しない。 |

Core 39/40 は executable pending intake と gated activation/trace を、Core 38/41
は direct role と missing role を分離し、1 logical task/commit 境界を守る。

## Accepted Algorithm CoreIr Task Graph

Chapter 20 は次の semantic family と negative boundary を命名するのに十分である。
Parser Tasks 32-34 は syntax coverage のみで、semantic producer ではない。各 task
は bounded joint source/checker/Core route を実装し、必要 identity が存在することを
semantic edit 前に確認する。

| Core task | Bounded source-derived family | Dependencies / prepared consumer | Exit boundary |
|---|---|---|---|
| 42 | Algorithm declaration/header、parameter/result、visibility、runtime/ghost `var`/`const`、mutability、resolved place/lvalue、assignment、executable `Pick` shell。 | Specs 20.1/20.3、parser 32-34 は syntax evidence、Core 33-35、必要時 Checker 277/Core 38、`MT10-CIR-ALG`。 | Core algorithm shell のみ。CFG、hidden loop state、call substitution、Pick VC、promotion、execution/extraction は扱わない。 |
| 43 | `if`/`while`/`return`/`break`/`continue` shell と owner、child order、condition/value、source/provenance。 | Spec 20.2.1-20.2.2/20.2.6、Core 42、`MT10-CIR-ALG`。 | CFG edge/path fact、exit proof、invariant attachment、reachability は扱わない。 |
| 44 | Range/collection loop shell。direction、bound/collection、explicit step、binder/body、hidden immutable bound/step value、hidden processed-set state を family 別に検証する。 | Spec 20.2.3-20.2.4、Core 42-43、`MT10-CIR-ALG`。 | Common `for` shell の1 taskだが range/collection contract と negative case は分離。CFG、finiteness/order fact、termination proof、fabricated hidden value は扱わない。 |
| 45 | `match` subject、ordered case、resolved pattern graph、capture、guard/body owner、explicit exhaustiveness/unsupported state。 | Spec 20.2.5、Core 42-43、`MT10-CIR-ALG`。 | Pattern identity/exhaustiveness/CFG fact/proof acceptance を推測しない。 |
| 46 | `requires`/`ensures`/`assert`、invariant/decreasing、call target/actual/result binder/substitution-request metadata、recursive group、declared terminating intent、measure availability。 | Specs 20.4-20.5/20.7-20.8/20.13、Core 34-35/42-45、`MT10-CIR-ALG`。 | Request/metadata 搬送のみ。call/result substitution の生成・適用・検証は VC Task-30 descendant。contract axiom、VC、termination proof/promotion/recursive encoding は作らない。 |
| 47 | Snapshot/claim shell、snapshot identity、captured algorithm context request、visible runtime/ghost locals、hidden-loop-value ref、claim body/order、missing/unsupported state。 | Spec 20.6、Core 37 theorem shellとCore 42-46、`MT10-CIR-ALG`。 | Claimは既にlower済みのCore-37 theorem/statement shellだけへlinkする。Source reconstruction、context fact、old-state substitution、claim proof、CFG capture、VC、acceptance は扱わない。 |

Chapter 20.9 MVM/`by computation`/runtime/gas と 20.10 extraction は parked
downstream work のまま。20.13 formula は VC authority であり、Core 46 は later
owner が必要とする authenticated metadata だけを記録する。

## Accepted Phase-10 ControlFlowIr Task Graph

| Core task | Bounded phase-10 family | Dependencies / prepared consumer | Exit boundary |
|---|---|---|---|
| 48 | Core 42-43 に対する deterministic basic block/edge、local、statement placement、program context、source map、fallthrough、structured exit、illegal break/continueとunsupported local declarationのsource-derived structural diagnostic。 | Core 42-43、`MT10-CFG-PV`。最初の real CFG baseline と同じ commit で `SnapshotKind::ControlFlowIr` と schema/guard を追加し、両structural diagnostic familyをreal-source negative caseでcoverする。 | Empty snapshot-infrastructure prerequisite、semantic contract attachment、VC/proof/artifact は禁止。 |
| 49 | Range/collection-loop CFG attachment。hidden immutable bound/step/domain value、loop/exit metadata、processed binding、normal/early-exit state。 | Core 44/48、`MT10-CFG-PV`。 | finiteness/order-independence theorem/VC と spelling からの hidden value 再構築は禁止。 |
| 50 | Match CFG attachment。ordered arm edge/path-condition metadata、capture initialization、join、explicit exhaustiveness/unsupported metadata。 | Core 45/48、`MT10-CFG-PV`。 | semantic algebraic matching、invented capture、exhaustiveness proof、VC は扱わない。 |
| 51 | Snapshot/claim flow state。snapshot program-point owner、captured-local identity/context、hidden-loop-value ref、claim-to-snapshot link。 | Nesting に応じ Core 47-50、`MT10-CFG-PV`。 | state theorem acceptance、old-state substitution、artifact/extraction schema、VC は扱わない。 |
| 52 | Contract/call/assert/invariant/decreasing/ghost effect/recursive group/termination metadata を CFG site/context へ attach。 | Core 46/48-51、`MT10-CFG-PV`。 | Callee/actual/result-binder/substitution-request metadata のみ。concrete substitution/VC context は VC-owned。discharge/promotion/ghost-erasure proof は扱わない。 |
| 53 | use-before-assignment、unreachable statement、immutable assignment、ghost leakage、malformed/missing call-contract、pattern/capture、snapshot/claim、alias/lvalue precisionのcomplete source-derived semantic flow diagnosticとstable internal detail/source ordering。 | Core 48-52、public code は別 authority 時だけdiagnostics registry、`MT10-CFG-PV`。各new diagnostic familyにsmallest real-source fail consumerを追加し、malformed handoffはcorruption testも行う。 | Structured local diagnosticのみ。illegal break/continue/unsupported-local structural negativeはCore 48。public code/proof/VC/acceptanceは禁止。 |

## Prepared mizar-test Consumers

これは open mizar-test Task 10 内の increment で、新 top-level task ではない。
最初の real producer/baseline が実行されるまで coverage を与えない。

| Increment | Stage/tag、phase、artifact | Dependencies / corruption boundary |
|---|---|---|
| `MT10-CIR-TE` | `type_elaboration` / `active_type_elaboration`、`expected_phase = "elaboration"`、pipeline output phase 9、body が Core 33-35 の complete deterministic `CoreIr::debug_text()` bytes である `SnapshotKind::CoreIr`。 | Exact checker/Core owner。wrong stage/tag/phase/domain/kind/hash、missing/duplicate/reordered/cross-owner/module/recovered/stale/partial input、nondeterministic bytes を reject。Broad Task-19 row は実行まで deferred。 |
| `MT10-CIR-FS` | `formula_statement` / `active_formula_statement`、`expected_phase = "elaboration"`、pipeline output phase 9、body が Core 37 の complete deterministic `CoreIr::debug_text()` bytes である `SnapshotKind::CoreIr`。 | `MT10-FS`、Checker 258/269-272、Core 33-35/37。同じ corruption boundary。truth/acceptance/proof verification/VC credit なし。 |
| `MT10-CIR-AS` | `advanced_semantics` / `active_advanced_semantics`、`expected_phase = "elaboration"`、pipeline output phase 9、body が Core 36/38-41 の complete deterministic `CoreIr::debug_text()` bytes である `SnapshotKind::CoreIr`。 | `MT10-AS`、該当 checker/Core、Core 40 は A1、Core 41 は S1。同じ boundary。blocked family は未実行・無 credit。 |
| `MT10-CIR-ALG` | `proof_verification` / `active_proof_verification`、`expected_phase = "elaboration"`、pipeline output phase 9、body が Core 42-47 の complete deterministic algorithm-bearing `CoreIr::debug_text()` bytes である `SnapshotKind::CoreIr`。 | 各 joint producer/lowerer。同じ boundary に owner/nesting/local-role/contract/snapshot-link check を追加。Stage name だけでは CFG/VC/proof result/MVM/extraction credit なし。 |
| `MT10-CFG-PV` | `proof_verification` / `active_proof_verification`、`expected_phase = "elaboration"`、pipeline output phase 10、body が全 `ControlFlowIr` を含む complete deterministic module-level `ControlFlowOutput::debug_text()` bytes である `SnapshotKind::ControlFlowIr`。 | `MT10-CIR-ALG` と Core 48-53 owning task。同じ boundary に block/edge/local/context/site/source-map integrity を追加。obligation handoff/`VcIr` は除外し、VC/proof/kernel/artifact/MVM/extraction credit なし。 |

最初の general snapshot registry/schema change は、それを必要とする最初の real
non-Task-180 baseline と同じ task/commit に含める。Empty infrastructure commit と
synthetic Core/CFG snapshot は禁止する。

## Gates And Cross-Crate Boundary

- Gate A1/MC-G004 は Core 40 を block し続ける。
- Gate S1 は Core 41 と、missing scheme/theorem role を実際に必要とする
  algorithm template slice を block する。
- MC-G005/CORE-AUDIT-G006 により public diagnostic code は外部 owner のまま。
- Task-32 docs commit 後、Core Tasks 31/32 が完了するため VC Task 30 は
  dependency-authorized になる。ただし docs-only mapping/decomposition だけで、
  unimplemented Core task/gate を保持し、VC generation/acceptance は行わない。
- Steps 6/7 は deferred のままである。

## Disagreement Classification

| Protocol class | Task-32 inventory / disposition |
|---|---|
| `spec_gap` | 新しい blocking Task-32 spec gap なし。English chapters は family/negative boundary を定義する。既存 MC-G005 は別の nonblocking public-code gap。 |
| `test_gap` | 全 non-Task-180 Core snapshot、source-derived algorithm/CFG baseline、real-source negative diagnostic consumer、corruption matrix が未実装。Core 33-53 と prepared consumer へ割当。 |
| `design_drift` | Umbrella ownership を本 graph で解消。Task 247 に algorithm producer number がない点は Task 32 が許可した joint contract で解消し、checker ID や general unnamed gate は捏造しない。 |
| `source_drift` | Task 180 以外の real source-to-checker-to-Core と全 source-derived CFG route が未実装。descendant graph へ割当。 |
| `source_undocumented_behavior` | 新規なし。Current APIs/Task-180 adapter は spec より narrow で boundary を文書化済み。 |
| `test_expectation_drift` | Parser Task 47 の omitted-`reconsider` drift は維持し、Task 32 では修復しない。 |
| `boundary_violation` | Current violation なし。Core 34 conversion と Core 37 source `reconsider` を分離し、concrete call/result substitution を Core 46/52 から除外した。Core raw syntax、identity/evidence fabrication、VC substitution は violation になる。 |
| `repo_metadata_conflict` | なし。Stale Core Task-30 ledger hash は別 `CORE-LEDGER-001` で修復済みで、Task 32 は metadata を自動修復しない。 |

## Task-32 Exit Criteria

Task 32 は次をすべて満たしたときだけ complete である。

- 全 remaining Core/CFG family が one bounded task と one prepared consumer、
  または preserved explicit gate を持つ。
- EN canonical/JA companion の Core/checker/mizar-test/VC/top-level ownership が一致する。
- trace は deferred ownership/reason wording だけを変え、status/coverage/test list
  は不変。
- `spec_coverage_audit.md` は ownership だけを更新し coverage credit は増やさない。
- source/fixture/expectation/runner count/test list/production layout/hash は不変。
- spec、test sufficiency、implementation scope、source/docs consistency の
  review-only review が finding なし。
- focused metadata/lint と full baseline verification が成功する。
- 1 docs/traceability Task-32 commit として commit する。

Commit 後は fresh inventory で最小の dependency-ready task を選ぶ。Checker 248
は Core 33 の最初の upstream producer であり、VC Task 30 も docs-only
decomposition として dependency-authorized になる。Canonical top-level sequence
に従い、missing prerequisite を飛び越えない。

## VC Task 30 accepted downstream mapping

VC Task 30 は完了し、downstream graph を
[source_vc_decomposition.md](../../mizar-vc/ja/source_vc_decomposition.md) で
canonicalize する。VC 31 は Core 31 の exact Task-180 structure だけを消費する。
VC 32-41 は本文書にある applicable exact Core 33-40 row に依存し、direct VC 41 は
Core 34-38 だけを消費する。missing scheme/theorem role はその外で Core 41/S1 の背後に
残る。VC 40 は real registration/cluster/reduction VC を decorate する前に complete 済み
VC 37/39 output も必要とする。VC 42-55 は exact Core 42-53 algorithm/CFG row に依存する。
VC 53 はさらに、current canonical authority が authenticated termination-evidence producer、
reference schema/identity、authentication gate、test を命名していないため blocked のまま。
concrete call/result substitution は VC-owned のまま。Core 40/A1 は blocked のままで、
どの VC descendant も未実装 Core row、
status、trace、formula、context を available と扱えない。この backlink は ownership
だけを追加し、Core/VC coverage を追加しない。
