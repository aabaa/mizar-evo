# Task PARSER-TEMPLATE-TYPEHEAD-277P1: template type-head 保存 prerequisite

> canonical English: [../en/PARSER-TEMPLATE-TYPEHEAD-277P1.md](../en/PARSER-TEMPLATE-TYPEHEAD-277P1.md)。正本は英語です。

Owner plan: [mizar-parser](../../mizar-parser/ja/00.crate_plan.md) と
[mizar-frontend](../../mizar-frontend/ja/00.crate_plan.md)。

stable owner section:

- parser [grammar](../../mizar-parser/ja/grammar.md#parser-template-typehead-277p1-required-template-type-head)、[source/spec audit](../../mizar-parser/ja/source_spec_audit.md#parser-template-typehead-277p1-audit-freeze)、[TODO](../../mizar-parser/ja/todo.md#independently-authorized-template-type-head-prerequisite)、[bilingual audit](../../mizar-parser/ja/bilingual_documentation_synchronization.md#parser-template-typehead-277p1-pair-freeze)、[exit qualification](../../mizar-parser/ja/crate_exit_report.md#post-closeout-qualification-parser-template-typehead-277p1);
- frontend [parsing](../../mizar-frontend/ja/parsing.md#parser-template-typehead-277p1-parser-version-freeze)、[cache key](../../mizar-frontend/ja/cache_key.md#parser-template-typehead-277p1-cache-assessment)、[orchestration](../../mizar-frontend/ja/orchestration.md#parser-template-typehead-277p1-replay-contract)、[source/spec audit](../../mizar-frontend/ja/source_spec_correspondence.md#parser-template-typehead-277p1-follow-through-audit)、[TODO](../../mizar-frontend/ja/todo.md#independently-authorized-template-type-head-follow-through)、[bilingual audit](../../mizar-frontend/ja/bilingual_documentation_synchronization.md#parser-template-typehead-277p1-pair-freeze)、[exit qualification](../../mizar-frontend/ja/crate_exit_report.md#post-closeout-qualification-parser-template-typehead-277p1)。

## 状態、authority、readiness

| 項目 | freeze 値 |
|---|---|
| 状態 | documentation は freeze 済み、implementation は別 commit で pending。 |
| 選択 checkpoint | 読み取り専用 inventory で `HEAD=2a6bf6ea`、`origin/main...HEAD=0/2`、protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4` を観測した。これは historical selection evidence であり implementation evidence ではない。 |
| Authority | `doc/spec/en/18.templates.md` §§18.2.2、18.2.6、18.10.2 と `doc/spec/en/13.term_expression.md` §13.4 と §13.4.2。 |
| 分類 | `source_drift` と Rust `test_gap`。`spec_gap` はない。 |
| 上位 consumer | Checker Task 277B は別の resolver declaration/use-identity prerequisite が残るため、本 task 後も blocked のままである。 |

immutable semantic seed は
`tests/miz/fail/templates/fail_template_fraenkel_over_type_param_001.miz`
(701 bytes、final LF、SHA-256 `32c4a1c1b6c9d98dcb085558a084929e07d4005bf92595865f144456e95854ec`)。
839-byte sidecar は SHA-256
`b47ac5113c89cd5703adb0ffd660b52d3e16908c92623dd2f63196aa6a215cb2`。
両方とも byte-identical のまま `advanced_semantics` で inactive に保つ。

## Freeze した挙動と境界

template-shaped `definition` block の内部でのみ、comprehension generator の
`identifier "is" type_expression` が要求する位置では bare one-token
`Identifier` を `TypeHead` として保存する。parse-local
template-definition scope/depth と explicit required-type policy を使う。
ordinary strict type parse を先に試し、scope 内でだけ one-token identifier
fallback を許す。`UserSymbol` は既存の `QualifiedSymbol` shape を維持する。
binding、name resolution、type inference、semantic identity、sethood 判定はしない。

global speculative type parse、template argument、bracket type argument、
`is` assertion alternative への fallback 拡張は禁止する。multi-token identifier、
block 外への leak、ordinary definition の変更、public API shape/diagnostic
vocabulary の変更も禁止する。frozen target の diagnostic vector と public
cache-version value だけは本 contract の通り変更する。

成功時は root `56` / 57 nodes、diagnostic/recovery は 0、`TypeExpression` と
`TypeHead` は `678..679`、generator `673..679`、condition `682..692`、
`SetComprehension` `663..694`、`FunctorDefinition` `623..695`、
`DefinitionBlockItem` `593..700` である。bare `T` は §18.10.2 に従い、後段の
semantic verification が missing sethood として拒否する。

## Exact scope、tests、cache

後続 implementation commit の Rust scope は
`crates/mizar-parser/src/{grammar.rs,module.rs,module/tests.rs}` と
`crates/mizar-frontend/src/{parsing.rs,orchestration.rs}` の5 pathだけである。
parser tests は `226 -> 229`:
`parser_parses_template_type_parameter_as_set_comprehension_generator_type`、
`parser_scopes_required_identifier_type_fallback_to_template_definitions`、
`parser_keeps_template_required_type_fallback_single_token_and_ambiguity_bounded`。
frontend は `153 -> 154` で
`task277p1_template_typehead_changes_ast_cache_namespace` を追加する。

同じ input が既に recovered AST を返すため、
`MIZAR_PARSER_CACHE_KEY_VERSION` は v2 から v3 に bump する。frontend replay は
v3 と unchanged control を確認する。これは cache namespace のみであり、frontend
grammar/semantic decision ではない。

## Inventory と documentation scope

parser baseline は 12 paths / 39,159 lines、path
`192f9d0b5e6534c4daab010ec51a9356e9e0fd6fb86876bd2600a75844e7566a`、content
`60a82837ae275862949212d21e3429cbe489763400f0f503dc1fa3daa09d761e`;
`grammar.rs` 243 lines /
`a4673e82d9866c5499ae919b09189b3902e9d62f293c3e7c9f8b96b1190ef476`、
`module.rs` 16,761 /
`e5464621161b45e6f7744c97b938d7ac82bd78f62870a39e36a45c4eace11b8d`、
tests 18,490 /
`bac4e49eaa094708aa6d7bcede96ca304b95c4d41366ef199f4cb59da469c085`、raw list
`05194f5916812d24c36130e60275e4ba9933ad9d2ee81cf4026e2379e383dbad`。
frontend は 9 paths / 11,030 lines、path
`c7b20f1ab8f9414109dce18eb6591a6f18c0b413cd5a11beb84fa1b785822101`、content
`03a81d83e7af3b12ce21eb028de2920cbf8e4a9f5a873aaad53cdbf1487b7b6e`;
`parsing.rs` 1,459 /
`c26f56bc2a052fd1d2660f0b2f545a48c1f4390da34feba534ebb5d750c17824`、
`orchestration.rs` 1,908 /
`84b49e07a831c87484a725d457e4f6611ed03e422d1417d2d2d71ee2a091166b`、raw list
`298dbed058e39cfe83d5381d49fe1d3d014cf3412d148c568f7470d440731d30`。
contract pair は `82/82 -> 83/83`。

documentation prerequisite は exactly 30 Markdown paths、すなわち本 EN/JA
contract pair、paired parser `00.crate_plan`、`grammar`、`source_spec_audit`、
`todo`、`bilingual_documentation_synchronization`、`crate_exit_report` と、paired
frontend `00.crate_plan`、`parsing`、`cache_key`、`orchestration`、
`source_spec_correspondence`、`todo`、`bilingual_documentation_synchronization`、
`crate_exit_report` である。implementation completion docs は plan を除く exactly
26 paths、Rust 5 path と合わせて 31 paths。
`doc/spec`、既存 `.miz`/expectation、trace、active stage/route、coverage、
`doc/design/spec_coverage_audit.md`
(`a31f6fb3bd2b561610630497c58284484d00716dd0b7f210f55bef3bc4bfa6db`)、Cargo、
frozen target 以外の diagnostic vocabulary/order、明示的に変える public
cache-version value 以外の public API shape、mizar-test/syntax/resolve/checker/Core
は protected。frozen target の diagnostic vector は recovered syntax diagnostic
から 0 件へ contract 通り変わるが、新しい diagnostic code/message vocabulary は
追加しない。trace
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3` は不変。

## Exit evidence

focused parser/frontend tests、replay/cache、fmt、denied-warning Clippy、workspace
tests、five CLIs、count/hash remeasurement、recursive contract/link lint、
`git diff --check` と、NO FINDINGS の independent reviews を別 implementation
commit 前に満たす。

## 次 task handoff

`PARSER-TEMPLATE-TYPEHEAD-277P1` を frozen Rust 5 path だけで実装し、parser test
3件と frontend replay/cache test 1件だけを追加する。semantic seed、expectation、
trace、coverage、diagnostic vocabulary、resolver/checker behavior は不変に保ち、
completion document exactly 26 path を同期して全 review/verification gate を通す。
authority、integration、stage、final scoring を担当する親は GPT-5.6 Sol `xhigh` を
維持する。本 checkpoint では GPT-5.6 Luna が exposed されていないため、独立して
bounded な parser/frontend implementation/review には GPT-5.6 Terra `high` を使い、
scope/authority ambiguity は Sol `xhigh` 親へ escalate する。
