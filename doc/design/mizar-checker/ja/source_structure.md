# ソース structure-term transport

> 正本は英語です。英語版:
> [../en/source_structure.md](../en/source_structure.md)。

この Task 254 transport は凍結を維持します。

## スコープ

Checker Task 254 は、source structure construction、selector access、
functional update occurrence の syntax-free / immutable な記述を所有する。
source shape、resolver-authenticated constructor root、written member、
`FieldUpdate` association、ordered child、unresolved request だけを運ぶ。
field/property identity、inheritance view、constructor coverage/default、
selector/update result、value type、fact、acceptance、proof、downstream IR は
決定しない。

正本の言語要件は Chapter 5 §§5.5/5.7 と Chapter 13 §13.3 である。
Task 252 は primary-term child、Task 253 は functor-application child を所有し、
Task 254 は row を複製せず dense root ID を参照する。structure-definition、
member、inheritance-view、constructor semantic payload は Task 263 に残る。

## Public transaction

`SourceStructureProducer::build` は `SourceStructureHandoffInput`、
`SymbolEnv`、`BindingEnv`、`SourcePrimaryTermHandoff`、optional
`SourceFunctorApplicationHandoff`、`TypedArena` を受け取る。入力は次の
7 個の source-ordered vector を持つ。

- structure-family term
- transparent structure wrapper
- resolver-authenticated constructor root
- written constructor / selector / update-path member
- parser `FieldUpdate` association container
- ordered child edge
- unresolved constructor-signature / member-identity / inheritance-path /
  result-type request

transaction 全体を検証した後だけ 7 個の dense immutable table を公開する。
各 public ID は `new` / `index` を持つ zero-based row index、table は `get`、
source-ordered `iter`、`len`、`is_empty` だけを公開し、row field は
read-only accessor だけで公開する。

term kind は `Constructor` / `SelectorAccess` / `FunctionalUpdate`、recovery は
`Normal` / `Degraded` である。member role は `ConstructorAssignment` /
`Selector` / `UpdatePathSegment`、edge role は `ConstructorValue` /
`SelectorBase` / `SelectorArgument` / `UpdateBase` / `UpdateValue` である。
target は Task-252 `Primary`、Task-253 root `Application`、後続 Task-254
`Structure` row のいずれかである。

## Public Enum Policy

| Public enum | compatibility policy |
|---|---|
| `SourceStructureTermKind` | `#[non_exhaustive]`。callerはlater structure-family source kindを許容する。 |
| `SourceStructureRecovery` | `#[non_exhaustive]`。callerはlater recovery classを許容する。 |
| `SourceStructureMemberRole` | `#[non_exhaustive]`。callerはlater written-member roleを許容する。 |
| `SourceStructureEdgeRole` | `#[non_exhaustive]`。callerはlater child-edge roleを許容する。 |
| `SourceStructureTarget` | `#[non_exhaustive]`。callerはlater frozen cross-family targetを許容する。 |
| `SourceStructureRequestKind` | `#[non_exhaustive]`。callerはlater unresolved request kindを許容する。 |
| `SourceStructureError` | `#[non_exhaustive]`。callerはvalidation failureをexhaustive matchしない。 |

この module が所有する exhaustive public enum exception はない。

## 検証と ownership

producer は source/module identity、dense source preorder、context、range、
canonical spelling、recovery、exact arena anchor、group/ordinal、single ownership
を認証する。term site は `source.term.structure.constructor` / `.selector` /
`.update` を使う。written member は role ごとに
`source.term.structure.member.constructor-assignment`、
`source.term.structure.member.selector`、
`source.term.structure.member.update-path-segment` を使う。whole update
container は `source.term.structure.field-update`、transparent wrapper は
`source.term.structure.parenthesized` を使う。

各 constructor は resolver-authenticated `Structure` root exactly 1 件を持つ。
local root は normal / conflict-free / source-preceding definition と exact
symbol/definition/contribution cross-index agreement、imported root は public
exported/re-exported provenance と authenticated import effect を要求する。
missing/pending/opaque signature shell は unresolved のまま許容し、
malformed/recovered provenance は fail closed する。

constructor assignment と update value は final member を保持する。
selector/update base は member を持たない。update-path segment は source-ordered
parent chain を作り、`FieldUpdate` は nonempty path 1 件と replacement edge 1 件を
ownするが term ではない。repeated label/path は Task 263 のため distinct ordered
row のまま保持する。

Primary child は same-context Task-252 root でなければならない。Application
child はどの Task-253 argument edge からも target にされない same-context
Task-253 root で、nested application は multiply own せず reject する。
Structure child は incoming edge 1 件を持つ後続 same-context Task-254 row である。
structure child を含む reverse Task-253 application とその他 frozen exclusion は
detached descendant なしで fail closed する。

各 structure term の edge list は、source-order の direct written child
全体と exactly 一致する。対象は Task-253 argument が既に own するものを除く
parentless Task-252 root、Task-253 root application、後続 Task-254 term である。
別 candidate に包含される candidate は近い child に属し、outer term へ detached
しない。direct child の省略、重複、retarget は許さない。constructor value は
assignment label の後かつ次 label の前、selector base は selector member の前に
終了し、argument は member の後に始まる。update base は最初の `FieldUpdate`
より前に終了し、replacement は owning `FieldUpdate` に strictly contained され
final path member の後に始まる。`FieldUpdate` spelling は written path を
` . ` で連結し、` := ` と effective replacement spelling を続けた文字列と
exactly 一致する。

## Derived dependency fingerprint

output の `primary_term_fingerprint` は exact Task-252 `debug_text()` から導出する。
`application_fingerprint` は application edge がある場合だけ exact Task-253
`debug_text()` 由来の `Some`、それ以外は `None` である。unrelated install 済み
Task-253 handoff は `None` と共存できる。

`TypedAst::with_source_structure` は one-shot で、Task 252 と target される Task 253
dependency の先行 install を要求し、producer-validated written partitionを保持して
fingerprint、target、cross-family ownership、arena siteを再検証する。
さらに `with_source_application` は既に install 済みの Task-254 handoff を
revalidateするため、Task-253 argumentによるTask-254 primary targetの再所有、
Task-253 applicationによるTask-254 termの包含、partial overlap、ownership
bypassはどちらのinstall順でもrejectする。`None` fingerprintと共存するunrelated
Task-253 handoffはrange/targetがTask 254とdisjointな場合だけ有効である。
`ResolvedTypedAst` は dense ID を rebuild/retarget せず同じ association を
revalidate / clone-preserve する。両ASTのdebug renderingはhandoffがpresentの
場合だけそれを含む。

Task 255が既にinstall済みなら、`with_source_structure`はTask 254 publish前にその
structure fingerprint、root-only target、nearest-family range partitionも
revalidateする。したがってlater structure handoffはinstalled Task-255
occurrenceをcontain/overlap/retargetできない。

## Private source consumer

raw `SurfaceAst`、source node ID、syntax kind は
`mizar-test::runner::type_elaboration::source_structure` だけに置く。production は
`fail_type_elaboration_local_structure_term_gap_001` の 3 functor definiens だけを
selectする。
leafはreal declaration shellをconsumeしてTask 248の
`SourceBindingContextProducer`を再利用し、generated definition contextを捏造しない。

exact term/wrapper/root/member/field-update/edge/request oracle は
5/0/3/9/2/10/26、同じ arena の Task-252
primary/reference/numeric-request slice は 8/0/8 である。real route に Task-253
row/fingerprint はない。transport 後は public diagnostic なしで Task-263
`type_elaboration.external_dependency.ast_payload_extraction` boundary を保持する。

## Verification boundary

checker test は dense table、5個すべてのarena key/wrong-key substitution、
member/path/`FieldUpdate` ownership、wrapper、local/imported root provenance、
request cardinality、Task-252/253/254 child、Task-253 root-only ownership、
conditional fingerprint 全matrix、corruption、determinism、installation、
clone preservation、atomic failureをcoverする。runner testはexact consumer/
oracle、lower-stage shape、synthetic child family、recovery、exclusion、mutation
isolation、deterministic replay、final ownership、他の全active type-elaboration
case exclusionをcoverする。

bounded trace row は
`spec.en.checker.type_elaboration.source_structure_term_payload` である。
Task 254 は MC-G017/MC-G018 の executable coverage を増やすが、semantic
structure/member/view behavior、later term family、accepted fact/proof、
Steps 6/7 は未実装のままである。

## Task 258B3M2B2B2P frozen proof-context reuse seam

B2Pがfreezeするのはcrate planのexact 172-byte/76-node proof sourceに対する、
existing public Task-254 constructor producerのfuture runner-private reuse
だけ。owned-kind mapはconstructor node 59の
`source.term.structure.constructor`とmember nodes 20/24の
`source.term.structure.member.constructor-assignment`だけ。qualified root
node 52は`source.surface.unowned`のままresolver-provenance traversalだけに
participateする。Task 252はnodes 54/57をprivate extraction rootsとしてのみ
使用してnumeral rowsをnodes 53/56でpublishするため、53/56は
`source.term.numeral`、54/57は`source.surface.unowned`のままであり、
他nodeをTask-254-ownedにしない。

handoffはexisting `BindingContextId(1)`とshared `SourceTermParts`を使い、
Task-48 `2/1/0`、Task-252 `6/4/2`、exact Task-254
`1/0/1/2/0/2/6` constructor/root/member/edge/request profileをpreserve。
root provenanceはimported public/exported/signature-free
`parser.type_fixtures::TypeCaseStruct#5`、edgesはmembers 0/1からprimaries
2/3だけ、application fingerprintはabsent。

later implementationはmizar-test source-structure leafに限定し、checker/
statement/witness APIをpublishせずlegacy Task-254 route/debug outputを
byte-compatibleに保つ。Chapter 5 §5.7 selector authorityはexplicitにexclude
し、current constructor semanticsではなくfuture B2B work。`FieldUpdate`/
functional updateはB2C。frozen runner tests 2件はbytes/nodes、ownership/
provenance、corruption precedence、stale/clean replay、legacy output、
empty upper familiesをexhaustし、checker testはない。

## Task 258B3M2B2B2P private reuse result

runnerはfrozen exact-source owned-kind selectorと
existing-context/shared-Task-252からunchanged public Task-254 producerへの
callをimplementした。exact runner tests 2件はlower profile、ownership、
resolver provenance、corruption precedence、stale replay、legacy outputを
含めpass。checker source/API、selector/update semanticsは変更せず、
B2Aが次consumer。

exact resultはTask-48 `2/1/0`、Task-252 `6/4/2`、Task-254
`1/0/1/2/0/2/6`、owned kinds 59/20/24、numeral sites 53/56、
unowned 52/54/57をpreserveする。imported public/exported、
signature-free `TypeCaseStruct#5` contribution 2/current-source origin
`7..27/[5]`をauthenticateする。malformed recovery near missは
`diagnostics=1, nodes=74, root=73, recovered=[52]`。existing Task-254
source-structure/typed/final debug hashes
`0d6af57b89e6156d8e5de6831568c81ec110880bebf1e4aeb4ab00563f4da6c8`,
`8264d1574faf67e19b6b84d6e11fa7ab6435335238b398fa0966bbfbc63d0599`,
`118a998bc5edb770c7818be1d74cbece0f566353bf9d3e6aabb817d994a3db40`
は不変。

## Task 258B3M2B2B2A frozen structure consumer

B2Aはcompleted B2Pのexact Task-254 handoffをconsumeするがbroadenしない。
structure term 0はproof context 1のconstructor 59、root 0、members 20/24、
`Primary(2/3)` edges、unresolved requests 6件、application fingerprint
なし。new statement witnessはterm 0をtargetし、resolver root/member/
field valueはtargetしない。

existing Task-254 public producer/tables/validation/debug bytes/legacy routeは
unchanged。field/property identity、coverage/defaults、value/result typing、
inheritance、selector、update、`FieldUpdate` semanticsはTask 263/B2B/B2C。

## Task 258B3M2B2B2A implemented structure consumer

statement routeはcompleted B2P seam/exact Task-254 handoffをpublic producer/
rowsを変えずconsumeする。structure term 0はproof context 1のconstructor
59、root 0、members 20/24、`Primary(2/3)` value edges 2件、requests 6件、
application fingerprintなし。statement witnessだけがdirected targetを
追加する。

B2P seamがliveになったためprivate dead-code allowanceをremoveしたが、
visibility/extraction/ownership/validation/debug bytesはunchangedで、
`source_structure.rs`は5,036 linesのまま。
selector、update、`FieldUpdate`、field identity、typing、inheritance、
semantic behaviorはdeferred。

## Task 258B3M2B2B2BP frozen proof-context selector reuse

B2A post-commit inventoryでgeneric Task-254 extractorは`SelectorAccess`を
model済みだが、proof-context private seamはconstructor-onlyであると
判明した。B2BPはTask-258 B2B consumerより前にrunner-private
`ImportedStructureSelectorSite`、
`imported_structure_selector_owned_node_kinds`、
`imported_structure_selector_handoff_in_context` siblingsをfreezeする。

exact 171-byte sourceのTask 254 outputは`2/0/1/3/0/3/9`。selector term 0
node 62は`SelectorBase`でconstructor term 1 node 61へ、constructor
member valuesは`Primary(2/3)`へ向く。owned nodesは
`62/61/29/20/24`。existing extractor/producer、binding context 1、
shared Task-252 roots、imported `TypeCaseStruct#5` provenance、current
debug grammarをreuseし、constructor B2P/B2A bytesをpreserveする。

checker API、Task-256/258 row、TypedAst statement installation、public
runner route、selector identity/type result、semantic behaviorはownしない。
future runner tests 2件だけがB2B前のprivate seamをfreezeする。

## Task 258B3M2B2B2BP implementation result

frozen production-private siblings 3件をrunner source-structure leafへ
実装し、existing generic Task-254 extractorとTask-252 proof-context
rootsをreuseする。full source/arena/provenance/ownership/fingerprint
authentication後にexact `2/0/1/3/0/3/9` lower tableだけをpublishする。
frozen tests 2件はPASSし、B2P、B2A、legacy Task-254、empty upper-family
bytesをpreserveする。

checker source/public APIは変更しない。selector identity、typing、
result、inheritance、proof、goal、theorem behaviorはこのtransport seamの
scope外。

## Task 258B3M2B2B2B frozen consumer boundary

B2BPはB2Bがconsumeするsole production-private lower seamとなった。同じ
171-byte sourceについてTask 254はbyte-for-byte
`2/0/1/3/0/3/9`のまま。selector term `0` node `62`は
`SelectorBase`でconstructor `Structure(1)`をtargetとし、selector
member identity node `29`をownする。constructor term `1` node `61`は
member identities nodes `20/24`と`Primary(2/3)`へのvalue edgesをownする。imported
root `TypeCaseStruct#5`はcontribution `2`、origin `7..27`、path `[5]`
をretainする。

B2Bはauthenticated tableをconsumeしてwitness node `64`を
`Structure(0)`へattachするだけ。Task-254 ownership/bytesはTask 258へ
移さない。implementationはrunner `source_statement.rs`でseamをconsumeし、
`source_structure.rs`からobsolete B2BP `dead_code` allowancesだけを
removeできるが、Task-254 extraction/public surface/existing testsは変更
しない。selector identity/type/result、inheritance、functional update、
`FieldUpdate`、全semantic behaviorはdeferred。

## Task 258B3M2B2B2B implemented consumer result

B2Bはfrozen B2BP private selector owned-kind/proof-context handoff seams
だけをconsumeする。authenticated Task-254 tableは
`2/0/1/3/0/3/9`のまま。node 62のselector `Structure(0)`はnode 61の
constructor `Structure(1)`を指し、membersは`29/20/24`、value edgesは
`Primary(2/3)`。Task 258が追加するのはwitness-to-selector edgeだけ。

B2BP extractor、lower rows、provenance、public surface、existing testsは
unchangedで、obsolete consumer-use `dead_code` allowancesだけをremove
した。checker `source_structure.rs`は5,036 linesのまま、runner
source-structure leafはcleanup後4,506 lines。selector
identity/type/result、inheritance、update/`FieldUpdate`、proof、goal、全
semantic behaviorはdeferred。

## Task 258B3M2B2B2CP frozen proof-context update reuse

fresh post-B2B inventoryではgeneric Task-254 extractorがfunctional
updatesをmodelする一方、production-private proof-context reuse surfaceは
constructor/selector profilesだけ。B2CPはB2C statement consumerより先に
runner-private `ImportedStructureUpdateSite`、owned-kind、in-context
handoff siblingsをfreezeする。

exact 181-byte/86-node sourceでTask 254は`2/0/1/3/1/4/9`をpublish。
functional update `Structure(0)`はnode 69、constructor `Structure(1)`は
node 65。membersはupdate path 30とconstructor assignments 20/24。
`FieldUpdate(0)`はnode/range `68/153..159`、spelling `x := 3`、member
0をown。edgesはupdate base -> `Structure(1)`、update value/member 0 ->
`Primary(4)`、constructor values/members 1/2 -> `Primary(2/3)`。
imported root `TypeCaseStruct#5`はcontribution 2、origin `7..27`、path
`[5]`をretainし、application fingerprintはabsent。

B2CPはTask-258 witness/statement rowをownしない。unchanged Task-254
public producerをreuseし、exact-source private selection、owned-kind
authentication、existing proof context、shared Task-252 partsだけを
freezeする。tests 2件は全byte/node、lower rows/corruptions、exact
missing-value recovery、replay、exact B2P constructor/B2BP selector
compatibility、empty upper familiesをcoverする。functional-copy
semantics、member identity、replacement/result typing、proof/goal/theorem
behavior、B2C witness ownershipはdeferred。

Task 256がlater ownするのはnodes `55/77`だけで、full update subtreeを
excludeする。containers `56/78`はunowned。B2Cだけがlater
take/witness nodes `72/71`をownし、そのwitnessをfunctional-update
`Structure(0)`へattachできる。B2CPはこれらupper rows/edgesをownしない。

Completion evidence: [central Task-258B3M2B2B2CP historical contract](../../task_contracts/ja/258B3M2B2B2CP.md#completion-evidence)。

## Task 258B3M2B2B2C frozen update consumer

B2Cはprivate B2CP
`ImportedStructureUpdateSite`、
`imported_structure_update_owned_node_kinds`、
`imported_structure_update_handoff_in_context` seamsをunchangedでconsume
する。Task-254は`2/0/1/3/1/4/9`のまま。update `Structure(0)`はnode
69、constructor `Structure(1)`はnode 65、membersは30/20/24、
`FieldUpdate(0)`はnode/range `68/153..159`。edgesはupdate baseから
`Structure(1)`、update value/member 0から`Primary(4)`、constructor
valuesから`Primary(2/3)`のまま。imported `TypeCaseStruct#5` provenanceは
contribution 2、origin `7..27/[5]`、public/exported/normal、
signature-freeのまま。

B2Cはstructure rowもpublic structure APIも追加しない。existing
`Structure(0)`へのstatement witness edgeだけを追加する。Task-256
equality nodes 55/77はupdate subtree全体をexcludeし、containers
56/78、transparent 70、root 58、private roots 60/63/67はunownedの
まま。functional-copy、member/replacement/result typing、proof、goal、
theorem semanticsはdeferred。consumer implementation時にobsoleteな
B2C-future `dead_code` allowancesだけをremoveしてよい。

## Task 258B3M2B2B2C implemented update consumer

B2C runner consumerは`ImportedStructureUpdateSite`、
`imported_structure_update_owned_node_kinds`、
`imported_structure_update_handoff_in_context`をunchanged利用し、obsolete
future-consumer `dead_code` allowancesをremoveした。structure table row/
edge/request/fingerprint grammar/public API/Task254 ownershipはunchanged。
witnessはexisting update `Structure(0)`だけをtargetとする。

focused checker/runner matricesはPASSしimplementation reviewはfindingsなし。
functional-copy、member/replacement/result typing、immutability、proof、
goal、theorem semanticsはdeferred。

## Task 258B3M2B2B2C broad structure verification

fmt/Clippy/crate/workspace gates、focused `4/4`/`5/5`、sibling
`12/12`/`21/21` suitesはunchanged Task254 inventory/hashesでPASS。B2Cは
existing structure handoffのconsumerだけで、structure/semantic ownership
追加なし。independent final source/docs/quality reviews、commit、
post-commit inventoryはpending。

Completion evidence: [central Task-258B3M2B2B2C historical contract](../../task_contracts/ja/258B3M2B2B2C.md#completion-evidence)。
