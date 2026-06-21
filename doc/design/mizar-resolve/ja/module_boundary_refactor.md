# module 境界リファクタリング gate

> 正本は英語です。英語版:
> [../en/module_boundary_refactor.md](../en/module_boundary_refactor.md)。

状態: task R-029 gate complete。

## 範囲

この gate は、task R-001〜R-028 後の `mizar-resolve` source layout を監査する。
oversized file、混在した責務、resolver module table と module spec 境界に沿って
分割すべき private helper を確認する。

この refactor は behavior-preserving である。public module name、public type、public
method、diagnostic payload ordering、deterministic debug rendering text、`.miz` test、
expectation sidecar、traceability metadata は移動または変更しない。

## 結果

- public module path は `mizar_resolve::{declarations, env, imports, labels,
  module_index, names, resolved_ast, symbols}` のままである。
- review bottleneck になっていた inline unit-test block を、module ごとの private
  `tests.rs` file へ移した。
- private renderer / validation / diagnostic helper block は、独立した review surface に
  なっていた箇所だけ private submodule へ移した:
  `env/snapshot.rs`、`resolved_ast/snapshot.rs`、
  `resolved_ast/validation.rs`、`names/diagnostics.rs`。
- public API、behavior contract、crate responsibility boundary、artifact schema は
  変更していない。
- 移動した API について source/spec 対応と二言語ドキュメント同期の scope を再実行し、
  新しい drift は見つからなかった。

## source layout

| public module | public source | R-029 後の private helper / test | gate result |
|---|---|---|---|
| `declarations` | `src/declarations.rs` | `src/declarations/tests.rs` | public declaration-shell API は module root に残し、test を implementation body から移した。 |
| `env` | `src/env.rs` | `src/env/snapshot.rs`, `src/env/tests.rs` | `SymbolEnv` と index API は module root に残し、deterministic snapshot rendering を private helper module へ移した。 |
| `imports` | `src/imports.rs` | `src/imports/tests.rs` | import path / graph API は module root に残し、test を implementation body から移した。 |
| `labels` | `src/labels.rs` | `src/labels/tests.rs` | label projection / resolution API は module root に残し、test を implementation body から移した。 |
| `module_index` | `src/module_index.rs` | `src/module_index/tests.rs` | resolver-side module-index seam は module root に残し、test を implementation body から移した。 |
| `names` | `src/names.rs` | `src/names/diagnostics.rs`, `src/names/tests.rs` | namespace / name / dot-chain API は module root に残し、crate-local internal diagnostic assembly を private helper module へ移した。 |
| `resolved_ast` | `src/resolved_ast.rs` | `src/resolved_ast/snapshot.rs`, `src/resolved_ast/validation.rs`, `src/resolved_ast/tests.rs` | Resolved AST data shape は module root に残し、deterministic snapshot rendering と validation helper を private module へ移した。 |
| `symbols` | `src/symbols.rs` | `src/symbols/tests.rs` | symbol / signature API は module root に残し、test を implementation body から移した。 |
| private recovery policy | `src/recovery.rs` | none | すでに小さく private なので分割不要。 |

## 再実行した監査

- source/spec 対応: public API source root は引き続き module spec と対応している。
  helper を移動した行は [source_spec_correspondence.md](./source_spec_correspondence.md) で
  private helper path も参照する。
- 二言語ドキュメント同期: この file を両言語 directory に追加し、[todo.md](./todo.md)
  と crate plan の task / status wording は R-029 完了として同期した。
- 境界規律: 分割は `mizar-resolve` 内に閉じている。parser、syntax、frontend、
  build、checker、proof、diagnostics registry、driver、artifact の責務は追加しない。

## verification

この gate は refactor 後に通常の resolver verification を要求する:

```text
cargo fmt --check
cargo test -p mizar-resolve
cargo clippy -p mizar-resolve --all-targets --all-features -- -D warnings
```

crate-wide close-out では full workspace と `mizar-test` plan gate を実行する。
