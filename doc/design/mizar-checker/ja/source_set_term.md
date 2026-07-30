# ソース set/choice/qua-term transport

> 正本は英語です。英語版:
> [../en/source_set_term.md](../en/source_set_term.md)。

## スコープ

Checker Task 255 は、source set enumeration、0または1件のfrozen source
conditionを持つindependent set comprehension、choice term、`qua` termの
syntax-free immutable記述を所有する。source shape、transparent wrapper、
written comprehension generator、bare builtin target-type site、direct
condition-wrapper provenance/spelling、ordered child edge、unresolved request
intentだけを運ぶ。comprehension variableのbind/capture、inner condition formula
の解決、sethood/nonemptiness、choice witness、`qua` reachability、result type、
fact、definition acceptance、proof/IR loweringは行わない。

正本の言語要件は Chapter 13 §§13.4-13.6、Chapter 7 §7.8.1、Chapter 8
§8.2.2 と Chapter 17/21 のsemantic dependencyである。Task 252はprimary child、
Task 253はapplication child、Task 254はstructure-family childを所有し、Task 255は
rowを複製せずdense root IDを参照する。comprehension binding/captureはTask 257、
condition formula ownershipはTasks 256-257、request resolutionはlater semantic
ownerに残る。

## Public transaction

`SourceSetTermProducer::build`は`SourceSetTermHandoffInput`、`BindingEnv`、
`SourcePrimaryTermHandoff`、optional `SourceFunctorApplicationHandoff`/
`SourceStructureHandoff`、`TypedArena`を受ける。入力は7個のsource-ordered
vectorを持つ。

- set/choice/`qua` term
- transparent set-term wrapper
- written comprehension generator
- term-ownedまたはgenerator-owned bare target-type site
- direct condition wrapperとterm-owned colon provenance
- ordered enumeration-element/comprehension-mapper/`qua`-base edge
- unresolved result-type/generator-sethood/choice-nonempty/`qua`-widening request

transaction全体をvalidateした後だけ7個のdense immutable tableをpublishする。
public IDはzero-based `new`/`index`、tableは`get`/source-ordered `iter`/`len`/
`is_empty`、validated rowはcrate planでfreezeしたread-only accessorだけを持つ。

term kindは`Enumeration`/`Comprehension`/`Choice`/`Qua`、recoveryは
`Normal`/`Degraded`、type headはbare `BuiltinSet`/`BuiltinObject`である。
targetはTask-252 `Primary`、Task-253 root `Application`、Task-254 root
`Structure`、later nested Task-255 `SetTerm`である。

## Public Enum Policy

| Public enum | compatibility policy |
|---|---|
| `SourceSetTermKind` | `#[non_exhaustive]`。callerはlater frozen set-family source kindを許容する。 |
| `SourceSetTermRecovery` | `#[non_exhaustive]`。callerはlater recovery classを許容する。 |
| `SourceSetTypeOwner` | `#[non_exhaustive]`。callerはlater target-site ownerを許容する。 |
| `SourceSetTypeRole` | `#[non_exhaustive]`。callerはlater term-owned target roleを許容する。 |
| `SourceSetTypeHead` | `#[non_exhaustive]`。callerはlater frozen bare builtin headを許容する。 |
| `SourceSetEdgeRole` | `#[non_exhaustive]`。callerはlater child-edge roleを許容する。 |
| `SourceSetTarget` | `#[non_exhaustive]`。callerはlater frozen cross-family targetを許容する。 |
| `SourceSetRequestKind` | `#[non_exhaustive]`。callerはlater unresolved request kindを許容する。 |
| `SourceSetTermError` | `#[non_exhaustive]`。callerはvalidation failureをexhaustive matchしない。 |

この module が所有する exhaustive public enum exception はない。

## 検証とownership

producerはsource/module identity、dense source preorder、context、range、recovery、
exact typed-arena anchor、group/ordinal、canonical token spelling、single ownershipを
認証する。arena keyは`source.term.set.enumeration`、`.comprehension`、`.choice`、
`.qua`、`.parenthesized`、`.comprehension-generator`、`.target-type`、
`.target-type-head`である。

canonical spellingはauthenticated rowからrecursiveに再構成する。enumeration
elementは`{ }`内で` , ` join、comprehensionはmapper、` where `、
`identifier is type` generator fragmentをjoinする。choiceは`the type`、`qua`は
`base qua type`、wrapperは`format!("( {} )", contained_spelling)`である。
generator spellingはlexer identifier 1個、bare type expression/headは両方とも
exact `set`/`object`である。

enumerationはelement edge 0件以上とfinal result request 1件を持つ。
comprehensionはgenerator 1件以上、generatorごとのbare type site/sethood
request、mapper edge 1件、final result requestを持つ。choiceはtarget type site、
nonempty request、final result requestを各1件持つ。`qua`はtarget type site、
base edge、widening request、final result requestを各1件持つ。

各written child slotでTask-252/253/254/255 effective occurrenceを列挙し、
descendant除去後のmaximal occurrence 1件がslot全体をcoverしなければならない。
Task 253/254が既ownするprimaryとTask 254が既ownするapplicationは再targetできない。
nested Task-253/254/255 descendantはnearest family ownerに残る。Task-255 childを
含むreverse Task-253/254 parent、conditioned comprehension、generator-reference
comprehension、non-bare target、その他frozen exclusionはdetached descendantなしで
fail closedする。

## Derived dependency fingerprint

outputは常にexact Task-252 `debug_text()`から`primary_term_fingerprint`を導出する。
`application_fingerprint`/`structure_fingerprint`はexact dependency
`debug_text()`で、そのfamilyをedge targetにする場合だけ`Some`になる。unrelated
installed optional handoffは、そのeffective occurrenceが全Task-255 term/wrapper/
target rangeとdisjointな場合だけ`None`と共存できる。

`TypedAst::with_source_set_term`はone-shotでtargeted dependencyの先行installを
要求する。`with_source_application`/`with_source_structure`はinstalled Task-255
handoffを再検証し、install orderによるownership/fingerprint bypassを許さない。
`ResolvedTypedAst`はrowをrebuild/renumberせず同じassociationをrevalidateして
clone-preserveする。typed/resolved debug renderingはhandoffがpresentの場合だけ
含める。

## Private source consumer

raw `SurfaceAst`、source node ID、syntax kindは
`mizar-test::runner::type_elaboration::source_set_term`だけに置く。productionは
`fail_type_elaboration_local_set_choice_qua_term_gap_001`の4 functor definiensだけを
selectする。leafはTask 248のreal binding-context transactionとTask 252 primary
producerを再利用し、comprehension `BindingId`を捏造しない。

exact Task-255 term/wrapper/generator/type-site/edge/request oracleは
4/0/1/3/4/7、shared arenaのTask-252
primary/reference/numeric-request sliceは4/0/4である。real routeにTask-253/254
row/fingerprintはない。transport validation後はpublic diagnosticなしでTask-260
`type_elaboration.external_dependency.ast_payload_extraction` boundaryを保持する。

## Verification boundary

checker testは全table/enum、全arena key、canonical spelling、wrapper、
kind別cardinality/request association、cross-family nearest ownership、optional
dependency fingerprint、install order、corruption、determinism、clone preservation、
atomic failureをcoverする。runner testはexact consumer/oracle、real lower-stage
shape、zero/many enumeration、independent multiple/nested comprehension、choice、
`qua`、wrapper、degraded transport、cross-family child、exclusion、mutation isolation、
deterministic replay、final ownership、他の全active type-elaboration case isolationを
coverする。

bounded trace rowは
`spec.en.checker.type_elaboration.source_set_choice_qua_term_payload`である。
Task 255はexecutable source transport coverageだけを変更し、generator/capture、
formula、typing、evidence、fact、proof、Steps 6/7 semanticsは未実装のままである。

## Task 255C1 frozen condition-bearing-comprehension extension

Task 255C1は、このmoduleをindependent conditioned comprehension 1件だけに対して
6 tableから7 source-ordered tableへ拡張する。canonical Chapters 10、13、14と
既存parser fixturesがcrate planで凍結したexact 191-byte source/rangeを
authorizeする。new exact profileはTask-253 `1/0/1/2/2`とone immutable
Task-252 `4/0/4` handoff上の
term/wrapper/generator/type-site/condition/edge/request
`1/0/1/1/1/1/2`である。

`SourceSetConditionInput`とimmutable rowはowner term/ordinal、
Task-255-owned colon site/range/spelling、direct condition-wrapper site、
condition range/spelling、recoveryを保持する。colonはtyped-arena key
`source.term.set.comprehension-condition-colon`を使い、`condition_site`は
Task-255 association key `source.term.set.comprehension-condition`でdirect
`FormulaExpression`をanchorする。Task 255はそのwrapperをsubtree boundary
としてauthenticateするが、inner `BuiltinPredicateApplication` formula site/
rowはTask 256に残す。contextはowner termからderiveする。
`SourceSetConditionId`は`new`/`index`、tableは
`get`/`iter`/`len`/`is_empty`、row/handoffは全frozen fieldと
read-only `conditions()`を公開する。

conditionはdenseにgroupされ、comprehensionあたり0または1件だけで、final
generator typeより後にあり、canonical term spellingへ` : condition`を加える。
Task-255 edge/requestは作らない。authenticated condition range内に完全に含まれる
全lower-family occurrenceはTask-255 direct-child discoveryから除外する。exact C1
routeはそこにTask-252 numeral 3/4をTask-255 edgeなしで保持する。Task 256は
later equality edgeをownできるが、このextensionはformula handoffをinstallしない。
condition range外の既存nearest-family/whole-subtree exclusionはすべて不変である。

condition rowはtype siteの後、edgeの前にrenderする。

```text
condition#<id> term=<term> ordinal=<n> colon_range=<s>..<e> colon_site=<site> colon_spelling=<quoted> condition_site=<site> range=<s>..<e> spelling=<quoted> recovery=<key>
```

empty condition tableは何もrenderせず、legacy debug byteをすべて保持する。既存16
input literalはempty vectorを受け取り、`to_input`はnonempty rowを
clone-preserveする。

checker corruption coverageはomitted/copied/out-of-range/wrong-kind
condition site、omitted/copied/out-of-range condition primary、
condition-contained Task-253/254/255 descendantを明示的にrejectする。
condition `FormulaExpression` siteをTask 255がownする一方、そのinner
`BuiltinPredicateApplication` formula siteをownしないことを証明し、condition
range直外のnearest-family ownership不変を再確認する。

private runnerはreusable unwrapped imported-`++` Task-253
extractor/builderを呼び、Task 252、Task 253、Task 255をone arenaへinstallする。
exact future fail sidecar/covered trace rowが証明するのはsource transportだけである。
generator binding/capture、inner condition-formula ownership/composition、
sethood/result answer、equality truth、definition acceptance、proof、IRはdeferredの
ままである。

## Task 255C1 implementation result

seven-table extensionをfrozenどおり実装した。condition rowはdirect
term-owned colon/wrapper siteをauthenticateし、wrapper subtree全体をrecorded
range内へ再帰的に制限し、contained Task-252 primaryをすべて要求し、
Task-253/254/255 descendantとcondition-directed edgeをrejectする。direct-child
discoveryから除外するのはauthenticated condition内lower-family rowだけである。
full debug、legacy empty-table byte equality、group/order/cardinality、
dependency substitution、rollback、final clone testsはpassする。

## Task 257C2 frozen consumer boundary

Task 257C2はexact Task-255C1 profileのcondition row 0を変更せずconsumeする。
rowはcolon/direct `FormulaExpression` wrapperだけを引き続きownする。
`177..182` rangeと`3 = 4` spellingは別のdirect Task-256 equality rowと一致
しなければならず、future Task-257C2 associationはsiteをownしない。
Task-252 primaries 2/3はTask-255 edgeからexcludedのままで、Task-256 equality
operandだけになる。Task-255 table/debug byte/fingerprint/request/validation
meaningは変更しない。frozen pre-Task-256C1 baselineでは、separate lower
taskがarbitrary overlap rejectionをweakenせず、このexact condition
containmentをTask-256 validatorの両installation orderでauthenticateする必要が
あった。Task 256C1は現在これを満たし、fresh Task-257C2 preflight/
implementationだけがprerequisite exit時点で残った。completed C2 routeは
condition rowを変更せずconsumeし、separate associationだけをownする。

## Task 256C1 frozen lower-owner boundary

Task 256C1はimmutable Task-255C1 condition rowをvalidation contextとしてだけ
consumeし、Task-255 table 7件を変更・再構築しない。term 0は
`139..184`の`Comprehension`、condition 0はそのtermのordinal 0で、
colon `175..176`、direct wrapper range `177..182`、spelling `3 = 4`、
normal recovery、owner-term context 0を保持する。condition siteはdistinct
Task-256 equality siteをdirect containし、そのcontextはexisting owner-term
contextと一致しなければならない。

condition rowへcontext/formula IDを追加せず、Task 256C1はimmutable owner
termからcontextをderiveする。Task 255はcolon/wrapper
boundaryだけをownし、Task 256はinner equality、Task 257C2はlater explicit
associationをownする。Task 256C1はTask-255 edge、fingerprint、debug、
request、semantic、validation schemaを変更しない。

## Task 256C1 implementation result

lower-owner boundaryは不変。Task 256C1はTask 256がoverlapping equalityを
checkするとき、既にvalidatedなimmutable condition rowだけをconsumeし、
Task-255 rowを追加・rewriteしない。exact conditioned profileは
`1/0/1/1/1/1/2`、debug/fingerprintは不変で、stale、wrong-owner、wrapped、
non-direct relationはTask 256でfail closedする。両installation orderはexact
authenticated relationだけをacceptする。

## Task 258B3M2B2B3P frozen proof-context enumeration reuse

B2C implementation commit
`e8373c683448e524cb98edde83fdf8de83a125cd`後、fresh inventoryはprivate
Task-255 set-enumeration reuse prerequisiteをselectする。exact 117-byte
final-LF source/hash/57-node mapはpaired crate planと一致する。set term 0は
site `Node(40)`、range `90..96`、source ordinal 0、proof context 1、
recovery `Normal`、spelling `{ 1 , 2 }`、kind `Enumeration`。
`EnumerationElement` edge 0はterm 0、ordinal 0、target `Primary(2)`
node/range `36/91..92`、edge 1はterm 0、ordinal 1、target
`Primary(3)` at `38/94..95`。request 0はterm 0、ordinal 0、kind
`ResultType`、`generator = None`、`type_site = None`。primary fingerprint
はexact Task252 handoff fingerprint、application/structure fingerprintsは
absent。comprehension/choice/condition/other Task255 rowsはない。

Task 48はcontexts 0/1、reserve binding 1、diagnostics zero。Task 252は
nodes `30/32/36/38/44/46`をownし、theorem/conclusion referencesと
numeric requests 2/3を持つ。Task 255はnode 40だけをown。Tasks
253/254/256/258はemptyで、term-expression/witness/statement/proof/
theorem/item/compilation/root containersはunowned。imported provenanceはない。

B3Pはexisting context-0 helperをbyte-for-byte preserveし、runner
`source_set_term` pathにexplicit-context private siblingだけを追加できる。
future tests exactly 2件は
`task258b3m2b2b3p_set_enumeration_proof_context_reuse_is_exact`と
`task258b3m2b2b3p_set_enumeration_corruption_replay_and_legacy_output_fail_closed`。
同じ2 testsでfinal LFを含む117 loaded-source bytes全件、stripped/
extra-LF variants、57 nodes全件のkind/range/recovery/ordered children各
field/root identity、local resolver全field/substitution、Task48全context/
binding fields、Task252全term/reference/numeric-request fields、Task255
全term/`EnumerationElement` edge/request/fingerprint fieldsをmutate/assert。
owned partition `{30,32,36,38,40,44,46}`とnode 56までのcomplement、
explicit validation precedence、stale-fingerprint replay、atomic rollback、
clean replay、exact final typed/resolved cloneをfreezeする。

Tasks253/254/256/258 empty、active/adjacent-family isolation、empty
semantic/proof/goal outputsもassert。legacy context-0はpreimplementation
Task111 literal hashes: handoff
`30b72230bb7ff39464962133b58df212e23afccccc8f4e4788ab9a9d0481c43a`、
typed `1bb296c06ab62691684260aa94987adee23081baa4a35aac9e485d95370d2cb9`、
resolved
`cdb4eaae9605f62269d6a74d64267a8fcb1e8d8008564d8b9e014037665df1e4`
をliteral assertし、old/new in-build equalityだけでは不可。

このlower taskはchecker row/API/statement witness/semantic claimを追加しない。
upper B3Aだけがlater `SourceStatementWitness -> SetTerm(0)`をown可能。
empty/singleton/3+、nested/parenthesized/comprehension/choice/`qua`、
sethood/element/result unification、existential/proof/goal/theorem behavior、
Tasks253/254/256/258はexclude。

## Task 258B3M2B2B3P documentation review status

specification/documentation、test-sufficiency、implementation-boundary、
source/documentation consistency reviewsはすべて**NO FINDINGS**。
exact source/hash、lint、library、production/test-list/CLI hashes、scope、
diff、trace no-op verificationはPASS。lower table/test oracleはfrozenで、
future private implementation `source_drift`/`test_gap`はplanned。
final quality、commit、post-commit、fresh inventoryはpending。

## Task 258B3M2B2B3P final quality status

final qualityは**NO FINDINGS**、全9 hard gates PASS、valid `98/100`
（`20/20/15/14/10/10/5/4`）。pendingはstage/commit、post-commit、
fresh implementation inventoryだけ。

## Task 258B3M2B2B3P implemented proof-context enumeration reuse

`285a1f11c310bb313c4c6b4feae914eb11f74754`のfrozen contractをexact
4 runner filesでimplementした。
`source_set_term_output_with_source_term_in_context`は`pub(super)`
explicit-context sibling、既存entry pointはcontext-0 delegate。
context-0 compatibilityはliteral handoff/typed/resolved hashes
`30b72230bb7ff39464962133b58df212e23afccccc8f4e4788ab9a9d0481c43a`、
`1bb296c06ab62691684260aa94987adee23081baa4a35aac9e485d95370d2cb9`、
`cdb4eaae9605f62269d6a74d64267a8fcb1e8d8008564d8b9e014037665df1e4`
で独立固定。

exact 2 testsは117-byte/LF、57-node surface fields、63-field resolver、
39-field binding、Task-252/255全field、coherent dependenciesのnon-None
fingerprintをrejectするshared fingerprint-only exact subprofile、
stale/simultaneous precedence、immediate clean replay、typed/resolved clone、
family/active isolationをcover。focused `2/2`、runner library `446/446`、
format/package Clippy/diffはPASS、test-sufficiency/implementation
reviewsは**NO FINDINGS**。source/docs consistency repeatとdocumentation/
boundary repeatも**NO FINDINGS**。lint-policy `15/14`、metadata `137`、
workspace Clippy/tests、5 CLI/current manifest/test-list hashes、exact
30-file scopeもPASS。

checker schema/API/semantic resultは追加なし。independent final qualityは
**NO FINDINGS**、全9 hard gates PASS、valid `98/100`
（`20/20/15/14/10/10/5/4`）。pendingはcommit/post-commit、fresh B3A
inventoryだけ。

## Task 258B3M2B2B3A frozen set-term consumer boundary

Task255/B3Pはbyte-exact不変。`SetTerm(0)`はnode/range `40/90..96`、
context1、ordinal0、`Normal`、spelling `{ 1 , 2 }`、`Enumeration`、
`EnumerationElement` edgesは`Primary(2/3)`、ResultType request 1、
primary fingerprint exact、application/structure fingerprints absent。
B3Aはunchanged `source_set_term_output_with_source_term_in_context`をconsume。

B3Aはset-term producer behaviorを追加せず、witness
`0 -> SetTerm(0) -> Primary(2/3)`とexact set fingerprint authentication
だけ。reverse/semantic edgeなし。両`source_set_term.rs`、result typing/
sethood/element unification、broader set formsはforbidden。

## Task 258B3M2B2B3A implemented consumer closure

B3Aはexisting runner seamからunchanged Task-255/B3P handoffをconsumeし、
statement-witness handoffへexact debug fingerprintをrecordする。両
`source_set_term.rs`は変更しない。enumeration edges 2件とresult-type
requestはsource transportだけで、result typing、sethood/element
unification、imported/broader set form、semantic edgeのcreditなし。
focused/package testsとimplementation reviewsは**NO FINDINGS**/PASS。
2回目のsource/documentation consistency repeatとfinal documentation/
boundary rereadも**NO FINDINGS**で、crate plans記載のparent final
verificationはexact `39`-file scopeを含めPASS。independent final
read-only quality reviewは**NO FINDINGS**。全9 hard gates PASS、score
capなし、valid `98/100`（`20/20/15/14/10/10/5/4`）。記載済み
semantic/coverage deferralsはunchanged residual risk。pendingは
dedicated implementation commit、postcommit invariant verification、
fresh next-task inventoryだけ。

## Task 258B3M2B2B3B reused empty-enumeration lower contract

B3BはTask-255 schemaまたはproducer behaviorを追加しない。existing
explicit-context extractorは`33/95..97`にexactly 1件の`Enumeration` term、
spelling `{ }`、context 1、wrapper/generator/type site/condition/edge 0件、
`ResultType` request 1件をyieldしなければならない。existing
zero-element producer testsがlower authorityのままであり、
`source_set_term.rs` source/test changeはauthorizeしない。B3Bはseparate
upper statement/witness consumerだけをownし、choice、comprehension、
`qua`、全semantic requestsをlater workとして保持する。

## Task 258B3M2B2B3B implementation lower-contract no-op

implementationはexisting B3A `SetTerm(SourceSetTermId)` targetとTask-255
zero-edge producer outputをconsumeしただけで、`source_set_term.rs`または
そのtestsを変更しない。8 base-resolver mutationsとnon-vacuous
zero-edge corruptionをupper testsでverifyし、lower ownershipを拡張して
いない。producer schema、fingerprint grammar、semantic requests、trace
creditはunchangedである。

post-auth injectionとstage-prefix/non-generic-guard assertionsはupper
consumer testsだけであり、lower ownershipはsource/test no-opのまま。
全test-sufficiency repeatsとfinal implementation repeatは
**NO FINDINGS**である。

## Task 258B3M2B2B3C reuse contract

Task-255 source changeは不要。frozen handoffは`1/0/0/1/0/0/2`。
`35/82..89`の`Choice` term 1件、expression/head `34/33/86..89`の
`ChoiceTarget` `BuiltinSet` type site 1件、child edge 0、その後
`ChoiceNonempty(type-site 0)`と`ResultType`。contextはproof context `1`、
application/structure fingerprintsはabsent。future B3C testsは全`39`
safely mutable input fieldsをmutate/replayし、generic dependency failureで
なくTask-255 stage errorをrequireする。

## Task 258B3M2B2B3C reused choice consumer

B3Cはunchanged exact Task-255 handoff、すなわち`Choice` 1、builtin-set
`ChoiceTarget` type site 1、wrapper/generator/condition/edge 0、ordered
`ChoiceNonempty`/`ResultType` requestsをconsumeする。両
`source_set_term.rs` owner/schema/producer/testは変更しない。upper witnessは
existing set fingerprintと`SetTerm(0)` targetだけをrecordし、exact 39-field
replay matrixはTask-255-owned error precedenceをconfirmする。choice
nonemptiness/stable symbol/type factと全semantic creditはdeferred。

## Task 258B3M2B2B3D qua reuse contract

Task-255 source changeは不要。frozen handoffは
`1/0/0/1/0/1/2`: `37/79..88`の`Qua` term 1件、
expression/head `36/35/85..88`のterm-owned `QuaTarget`
`BuiltinSet` type site 1件、`QuaBase -> Primary(2)` edge 1件、その後
ordered `QuaWidening(type-site 0)`と`ResultType`。contextはproof
context `1`、application/structure fingerprintsはabsent。future B3D
testsは全`44` safely mutable Task-255 fieldsをmutateし、各resultを
replayしてTask-255-owned errorsをrequireする。両
`source_set_term.rs` ownersはunchangedで、wideningはdischargeしない。
