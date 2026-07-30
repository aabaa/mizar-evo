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

## Planned R-032A / R-032B Ownership Recheck

R-032 does not reopen the completed R-029 refactor gate. It is split across
existing public module owners without changing public module layout:

- R-032A production/test/policy: `src/resolved_ast.rs`,
  `src/resolved_ast/tests.rs`, and `tests/lint_policy.rs` for only the R-026
  `SurfaceResolvedArenaError` owning-spec decision entry;
- R-032B production/test/policy: `src/labels.rs`,
  `src/labels/tests.rs`, and `tests/lint_policy.rs` for only the R-026
  `ProofLabelSourceCollectionError` / `labels.md` owning-spec decision;
- synchronized design records; no new module is planned.

R-032A owns the complete validated structural map and exact public error table
in `resolved_ast.md`, including typed resolution-state/reference-key
mismatches. R-032B's exact `'a` impl stores only AST/arena borrows, owns
namespace/contribution, does not store module, validates module in `new`, and
uses `resolved.module()` in `collect`. Both operations return the exact public
error enum. No callback, unmapped side channel, fabricated id, unchecked
conversion, or panic crosses this seam. Its module-global ordinal walk and
exact `proof-step-v1` identity remain label-owned. The exhaustive direct-edge
table is default-deny: an unlisted/recovered/malformed/wrapped edge cannot leak
syntax or semantic traversal across the boundary and produces no row/ordinal.
Its upper boundary is exact `Root` -> `CompilationUnit` -> `ItemList`;
theorems outside direct item-list children are unreachable.
R-032A's per-node arena
origin `[surface_id]` and R-032B's richer table origins are intentionally
distinct and independently validated.

Parser/frontend production, Cargo/workspace metadata, other resolver modules,
public checker handoffs, and checker/type/proof/Core/CFG/VC responsibilities
are excluded. If implementation pressure requires another source owner, public
boundary, or mapping owner, stop and re-review the frozen R-032A/R-032B contract
instead of broadening the change.

R-032A preflight triggered that stop rule correctly. Dense
`SurfaceNodeId`-bearing iteration belongs to mizar-syntax S-026 and must land
there in separate commits; R-032A may only consume the accessor from its
frozen resolver files. Unsafe or dummy-AST id fabrication remains forbidden.

R-032A implementation preflight also exercised the stop rule when the
two-Rust-file wording omitted the mandatory R-026 public-enum decision owner.
The omission is High `design_drift`, with no semantic `spec_gap`: the existing
enum policy and source/spec correspondence already authorize the exact
`tests/lint_policy.rs` entry. A separate synchronized docs-only correction
therefore freezes exactly three Rust files for implementation; no other lint
or module-layout change is authorized.

## R-032A implementation result

R-032A used exactly `src/resolved_ast.rs`, its private
`src/resolved_ast/tests.rs`, and the sole
`tests/lint_policy.rs` owning-spec decision entry authorized above. The
existing `resolved_ast` public module remains the owner; no module split,
ownership transfer, callback, parallel map, or syntax/checker/runner source
change was introduced. At R-032A completion the R-032B stream remained in the
existing `labels` owner as the next work; its current first logical task is
the separate lint-policy docs correction below.

## R-032B lint-policy frozen-scope correction (current prerequisite)

R-032B implementation inventory exercised the stop rule again: its frozen
public `ProofLabelSourceCollectionError` is necessarily scanned by the R-026
guard. The omitted policy owner is High `design_drift`, not a semantic
`spec_gap`, `test_gap`, or test-intent change. The later implementation owns
exactly `src/labels.rs`, `src/labels/tests.rs`, and
`tests/lint_policy.rs`, where the last file may receive only the sole
`ProofLabelSourceCollectionError` owning-spec decision with
`spec_name: "labels.md"`. No module split or ownership transfer is authorized.

The current synchronized docs-only prerequisite covers exactly 31 design
files: 16 resolver, eight checker, six `mizar-test`, and one global ledger.
It changes no source, specification, fixture, sidecar, expectation, trace
status/count, Cargo metadata, semantic contract, or test intent.
`spec_coverage_audit.md` is a deliberate no-op. The independent
specification, test/scope, and source/documentation consistency reviews
report **NO FINDINGS**, and the docs-only verification/count/hash gates PASS.
Independent final read-only quality also reports **NO FINDINGS**; all nine
hard gates PASS with no cap at valid `100/100`
(`20/20/15/15/10/10/5/5`). Only task-only staging/cached-diff review, commit,
and post-commit invariant/fresh-inventory gates remain pending; fresh inventory
after that separate commit gates the three-Rust-file R-032B implementation.
