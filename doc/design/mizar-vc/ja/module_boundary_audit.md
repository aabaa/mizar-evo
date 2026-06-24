# Module-Boundary Refactor Gate: mizar-vc

> 正本言語: 英語。英語正本:
> [../en/module_boundary_audit.md](../en/module_boundary_audit.md)。

Task 22 は crate closeout 前に `mizar-vc` の source layout を監査する。必須の
move-only split が見つからない限り audit-only task である。この task は Rust source、
public API、diagnostic、deterministic rendering、artifact-facing schema、`.miz` fixture、
expectation、`doc/spec`、traceability metadata、runner support、downstream
ATP/kernel/proof/cache integration を変更しない。

## 範囲と入力

この audit は [internal 07](../../internal/ja/07.crate_module_layout.md)、mizar-vc crate
plan、現在の module specification に従う:

- [vc_ir.md](./vc_ir.md)
- [generator.md](./generator.md)
- [discharge.md](./discharge.md)
- [dependency_slice.md](./dependency_slice.md)
- [source_spec_audit.md](./source_spec_audit.md)
- [architecture_22_audit.md](./architecture_22_audit.md)

現在の source line count:

| Path | 行数 | 主な責務 |
|---|---:|---|
| `crates/mizar-vc/src/lib.rs` | 10 | public module export boundary。 |
| `crates/mizar-vc/src/vc_ir.rs` | 3517 | `VcSet`、VC IR data shape、validation、status projection、anchor、fingerprint、rendering、tests。 |
| `crates/mizar-vc/src/generator.rs` | 3368 | seed-intake candidate generation、flow-derived candidate、normalization、anchor construction、tests。 |
| `crates/mizar-vc/src/discharge.rs` | 2113 | deterministic pre-ATP discharge、evidence/explanation、evidence hashing、tests。 |
| `crates/mizar-vc/src/dependency_slice.rs` | 2573 | dependency-slice collection、reusable fingerprint、proof-reuse candidate key、tests。 |
| `crates/mizar-vc/tests/determinism_suite.rs` | 676 | cross-module deterministic pipeline/reuse coverage。 |
| `crates/mizar-vc/tests/lint_policy.rs` | 849 | manifest、public-module、lint-policy、audit、public-enum guards。 |

## 境界レビュー

| 境界 | finding | 判断 |
|---|---|---|
| public module exports | `lib.rs` は `vc_ir`、`generator`、`discharge`、`dependency_slice` だけを export し、paired module spec と lint-policy guard に一致している。 | split や re-export 変更は不要。 |
| `vc_ir.rs` | 大きい file だが、owned data shape、validation、deterministic rendering、status projection、anchor、canonical fingerprint に凝集している。validation/rendering/fingerprint helper の分割は review size を下げ得るが、mixed public ownership や behavior-boundary violation はない。 | watchlist のみ。closeout 前に必須の move-only split は不要。 |
| `generator.rs` | 大きい file だが、seed-derived candidate production と normalization に凝集している。algorithm fixture helper が test size の多くを占める。flow candidate や tests の private submodule 化は maintenance work になり得るが、現在の public API と spec boundary は一致している。 | watchlist のみ。closeout 前に必須の move-only split は不要。 |
| `discharge.rs` | medium-large file で、deterministic discharge という単一責務を持つ。evidence record、rule selection、tests は discharge spec によって結合している。 | split 不要。 |
| `dependency_slice.rs` | 大きい file だが、dependency collection、unknown coverage、reusable fingerprinting、proof-reuse candidate key に凝集している。fingerprint payload や tests の private helper split は review friction を下げ得るが、source/spec ownership は 1 module のままである。 | watchlist のみ。closeout 前に必須の move-only split は不要。 |
| integration tests | `determinism_suite.rs` と `lint_policy.rs` は長いが、cross-module behavior と policy guard を意図的に cover する。 | crate completion に split は不要。 |

## 分類

- `design_drift`: なし。public module boundary は module spec と internal ownership map に
  まだ一致している。
- `source_drift`: なし。source file は大きいが、public ownership boundary をまたがず、
  documented module responsibility と矛盾しない。
- `source_undocumented_behavior`: この audit では観測しない。
- `test_gap`: module-boundary gate についてはなし。Task 22 は docs-only で diff check を使う。
  source-moving task なら Rust verification が必要である。
- `repo_metadata_conflict`: 観測しない。
- `deferred`: 任意の maintenance refactor として、将来 `vc_ir`、`generator`、
  `dependency_slice` 内の private helper cluster や tests を分割してよい。ただし crate exit gate
  には不要であり、実施する場合は独立した move-only task にしなければならない。

## Gate 判断

closeout 前に必須の move-only split はない。現在の public module boundary のまま crate は
closeout quality review に進める。line count は maintenance watchlist であり hard gate failure ではない。
file は documented module responsibility と一致し続け、すべての public surface は lint-policy と
source/spec audit によって guard されているためである。
