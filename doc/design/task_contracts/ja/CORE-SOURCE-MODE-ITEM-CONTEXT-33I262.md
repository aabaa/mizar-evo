# Task CORE-SOURCE-MODE-ITEM-CONTEXT-33I262: Task262 mode item context

> canonical English:
> [EN contract](../en/CORE-SOURCE-MODE-ITEM-CONTEXT-33I262.md)。

Status: implementation/verification complete。Independent pre-sourceと
post-source reviewはdocumentation repair後no findingsで終了。Task-only commitと
postcommit inventoryはpending。本taskはzero-semantic/zero-creditで、Core33を
completeせず`MT10-CIR-TE`をactivateしない。

## Identity、authority、readiness

- Task: `CORE-SOURCE-MODE-ITEM-CONTEXT-33I262`。
- Owner: `mizar-core::elaborator` Core33。
- Owning plan: [`mizar-core` crate plan](../../mizar-core/ja/00.crate_plan.md)。
- Dependency: exact Task248 Profile-B、active Task262
  `SourceModeDefinitionHandoff`、completed 33LB。33I259--261はprotected
  precedentでinputではない。
- Future consumer: complete Core33--35後の`MT10-CIR-TE`のみ。
- Coverage: zero。Task277Bはnot-ready/zero-credit。

Authority orderはspec、existing `.miz`、trace、expectation、design、source。
Chapter7 §§7.1/7.2/7.7/7.8/7.9がmode identity、parameter order、RHS
inhabitation、`sethood`、predicate encodingを固定し、Chapters11/12/16がidentity/
visibility/order/obligation boundaryを固定する。Checker順序とTask261 contractの
user orderingによりTask262はdependency-minimal successor。`spec_gap`はなく、
missing Core association/private consumerはbounded `design_drift`/`test_gap`。

## Frozen public API

`elaborator.rs`は次だけを追加する。

- `SourceModeCoreItemAssociation`と`source_item()`/`definition()`/`symbol()`/
  `core_item()`。
- `SourceModeCoreItemAssociationTable`と`get(SourceModeDefinitionId)`/
  `iter()`/`len()`/`is_empty()`。
- 33LB、Task248、Task262、tableをby-value retainする
  `SourceModeCoreContextHandoff`と`source_id()`/`module_id()`/`context()`/
  `source_bindings()`/`source_context()`/`checker_owner()`/`items()`/
  `debug_text()`。
- non-exhaustive `SourceModeCoreContextError`：`EnvironmentMismatch`、
  `InvalidSourceBindingContext`、`InvalidCheckerOwner`、`InvalidCoreContext`、
  `InvalidItemAssociation`。
- `SourceModeCoreContextProducer::build(SourceBindingCoreContextHandoff,
  SourceBindingContextHandoff, SourceModeDefinitionHandoff) ->
  Result<SourceModeCoreContextHandoff, SourceModeCoreContextError>`。

全fieldはprivate。Complete postvalidation後のみpublishする。Generic adapter、
constructor、installer、compatibility、unchecked admission、Core/Typed/Resolved
slotを追加せず、33LB/33I259--261 APIを変更しない。

## Exact profileとboundary

- Sourceは141 bytes/final LF、SHA-256
  `3271f243670bd781c7167ff0d3bf463263a318abbe261aabdde1842c532a725e`。
- Task248はexact `1/2/2/2/2/2/0`。Item 0はshell 0/site 50/range
  `0..140`/context 1/local context 1/scope `[0]`、module site 53、parameter
  sites 37/41。
- Task262 tableは`1/2/1/1/1/1`、Task248 fingerprint exact、Task249+249M
  fingerprint nonempty、base obligation count 0。
- Definition 0はwhole Mode symbol、resolver definition 0、contribution 0、site
  49、inner `45..135`、ordinal 0、context 1、normal、exact spelling、
  application/expansion/request/property `0/0/0/Some(0)`、origin `[4,0,10,0]`。
- Parameters 0/1はbindings/types 0/1、sites 37/41、ranges
  `13..26`/`29..42`、declarations `17..18`/`33..34`、pattern occurrences
  `86..87`/`89..90`、context 1。
- Application 0はordered `[0,1]`、site42/range`73..91`。Expansion 0とRHS
  request 0はsite44/range`95..98`/RHS0。Property 0は`Sethood`、site48、range
  `102..135`、justification `113..134`、obligation id0。
- Coreはwhole SymbolIdで選ぶexact one valid public `Mode`、one pending
  `DefinitionalItem`/worklist、dependency/diagnostic/generated origin/obligation
  seedなし。

Associationはtyped `SourceModeDefinitionId(0)`でkeyed one row、context linkで
SourceItem 0を選ぶ。Numeric ID、name、FQN alone、range、seed/map/worklist orderは
joinではない。Core sourceはouter `0..140`でなくinner `45..135`、provenanceは
`source-mode-core-item-v1.definition.0`。RHS、inhabitation、sethood、computation、
pending obligationはchecker-ownedのままでCore34--36へdeferする。

## Default deny、installation、deferral

Environment、Task248 exact shape、Task262 cardinality/fingerprint/base count/
resolver/origin/全row、context link、association、Core item/source-map/boundary/
worklist/dependency/diagnostic/generated-originのmissing/extra/duplicate/reordered/
stale/mismatch/recovered/partialをfail closed。Sort、repair、inference、unchecked
admissionは禁止。Source-type handoffとobligation tableはinputでなくchecker trust
boundaryであり、second lower slotは追加しない。

Existing private Task262 leafだけがone Mode seed、Core preparation、33LB、standalone
producerをcomposeする。Exactly two testsがpositive/deterministic state、ten Core
mutations、four foreign combinationsを検証する。Production route/installer/slot、
`.miz`/expect/trace/metadata/coverage変更なし。Task263/264、generic Core33、Core34
mode/RHS/inhabitation/sethood、Core35 formula、Core36 body/correctness/obligation、
proof/discharge/acceptance、`GeneratedOrigin`、C4C8、snapshot、`MT10-CIR-TE`、
diagnostic、Task277Bはdeferred。

## Artifacts、baseline、exit

Source changeは`crates/mizar-core/src/elaborator.rs`と
`crates/mizar-test/src/runner/tests/type_elaboration/source_mode_definition.rs`のみ。
Derived docsはpaired contract、paired Core owner docs、paired mizar-test harness/
bilingual audit、central audit。Checker docsは不変。

Freeze baselineはCore `21540 / 805739`、SHA
`68d9623412dc1f1186ded06eff762d498e6d5b5431eca0f018bcc55df28ea07a`、test leaf
`1242 / 45711`、SHA
`7ae8f4d7cd6805d85afe92380cd4fc702bfafc7124ee01f3283e36e460b2b798`。
Contract tree `113/113 -> 114/114`、Core tests `163`、mizar-test `638 -> 640`、
metadata `137`。

Protected Task262 source/expectationは
`3271f243670bd781c7167ff0d3bf463263a318abbe261aabdde1842c532a725e` /
`046b5a686600f78e1598c515c05f8124ec19edef56a14385a2d05bced527601e`、trace
`17bba212e5216256b5883ce641048de263cd045a12adf060ed354973a6ae6728`、stash
`f65cf4a13752ec380710814a9ac6392ccb9d75d4`。

Entry HEAD `4c6ecafc2a9bee7a4eb6e3f27336733fc672bd57`、origin/main
`de42b58f7322128566326c8ee1d3d1e9a5fe4d77`、divergence `0/2`。
Mismatchはreport-only `repo_metadata_conflict`。Fetch/push/stash mutation/repair禁止。

Pre-source spec/equivalenceとbilingual/boundary、post-source test/implementation/
source-docs-API reviewをno findingsまで行う。Focused/protected/lint/metadata/fmt/
offline metadata/full Clippy/all-feature testsをpassし、hard gates `9/9`、score
`>=90/100`、task-only commit、clean postcommit、protected invariance、Task277B
not-ready/zero-credit、fresh inventoryでexitする。

## Completion evidence

Standalone producerとexact two private Task262 testはcomplete。Final source
measurementは`elaborator.rs` `22350 / 839135`、SHA-256
`3fe6e32d621f6516b54a67fd7649e6504b619c3e5e570ed26143060b5e849510`、Task262
test leaf `1637 / 60702`、SHA-256
`87355decdec7f657bbe421190428b4aa4fd0e47e1420df3962e6063584644bc5`。
Contract treeはexactly `114/114`、Core library `163`、mizar-test `640`
（`638 + 2`）、metadata `137`。

Pre-source specification/equivalenceとbilingual/boundary reviewはno findings。
Post-source test-sufficiency、implementation、source/documentation/API reviewは、
public API inventoryとstatus driftのdocumentation repair後no findings。Core lintの
link failureはJA owning-plan link追加でrepairした。

Focused Task262 `2/2`、Task262 route `6/6`、protected Task259--262 item-context
probe `8/8`、Core test/integration/lint `163/163`、mizar-test `640/640`、Core
lint `12/12`、mizar-test lint `15/15`、metadata `137/137`がpass。Formatting、
offline metadata、full warnings-denied Clippy、integration/doctestを含む
`cargo test --all-features`、`git diff --check`もpass。Protected Task262
source/expectation/trace hashはfrozen valueと一致し、stash
`f65cf4a13752ec380710814a9ac6392ccb9d75d4`は不変。

Parent reviewはhard gate `9/9`をpassし、score capなしのquality score
`98/100`（specification `20/20`、test contract `20/20`、traceability `15/15`、
implementation `15/15`、design/source sync `10/10`、boundary `10/10`、
verification `5/5`、handoff `3/5`）。Task262はzero Core creditのまま。Core34--36 mode/RHS/type/
inhabitation/sethood semantics、`GeneratedOrigin`、production install、
`MT10-CIR-TE`、diagnostic、coverage credit、Task277Bはdeferred/not-ready。
Exact task-only commitとfresh postcommit successor inventoryはpending。
