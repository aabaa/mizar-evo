# Task CORE-SOURCE-PROPERTY-SELECTOR-TYPE-CONTEXT-34I264: Task264 selector/type context

> canonical English:
> [EN contract](../en/CORE-SOURCE-PROPERTY-SELECTOR-TYPE-CONTEXT-34I264.md)。

Status: task commit時点でcomplete。Representation-only/zero-semantic/
zero-credit Core34 prerequisiteで、language behavior、test intent、diagnostic、
obligation、trace、metadata、coverage creditを変更しない。

## Identity、authority、readiness

- Task: `CORE-SOURCE-PROPERTY-SELECTOR-TYPE-CONTEXT-34I264`。
- Owner: `mizar-core::elaborator` Core34。
- Predecessor: Task264C `3cb1b31c`、Task33I264 `0f61a860`。
- Input: complete `SourcePropertyCarrierCoreContextHandoff`とexact
  `SourceTypeApplicationHandoff`。
- Consumer: separately reviewed CoreIR definition-owner prerequisite、その後Core35/36。
- Owning plan: [mizar-core Task Index](../../mizar-core/ja/00.crate_plan.md#task-index)。
- Coverage: semantic/execution/trace/metadata/coverage credit zero。

Authorityはspec、existing `.miz`、trace、expectation、design、source。Chapter5は
`carrier`をfield、`marker`をvirtual propertyとし、両方にselector/function identityと
type guardを与え、propertyをconstructor argumentから除外する。Chapter7はexisting
property targetとdeclared carrier domainを定める。Task264 means/equals sourcesが
same-source carrier、member 2件、parameter type、property return rowを固定する。

`spec_gap`なし。Task264C/33I264がidentityとcarrier/Core prerequisiteを提供済み。
Missing authenticated property-target/return-member associationはbounded
`design_drift`/`test_gap`。現行
`CoreDefinition.item`がforbidden selector aliasなしでpropertyをownできない別の
`design_drift`は残り、本taskはそれを解決したと主張しない。

## Frozen API、oracle、debug

Canonical EN contractのexact public APIをlogical parityとして採択する。追加するのは
private-field association/handoff、non-exhaustive error、unit producerだけ。Handoffは
complete carrier contextとsource typeをby-value retainし、property-selector association
exactly 1件をderiveする。Public table、
`CoreDefinition` owner、normalized Core typeはpublishしない。
Definition/contribution identityはretained checker owner内でtypedのままcomplete validateし、
associationから重複publishしないためresolver-environment typeはCore APIへ越えない。

Exact no-final-LF debugは次である。

```text
source-property-selector-type-context-v1|module=<package>.<path>|carrier-item=<id>|property=<fqn>:2:0:1:2
```

Validation順はenvironment→retained carrier context→source type→association。
Source/module一致、Task33I264 complete revalidation、checker owner fingerprintとのexact
debug replay一致を要求するが、debug textからrowをreconstructしない。

Type profileはapplication/expression/argument/definition-return/mode-RHS/member
`1/3/0/0/0/2`。Application 0はbinding/ordinal/root `0/0/0`。Expression 0は
whole `Task264Carrier`/contribution 0/range `130..144`、expression 1/2はbuiltin
`set` range `62..65`/`90..93`。Member 0/1はordinal `0/1`、range
`45..66`/`71..94`、root `1/2`。Means node座標は`63/64`、`55/54`、
`58/57`、member `56/59`、equalsは`45/46`、`37/36`、`40/39`、member
`38/41`。全rowはnormal/exact spelling。

Checker parameter 0はwritten type 0/carrier binding、target 0はretained `marker`
property identityをexplicitにreturn member 1へlinkする。Sole associationはそのtyped
target edgeをmember 1/root 2へ辿る。Lower handoffは`carrier` field→member 0 edgeを
authenticateしないため、本taskはfield associationをpublishしない。Carrier itemは
Task33I264 item。Name/FQN alone/range/numeric id/debug/map orderはjoinにしない。

## Test、scope、handoff

Source editは`elaborator.rs`とexisting private Task264 leafだけ。Exactly 2 testsを追加:
means/equals positive/replay/retention/exact property association/all exact type rows/debug、
same-environment means/equals
cross-profileは`InvalidSourceType`、foreign transactionは`EnvironmentMismatch`となる
fail-closed/isolation。Public test-only mutatorは追加せず、lower handoffのprivate immutable
constructionをrow mutation safetyとしてtrustする。Production selectionは不変。

Shell/member/property Core item、new item kind、selector alias、dependency、
`CoreTypePredicate`、normalized type、binder/fact/coercion/view/term/formula/definition/
diagnostic/obligation/installer/route/snapshot/coverageは追加しない。Body/correctness/
coherence/property value/proof/acceptance/dischargeはlowerしない。Task263非input。
Spec/`.miz`/expectation/trace/metadata/protected artifactは不変。

Derived ownerはpaired contract、Core plan/elaborator/source-spec/decomposition/TODO/
bilingual/module-boundary/ledger、mizar-test harness/bilingual/module-boundary、central audit。
Checker source/designと`core_ir.rs`/`core_ir.md`はread-only dependency。

Stable owner sectionは
[elaborator API/invariant](../../mizar-core/ja/elaborator.md#task-34i264-task264-selectortype-context)、
[source/spec disposition](../../mizar-core/ja/source_spec_audit.md#core-34-task264-selectortype-context-mapping)、
[private harness route](../../mizar-test/ja/harness.md#core-task-34i264-private-task264-selectortype-probe)。

Baseline HEAD `0f61a86062707e2e6ec4c7ed611c03cc7b91ee00`。`elaborator.rs`は
`23335 / 877140`、SHA
`83d5884a24013345cb486d76b1df448a52ff860c89d9c491b201e70ba2eedd29`。
Task264 leafは`699 / 31668`、SHA
`e45ac5bdbcbbab3fb0eeb4a281058dc2bad8330235db6590b432b76cb69c3d48`。
Central auditは`7373 / 558238`、SHA
`4888efe82c6900faaf132918c55f085449415e86a36be2476b39fed88e450240`。
Contract tree `118/118 -> 119/119`、Core test `163`、mizar-test `644 -> 646`。

Final measured artifactは`elaborator.rs` `23682 / 890332`、SHA
`a91e2456c279ffec9a2f67a18d9741f8885228c5a31d600e41622fcd1e03bfb9`、
private Task264 leaf `1017 / 44370`、SHA
`23ad08e3ac46e36ee34121cee49873b90f796fd50a18ad632aeca032598e79b6`、
central audit `7394 / 559472`、SHA
`84707772a1bee9acb4a8e713252db848f0aea2421c4f08937751c835d680f749`。
Contract tree `119/119`、Core/checker/mizar-test library test `163/580/646`。

Pre-source specification/API・bilingual/boundary reviewはrepair後no findings。
Post-source test-sufficiency、implementation/default-deny、source/documentation/API、
bilingual/boundary reviewも、unauthenticated field joinを除去した後はblocking findingなし。
Focused test `2/2`、Core/checker/mizar-test package testとlint/metadata、
`cargo fmt --all -- --check`、warnings-denied all-target/all-feature Clippy、offline metadata、
all-feature workspace test/doctestがpassした。Protected artifact、checker owner、
`core_ir.rs`、protected stashは不変。Independent final read-only reviewはhard gate
`9/9`、blocking/high/medium findingなし、valid uncapped quality `100/100`。Task-only
commitとclean postcommit proofでtaskを閉じる。次はsynthetic selector item/aliasなしの
smallest CoreIR structure-member definition-owner representationをfresh inventoryする。
その後だけCore35/36が`CoreDefinition` compatible ownerをclaimでき、Task277Bはnot-ready。
