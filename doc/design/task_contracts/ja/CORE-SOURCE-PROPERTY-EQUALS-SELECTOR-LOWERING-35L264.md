# Task CORE-SOURCE-PROPERTY-EQUALS-SELECTOR-LOWERING-35L264: Task264 equals term lowering

> canonical English:
> [../en/CORE-SOURCE-PROPERTY-EQUALS-SELECTOR-LOWERING-35L264.md](../en/CORE-SOURCE-PROPERTY-EQUALS-SELECTOR-LOWERING-35L264.md)。

Status: implemented/verified、hard gate `9/9`、valid independent quality `100/100`。
Specialized representation-only、
unattached Core35 loweringで、language behavior/active route/trace/metadata/coverage credit
変更なし。

## Identity、authority、classification

| Field | Frozen value |
|---|---|
| Task | `CORE-SOURCE-PROPERTY-EQUALS-SELECTOR-LOWERING-35L264` |
| Primary owner | `mizar-core::elaborator` Core35 |
| Required predecessors | Task35E264、Task264D、Task33P264、IR264 |
| Input | Complete `SourcePropertyEqualsSelectorTermSeedHandoff` 1件 |
| Consumer | Separately reviewed Task264 Core36 property definition/body input |
| Owning plan | [`mizar-core` Task Index](../../mizar-core/ja/00.crate_plan.md#task-index) |
| Coverage | Active execution/trace/metadata/coverage creditすべてzero |

Authorityは`doc/spec/en/`、existing `.miz`、trace、expectation、design、sourceの順。
Chapter 13とprotected equals fixtureがdirect `M.carrier` termを固定し、Task35E264が
`Var(0)`→`Select(field, seed0)`、non-item property owner、source/provenanceをauthenticate済み。
`spec_gap`/language/test-intent choiceなし。Missing specialized table/map lowererはbounded
`design_drift`/`source_drift`、private assertion 2件は`test_gap`。

Generic `TermAndFormulaLoweringInput.owner`変更はscope外。Generated origin/diagnostic/
obligationのordinary item ownershipまで拡張せず、carrier item 0をproperty ownerへ代用しない。

## Frozen API、representation、validation

Exact public APIはEN contractのRust blockをcanonicalとする。Private-field associationは
base seed/term、selector seed/term、root term getterだけを持つ。Handoffはcomplete
Task35E264 capability、local `CoreTermTable`、term-only `CoreSourceMap`、associationをretainし、
definition ownerはseed handoffからdelegateする。Non-exhaustive errorは
`InvalidSeedHandoff`/`InvalidTermLowering`、producer inputはcomplete handoff 1件だけ。

Exact rowsはlocal term 0 `Var(CoreVarId(0))`、local term 1
`Select { selector: <whole field SymbolId>, base: CoreTermId(0) }`。Associationはseed
`0/1`→term `0/1`、root 1。Local idsでありglobal `CoreIr` installation/body attachmentではない。

Term sourceはTask35E264 direct sourceへexisting merge ruleでChecker provenanceを追加する。
Term 0は`173..174`/`.base`、term 1は`173..182`/`.selector`。Term-source map 2件と一致し、
other source-map domainはempty。Debugはexact
`source-property-equals-selector-term-lowering-v1|module=<package>.<path>|owner-anchor=0|property=<property-fqn>|seed=0:1|term=0:1|root=1`
でfinal LFなし。

Validation precedenceはseed handoff→definition owner→association→term table→source map→
complete postvalidation。Ownerはanchor 0/non-item/sole `marker`、association `0->0`/
`1->1`/root1、dense exact terms、whole selector symbol、term0 base、exact merged source/
provenance、term-only mapをrequireする。Private fields/branded inputがintegrity boundary。

## Scope、baseline、exit

Rust editは`crates/mizar-core/src/elaborator.rs`と
`crates/mizar-test/src/runner/tests/type_elaboration/source_property_implementation.rs`だけ。
Deterministic exact loweringとunattached foreign-transaction isolation test 2件を追加。
Core/checker `164/582`不変、mizar-test `650 -> 652`、Task264 family `12 -> 14`。

Generic lowerer/input変更、carrier owner substitution、CoreIr installation、formula、definition
row/body、field/type、normalized type/fact/guard、generated origin、diagnostic、obligation、
correctness/coherence、means `it`、route/snapshot/acceptance/fact/axiom/creditなし。
`doc/spec`/`.miz`/expectation/trace/checker/`core_ir.rs`/VC/Cargo/module topologyを編集しない。

Stable owner sectionはCore [Task35L264 API](../../mizar-core/ja/elaborator.md#task-35l264-task264-equals-selector-term-lowering)、
[decomposition](../../mizar-core/ja/source_family_decomposition.md#task-35l264-task264-equals-selector-term-lowering)、
mizar-test [private probe](../../mizar-test/ja/harness.md#core-task-35l264-private-task264-equals-term-lowering-probe)。
Central auditはbounded gap close/follow-up ownershipだけを更新し、coverage
`430/396/0/23`/creditは不変。

Baseline HEAD `112129671cfaefe5635676697baa3e9e028cb548`。
`elaborator.rs` `24788 / 934127`、SHA-256
`10bde6f70141a7848e73278b23f3d66c866d158acbee65b6bab3093e7b5210d2`、private leaf
`1633 / 68263`、SHA-256
`bd320b1ec77859417b13708412e5e44b5030609b6632773388655a1be57ef9ee`、central audit
`7474 / 564328`、SHA-256
`16a1f0fce5b0ec82f706f81a34154c24dec6e4a13d8022ce3052d513b997cb67`。
Contract tree `124/124 -> 125/125`、protected hash/stash
`f65cf4a13752ec380710814a9ac6392ccb9d75d4`不変。

Pre/post-source review、verification、hard gate `9/9`、quality `>=90/100`、task-only
commit/clean postcommit/fresh inventory必須。Nextはunattached root向けseparate Core36
property definition/body input。Means `it`、correctness/coherence、route、Task277Bはseparate。

## Implementation evidence

Specialized producer/private test 2件をexact Rust 2 pathへimplemented。Independent
pre-source authority/API、implementability、bilingual/boundary reviewはfindingなし。
Post-source test sufficiency、implementation/default-deny、source/doc/bilingual/boundary
reviewもstale measurement 1件の修正後にremaining findingなし。

Focused Task35L264 `2/2`、Task264 family `14/14`。Core `164` unit/`2` determinism/
`12` lint、mizar-test `652` unit/`3` layout/`15` lint/`137` metadata/`2` public-enum/
`21` snapshot pass。Format、offline metadata、`git diff --check`、warnings-denied
all-target/all-feature Clippy、enlarged-stack all-feature workspace test/doctest pass。
Coverage plan `430/396/0/23`不変。

Post-source measurement: `elaborator.rs` `25108 / 945613`、SHA-256
`55597e4a5e18fc13fe2909eaea504cab2be16d48bf526a7f1b1c93d82c7706b4`、private Task264 leaf
`1891 / 77732`、SHA-256
`92d664fab2dbe790433398606652ce2b8e65974c642d092d30411cb98fe1b437`。
Contract tree `125/125`。Final audit `7490 / 565242`、SHA-256
`a1d534d9e533d9266744d8ff874adb7dd7119d9e2402970d4baeeb9138c2366f`。Protected
fixture/expectation/trace/stash不変。Parent/independent hard gate `9/9` pass。Final
read-only auditはfindingなし、score capなし、valid `100/100`
（`20/20/15/15/10/10/5/5`）。Exact task-only commitがremaining。
