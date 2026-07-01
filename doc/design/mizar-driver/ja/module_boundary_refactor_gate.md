# Module 境界リファクタリング gate

> 正本は英語です。英語版:
> [../en/module_boundary_refactor_gate.md](../en/module_boundary_refactor_gate.md)。

状態: task D-021 で完了。

## 範囲

この監査は、完了済みの `mizar-driver` source layout を
[internal 07](../../internal/ja/07.crate_module_layout.md)、module spec、
source/spec correspondence record に照合する。目的は、public API、deterministic output
schema、diagnostic record、artifact-boundary schema、consumer-visible behavior を変えずに
review bottleneck を取り除くことである。

## 結果

- 完了済み driver surface に対する未解決の blocking/high module-boundary drift は残らない。
- public module surface は `src/lib.rs` の `cli`、`driver`、`events`、`registry`、
  `request` のままである。
- 追加したのは private helper module のみである。phase semantics、proof/cache
  authority、artifact serialization、publication token、LSP conversion、fake producer
  output は `mizar-driver` に移していない。
- public API ownership は module spec と `source_spec_correspondence.md` で引き続き
  文書化される。移動した code は private helper code または private unit-test code である。

## 分割 summary

| 領域 | Before | After | Boundary result |
|---|---|---|---|
| `cli` | `src/cli.rs` が argument parsing、request preparation、driver invocation、output rendering、JSON escaping、exit-code rendering を混在させていた。 | `src/cli.rs` は public CLI request/invocation surface と driver submission preparation を保持する。`src/cli/output.rs` は private rendering、JSON escaping、owner-gap output、exit-code projection helper を所有する。 | public `CliInvocation`、`CliBatchInput`、`CliOutput`、`run_*` function は `cli` module に残る。output byte は既存 CLI / determinism test で cover される。 |
| `driver` | `src/driver.rs` が public driver/session/watch type、submit/cancel orchestration、event construction、scheduler helper logic、watch helper logic、unit test を混在させていた。 | `src/driver.rs` は public driver data type と submit/cancel orchestration を保持する。`src/driver/event_log.rs`、`src/driver/scheduler.rs`、`src/driver/watch.rs` が private helper logic を持つ。`src/driver/tests.rs` は private unit test を持つ。 | public `CompilerDriver`、`BuildSubmission`、watch struct、submit/cancel API は `driver` module に残る。event ordering、cancellation、scheduler consumption、watch freshness は引き続き test される。 |
| `registry` | `src/registry.rs` が public registry/query-boundary type と phase catalog、phase ranking、stable query fingerprint helper を混在させていた。 | `src/registry.rs` は public registry/query-boundary type と service execution method を保持する。`src/registry/catalog.rs` が private phase requirement、ranking、owner/availability lookup、stable fingerprint helper を持つ。 | public `PhaseRegistry`、`DriverQueryBoundary`、`PhaseService`、`required_phase_services` は `registry` module に残る。cache-key purity と deterministic registration は引き続き test される。 |

## Source size check

この gate では、無関係な private helper family を混在させる file、または helper
extraction 後も public facade が概ね 1,000 行を超える file を review bottleneck と扱う。
今回の分割は、helper file を single-purpose に保ちつつ、最大の mixed-responsibility file を
小さくした:

| File | 分割後の行数 | Role |
|---|---|
| `src/request.rs` | 413 | request/session boundary |
| `src/registry.rs` | 602 | public registry and query-boundary API。catalog extraction 前は 808 行。 |
| `src/registry/catalog.rs` | 216 | private phase catalog and fingerprint helpers |
| `src/driver.rs` | 835 | public driver front door and submit/cancel orchestration。helper/test extraction 前は 1,344 行。 |
| `src/driver/event_log.rs` | 198 | private driver event construction |
| `src/driver/scheduler.rs` | 54 | private scheduler result projections |
| `src/driver/watch.rs` | 118 | private watch helper logic |
| `src/driver/tests.rs` | 182 | private driver unit tests |
| `src/events.rs` | 499 | protocol-agnostic event stream |
| `src/cli.rs` | 694 | public CLI request mapping and batch entry point。output extraction 前は 1,275 行。 |
| `src/cli/output.rs` | 594 | private CLI rendering and JSON helpers |

## Follow-up record

この gate は新しい blocking/high task を導入しない。

既存の non-driver owner gap は変更しない:

- semantic/proof/artifact adapter は `external_dependency_gap` のまま;
- real LSP bridge と file-watcher / coalescing owner seam は
  `external_dependency_gap` / `deferred` のまま;
- full real clean / incremental / parallel equivalence は `deferred` のまま;
- 欠落している `mizar-artifact` closeout metadata は report-only
  `repo_metadata_conflict` のまま。

## 検証

D-021 は Rust source と design documentation を変更する。task 単位の check:

- `cargo fmt --check`
- `cargo test -p mizar-driver`
- `cargo clippy -p mizar-driver --all-targets -- -D warnings`
- `git diff --check`
- task 関連 path を stage した後の `git diff --cached --check`

review が cross-crate behavior drift を見つけない限り、この private module split に隣接
crate test は不要である。ただし Rust source change であるため、final crate closeout では
repository hard-gate command として `cargo fmt --check`、
`cargo clippy --all-targets --all-features -- -D warnings`、`cargo test` を実行しなければならない。
