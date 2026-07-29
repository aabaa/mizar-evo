# ソース functor-application transport

> Canonical language: English. 正本:
> [../en/source_application.md](../en/source_application.md).

## スコープ

Checker Task 253 は、ソース functor application occurrence の syntax-free
かつ immutable な記述を所有する。運ぶのは source shape と未解決 dependency
だけである。candidate の applicability / completeness / viability /
ranking / winner、semantic signature / result type、functor definition や
inline substitution、fact、proof、CoreIr、ControlFlowIr、VC は所有しない。

正本となる言語要件は Chapters 10、13 §13.2、15 §15.2.3、19 である。
Task 252 は primary-term occurrence、binding reference、numeric request を
所有し、Task 253 は行を複製せず dense ID を参照する。inline semantics は
Task 270、template direct transport は Task 277、candidate collection /
selection は Task 278 に残る。

## Public transaction

`SourceFunctorApplicationProducer::build` は
`SourceFunctorApplicationHandoffInput`、`SymbolEnv`、`BindingEnv`、
`SourcePrimaryTermHandoff`、`TypedArena` を受け取る。入力と出力は次の
5 個の source-ordered dense table である。

- application
- transparent application wrapper
- 個別に認証した resolver functor reference
- Task-252 primary または後続 Task-253 application への ordered argument edge
- 未解決 candidate-signature / application-result type request

各 public ID は `new` / `index` を持つ zero-based row index であり、table
は `get`、source-ordered `iter`、`len`、`is_empty` だけを公開する。
transaction 全体が検証されるまで一行も公開しない。

kind は `Symbolic` / `Inline`、form は `Bare` / `Prefix` / `Infix` /
`Postfix` / `Bracket` / `Functional` である。Inline は Functional のみで
candidate/request を持たない。Symbolic は 1 個以上の個別 candidate、
各 candidate の signature request、最後の application-result request を
持つ。

## Public Enum Policy

| Public enum | compatibility policy |
|---|---|
| `SourceFunctorApplicationKind` | `#[non_exhaustive]`。callerはlater application-shape classを許容する。 |
| `SourceFunctorApplicationRecovery` | `#[non_exhaustive]`。callerはlater recovery classを許容する。 |
| `SourceFunctorApplicationForm` | `#[non_exhaustive]`。callerはlater written source formを許容する。 |
| `SourceFunctorHeadSite` | `#[non_exhaustive]`。callerはlater head-site shapeを許容する。 |
| `SourceFunctorArgumentTarget` | `#[non_exhaustive]`。callerはlater frozen cross-family targetを許容する。 |
| `SourceFunctorTypeRequestKind` | `#[non_exhaustive]`。callerはlater unresolved request kindを許容する。 |
| `SourceFunctorApplicationError` | `#[non_exhaustive]`。callerはvalidation failureをexhaustive matchしない。 |

この module が所有する exhaustive public enum exception はない。

## 検証と ownership

producer は source/module identity、dense preorder、group と ordinal、
context、非空 range、typed-arena anchor/recovery、canonical token spelling、
head position、delimiter、form/cardinality、wrapper nesting、argument order /
non-overlap、nested application の single incoming ownership を認証する。

Primary argument は同じ context にある Task-252 root row で application
内になければならない。inner descendant、duplicate ownership、partial
argument list、owner 未凍結の cross-family target は transaction 全体を
reject する。application を囲む括弧は outer-to-inner の Task-253 wrapper
であり、Task-252 `Parenthesized` row を捏造しない。

candidate input は application、ordinal、symbol、source contribution のみを
持つ。producer が origin、visibility、export status、optional signature
shell を clone する。同一 module candidate は normal / conflict-free /
source-preceding functor definition、imported candidate は public exported
または re-exported provenance を必要とする。missing / pending / opaque
signature は未解決 provenance として許可し、malformed は reject する。

## Derived dependency fingerprint

output の `primary_term_fingerprint` は build 時の Task-252 exact
`debug_text()` から導出する。`TypedAst::with_source_application` は
Task-252 handoff の先行 install を要求し、fingerprint と全 Primary target
を再検証する。replacement と non-equivalent な同一 source/module
substitution は atomic に失敗し、equivalent clone は許可する。
Task 254が既にinstall済みなら、同じtransactionでTask 253 publish前にその
structure handoffもrevalidateする。これによりTask-253 argumentによるTask-254
primary targetのownership、Task-254 termとのreverse containment/partial overlap、
closest Task-254 termがownしないcontained applicationをinstall順に依存せず
rejectする。
Task 255が既にinstall済みなら、Task 253 publish前にそのapplication fingerprint、
root-only target、nearest-family range partitionもrevalidateする。したがってlater
applicationはinstalled Task-255 occurrenceをcontain/overlap/retargetできない。

`ResolvedTypedAst` は同じ association を再検証して clone-preserve
するだけで、dense ID を再構築・retarget しない。

## Private source consumer

raw `SurfaceAst`、source node ID、syntax kind は
`mizar-test::runner::type_elaboration::source_application` だけに置く。
production selector は次の exactly 2 cases である。

1. `1 divides (1 ++ 2)` 内の imported `1 ++ 2`
2. frozen local two-functor definition block の second definiens 内
   `task253_local_source(x)`

aggregate application/wrapper/candidate/argument/request は 2/1/2/3/4、
co-installed Task-252 primary/reference/numeric-request は 3/1/2 である。
local actual は Task-248 definition parameter の `BindingId(1)` /
`BindingContextId(1)` / use ordinal 2 である。imported parentheses は
Task-253 wrapper 1 行で、Task-252 parenthesized row はない。

imported case の outcome/detail/public diagnostics は不変である。local
case は Task 253 を検証後、Task-260
definition-declaration payload gap
`type_elaboration.external_dependency.ast_payload_extraction` に public
diagnostic なしで留まる。

## Verification boundary

checker tests は dense table、全 form/cardinality、inline schema、degraded
recovery、wrapper、root-only primary、dependency fingerprint substitution、
nested application、candidate provenance/signature、request、corruption、
determinism、atomic failure を覆う。runner tests は exact 2 selectors、
aggregate oracle、local binding coordinates、wrapper ownership、corruption
isolation、deterministic replay、final clone preservation、他の全 active
type-elaboration case の exclusion に加え、private extractor と public
producer を通る ordinary/inline/nested/parenthesized/wrapped/degraded/
candidate-subset/template-and-mixed synthetic matrix 全体を覆う。

bounded trace row は
`spec.en.checker.type_elaboration.source_functor_application_payload` である。
MC-G017/MC-G020 は executable coverage が増えるが partial のままであり、
semantic term/formula/definition、overload selection、later cross-family
terms、accepted facts/proofs、Steps 6/7 は未実装である。

## Task 255C1 frozen private reuse seam

later implementationはprivate mizar-test consumerだけにbounded unwrapped
imported-`++` extractor/builderを追加する。exact mapper node/head/argumentsと
imported candidate provenanceをvalidateし、supplied complete Task-252 partsに
対してexisting public Task-253 `1/0/1/2/2` profileをbuildする。Task-255
codeはこのseamを呼び、application rowをduplicateしない。existing wrapped
theorem selectorとpublic checker APIは不変である。

## Task 255C1 reuse-seam result

private unwrapped imported-`++` extractor/builderを実装し、conditioned-
comprehension routeが使うsole Task-253 producerとした。supplied shared
Task-252 `4/0/4` handoffに対して`1/0/1/2/2`をbuildし、exact imported
symbol/contributionを保持する。既存wrapped selector、public API、debug byte、
全prior Task-253 testsは不変である。

## Task 257C2 frozen consumer boundary

complete Task-257C2 runnerは同じprivate unwrapped imported-`++` seamと
immutable `1/0/1/2/2` handoffをreuseする。new associationはTask-255
dependency chainをauthenticateするためにexact debug fingerprintだけを保持し、
application target/candidate/argument/request/resolver selection/public
Task-253 APIを追加しない。frozen pre-Task-256C1 baselineでは、routeは
separate lower taskがcondition-container graphをexecutableにした後だけ開始
できた。Task 256C1は両installation orderをpassし、Task 253自体は変更
しないまま、completed Task-257C2 routeはexact handoff/fingerprintをreuse
する。

## Task 258B3M2B2B1P proof-context reuse contract

Task 253は既に任意のauthenticated binding contextをacceptし、両Task-252
argumentsがapplication contextを使うことを要求する。private runner reuse
helperはより狭く、frozen Task-255/257 consumersへcontext 0を供給する。
B1Pはそのhelper/output bytesを維持し、存在する`BindingContextId`を受ける
private siblingを追加する。

future `take 1 ++ 2;` consumerでは、siblingは既存unwrapped-imported
extractor/builderをreuseし、proof context 1上でsymbolic infix application
1、wrapper 0、imported functor candidate 1、ordered numeral arguments 2、
unresolved type requests 2をexactにproduceする。missing context、
argument-context disagreement、wrapper、wrong application/head/argument
range/form、non-unique/substituted candidate provenance、stale replayは
publicationなしでrejectする。public Task-253 checker API、fingerprint
grammar、result semantics、既存context-0 behaviorは変更しない。

## Task 258B3M2B2B1P implementation result

runner-private siblingはexplicit existing contextを受け、その他を変更しない
exact extractionを既存public producerへ渡す。legacy entry pointはcontext
0でdelegateし、debug SHA-256
`9f1449159bf362bc90c4b41f3e4befb9a6d54f4152b836063f5cc07083d82a8d`
を維持。proof-context-1 constructionはordered `Primary(2/3)` argumentsと
imported `parser.type_fixtures::++` provenanceを保存する。独立したcontext、
range、form、target、candidate/contribution、stale-fingerprint、replay
corruptionはrejectする。checker source、public table、fingerprint grammar、
semantic result、active consumerは不変。

## Task 258B3M2B2B1A exact statement consumer

Task 253の最初のTask-258 consumerはB1P sourceのsingle unwrapped imported
infix applicationだけ。Task-253 handoffはapplication 0 `116..122`/
proof context 1、wrappers 0、exported imported
`parser.type_fixtures::++` candidate 1、ordered `Primary(2/3)` arguments、
unresolved requests 2のまま。public table/validation/debug bytesは不変。

new cross-family edgeはTask 253ではなくwitness handoffのoptional
application fingerprintと`Application(0)` targetがownする。transparent
node 47はwrapperではなく、node 46がtarget。Task 256 equalitiesは
application-independentでfingerprint `None`を維持する。missing/stale/
substituted/wrapped/unrelated Task-253はstatement-witness consumerで
rejectする。other Task-253 formsとTask-254/255 witness termsはdeferred。

## Task 258B3M2B2B1A consumer implementation result

exact imported-infix handoffはB1A statement witnessだけがconsumeする。
consumerはapplication 0、`parser.type_fixtures::++` local/FQN resolution、
complete candidate contribution/path/export provenance、argument targets/
numeric requests、source/module/ranges/form、Task-252 primary fingerprintを
authenticateする。matching application fingerprintはB1Aだけが保持し、
legacy statement witnessesは`None`、unchanged debug bytesの
application-independentなまま。missing、stale、substituted、wrapped、
orphan、partial installはpublishなしでfailする。other Task-253 shapesと
全Task-254/255 witness targetsはdeferred。

## Task 258B3M2B2B1B1P wrapped proof-context reuse contract

次のTask-253-owned prerequisiteは、158-byte/67-node B1B1 sourceにある
node 48のexact parenthesized `1 ++ 2` application、wrapper node 50、
proof context 1のprivate reuse seamだけである。public Task-253はexact
`1 application / 1 wrapper / 1 candidate / 2 arguments / 2 requests`
shapeを既にrepresentする。new private siblingはshared Task-252 `6/4/2`に
対してそのproducerをreuseしなければならず、wrapper 50をTask-252
primaryやfuture Task-258 witness targetに変えてはならない。

existing unwrapped context-0/context-1 helper outputsはbyte-identicalの
まま。`InfixExpression 130..136`をdirectに囲むnormal
`ParenthesizedTerm 129..137`ちょうど1件だけをadmitする。missing、
extra、nested、recovered、detached、reordered、wrong-range、stale、
substituted、context-mismatched wrapper/applicationはhandoffなしでfail
する。このseamはstatement、witness、semantic type、proof、
substitution、goal effectをownしない。

future runner tests 2件は全158 source-byte offsets、全67 arena-nodeの
kind/range/recovery/ordered-child identitiesとroot identityをexhaustし、
dormant public routeをunselectedに保つ。successは全application、
wrapper、candidate、argument、request、imported-origin fieldsに加え、
typed/final parityとempty upper tablesをfreezeする。failuresは
selector、Task-252、Task-253、typed-install fingerprint layersをこの
precedenceでdistinguishし、atomic rollback/replayをproveする。legacy
context-0/context-1 unwrapped hashesはそれぞれ
`9f1449159bf362bc90c4b41f3e4befb9a6d54f4152b836063f5cc07083d82a8d`
と
`0fd83f61a40d3fd43816a52b70fca4fa4cf7f1d6e9172d3c5fe558c5d4add80d`
のまま。future runner testsちょうど2件がこのlower prerequisiteをownし、
B1B1 statement consumerはlater logical taskのまま。

## Task 258B3M2B2B1B1P implementation result

wrapped proof-context reuse seamは、exact 67-node arena、wrapper/application
containment、Task-252 fingerprint、complete imported `++` identityを
authenticateした後、existing Task-253 producerへdelegateする。same-source
resolver substitutions 5件はselector admissionでfailする。frozen tests
2件は全byte/node mutations、exact diagnostic/node near-miss matrix、
producer/typed-install precedence、replay/clone、legacy hash gateをpassする。
public producer/table/APIもunwrapped outputも変更なし。

## Task 258B3M2B2B1B1 exact wrapped statement consumer

B1B1はB1B1P wrapped proof-context handoffの最初のstatement consumer。
Task 253は不変で、application 0 `130..136`、wrapper 0 `129..137`、
imported `parser.type_fixtures::++#12`、ordered `Primary(2/3)` arguments、
unresolved requests 2件をownする。statement witnessはapplication 0を
targetし、wrapper 0をtarget/copyしない。

new Task-253 API/row/fingerprint/validatorは禁止。runner-private B1B1
selectorはexisting wrapped seamをreuseし、checker-private B1B1 statement
profileはexisting application fingerprint/atomic installer経由でimmutable
handoffをconsumeする。B1A unwrapped bytesはcompatibleなまま、全other
wrapped/application formsをdeferする。
