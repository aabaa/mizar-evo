# Task RESOLVE-TEMPLATE-TYPEPARAM-277R1: resolver template type-parameter identity prerequisite

> canonical English: [../en/RESOLVE-TEMPLATE-TYPEPARAM-277R1.md](../en/RESOLVE-TEMPLATE-TYPEPARAM-277R1.md)。正本は英語です。

Owner plan は [mizar-resolve](../../mizar-resolve/ja/00.crate_plan.md#task-index)
と [mizar-test](../../mizar-test/ja/00.crate_plan.md#task-index) である。

Stable owner surface は resolver の `names`、source/spec correspondence、module
boundary、TODO、bilingual record、exit addendum と、mizar-test の harness、module
boundary audit、TODO、bilingual audit である。英語正本の stable link を正準とし、
本 companion はそれらの所有境界を論理的に同期する。

## Status、authority、classification

| 項目 | freeze 値 |
|---|---|
| Status | documentation prerequisite は `2438cbb7d39c1844557293b270ef1784cfc31ece` としてcommit済み。exact 4-path Rust implementationとtest 5件はcomplete/verified。independent source/documentation、bilingual、final-quality reviewは**NO FINDINGS**。全9 hard gatesはscore capなしのvalid `100/100`でPASS。exact staging、commit、post-commit inventoryが残る。 |
| Selection checkpoint | `HEAD=0827e494df96afacba4f35b9cc23dfbbb737d141`、`origin/main...HEAD=0/5`、protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4` は unchanged。 |
| Authority | `doc/spec/en/18.templates.md` §§18.2.1、18.2.2、18.2.6、18.10.2 と `doc/spec/en/13.term_expression.md` §13.4.2。parser prerequisite は [PARSER-TEMPLATE-TYPEHEAD-277P1](./PARSER-TEMPLATE-TYPEHEAD-277P1.md)。 |
| Classification | `source_drift`、`design_drift`、Rust `test_gap`。`spec_gap` はない。後段の missing-sethood verdict は checker-owned のまま。 |
| Consumer | これは後続 Task 277B の resolver prerequisite のまま。completionはTask 277B readyやchecker implementationを選択しない。 |

immutable semantic seed は
`tests/miz/fail/templates/fail_template_fraenkel_over_type_param_001.miz`
(701 bytes、final LF、SHA-256
`32c4a1c1b6c9d98dcb085558a084929e07d4005bf92595865f144456e95854ec`)。
839-byte sidecar は SHA-256
`b47ac5113c89cd5703adb0ffd660b52d3e16908c92623dd2f63196aa6a215cb2`で、
`advanced_semantics` inactive のまま不変である。

## Freeze した real-source profile

real parser profile は root `56` / 57 nodes、diagnostic なし。

| role | Surface node | range |
|---|---:|---:|
| enclosing declaration | `DefinitionBlockItem#53` | `593..700` |
| direct template parameter | `TemplateParameter#31` | `606..620` |
| declaration binder | `Identifier` token `#2`、text `T` | `610..611` |
| generator type head | `TypeHead#39` → identifier token `#21`、text `T` | `678..679` |
| enclosing generator / term | `ComprehensionVariableSegment#41`、`SetComprehension#49`、`FunctorDefinition#52` | `673..679`、`663..694`、`623..695` |

declaration wrapper は明示的に **`DefinitionParameter` ではない**。leading template
`let T be type;` は `TemplateParameter#31` である。本 task は validated
declaration/use structural relation だけを transport し、bare type の sethood、
template actual/substitution、Chapter 18/13 の後段 rejection を判定しない。

## Freeze した resolver API と validation

`crates/mizar-resolve/src/names.rs` だけが次の public data/API 名を追加できる。

- `TemplateTypeParameterBindingId` (`new` / `index`)
- `TemplateTypeParameterBinding`
- `TemplateGeneratorTypeHeadLink`
- `TemplateTypeParameterBindingTable` / `TemplateGeneratorTypeHeadLinkTable`
- `TemplateTypeParameterSourceCollection`
- `TemplateTypeParameterSourceCollector`

`TemplateTypeParameterBinding` の field/getter は `definition_block`、`parameter`、
`binder`、`spelling`、`source_range`、`source_ordinal`。`TemplateGeneratorTypeHeadLink`
の field/getter は `definition_block`、`type_head`、`identifier`、`binding`、
`source_range`、`source_ordinal`。`TemplateTypeParameterBindingTable` は
`get(BindingId)`、`(id, row)` を返す `iter`、`len`、`is_empty`、
`TemplateGeneratorTypeHeadLinkTable` は `get(usize)`、row を返す `iter`、`len`、
`is_empty` を持つ。LinkId は追加しない。collection は `source_id`、`module`、
`bindings`、`generator_links`、`debug_text` を公開する。

exact collector signature は
`new(&SurfaceAst, &ModuleId, &SurfaceResolvedArena) -> Result<Self, SurfaceResolvedArenaError>`
と `collect(&self) -> Result<TemplateTypeParameterSourceCollection, SurfaceResolvedArenaError>`。
両 boundary で complete structural arena を validate する。custom public error enum /
lint-policy change はない。

collection は `DefinitionBlockItem` owner ごとに default-deny。unrecovered、unbounded、
single-binder direct `TemplateParameter` と同 owner の `TypeExpression`、
`ComprehensionVariableSegment`、`SetComprehension` 配下 generator-role `TypeHead` だけを
許す。この fixture は binding/link 各1件。exact `Identifier` token/text equality は明示的に
認可した **non-inferential resolver structural match**。same-owner duplicate spelling は
binding を残すが ambiguous link は作らない。recovery、bound、constraint、multiple binder、predicate/functor、wrapper、
cross-owner、non-generator role は ignore する。

resolver-owned node identity は `SurfaceResolvedArena::resolved_node_for` のみから得る。
ID construction、`SymbolId`、`NameRef` resolution、`ResolvedAst` field extension、alias
inference、diagnostic、type fact、sethood、verdict、checker state、public route は禁止する。

## Implementation、tests、inventory

後続 implementation の Rust scope は exact 4 paths:

1. `crates/mizar-resolve/src/names.rs`
2. `crates/mizar-resolve/src/names/tests.rs`
3. `crates/mizar-test/src/runner/tests.rs`
4. `crates/mizar-test/src/runner/tests/type_elaboration/template_parameter_identity.rs`

resolver test は exact 4、`148 -> 152`:

- `task277r1_collects_exact_template_generator_identity`
- `task277r1_isolates_scope_and_ignores_non_generator_roles`
- `task277r1_rejects_unsupported_parameter_and_recovery_shapes`
- `task277r1_revalidates_surface_resolved_arena_and_replays_deterministically`

`mizar-test` は exact
`task277r1_real_fixture_links_exact_template_generator_identity` を追加し `608 -> 609`。
existing helper を通じ immutable fixture を parse して frozen real profile だけを assert
する。production route/detail key/checker output/Typed-Resolved slot/active-stage selection/
semantic test にはしない。

resolver baseline は 23 Rust paths / 33,402 lines、path hash
`4d8c3c499b238814a839ae11994503bbb28f54a3690921c66429dccd298d47c8`、content hash
`894297b7f5e7a1ba387c1bcf1c34d528b60482e7f0ac8a623a9c452aaf26d633`。
`names.rs` は 2,749 lines /
`eff47c86f043c83daecef2631e0a53472bacd79a8288adb125bcc7139c762081`、
`names/tests.rs` は 2,197 /
`6770e085061c29cad9d571d09741b7384175189b5a9d0bfdf1de6c765cdc0a7f`。
`cargo test -q -p mizar-resolve --lib -- --list | sha256sum` は
`c99d9d179cf14ab9ccd274b11d0404bdc47a64d23a2aa914c69ba674d01a3fee`。

production mizar-test inventory は 38 paths / 80,090 lines、path/content hash
`0ef395004f7feaadf60da0daba7b5da9c52ea4974850adfa2bd9d09081b242aa` /
`990b5ad4798786d9f87c03f76fdbad92fc2edf1f6d84ef3baad67254c79fdd70` のまま。
production runner source は implementation diff の対象外。
`runner/tests.rs` は 61 lines /
`8eb35411834b0a6af48f935839f5c83d063fd7226565fd35478fe9e4a3f7c659`。
`cargo test -q -p mizar-test --lib -- --list | sha256sum` は
`0245b6b6d3f5f0687b5df3f8c7d1edc25cefe2e95ac04b2d7c4a89b141f99aa2`。

contract tree は `83/83 -> 84/84`。documentation prerequisite の exact 26 Markdown
paths は次の通り。

1. `doc/design/task_contracts/en/RESOLVE-TEMPLATE-TYPEPARAM-277R1.md`
2. `doc/design/task_contracts/ja/RESOLVE-TEMPLATE-TYPEPARAM-277R1.md`
3. `doc/design/mizar-resolve/en/00.crate_plan.md`
4. `doc/design/mizar-resolve/ja/00.crate_plan.md`
5. `doc/design/mizar-resolve/en/names.md`
6. `doc/design/mizar-resolve/ja/names.md`
7. `doc/design/mizar-resolve/en/source_spec_correspondence.md`
8. `doc/design/mizar-resolve/ja/source_spec_correspondence.md`
9. `doc/design/mizar-resolve/en/bilingual_documentation_synchronization.md`
10. `doc/design/mizar-resolve/ja/bilingual_documentation_synchronization.md`
11. `doc/design/mizar-resolve/en/todo.md`
12. `doc/design/mizar-resolve/ja/todo.md`
13. `doc/design/mizar-resolve/en/crate_exit_report.md`
14. `doc/design/mizar-resolve/ja/crate_exit_report.md`
15. `doc/design/mizar-resolve/en/module_boundary_refactor.md`
16. `doc/design/mizar-resolve/ja/module_boundary_refactor.md`
17. `doc/design/mizar-test/en/00.crate_plan.md`
18. `doc/design/mizar-test/ja/00.crate_plan.md`
19. `doc/design/mizar-test/en/harness.md`
20. `doc/design/mizar-test/ja/harness.md`
21. `doc/design/mizar-test/en/module_boundary_audit.md`
22. `doc/design/mizar-test/ja/module_boundary_audit.md`
23. `doc/design/mizar-test/en/bilingual_sync_audit.md`
24. `doc/design/mizar-test/ja/bilingual_sync_audit.md`
25. `doc/design/mizar-test/en/todo.md`
26. `doc/design/mizar-test/ja/todo.md`

future implementation completion documentation はこの list から plan-index 4 paths
(3、4、17、18) だけを除く exact 22 paths。Rust 4 paths と合わせて total scope は
exact 26 paths である。

## Protected scope と exit

`doc/spec`、全 `.miz` / expectation / sidecar、trace metadata
(`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`)、
`doc/design/spec_coverage_audit.md`
(`a31f6fb3bd2b561610630497c58284484d00716dd0b7f210f55bef3bc4bfa6db`)、Cargo、parser、
frontend、checker、Core、production runner、diagnostic surface、active stage、coverage
credit、formal/actual substitution、overload、sethood/verdict semantic は不変。

generic template resolution、exact token equality を超える spelling inference、alias/
shadowing、bound/constraint interpretation、semantic sethood、checker activation、public
diagnostic、fixture activation、parser/frontend/cache change、Task 277B readiness が必要に
なれば停止して parent authority に戻す。coverage-audit delta はない。

本 documentation prerequisite の exit は exact-scope review、`git diff --check`、recursive
task-contract/link lint の PASS。後続 fresh preflight は implementation 前に全 baseline を
remeasure し、frozen tests 5件、relevant crate/workspace checks、protected hash/count gates を
満たすまで separately authorized commit をしない。

## Implementation evidence と current status

implementation前のfresh preflightは全frozen baselineを再現した。exact Rust 4 pathsは
seven-name public resolver API、two-boundary `SurfaceResolvedArena` validation、resolver-only
node identity、owner-local exact token match、duplicate-binding ambiguity omission、global
generator-link source order、candidate `SetComprehension` subtree全体のrecovery fail-closeを
実装する。private mizar-test leaf 1件はproduction route/semantic verdictを追加せずimmutable
real fixtureだけをobserveする。

final measurement:

- resolver sourceは23 paths / 34,661 lines、path SHA-256
  `4d8c3c499b238814a839ae11994503bbb28f54a3690921c66429dccd298d47c8` unchanged、
  content-manifest SHA-256
  `d3f423448046180bb2db90f50d12518937fe00f5d0fb2ba188348db9bd08ab0e`。
- `names.rs`は3,248 lines /
  `de87c34a9afedd3649b410f4cf422b883a6fd567a1d61dc78221945320476548`、
  `names/tests.rs`は2,957 /
  `6d7c6c03fb15edd28af5428cf134bebb7d91686941429ea48d2e432837b55b40`。
- resolver libraryは152 tests、raw-list SHA-256
  `924e4652edfc9303d5d5742d3e3eb2b9a095ee6f0f543c8b7caa0a78f0c7b747`。
- `runner/tests.rs`は62 lines /
  `7c5cc9541b1cd2aabe050d3791e9153faeb302803cfa79abe39bfb58cb181d60`、
  new leafは67 /
  `5cafa3b0cd46ed29b8981f509b3fbec98f40be14e2ce8eee83bc7f10314bc1b8`。
- mizar-test libraryは609 tests、raw-list SHA-256
  `ea6e33af0de7353fa13517962c3b0e182cbcb3fc64bb06e5a61e3113daadb82c`。
  productionはfrozen path/content hashの38 paths / 80,090 linesで不変。

finding repair後のindependent test-sufficiency/implementation reviewは**NO FINDINGS**。
focused `4/4 + 1/1`、resolver package `152` library + `11` lint-policy + existing doctest、
mizar-test `609` library + `15` lint-policy + `137` metadata、full workspace `cargo test`、
`cargo fmt --all --check`、all-target/all-feature warnings-denied Clippy、offline Cargo metadata、
frozen stdout hashのfive CLI、protected hash/count replay、`git diff --check`は全PASS。
fixture/sidecar/trace/coverage audit/active stage/diagnostic/semantic coverage/checker/production
runnerは不変。independent source/documentation consistency、bilingual reviewは**NO
FINDINGS**。independent final-quality reviewも**NO FINDINGS**で、全9 hard gatesはscore
capなしのvalid `100/100`（`20/20/15/15/10/10/5/5`）でPASS。staging、implementation
commit、post-commit proof、fresh successor inventoryを未完のまま保持する。

plan、parse-only、declaration-symbol、type-elaboration、proof-verificationのfive CLI
stdout SHA-256は順に
`700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718`、
`a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`、
`71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`、
`4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`、
`ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`。

## 次 task handoff

exact Rust 4 + completion docs 22 pathsをexact stage/cached-diff reviewし、task-only
implementation commit、post-commit proof、fresh successor inventoryへ進む。parent
authority/integration/staging/final scoringはGPT-5.6 Sol `xhigh`。Lunaは未提供で、effective
bounded implementation/review routeはGPT-5.6 Terra `xhigh`。authority/boundary ambiguityは
すべてSolへescalateする。
