# Crate Exit Report: mizar-parser

> 正本は英語です。英語版:
> [../en/crate_exit_report.md](../en/crate_exit_report.md)。

## 結果

状態: post-Task-46 `mizar-parser` milestone は complete。

品質スコア: independent reviewで99/100。

score cap: report preparation時点でなし。下記hard gateはすべてpassする。
P-265-47Dはnonblocking/human-owned。別途classifiedされたfrontend string-position
heuristicはparser target外であり、parser creditやclosureを主張しない。

`PARSER-CRATE-POST-TASK46-CLOSEOUT`はparser crate milestoneだけを閉じる。global
Step 5を閉じず、parser Task 49を推定・promoteせず、Steps 6/7をauthorizeしない。

## 範囲

milestone scope:

- parser Tasks 1-48。
- frontend-adapted token input、source-shaped `SurfaceAst` output、concrete
  grammar/Pratt parsing、syntax recovery、deterministic behavior、active
  parse-only corpus。
- source/spec correspondence、reserved-word coverage、bilingual
  synchronization、public-enum policy、parser module-boundary policy。
- current parser/syntax/corpus/traceability/count/hash evidence。

含むもの:

- canonical `reconsider_tail` 3形、top-level `PropertyImplementation`、exact
  infix/prefix/postfix `OperatorDeclaration`を含むcompleted parser grammar surface。
- annotation/visibility付きtop-levelとdefinition-local operator-declaration
  placement、append-only syntax kind 193、local recovery。
- parser unit/determinism/lint-policy、syntax、frontend、real parse-only runner evidence。
- paired English/Japanese parser design documentと本report。

含まないもの:

- Chapter 8とChapters 4/15 + Appendix Aの`reconsider` item list wordingに関する
  nonblocking human-owned P-265-47D `spec_gap`。
- independently classified frontend overbroad string-position heuristic。external
  frontend `source_drift` / `source_undocumented_behavior` + unit
  `test_expectation_drift`のまま。
- operator activation、active-functor validation、overload meaning、resolution、
  semantic precedence-range validation、source declarationによるPratt metadata mutation。
- checker/proof/Core/CFG/VC/artifact behavior、global Step-5 completion、unapproved
  future grammar growth、Task 49、Steps 6/7。

## Disagreement classification

- `design_drift`: staleなpre-Task-46 live-status textとobsoleteなreview-derived
  coverage backlogは本paired documentation synchronizationでclosed。
- `source_drift` / `test_gap`: aliased parser P-043-01/P-046 operator-declaration
  gapは本closeout前のTask 46でclosed。
- `spec_gap`: parser inventoryに残るのはP-265-47Dだけで、nonblocking/human-owned。
- `source_drift` / `source_undocumented_behavior` /
  `test_expectation_drift`: overbroad string-position heuristicはexternal frontend
  findingのままで、parser creditではない。
- parser-scope `test_gap`、`boundary_violation`、`repo_metadata_conflict`は残らない。
  metadata repairは実行しない。

## Hard Gates

| Protocol gate | 状態 | 証拠 |
|---|---|---|
| 1. blocking/high specification inconsistencyなし | Pass | English canonical spec、Appendix-A parser normalization、active corpus、exact trace row、parser designはTasks 1-48で一致する。P-265-47Dはnonblocking/human-owned。 |
| 2. undocumented parser source behaviorなし | Pass | [source_spec_audit.md](./source_spec_audit.md)がparser public surface/implementationをspec/testsへtraceする。parser-scope `source_undocumented_behavior`はなく、known frontend heuristicはexternal/uncredited。 |
| 3. milestone-owned coverageまたはexplicit deferral | Pass | parse coverageは44/44。implemented parser grammar sliceはactive coverageまたはdocumented owner boundaryを持ち、parser-owned deferred grammar taskはない。 |
| 4. expectation integrity | Pass | Task 46はnew pass/fail source/sidecarだけを追加し、existing `.miz`/expectationはunchanged。以前のTask-47 sidecar repairはcurrent implementationではなくcanonical grammarへ合わせた。 |
| 5. design/source synchronization | Pass | paired parser/syntax/mizar-test docs、crate/global index、source、wrapper ownership、recovery、current oracleはTask 46まで一致する。 |
| 6. responsibility boundary | Pass | parsingはsyntax-only。resolver/checker/proof/Core/CFG/VC/artifact/cache/build responsibilityを`mizar-parser`へ移していない。 |
| 7. coverage-audit synchronization | Pass | Task 46がcoverage changeを`doc/design/spec_coverage_audit.md`へ更新済み。本closeoutはcoverage/trace status/owner/deferred rationaleを変えないためauditはunchanged。 |
| 8. verification | Pass | focused/relevant crate test、format、denied-warning workspace Clippy、full workspace test、5 CLI、count/hash manifest、diff checkがpass。 |
| 9. residual-risk classification | Pass | P-265-47Dはhuman-owned/nonblocking、frontend heuristicはexternal/uncredited。parser `boundary_violation`や`repo_metadata_conflict`なし。 |

## Score Breakdown

independent read-only reviewerは全hard gateを確認した。1 pointの減点はnonblockingで
human-ownedのP-265-47D wording gapによる。

| Category | Points |
|---|---:|
| Specification completeness | 19/20 |
| Test contract and coverage | 20/20 |
| Traceability | 15/15 |
| Implementation correctness | 15/15 |
| Design/source synchronization | 10/10 |
| Boundary discipline | 10/10 |
| Verification health | 5/5 |
| Handoff quality | 5/5 |
| Total | 99/100 |

## 残存項目

| Item | 分類と理由 | Owner / action |
|---|---|---|
| P-265-47D | nonblocking human-owned `spec_gap`: Chapter 8は`reconsider_item` 1件、Chapters 4/15とAppendix Aはlistを指定する。 | human specification ownerがEnglish canonical wordingをexplicitにreconcileする。parser behaviorからspec editを推定しない。 |
| Frontend string-position heuristic | external frontend `source_drift` / `source_undocumented_behavior` + unit `test_expectation_drift`: canonicalなoperator-declaration/string-annotation positionより広いpunctuation後でquoted textを受理する。 | fresh frontend authorityでseparate taskを定義する。parser closeoutへ混在させずIDを発明しない。 |

current inventoryはnonempty successor `mizar-parser` implementation taskをauthorizeしない。

## Human Review Surface

- [00.crate_plan.md](./00.crate_plan.md)、[todo.md](./todo.md)、本report。
- [grammar.md](./grammar.md)、[pratt.md](./pratt.md)、
  [recovery.md](./recovery.md)、[source_spec_audit.md](./source_spec_audit.md)。
- paired English/Japanese parser/syntax/mizar-test Task-46 addenda。
- `doc/spec/en/10.functors.md`、`12.modules_and_namespaces.md`、Appendix A、
  exact Task-46 trace row/pass/fail sidecar。
- `crates/mizar-parser/src/`、paired `mizar-syntax` kind/accessor support、
  `crates/mizar-test/tests/metadata.rs`。

本docs-only closeoutはspecification/source/test/`.miz`/expectation/snapshot/traceability
row/coverage mapping/oracleを変更しない。

## Test Expectation Summary

| Evidence group | Intent | Expected phase/outcome |
|---|---|---|
| Parser unit/determinism/lint-policy + syntax test | AST ownership、exact slot、recovery boundary、append-only kind identity、deterministic output、public policyをguard。 | Rust test pass。closeout-time expectation changeなし。 |
| Task-47 `reconsider` corpus | canonical grammar下でomitted/explicit-`by`/proof-block tailをcover。 | active parse-only contract pass。 |
| Task-48 property-implementation pass/fail pair | exact top-level placement、means/equals body、correctness ordering、bounded recoveryをcover。 | active parse-only contract pass。 |
| Task-46 operator-declaration pass/fail pair | exact infix/prefix/postfix form、associativity word、annotation/visibility/local placement、malformed slot/delimiter、following-item preservationをcover。 | pass sidecarはdiagnostic 0、fail sidecarはexisting syntax code 6件に一致。 |

## 検証

commands:

```text
cargo test -p mizar-parser
cargo test -p mizar-syntax
cargo test -p mizar-frontend
cargo test -p mizar-test
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo run -p mizar-test -- plan --tests-root tests --manifest tests/coverage/spec_trace.toml
cargo run -p mizar-test -- parse-only --tests-root tests --manifest tests/coverage/spec_trace.toml
cargo run -p mizar-test -- declaration-symbol --tests-root tests --manifest tests/coverage/spec_trace.toml
cargo run -p mizar-test -- type-elaboration --tests-root tests --manifest tests/coverage/spec_trace.toml
cargo run -p mizar-test -- proof-verification --tests-root tests --manifest tests/coverage/spec_trace.toml
git diff --check
```

results:

- parser unit 225 / determinism 3 / lint-policy 14、syntax unit 55 /
  lint-policy 8。frontend/mizar-test relevant suiteもpass。
- plan 409 cases / 370 requirements、pass/fail 223/186、warnings/errors 23/0。
  parse/declaration/type/proof coverageは44/44、10/5、236/224、4/1。
- active parse/declaration/type/proof admission 101/5/188/1はすべてpass。
- plan/parse/declaration/type/proof stdout hashは
  `9b1e3058bde355163b1153339250647633beef9920456615cf6661c4140a93cf`、
  `a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`、
  `210055108c257ff65c6f45fb654c82e506653ec4617b68d111893bb3aa1da5a8`、
  `1dadbeabb219f5853c713ad53aa1cc7cd720a0e80abd7f882e9e0a5ea7802625`、
  `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`。
- raw/normalized 276-test-list hashは
  `967495e78e1068f592e64834ea3ffb9eac9c25692ea5cbd4f11006a679c66590`、
  `1be4ae09188b27a40814adc6597de4806dabb13bcac019b294154e1455072adf`。
- `mizar-test` productionは18 paths / 20,088 lines、path/content hashは
  `63e4e770b0d10872415548410d417071c1901f3ffa5aea964a81d2dbbc572ed0`、
  `7e5adca22db2b73f94f04c406f10788f2cd49ba48109bb105a3fd076c339d560`。
- parser productionは12 paths / 38,940 lines、path/content hashは
  `192f9d0b5e6534c4daab010ec51a9356e9e0fd6fb86876bd2600a75844e7566a`、
  `6f27be7c5689cc12b6cf684736bc44b1f92acebf6ce313ce581b22a46451cb5b`。
- format、denied-warning Clippy、full workspace test、diff checkはpass。

23 warningsはexisting cross-workspace soundness/corpus-size warningであり、parser
closeout errorではない。plan errorsは0。

## Handoff

current authority下の次の`mizar-parser` workはない。次turnはfresh canonical Step-5
inventoryから開始する。Task 49を推定せず、Steps 6/7をpromoteしない。external frontend
string-position findingは独自authorityを要し、暗黙にparser follow-upと扱わない。

推奨reasoning setting: 次のcross-crate authority inventoryまたはcanonical
grammar/semantic decisionは`xhigh`、bounded/authorized implementationは`high`、pure
documentationは`medium`。

## Post-closeout qualification: `PARSER-RECOVERY-B1B1P-P1`

上記99/100 scoreはhistorical post-Task-46 milestone resultであり、arbitrary
valid-UTF-8 recovery pathが現在すべてpanic-freeであるという証拠ではない。Checker
B1B1P fresh preflightは[00.crate_plan.md](./00.crate_plan.md)でfreezeしたexact
9-case fallback-ownership `source_drift`とRust `test_gap`を発見した。本bounded
prerequisiteだけがchecker taskへ戻る前のauthorized parser workである。

historical hard-gate resultは記録として保持するが、separate implementation commitが
independent reviewと全gateをfresh score 90/100以上でpassするまでcurrent parser
closeoutをqualifyする。parser Task 49を推定せず、specification coverageを変更せず、
broader grammar/recovery workをauthorizeしない。

### Qualification resolution

separate implementationはexact 9-case `source_drift`とRust `test_gap`をcloseした。
test/implementation reviewはno findingsであり、focused/crate test、unchanged
corpus/CLI oracle、test-list inventory、production manifestがpassする。excluded
5 inputは`ast = None`、unclaimed/previously returning controlは不変である。fresh
read-only hard-gate reviewは全hard gateをPASSとし、implementationを96/100と評価した。
Task 49やcoverage promotionへ進まない。

## Post-Closeout Qualification: `PARSER-TEMPLATE-TYPEHEAD-277P1`

historical parser closeout は frozen
[template type-head prerequisite](../../task_contracts/ja/PARSER-TEMPLATE-TYPEHEAD-277P1.md)
によって追加で qualify される。selected source は既に recovered AST を返すため、
bounded parser implementation は worktree で complete。focused regression 3件と
parser library 229件は pass し、frontend consumer は cache namespace v3 で exact
diagnostic-free 57-node AST を観測する。source/documentation-consistency と repaired
bilingual/boundary re-review は **NO FINDINGS**、coverage-audit no-op は confirmed。
上記 historical crate score は本 task に再利用しない。independent final-quality review
は **NO FINDINGS**、9 hard gate は全て PASS、cap はなく、task-specific score は
**99/100**。staging/cached review、implementation commit、post-commit proof、fresh
inventory は central contract で pending。Task 49、coverage promotion、semantic
acceptance、Checker Task 277B readiness は不変である。
