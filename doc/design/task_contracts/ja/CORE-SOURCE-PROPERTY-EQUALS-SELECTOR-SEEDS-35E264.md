# Task CORE-SOURCE-PROPERTY-EQUALS-SELECTOR-SEEDS-35E264: Task264 equals selector seeds

> canonical English:
> [../en/CORE-SOURCE-PROPERTY-EQUALS-SELECTOR-SEEDS-35E264.md](../en/CORE-SOURCE-PROPERTY-EQUALS-SELECTOR-SEEDS-35E264.md)。

Status: task-only commit向けcomplete。Independent review/verification/hard gate `9/9`、
valid uncapped quality `100/100`はcomplete。Standalone representation-only、
zero-semantic/zero-credit Core35 input prerequisite。

## Identity、authority、classification

| Field | Frozen value |
|---|---|
| Task | `CORE-SOURCE-PROPERTY-EQUALS-SELECTOR-SEEDS-35E264` |
| Primary owner | `mizar-core::elaborator` Core35 input |
| Predecessor | Task264D、Task33P264、IR264 |
| Input | Complete Task33P264 parameter context + Task264D equals selector identity |
| Consumer | Property-owner-aware Core35 lowering、then Core36 |
| Owning plan | [`mizar-core` Task Index](../../mizar-core/ja/00.crate_plan.md#task-index) |
| Coverage | semantic/execution/trace/metadata/coverage creditすべてzero |

Authorityは`doc/spec/en/`、existing `.miz`、trace、expectation、design、sourceの順。
Chapter 7/13とprotected equals fixtureがdirect term definiens `M.carrier`を固定する。
Task264Dはwhole field symbol/source graph/binding 0、Task33P264はfree term
`CoreVarId(0)`、IR264はnon-item property ownerをauthenticate済み。`spec_gap`なし。

Missing joint seed graphはbounded `design_drift`/`source_drift`、private test 2件は
`test_gap`。Generic lowererは`CoreItemId` ownerを要求するためdirect loweringはnot-ready。
Carrier item 0をproperty ownerとして代用せず、`CoreTermId`もpublishしない。

## Frozen API、representation、validation

Exact public APIはEN contractのRust blockをcanonicalとする。Private-field associationは
parameter/binding/Core-variable、source base/selector、local base/selector seed getterだけを
持つ。Handoffはauthenticated `CoreDefinitionOwner`、complete Task33P264/Task264D、exact
ordered seed 2件、associationをby-value retainする。Errorは
`EnvironmentMismatch`/`InvalidParameterContext`/`InvalidSelectorIdentity`/
`InvalidDefinitionOwner`/`InvalidTermSeeds`。Producer inputはcomplete handoff 2件だけ。

Seed 0は`Var(CoreVarId(0))`、direct range `173..174`、Checker provenance
`source-property-equals-selector-term-seed-v1.base`。Seed 1はTask264D whole field
symbolの`Select { base: CoreTermSeedId(0) }`、direct range `173..182`、Checker
provenance `source-property-equals-selector-term-seed-v1.selector`。Direct source ref
自体のprovenanceはempty。Associationは`0/0/0`、source `0/0`、seed `0/1`。

Debugはexact
`source-property-equals-selector-term-seeds-v1|module=<package>.<path>|owner-anchor=0|property=<property-fqn>|selector=<field-fqn>|source=0:0|seed=0:1|parameter=0:0:0`
でfinal LFなし。

Validation precedenceはenvironment→parameter context→selector identity→definition
owner→term seeds→complete postvalidation。Same exact property/source/module、Task33P264
`0/0/0`、Task264D exact nine zero ids/normal rows/ranges/fingerprint/whole symbol、
property owner anchor 0/non-item/`marker`、ordered seed/source/provenance/associationをrequire。
Means/mixed/foreign/stale/wrong id/owner/symbol/extra/missing/reordered/spelling-only inputは
publish前にfail closed。

## Scope、baseline、exit

Rust editは`crates/mizar-core/src/elaborator.rs`と
`crates/mizar-test/src/runner/tests/type_elaboration/source_property_implementation.rs`だけ。
Deterministic exact seed graphとmixed/foreign/default-denyのtest 2件を追加し、Core unit
164は不変、mizar-testは`648 -> 650`。

Generic lowering、carrier-item owner substitution、`CoreTermId`/table/source map、formula、
field/type association、normalized type/fact/guard、`it`、definition/correctness、diagnostic/
obligation/route/snapshot/creditを追加しない。`doc/spec`/`.miz`/expectation/trace/checker/
`core_ir.rs`/VCを編集しない。

Baseline HEADは`3d72789d1344df89bd17908415e10f550e1d2fc6`。
`elaborator.rs` `24256 / 912957`、SHA-256 `d48f63f0a8427bc6e9c4290affcc8ae5e909c8b2af5391398256894d1334ff11`、
private leaf `1377 / 58827`、SHA-256 `b7b443a4638ce10216b6ffc1f72c0324dd8806ab5a33f2d5c8b8e77ac788afc4`、
central audit `7459 / 563428`、SHA-256 `085f0836ac2a0047eb59d058aa93310e3c7f750fe090fbc230b32ca01e87b9cf`。
Contract tree `123/123 -> 124/124`、unit count開始`164/582/648`、protected hash/stash
`f65cf4a13752ec380710814a9ac6392ccb9d75d4`不変。

Implemented source実測は`elaborator.rs` `24788 / 934127`、SHA-256
`10bde6f70141a7848e73278b23f3d66c866d158acbee65b6bab3093e7b5210d2`、private
Task264 leaf `1633 / 68263`、SHA-256
`bd320b1ec77859417b13708412e5e44b5030609b6632773388655a1be57ef9ee`。
Contract tree実測`124/124`。Final central auditは`7474 / 564328`、SHA-256
`16a1f0fce5b0ec82f706f81a34154c24dec6e4a13d8022ce3052d513b997cb67`。

Paired Core/mizar-test owner docsとcentral auditをreadiness/follow-up ownershipだけ同期し、
coverage `430/396/0/23`は不変。Pre/post-source review、verification、hard gate `9/9`、
quality `>=90/100`、task-only commit/postcommit/fresh inventory必須。Nextはproperty-owner-aware
Core35 lowering。Means `it`、Core36、Task277Bはseparate/not-ready。

Stable owner sectionはCore [Task35E264 API](../../mizar-core/ja/elaborator.md#task-35e264-task264-equals-selector-term-seeds)、
[decomposition](../../mizar-core/ja/source_family_decomposition.md#task-35e264-task264-equals-selector-seed-input)、
mizar-test [private probe](../../mizar-test/ja/harness.md#core-task-35e264-private-task264-equals-seed-probe)。

## Completion evidence

Pre-source specification/API/bilingual/boundary reviewはblocking/high/medium findingなし。
Post-source implementation/default-deny/test-sufficiency reviewもfindingなし。Source/docs/API/
bilingual/boundary reviewのmedium lifecycle/gap-status/line-count finding 3件は、EN/JA
status、closed-gap record、public API inventory、current boundary measurementの同期後、
finding-specific re-reviewで全解消した。

Focused Task35E264 `2/2`、complete Task264 private family `12/12`、Core unit `164`/
determinism `2`/lint `12`、mizar-test unit `650`/layout `3`/lint `15`/metadata `137`/
public-enum `2`/snapshot `21`がpass。Fmt、offline metadata、`git diff --check`、
warnings-denied all-target/all-feature Clippy、enlarged-stack all-feature workspace
test/doctestがpass。Coverage planは`430` cases/`396` requirements/errors 0/warnings 23。

Protected means/equals `.miz`/expectation/trace/stash hashは不変。Specification/
expectation/trace/snapshot/diagnostic/obligation/route/semantic credit変更なし。Parent/
independent hard gate `9/9` pass。Final independent read-only auditはfinding/score capなし、
valid `100/100` (`20/20/15/15/10/10/5/5`)。Exact staging/task-only commitをprepareし、
clean postcommit proof/fresh successor inventoryはoperational handoffへ記録する。
