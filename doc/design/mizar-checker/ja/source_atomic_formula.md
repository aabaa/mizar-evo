# ソース atomic-formula transport

> 正本は英語です。英語版:
> [../en/source_atomic_formula.md](../en/source_atomic_formula.md)。

## スコープ

Checker Task 256 は Task 257C1 により拡張され、ordinary predicate
application、exact two-segment predicate-chain transport 1件、equality、
inequality、membership、bare builtin type assertion、simple imported
attribute assertionという限定された source atomic formula の syntax-free
immutable 記述を所有する。source occurrence、transparent wrapper、predicate
segment と direct polarity token、predicate head と resolver candidate
provenance、formula-owned asserted-type/attribute site、nearest-family direct
term edge、unresolved expected-input request だけを運ぶ。

authority は Chapters 9/14 で、Chapters 3/6/13/19 が type、attribute、term、
resolver boundary を定める。Task 252/253/254/255 はそれぞれ primary、
application、structure、set/choice/`qua` term を所有し、本module は row を
複製せず dense root ID を参照する。predicate-chain applicability、implicit
conjunction、semantic segment negation、broader predicate chain、connective/
quantifier、condition formula、candidate selection、assertion truth、formula
result、theorem acceptance、fact、proof、downstream IR は deferred のままである。

## Public transaction

`SourceAtomicFormulaProducer::build` は
`SourceAtomicFormulaHandoffInput`、`BindingEnv`、`SymbolEnv`、required
`SourcePrimaryTermHandoff`、optional Task-253/254/255 handoff、shared
`TypedArena` を受ける。入力は9個の source-ordered vector を持つ。

- atomic formula
- transparent formula wrapper
- predicate-chain segment と source polarity
- ordinary predicate head
- individually resolver-authenticated predicate candidate
- formula-owned bare asserted-type site
- formula-owned simple assertion attribute
- formula-to-nearest-term-family edge
- unresolved operand/candidate-signature/type-reachability/attribute-
  admissibility request

transaction 全体の validation 後だけ9個の dense immutable table を publish
する。public ID は zero-based `new`/`index`、table は `get`/source-ordered
`iter`/`len`/`is_empty`、row は read-only accessor を持つ。handoff は常に
Task 252 を fingerprint し、edge が target にする場合だけ Task 253/254/255
を conditional fingerprint する。

## Public Enum Policy

| Public enum | compatibility policy |
|---|---|
| `SourceAtomicFormulaKind` | `#[non_exhaustive]`。caller は later frozen atomic source kind を許容する。 |
| `SourceAtomicFormulaRecovery` | `#[non_exhaustive]`。caller は later recovery class を許容する。 |
| `SourceAssertionTypeHead` | `#[non_exhaustive]`。caller は later bare builtin head を許容する。 |
| `SourceAssertionAttributePolarityInput` | `#[non_exhaustive]`。caller は later source polarity form を許容する。 |
| `SourcePredicateSegmentPolarityInput` | `#[non_exhaustive]`。caller は later predicate-segment polarity form を許容する。 |
| `SourceAtomicEdgeRole` | `#[non_exhaustive]`。caller は later direct-slot role を許容する。 |
| `SourceAtomicTermTarget` | `#[non_exhaustive]`。caller は later cross-family target を許容する。 |
| `SourceAtomicRequestKind` | `#[non_exhaustive]`。caller は later unresolved request kind を許容する。 |
| `SourceAtomicFormulaError` | `#[non_exhaustive]`。caller は validation failure を exhaustive match しない。 |

この module が所有する exhaustive public enum exception はない。

## 検証と ownership

source/module identity、dense source order、context、recovery、range、
typed-arena key、canonical token spelling、formula-local ordinal、table
association、resolver symbol/contribution provenance、single ownership を
認証する。formula key は predicate/equality/inequality/membership/type
assertion/attribute assertion を区別する。専用 key は predicate segment、
segment-level `does`/`do`/`not` token、predicate head、asserted type
expression/head、attribute occurrence/target、`non`、wrapper を所有する。

各 direct written term slot は Task 252/253/254/255 の maximal root
occurrence 1件へ対応する。descendant は nearest term family に残る。
duplicate/overlap/partial/non-root/reverse-contained/cross-context target は
atomic に fail する。absent fingerprint の unrelated optional handoff は、
その occurrence が全 formula/wrapper/direct-slot range と disjoint な場合だけ
共存できる。

segment row が空の legacy predicate formula は ordinary head 1件と1件以上の
authenticated candidate を要求し、candidate ごとに candidate-signature
request 1件を持つ。predicate-chain formula は dense segment 2件以上、
segment ごとに linked head 1件、exact polarity-token provenance、隣接 segment
間の shared-boundary edge 1件を要求し、frozen C1 profile は head ごとに
candidate/request 1件を要求する。
equality/inequality は operand request 2件、membership は right/container
request だけを持つ。bare type assertion は asserted type site と
reachability request を持つ。simple attribute assertion は1件以上の
authenticated attribute row を要求し、attribute ごとに admissibility
request 1件を持つ。request は intent だけで、answer、selected candidate、
type、fact、truth を含まない。

## AST installation

`TypedAst::with_source_atomic_formula` は one-shot で、targeted lower-family
dependency の先行 install を要求する。later Task-253/254/255 installer は
installed Task-256 handoff を再検証し、install order による fingerprint/
ownership bypass を許さない。replacement と non-equivalent dependency
substitution は AST を変更せず fail する。

`ResolvedTypedAst::assemble` は row を rebuild/renumber せず exact handoff を
再検証して clone-preserve する。handoff は semantic type、fact、coercion、
obligation、diagnostic、expression metadata、cluster fact を追加しない。

## Private source consumer

raw `SurfaceAst`、source node ID、syntax kind は
`mizar-test::runner::type_elaboration::source_atomic_formula`だけに置く。
production は unchanged Task-256 base fixture 8件、すなわち numeral
equality、inequality、membership、bare builtin type assertion、imported
predicate/functor、positive/negative imported attribute assertion、
set-enumeration equality と、exact Task-257C1 two-segment imported
predicate-chain fixture を select する。

8 transaction の Task-256 formula/wrapper/predicate-head/candidate/type-site/
attribute/edge/request aggregate は `8/0/1/1/1/2/13/11` である。shared
lower-family aggregate は Task 252 `16/0/16`、Task 253 `1/1/1/2/2`、
Task 255 `2/0/0/0/4/2` で、real Task-254 target はない。C1 route は別に
Task-252 `3/0/3` と Task-256
formula/wrapper/segment/head/candidate/type-site/attribute/edge/request
`1/0/2/2/2/0/0/3/2` を持つ。private composer は各 transaction を1 arena
で構築する。base 8 routes の既存 semantic outcome/detail key は
byte-identical のまま、C1 は transport だけを publish して frozen empty
external detail vector を返す。

## Verification boundary

checker test は dense table、formula kind、wrapper、canonical spelling、
provenance、request cardinality、arena/dependency identity、nearest-family
ownership、corruption、deterministic replay、installation、atomic failureを
coverする。runner test は base consumer 8件と exact C1 consumer、ordered
segment/edge/request、polarity token、shared-boundary reuse、lower-family
fingerprint、imported provenance/anchor、same-arena composition、selector
isolation、mutation failure、final `TypedAst`/`ResolvedTypedAst` ownership、
unchanged base-route external detail を coverする。

bounded trace row は base transaction の
`spec.en.checker.type_elaboration.source_atomic_formula_payload` と C1 の
`spec.en.checker.type_elaboration.source_predicate_chain_segment_payload`
である。executable source-transport coverage だけを追加し、semantic formula
work と Steps 6/7 は未実装のままである。

## Task 257B1 Consumer Addendum

Task 257B1は本moduleのexisting equality 1件とprimary-term operand edge 2件を、
universal body 1件のauthenticated dependencyとしてreuseする。atomic-formula row
ownership、validation、semantic deferralは変更せず、new formula-composition
handoffはcross-family parent associationだけを保持する。

Task 257B2はexact `8/0/0/0/0/0/16/16`の8 equality rowsをreuseする。16
operand edgesは引き続き本familyがownし、新composition tableはatomic rootを
repeated/fixed conjunction/disjunction parentへassociateするだけでatomic
semanticsを変更しない。

## Task 257B3 Frozen Consumer Addendum

Task 257B3はexact `3/0/0/0/0/0/6/6`、すなわちouter restriction
`x = x`、inner restriction `r = y`、innermost body `x = r`の3 equality
rowをreuseする。Task-252 operand edge 6件とunresolved operand-type request
6件は本family所有のまま。formula compositionはrestriction-parent association
2件とbody-parent association 1件だけを追加し、equality truth/operand typingを
変更しない。
atom 0/term 0・1はnested context 1、atom 1・2/term 2..5はcontext 3を使う。
3 atomすべて`Equality`/`Normal`、source ordinal `0..2`。source order、
spelling、range containment、request/edge ordinalはexact profile
discriminatorのまま。

Task 257B3はこのexact 3 atoms/6 operand rowsのexecutable reciprocal
consumerになった。atomic ownershipと全semantic deferralは不変。

## Task 257C1 frozen predicate-chain segment extension

Task 257C1は本producerをsyntax-free
`SourcePredicateSegment{Id,Table,Input}` row/immutable segment output、
non-exhaustive positive/negative polarity input、
`SourceAtomicEdgeRole::PredicateChainBoundary`で拡張する。exact 107-byte
consumerは`75..86` / `87..105`のdense segment 2件、`77..84` /
`96..103`のhead、`87..91` / `92..95`のnormal `does` / `not` tokenを持つ。
両headは同じimported `divides` candidate/provenanceを独立に保持する。

Task-256 formula/wrapper/segment/head/candidate/type/attribute/edge/request
profileはexact `1/0/2/2/2/0/0/3/2`。segment 0はedge 0/1、segment 1は
edge 1をimplicit left boundaryとしてreuseした後edge 2を使う。edge 1は`2`
の単一Task-252 primaryをtargetとしcopyしない。exact preceding-final-term
ruleの下でのみ、later segment外にtargetを持てる唯一のedgeである。legacy
empty-segment predicate applicationはone-head/byte-compatibleのまま。

nonempty debug segment lineはwrapper後/head前に置き、headerは
`source-atomic-formula-debug-v1`のまま。このsliceはsource partition、
token provenance、edge、candidate、request、final ownershipだけをtransport
する。predicate applicability/selection、implicit conjunction、semantic
negation、truth、fact、theorem acceptance、proof、IRはdeferred。

`predicate_segments`追加は4 production filesのexisting input literal 11件を
変更する。本moduleの`to_input` conversionはsegment rowをclone-preserveし、
legacy fixtureはempty rowsを使う。atomic runnerはexact C1 consumerだけに
nonempty rows、それ以前のatomic routeと全formula-composition literalには
empty vectorを供給する。これはcompile-time input compatibility editであり、
別family admissionではない。

### Task 257C1 implementation status

frozen contractどおり実装済み。handoffはvalidated segment row 2件、
source-token polarity、same-provenance head/candidate 2組、shared boundary
1件を含むprimary edge 3件、unresolved signature request 2件を公開する。
exact/corruption/legacy-byte/install/clone testはpassし、semantic predicate
resultは生成しない。

## Task 257C2 frozen consumer boundary

Task 257C2はTask-255C1 condition内のnormal equality 1件だけについてexisting
Task-256 producerのreuseをtargetとする。profileは
formula/wrapper/segment/head/candidate/type/attribute/edge/request
`1/0/0/0/0/0/0/2/2`。formulaは`177..182`のdirect
`BuiltinPredicateApplication` siteをownし、ordered operand edge 2件は
Task-252 primaries 2/3へ向き、unresolved `OperandExpectedType` request 2件を
持つ。enclosing Task-255 `FormulaExpression` wrapperやsemantic resultは
ownしない。

runnerはreusable built-in-equality builderでこのrowをextractし、same arenaの
Task-252/253/255 handoffに対してvalidateする。frozen pre-Task-256C1
baselineでは、`validate_cross_family_ranges`はset term側がformulaをcontain
するため、両install orderでlegitimate enclosing Task-255 set termをreject
した。separate Task-256C1 implementationは完了し、このTask-255 condition
relationだけをnarrowにauthenticateしつつ、arbitrary/copied/stale/wrong-range
overlap rejectionを保持する。Task 257C2のlower-stage blockは解消したが、
completed C2 routeはfresh preflight後にこのhandoffをreuseする。このpublic
lower-family transactionへfield/row/enum/request kind/debug byte/semantic
behaviorは追加しない。predicate-chain conjunction/negationはlater
Task-257C sliceに残る。

## Task 256C1 frozen condition-container compatibility

Task 256C1はexact Task-255C1 condition/equality consumerに対するprivate
cross-family range validationだけを変更する。existing set-term overlap ruleは
disjoint rowとatomic-formula-contains-set-term operandを従来通りacceptする。
加えてTask-255 `Comprehension` termがmatching condition rowをownし、
condition/`Equality` formulaがequal range/spellingとnormal recoveryを持ち、
formula contextがenclosing term contextと一致し、condition siteのarena
nodeがdistinct formula siteをdirect containする場合だけ、inverse
set-container relationをacceptする。

arbitrary/substituted/cross-source copied/stale/wrong-term/wrong-kind/
wrong-range/spelling/recovery/context/non-direct/partial/crossing/unrelated
overlapは
`SetTermDependencyMismatch`を保持する。他atomic kind、composite condition、
generator-dependent conditionはdeferred。

public/crate-visible producer/installation signature、table、ID、accessor、
enum、error variant、fingerprint、debug byteは変更しない。
`set_term_fingerprint()`は`None`のまま。optional matching set handoffは
validation contextだけなので、その有無でbuildしたatomic handoffはequalかつ
debugはbyte-identical。checker tests exact 3件がvalid relation、
`TypedAst`両install order/rollback、corruption/preservation matrixをfreezeする。
applicableなrelation near missはpair前に両lower familyで個別validateし、
pair時だけexact `SetTermDependencyMismatch`でfailする。optional-set
substitutionもno publication/valid replayをproveする。

## Task 256C1 implementation result

frozen private compatibility pathを実装した。各overlapping iterated Task-255
termをnormal raw equality/effective occurrenceと比較し、wrapper enlargementが
ないこと、owner-term context、matching normal condition row、condition site
からdistinct equality siteへのdirect edgeをauthenticateする。ID/ordinalは
hard-coded fixture constantではなくdataのままである。

optional set handoffはvalidation-onlyを保持する。exact atomic handoff/debug
byteは有無でidentical、`set_term_fingerprint() == None`である。既存
disjoint/formula-contains-set caseはpassを保持し、copied/substituted/stale/
wrong-owner/range/spelling/recovery/kind/context、wrapped、non-direct、
arbitrary、partial/crossing relationはfail closedする。exactly 3 testsが
これらと`TypedAst`両order/rollback/replayをcoverする。public schema、error、
fingerprint、debug、semantic result、runner、traceは変更していない。

## Task 257C3 frozen downstream consumer

Task 257C3はcommitted Task-257C1 `1/0/2/2/2/0/0/3/2` handoffを本module
変更なしで読む。separate composition handoffはcomplete atomic debugを
fingerprintし、conjunction 0をsegments 0/1/shared edge 1へ、negation 0を
already negative segment 1へassociateする。本moduleはpolarity token、
head/candidate、argument edge、imported provenanceのsole ownerのまま。
documentation prerequisiteはsource/public API/test/trace/hashを変更しない。

## Task 257C3 downstream consumption result

Task 257C3はproduction schema/rowを変更せずcommitted handoffをconsumeする。
composition validatorはexisting installation validatorを呼び、exact
`1/0/2/2/2/0/0/3/2` profile、shared boundary、negative tokenのrange/
spelling/recovery、common candidate symbol、imported contributionを
authenticateする。same source/module/arena上のtest-only coherent
single-predicate profileにより、earlier identity mismatchではなくC3 profile
guardがrejectをownすることを確認する。
