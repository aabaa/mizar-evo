# Crate Exit Report: mizar-lexer

> 正本は英語です。英語版: [../en/crate_exit_report.md](../en/crate_exit_report.md)。

## Result

Status: commit `b9f2482` (`docs: refine lexical symbol naming rules`) から開始した
`mizar-lexer` crate-wide autonomous-development milestone について complete。

Quality score: reviewed 94/100。

Score caps applied: なし。必須の read-only crate-exit review は全 Crate Exit Gates の pass を
確認した。残る gap は `MLX-GAP-001` と `MLX-GAP-005` のみで、どちらも明示的に deferred かつ
lexer-owned implementation scope 外である。`MLX-GAP-007` と `MLX-GAP-008` は解消済み。

## Scope

Milestone scope:

- `b9f2482` の lexical-symbol naming updates を `mizar-lexer` に適用する;
- `mizar-lexer` Ordered Task List の non-deferred item をすべて完了する;
- `MLX-GAP-007` と `MLX-GAP-008` を解消する;
- lexer handoff に必要な `mizar-frontend` と `mizar-parser` の paired work を行う;
- English/Japanese design documents を同期する。

Included:

- range-aware current-module lexical declaration support;
- notation symbols と readable constructor names の分離;
- source-position-aware parser-facing operator metadata;
- `TokenizeRequest.current_module`、parser-input local declaration forwarding、
  parser cache-key updates、token source position による Pratt lookup filtering;
- 上記 behavior の lexer、frontend、parser、cache tests;
- crate plan、TODO、lexer/frontend/parser handoff documents、本 exit report の bilingual
  design updates。

Excluded or deferred:

- pre-parser lexer phases のための `.miz` fixtures 追加 (`MLX-GAP-001`);
- authoritative selector-vs-namespace semantic resolution (`MLX-GAP-005`);
- module resolution、type checking、overload resolution、proof checking、LSP/user-facing
  diagnostic rendering;
- current implementation behavior に合わせる目的での `doc/spec/en`、既存 `.miz` tests、
  test expectations の変更。

## Hard Gates

| Gate | Status | Evidence |
|---|---|---|
| Specification consistency | Pass | `doc/spec/en` と `b9f2482` の bilingual spec/design updates を正として扱った。実装をブロックする spec gap や contradiction はなかった。実装済みタスクについて review-only agents は残存 specification/documentation findings なしと報告した。 |
| Source behavior documented and tested | Pass | local declaration ranges、constructor-name restrictions、selector exclusion、non-introducing `deffunc`/`defpred`/`algorithm`、imported/local overload preservation、source-position-aware operator metadata は lexer/frontend/parser design notes に記録され、Rust tests で cover されている。 |
| Milestone-owned tests | Pass with deferred `.miz` rationale | crate-local lexer tests、parser tests、frontend tests、cache tests、`mizar-test` verification が実装 contracts を cover する。`.miz` additions は complete parser source が意味を持つ前に動く lexer phases に限って defer されている。 |
| Test expectation discipline | Pass | 既存 `.miz` test または `.expect.toml` file を current implementation behavior に合わせるためには変更していない。 |
| Design/source sync | Pass | `doc/design/mizar-lexer`、`doc/design/mizar-frontend`、`doc/design/mizar-parser` は実装済み handoff を説明する。source/documentation consistency reviews は no findings で終了した。 |
| Boundary discipline | Pass | lexer changes は lexical and handoff-oriented の範囲に留まる。`private`/`public` visibility は lexing では解釈せず、selector semantics は downstream-owned のまま、parser/frontend changes は lexer metadata を消費するだけである。 |
| Verification | Pass | narrow crate tests、full workspace formatting、Clippy、full tests、traceability planning は passed。詳細は下記に記録する。 |
| Residual risk | Pass | 残リスクは deferred または downstream-owned として分類済み。未解決の `source_undocumented_behavior`、`test_expectation_drift`、`boundary_violation`、`repo_metadata_conflict` はこの milestone に残っていない。 |

## Score Breakdown

| Category | Points |
|---|---:|
| Specification completeness | 19/20 |
| Test contract and coverage | 18/20 |
| Traceability | 14/15 |
| Implementation correctness | 14/15 |
| Design/source synchronization | 10/10 |
| Boundary discipline | 10/10 |
| Verification health | 5/5 |
| Handoff quality | 4/5 |
| Total | 94/100 |

この reviewed score は、意図的な `.miz` fixture deferral、downstream selector-vs-namespace
handoff、lexer/parser/frontend boundary に残る review risk に対する減点を含む。read-only
crate-exit review が全 hard gates pass を確認したため、この score は有効である。

## Deferred Items

| ID | Reason | Owner | Unblock condition |
|---|---|---|---|
| MLX-GAP-001 | lexer-owned tests は、sub-file lexical behavior が reviewable contract である箇所では引き続き `.src` fixtures を使う。 | Parser/frontend milestone | phase isolation を失わず同じ behavior を complete-source lexing/parser integration で assert できる時点で `.miz` tests を追加する。 |
| MLX-GAP-005 | `spec.en.02.lexical.dot_disambiguation.selector_shadowing_handoff` は downstream semantic work のまま。 | Parser/resolver integration | authoritative selector-vs-namespace semantic resolution の complete-source tests と downstream implementation を追加する。 |

この milestone で解消した項目:

- `MLX-GAP-007`: current-module declarations と operator metadata は declaring item 完了後にのみ
  active になり、lexer/frontend/parser lookup は source-position-aware になった。
- `MLX-GAP-008`: lexer metadata は arbitrary notation symbols と constructor-name spellings を分離し、
  selector/generic constructor summary entries は lexer summary boundary で reject される。
- 以前に記録された `MLX-GAP-002`、`MLX-GAP-003`、`MLX-GAP-004`、`MLX-GAP-006` は、
  crate plan、README links、traceability classification、ownership-boundary documentation により
  resolved のままである。

## Human Review Surface

Primary documents:

- [00.crate_plan.md](./00.crate_plan.md)
- [todo.md](./todo.md)
- [lexical_environment.md](./lexical_environment.md)
- [../ja/00.crate_plan.md](../ja/00.crate_plan.md)
- [../ja/todo.md](../ja/todo.md)
- [../ja/lexical_environment.md](../ja/lexical_environment.md)
- [../../mizar-frontend/en/lexing.md](../../mizar-frontend/en/lexing.md)
- [../../mizar-frontend/en/parsing.md](../../mizar-frontend/en/parsing.md)
- [../../mizar-parser/en/expression_parser.md](../../mizar-parser/en/expression_parser.md)
- [../../../spec/en/02.lexical_structure.md](../../../spec/en/02.lexical_structure.md)
- [../../../spec/en/11.symbol_management.md](../../../spec/en/11.symbol_management.md)
- [../../../spec/en/12.modules_and_namespaces.md](../../../spec/en/12.modules_and_namespaces.md)

Representative source and tests:

- `crates/mizar-lexer/src/lexical_environment.rs`
- `crates/mizar-lexer/src/disambiguator.rs`
- `crates/mizar-lexer/src/tests/lexical_environment.rs`
- `crates/mizar-frontend/src/lexing.rs`
- `crates/mizar-frontend/src/parsing.rs`
- `crates/mizar-frontend/src/orchestration.rs`
- `crates/mizar-frontend/src/cache_key.rs`
- `crates/mizar-parser/src/lib.rs`

Implementation commits:

- `c038b1c` `feat: add range-aware lexer declarations`
- `d133ad0` `feat: split lexer notation metadata`
- `6279950` `feat: make operator metadata source-position aware`

## Test Expectation Summary

implementation commits では `.miz` source fixture、`.expect.toml` sidecar、canonical specification
file は変更していない。

追加または更新した Rust coverage:

- declaration-before-use、declaration-after-use、declaration self-non-activation、
  local/import overload preservation、constructor-name restrictions、selector exclusion、aliases、
  visibility no-op behavior、operator metadata recording、non-introducing local forms の lexer tests;
- current-module tokenization、parser-input forwarding、preprocessing coordinate mapping、
  orchestration、cache-key sensitivity の frontend tests;
- source-position-aware operator metadata と latest-active operator selection の parser tests。

## Verification

各 task commit の前に narrow verification を実行した:

```sh
cargo test -p mizar-lexer
cargo test -p mizar-parser
cargo test -p mizar-frontend
cargo test -p mizar-test
```

milestone の full verification:

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo run -p mizar-test -- plan --tests-root tests --manifest tests/coverage/spec_trace.toml
```

Results:

- `cargo fmt --check`: passed。
- `cargo clippy --all-targets --all-features -- -D warnings`: passed。
- `cargo test`: passed。
- `cargo run -p mizar-test -- plan --tests-root tests --manifest tests/coverage/spec_trace.toml`: 162 test cases、90 requirements、0 errors、`mizar-lexer` scope 外の existing planned requirements warnings 4 件で passed:
  - `spec.en.algorithm.vc.assignment_loop_exits`
  - `spec.en.binding.substitution.capture_avoidance`
  - `spec.en.elaboration.choice_comprehension.lowering`
  - `spec.en.type_soundness.escape_and_guard_failures`

## Handoff

Next recommended work:

- selector-vs-namespace semantic handoff の parser/resolver/frontend work を開始し、
  complete-source `.miz` tests が lexer `.src` fixture corpus をどこで補完すべきか判断する。

Known constraints:

- spec-derived または explicitly approved test-intent change なしに、implementation behavior に合わせて
  lexical expectations を rebaseline しない。
- design-derived traceability ids は canonical `doc/spec/en` requirements に従属させる。
- selector-vs-namespace semantic resolution は `mizar-lexer` の外に保つ。
- parser expression handling を変更するときは source-position-aware operator metadata contract を維持する。

Open questions:

- どの parser/resolver milestone で
  `spec.en.02.lexical.dot_disambiguation.selector_shadowing_handoff` を `partial` から
  covered complete-source behavior に移すべきか。
- どの lexer `.src` fixture families に、phase-isolated lexer review を弱めず `.miz` companions を
  追加できるか。

Recommended reasoning setting for the next task:

- `high`。次の有用な作業は lexer、parser、frontend、resolver の ownership boundaries をまたぎ、
  test-authority decisions を慎重に扱う必要があるため。documentation-only traceability cleanup なら
  `medium` に下げてよく、canonical syntax、resolver semantics、proof/type behavior を変更するなら
  `high` より上げる。
