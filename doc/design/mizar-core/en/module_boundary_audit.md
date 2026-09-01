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
| `src/core_ir.rs` | 4393 | `core_ir.md` | Large but cohesive data-shape and validation module. IR264 adds one private-field owner sum representation, its validation, compatible debug rendering, and one unit-test matrix without a new module responsibility. No split required. |
| `src/binder_normalization.rs` | 5828 | `binder_normalization.md` | Large but cohesive binder/substitution/canonicalization module. Future private helper extraction is optional. |
| `src/elaborator.rs` | 24788 | `elaborator.md` | Largest review-risk file, but its sections map to the six elaboration steps in the owning spec. Tasks 31, 33C4C8, 33LB, 33I259--264, 33P264, 34I264, IR264, and 35E264 are localized owner-specific adapters/contexts. Task33P264 reuses the existing Task33LB producer for one property-parameter association; IR264 keeps the authenticated property-owner initializer in `core_ir.rs`; Task35E264 retains two existing seed shapes without lowering. No new elaborator responsibility or split is required. |
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

Task 33C4C8 rechecks the boundary after adding one localized Step-1 association
to `elaborator.rs`. It consumes only the immutable checker Task33C capability,
uses existing resolver/session identity types already permitted by the module,
and exposes no raw-syntax, Typed/Resolved installation, semantic lowering, or
downstream route. The existing module remains the owning boundary and no split
is required.

Task33I264 repairs the stale pre-33LB/33I259--263 line count above and adds only
one localized singleton carrier-context section beside the existing Core-33
associations. The public module boundary remains `elaborator`; Task263 is not
generalized, and no production runner or new module responsibility appears.
The final line count is remeasured after implementation.

Task34I264 adds a second localized association context to the same
`elaborator` owner and two assertions to the existing private Task264 leaf. It
does not edit `core_ir.rs`, add a module or Cargo edge, or move raw syntax into
Core. The final source is 23,682 lines; no split is required for this bounded
task.

Task IR264 changes the cohesive `core_ir` owner/validation boundary rather
than introducing a fifth public module or a cross-crate ownership layer. The
public `CoreDefinitionOwner` hides its representation; ordinary item owners
remain caller-constructible while the property form can be minted only by the
authenticated Task34I264 handoff's inherent method in `core_ir.rs`. Validation
therefore remains beside `CoreIr::try_new`, and the elaborator and VC edits are
mechanical ordinary-owner migrations. Final `core_ir.rs` is 4,393 lines and
`elaborator.rs` is 23,685 lines. No move-only split is required for this
bounded prerequisite.

Task34D264 adds one private-field association and accessors beside the existing
Task34I264 validation in `elaborator`; it adds no module, Cargo edge, producer,
error type, or cross-crate responsibility. The same private mizar-test leaf is
extended only with assertions. Post-source line counts are recorded in the task
contract, and no move-only split is required for this bounded prerequisite.

Task33P264 remains another localized Core33 association in `elaborator`. It
reuses the existing 33LB producer over a cloned carrier context and adds no
module, Cargo edge, raw-syntax dependency, destination slot, semantic lowering,
or production route. The private Task264 leaf is the only test consumer.

Task35E264 remains a localized Core35-input adapter in `elaborator`. It retains
existing public handoffs and two existing `CoreTermSeed` values without adding
a module, Cargo edge, CoreIR type, raw-syntax/resolver dependency, generic
lowering owner, production route, or destination slot. The same private
Task264 leaf is the only test consumer; no split is required.

Implemented Task35L264 is the matching localized pure-term adapter. It consumes the
complete Task35E264 capability and owns only an unattached two-row term table,
term-only source map, and local association. It does not generalize the
ordinary-item generic lowerer, install `CoreIr`, or add definition/route
ownership, so no module split or topology change is selected.

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
