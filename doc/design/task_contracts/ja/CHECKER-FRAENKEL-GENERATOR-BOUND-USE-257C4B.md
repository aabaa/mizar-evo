# Task CHECKER-FRAENKEL-GENERATOR-BOUND-USE-257C4B: Fraenkel Generator Bound-Use Transport

> canonical English: [../en/CHECKER-FRAENKEL-GENERATOR-BOUND-USE-257C4B.md](../en/CHECKER-FRAENKEL-GENERATOR-BOUND-USE-257C4B.md)。正本は英語です。

Owner planは[mizar-checker](../../mizar-checker/ja/00.crate_plan.md#task-index)と
[mizar-test](../../mizar-test/ja/00.crate_plan.md#task-index)。stable ownerはchecker
[formula composition](../../mizar-checker/ja/source_formula_composition.md#task-257c4b-fraenkel-generator-bound-use-transport)、
[source/spec](../../mizar-checker/ja/source_spec_audit.md#task-257c4b-fraenkel-generator-bound-use-classification)、
[boundary](../../mizar-checker/ja/module_boundary_audit.md#task-257c4b-fraenkel-generator-bound-use-boundary)、
[TODO](../../mizar-checker/ja/todo.md#task-257c4b-fraenkel-generator-bound-use-transport)、
[bilingual](../../mizar-checker/ja/bilingual_sync_audit.md#task-257c4b-frozen-contract-parity)、mizar-test
[harness](../../mizar-test/ja/harness.md#checker-task-257c4b-private-bound-use-probe)、
[boundary](../../mizar-test/ja/module_boundary_audit.md#checker-task-257c4b-frozen-module-boundary)、
[TODO](../../mizar-test/ja/todo.md#checker-task-257c4b-private-bound-use-probe)、
[bilingual](../../mizar-test/ja/bilingual_sync_audit.md#checker-task-257c4b-frozen-contract-parity)である。

## Status, authority, and readiness

Exact 3 Rust path implementationはclean preflight
`53987a3fdc1a927dbcbd2b9ed22e9817c8b68f2d`（`origin/main...HEAD=0/21`、
`stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`）をbaseにcompletion worktreeで完了した。
Retained structural/`TypedAst` corruptionとdependency-before-row precedenceをexisting test内でrepair後、implementation/test-sufficiency reviewは**NO FINDINGS**。sole Low baseline/current wordingをcontract pairでrepair後、final source-doc reviewとindependent bilingual/boundary reviewも**NO FINDINGS**。final-qualityも**NO FINDINGS**、全`9/9` hard gate PASS、valid uncapped `100/100`（`20/20/15/15/10/10/5/5`）。Exact staging/cached reviewは23 paths（3 Rust/20 docs）、new private leaf 1件、review時unstaged 0、cached diff check PASS、cached stat `1096/123`、両lint `15/15`でPASS。task-only commit、post-commit proof、fresh successorだけがpendingである。

Authority順はcanonical Ch.13 §13.4 umbrella（§§13.4.2/13.4.4/13.8.6）、Ch.18 §18.10.2、immutable F5 `.miz`、trace、expectation、canonical EN Architecture 16、completed R2/277B-L/277C/C4A、その後derived owner/source observation。C4Aがexact source-to-lookup ordinalと`BindingId` identityを既にfreezeしたため、C4Bはその3 normalized useだけをexisting bindingへtransportする。sethood、term/reference、capture semanticsは決定しない。

Missing separate C4B recordの`design_drift`は本prerequisiteでclosed。exact implementationはbounded `source_drift`/Rust `test_gap`をcloseする。`spec_gap`/new semantic intentなし。nested comprehension outer-use fixture不在のactual capture `test_gap`はseparate deferred。Task277Bは`MC-G020`/`MC-G021`未解消でnot-ready/zero credit。

## Frozen boundary, ABI, and F5 oracle

`source_formula_composition.rs`のC4Bはexact `&SourceFraenkelGeneratorBindingContextHandoff`だけをconsumeし、opaque C4A cloneのversion/domain/full validationをbuild/validationごとに行ってから各normalized ordinalを`BindingEnv::lookup`する。R2/277C/`TypedAst`/raw resolver ID/caller DTO/role enumをdirect inputにしない。

Exact public ABI/signature/getter type/debug/error precedence/test designはlanguage-local [formula owner](../../mizar-checker/ja/source_formula_composition.md#task-257c4b-fraenkel-generator-bound-use-transport)に同期する。public familyはone ID/immutable row/table/handoff/`#[non_exhaustive]` error/producerだけで、Typed/Resolved install/production consumerなし。human summaryだけをtrustせず、C4A full validationを通じ全R2/277C/TypedAst snapshot fieldをrevalidateする。

Exact three rowは次の通り。

| ID | use position | binding context | R2 use | source ordinal | lookup ordinal | context | binding |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 0 | 0 | 0 | 0 | 0 | 1 | 1 | 0 |
| 1 | 1 | 0 | 1 | 1 | 2 | 1 | 0 |
| 2 | 2 | 0 | 2 | 2 | 3 | 1 | 0 |

全row lookupは`Local(binding0)`。ordinal0はC4A pre-visibility `ForwardReference` probeでありC4B rowではない。default-denyでpartial/extra/reorder/duplicate/recovery/nested/multiple/shadow/stale/non-localをatomic rejectする。

## Scope, tests, and protected boundary

Completed Rust exact 3 pathは`crates/mizar-checker/src/source_formula_composition.rs`、
`crates/mizar-test/src/runner/tests.rs`、new private
`crates/mizar-test/src/runner/tests/type_elaboration/template_fraenkel_generator_bound_use.rs`。
checker exact testsは`task257c4b_builds_exact_fraenkel_generator_bound_uses`、
`task257c4b_rejects_environment_and_binding_context_dependency_corruption`、
`task257c4b_rejects_bound_use_and_lookup_corruption`、
`task257c4b_rebuilds_deterministically_without_mutation`で`550 -> 554`。sole private testは
`task257c4b_real_fixture_builds_exact_fraenkel_generator_bound_uses`で`613 -> 614`。

`SourcePrimaryTerm`/reference/Task252、R2 role copy、raw resolver ID、capture mutation、formula/type/sethood/request/verdict/diagnostic/fact/obligation、Typed/Resolved install、facade/dispatch/production route/active stage/trace/coverage creditを禁止する。違反は`boundary_violation`。Task277Bはzero credit。

Implementation baselineは`source_formula_composition.rs` 7303 /
`f6da763061479e74e7b8f39169ecad311bb9bf879e91e93824d9899798017abc`、`runner/tests.rs` 66 /
`85ae891b185ed1eeb5940998c5eef5ece793b472b8f3fa4be3c0b96d217e1f07`、new leaf absent。checker production 32/193103、path/content
`9dc5b02f26679677e593ea755394d68533173d2be988b7ef1ddcfd84a41b9787` /
`cfc9a2bc5359f9baeea39f304e3c9dd15fcbd27749f1c746eb3ab695b84f8dab`。対応するmizar-test production baselineは38/80090、
`0ef395004f7feaadf60da0daba7b5da9c52ea4974850adfa2bd9d09081b242aa` /
`990b5ad4798786d9f87c03f76fdbad92fc2edf1f6d84ef3baad67254c79fdd70`。raw baselineはchecker 550 /
`ba24ea98b25617e41c832ef2dc0878f0502249d68d281fdb4e4c1a7e66c71885`、test 613 /
`a408a7099e886be8c6f4173325e40e4d9b3e28e42e8cc6cbad9bf88ce95e2741`。

Completion docsはplan 4件を除くexact20で、3 Rustと合計23 paths。Final Rust measurementはchecker owner 7958 / `90b339d9707f9f8d847b678721e8db0ef6a00e4a15dbb41474a0cf6980f47168`、registry 67 / `94bc44e8ba47ca568670adeec74d20f6738b3fc337da2422871095137040e8c4`、private leaf 121 / `bea54489cf0c85d3026f950d080a0ffc609719fda28520b9e7b2f59d5fc162bc`。exact3 path hashは`b55deb1e11851b50d135785ff685dd8df5803cff3d89205903370d5421ac55fa`、deltaは777 insertions/0 deletions。checker productionは32/193758、path hash unchanged、content `90d8e277c6878b372090efbde122f3e95e5c50dce0475c9e50bbcabcb8eb1424`。raw listはchecker 554 / `78f0291fb13aed8a8adbbc5aa1db9df1a7415fc9d8cf35820e1ad9e40aad2ace`、mizar-test 614 / `419ac370d2ec222cc822186db62595b5ebed71e1059e10fa95dc00741acc9778`。mizar-test production 38/80090/hash unchanged。

spec/`.miz`/expectation/trace/Cargo/audit/ledgerはno delta。F5/expectation/trace hash、coverage audit、schema-v2 ledgerはfrozen値のまま。legacy heading/redirect/neighbor anchor/Task Index owner/ledger rowを変更しない。

## Gates and handoff

Spec/API prerequisiteとimplementation/test-sufficiency reviewは**NO FINDINGS**。focused `4 + 1`、library `554/554`/`614/614`、`cargo fmt --all -- --check`、package/full workspace all-target/all-feature Clippy `-D warnings`、full `cargo test`、Rust/completion-doc diff checkはPASS。Full testには両lint `15/15`、metadata `137/137`、public-enum `2/2`を含む。

Unchanged CLI stdout SHA-256はplan `700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718`、parse `a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`、declaration `71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`、type `4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`、proof `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`で、existing warnings/errors `23/0`を維持する。final-qualityは**NO FINDINGS**、全`9/9` hard gate PASS、valid uncapped `100/100`（`20/20/15/15/10/10/5/5`）、exact staging/cached reviewもPASS。task-only commit、post-commit proof、fresh inventoryだけがpending。

Sol `xhigh`がcommit acceptance/successor inventory、Terra `xhigh`がfinding-specific bounded re-reviewを担当する。ambiguity/API expansion/Task252/capture/semantic boundary/disputed findingはSolへ戻す。残るlifecycle完了後はfresh inventoryを行い、capture/sethood/Task277Bを自動選択しない。
