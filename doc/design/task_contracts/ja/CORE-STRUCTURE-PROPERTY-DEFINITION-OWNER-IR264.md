# Task CORE-STRUCTURE-PROPERTY-DEFINITION-OWNER-IR264: authenticated structure-property definition owner

> canonical English:
> [EN contract](../en/CORE-STRUCTURE-PROPERTY-DEFINITION-OWNER-IR264.md)。

Status: task-only commit時点でcomplete。Representation/validation-only、
zero-semantic/zero-credit CoreIR prerequisiteであり、language behavior、`.miz`/expectation/
trace test intent、diagnostic、proof/acceptance policy、metadata、coverage creditを変更しない。
Selected owner representationのderived Rust validation-test intentは意図的に追加する。

## Identity、authority、readiness

- Task: `CORE-STRUCTURE-PROPERTY-DEFINITION-OWNER-IR264`。
- Owner: `mizar-core::core_ir`と`mizar-core::elaborator`のauthenticated Task34I264 adapter。
- Predecessor: Task264C `3cb1b31c`、Task33I264 `0f61a860`、Task34I264 `85648a07`。
- Consumer: later Core35/36 Task264 property-definition task。
- Owning plan: [mizar-core Task Index](../../mizar-core/ja/00.crate_plan.md#task-index)。
- Coverage: semantic/execution/trace/metadata/coverage credit zero。

Authorityはspec、existing `.miz`、trace、expectation、design、source。Chapter5は
structure propertyをdeclared carrier/type guardを持つfunction symbolとしconstructor
argumentから除外し、Chapter7はexisting property target/carrier domainを定めるが、
CoreIR owner表現は規定しない。`spec_gap`/source contradictionなし。

Derived `design_drift`は、`CoreDefinition.item`がmodule-level `CoreItem`しか指せず、
Task34I264は`Task264Carrier` structure item配下のexisting `marker` property selectorだけを
authenticateし、`carrier` field edgeをauthenticateしない点である。複数のderived designが
可能なため、autonomous-design ruleにより、ordinary item ownerを保持しauthenticated adapter
だけがproperty formをmintできるsmallest fail-closed designを本contractで選択する。
Missing Rust matrixはbounded `test_gap`で、新しい`.miz` intentなしに閉じる。

## Frozen API、validation、compatibility

Canonical EN contractのexact APIをlogical parityとして採択する。Public private-field
`CoreDefinitionOwner`は`for_item`、`anchor_item`、`item`、`property_symbol`だけを公開する。
Property initializerは`core_ir.rs` privateでpublic/crate-private generic member/field constructorは
存在しない。Property formはanchor itemとauthenticated source/module/property symbolをprivateに
保持する。`CoreDefinition.item`は`pub owner: CoreDefinitionOwner`へ置換し、他fieldは不変。

`core_ir.rs`内で直接実装する
`SourcePropertySelectorTypeContextHandoff::definition_owner()`だけがTask34I264の
`source_id`、`module_id`、`carrier_item`、sole validated `marker` associationをcopyして
property ownerを返す。Owner private fieldをownするmodule内のinherent implementationなので
sole non-test initializerとなり、他の`mizar-core` moduleもproperty formをmintできない。
Caller値からreconstructせず、public generic member/field constructor/mutatorは追加しない。

`CoreIrError::InvalidDefinitionOwner { definition, reason }`を追加し、exact reasonは
`property-anchor-not-valid-structure`、`property-symbol-mismatch`、
`property-environment-mismatch`、`property-symbol-aliases-anchor`。Invalid anchor indexはexisting
`InvalidReference { table: "item", ... }`のまま。

Definition validationはsource/source-map→anchor index→propertyならValid Structure→
definition symbol一致→owner/CoreIr source/moduleとanchor/property module一致→propertyとanchor
symbol非一致→binder/body/correctness/generated dependencyの順。First failure wins、
`CoreIr::try_new`はatomic。Name/FQN alone/range/dense id/source order/debug textはjoinにしない。

Existing Step4 `DefinitionSeed.owner`、item-keyed `definition_map`、obligation/proof/generated
owner、definition id/source-map/unfolding/VC/dischargeは不変。Direct `CoreDefinition` construction
4箇所だけを`for_item`へmechanical migrationする。

`core-ir-debug-v1`は維持する。`CoreDefinitionOwner`/`CoreDefinition` manual `Debug`でordinary
item rowをlegacy `item: CoreItemId(...)`を含めbyte-identicalに保つ。Property rowのexact grammarは
EN contract記載どおり、`owner: StructureProperty { anchor_item: <CoreItemId>, source_id:
<SourceId>, module_id: <ModuleId>, property_symbol: <SymbolId> }`の後にexisting fieldをcurrent
order/Debug表現で並べる。Ordinary owner単体は`Item(<CoreItemId>)`。Version bump、snapshot/
expectation editなし。

## Test、scope、artifacts、handoff

CoreIR unit test exactly 1件を追加し、extra Core itemなしのvalid/replay property owner、invalid
anchor、non-Structure/non-Valid anchor、symbol mismatch、foreign source/module、anchor alias、
deterministic debug、atomic rejectionをcoverする。Existing Step4 positiveへordinary item owner/
item-keyed map assertion、existing Task264 means/equals positiveへcarrier item 0、`item()==None`、
authenticated `marker`だけのowner assertionを追加する。Existing cross-profile/foreign testが
adapter negative oracle。Public test-only constructorなし。

CoreIR unit testはowner validation専用にexisting trivial fixture term bodyを持つnon-production
fixture definition 1件をinsertできるが、semantic/coverage creditを与えない。それ以外では
selector/member/property Core item、new item kind/alias、field association、normalized type、
binder/term/formula/source-derived/property definition body/correctness/coherence obligation/diagnostic/production route/
snapshot/artifact/VC/proof/acceptance/coverageは追加しない。Core35/36、`DefinitionSeed`、
`definition_map`、downstream owner typeを拡張しない。Task263非input。Spec/`.miz`/existing
expectation/snapshot/trace/metadata/protected artifactは不変。

Source scopeは`crates/mizar-core/src/core_ir.rs`、`crates/mizar-core/src/elaborator.rs`、
`crates/mizar-test/src/runner/tests/type_elaboration/source_property_implementation.rs`、
`crates/mizar-vc/src/generator/task180.rs`のmechanical item-owner construction。
Doc ownerはpaired contract、mizar-core plan/CoreIR/elaborator/source-spec/decomposition/TODO/
ledger/bilingual/boundary、`doc/design/architecture/en/06.elaboration_and_core_ir.md`とexact
JA companion `doc/design/architecture/ja/06.elaboration_and_core_ir.md`、mizar-test
harness/bilingual/boundary、central audit。VC editはexisting-item-owner mechanical migrationで
VC-owned API/invariant/test intent/docを変更しないためmizar-vc owner docは変更しない。
Checker/kernel/artifact/Cargo変更なし。

Completion時に次のcurrent-state claimをEN/JAでreplace-in-placeする。Core `elaborator.md`と
mizar-test `harness.md`の「definition ownerなし」は「authenticated owner valueのみ、
`CoreDefinition` row/body/semantic publicationなし」へ狭める。Core `source_spec_audit.md`、
`source_family_decomposition.md`、`todo.md`、`task_ledger.md`はpending owner prerequisiteを
completed zero-credit owner valueへ進め、Core35/36 deferralを維持する。Core bilingual/
module-boundary auditはpublic owner/final inventoryを記録し、central auditはtrace/coverage
status不変のままfollow-up ownerを進める。

Stable owner sectionは
[CoreIR structure-property owners](../../mizar-core/ja/core_ir.md#structure-property-definition-owners)、
[Task264 authenticated owner factory](../../mizar-core/ja/elaborator.md#task-ir264-authenticated-property-definition-owner)、
[Task264 private probe](../../mizar-test/ja/harness.md#core-task-34i264-private-task264-selectortype-probe)。

Baseline HEAD `85648a076ae40538dafabea93faaf63f7b516978`。`core_ir.rs`は
`4016 / 132375`、SHA `4458bc2353c437d4427b39f96e0041bf1c321e19cff0ec4565c3f50084f83c4c`。
`elaborator.rs`は`23682 / 890332`、SHA
`a91e2456c279ffec9a2f67a18d9741f8885228c5a31d600e41622fcd1e03bfb9`。
Task264 leafは`1017 / 44370`、SHA
`23ad08e3ac46e36ee34121cee49873b90f796fd50a18ad632aeca032598e79b6`。
VC Task180は`1323 / 50775`、SHA
`1e471e4058d091be83d865542d8d27467cc10fd09c6a2fc82ae80571d314436c`。
Central auditは`7394 / 559472`、SHA
`84707772a1bee9acb4a8e713252db848f0aea2421c4f08937751c835d680f749`。
Contract tree `119/119 -> 120/120`、Core library test `163 -> 164`、mizar-test `646`不変。

Pre-source spec/API・bilingual/boundary review、post-source test-sufficiency・implementation/
default-deny・source-doc・bilingual/boundary・final quality reviewを行う。Focused CoreIR/Step4/
Task264/VC、affected package/lint/metadata、fmt、warnings-denied Clippy、offline metadata、
all-feature workspace/doctest、protected invariance、hard gate `9/9`、quality `>=90`、task-only
commit、clean proofがexit。

完了後はsmallest Core35 property domain/return-type inputまたはCore36 property body seedをfresh
inventoryできるが、本taskはauthorizeしない。Multiple implementation、field owner、property
value/correctness/coherence semantics、Task277Bはdeferred/not-ready。

## Completion evidence

Pre-source specification/API・bilingual/boundary reviewはcontract repair後no findings。
Post-source test-sufficiencyのsource-map precedence/independent environment operand findingを
repairし、blocking/high/medium findingなしを再確認した。Implementation、source-doc/API、
bilingual/boundary、final quality reviewはblocking/high/medium findingなし。Independent final
resultはhard gate `9/9`、valid uncapped quality `100/100`。

Focused CoreIR owner、Step4、Task264 means/equals `2/2`、VC Task180 probeがpass。
Package suiteはmizar-core `164`、mizar-test `646` + lint `15` + metadata `137`、mizar-vc
`105`、required enlarged test stackによるmizar-checker `580`がpass。Fmt、offline metadata、
warnings-denied all-target/all-feature Clippy、enlarged-stack all-feature workspace test/doctestも
pass。Contract treeは`120/120`。Protected Task264 `.miz`/expectation/trace/checker property・
source-type input、stash `f65cf4...`は不変で、commit時にunstaged/staged diff checkをpassさせる。

Final measured artifactは`core_ir.rs` `4393 / 146011`、SHA-256
`4e614a6ee98d0ef6b93dcd5d708728e41b79f613b16880269550051450793fd1`、
`elaborator.rs` `23685 / 890564`、SHA-256
`1d78d960032e2f4086f712d258a8ec247aa12daeff88f51c6afe8f4d880a7162`、
Task264 leaf `1022 / 44699`、SHA-256
`e584e3a36d8c8911d4e5f49209128cb35e81d0c93d254419476b93557a86fdca`、
VC Task180 `1324 / 50836`、SHA-256
`1622fea0fdb24ac900ef22a9ac604ee5a45cb66a40eebaee0c540b600b71df61`、
central audit `7413 / 560700`、SHA-256
`a085dc14b0479cfab399ce5b594134b812b094b71d0885f948aa1ec1bea0f40a`。
