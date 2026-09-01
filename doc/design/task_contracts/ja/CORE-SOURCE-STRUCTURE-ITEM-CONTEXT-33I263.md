# Task CORE-SOURCE-STRUCTURE-ITEM-CONTEXT-33I263: Task263 structure item context

> canonical English:
> [EN contract](../en/CORE-SOURCE-STRUCTURE-ITEM-CONTEXT-33I263.md)。

Status: exact task-only commit時点でcomplete。全independent reviewはrepair後
no findings、required focused/broad verificationは全pass、final read-only qualityは
hard gate `9/9`、score capなしのvalid `98/100`。Commit自身のhashは埋め込めないため
final handoffで報告する。本taskはzero-semantic/zero-credit Core-33 prerequisiteで、
Core33をcompleteせず`MT10-CIR-TE`をactivateしない。

## Identity、authority、readiness

- Task: `CORE-SOURCE-STRUCTURE-ITEM-CONTEXT-33I263`。
- Owner: `mizar-core::elaborator` Core 33。
- Owning plan: [`mizar-core` crate plan](../../mizar-core/ja/00.crate_plan.md)。
- Input: active exact Task263 `SourceStructureDefinitionHandoff`とprepared
  `CoreContext`だけ。Task248/33LB/33I259--262はinputではない。
- Coverage: zero。Task277Bはnot-ready/zero-credit。

Authority orderはspec、existing `.miz`、trace、expectation、design、source。
Chapter 5がstructure identity、fields-only constructor order、inheritance、
root/path/view mapping、mapped-member type-inclusion boundaryを固定し、Chapters
11/12/16がidentity/visibility/order/obligation boundaryを固定する。Existing
320-byte Task263 sourceとchecker handoffがexact `2/4/1/2/0`をauthenticateする。
`spec_gap`はない。Missing Core association/private consumerはbounded
`design_drift`/`test_gap`。User採択によりDerived Core itemはBase Core itemへの
local dependencyをexact 1本retainする。

## Frozen API

`elaborator.rs`は次だけを追加する。

- `SourceStructureCoreItemAssociation`と`definition()`/`symbol()`/`core_item()`。
- typed definition ID keyedのsource-ordered
  `SourceStructureCoreItemAssociationTable`と`get`/`iter`/`len`/`is_empty`。
- `CoreContext`、Task263 checker owner、tableをby-value retainする
  `SourceStructureCoreContextHandoff`と`source_id()`/`module_id()`/`context()`/
  `checker_owner()`/`items()`/`debug_text()`。
- non-exhaustive error precedence: `EnvironmentMismatch`、
  `InvalidCheckerOwner`、`InvalidCoreContext`、`InvalidItemAssociation`。
- `SourceStructureCoreContextProducer::build(CoreContext,
  SourceStructureDefinitionHandoff) -> Result<SourceStructureCoreContextHandoff,
  SourceStructureCoreContextError>`。

全fieldはprivate。Complete postvalidation後のみpublishする。Generic adapter、
constructor、installer、compatibility、unchecked admission、Core/Typed/Resolved
slotは追加せず、prior APIを変更しない。

`SourceStructureCoreContextError`はdownstream forward-compatible public surfaceで
`#[non_exhaustive]`を維持する。Rust edit前にsynchronized EN/JA public-enum policy tableと
source/spec public API inventoryへnew enumと全5 API groupを追加する。

## Exact profileとdependency

- Fixtureは320 bytes/final LF、SHA
  `078eaee4b17341c9d8ebeb8a1f631ca984873bd07eb4e5d9c1a9486b39ac6671`。
- Checker profileはdefinitions/members/inheritances/mappings/coherence
  `2/4/1/2/0`、nonempty source-type fingerprint、base obligation count 0。
- Definition 0 Baseはdefinition/contribution `0/0`、site57、`13..98`、ordinal0、
  members `[0,1]`、constructor fields `[0]`、origin `[4,0,11,0]`。
- Definition 1 Derivedは`3/0`、site65、`102..190`、ordinal1、members `[2,3]`、
  constructor fields `[2]`、origin `[4,0,11,1]`。
- Members 0--3はexact owner/ordinal/kind/site/range/written-type/constructor
  profilesをretainするchecker-owned row。Core member itemは作らない。
- Inheritance 0はchild1→parent0、site70、`194..314`、ordinal0、mappings
  `[0,1]`。MappingsはField/Property、view-parent-root `2/0/0`と`3/1/1`、
  path `[0]`をexact validateするだけでlowerしない。
- Coreはvalid public `Structure` 2件。Base dependencyは空、DerivedはBaseへの
  local dependency exactly 1本。External/missing dependency、diagnostic、import、
  generated origin、obligation、binder、partial/recoveryは0。各itemはpending
  `DefinitionalItem` boundary/worklist entry 1件。

Associationはtyped definition 0/1 keyed 2行で、whole `SymbolId`だけをCore
registryへjoinする。Numeric ID、name、FQN alone、range、seed/map/worklist orderは
joinではない。Sourceはdefinition inner range、provenanceは
`source-structure-core-item-v1.definition.0/1`。Direct inheritanceだけが
Derived→Base dependency authority。Constructor/member/view/coherence/semantic
artifactは生成しない。

## Default deny、installation、deferral

Environment mismatch、nonexact checker row/cardinality/fingerprint/base count、
association missing/extra/reorder/stale、Core symbol/kind/visibility/status/source/
provenance/source-map/boundary/worklist drift、Base dependency、DerivedのBase以外の
dependency、external/missing dependencyをfail closed。Sort、repair、inference、
unchecked admission、partial publicationは禁止。

Existing private Task263 leafだけがdefinitionからStructure seed 2件をderiveし、
inheritance 0からDerived seedへBase symbol dependency 1本をattachし、Core contextを
prepareしてstandalone producerをcallする。Exactly two testsがpositive/
deterministic mapping、local dependency、Core mutation、foreign environmentを検証する。

Production install/route/slot、`.miz`/expect/trace/metadata/coverage変更なし。
Task264、generic/complete Core33、Core34 structure/member/type/view、Core35 term/formula、
Core36 constructor/body/correctness/obligation、proof/discharge/acceptance、
`GeneratedOrigin`、snapshot、`MT10-CIR-TE`、diagnostic、Task277Bはdeferred。

## Artifacts、baseline、exit

Sourceは`elaborator.rs`とexisting Task263 private test leafだけ。Derived docsはpaired
contract、paired Core owner docs、paired mizar-test harness/bilingual audit、central audit。
Checker docsは不変。

Freeze baselineはCore `22350 / 839135`、SHA
`3fe6e32d621f6516b54a67fd7649e6504b619c3e5e570ed26143060b5e849510`、Task263
leaf `218 / 8495`、SHA
`144bb7b9e98d7a9ae7b1824a4b6a489b840efe54b11fdcbe8f202a2b9d2816b0`。
Contract tree `114/114 -> 115/115`、Core tests `163`、mizar-test `640 -> 642`、
metadata `137`。

Protected fixture/expect/trace hashは
`078eaee4b17341c9d8ebeb8a1f631ca984873bd07eb4e5d9c1a9486b39ac6671` /
`d82c8d3102ea34fdb4a32792167c4b109b96b9c05265d3f04e6310278178e8ac` /
`17bba212e5216256b5883ce641048de263cd045a12adf060ed354973a6ae6728`、stash
`f65cf4a13752ec380710814a9ac6392ccb9d75d4`。Entry HEAD/originは
`74208cf797f2a9a24716f5b93d2189986f111109`、originは
`5c0488382989af76ef13a281a81d5630ee7eff68`、Task263 edit前divergence `0/1`。

Pre-source spec/equivalenceとbilingual/boundary、post-source test/
implementation/source-docs-API reviewをno findingsまで行う。Focused/protected/
lint/metadata/fmt/offline metadata/full Clippy/all-feature testsをpassし、hard gates
`9/9`、read-only score `>=90/100`、task-only commit、clean postcommit、protected
invariance、Task277B not-ready/zero-credit、fresh successor inventoryでexitする。

## Completion evidenceとhandoff

Standalone producerとprivate Task263 test exactly 2件はcomplete。Current Coreは
`22947 / 862541`、SHA
`e9ea1d6eabb191d7d3b8c22fe1fc11626d2e0dab86690dee662f851bb487f85c`、Task263 leafは
`731 / 28867`、SHA
`085116fa94e344eb353084c5f5511f3a007cd9a9168277dc995a5ca4ef86ec80`。Contract tree
`115/115`、Core tests `163`、mizar-test `642`、metadata `137`。

Pre-source reviewはenum/API inventory、test reviewはinheritance-derived setup/
context equality/source-boundary-worklist/default-deny、implementation reviewはbinder
admissionをrepair後no findings。Focused `2/2`、Task263 route `6/6`、protected
item-context `10/10`、checker `5/5`、Core/mizar-test `163/163`・`642/642`、lint
`12/12`・`15/15`、focused Clippy、diff checkがpass。Protected hashes/stashは不変。

Final source/docs/API reviewはno findings。`cargo fmt --all -- --check`、offline
metadata、full warnings-denied Clippy、doctestを含む`cargo test --all-features`、
metadata `137/137`は全pass。Independent final qualityはblocking/high/medium findingなし、
hard gate `9/9`、score capなしのvalid `98/100`。Exact staging、task-only commit、clean
postcommit/fresh-successor inventoryだけがtransactional stepとして残り、semantic
acceptanceをreopenしない。次候補はfresh authority/readiness確認後のTask264 property
implementation item context。全owner familyとseparate contractがreadyになるまでgeneric
Core33 installはdeferred。
