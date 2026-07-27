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
| `src/lib.rs` | 43 | crate boundary and public module exports | `00.crate_plan.md` and `source_spec_audit.md` | no | no | Keep as the crate root; Task 257B1 exports the documented syntax-free formula-composition module. |
| `src/typed_ast.rs` | 4188 | typed AST data model | `typed_ast.md` | no | no | Large but cohesive typed-AST tables, ids, validation, rendering, tests, bidirectional Task-253/254/255/256 installation checks, Task-257A one-shot installation, Task-257B1/B2/B3 combined installation, and Task-257C2 condition-composition ownership; monitor ergonomics after downstream use. |
| `src/binding_env.rs` | 3143 | binding environment and resolver shell boundary | `binding_env.md` | no | no | Cohesive binding/context data layer, including source-formula context identity; no behavior-neutral split required. |
| `src/source_context.rs` | 1150 | syntax-free source-item and binding-context producer | `source_context.md` | no | no | Cohesive Task-248 validation, table construction, recovery, handoff, and boundary tests; no split required. |
| `src/source_atomic_formula.rs` | 8460 | syntax-free source atomic-formula producer | `source_atomic_formula.md` | no | no | Cohesive Task-256/257C1 nine-table association, resolver provenance, predicate-segment/shared-boundary validation, cross-family ownership/fingerprint validation, deterministic rendering, install checks, and compatibility literals; no split required. |
| `src/source_composite_formula.rs` | 4700 | syntax-free source composite-formula/binder producer | `source_composite_formula.md` | no | no | Cohesive Task-257A/B1/B2/B3 exact profiles, binding extension, wrapper/tree validation, rendering, install checks, and corruption/profile tests; no split required. |
| `src/source_formula_composition.rs` | 4120 | syntax-free cross-family formula composition producer | `source_formula_composition.md` | no | no | Cohesive Task-257B1/B2/B3 atomic-edge/bound-use associations plus the separate Task-257C2 condition-to-atomic transaction, dependency fingerprints, deterministic rendering, installation, and corruption tests; no split required. |
| `src/source_attribute.rs` | 3074 | syntax-free source-attribute producer | `source_attribute.md` | no | no | Cohesive Task-250 flat tables, environment/parent/arena/provenance validation, deterministic rendering, and corruption tests; no split required. |
| `src/source_evidence.rs` | 2413 | syntax-free source-evidence request/reference producer | `source_evidence.md` | no | no | Cohesive Task-251 request/response tables, upstream association, catalog/payload validation, deterministic rendering, and corruption tests; no split required. |
| `src/source_term.rs` | 2207 | syntax-free source primary-term producer | `source_term.md` | no | no | Cohesive Task-252 term/reference/request tables, binding and parent validation, deterministic rendering, and corruption tests; no split required. |
| `src/source_application.rs` | 4001 | syntax-free source functor-application producer | `source_application.md` | no | no | Cohesive Task-253 application/wrapper/candidate/argument/request tables, dependency and provenance validation, deterministic rendering, and corruption tests; no split required. |
| `src/source_set_term.rs` | 6806 | syntax-free source set-term producer | `source_set_term.md` | no | no | Cohesive Task-255/255C1 seven-table association, condition-subtree exclusion, cross-family ownership/fingerprint validation, deterministic rendering, install checks, and corruption tests; no split required. |
| `src/source_structure.rs` | 5036 | syntax-free source structure-term producer | `source_structure.md` | no | no | Cohesive Task-254 term/wrapper/root/member/field-update/edge/request tables, written-partition and cross-family dependency/provenance validation, deterministic rendering, and corruption tests; no split required. |
| `src/source_type.rs` | 3294 | syntax-free source-type application producer | `source_type.md` | no | no | Cohesive Task-249 flat tables, environment/arena/form/graph/provenance validation, deterministic rendering, and exhaustive corruption tests; no split required. |
| `src/type_checker.rs` | 13235 | phase-6 type checking over checker-owned payloads | `type_checker.md` | no | no | Largest file but still within the phase-6 spec boundary; normalization, reserve and authenticated exact theorem-owner handoff validation, declaration checking, inference, coercions, fact queries, diagnostics, rendering, and tests remain behavior-coupled. |
| `src/registration_resolution.rs` | 5888 | phase-7 registration validation, activation, and existential gates | `registration_resolution.md` | no | no | Cohesive registration data layer and gate logic; no behavior-neutral split required. |
| `src/cluster_trace.rs` | 3948 | cluster closure and reduction trace recording | `cluster_trace.md` | no | no | Cohesive trace/replay module; no behavior-neutral split required. |
| `src/overload_resolution.rs` | 8004 | phase-8 overload pipeline | `overload_resolution.md` | no | no | Large but cohesive overload collection, template expansion, viability, specificity, selection, rendering, and tests; monitor ergonomics after downstream use. |
| `src/resolved_typed_ast.rs` | 7004 | final resolved typed AST assembly | `resolved_typed_ast.md` | no | no | Cohesive final projection module, including Task-251/252/253/254/255/256/257A/257B1/B2/B3/C2 clone-preserving handoffs; no behavior-neutral split required. |
| `src/determinism_suite.rs` | 1101 | test-only cross-module determinism suite | `00.crate_plan.md` and `source_spec_audit.md` | no | no | Keep as private `#[cfg(test)]` crate support. |
| `tests/lint_policy.rs` | 1846 | cross-cutting policy and audit guards | `source_spec_audit.md`, `bilingual_sync_audit.md`, and `module_boundary_audit.md` | no | no | Large support test but intentionally centralizes repository-policy guardrails; no split required for task 34. |

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

## Task 266 Current-Layout Addendum

Task 266 remains inside existing checker ownership boundaries and requires no
module split. Resolver-global owner validation stays in `type_checker.rs`;
`resolved_typed_ast.rs` consumes only checker-owned owner, binding, inference,
and typed-AST payloads. The boundary lint forbids `SymbolEnv` and
`mizar_resolve::env` scans in the final projection module and passes.

## Task 250 Current-Layout Addendum

Task 250 adds one cohesive public `source_attribute.rs` owner. Raw syntax
remains in the private `mizar-test` leaf; the checker module accepts only
syntax-free Task-249, binding, symbol, and typed-arena dependencies. The
five-table data model, validation, construction, rendering, and corruption
tests remain behavior-coupled, so no private split is required. `TypedAst`
owns the immutable handoff and `ResolvedTypedAst` remains clone-only.

## Task 251 Current-Layout Addendum

Task 251 adds one cohesive public `source_evidence.rs` owner. It accepts only
syntax-free Task-249/250 handoffs, resolver identities, checker facts/gates,
and dependency records; raw syntax remains in `mizar-test`. Request/response
association, state/cardinality validation, catalog and payload authentication,
deterministic rendering, and the corruption matrix are behavior-coupled, so
no private split is required. `TypedAst` owns the immutable handoff and
`ResolvedTypedAst` remains clone-only.

## Task 253 Current-Layout Addendum

Task 253 adds one cohesive public `source_application.rs` owner. It accepts
only syntax-free resolver, binding, Task-252, and typed-arena inputs; raw
syntax remains in the private `mizar-test` leaf. Five-table association,
application and wrapper geometry, root-only/cross-application ownership,
candidate provenance, unresolved requests, exact dependency fingerprint,
rendering, and corruption tests are behavior-coupled, so no private checker
split is required. `TypedAst` owns the one-shot immutable handoff and
`ResolvedTypedAst` revalidates then clone-preserves it.

## Task 252 Current-Layout Addendum

Task 252 adds one cohesive public `source_term.rs` owner. It accepts only
syntax-free binding and typed-arena inputs plus the canonical
`mizar_lexer::is_identifier` vocabulary predicate; it imports no raw syntax.
Raw `SurfaceAst` traversal remains in one private `mizar-test` leaf.
Term/reference/request association, binding lookup, parent closure, numeric
cardinality, rendering, and corruption tests remain behavior-coupled, so no
private checker split is required. `TypedAst` owns the immutable handoff and
`ResolvedTypedAst` remains clone-only.

## Task 254 Current-Layout Addendum

Task 254 adds one cohesive 5,036-line public `source_structure.rs` owner. It
accepts only syntax-free resolver, binding, Task-252/253, and typed-arena
inputs; raw syntax and the Task-248 source-context extraction remain in the
private `mizar-test` leaf. Seven-table association, constructor provenance,
member/`FieldUpdate` geometry, cross-family root ownership, conditional
fingerprints, rendering, and corruption tests are behavior-coupled, so no
private checker split is required. `TypedAst` owns the one-shot immutable
handoff and `ResolvedTypedAst` revalidates then clone-preserves it.

## Checker Task 257C3 Frozen Boundary Recheck

The planned two-table predicate-chain composition remains in the cohesive
`source_formula_composition.rs` owner and reuses existing Task-252/256 public
handoffs. No source file, path, or measured line count changes in this
documentation prerequisite. The future mizar-test route keeps raw traversal
and resolver selection in `source_formula.rs`, reuses the Task-252/256 lower
builder in `source_atomic_formula.rs`, and owns only complete-route
orchestration in `source_formula_composition.rs`. Typed/resolved checker
modules will own only optional installation and final projection. No split is
required.

## Task 256C1 Frozen Boundary Recheck

Fresh inventory measures `source_atomic_formula.rs` at 7,428 lines,
`source_set_term.rs` at 6,806, `typed_ast.rs` at 4,117, and
`resolved_typed_ast.rs` at 6,950. Task 256C1 changes only the first cohesive
owner: a private range predicate and its checker-local tests remain coupled
to the existing nine-table validation matrix. No new module, public API,
runner owner, or cross-crate dependency is justified. `TypedAst`,
`source_set_term`, resolved ownership, and all mizar-test production paths
remain unchanged.

## Checker Task 257C2 Frozen Boundary

Task 257C2 remains within the existing `source_formula_composition.rs` owner
but freezes a separate condition-formula transaction rather than mixing it
into the Task-257B composite/bound-use tables. Its checker input is limited to
syntax-free Task-252/253/255/256 handoffs and `TypedArena`; raw AST selection,
loaded-source guards, and parser/resolver inspection stay in the private
`mizar-test` leaves. The new association owns no site or semantic result.
Production is unchanged in this prerequisite, so the measured 3,117-line
module and all current boundary-table counts remain unchanged. Implementation
must remeasure before editing and repeat this audit afterward. The frozen
pre-Task-256C1 preflight found a separate condition-container compatibility
`source_drift` in `source_atomic_formula.rs`. Its dedicated documentation and
implementation commits preserved that lower module's ownership and now pass
both lower-handoff installation orders. At this frozen-boundary exit only
fresh Task-257C2 preflight remained before editing this module; the completed
implementation is recorded in the implementation recheck.

## Task 255C1 Current-Layout Addendum

No checker production path was added. `source_set_term.rs` is the cohesive
6,806-line owner of the seven Task-255 tables, recursive condition-subtree
boundary, cross-family partition, fingerprints, rendering, installation, and
their focused matrices. The six compatibility literals leave
`source_atomic_formula.rs` at 7,428 lines. The public module split remains
appropriate; no raw syntax or semantic formula ownership entered the checker.

## Checker Task 257C1 Boundary Recheck

No checker production path was added. The now 7,422-line
`source_atomic_formula.rs` remains the cohesive syntax-free owner of the
extended nine-table transaction. Segment/head/candidate/edge/request
association, polarity-token authentication, shared-boundary validation,
dependency fingerprints, rendering, installation revalidation, and rollback
remain behavior-coupled. Raw parsing and exact-source selection remain
private to `mizar-test`; no checker split is warranted. `TypedAst` and
`ResolvedTypedAst` retain the existing publication boundary.

## Checker Task 257B3 Boundary Recheck

No checker production path was added. The fourth profile remains inside the
cohesive composite/composition owners; exact parser/resolver extraction stays
private to `mizar-test`. Two debug-oracle text assets are test-only. The
bounded extension does not justify a new module split.

## Task 257B2 Boundary Delta

No checker module was added. `source_composite_formula` now owns the five
frozen connective kinds, four same-family roles, six-wrapper validation, and
third profile; `source_formula_composition` owns the four atomic roles and
exact `8/0` cross-family table. `TypedAst` and `ResolvedTypedAst` keep the
existing combined publication boundary. Runner extraction stays in the
existing private formula-composition leaf, so no new split is justified.

## Task 257B1 Current-Layout Addendum

Task 257B1 adds one cohesive public `source_formula_composition.rs` owner and
a bounded second-profile extension to `source_composite_formula.rs`. Both
accept syntax-free Task-252/256/257 and typed-arena dependencies only; raw
formula syntax remains in the private `mizar-test` leaf. Atomic-edge and
bound-use association, dependency fingerprints, combined installation,
deterministic rendering, and corruption tests remain behavior-coupled, so no
private checker split is required.

Final line counts are `lib.rs` 43, `typed_ast.rs` 4,110,
`source_composite_formula.rs` 2,913, `source_formula_composition.rs` 1,475,
`resolved_typed_ast.rs` 6,949, and `tests/lint_policy.rs` 1,846.

## Task 257A Current-Layout Addendum

Task 257A adds one cohesive 2,790-line public
`source_composite_formula.rs` owner. It accepts only syntax-free binding and
typed-arena inputs; raw formula syntax remains in the private `mizar-test`
leaf. Seven-table association, source-derived `2/1/4` binding extension,
tree/context/binder/type validation, installation revalidation,
deterministic rendering, and the real/synthetic/corruption/exclusion matrix
remain behavior-coupled, so no private checker split is required. `TypedAst`
owns the one-shot immutable handoff and `ResolvedTypedAst` revalidates then
clone-preserves it.

## Task 255 Current-Layout Addendum

Task 255 adds one cohesive 5,547-line public `source_set_term.rs` owner. It
accepts only syntax-free binding, Task-252/253/254, and typed-arena inputs;
raw syntax remains in the private `mizar-test` leaf. Six-table association,
canonical spelling/cardinality, nearest-family ownership, conditional
fingerprints, installation revalidation, rendering, and the corruption
matrix remain behavior-coupled, so no private checker split is required.
`TypedAst` owns the one-shot immutable handoff and `ResolvedTypedAst`
revalidates then clone-preserves it.

## Task 257C2 Implementation Boundary Recheck

Task 257C2 extends the existing cohesive
`source_formula_composition.rs` owner to 4,120 lines with a separate
condition-to-atomic transaction and its three compound checker tests.
`typed_ast.rs` is 4,188 lines and `resolved_typed_ast.rs` is 7,004 lines
after adding the optional one-shot/final-clone ownership; the lower
`source_atomic_formula.rs` remains 8,460 lines. The checker still accepts only
syntax-free Task-252/253/255/256 handoffs plus `TypedArena`; raw parsing and
resolver traversal remain in `mizar-test`. No new module or dependency
direction is justified.

The checker library has 332 tests with raw/normalized test-list hashes
`67be737fdd647f6b316b4b42d40c1270aaacb0db849061906672b7f0d7aaf063` /
`422abe080fdf03a9af096bef22429e74bdbe49fbb8b24d477eba58e577b58f0e`.

## Task 256C1 Implementation Boundary Recheck

Task 256C1 changes only the cohesive private validation path inside
`source_atomic_formula.rs`, now 8,460 lines including its exact three-test
matrix. `source_set_term.rs` remains 6,806 lines, `typed_ast.rs` remains
4,117, and `resolved_typed_ast.rs` remains 6,950. No module, public schema,
runner owner, or dependency direction changed, so no split or boundary move
is warranted.

## Task 256 Current-Layout Addendum

Task 256 adds one cohesive 6,414-line public `source_atomic_formula.rs`
owner. It accepts only syntax-free resolver, binding, Task-252/253/254/255,
and typed-arena inputs; raw formula syntax remains in the private
`mizar-test` leaf. Eight-table association, predicate and attribute
provenance, bare asserted-type ownership, nearest-family cross-family edges,
conditional fingerprints, installation revalidation, rendering, and the
real/synthetic/corruption/exclusion matrix remain behavior-coupled, so no
private checker split is required. `TypedAst` owns the one-shot immutable
handoff and `ResolvedTypedAst` revalidates then clone-preserves it.
