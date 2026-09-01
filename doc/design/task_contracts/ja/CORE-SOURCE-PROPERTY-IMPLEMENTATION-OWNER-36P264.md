# Task CORE-SOURCE-PROPERTY-IMPLEMENTATION-OWNER-36P264: Task264 Core owner disposition

> canonical English:
> [EN contract](../en/CORE-SOURCE-PROPERTY-IMPLEMENTATION-OWNER-36P264.md)。

Status: exact docs-only task commit時点でcomplete。全independent reviewはrepair後
no findings、hard gate `9/9`、score capなしのvalid `99/100`。Commit自身のhashは
埋め込めないためfinal handoffで報告する。Language behavior、public API、Rust source、
test intent、coverage creditを変更しない。

## Identity、authority、classification

- Task: `CORE-SOURCE-PROPERTY-IMPLEMENTATION-OWNER-36P264`。
- Primary owner: `mizar-core` Core Task 36 definition lowering。
- Owning plan: [`mizar-core` crate plan](../../mizar-core/ja/00.crate_plan.md)。
- Lower authority: complete Checker Task264
  `SourcePropertyImplementationHandoff`。
- Result: property-implementation shell向けCore33 itemは作らない。
- Coverage: semantic/execution/trace/metadata/coverage credit zero。

Authority orderはspec、existing `.miz`、trace、expectation、design、source。
Chapter 7 §§7.4.1/7.8.2はproperty implementationをmode domain上のvirtual
structure property valueを供給するdefinitionとし、Chapter 5はconstructor fieldと
分離し、Chapter 16はexistence/uniqueness/coherenceをcorrectness-obligation
boundaryに置く。Accepted Core graphではchecker Tasks259--264のdefinition shell/body/
correctness referenceはCore33--35後のCore36 ownerである。

`spec_gap`はない。Prior Task264 Core33 item候補は、shellにsemantic item identityが
ない点を未分類だったbounded `design_drift`。Complete same-source Core33--35 routeの
欠落はlater executable task所有の`source_drift`/`test_gap`。Current
`source_undocumented_behavior`、`test_expectation_drift`、boundary repairはない。

## Exact inventoryとdecision

Means/equals source hashは
`cc90659f10cae4ef68890624df9b8b9d3f0e830dae5e20cc195dc8b263c5fa2b` /
`175135aaf40b9eab1a28e73ca1aae9f250e66278410d50575cdd279f6d7a2784`、
expectation hashは
`bced77302602f43f3237424aa2963e5522c1458e879e606c68d1a516cd737c3a` /
`c491d7ea65e1c096d869af4666a06a053a5a0b213d9e79483d13e5ec91b75b6e`。
各transactionはimplementation/parameter/target/definiens `1/1/1/1`、meansだけ
correctness 2件とpending initial obligation 2件、equalsは両方0。

Resolver property-implementation shellはcontext-onlyで、signature projection、
`SymbolId`、`DefinitionId`、contribution、semantic originを持たない。Targetはexisting
`marker` selectorで、parameter typeはuse siteの`Task264Carrier`を参照するが、どちらも
implementation shell identityではない。Checker handoffはCore item構築に必要なcarrier
structure-definition owner range/provenanceもpublishしない。

`CoreItem`はwhole `SymbolId`を要求し、`CoreItemKind`にproperty-implementation/
selector kindはない。従ってfail-closed decisionは次である。

1. Task264 shell向けCore item/association rowを作らない。
2. Target selectorを`Functor`/`Structure`等へaliasしない。
3. Speculative item kind/synthetic symbol/identity/dependency/source/provenanceを追加しない。
4. Task33I263をcurrent Task-specific Core33 item-association prerequisiteの最後とする。
5. Exact same-source Core33--35 contextとlower ownerがreadyになった後だけTask264を
   Core36でconsumeする。

Current `CoreDefinition`は`CoreItemId` owner必須なので、Core36はtarget selectorだけへ
bodyをassociateできない。Existing Task263 Core contextは別source/別structure向けで、
Task264 inputではない。Task264 lowering前に、separately reviewed lower checker taskが
Task264 transaction自身の`Task264Carrier` definition/member identityをsyntax-free
same-source handoffとしてpublishする必要がある。その後のCore33 carrier contextと
Core34 structure-member prerequisiteが`CoreDefinition.item` compatibleなauthenticated
selector-owner mappingをpublishするか、separately reviewed CoreIR representation taskが
owner modelを変更する。本contractはいずれも選択せず、property-implementation-shell
itemをauthorizeしない。このchain未完了時Core36はhard-blockedで、name/range/numeric
id/spelling/別source contextからownerを再構築しない。

Distinct owner prerequisite後だけ、future Core36はauthenticated target selector/domainへ
body/correctness referenceをassociateできる。Accepted property value/fact/axiom/proof/
dischargeはpublishしない。

## Artifact、verification、handoff

Changeはpaired contract、paired Core plan/TODO/ledger/source-family decomposition/
bilingual audit、central coverage auditだけ。Checker/mizar-test designは不変。
Contract tree `115/115 -> 116/116`。

Protected checker sourceは`2460 / 89030`、SHA
`82a9c45e8a7201e85afe961aefde74f35dd49dac359d4be51062d507294b08ee`、Core
`core_ir.rs`は`4016 / 132375`、SHA
`4458bc2353c437d4427b39f96e0041bf1c321e19cff0ec4565c3f50084f83c4c`。
Trace hashは`17bba212e5216256b5883ce641048de263cd045a12adf060ed354973a6ae6728`、stashは
`f65cf4a13752ec380710814a9ac6392ccb9d75d4`。

Task Index ownership、`CoreDefinition.item` prerequisite、different-source successor errorを
repair後、全independent spec/test/boundary/source-doc/bilingual reviewはno findings。
Core lint `12/12`、mizar-test lint `15/15`（recursive contract/linkを含む）、`git diff
--check`はpass。Docs-onlyでsource/test behavior変更がないためbroad Rust suiteは再実行
しない。Final read-only qualityはno findings、hard gate `9/9`、score capなしのvalid
`99/100`。Exact staging/docs-only commit/clean postcommit/fresh inventoryだけがtransactional
exit stepとして残る。

Central auditはCore33/Core36 follow-up owner訂正だけを記録し、spec/test/trace/
metadata/runner/coverage stateを変えない。Commit後はexisting Task264 sourceから
`Task264Carrier`と`carrier`/`marker` member identityをsemantics/test intent変更なしで
publishするsmallest checker-owned lower `source_drift` prerequisiteをfresh inventoryする。
Separate commit後だけTask264 same-source Core33 carrier contextとCore34 selector/type-owner
prerequisiteへ進む。Task264はchain完了までCore36でhard-blocked、Task263はprotected
different-source precedent、Task277Bはnot-ready/zero-credit。
