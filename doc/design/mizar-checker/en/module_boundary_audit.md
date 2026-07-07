# Module Boundary Audit: mizar-checker

> Canonical language: English. Japanese companion:
> [../ja/module_boundary_audit.md](../ja/module_boundary_audit.md).

Task 34 audits whether the current `mizar-checker` source layout should be
split before downstream crates consume the checker boundary. It is a layout
gate only: it does not change checker source behavior, public APIs,
diagnostics, deterministic renderings, artifact-facing schemas, `.miz`
fixtures, or expectations.

## Split Gate

A behavior-neutral private module split is required only when a checker-owned
file creates a concrete layout/review bottleneck inside an already-owned module
boundary. The following are not layout fixes: crate ownership violations,
undocumented public APIs, behavior drift, API exposure, diagnostic changes, or
schema changes. Those are hard-gate findings under the autonomous crate
protocol and must be fixed, deferred with an owner, or moved to a separate
specification task; task 34 must not hide them behind file moves.

Large but cohesive files are recorded as monitored ergonomics notes when their
public surface, diagnostics, deterministic rendering, and module ownership
remain aligned with their owning specifications.

## Source Layout Inventory

| Path | Lines | Boundary label | Owning specification | Split required | Hard-gate finding | Decision |
|---|---:|---|---|---|---|---|
| `src/lib.rs` | 32 | crate boundary and public module exports | `00.crate_plan.md` and `source_spec_audit.md` | no | no | Keep as the crate root; it only exposes documented modules and test-only determinism support. |
| `src/typed_ast.rs` | 3527 | typed AST data model | `typed_ast.md` | no | no | Large but cohesive typed-AST tables, ids, validation, rendering, and tests; monitor ergonomics after downstream use. |
| `src/binding_env.rs` | 3090 | binding environment and resolver shell boundary | `binding_env.md` | no | no | Cohesive binding/context data layer; no behavior-neutral split required. |
| `src/type_checker.rs` | 10542 | phase-6 type checking over checker-owned payloads | `type_checker.md` | no | no | Largest file but still within the phase-6 spec boundary; normalization, reserve source handoff production, declaration checking, inference, coercions, fact queries, diagnostics, rendering, and tests remain behavior-coupled, so split later only with a focused private-layout task if review friction becomes concrete. |
| `src/registration_resolution.rs` | 5888 | phase-7 registration validation, activation, and existential gates | `registration_resolution.md` | no | no | Cohesive registration data layer and gate logic; no behavior-neutral split required. |
| `src/cluster_trace.rs` | 3948 | cluster closure and reduction trace recording | `cluster_trace.md` | no | no | Cohesive trace/replay module; no behavior-neutral split required. |
| `src/overload_resolution.rs` | 8004 | phase-8 overload pipeline | `overload_resolution.md` | no | no | Large but cohesive overload collection, template expansion, viability, specificity, selection, rendering, and tests; monitor ergonomics after downstream use. |
| `src/resolved_typed_ast.rs` | 3728 | final resolved typed AST assembly | `resolved_typed_ast.md` | no | no | Cohesive final projection module; no behavior-neutral split required. |
| `src/determinism_suite.rs` | 1096 | test-only cross-module determinism suite | `00.crate_plan.md` and `source_spec_audit.md` | no | no | Keep as private `#[cfg(test)]` crate support. |
| `tests/lint_policy.rs` | 1786 | cross-cutting policy and audit guards | `source_spec_audit.md`, `bilingual_sync_audit.md`, and `module_boundary_audit.md` | no | no | Large support test but intentionally centralizes repository-policy guardrails; no split required for task 34. |

## Task 34 Classification

| Class | Evidence | Action |
|---|---|---|
| `spec_gap` | No language specification behavior is changed by this audit. | No spec edit. |
| `test_gap` | The task is a source-layout gate; executable coverage is the lint-policy guard over this audit table and existing source/spec and bilingual guards. | Add no `.miz` fixtures. |
| `design_drift` | The crate plan, TODO, source/spec audit, bilingual audit, and this layout audit are synchronized for the current source files. | Record task 34 completion and guard future audit drift. |
| `source_drift` | Source behavior is unchanged; no file move or private split is required by the current evidence. | No source/API edits beyond the lint-policy test. |
| `source_undocumented_behavior` | Task 32's guard still covers public source/spec correspondence; task 34 finds no new undocumented public API. | Future public surface drift remains a hard gate, not a split trigger. |
| `boundary_violation` | The current public modules remain within the checker ownership boundary described by internal 07 and the module specs. | No boundary repair or deferral. |
| `external_dependency_gap` | None new. Existing checker external gaps remain recorded in the crate plan and source/spec audit. | No new deferral. |
| `deferred` | No required behavior-neutral module split is deferred by task 34. Large cohesive files are monitored ergonomics notes only. | Future split work must be a behavior-neutral private-layout task with its own review and commit. |

## Completion Decision

Task 34 is complete when this English audit and its Japanese companion, the
crate plan and todo updates, the source/spec and bilingual audit updates, and
the lint-policy module-boundary guard are committed together. Task 34 does not
claim crate completion by itself; the closeout task has since recorded the
crate exit report, and the report records the read-only quality review result.
