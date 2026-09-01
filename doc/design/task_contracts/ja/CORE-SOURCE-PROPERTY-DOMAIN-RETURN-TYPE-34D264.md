# Task CORE-SOURCE-PROPERTY-DOMAIN-RETURN-TYPE-34D264: Task264 domain/return type input

> canonical English:
> [EN contract](../en/CORE-SOURCE-PROPERTY-DOMAIN-RETURN-TYPE-34D264.md)。

Status: task commit時点でcomplete。Representation-only、zero-semantic、
zero-creditのCore34 prerequisiteであり、language behaviorとprotected `.miz`/
expectation/trace test intentは変更しない。Derived Rust validation-test intentだけを
拡張し、diagnostic、obligation、metadata、coverage creditは変更しない。

## Identity、authority、readiness

- Task: `CORE-SOURCE-PROPERTY-DOMAIN-RETURN-TYPE-34D264`。
- Owner: `mizar-core::elaborator`、Core34。
- Predecessor: Task264C `3cb1b31c`、Task33I264 `0f61a860`、Task34I264
  `85648a07`、IR264 `e96e12d1`。
- Input: complete `SourcePropertySelectorTypeContextHandoff`。
- Consumer: separately reviewed Core35 Task264 term/formula body input、その後Core36。
- Plan: [mizar-core Task Index](../../mizar-core/ja/00.crate_plan.md#task-index)。
- Coverage: semantic/execution/trace/metadata/coverage credit zero。

Authorityはspec、existing `.miz`、trace、expectation、design、source。Chapter 5/7と
Task264 `.miz` 2件はparameter `M`のdeclared `Task264Carrier` domainとexisting
`marker -> set` returnを固定する。Checker handoffはparameter binding 0、written-type
application 0/root 0、property target 0、return member 1/root 2を認証済み。

`spec_gap`なし。Missing durable Core-facing domain relationはbounded
`design_drift`/`source_drift`で、existing private Task264 probe拡張がbounded
`test_gap`を閉じる。既存分解どおりsource-derived typeはCore34、term/formulaは
Core35、definitionはCore36がownerであり、domain/return typeをCore35とした旧derived
wordingはこの分解へrepairする。Language/test intent変更ではない。

## Frozen API、validation、scope

Canonical EN contractのexact APIをlogical parityとして採択する。追加するのはprivate-field
`SourcePropertyDomainTypeAssociation`と、その`binding()`、`application()`、
`root()`、`carrier_item()` accessor、およびexisting
`SourcePropertySelectorTypeContextHandoff::domain()`だけ。Handoffはprivate `domain`
fieldを持ち、complete carrier/source-type inputをby-value retainする。Existing
`SourcePropertySelectorTypeAssociation`はreturn-side associationのまま。Constructor、
table、mutator、duplicate return APIは追加せず、existing v1 debug bytesは不変。

Build/postvalidationはchecker parameter 0のbinding 0/written type application 0を
application 0/root expression 0へ辿り、property target 0のsubjectが同じparameter
binding 0であることをdomain/return publish前に要求する。Expressionはchecker identityが認証したsame-source
whole `Task264Carrier` symbol/contributionで、Core registryはexact symbolをretained
Task33I264 carrier item 0へmapしなければならない。Domain relationは両profileで
`binding/application/root/carrier-item = 0/0/0/0`。Return relationはexisting
`marker/member/root = marker/1/2`。Joinはtyped id/whole symbol identityを用い、
spelling/range/numeric id/FQN/debug/map order aloneを用いない。

Source editは`elaborator.rs`とexisting private Task264 assertion leafだけ。Existing
means/equals positive testへparameter→target subject edge、domain relation、registry join、retained return、replay、
unchanged debugを追加する。Existing cross-profile/foreign testがfail-closed transaction
proofのまま。`.miz`、expectation、trace、snapshot、selector、public test mutatorは不変。

`CoreTypePredicate`、normalized type string、guard、binder/type fact、field→member0、
item/dependency、term/formula/definition、correctness/coherence seed、diagnostic、obligation、
production route、coverageを追加しない。`it`/`M.carrier`/means/equals/correctness/
coherence/proof/acceptance/dischargeをlowerしない。Task263とTask248/33LBは非input。
Spec、existing `.miz`、expectation、trace、protected artifactは不変。

## Artifact、review、verification、handoff

Derived ownerはpaired contract、Core plan/elaborator/decomposition/source-spec/TODO/
ledger/bilingual/boundary、paired mizar-test harness records、central audit。
`core_ir.rs`、checker source/design、mizar-vc、language artifactはread-only。

Stable owner sectionは
[elaborator API/invariant](../../mizar-core/ja/elaborator.md#task-34d264-task264-domainreturn-type-input)、
[source/spec mapping](../../mizar-core/ja/source_spec_audit.md#core-34d264-task264-domainreturn-type-input-mapping)、
[private harness probe](../../mizar-test/ja/harness.md#core-task-34d264-private-task264-domainreturn-type-probe)。
Exact source-write setは`elaborator.rs`とexisting Task264 private assertion leaf。
Paired owner docs、crate-plan/TODO/ledger/decomposition/bilingual/boundary record、central
auditがderived doc-write set。`core_ir.rs`、checker/VC全file、`doc/spec`、`.miz`、
expectation、trace metadata、snapshot、protected artifactはexplicit no-impact/read-only。

Baseline HEAD `e96e12d1767ab1d6a85e881328d5965e1afa15d1`。`core_ir.rs`は
`4393 / 146011`、SHA
`4e614a6ee98d0ef6b93dcd5d708728e41b79f613b16880269550051450793fd1`、
`elaborator.rs`は`23685 / 890564`、SHA
`1d78d960032e2f4086f712d258a8ec247aa12daeff88f51c6afe8f4d880a7162`、
Task264 leafは`1022 / 44699`、SHA
`e584e3a36d8c8911d4e5f49209128cb35e81d0c93d254419476b93557a86fdca`、
central auditは`7413 / 560700`、SHA
`a085dc14b0479cfab399ce5b594134b812b094b71d0885f948aa1ec1bea0f40a`。
Contract treeは`120/120 -> 121/121`、Core/checker/mizar-test testは
`164/580/646`のまま。Protected stash
`f65cf4a13752ec380710814a9ac6392ccb9d75d4`は不変。

Post-source measured artifactは`elaborator.rs` `23785 / 893870`、SHA
`f65c64b2a59ab68689b6a53e0c334b231545cfcfe33f6b5e178ce18ebe3d7928`、
private Task264 leaf `1064 / 46418`、SHA
`b474a1bc55997d79fe1a0e83ee194c25c28ee06a0f86f6a0abaa8b9f8bcf5b4d`、
central audit `7432 / 561797`、SHA
`4f8b5be69030061211b6b6ea87a1febcda3f390c7aaf099eba46b2b960f3b197`。
Contract treeは`121/121`、test countは`164/580/646`のまま。Frozen write setに
対するsource/audit measurementはfinalで、review/gate resultはclosure時に別途記録する。

Pre-source specification/APIとbilingual/boundary reviewはblocking findingなしを要求。
Post-source test-sufficiency、implementation/default-deny、source/documentation/API、
bilingual/boundary、final quality reviewはblocking/high/medium findingなしを要求する。
Focused Task264、affected package/lint/metadata、fmt、offline metadata、warnings-denied
all-feature Clippy、enlarged-stack all-feature workspace test/doctestを実行する。
Hard gate `9/9`、quality `90/100`以上、task-only commit、clean postcommitでcloseする。

Completion後、smallest Core35 Task264 body inputをfresh inventoryできる。Parameter `M`と
current-definition-result `it`のexact Core variable representationは別reviewで決め、
Core36 definitionとcorrectness/coherenceはdeferする。Task277Bはnot-ready。

## Completion evidence

Pre-source specification/API・bilingual/boundary reviewは、target-subject edge、ordered
plan link、stable owner link、exact no-impact map、protected/derived test-intent wordingの
repair後blocking/high/medium findingなし。Post-source test-sufficiency、implementation/
default-deny、source/documentation/API、bilingual/boundary reviewも、active-stateと
measured-evidence wording同期後findingなし。

Focused Task264 selector/type `2/2`、Core unit `164`/determinism `2`/lint `12`、
mizar-test library `646`/layout `3`/lint `15`/metadata `137`/public-enum `2`/
snapshot `21`がpass。Fmt、offline metadata、`git diff --check`、warnings-denied
all-target/all-feature Clippy、enlarged-stack all-feature workspace test/doctestがpass。
Protected language/test artifact、checker、VC、`core_ir.rs`にdiffなし。Frozen
`core_ir.rs` hash/protected stashは不変。Parent hard gate `9/9` pass、independent final
resultは以下に記録する。

Initial final-quality auditはstale lifecycle evidenceだけをfindingとし、gate 5/scoreを
invalidに保留した。本paired completion recordとsynchronized active-owner statusがその
documentation-only findingをrepairした。Finding-specific read-only recheckはblocking/
high/medium findingなし、hard gate `9/9`、score capなしのvalid uncapped quality
`100/100`。Operational closureはtask-only commitとclean postcommit proofだけ。
