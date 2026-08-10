# Task RESOLVE-FRAENKEL-GENERATOR-VAR-277R2: Fraenkel generator-variable identity prerequisite

> canonical English: [../en/RESOLVE-FRAENKEL-GENERATOR-VAR-277R2.md](../en/RESOLVE-FRAENKEL-GENERATOR-VAR-277R2.md)。正本は英語です。

Owner plan は [mizar-resolve](../../mizar-resolve/ja/00.crate_plan.md#task-index)
と [mizar-test](../../mizar-test/ja/00.crate_plan.md#task-index)。stable owner section は resolver
の [names](../../mizar-resolve/ja/names.md#resolver-task-277r2-fraenkel-generator-variable-identity)、[source/spec](../../mizar-resolve/ja/source_spec_correspondence.md#resolver-task-277r2-sourcespecification-correspondence)、[boundary](../../mizar-resolve/ja/module_boundary_refactor.md#resolver-task-277r2-module-boundary)、[TODO](../../mizar-resolve/ja/todo.md#resolver-task-277r2-frozen-documentation-prerequisite)、[bilingual](../../mizar-resolve/ja/bilingual_documentation_synchronization.md#resolver-task-277r2-bilingual-synchronization)、[exit](../../mizar-resolve/ja/crate_exit_report.md#resolver-task-277r2-post-exit-prerequisite)、mizar-test の [harness](../../mizar-test/ja/harness.md#resolver-task-277r2-test-only-fixture-probe)、[boundary](../../mizar-test/ja/module_boundary_audit.md#resolver-task-277r2-test-module-boundary)、[TODO](../../mizar-test/ja/todo.md#resolver-task-277r2-test-only-fixture-probe)、[bilingual](../../mizar-test/ja/bilingual_sync_audit.md#resolver-task-277r2-contract-parity) である。

## Status、authority、classification

| 項目 | freeze 値 |
|---|---|
| Status | clean `HEAD=f2cb57e752b4dbed95761b9d302a1766b7f0f53a` でdocumentation prerequisiteをfreeze。implementation、implementation-time review、full verification、quality scoring、staging、commit、post-commit proofはfuture work。Task 277Bはnot ready、semantic credit zeroのまま。 |
| Authority | `doc/spec/en/13.term_expression.md` §§13.4.2、13.4.4、13.8.6、`doc/spec/en/18.templates.md` §18.10.2、immutable F5 source/expectation/trace row。 |
| Dependencies | complete [277R1](./RESOLVE-TEMPLATE-TYPEPARAM-277R1.md) / [277B-L](./277B-L.md) はread-only context。本taskはindependent resolver-owned generator-variable relationを作り、それらのIDをextend/consumeしない。 |
| Classification | `source_drift`、`design_drift`、Rust `test_gap`。`spec_gap`はない。後段sethood decision/missing-sethood verdictはchecker-owned。 |
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

## Freezeしたimplementation、tests、inventory

future Rust scopeはexact 5 paths:

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

| inventory | frozen baseline |
|---|---|
| resolver production | 23 paths / 34,661 lines; path `4d8c3c499b238814a839ae11994503bbb28f54a3690921c66429dccd298d47c8`; content `d3f423448046180bb2db90f50d12518937fe00f5d0fb2ba188348db9bd08ab0e` |
| `names.rs` | 3,248 / `de87c34a9afedd3649b410f4cf422b883a6fd567a1d61dc78221945320476548` |
| `names/tests.rs` | 2,957 / `6d7c6c03fb15edd28af5428cf134bebb7d91686941429ea48d2e432837b55b40` |
| resolver lint | 1,032 / `380b78b87590ae8471d8af80ec65cabf0cfa958d234cc6256571daa6c0568d9a` |
| resolver list | 152 / `924e4652edfc9303d5d5742d3e3eb2b9a095ee6f0f543c8b7caa0a78f0c7b747`; expected 156 |
| mizar-test list | 610 / `2d7e12fe5467d07fa4cef605c4d83cd8079ef8b5e0ea3e3431923b08a1532337`; expected 611 |
| `runner/tests.rs` | 63 / `8873ea62bf642a8287eeacbfdaea06eccd0d917a17be9cd54a7764b0b6bea295` |
| contract trees | `85/85 -> 86/86` |

protected production runnerは38 paths / 80,090 lines、path/content SHA-256
`0ef395004f7feaadf60da0daba7b5da9c52ea4974850adfa2bd9d09081b242aa` /
`990b5ad4798786d9f87c03f76fdbad92fc2edf1f6d84ef3baad67254c79fdd70`。

documentation prerequisiteはexact 26 Markdown paths。paired contract、EN/JA plan index 4件、
paired resolver names/source-spec/boundary/TODO/bilingual/exit、paired mizar-test harness/
boundary/TODO/bilingual。completion documentationはplan 4 pathsだけを除くexact 22 pathsで、
Rust 5 pathsと合わせimplementation changeはexact 27 paths。

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

documentation prerequisiteはindependent specification/contract、bilingual、boundary、source/docs
reviewでunresolved findingなし、exact 26-path review、`git diff --check`、recursive paired-contract/
local-link lintを要求する。このcommitはimplementation evidenceをclaimしない。

fresh preflight後のimplementationはfocused 4+1、resolver/mizar-test library/lint 156/611、package/
workspace warnings-denied Clippy、full `cargo test`、`cargo fmt --all --check`、diff、offline metadata/
existing metadata suite、five CLI output/hash unchanged、protected count/hash replayをPASSする。
test-sufficiency、implementation、source/docs、bilingual、boundary、final-quality reviewはunresolved
findingなしで完了し、staging/task-only commit前に全9 hard gate PASS、uncapped score 90/100以上。

## 次task handoff

fresh clean preflightが本contractを再現した後だけ
`RESOLVE-FRAENKEL-GENERATOR-VAR-277R2`をimplementする。authority/scope/disputed semantics/final
scoringはGPT-5.6 Sol `xhigh`、bounded five-path implementationとindependent precision reviewは
GPT-5.6 Terra `xhigh`。Lunaはruntimeで未提供。ambiguity/scope expansionはSolへescalateし、
Task 277B not-ready/semantic-credit-zeroを保持する。
