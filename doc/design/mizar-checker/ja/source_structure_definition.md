# Source structure-definition transport

> canonical languageは英語である。canonical文書:
> [../en/source_structure_definition.md](../en/source_structure_definition.md)。

## Task 263 scopeとauthority

Checker Task 263は、2個の通常zero-parameter structure宣言、field/property
selector宣言、1本のdirect inheritance edge、exact member coverage、fields-only
constructor順序、root/path/view member associationのsyntax-free immutable intakeを
所有する。canonical authorityは`doc/spec/en/05.structures.md` §§5.1--5.8、
constructor/selector declaration boundaryに限るChapter 13 §§13.3--13.3.3、
definition-correctness/initial-obligation boundaryに限るChapter 16
§§16.6--16.6.4、inheritance-path/upcast-viability boundaryに限るChapter 19
§19.2.2である。winner/implicit conversionはdeferする。declaration identity/
provenanceはChapter 5とcommitted Task-263R resolver supporting authorityに
groundする。既存structure parser fixture、active mixed mode/structure gapと
sidecar/trace、committed Tasks 248--262、Task 263R、Task 249Sはrepositoryの
authority順に従うsupporting authorityである。

fresh inventoryはmissing upper producer/consumerを`source_drift`とcanonical由来
`test_gap`、missing frozen contractを`design_drift`と分類する。blocking
`spec_gap`はない。Task 263Rはfalse cross-structure selector conflictを、Task
249Sはstandalone member-type ownerを既に別commitで閉じた。Task 263はそれらの
public APIをconsumerとして使うだけで、lower ownerを書き換えない。

Task 263はauthenticated declaration shapeだけを運ぶ。structure、constructor、
selector、inheritance、redefinitionのacceptance、upcast選択、property constructor
argument、member identity推論、guard/goal生成、identical type向けobligation、proof
discharge、fact/axiom、Core/CFG/VCを作らない。

## Frozen exact sourceとabsence

future active sourceはfinal LFを含めてexactly次の通りである。

```mizar
definition
  struct Task263Base where
    field carrier -> set;
    property marker -> set;
  end;

  struct Task263Derived where
    field carrier -> set;
    property marker -> set;
  end;

  inherit Task263Derived extends Task263Base where
    field carrier from carrier;
    property marker from marker;
  end;
end;
```

320 bytes、16 lines、SHA-256は
`078eaee4b17341c9d8ebeb8a1f631ca984873bd07eb4e5d9c1a9486b39ac6671`である。
1 normal definition block、2 zero-parameter structures、4 independently
written bare builtin-`set` member types、1 direct inheritance、2 explicit
same-spelling mappingsだけを含む。`let`、reserve/default、binding context、
inherited parameter、rename/narrow、default field、second parent、diamond、cycle、
recovery、constructor/selector/update term、property implementation、correctness
clause、theorem、proof、diagnosticはない。

parameter/contextはexactly absentとfreezeする。Task 263は
`SourceBindingContextHandoff`を受けず、`BindingId`/`BindingContextId`を保存せず、
positive parameter rowを公開しない。parameterized structureはfuture separate
contractであり、このsourceにcontextをfabricateすることは`boundary_violation`である。

## Frozen Surface oracle

frontend diagnosticsは0、dense Surface rowsは75、rootはnode 74 / `0..319` /
normalである。leaf token rows 0--49とstructured rows 50--74のtoken text、range、
kind、childrenはcanonical EN文書の2 tableとexactly一致する。主要なstructured
associationは次の通りである。

| Node | Kind | Range | Children |
| ---: | --- | --- | --- |
| 50/58 | `StructurePattern` | `20..31` / `109..123` | `[2]` / `[17]` |
| 53/56 | base field/property | `42..63` / `68..91` | `[4,5,6,52,8]` / `[9,10,11,55,13]` |
| 57 | base `StructureDefinition` | `13..98` | `[1,50,3,53,56,14,15]` |
| 61/64 | derived field/property | `134..155` / `160..183` | `[19,20,21,60,23]` / `[24,25,26,63,28]` |
| 65 | derived `StructureDefinition` | `102..190` | `[16,58,18,61,64,29,30]` |
| 66/67 | inheritance targets | `202..216` / `225..236` | `[32]` / `[34]` |
| 68/69 | field/property redefinition | `247..274` / `279..307` | `[36,37,38,39,40]` / `[41,42,43,44,45]` |
| 70 | `InheritanceDefinition` | `194..314` | `[31,66,33,67,35,68,69,46,47]` |
| 71/72/73/74 | block/list/unit/root | `0..319` | canonical EN rows |

private runnerは全320 bytes、final LF、75 rowのkind/range/recovery/ordered children、
root、sibling orderを認証する。checker productionにはsource text、parser type、raw
token、Surface kind/node numberを渡さない。

## Frozen resolver provenance

Task-263R後のexact resolver profileはshell/projection/symbol/definition/diagnostic
`10/8/8/8/0`、local contribution 1である。shellはblock 71、structure 57、field
53、property 56、structure 65、field 61、property 64、inheritance 70、field
redefinition 68、property redefinition 69の順で、source ordinal 0--9を持つ。

| Role | kinds | Definition | Range | Path |
| --- | --- | ---: | --- | --- |
| base structure | `Structure/Structure` | 0 | `13..98` | `[4,0,11,0]` |
| base carrier | `Selector/Selector` | 1 | `42..63` | `[4,0,11,0,18,0]` |
| base marker | `Selector/Selector` | 2 | `68..91` | `[4,0,11,0,19,1]` |
| derived structure | `Structure/Structure` | 3 | `102..190` | `[4,0,11,1]` |
| derived carrier | `Selector/Selector` | 4 | `134..155` | `[4,0,11,1,18,0]` |
| derived marker | `Selector/Selector` | 5 | `160..183` | `[4,0,11,1,19,1]` |
| field mapping | `Redefinition/Redefinition` | 6 | `247..274` | `[4,0,20,2,21,0]` |
| property mapping | `Redefinition/Redefinition` | 7 | `279..307` | `[4,0,20,2,22,1]` |

全rowはnormal、local、public/exported、non-overloadable、conflict-free、contribution
0である。checkerはsymbol/definition/contribution/origin pairをexactに認証し、FQN
textやopaque signatureからowner/member identityを再構築しない。

## Frozen lower bundle

存在するlower handoffはTask-249+249S `SourceTypeApplicationHandoff`だけで、
applications/expressions/arguments/definition returns/mode RHS/structure members
は`0/4/0/0/0/4`である。member IDs 0--3はnodes 53/56/61/64とroots 0--3を所有し、
全expressionはargument-free bare normal builtin `set`である。Task 263はcomplete
lower `debug_text()`をfingerprintにし、`SourceTypeStructureMemberId`だけでlinkする。

Task 248はparameter/contextがないためabsent、Task-249R/249Mも同じhandoff内で
absentである。term/application/structure-term/set/formula/evidenceとTasks 259--262
definition handoffはすべてabsentである。runnerは既存producerをcomposeするだけで、
binding/context/application/return/mode-RHS/member type/resolver relation/semantic
resultをfabricateしない。

## Exact public syntax-free ABI

new `source_structure_definition.rs`は次のdense IDsを追加する。

```rust
SourceStructureDefinitionId
SourceStructureMemberId
SourceStructureInheritanceId
SourceStructureMappingId
SourceStructureCoherenceRequestId
```

全IDはvector-order denseで`new/index`だけを公開する。caller inputは
`SourceStructureDefinitionHandoffInput { source_id, module_id, definitions,
members, inheritances, mappings }`である。EN canonical ABIに従い:

- definition inputはresolver identity、site/range/ordinal/recovery/spelling、
  ordered `members`、fields-only `constructor_fields`を持つ。
- member inputはresolver identity、owner/ordinal、`Field|Property`、site/range/
  recovery/spelling、Task-249S `written_type`、optional constructor ordinalを持つ。
- inheritance inputはchild/parent、site/range/ordinal/recovery/spelling、ordered
  mappingsを持つ。
- mapping inputはredefinition resolver identity、inheritance/ordinal/kind、
  `view_member`/`parent_member`/`root_member`、explicit path、site/range/recovery/
  spellingを持つ。

public non-exhaustive enumsは
`SourceStructureMemberKind::{Field,Property}`、
`SourceStructureCoherenceRequestKind::MemberTypeInclusion`、
`SourceStructureDefinitionRecovery::{Normal,Degraded}`である。callerはparameter/
context、coherence request、obligation、type verdict、chosen upcast、origin、
fingerprint、fact、acceptance、proof、diagnosticを供給しない。

immutable rows/tables/handoffはcanonical ENのsame-named read-only getterだけを
公開し、5 tablesは`get/iter/len/is_empty`だけを持つ。handoffはsource/module、
complete source-type fingerprint、baseline obligation count、definitions、members、
inheritances、mappings、derived coherence requests、deterministic `debug_text()`を
公開する。

handoff private stateはimmutable
`base_initial_obligations_snapshot: InitialObligationTable`も保持する。public getter
はなくcaller-suppliedでもない。producerがauthenticated baselineからcloneする。
public countはcompact cardinality invariantでありsnapshotの代替ではない。

producer ABIは次である。

```rust
SourceStructureDefinitionProducer::build(
    input,
    env: &SymbolEnv,
    source_type: &SourceTypeApplicationHandoff,
    base_initial_obligations: &InitialObligationTable,
    arena: &TypedArena,
) -> Result<SourceStructureDefinitionProjection, SourceStructureDefinitionError>
```

projectionはbaseline clone、handoff、final obligation cloneを持つ。error variants
はsource/dependency、resolver、definition/member/inheritance/mapping/coherence、
obligation、arena、unsupported shapeをcanonical EN名でfreezeする。new
`InitialObligationKind`、public diagnostic、mutable/replacement APIは追加しない。

## Public Enum Policy

| public enum | compatibility policy |
| --- | --- |
| `SourceStructureMemberKind` | `#[non_exhaustive]`; later member classにはseparate canonical authority/testが必要 |
| `SourceStructureCoherenceRequestKind` | `#[non_exhaustive]`; later coherence-request classにはseparately frozen semantic ownerが必要 |
| `SourceStructureDefinitionRecovery` | `#[non_exhaustive]`; callerはlater recovery classをtolerateする |
| `SourceStructureDefinitionError` | `#[non_exhaustive]`; callerはvalidation failureをexhaustive matchしない |

この module が所有する exhaustive public enum exception はない。

## Exact rows、constructor、root/path/view

active profileは`2/4/1/2/0`である。

| Row | Exact association |
| --- | --- |
| definitions 0/1 | resolver defs 0/3、sites 57/65、ordinals 0/1、members `[0,1]`/`[2,3]`、constructor fields `[0]`/`[2]` |
| members 0/1 | base carrier field / marker property、resolver defs 1/2、spellings `field carrier -> set;` / `property marker -> set;`、written types 0/1、constructor ordinals `Some(0)`/`None` |
| members 2/3 | derived carrier field / marker property、resolver defs 4/5、同じexact statement spellings、written types 2/3、constructor ordinals `Some(0)`/`None` |
| inheritance 0 | child 1 -> parent 0、site 70、`194..314`、mappings `[0,1]` |
| mapping 0 | resolver def 6、Field、view/parent/root `2/0/0`、path `[0]`、site 68 |
| mapping 1 | resolver def 7、Property、view/parent/root `3/1/1`、path `[0]`、site 69 |

structure symbolがdefault-constructor declaration identityを兼ね、constructor
vectorにはfieldsだけがsource orderで入る。全member rowはpropertyを含むselector
declaration identityである。direct edgeは全parent/base membersをexactly once
coverし、kind/spelling/root/pathを保持する。このexact sourceでは各child viewも
1 mappingにparticipateするが、canonical parent-coverage ruleの代替ではなくbounded
shape checkである。

definition/inheritanceのexact substring spellingはEN canonicalの3 separate Rust
strings（surrounding indentation/final LFなし）であり、member/mappingはactive-row
tableのsingle statementとexactly一致する。

type pairs `2 -> 0`、`3 -> 1`はindependently writtenだがauthenticated shapeは
ともにbare builtin `set`である。spellingからtype equalityを推測せずlower rowを
比較する。Chapter 5 §5.3によりidentical typeはproofを要求しないため、derived
coherence request tableは0 rowsである。

## Initial obligationsとsemantic boundary

baseline lengthを`b`とする。projectionはbaselineをbyte-identical cloneし、handoff
にsecond immutable byte-equal snapshotと`b`を保存し、final tableも同じ
length/content/order/IDsで返す。existing owner
validatorに従うunrelated obligation kindsもbyte-preserveし、Task 263はkindをglobalに
claim/rejectしない。0 rowsをappendしnew kindを追加しない。baselineの任意row変更、
same-length row mutation、任意suffix、zero-request profileへのobligation associationは
fail-closedである。
orphan predicate/functor/attribute/mode domainはexisting absence validatorにより
invalidのままで、byte preservationはそのruleを迂回しない。

exact runner baselineは`b = 0`でprojection/handoff snapshot/finalはemptyである。
checker testsはnonempty unrelated baseline compositionとsame-length mutation rejectionも
検証する。buildはprojection baseline == private snapshot == final、Typed installは
current == projection baseline == private snapshot == final、final assemblyはcurrent
complete table == private snapshotかつ`len == b`を要求する。

このunchanged tableはacceptanceを意味しない。future nonidentical typeに対して
freezeされるのは`MemberTypeInclusion` requestだけである。guard、quantified goal、
assumptions、facts、proof/discharge、diagnostic、acceptance、obligation-kind ownerは
separate canonical authorityが必要で、Task 263は発明しない。

## Validation、determinism、Typed/final ownership

validationはwrong identity/arena、non-dense/cardinality、reorder/dangling/cross-owner/
cross-kind、parameter/context representationを必要とするexact shape/spelling drift、
site/range/ordinal/spelling/recovery、coverage欠落/重複、
property constructor、field order、wrong child/parent/root/path/view、cycle/self/second
edge/rename/narrow、stale resolver、stale lower fingerprint/root、nonzero coherenceを
deriveするtype relationまたはtest-injected coherence row、
obligation drift、partial/unsupported subtreeを拒否する。failureは全inputを不変に
保ち、partial handoffを返さない。global orderはsource identity、dependency identity、
cardinality/shape、resolver、definition、member、inheritance、mapping/coverage、lower
type、coherence、obligation、arenaの12 categoriesである。

debugはEN canonicalのexact no-blank-line grammarでheader
`source-structure-definition-debug-v1`、module、Rust-debug escaped complete
source-type fingerprint、baseline count、5-table profile、definition/member/
inheritance/mapping/coherence rowsをtable/dense-ID orderでrenderしfinal LFで終わる。
Stringは`{:?}`、ID listは`Vec<usize>`の`{:?}`（comma+single space）、active siteは
`node#<id>`、originはrange、recoveryはnormalである。active profile lineはexactly
`profile: definitions=2 members=4 inheritances=1 mappings=2
coherence_requests=0`で、`coherence-request#` lineは0件である。active module line、
全row field/order/escaping/none形式はEN grammarとexactly一致する。build/clone/
install/assembly/replayはbyte-deterministicである。

private obligation snapshotは`debug_text()`にrenderしない。Task 263はequality/
authenticationだけを所有しnew public obligation serializationを追加しない。countだけを
renderし、same-length corruptionはprivate equalityで検出する。

handoffはbuild時に認証したresolver `(symbol, definition, contribution)` identity 8件の
ordered snapshotもprivateに保持する。Typed/final replayはresolver envがscope外になった
後もdefinition/member/mapping各rowをこのimmutable snapshotと比較してからstructural
validationへ進むため、same-module symbol substitutionは通過しない。getterはなく、
`debug_text()`にはrenderしないが、public row identityは上記frozen grammarで完全に
renderする。

`TypedAst`だけが`with_source_structure_definition` transactionとgetter、
`TypedAstError::InvalidSourceStructureDefinition`を所有する。exact Task-249S lowerと
current/projection/private snapshot/final obligation equalityを認証し、Tasks 259 predicate、260 functor、
261 attribute、262 mode familyとのmixed occupancyを両方向で拒否する。Task 259の
correctness、facts、mixed predicate/functor boundaryを変更しない。

`ResolvedTypedAst::assemble`はtyped ownerからcloneしてfinal lower/obligationを
再認証し、同じgetterと`ResolvedTypedAstError::InvalidSourceStructureDefinition`だけを
追加する。replaceable inputはなく、types/facts/coercions/diagnosticsと既存definition
familyのsemantic outputを変更しない。既存installer 4件へのbounded changeは、
preinstalled Task-263 handoffを拒否するfrozen reverse-order guardだけである。

## Runner、tests、trace intent

implementationはexact sourceの新規pass pair
`pass_type_elaboration_structure_definition_payload_001.miz` / `.expect.toml`
とrequirement
`spec.en.checker.type_elaboration.source_structure_definition_payload`を1件だけ追加する。
sidecarはpass/type_elaboration/type_check、public diagnostics/payloads emptyである。
これはtransport creditだけで、acceptance/proof/fact/Core/CFG/VC creditではない。
既存mixed gap、既存`.miz`/sidecar/expectation、parser/resolver fixtureはbyte-identicalに
保つ。

checker tests 5件とrunner tests 4件のexact names、mutation coverageはEN canonical
documentのlistに従う。source/Surface/resolver/lower、coverage/constructor、root/path/
view、zero coherence、unchanged arbitrary baseline、arena、one-shot typed/final、debug、
sibling/route isolation、non-publicationを独立に検証する。

checker tests 2/3はsingle-fault-onlyにしない。frozen 12-category failure orderの全
adjacent pairについて2 faultsを同時に持つinputを作り、earlier categoryのexact error
variant/indexが勝つことをassertする。resolver-vs-definition、definition-vs-member、
member-vs-mapping/coverage、mapping-vs-lower-type、obligation-vs-arenaではlater-row
higher-priority faultとearlier-row lower-priority faultも組み合わせ、row orderが
category orderを上書きしないことを証明する。existing five test names/count内で行う。

## Counts、scope、deferrals、exit

docs prerequisiteはRust、fixture、sidecar、expectation、trace row/status/count、test
list、production path、Cargo、CLI、hashを変更しない。baselineはchecker/runner/
resolver/syntax `462/524/146/59`、metadata `425/393`、pass/fail `232/193`、active
`101/7/202/1`、type `257 = 245 + 12`、warnings/errors `23/0`、checker production
`27/156019`とcanonical EN記載hashである。

implementation projectionはchecker `467`、runner `528`、metadata `426/394`、
pass/fail `233/193`、active `101/7/203/1`、type `258 = 246 + 12`で、resolver/syntax
`146/59`である。manifest/list/CLI/corpus/trace hashはfresh measureする。

implementation scopeはnew checker module/tests/export/Typed/final/source-spec/lint、
frozen bidirectional isolation matrix用cfg(test)-only predicate/functor/mode projection
fixtureとpredicate/mode test-module visibility、private runner leaf/tests/registerと既存
sibling test leaves 4件のcount-oracle-only update、new pass pair/trace row、同期EN/JA
auditsだけである。`doc/spec`、existing artifacts、parser/resolver、Task-249S、
frozen Task-263 mutual-exclusion guard以外のTasks 259--262 semantics/output、public
diagnostic、fact/proof/Core/CFG/VC/Cargo/unrelated metadataは変更禁止である。

parameterized/default/multiple-parent/diamond/cycle/rename/narrow、nonidentical coherence
goal/guard/obligation、use-site path choice、constructor/selector/update semantics、
extensionality/axioms/upcast/evidence/acceptance、Task 264 property implementation、facts、
diagnostics、proof/discharge、Core/CFG/VC、mixed meaningはdeferredである。

Task 263はdocs-only commit、fresh dependency inventory、exact implementation、4種の
NO FINDINGS review、9 hard gates PASS/no cap/90+、全verification、task-only staging/
implementation commit、clean HEAD/origin/stash確認をすべて満たした後だけ完了し、
停止せずdependency-ordered Task 264+へ戻る。

## Active implementation result

上記exact API/consumerをsemantic expansionなしに実装した。checker 5件、runner 4件は
PASSし、12-category precedence orderのadjacent pair 11件をfrozen test names内で
exerciseする。library/countは`467/528/146/59`、metadata `426/394`、active type
`203`、sole new trace rowはsole new pass sidecarでcovered。independent reviewは
**NO FINDINGS**、全9 hard gatesはscore capなしの`100/100`、全verificationは
PASSである。exact staging/commit/clean post-commit inventoryのみpendingである。
