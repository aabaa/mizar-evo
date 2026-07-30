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
| `src/lib.rs` | 44 | crate boundary and public module exports | `00.crate_plan.md` and `source_spec_audit.md` | no | no | Keep as the crate root; it exports the documented syntax-free formula-composition and source-statement modules. |
| `src/typed_ast.rs` | 5005 | typed AST data model | `typed_ast.md` | no | no | Large but cohesive typed-AST tables, ids, validation, rendering, tests, Task-253/254/255/256/257 installation checks, and mutually exclusive Task-248/258A/258B1/258B2/258B3/258B3N/258B3M1/258B3M2A/258B3M2B1/258B3M2B2A/258B3M2B2B1A/258B3M2B2B1B1/258B3M2B2B2A/258B3M2B2B2B/258B3M2B2B2C/258B3M2B2B3A/258B3M2B2B3B/258B3M2B2B3C/258B3M2B2B3D/258B3M2B2B3E/258B4A/258B4B/258B4C ownership; monitor ergonomics after downstream use. |
| `src/binding_env.rs` | 3156 | binding environment and resolver shell boundary | `binding_env.md` | no | no | Cohesive binding/context data layer, including source-formula, Task-258B1 statement-context identity, and the unchanged context contract reused by Task-258B2; no behavior-neutral split required. |
| `src/source_context.rs` | 1150 | syntax-free source-item and binding-context producer | `source_context.md` | no | no | Cohesive Task-248 validation, table construction, recovery, handoff, and boundary tests; no split required. |
| `src/source_atomic_formula.rs` | 8511 | syntax-free source atomic-formula producer | `source_atomic_formula.md` | no | no | Cohesive Task-256/257C1 nine-table association, resolver provenance, predicate-segment/shared-boundary validation, cross-family ownership/fingerprint validation, deterministic rendering, install checks, compatibility literals, and test-only dependency corruption seams; no split required. |
| `src/source_composite_formula.rs` | 4700 | syntax-free source composite-formula/binder producer | `source_composite_formula.md` | no | no | Cohesive Task-257A/B1/B2/B3 exact profiles, binding extension, wrapper/tree validation, rendering, install checks, and corruption/profile tests; no split required. |
| `src/source_formula_composition.rs` | 5366 | syntax-free cross-family formula composition producer | `source_formula_composition.md` | no | no | Cohesive Task-257B1/B2/B3 atomic-edge/bound-use associations plus separate Task-257C2 condition-to-atomic and Task-257C3 predicate-chain transactions, dependency fingerprints, deterministic rendering, installation, and corruption tests; no split required. |
| `src/source_attribute.rs` | 3074 | syntax-free source-attribute producer | `source_attribute.md` | no | no | Cohesive Task-250 flat tables, environment/parent/arena/provenance validation, deterministic rendering, and corruption tests; no split required. |
| `src/source_evidence.rs` | 2413 | syntax-free source-evidence request/reference producer | `source_evidence.md` | no | no | Cohesive Task-251 request/response tables, upstream association, catalog/payload validation, deterministic rendering, and corruption tests; no split required. |
| `src/source_term.rs` | 2218 | syntax-free source primary-term producer | `source_term.md` | no | no | Cohesive Task-252 term/reference/request tables, binding and parent validation, deterministic rendering, and corruption tests including Task-258A dependency revalidation; no split required. |
| `src/source_application.rs` | 4001 | syntax-free source functor-application producer | `source_application.md` | no | no | Cohesive Task-253 application/wrapper/candidate/argument/request tables, dependency and provenance validation, deterministic rendering, and corruption tests; no split required. |
| `src/source_set_term.rs` | 6806 | syntax-free source set-term producer | `source_set_term.md` | no | no | Cohesive Task-255/255C1 seven-table association, condition-subtree exclusion, cross-family ownership/fingerprint validation, deterministic rendering, install checks, and corruption tests; no split required. |
| `src/source_statement.rs` | 47593 | syntax-free source statement producer | `source_statement.md` | no | no | Cohesive Task-258A/258B1/258B2/258B3/258B3N/258B3M1/258B3M2A/258B3M2B1/258B3M2B2A/258B3M2B2B1A/258B3M2B2B1B1/258B3M2B2B2A/258B3M2B2B2B/258B3M2B2B2C/258B3M2B2B3A/258B3M2B2B3B/258B3M2B2B3C/258B3M2B2B3D/258B3M2B2B3E/258B4A/258B4B/258B4C statement and witness transactions, resolver/binding/lower/application/structure/set/formula provenance, zero-edge/qua/comprehension/composite-root ownership, subtree validation, rendering, paired typed/final installation, and corruption matrices; no split required. |
| `src/source_structure.rs` | 5036 | syntax-free source structure-term producer | `source_structure.md` | no | no | Cohesive Task-254 term/wrapper/root/member/field-update/edge/request tables, written-partition and cross-family dependency/provenance validation, deterministic rendering, and corruption tests; no split required. |
| `src/source_type.rs` | 3294 | syntax-free source-type application producer | `source_type.md` | no | no | Cohesive Task-249 flat tables, environment/arena/form/graph/provenance validation, deterministic rendering, and exhaustive corruption tests; no split required. |
| `src/type_checker.rs` | 13235 | phase-6 type checking over checker-owned payloads | `type_checker.md` | no | no | Largest file but still within the phase-6 spec boundary; normalization, reserve and authenticated exact theorem-owner handoff validation, declaration checking, inference, coercions, fact queries, diagnostics, rendering, and tests remain behavior-coupled. |
| `src/registration_resolution.rs` | 5888 | phase-7 registration validation, activation, and existential gates | `registration_resolution.md` | no | no | Cohesive registration data layer and gate logic; no behavior-neutral split required. |
| `src/cluster_trace.rs` | 3948 | cluster closure and reduction trace recording | `cluster_trace.md` | no | no | Cohesive trace/replay module; no behavior-neutral split required. |
| `src/overload_resolution.rs` | 8004 | phase-8 overload pipeline | `overload_resolution.md` | no | no | Large but cohesive overload collection, template expansion, viability, specificity, selection, rendering, and tests; monitor ergonomics after downstream use. |
| `src/resolved_typed_ast.rs` | 7353 | final resolved typed AST assembly | `resolved_typed_ast.md` | no | no | Cohesive final projection module, including Task-251/252/253/254/255/256/257/258A/258B1/258B2/258B3/258B3N/258B3M1/258B3M2A/258B3M2B1/258B3M2B2A/258B3M2B2B1A/258B3M2B2B1B1/258B3M2B2B2A/258B3M2B2B2B/258B3M2B2B2C/258B3M2B2B3A/258B3M2B2B3B/258B3M2B2B3C/258B3M2B2B3D/258B3M2B2B3E/258B4A/258B4B/258B4C clone-preserving handoffs and semantic coexistence guards; no behavior-neutral split required. |
| `src/determinism_suite.rs` | 1101 | test-only cross-module determinism suite | `00.crate_plan.md` and `source_spec_audit.md` | no | no | Keep as private `#[cfg(test)]` crate support. |
| `tests/lint_policy.rs` | 1877 | cross-cutting policy and audit guards | `source_spec_audit.md`, `bilingual_sync_audit.md`, and `module_boundary_audit.md` | no | no | Large support test but intentionally centralizes repository-policy guardrails, including Task-258A/258B1/258B2 public-surface and test-only syntax dependency policy; no split required for task 34. |

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

## Task 258B3N Boundary Result

Implementation stayed within the planned owners:
`source_statement.rs` owns the table, producer, validation, rendering, and
four checker tests; `typed_ast.rs` and `resolved_typed_ast.rs` own paired
publication/revalidation; the existing mizar-test statement leaf and
facades own the dormant consumer and five runner tests. No module,
dependency direction, binding owner, or semantic owner was introduced.

## Task 258B3N Planned Boundary

The named-witness extension remains cohesive in `source_statement.rs`: one
dense name table, B3/B3N profile validation, shared-arena authentication,
paired typed/final ownership, and the four-test matrix. The runner consumer
stays in the existing statement leaf with five tests. No module split,
dependency-direction change, or semantic owner is authorized.

## Task 258B3 Frozen Boundary Result

The future witness transaction stays in the existing
`source_statement.rs` owner beside the base/reference transactions.
`typed_ast.rs` owns atomic paired installation and
`resolved_typed_ast.rs` owns final revalidation/clone. `binding_env.rs`
remains unchanged. Raw `SurfaceNodeKind`, source hashing, parser/resolver
selection, and all-index parity remain private runner responsibilities; no
normal checker dependency on `mizar-syntax` is permitted.

The one-row companion, two fingerprints, `[0,1,2]` cross-table order, and
take/witness containment are behavior-coupled with base validation, so no new
checker module is justified. This docs-only prerequisite changes neither
module topology nor the measured `7334/4550/7172/3156` line baseline.

## Task 258B2 Implemented-Boundary Addendum

No module or dependency edge was added. Final affected sizes are
`source_statement.rs` 7,334 lines, `typed_ast.rs` 4,550, and
`resolved_typed_ast.rs` 7,172; `binding_env.rs` remains unchanged at 3,156.
Raw parser/resolver inspection remains in the private runner leaf, while the
checker receives only Task-48/252/256 syntax-free handoffs and exact resolver
provenance. This preserves the existing ownership split and introduces no
semantic owner.

## Checker Task 258A Implementation Boundary Recheck

Task 258A adds one cohesive 2,840-line public `source_statement.rs` owner.
Its syntax-free five-table transaction, resolver-provenance authentication,
owned binding environment, dependency fingerprints, arena/subtree
validation, deterministic rendering, and corruption matrix are
behavior-coupled and do not justify a split. Raw parser/resolver selection
stays in the private runner leaf. `typed_ast.rs` is 4,378 lines and
`resolved_typed_ast.rs` 7,146; they retain only the established
one-shot/final-clone publication boundary. The 2,218-line `source_term.rs`
and 8,511-line `source_atomic_formula.rs` changes are test-only corruption
seams for direct dependency revalidation.

The checker library has 338 tests with raw/normalized hashes
`6a534979eea0c1323bf5b5d6de2a0c2f397e9b574cef70774ca50a80a3833330` /
`405dbb1098c0ffa329fa2a16c55e4beb6737cb442637e8c44731c16acdb4327b`.
No dependency direction or owner crate changed.

## Checker Task 258A Frozen-Contract Boundary

This documentation prerequisite adds no checker source path, public module,
dependency direction, or line-count change. The future
`source_statement.rs` is frozen as one cohesive syntax-free owner for the
five theorem-owner/statement/context/input/candidate tables, dependency
fingerprints, its exact owned BindingEnv, asymmetric production plus named
test-only Task-248 exclusion,
transaction validation, and deterministic rendering. Raw
statement selection remains in a private `mizar-test` leaf; truth, proof,
acceptance, and publication remain in later semantic owners. No pre-
implementation module split is justified, and the current source-layout
inventory remains unchanged.

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

## Task 258B2 Planned-Boundary Addendum

Task 258B2 adds no module during this documentation prerequisite and does not
change the current line-count table. The planned implementation extends the
existing cohesive `source_statement.rs` owner with one assumption source
kind and one exact base-only profile. `binding_env.rs`, `typed_ast.rs`, and
`resolved_typed_ast.rs` retain their existing validation and publication
roles; raw parser/resolver syntax remains in the private runner leaf. No
semantic-stage owner or dependency direction changes, so no split or
boundary move is warranted before implementation.

## Checker Task 257C3 Implementation Boundary Recheck

No checker module was added. `source_formula_composition.rs` is 5,317 lines
and remains the cohesive owner of three independent syntax-free composition
transactions. `typed_ast.rs` is 4,280 lines,
`source_atomic_formula.rs` 8,506, and `resolved_typed_ast.rs` 7,050; the
atomic change is test-only fixture support and the typed test-only occupancy
seams directly exercise reciprocal guards. Raw extraction and resolver
selection remain private to `mizar-test`, so no dependency direction or
split is justified.

The checker library has 335 tests with raw/normalized test-list hashes
`de92623800741813a88a2521eaaa99a757f4fccb7d7be4a025e4108c8660e1e0` /
`7bfae9a1d5f8ec503232a6c68f324cdee0cba65e1b422c563aea9f9951affa64`.

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

## Task 258B3M1 Boundary Addendum

The exact multiple-witness profile remains cohesive in the existing
`source_statement.rs`, `typed_ast.rs`, and `resolved_typed_ast.rs` owners.
It consumes only syntax-free Task-48/252/256/base handoffs plus
`TypedArena`; raw parser/resolver traversal stays in the existing private
`mizar-test` statement leaf. No new module, crate edge, public schema,
semantic owner, or dependency direction is authorized. Current sizes remain
`12114/4644/7200/3156` for this documentation prerequisite.

## Task 258B3M1 Implementation Boundary

The implementation changes only the existing checker statement producer,
typed/final consumers, dormant runner statement leaf/facades, and their
compound tests. No module, crate edge, public schema, active route,
semantic owner, or dependency direction is added. Checker module sizes are
`14045/4659/7201/3156`; runner statement leaf/facade/root/test sizes are
`3724/688/2501/7246`, within the documented statement-leaf exception.

## Task 258B3M2A Planned Boundary

The exact numeral-witness profile remains cohesive in existing
`source_statement.rs`, `typed_ast.rs`, and `resolved_typed_ast.rs`. It
consumes syntax-free Task-48/252/256/base handoffs and `TypedArena`; raw
parser/resolver traversal stays in the private `mizar-test` statement leaf.
The runner selector and future tests stay in their existing statement
production/test leaves and facades. No new module, crate edge, public
schema, active route, semantic owner, dependency direction, or module split
is authorized. This documentation prerequisite preserves checker sizes
`14045/4659/7201/3156`, runner sizes `3724/688/2501/7246`, and 30
production paths / 38,103 lines.

## Task 258B3M2A Implementation Boundary

The implementation stays in the planned statement producer, typed/final
consumers, dormant runner statement leaf/facades/root, and paired test leaf.
No module, crate edge, public schema, active route, semantic owner, or
dependency direction was added. Checker sizes are
`15746/4660/7202/3156`; runner statement leaf/facade/root/test sizes are
`4185/691/2505/8611`, with 30 production paths / 38,571 lines. The enlarged
private statement leaves remain cohesive under the documented exception; no
behavior-neutral split is warranted.

## Task 258B3M2B1 Planned Boundary

The documentation prerequisite changes no module. Future work is confined
to the existing checker statement producer/typed/final consumers and the
runner statement leaf/facades/root/test leaf. Task-252 retains the
parenthesized wrapper/child, Task-256 retains only the two equality pairs,
and Task-258 privately maps five roots to six primary rows. Raw source,
parser, and resolver authentication stays in the private runner statement
leaf; the checker consumes only syntax-free Task-48/252/256/base handoffs
and `TypedArena`. No crate edge, public schema, active route, semantic
owner, dependency direction, or behavior-neutral split is authorized.
Baselines remain checker `15746/4660/7202/3156`, runner
`4185/691/2505/8611`, and 30 paths / 38,571 lines.

## Task 258B3M2B1 Implementation Boundary

The implementation stays in the planned checker statement producer,
typed/final consumers, dormant runner statement leaf/facades/root, and paired
test leaf. Raw parser/resolver objects remain runner-private and only
syntax-free authenticated handoffs cross to the checker. Checker sizes are
`17569/4661/7203/3156`; runner statement leaf/facade/root/test sizes are
`4676/695/2508/9902`, with 30 production paths / 39,069 lines. No module,
crate edge, public schema, active route, semantic owner, or dependency
direction was added; no behavior-neutral split is warranted.

## Task 258B3M2B2A Frozen Module Boundary

The planned implementation remains in the existing checker statement
producer, typed/final consumers, dormant runner statement leaf/facades/root,
and paired test leaf. Raw parser/resolver state remains runner-private;
only Task-48/252/256/base/witness syntax-free handoffs cross the crate
boundary. This docs-only prerequisite leaves measured sizes
`17569/4661/7203/3156` and `4676/695/2508/9902`, 30 production paths /
39,069 lines, module layout, crate edges, public schema, active routes,
semantic owners, and dependency direction unchanged. No split is warranted.

## Task 258B3M2B2A Implementation Boundary

Implementation stays inside the existing statement producer, typed/final
installers, runner statement leaf/facades/root, and statement test leaf. The
measured checker sizes are `19571/4662/7204/3156`; runner statement
leaf/facade/root/test sizes are `5188/699/2513/11234`. The unchanged
30-path production manifest now totals 39,590 lines with path/content hashes
`98f3b264a59fed5b08c3e8f20e7ca58ff54efaa154eab16a7572a69ce923f275` /
`291da8a26e90f75e7f54e221314c1fcb9ebba375c238a07b02a161f7af6dfe66`.
No module split, crate edge, public schema, active route, semantic owner, or
dependency direction changed.

## Task 258B3M2B2B1A Implementation Boundary

The implementation remains inside the existing checker
`source_statement.rs`, `typed_ast.rs`, and `resolved_typed_ast.rs` owners and
the private runner statement leaf/facades/root/test leaf. Raw
parser/resolver data never crosses the crate boundary; only authenticated
Task-48/252/253/256/base/witness handoffs do. The additive public checker
surface is limited to the `Application` witness target, its optional
B1A-only fingerprint, the application-aware producer entry point, and the
atomic three-handoff typed installer.

Measured checker sizes are `21664/4742/7224/3156`; runner statement
leaf/facade/root/test sizes are `5618/706/2520/11945`. The unchanged
30-path production manifest totals 40,298 lines with path/content hashes
`98f3b264a59fed5b08c3e8f20e7ca58ff54efaa154eab16a7572a69ce923f275` /
`201868442e6a9b6c20188a9f4ed9a65698d12a595cfef1ddd082071b9f090b41`.
No module split, crate edge, active route, fixture, trace status/count,
semantic owner, or dependency direction changed.

## Task 258B3M2B2B1B1P Frozen Boundary

B1B1P changes no checker module or public API. Its later implementation is
runner-private inside the existing Task-253 source-application leaf and two
runner tests. The checker remains the syntax-free consumer of the unchanged
Task-252/253 public handoffs. No statement, witness, typed/final installer,
semantic/proof/goal owner, crate edge, or dependency direction is authorized.
Baseline checker modules remain `21664/4742/7224/3156`.

## Task 258B3M2B2B1B1P Implemented Boundary

Checker modules and public APIs remain byte-for-byte outside the
implementation diff. The runner implementation stays in the existing private
source-application leaf, its private facade/root imports, and paired test
leaf; no module or production path was added. Runner sizes are
`2652/708/2523/3727`, and the 30-path production manifest totals 41,173
lines with path/content hashes
`98f3b264a59fed5b08c3e8f20e7ca58ff54efaa154eab16a7572a69ce923f275` /
`ec189d8b9cf1004ae720be75b33365d2348897e34f780fa202f9f3d03a336f66`.
No statement extraction, checker dependency, public/active route, binding,
semantic owner, crate edge, or dependency direction changed.

## Task 258B3M2B2B1B1 Frozen Boundary

B1B1 requires a new explicit crate-private profile in
`src/source_statement.rs` and corresponding existing-installer/final
revalidation enumeration in `src/typed_ast.rs` and
`src/resolved_typed_ast.rs`. It adds no public checker API and no new
module. The public `Application` witness target, optional fingerprint,
application-aware producer, and atomic installer are reused unchanged.

The runner consumer remains in the existing source-statement leaf and calls
the completed private source-application wrapped seam. Facade/root imports
change only as required by that dormant route. `lib.rs`, Task-253 public
schema, fixtures, expectations, sidecars, trace metadata, and active
dispatch are outside scope. Current checker sizes `21664/4742/7224/3156` and
runner statement/application/facade/root/test sizes
`5618/2652/708/2523/11945/3727` are documentation baselines, not targets.
The checker source manifest remains 23 paths / 115,631 lines with hashes
`c2eea2db9187c48dd830a010eff37f09b90467f9012a9fe6b3ac669b6d1dac42` /
`0d79034477a92c850563478abda36df1e50c951a447f79fca886830ade8acce0`.

## Task 258B3M2B2B1B1 Implemented Boundary

Implementation remained in the frozen private modules. Checker module sizes
are `24236/4743/7225/4001`; the 23-path checker manifest is 118,205 lines
with unchanged path hash and content hash
`a4656745edbba7e9b8c382c4d67ac691484d6a067e2b7a0f0f7b5d7a7fc5996e`.
No module, dependency, public API, active route, fixture, trace, or semantic
owner crossed the boundary.

## Task 258B3M2B2B2P Implemented Boundary

Implementation is confined to the existing private source-structure leaf,
its private facade/root imports, and paired source-structure test leaf. It
adds no module, production path, checker dependency, public re-export,
statement consumer, active route, fixture, expectation, sidecar, trace
owner, or semantic dependency. Checker/runner libraries are `378/425`.

Runner source-structure leaf/facade/root/test sizes are
`2857/715/2531/2991`. Production remains 30 paths and now totals 42,686
lines with path/content hashes
`98f3b264a59fed5b08c3e8f20e7ca58ff54efaa154eab16a7572a69ce923f275` /
`d15292becaa5aac33c23a559aff7085ee8cb9116e44a034b80148a7d65acb155`;
raw/normalized runner-test hashes are
`b78230532c45f58ba96e70810d9613d96098ab0ec975a7317c7d6d0a548956ab` /
`97e68290a6b5a3e81373084293461eda85ab0c508d7ce3002e988ebf27806c38`.
B2A statement/witness documentation remains the next separate logical task.

## Task 258B3M2B2B2A Frozen Module Boundary

Future checker writes are restricted to existing `source_statement.rs`,
`typed_ast.rs`, and `resolved_typed_ast.rs`; runner writes stay in the
existing source-statement leaf, private facades/root, and statement test
leaf. The completed B2P source-structure seam is reused unchanged; a
source-structure edit is permitted only if required to remove its B2P
dead-code expectation when the existing private seam becomes live. Its
visibility, extractor, and row construction remain unchanged. No module,
crate dependency, production path, active route, fixture, sidecar,
expectation, trace owner, or semantic dependency is added.

The only public surface growth is the `Structure(SourceStructureTermId)`
target, optional structure fingerprint accessor, exact structure-aware
producer entry point, and atomic TypedAst installer. ResolvedTypedAst needs
no new accessor. Raw parser/resolver inputs remain runner-private; only
syntax-free Task-48/252/254/256/258 handoffs cross the boundary.

## Task 258B3M2B2B2A Implemented Module Boundary

Implementation stayed in the frozen checker statement, typed, and final
owners and the existing runner statement leaf/facades/tests; the B2P
source-structure seam only lost its obsolete private dead-code allowance.
No module, dependency, production path, active route, fixture, expectation,
sidecar, trace owner, or semantic owner was added.

The checker modules are statement/typed/final/structure
`27194/4829/7241/5036`. The 23-path, 121,265-line checker manifest retains
path hash
`c2eea2db9187c48dd830a010eff37f09b90467f9012a9fe6b3ac669b6d1dac42`
and has content hash
`d4683b1df3c2ef9d69e382bf4cad35d3d434f337d16887086eed88d2a9b8d8f3`.
The additive API and atomic typed/final behavior are implemented without
broadening B2B/B2C or any semantic boundary.

## Task 258B3M2B2B2BP Private Runner Boundary Completion

B2BP changes no checker file, public API, module path, dependency, or
production manifest. The implementation is confined to the existing
`mizar-test` source-structure leaf, its private facade/root test visibility,
and its structure test leaf. The future B2B statement consumer remains a
separate task; no statement, Task-258, semantic, proof, goal, Core, CFG, or
VC owner moves into B2BP.

## Task 258B3M2B2B2B Frozen Module Boundary

B2B adds no public API or module. Checker implementation is restricted to
private exact-profile additions in existing `source_statement.rs`,
`typed_ast.rs`, and `resolved_typed_ast.rs`. Runner implementation is
restricted to the existing private source-statement leaf, its facade/root
registration, and statement test leaf. The completed B2BP source-structure
logic is immutable; only obsolete future-consumer `dead_code` allowances
may be removed when the consumer becomes live.

No checker `source_structure` change, new builder/installer/accessor/table,
dependency, production path, active route, fixture, sidecar, expectation,
trace owner, or semantic owner is authorized. The public `Structure`
witness target, structure fingerprint, structure-aware producer, and
atomic installer are reused unchanged.

Current checker statement/typed/final/structure sizes are
`27194/4829/7241/5036`; its 23-path / 121,265-line manifest and hashes are
unchanged. Runner statement/structure/facade/root/statement-test/
structure-test sizes are `6414/4514/722/2538/15058/4315`; its 30-path /
44,809-line manifest and hashes are unchanged. Documentation baseline tests
are `382/432`; implementation projects `386/437`.

## Task 258B3M2B2B2B Implemented Module Boundary

Implementation stays in the frozen eight files: checker
`source_statement.rs`, `typed_ast.rs`, and `resolved_typed_ast.rs`; runner
source-statement leaf, private facade/root registration, statement test
leaf, and the B2BP source-structure allowance-cleanup file. B2B consumes
only the pre-existing private selector owned-kind and proof-context handoff
seams. The B2BP extractor, Task-254 construction, and public surface remain
unchanged.

No module, dependency, public API, semantic/proof/goal owner, corpus active
route, fixture, expectation, sidecar, or trace owner/credit was added.
Checker statement/typed/final/structure sizes are
`29941/4830/7244/5036`; its 23-path / 124,016-line manifest keeps path hash
`c2eea2db9187c48dd830a010eff37f09b90467f9012a9fe6b3ac669b6d1dac42`
and has content hash
`df0c806d8adf6283b2ac3341e11bab62a0f11ef216d48729852e98c40079d7d1`.
Libraries are `386/437`.

## Task 258B3M2B2B2C Frozen Module Boundary

B2CP commit `b146f0f72dceac2233c9d679b7820e264974b227` is complete.
B2C adds no module, dependency, production path, table, ID, schema, error,
accessor, builder, installer, or public re-export. It reuses unchanged
`SourceStatementWitnessTermTarget::Structure`, the structure fingerprint,
the structure-aware witness producer, the combined TypedAst installer, and
ResolvedTypedAst final-clone validation.

Implementation is restricted to eight existing files: checker
`source_statement.rs`, `typed_ast.rs`, and `resolved_typed_ast.rs`; runner
source-statement leaf, source-structure allowance cleanup, private
`type_elaboration.rs` facade, `runner.rs` test registration, and the
statement test leaf. The B2CP update extractor/producer and owned-kind logic
are consumed unchanged through their private seam. No new active root
dispatch, fixture, expectation, sidecar, trace, semantic/proof/goal, Core,
CFG, or VC owner is authorized.

Documentation baseline is checker/runner `386/439`; implementation projects
`390/444`. Checker sizes remain `29941/4830/7244/5036` and 23 paths /
124,016 lines with hashes `c2ee...` / `df0c...`. Runner sizes remain
`6826/6065/730/2546/17120/5848` and 30 paths / 46,788 lines with hashes
`98f3...` / `bbcc...`. All four independent reviews have no findings and
complete documentation/count/hash verification passes. Independent final
quality has no findings, all nine hard gates PASS, and the valid score is
`98/100`. The commit and fresh implementation inventory remain open.

## Task 258B3M2B2B2C Implemented Module Boundary

Prerequisite commit `d6076cc757ce675d1b46a720b4f00805923d3c70` and its
fresh inventory are complete. The source transaction changes exactly the
three checker and five runner files frozen above. Checker changes are private
B2C profile validation plus reuse of the existing atomic typed/final
structure-statement APIs. Runner changes are the exact private extractor and
route, consumption of the unchanged B2CP update site/owned-kind/handoff seam,
obsolete allowance cleanup, and test-only facade/root registration. Formatter
recovery was audited and left no unrelated semantic churn.

No crate, dependency, production path, public API/re-export, active corpus
case, fixture, expectation, sidecar, trace, diagnostic-credit, or semantic
owner changed. Libraries are `390/444`; checker is 23 paths / 126,115 lines
with sizes `32036/4832/7246/5036`, while runner is 30 paths / 47,203 lines
with sizes `7240/6055/735/2552/19275/5848`. The four checker and five runner
tests pass, and final test-sufficiency and implementation reviews have no
findings. Broad workspace verification and the remaining final reviews and
commit gates are pending.

## Task 258B3M2B2B2C Broad Boundary Verification

Format, workspace Clippy, both crate suites and policy suites, full workspace
tests, focused `4/4` and `5/5`, and sibling `12/12` and `21/21` suites pass.
Fresh sizes, manifests, and hashes match the implemented boundary above, so
no module, path, dependency, public, active-route, or semantic boundary
changes are required. Independent final consistency/quality review and the
commit/post-commit gates remain pending.

## Task 258B3M2B2B2C Final Boundary Review Status

Independent final source/documentation consistency and final quality both
report **NO FINDINGS**. All nine hard gates PASS and the valid score is
`98/100`; the exact boundary evidence remains unchanged. Only cached-diff/
staging audit, implementation commit, and post-commit inventory/fresh-next-
task gates remain pending.

## Task 258B3M2B2B3A Frozen Module Boundary

The later implementation changes exactly seven existing files: checker
`source_statement.rs`, `typed_ast.rs`, `resolved_typed_ast.rs`; runner
`type_elaboration/source_statement.rs`, `type_elaboration.rs`, `runner.rs`,
and `runner/tests/type_elaboration/source_statement.rs`. No new file,
module, dependency, re-export, active route, fixture, or corpus artifact is
allowed. Both `source_set_term.rs` files and all other source/tests are
forbidden; the runner consumes unchanged B3P
`source_set_term_output_with_source_term_in_context`.

The only checker boundary is additive `SetTerm(SourceSetTermId)`, optional
set fingerprint/getter, `build_with_set_term`, set-aware `TypedAst`
installer, and exact `ResolvedTypedAst` allow/revalidate/clone. The tuple is
application/structure `None`, set `Some`; legacy API/debug stays literal.
This prerequisite owns exactly `32` design docs and no executable artifact.
Independent documentation/boundary and source/documentation consistency
repeats report **NO FINDINGS**; final quality reports **NO FINDINGS**, all
nine hard gates PASS, valid `98/100`. Only the documentation-only commit and
post-commit/fresh implementation inventory remain.

## Task 258B3M2B2B3A Implementation Boundary Closure

Implementation changes only the frozen three checker and four runner files.
Both set-term source owners, module topology, visibility, dependencies,
public routes, authority artifacts, and semantic ownership remain unchanged.
The implementation review and targeted boundary checks pass with
**NO FINDINGS**. The second source/documentation consistency repeat and
final documentation/boundary reread also report **NO FINDINGS**; parent
final verification listed in the crate plans passes, including exact
`39`-file scope. Independent final read-only quality review reports
**NO FINDINGS**. All nine hard gates PASS with no score cap; the valid score
is `98/100` (`20/20/15/14/10/10/5/4`). The stated semantic and coverage
deferrals remain unchanged as residual risk. Only the dedicated
implementation commit, post-commit invariant verification, and fresh
next-task inventory remain pending.

## Task 258B3M2B2B3B Boundary Audit

B3B is an upper statement-consumer profile, not a new Task-255 producer.
It may edit only the three checker statement/typed/final owners and the four
runner statement/facade/test owners frozen in the crate plan.
`source_set_term.rs`, parser, resolver, canonical specification, corpus,
expectations, sidecars, trace metadata, semantic/proof/goal owners, B4, and
B5 are forbidden. The existing inactive template fixture is parser/source
evidence only and keeps its advanced-semantics rejection intent. No
blocking `spec_gap`, boundary violation, or repository-metadata conflict is
present.

Repeated boundary and implementation-scope reviews report **NO FINDINGS**.
The exact-32 documentation-only scope, unchanged forbidden paths, and all
nine hard gates PASS; independent final quality is valid `98/100`.

## Task 258B3M2B2B3B Implementation Boundary Closure

Implementation changes only the frozen three checker and four runner files.
Both set-term source owners, parser/resolver owners, module topology,
visibility, dependencies, public routes, authority artifacts, and semantic
ownership remain unchanged. The measured checker owner sizes are
`36568/4930/7266`; all test-sufficiency repeats and final implementation
repeat report **NO FINDINGS**. Focused tests, libraries `398/456`,
workspace Clippy/tests, format, and diff PASS. The synchronized scope is
exactly `39` files. Source/documentation consistency repeat reports
**NO FINDINGS** after independently confirming scope, metrics/hashes,
authority, trace, and `source_set_term` no-ops. Final
documentation/boundary and independent quality reviews are
**NO FINDINGS**, all hard gates PASS, valid `98/100`. Cached-diff/staging,
commit, post-commit, and fresh inventory remain pending.

## Task 258B3M2B2B3C Documentation Boundary

This prerequisite changes design/ledger/audit documentation only. The future
implementation boundary is exactly checker `source_statement.rs`,
`typed_ast.rs`, `resolved_typed_ast.rs` and four paired runner owners. Both
`source_set_term.rs` owners, every authority artifact, public schema, error/
debug grammar, dependency, active route, and semantic/trace credit remain
outside scope. B3C reuses the set-only statement fingerprint and atomic
typed/final APIs; no lower prerequisite is authorized. Boundary review after
the completed documentation diff remains pending.

## Task 258B3M2B2B3C Implementation Boundary Closure

Implementation changes only the frozen three checker and four runner
consumers. Both `source_set_term.rs` owners, parser/resolver owners, module
topology, visibility, dependencies, public routes, authority artifacts, and
semantic ownership remain unchanged. Checker owner sizes are now
`38891/4932/7268`; the production manifest remains 23 paths and grows only
to 133,092 lines. The two test-review gaps and one implementation finding
were remediated within the frozen owners; repeated reviews report
**NO FINDINGS**. Final documentation/boundary review reports **NO FINDINGS**,
and independent quality passes all nine hard gates at valid `98/100`.

## Task 258B3M2B2B3D Documentation Boundary

B3D freezes another private exact consumer of the existing cohesive
`source_statement`/typed/final ownership. The future seven-file change may
select one `Qua` set term, publish one witness-to-SetTerm edge, and revalidate
the existing set fingerprint only. Both `source_set_term.rs` owners,
parser/resolver/binding code, public schemas/errors/debug grammar,
dependencies, active routing, and every semantic owner remain unchanged.
Current module sizes and production manifests remain the B3C closure values;
no split or boundary migration is authorized.

Final read-only measurements reproduce `38891/6806/4932/7268`,
`23/133092`, and the frozen checker production path/content hashes. Focused,
crate, Clippy, formatting, and workspace verification pass without a source
or boundary change.

## Task 258B3M2B2B3D Implementation Boundary Inventory

Implementation changes only checker `source_statement.rs`, `typed_ast.rs`,
and `resolved_typed_ast.rs`, plus runner
`type_elaboration/source_statement.rs`, `type_elaboration.rs`, `runner.rs`,
and `tests/type_elaboration/source_statement.rs`. Checker owner sizes are
now `41452/4933/7270`; the unchanged Task-255 leaf remains `6806`.
Production remains 23 paths/135,656 lines with the frozen path set.

Both `source_set_term.rs` owners, parser/resolver/binding owners, module
topology, visibility, dependencies, public routes/schemas/errors/debug,
authority and corpus artifacts, active routing, and semantic phases remain
outside the diff. The four checker/five runner tests and
`32/70/44/72/62/21` matrices stay inside the authorized statement/test
owners. Test-sufficiency and independent implementation reviews report
**NO FINDINGS**. Repeated source/documentation and boundary review also
reports **NO FINDINGS** after the Medium review-state and Low canonical
qua-edge/family-order corrections. Both packages, formatting, full Clippy,
workspace tests, five CLIs, and count/hash reruns PASS without expanding the
boundary. Independent final read-only quality review reports
**NO FINDINGS**; all nine hard gates PASS with no cap at valid `100/100`
(`20/20/15/15/10/10/5/5`). Only exact staging/cached-diff review,
implementation commit, and post-commit/fresh-next-task gates remain pending.

## Task 258B3M2B2B3E Documentation Boundary

B3E adds no module or dependency. The future exact consumer remains confined
to the cohesive checker statement/typed/final modules and four existing
runner statement/facade/test modules. Both Task-255 `source_set_term.rs`
owners, parser/resolver/binding source, public APIs/errors/debug grammar,
authority artifacts, active routing, and semantics are excluded. The
current documentation-only scope changes no production boundary; source
split decisions remain unchanged.

## Task 258B3M2B2B3E Implementation Boundary Inventory

Implementation changes only checker `source_statement.rs`, `typed_ast.rs`,
and `resolved_typed_ast.rs`, plus the frozen four runner consumers. Checker
owner sizes are now `43598/4934/7272`; the unchanged Task-255
`source_set_term.rs` owner remains `6806`. The implementation adds only a
private exact B3E statement/witness profile, typed-installation allowlist,
and final clone/revalidation allowlist. It adds no public API, error/debug
grammar, dependency, module, parser/resolver/binding owner, active route, or
semantic table.

The four checker tests exercise the frozen `32/70/53/72/62/21` matrices,
generator-stage precedence, complete-subtree ownership, all 120 family
orders, and clone/replay/semantic deferrals. Both `source_set_term.rs` owners
and every authority/corpus/trace artifact remain unchanged. Focused checker
`4/4` and the 410-test checker library pass. Source/documentation and
boundary re-review reports **NO FINDINGS** after the three bounded
`design_drift` corrections. Checker `410+15`, runner
`471+3/14/137/2/21`, formatting, full Clippy, workspace tests, five CLIs,
and count/hash/scope reruns PASS without expanding the boundary. Independent
final quality reports **NO FINDINGS**; all nine gates PASS at valid
`100/100`. Staging and post-commit gates subsequently closed in
implementation commit `e4479691db3b0a8785bb16e94d386bd71a394274`;
fresh inventory selected Task 258B4A.

## Task 258B4A Boundary Freeze

B4A keeps lower formula rows, contracts, and behavior unchanged.
`source_formula_composition.rs` changes only the production visibility of
its existing output helper to `pub(in crate::runner)`;
`source_statement.rs` authenticates that Task-257B1 handoff and creates the upper
`Composite(0)` statement association; `typed_ast.rs` owns atomic paired
installation and `resolved_typed_ast.rs` owns final revalidation.

The runner boundary is limited to that one crate-private visibility seam and
the existing statement selector/wiring/test surfaces. Parser, resolver,
binding, all checker Task-252/256/257 owners, every other lower runner
surface, fixtures, expectations, sidecars, and trace metadata remain outside
the write scope. The eight-file implementation boundary is cohesive and
requires no module split or ownership transfer.

Fresh read-only boundary review reports **NO FINDINGS**. The visibility seam
does not transfer lower ownership or authorize any lower behavior change;
implementation boundary review remains a separate later task.

## Task 258B4A Implemented Boundary Inventory

Only the frozen three checker and five runner files changed. Checker owners
now measure `45,476`, `5,004`, and `7,347` lines. The runner owners measure
`12,737`, `1,853`, `810`, `2,627`, and `27,349` lines. The sole lower-family
edit is the crate-private visibility of the existing validated Task-257B1
helper. Parser, resolver, binding, Task-252/256/257 checker owners, all other
lower runner owners, corpus artifacts, trace metadata, and semantic phases
remain unchanged. Independent implementation review reports **NO
FINDINGS**; the write set is cohesive and requires no split or ownership
transfer.

Final source/documentation and boundary consistency reports **NO FINDINGS**
after the bounded documentation corrections. Complete verification PASSes.
Independent final quality reports **NO FINDINGS**, all nine hard gates PASS,
and valid `100/100`; only staging, commit, and post-commit inventory remain.

## Task 258B4B Boundary Freeze

B4B reuses the B4A checker-owned composite-statement API and the already
crate-private runner Task-257B2 output. The future write set is exactly
checker `source_statement.rs`, `typed_ast.rs`, `resolved_typed_ast.rs`, and
runner `type_elaboration/source_statement.rs`, `type_elaboration.rs`,
`runner.rs`, and `runner/tests/type_elaboration/source_statement.rs`.
`source_formula_composition.rs` requires no edit because B4A already exposed
the required validated helper.

Task 252 retains numeral occurrences, Task 256 retains equality occurrences,
Task 257/257B2 retain connective/wrapper/binder/composition occurrences, and
Task 258 alone owns theorem node 120 plus the two upper `Composite(0)`
associations. Parser, resolver, binding, lower checker/runner owners,
specification, corpus, expectations, sidecars, trace, public runner schemas,
and semantic phases are forbidden. This seven-file boundary has no required
ownership transfer or lower-stage prerequisite. Repeated read-only boundary
and source/documentation consistency review reports **NO FINDINGS**.
Implementation boundary review remains a separate later task.

## Task 258B4B Implemented Boundary Inventory

The implementation remains inside the frozen seven-file boundary: checker
`source_statement.rs`, `typed_ast.rs`, and `resolved_typed_ast.rs`; runner
`type_elaboration/source_statement.rs`, `type_elaboration.rs`, `runner.rs`,
and `runner/tests/type_elaboration/source_statement.rs`. Checker owner sizes
are `46,466`, `5,004`, and `7,350` lines. Runner owner sizes are `13,629`,
`814`, `2,629`, and `28,408` lines. The lower
`source_formula_composition.rs` owner remains exactly `1,853` lines and is
an explicit no-op.

This boundary adds no public API or ownership transfer. Task 257B2 retains
all 42 lower-owned nodes, the local theorem is the single statement-owned
node, and 81 nodes remain unassigned; Task 258 adds only the two upper
`Composite(0)` links. Parser, resolver, binding, every other lower owner,
corpus, expectations, sidecars, trace, and semantic phases remain unchanged.
Separate implementation review and the final read-only
boundary/source-documentation/bilingual repeat report **NO FINDINGS**.
Focused `4/4 + 5/5`, full offline workspace tests, formatting, full offline
Clippy, five CLI, count/hash, exact seven-file scope, audit no-op,
forbidden-artifact, and unchanged-stash gates PASS. Independent final
quality reports **NO FINDINGS**; all nine hard gates PASS with no cap at
valid `100/100` (`20/20/15/15/10/10/5/5`). Staging/cached-diff review,
the implementation commit, post-commit inventory, and B4C remain pending.

## Task 258B4C Documentation and Future Implementation Boundary

Task 258B4B closed at
`752c17ae7d552d5268d1028612b8174e480b6f3e` inside its exact seven-file
boundary. The post-commit tree is clean, ahead 1/behind 0 after report-only
origin metadata movement, and the protected stash is unchanged.

B4C first requires one separately reviewed and committed lower-stage
compatibility prerequisite. Its entire production/test write boundary is
runner `type_elaboration/source_formula.rs` plus
`runner/tests/type_elaboration/source_formula_composition.rs`. That change
may only extend the exact Task-257B3 selector from the active 138-byte,
one-final-LF source to the private 139-byte, two-final-LF sibling and add
zero-/three-LF rejection tests. Checker lower owners, binding/resolver/parser
owners, and production runner `source_formula_composition.rs` are explicit
no-ops. This is selector compatibility, not lower table, ownership, or
semantic expansion.

Only after that prerequisite commits may B4C edit the same seven upper
consumers as B4B: checker `source_statement.rs`, `typed_ast.rs`, and
`resolved_typed_ast.rs`; runner `type_elaboration/source_statement.rs`,
`type_elaboration.rs`, `runner.rs`, and
`runner/tests/type_elaboration/source_statement.rs`. Task 252 retains six
reference sites, Task 256 retains three equality roots, and Task 257B3
retains the 24 lower-owned sites including composite root 60. Task 258 may
own only theorem node 62 and two upper `Composite(0)` associations; 41
Surface nodes remain unowned.

The two future commits must preserve exact B1/B4A, B2/B4B, and B3/B4C
pairing, zero input facts, statement context visibility `[0]`, public APIs,
debug/error grammar, active authority artifacts, trace status/counts, and
all semantic/proof/IR boundaries. Parser, resolver, corpus, sidecars,
expectations, specification, B5, and unrelated owners are forbidden.
Documentation-only review and verification remain pending.

## Task 258B4C Documentation Boundary Review Status

After correcting one Medium `boundary_violation` in typed/final ownership
wording, repeated boundary and source/documentation review reports **NO
FINDINGS**. Raw authentication belongs to the runner selector and statement
producer; checker installers retain only handoff/fingerprint/arena
validation. The exact 32-document scope, both future write boundaries, all
forbidden no-ops, counts/hashes, offline tests, formatting, Clippy, and stash
gates PASS. Independent final quality reports **NO FINDINGS**, all nine hard
gates PASS, no cap, and valid `100/100`; only staging, commit, and
post-commit gates remain.

## Task 258B4C Implemented Boundary Inventory

The implementation remains in the frozen seven source files. Checker owners
measure `source_statement.rs=47,593`, `typed_ast.rs=5,005`, and
`resolved_typed_ast.rs=7,353`. Runner owners measure
`type_elaboration/source_statement.rs=14,479`,
`type_elaboration.rs=820`, `runner.rs=2,635`, and the statement test leaf
`29,948`; lower production `source_formula_composition.rs=1,853` is an
explicit no-op.

Task 257B3 retains all 24 lower-owned nodes; Task 258 owns only theorem node
62 and two upper `Composite(0)` associations, leaving 41 nodes unowned.
Parser, resolver, binding, other lower owners, corpus, expectations,
sidecars, trace, public schemas, and semantic phases remain unchanged.
Independent implementation and test-sufficiency reviews report **NO
FINDINGS**; no split or ownership transfer is required.

## Task 258B5A Frozen Consumer Boundary

The implementation boundary is exactly checker
`source_statement.rs`, `typed_ast.rs`, and `resolved_typed_ast.rs`, plus
runner `type_elaboration/source_statement.rs`, its two private facades
`type_elaboration.rs` and `runner.rs`, and the existing statement test leaf.
The runner may construct one crate-private B5A transaction and checker may
generalize exact B1/B5A pairing validation; no public DTO, enum, accessor,
producer/installer signature, error variant, or debug grammar changes.

Parser, resolver, BindingEnv, Task-252 and Task-256 producers, all other
lower families, active fixtures, expectations, sidecars, trace metadata, and
semantic phases are exclusion boundaries. The 93-node arena assigns only ten
term, five formula, and five statement nodes; label/reference/proof structure
and the other 73 nodes remain Surface-owned. B5B imports and B5C negative
routes are separate dependency-ordered tasks, not extensions of B5A.

### Task 258B5A Boundary Review Result

Independent source/documentation boundary review reports **NO FINDINGS**:
the seven consumers are sufficient, every excluded owner remains unchanged,
and no module split or ownership transfer is required.
