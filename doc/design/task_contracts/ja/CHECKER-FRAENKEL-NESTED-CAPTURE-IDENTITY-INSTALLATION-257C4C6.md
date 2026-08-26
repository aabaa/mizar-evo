# Task CHECKER-FRAENKEL-NESTED-CAPTURE-IDENTITY-INSTALLATION-257C4C6: nested Fraenkel capture-identity installation

> canonical English:
> [../en/CHECKER-FRAENKEL-NESTED-CAPTURE-IDENTITY-INSTALLATION-257C4C6.md](../en/CHECKER-FRAENKEL-NESTED-CAPTURE-IDENTITY-INSTALLATION-257C4C6.md)。

Owning planは[mizar-checker](../../mizar-checker/ja/00.crate_plan.md#task-index)と
[mizar-test](../../mizar-test/ja/00.crate_plan.md#task-index)。Durable owner sectionはchecker
[source formula composition](../../mizar-checker/ja/source_formula_composition.md#task-257c4c6-capture-identity-installation-boundary)、
[TypedAst](../../mizar-checker/ja/typed_ast.md#task-257c4c6-capture-identity-installation)、
[ResolvedTypedAst](../../mizar-checker/ja/resolved_typed_ast.md#task-257c4c6-capture-identity-installation)、
private test [harness](../../mizar-test/ja/harness.md#checker-task-257c4c6-private-capture-identity-installation-probe)。

## Status、決定、目的

**Status:** complete。

Completed C4C5後の人間判断はchecker-only / zero-semantic successorを1件選択する。
`TypedAst`と`ResolvedTypedAst`をalready-authenticated C4C5 receiptのimmutable destinationとする。
各ASTはprivate boxed installation wrapperを1件所有し、public accessはexisting C4C5 handoffへの
borrowだけである。InstallationはC4C5 retained pre-install snapshotに対してexact final
`TypedAst`をauthenticateし、final assemblyはそのauthenticated typed ownerからだけcloneする。

本taskはcapture semantics、C4C4 captured state、capture set、Core identity map、Fraenkel
lowering、generated parameter/origin、general parameter order、Task277B activationを追加しない。
未選択だったinstallation owner/APIは人間判断により解消する`design_drift`、exact final-owner/
corruption test欠落は`test_gap`、receiptをsemantic capture/Core readinessとして扱うことは
`boundary_violation`。このzero-semantic transportに`spec_gap`はない。

## Authorityとprotected meaning

Authority orderはcanonical Chapter 13 §§13.4.2/13.4.4/13.8.6、existing nested-capture
`.miz`、sole trace row、inactive expectation、completed C4C2/C4C3/C4C4/C4C5、その後に
derived owner docs/source inventory。

Frozen meaningは次のまま。

- inner mapper `x@94..95`はouter generator `x@136..137`のresolved binding identityを参照する。
- inner generator `y@102..103`はlocalでcaptureしない。
- associationはspellingやchecker/resolver/future Core numeric ID equalityでなくresolved identity。
- C4C4 outer-x projectionはby-value、captured stateはempty。
- C4C5 `source_ordinal == 0`はexact association coordinateだけで、general capture/Core
  parameter/application-argument orderではない。
- Task277Bはnot-ready/zero-credit。

Protected `.miz`/expectation/trace SHA-256は順に
`c3b8bd62c16406ccedee2e64a71ef62a5c4b329d2319be33ad3834a9541af431`、
`9ed000a30c1d519bd665f338c636fb9e529e9848a285209bebe6728f19961b92`、
`d4d817e83aac78d19e729702b26c62604fc57581eec18672a5c26ec44efe7a81`。
byte-identicalを維持する。

## Ownership、dependency、storage

Existing C4C5 handoffのsole producer/complete validatorは`source_formula_composition`のまま。
New crate-private typed-owner seamはC4C5 full validation、retained C4C3 pre-install
`TypedAst`の非公開取得、destination slotだけを除くcurrent final typed snapshotとの全field
equality、missing/foreign/stale/recovered/partial/additionally-populated final snapshot rejectionを行う。
C4C3 validationはalready-installed C4C6 ownerとpopulated resolved root、local context、type、
fact、coercion、initial-obligation、typed-diagnostic tableをrejectする。Installed receiptの
recursive retentionとsemantic typed stateのzero-semantic境界越えをともに禁止する。

両ASTはone boxed C4C5 handoffを持つcrate-private
`InstalledSourceNestedFraenkelCaptureIdentity` wrapperを格納する。C4C5はC4C3 `TypedAst`を
transitive retainするため、direct by-value slotはrecursively sizedとなり不可。Wrapperはprivate
construction/immutable borrow/`Clone`/`PartialEq`/`Eq`/concise custom `Debug`だけでpublic APIではない。
`TypedAstParts`と`ResolvedTypedAstInputs`にfieldを追加しない。

## Frozen public APIとerror

`TypedAst`はexact getter
`source_nested_fraenkel_capture_identity() -> Option<&SourceNestedFraenkelCaptureIdentityHandoff>`と
consuming one-shot installer
`with_source_nested_fraenkel_capture_identity(self, handoff) -> Result<Self, TypedAstError>`だけを追加する。
`ResolvedTypedAst`は同じread-only getterだけを追加する。Mutable/raw wrapper getter、`Default`、
replacement、adapter、conversion、independent resolved input、profile selector、unchecked installはない。

`TypedAstError::InvalidSourceNestedFraenkelCaptureIdentity`のexact displayは
`typed AST source nested Fraenkel capture-identity handoff is inconsistent`。
`ResolvedTypedAstError`のsame variant displayは
`resolved typed AST source nested Fraenkel capture-identity handoff is inconsistent`。
Diagnosticは追加しない。

## Installation/final-assembly oracle

Typed installはconsuming/one-shot/immutable/atomic。Complete handoff、empty slot、C4C5 retained
C4C3 snapshotとentire pre-install `TypedAst`のexact equalityだけをacceptする。Equalityはsource/module、
arena/root/resolved links、all source owner、context/type/fact/coercion/initial-obligation/diagnostic、
recoveryをauthenticateし、source/module equalityだけでは不足する。

Every existing public `TypedAst::with_source_*` installerはC4C6 slot present時にそのinstallerのexisting
errorでrejectする。Reverse orderではretained snapshotにないowner/tableをC4C6 installerがrejectする。
Sort/dedup/inference/repair/overwrite/merge/partial publicationは禁止。Test-only injectionはrejection
oracleだけに使う。

Final assemblyにindependent receipt inputはない。Typed slot present時はC4C5とexact final typed snapshotを
再validateし、empty cluster/overload/expression/node-hint inputとstatement semantic/proof bundle absentを
要求し、same immutable boxed ownerを`ResolvedTypedAst`へcloneする。Type fact/overload/coercion/obligation/
diagnostic/checked formula/statement/proof/capture/Core payloadはpublishしない。Failureはpartial ASTなしで
frozen errorを返す。C4C4 captured stateはsuccess/failure/clone/replayの全てでempty。

## Debugとboundary

Absent時は両debug byteを完全維持する。Present時はexisting C4C5 `debug_text()`とnewline 1件をexisting
`source_formula_composition`位置の直後、`source_condition_formula_composition`の前へexact once appendする。
C4C5 standalone debug grammarは変更しない。

本installationはchecker-owned receipt destinationだけ。`CoreVarId`、sethood/membership evidence、
mapper/predicate graph、generated owner/key/functor、`GeneratedOrigin`/use、parameter/argument、durable Core
provenanceは追加しない。Existing Core `Apply + GeneratedOrigin` representationにconsumerを追加しない。

## Tests、artifact、baseline

Checker exact testsはcanonical English contract記載の6件、existing private mizar-test leafはexact
`task257c4c6_real_imported_fixture_installs_typed_capture_identity_receipt` 1件を追加する。Library-test-onlyで
runner registry/active dispatchには入らない。

Test 4はtest-only injection seamでexact pre-C4C3 typed profileへC4C6 ownerを置き、
`SourceNestedFraenkelBinderUseProducer::build`がnew C4C3 handoffをpublishせず
`InvalidTypedDependency`を返すことを要求する。Ordinary existing-installer-after-C4C6 directionもtest 4、
C4C6-after-mismatched/additionally-populated typed ownerはtest 3がcoverする。Test 4はpopulated
semantic typed tableもC4C3 publication前にrejectする。Tests 5/6はrepresentative nonempty input
rejectionを含むsyntax-only final-input oracleと、success/replay/clone/failure pathでのempty captured
stateを直接保存する。

Production sourceは`source_formula_composition.rs`、`typed_ast.rs`、`resolved_typed_ast.rs`だけ。
Test sourceはexisting private nested-capture leafだけ。Owner docsは本contract pair、crate-plan row、checker
typed/resolved/source-composition、checker todo、paired module-boundary/source-spec audit、mizar-test harness、
paired checker/mizar-test bilingual audit、zero-credit coverage auditだけ。

Clean `HEAD ffc882675141a3e25bc78a47affc018bfe3685e1` baseline:

| Path | Lines | SHA-256 |
|---|---:|---|
| `typed_ast.rs` | 6897 | `673ca701208e051071997dc3649628af2ed9344bff6e6be78ba9871e717762ba` |
| `resolved_typed_ast.rs` | 8908 | `c89d138f843885c8ea49139ce742f0e4b78bd0c5abc6865d4f9362b9f3ba68ae` |
| `source_formula_composition.rs` | 9940 | `1b4efce50a86f36357478f1dcf98f64bda96a710de6ed1b8caa79e056cc3a515` |
| private mizar-test leaf | 519 | `4c403bdc7b060e52b5ba6585b82d5f34485813a49d4d035ac7214239206b72cf` |

Paired module-boundary auditは`1918/1779` lines、SHA-256
`86accc2e478137ebae57c3851d726a9163de5be03e386e1257a0177bd6bbe558` /
`258fba5760d404dccbcea0f53979f520fc8ce12994e88ba7f7d68e3cc641621b`。
Paired source-spec auditは`6300/5946` lines、SHA-256
`abbb8deffe73a7e286688e09d144555258e2be9f892657f6f416f530825f722e` /
`28aedf8ccccbfac26ea5975c4c7172ceccc8ab2a7f06aecc0701e69fe9e024ec`。
Owner-local inventory/public-enum claimを維持して4 audit hashすべてがchangeする。

Checker bilingual auditは`1994/1840` lines、SHA-256
`47468c44fd462be1743f029dc7a1ba8573deedcc53dd84410b140189d9c969c4` /
`31c0df262189356e4571f0f45e727fc4c58667308b5b57011fde0b188d012436`。
mizar-test bilingual auditは`2008/1855` lines、SHA-256
`5945f15d7bca346c50ce4beff89f4cc8023ca26f98088ee6097bbcfe6e40e628` /
`01aea1f8a59bb43aeb36475008d969fdba406c2a7aeb7424e1db0ab8d6526e55`。
English-canonical logical parityを維持して4 bilingual hashすべてがchangeする。

Checker productionは32 paths、path-list hash
`9dc5b02f26679677e593ea755394d68533173d2be988b7ef1ddcfd84a41b9787`のまま。
Raw library countはchecker `566 -> 572`、mizar-test `622 -> 623`、paired contract countは
`100/100 -> 101/101`。Affected source hash 4件だけがchangeし、protected authority hash、checker path
count/hash、diagnostic、public module count、existing test name、active route countは不変。

`doc/design/spec_coverage_audit.md`はdurable Typed/Resolved receipt ownerについてzero-credit mapping 1件を
追加する。Trace row/status/backlink、`.miz` intent、expectation、diagnostic、active route、semantic result、
coverage creditは変更しない。

## Review、verification、exit、next handoff

Implementation前にindependent spec/equivalenceとbilingual/boundary/API review、実装後にtest-sufficiency、
implementation、source/docs/API reviewをno blocking/high findingsまで行い、repair後はfinding-specific
re-reviewする。Focused 7 tests、checker/mizar-test library/lint、metadata、fmt、warnings-denied full Clippy、
full `cargo test`、diff check、protected hash/count/path hash/Task277B、exact staging、task-only commit、clean
postcommit proofを通す。Final reviewは`9/9` hard gateと`>=90/100`を要求する。

## Precommit implementation completion checkpoint

Final source measurementはcanonical English contractの4-path tableどおりで、checker productionは
`32` paths / `197561` lines、path/content-manifest SHA-256は
`9dc5b02f26679677e593ea755394d68533173d2be988b7ef1ddcfd84a41b9787` /
`a3d99114263d46552a59a14055e60b5938c683a4dd555423a1bc409335712ccc`。Contract treeは
`101/101`、checker/mizar-test raw library test-list hashは
`53472bd49c9f8d6cb2c6950aaa805a9652375e24953a489f1d3497ac6d97ab8a` /
`0a75e7fad8a2cbb0883b62a172163457f6fc66b8a28004b1f741567233f2348b`。

Final paired auditはmodule-boundary `1930/1790`、source-spec `6301/5947`、checker bilingual
`2009/1853`、mizar-test bilingual `2019/1865` lines。Exact SHA-256はcanonical English
checkpointがsole ownerである。

Initial independent reviewはpre-C4C3 injection、final-input contamination、captured-empty、runner
debug/clone、semantic-table guardの不足を指摘した。Frozen API/test name/count/boundaryを変えず
全てrepairし、specification、test-sufficiency、implementation、source/docs/API、bilingual/
boundary finding-specific re-reviewはすべて**NO FINDINGS**。

Focused `6/6 + 1/1`、library `572/572 + 623/623`、lint `15/15 + 15/15`、metadata
`137/137`、public-enum `2/2`はPASS。Package/full-workspace warnings-denied Clippy、fmt、full
`cargo test`、Cargo metadata、diff checkもPASS。Protected `.miz`/expectation/trace hashはexact、
`doc/spec`、protected artifact、`mizar-core`はdiff zero。C4C4 captured stateはempty、Task277Bは
not-ready/zero-creditで、semantic/route/diagnostic/Core/GeneratedOrigin surfaceは不変。Independent
final-quality reviewは**NO FINDINGS**、score capなしの`9/9` hard gate / valid uncapped
`100/100` (`20/20/15/15/10/10/5/5`)。Finding-specific exact-scope correctionはapproved
`29` paths = tracked modification `27` + new paired contract `2`を確認した。Exact staging、commit、
clean postcommit proofが上記implementation snapshotのnext gateだった。

## Postcommit proofとfresh successor inventory

Reviewed task-only implementationはbaseline
`ffc882675141a3e25bc78a47affc018bfe3685e1`上の
`b17cbfe5dad0bcb11502b4c7feef814df6adf8fb`としてcommitした。
`git show --check`はPASS、immediate worktreeはclean、`origin/main...HEAD`は`0/1`。
Protected stash `f65cf4a13752ec380710814a9ac6392ccb9d75d4`、authority hash 3件、
source measurement、contract count、C4C4 empty captured state、Task277B
not-ready/zero-creditは不変。Commitは自己hashを含められないため、closure-record commit hashと
その後のclean proofはfinal handoffで報告する。

Freshなauthority/checker/Core independent inventoryは、同一milestone内の一意なsuccessorを
選択しなかった。Canonical Chapter 13はresolved binder identityによるcaptureと、generated
`params`がsurrounding free variablesであることを固定するが、一般化したcaptured-parameter order
またはapplication-argument orderを固定しない。Exact fixtureからcaptured outer `x` 1件は導けるため、
そのfixture内ではorderがvacuousだが、multi-capture ruleのauthorityにはならない。Core Task 33は
future Core context/binder identity/provenance、Core Task 35はfuture term/formula/generated-origin
loweringを所有し、未完了のCore Tasks 33/34へ依存する。Accepted Core descendant contractは
両taskがchecker-owned / syntax-free / source-ordered final projectionをconsumeすることを既に
要求する。しかしexact resolver-binding-to-`CoreVarId` joinまたは
captured-parameter-to-application-argument positional joinはassignしない。

Current explicit Core APIは`params: Vec<CoreVarId>`と
`args: Vec<CoreTermSeedId>`を独立に受け取る。Argument orderを保持し、reused parameter equalityは
checkするが、checker/resolver identityから`CoreVarId`へのauthenticated map、parameter/argument
cardinality、positional correspondenceを認証しない。`GeneratedOriginUse`はlowering outputであり、
durable `CoreIr` tableではない。C4C6はこれらのfield/ownerを意図的に供給しない。

残るcandidateは次のとおり。

| Candidate | Boundary assessment |
|---|---|
| Checker-owned complete/source-ordered final projectionを作り、後でCore-33/Core-35 consumerがconsume | **推奨かつaccepted Core descendant contractと整合。** `CoreVarId`を含めずauthenticated binder identityとcomplete generator/mapper/predicate provenanceを運ぶ。ただしexact field/cardinality/generalized capture order/corruption oracleはhuman freezeが必要。 |
| Existing exact C4C6 receiptをcomplete Core projectionとして扱う | Minimalだが、C4C6はinner generator `y`、complete term graph、generated owner/key/functor、params/args、generalized order ruleを意図的に省く。 |
| Missing associationをCore 33またはCore 35で直接allocate/infer | Current Core inputはcaller-assigned `CoreVarId`を受け取り、このsource joinをauthenticateしない。そこでのreconstructionはchecker-final-projection boundaryに違反し、Core 35では未完了Core 33/34 dependencyも迂回する。 |

Unassigned exact joinと競合final-projection surfaceは`design_drift`、general cardinality/order/
mapping/corruption test欠落は`test_gap`。One-row `source_ordinal`、checker/resolver numeric ID、またはcurrent
Core vector orderをmissing ruleとして扱うことは`boundary_violation`。Canonical ordering ruleは
absentであり、next contractがそのorderをprivate alpha-invariant Core conventionではなくnormative
transportの一部にする場合、この欠落は`spec_gap`でもある。Authority contradictionと
repository-metadata conflictはない。

したがって、このinventoryはtask ID、API、field、adapter、installer、route、semantic
implementationを作らない。Successorに必要な最小human decisionは、Core IDを含まないcomplete
Core-facing projectionのsole ownerをcheckerとしてfreezeし、distinct captured identityをその
authenticated binder declaration/source orderで並べ、later Core consumerにgenerated parameterと
application argumentの同一position保持およびexact default-deny mismatch checkを要求するかどうかである。

`doc/spec`、existing `.miz`/expectation/trace、diagnostic、C4C4 captured state、Task255、active behavior、
semantic result、Task277B readinessを変更しない。Actual capture semantics、Core33/35 transport、numeric-ID
reinterpretation、display-name join、parameter/argument order、generated origin、sort/repair/inference/
unchecked admissionを追加しない。

Fresh inventoryは上記理由によりsuccessorを選択しなかった。C4C5 `source_ordinal`をgeneral
parameter orderへ昇格してはならない。
