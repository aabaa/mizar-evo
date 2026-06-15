# Crate Exit Report: mizar-session

> 正本は英語です。英語版: [../en/crate_exit_report.md](../en/crate_exit_report.md)。

## Result

Status: 現在の `mizar-session` milestone について complete。

Quality score: reviewed 95/100。

Score caps applied: なし。`MS-GAP-001` として記録していた package-name spelling
`spec_gap` は、正本のパッケージ/モジュール仕様で解決済みであり、引き続き
`mizar-session` 実装境界の外にあります。

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

- `mizar-session` 内での package-name spelling 強制;
- syntax または semantics を所有する crate のための `.miz` language test 追加;
- scheduling、artifact publication、diagnostic aggregation、cache compatibility、
  IR storage、proof policy。

## Hard Gates

| Gate | Status | Evidence |
|---|---|---|
| Specification consistency | Pass | [00.crate_plan.md](./00.crate_plan.md) の `MS-GAP-001` は、正本のパッケージ/モジュール仕様と日本語 companion の同期により package-name spelling `spec_gap` が解決済みであることを記録する。 |
| Test contract | Pass | Rust unit/integration tests が crate-owned contract をカバーする。`.miz` test-first addition は、この crate が language behavior を所有しないため適用外。 |
| Traceability | Pass | [todo.md](./todo.md) tasks 1-32 が module specs、source files、tests を結ぶ。[00.crate_plan.md](./00.crate_plan.md) が task decomposition を要約する。 |
| Design/source sync | Pass | module design docs は public API/error surface を文書化し、README status は completed implementation と同期している。 |
| Boundary discipline | Pass | [README.md](./README.md) と [00.crate_plan.md](./00.crate_plan.md) は scheduling、IR storage、diagnostic aggregation、artifact publication、proof policy を除外する。 |
| Verification | Pass | current branch verification results は下記に記録した。`mizar-test -- plan` は exit successfully し、`mizar-session` scope 外の既存 planned/no-tests warnings が 4 件ある。 |
| Residual risk | Pass | package-name spelling enforcement は upstream build-plan の関心事に残る。`mizar-session` behavior はその検証をローカルで行うことに依存しない。 |

## Score Breakdown

| Category | Points |
|---|---:|
| Specification completeness | 20/20 |
| Test contract and coverage | 19/20 |
| Traceability | 14/15 |
| Implementation correctness | 14/15 |
| Design/source synchronization | 10/10 |
| Boundary discipline | 10/10 |
| Verification health | 4/5 |
| Handoff quality | 4/5 |
| Total | 95/100 |

reviewed score は、crate plan が後追いであることに対して小さな減点を残します。

## Deferred Items

なし。`MS-GAP-001` は解決済み。英語正本と日本語 companion の両方で、パッケージ名は
小文字の `snake_case` (`[a-z][a-z0-9]*(?:_[a-z0-9]+)*`) であり、ハイフン正規化は
定義しない。

## Human Review Surface

人間レビューでは主に以下を確認します。

- [00.crate_plan.md](./00.crate_plan.md)
- 本 report
- [README.md](./README.md)
- [todo.md](./todo.md)
- [source.md](./source.md)、特に source-loading error boundary
- [snapshot.md](./snapshot.md)、特に source identity validation boundary
- [../../../spec/ja/12.modules_and_namespaces.md](../../../spec/ja/12.modules_and_namespaces.md)
- [../../../spec/ja/23.package_management_and_build_system.md](../../../spec/ja/23.package_management_and_build_system.md)
- 同じファイルの英語正本

この migration は package naming specification text と downstream validation policy を
変更し、executable language test intent は変更しないため、`.miz` file は human review surface
に含めません。

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

- upstream build-plan validation を強め、`mizar-session` に渡される package id が
  同期済みの package-name spelling、すなわち小文字の `snake_case`
  (`[a-z][a-z0-9]*(?:_[a-z0-9]+)*`) でハイフン正規化なし、をすでに満たすようにする。

Known constraints:

- package-name spelling enforcement を `mizar-session` に移さず、upstream build-plan
  layer に保つ。
- `mizar-session` を lexer/parser semantics から疎結合に保つ。lexer-span から
  session-coordinate への橋渡しは frontend が所有する。

Open questions:

- `MS-GAP-001` についてはなし。canonical spelling は小文字の `snake_case`。

Recommended reasoning setting for the next task:

- `medium`。残る作業は、同期済み仕様テキストに対する bounded validator follow-up でよい。
  parser、resolver、package manifest semantics も変更する場合は `high` に上げる。
