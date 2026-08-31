# Task CORE-SOURCE-ATTRIBUTE-ITEM-CONTEXT-33I261: Task261 attribute item context

> canonical English:
> [EN contract](../en/CORE-SOURCE-ATTRIBUTE-ITEM-CONTEXT-33I261.md)。

Status: implementation/verification complete。Exact task-only commitと
postcommit successor inventoryはpending。本taskは、完了したCore Task
33I260に続くuser-selected Task261-specific successorである。Zero-semantic/
zero-creditで、Core 33を完了せず`MT10-CIR-TE`をactivateしない。

## Identity、authority、decision

- Task: `CORE-SOURCE-ATTRIBUTE-ITEM-CONTEXT-33I261`。
- Primary owner: `mizar-core::elaborator`、Core Task 33。
- Owning plan: [mizar-core crate plan](../../mizar-core/ja/00.crate_plan.md)。
- Checker dependency: 既存Task248 Profile-B
  `SourceBindingContextHandoff`とactive Task261
  `SourceAttributeDefinitionHandoff`。
- Core dependency: 完了済み33LB。Task259/260 handoffはprotected precedentで
  ありinputではない。
- Prepared consumer: Core33--35がcomplete deterministic real `CoreIr`を生成
  した後のfuture `MT10-CIR-TE`のみ。
- User decision: 同時にreadyなTask262より先にexact Task261 family-specific
  one-row standalone handoffを選択する。
- Coverage: zero semantic/execution credit。Task277Bはnot-ready/zero-credit。

Authority orderは`doc/spec/en/`、既存`.miz`、trace、expectation、design、source。
Chapter 6 §§6.1/6.2/6.8.1/6.9はordinary attribute-definition formとpredicate-
style identityを固定し、Chapters 11/12はcurrent-module identity、visibility、
source order、item完了後activationを固定する。Chapter 16 §§16.6/16.7.2では
attribute-specific correctnessはredefinition coherenceであり、exact ordinary
Task261はinitial obligationを所有しない。

既存pass source/checker handoffは、normal Task248 Profile-B definition block内の
one normal public attribute definitionをauthenticateする。Fresh inventoryでは
Task261/262がともにreadyでauthority tie-breakがなかったため、user採択がordering
だけを供給した。`spec_gap`はなく、missing Core association/private consumerは
bounded `design_drift`/`test_gap`。Remote baseline差はreport-only
`repo_metadata_conflict`。

## Frozen public APIとownership

`crates/mizar-core/src/elaborator.rs`は次だけを追加できる。

- immutable `SourceAttributeCoreItemAssociation`：`source_item()`、
  `definition()`、`symbol()`、`core_item()`。
- source-ordered immutable `SourceAttributeCoreItemAssociationTable`：
  `get(SourceAttributeDefinitionId)`、`iter()`、`len()`、`is_empty()`。
- complete 33LB、Task248 source context、Task261 checker handoff、association
  tableをby-value retainするimmutable `SourceAttributeCoreContextHandoff`：
  `source_id()`、`module_id()`、`context()`、`source_bindings()`、
  `source_context()`、`checker_owner()`、`items()`、non-authoritative
  `debug_text()`。
- non-exhaustive `SourceAttributeCoreContextError`：precedence順に
  `EnvironmentMismatch`、`InvalidSourceBindingContext`、
  `InvalidCheckerOwner`、`InvalidCoreContext`、`InvalidItemAssociation`。
- `SourceAttributeCoreContextProducer::build(SourceBindingCoreContextHandoff,
  SourceBindingContextHandoff, SourceAttributeDefinitionHandoff) ->
  Result<SourceAttributeCoreContextHandoff,
  SourceAttributeCoreContextError>`。

全fieldはprivate。Producerは全inputをby valueでconsumeし、complete
postvalidation後だけpublishする。Constructor、adapter、installer、unchecked
admission、compatibility layer、`CoreContextInput`/`CoreContext`/`CoreIr` field、
Typed/Resolved slot、public generic definition-family abstractionを追加せず、33LB/
33I259/33I260 APIを変更しない。

## Cardinality、identity、order、provenance

- Existing sourceは116 bytes/final LF、SHA-256
  `ffd4954aad628d7946aaf7afb1b472a6bdfca7bce5ba0cf09f5b284c9dda07bf`。
- Task248 exact Profile Bは`1/2/2/2/2/2/0`。One normal
  `DefinitionBlock` `SourceItemId(0)`、two ordered parameter declarations/
  bindings、exact module/definition binding/local-type contexts、two links、zero
  diagnostics。
- Task261 exact tableはdefinitions/parameters/subjects/definientia
  `1/2/1/1`。Task248/249/252/256 fingerprintはnonemptyで、initial-obligation
  input/projectionはない。
- Definition 0はtyped id 0、whole attribute symbol、resolver definition 0、
  contribution 0、ordinal 0、context 1、site 40、inner range `45..110`、exact
  spelling、normal local origin `[4,0,7,0]`。
- Parameters 0/1はbindings 0/1、type applications 0/1、sites 27/31、ordinals
  0/1、ranges `13..26`/`29..42`、declaration ranges `17..18`/`33..34`、
  context 1をretain。Subject 0はbinding 0/site 40/range `78..79`/`x`。
  Definiens 0はatomic formula 0/site 39/range `104..109`/`x = y`。
- Coreはretained whole `SymbolId`でlookupしたexact one valid public
  `Attribute` item。Dependency、diagnostic、import、generated origin、partial/
  recoveryはなく、one pending `DefinitionalItem` boundary/worklist rowを持つ。

Context linkは`SourceItemId(0)`を選択し、tableはtyped definition id 0でkeyed
one row。Numeric idをreinterpretせず、display name、FQN alone、range alone、
shell ordinal、seed/map/worklist orderをjoinに使わない。Item/source map/boundary/
worklistはouter `0..115`でなくinner `45..110`とexact provenance
`source-attribute-core-item-v1.definition.0`を使う。`Valid`はitem shellのみで、
bodyは`PendingBody`、equality formulaはchecker-ownedのまま。

## Default-deny oracle

Sort、repair、inference、recovery、unchecked admission、partial publicationなしで、
次をrejectする。

1. source/module/`BindingEnv` mismatchまたはforeign inputs。
2. Task248 Profile-Bのitem/declaration/binding/context/local-context/link/range/
   site/role/order/owner/recovery/diagnostic mismatch。
3. Task261 cardinality/lower fingerprint/resolver identity/symbol/definition/
   contribution/origin/definition/parameter/subject/definiens mismatch。
4. missing/`None`/foreign/mismatched context linkまたはsource item。
5. missing/extra/duplicate/reordered/stale/mismatched/orphan association。
6. Core item/symbol/kind/visibility/status/range/provenance/source map/worklist/
   dependency/diagnostic/generated-origin/boundary mismatch。
7. display name、spelling、FQN alone、range alone、numeric id、shell ordinal、
   seed/map/worklist iterationによるjoin。

## Installation boundaryとdeferred semantics

既存private Task261 real-source test leafだけがauthenticated definitionからone
Core seedを作り、Core contextをprepareし、complete `BindingEnv`へ33LBをapplyし、
standalone producerを呼ぶ。Retained inputs、one-row association、full item/source-
map/boundary/worklist、deterministic replay、default-deny matrixを検証する。

Production runner branch、Typed/Resolved/CoreContext/CoreIr installationはない。
`.miz`、expectation、trace、active result、diagnostic、metadata、coverageを変更しない。

Task262--264、generic/complete Core33 inventory、Core34 attribute/type/evidence、
Core35 formula、Core36 definition body/correctness、attribute application、
redefinition/coherence、initial obligations、proof/acceptance、`GeneratedOrigin`、
C4C8 composition、snapshot、`MT10-CIR-TE`、diagnostic、Task277Bはdeferred。

## Artifacts、audit、baseline、exit

Source changeはexactly次の2 files。

1. `crates/mizar-core/src/elaborator.rs`。
2. `crates/mizar-test/src/runner/tests/type_elaboration/source_attribute_definition.rs`。

Derived docsはpaired contract、paired Core plan/source-family/TODO/elaborator/
source-spec/bilingual/task-ledger、paired mizar-test harness/bilingual audit、central
`spec_coverage_audit.md`に限定する。Checker API/ownerは変わらないためchecker
docsは変更しない。Central auditはzero-credit Core mapping/follow-up narrowingだけを
記録し、spec/test intent/trace/coverage creditは不変。

Freeze baselineは`elaborator.rs` `20805 / 775898`、SHA-256
`b8ca96a9ca86078b664a2f6f2581f45f820f13b9dff20ee624adbb32e04aa22e`、
Task261 test leaf `1113 / 41268`、SHA-256
`6d7f492627f32f80df9a9dd17fb0548bae3a1107279837013d7f38556053766d`。
Contract trees `112/112 -> 113/113`。Exactly two private testsを追加し、Core
test `163`、mizar-test `636 -> 638`、metadata `137`を想定する。

Protected Task261 source/expectationは
`ffd4954aad628d7946aaf7afb1b472a6bdfca7bce5ba0cf09f5b284c9dda07bf` /
`ed8bc242f86206a56d178ef1d665faaa36c24d4943e7ca70e53af3decbecf4d8`。
Task260/259/248/reserve/C4C7 hashes、trace
`17bba212e5216256b5883ce641048de263cd045a12adf060ed354973a6ae6728`、stash
`f65cf4a13752ec380710814a9ac6392ccb9d75d4`も不変。

Entry `HEAD`は`f8e9fc212f1c24a65b7fa1b2faa0e57e18927b9e`、actual
`origin/main`は`de42b58f7322128566326c8ee1d3d1e9a5fe4d77`、divergence
`0/1`。Original expected remoteとの差はreport-only `repo_metadata_conflict`で、
fetch/push/stash mutation/metadata repairは禁止。

Pre-source independent spec/equivalenceとbilingual/boundary review、post-source
test-sufficiency/full implementation/source-docs-API reviewをfindingなしまで行う。
Focused Task261/protected probes、Core/mizar-test lint、metadata、fmt、offline
metadata、warnings-denied Clippy、all-feature tests、protected hash/count/status、
`git diff --check`をpassさせる。Docs-only evidence edit後にrisk不変のbroad suiteを
反復しない。

Exitはhard gates `9/9`、parent score `>=90/100`、exact task-only commit、clean
postcommit、protected invariance、Task277B not-ready/zero-credit、fresh successor
inventoryを要求する。このcontractはsuccessorを選ばない。

## Completion evidence

Standalone producerとexact two-test private Task261 consumerはcomplete。Final
source measurementは`elaborator.rs` `21540 / 805739`、SHA-256
`68d9623412dc1f1186ded06eff762d498e6d5b5431eca0f018bcc55df28ea07a`、Task261
test leaf `1510 / 56394`、SHA-256
`f4bfcaa0fe0446b36a316b06763d39ca84a37bb1acc4e18b3e212de022341c0e`。
Contract treeはexactly `113/113`、Core library `163`、mizar-test library `638`
(`636 + 2`)、metadata `137`。

Pre-source specification/equivalence reviewはno findings。Bilingual/boundary
reviewのpublic API inventory不足はpaired修正後no findings。Post-source test
sufficiency reviewのcontext-level empty-state evidence不足は4 assertions追加後no
findings。Implementation reviewのTask248 Profile-B exactness 3件と、初回repairが
protected Task259 validatorへ誤適用された回帰は、Task259変更を除去しTask261だけへ
配置後no findings。Task249/252/256 fingerprint nonemptyはfrozen checker-handoff
trust boundaryであり、lower input追加はfrozen API違反となる。Source/docs/API review
のstale statusとJA return signature不足は本updateで修正し、staging前にfinal
re-reviewする。

Focused Task261 `2/2`、protected Task259/260は各`2/2`、Core `163/163`、
mizar-test lint `15/15`、metadata `137/137`がpass。Fmt、offline metadata、full
warnings-denied Clippy、`cargo test --all-features`（integration/doctest含む）もpass。
Protected Task261 source/expectation/trace hashesとstash
`f65cf4a13752ec380710814a9ac6392ccb9d75d4`は不変。

Parent hard gatesは`9/9`、valid uncapped scoreは`98/100`：specification
`20/20`、test contract `19/20`、traceability `15/15`、implementation `14/15`、
design/source synchronization `10/10`、boundary discipline `10/10`、verification
`5/5`、handoff `5/5`。No cap。Task261はzero-credit。Core34--36、
`GeneratedOrigin`、production install、`MT10-CIR-TE`、diagnostic、coverage、
Task277Bはdeferredで、Task277Bはnot-ready/zero-credit。

Report-only `repo_metadata_conflict`は継続。Precommit `HEAD`は
`f8e9fc212f1c24a65b7fa1b2faa0e57e18927b9e`、actual `origin/main`は
`de42b58f7322128566326c8ee1d3d1e9a5fe4d77`、divergence `0/1`。Fetch/push/
stash mutation/metadata repairは行っていない。Exact task-only commitとfresh
postcommit successor inventoryはpending。
