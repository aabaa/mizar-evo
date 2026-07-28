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
| `src/typed_ast.rs` | 4661 | typed AST data model | `typed_ast.md` | no | no | Large but cohesive typed-AST tables, ids, validation, rendering, tests, Task-253/254/255/256/257 installation checks, and mutually exclusive Task-248/258A/258B1/258B2/258B3/258B3N/258B3M1/258B3M2A/258B3M2B1 ownership; monitor ergonomics after downstream use. |
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
| `src/source_statement.rs` | 17569 | syntax-free source statement producer | `source_statement.md` | no | no | Cohesive Task-258A/258B1/258B2/258B3/258B3N/258B3M1/258B3M2A/258B3M2B1 statement, local-reference, witness, witness-name, numeral-witness, and parenthesized-witness transactions, resolver/binding/lower provenance, subtree validation, rendering, paired typed/final installation, and corruption matrices; no split required. |
| `src/source_structure.rs` | 5036 | syntax-free source structure-term producer | `source_structure.md` | no | no | Cohesive Task-254 term/wrapper/root/member/field-update/edge/request tables, written-partition and cross-family dependency/provenance validation, deterministic rendering, and corruption tests; no split required. |
| `src/source_type.rs` | 3294 | syntax-free source-type application producer | `source_type.md` | no | no | Cohesive Task-249 flat tables, environment/arena/form/graph/provenance validation, deterministic rendering, and exhaustive corruption tests; no split required. |
| `src/type_checker.rs` | 13235 | phase-6 type checking over checker-owned payloads | `type_checker.md` | no | no | Largest file but still within the phase-6 spec boundary; normalization, reserve and authenticated exact theorem-owner handoff validation, declaration checking, inference, coercions, fact queries, diagnostics, rendering, and tests remain behavior-coupled. |
| `src/registration_resolution.rs` | 5888 | phase-7 registration validation, activation, and existential gates | `registration_resolution.md` | no | no | Cohesive registration data layer and gate logic; no behavior-neutral split required. |
| `src/cluster_trace.rs` | 3948 | cluster closure and reduction trace recording | `cluster_trace.md` | no | no | Cohesive trace/replay module; no behavior-neutral split required. |
| `src/overload_resolution.rs` | 8004 | phase-8 overload pipeline | `overload_resolution.md` | no | no | Large but cohesive overload collection, template expansion, viability, specificity, selection, rendering, and tests; monitor ergonomics after downstream use. |
| `src/resolved_typed_ast.rs` | 7203 | final resolved typed AST assembly | `resolved_typed_ast.md` | no | no | Cohesive final projection module, including Task-251/252/253/254/255/256/257/258A/258B1/258B2/258B3/258B3N/258B3M1/258B3M2A/258B3M2B1 clone-preserving handoffs and semantic coexistence guards; no behavior-neutral split required. |
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
