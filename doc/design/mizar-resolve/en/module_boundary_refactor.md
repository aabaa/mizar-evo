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
existing `labels` owner as the next work; its separate lint-policy correction
below is now committed.

## R-032B lint-policy frozen-scope correction (completed prerequisite record)

R-032B implementation inventory exercised the stop rule again: its frozen
public `ProofLabelSourceCollectionError` is necessarily scanned by the R-026
guard. The omitted policy owner is High `design_drift`, not a semantic
`spec_gap`, `test_gap`, or test-intent change. The later implementation owns
exactly `src/labels.rs`, `src/labels/tests.rs`, and
`tests/lint_policy.rs`, where the last file may receive only the sole
`ProofLabelSourceCollectionError` owning-spec decision with
`spec_name: "labels.md"`. No module split or ownership transfer is authorized.

The completed synchronized docs-only correction covers exactly 31 design
files: 16 resolver, eight checker, six `mizar-test`, and one global ledger.
It changes no source, specification, fixture, sidecar, expectation, trace
status/count, Cargo metadata, semantic contract, or test intent.
`spec_coverage_audit.md` is a deliberate no-op. The independent
specification, test/scope, and source/documentation consistency reviews
report **NO FINDINGS**, and the docs-only verification/count/hash gates PASS.
Independent final read-only quality also reports **NO FINDINGS**; all nine
hard gates PASS with no cap at valid `100/100`
(`20/20/15/15/10/10/5/5`). At that pre-commit record, only task-only
staging/cached-diff review, commit, and post-commit invariant/fresh-inventory
gates remained pending. They subsequently completed in correction commit
`f1cf0a5d15f2db51176e9e91a4f5a6447a88ad7a` and its fresh inventory.

## R-032B implementation result

R-032B remains entirely within the existing `labels` owner. Commit
`b3a7e79a6b60db2974e911c69bb56ff5f4609064` changed exactly
`src/labels.rs`, its private `src/labels/tests.rs`, and
the sole `tests/lint_policy.rs` decision authorized above. It adds no module
split, ownership transfer, callback, unmapped side channel, fabricated id,
syntax/checker/runner source, active artifact, or semantic owner.

The collector consumes the R-032A arena and owns only label projection,
reference candidate, scope/ordinal/completion, and resolver provenance
derivation. The historical private `mizar-test` B5C route was its production
consumer; public checker unresolved-reference handoff remains excluded.
The Medium third-child and unauthorized `Default` / `From` implementation
findings and every test-sufficiency finding are fixed. Final fresh
test-sufficiency, implementation, and source/documentation rereviews report
**NO FINDINGS**, and all pre-quality verification gates PASS. Independent
final quality also reports **NO FINDINGS**; all nine hard gates PASS with no
score cap at valid `100/100` (`20/20/15/15/10/10/5/5`). Task-only
restaging/cached-diff review, commit, and post-commit invariant/fresh
inventory are complete.

## Checker Task 258B5C boundary status

The historical B5C source/test delta is limited to the private `mizar-test`
consumer, exactly two fail fixtures, two sidecars, two covered trace rows,
and four frozen active-count/CLI assertions in
`crates/mizar-test/tests/metadata.rs` from declaration stage `5` to `7`. It
consumes unchanged R-032A
`SurfaceResolvedArena` and R-032B
`ProofLabelSourceCollector` / `LabelResolver` APIs and makes no resolver
production/API change. Plan/pass/fail counts are `421/389` and `228/193`;
active parse/declaration/type/proof is `101/7/198/1`; warning/error counts are
`23/0`.

Public codes remain empty; only private key
`declaration_symbol.label.proof_scope_confinement` authenticates the route.
The two confinement negatives close only that R-G007 slice; import, name,
dot-chain, and other label-reference work remains open. B5C test,
implementation, source/documentation reviews and all verification gates are
complete. Independent final quality reports **NO FINDINGS**; all nine hard
gates PASS with no score cap at valid `100/100`. Task-only cached-diff review,
dedicated commit `33ac57e96f048dc40559565f54369cac854409a7`, and post-commit fresh inventory
are complete at this historical checkpoint.

## Checker Task 263R Frozen Boundary

The prerequisite and later repair remain inside the existing `symbols` owner.
Implementation may edit only `src/symbols.rs` and its private
`src/symbols/tests.rs`. The owner discriminator is an internal declaration-
shell identity used only by duplicate classification; it is not added to
`SymbolId`, `DefinitionShell`, `SignatureShell`, `SymbolEnv`, module summaries,
or public APIs. No module split, dependency edge, resolver/checker ownership
transfer, lint-policy decision, fixture, runner, or Cargo change is authorized.
Task 263 consumes the corrected resolver result only after a dedicated lower
commit and fresh inventory.

## Checker Task 263R Implemented Boundary

The implementation stays in the frozen existing `symbols` owner and changes
exactly `src/symbols.rs` plus its private `src/symbols/tests.rs`. The new
selector-owner field, parent walk, and conflict-key component are private.
There is no module split, public surface change, dependency edge, lint-policy
change, resolver/checker ownership transfer, runner route, corpus artifact,
trace metadata, or Cargo change. Task 263 remains the only future production
consumer after this lower commit and fresh inventory.

The final boundary/consistency review reports **NO FINDINGS**, and all nine
quality gates PASS at uncapped `100/100`; exact staging and commit remain.

## Checker Task 264R Frozen Boundary

The lower correction remains inside existing modules: enum/mapping and shell
tests in `declarations.rs` / `declarations/tests.rs`, plus no-projection,
stable sibling/anchor fingerprints, append-only code/key, and symbol tests in
`symbols.rs` / `symbols/tests.rs`. It adds no module, dependency, Cargo target,
public semantic identity, or diagnostic class. The only public ABI delta is the
non-exhaustive append-only shell variant. Checker Task 248P and Task 264 remain
separate consumers; no module split or line-count threshold decision changes.

## Checker Task 264R Implemented Boundary

The implementation stays in the existing `declarations` and `symbols` owners
and their private test modules: exactly four Rust files, no new module or
dependency, and only the frozen append-only public shell variant. Checker,
runner, corpus, trace, Cargo, lint inventory, and module-split policy are
unchanged.

## Resolver Task 277R1 Module Boundary

The later implementation remains in the existing public `names` owner and its
private `names/tests.rs` module. `SurfaceResolvedArena` remains the sole
resolver identity authority: the collector validates it and calls
`resolved_node_for`, but neither changes `resolved_ast.rs` nor constructs IDs.
There is no module split, dependency/Cargo change, `SymbolId`, `NameRef`,
`ResolvedAst` field, public diagnostic/error enum, checker handoff, or
resolver-to-checker ownership transfer. The two mizar-test paths are test-only
consumers; production runner and all active routes remain outside this
boundary.
