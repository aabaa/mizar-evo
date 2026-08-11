# Task RESOLVE-FRAENKEL-NESTED-CAPTURE-257C4C2: exact nested Fraenkel resolver identity

> canonical English: [../en/RESOLVE-FRAENKEL-NESTED-CAPTURE-257C4C2.md](../en/RESOLVE-FRAENKEL-NESTED-CAPTURE-257C4C2.md)。正本は英語であり、本書はlogical synchronized Japanese companionである。

Owning planは[mizar-resolve](../../mizar-resolve/ja/00.crate_plan.md#task-index)と
[mizar-test](../../mizar-test/ja/00.crate_plan.md#task-index)。durable ownerはresolver
[names](../../mizar-resolve/ja/names.md#resolver-task-257c4c2-exact-nested-fraenkel-identity)とtest
[harness](../../mizar-test/ja/harness.md#resolver-task-257c4c2-private-imported-fixture-probe)。

## status、purpose、readiness

**Status:** frozen documentation prerequisite。独立specification review後にのみ実装する。

clean `HEAD b7f52dfa8d804c0adb4896cc5f1b9473ac99431c`のfresh inventoryは、既存の
import済みnested-comprehension oracleに対するresolver-owned最小依存taskを選定した。
既存`FraenkelGeneratorVariableSourceCollection`をexact profileだけ拡張する。checker capture、
Task 252 occurrence、type/sethood、request、verdict、diagnostic、active runner、Task 277Bは実装しない。

Chapter 13 §§13.4.4/13.8.6はinner mapper `x`がouter generatorをresolved binder identityで
参照することを一意に要求し、既存`.miz`/inactive sidecarはinner `y`をdistinctに保つexact positive
relationを固定する。C4C1はzero-diagnostic frontend admissionを完了し、R2は必要なresolver
binding/use tableを既にownする。従ってblocking `spec_gap`はない。全nested拒否はこのexact profileに
対する`source_drift`、missing focused testは`test_gap`、owner documentのunqualified exclusionは
`design_drift`。C4A/C4B reuse、Task-252 row、checker capture publicationは`boundary_violation`となる。

## authorityと依存

authority orderはcanonical Chapter 4 §4.6とChapter 13 §§13.4.2/13.4.4/13.8.6、既存
`pass_types_nested_comprehension_outer_generator_capture_001.miz`、sole trace row、inactive expectation、
completed R2/C4C0/C4C1、derived owner/source inventoryの順。sourceは`164` bytes / SHA-256
`c3b8bd62c16406ccedee2e64a71ef62a5c4b329d2319be33ad3834a9541af431`のまま。
sidecarはinactive `advanced_semantics`、`pass/type_check`、diagnostic code/active tagなし。
traceはtest intentだけでexecution/semantic creditを与えない。

## frozen exact resolver relation

既存R2 public type/getter/error/enum/debug grammarをbyte-for-byte reuseし、新規public itemは追加しない。
exact normal C4C0 sourceのcollectionは次の2 bindingと1 mapper useを返す。

| Binding ID | comprehension | spelling | segment | binder | ordinal |
|---:|---|---|---|---|---:|
| `0` | inner | `y` | `102..121` | `102..103` | `0` |
| `1` | outer | `x` | `136..155` | `136..137` | `1` |

sole useはinner mapper `x@94..95`。`comprehension()`/`role_owner()`はinner側、`binding()`はouter
binding `1`、roleは既存`Mapper`、global/mapper-local ordinalはいずれも`0`。debug grammarは既存のまま
`bindings=2|uses=1`で終わる。これはresolver identityだけでchecker capture row/core parameterではない。

Admissionはdefault-deny。unrecoveredな1 definition/functor、outer condition-free comprehension、
そのmapperである1 inner condition-free comprehension、各1 generator、exact inner mapper `x`、distinct
inner `y`/outer `x`、2件のexact `Element of NAT` type expressionだけを認める。各type subtreeは
attribute chainなしのnormal `TypeExpression`、direct `TypeHead`、sole spelling `Element`の
`QualifiedSymbol`、reserved word `of`から始まりsole spelling `NAT`のnormal term expression 1件を持つ
direct `TypeArguments`で、rangeは`107..121`/`141..155`。recovery、condition、extra/missing/reordered
generator、inner/outer同名、outer `x`以外のinner mapper、いずれか一方のalternate/different-shaped
generator type、extra reference/nesting/wrapper、partial matchはcandidate zero row。既存F5 outputと
malformed synthetic nested exclusionはbyte-compatibleに保つ。
全identityは`resolved_node_for`由来で、global source-range/node order、dense ordinal、`new`/`collect`
complete-arena revalidationを維持し、raw `SurfaceNodeId`をpublic APIへ出さない。

## implementation/test scope

Rust変更は`crates/mizar-resolve/src/names.rs`、`crates/mizar-resolve/src/names/tests.rs`、
`crates/mizar-test/src/runner/tests.rs`、new private
`crates/mizar-test/src/runner/tests/type_elaboration/fraenkel_nested_capture_identity.rs`のexact 4 paths。

resolver testは次のexact 4件。

1. `task257c4c2_collects_exact_nested_capture_relation`;
2. `task257c4c2_preserves_outer_scope_and_distinct_inner_binding`;
3. `task257c4c2_rejects_near_miss_nested_profiles`（otherwise-normal alternate inner/outer typeを含む）;
4. `task257c4c2_revalidates_arena_and_replays_deterministically`。

private mizar-testは
`task257c4c2_real_imported_fixture_links_inner_mapper_to_outer_generator` 1件。C4C1 frontend helper/providerを
reuseし、real AST lower、direct collector、exact public relation、normal kind/range、replay、empty resolver
augmentation不変だけをassertする。production/advanced-semantics routeではない。raw library countは
resolver `156 -> 160`、mizar-test `618 -> 619`。baselineは`names.rs` `3920` lines /
`9a4b1a0e289c058a40c5af91d00fb836eca7af3a1d83bfcfa9b60227ce46d14a`、`names/tests.rs`
`3601` / `31228c3502a08276a0c395715f74a6a5143a11c315145595ac88f93163e6863a`、`tests.rs`
`67` / `94bc44e8ba47ca568670adeec74d20f6738b3fc337da2422871095137040e8c4`。new leafはabsent。
raw sorted test-list hashはresolver `7c84ee615616d7f0982454c8d04e9eef2fcb451efbb8fd576296e28af3cb6301`、
mizar-test `d145e5bf5c8ae3f8231ffe73ee034b639001d349c99dd4f00f3c60b6382db4c1`。contract treeは
`91/91 -> 92/92`。

## protected scope、audit、exit

`doc/spec`、既存`.miz`/expectation/trace、frontend/parser/import provider、resolver import augmentation、
checker、C4A/C4B、Task 252、Typed/Resolved checker installation、Cargo、diagnostic、dispatch、active coverageは
変更しない。inactive expectationを再解釈せず、executable passをclaimしない。

`doc/design/spec_coverage_audit.md`にはzero-credit design mappingだけを追加する。resolver identity owner
完了後もchecker capture、Task-252 occurrence、type/sethood、request/verdict/diagnostic/production、Task 277Bは
deferred。trace/expectationのintent/statusは不変なのでbyte-identicalに保つ。

independent specification/contract、test sufficiency、implementation、source/docs/API、bilingual/boundary、
final quality reviewを行い、material findingごとにre-reviewする。focused 5 tests、resolver/mizar-test library、
両lint、metadata、format、workspace warnings-denied Clippy/full tests、5 CLI replay、protected hash、scope/diff、
postcommit proofを通す。**NO FINDINGS**、9 hard gates、valid `>=90/100`、task-only commit、clean fresh
inventoryでexitする。separate successorがdependency-readyと証明できなければ停止し、Task 277Bは
not-ready/zero creditのまま。

推奨routingはauthority/API/boundary/final scoreをGPT-5.6 Sol `xhigh`、frozen implementation/reviewを
Terra `high`/`xhigh`。new public item、general nested/shadow semantics、checker payload、diagnostic、active routeが
必要ならedit前に親へ戻す。
