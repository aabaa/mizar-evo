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

本recordはdocs-only implementation prerequisiteで、Rust implementationは未開始。clean
`3d6add94f4b29d395a9362b56c05cc9256efa945`、`origin/main...HEAD=0/20`、
`stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`からcompleted C4A後のseparate
Task-257C sliceとしてselectした。exact docs review、task-only docs commit、fresh clean preflight後だけ実装を開始できる。

Authority順はcanonical Ch.13 §13.4 umbrella（§§13.4.2/13.4.4/13.8.6）、Ch.18 §18.10.2、immutable F5 `.miz`、trace、expectation、canonical EN Architecture 16、completed R2/277B-L/277C/C4A、その後derived owner/source observation。C4Aがexact source-to-lookup ordinalと`BindingId` identityを既にfreezeしたため、C4Bはその3 normalized useだけをexisting bindingへtransportする。sethood、term/reference、capture semanticsは決定しない。

Missing separate C4B recordの`design_drift`は本prerequisiteでclosed。future implementationはbounded `source_drift`/Rust `test_gap`をownする。`spec_gap`/new semantic intentなし。nested comprehension outer-use fixture不在のactual capture `test_gap`はseparate deferred。Task277Bは`MC-G020`/`MC-G021`未解消でnot-ready/zero credit。

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

Future Rust exact 3 pathは`crates/mizar-checker/src/source_formula_composition.rs`、
`crates/mizar-test/src/runner/tests.rs`、new private
`crates/mizar-test/src/runner/tests/type_elaboration/template_fraenkel_generator_bound_use.rs`。
checker exact testsは`task257c4b_builds_exact_fraenkel_generator_bound_uses`、
`task257c4b_rejects_environment_and_binding_context_dependency_corruption`、
`task257c4b_rejects_bound_use_and_lookup_corruption`、
`task257c4b_rebuilds_deterministically_without_mutation`で`550 -> 554`。sole private testは
`task257c4b_real_fixture_builds_exact_fraenkel_generator_bound_uses`で`613 -> 614`。

`SourcePrimaryTerm`/reference/Task252、R2 role copy、raw resolver ID、capture mutation、formula/type/sethood/request/verdict/diagnostic/fact/obligation、Typed/Resolved install、facade/dispatch/production route/active stage/trace/coverage creditを禁止する。違反は`boundary_violation`。Task277Bはzero credit。

Baselineは`source_formula_composition.rs` 7303 /
`f6da763061479e74e7b8f39169ecad311bb9bf879e91e93824d9899798017abc`、`runner/tests.rs` 66 /
`85ae891b185ed1eeb5940998c5eef5ece793b472b8f3fa4be3c0b96d217e1f07`、new leaf absent。checker production 32/193103、path/content
`9dc5b02f26679677e593ea755394d68533173d2be988b7ef1ddcfd84a41b9787` /
`cfc9a2bc5359f9baeea39f304e3c9dd15fcbd27749f1c746eb3ab695b84f8dab`。mizar-test production 38/80090、
`0ef395004f7feaadf60da0daba7b5da9c52ea4974850adfa2bd9d09081b242aa` /
`990b5ad4798786d9f87c03f76fdbad92fc2edf1f6d84ef3baad67254c79fdd70`。raw baselineはchecker 550 /
`ba24ea98b25617e41c832ef2dc0878f0502249d68d281fdb4e4c1a7e66c71885`、test 613 /
`a408a7099e886be8c6f4173325e40e4d9b3e28e42e8cc6cbad9bf88ce95e2741`。

本prerequisiteはexact 24 Markdown path（contract pair、4 plan index、checker 5 owner pair、mizar-test 4 owner pair）だけ。contract pairは`88/88 -> 89/89`。future completion docsはplan 4件を除くexact20、3 Rustと合計23 paths。spec/`.miz`/expectation/trace/Cargo/audit/ledger/Rustは本docs taskでno delta。F5/expectation/trace hashは`32c4...`/`b47a...`/`55b7...`、coverage auditは`a31f...`、schema-v2 ledgerは`2a66...`のまま。legacy heading/redirect/neighbor anchor/Task Index owner/ledger rowを変更しない。

## Gates and handoff

docs prerequisiteはexact24 scope、EN/JA parity、stable fragment/link、protected no-op、`git diff --check`、checker/mizar-test両lint policy、independent spec/API/test/bilingual/boundary/source-doc reviewをPASSする。future implementationはfocused `4 + 1`、library `554/554`/`614/614`、format、package/workspace Clippy、full tests、両lint、metadata、CLI、scope/hash、independent reviewをPASSする。canonical consistency、exact test matrix、trace zero-credit、ABI/full validation、design/source parity、boundary、audit/ledger no-op、verification、classified deferralの全`9/9` hard gatesとvalid `>=90/100`が必須。

Sol `xhigh`がauthority/API integration/final acceptance、Terra `xhigh`がbounded implementation/reviewsを担当する。ambiguity/API expansion/Task252/capture/semantic boundary/disputed findingはSolへ戻す。C4B完了後はfresh inventoryを行い、capture/sethood/Task277Bを自動選択しない。
