# Module Boundary Audit: mizar-kernel

> Canonical language: English. Japanese companion:
> [../ja/module_boundary_audit.md](../ja/module_boundary_audit.md).

## Scope

Task 22 audits the `mizar-kernel` source layout before crate closeout. The
audit checks whether public modules, private helpers, and tests still follow
the module responsibilities in [todo.md](./todo.md),
[source_spec_audit.md](./source_spec_audit.md), and
[internal 07](../../internal/en/07.crate_module_layout.md).

The only runtime/module source change in this task is a move-only split of
module-local test modules; the executable source-layout lint guard is updated
to recognize those private test files. Runtime code, public APIs, diagnostics,
deterministic renderings, artifact-facing schemas, certificate semantics,
rejection semantics,
`doc/spec`, `.miz` fixtures, expectations, SAT/ATP/proof search, premise
selection, overload resolution, cluster search, implicit coercion insertion,
fallback inference, global mutable state, and downstream ATP/proof/cache/artifact
integration are unchanged.

## Source Inventory

The public module set remains exactly the six spec-backed modules exported from
`src/lib.rs`: `certificate_parser`, `checker`, `clause`, `rejection`,
`resolution_trace`, and `substitution_checker`.

| Parent module | Before task 22 | After task 22 parent | New private test file | Classification | Action |
|---|---:|---:|---:|---|---|
| `certificate_parser` | 2971 lines | 1666 lines | `src/certificate_parser/tests.rs` (1295 lines) | `design_drift` / `source_drift` | Move-only split of inline tests. |
| `checker` | 5412 lines | 2211 lines | `src/checker/tests.rs` (3180 lines) | `design_drift` / `source_drift` | Move-only split of inline tests. |
| `clause` | 1462 lines | 924 lines | `src/clause/tests.rs` (534 lines) | `design_drift` / `source_drift` | Move-only split of inline tests. |
| `rejection` | 1077 lines | 472 lines | `src/rejection/tests.rs` (599 lines) | `design_drift` / `source_drift` | Move-only split of inline tests. |
| `resolution_trace` | 2114 lines | 653 lines | `src/resolution_trace/tests.rs` (1446 lines) | `design_drift` / `source_drift` | Move-only split of inline tests. |
| `substitution_checker` | 4719 lines | 2648 lines | `src/substitution_checker/tests.rs` (2054 lines) | `design_drift` / `source_drift` | Move-only split of inline tests. |

The drift subtype is review-boundary drift: the implementation files combined
trusted runtime code with large test fixtures, which made review of runtime
logic harder without adding a public module responsibility. Moving tests into
private `#[cfg(test)] mod tests;` files reduces review pressure while keeping
the tests inside their owning parent module.

## Boundary Decision

- No public module is added, removed, or renamed.
- No runtime helper is moved in task 22.
- The new directories under `crates/mizar-kernel/src/` are private module
  directories used only for `tests.rs`.
- The source layout lint allows exactly these private test files and still
  requires paired English/Japanese specs before any public module is exposed.
- The Task 22 lint guard requires the private test files and paired
  `module_boundary_audit.md` documents to be present in the Git index before
  final verification passes, so untracked split files cannot be omitted from
  the commit.
- The Task 20 source/spec audit is updated so test traceability points at the
  new private test files, and the lint guard checks those traceability paths.
- The Task 21 bilingual sync audit is updated to include this new paired audit
  document.

## Gap Classification

| ID | Class | Evidence | Current action |
|---|---|---|---|
| KERNEL22-G001 | `design_drift` / `source_drift` | Inline `#[cfg(test)] mod tests` blocks made the trusted runtime modules large enough to obscure review boundaries. | Fixed by move-only private test-module split. |
| KERNEL22-G002 | `external_dependency_gap` / `deferred` | Source-derived certificates, ATP proof translation, cluster/reduction payload producers, derived-fact payload schemas, service envelopes, and downstream proof/cache/artifact consumers remain outside the crate. | Keep external/deferred classifications; do not add placeholder integration. |
| KERNEL22-G003 | `repo_metadata_conflict` | None observed in task 22. | Report only if future metadata conflicts appear; do not auto-repair unrelated metadata. |

## Verification Plan

Task 22 changes Rust source layout without intended behavior changes, so the
required verification is:

- `cargo fmt --check`;
- `cargo test -p mizar-kernel`;
- `cargo clippy -p mizar-kernel --all-targets --all-features -- -D warnings`;
- `git diff --check`;
- `git diff --cached --check` after explicit path staging.

The Task 22 lint guard is rerun after explicit path staging because it checks
that the new private test files and paired module-boundary audit documents are
tracked in the Git index.

`cargo test -p mizar-core` and `cargo test -p mizar-checker` are not required
unless the move reveals a source change to binder contracts or checker/trace
runtime behavior; task 22 is a private test-layout split only.
