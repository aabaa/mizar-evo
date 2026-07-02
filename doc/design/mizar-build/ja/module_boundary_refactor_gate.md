# Module-Boundary Refactor Gate

> Canonical language: English. Canonical document:
> [../en/module_boundary_refactor_gate.md](../en/module_boundary_refactor_gate.md).

## Scope

task 26 は architecture-22 follow-up audit 後の `mizar-build` source layout を
監査する。この gate は crate closeout の前に、oversized files、mixed
responsibilities、module table と module specification boundaries に沿って分割すべき
private helpers がないかを確認する。

refactor scope は意図的に狭い:

- public modules、public items、diagnostics、artifact-facing schemas、
  deterministic renderings は変更しない。
- behavior cleanup や API exposure を move に混ぜない。
- `mizar-build` は `mizar-driver` に依存しないままでなければならない。
- cache-aware scheduling は execution-skip optimization のみであり、
  `mizar-build` は `mizar-cache` cache keys、dependency fingerprints、
  proof-reuse validation records を構築してはならない。

## Audited Inputs

- [00.crate_plan.md](./00.crate_plan.md)
- [todo.md](./todo.md)
- [planner.md](./planner.md)
- [module_index.md](./module_index.md)
- [task_graph.md](./task_graph.md)
- [scheduler.md](./scheduler.md)
- `crates/mizar-build/src/{planner,module_index,task_graph,scheduler}.rs`

## Gap Classification

| ID | Class | Evidence | Action |
|---|---|---|---|
| BUILD-G-018 | resolved `source_drift` / layout-only | task 26 の前は、`planner.rs`、`module_index.rs`、`task_graph.rs`、`scheduler.rs` がそれぞれ大きな inline unit-test module を public module file 内に保持していた。実装責務は specs と一致していたが、大きな test blocks は review bottleneck になり、source/spec boundary audit を読みにくくしていた。 | task 26 は inline unit-test body のみを `src/{planner,module_index,task_graph,scheduler}/tests.rs` の private child modules へ移動した。parent modules、public exports、diagnostics、behavior は変更していない。 |

task 26 では unresolved `spec_gap`、`test_gap`、`design_drift`、
`source_drift`、`source_undocumented_behavior`、`test_expectation_drift`、
`boundary_violation`、`repo_metadata_conflict` は残っていない。実 driver、IR、
producer publication-token integration に関する既存の external dependency gaps は変更しない。

## Implemented Split

parent module files は同じ public API surface を保持し、各 inline
`#[cfg(test)] mod tests { ... }` block を次の形に置き換える:

```rust
#[cfg(test)]
mod tests;
```

移動された test modules は private unit-test children のままである。同じ public
modules の children のままなので、`super` を通じて private helpers へアクセスでき、
production dependency edge は追加されない。

task 26 はこの split を次の対象へ適用する:

- `crates/mizar-build/src/planner.rs` ->
  `crates/mizar-build/src/planner/tests.rs`
- `crates/mizar-build/src/module_index.rs` ->
  `crates/mizar-build/src/module_index/tests.rs`
- `crates/mizar-build/src/task_graph.rs` ->
  `crates/mizar-build/src/task_graph/tests.rs`
- `crates/mizar-build/src/scheduler.rs` ->
  `crates/mizar-build/src/scheduler/tests.rs`

implementation helpers はこの task では分割しない。監査した helpers は所有する
module specs 内で cohesive であり、今ここで分割すると、具体的な behavior risk や
review risk を減らさずに新しい private APIs を作ることになる。

## Verification

task 26 は Rust source layout を変更するため、required verification は次の通り:

- `cargo test -p mizar-build`
- `cargo clippy -p mizar-build --all-targets -- -D warnings`
- `cargo fmt --check`

task 26 verification はこれらの commands に合格した。追加の regression checks として
`cargo test -p mizar-cache`、`cargo test -p mizar-artifact`、`cargo test -p
mizar-vc`、`cargo test -p mizar-proof` も合格した。task 26 時点では driver
integration は `external_dependency_gap` のままだった。task 27 は stubbed driver
dependency を追加せず、build-owned dispatch seam を扱う。
