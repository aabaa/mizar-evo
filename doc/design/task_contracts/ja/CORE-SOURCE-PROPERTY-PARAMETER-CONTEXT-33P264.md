# Task CORE-SOURCE-PROPERTY-PARAMETER-CONTEXT-33P264: Task264 parameter Core context

> canonical English:
> [../en/CORE-SOURCE-PROPERTY-PARAMETER-CONTEXT-33P264.md](../en/CORE-SOURCE-PROPERTY-PARAMETER-CONTEXT-33P264.md)。

Status: task-only commit上でcomplete。Independent review/verification/hard gate `9/9`、
valid uncapped quality `100/100`はcomplete。Representation-only、zero-semantic、
zero-credit Core33 prerequisiteで、language behavior、protected test
intent、diagnostic、obligation、trace、metadata、coverage creditを変更しない。

## Identity、authority、classification

| Field | Frozen value |
|---|---|
| Task | `CORE-SOURCE-PROPERTY-PARAMETER-CONTEXT-33P264` |
| Primary owner | `mizar-core::elaborator` Core33 |
| Required predecessor | Task33LB、Task264 checker transaction、Task33I264、Task34I264/34D264 |
| Input | Complete `SourcePropertySelectorTypeContextHandoff`とexact Task264 `SourceBindingContextHandoff` |
| Consumer | Separately reviewed Task264 Core35 means/equals body input |
| Owning plan | [`mizar-core` Task Index](../../mizar-core/ja/00.crate_plan.md#task-index) |
| Coverage | semantic/execution/trace/metadata/coverage creditすべてzero |

Authorityは`doc/spec/en/`、existing `.miz`、trace、expectation、design、sourceの順。
Chapters 4/5/7とTask264 `.miz` 2件が`Task264Carrier`上definition parameter `M` 1件を
固定する。Checkerはparameter 0/binding 0/target subject 0/exact source context/domain
application/rootをauthenticate済みで、Task33LBがbinding-to-free-term-Core-variable
allocationを既にownする。

`spec_gap`なし。Missing branded property-parameter/Core-variable joinはbounded
`design_drift`/`source_drift`、private assertion不足はbounded `test_gap`。他gap classなし。

## Frozen API、representation、validation

Exact public APIはEN contractのRust blockをcanonicalとする。Private-field
`SourcePropertyParameterCoreVariableAssociation`はparameter/binding/core-var getter、
`SourcePropertyParameterCoreContextHandoff`はsource/module/context、complete selector/
source/33LB dependency、association、debug getterだけを持つ。Non-exhaustive errorは
`EnvironmentMismatch`、`InvalidSelectorContext`、`InvalidSourceContext`、
`InvalidBindingContext`、`InvalidAssociation`。Producerはcomplete selector contextと
source contextだけをconsumeする。

Constructionはselector carrier contextのexact cloneとsource contextのbinding envを
existing `SourceBindingCoreContextProducer`へ渡す。Replayはsame deterministic rebuildと
complete equalityを要求し、別allocation algorithmやnumeric ID aloneを使わない。

Means/equals共通exact associationは`parameter/binding/core-var=0/0/0`。Carrier contextに
existing variableがないためTask33LB max-plus-one allocationは`CoreVarId(0)`となる。
Variableはfree/term/`definition-parameter`、declaration range `125..126`、checker
provenance `source-binding-core-variable-v1.binding.0`だけ、type fact empty、binder frame
なし。Debugはexact
`source-property-parameter-core-context-v1|module=<package>.<path>|carrier-item=<id>|bindings=1|parameter=0:0:0`
でfinal LFなし。

Validation precedenceはenvironment、selector context、source context、derived binding
context、association、complete postvalidation。Same source/module、selector domain
binding/application/root `0/0/0`、carrier item 0、return member/root `1/2`、checker
source-context fingerprint、exact one item/declaration/binding/two contexts、normal `M`、
scope `[4]`/context1、parameter/target/domain binding join、retained carrier itemと追加variable
だけをrequireする。Mixed/foreign/recovered/incomplete/stale/extra/corrupt inputはpublish前に
fail closed。

## Scope、test、forbidden behavior

Rust editは`crates/mizar-core/src/elaborator.rs`とexisting private Task264 test leafだけ。
Means/equals deterministic exact testとcross-profile/foreign/default-deny testの2件を追加し、
Core unitは164、mizar-testは`646 -> 648`をprojectする。Existing `.miz`/expectation/trace/
snapshot/selection/production routeは不変。

Generic adapter、`CoreContextInput` field、Typed/Resolved/CoreIr slot、property/field item、
normalized type/guard/fact、binder frame、term/formula/selector lowering、`it`、definition body、
correctness/coherence、diagnostic/obligation/route/creditを追加しない。`doc/spec`、checker、
`core_ir.rs`、VC、protected artifactを編集しない。Task264Dはlater equals-only inputで、
means current-resultはseparate prerequisite。

## Artifact、baseline、review、exit

Durable ownerはpaired contract、paired Core plan/elaborator/decomposition/TODO/source-spec/
bilingual/boundary、paired mizar-test harness/bilingual/boundary、central audit。Stable linkは
[elaborator Task33P264](../../mizar-core/ja/elaborator.md#task-33p264-task264-parameter-core-context)、
[decomposition](../../mizar-core/ja/source_family_decomposition.md#task-33p264-task264-parameter-core-context)、
[private probe](../../mizar-test/ja/harness.md#core-task-33p264-private-task264-parameter-probe)。

Clean baselineはHEAD `2a06adfb8172b497d28b75f0a03cbdb593831b0f`。
`elaborator.rs` `23785 / 893870`、SHA-256
`f65c64b2a59ab68689b6a53e0c334b231545cfcfe33f6b5e178ce18ebe3d7928`、private leaf
`1143 / 49788`、SHA-256
`c5d746b3d16bca7088aedfc3a31a286a84737790f8936d1ff48488a95e0b196d`、central audit
`7444 / 562566`、SHA-256
`9da542688b7e83bdaa4204576ee061aef36080a4c2f49f52163b6bfe40ad9de5`。
Contract tree `122/122 -> 123/123`、Core/checker/mizar-test unit count開始値
`164/582/646`。Protected hash/stash `f65cf4a13752ec380710814a9ac6392ccb9d75d4`不変。

Implemented source実測は`elaborator.rs` `24256 / 912957`、SHA-256
`d48f63f0a8427bc6e9c4290affcc8ae5e909c8b2af5391398256894d1334ff11`、private
Task264 leaf `1377 / 58827`、SHA-256
`b7b443a4638ce10216b6ffc1f72c0324dd8806ab5a33f2d5c8b8e77ac788afc4`。
Contract tree実測は`123/123`。Final central auditは`7459 / 563428`、SHA-256
`085f0836ac2a0047eb59d058aa93310e3c7f750fe090fbc230b32ca01e87b9cf`。

Pre/post-source independent reviewをfindingなしまでrepeatし、focused/package/lint/
metadata/fmt/offline metadata/Clippy/workspace all-feature test/doctestを実行する。Exitは
hard gate `9/9`、quality `>=90/100`、task-only commit、clean postcommit proof、next Core35
fresh inventory。Completion後equalsはparameter context+Task264Dをseparate taskでconsume可。
Meansはexplicit `it` representationが必要、Core36/Task277Bはnot-ready/zero-credit。

## Completion evidence

Pre-source specification/API/bilingual/boundary reviewはclean-HEAD baselineとdoc
worktreeの区別、JA owning-plan backlink修復後にblocking/high/medium findingなし。
Post-source test sufficiency/implementation/default-deny reviewはfindingなし。
Source/docs/API/bilingual/boundary reviewのmedium lifecycle/measurement finding 2件は、
EN/JA status、closed-gap record、public inventory、boundary measurement、post-source
artifact evidence同期後のfinding-specific re-reviewで全解消した。

Focused Task33P264 `2/2`、complete Task264 private family `10/10`、Core unit `164`/
determinism `2`/lint `12`、mizar-test unit `648`/layout `3`/lint `15`/metadata `137`/
public-enum `2`/snapshot `21`がpass。Fmt、offline metadata、`git diff --check`、
warnings-denied all-target/all-feature Clippy、enlarged-stack all-feature workspace
test/doctestがpass。Coverage planは`430` cases/`396` requirements/errors 0/warnings 23
で不変。

Protected means/equals `.miz` hashは`cc90659f...`/`175135aa...`、expectationは
`bced7730...`/`c491d7ea...`、traceは`17bba212...`、stashは`f65cf4a...`で不変。
Specification/expectation/trace/snapshot/diagnostic/obligation/route/semantic credit
変更なし。Parent/independent hard gate `9/9` pass。Independent final read-only auditは
findingなし、score capなし、valid `100/100` (`20/20/15/15/10/10/5/5`)。Exact staging/
task-only commitをprepareし、clean postcommit proof/fresh successor inventoryは
self-referential task commitでなくoperational handoffへ記録する。
