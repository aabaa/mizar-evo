# Crate Exit Report: mizar-lexer

> 正本は英語です。英語版: [../en/crate_exit_report.md](../en/crate_exit_report.md)。

## Result

Status: 現在の retrospective `mizar-lexer` milestone について complete。deferred
parser/resolver integration items は下記に記録する。

Quality score: reviewed 91/100。

Score caps applied: なし。残る `partial` traceability item は、未解決の
lexer-owned hard-gate failure ではなく、deferred かつ parser/resolver-owned として分類する。
未解決の `source_undocumented_behavior` または `test_expectation_drift` は、この crate
scope には残っていない。

## Scope

Milestone scope:

- 完了済み `mizar-lexer` crate の autonomous crate-development evidence を記録する;
- lexical fixture corpus を現在の human-reviewable lexer test surface として文書化する;
- design-derived traceability ids と partial selector-shadowing handoff を分類する;
- English/Japanese design documents を同期する。

Included:

- raw scanning、source preprocessing、reserved tables、import pre-scan、lexical
  environment construction、scope skeletons、final disambiguation、diagnostics
  payloads、Phase 7 regression/property/fuzz handoff evidence;
- [00.crate_plan.md](./00.crate_plan.md) と本 report;
- English/Japanese design documents の README index updates。

Excluded:

- parser/frontend-owned complete-source behavior のための新しい `.miz` tests 追加;
- `doc/spec/en`、既存 lexical fixtures、expectations、source implementation
  behavior の変更;
- lexer handoff を超える selector-vs-namespace semantics の解決;
- module resolution、type checking、overload resolution、proof checking、LSP/user-facing
  diagnostic rendering。

## Hard Gates

| Gate | Status | Evidence |
|---|---|---|
| Specification consistency | Pass | lexer-owned behavior は `doc/spec/en/02.lexical_structure.md`、`doc/spec/en/11.symbol_management.md`、`doc/spec/en/12.modules_and_namespaces.md`、`doc/spec/en/04.variables_and_constants.md`、`doc/spec/en/16.theorems_and_proofs.md` に trace される。この migration に canonical spec edit は含まない。 |
| Test contract | Pass with deferred `.miz` rationale | `tests/lexical`、`tests/property`、`tests/fuzz`、`crates/mizar-lexer` tests が lexer-owned pre-parser contracts を cover する。現在の lexer tests は full parser source file として意味を持つ前の sub-file lexical fixtures を必要とするため、`.miz` additions は defer。 |
| Traceability | Pass | `tests/coverage/spec_trace.toml` は zero errors で validate される。design-derived ids は [00.crate_plan.md](./00.crate_plan.md) で canonical specification に従属する executable implementation contracts として分類する。 |
| Design/source sync | Pass | 既存 module design docs は implemented source files を説明する。この migration で不足していた crate plan、exit report、README index entries を追加する。 |
| Boundary discipline | Pass | [README.md](./README.md) と [00.crate_plan.md](./00.crate_plan.md) は file I/O ownership、module resolution、authoritative parsing、name/type/overload/proof semantics、LSP/user-facing coordinate rendering を除外する。 |
| Verification | Pass | current branch verification results は下記に記録した。`mizar-test plan` は successfully exit し、`mizar-lexer` scope 外の既存 planned/no-tests warnings が 4 件ある。 |
| Residual risk | Pass | `MLX-GAP-001` と `MLX-GAP-005` は owner 付きで deferred。`MLX-GAP-004` と `MLX-GAP-006` は score cap を発生させる residual source behavior ではなく、解決済みの documentation/metadata drift。 |

## Score Breakdown

| Category | Points |
|---|---:|
| Specification completeness | 18/20 |
| Test contract and coverage | 18/20 |
| Traceability | 13/15 |
| Implementation correctness | 14/15 |
| Design/source synchronization | 9/10 |
| Boundary discipline | 10/10 |
| Verification health | 5/5 |
| Handoff quality | 4/5 |
| Total | 91/100 |

この score は、evidence が retrospective であること、lexer-owned `.miz` fixtures がないこと、
design-derived traceability ids、deferred selector-shadowing handoff に対して減点を残す。

## Deferred Items

| ID | Reason | Owner | Unblock condition |
|---|---|---|---|
| MLX-GAP-001 | lexer-owned tests は `.src` fixtures を使う。複数の lexer phase は complete parser source file として意味を持つ前に動作するため。 | Parser/frontend milestone | complete-source lexing/parser integration が同じ behavior を phase isolation を失わず assert できる時点で `.miz` tests を追加する。 |
| MLX-GAP-005 | `spec.en.02.lexical.dot_disambiguation.selector_shadowing_handoff` が `partial` のまま。 | Parser/resolver integration | authoritative selector-vs-namespace semantic resolution の complete-source tests と downstream implementation を追加する。 |

この migration で解決した項目:

- `MLX-GAP-002` と `MLX-GAP-003`: autonomous crate-plan/report evidence と
  README links を追加した。
- `MLX-GAP-004`: design-derived traceability ids は canonical language authority ではなく、
  subordinate implementation contracts として文書化した。
- `MLX-GAP-006`: source-loading helper ownership は、本番 source identity ownership ではなく、
  tests/early integration 用の lexer boundary contract として文書化した。

## Human Review Surface

人間レビューでは主に以下を確認します。

- [00.crate_plan.md](./00.crate_plan.md)
- 本 report
- [README.md](./README.md)
- [raw_lexer.md](./raw_lexer.md)
- [import_prescan.md](./import_prescan.md)
- [lexical_environment.md](./lexical_environment.md)
- [scope_skeleton.md](./scope_skeleton.md)
- [disambiguator.md](./disambiguator.md)
- [test_and_implementation_plan.md](./test_and_implementation_plan.md)
- [todo.md](./todo.md)
- [../../../spec/en/02.lexical_structure.md](../../../spec/en/02.lexical_structure.md)
- [../../../spec/en/04.variables_and_constants.md](../../../spec/en/04.variables_and_constants.md)
- [../../../spec/en/11.symbol_management.md](../../../spec/en/11.symbol_management.md)
- [../../../spec/en/12.modules_and_namespaces.md](../../../spec/en/12.modules_and_namespaces.md)
- [../../../spec/en/16.theorems_and_proofs.md](../../../spec/en/16.theorems_and_proofs.md)
- [../../../spec/ja/02.lexical_structure.md](../../../spec/ja/02.lexical_structure.md)
- [../../../spec/ja/04.variables_and_constants.md](../../../spec/ja/04.variables_and_constants.md)
- [../../../spec/ja/11.symbol_management.md](../../../spec/ja/11.symbol_management.md)
- [../../../spec/ja/12.modules_and_namespaces.md](../../../spec/ja/12.modules_and_namespaces.md)
- [../../../spec/ja/16.theorems_and_proofs.md](../../../spec/ja/16.theorems_and_proofs.md)
- `tests/coverage/spec_trace.toml`
- representative `tests/lexical/**/*.expect.toml` sidecars
- 同じ design files の日本語 companion

この migration の changed review surface には `.miz` file は含めません。既存 parser `.miz`
tests は parser-owned のままで変更しません。

## Test Expectation Summary

この migration では `.expect.toml` files、snapshots、lexical fixtures、source code、
benchmark code は変更しません。

| Test | Intent | Expected outcome | Expected phase | Diagnostics | Spec refs |
|---|---|---|---|---|---|
| `tests/lexical/pass/*.expect.toml` | positive lexer contract fixtures。 | Pass | Lexical | Fixture-specific | `tests/coverage/spec_trace.toml` lexical entries |
| `tests/lexical/fail/*.expect.toml` | negative and recoverable lexer contract fixtures。 | Fail | Lexical | Fixture-specific | `tests/coverage/spec_trace.toml` lexical entries |
| `tests/property/*.expect.toml` | Phase 4/5/7 invariants の metadata/property anchors。 | Metadata only | Lexical/property | None unless fixture-specific | `tests/coverage/spec_trace.toml` property entries |
| `tests/fuzz/lexer_phase7_fuzz_handoff_001.expect.toml` | minimized lexer regressions の fuzz handoff anchor。 | Metadata only | Lexical/fuzz | None | Phase 7 and raw span refs |

## Verification

この protocol handoff のために実行した commands:

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
- `cargo run -p mizar-test -- plan --tests-root tests --manifest tests/coverage/spec_trace.toml`: 72 test cases、57 requirements、0 errors、`mizar-lexer` scope 外の existing planned requirements warnings 4 件で passed:
  - `spec.en.algorithm.vc.assignment_loop_exits`
  - `spec.en.binding.substitution.capture_avoidance`
  - `spec.en.elaboration.choice_comprehension.lowering`
  - `spec.en.type_soundness.escape_and_guard_failures`

## Handoff

Next recommended work:

- `mizar-lexer` token streams を消費する parser/frontend integration work を開始し、
  complete-source `.miz` tests が一部 lexical `.src` fixtures を置き換える、または補完できるかを判断する。

Known constraints:

- spec-derived または explicitly approved test-intent change なしに、implementation
  behavior に合わせて lexical expectations を rebaseline しない。
- design-derived traceability ids は canonical `doc/spec/en` language requirements に
  従属させる。
- selector-vs-namespace semantic resolution は `mizar-lexer` の外に保つ。

Open questions:

- どの parser milestone で
  `spec.en.02.lexical.dot_disambiguation.selector_shadowing_handoff` を `partial` から
  covered complete-source behavior に移すべきか。

Recommended reasoning setting for the next task:

- `high`。次に有用な作業は lexer/parser/frontend boundaries をまたぎ、`.miz` tests
  が primary review surface になる地点を判断する可能性があるため。documentation-only
  traceability cleanup なら `medium` に下げてよく、canonical syntax または resolver
  semantics を変更するなら `high` より上げる。
