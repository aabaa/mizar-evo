# Task RESOLVE-FRAENKEL-GENERATOR-VAR-277R2: Fraenkel generator-variable identity prerequisite

> canonical English: [../en/RESOLVE-FRAENKEL-GENERATOR-VAR-277R2.md](../en/RESOLVE-FRAENKEL-GENERATOR-VAR-277R2.md)。正本は英語です。

Owner plan は [mizar-resolve](../../mizar-resolve/ja/00.crate_plan.md#task-index)
と [mizar-test](../../mizar-test/ja/00.crate_plan.md#task-index)。stable owner section は resolver
の [names](../../mizar-resolve/ja/names.md#resolver-task-277r2-fraenkel-generator-variable-identity)、[source/spec](../../mizar-resolve/ja/source_spec_correspondence.md#resolver-task-277r2-sourcespecification-correspondence)、[boundary](../../mizar-resolve/ja/module_boundary_refactor.md#resolver-task-277r2-module-boundary)、[TODO](../../mizar-resolve/ja/todo.md#resolver-task-277r2-frozen-documentation-prerequisite)、[bilingual](../../mizar-resolve/ja/bilingual_documentation_synchronization.md#resolver-task-277r2-bilingual-synchronization)、[exit](../../mizar-resolve/ja/crate_exit_report.md#resolver-task-277r2-post-exit-prerequisite)、mizar-test の [harness](../../mizar-test/ja/harness.md#resolver-task-277r2-test-only-fixture-probe)、[boundary](../../mizar-test/ja/module_boundary_audit.md#resolver-task-277r2-test-module-boundary)、[TODO](../../mizar-test/ja/todo.md#resolver-task-277r2-test-only-fixture-probe)、[bilingual](../../mizar-test/ja/bilingual_sync_audit.md#resolver-task-277r2-contract-parity) である。

## Status、authority、classification

| 項目 | freeze 値 |
|---|---|
| Status | exact Rust 5-path implementationはcomplete。exact `4 + 1` regression、resolver/mizar-test library・lint suite、package/workspace Clippy、workspace test、offline metadataとそのsuite、format、diff、five frozen CLI hash、protected path-hash checkはcomplete。independent test-sufficiency、implementation、source/documentation/API、bilingual、boundary、final-quality reviewは**NO FINDINGS**。全9 hard gateはscore capなしvalid `100/100`（`20/20/15/15/10/10/5/5`）でPASS。exact staging/cached-diff review、task-only implementation commit、post-commit proof、fresh inventoryは下のhistorical pre-closure checkpointでcomplete。successorはselectせず、Task 277Bはnot ready、semantic credit zeroのまま。 |
| Authority | `doc/spec/en/13.term_expression.md` §§13.4.2、13.4.4、13.8.6、`doc/spec/en/18.templates.md` §18.10.2、immutable F5 source/expectation/trace row。 |
| Dependencies | complete [277R1](./RESOLVE-TEMPLATE-TYPEPARAM-277R1.md) / [277B-L](./277B-L.md) はread-only context。本taskはindependent resolver-owned generator-variable relationを作り、それらのIDをextend/consumeしない。 |
| Classification | classified `source_drift`とRust `test_gap`はbounded implementationとexact regressionでresolve。completion documentationが対応する`design_drift`をreconcileし、`spec_gap`はない。後段sethood decision/missing-sethood verdictはchecker-owned。 |
| Consumer | separately frozenなlower transport/checker taskだけがconsumeできる。本taskはconsumerをselectせずTask 277Bをreadyにしない。 |

immutable seed は
`tests/miz/fail/templates/fail_template_fraenkel_over_type_param_001.miz`、701 bytes、final
LF、SHA-256 `32c4a1c1b6c9d98dcb085558a084929e07d4005bf92595865f144456e95854ec`。
839-byte expectation は
`b47ac5113c89cd5703adb0ffd660b52d3e16908c92623dd2f63196aa6a215cb2`、
`advanced_semantics` inactive。mapped trace seedはinactive、checker-plan gap `MC-G020` /
`MC-G021`はdeferredのままで、source、expectation、trace、stage、coverage creditは変更しない。

## Freezeしたreal-source profile

parser profileはnormal 57 nodes、root `56`、diagnosticなし。

| role | Surface node | range / ordinal |
|---|---:|---:|
| enclosing declaration | `DefinitionBlockItem#53` | `593..700` |
| enclosing functor | `FunctorDefinition#52` | `623..695` |
| comprehension | `SetComprehension#49` | `663..694` |
| generator segment / binder | `ComprehensionVariableSegment#41` / `Identifier#19` text `x` | `673..679` / `673..674`; binding `0` |
| mapper owner / reference / identifier | `TermExpression#38` / `TermReference#37` / `Identifier#17` | `665..666`; global `0`、role `0` |
| condition owner / first reference / identifier | `FormulaExpression#48` / `TermReference#42` / `Identifier#24` | `686..687`; global `1`、role `0` |
| condition second reference / identifier | `TermReference#44` / `Identifier#26` | `691..692`; global `2`、role `1` |

binderはsource上で宣言より前のmapper useと後段condition useをscopeする。identityはこのexact
comprehension内のsole bounded same-spelling structural matchで、general lexical/template/
alias/shadow resolverではない。

## Freezeしたresolver APIとvalidation

`crates/mizar-resolve/src/names.rs` だけが次を追加できる。

- `FraenkelGeneratorVariableBindingId` (`new(index: usize) -> Self` / `index(self) -> usize`)
- immutable `FraenkelGeneratorVariableBinding`
- non-exhaustive `FraenkelGeneratorVariableUseRole::{Mapper, Condition}`
- immutable `FraenkelGeneratorVariableUseLink`
- `FraenkelGeneratorVariableBindingTable` / `FraenkelGeneratorVariableUseLinkTable`
- `FraenkelGeneratorVariableSourceCollection`
- `FraenkelGeneratorVariableSourceCollector`

binding getterは `definition_block()`、`functor_definition()`、`comprehension()`、`segment()`、
`binder()` が各 `ResolvedNodeId`、`spelling() -> &str`、`segment_range()` / `binder_range()` が
各 `SourceRange`、`source_ordinal() -> usize`。use linkは `definition_block()`、
`functor_definition()`、`comprehension()`、`role_owner()`、`term_reference()`、`identifier()` が各
`ResolvedNodeId`、`binding() -> FraenkelGeneratorVariableBindingId`、`role() ->
FraenkelGeneratorVariableUseRole`、global `source_ordinal() -> usize`、role-local
`role_source_ordinal() -> usize`、`identifier_range() -> SourceRange`。

binding tableは`get(id)`、`(id,row)` iterator、`len`、`is_empty`、use-link tableはseparate IDなしで
`get(index)`、row iterator、`len`、`is_empty`。両tableはdense/deterministic。collectionは
`source_id() -> SourceId`、`module() -> &ModuleId`、`bindings()`、`uses()`、`debug_text()`。
F5 summaryはexactに
`fraenkel-generator-variable-source-v1|module=<module>|bindings=1|uses=3`。

exact signatureは
`FraenkelGeneratorVariableSourceCollector::new(&SurfaceAst, &ModuleId,
&SurfaceResolvedArena) -> Result<Self, SurfaceResolvedArenaError>` と
`collect(&self) -> Result<FraenkelGeneratorVariableSourceCollection,
SurfaceResolvedArenaError>`。両boundaryでcomplete arenaをvalidateし、identityは
`SurfaceResolvedArena::resolved_node_for`だけから得る。custom error/diagnosticは追加しない。

collectionはdefault-deny。exact normal single-generator/single-binder Fraenkel shapeだけを認め、
sole bounded exact spellingでmapper/condition identifier-term referenceをbinderへassignする。
binding/useをsource range+node identityでsortし、dense global ordinalとrole-local ordinalを付与する。
candidate内recovery、nonexact edge/wrapper、multi generator/binder、nested comprehension/binder、
shadow、ambiguous same spelling、unsupported shapeはbinding/useともzero rowsで、partial rowを残さない。

resolver ID construct、`SymbolId` allocation、`NameRef` resolve、`ResolvedAst` change、type、
`SourceSetTerm`、`SourceFormula`、別taskの`BindingId`、sethood/evidence/diagnostic/verdict publishは禁止。
template parameter/R1/277B-L IDはcarryしない。

## Implementation、tests、measured inventory

implementation Rust scopeはexact 5 paths:

1. `crates/mizar-resolve/src/names.rs`
2. `crates/mizar-resolve/src/names/tests.rs`
3. `crates/mizar-resolve/tests/lint_policy.rs`
4. `crates/mizar-test/src/runner/tests.rs`
5. `crates/mizar-test/src/runner/tests/type_elaboration/fraenkel_generator_variable_identity.rs`

resolverはexact 4 testsを追加し`152 -> 156`:

- `task277r2_collects_exact_f5_generator_binding_and_uses`
- `task277r2_scopes_mapper_before_binder_and_orders_condition_uses`
- `task277r2_ignores_unsupported_and_recovered_fraenkel_shapes`
- `task277r2_revalidates_surface_resolved_arena_and_replays_deterministically`

mizar-testはexact
`task277r2_real_fixture_links_exact_fraenkel_generator_binding_and_uses`を追加し`610 -> 611`。
private leafはexisting helperでimmutable fixtureをparseしてresolver collectorをdirect callする。
production route/dispatcher/detail key/checker output/typed-resolved slot/active-stage/semantic testではない。

| inventory | final measured value |
|---|---|
| resolver production | 23 paths / 35,977 lines; path `4d8c3c499b238814a839ae11994503bbb28f54a3690921c66429dccd298d47c8`; content `8bf8f814a852adce43de16711d7ca7529263b9de79ee002438162c2234b6e4d8` |
| `names.rs` | 3,920 lines / `9a4b1a0e289c058a40c5af91d00fb836eca7af3a1d83bfcfa9b60227ce46d14a` |
| `names/tests.rs` | 3,601 lines / `31228c3502a08276a0c395715f74a6a5143a11c315145595ac88f93163e6863a` |
| resolver lint | 1,037 lines / `1a84ba67b715b8df752accd18895fc89a8e727769061a89570b2b4fe15d1182d` |
| resolver list | 156 / `7c84ee615616d7f0982454c8d04e9eef2fcb451efbb8fd576296e28af3cb6301` |
| mizar-test list | 611 / `6eaaca04215420028c57731bc14144e2b73ca719dc6cc35f64a5a421e2a1c426` |
| `runner/tests.rs` | 64 lines / `8ae81a6ca4dadd9a58165f09bdde4d2ad3cdcd0884ad7521fe5d1ea90539b316` |
| private fixture leaf | 106 lines / `69b54a4effcb7a740d6588070e6951e3a772cd1818ef9fedcb36426642bf3bf4` |

protected production runnerは38 paths / 80,090 lines、path/content SHA-256
`0ef395004f7feaadf60da0daba7b5da9c52ea4974850adfa2bd9d09081b242aa` /
`990b5ad4798786d9f87c03f76fdbad92fc2edf1f6d84ef3baad67254c79fdd70`。

completion documentationはexact 22 Markdown paths。paired contract、paired resolver
names/source-spec/boundary/TODO/bilingual/exit、paired mizar-test harness/boundary/TODO/
bilingualである。freezeしたplan index 4件は不変。Rust 5 pathsと合わせimplementation changeは
exact 27 paths。

## Protected scopeとaudit decision

`doc/spec`、全`.miz`/expectation、trace
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`、
`doc/design/spec_coverage_audit.md`
`a31f6fb3bd2b561610630497c58284484d00716dd0b7f210f55bef3bc4bfa6db`は不変。
F5は`MC-G020`/`MC-G021` inactiveのままでcoverage/follow-up owner changeがないためaudit
deltaはない。Cargo、parser、frontend、checker、Core、production runner、diagnostic、active
stage、semantic credit、legacy section/redirect/ledger、complete R1/277B-L API/evidenceもprotected。

new language/test-intent、generic name resolution、nested/shadow support、partial recovery、
type/sethood/evidence/verdict、diagnostic、checker/fixture activation、production route、protected
artifact change、Task 277B readyが必要なら停止してSol parentへauthorityを返す。

## Review、verification、exit

exact resolver 4 testsとreal-fixture 1 testはPASS。resolver/mizar-test library・lint suiteは
156/611に到達し、package Clippy、format、implementation-time diff checkはPASS。independent
test-sufficiency / implementation reviewは**NO FINDINGS**。

workspace-wide `cargo test` とwarnings-denied Clippy、offline Cargo metadataとそのsuite、unchanged
five CLI output/hash、protected count/hash replayもPASS。freezeしたF5 source/expectation、trace、
coverage audit、production runner、resolver manifest/test-list hash、full protected path setはexactに
reproduceする。independent source/documentation/API、bilingual、boundary、final-quality reviewは
**NO FINDINGS**、全9 hard gateはscore capなしvalid `100/100`
（`20/20/15/15/10/10/5/5`）でPASS。exact staging/cached-diff review、task-only implementation
commit、post-commit proof、fresh successor inventoryは下のhistorical pre-closure checkpointでcomplete。

## Historical Immediate-Post-Implementation Checkpoint

これはtask-only implementation commit
`534a8797dc4066f0b07f47dbf440e35369ab80c5`のhistorical pre-closure recordであり、current
`HEAD`をrepresentまたはclaimしない。このcommit直後のread-only inventoryでは
`HEAD=534a8797dc4066f0b07f47dbf440e35369ab80c5`、clean worktree、
`origin/main...HEAD=0/13`、unchanged
`stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`をobserveした。

exact 27-path implementation commitのpath hashは
`43610037f566f01659392ec43c721af6b85ccb28fdbf43a5973b155e6937d62e`。
exact staging/cached-diff review、task-only implementation commit、post-commit proof、fresh
successor inventoryはこのhistorical checkpointでcomplete。successorはselectせず、Task 277Bは
not ready、semantic credit zeroのまま。

## 次task handoff

`RESOLVE-FRAENKEL-GENERATOR-VAR-277R2` はsuccessorをselectしない。Task 277B not-ready/
semantic-credit-zeroを保持する。後続lower transport/checker workにはseparate frozen authority、
scope、review recordが必要である。
