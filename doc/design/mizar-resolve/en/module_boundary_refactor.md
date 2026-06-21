# Module-Boundary Refactor Gate

> Canonical language: English. Japanese companion:
> [../ja/module_boundary_refactor.md](../ja/module_boundary_refactor.md).

Status: task R-029 gate complete.

## Scope

This gate audits the `mizar-resolve` source layout after tasks R-001 to R-028.
It looks for oversized files, mixed responsibilities, and private helpers that
should be split along the resolver module table and module-spec boundaries.

The refactor is behavior-preserving. It does not move public module names,
public types, public methods, diagnostic payload ordering, deterministic debug
rendering text, `.miz` tests, expectation sidecars, or traceability metadata.

## Result

- Public module paths remain `mizar_resolve::{declarations, env, imports,
  labels, module_index, names, resolved_ast, symbols}`.
- Review-bottleneck inline unit-test blocks moved to per-module private
  `tests.rs` files.
- Private renderer/validation/diagnostic helper blocks moved to private
  submodules where they had become independent review surfaces:
  `env/snapshot.rs`, `resolved_ast/snapshot.rs`,
  `resolved_ast/validation.rs`, and `names/diagnostics.rs`.
- No public API, behavior contract, crate responsibility boundary, or artifact
  schema changed.
- The source/spec correspondence and bilingual documentation synchronization
  scopes were re-run for moved APIs and found no new drift.

## Source Layout

| Public module | Public source | Private helpers/tests after R-029 | Gate result |
|---|---|---|---|
| `declarations` | `src/declarations.rs` | `src/declarations/tests.rs` | Public declaration-shell API stayed in the module root; tests moved out of the implementation body. |
| `env` | `src/env.rs` | `src/env/snapshot.rs`, `src/env/tests.rs` | `SymbolEnv` and index APIs stayed in the module root; deterministic snapshot rendering moved to a private helper module. |
| `imports` | `src/imports.rs` | `src/imports/tests.rs` | Import path and graph APIs stayed in the module root; tests moved out of the implementation body. |
| `labels` | `src/labels.rs` | `src/labels/tests.rs` | Label projection/resolution APIs stayed in the module root; tests moved out of the implementation body. |
| `module_index` | `src/module_index.rs` | `src/module_index/tests.rs` | Resolver-side module-index seam stayed in the module root; tests moved out of the implementation body. |
| `names` | `src/names.rs` | `src/names/diagnostics.rs`, `src/names/tests.rs` | Namespace/name/dot-chain APIs stayed in the module root; crate-local internal diagnostic assembly moved to a private helper module. |
| `resolved_ast` | `src/resolved_ast.rs` | `src/resolved_ast/snapshot.rs`, `src/resolved_ast/validation.rs`, `src/resolved_ast/tests.rs` | Resolved AST data shapes stayed in the module root; deterministic snapshot rendering and validation helpers moved to private modules. |
| `symbols` | `src/symbols.rs` | `src/symbols/tests.rs` | Symbol/signature APIs stayed in the module root; tests moved out of the implementation body. |
| private recovery policy | `src/recovery.rs` | none | Already small and private; no split needed. |

## Re-Run Audits

- Source/spec correspondence: public API source roots still match the module
  specs. Rows whose helpers moved now cite the private helper paths in
  [source_spec_correspondence.md](./source_spec_correspondence.md).
- Bilingual documentation synchronization: this file was added in both
  language directories, and task/status wording now treats R-029 as complete in
  [todo.md](./todo.md) and the crate plan.
- Boundary discipline: the split is local to `mizar-resolve`; it does not add
  parser, syntax, frontend, build, checker, proof, diagnostics registry, driver,
  or artifact responsibilities.

## Verification

The gate requires the normal resolver verification after the refactor:

```text
cargo fmt --check
cargo test -p mizar-resolve
cargo clippy -p mizar-resolve --all-targets --all-features -- -D warnings
```

Crate-wide close-out must run the full workspace and `mizar-test` plan gates.
