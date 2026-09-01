# Task CORE-SOURCE-PROPERTY-CARRIER-ITEM-CONTEXT-33I264: Task264 carrier item context

> canonical English:
> [EN contract](../en/CORE-SOURCE-PROPERTY-CARRIER-ITEM-CONTEXT-33I264.md)。

Status: complete、task-only commit待ち。
Representation-only/zero-semantic/zero-credit Core33 prerequisiteで、language behavior、protected test intent、
diagnostic、obligation、trace、metadata、coverage creditを変更しない。

## Identity、authority、readiness

- Task: `CORE-SOURCE-PROPERTY-CARRIER-ITEM-CONTEXT-33I264`。
- Owner: `mizar-core::elaborator` Core Task33。
- Plan: [`mizar-core` crate plan](../../mizar-core/ja/00.crate_plan.md)。
- Predecessor: committed checker Task264C `3cb1b31c8727f244933c9750214101da333cf139`。
- Input: prepared `CoreContext`とexact Task264
  `SourcePropertyImplementationHandoff`。
- Consumer: Core34 authenticated selector/type owner、その後Core35/36。
- Coverage: semantic/execution/trace/metadata/coverage credit zero。

Stable owner linkはelaboratorの
[Task264 carrier item API/invariant](../../mizar-core/ja/elaborator.md#task-33i264-task264-carrier-item-context)、
[source/spec boundary](../../mizar-core/ja/source_spec_audit.md#task-33i264-sourcespecification-boundary)、
mizar-testの
[private probe boundary](../../mizar-test/ja/harness.md#core-task-33i264-private-task264-carrier-probe)。
本contractはorchestration/exact freeze/completion/handoffを、owner sectionはdurable
API/boundary/test designを所有する。

Authorityはspec、existing `.miz`、trace、expectation、design、source。Chapter5は
structureとfield/property memberを区別し、Chapter7 §§7.4.1/7.8.2はproperty
implementationをdeclared carrier domain上のexisting propertyへ結び、Chapters11/12は
whole-symbol identity/source orderを保持する。Committed Task264Cがsame-source
`Task264Carrier`/`carrier`/`marker`をauthenticate済み。Task36P264のno-shell-item
dispositionは不変。

`spec_gap`なし。Task264Cがlower `source_drift`を解消済み。Missing carrier/Core
association/private assertionはbounded `design_drift`/`test_gap`。Lower receiptがまだ
missingというcurrent-state claimは本task owner docsでrepairする`design_drift`で、completed
historical contractはrewriteしない。その他のdrift/boundary/metadata conflictなし。

## Frozen API、representation、debug

Canonical EN contractのexact Rust APIをlogical parityとして採択する。追加するのはprivate
fieldを持つ`SourcePropertyCarrierCoreContextHandoff`、non-exhaustive
`SourcePropertyCarrierCoreContextError`、unit producerだけ。Handoff getterは
`source_id()`、`module_id()`、`context()`、`checker_owner()`、`carrier_item()`、
`debug_text()`。Error precedenceは`EnvironmentMismatch`、`InvalidCheckerOwner`、
`InvalidCoreContext`、`InvalidItemAssociation`。Producerはprepared contextとcomplete
Task264 handoffをby-value consumeし、complete postvalidation後だけpublishする。

Public association row/tableは追加しない。Scalar `carrier_item()`と
`checker_owner().carrier_identity()`がexact associationを形成し、Core-owned typed source
definition IDを発明しない。Task263 two-row APIはdifferent-source precedentでinputでも
generalization対象でもない。

Exact no-final-LF debugは次である。

```text
source-property-carrier-core-item-context-v1|module=<package>.<path>|carrier=<whole-fqn>:0:0|item=<core-item-id>
```

## Exact oracle、test、deferral

Checker ownerはimplementation/parameter/target/definiens `1/1/1/1`、correctnessは
means `2`/equals `0`、required lower fingerprint nonempty、existing optional profile
separationを保つ。Carrier roleはdefinition `0/1/2`、contribution `0`、same moduleの
distinct whole symbol、normal local originで、structure `13..101/[4,0,11,0]`、field
`45..66/[4,0,11,0,18,0]`、property `71..94/[4,0,11,0,19,1]`。Target 0はretained
property identityと一致する。Coreはresolver spelling/source-type headを再authenticateせず、
Task264Cのprivate immutable construction/replay guaranteeをconsumeする。

Core contextはsource/module一致、exact whole structure `SymbolId` lookupで選ぶpublic valid
`Structure` item 1件のみ。Range `13..101`、dependency/diagnosticなし、sole provenance
`source-property-carrier-core-item-v1.structure`、empty dependency resolution、exact item
source map、pending `DefinitionalItem` boundary 1件、pending worklist 1件。Binder/checker
site/dependency summary/generated origin/other item/term/formula/definition/source-map domain/
external/missing dependencyはzero。Environment→checker→Core→association順にfail closedし、
sort/repair/inference/numeric reinterpretation/partial publicationをしない。

Source editは`elaborator.rs`とexisting private Task264 assertion leafだけ。Exactly 2 testsが
means/equals positive/replay/Core stateと、Core corruption/foreign environmentをcoverする。
Production runner routeは不変。Property shell/`carrier`/`marker` item、selector alias/
association/dependency、new item kindは作らない。Field/property owner、type/term/formula/
definiens/value/correctness/obligation/proof/acceptance/CoreDefinition/CFG/VC/diagnostic/
installation/MT10/Task277Bはdeferred。Spec、`.miz`、expectation、trace、metadata、selection、
coverageを変更しない。

## Artifact、baseline、review、handoff

Derived ownerはpaired contract、Core plan/elaborator/source-spec/decomposition/TODO/
bilingual/module-boundary/ledger、mizar-test harness/bilingual/module-boundary、central audit。
Checker docs/historical contractは不変。Auditはzero-credit mappingとCore34 follow-upだけを
更新し、chapter/trace/coverage statusは不変。

Baseline HEADは`3cb1b31c8727f244933c9750214101da333cf139`。Core sourceは
`22947 / 862541`、SHA `e9ea1d6eabb191d7d3b8c22fe1fc11626d2e0dab86690dee662f851bb487f85c`。
Task264 leafは`258 / 13953`、SHA
`b5d86410fca9546872fb25ce644381284c97ad58f2e7f703319af99b14cd149a`。
Contract tree `117/117 -> 118/118`、Core tests `163`、mizar-test `642 -> 644`。
Protected source/expect/trace/stashはTask264C値から不変。

Pre-source spec/API/bilingual/boundary、post-source test/implementation/source-doc/API/
bilingual/final-quality reviewをrepair後no findingsまで行う。Focused/package/lint/metadata/
fmt/Clippy/all-feature workspace/doctest、protected invariance、hard gate `9/9`、quality
`>=90/100`、task-only commit、clean postcommit、fresh Core34 inventoryでexitする。

次はretained `carrier`/`marker` identityとexisting Task264 type evidenceから、
`CoreDefinition.item`互換のauthenticated selector/type ownerを作るsmallest Core34 taskを
fresh inventoryする。Core34/Core35完了までCore36はblockedで、owner-model/semanticsを
本taskは選ばない。

## Completion evidence

Frozen API/exact default-deny validationを実装した`elaborator.rs`のfinal sizeは
`23335 / 877140`、SHA-256
`83d5884a24013345cb486d76b1df448a52ff860c89d9c491b201e70ba2eedd29`。
Existing private Task264 leafはexactly 2 testsを追加して`699 / 31668`、SHA-256
`e45ac5bdbcbbab3fb0eeb4a281058dc2bad8330235db6590b432b76cb69c3d48`。
Contract treeは`118/118`、Core library testは`163`のまま、mizar-testは`644`。

Independent pre-source spec/API/bilingual/boundary、post-source test-sufficiency/
implementation-default-deny、completion-state repair後のfinal source-doc-API/
bilingual-boundary reviewはblocking/high/medium findingなし。Focused Task264
`6/6`、checker Task264 `5/5`、Core/checker/mizar-test packageはlibrary
`163/580/644`とintegration/lint/metadata/doctestがpass。Fmt、offline Cargo
metadata、full all-target/all-feature warnings-denied Clippy、all-feature workspace/
doctestがpass。Metadata validationはerror 0、既存baseline warning 23。Protected
Task264 source/expect/trace hashとstash
`f65cf4a13752ec380710814a9ac6392ccb9d75d4`は不変。Final diff checkもpassし、spec、
`.miz`、expectation、trace、metadata、production route、semantics、coverage creditを
変更していない。

Final independent read-only reviewはblocking/high/medium findingなし、hard gate
`9/9`、valid uncapped quality `100/100`。
