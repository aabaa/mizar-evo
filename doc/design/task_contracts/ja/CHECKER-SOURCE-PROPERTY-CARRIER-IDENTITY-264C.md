# Task CHECKER-SOURCE-PROPERTY-CARRIER-IDENTITY-264C: Task264 carrier identity transport

> canonical English:
> [EN contract](../en/CHECKER-SOURCE-PROPERTY-CARRIER-IDENTITY-264C.md)。

Status: 本recordを含むexact task-only commit上でcomplete。Implementation/post-source
review/final read-only quality scoringにblocking/high/medium findingは残っていない。本taskは
representation-only checker prerequisiteで、language behavior、protected test intent、
diagnostic、semantics、obligation、trace、metadata、coverage creditを変更しない。

## Identity、authority、classification

- Task: `CHECKER-SOURCE-PROPERTY-CARRIER-IDENTITY-264C`。
- Primary owner: `mizar-checker::source_property_implementation`。
- Owning plan: [`mizar-checker` crate plan](../../mizar-checker/ja/00.crate_plan.md)。
- Lower authority: existing Task264 means/equals `.miz`、trace、expectation、completed checker route。
- Consumer: future Task264 same-source Core33 carrier context、その後のCore34 selector/type owner。
- Coverage: semantic/execution/trace/metadata/coverage credit zero。

Stable durable-owner linkはchecker designの
[carrier identity transport](../../mizar-checker/ja/source_property_implementation.md#carrier-identity-transport)、
[exact handoff/API](../../mizar-checker/ja/source_property_implementation.md#public-contract--rows)、
[consumer/test boundary](../../mizar-checker/ja/source_property_implementation.md#tests--impact--deferrals)
である。本contractはtask freeze/completion evidenceを所有し、これらのsectionはdurable
module API/invariant/test designを所有する。

Authority orderはspec、existing `.miz`、trace、expectation、design、source。Chapter 7
§§7.4.1/7.8.2はproperty implementationがdeclared carrier domain上のexisting
structure propertyをtargetとすることを要求し、Chapter 5はstructureとmemberを区別する。
Exact Task264 resolver transactionはlocal public/exported `Task264Carrier`、`carrier`、
`marker`を既にauthenticateしているため、そのtuple保持はderived transportでありlanguage/
test intent変更ではない。

`spec_gap`なし。Carrier/field identity未保持は`source_drift`、public/replay assertion
不足はbounded `test_gap`、dependent Core `design_drift`はTask36P264が所有する。
その他のdrift/boundary/metadata conflictはない。

## Frozen API、validation、boundary

Private fieldを持つpublic `SourcePropertyCarrierIdentity`を1個だけ追加し、structure、
field、propertyそれぞれについて`SymbolId`、`DefinitionId`、
`SourceContributionId`、`SemanticOrigin`の12 getterを公開する。
`Debug + Clone + PartialEq + Eq`をderiveし、existing public handoffのderive contractを
維持する。
`SourcePropertyImplementationHandoff::carrier_identity()`を追加し、既存のprivate
marker-only snapshotをこの値で置換する。Producer signature/input/projection/table/error/
Typed/Resolved destination/installerは不変。Role enum、second handoff slot、generic
abstraction、Core/semantic resultは追加しない。Exact Rust signatureはcanonical EN contract
が所有し、このcanonical ownershipを本JA companionのlogical parityとして明示的に採択する。

Identityはexisting `SymbolEnv`からだけderiveする。

- structure: definition 0、`Structure`、`Task264Carrier`、contribution 0、
  `13..101/[4,0,11,0]`。
- field: definition 1、`Selector`、`carrier`、contribution 0、
  `45..66/[4,0,11,0,18,0]`。
- property: definition 2、`Selector`、`marker`、contribution 0、
  `71..94/[4,0,11,0,19,1]`。

全symbolはnormal/local/public/exported/conflict-freeで、sole local-source
contributionのexact `3/3` effectsに属する。Parameter application headはretained
structure、target row 0はretained propertyと一致し、不一致はexisting
`InvalidResolverTarget { index: 0 }`。Complete resolver authenticationはbuild時だけ行う。
Final replayには`SymbolEnv`がないため、immutable snapshotのexact role id/normal origin/
shared contribution/module/distinct whole symbol/property-target self-consistencyを検証し、
retained source-type handoffがindependent reauthenticateするのはstructure parameter headだけで
ある。Field/propertyにindependent lower oracleがあるとは主張しない。Private fieldはexternal
forgeryを防ぎ、name/range/numeric id/map order/Task263/別sourceからidentityを再構築しない。

Debugは`source-property-implementation-debug-v2`となり、existing module lineと7
fingerprint lineの直後かつ`implementation#0`前に、canonical EN contractのexact grammarでidentity row 0/1/2を
structure/field/property順に出す。Field間はASCII space 1個、decimalにpaddingなし、pathは
Rust `Debug` list punctuation、各rowはLF終端、extra blank lineなしである。
Existing checker 5 test/runner 4 testを増やさず拡張し、12 getter、exact debug、3 role
construction mutation、全snapshot invariant replay mutation、property-target/structure-head
linkをcoverする。Unique same-module field-symbol replacementにはindependent replay oracleが
ないため、build-time resolver authenticationとprivate-field immutabilityが保護し、replay
による過大な認証claimはしない。

Spec、`.miz`、sidecar、trace、runner selection、diagnostic、obligation、semantics、property
value、proof/discharge、Core item/kind/definition、Typed/Resolved install shape、Task277Bを
変更しない。Task263 handoffはdifferent-sourceで入力ではない。Coverage creditはzero。

## Baseline、review、exit

Checker source baselineは`2460 / 89030`、SHA
`82a9c45e8a7201e85afe961aefde74f35dd49dac359d4be51062d507294b08ee`、checker
support testは`2004 / 71309`、SHA
`7c178ca3911c2c16b8ebf44f28a1128a562f68e2b3769840ee40f97c85bf755e`、runner
Task264 testは`236 / 12697`、SHA
`602211a63cf51972f46141f4ac8c8b460aa056f19f06a325795eec5f9c6c0880`。
Contract treeは`116/116 -> 117/117`、protected trace SHAは
`17bba212e5216256b5883ce641048de263cd045a12adf060ed354973a6ae6728`。

Artifact ownershipはchecker source/existing unit test/private runner Task264 assertion file、
paired property design/plan/TODO/source API audit/bilingual audit、本contract、central coverage
audit、paired checker module-boundary audit、paired mizar-test runner-boundary inventoryである。
Runner producerは不変。Protected means/equals source hashは
`cc90659f10cae4ef68890624df9b8b9d3f0e830dae5e20cc195dc8b263c5fa2b` /
`175135aaf40b9eab1a28e73ca1aae9f250e66278410d50575cdd279f6d7a2784`、expectation hashは
`bced77302602f43f3237424aa2963e5522c1458e879e606c68d1a516cd737c3a` /
`c491d7ea65e1c096d869af4666a06a053a5a0b213d9e79483d13e5ec91b75b6e`で不変。

Pre-source spec/API/bilingual/boundary review、post-source test/implementation/default-deny/
source-doc/API/final-quality reviewを行う。Focused Task264、checker/mizar-test package、
metadata/lint、fmt、warnings-denied Clippy、all-feature workspace、hard gate `9/9`、quality
`>=90/100`、task-only commit、clean postcommit、same-source Core33 fresh inventoryでexitする。

## Completion evidence

Final Rust measurementはchecker source `2625 / 94187`、SHA
`0a0f9d887aa6cda7ef11c18936cd27503326c587ad7c8bc3193565828e91fe58`、checker
Task264 support test `2152 / 77946`、SHA
`7a7ddb8730f0a39d5739a11a3a2ca094e3997a446fb5b8556b168e34cc48b54d`、private
runner Task264 test `258 / 13953`、SHA
`b5d86410fca9546872fb25ce644381284c97ad58f2e7f703319af99b14cd149a`。
Contract treeはexact `117/117`、protected source/expectation/trace/stash hashは不変。

Independent spec/API、bilingual/boundary、test sufficiency、implementation/default-deny、
source-doc/API reviewはderive/debug/replay scope、public/module inventory、isolated
foreign-module replay test repair後no findings。Focused checker Task264 `5/5`、private runner
Task264 `4/4`、checker `580/580` + lint `16/16`、mizar-test `642/642`、layout `3/3`、
lint `15/15`、metadata `137/137`、enum `2/2`、snapshot `21/21`。Workspace all-feature
test/doctest、warnings-denied all-target/all-feature Clippy、fmt、offline metadata、recursive
contract link、diff checkはpass。Default stackのfirst checker-package runはunrelated existing
deep testでstack overflowしたが、同じcomplete suiteは`RUST_MIN_STACK=16777216`でassertion
failureなしにpassした。

First final-quality passでstale exact API/debug owner blockとmissing stable owner-section linkを
検出した。EN/JA owner document/contractをrepairした後のbilingual/boundary passでは、paired
mizar-test runner-boundary inventoryのassertion leaf countがstale `236` lineであることを
検出し、measured `258` lineへ同期した。両lint surfaceをrerunし、finding-specific
bilingual/final-quality reviewをrepeatした。Final reviewは**NO FINDINGS**、hard gate
`9/9` PASS、valid uncapped quality `100/100`である。

本recordを含むexact 20-path payloadがtask-only commitである。Clean postcommit proofと
fresh same-source Core33 inventoryはread-only successor evidenceであり、このcompleted task
recordをamendしない。
