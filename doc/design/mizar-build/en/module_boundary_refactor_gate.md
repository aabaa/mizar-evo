# Module-Boundary Refactor Gate

> Canonical language: English. Japanese companion:
> [../ja/module_boundary_refactor_gate.md](../ja/module_boundary_refactor_gate.md).

## Scope

Task 26 audits the `mizar-build` source layout after the architecture-22
follow-up audit. The gate checks for oversized files, mixed responsibilities,
and private helpers that should be split along the module table and module
specification boundaries before crate closeout.

The refactor scope is intentionally narrow:

- public modules, public items, diagnostics, artifact-facing schemas, and
  deterministic renderings must remain unchanged;
- no behavior cleanup or API exposure may be mixed into the move;
- `mizar-build` must remain independent from `mizar-driver`;
- cache-aware scheduling remains an execution-skip optimization only, and
  `mizar-build` must not construct `mizar-cache` cache keys, dependency
  fingerprints, or proof-reuse validation records.

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
| BUILD-G-018 | resolved `source_drift` / layout-only | Before task 26, `planner.rs`, `module_index.rs`, `task_graph.rs`, and `scheduler.rs` each kept substantial inline unit-test modules inside the public module file. The implementation responsibilities matched their specs, but the large test blocks made these files review bottlenecks and obscured the source/spec boundary audit. | Task 26 moved only the inline unit-test bodies into private child modules at `src/{planner,module_index,task_graph,scheduler}/tests.rs`; parent modules, public exports, diagnostics, and behavior are unchanged. |

No unresolved `spec_gap`, `test_gap`, `design_drift`, `source_drift`,
`source_undocumented_behavior`, `test_expectation_drift`, `boundary_violation`,
or `repo_metadata_conflict` remains for task 26. Existing external dependency
gaps for real driver, IR, and producer publication-token integration remain
unchanged.

## Implemented Split

The parent module files retain the same public API surface and replace each
inline `#[cfg(test)] mod tests { ... }` block with:

```rust
#[cfg(test)]
mod tests;
```

The moved test modules remain private unit-test children. Because they are
still children of the same public modules, they retain access to private
helpers through `super` and do not add any new production dependency edge.

Task 26 applies this split to:

- `crates/mizar-build/src/planner.rs` ->
  `crates/mizar-build/src/planner/tests.rs`;
- `crates/mizar-build/src/module_index.rs` ->
  `crates/mizar-build/src/module_index/tests.rs`;
- `crates/mizar-build/src/task_graph.rs` ->
  `crates/mizar-build/src/task_graph/tests.rs`;
- `crates/mizar-build/src/scheduler.rs` ->
  `crates/mizar-build/src/scheduler/tests.rs`.

Implementation helpers are not split in this task. The audited helpers are
cohesive within their owning module specs, and splitting them now would create
new private APIs without reducing a concrete behavior or review risk.

## Verification

Because task 26 changes Rust source layout, the required verification is:

- `cargo test -p mizar-build`
- `cargo clippy -p mizar-build --all-targets -- -D warnings`
- `cargo fmt --check`

Task 26 verification passed those commands. Additional regression checks also
passed for `cargo test -p mizar-cache`, `cargo test -p mizar-artifact`, `cargo
test -p mizar-vc`, and `cargo test -p mizar-proof`. At the time of task 26,
driver integration remained an `external_dependency_gap`; task 27 handles the
build-owned dispatch seam without adding a stubbed driver dependency.
