# Source primary-term handoff

`build_from_occurrences` はresolverの検査済みscopeとmodule contextへの束縛射影で全variable leaf参照を認証する。
NormalなValue括弧は実際のTermExpression・括弧の連鎖を保持し、参照はsealed leafだけに属し、Resolver clockは展開されたPrimary IDと独立である。
既存node validatorは構築時と[native C2再生](source_formula_composition.md#変数参照の条件式関連付け)で省略可能な認証対象childも検査する。
Generic producerと単独のPrimary設置は従来のnode単位metadata保証を保ち、宣言位置検査を緩めたりsource receiptを作ったりしない。

> canonical languageはEnglishである。
> [English source_term.md](../en/source_term.md)

## Original-source nested reserved occurrences
crate-private `SourcePrimaryTermProducer::build_from_original_reserved_statement_occurrences`は元source/symbols/全occurrences/typed arenaから実binding projectionと既存environment/primary pairを構築し、caller製positive rowを受け取らない。全termの実site/range/winnerと実scope map上のproof contextを照合し、module contextの`None`を実root scopeへ対応付ける。全referenceの実lexical scope/occurrence ordinalを保持する。
唯一の`ReservedVariable` identityはmodule所有で全proof contextへvisibleとなり、proof自身のbinding listは空とする。B1のchild proofは一つ、B5Aは兄弟となる二つ。parentは実structural parent/scopeから導き、挿入順やlabel scopeへのcastを使わない。native ordinal 0をlegacy declaration-event lookupへ渡さない。既存public module/scoped producerのtransaction規則とReserve境界を維持し、新public type/field/binderを追加しない。
native B2：全6個の元variable occurrenceを認めた三つのequalityの実operandへjoinし、native ordinal `0..5`と各実use scopeを保持する。全occurrenceの網羅性はcontext数と独立に検証し、moduleと単一proofのcontext数からreference数を導かない。
native B3は同じ全occurrence認証により全5個の実reference use ordinal `0..4`と実lexical scopeを保持する。equality/witness operand censusは[intake](type_checker.md#original-source-reserved-statement-inputs)が所有し、このproducerはsyntax-freeを保つ。witness targetは実siteから導き、全atomic edge/requestから除外し、固定reference IDやformula index算術を使わない。
native B3Nは同じreserved occurrence五つとnative clockを保持する。name tokenはprimary term/referenceでもBindingIdでもない。定義roleとname use拒否はintakeが所有し、このproducerへnamed-take構文やabbreviation意味処理を加えない。
native mixed B3M1は実reserved occurrence全6件と実Resolver clock/lexical scopeを保持する。二つのwitness operandは実primary/reference rowであり、nameはこのtransportに含めない。二witness censusとname use拒否はintakeが所有するため、このproducerとpublic挙動は変更しない。
native B3M2B1はsource順primary ID/source ordinal `0..5`を保持し、wrapper 2の後にchild 3 (`parent=2`)を置く。実referenceは`[0,1,3,4,5]`へjoinし、Resolver use ordinal `0..4`と実scopeを独立して保持する。wrapper 2はbinding/reference/numeric requestを持たない。intakeが認証したwrapper/child site relationをこのsyntax-free producerが一度だけ消費し、childを実occurrenceへjoinし、不一致childまたは未使用relationを拒否する。reference clockをprimary ID/ordinalから導かない。
original-sourceの単一unnamed witnessでtransparent groupの有限深さを`d`とし、preorder primaryを`d+5`件、wrapper IDを`2..d+1`、実reference leafを`d+2`、conclusion leafを`d+3,d+4`とする。実referenceは`[0,1,d+2,d+3,d+4]`へjoinし、Resolver clock `0..4`と実scopeを保つ。深さ2のchainは`2 -> 3 -> 4`である。TypeCheckerが全source chainを認証し、syntax-free producerは既存private site/range/child tupleを深さの固定上限や新abstractionなしで一度消費する。先行parent・厳密包含range・単一child検証により不一致・重複・循環・非連結・未使用relationを拒否する。wrapperはreference/binding/numeric requestを増やさず、clockをprimary IDから導かない。
original empty-enumeration lower profileのset witnessはprimary/referenceを増やさない。theorem/conclusionの実variable occurrence四つ、native clock `0..3`、全source/scope/binding joinと空numeric requestを保つ。[intake](type_checker.md#original-source-empty-enumeration-lower-acquisition)がnonreference witnessと別set handoff/typeを認証・所有し、このproducerはsyntax-freeのまま変更しない。

original singleton-enumeration lower profileは要素を含む実reserved reference五つをこの未変更producerで保持する。enumeration自身のprimary/referenceは作らず、[intake](type_checker.md#original-source-singleton-enumeration-lower-acquisition)が実element edgeとintrinsic typeを所有する。

original bare-set choice lower profileは実theorem/conclusion reference四つとnative clockだけを既存producerで保持し、choice/written targetからprimary・reference・bindingを作らない。[intake](type_checker.md#original-source-builtin-set-choice-lower-acquisition)がsource認証・target typing・inhabitationを所有する。

## 目的とauthority

public `source_term` moduleはChecker Task 252を実装する。variable/local-constant
reference、`it`、numeral、transparent parenthesisのsource occurrenceをraw
syntax importなしでcheckerへtransportする。canonical authorityはChapter 04
§§4.1-4.3、4.4.1、4.6とChapter 13 §§13.1、13.8.1-13.8.2、13.8.8であり、
broader term/source-to-checker gapはMC-G017/MC-G020が追跡する。

moduleはtransport-onlyである。source shape、binding lookup、missing numeric-type
requestをauthenticateするが、numeric typeを選択せず、semantic term/formula、
current definition result type、fact/axiom、FOL/downstream IRを作らない。

## Task 269SDU Descendant Given Occurrence/Reference Contract

[central Task-269SDU contract](../../task_contracts/ja/269SDU.md) の下で、
`source_term.rs` はpublic `SourceProofLocalGivenDescendantUseTermHandoff`、producer、
non-exhaustive errorを実装する。handoffはcomplete SDT dependencyをby-value consumeし、
exact dependency/debug fingerprintをpreserveする。source-term rowは`y@118..119` 1件だけで、
typed node `3`、ordinal `0`、descendant context `2` (`[0,0]`)、
`VariableReference`/`Value`、`term 0 -> binding 1` の`Variable` reference 1件であり、
numeric requestは0。

validationはexact SDT dependency/prefix、exact term input/source-term fingerprint、
complete five-node arena、one-shot installationの順でatomicに行う。arenaは3-node SDT
prefix、node `3` `source.term.variable-reference@118..119`、children `[2,3]`のroot
node `4` `source.proof-local.given-descendant-use.term-root@0..179`を持ち、全nodeは
Unknown/Normal、unresolved、unlinkedのまま。profile限定`GivenWitness -> Variable`
admissionだけを許可しgeneric admissionは不変。error順は`InvalidDependency`、
`InvalidSourceTerm`、`InvalidInstallation`。

この実装済みownerはSet binding/abbreviation/capture、equality/formula/fact/guard、
proof completion、generic term publication、diagnostic、downstream IR、dispatch、
coverage creditをexcludeする。特に`z@114..115`、`q@129..130`、`z@133..134`はabsentのまま。

## Public model

`SourcePrimaryTermHandoffInput`はsource/module transaction 1件と、次のordered
input table 3件を持つ。

- `SourcePrimaryTermInput`
- `SourcePrimaryTermReferenceInput`
- `SourceNumericTypeRequestInput`

`SourcePrimaryTermProducer::build`はsyntax-free `BindingEnv`/`TypedArena`に
対してrowをauthenticateし、`SourcePrimaryTermHandoff`をatomicにpublishする。
immutable `SourcePrimaryTermTable`、`SourcePrimaryTermReferenceTable`、
`SourceNumericTypeRequestTable`がexposeするのはborrowed lookup、source-order
iteration、length、emptinessだけである。dense identityは
`SourcePrimaryTermId`、`SourcePrimaryTermReferenceId`、
`SourceNumericTypeRequestId`である。

term rowはnode site、exact source range、dense pre-order source ordinal、
binding context、recovery、token-normalized spelling、kind、role、optional
parentを保持する。reference rowはterm/binding identityとroleを保持し、lexical
scope/use ordinalはproducer-derived outputである。numeric requestはexact numeral
term/site/range/spellingとdense request ordinalを保持する。`debug_text()`は全tableを
deterministically renderする。

## Public Enum Policy

| Public enum | compatibility policy |
|---|---|
| `SourcePrimaryTermKind` | `#[non_exhaustive]`。callerはlater primary-term familyを許容する。 |
| `SourcePrimaryTermRole` | `#[non_exhaustive]`。callerはlater source roleを許容する。 |
| `SourcePrimaryTermReferenceRole` | `#[non_exhaustive]`。callerはlater authenticated binding roleを許容する。 |
| `SourcePrimaryTermRecovery` | `#[non_exhaustive]`。callerはlater recovery classを許容する。 |
| `SourcePrimaryTermError` | `#[non_exhaustive]`。callerはvalidation failureをexhaustive matchしない。 |
| `SourceProofLocalGivenUseTermError` | `#[non_exhaustive]`。callerはTask 269GU dependency/input/installation failureをexhaustive matchしない。 |
| `SourceProofLocalGivenConditionUseTermError` | `#[non_exhaustive]`。callerはTask 269GCU dependency/input/installation failureをexhaustive matchしない。 |
| `SourceProofLocalGivenDescendantUseTermError` | `#[non_exhaustive]`。callerはTask 269SDU dependency/input/installation failureをexhaustive matchしない。 |
| `SourceNestedFraenkelMapperPrimaryError` | `#[non_exhaustive]`。callerはC4C4 dependency/binding-environment/source-term projection failureをexhaustive matchしない。 |

この module が所有する exhaustive public enum exception はない。

## Validationとatomicity

term idと`source_ordinal`はequal dense pre-order indexである。全siteはunique
`TypedSiteRef::Node`で、arena kind/range/recoveryがrowとexact matchする。
identifier referenceはcanonical `mizar_lexer::is_identifier` predicateがaccept
するnonempty binding-authenticated spellingを要求する。first characterはASCII
alphabeticまたは`_`、remaining characterはASCII alphanumeric、`_`、
apostropheだけで、reserved wordをrejectする。このcheckはlexical vocabularyを
reuseするがraw syntaxをimportしない。`it`はexact `it`、numeralはASCII
digitだけ、parenthesis spellingはtoken間ASCII space 1件のexact
`( <child spelling> )`である。

各parentはsame contextのearlier parenthesisで、rangeがimmediate child 1件を
strict containする。parentだけがchildをownする。root/siblingはsource orderを
保ち、nested parentはTask-252 kind 5件だけのclosed acyclic pre-order treeを作る。
private runner extractorはlater term family descendantを含むparenthesized subtree
全体をexcludeする。

variable/constant rowはreference exactly 1件を持つ。generic profileのvariableは
`ReservedVariable`/`LetBinding`/`QuantifierBinder`/`DefinitionParameter`だけ、
constantは`LocalAbbreviation`だけを受ける。`it`/numeral/parenthesisはbinding
referenceを持たない。全numeralはnumeric request exactly 1件、他kindは0件である。

各referenceについてproducerはterm contextからlexical scopeをcloneし、
declaration range endがterm start以前であるbinding row数を`use_ordinal`とする。
previous referenceはordinalを進めない。normal binding groupはsource-order
singletonでvisibilityがdense indexと等しい。exact consecutive duplicate groupは
spelling/kind/owner context/`BinderIdentity`/rangeとgroup final row dense indexの
visibilityを共有する。このgroupを`BindingEnv::lookup`が`Ambiguous`としてreject
できるまで保持する。

producerはresolver payloadなしで`BindingLookupSite::new`を構築し、supplied
local binding exact winnerだけを要求する。forward/ambiguous/missing scope
payload/unresolved/different winner/lookup errorはfail closedである。このpathで
`Resolver`はstructurally unreachableである。inputをsort/repair/partial publish
しない。

## Ownershipとconsumer

`TypedAst::with_source_term`はsource/moduleと全arena nodeをrevalidateしてoptional
immutable handoff 1件をinstallし、replacementをrejectする。`ResolvedTypedAst`は
handoffをclone-preserveするだけで、`source_term()`をexposeする。

private `mizar-test::runner::type_elaboration::source_term` leafがraw
`SurfaceAst` extractionをownする。exact real selectorは次の3件である。

1. `fail_type_elaboration_term_formula_gap_001`
2. `pass_type_elaboration_reserved_variable_equality_001`
3. `pass_type_elaboration_parenthesized_reserved_variable_equality_001`

aggregate handoffはterm 7/reference 4/numeric request 2で、existing semantic
outcome/detail keyは不変である。synthetic testはsemantic acceptanceを追加せず
local constant、`it`、nested parenthesis、mixed-family exclusionをexerciseする。

## Verificationとdefer

checker testは全kind/role、dense order、binding-event order、shadow/forward/
ambiguous/missing/unresolved lookup、reference/numeric-request cardinality、
parent graph、source/module/site/range/kind/spelling/recovery/context corruption、
deterministic rendering、typed-AST installationをcoverする。runner testはexact
real selector、7/4/2 oracle、synthetic dependency boundary、isolation、
corruption、deterministic replay、final resolved preservationをcoverする。

covered trace requirementは
`spec.en.checker.type_elaboration.source_primary_term_payload`である。
application、structure/set/choice/comprehension/`qua` term、formula graph、
definition result semantics、real proof-local constant production、numeric
response、accepted fact/declaration/proof、downstream IR、Tasks 253+、Steps 6/7は
explicit ownerに残る。

## Original-source theorem scoped primary references
同じ入口は、元の封印済みResolverが選んだ宣言前のgenerator mapperを含む完全なnormal変数leafについて実Comprehension字句環境を受け取る。参照metadataを与え、mapper形成や型付けは与えない。許容される先行simple predicate-definition prefixは実際の空の字句境界を保持する。定義仮引数/bodyはResolver receiptとPrimary censusから意図的に除外される。兄弟contextとprefix外の使用を含め、全theorem/proof occurrenceは必須のままである。構築時に元のsource環境と完全なoccurrence censusを再生し、通常のTyped/final installationは既存のnode-level metadata保証を維持する。
`SourcePrimaryTermProducer::build_from_scoped_theorem_occurrences(input, binding_env, arena, source, symbols, scope)` は既存の所有 SourcePrimaryTermHandoffInput と借用 BindingEnv、TypedArena、SurfaceResolvedArena、SymbolEnv、ResolvedVariableScope を受け、既存 SourcePrimaryTermHandoff または SourcePrimaryTermError を返す。実在する定理参照と字句 context を認証する別入口であり、既存 builder と検証は変更しない。
最初に source/symbols/scope から[実在する定理 BindingEnv](type_checker.md#original-source-theorem-binding-contexts)を取得し、未使用の宣言・context を含めて渡された環境全体との一致を要求する。上流拒否と環境不一致は InvalidTransaction。非公開 scoped profile で既存の主項検証を再利用し、この入口だけ FunctionBinder の変数参照を許可する。Generic と flat 入口は変更しない。宣言イベント順序、実際の BindingEnv.lookup の勝者、node/range/cardinality と既存エラーを保持し、全 term を Normal VariableReference/Value に制限する。ResolverLocal identity の検証は使用される Reserve 参照を引き続き拒否する。Binder root、型、formation evidence は生成しない。
各生成参照の実在 typed-site node の resolved link を source.resolved_node_for(reference.node()) を介して実際の元 Resolver 参照に対応付ける。surface と typed の数値 node ID をキャスト・同一視しない。Resolver 参照の一意な全数対応、Term/Variable role、正確な range/spelling/binding と、term context の lexical scope が実際の使用 scope に一致することを要求する。不一致は既存 InvalidReference とし、部分 handoff を返さない。
全occurrenceと完全なcensusの認証後に限り、前方のComprehensionGenerator勝者に元の宣言ordinal + 1による非公開の論理lookup可視性を与える。通常の前方参照は拒否を保つ。封印済み Resolver 参照の scope と参照列挙 ordinal を既存の非公開出力参照 field に保持する。この ordinal は Generic build の宣言イベント lookup clock と異なり、相互に代用しない。他の生成 field、dense ID/順序と source identity は保持する。caller が scope/use ordinal を注入する field や新表現は追加しない。
既存 install は source/module と arena 形状を検証し、BindingEnv lookup は再実行しない。本入口は primary 参照だけを認証し、[scoped atomic/inference の構成](type_checker.md#original-source-theorem-scoped-inference) は TypeChecker が所有する。theorem の採用、汎用 source-context handoff、完全な context/type phase、composite/composition、Artifact/publication/currentness/reuse/acceptance は延期し、既存の定理・証明方針は変更しない。
### Scoped primary reference tests
実在 predicate-prefix ケースは既存の [Atomic producer](source_atomic_formula.md) で実際の singleton condition atom も導き、元 collector の provenance、scoped Primary operand と Unknown metadata を保持する。独立 Comprehension は元の内側 context で変更のない [Set producer](source_set_term.md#scoped-comprehension-structural-tests) に渡し、同一 arena 上で Set dependency なしに同一の独立 Atomic product を再生する。外側 Membership は実際の bare positive or negative singleton predicate 条件とも [Atomic ownership validation](source_atomic_formula.md) で対を構成する。Nested Comprehension、runner integration、推論、受理は保留する。
既存の非 Binder テストは実在 Disk/OpenBuffer の equality/primary 取得と scoped 環境を保持する。Binder 参照テストでは実際の元ソースの range/resolved link と独立した fixture の宣言・参照・context 一覧から、caller 所有の Unknown/Normal TypedArena leaf と primary 入力を導く。推論済み・checked semantic phaseは与えない。全出力行、字句 scope、異なる宣言・参照 clock、source identity、入力不変性を検証する。既存 Generic/flat 入口の検証を維持し、未使用の fresh head と自由な外側参照の実際の勝者も確認する。
負例は foreign source/environment/scope、未使用宣言/context の変更、term/reference/site の不完全・不一致と複合 fault の権威優先を扱う。実在 nested use を Generic lookup が成功する祖先 context へ移す例、および通常の形状検証は成功する誤 resolved-node 対応を含め、新入口が両者を拒否する。下位 guard の全行列は複製しない。合成変異は拒否専用とする。空のTyped/final containerは認証済みmetadataを再生し、推論済み型・fact・VC/token evidenceやscoped inference/Core/証明受理は与えない。

## Task 257C4C4 nested Fraenkel mapper primary

Implemented [C4C4 contract](../../task_contracts/ja/CHECKER-FRAENKEL-NESTED-MAPPER-PRIMARY-257C4C4.md)は
C4C3-dependent/zero-semanticなspecialized Task252 transactionを1件追加した。
Exact public familyは`SourceNestedFraenkelMapperPrimaryHandoff`、non-exhaustive
`SourceNestedFraenkelMapperPrimaryError`、
`SourceNestedFraenkelMapperPrimaryProducer`。Handoffはcomplete C4C3 dependency、
exact 3-context/outer-x 1-binding/0-diagnostic `BindingEnv`、one-node immutable
projection arena、existing `SourcePrimaryTermHandoff` `1/1/0`をretainする。

Sole primaryはinner mapper `x@94..95`の`VariableReference`/`Value`。Sole referenceは
C4C3 resolver binding `1` / outer binder `x@136..137`を表すprojection-local checker
binding `0`をselectする。Binding ownerはouter-comprehension context、useはinner child
context。Profile-scoped logical ordinal `1`だけがsource上後置されたgeneratorをvisibleに
する。Inner `y`はrevalidated C4C3 dependencyだけに残りTask252へduplicateしない。

Private specialized profileはpublic generic Task252 source-order derivation/forward rejectionを
弱めない。Exact getter/debug/error precedence/context/binding/arena/row/default-deny matrixは
task contractだけがownする。Sole current consumerはexisting private nested-capture library
test leafである。Typed/Resolved installation、active route、capture identity、generated-core
parameter、type/sethood answer、semantic result、diagnostic、coverage creditは追加しない。

C4C5はこのC4C4 handoffへcrate-private `validate_complete()` reauthentication seamだけを
追加する。Separate Task257C formula-composition ownerがcomplete C4C4 handoffをby-value
consume/retainし、`source_term`はcapture-identity tableをpublishせず、C4C4 projected
bindingのcaptured stateをemptyに保つ。

## Task 257B1 Consumer Addendum

Task 257B1はexact pass consumerへadditional `VariableReference`/`Value` row
2件とbinding reference 2件を追加する。両referenceはbody context 1でexplicit
quantifierのbinding 0をselectする。occurrence/lookup-winner ownershipはTask 252
だけに残り、formula-composition handoffはbinder-to-reference associationだけを
記録し、captured-free-variable metadataをrepurposeしない。

Task 257B2はbody context 1の16 numeral rows/16 numeric-type requestsをreuse
する。explicit `x` binderはunusedなのでreferenceは0、captured identitiesは
emptyであり、composition layerはbound-use rowをfabricateしない。

## Task 257B3 Frozen Consumer Addendum

Task 257B3はexact `VariableReference`/`Value` term 6件とTask-252
lookup-selected reference 6件をreuseする。source orderで`x` 3件はouter
quantifier binding 1、`y` 1件はbinding 2、`r` 2件はreserved binding 0ではなく
inner quantifier binding 3をselectする。term/reference 0・1はcontext 1、
2..5はcontext 3。termは`VariableReference`/`Value`/`Normal`、source
ordinal `0..5`、referenceはvariable roleを保持する。scope path/local identityは
source-derived resolver-shaped preflight fact、use ordinalはauthenticated
Task-252 producer outputである。formula compositionはowning-edge association
だけを記録し、occurrence/reference/spelling/lexical-scope/lookup-winner
ownershipはTask 252に残す。

Task 257B3はこのsix-row reciprocal consumerを実行し、binding ids
`1,1,3,2,1,3`とuse ordinals `2,2,4,4,4,4`を検証する。全occurrence/
referenceのownershipはTask 252に残る。

## Task 257C1 frozen consumer addendum

Task 257C1はsource term `1`、`2`、`3`のTask-252 `Numeral`/`Value`
primary/numeric requestをexact `3/0/3`でreuseする。primary 1
（`2`、`85..86`）は単一occurrenceで、新Task-256 shared-boundary edgeはterm/
requestをduplicateせず、隣接する両segment descriptionから同じrowを参照する。
occurrence、spelling、range、arena、numeric-request ownershipはTask 252、
predicate grouping/polarityはTask 256に残る。

Task 257C1 active pass consumerはこのfrozen backlinkを実行する。実測
`3/0/3` profileとsingle middle-primary identityを保持し、Task-252 API/
semantic numeric resultは変更しない。

## Task 255C1 frozen backlink

exact conditioned-comprehension prerequisiteはone immutable Task-252
`4/0/4` handoffをbuildする。primaries 0/1はTask-253 mapper argument、
primaries 2/3はauthenticated condition range内のequality operandである。後者は
ordinary Task-252 occurrence/numeric-request rowのままだがTask-255 edgeを
持たない。formula/numeric semanticsをここへ与えず、later Task 256がtargetする
exact objectを保持する。

## Task 255C1 transport result

exact routeはこのsingle `4/0/4` handoffをpublishする。copied、omitted、
range-substituted condition primaryはcomplete Task-255 transactionをfailさせる。
authentic condition operands 2件はTask-255 edgeを持たず、later Task-256
installation向けに保持される。

## Task 257C2 frozen consumer boundary

Task 257C2は同じimmutable `4/0/4` handoffをreuseする。Task-256 equality
edgesはprimaries 2/3をdirectにtargetし、Task-257C2 associationがtargetするのは
condition/formula IDだけ。Task-252 row/request/parent/context/fingerprint/
debug byte/numeric semantic meaningは変更しない。frozen pre-Task-256C1
baselineではrouteはseparate lower taskをgateとしていた。Task 256C1は両
installation orderをpassし、Task 252自体にcompatibility editは不要なまま、
completed Task-257C2 routeはfingerprint/debug byteを変更せずexact rowを
reuseする。

## Task 257C3 frozen downstream consumer

Task 257C3はexisting Task-252 `3/0/3` numeral handoffをexact fingerprintと、
primary 1をtargetするTask-256 shared boundary edge経由だけで
reauthenticateする。term/reference/request row、parent edge、ownership、
Task-252 APIを追加しない。本documentation prerequisiteでは全Task-252
byte/testが不変。

## Task 257C3 downstream consumption result

implementationはexact immutable `3/0/3` handoff/debug fingerprintをreuseし、
Task-252 production API/rowを追加しない。same source/module/arena上のcoherent
two-term test-only handoffは個別validate後、C3 exact-profile boundaryだけで
failする。

## Task 258A frozen downstream consumer

Task 258Aは`74..75`/`78..79`のnormal `VariableReference`/`Value` primary
2件とreserved binding 0へlookup-authenticatedされたreference 2件だけを
reuseする。profileは`2/2/0`、1 binding rowが両useより前にcompleteするため
Task-252 `SourcePrimaryTermReference::use_ordinal()`は両方1。これはrunner
upstream binding/use source-event lookup ordinal 1/2とはdistinct。
statement input factはreference IDs `[0, 1]`を指すだけでbinding/spelling/
range/lookup winner/source ordinalをcopyしない。Task 252が全occurrence/
reference ownership/semanticsを保持する。本prerequisiteはTask-252 API/
source/test/debug byteを変更しない。

## Task 269GUP source-term exclusion

GUPはexact sibling binding envだけを作る。checker/runner source-term、Task252 allowlist、
term/reference/request table/testはbyte-identical。`116..117`/`120..121`はselector-only。
GUPT後のTask269GUだけがfuture GivenWitness Variable admission/occurrenceをownする。
Completion evidence: [central Task-269GUP historical contract](../../task_contracts/ja/269GUP.md#completion-evidence)。

## Task 269GU proof-`given` later-use term/reference 凍結契約

### 選択、authority、classification

fresh clean HEAD `c529245138b6d40be65c590ba701fef4f4ea0881`はcommitted
GUPT source-type prerequisiteを含み、Task 269GUだけを選択する。canonical
Chapter 4 §4.6.1(5)、Chapter 15 §§15.3.3/15.10、Chapter 16 §16.3.3
item 5a/§16.4.2とuser確認は、`given` witnessがdeclaration condition内をbindし、
その後はinnermost enclosing proof/reasoning blockの残りと、shadowされないchild
blockで有効、parent/siblingでは無効であることを定める。Chapter 8 §8.1は既に
実装済みのdeclared typeだけ、Chapter 3は通常のin-scope variable解釈だけを与え、
Chapter 13 §§13.1.1/13.8.1がin-scope identifier occurrenceをvariable referenceとする。

exact sourceは128 byte・final LF 1件、source SHA
`ec15ded78ae96022840a8419a85d74643de3b37337e9a202cbda77ee97aa7c01`、
54 Surface node/root 53、snapshot SHA
`c64297ce72e380a2e4146276966e085d780f8b38f2528d5abaa440a50c67db6d`である。
later leafはunrecovered `TermReference` node 41/43、`y@116..117`/
`y@120..121`だけである。GUPはshell/theorem/resolver provenance、proof
`62..126`、given `70..108`、name `76..77`、scope `[0]`/ordinal 1を認証済み、
GUPTはexact `2/2/0` typed BindingEnvと`2/2/0/0/0/0` source typeをownする。

blocking `spec_gap`はない。未実装composition/testはbounded `source_drift`/
`test_gap`、stale ledgerは`design_drift`。generic Task-252 pathへの無制限
`GivenWitness` admission、old GUP/GUPT変更、formula/proof semantics出版は
`boundary_violation`。origin `0/9`はreport-only `repo_metadata_conflict`である。

### Public compositeとexact payload

`source_term.rs`へpublic `SourceProofLocalGivenUseTermHandoff`、
`SourceProofLocalGivenUseTermProducer`、non-exhaustive
`SourceProofLocalGivenUseTermError { InvalidDependency, InvalidSourceTerm,
InvalidInstallation }`を追加する。handoff getterはsource/module、owned GUPT
dependency、そのcomplete debug fingerprint、`SourcePrimaryTermHandoff`とその
complete fingerprintをborrowする。producerはGUPTをby valueでconsumeし、
`SourcePrimaryTermHandoffInput`と`TypedArena`からatomicに構築する。
handoffとerror enumはいずれも`Debug + Clone + PartialEq + Eq`を実装し、errorは
さらに`std::error::Error`を実装する。

exact profileは`2/2/0`である。term 0/1はsite node 3/4、range
`116..117`/`120..121`、source ordinal 0/1、context 1、Normal、spelling `y`、
`VariableReference`/`Value`、parentなし。reference 0/1は各term、binding 1、
`Variable`、producer-derived scope `[0]`、use ordinalは双方2。reserve binding 0と
witness binding 1が両use以前にcompleteし、first occurrenceはbinding eventを
増やさないためである。numeric requestは0。context 1/scope `[0]` lookupは両方
local binding 1を返す。

generic `SourcePrimaryTermProducer::build`の既存allowlist behaviorは不変。
private GU profileだけがexact compositeで`GivenWitness -> Variable`を許可する。
これによりcanonical決定をunauthenticated global admissionへ拡張しない。

arenaはGUPT prefixを保つdistinct 6 nodeである。node 0/1/2は
`reserve-type@14..17`、`type@84..87`、`type-root@0..127 children [0,1]`。
node 3/4は`source.term.variable-reference` at `116..117`/`120..121`。
node 5は`source.proof-local.given-use.term-root@0..127 children [2,3,4]`でroot。
全nodeはresolved nodeなし、Unknown、Normal、empty links。dependencyはnode
0--2を含む全public component/fingerprintで再認証し、standalone GUPTの3-node
contractを弱めない。source/module/payload/fingerprint/lookup/arena mismatchは
atomic failureである。

debug headerは`source-proof-local-given-use-term-debug-v1`、module、quoted
complete dependency fingerprint、quoted complete source-term fingerprint。
error stringは`source proof-local given-use term dependency is invalid`、
`source proof-local given-use source term is invalid`、`source proof-local
given-use term installation is invalid`。
validation precedenceはdependency/GUPT prefix再認証、exact source-term input/
profile検証、exact full-arena/one-shot installation検証の順である。どのfailureも
partial handoffまたはAST ownerをpublishしない。

### Typed/final ownership、runner、scope

`TypedAst`/`ResolvedTypedAst`はboxed optional
`source_proof_local_given_use_term`だけをownし、getter/one-shot installerと
exact Invalid errorを追加する。direct GUPT/binding/type/term fieldはinstallしない。
six nodesはfinalで`source.proof-local.given-use.term`へone-for-one projectionする。
全semantic table/node-hint inputはempty、全old ownerとboth-order mutually exclusive。

dormant runner outputは`{ typed_ast, resolved }`、mutationは`None`、
`WrongDependencyModule`、`WrongTermRange`、`WrongReferenceBinding`、
`WrongArenaRoot`、`WrongArenaKind`。selector argsはGUPTと同じでcfg-test版だけ
mutationを追加する。mismatchは`None`、selected failureは`Some(Err(_))`。
route-local stringは`Task269GU GUPT dependency is missing`だけ。exact GUPT
private outputからauthenticated owned dependencyをcloneし、2 term/referenceと
6-node arenaだけを構築する。public dispatchから到達不能。

implementationはexact 7 Rust files、checker `source_term.rs`/`typed_ast.rs`/
`resolved_typed_ast.rs`とrunner proof-local leaf/facade 2件/test leafだけ。
`source_type.rs`、`source_proof_local_declaration.rs`、`binding_env.rs`、runner
`source_statement.rs`、parser/resolver、canonical spec、fixture/sidecar/
expectation/trace/metadata/Cargo/diagnostic/dispatch/CLI/active resultは禁止。

checker/runner各4 testで、2 occurrence/lookup、全fingerprint、全dependency/input/
arena corruptionとprecedence、clone replay、one-shot/both-order ownership、generic/
Let/old-Given/GUP/GUPT/near-miss isolation、zero semantic publicationを網羅する。
docsはchecker 14 pairs、runner 6 pairs、global 2の42 Markdown。baselineは
libraries `506/568`、production `30/174332`/`37/75074`、implementation projection
`510/572`。path/content/test-list hash、corpus `428/395`、pass/fail `235/193`、
warnings/errors `23/0`、stage `101/7/205/1`、type coverage `259=247+12`、trace
SHA `55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`と
全CLI/fixture hashはEN canonical記載値を共有し、artifact/countは変更しない。

GUのownershipはlater primary-term occurrence/referenceだけ。equality/
BuiltinPredicateApplication/proposition/`thus`、condition/label/fact、existential/
Skolem、guard/assume、capture/export、goal/thesis、initial obligation、proof/
discharge/acceptance、theorem acceptance、Core/CFG/VCは明示deferする。

exitはEN/JA spec review NO FINDINGS、docs-only 9 hard gates uncapped `>=90/100`、
docs prerequisite commit、fresh preflight、exact seven-file/eight-test implementation、
test/implementation/source-docs各review NO FINDINGS、全verification/count/hash、
exact stage/implementation commit、clean tree/origin report/stash不変、その直後の
fresh next-task inventoryである。

Completion evidence: [central Task-269GU historical contract](../../task_contracts/ja/269GU.md#completion-evidence)。

## Task 269GCP frozen term deferral

condition leaf `107..108`/`111..112`はexact selector evidenceだけ。GCPは
`SourcePrimaryTermHandoff`、profile admission、Typed/final term ownerを追加しない。
Task269GCUはexact GC binding/GCT type dependencyをby-value取得後だけtransport可。

Completion evidence: [central Task-269GCP historical contract](../../task_contracts/ja/269GCP.md#completion-evidence)。

## Task 269GC frozen term deferral

GCはterm/occurrence/reference/use-site resolver/equality operand/Typed-final term
nodeをpublishしない。GCP-authenticated `107..108`/`111..112`はopaque。exact GCT
composite後GCUだけがtransport可、descendant useはlater。

Completion evidence: [central Task-269GC historical contract](../../task_contracts/ja/269GC.md#completion-evidence)。

## Task 269GCT frozen term deferral

written typeにterm argumentがないためsource-type argument tableはempty。term/
occurrence/reference/use-site resolver/equality operand/Typed-final term nodeなし。
condition leaf `107..108`/`111..112`はexcludedでexact GCT後GCUだけがtransport可。

Completion evidence: [central Task-269GCT historical contract](../../task_contracts/ja/269GCT.md#completion-evidence)。

## Task 269GCU frozen given-condition term/reference composition

GCUはexact `SourceProofLocalGivenConditionTypeHandoff`をby-value consumeし、
declaration conditionの`y@107..108`/`y@111..112`だけをpublishする。inputは
typed node 3/4、ordinal 0/1、context 1、normal、spelling `y`、
`VariableReference`/`Value`、parentなしのterm 2 rowsと、term 0/1から
`BindingId(1)`へ`Variable`としてlinkするreference 2 rows、numeric request 0。
common producerはuse ordinal 2をderiveし、scope `[0]`、type site
`Source(90..93)`のGCT-owned `GivenWitness`へunique lookupする。

private `SourcePrimaryTermBindingProfile::ProofLocalGivenConditionUse`だけが
`GivenWitness -> Variable`をadmitする。generic/older GU profileは不変。arenaは
GCT reserve-type/type/type-root nodes 0--2、variable-reference nodes 3/4、
`source.proof-local.given-condition.term-root@0..133` node 5 children
`[2,3,4]`のexact 6 nodes。root 5、全node unresolved/unknown/normal/link-free。

public handoff field orderはsource_id、module_id、
`SourceProofLocalGivenConditionTypeHandoff` dependency、dependency fingerprint、
`SourcePrimaryTermHandoff`、term fingerprint。getter、`debug_text`、crate-private
installation validatorを持つ。producer signatureは
`build(dependency, SourcePrimaryTermHandoffInput, &TypedArena)`。
non-exhaustive errorは`InvalidDependency`、`InvalidSourceTerm`、
`InvalidInstallation`で、この順にprecedenceを持つ。

debug headerは
`source-proof-local-given-condition-use-term-debug-v1`、続いてmodule、Rust-
Debug quoted complete GCT fingerprint、quoted source-term fingerprintを各LFで
renderする。Displayは順に
`source proof-local given-condition-use term dependency is invalid`、
`source proof-local given-condition-use source term is invalid`、
`source proof-local given-condition-use term installation is invalid`。

GCUはidentifier occurrence/reference transportだけをownする。label `G`、
equality/formula/condition/fact/guard/proof/obligation、later/descendant
occurrence、capture/export、generic source-term、active credit/dispatch、IRはexcluded。

Completion evidence: [central Task-269GCU historical contract](../../task_contracts/ja/269GCU.md#completion-evidence)。

## Task 269SDP term deferral

`y@118..119`と`z@133..134`はSurface TermReference syntaxだけ。primary-term/
reference winner/use ordinal/arena/type/captureはpublishしない。descendant
`y` occurrenceは後続、`z` replayは`set` reconciliation待ち。

Completion evidence: [central Task-269SDP historical contract](../../task_contracts/ja/269SDP.md#completion-evidence)。

## Task 269SDC frozen term/occurrence deferral

context-2 lookup oracleはabstract `BindingEnv`の継承可視性だけを証明する。
SDCは`y@118..119`のterm arena/reference row/request/source-site mappingを
作らず、descendant occurrence/capture creditは0。`z@114..115`、
`z@133..134`、`q@129..130`はopaque。occurrence consumerは別のGiven
type owner後で、全Set closure/captureはcanonical reconciliationまでblocked。

Completion evidence: [central Task-269SDC historical contract](../../task_contracts/ja/269SDC.md#completion-evidence)。
