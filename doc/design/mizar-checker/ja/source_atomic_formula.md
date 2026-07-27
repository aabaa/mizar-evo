# ソース atomic-formula transport

> 正本は英語です。英語版:
> [../en/source_atomic_formula.md](../en/source_atomic_formula.md)。

## スコープ

Checker Task 256 は、ordinary predicate application、equality、inequality、
membership、bare builtin type assertion、simple imported attribute assertion
という限定された source atomic formula の syntax-free immutable 記述を所有する。
source occurrence、transparent wrapper、predicate head と resolver candidate
provenance、formula-owned asserted-type/attribute site、nearest-family direct
term edge、unresolved expected-input request だけを運ぶ。

authority は Chapters 9/14 で、Chapters 3/6/13/19 が type、attribute、term、
resolver boundary を定める。Task 252/253/254/255 はそれぞれ primary、
application、structure、set/choice/`qua` term を所有し、Task 256 は row を
複製せず dense root ID を参照する。predicate chain、negation/connective/
quantifier、condition formula、candidate selection、assertion truth、formula
result、theorem acceptance、fact、proof、downstream IR は deferred のままである。

## Public transaction

`SourceAtomicFormulaProducer::build` は
`SourceAtomicFormulaHandoffInput`、`BindingEnv`、`SymbolEnv`、required
`SourcePrimaryTermHandoff`、optional Task-253/254/255 handoff、shared
`TypedArena` を受ける。入力は8個の source-ordered vector を持つ。

- atomic formula
- transparent formula wrapper
- ordinary predicate head
- individually resolver-authenticated predicate candidate
- formula-owned bare asserted-type site
- formula-owned simple assertion attribute
- formula-to-nearest-term-family edge
- unresolved operand/candidate-signature/type-reachability/attribute-
  admissibility request

transaction 全体の validation 後だけ8個の dense immutable table を publish
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
assertion/attribute assertion を区別する。専用 key は predicate head、
asserted type expression/head、attribute occurrence/target、`non`、wrapper
を所有する。

各 direct written term slot は Task 252/253/254/255 の maximal root
occurrence 1件へ対応する。descendant は nearest term family に残る。
duplicate/overlap/partial/non-root/reverse-contained/cross-context target は
atomic に fail する。absent fingerprint の unrelated optional handoff は、
その occurrence が全 formula/wrapper/direct-slot range と disjoint な場合だけ
共存できる。

predicate formula は ordinary head と1件以上の authenticated candidate を
要求し、candidate ごとに candidate-signature request 1件を持つ。
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
production は既存 active fixture 8件、すなわち numeral equality、
inequality、membership、bare builtin type assertion、imported
predicate/functor、positive/negative imported attribute assertion、
set-enumeration equality だけを select する。

8 transaction の Task-256 formula/wrapper/predicate-head/candidate/type-site/
attribute/edge/request aggregate は `8/0/1/1/1/2/13/11` である。shared
lower-family aggregate は Task 252 `16/0/16`、Task 253 `1/1/1/2/2`、
Task 255 `2/0/0/0/4/2` で、real Task-254 target はない。private composer は
各 transaction を1 arena で構築して既存 semantic route をそのまま実行するため、
outcome/detail key は byte-identical のままである。

## Verification boundary

checker test は dense table、formula kind、wrapper、canonical spelling、
provenance、request cardinality、arena/dependency identity、nearest-family
ownership、corruption、deterministic replay、installation、atomic failureを
coverする。runner test は8 exact consumer、ordered edge/request、
lower-family fingerprint、imported provenance/anchor、same-arena composition、
selector isolation、mutation failure、final `TypedAst`/`ResolvedTypedAst`
ownership、unchanged external detail を coverする。

bounded trace row は
`spec.en.checker.type_elaboration.source_atomic_formula_payload` である。
Task 256 は executable source-transport coverage だけを追加し、semantic
formula work と Steps 6/7 は未実装のままである。

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
