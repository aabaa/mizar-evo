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

## 計画済み R-032A / R-032B ownership recheck

R-032 は完了済み R-029 refactor gate を再オープンしない。既存 module owner に
split し、public module layout を変えない。

- R-032A: `src/resolved_ast.rs` / `src/resolved_ast/tests.rs` /
  sole R-026 `SurfaceResolvedArenaError` owning-spec decision entry 用
  `tests/lint_policy.rs`。
- R-032B: `src/labels.rs` / `src/labels/tests.rs`。
- paired docs。新 module は計画しない。

R-032A は complete validated structural map と state/key mismatch を含む exact
public error table を所有する。R-032B exact `'a` impl は AST/arena borrow だけを
store、namespace/contribution を own、module を store せず `new` で validate、
`collect` で `resolved.module()` を使う。両 operation は exact public error
enum を返す。module-global ordinal と exact `proof-step-v1` identity は label owner。
exhaustive direct-edge table は default-deny で、unlisted/recovered/malformed/
wrapped edge は syntax/semantic traversal を漏らさず row/ordinal を出さない。
upper boundary は exact `Root` -> `CompilationUnit` -> `ItemList` で、direct
item-list child 以外の theorem は unreachable。
callback、unmapped side channel、fabricated id、unchecked conversion、panic はない。
R-032A node origin `[surface_id]` と R-032B richer table-origin path は意図的に
異なり、それぞれ validate する。

parser/frontend production、Cargo/workspace metadata、他 resolver module、public
checker handoff、checker/type/proof/Core/CFG/VC responsibility は除外する。
implementation pressure が別 source owner、public boundary、mapping owner を必要と
する場合は、変更を広げず frozen R-032A/R-032B contract を停止・再 review する。

R-032A preflight はこの stop rule を正しく発火した。dense
`SurfaceNodeId`-bearing iteration は mizar-syntax S-026 が所有し別 commit で
land する。R-032A は frozen resolver files からその accessor を consume するだけ。
unsafe / dummy-AST id fabrication は禁止のまま。

R-032A implementation preflight は、two-Rust-file wording が mandatory
R-026 public-enum decision owner を欠いた時にも stop rule を発火した。この
欠落は High `design_drift` で semantic `spec_gap` はない。existing enum policy
と source/spec correspondence が exact `tests/lint_policy.rs` entry をすでに
authorizeする。したがって別同期docs-only correctionはimplementationをexact
Rust 3 filesへfreezeし、他lint/module-layout changeはauthorizeしない。

## R-032A implementation result

R-032Aは上記でauthorizeしたexact `src/resolved_ast.rs`、private
`src/resolved_ast/tests.rs`、sole `tests/lint_policy.rs` owning-spec
decision entryだけを使用した。existing `resolved_ast` public moduleがownerの
ままで、module split、ownership transfer、callback、parallel map、
syntax/checker/runner source changeはない。R-032Bは次のseparate logical task
としてexisting `labels` ownerに残る。
