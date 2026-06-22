# mizar-checker TODO

> Canonical language: English. Japanese companion: [../ja/todo.md](../ja/todo.md).

## Status Legend

- [ ] not started
- [~] in progress
- [x] done

## Module Implementation

Module specs do not exist yet; each is written by its own spec task (English
and Japanese in the same change) before the implementation tasks that cite it.
Module names follow the minimum split of
[internal 07](../../internal/en/07.crate_module_layout.md); the crate refines
architecture 04, 05, 16, 17, 18, and 19.

| Module | Spec | Source | Status |
|---|---|---|---|
| typed_ast | `typed_ast.md` (task 2) | `src/typed_ast.rs` | [x] |
| binding_env | `binding_env.md` (task 4) | `src/binding_env.rs` | [x] |
| type_checker | `type_checker.md` (task 6) | `src/type_checker.rs` | [~] |
| registration_resolution | `registration_resolution.md` (task 13) | `src/registration_resolution.rs` | [ ] |
| cluster_trace | `cluster_trace.md` (task 15) | `src/cluster_trace.rs` | [ ] |
| overload_resolution | `overload_resolution.md` (task 21) | `src/overload_resolution.rs` | [ ] |
| resolved_typed_ast | `resolved_typed_ast.md` (task 27) | `src/resolved_typed_ast.rs` | [ ] |

`mizar-checker` implements pipeline phases 6-8: `ResolvedAst` plus `SymbolEnv`
in, `TypedAst`, `ResolutionTrace`, and `ResolvedTypedAst` out. It is built in
three waves matching the phases: type checking (phase 6), cluster/registration
resolution with replayable traces (phase 7), and overload resolution (phase 8).
Soft types are semantic metadata: every fact must remain explainable as a
logical predicate or a registration-derived fact, and no wave performs proof
search.

Dependency order: `typed_ast` data → `binding_env` / `type_checker` (wave 1)
→ `registration_resolution` / `cluster_trace` (wave 2) →
`overload_resolution` / `resolved_typed_ast` (wave 3).

Each task below is deliberately small — one module spec, or one behavior slice
of one module — so that a single task can be implemented, tested, and
committed autonomously without holding the rest of the crate in flight.

## Crate Prerequisites

The crate depends on `mizar-session` and `mizar-resolve` (and on
`mizar-syntax` transitively). Wave 1 needs `mizar-resolve` tasks 14 and 20
(name resolution, `SymbolEnv` skeleton); waves grow with `mizar-resolve`
task 21 signature increments and the corresponding `mizar-parser` definition
grammar tasks (23-31). Architecture:
[04.type_and_registration_resolution.md](../../architecture/en/04.type_and_registration_resolution.md),
[05.overload_resolution.md](../../architecture/en/05.overload_resolution.md),
[16.substitution_and_binding.md](../../architecture/en/16.substitution_and_binding.md),
[17.cluster_trace_format.md](../../architecture/en/17.cluster_trace_format.md);
crate ownership: [internal 07](../../internal/en/07.crate_module_layout.md).

## Resolved And Open Decisions

- **TypedAst arena representation: resolved by task 3.** `TypedAst` uses a
  homogeneous `TypedNodeKind` arena with dense local ids, mirroring the current
  `mizar-syntax` compatibility view and `mizar-resolve` arena style. Task 3
  does not add a direct `mizar-syntax` dependency for node-kind storage; it uses
  a checker-local source-shape projection. `ResolvedTypedAst` revisits the same
  decision in task 28.
- **Registration activation gating: open, resolved by task 19.** Local
  registrations must not affect automatic inference until their proof
  obligations are accepted by the configured verifier policy (architecture 04
  constraints). The first iteration needs an interim policy because phases
  11-14 do not exist yet. The default interim policy is no unverified
  activation: generated obligations are recorded with pending/unverified
  status, and registrations do not enter the active database until an accepted
  verifier status is available. Registered at the top level; revisited when
  `mizar-vc`/`mizar-proof` land.
- **Trace schema conformance: resolved.**
  [17.cluster_trace_format.md](../../architecture/en/17.cluster_trace_format.md)
  is the canonical `ResolutionTrace` schema; `cluster_trace.md` refines it,
  it does not fork it.
- **Diagnostics record: follows the `mizar-resolve` decision** on
  `mizar-diagnostics` adoption timing; the checker uses whatever record the
  resolver adopted. Registered at the top level.

## Ordered Task List

Keep `cargo test -p mizar-checker` green after each task (see
[Recommended Verification](#recommended-verification)).

### Wave 1: type checking (phase 6)

1. **Crate scaffold and lint-policy guard.** [x]
   - Add the `mizar-checker` workspace member depending on `mizar-session`
     and `mizar-resolve`; add `tests/lint_policy.rs` mirroring the
     `mizar-frontend` guard.
   - Tests: lint-policy guard passes; workspace builds.
   - Deps: `mizar-resolve` task 5. Spec: architecture 04.
   - Completed by task 1: the crate scaffold, minimal crate root, dependency
     boundary, and lint-policy guard are in place; no checker semantics or
     public APIs beyond the crate boundary were introduced.

2. **Spec: `typed_ast.md`.** [x]
   - Write the `TypedAst` data-shape spec (English and Japanese, no code):
     node arena, `TypeTable`, `TypeFactTable`, `CoercionTable`,
     `InitialObligation` with `InitialObligationId` (never `VcId`), and the
     partial-typing-after-error contract.
   - Deps: 1. Spec: architecture 04 "Typed AST",
     [01.ir_layers.md](../../architecture/en/01.ir_layers.md).
   - Completed by task 2: `typed_ast.md` now defines the logical data shape,
     local context snapshots, table/status invariants, `InitialObligationId`
     boundary, partial-typing recovery, task-3 test obligations, and deferred
     arena-representation decision.

3. **Implement `typed_ast` data shapes.** [x]
   - Implement the arena and tables per task 2, resolving the arena
     representation decision, plus a deterministic debug rendering.
   - Tests: id determinism; table round-trips; rendering stability.
   - Deps: 2. Spec: `typed_ast.md`.
   - Completed by task 3: `src/typed_ast.rs` implements dense ids, the
     homogeneous `TypedNodeKind` arena, local context snapshots, typed/fact/
     coercion/obligation/diagnostic tables, validation, and
     `typed-ast-debug-v1` rendering. Unit tests cover determinism, table
     round-trips, context/status invariants, proof-boundary guards, and stable
     rendering.

4. **Spec: `binding_env.md`.** [x]
   - Write the binding/context spec (English and Japanese, no code): layered
     local type contexts over `SymbolEnv` (module, block, binding layers;
     architecture 04 Step 1) and checker-side bound-variable handling
     consistent with architecture 16 (binder identity, no capture).
   - Deps: 1. Spec: architecture 04 "Step 1",
     [16.substitution_and_binding.md](../../architecture/en/16.substitution_and_binding.md).
   - Completed by task 4: `binding_env.md` now defines the checker-owned
     binding/context boundary, layered context graph, binding identities,
     lookup order, reserved-variable handling, closure metadata expectations,
     diagnostics, deterministic rendering, task-5 test obligations, and
     external dependency gaps.

5. **Binding environment and context build.** [x]
   - Implement context construction over `SymbolEnv` and `ResolvedAst`
     bindings per task 4.
   - Tests: lookup order across layers; reserved-variable contexts; binder
     scoping fixtures; deterministic iteration.
   - Deps: 3, 4, `mizar-resolve` task 20. Spec: `binding_env.md`.
   - Completed by task 5: `src/binding_env.rs` implements the checker-owned
     binding-env data layer, validation, module-shell construction over
     `ResolvedAst` plus `SymbolEnv`, local lookup over explicit binding
     payloads, resolver `NameRefEntry::resolution()` fallback, deterministic
     `binding-env-debug-v1` rendering, and external-gap diagnostics for
     resolver/source-walk payloads that are not currently exposed.

6. **Spec: `type_checker.md`.** [x]
   - Write the checking/inference spec (English and Japanese, no code) with
     named sections the implementation tasks cite: type-expression
     normalization (types as normalized predicates, Step 2), declaration and
     local-binding checking (Step 3), term/formula inference (Step 4),
     coercion candidates and initial obligations, type facts, and
     partial-typing recovery.
   - Deps: 4. Spec: architecture 04 "Step 2"-"Step 4",
     [03.type_system.md](../../../spec/en/03.type_system.md),
     [08.type_inference.md](../../../spec/en/08.type_inference.md),
     [13.term_expression.md](../../../spec/en/13.term_expression.md).
   - Completed by task 6: `type_checker.md` now defines the phase-6 boundary,
     normalized type model, task 7 normalization, task 8 declaration/local
     binding checking, task 9 term/formula inference, task 10 coercion and
     initial-obligation behavior, task 11 fact queries, partial recovery,
     deterministic rendering expectations, and external/deferred gates.

7. **Type-expression normalization.** [x]
   - Implement normalization of surface type expressions into canonical
     predicate form (attribute order, `non`, radix-type handling).
   - Tests: attribute-order canonicalization; idempotent normalization.
   - Deps: 5, 6. Spec: `type_checker.md` (normalization section).
   - Completed by task 7: `src/type_checker.rs` implements
     `TypeNormalizationOutput` with a task-local `NormalizedTypeTable`,
     checker-owned type-expression payload normalization, deterministic type
     ids/debug rendering, explicit mode-expansion provider support, `TypeEntry`
     emission, degraded diagnostics for missing explicit mode-expansion provider
     payloads, and unsupported-payload recovery. Resolver/source-walk site
     extraction and full signature payloads remain external dependencies.

8. **Declaration and local-binding checking.** [x]
   - Check declarations and local bindings (`let`, `reserve`, `set`, …)
     against normalized types; diagnose illegal declarations; keep partial
     output after errors.
   - Tests: per-binding fixtures; diagnostics carry binding ranges.
   - Deps: 7. Spec: `type_checker.md` (declaration section).
   - Completed by task 8: `DeclarationChecker` accepts checker-owned
     declaration/context payloads over `BindingEnv`, attaches normalized types
     to binding declaration sites, builds local type-context snapshots, records
     checked-declaration assumption facts, drops invalid/degraded assumption
     payloads with diagnostics, preserves partial output after illegal
     declarations, and
     emits deferred diagnostics for missing RHS/body/reserve/evidence payloads
     without walking raw syntax or fabricating task-10 obligations.

9. **Term and formula type inference.** [x]
   - Infer types for terms and formulas into `TypeTable`, leaving overload
     roots open where candidates remain (architecture 04 "Overload Candidate
     Filtering Is Allowed, Root Selection Is Deferred").
   - Tests: inference fixtures per term/formula kind the parser produces;
     partially inferred results on type errors.
   - Deps: 8. Spec: `type_checker.md` (inference section).
   - Completed by task 9: `TermFormulaChecker` accepts checker-owned
     term/formula payloads, records per-term `TypeEntry`s, checked-formula
     well-formedness, task-local inference facts, deterministic open candidate
     sets, expected-type constraints, and partial/error/skipped recovery
     without final overload selection, raw syntax walking, `CoercionTable`
     emission, or `InitialObligation` fabrication.

10. **Coercion candidates, sethood, non-emptiness, and narrowing obligations.** [x]
    - Record widening/narrowing/`qua` coercion candidates in `CoercionTable`
      and emit sethood/non-emptiness/narrowing `InitialObligation`s.
    - Tests: candidate sets per coercion kind; obligations carry
      `InitialObligationId` and source ranges. Include fail fixtures for
      missing sethood/non-emptiness evidence and invalid `qua` narrowing.
    - Deps: 9. Spec: `type_checker.md` (coercion/obligation section).
    - Completed by task 10: `CoercionObligationChecker` accepts checker-owned
      coercion and initial-obligation payloads, records widening/source-`qua`/
      narrowing candidates, creates sethood/non-emptiness/narrowing
      `InitialObligation`s with deterministic local ids and source ranges,
      preserves input fact ids for supporting facts, appends obligation-backed
      facts, and leaves missing inheritance/summary/cluster/sethood/
      non-emptiness/proof-query inputs as external dependency gaps rather than
      assigning `VcId`s, discharging obligations, or fabricating inserted views.

11. **Type-fact recording and queries.** [ ]
    - Implement fact recording during inference and the deterministic query
      API later used by registration and overload waves.
    - Tests: fact provenance; query determinism; no fact duplication.
    - Deps: 9, 10. Spec: `type_checker.md` (type-facts section).

12. **Corpus runner at stage `type_elaboration`.** [ ]
    - Wire `tests/miz/{pass,fail}/` cases at stage `type_elaboration` through
      the harness with `spec_trace.toml` entries; seed pass/fail cases for
      tasks 7-11.
    - Deps: 10, 11. Spec: [staged_model.md](../../mizar-test/en/staged_model.md).

### Wave 2: cluster and registration resolution (phase 7)

13. **Spec: `registration_resolution.md`.** [ ]
    - Write the registration spec (English and Japanese, no code): pending
      versus activated databases, existential gating, reduction rewrites
      with provenance, validation obligations (architecture 04 Steps 5-6).
    - Deps: 2. Spec: architecture 04 "Registration Databases",
      [17.clusters_and_registrations.md](../../../spec/en/17.clusters_and_registrations.md).

14. **Registration index.** [ ]
    - Implement the pending/activated registration databases over
      `SymbolEnv` registration declarations.
    - Tests: pending entries never fire; activation moves entries
      deterministically; per-source contribution tracking.
    - Deps: 11, 13, `mizar-resolve` task 21 (registration increment). Spec:
      `registration_resolution.md`.

15. **Spec: `cluster_trace.md`.** [ ]
    - Write the `ResolutionTrace` spec (English and Japanese, no code) as a
      refinement of the canonical schema: cluster steps, reduction steps,
      antecedent facts, audit keys, deterministic traversal, and replay cost
      bounds.
    - Deps: 13. Spec:
      [17.cluster_trace_format.md](../../architecture/en/17.cluster_trace_format.md).

16. **Cluster resolution closure with trace recording.** [ ]
    - Implement attribute propagation to closure (architecture 04 Step 5)
      with deterministic traversal, recording every application into
      `ResolutionTrace`.
    - Tests: closure fixtures; traces replay to the same derived facts;
      deterministic application order.
    - Deps: 14, 15. Spec: `cluster_trace.md`, `registration_resolution.md`.

17. **Cluster loop detection and bounded saturation.** [ ]
    - Detect cluster loops and emit bounded-saturation diagnostics instead of
      diverging (architecture 17 "Cluster Loop Detection").
    - Tests: loop fixtures terminate with stable diagnostics; bound is
      configuration-visible.
    - Deps: 16. Spec: [17.cluster_trace_format.md](../../architecture/en/17.cluster_trace_format.md).

18. **Reduction applications.** [ ]
    - Implement reduction rewrites (redex paths, substitutions, guard
      evidence) with full provenance recorded into `ResolutionTrace`.
    - Tests: redex path correctness; guard evidence required; replayable
      traces.
    - Deps: 16. Spec: `registration_resolution.md` (reduction section),
      architecture 17 "Reduction Step".

19. **Pending-registration validation and activation gating.** [ ]
    - Validate pending registration declarations (architecture 04 Step 6),
      emit their obligations, and implement the interim activation-gating
      policy; record the decision here and at the top level.
    - Tests: invalid registrations diagnosed; unverified registrations never
      affect inference; policy-admitted activation requires accepted verifier
      status from a later proof/artifact input.
    - Deps: 17, 18. Spec: `registration_resolution.md`.

20. **Existential gating of attributed type use.** [ ]
    - Enforce that attributed types are usable only where existential
      registrations justify non-emptiness (architecture 04 "Existential
      Registrations Gate Attributed Type Use").
    - Tests: missing-existential fixtures fail with stable diagnostics.
    - Deps: 19. Spec: `registration_resolution.md`,
      [17.clusters_and_registrations.md](../../../spec/en/17.clusters_and_registrations.md).

### Wave 3: overload resolution (phase 8)

21. **Spec: `overload_resolution.md`.** [ ]
    - Write the overload spec (English and Japanese, no code) with named
      sections: site/candidate collection with provenance, template
      expansion, viability over recorded facts, specificity partial order
      (per-site graphs, no global DAG), root selection and refinement joins,
      `qua` view insertion (widening only, multiple-inheritance ambiguity),
      and failed-site preservation (architecture 05).
    - Deps: 2. Spec: architecture 05,
      [19.overload_resolution.md](../../../spec/en/19.overload_resolution.md),
      [18.templates.md](../../../spec/en/18.templates.md).

22. **Candidate site collection.** [ ]
    - Collect overload sites and candidate sets with provenance from
      `TypedAst` and `SymbolEnv` (already scope/visibility filtered).
    - Tests: site coverage per application form; provenance retained;
      deterministic candidate order.
    - Deps: 11, 21. Spec: `overload_resolution.md` (sites section).

23. **Template expansion.** [ ]
    - Expand template candidates into concrete candidates ahead of ordinary
      candidate ordering; record exclusion reasons for non-expandable
      templates.
    - Tests: expansion fixtures; exclusions carry reasons.
    - Deps: 22, `mizar-parser` task 31. Spec: `overload_resolution.md`
      (templates section).

24. **Viability filtering.** [ ]
    - Filter candidates by viability using recorded type facts only — no new
      inference (architecture 05 "Viability Uses Type Facts, Not New
      Inference").
    - Tests: viability fixtures; rejection reasons preserved for diagnostics.
    - Deps: 23. Spec: `overload_resolution.md` (viability section).

25. **Specificity graph construction.** [ ]
    - Build per-site specificity graphs over viable candidates.
    - Tests: ordering fixtures; incomparable pairs stay incomparable;
      deterministic graph rendering.
    - Deps: 24. Spec: `overload_resolution.md` (specificity section).

26. **Root selection, refinement joins, and view insertion.** [ ]
    - Select overload roots, join coherent refinement groups, insert `qua`
      views, and preserve failed sites explicitly (architecture 05 Step 5).
    - Tests: selection fixtures including refinement joins; ambiguity
      diagnostics with candidate lists; failed sites never become valid
      output.
    - Deps: 25. Spec: `overload_resolution.md` (selection/views sections).

27. **Spec: `resolved_typed_ast.md`.** [ ]
    - Write the `ResolvedTypedAst` data-shape spec (English and Japanese, no
      code): final types, `OverloadResolutionTable`,
      `CoercionInsertionTable`, `ClusterFactTable`, expression metadata.
    - Deps: 21. Spec: [01.ir_layers.md](../../architecture/en/01.ir_layers.md),
      architecture 05 "Step 6".

28. **`ResolvedTypedAst` assembly.** [ ]
    - Assemble the final source-shaped semantic AST with expression metadata
      for LSP and artifacts, plus a deterministic debug rendering.
    - Tests: assembly fixtures; metadata lookup by `ExprId`; rendering
      stability.
    - Deps: 26, 27. Spec: `resolved_typed_ast.md`.

### Hardening and cross-cutting follow-ups

29. **Corpus growth at stages `formula_statement` and `advanced_semantics`.** [ ]
    - Add registration/overload corpus cases (clusters, reductions,
      ambiguity, refinement joins) with `spec_trace.toml` entries; grow
      toward the 40/60 pass/fail mix.
    - Include the review-audit advanced-semantics negative cases: witness
      leakage from `now`/`proof` blocks, unmet `deffunc`/`defpred` guards,
      missing sethood for comprehensions, and invalid `qua` narrowing.
    - Deps: 20, 28. Spec: [staged_model.md](../../mizar-test/en/staged_model.md).

30. **Determinism suite.** [ ]
    - Property coverage that identical inputs produce identical types,
      facts, traces, candidate orders, and diagnostics.
    - Deps: 28. Spec: [20.test_strategy.md](../../architecture/en/20.test_strategy.md).

31. **Public-enum forward-compatibility policy.** [ ]
    - Apply the `mizar-frontend` task-25 procedure to each public enum;
      record decisions in the owning module specs.
    - Deps: 28. Spec: all module specs.

32. **Source/spec correspondence audit.** [ ]
    - Trace every public API and promised behavior in the module specs to
      implementation and tests; record gaps as follow-up tasks.
    - Deps: 31. Spec: all module specs and this TODO.

33. **Bilingual documentation sync audit.** [ ]
    - Compare each English canonical document under
      `doc/design/mizar-checker/en/` with its Japanese companion and
      synchronize content.
    - Deps: 32. Spec: repository documentation policy.

34. **Module-boundary refactor gate.** [ ]
    - Before treating the crate as ready for downstream consumers, audit the
      source layout for oversized files, mixed responsibilities, and private
      helpers that should be split along the module table and spec boundaries.
      Split any review-bottleneck implementation files into private modules
      without changing public APIs, diagnostics, deterministic renderings,
      artifact-facing schemas, or consumer-visible behavior.
    - After any split, update this module table/source paths as needed and
      re-run the source/spec and bilingual documentation audit scopes for the
      moved APIs. Do not mix behavior cleanup or API exposure into the move;
      those require their own spec tasks.
    - Deps: 33. Spec: this TODO,
      [internal 07](../../internal/en/07.crate_module_layout.md), all module
      specs.

## Recommended Verification

Run after each task:

```text
cargo test -p mizar-checker
cargo clippy -p mizar-checker --all-targets -- -D warnings
```

For tasks that touch the resolver boundary or the corpus, also run:

```text
cargo test -p mizar-resolve
cargo test -p mizar-test
```

Check the task off here once tests pass.

## Notes

- The checker owns soft-type facts, replayable registration effects, and
  overload finalization only: no proof search, no ATP premise selection, no
  arbitrary first-order reasoning.
- `VcId`s are never assigned here; phases 6-8 emit `InitialObligationId`s
  that `mizar-vc` later converts exactly once.
- Wave breadth is paced by `mizar-resolve` signature increments and the
  parser definition grammar tasks; do not check declaration kinds the
  resolver cannot yet collect.
- Dependency-slice and fingerprint integration (architecture 18) arrives
  with `mizar-cache`; the checker only has to keep per-source contribution
  tracking accurate so slices stay computable.
