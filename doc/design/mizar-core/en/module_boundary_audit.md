# Module-Boundary Refactor Gate: mizar-core

> Canonical language: English. Japanese companion:
> [../ja/module_boundary_audit.md](../ja/module_boundary_audit.md).

Task 24 audits the `mizar-core` source layout before closeout. It checks
whether oversized files, mixed responsibilities, or private helpers require a
behavior-neutral move before the crate is treated as ready for downstream
consumers.

This task is audit-only. No Rust source is moved in Task 24 because the current
public module layout matches the module specification boundary, no current
review-bottleneck requiring a split is found, and no blocking boundary
violation is observed.

## Scope And Method

The audit covers:

- `crates/mizar-core/src/lib.rs`
- `crates/mizar-core/src/core_ir.rs`
- `crates/mizar-core/src/binder_normalization.rs`
- `crates/mizar-core/src/elaborator.rs`
- `crates/mizar-core/src/control_flow.rs`
- crate-local integration tests under `crates/mizar-core/tests/`
- English/Japanese module specs under `doc/design/mizar-core/{en,ja}/`
- lint and audit guards in `crates/mizar-core/tests/lint_policy.rs`

The review compares source layout with the module table in `todo.md`, the
source/spec audit, and the owning module specifications. It treats file size as
a reviewability signal, not a reason to move code by itself. A split is
required only when a file mixes responsibilities across module/spec boundaries,
exposes unexpected public APIs, or blocks safe review of future work.

## Source Inventory

| Source | Approx. lines at audit | Owning spec | Boundary result |
|---|---:|---|---|
| `src/lib.rs` | 9 | module table in `todo.md` | Exports exactly `binder_normalization`, `control_flow`, `core_ir`, and `elaborator`. No drift. |
| `src/core_ir.rs` | 4016 | `core_ir.md` | Large but cohesive data-shape module. Task 31 adds only the specified pending-proof status variant. No split required. |
| `src/binder_normalization.rs` | 5828 | `binder_normalization.md` | Large but cohesive binder/substitution/canonicalization module. Future private helper extraction is optional. |
| `src/elaborator.rs` | 17132 | `elaborator.md` | Largest review-risk file, but its sections map to the six elaboration steps in the owning spec. Task 31 adds one localized exact-adapter section and its fail-closed test matrix after generic proof lowering; it consumes checker payloads and does not introduce a new module responsibility. No split is required. |
| `src/control_flow.rs` | 6718 | `control_flow.md` | Large but maps to phase-10 CFG, contracts, diagnostics, and handoff sections. No mandatory split in this task. |
| `tests/determinism_suite.rs` | 627 | `00.crate_plan.md`, task 20 | Cross-module integration test; no boundary issue. |
| `tests/lint_policy.rs` | 1215 | task 1, task 21, task 22, task 31 policies | Policy/audit guard test; the Task-31 exception strips only the exact `ExportStatus`/`Visibility` import in `elaborator.rs` and continues to reject `SymbolEnv`, resolver behavior, aliases, and all other resolver-environment APIs. |

`tests/lint_policy.rs` guards the current public module list, rejects public
nested modules/re-exports in semantic module files until policy guards are
updated, checks public enum policy drift, and checks the Task 22 source/spec
audit inventory. These guards make the public boundary explicit even though the
implementation files remain physically large.

Task 30 rechecked this audit after adding explicit template type-parameter
sethood payloads, Fraenkel cross-reference validation, and Rust fixtures to
`src/elaborator.rs`. The public module boundary and owning spec remain
unchanged; no move-only split is required by the new localized Step 2/Step 3
elaboration behavior.

Task 31 rechecked the boundary after adding the exact Task-180 adapter. The
adapter remains in the owning phase-9 elaborator module, depends on the
checker-owned `ResolvedTypedAst` bundle rather than raw syntax, and exposes
only the specified borrowed function and typed error. The narrow resolver
metadata exception is structurally guarded and does not admit `SymbolEnv` or
name resolution. No source file is moved or added.

## Classification

| ID | Class | Evidence | Action |
|---|---|---|---|
| CORE-BOUNDARY-G001 | `deferred` | `src/elaborator.rs` is the largest implementation file and contains step-specific lowering helpers plus dense task-local tests. | Defer any private extraction to a dedicated move-only task that can split Step 1-6 helper/test sections without changing public APIs or behavior. |
| CORE-BOUNDARY-G002 | `deferred` | `src/control_flow.rs` contains CFG construction, contract/ghost/termination attachment, diagnostics, handoff, and tests in one phase-10 module. | Future move-only task may split private builder/diagnostic/handoff helpers if reviewability bottlenecks emerge. |
| CORE-BOUNDARY-G003 | `deferred` | `src/binder_normalization.rs` contains raw normalization, substitution, closure expansion, canonicalization, and tests in one binder module. | Future move-only task may split private helper sections after closeout if needed. |
| CORE-BOUNDARY-G004 | `external_dependency_gap` | Task 31 closes only the exact Task-180 source-derived CoreIr/snapshot seam. All other source-derived payload families, ControlFlowIr snapshots, and downstream VC/kernel/proof/artifact consumers remain unavailable. | Do not generalize the exact adapter or create placeholder modules. Completed docs-only Task 32 assigns source-derived work to Core Tasks 33-53 and five prepared consumers while preserving downstream gates. |

No `boundary_violation`, module-layout/source-boundary drift,
`source_undocumented_behavior`, `repo_metadata_conflict`, or blocking
`design_drift` is observed. The route-level `source_drift` recorded by Task 32
remains assigned to Core Tasks 33-53. The older
architecture-06 submodule names are already refined by the task-0 plan and the
module specs; this audit does not reopen that historical design drift.

## Split Decision

No files are split in Task 24.

Rationale:

- Public module boundaries already match the module table and owning specs.
- The large implementation files are cohesive around their public module
  responsibilities and are covered by task-local tests.
- The audit finds large review-risk files but no current review-bottleneck
  implementation file that must be split under the TODO rule.
- Moving thousands of lines immediately before closeout would be mechanical
  churn with high review cost and no behavior gain.
- A safe split should be a dedicated move-only follow-up with disjoint path
  ownership, unchanged public APIs, unchanged diagnostics, unchanged debug
  renderings, and full Rust verification.

If a future task performs a split, it must update the module-boundary audit,
rerun the source/spec audit scope for moved APIs, rerun the bilingual
documentation sync scope for any path/document changes, and keep the public
module exports unchanged unless a new spec task explicitly changes them.

## Verification

Because Task 24 is audit-only and does not change Rust source:

- `git diff --check` before staging.
- `git diff --cached --check` after explicit path staging.

If later review requires source movement in this task, run:

- `cargo fmt --check`
- `cargo test -p mizar-core`
- `cargo clippy -p mizar-core --all-targets -- -D warnings`
