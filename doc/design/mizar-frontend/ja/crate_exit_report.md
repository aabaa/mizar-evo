# Crate Exit Report: mizar-frontend

> 正本は英語です。英語版: [../en/crate_exit_report.md](../en/crate_exit_report.md)。

## 結果

状態: 現在の retrospective `mizar-frontend` milestone について complete。ただし、
parser-growth と producer-backed fallback に関する deferred item を下に記録する。

品質スコア: reviewed 93/100。

適用された score cap: なし。`mizar-frontend` scope には、未解決の hard gate
failure、`source_undocumented_behavior`、`test_expectation_drift` は把握していない。
deferred item は parser-owned の将来 grammar growth、または upstream producer の
availability limit である。

## 範囲

Milestone scope:

- 完了済み `mizar-frontend` crate の autonomous crate-development evidence を記録する。
- frontend-owned orchestration behavior、parser seam coverage、fallback diagnostic
  surface、将来の parser-growth obligation を分類する。
- 英語・日本語 frontend design index を crate plan と exit report に同期する。
- implementation、canonical specification、`.miz` tests、expectation files は変更しない。

Included:

- source loading projection、span bridge、preprocessing、import stub、active lexical
  environment provider boundary、parser-assisted lexing、parser seam、orchestration、
  diagnostics、cache keys、deterministic output、lint/public API policy、frontend fuzz
  target、Criterion baseline evidence。
- [00.crate_plan.md](./00.crate_plan.md) と本 report。
- 英語・日本語 design document の README index 更新。

Excluded:

- `doc/spec/en`、`doc/spec/ja`、既存 `.miz` tests、expectation files、source
  implementation、fuzz implementation、benchmark code の変更。
- parser-owned grammar expansion、template/fixity `.miz` seed activation、
  module resolution、semantic name/type/overload/proof behavior、cache storage、
  artifact publication、LSP display rendering。
- 現在の upstream crates がまだ生成できない future non-exhaustive variant に対する
  producer-backed tests の追加。

## Hard Gates

| Gate | 状態 | 根拠 |
|---|---|---|
| Specification consistency | Pass | [00.crate_plan.md](./00.crate_plan.md) が frontend behavior を architecture、component design、language-spec refs へ対応付ける。この migration では canonical spec edit は含まない。 |
| Test contract | Pass with deferred `.miz` rationale | `crates/mizar-frontend` Rust tests が orchestration contract を網羅する。active parser `.miz` seeds は現在 frontend-reachable な parser seam behavior を網羅する。complete-source ではない orchestration surface は Rust-tested のままにする。 |
| Traceability | Pass | [source_spec_correspondence.md](./source_spec_correspondence.md) が task 1-29 の source/design/test correspondence を記録する。[00.crate_plan.md](./00.crate_plan.md) はその evidence を autonomous-protocol tasks と gaps に再分類する。 |
| Design/source sync | Pass | 既存 module specs は実装済み source files を記述している。この migration は欠けていた crate plan、exit report、README index entries を追加する。 |
| Boundary discipline | Pass | [README.md](./README.md) と [00.crate_plan.md](./00.crate_plan.md) は source identity ownership、lexer rule、syntax node ownership、parser grammar/recovery、semantic phases、cache storage、LSP rendering を除外している。 |
| Verification | Pass | 現在 branch の verification result を下に記録する。`mizar-test plan` は `mizar-frontend` scope 外の既存 planned/no-tests warning を伴って成功する。 |
| Residual risk | Pass | `MF-GAP-003`、`MF-GAP-004`、`MF-GAP-007` は owner と unblock condition 付きで deferred。blocking/high frontend-owned finding は残っていない。 |

## スコア内訳

| Category | Points |
|---|---:|
| Specification completeness | 18/20 |
| Test contract and coverage | 18/20 |
| Traceability | 14/15 |
| Implementation correctness | 15/15 |
| Design/source synchronization | 9/10 |
| Boundary discipline | 10/10 |
| Verification health | 5/5 |
| Handoff quality | 4/5 |
| Total | 93/100 |

retrospective evidence であること、frontend-only orchestration contract に Rust tests を
使っていること、producer-backed fallback coverage が deferred であることに対して、
小さな減点を残す。

## Deferred Items

| ID | 理由 | Owner | Unblock condition |
|---|---|---|---|
| MF-GAP-003 | 多くの frontend-only orchestration contract は、complete `.miz` source behavior だけでなく provider seam、source-load failure、cache-key assertion、direct fallback constructor を必要とする。 | Frontend maintenance | 挙動が complete-source language/parser behavior になった場合だけ `.miz` tests を追加する。orchestration-only contract は Rust tests を維持する。 |
| MF-GAP-004 | いくつかの reserved/fallback diagnostic surface は、現在の upstream crate から producer できない。 | Owning lexer/session/parser/syntax tasks | concrete upstream variant が存在したときに producer-backed frontend tests を追加する。 |
| MF-GAP-007 | planned template/fixity parser seeds は、parser task が frontend seam 経由で diagnostic を満たせるまで deferred である。 | Parser/frontend integration | parser grammar/fixity support が frontend-reachable になり、traceability metadata を planned から covered へ移せるときに、seed を activate または置き換える。 |

この migration で解決した項目:

- `MF-GAP-001` と `MF-GAP-002`: autonomous crate-plan/report evidence と README
  links を追加した。
- `MF-GAP-005`: task-16 correspondence evidence を protocol hard-gate record に
  取り込んだ。
- `MF-GAP-006`: parser grammar growth は parser-owned のままとし、frontend follow-up
  は passthrough、merge-order、fuzz、cache-key effects に限定する。

## Human Review Surface

human reviewer は主に retrospective protocol evidence を確認する。

- [00.crate_plan.md](./00.crate_plan.md)
- 本 report
- [README.md](./README.md)
- [source_spec_correspondence.md](./source_spec_correspondence.md)
- [bilingual_documentation_synchronization.md](./bilingual_documentation_synchronization.md)
- [todo.md](./todo.md)
- [orchestration.md](./orchestration.md)
- [parsing.md](./parsing.md)
- [lexing.md](./lexing.md)
- [lexical_env.md](./lexical_env.md)
- [cache_key.md](./cache_key.md)
- `tests/coverage/spec_trace.toml`
- 同じ frontend design files の日本語 companion

この report が参照する canonical language/test surface:

- `doc/spec/en/02.lexical_structure.md`
- `doc/spec/en/11.symbol_management.md`
- `doc/spec/en/12.modules_and_namespaces.md`
- `doc/spec/en/15.statements.md`
- `doc/spec/en/16.theorems_and_proofs.md`
- `doc/spec/en/22.error_handling_and_diagnostics.md`
- `doc/spec/en/appendix_a.grammar_summary.md`
- `tests/miz/pass/parser/pass_parser_minimal_token_stream_001.miz`
- `tests/miz/fail/parser/fail_parser_missing_definition_end_001.miz`
- `tests/miz/fail/parser/fail_parser_stray_end_001.miz`
- `tests/miz/pass/parser/pass_parser_template_arguments_001.miz`
- `tests/miz/fail/parser/fail_parser_template_arguments_chained_iff_001.miz`
- 下の summary にある active/planned parser `.expect.toml` sidecars

この migration では、source implementation、`.miz` file、`.expect.toml` file、
canonical language specification file は変更しない。

## Test Expectation Summary

この migration は `.expect.toml` files、snapshots、`.miz` files、source code、fuzz
targets、benchmark code を変更しない。

| Test | Intent | Expected outcome | Expected phase | Diagnostics | Spec refs |
|---|---|---|---|---|---|
| `tests/miz/pass/parser/pass_parser_minimal_token_stream_001.expect.toml` | frontend seam 経由の active parse-only token preservation。 | Pass | Parse-only/frontend seam | fixture-specific metadata を除き none expected | `tests/coverage/spec_trace.toml` parser entries |
| `tests/miz/fail/parser/fail_parser_missing_definition_end_001.expect.toml` | frontend seam 経由の active missing-`end` recovery。 | Fail/recover | Parse-only/frontend seam | Missing-end syntax diagnostics。tags により frontend recovery diagnostics を許容。 | `tests/coverage/spec_trace.toml` parser recovery entries |
| `tests/miz/fail/parser/fail_parser_stray_end_001.expect.toml` | frontend seam 経由の active stray-`end` rejection。 | Fail/recover | Parse-only/frontend seam | Stray-end syntax diagnostics。tags により frontend recovery diagnostics を許容。 | `tests/coverage/spec_trace.toml` parser recovery entries |
| `tests/miz/pass/parser/pass_parser_template_arguments_001.expect.toml` | 将来の parser template seed。 | Planned/deferred | Parse-only | frontend-reachable parser support が存在するまで deferred。 | `spec.en.syntax.template_arguments.parser` |
| `tests/miz/fail/parser/fail_parser_template_arguments_chained_iff_001.expect.toml` | 将来の parser template/fixity fail seed。 | Planned/deferred | Parse-only | frontend-visible diagnostics が存在するまで deferred。 | `spec.en.syntax.template_arguments.parser` |

## Verification

この protocol handoff のために実行したコマンド:

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo run -p mizar-test -- plan --tests-root tests --manifest tests/coverage/spec_trace.toml
```

結果:

- `cargo fmt --check`: passed。
- `cargo clippy --all-targets --all-features -- -D warnings`: passed。
- `cargo test`: passed。
- `cargo run -p mizar-test -- plan --tests-root tests --manifest tests/coverage/spec_trace.toml`: 72 test cases、57 requirements、0 errors、`mizar-frontend` scope 外の既存 planned requirement 4 件の warnings とともに passed。
  - `spec.en.algorithm.vc.assignment_loop_exits`
  - `spec.en.binding.substitution.capture_avoidance`
  - `spec.en.elaboration.choice_comprehension.lowering`
  - `spec.en.type_soundness.escape_and_guard_failures`

## Handoff

Next recommended work:

- 現在 deferred の parse-only `.miz` seed を frontend seam 経由で activate できる
  parser-owned grammar/fixity work を開始する。その後、parser output semantics が
  変わる場合は、diagnostic passthrough、merge-order、fuzz、
  `MIZAR_PARSER_CACHE_KEY_VERSION` への影響に限定した frontend follow-up を開く。

Known constraints:

- parser grammar/recovery decision を `mizar-frontend` へ移さない。
- 現在の implementation behavior に合わせるためだけに `.miz` や expectation を変更しない。
- source/session、lexer、syntax、parser、resolver、cache-storage、LSP display
  boundary を明示したままにする。
- 英語 design docs と日本語 companion を同じ変更で同期する。

Open questions:

- どの parser milestone で最初に `spec.en.syntax.template_arguments.parser` を
  frontend seam 経由で activate するか。
- 将来の upstream non-exhaustive variant のうち、どれが最初に producer-backed
  frontend fallback coverage を必要とするか。

次タスクの推奨 reasoning setting:

- `high`。次の有用な作業は parser/frontend boundary、traceability metadata、active
  `.miz` seeds、diagnostics、cache-version behavior を横断するため。README や
  traceability の documentation-only cleanup なら `medium` へ下げてよく、canonical
  grammar または semantic language behavior を変更する場合は `high` より上げる。
