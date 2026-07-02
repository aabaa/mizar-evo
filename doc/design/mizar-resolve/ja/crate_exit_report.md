# crate exit report: mizar-resolve

> 正本は英語です。英語版:
> [../en/crate_exit_report.md](../en/crate_exit_report.md)。

## 結果

状態: non-deferred task R-001〜R-029 について complete。closeout 時点では task
R-024 は R-G003 `external_dependency_gap` として明示的に deferred だった。2026-07-02
roadmap sync は artifact 側の解除条件が満たされたこと、R-024 を resolver integration
work として再開すべきことを記録する。

Quality score: 94/100。

Score cap: なし。read-only quality review は blocking / high / medium finding なし。
low note は parent verification の通過に依存していたが、その verification は通過した。

## 範囲

milestone scope: `mizar-resolve` task R-001〜R-029。

含むもの:

- R-001〜R-023 は task-by-task commit 済み。
- R-025〜R-029 は task-by-task commit 済み。
- R-023 は初期 active `declaration_symbol` corpus runner seed と traceability
  metadata を追加した。
- R-029 は behavior-preserving な private module / test split を完了した。

除外:

- R-024 summary-backed `ModuleSummary` reuse は closeout 範囲から除外した。
  現在は `mizar-artifact` task 5 が canonical schema、writer、validating reader、
  version compatibility policy を提供しているため、R-024 は artifact-blocked item ではなく
  次の resolver work である。
- public resolver diagnostic code allocation は R-G001 `spec_gap` のまま。現 resolver
  diagnostics は crate-local/internal に保つ。
- import / name / dot-chain / label fact についてのより広い semantic `.miz` assertion は
  R-G007 `test_gap` のまま。

## milestone gate

| milestone | scope | decision |
|---|---|---|
| A | R-001〜R-007 foundation / module-index seam | Passed。crate scaffold、`ResolvedAst`、`SymbolEnv`、deterministic snapshot、resolver-side module-index seam は commit 済み。 |
| B | R-008〜R-016 imports / names | Passed。import graph / path resolution、declaration shell、namespace / name lookup、internal diagnostics、dot-chain finalization は commit 済み。public diagnostic code は R-G001 で deferred。 |
| C | R-017〜R-023 labels / symbols / corpus runner | Passed。label resolution、signature collection、recovered syntax policy、active `declaration_symbol` runner seed は commit 済み。 |
| D | R-024 ModuleSummary reuse | closeout 時点では R-G003 `external_dependency_gap` として deferred。resolver-owned artifact schema、reader、writer、shim は作成していない。artifact 側 blocker は `mizar-artifact` task 5 により解消済みで、R-024 を再開すべきである。 |
| E | R-025〜R-029 hardening / audit / refactor | Passed。determinism、public enum policy、source/spec audit、bilingual sync audit、module-boundary refactor、full verification、quality review は完了。 |

## hard gate

| gate | status | evidence |
|---|---|---|
| specification consistency | Passed | unclassified な blocking/high `spec_gap` は残らない。R-G001 / R-G003 / R-G006 / R-G007 は分類済み。 |
| test contract | Passed | 既存 expectation は rebaseline していない。新規 `.miz` test は R-023 の spec-derived `declaration_symbol` seed に限る。 |
| traceability | Passed | R-023 fixture は expectation sidecar と `tests/coverage/spec_trace.toml` entry を持つ。 |
| design/source sync | Passed | `source_spec_correspondence.md`、`bilingual_documentation_synchronization.md`、`module_boundary_refactor.md` は同期済み。 |
| boundary discipline | Passed | resolver は parser / syntax / frontend / session / build / checker / proof / driver / artifact の責務を所有しない。 |
| verification | Passed | full workspace tests、full clippy、formatting、`mizar-test plan` が完了。 |
| residual risk | Passed | 残る item は deferred、external dependency、または future test-growth record。 |

## score breakdown

| category | points |
|---|---:|
| Specification completeness | 19/20 |
| Test contract and coverage | 18/20 |
| Traceability | 15/15 |
| Implementation correctness | 14/15 |
| Design/source synchronization | 10/10 |
| Boundary discipline | 10/10 |
| Verification health | 4/5 |
| Handoff quality | 4/5 |
| Total | 94/100 |

## deferred item

| ID | reason | owner | unblock condition |
|---|---|---|---|
| R-G001 | public resolver diagnostic code range が `doc/spec/en` chapter 22 に存在しない。 | spec / diagnostics planning | user-facing resolver diagnostic integration 前に public resolver diagnostic ownership を割り当てる。 |
| R-G003 / R-024 | artifact-backed `ModuleSummary` reuse は closeout で実装していない。元の artifact task-5 依存は現在満たされている。 | `mizar-resolve` | canonical `mizar-artifact` schema / writer / reader を使って R-024 を再開する。resolver-local artifact format は作らない。 |
| R-G006 | parser/syntax が module-level scheme/template declaration source role を公開していない。 | `mizar-parser` / `mizar-syntax` | owning source role を公開する。それまでは resolver が module-level scheme/template symbol を創作しない。 |
| R-G007 | import / name / dot-chain / label fact のより広い active semantic `.miz` assertion は未実装。 | future `mizar-test` / resolver corpus work | `doc/spec/en` 由来で runner assertion を拡張し、挙動創作や既存 test rebaseline は行わない。 |

## human review surface

`mizar-resolve` 期間中に追加または変更した primary human-review artifact:

- `tests/miz/pass/resolve/pass_resolve_declaration_symbol_smoke_001.miz`
- `tests/miz/fail/resolve/fail_resolve_duplicate_theorem_symbol_001.miz`

`doc/spec/en` と `doc/spec/ja` は変更していない。既存 `.miz` test と既存 expectation は
implementation に合わせて rebaseline していない。

Codex が維持した derived artifact:

- `doc/design/mizar-resolve/en|ja/*.md`
- `crates/mizar-resolve/**`
- R-023 active declaration-symbol seed の expectation sidecar と
  `tests/coverage/spec_trace.toml` entry。

## test expectation summary

| test | intent | expected outcome | expected phase | diagnostics | spec refs |
|---|---|---|---|---|---|
| `tests/miz/pass/resolve/pass_resolve_declaration_symbol_smoke_001.miz` | parser-backed declaration shell、visibility-bearing declaration、theorem / lemma declaration が symbol collection に到達すること。 | pass | resolve | none | `spec.en.11.symbol_management.signatures`, `spec.en.11.symbol_management.visibility`, `spec.en.12.modules.visibility.semantic`, `spec.en.16.theorems_and_proofs.labels.declaration_symbols` |
| `tests/miz/fail/resolve/fail_resolve_duplicate_theorem_symbol_001.miz` | same-scope duplicate theorem label が proof checking 前の declaration-symbol resolution で拒否されること。 | fail | resolve | internal detail key `declaration_symbol.symbol.duplicate_declaration`; public diagnostic code は空 | `spec.en.16.theorems_and_proofs.labels.same_scope_uniqueness` |

## verification

close-out で実行した command:

```text
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo run -p mizar-test -- plan --tests-root tests --manifest tests/coverage/spec_trace.toml
```

結果:

- `cargo fmt --check`: passed。
- `cargo test`: passed。
- `cargo clippy --all-targets --all-features -- -D warnings`: passed。
- `mizar-test plan`: 0 errors、4 warnings で passed。warning は既存の planned
  requirement without tests:
  `spec.en.algorithm.vc.assignment_loop_exits`,
  `spec.en.binding.substitution.capture_avoidance`,
  `spec.en.elaboration.choice_comprehension.lowering`,
  `spec.en.type_soundness.escape_and_guard_failures`。

## task commit

| task | commit |
|---|---|
| R-001 | `8192219` `feat: scaffold mizar-resolve crate` |
| R-002 | `3bfb0e6` `docs: specify resolved ast shape` |
| R-003 | `de157b7` `docs: specify symbol environment shape` |
| R-004 | `7e9d40d` `feat: add resolved ast data shapes` |
| R-005 | `b8da8fe` `feat: add symbol env data shapes` |
| R-006 | `c9eef80` `feat: add resolver debug snapshots` |
| R-007 | `c069ab8` `feat: add resolver module-index seam` |
| R-008 | `c0d9224` `docs: specify resolver import resolution` |
| R-009 | `1c01bca` `feat: add resolver import graph` |
| R-010 | `03fa162` `feat: resolve import path candidates` |
| R-011 | `e3dd505` `feat: collect declaration shells` |
| R-012 | `3ab02b9` `docs: specify resolver name resolution` |
| R-013 | `178aba3` `feat: resolve namespace paths` |
| R-014 | `9ae672e` `feat: resolve symbol name references` |
| R-015 | `bad8964` `feat: add internal name diagnostics` |
| R-016 | `98749bf` `feat: finalize resolver dot chains` |
| R-017 | `89b85a7` `docs: specify resolver label resolution` |
| R-018 | `cadd158` `feat: resolve theorem and proof labels` |
| R-019 | `9de66c7` `docs: specify resolver signature collection` |
| R-020 | `ed24976` `feat: add symbol collection skeleton` |
| R-021 | `363d55b` `feat: extract parser-backed signatures` |
| R-022 | `4892e5e` `feat: handle resolver recovered syntax` |
| R-023 | `0e0ee9a` `feat: add declaration-symbol corpus runner` |
| R-024 | `cf1084c` `docs: defer module summary reuse gate` |
| R-025 | `b433f32` `test: add resolver determinism regression` |
| R-026 | `d1b7e66` `docs: record resolver enum compatibility policy` |
| R-027 | `085be10` `docs: audit resolver source spec correspondence` |
| R-028 | `dcbf2a9` `docs: audit resolver bilingual documentation sync` |
| R-029 | `7011d5a` `refactor: split resolver private modules` |

## handoff

recommended next task: `mizar-resolve` R-024 `ModuleSummary` reuse を再開する。

recommended reasoning setting: xhigh。narrow な documentation-only preparation task
だけ high へ下げてもよい。implementation では resolver / artifact / build
cacheability boundary をまたぎ、crate ownership を保つ必要があるため xhigh を保つ。

prompt:

```text
Resume mizar-resolve R-024 now that mizar-artifact task 5 provides the
canonical ModuleSummary schema, writer, validating reader, and version
compatibility policy. Follow AGENTS.md and
doc/design/autonomous_crate_development.md through review-only agents,
verification, and commit.

Implement resolver-side consumption of canonical ModuleSummary artifacts without
creating resolver-owned artifact schemas, readers, writers, or shim formats.
Compare summary-backed and source-backed resolution on shared fixtures, keep
artifact ownership in mizar-artifact, and update paired EN/JA design docs and
focused tests as needed.
```
