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
| `src/lib.rs` | 51 | crate boundary and public module exports | `00.crate_plan.md` and `source_spec_audit.md` | no | no | Keep as the crate root; it exports the documented syntax-free formula-composition, definition, statement, and Task-269A proof-local declaration source modules. |
| `src/typed_ast.rs` | 6513 | typed AST data model | `typed_ast.md` | no | no | Large but cohesive typed-AST tables, validation, rendering, and one-shot handoffs, including mutually exclusive Task-259--264 definition transactions and separate Task-269A/B plus privately boxed Task-269C/269CT/269G/269GT/269GUPT/269GU/269GC/269GCT/269GCU proof-local owners; monitor ergonomics after downstream use. |
| `src/binding_env.rs` | 3168 | binding environment and resolver shell boundary | `binding_env.md` | no | no | Cohesive binding/context data layer, including source-formula, Task-258B1 statement-context identity, the unchanged Task-258B2 context contract, and exact Task-269A/269G installed-local tests; no behavior-neutral split required. |
| `src/source_context.rs` | 1727 | syntax-free source-item and binding-context producer | `source_context.md` | no | no | Cohesive Task-248 validation, table construction, recovery, handoff, and boundary tests; no split required. |
| `src/source_atomic_formula.rs` | 8511 | syntax-free source atomic-formula producer | `source_atomic_formula.md` | no | no | Cohesive Task-256/257C1 nine-table association, resolver provenance, predicate-segment/shared-boundary validation, cross-family ownership/fingerprint validation, deterministic rendering, install checks, compatibility literals, and test-only dependency corruption seams; no split required. |
| `src/source_composite_formula.rs` | 4700 | syntax-free source composite-formula/binder producer | `source_composite_formula.md` | no | no | Cohesive Task-257A/B1/B2/B3 exact profiles, binding extension, wrapper/tree validation, rendering, install checks, and corruption/profile tests; no split required. |
| `src/source_formula_composition.rs` | 5366 | syntax-free cross-family formula composition producer | `source_formula_composition.md` | no | no | Cohesive Task-257B1/B2/B3 atomic-edge/bound-use associations plus separate Task-257C2 condition-to-atomic and Task-257C3 predicate-chain transactions, dependency fingerprints, deterministic rendering, installation, and corruption tests; no split required. |
| `src/source_attribute.rs` | 3074 | syntax-free source-attribute producer | `source_attribute.md` | no | no | Cohesive Task-250 flat tables, environment/parent/arena/provenance validation, deterministic rendering, and corruption tests; no split required. |
| `src/source_attribute_definition.rs` | 1516 | syntax-free source attribute-definition producer | `source_attribute_definition.md` | no | no | Cohesive Task-261 four-table handoff, exact resolver/lower/context ownership, obligation-preserving one-shot validation, deterministic rendering, and Task-259/260 isolation; production remains syntax-free. |
| `src/source_evidence.rs` | 2413 | syntax-free source-evidence request/reference producer | `source_evidence.md` | no | no | Cohesive Task-251 request/response tables, upstream association, catalog/payload validation, deterministic rendering, and corruption tests; no split required. |
| `src/source_term.rs` | 5307 | syntax-free source primary-term producer | `source_term.md` | no | no | Cohesive Task-252 term/reference/request tables plus the exact Task-269GU/269GCU proof-local term/reference composites, binding and parent validation, deterministic rendering, dependency/fingerprint/arena corruption matrices, and cfg(test)-only corruption seams; production remains syntax-free and no split is required. |
| `src/source_application.rs` | 4001 | syntax-free source functor-application producer | `source_application.md` | no | no | Cohesive Task-253 application/wrapper/candidate/argument/request tables, dependency and provenance validation, deterministic rendering, and corruption tests; no split required. |
| `src/source_functor_definition.rs` | 2237 | syntax-free source functor-definition producer | `source_functor_definition.md` | no | no | Cohesive Task-260 five-table handoff, baseline-preserving two-obligation projection, resolver/lower provenance validation, deterministic rendering, and typed/final Task-259 isolation checks; production remains syntax-free. |
| `src/source_mode_definition.rs` | 1877 | syntax-free source mode-definition producer | `source_mode_definition.md` | no | no | Cohesive Task-262 six-table handoff, standalone-RHS fingerprint, unresolved inhabitation request, linked pending Sethood projection, deterministic rendering, and typed/final Tasks-259--261 isolation; production remains syntax-free. |
| `src/source_predicate_definition.rs` | 1794 | syntax-free source predicate-definition producer | `source_predicate_definition.md` | no | no | Cohesive Task-259 five-table handoff, baseline-preserving pending-obligation projection, resolver/lower provenance validation, deterministic rendering, and typed/final installation checks; production source remains syntax-free. |
| `src/source_property_implementation.rs` | 2460 | syntax-free source property-implementation producer | `source_property_implementation.md` | no | no | Cohesive Task-264 five-table equals/means handoff, resolver/return/lower/arena validation, baseline-preserving pending-obligation projection, deterministic rendering, and typed/final sibling isolation; production remains syntax-free. |
| `src/source_set_term.rs` | 6806 | syntax-free source set-term producer | `source_set_term.md` | no | no | Cohesive Task-255/255C1 seven-table association, condition-subtree exclusion, cross-family ownership/fingerprint validation, deterministic rendering, install checks, and corruption tests; no split required. |
| `src/source_statement.rs` | 52266 | syntax-free source statement producer | `source_statement.md` | no | no | Cohesive Task-258 statement/witness transactions and their corruption matrices; the same four Task-269 tests reuse the private exact B3N/B3M1 fixtures, including all-field arena and isolated cross-profile rejection, while production proof-local ownership remains in its dedicated module. |
| `src/source_proof_local_declaration.rs` | 6679 | syntax-free proof-local declaration producer | `source_proof_local_declaration.md` | no | no | Cohesive Task-269A/B named-witness, Task-269C proof-`let`, Task-269G proof-`given`, Task-269GUP new-source binding, and Task-269GC declaration-condition binding inputs/rows/tables/handoffs, exact lower and independent theorem authentication, resolver-local lexical binding transitions and lookup replay, deterministic rendering, phase-ordered replay, owner validation, and the Task-269GCU cfg(test)-only ownership sentinel; no syntax, type, term/use, condition, fact, or proof-semantic ownership. |
| `src/source_structure.rs` | 5036 | syntax-free source structure-term producer | `source_structure.md` | no | no | Cohesive Task-254 term/wrapper/root/member/field-update/edge/request tables, written-partition and cross-family dependency/provenance validation, deterministic rendering, and corruption tests; no split required. |
| `src/source_structure_definition.rs` | 1773 | syntax-free source structure-definition producer | `source_structure_definition.md` | no | no | Cohesive Task-263 definition/member/inheritance/mapping/coherence tables, private resolver/baseline snapshots, exact contribution-effect and own-domain obligation validation, deterministic rendering, and compound precedence tests; production remains syntax-free. |
| `src/source_type.rs` | 13099 | syntax-free source-type application producer | `source_type.md` | no | no | Cohesive Task-249 flat/extension families plus exact Task-269CT/269GT/269GUPT/269GCT proof-local composites, environment/arena/form/graph/provenance validation, deterministic rendering, exhaustive corruption tests, and cfg(test)-only corruption seams; production remains syntax-free and no split is required. |
| `src/type_checker.rs` | 13244 | phase-6 type checking over checker-owned payloads | `type_checker.md` | no | no | Largest file but still within the phase-6 spec boundary; normalization, reserve and authenticated exact theorem-owner handoff validation, declaration checking, inference, coercions, fact queries, diagnostics, rendering, tests, and Task-259/260/264 obligation-kind serializers remain behavior-coupled. |
| `src/registration_resolution.rs` | 5897 | phase-7 registration validation, activation, and existential gates | `registration_resolution.md` | no | no | Cohesive registration data layer, gate logic, and Task-259/260/264 obligation-kind serializers; no behavior-neutral split required. |
| `src/cluster_trace.rs` | 3948 | cluster closure and reduction trace recording | `cluster_trace.md` | no | no | Cohesive trace/replay module; no behavior-neutral split required. |
| `src/overload_resolution.rs` | 8004 | phase-8 overload pipeline | `overload_resolution.md` | no | no | Large but cohesive overload collection, template expansion, viability, specificity, selection, rendering, and tests; monitor ergonomics after downstream use. |
| `src/resolved_typed_ast.rs` | 8530 | final resolved typed AST assembly | `resolved_typed_ast.md` | no | no | Cohesive final projection module, including clone-preserving Task-259--264 definition handoffs and complete, mutually exclusive Task-269A/B plus privately boxed Task-269C/269CT/269G/269GT/269GUPT/269GU/269GC/269GCT/269GCU proof-local replay; no behavior-neutral split required. |
| `src/determinism_suite.rs` | 1101 | test-only cross-module determinism suite | `00.crate_plan.md` and `source_spec_audit.md` | no | no | Keep as private `#[cfg(test)]` crate support. |
| `tests/lint_policy.rs` | 1941 | cross-cutting policy and audit guards | `source_spec_audit.md`, `bilingual_sync_audit.md`, and `module_boundary_audit.md` | no | no | Centralized policy guardrails include Task-259--264 and Task-269A module/spec/public-enum coverage and the unchanged production syntax boundary. |
| `tests/support/source_attribute_definition_unit.rs` | 1070 | test-only Task-261 unit-test support | `source_attribute_definition.md` and this audit | no | no | Non-integration child support for the exact producer, obligation preservation, corruption, ownership, replay, and cfg(test)-only Task-262 reverse-isolation fixture. |
| `tests/support/source_functor_definition_unit.rs` | 3798 | test-only Task-260 unit-test support | `source_functor_definition.md` and this audit | no | no | Non-integration child support; cfg(test)-only helpers reuse actual Task-259/260 producers for Task-261 and Task-263 reverse-isolation checks without changing production ownership. |
| `tests/support/source_mode_definition_unit.rs` | 1237 | test-only Task-262 unit-test support | `source_mode_definition.md` and this audit | no | no | Non-integration child support for exact rows, obligation suffixes, Typed/final replay, all sibling-family installation orders, and the cfg(test)-only Task-263 mode projection/owner fixture. |
| `tests/support/source_predicate_definition_unit.rs` | 1979 | test-only Task-259 unit-test support | `source_predicate_definition.md` and this audit | no | no | Non-integration child support retains the existing test-only syntax dependency and adds only the cfg(test)-only Task-263 predicate projection fixture; no production import, lint exception, public resolver API, or semantic owner is added. |
| `tests/support/source_property_implementation_unit.rs` | 2004 | test-only Task-264 unit-test support | `source_property_implementation.md` and this audit | no | no | Non-integration child support for exact equals/means construction, corruption, nonempty-baseline transactionality, final replay, orphan/extra rejection, and actual Task-259 isolation. |
| `tests/support/source_structure_definition_unit.rs` | 1502 | test-only Task-263 unit-test support | `source_structure_definition.md` and this audit | no | no | Non-integration child support for complete exact rows/debug bytes, resolver/row/metadata/shape corruption, all 12 adjacent precedence categories, contribution/baseline transactionality, and bidirectional Tasks-259--262 Typed/final isolation. |

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

## Task 258B5A Implemented Consumer Boundary

Implementation remains inside the frozen seven consumers. Checker owners are
`source_statement.rs`, `typed_ast.rs`, and `resolved_typed_ast.rs`; runner
owners are `type_elaboration/source_statement.rs`,
`type_elaboration.rs`, `runner.rs`, and the existing statement test leaf.
The implementation adds private B5A construction, exact B1/B5A paired
validation, resolver-node-kind authentication, and clone preservation
without creating a public module or moving an owner.

Parser, resolver, BindingEnv, Task-252, Task-256, all sibling lower
families, active fixtures, expectations, sidecars, trace metadata, and
semantic phases remain unchanged. Exact `20/73` ownership retains all
label, citation, proof-block, and wrapper nodes as arena provenance. B5B and
B5C remain outside this boundary; no split or ownership transfer is
required.

## Task 258B5B Frozen Three-Commit Boundary

This prerequisite changes synchronized design documentation only. The next
lower-stage commit is restricted to runner `import_fixtures.rs` and the
existing statement test leaf, adding a crate-private opt-in imported `Ref`
label plus exactly two tests. It must not change the normal augmentation
function or any checker file.

Only after that commit may the upper task change the same seven consumers as
B5A: checker `source_statement.rs`, `typed_ast.rs`,
`resolved_typed_ast.rs`; runner `type_elaboration/source_statement.rs`,
`type_elaboration.rs`, `runner.rs`, and the statement test leaf. Public
checker citation target/kind changes belong to that upper commit. Parser,
resolver, artifact schema, BindingEnv, Tasks 252/256, public runner/CLI,
fixtures, expectations, sidecars, trace metadata, and semantics remain
outside both implementation scopes. No module split or ownership transfer is
required.

## Task 258B5B Implemented Consumer Boundary

Documentation commit `141dc44a757555e8d4837756515e1577f672348b`
precedes isolated lower commit
`46dd9db56ced2fcc57799420de9d5fed06f284f5`. The current upper diff is
confined to the same seven consumers frozen above. Checker statement
production owns the imported target, resolver projection/import replay,
row validation, and debug schema; typed installation owns exclusive
B1/B5A/B5B profile pairing; final assembly owns clone-time revalidation.
The four runner consumers remain private extraction, facade, and test
owners.

No parser, resolver, artifact, BindingEnv, Task-252/256 producer, public
runner/CLI, corpus, expectation, sidecar, trace, or semantic owner moves.
The lower helper is a prior dependency rather than an eighth upper file.
Current checker owners measure `50732/5008/7356`; production remains 23
paths. The files remain large but cohesive, and this task requires no module
split or ownership transfer.

## Task 258B5C Frozen Non-Consumer Boundary

This commit owns synchronized design documentation only. The next two
prerequisites are owned by `mizar-resolve`: R-032A owns the validated
one-to-one structural `SurfaceResolvedArena` in `resolved_ast.rs`, its tests,
and the sole `tests/lint_policy.rs` R-026 owning-spec entry for
`SurfaceResolvedArenaError`; R-032B owns proof-step projections, simple
unqualified candidates, proof-scope paths, ordinals, and provenance
collection in `labels.rs` and its tests, plus only the
`tests/lint_policy.rs` R-026 decision mapping
`ProofLabelSourceCollectionError` to `labels.md`. R-032A exact state/key
errors and all node payloads use
`SurfaceNodeId`; R-032B borrows ast/resolved under one `'a`, validates but
does not store module, owns namespace/contribution, and returns `Self`.
Only R-032B may consume the R-032A map. A later `mizar-test`
declaration-symbol task owns the
two active fixtures, sidecars, trace rows, runner observations, and tests.

`mizar-checker` is explicitly not a B5C implementation consumer.
`SourceStatementReferenceHandoff` rejects unresolved results and requires a
keyed `Resolved` node, so no checker statement, reference, citation,
binding, typed, or final owner may accept these negative routes. Parser,
artifact, Tasks 252/253, B1/B5A/B5B, and all semantic phases remain
unchanged. No module split or ownership transfer is required; moving scope
derivation, resolved ids, or ordinals into the runner would be a
`boundary_violation`.

R-032A implementation preflight corrected only its ownership accounting:
omitting the mandatory R-026 enum-decision owner was High `design_drift`,
not a semantic `spec_gap`. The correction is a separate docs-only commit;
the later exact three-Rust-file resolver implementation does not add a
checker consumer or move any boundary above.

R-032B implementation preflight found the analogous mandatory public-enum
owner omission. The prior two-Rust-file scope is High `design_drift`, not a
semantic `spec_gap` or `test_gap`. Its current separate docs-only correction
freezes later implementation to exactly `labels.rs`, `labels/tests.rs`, and
the sole `tests/lint_policy.rs` decision above. The correction itself owns
exactly 31 design files: eight paired resolver families, four paired checker
families, three paired `mizar-test` families, and the global design TODO.
It precedes R-032B implementation in the effective seven-task order through
active B5C and changes no production source, test intent, fixture,
expectation, sidecar, trace row/status/count, public diagnostic code,
semantic behavior, or coverage state. The coverage audit remains a
deliberate no-op because no mapping, owner, deferral, or credit changes.

The later runner is selected only by frozen source bytes plus normal AST,
then authenticates shared resolver env/module and one id-0 matching
local-source contribution. Input corruption and authenticated confinement
use separate private detail keys. Current documentation ownership is exactly
48 design files.

R-032B's owner boundary is further closed by its default-deny edge table:
exact `Root -> CompilationUnit -> ItemList -> direct TheoremItem -> direct
ProofBlock`; then
only direct normal `CompactStatement`/`ConclusionStatement`, compact
proposition-label inspection, direct statement proof/justification children,
and the exact simple-reference chain. Forbidden formula/token/wrapper,
unsupported/recovered/malformed, qualified/grouped/bulk, and template
subtrees receive no ordinal and no descent. Root and CompilationUnit each
require their exact one normal structural child; ItemList scans only direct
normal theorem children and skips/no-descends other item children. Positive
upper edges and negative missing/additional/wrong, direct Root/Compilation
theorem relocation, and `VisibleItem` wrapping, plus lower forbidden
relocation and mixed-list tests, belong to resolver.

The later runner owns only independent authentication of environment module,
derived namespace, the exact one id-0 LocalSource contribution record and
source id, and every projection's module/namespace/contribution. Its
field-by-field mutation matrix maps exclusively to `proof_scope_input`;
authenticated confinement alone maps to `proof_scope_confinement`. Neither
boundary adds a checker consumer or changes the 48-file scope.

R-032A preflight correctly stopped a prospective resolver-side workaround.
Dense compatibility-node ids belong to mizar-syntax S-026; R-032A only
consumes that accessor later. This prevents a new checker, runner, or resolver
ownership leak and leaves every frozen B5C consumer boundary unchanged.

## Task 258B5C Implemented Non-Consumer Result

The active B5C route is confined to `mizar-test`. It consumes resolver-owned
R-032A/R-032B output and never enters checker statement/reference handoffs.
The exact source scope additionally includes four count assertions in
`crates/mizar-test/tests/metadata.rs`, discovered as
`test_expectation_drift` plus write-scope `design_drift`; this adds no runtime
owner.

No checker file, public API, diagnostic code, binding/type/proof/goal result,
or Core/CFG/VC boundary changes. The checker remains an explicit
non-consumer of both unresolved confinement cases.

## Task 259 Frozen Module Boundary

Future `mizar-checker::source_predicate_definition` owns only the syntax-free
five-table predicate-definition handoff and the transactional insertion of
one pending predicate-property obligation. `mizar-test` owns raw
`SurfaceAst` inspection, exact-source selection, direct-sibling association,
and construction of syntax-free inputs. Resolver owns the predicate
`SymbolEntry`, `DefinitionEntry`, source contribution, and origin. Task 259
does not take parser/resolver ownership.

The resolver's generic `PropertyClause` Attribute/Attribute projection is
not a semantic predicate-property input. The private runner authenticates
`symmetry` from the exact normal same-block/later-sibling source shape, and
the checker validates only the resulting source-keyed property site. The
definition-local assumption is a Task-259 guard, not a Task-258 statement.
The justification subtree is retained for future Task 272; Task 259 stores
only its `SourceAnchor` and performs no proof work.

At the original Task-259 freeze, Task 248 could not publish the two definition
parameters. The separate documentation/implementation commits `f9b47375` and
`ca54135f` now widen exactly that admitted profile while preserving the
existing public `SourceBindingContextHandoff`. Reconstructing
`BindingEnv` in Task 259 or in the runner would be a `boundary_violation`.
Tasks 249, 252, and 256 remain lower owners of type, term, and equality rows.
Task 260 owns functor-definition intake. No Core, CFG, VC, fact, axiom,
accepted definition, public diagnostic, or proof owner moves.

## Task 248 Two-Parameter Extension Module Boundary

`mizar-checker::source_context` remains the sole owner of Profile-B
validation, `BindingEnv` construction, dense bindings/contexts, and the
existing immutable handoff. It imports no syntax and adds no public API.
`mizar-test` owns only a private exact direct-parameter extractor, real
resolver-shell authentication, and validation of caller-owned sites against
the shared typed arena. The helper returns a projection; it cannot allocate a
competing arena/typed AST or select an active route.

Task 259 owns exact whole-source selection and later predicate tables. Tasks
249/252/256 retain type/term/formula extraction, and Task 272 retains the
property proof. Guard, predicate, property, and justification descendants are
no-row/no-descent at the Task-248 helper. This split closes the prospective
binding-reconstruction `boundary_violation` without moving any semantic
owner.

## Task 259 Corrected Future Module Boundary

The future public checker module is exactly
`src/source_predicate_definition.rs`; `src/lib.rs`,
`src/typed_ast.rs`, and `src/resolved_typed_ast.rs` are its only stateful
checker consumers. `type_checker.rs` and `registration_resolution.rs` consume
only the new obligation-kind debug name. `tests/lint_policy.rs` consumes the
module for its documented-module, public-enum, and source/spec-audit
allowlists. Its no-syntax boundary guard automatically scans every checker
`.rs` file and needs no task-specific allowlist entry.

`TypedAst` owns the one-shot atomic replacement of its authenticated baseline
obligation table with the producer-completed table and the Task-259 handoff.
`ResolvedTypedAst` receives no runner-replaceable input; it privately clones
the typed-owned completed table, revalidates the correctness link and four
lower fingerprints, and clone-preserves the handoff. It publishes no new
obligation/fact/proof/acceptance getter.

The runner's new private leaf owns whole-source selection, same-block sibling
authentication, a shared surface-indexed arena, and syntax-free input
construction. It reuses the completed Task-248 Profile B and lower Tasks
249/252/256. The facade selects this exact route before generic type-gap
fallback. Four mechanical active-type count assertions and one new
fixture/sidecar/trace row are non-semantic consumers. No ownership moves to
parser, resolver, Core/CFG/VC, facts, proofs, artifacts, or Task 260+.

## Task 259 Active Module-Boundary Result

The frozen split is implemented without ownership transfer. The checker
module owns all five immutable `1/2/1/1/1` tables, four lower fingerprints,
the baseline-preserving pending-obligation projection, atomic typed install,
and private final clone/revalidation. The runner owns only exact-source/AST,
same-block sibling, resolver-provenance, subtree-exclusion, and shared-arena
composition. Task 272 retains proof/discharge and Task 260 retains the mixed
functor-definition family.

Five checker tests reside in the external non-integration child
`tests/support/source_predicate_definition_unit.rs`. Its existing test-only
syntax dependency constructs opaque resolver shell ids; physical production
source remains syntax-free and no lint exception/public resolver API is
added. This closes the candidate test-layout `boundary_violation`. Two runner
source-statement active-count assertions were independently reviewed as
mechanical `198 -> 199` consumers with empty-selection checks preserved.

Fresh source-review measurements are checker producer `1794` lines, external
test-support `1974`, runner production leaf `1233`, and paired runner test
leaf `517`. The checker production manifest is `24/147030` with path/content
hashes `022586d6096dfa2eb05d6b0b9e91bf6dea71e5fc0a036f54a3bb462c7af16ac5` /
`14ab798c611d954f9ea346367547240e58e9c5d0e04ec8a4ae68e2f20b71860b`;
runner is `31/63248` with
`0d6edf22a94efd3497423f427accaf34341d223f4339a0adf9c4a7a523271e89` /
`a9abe9fcbc4a9b04e84fcb6402e13b95cdcd71e7ed2952dbf1a8fb2e1b551a9f`.
Final boundary review ended with no findings, and the quality review passed
all nine hard gates with an uncapped `100/100`; commit/post-commit gates remain.

## Task 260 Frozen Boundary

Task 260 adds one future checker-owned syntax-free functor-definition module.
Raw source bytes, Surface kinds and node IDs, sibling association, and resolver
selection remain private to `mizar-test`. The checker consumes only resolver
identities, lower dense IDs/fingerprints, typed sites/ranges/contexts, styles,
return types, definiens targets, correctness associations, and a caller-owned
baseline obligation table.

Task 248 owns bindings/context; Task 249 owns written types; Tasks 252--256 own
definiens roots; Task 260 only associates them with functor definitions and
appends pending existence/uniqueness rows. Task 259 is an independent sibling.
Proof/acceptance/fact/VC and Task 261+ remain outside the boundary. Current
production remains checker `24/147030` and runner `31/63248` during the
documentation prerequisite.

The enum-extension boundary names all three exhaustive
`InitialObligationKind` serializers: `typed_ast.rs`, `type_checker.rs`, and
`registration_resolution.rs`. Each receives only the two frozen Task-260
debug names. Task 260 does not edit Task-259 validation; instead it rejects a
Task-259 handoff or predicate-property baseline and leaves mixed coexistence
for a separately authorized owner.

## Task 249R Boundary Addendum

The only executable owner is `source_type.rs`; its four tests remain in that
module's existing private test region. No syntax dependency, runner hook,
resolver edit, public diagnostic, lint exception, second Typed/Resolved field,
or Cargo change is authorized. Task 260 may consume the new return IDs only
after the separate Task-249R implementation commit. Fabricated `BindingId`
rows and Task-260 producer work are explicit boundary violations.

## Task 260 Active Boundary Result

The implemented checker boundary adds one documented syntax-free production
module, now `2237` lines, and its non-integration child test body, now `3782`
lines. `typed_ast.rs` is `5172` lines after the Task-260 field, accessors,
transaction, and `cfg(test)` malformed-final-state injectors. The public
module/export/enum/source-spec/allow inventories are synchronized and lint
policy passes `15/15`; the checker still has no production syntax dependency.

The actual producer authenticates optional Task-253/254/255 targets before
returning the frozen semantic deferral, so invalid IDs and arena owners cannot
hide behind a blanket rejection. It never publishes those optional targets.
Task 259's installer remains unchanged; only the Task-260 installer and final
assembler enforce this task's mutual-exclusion contract. Checker production is
`25/150547` with path/content hashes
`0aad6b74904f456a462b0f481c84916a3234f5fecf302d9f048b380da8c3f846` /
`8b1c66cb73086b01d23a7cf8f7db2bebd0bab13218113c436f3d892a79a436d6`.

## Task 261 Frozen Boundary

Task 261 will add one documented syntax-free production module
`src/source_attribute_definition.rs` and one non-integration child support
body. The production owner is limited to four dense `1/2/1/1` tables,
resolver/lower authentication, deterministic rendering, and typed/final
validation. Raw syntax and resolver collection remain in one private
`mizar-test` route; checker production retains no syntax dependency.

The future module consumes Task-248/249/252/256 handoffs without modifying
them and preserves the obligation table. It owns no attribute-use evidence,
formula semantics, accepted attribute, fact, cluster, proof, IR, or VC. Public
module/export/enum/source-spec allowlists and exact source/support line counts
are updated only in the implementation commit. The documentation prerequisite
adds no production path and leaves the current `25/150547` manifest and hashes
unchanged.

## Task 261 Active Boundary Result

Task 261 now owns the documented `1516`-line syntax-free production module
and `1062`-line non-integration test body. Raw Surface/resolver extraction
stays in the private runner leaf. The checker validates exact shell-41
context ownership, four lower fingerprints, dense `1/2/1/1` rows, one-shot
typed/final ownership, unchanged obligations, and Task-259/260 exclusion.
Checker production is `26/152184`; exact manifest hashes are recorded in the
crate plan. No proof, fact, acceptance, IR, or VC owner moved.

## Task 262 Frozen Boundary

Task 262 will add one documented syntax-free production module
`src/source_mode_definition.rs` and one non-integration child test body. The
production owner is limited to six dense `1/2/1/1/1/1` tables, resolver and
post-prerequisite Task-248/249/249M fingerprint authentication, deterministic rendering, one
unresolved RHS-inhabitation request, one baseline-appended pending `Sethood`
row, and typed/final validation. Raw Surface/resolver selection remains in one
private `mizar-test` route; checker production retains no syntax dependency.

Task 262 implementation is blocked until a separate checker-only Task 249M
adds one standalone mode-RHS row to the Task-249 handoff. Reusing a third
binding-linked application is forbidden. Task 249M owns only that lower row,
its extension API/fingerprint/debug rendering, and four checker tests; it
lands in separate documentation and implementation commits before this
module.

The module owns no evidence response, base-shape decision, accepted mode,
expansion/interface fact, ParamGuard/FOL composition, proof, discharge, Core,
CFG, or VC. Public module/export/enum/source-spec allowlists and exact source/
support line counts change only in the implementation commit. This prerequisite
adds no production path and preserves checker production `26/152184` and both
recorded hashes.

## Task 249M Frozen Boundary

Task 249M remains inside the existing syntax-free `src/source_type.rs` owner.
It adds one standalone mode-RHS row/table/producer and four private module
tests without a new module, syntax dependency, runner hook, Typed/Resolved
field, diagnostic, or Cargo edge. The docs prerequisite preserves current
checker production `26/152184`; implementation must fresh-measure the same
26-file boundary. Task-262 production remains excluded.

## Task 249M Implemented Module Boundary

The existing `source_type` module now owns the standalone row/table/producer
and four tests. No module or dependency edge was added. Checker production is
`26/153116`; the sorted path hash remains
`e290d082e428124d3fd21919e76b88458daabfa44b7009a8cb1b3d8c430fec53` and the
ordered per-file content hash is
`3c85673ebb527cb33bb4b042b1b1194bda34a5348b4b6b20142617db47bde2f2`.
Task 262 remains a separate consumer task.

## Task 262 Active Module Boundary

Task 262 adds one syntax-free production owner,
`src/source_mode_definition.rs` (`1877` lines), and one external test-support
body (`1227` lines). Raw Surface and
resolver projection remain private to the runner. The checker manifest is
`27/155114`, with sorted path hash
`180b090a167912f0b04f014180ec6755aa5bde54eecd49f0990cc87fb566667f` and
ordered content hash
`4de970d1f6e4b05b6b9004856de61e68574588163317193d973cc5a5410f6022`.
The cfg(test)-only Task-261 fixture export exists solely to prove reverse
mixed-family rejection and adds no release API or dependency edge. All proof,
acceptance, fact, IR, VC, and Task-263 structure ownership stays outside.
## Task 249S Frozen Module Boundary

Task 249S remains inside the existing syntax-free `src/source_type.rs` owner.
Its standalone producer adds one immutable member-owner table and four local
tests; it does not weaken the binding-owned producer or reuse Task-249R/249M.
The future Task-263 upper producer belongs in a new dedicated module and is
explicitly out of this prerequisite. No runner, parser, resolver, Cargo, or
corpus file moves or splits here. Current `source_type.rs` remains 5,339 lines
until the separate implementation commit.

## Task 249S Active Module Boundary

The implementation remains confined to the existing syntax-free
`src/source_type.rs` owner plus synchronized documents. The module is
`6244` lines and the checker production manifest is `27/156019`; its
path/content hashes are
`180b090a167912f0b04f014180ec6755aa5bde54eecd49f0990cc87fb566667f` /
`37a7bb07a441086ee2915f601dedbca002f9a356b53a32050c29d467eb56b9f1`.
The new member table is behavior-coupled to the existing source-type
transaction and installation validator, so no split is warranted.

## Task 263 Frozen Module Boundary

Task 263 will add one dedicated syntax-free production owner,
`src/source_structure_definition.rs`, and one non-integration checker support
body. It consumes resolver identities, the committed Task-249S handoff, an
immutable obligation baseline, and typed arena sites. Raw source/Surface
selection, shell association, exact-source authentication, and consumer route
selection remain private to `mizar-test`; parser and resolver production do not
move.

The checker module owns only `2/4/1/2/0` immutable declaration/member/edge/
mapping/request tables, exact coverage and fields-only constructor validation,
resolver/lower authentication, unchanged-obligation projection,
deterministic debug, and Typed/final one-shot validation. Task 259 remains an
isolated sibling transaction; Tasks 260--262 are likewise excluded. No
definition acceptance, diagnostic, fact, proof, Core, CFG, or VC owner moves.
The docs prerequisite leaves checker `27/156019` and runner `34/67087`
production unchanged.

The private obligation snapshot remains behavior-coupled to this transaction;
it is not a second owner or public serializer. Stable debug renders its count,
not bytes. Exact grammar and compound precedence tests remain inside the same
module/support owners and create no extra boundary.

## Task 263 Active Module Boundary

`src/source_structure_definition.rs` is the sole 1,773-line production owner;
`tests/support/source_structure_definition_unit.rs` is the 1,502-line primary
checker support owner. Bounded changes to `lib.rs`, `typed_ast.rs`,
`resolved_typed_ast.rs`, `source_type.rs`, the cfg(test)-only sibling-module
visibility in `source_predicate_definition.rs` / `source_mode_definition.rs`,
the predicate/functor/mode support projection helpers, and lint inventory
preserve the syntax-free production boundary. Checker production is
`28/157908` with path/content
hashes `6e4bc96ef04cb5f011d53c651bb93549992e3c7fd0e7595b851d7181c8a65dcd` /
`8f0d26afe33ac1c2d570c7704371b8b4e86357b59fb0cccab22ac820dacf990e`.

## Task 264R No-Checker-Source Boundary

Task 264R modifies no checker module or public API. Its owner files are the four
frozen resolver production/test files. `binding_env.rs` remains unchanged and
cannot admit the new shell until separate Task 248P; property payload ownership
then remains separate Task 264. Checker production stays `28/157908` with the
hashes above, and no module split, dependency, lint inventory, or size decision
changes in this documentation prerequisite.

## Task 264R Implemented No-Checker-Source Boundary

The completed lower implementation changes only the four frozen
`mizar-resolve` files. No `mizar-checker` module, public API, dependency, lint
inventory, production count/hash, or line-count decision changes. The checker
consumer remains exclusively deferred to Task 248P and Task 264.

## Task 248P Frozen One-File Checker Boundary

Task 248P changes exactly `src/source_context.rs`: append one public
non-exhaustive item role, admit closed Profile C, reuse the existing binding
role/context tables, and add two inline tests. No module, dependency, Cargo,
runner, parser/resolver, corpus, trace, diagnostic-code, or lint-policy path is
added. Documentation preserves checker production `28/157908` and its current
path/content hashes; implementation keeps 28 paths, projects checker library
`467 -> 469`, and must remeasure lines/content. Runner remains byte-identical
at `35/67939`. Property payload ownership stays in separate Task 264.

## Task 248P Implemented One-File Checker Boundary

The implementation diff changes only `src/source_context.rs`. Checker
production remains 28 paths and is now 158,478 lines with path/content hashes
`6e4bc96ef04cb5f011d53c651bb93549992e3c7fd0e7595b851d7181c8a65dcd` /
`19a0dd0472f0e3b40c486ab9451322be03aab4322c53d30cff03ef5e6f8c8490`.
No module, dependency, Cargo, lint-policy, runner, corpus, trace, diagnostic, or
Task-264 semantic boundary changed.

## Task 264 Frozen Module Boundary

Task 264 will add only `source_property_implementation.rs` to the checker
public source-transport layer. It consumes resolver identities and existing
syntax-free lower handoffs; it imports no parser or syntax type. `lib.rs`
exports the module, `TypedAst` owns one private optional handoff installed only
by the projection transaction, and `ResolvedTypedAst` clone-preserves it after
full revalidation. `TypedAstParts` and `ResolvedTypedAstInputs` receive no
public construction field.

The private runner owns raw-source/Surface selection. The new checker family
cannot own Task-249PI lower construction, proof/acceptance/fact/IR/VC modules,
or Task-259 behavior. The docs prerequisite changes no Rust boundary. Task
249PI is the mandatory next lower owner before this boundary becomes active.

## Task 249PI Frozen Module Boundary

Task 249PI remains entirely inside existing syntax-free
`src/source_type.rs`. It adds no module, Cargo edge, parser/syntax dependency,
runner route, public raw-AST exposure, Typed/final field, or semantic-result
owner. The existing structure-member producer borrows an authenticated base
and appends only two existing member rows transactionally. The implementation
write scope is that one checker source file plus synchronized derived docs;
Task 264 remains the sole later runner/property-payload owner.

## Task 249PI Implemented Module Boundary

The implementation changes only existing `src/source_type.rs`, now 7,423
lines. The production manifest remains 28 paths and is 159,648 lines with
unchanged path hash
`6e4bc96ef04cb5f011d53c651bb93549992e3c7fd0e7595b851d7181c8a65dcd`
and content hash
`7d38e5c9fbc3ee2cb09d0d5d1187c4d29d1086c56f0b2dcd7f07cd0b60be283c`.
No module, dependency, runner, semantic owner, or public raw-syntax boundary changed.

## Task 264 Implemented Module Boundary

Task 264 adds one cohesive syntax-free production module,
`src/source_property_implementation.rs`; `lib.rs` exports it, `TypedAst` owns a
private optional projection-installed handoff, and `ResolvedTypedAst`
clone-preserves it after full revalidation. The only lower-file change outside
that owner is a cfg(test)-only primary-term corruption hook. No Cargo edge,
parser/syntax production import, public raw AST, proof/fact/IR/VC owner, or
Task-259 semantic behavior is added. Raw source/Surface authentication remains
private to the new `mizar-test` leaf route.

The current production manifest is `29/162347`, with path/content hashes
`37b91c2c419b83fa63150fe65d09b56c474dfa3d61134ba84056009dcdb923c1` /
`450abc3b7407f206c27b04613737716cf2192fb46c8960c8e167fcf0900fa143`.
The new owner is 2460 lines and its external test support is 2004 lines;
`lib.rs`, `typed_ast.rs`, `source_term.rs`, `type_checker.rs`,
`registration_resolution.rs`, `resolved_typed_ast.rs`, and lint policy are
respectively 50, 5455, 2263, 13244, 5897, 7727, and 1931 lines.

## Task 269A Frozen Module Boundary

The only new checker production owner is syntax-free
`src/source_proof_local_declaration.rs`. `lib.rs` exports it; `TypedAst` and
`ResolvedTypedAst` each add one private optional handoff plus a read-only
getter, while `TypedAstParts` and `ResolvedTypedAstInputs` remain unchanged.
The module may depend on checker binding/statement/term/typed types and
resolver `LocalTermBinding`, but not parser or syntax types.

The implementation also updates `tests/lint_policy.rs` so the module-to-doc,
public-API, enum-policy, module-export, and source-layout guards recognize the
new owner. This is guard maintenance, not an additional test or semantic
owner. Source/spec and module-layout inventories become active only with the
implementation commit.

The runner owns exact raw-source/Surface authentication in one private dormant
leaf with no public dispatch. Existing Task-258B3N node ownership remains in
`source_statement`; the new module adds no node kind/role and cannot own
Task-269B+, Tasks 270--272, proof/fact/IR/VC behavior, or corpus artifacts.
Docs-time production manifests remain checker `29/162347` and runner
`36/69417`; implementation will add one source path to each and remeasure all
line/path/content hashes.

## Task 269B module-boundary no-op

The committed Task-269A baseline is checker `30/164419` and runner
`37/69729`. Task 269B adds no module, path, public surface, or dependency. The
existing checker owner accepts one more private exact lower profile and the
existing runner leaf adds one private selector branch. Parser/Surface
authentication stays runner-owned; syntax-free transaction validation stays
checker-owned. Path counts remain `30/37`, with lines/content remeasured only
after implementation.

## Task 269B implemented module boundary

Implementation remains inside the existing owners: checker
`source_proof_local_declaration.rs`, adjacent private tests in
`source_statement.rs`, and the existing `TypedAst` installer allowlist; runner
uses its existing private proof-local production/test leaves. No module,
export, dependency, public surface, or active dispatch changed. Checker source
is `30/165219` with path/content hashes
`c89f43f6abebf7ebeb3ac9394ecd8ea3186ad28934c75526d2cc0b85a66ebad5` /
`1fb5ea739c810ff66ed551b359ffa7cbb26265c0057fa18f5128ee5966bad958`;
runner production is `37/69872` with
`1f9e2c9c6589412d832eb92015d913c1b2e0f1309cba9c5c991e08b04d67a73d` /
`1cdc1720e687890e9007664a2140a10194b7637add9b8eb589ee92d9cce3a771`.

## Checker Task 269CP frozen module boundary

The documentation prerequisite changes no module. The implementation target
is limited to the existing `mizar-test` source-statement production leaf, the
existing test-only re-export facade `type_elaboration.rs`, the existing test-
only root facade import in `runner.rs`, and the proof-local runner test file.
`mizar-checker` stays at
30 production paths and 482 tests. The implemented runner stays at 37
production paths / 71,194 lines with path/content SHA-256
`1f9e2c9c6589412d832eb92015d913c1b2e0f1309cba9c5c991e08b04d67a73d` /
`4dcfc69a867dea5c12457d94825493a8a48e4fd5ac7b91d86412371ac25f6b03`.
Its library is 540 tests with raw/normalized test-list SHA-256
`8b9a2b9ea4aad3c6ed0b6eae32a0285d6a9fe1b5389dcc31ebc7adb872317522` /
`a8955748da86930f3e2165637e170d68c77756cbc03f3ff38b3f8de0d21cbc50`.
A new checker module or parser/resolver edit remains outside 269CP.

## Checker Task 269C frozen module boundary

No new module/path or dependency is added. The exact source scope is existing
checker `source_proof_local_declaration.rs`, `typed_ast.rs`, and
`resolved_typed_ast.rs`; existing runner proof-local leaf, two test-only
facades, and proof-local test leaf. Raw source/Surface/resolver selection stays
in `mizar-test`; only the frozen syntax-free input and public BindingEnv cross
to checker. Production paths remain checker/runner `30/37`, baseline lines
`165219/71194`, and path hashes remain
`c89f43f6abebf7ebeb3ac9394ecd8ea3186ad28934c75526d2cc0b85a66ebad5` /
`1f9e2c9c6589412d832eb92015d913c1b2e0f1309cba9c5c991e08b04d67a73d`.
Parser/resolver, Cargo, active dispatch, fixture/trace, and source-type owners
are excluded.

## Checker Task 269C Implemented Module Boundary

Implementation changes exactly the frozen seven Rust source files and adds no
path, module, dependency, or public runner route. Production remains 30/37
paths and measures `167058/71412` lines with unchanged path hashes and content
hashes `d5d6c3bf...` / `bf8c5a24...`. Parser/resolver, source-type, Cargo,
active corpus/trace, and semantic owners remain outside the boundary.

## Task 269CT Frozen Boundary

The documentation prerequisite changes design records only. Later
implementation owns exactly checker `source_type.rs`, `typed_ast.rs`, and
`resolved_typed_ast.rs`, plus four existing dormant-runner/facade/test files.
Production paths stay `30/37`; parser, resolver, fixtures, expectations,
trace, metadata, Cargo, diagnostic codes, public dispatch, and semantic owners
are excluded.

## Task 269CT Implemented Boundary

The implementation changes exactly the frozen seven Rust files and adds no
module, path, dependency, Cargo edge, runner export, dispatch arm, or corpus
owner. Checker/runner production is `30/168322` and `37/71647`; unchanged path
hashes are `c89f43f6abebf7ebeb3ac9394ecd8ea3186ad28934c75526d2cc0b85a66ebad5`
and `1f9e2c9c6589412d832eb92015d913c1b2e0f1309cba9c5c991e08b04d67a73d`,
with content hashes
`4d0c793a47dac672e5f395c9c2b9e7c9274b5d776b54870888ba5c918f751dc2`
and `0f8f5926b9bee23c92d1f05e9cc9e85b4c0561b543e9e0a1e4c825f43b6c5798`.

## Task 269GP Frozen Boundary

The docs prerequisite changes no source. Later implementation changes only
four existing runner files; checker/parser/resolver modules, paths,
dependencies, and public APIs remain unchanged. Production paths stay `30/37`
and checker tests stay `490`; runner tests project `548 -> 552`.
Repeated source/docs and final-quality reviews report **NO FINDINGS** and
confirm this exact boundary.

The implementation now occupies exactly those four existing runner files.
No module/path/dependency/public-API change occurred. Runner tests measure
`552` and production measures `37/72916`; checker production/tests remain
`30/168322` and `490`. The bounded `source_drift` is closed without crossing
the frozen boundary.

## Task 269GS Documentation Boundary

The former canonical blocker is resolved entirely in paired specification and
design documents. Checker and runner module paths, dependencies, visibility,
public APIs, production inventories, and test binaries remain byte-identical.
Binding implementation remains owned by the separate Task 269G; type admission
remains Task 269GT.

## Task 269G Boundary Delta

The checker owns the new syntax-free public binding family and `GivenWitness`
kind; the runner owns exact lower/base assembly and a private dormant consumer.
Raw AST/resolver/source values stop at the runner. Only eight existing Rust
files may change, with no module/path/Cargo edge. Type/condition/fact/proof and
all downstream owners remain outside the boundary.

## Task 269GT Frozen Boundary

Implementation may change exactly
`crates/mizar-checker/src/source_type.rs`,
`crates/mizar-checker/src/typed_ast.rs`,
`crates/mizar-checker/src/resolved_typed_ast.rs`,
`crates/mizar-test/src/runner/type_elaboration/source_proof_local_declaration.rs`,
`crates/mizar-test/src/runner/type_elaboration.rs`,
`crates/mizar-test/src/runner.rs`, and
`crates/mizar-test/src/runner/tests/type_elaboration/source_proof_local_declaration.rs`.
The facade hops remain test-only. No module/path/Cargo edge is added. Binding/
lower/parser/resolver/corpus artifacts remain outside the write boundary;
semantic and downstream owners remain excluded.

### Task 269GT implemented boundary

Implementation changed exactly the frozen seven Rust files and added no module, Cargo, parser, resolver, lower-stage, fixture, or dispatch edge. Checker/runner production are `30/171383` and `37/73351`; path hashes remain `c89f43f6abebf7ebeb3ac9394ecd8ea3186ad28934c75526d2cc0b85a66ebad5` / `1f9e2c9c6589412d832eb92015d913c1b2e0f1309cba9c5c991e08b04d67a73d`, and content hashes are `4a2635cbde94426652d75bfad176d9f167242630d6e1996ab4087ddf14e20abf` / `747a923200a6c23c58adfca7211c82724ff83e1a808b3e045cc73027054f4d07`.

## Task 269GUP Frozen Boundary

The implementation may change exactly six existing Rust files: checker
`source_proof_local_declaration.rs`; runner `source_statement.rs`,
`source_proof_local_declaration.rs`, `type_elaboration.rs`, `runner.rs`, and
the existing proof-local test leaf. Binding production remains with its
established owner. `source_type.rs`, both source-term leaves, Typed/final,
parser/resolver production, modules, Cargo, artifacts, dispatch, and CLIs are
excluded. Pre-implementation production remains `30/171383` and `37/73351`
with the recorded path/content hashes; path/module/dispatch inventory stays
fixed while changed production content is remeasured after implementation.
### Task 269GUP implemented binding profile

The frozen six-file transaction and its exact four checker/four runner tests are implemented. Libraries measure `502/564`; checker/runner production is `30/172531` and `37/74826`, with unchanged path hashes and content hashes `e0342952a01a0b379cf7b06ad243cd40a1656e940480196323cf43fbe7d8f7c5` / `8fe7c8c0b7e855e5113f3830873e133f42c8048a3272055e2fddd5ebd9cbb1bc`.

This closes only dormant private lexical-binding evidence and grants zero active corpus, trace, type, term/use, condition/fact, goal/proof, obligation, diagnostic, or CLI credit. Task 269GUPT is next; Task 269GU, capture, and Task 270 remain deferred.

## Task 269GUPT Frozen Module Boundary

The implementation boundary is exactly checker `source_type.rs`, `typed_ast.rs`, `resolved_typed_ast.rs` and runner `type_elaboration/source_proof_local_declaration.rs`, `type_elaboration.rs`, `runner.rs`, and the existing proof-local test leaf. Checker `source_proof_local_declaration.rs`, `binding_env.rs`, `source_term.rs`, runner `source_statement.rs`, parser/resolver, public dispatch, canonical artifacts, trace, metadata, Cargo, and diagnostics are excluded. Production path counts remain `30/37`; docs-only baseline lines are `172531/74826`.

### Task 269GUPT implemented module boundary

The exact seven-file boundary is preserved. Checker production is now
`30/174332`, runner production `37/75074`; path inventories are unchanged.
No excluded owner, dispatch, artifact, Cargo target, or diagnostic changed.

## Task 269GU Frozen Module Boundary

Owned checker files are exactly `source_term.rs`, `typed_ast.rs`, and
`resolved_typed_ast.rs`; owned runner files are the proof-local declaration
leaf, its two test-only facade hops, and its existing test leaf. `source_type.rs`,
`source_proof_local_declaration.rs`, `binding_env.rs`, runner
`source_statement.rs`, parser/resolver, dispatch, artifacts, metadata, Cargo,
and diagnostics are excluded. Production path counts remain `30/37`.

### Task 269GU implemented module boundary

The exact seven-file boundary is preserved. Checker production is now
`30/176258`, runner production `37/75339`; path inventories are unchanged.
No excluded owner, dispatch, artifact, Cargo target, or diagnostic changed.

## Task 269GCP Frozen Boundary

Only four existing `mizar-test` runner files may change in implementation.
`mizar-checker`, resolver, parser, fixtures, metadata, Cargo targets, public
dispatch, and every active artifact are read-only. Adding a public checker
owner, loosening GUP/GUPT/GU, or reconstructing a binding above the future GC
owner is a `boundary_violation`.

### Task 269GCP implemented boundary

Only the frozen four existing `mizar-test` files changed. Checker production
remains `30/176258`; runner production is `37/76642` with unchanged path hash
`1f9e2c9c6589412d832eb92015d913c1b2e0f1309cba9c5c991e08b04d67a73d`
and content hash
`adaeaad8bf2943e05e402f1bc565b5bb0f9a509fb74ffdcd9bbb05eab4d86b22`.
No excluded owner, artifact, dispatch, Cargo target, or diagnostic changed.

## Task 269GC Frozen Boundary

Runner alone owns Surface/shell/resolver/source text and converts the exact GCP
row into syntax-free input. Checker owns the distinct public GC producer,
binding transaction, and Typed/final replay. The ABI carries only identity,
ranges, lower fingerprint, `LocalTermBinding`, and reserve `BindingEnv`.
Condition/type/term syntax, occurrence IDs, facts, proof state, diagnostics,
and active dispatch do not cross the boundary. G/GUP/GCP remain immutable.

### Task 269GC implemented boundary

Exactly the frozen three checker and four runner files changed. Checker module
sizes are `source_proof_local_declaration.rs=6660`, `typed_ast.rs=6281`, and
`resolved_typed_ast.rs=8340`; checker production is `30/177771`, with path hash
`c89f43f6abebf7ebeb3ac9394ecd8ea3186ad28934c75526d2cc0b85a66ebad5`
and content hash
`4e9220617eac3d5e993c2cee6adfb4958e4cb70e9ddbec83fb0c8955c86aa9fd`.
Runner production is `37/76863`, with path hash
`1f9e2c9c6589412d832eb92015d913c1b2e0f1309cba9c5c991e08b04d67a73d`
and content hash
`6efd7b0ecf2f94b6440b910cef62796e5093d8618ffe196d17ce99bd2245619f`.
No excluded syntax, artifact, dispatch, diagnostic, Cargo, or semantic owner
crossed the frozen boundary.

## Task 269GCT Frozen Module Boundary

Checker `source_type.rs` exclusively validates the by-value GC dependency,
creates the type-site overlay, validates the common source-type input/arena,
and owns the immutable composite. `typed_ast.rs` and `resolved_typed_ast.rs`
only install/replay it. The proof-local runner reads GC/GCP getters and creates
syntax-free input; it never owns validation or exposes the composite publicly.
No parser/resolver/lower module, `binding_env.rs`, active dispatcher, artifact,
metadata, diagnostic, Cargo file, condition occurrence, semantic table, or
downstream IR crosses this boundary.

### Task 269GCU implementation status

After documentation prerequisite `15f47a837bc2f52d4cd30e8a4dcb86c16f2961d3`,
the seven frozen implementation files, one `cfg(test)`-only predecessor
ownership-sentinel support file, and four checker/four private runner tests are
present. The support seam closes the review-discovered Task-269A both-order
`test_gap` without changing production API or behavior. The public family is
`SourceProofLocalGivenConditionUseTerm{Handoff,Producer,Error}`; Typed and
Resolved own the same boxed composite atomically. Libraries are `522/588`.
Checker production is `30/181154`, with unchanged path hash
`c89f43f6abebf7ebeb3ac9394ecd8ea3186ad28934c75526d2cc0b85a66ebad5`
and content hash
`f9901821c2242bfe66321c57982b54b78425c7940c5a7c47c93c43a8c2c035dc`.
Runner production is `37/77435`, with unchanged path hash
`1f9e2c9c6589412d832eb92015d913c1b2e0f1309cba9c5c991e08b04d67a73d`
and content hash
`0651af8339c147d04f88be237f8f49fc716b7da3ff90238be50a9527e89992b7`.
Raw/normalized test-list hashes are checker
`d453ca1e8a7cf9870f14a0f933451ca201c19cc8c8367d51767c40a941766f82` /
`7cd84f6cd8e6d1070b39be9e5f1031512cc2c1b664829f10d337f1b67bcb74b3`
and runner
`7a99bcbb35838b6c1df31dec7b7c70d9c569df86bdc6f5c68d72f41578be2a9e` /
`e49dac17564f330ad5c73018538bf5736720e47f4833709c1b9d36622208888a`.

The implementation closes only the two frozen own-condition `y` term/reference
occurrences. The authoritative block-scope decision makes a `given` binding
visible through the remainder of its innermost block and descendant blocks,
subject to inner shadowing, but descendant-use/capture implementation remains
a separate successor. No canonical specification, `.miz`, fixture, sidecar,
expectation, trace row/status/backlink, metadata, diagnostic, public dispatch,
CLI byte, active result, or semantic credit changed. Equality/formula/fact,
guard, goal, proof/obligation/acceptance, export/capture enforcement,
downstream IR, and Task 270 remain deferred. Independent test-sufficiency,
implementation, and source/documentation reviews report **NO FINDINGS**.
Final read-only quality reports **NO FINDINGS**: all nine hard gates PASS
without a score cap at `100/100`. Focused and full measured gates pass.
Exact staging and the implementation commit remain.

### Task 269GCT implementation status

After documentation prerequisite `b43081161b31fcc4bc23ac2fd42c5c42e772ab78`,
the exact seven-file implementation and four checker/four private runner tests
are present. The new public checker family is
`SourceProofLocalGivenConditionType{Handoff,Producer,Error}`; Typed and
Resolved own the same boxed composite atomically. Libraries are `518/584`.
Checker production is `30/179612`, with unchanged path hash
`c89f43f6abebf7ebeb3ac9394ecd8ea3186ad28934c75526d2cc0b85a66ebad5`
and content hash
`8078ee6235c8ca52ce8cdba0be9a347231260d3421c54625a3fc96cf395c9718`.
Runner production is `37/77159`, with unchanged path hash
`1f9e2c9c6589412d832eb92015d913c1b2e0f1309cba9c5c991e08b04d67a73d`
and content hash
`5b0e68f35d37fcf843f7cb64885f09bfa9dd5423c17506713e096811a5ddf689`.
Raw/normalized test-list hashes are checker
`6d10b524115a209f198bc5085a726bc1fcc6f92dc3e25a8056e29975b708b656` /
`502f7535a34b9d2224c67e6db15f4eaf45f05eec2a2fe4c914704ecf162d89b2`
and runner
`d599bd69654d000f44858942cec771742d8c3c9e0d2ca459d7fecc84d76752c9` /
`bc3cdabbc6424b0f01d817ed323dd823ff57d1d8d4261220dc3d9c37d9004a61`.

The implementation changes no canonical specification, `.miz`, fixture,
sidecar, expectation, trace row/status/backlink, metadata, diagnostic, public
dispatch, CLI byte, active result, or semantic credit. GCU still owns both
condition occurrences and every wider semantic effect. Independent test-sufficiency, implementation, source/documentation, and
final-quality reviews report **NO FINDINGS**. All nine hard gates PASS with no
score cap at `100/100`; focused and crate suites, lint policies, formatting,
Clippy, workspace tests, metadata, all five CLIs, count/hash oracles, and diff
checks pass. Dedicated implementation commit
`d6fb0ed28ced4d4706a1793b3aedd2a20eea0749` is complete.

## Task 269GCU Frozen Module Boundary

`source_term.rs` exclusively validates the by-value GCT dependency, exact
two-term/two-reference input, private binding profile, six-node arena, and
immutable composite. `typed_ast.rs` and `resolved_typed_ast.rs` only install
and replay it. The proof-local runner constructs syntax-free input and remains
private. No parser/resolver/lower module, `binding_env.rs`, active dispatcher,
artifact, metadata, diagnostic, Cargo file, formula/fact/condition table, or
downstream IR crosses this boundary.
