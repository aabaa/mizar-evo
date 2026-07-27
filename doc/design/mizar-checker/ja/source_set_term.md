# ソース set/choice/qua-term transport

> 正本は英語です。英語版:
> [../en/source_set_term.md](../en/source_set_term.md)。

## スコープ

Checker Task 255 は、source set enumeration、condition-free independent set
comprehension、choice term、`qua` term の syntax-free immutable 記述を所有する。
source shape、transparent wrapper、written comprehension generator、bare builtin
target-type site、ordered child edge、unresolved request intent だけを運ぶ。
comprehension variable の bind/capture/condition 解決、sethood/nonemptiness、
choice witness、`qua` reachability、result type、fact、definition acceptance、
proof/IR loweringは行わない。

正本の言語要件は Chapter 13 §§13.4-13.6、Chapter 7 §7.8.1、Chapter 8
§8.2.2 と Chapter 17/21 のsemantic dependencyである。Task 252はprimary child、
Task 253はapplication child、Task 254はstructure-family childを所有し、Task 255は
rowを複製せずdense root IDを参照する。comprehension binding/captureはTask 257、
condition formula ownershipはTasks 256-257、request resolutionはlater semantic
ownerに残る。

## Public transaction

`SourceSetTermProducer::build`は`SourceSetTermHandoffInput`、`BindingEnv`、
`SourcePrimaryTermHandoff`、optional `SourceFunctorApplicationHandoff`/
`SourceStructureHandoff`、`TypedArena`を受ける。入力は6個のsource-ordered
vectorを持つ。

- set/choice/`qua` term
- transparent set-term wrapper
- written comprehension generator
- term-ownedまたはgenerator-owned bare target-type site
- ordered enumeration-element/comprehension-mapper/`qua`-base edge
- unresolved result-type/generator-sethood/choice-nonempty/`qua`-widening request

transaction全体をvalidateした後だけ6個のdense immutable tableをpublishする。
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
