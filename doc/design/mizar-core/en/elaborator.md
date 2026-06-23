# Elaborator

> Canonical language: English. Japanese companion:
> [../ja/elaborator.md](../ja/elaborator.md).

Status: `mizar-core` task 7 module specification. This document is the
normative task-local design for tasks 8-13. It refines
[architecture 06](../../architecture/en/06.elaboration_and_core_ir.md) and
uses [core_ir.md](./core_ir.md) and
[binder_normalization.md](./binder_normalization.md) as its output contracts.

## Purpose

`elaborator` lowers checker-owned `ResolvedTypedAst` material into
backend-neutral `CoreIr`. It is the last source-shaped semantic boundary:
inputs still preserve source expression metadata, overload records, inserted
views, type facts, diagnostics, and recovery status; outputs use canonical core
symbols, binder-normalized terms/formulas, explicit type predicates, definition
boundaries, proof skeletons, algorithm shells, source maps, and obligation
seeds.

The module must not inspect raw syntax, re-run resolver/checker/registration
closure/overload selection, run proof search, build CFGs, assign `VcId`s,
declare proof acceptance, or invent artifact/kernel schemas.

## References

- [architecture 06](../../architecture/en/06.elaboration_and_core_ir.md):
  phase-9 Step 1 through Step 6.
- [architecture 05](../../architecture/en/05.overload_resolution.md):
  overload selection, inserted `qua` views, and `ResolvedTypedAst` boundary.
- [architecture 08](../../architecture/en/08.reasoning_boundary.md):
  Mizar-side semantic processing before ATP/proof dispatch.
- [architecture 16](../../architecture/en/16.substitution_and_binding.md):
  binder identity, capture avoidance, and deterministic replay.
- [mizar-checker resolved typed AST](../../mizar-checker/en/resolved_typed_ast.md):
  source-shaped final semantic input.
- [mizar-checker source/spec audit](../../mizar-checker/en/source_spec_audit.md)
  and [crate exit report](../../mizar-checker/en/crate_exit_report.md):
  deferred upstream payload gaps and completed checker boundary.
- [core_ir.md](./core_ir.md): core data shape, generated origins, diagnostics,
  source maps, definitions, proof nodes, algorithms, and obligation seeds.
- [binder_normalization.md](./binder_normalization.md): normalized binder
  representation, guards, generated semantic records, substitution, and
  alpha-equivalence.

## Responsibility

`elaborator` owns:

- validating that checker-provided semantic records are sufficient for core
  lowering;
- preparing the current-module core context, including canonical item ids,
  definition boundaries, local binder contexts, source-map builders, generated
  origin registries, and dependency summaries;
- lowering normalized checker types and type facts to explicit core type
  predicates and assumptions;
- lowering resolved terms and formulas to binder-normalized core nodes;
- recording source-written and inserted `qua` views as explicit provenance and
  already-established type facts;
- lowering structures, modes, attributes, predicates, functors, theorems,
  schemes, registrations, reductions, and algorithms to core item shells;
- recording definition expansion policies, definition-time closures, formal
  type guards, generated-origin dependencies, and correctness obligation seeds;
- lowering proof skeletons into core proof trees with explicit thesis
  replacement, assumptions, labels, citations, and terminal goal seeds;
- lowering algorithm bodies only as core algorithm statement shells with
  contracts, ghost/runtime metadata, and termination terms;
- preserving explicit diagnostics and error/skipped nodes for failed semantic
  sites.

`elaborator` does not own:

- source-to-checker extraction or AST walking;
- name resolution, type checking, cluster saturation, registration activation,
  overload candidate selection, or template inference;
- proof acceptance, ATP dispatch, kernel checking, or certificate production;
- `VcId` assignment, VC slicing, or control-flow graph construction;
- artifact serialization, cache reuse anchors, public diagnostic-code registry
  allocation, or stable external schema publication.

## Input Contract

Task 8-13 implementations consume explicit checker-owned inputs only:

- `ResolvedTypedAst` nodes, expression metadata, overload records, inserted
  coercions/views, candidate summaries, diagnostics, and recovery state;
- checker type/fact rows referenced by `ResolvedTypedAst`;
- accepted cluster/reduction trace rows already materialized by checker;
- canonical `SymbolId`s from resolver/checker outputs;
- dependency core summaries supplied by the caller;
- caller-owned source-map spans from `ResolvedTypedAst`.

Missing checker payloads are not filled by source inspection. They are
classified as `external_dependency_gap` when they require upstream extraction or
an unavailable external/downstream crate. Work that is core-owned but scheduled
for a later `mizar-core` task is classified as `deferred`.

## Output Contract

Successful elaboration produces a `CoreIr` with:

- deterministic current-module item order;
- explicit source maps for all core nodes that can later produce diagnostics,
  obligations, metadata, snapshots, artifacts, or source-mapped downstream
  records;
- core terms/formulas whose binders satisfy `binder_normalization.md`;
- definition records with expansion policy and correctness seeds;
- proof records with terminal goal seeds, not accepted proof status;
- algorithm records with statement shells, not CFG blocks;
- generated origins for local abbreviations, comprehensions, type predicates,
  error placeholders, and algorithm picks where the core shape requires them;
- structured diagnostics for unsupported or malformed lowering.

Failed semantic sites remain explicit `Error` nodes or skipped/error items. A
failed overload, missing type fact, unsupported source form, malformed proof
skeleton, or unavailable algorithm metadata must never become a valid core term
or formula.

## Gap Classification

| ID | Class | Description | Task-7 decision |
|---|---|---|---|
| ELAB-G001 | `spec_gap` | `elaborator.md` did not exist before task 7. | This document closes the module-spec gap for tasks 8-13. |
| ELAB-G002 | `test_gap` | No `src/elaborator.rs` tests exist before task 8. | Tasks 8-13 add Rust fixtures per section. |
| ELAB-G003 | `external_dependency_gap` | Source-to-checker extraction for full semantic payloads is still deferred by checker closeout. | Use explicit `ResolvedTypedAst` fixtures; do not scan source or fabricate payloads. |
| ELAB-G004 | `external_dependency_gap` | Artifact schema and cache reuse anchors are outside `mizar-core`. | Preserve provenance and dependency summaries only. |
| ELAB-G005 | `external_dependency_gap` | Proof acceptance, VC generation, kernel checking, and certificate schemas are downstream crates. | Emit obligation seeds and proof skeletons only. |
| ELAB-G006 | `deferred` | Source-derived `.miz` core snapshots require mizar-test stage support. | Use Rust fixtures until staged source-to-core snapshots exist. |
| ELAB-G007 | `source_drift` | Before task 8, source had no `elaborator` module. | Task 8 introduced the module; tasks 9-13 keep closing the remaining implementation slices. |

## Step 1: Core Context Preparation

Task 8 implements this section.

Input:

- `ResolvedTypedAst`;
- dependency core summaries supplied by the caller;
- resolver/checker canonical symbol identities;
- checker diagnostics and recovery records.

Output:

- `CoreContext`;
- current-module item registry;
- definition boundary registry;
- generated-origin registry;
- binder and variable metadata context;
- source-map and diagnostic builders;
- deterministic elaboration worklist.

Rules:

- Assign `CoreItemId`s in deterministic source/module order after recording
  source-order diagnostics.
- Store canonical `SymbolId`s, never raw source spellings, as item and
  reference identity.
- Record dependency summaries as read-only inputs. Missing summaries for a
  required reference are `UnsupportedLowering` or `UnresolvedSemanticInput`
  diagnostics, not a reason to inspect source.
- Initialize definition boundaries before lowering bodies so recursive and
  mutually dependent references can be represented without eager inlining.
- Create binder contexts from checker-resolved binder ids, roles, sorts, and
  type facts. Display names are diagnostic-only.
- Preserve failed `ResolvedTypedAst` sites in an error/skipped worklist state.

Tests for task 8 must cover deterministic item ids, dependency-summary absence,
canonical ids never using raw spelling, failed overload/worklist preservation,
and source-map builder initialization.

## Step 2: Type And Fact Lowering

Task 9 implements this section.

Input:

- normalized checker type rows;
- visible type facts and cluster facts;
- inserted views and source-written `qua` metadata;
- checker initial obligations and deferred evidence rows.

Output:

- `CoreTypePredicate` applications;
- core assumptions and guard formulas;
- view explanation provenance;
- carried obligation seed references.

Erasure rules:

- A declared binder type `x be T` becomes a core binder for `x` plus a
  `TypePred { subject: x, ty: pred(T) }` guard/assumption.
- A formula assertion `x is T` becomes the same `TypePred` formula.
- Attribute chains lower to conjunctions of explicit predicate facts in
  deterministic predicate order. Negative polarity lowers to `Not(TypePred)`.
- Mode/radix expansions lower through the checker-normalized type head and
  must not reconstruct type syntax.
- A source-written `qua` and an inserted view lower in this step only to view
  provenance and already-established type facts for the variable-subject fact
  being erased. The underlying term remains a Step 3 lowering responsibility.
  They are not new proof steps.
- Reconsider/narrowing payloads become a fresh or narrowed core binding plus
  a carried obligation seed when the checker supplied one.
- Missing sethood, non-emptiness, coercion, or cluster evidence becomes a
  diagnostic/error node or deferred seed; the elaborator must not prove it.

Tests for task 9 must cover each erasure rule, positive/negative attribute
polarity, inserted and source-written `qua`, missing evidence diagnostics, and
deterministic conjunction ordering.

## Step 3: Term And Formula Lowering

Task 10 implements this section.

Input:

- resolved expression metadata and overload resolution records;
- lowered type predicates/facts from Step 2;
- binder contexts from Step 1;
- generated-origin registry.

Output:

- `CoreTermTable` rows;
- `CoreFormulaTable` rows;
- expression source-map entries;
- explicit diagnostics for failed sites.

Term rules:

- Variables lower by stable checker/resolver `CoreVarId` and then normalize
  through `binder_normalization`.
- Constants and selected functor roots lower to canonical `SymbolId`s.
- Applications lower to `CoreTermKind::Apply` with already selected roots and
  lowered arguments.
- Selectors, tuples, and set enumerations lower to their explicit core nodes.
- Source-written or inserted `qua` lowers to the underlying term plus view
  provenance and type facts; it does not create an implicit cast node.
- Stable choice terms lower to ordinary `Apply` nodes whose functor is a
  generated choice symbol. `CoreTermKind::Generated` must not be used for
  stable choices.
- Stable choice generated symbols are keyed by the owning core item/proof or
  definition context, alpha-normalized target type, and explicit free
  parameters. The same owner/key pair reuses the same generated symbol.
- Fraenkel comprehensions lower to generated set-valued symbols with
  alpha-normalized generator/mapper/predicate keys and explicit free-parameter
  payloads.
- Algorithm statement picks lower to `CoreAlgorithmStmtKind::Pick` in Step 6,
  not to shared stable choice symbols.

Formula rules:

- Predicate applications, equality, type assertions, attributes, connectives,
  and quantifiers lower to explicit `CoreFormulaKind` rows.
- Quantifier binders are processed left to right. A binder's guard may see
  prior binders and itself, but not later binders.
- Failed overload sites, missing selected roots, malformed type evidence, or
  unsupported surface forms lower to `Error(CoreDiagnosticId)` and never to a
  valid logical node.
- Generated term semantic records must include owner, kind, key, normalized
  params, and normalized arguments; evidence/source text is provenance only.

Tests for task 10 must cover variables, constants, applications, selectors,
tuples, set enumerations, predicates, equality, type predicates, connectives,
quantifier guards, inserted/source `qua`, stable choice, Fraenkel
comprehension, failed overload preservation, and generated-key determinism.
Stable-choice fixtures must assert that `the T` lowers to an ordinary
`Apply(choice_T(params))` generated symbol rather than `CoreTermKind::Generated`.
Fraenkel fixtures must assert that required sethood evidence is preserved as
explicit provenance or obligation input, and that missing sethood evidence
remains an error/deferred seed instead of a fabricated valid set term.

## Step 4: Definition Lowering

Task 11 implements this section.

Input:

- resolved declarations and signatures;
- lowered terms/formulas;
- checker correctness/deferred obligation metadata;
- dependency and visibility summaries.

Output:

- `CoreItem` rows for structures, modes, attributes, predicates, functors,
  theorems, schemes, registrations, reductions, generated definitions, and
  algorithms;
- `CoreDefinitionTable` rows;
- correctness obligation seeds;
- generated dependencies.

Rules:

- Definition boundaries are stable and registered before bodies are lowered.
- Visibility and export metadata from resolved declarations are preserved on
  the resulting `CoreItem` rows and dependency summaries.
- The elaborator records expansion policy (`Opaque`, `Transparent`,
  `Reducible`, or `Computable`) but does not eagerly inline definitions.
- Formal binders include type guards normalized under the binder-scope rule.
- `set`, `deffunc`, and `defpred` uses expand from definition-time closures
  using capture-avoiding substitution. Captured free variables are stable ids,
  not display names.
- Conditional definitions lower to ordered guarded branches. `otherwise` is
  represented as the negation of earlier guards, and missing coverage remains
  an explicit obligation/diagnostic.
- Existence, uniqueness, coherence, compatibility, reducibility, and coverage
  checks become obligation seeds. They are not accepted in this crate.
- Items with failed prerequisites are skipped or marked error with structured
  diagnostics.

Tests for task 11 must cover definition boundary ordering, expansion policies,
formal guards, local abbreviation closure expansion, conditional branches,
correctness seed emission, generated dependencies, and skipped/error item
preservation. Exported definitions that contain stable choices must reuse the
definition-owned generated choice symbols when unfolded; uses must not
regenerate fresh choice symbols at the unfolding site.

## Step 5: Proof-Skeleton Lowering

Task 12 implements this section.

Input:

- theorem/lemma propositions;
- checker proof skeleton payloads when available;
- proof labels and citations;
- lowered formulas and binder contexts.

Output:

- `CoreProofTable` rows;
- core proof nodes;
- theorem/proof status metadata;
- terminal proof goal obligation seeds.

Rules:

- `thesis` is replaced by the current core formula. It is not preserved as a
  magic identifier.
- Introduced variables become `CoreBinder`s with type guards.
- Assumptions and labeled steps become explicit proof nodes with source/core
  provenance.
- Citations reference labels, canonical symbols, or generated origins already
  present in the semantic input.
- Branching proof forms such as cases, suppose, and now preserve structure and
  produce terminal goal seeds at open proof leaves.
- `open`, `assumed`, `conditional`, and `error` statuses are recorded; none of
  them proves a theorem in `mizar-core`.
- Malformed or missing proof skeleton payloads produce
  `MalformedProofSkeleton` diagnostics and error proof nodes.

Tests for task 12 must cover thesis replacement, introduced binders,
assumptions, labels/citations, branch kinds, terminal goal seeds, theorem
statuses, and malformed proof skeleton diagnostics. Theorem and lemma
proposition fixtures must include stable choices whose generated symbols are
owned by the theorem/lemma proposition context and are preserved through
proof-skeleton lowering.

## Step 6: Algorithm-Shell Lowering

Task 13 implements this section.

Input:

- resolved algorithm declarations;
- lowered contract formulas and termination terms;
- ghost/runtime metadata;
- source-shaped resolved statement payloads.

Output:

- `CoreAlgorithmTable` rows;
- `CoreAlgorithmStmtTable` statement shells;
- contract sets;
- algorithm diagnostics and generated pick origins.

Rules:

- Parameters and optional result binders are recorded with roles, guards, and
  source maps.
- `requires`, `ensures`, `assert`, `invariant`, and `decreasing` clauses lower
  to core formulas or terms in phase 9.
- Statement order and nesting are preserved, but basic blocks and CFG edges are
  deferred to `control_flow.md`.
- Every `CoreAlgorithmStmt` row, including nested statements, records the
  containing algorithm as its owner. Phase 10 may trust this owner relation.
- Assignments preserve resolved target places and lowered value terms.
- `if`, `while`, `match`, `return`, `break`, and `continue` remain statement
  shells for phase 10.
- Algorithm `pick` statements record local binders and witness type facts as
  `CoreAlgorithmStmtKind::Pick`; they do not reuse stable choice symbols.
- Ghost/runtime classification is preserved as metadata. The elaborator does
  not perform extraction-oriented erasure.

Tests for task 13 must cover parameters/results, contracts, assertions,
invariants, decreasing terms, statement nesting/order, pick bindings,
ghost/runtime metadata, and diagnostics for missing algorithm payloads.
Executable algorithm statement occurrences of `the` must lower to local `Pick`
bindings, not shared stable choice symbols. Ghost-only `Pick` sites must remain
marked as ghost metadata for later phase-10/phase-11 erasure and checking.

## Diagnostics

Elaboration diagnostics use `CoreDiagnostic` rows with deterministic ordering.
The local classes are those in `core_ir.md`:

- `UnresolvedSemanticInput`;
- `InvalidErasure`;
- `UnsupportedLowering`;
- `MalformedProofSkeleton`;
- `SourceMapping`;
- `AlgorithmShell`.

Diagnostics include source provenance from `ResolvedTypedAst`, semantic
provenance such as overload result or inserted view ids, and the owning core
node when one exists. Public diagnostic-code allocation remains an
`external_dependency_gap`; task 7 does not allocate registry codes.

## Determinism

Elaboration must be deterministic across worker counts, map iteration order,
and diagnostic rendering:

- item worklists are sorted by canonical source/module order;
- generated origin keys use alpha-normalized semantic records;
- conjunctions from erasure use stable predicate ordering;
- source-map provenance is sorted by phase and source range;
- diagnostics are emitted in source/core order with stable message keys;
- skipped/error nodes are preserved in traversal order.

## Forbidden Behavior

`elaborator` must not:

- inspect raw source syntax to fill checker payload gaps;
- compare source display names for semantic identity;
- re-run name resolution, type checking, registration activation, cluster
  closure, overload selection, or template inference;
- turn failed semantic sites into valid core terms/formulas;
- eagerly inline every definition;
- assign `VcId`s, build CFGs, run proof search, mark proof acceptance, call the
  kernel, emit artifact schemas, or invent cache/proof reuse anchors.
