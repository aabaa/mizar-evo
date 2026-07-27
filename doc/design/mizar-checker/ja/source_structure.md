# ソース structure-term transport

> 正本は英語です。英語版:
> [../en/source_structure.md](../en/source_structure.md)。

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
