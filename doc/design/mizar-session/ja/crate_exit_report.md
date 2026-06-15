# Crate Exit Report: mizar-session

> 正本は英語です。英語版: [../en/crate_exit_report.md](../en/crate_exit_report.md)。

## Result

Status: 現在の `mizar-session` milestone について complete。

Quality score: reviewed 93/100。

Score caps applied: なし。package-name spelling issue は `mizar-session` 実装境界の
外にある deferred human-owned `spec_gap` なので、crate score の cap にはしません。

## Scope

Milestone scope:

- `mizar-session` が所有する source identity、build snapshot、source-version、
  source-map、retention contract を完成させる;
- English/Japanese component design docs を実装済み Rust API とテストに同期する;
- 完了済み crate に autonomous crate-development evidence を追加する。

Included:

- `ids`、`source`、`snapshot`、`source_map`、`retention` module contract;
- deterministic snapshot/source-map behavior;
- [todo.md](./todo.md) に記録された source-loading error fidelity follow-up;
- bilingual design synchronization;
- [00.crate_plan.md](./00.crate_plan.md) と本 report の protocol evidence。

Excluded:

- `doc/spec` 間の package-name spelling 解決;
- syntax または semantics を所有する crate のための `.miz` language test 追加;
- scheduling、artifact publication、diagnostic aggregation、cache compatibility、
  IR storage、proof policy。

## Hard Gates

| Gate | Status | Evidence |
|---|---|---|
| Specification consistency | deferred low-risk item 付き Pass | [00.crate_plan.md](./00.crate_plan.md) の `MS-GAP-001` が package-name spelling `spec_gap` を human-owned かつ crate boundary 外として記録する。 |
| Test contract | Pass | Rust unit/integration tests が crate-owned contract をカバーする。`.miz` test-first addition は、この crate が language behavior を所有しないため適用外。 |
| Traceability | Pass | [todo.md](./todo.md) tasks 1-32 が module specs、source files、tests を結ぶ。[00.crate_plan.md](./00.crate_plan.md) が task decomposition を要約する。 |
| Design/source sync | Pass | module design docs は public API/error surface を文書化し、README status は completed implementation と同期している。 |
| Boundary discipline | Pass | [README.md](./README.md) と [00.crate_plan.md](./00.crate_plan.md) は scheduling、IR storage、diagnostic aggregation、artifact publication、proof policy を除外する。 |
| Verification | Pass | current branch verification results は下記に記録した。`mizar-test -- plan` は exit successfully し、`mizar-session` scope 外の既存 planned/no-tests warnings が 4 件ある。 |
| Residual risk | deferred item 付き Pass | package-name spelling `spec_gap` は language/package specification work へ deferred。現時点の `mizar-session` behavior は spelling の選択に依存しない。 |

## Score Breakdown

| Category | Points |
|---|---:|
| Specification completeness | 18/20 |
| Test contract and coverage | 19/20 |
| Traceability | 14/15 |
| Implementation correctness | 14/15 |
| Design/source synchronization | 10/10 |
| Boundary discipline | 10/10 |
| Verification health | 4/5 |
| Handoff quality | 4/5 |
| Total | 93/100 |

reviewed score は、deferred package-name spelling `spec_gap` と crate plan が
後追いであることに対して小さな減点を残します。

## Deferred Items

| ID | Reason | Owner | Unblock condition |
|---|---|---|---|
| MS-GAP-001 | `doc/spec/ja/23.package_management_and_build_system.md` は `[a-z][a-z0-9-]*`、`doc/spec/ja/12.modules_and_namespaces.md` は `snake_case` package name を使う。 | human language/package spec owner | English canonical package-name rules を揃え、日本語 companion を同期し、必要なら upstream build-plan validation を強める。 |

## Human Review Surface

人間レビューでは主に以下を確認します。

- [00.crate_plan.md](./00.crate_plan.md)
- 本 report
- [README.md](./README.md)
- [todo.md](./todo.md)
- [source.md](./source.md)、特に source-loading error boundary
- [snapshot.md](./snapshot.md)、特に source identity validation boundary
- 同じファイルの英語正本

この migration は language behavior や test intent を変更しないため、`doc/spec` と
`.miz` file は human review surface に含めません。

## Test Expectation Summary

この migration では `.expect.toml` または snapshot expectation を変更していません。

| Test | Intent | Expected outcome | Expected phase | Diagnostics | Spec refs |
|---|---|---|---|---|---|
| N/A | `mizar-session` は `.miz` execution behavior ではなく Rust infrastructure contract を所有する。 | N/A | N/A | N/A | N/A |

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
- `cargo run -p mizar-test -- plan --tests-root tests --manifest tests/coverage/spec_trace.toml`: 72 test cases、57 requirements、0 errors、既存 planned requirements without tests の warnings 4 件で passed:
  - `spec.en.algorithm.vc.assignment_loop_exits`
  - `spec.en.binding.substitution.capture_avoidance`
  - `spec.en.elaboration.choice_comprehension.lowering`
  - `spec.en.type_soundness.escape_and_guard_failures`

## Handoff

Next recommended work:

- `MS-GAP-001` を解決する。English canonical specs と Japanese companions の
  package-name spelling を揃え、その後 upstream build-plan validation が hyphenated
  names、snake_case names、または normalization 付きの両方を受け入れるべきか決める。

Known constraints:

- spec が揃う前に `mizar-session` package-id validation で spelling gap を決めない。
- `mizar-session` を lexer/parser semantics から疎結合に保つ。lexer-span から
  session-coordinate への橋渡しは frontend が所有する。

Open questions:

- registry package と import path の canonical package-name spelling はどれか。

Recommended reasoning setting for the next task:

- `high`。`MS-GAP-001` の解決は canonical language/package specification、日本語同期、
  downstream validation policy に触れるため。次タスクが例の収集のみで spec edit を
  行わないなら `medium` に下げてよい。
