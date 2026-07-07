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
| registration_resolution | `registration_resolution.md` (task 13) | `src/registration_resolution.rs` | [~] |
| cluster_trace | `cluster_trace.md` (task 15) | `src/cluster_trace.rs` | [~] |
| overload_resolution | `overload_resolution.md` (task 21) | `src/overload_resolution.rs` | [~] |
| resolved_typed_ast | `resolved_typed_ast.md` (task 27) | `src/resolved_typed_ast.rs` | [~] |

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
- **Registration activation gating: resolved by task 19.** Local
  registrations must not affect automatic inference until their proof
  obligations are accepted by the configured verifier policy (architecture 04
  constraints). Task 19 implements the interim policy because phases 11-14 do
  not exist yet: generated obligations are recorded with pending/unverified
  status, and registrations do not enter the active database until an explicit
  accepted verifier/artifact status input is available. Registered at the top
  level; revisited when `mizar-vc`/`mizar-proof` land.
- **Trace schema conformance: resolved.**
  [17.cluster_trace_format.md](../../architecture/en/17.cluster_trace_format.md)
  is the canonical `ResolutionTrace` schema; `cluster_trace.md` refines it,
  it does not fork it.
- **Diagnostics record: follows the `mizar-resolve` decision** on
  `mizar-diagnostics` adoption timing; the checker uses whatever record the
  resolver adopted. Registered at the top level.
- **Constructor property value source: resolved by task 35.** Default
  structure constructors accept fields only; `property` values come only from
  Chapter 7 property implementations. Task 35 updates spec 05/07 in English
  and Japanese, adds a reject-first inactive `advanced_semantics` seed, and
  records traceability without changing checker/core source semantics.

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

11. **Type-fact recording and queries.** [x]
    - Implement fact recording during inference and the deterministic query
      API later used by registration and overload waves.
    - Tests: fact provenance; query determinism; no fact duplication.
    - Deps: 9, 10. Spec: `type_checker.md` (type-facts section).
    - Completed by task 11: `TypeFactQueryEngine` answers deterministic
      point queries over existing checker fact tables, respects local
      assumption visibility through optional `LocalTypeContextTable`, returns
      explicit `Satisfied` / `Missing` / `Contradicted` statuses, reports
      contradiction diagnostics without mutating facts, preserves provenance
      for ordering/explanation rather than point-query matching, and leaves
      statement/proof assumption, theorem acceptance, and phase-7 trace facts
      as MC-G019 external dependency gaps.

12. **Corpus runner at stage `type_elaboration`.** [x]
    - Wire a stage `type_elaboration` external-gap fail case through the
      harness with a `spec_trace.toml` entry; defer real task 7-11 semantic
      pass/fail seeds until source-to-checker payload extraction exists.
    - Deps: 10, 11. Spec: [staged_model.md](../../mizar-test/en/staged_model.md).
    - Completed by task 12 as a boundary-preserving runner: the active
      `type-elaboration` harness command runs `.miz` cases through frontend
      parsing and resolver symbol collection, then reports MC-G020
      `type_elaboration.external_dependency.ast_payload_extraction` until an
      AST-wide source-to-checker payload extraction API exists. Real task 7-11
      semantic pass/fail `.miz` assertions are deferred rather than accepted
      through fabricated checker payloads.

### Wave 2: cluster and registration resolution (phase 7)

13. **Spec: `registration_resolution.md`.** [x]
    - Write the registration spec (English and Japanese, no code): pending
      versus activated databases, existential gating, reduction rewrites
      with provenance, validation obligations (architecture 04 Steps 5-6).
    - Deps: 2. Spec: architecture 04 "Registration Databases",
      [17.clusters_and_registrations.md](../../../spec/en/17.clusters_and_registrations.md).
    - Completed by task 13: `registration_resolution.md` now defines the
      phase-7 boundary, pending/activated registration database split,
      validation and `InitialObligationId` rules, existential gating, cluster
      closure, reduction provenance, deterministic diagnostics/recovery,
      planned tests for tasks 14 and 16-20, and MC-G021 external/deferred
      payload gaps without adding source behavior.

14. **Registration index.** [x]
    - Implement the pending/activated registration databases over
      `SymbolEnv` registration declarations.
    - Tests: pending entries never fire; activation moves entries
      deterministically; per-source contribution tracking.
    - Deps: 11, 13, `mizar-resolve` task 21 (registration increment). Spec:
      `registration_resolution.md`.
    - Completed by task 14: `RegistrationDatabase` builds checker-owned
      pending/activated/rejected tables from resolver registration origins,
      preserves origin/visibility/export/contribution metadata, records
      MC-G021 external-gap pending records, rejects malformed and invalid
      activation inputs, accepts only full caller-supplied activation payloads,
      keeps pending/rejected entries from contributing to inference, and
      renders deterministic checker-owned ordering without parsing opaque
      resolver shells.

15. **Spec: `cluster_trace.md`.** [x]
    - Write the `ResolutionTrace` spec (English and Japanese, no code) as a
      refinement of the canonical schema: cluster steps, reduction steps,
      antecedent facts, audit keys, deterministic traversal, and replay cost
      bounds.
    - Deps: 13. Spec:
      [17.cluster_trace_format.md](../../architecture/en/17.cluster_trace_format.md).
    - Completed by task 15: `cluster_trace.md` refines the canonical
      architecture-17 schema without forking it, fixes checker-local
      cluster/reduction step ownership, antecedent fact references, audit
      keys, deterministic traversal, replay-cost bounds, diagnostics, and
      planned tests for tasks 16-18. Source behavior remains deferred to task
      16, and real semantic payloads remain gated by MC-G021.

16. **Cluster resolution closure with trace recording.** [x]
    - Implement attribute propagation to closure (architecture 04 Step 5)
      with deterministic traversal, recording every application into
      `ResolutionTrace`.
    - Tests: closure fixtures; traces replay to the same derived facts;
      deterministic application order; subtype-compatible conditional
      clusters; pending/rejected/unaccepted registrations do not fire.
    - Deps: 14, 15. Spec: `cluster_trace.md`, `registration_resolution.md`.
    - Completed by task 16: `cluster_trace` exposes a checker-owned cluster
      closure data layer over explicit `ClusterRuleInput`/`ClusterFactInput`
      payloads and task-14 activated registrations. It records replayable
      cluster steps, derived closure facts with trace provenance, deterministic
      traversal profiles, and checker-local diagnostics without reductions,
      artifact emission, `TypeFactTable` mutation, or fabricated resolver-shell
      semantics.

17. **Cluster loop detection and bounded saturation.** [x]
    - Detect cluster loops and emit bounded-saturation diagnostics instead of
      diverging (architecture 17 "Cluster Loop Detection").
    - Tests: loop fixtures terminate with stable diagnostics; bound is
      configuration-visible; contradictory derivations are fatal and do not
      export degraded verified facts.
    - Deps: 16. Spec: [17.cluster_trace_format.md](../../architecture/en/17.cluster_trace_format.md).
    - Completed by task 17: cluster closure now tracks fact ancestry/depth,
      diagnoses direct and indirect loops, enforces depth and generated-fact
      bounds with traversal profile/cache-key visibility, reports explicit
      conflict-fingerprint contradictions as incomplete closure results, and
      avoids inserting rejected degraded facts. Source-derived `TypeFactTable`
      contradiction checks and artifact/cache integration remain deferred.

18. **Reduction applications.** [x]
    - Implement reduction rewrites (redex paths, substitutions, guard
      evidence) with full provenance recorded into `ResolutionTrace`.
    - Tests: redex path correctness; guard evidence required; source redex,
      target term, rule FQN, rule-view fingerprint, selection key,
      enclosing-term fingerprint, and source provenance recorded; `such` side
      conditions are applicability-only; pending/rejected/unaccepted
      reductions do not rewrite; invalid substitutions and mismatched
      strategy-audit keys are diagnosed; replayable traces.
    - Deps: 16. Spec: `registration_resolution.md` (reduction section),
      architecture 17 "Reduction Step".
    - Completed by task 18: `ReductionTraceBuilder` records replayable
      reduction steps over explicit payloads, preserves architecture-17
      provenance fields, validates active reduction registrations, rule-view
      fingerprints, substitutions, guard evidence, and strategy-audit keys, and
      treats `such` guards as applicability-only evidence. Raw syntax matching,
      resolver-shell parsing, artifact/cache integration, and source-derived
      reduction extraction remain deferred.

19. **Pending-registration validation and activation gating.** [x]
    - Validate pending registration declarations (architecture 04 Step 6),
      emit their obligations, and implement the interim activation-gating
      policy; record the decision here and at the top level.
    - Tests: invalid registrations diagnosed; kind-specific validation covers
      existential, conditional, functorial, and reduction patterns, including
      reduction free-variable/occurrence/orientation/source-provenance checks;
      unverified registrations never affect inference; policy-admitted
      activation requires accepted verifier status from a later proof/artifact
      input.
    - Deps: 17, 18. Spec: `registration_resolution.md`.
    - Completed by task 19: `RegistrationValidationInput` validates explicit
      checker-ready pending payloads, emits checker-local
      `InitialObligationId`s, keeps validated records pending with
      `inference=false`, rejects recovered origins and malformed kind-specific
      payloads, enforces fixed spec-17.6.4 reduction size/variable rules, and
      rejects activation inputs whose verifier/artifact status is missing or
      rejected. Source extraction, accepted-status production/import, artifact
      reuse, and active `.miz` semantic fixtures remain deferred.

20. **Existential gating of attributed type use.** [x]
    - Enforce that attributed types are usable only where existential
      registrations justify non-emptiness (architecture 04 "Existential
      Registrations Gate Attributed Type Use").
    - Tests: missing-existential fixtures fail with stable diagnostics;
      pending/rejected/unaccepted existential registrations do not satisfy
      gates; activated gates require visible guards and do not seed verified
      facts after degraded recovery.
    - Deps: 19. Spec: `registration_resolution.md`,
      [17.clusters_and_registrations.md](../../../spec/en/17.clusters_and_registrations.md).
    - Completed by task 20: `ExistentialGateOutput` evaluates explicit
      checker-owned gate payloads against activated existential registrations,
      binds candidates to accepted validation kind plus
      pattern/correctness/evidence/fingerprint records, requires visible
      consumable guard fact evidence, matches the full accepted attributed-type
      pattern, applies deterministic result precedence, and ensures only
      satisfied normal gates may seed verified facts. Source extraction,
      artifact reuse, accepted-status production, and active `.miz` gate
      fixtures remain deferred.

### Wave 3: overload resolution (phase 8)

21. **Spec: `overload_resolution.md`.** [x]
    - Write the overload spec (English and Japanese, no code) with named
      sections: site/candidate collection with provenance, template
      expansion, viability over recorded facts, specificity preorder
      (per-site graphs, no global DAG), root selection and refinement joins,
      `qua` view insertion (widening only, multiple-inheritance ambiguity),
      and failed-site preservation (architecture 05).
    - Deps: 2. Spec: architecture 05,
      [19.overload_resolution.md](../../../spec/en/19.overload_resolution.md),
      [18.templates.md](../../../spec/en/18.templates.md).
    - Completed by task 21: `overload_resolution.md` now defines the
      checker-local phase-8 boundary, explicit site/candidate payloads,
      template expansion, viability over recorded facts, per-site specificity
      graphs, root selection, refinement joins, widening-only inserted `qua`
      views, failed-site preservation, diagnostics, determinism, planned task
      coverage for tasks 22-26, and MC-G027 test/deferred/external gaps. No
      code was added.

22. **Candidate site collection.** [x]
    - Collect explicit overload site and candidate payloads carrying
      `TypedAst` site refs and resolver symbol ids after scope/visibility
      filtering.
    - Tests: site coverage per application form; provenance retained;
      deterministic candidate order.
    - Deps: 11, 21. Spec: `overload_resolution.md` (sites section).
    - Completed by task 22: `src/overload_resolution.rs` exposes
      checker-owned `OverloadCollectionOutput::collect` over explicit site and
      candidate payloads. It assigns deterministic local site/candidate ids,
      preserves site and candidate provenance, source-written `qua`, template,
      and coherence metadata, diagnoses duplicate site keys and missing
      candidate-site links while retaining rejected input provenance, defers
      unsupported roles with stable diagnostics, and preserves already
      scope/visibility-filtered candidate sets without
      scanning `SymbolEnv`, walking raw syntax, expanding templates, checking
      viability, selecting roots, or projecting `ResolvedTypedAst`.

23. **Template expansion.** [x]
    - Expand template candidates into concrete candidates ahead of ordinary
      candidate ordering; record exclusion reasons for non-expandable
      templates.
    - Tests: expansion fixtures; constrained-template evidence cases;
      exclusions carry reasons.
    - Deps: 22, `mizar-parser` task 31. Spec: `overload_resolution.md`
      (templates section).
    - Completed by task 23: `TemplateExpansionOutput::expand` validates only
      explicit `TemplateCandidatePayload` metadata retained by task 22. It
      copies non-template candidates, instantiates successful templates into
      concrete candidates with `CandidateOrigin::TemplateDerived`, records
      substitutions and `TemplateExpansionTable` rows, preserves skipped
      template candidates with stable rejection/deferred diagnostics, and
      covers explicit arguments, omitted inference payloads, accepted/missing/
      deferred constraints, source-`qua` widening/narrowing status,
      non-template priority, unsupported/deferred candidates, and deterministic
      rendering without cluster expansion, fresh fact inference, viability,
      specificity, root selection, or view insertion.

24. **Viability filtering.** [x]
    - Filter candidates by viability using recorded type facts only — no new
      inference (architecture 05 "Viability Uses Type Facts, Not New
      Inference").
    - Tests: viability fixtures; consumable versus pending/degraded/rejected
      fact evidence; rejection reasons preserved for diagnostics.
    - Deps: 23. Spec: `overload_resolution.md` (viability section).
    - Completed by task 24: `CandidateViabilityOutput::filter` consumes
      `TemplateExpansionOutput` and explicit checker-owned viability payloads
      keyed by concrete candidate id. It emits only fully viable candidates,
      records decision rows for every candidate, preserves accepted exact,
      consumable fact, widening, and source-`qua` view plans, rejects
      pending/degraded/rejected/out-of-scope/missing/narrowing evidence with
      stable diagnostics, blocks ambiguous or externally deferred payloads, and
      avoids new type inference, fact derivation, cluster firing, root
      selection, or view insertion.

25. **Specificity graph construction.** [x]
    - Build per-site specificity graphs over viable candidates.
    - Tests: ordering fixtures; incomparable pairs stay incomparable;
      deterministic graph rendering.
    - Deps: 24. Spec: `overload_resolution.md` (specificity section).
    - Completed by task 25: `SpecificityGraphOutput::build` consumes
      `CandidateViabilityOutput` and explicit checker-owned pairwise
      comparison payloads keyed by viable candidate ids. It emits one graph per
      site, one node per viable concrete candidate, comparison rows for same-
      site pairs, directed edges only for accepted at-least-as-specific
      relations, explicit incomparable rows without edges, and stable
      diagnostics for missing, duplicate, unknown, and cross-site comparison
      payloads. It does not derive facts, inspect result types for ordering,
      apply root-selection tie-breakers, join refinements, or insert views.

26. **Root selection, refinement joins, and view insertion.** [x]
    - Select overload roots, join coherent refinement groups, insert `qua`
      views, and preserve failed sites explicitly (architecture 05 Step 5).
    - Tests: selection fixtures including strongest-type, attribute-union, and
      incompatible refinement joins; ambiguity diagnostics with candidate
      lists; missing/duplicate/unknown/blocked payload diagnostics; missing or
      ambiguous ordinary-root candidate diagnostics; deterministic selection
      rendering; failed sites never become valid output.
    - Deps: 25. Spec: `overload_resolution.md` (selection/views sections).
    - Completed by task 26: `OverloadSelectionOutput::resolve` consumes
      `SpecificityGraphOutput` and explicit checker-owned selection payloads.
      It selects a unique maximal non-redefinition ordinary root candidate from
      per-site graphs, records `NoMatch`, `Ambiguous`,
      `IncompatibleRefinementJoin`, and blocked sites as failed outputs,
      validates same-root redefinition payloads with accepted coherence,
      accepts strongest-result and attribute-union exposed result metadata only
      after root selection, records accepted widening/source-`qua` inserted views,
      and rejects non-selected refinements, missing payloads, narrowing/missing
      view evidence, or blocked specificity graphs without fabricating success
      or applying additional root-selection tie-breakers.

27. **Spec: `resolved_typed_ast.md`.** [x]
    - Write the `ResolvedTypedAst` data-shape spec (English and Japanese, no
      code): final types, `OverloadResolutionTable`,
      `CoercionInsertionTable`, `ClusterFactTable`, expression metadata.
    - Deps: 21. Spec: [01.ir_layers.md](../../architecture/en/01.ir_layers.md),
      architecture 05 "Step 6".
    - Completed by task 27: `resolved_typed_ast.md` defines the final
      source-shaped semantic AST boundary, node and expression metadata tables,
      overload resolution projection, coercion insertion metadata, cluster fact
      references/provenance preservation, failed-site preservation, deterministic rendering
      expectations, task-28 planned tests, and deferred source-extraction /
      artifact gaps without code.

28. **`ResolvedTypedAst` assembly.** [x]
    - Assemble the final source-shaped semantic AST with expression metadata
      for LSP and artifacts, plus a deterministic debug rendering.
    - Tests: assembly fixtures; metadata lookup by `ExprId`; rendering
      stability.
    - Deps: 26, 27. Spec: `resolved_typed_ast.md`.
    - Completed by task 28: `ResolvedTypedAst::assemble` projects explicit
      checker-owned typed AST, cluster fact, overload
      collection/template/viability/specificity, and selection outputs into
      source-shaped resolved nodes, expression metadata, collection/expanded/
      viable candidate summaries, template expansion summaries, viability
      decisions, specificity graph summaries, overload records, inserted
      coercions, diagnostics, and deterministic rendering while preserving
      failed sites and keeping source extraction, artifacts, public diagnostic
      codes, and active `.miz` fixtures deferred.

### Hardening and cross-cutting follow-ups

29. **Deferred corpus obligations at stages `formula_statement` and `advanced_semantics`.** [x]
    - Record deferred registration/overload corpus obligations (clusters,
      reductions, ambiguity, refinement joins) with `spec_trace.toml` entries;
      active 40/60 pass/fail growth remains future work.
    - Record the review-audit advanced-semantics negative obligations as
      deferred: witness leakage from `now`/`proof` blocks, unmet
      `deffunc`/`defpred` guards, missing sethood for comprehensions, and
      invalid `qua` narrowing.
    - Deps: 20, 28. Spec: [staged_model.md](../../mizar-test/en/staged_model.md).
    - Completed by task 29 as a deferred corpus-record task: `spec_trace.toml`
      now records deferred formula/statement, cluster/reduction,
      overload/refinement, and review-audit negative obligations with concrete
      MC-G019/MC-G020/MC-G021/MC-G023/MC-G027 and runner blockers. No active
      `.miz` fixtures were added because `mizar-test` has no active
      `formula_statement` / `advanced_semantics` runner and mizar-checker still
      lacks the source-to-checker semantic payload extraction needed for those
      cases.

30. **Determinism suite.** [x]
    - Property coverage that identical inputs produce identical types,
      facts, traces, candidate orders, and diagnostics.
    - Deps: 28. Spec: [20.test_strategy.md](../../architecture/en/20.test_strategy.md).
    - Completed by task 30: `crates/mizar-checker/src/determinism_suite.rs`
      adds checker-owned Rust regressions for exact same-input reruns and
      canonicalized equivalent-input permutations across type normalization,
      type-fact contradiction queries, cluster closure traces, overload
      collection/template/viability/specificity/selection outputs, and final
      `ResolvedTypedAst::assemble` projection. No active `.miz` fixtures were
      added because stage runners and source-to-checker payload extraction
      remain deferred under the existing external gaps.

31. **Public-enum forward-compatibility policy.** [x]
    - Apply the `mizar-frontend` task-25 procedure to each public enum;
      record decisions in the owning module specs.
    - Deps: 28. Spec: all module specs.
    - Completed by task 31: every current checker-owned public enum is
      classified as a downstream forward-compatible API surface that must
      remain `#[non_exhaustive]`. Each owning EN/JA module spec records a
      `Public Enum Policy` table and no-exhaustive-exceptions statement, and
      `tests/lint_policy.rs` guards future source/spec drift for public enum
      attributes and policy rows.

32. **Source/spec correspondence audit.** [x]
    - Trace every public API and promised behavior in the module specs to
      implementation and tests; record gaps as follow-up tasks.
    - Deps: 31. Spec: all module specs and this TODO.
    - Completed by task 32: [source_spec_audit.md](./source_spec_audit.md)
      inventories every current checker `pub mod` export, top-level public
      item, and public `dense_id!` / `string_key!` newtype, then traces module
      behavior promises to implementation, Rust tests, or explicit MC-G
      `external_dependency_gap` / `test_gap` / `deferred` rows.
      `tests/lint_policy.rs` guards that inventory and gap reconciliation. No
      source/API behavior, `.miz` fixture, or expectation changed in this audit
      task.

33. **Bilingual documentation sync audit.** [x]
    - Compare each English canonical document under
      `doc/design/mizar-checker/en/` with its Japanese companion and
      synchronize content.
    - Deps: 32. Spec: repository documentation policy.
    - Completed by task 33: [bilingual_sync_audit.md](./bilingual_sync_audit.md)
      inventories every current English/Japanese checker design document pair,
      records companion links and comparison basis, records `none` sync debt
      for each pair, and `tests/lint_policy.rs` guards future pair inventory
      drift.

34. **Module-boundary refactor gate.** [x]
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
    - Completed by task 34: [module_boundary_audit.md](./module_boundary_audit.md)
      inventories every current checker Rust source/test-support file with
      line count, boundary label, owning specification, split decision, and
      hard-gate status. No behavior-neutral split is required; large cohesive
      files are monitored ergonomics notes only, and `tests/lint_policy.rs`
      guards future source-layout audit drift.

### Wave 4: semantic-audit follow-ups (2026-07-03)

[semantic_spec_audit.md](./semantic_spec_audit.md) audited the checker-scoped
specification chapters (03, 05-08, 13, 14, 17-19) and recorded findings
SSA-001 through SSA-020 plus a 16-fixture adversarial rejection corpus. The
tasks below convert every finding into either an owned task or an explicit
disposition. Spec-decision tasks (35-44) come first because AGENTS.md places
`doc/spec/en/` above design docs and code: the checker must not implement
behavior the spec has not decided. Each spec task chooses among the audit's
proposed resolutions (or records a superior one), updates `doc/spec/en/` and
`doc/spec/ja/` in the same change, adds or activates reject-first corpus
seeds where the decision creates new rejections, and updates
`tests/coverage/spec_trace.toml`.

Finding dispositions (every SSA id maps to a task or a recorded reason):

| Finding | Disposition |
|---|---|
| SSA-001 | task 35 |
| SSA-002, SSA-011, SSA-012 | task 36 |
| SSA-003, SSA-010, SSA-016, SSA-019 | task 37 |
| SSA-004 | task 38 |
| SSA-005 | task 39 |
| SSA-006 | task 40 |
| SSA-007, SSA-008, SSA-020 | task 41 |
| SSA-009 | task 42 |
| SSA-013, SSA-014 | task 43 |
| SSA-015, SSA-017 | task 44 |
| SSA-018 | no task: the greedy `of`/`over` parse is deterministic and documented (spec 19.6.4); a scope-sensitivity lint belongs to the future diagnostics-adoption wave and is recorded in that wave, not here |
| corpus seeds | task 49 activates the 16 audit fixtures plus the task-35 constructor-property seed, task-36 duplicate-coverage seed, task-37 ordinary/template-derived equivalent-root and same-return signature-conflict seeds, task-38 functorial-`for` guard seed, task-39 property-overlap coherence seed, and task-44 omitted-`reconsider`/ambiguous-redefinition-target seeds when the required runners, parser support, declaration-symbol support, and source-to-checker payload extraction land |

35. **Spec decision: constructor property arguments vs extensionality (SSA-001).** [x]
    - Resolve the critical §5.5.1/§5.8.4/§5.8.5 inconsistency. Recommended
      resolution 1: constructors accept fields only; property values always
      come from §7.4.1 property implementations. Update spec 05 and 07
      (English and Japanese, same change) and reconcile with the
      exact-instance extensionality text already landed by the template audit
      (spec 05 §5.8.5, commit `cef7e109`).
    - Acceptance: the spec states exactly one source for property values; a
      reject-first `.miz` seed pins the rejected constructor-property form
      with a sidecar and `spec_trace.toml` entry; no axiom family in §5.8 can
      derive `b1 = b2` for distinct property arguments.
    - Verify: `cargo test -p mizar-test`; corpus JSON/TOML validity.
    - Deps: none (first of the spec wave). Refs: SSA-001;
      [template_encoding_audit.md](../../mizar-core/en/template_encoding_audit.md) F1.
    - Completed by task 35: spec 05 now makes default constructors
      fields-only and removes property projection axioms; spec 07 states that
      property implementations are the sole property-value source. Added
      `fail_structure_constructor_property_arg_001` as an inactive
      `advanced_semantics` reject-first seed and traceability rows
      `spec.en.05.structures.constructor_fields_only.semantic` and
      `spec.en.07.modes.property_implementation.not_constructor_source.semantic`.
      No checker/core source semantics changed.

36. **Spec decision: structure member identity, upcast paths, acyclicity (SSA-002, SSA-011, SSA-012).** [x]
    - Define diamond member identity as the root declaration reached by the
      `from` chain (or record a superior rule); require the child member type
      to be `⊑` every parent's member type with per-parent coherence
      obligations; state whether §19.2.2 path uniqueness is syntactic or
      semantic; add an explicit inheritance-acyclicity rule and diagnostic to
      §5.3. Update spec 05 and 19 (English and Japanese, same change) and
      keep the rules consistent with the reduct-view encoding (`view_{D→B}`)
      landed in §5.8.3/§13.8.7.
    - Acceptance: existing seeds
      `fail_structure_diamond_member_type_conflict_001`,
      `fail_structure_inherit_uncovered_member_001`,
      `fail_structure_inherit_cycle_001`,
      `fail_overload_inheritance_path_ambiguity_001` remain valid under the
      decided rule (revise sidecar notes only if the decision changes their
      rationale); renamed-member identity cases have a decided outcome.
    - Verify: `cargo test -p mizar-test`.
    - Deps: 35. Refs: SSA-002, SSA-011, SSA-012; template audit F1.
    - Completed by task 36: spec 05 now defines inherited member identity as
      root declaration plus inheritance path/view, requires exact member
      coverage and per-parent type-inclusion obligations discharged by the
      existing `coherence` block, keeps renamed same-root paths as distinct
      views, and names `structures.inherit.cycle` for acyclicity failures.
      Spec 19 now states that implicit upcast path uniqueness is syntactic over
      resolved `inherit` declaration paths. Added
      `fail_structure_inherit_duplicate_member_coverage_001` as an inactive
      duplicate-coverage seed; no renamed-view reject seed was added because
      renamed-view exposure remains valid positive behavior. Existing
      structure/overload seeds and the template view-leak seed remain the other
      guards. No checker/core source semantics changed.

37. **Spec decision: overload tie-break and tie ambiguity (SSA-003, SSA-010, SSA-016, SSA-019).** [x]
    - Fix §19.6.1 Cases 2-3 against §19.4.3: either add explicit
      constraint-strictness and non-template-beats-template rules, or keep
      pure `⊑` selection and correct the case outcomes. Extend §19.4.4 to "no
      unique maximal root" (covers equally specific distinct roots); extend
      the §19.1 conflict rule to identical-signature declarations regardless
      of return type; reword §19.2.3 antisymmetry to closure-equivalence
      classes; drop the triplicated §19.6.1 sentence. Update architecture 05's
      tie-breaker list and `overload_resolution.md` in the same change.
    - Acceptance: §19.6.1 examples and §19.4.3/§19.4.4 rules agree; a
      tie-ambiguity `.miz` seed joins
      `fail_resolve_same_signature_return_conflict_001` with sidecar and
      trace entries.
    - Verify: `cargo test -p mizar-test`.
    - Deps: none. Refs: SSA-003, SSA-010, SSA-016, SSA-019.
    - Completed by task 37: spec 19 now keeps Phase B overload selection on
      instantiated concrete parameter vectors under the normal `⊑` preorder;
      template declared constraint strictness is not a tie-breaker,
      non-template priority applies only to mutually equivalent concrete
      vectors, return type remains excluded, and ambiguity is any nonempty
      maximal-root set with more than one distinct root. Ordinary definitions
      with identical argument signatures are declaration conflicts regardless
      of return type, and
      §19.6.1 examples now match those rules. Architecture 05 and
      `overload_resolution.md` were synchronized. Added inactive seeds
      `fail_overload_equivalent_roots_ambiguity_001`,
      `fail_overload_template_equivalent_roots_ambiguity_001`, and
      `fail_resolve_same_signature_same_return_conflict_001`; the last stays
      inactive until resolver declaration-symbol support grows beyond the
      current different-return diagnostic. Mizar-core task 26 / template-audit
      F7 records the separate Phase A omitted-template inference determinism
      rule. No checker/core/resolver source semantics changed.

38. **Spec decision: functorial cluster `for T` semantics (SSA-004).** [x]
    - Specify the applicability-guard reading (registration fires where the
      result's known normalized type is the full `for` type expression, or a
      subtype of it, mirroring §17.7.2) and add `is_T(F(args))` premises to
      the coherence obligation; update the §17.9.3 encoding tables so
      `for T` is no longer dropped. Spec 17 English and Japanese in the same
      change.
    - Acceptance: every §17.9.3 row involving `for T` shows the guard; a
      reject-first seed pins a functorial registration applied outside its
      `for` type.
    - Verify: `cargo test -p mizar-test`.
    - Deps: none. Refs: SSA-004.
    - Completed by task 38: spec 17 now defines a functorial cluster's
      trailing `for` type expression as an applicability guard over the full
      known normalized result type, including parameters and attributes. The
      coherence obligation and §17.9.3/§17.9.6 encodings include the
      result-type guard as a premise instead of dropping it. Added inactive
      `advanced_semantics` seed
      `fail_cluster_functorial_for_guard_001` plus traceability row
      `spec.en.17.clusters.functorial_for_guard.semantic`; the seed pins that
      the registration itself can remain valid while the consequent attribute
      is unavailable at same-radix use sites that lack the guarded attribute.
      Spec 16's proof-obligation summary and spec 23's registration-node
      discussion were synchronized with the guarded functorial obligation and
      now defer the detailed encoding to Chapter 17. No checker/core source
      semantics changed.

39. **Spec decision: property-implementation coherence (SSA-005).** [x]
    - Require any two `property S.p means/equals` implementations with
      overlapping domains to be related by a coherence obligation, or
      restrict each property to one implementation per `inherit`-connected
      mode family; update spec 07 §7.4.1/§7.8.2 (English and Japanese).
    - Acceptance: the chosen rule names the obligation form or the
      restriction diagnostic; a reject-first seed pins the uncovered overlap.
    - Verify: `cargo test -p mizar-test`.
    - Deps: 35 (property-value source must be settled first). Refs: SSA-005.
    - Completed by task 39: spec 07 now requires overlapping
      implementations of the same struct property to carry an accepted
      `coherence` correctness condition. The grammar admits an optional
      `coherence` block after property `means` existence/uniqueness and after
      property `equals`, but the block is semantically mandatory for overlaps;
      spec 16 and Appendix A were synchronized. Added inactive seed
      `fail_mode_property_overlap_missing_coherence_001` and traceability row
      `spec.en.07.modes.property_implementation.coherence.semantic`, plus
      deferred parser row `spec.en.07.modes.property_implementation.parser`.
      No checker/core source semantics changed.

40. **Spec contract: registration activation timing (SSA-006).** [x]
    - Keep §17.1 item-ordered activation as the language contract and state
      explicitly that correctness-condition acceptance may be asynchronous:
      an implementation may hold a module pending but must not reject a use
      site that a completed verification pass would accept. Record in
      `registration_resolution.md` that the task-19 interim policy is a
      conservative approximation to be lifted when `mizar-vc`/`mizar-proof`
      integration lands.
    - Acceptance: spec 17 (en+ja) states the asynchronous-acceptance
      contract; `registration_resolution.md` (en+ja) names the interim policy
      as such; `fail_mode_existential_after_declaration_001` remains the
      user-visible ordering error.
    - Verify: `cargo test -p mizar-test`.
    - Deps: none. Refs: SSA-006, architecture 04.
    - Completed by task 40: spec 17.1 keeps item-ordered activation as the
      language contract and states that correctness acceptance may arrive
      asynchronously from proof/kernel/artifact phases. Architecture 04 and
      `registration_resolution.md` now name task 19's no-accepted-input
      behavior as an interim conservative approximation, not a final rejection
      policy for later source items that a completed pass would accept. The
      existing inactive seed `fail_mode_existential_after_declaration_001` now
      traces the negative non-retroactive slice through
      `spec.en.17.clusters.registration_activation_timing.semantic`; positive
      accepted-local activation remains deferred on MC-G020/MC-G021/MC-G025/
      MC-G026. No checker/core source semantics changed.

41. **Spec clarifications: closure termination, contradiction site, `attr(args)` (SSA-007, SSA-008, SSA-020).** [x]
    - State in §17.7.1 that closure termination follows from the restricted
      adjective grammar and that extending adjectives to term arguments
      requires a new termination argument; specify closure-time detection of
      contradictory derived attributes as a fatal `cluster` diagnostic and
      reword §17.7.3's ATP-time mention; resolve §3.3/§6.2/§17.10
      `attr(args)`: either define its declaration/registration story or
      remove it from `attribute_ref` (removal is recommended — admitting it
      into clusters breaks the termination argument). Spec 03, 06, 17
      English and Japanese in the same change.
    - Acceptance: termination argument is stated as load-bearing;
      `fail_cluster_contradictory_consequent_001` maps to the closure-time
      diagnostic; `attribute_ref` grammar and declaration grammar agree.
    - Verify: `cargo test -p mizar-test`.
    - Deps: none. Refs: SSA-007, SSA-008, SSA-020.
    - Completed by task 41: spec 17.7.1 and spec 19.2.1 now make the
      restricted no-argument cluster `adjective` grammar the load-bearing
      termination premise, and architecture 04 treats saturation bounds as
      defensive failure diagnostics rather than successful truncated
      semantics. Spec 17.7.3 now specifies closure-time fatal `cluster`
      diagnostics for contradictory derived attributes, including the static
      contradictory-consequent seed. Spec 03/06/Appendix A define
      `attribute_name(args)` as a use-site application of a declared
      parameterized attribute while excluding argument lists from cluster
      registration adjectives. Updated traceability with
      `spec.en.17.clusters.restricted_adjective_grammar.parser`. No
      checker/core source semantics changed.

42. **Spec clarification: reduction determinism signature (SSA-009).** [x]
    - Restate §17.6.4 normalization determinism as a function of (term,
      in-scope rules, discharged side-condition set); define combined
      specificity as pattern subsumption first, then position-wise guard
      comparison, remaining mixed cases incomparable with FQN tie-break.
      Spec 17 English and Japanese; mirror in
      `registration_resolution.md` (reduction section).
    - Acceptance: the determinism statement's inputs match the matching
      row's dependencies; task-18 behavior (`such` guards
      applicability-only) is derivable from the spec text.
    - Verify: `cargo test -p mizar-test`.
    - Deps: none. Refs: SSA-009.
    - Completed in task 42: spec 17 now defines reduction normalization as a
      deterministic function of the term, in-scope activated reduction rules,
      and discharged side-condition set. `such` guards are applicability-only
      evidence, not specificity inputs. Rule selection is pattern-first,
      guard-second with position-wise §19.2.3 comparison and FQN tie-break for
      equal, mixed, or incomparable cases. `registration_resolution.md` mirrors
      the rule. No checker/core source semantics changed.

43. **Spec clarification: sethood for dependent modes and built-in inhabitation (SSA-013, SSA-014).** [x]
    - Give the parameterized sethood obligation form
      (`∀params. ∃S. ∀x. (is_T(x, params) → x ∈ S)`) in §7.8.1 and state
      that §13.4.2 comprehension gates check sethood at the instantiated
      parameters; reconcile §7.8 vs §17.3.4 on unattributed bases and add
      the built-in inhabitation table (`object`, `set`, struct radixes).
      Coordinate with the template-actual inhabitation gate added by the
      template audit (§17.3.4). Spec 07, 13, 17 English and Japanese.
      Ch18 may also be synchronized where it references the gate.
    - Acceptance: the checker's existential gate (task 20) has a decidable
      rule for every base-type shape; sethood export status (module
      interface or not) is stated.
    - Verify: `cargo test -p mizar-test`.
    - Deps: none. Refs: SSA-013, SSA-014; template audit F2.
    - Completed in task 43: spec 07 now gives guarded parameterized
      existence and sethood obligations, and states exported sethood is a
      module-interface semantic fact whose witness term is not exported. Spec
      13 checks sethood at the resolved mode and normalized argument tuple.
      Spec 17 adds the inhabitation table for attributed existential
      registrations, built-in `object`/`set`, accepted modes, bare structure
      radixes via constructor witnesses over inhabited fields, and bare schema
      type parameters inside template bodies via §18.10.2. Spec 18 type
      actuals use the same table. Existing inactive sethood,
      existential, and template seeds keep their rejection intent; positive
      source-derived coverage remains deferred. No checker/core source
      semantics changed.

44. **Spec clarification: `reconsider` discharge and ambiguous redefinition target (SSA-015, SSA-017).** [x]
    - State that omitted `reconsider` justification is legal iff the
      narrowing obligation is discharged by proof-free widening,
      inheritance/view, cluster-closure, or already recorded local type facts,
      otherwise a diagnostic requests a justification (§8.2); name an
      "ambiguous redefinition target" diagnostic for `redefine` without
      `coherence with` when several originals qualify (§19.4.1). Spec 08 and
      19 English and Japanese.
    - Acceptance: both behaviors have named diagnostics and one reject-first
      seed each.
    - Verify: `cargo test -p mizar-test`.
    - Deps: 37 (shares chapter 19 edits). Refs: SSA-015, SSA-017.
    - Completed in task 44: spec 04/08/15/Appendix A now agree that omitted
      `reconsider` justification is syntax-admissible, while spec 08/22 gate
      it to proof-free widening/inheritance/cluster-closure/local-fact
      discharge and otherwise require `type.narrowing_requires_proof`; the same
      grammar update makes proof-block `reconsider` explicit. Spec
      19/22 now name `resolve.ambiguous_redefinition_target` for omitted
      `coherence with` when several visible earlier roots are strictly
      sharpened; declaration/import order and return type do not choose. Two
      inactive advanced-semantics seeds pin the decisions. Existing parser
      omitted-justification and proof-block behavior remains a deferred parser task-47
      `source_drift` / `test_expectation_drift`; no checker/core source
      semantics changed.

45. **Checker alignment: overload tie-break implementation.** [x]
    - Align `overload_resolution.md` and the wave-3 implementation (tasks
      23-26 surfaces: template expansion priority, specificity comparisons,
      root selection) with the task-37 decision; add Rust regressions for the
      decided Case 2/3 outcomes and the tie-ambiguity rule.
    - Also align declaration-time `redefine`-family target inference with
      task 44: omitted `coherence with` may infer a target only when exactly
      one visible earlier ordinary root of the same symbol kind and arity is
      strictly sharpened. Multiple qualifying roots must preserve a failed
      record/diagnostic instead of choosing by declaration order, import order,
      or return type.
    - Acceptance: `cargo test -p mizar-checker` covers the decided outcomes;
      no undocumented tie-breaker or omitted-redefinition-target chooser remains
      in code.
    - Verify: `cargo test -p mizar-checker`,
      `cargo clippy -p mizar-checker --all-targets -- -D warnings`.
    - Deps: 37, 44. Refs: SSA-003, SSA-010, SSA-017; architecture 05.
    - Completed in task 45: `overload_resolution.rs` now has explicit-payload
      regressions for the task-37 Case 2/3 outcomes: equivalent distinct
      template-derived roots remain ambiguous, encoded non-template priority
      and strictly-more-specific template edges select the intended root, and
      an unencoded ordinary/template-derived equivalence tie stays ambiguous.
      Same-root accepted redefinition metadata likewise cannot break a
      distinct-root tie. `overload_resolution.md`, the checker plan/audits,
      and the top-level coverage audit now state that omitted `coherence with`
      target diagnostics are declaration-checking/source-extraction producer
      behavior; this data layer accepts only already-bound redefinition
      payloads and preserves missing/deferred/rejected producer records. The
      inactive `.miz` overload/redefinition seeds and deferred traceability rows
      remain unchanged under MC-G027/MC-G030.

46. **Checker alignment: closure contradiction and termination rules.** [x]
    - Encode the task-41/42 decisions in `cluster_trace.md` and
      `registration_resolution.md` (en+ja) and align the task 16-18
      implementation: closure-time contradiction as fatal diagnostic
      (severity per §17.7.3), grammar-based termination note beside the
      defensive saturation bound, corrected reduction-determinism signature.
    - Acceptance: module specs cite the new spec text; existing determinism
      suite (task 30) extended for side-condition-set dependence.
    - Verify: `cargo test -p mizar-checker`,
      `cargo clippy -p mizar-checker --all-targets -- -D warnings`.
    - Deps: 41, 42. Refs: SSA-007, SSA-008, SSA-009.
    - Completed in task 46: `cluster_trace.rs` now asserts that explicit
      closure contradictions produce the checker-local contradiction class with
      error severity, fatal recovery, incomplete closure status, and no
      degraded export for the contradictory generated fact. The determinism
      suite now records an explicit-payload reduction trace snapshot proving
      that equivalent discharged guard order is canonical, changing the
      discharged `such` evidence changes the trace identity, and the
      strategy-audit key remains unchanged and free of `such` specificity.
      `cluster_trace.md`, `registration_resolution.md`, the checker
      plan/audits, and the top-level coverage audit now cite the task-41/42
      spec decisions. Source-derived normalization results, source-derived
      cluster contradiction extraction, artifact/cache replay, and active
      `.miz` semantic fixtures remain deferred under MC-G020/MC-G021/MC-G023/
      MC-G030.

47. **Checker alignment: existential gate and activation contract.** [x]
    - Align the task-20 existential gate with the task-43 built-in
      inhabitation table and parameterized sethood form, and record the
      task-40 activation contract in `registration_resolution.md` as the
      target behavior the interim policy approximates.
    - Also align omitted `reconsider` justification handling with task 44:
      no proof search and no implicit `by`; proof-free widening, unique
      inheritance/view evidence, active cluster closure, or already recorded
      local type facts must discharge every target obligation, otherwise the
      failed site reports `type.narrowing_requires_proof`.
    - Acceptance: gate behavior for `mode M is set`, built-ins, and struct
      radixes matches the decided table with Rust regressions, and omitted
      `reconsider` keeps the semantic E0102 gate without parser-only rejection.
    - Verify: `cargo test -p mizar-checker`,
      `cargo clippy -p mizar-checker --all-targets -- -D warnings`.
    - Deps: 40, 43, 44. Refs: SSA-006, SSA-013, SSA-014, SSA-015.
    - Completed in task 47: `registration_resolution.rs` now accepts
      explicit base-shape inhabitation evidence only for unattributed exact
      pattern matches, with built-in `object`/`set`, accepted mode tuple,
      zero-field or fully guarded structure constructor, and schema type
      parameter coverage. Attributed gates still require active existential
      candidates; hidden, non-consumable, incomplete, or mismatched guard
      evidence blocks or rejects the gate without seeding verified facts.
      `type_checker.rs` now distinguishes explicit from omitted narrowing
      requests, accepts omitted `reconsider` only when supplied consumable
      proof-free evidence already discharges the target, and reports
      `type.narrowing_requires_proof` without creating an implicit obligation
      otherwise. `registration_resolution.md`, `type_checker.md`, the checker
      plan/audits, and the top-level coverage audit record the task-40/43/44
      contracts. Source-derived base-shape extraction, positive accepted-local
      activation, source-derived omitted-`reconsider` parser/extraction
      coverage, artifacts, and active `.miz` fixtures remain deferred under
      MC-G018/MC-G020/MC-G021/MC-G025/MC-G026/MC-G030.

48. **Reserve source declaration producer seam.** [x]
    - Promote the existing reserve-only builtin declaration bridge into a
      checker-owned, syntax-free producer seam in `type_checker`: consume
      upstream-extracted source/module identity, reserve source range, binding
      spelling/range, and bare builtin `set`/`object` type-expression
      spelling/range/head; build the checker-owned `BindingEnv` and
      `DeclarationCheckingOutput`; expose deterministic typed-site ids for the
      active `mizar-test` runner to continue assembling `TypedAst`,
      `ResolvedTypedAst`, summary-readiness, and binder-only core context
      checks.
    - Acceptance: `mizar-checker` keeps no direct `mizar-syntax` dependency;
      non-builtin declarations, attributes, unsupported mode/structure
      payloads outside promoted diagnostic slices, terms, formulas, coercions,
      overload evidence, facts, proof skeletons,
      CoreIr/ControlFlowIr/VC/proof payloads, and new active `.miz` coverage
      remain deferred under MC-G020; active `type-elaboration` results stay
      byte-stable.
    - Verify: `cargo test -p mizar-checker`, `cargo test -p mizar-test`.
    - Deps: task 47; external source families remain MC-G020. Refs:
      Step 5 source-derived semantic bridge; mizar-test task 10.
    - Completed in task 48: `type_checker.rs` now exposes
      `SourceReserveDeclarationBridge`, `SourceReserveBindingInput`, and
      `SourceReserveDeclarationHandoff` for the supported reserve-only builtin
      slice. `mizar-test` still owns real `.miz` AST extraction and lower-stage
      runner gating, then delegates checker handoff production through this
      seam before its existing `TypedAst`/`ResolvedTypedAst`/core readiness
      assertions. No `.miz` expectations, public diagnostic codes,
      CoreIr/ControlFlowIr/VC/proof rows, or broader semantic payload families
      were promoted.

49. **Audit-corpus activation and task-29 record revision.** [ ]
    - When the `advanced_semantics`/`formula_statement` runners,
      property-implementation parser support, and source-to-checker payload
      extraction land (mizar-test runner growth +
      MC-G020/MC-G021/MC-G023/MC-G027 plus MC-G030/property-implementation
      payload extraction for the task-39 seed), activate the 16 semantic-audit
      fixtures plus the task-35 constructor-property seed, task-36
      duplicate-coverage seed, task-37 ordinary/template-derived
      equivalent-root ambiguity seeds, task-38 functorial-`for` guard seed,
      task-39 property-overlap coherence seed, and task-44 omitted-`reconsider`
      / ambiguous-redefinition-target seeds.
      Activate the task-37 same-return signature-conflict seed when the
      declaration-symbol runner supports that resolver diagnostic. Revise the
      task-29 deferred corpus records to point at (or be superseded by) the
      audit requirement ids.
    - Acceptance: `mizar-test` plan shows the fixtures active with zero plan
      errors; deferred records no longer double-count them.
    - Verify: `cargo test -p mizar-test`.
    - Deps: 35-44 decided; external: mizar-test runner support. Refs:
      semantic_spec_audit.md "Adversarial Corpus".

50. **Source-derived attributed reserve evidence-gap bridge.** [x]
    - Extend the task-48 reserve source declaration seam just far enough to
      accept source-derived attribute chains on builtin `set`/`object` reserve
      type expressions when the attribute symbol is already present in the
      resolver `SymbolEnv`.
    - Acceptance: same-module source-derived attributes are preserved in
      checker-owned `TypeExpressionInput` and normalized by declaration
      checking; attributed reserve declarations remain active fail cases with
      `checker.declaration.deferred.evidence_query` until a real existential
      registration/evidence-query seam exists. Imported attribute symbols,
      non-builtin heads, unsupported mode/structure payloads outside promoted
      diagnostic slices, terms, formulas, proof
      skeletons, CoreIr/ControlFlowIr/VC/proof payloads, and successful
      attributed declarations remain deferred under MC-G020/MC-G021/MC-G026.
    - Verify: `cargo test -p mizar-checker`, `cargo test -p mizar-test`.
    - Deps: task 48; external evidence remains MC-G021/MC-G026. Refs:
      Step 5 source-derived semantic bridge; mizar-test task 10;
      spec 03 type expressions; spec 17 existential gates.
    - Completed in task 50: `type_checker.rs` now accepts source-derived
      attribute payloads on the syntax-free reserve bridge and marks those
      declarations with `MissingEvidenceQuery` rather than fabricating
      existential evidence. `mizar-test` adds an active same-module attributed
      reserve fail fixture that reaches the checker diagnostic, while the
      existing import-backed attributed reserve fixture remains on the broader
      extraction gap until imported symbols enter the active runner `SymbolEnv`.

51. **Source-derived local mode reserve expansion-gap bridge.** [x]
    - Extend the task-48 reserve source declaration seam just far enough to
      accept source-derived reserve type heads that resolve to a unique
      same-module `LocalSource` mode symbol with no type arguments or source
      attributes.
    - Acceptance: the checker-owned bridge validates that symbol heads are
      exact `SymbolKind::Mode` entries from the current module's local source,
      then declaration checking reaches the existing
      `checker.type.external.mode_expansion_payload` diagnostic because real
      mode-expansion payload extraction is not implemented. Imported modes,
      structures, mode arguments, unresolved/ambiguous heads, mode expansion
      extraction, terms, formulas, CoreIr/ControlFlowIr/VC/proof payloads, and
      successful local-mode reserve declarations remain deferred under MC-G020.
    - Verify: `cargo test -p mizar-checker`, `cargo test -p mizar-test`.
    - Deps: task 48; external mode expansion remains MC-G014/MC-G020. Refs:
      Step 5 source-derived semantic bridge; mizar-test task 10; spec 03 type
      expressions; spec 07 modes; spec 17 accepted-mode inhabitation evidence.
    - Completed in task 51: `type_checker.rs` validates local source-backed
      mode heads on the syntax-free reserve bridge and preserves the existing
      missing mode-expansion diagnostic rather than unfolding from raw syntax.
      `mizar-test` adds an active same-module local-mode reserve fail fixture,
      while imported modes and argument-bearing mode heads remain on the
      broader extraction gap.

52. **Source-derived local structure reserve evidence-gap bridge.** [x]
    - Extend the task-48 reserve source declaration seam just far enough to
      accept source-derived reserve type heads that resolve to a unique
      same-module `LocalSource` structure symbol with no type arguments or
      source attributes.
    - Acceptance: the checker-owned bridge validates that symbol heads are
      exact `SymbolKind::Structure` entries from the current module's local
      source, marks those reserved-variable declarations with
      `MissingEvidenceQuery`, and reaches
      `checker.declaration.deferred.evidence_query` because real
      base-shape/constructor-witness evidence extraction is not implemented.
      Imported structures, structure arguments, attributed structure heads
      beyond the later promoted task-53 diagnostic slice, successful
      local-structure reserve declarations, structure field/default payload
      extraction, CoreIr/ControlFlowIr/VC/proof payloads, and broader semantic
      pass coverage remain deferred under MC-G020/MC-G026.
    - Verify: `cargo test -p mizar-checker`, `cargo test -p mizar-test`.
    - Deps: task 48; external base-shape evidence remains MC-G020/MC-G026.
      Refs: Step 5 source-derived semantic bridge; mizar-test task 10; spec 03
      type expressions; spec 05 structures; spec 17 base-shape inhabitation
      evidence.
    - Completed in task 52: `type_checker.rs` validates local source-backed
      structure heads on the syntax-free reserve bridge and preserves the
      missing evidence-query diagnostic rather than inferring structure
      inhabitation from a symbol. `mizar-test` adds an active same-module
      local-structure reserve fail fixture with a real field-bearing local
      `struct`, while imported structures and argument-bearing structure heads
      remain on the broader extraction gap.

53. **Source-derived attributed local structure reserve evidence-gap bridge.** [x]
    - Extend the task-48 reserve source declaration seam just far enough to
      accept source-derived no-argument attribute payloads attached to a unique
      same-module `LocalSource` structure reserve head with no type arguments.
    - Acceptance: the checker-owned bridge validates exact local
      `SymbolKind::Structure` provenance for the symbol head, keeps attributed
      local mode heads outside the later task-54 diagnostic slice on the broader
      extraction gap, marks the attributed
      local-structure reserved-variable declaration with
      `MissingEvidenceQuery`, and reaches
      `checker.declaration.deferred.evidence_query` because real existential
      evidence for the full normalized attributed type is not implemented.
      Imported attributes or structures, attribute arguments, qualified
      attribute disambiguation, structure arguments, successful attributed
      structure reserve declarations, structure field/default/base-shape
      extraction, CoreIr/ControlFlowIr/VC/proof payloads, and broader semantic
      pass coverage remain deferred under MC-G020/MC-G026.
    - Verify: `cargo test -p mizar-checker`, `cargo test -p mizar-test`.
    - Deps: tasks 48, 50, and 52; external full attributed-type existential
      evidence remains MC-G020/MC-G026. Refs: Step 5 source-derived semantic
      bridge; mizar-test task 10; spec 03 type expressions; spec 05
      structures; spec 17 existential and base-shape inhabitation evidence.
    - Completed in task 53: `type_checker.rs` admits same-module source
      attributes only for local structure heads on the syntax-free reserve
      bridge, while task 54 later owns the attributed local-mode diagnostic
      slice.
      `mizar-test` adds an active same-module attributed local-structure
      reserve fail fixture and keeps imported/argument-bearing forms on the
      broader extraction gap.

54. **Source-derived attributed local mode reserve expansion-gap bridge.** [x]
    - Extend the task-48 reserve source declaration seam just far enough to
      accept source-derived no-argument attribute payloads attached to a unique
      same-module `LocalSource` mode reserve head with no type arguments.
    - Acceptance: the checker-owned bridge validates exact local
      `SymbolKind::Mode` provenance for the symbol head, preserves the
      same-module source-derived attributes, does not attach
      `MissingEvidenceQuery` before a real mode expansion exists, and reaches
      `checker.type.external.mode_expansion_payload` because real
      mode-expansion payload extraction is not implemented. Imported
      attributes or modes, attribute arguments, qualified attribute
      disambiguation, mode arguments, successful attributed mode reserve
      declarations, real mode expansion, accepted-mode/base evidence,
      existential evidence for the fully expanded attributed type,
      CoreIr/ControlFlowIr/VC/proof payloads, and broader semantic pass
      coverage remain deferred under MC-G014/MC-G020/MC-G026.
    - Verify: `cargo test -p mizar-checker`, `cargo test -p mizar-test`.
    - Deps: tasks 48, 50, and 51; external mode-expansion and existential
      evidence remain MC-G014/MC-G020/MC-G026. Refs: Step 5 source-derived
      semantic bridge; mizar-test task 10; spec 03 type expressions; spec 07
      modes; spec 17 existential and accepted-mode inhabitation evidence.
    - Completed in task 54: `type_checker.rs` admits same-module source
      attributes for local mode heads on the syntax-free reserve bridge without
      treating the missing existential evidence as an evidence-query diagnostic.
      `mizar-test` adds an active same-module attributed local-mode reserve fail
      fixture and keeps imported/argument-bearing forms on the broader
      extraction gap.

55. **Source-derived bare local mode expansion bridge.** [x]
    - Extend the active type-elaboration source bridge just far enough to
      produce a real `ModeExpansion` for bare reserve uses of a unique
      same-module `LocalSource` no-argument mode definition whose unrecovered
      source definition precedes the reserve use, has no definition-local
      parameter/assumption context, and has a bare builtin `set` / `object`
      RHS.
    - Acceptance: the runner extracts the expansion from `SurfaceAst` and
      passes it through the checker-owned syntax-free reserve seam; the
      resulting bare local-mode reserve declaration is an active pass case
      through `BindingEnv`, `DeclarationChecker`, `TypedAst`,
      `ResolvedTypedAst`, summary-readiness, and binder-only `CoreContext`.
      The runner withholds mode expansions for attributed local-mode reserve
      uses, mixed attributed/bare local-mode sources, attributed mode RHSs,
      imported/argument-bearing/parameterized/contextual modes, unresolved or
      ambiguous heads, and non-reserve declarations. Those families remain on
      the existing missing-expansion or broader extraction gaps.
    - Verify: `cargo test -p mizar-checker`, `cargo test -p mizar-test`.
    - Deps: tasks 48, 51, and 54; broader mode expansion and existential
      evidence remain MC-G014/MC-G020/MC-G026. Refs: Step 5 source-derived
      semantic bridge; mizar-test task 10; spec 03 type expressions; spec 07
      modes; spec 17 base-shape inhabitation evidence.
    - Completed in task 55: `mizar-test` extracts a real AST-derived
      `ModeExpansion` for the narrow bare local-mode reserve slice and the
      checker source reserve seam accepts explicit mode-expansion payloads
      without fabricating evidence. A new active pass fixture covers the local
      mode expansion bridge, while attributed/mixed/attributed-RHS cases stay
      fail-closed on missing expansion or evidence gaps.

56. **Source-derived local mode expansion chain bridge.** [x]
    - Extend the task-55 bridge just far enough to produce real chained
      `ModeExpansion` payloads when a same-module bare local-mode reserve head
      expands to a preceding same-module no-argument local mode whose own
      preceding source definition has an accepted bare builtin `set` /
      `object` RHS expansion.
    - Acceptance: the runner inserts both source-derived expansions before the
      checker-owned reserve seam, active pass fixtures cover `B -> A -> set`
      and `B -> A -> object`, and an active fail fixture proves an attributed
      dependency withholds the whole chain and reaches the missing
      mode-expansion diagnostic. Forward references, ambiguous/imported/cyclic
      dependencies, partial chains without an accepted dependency expansion,
      attributed uses/RHSs, arguments, parameterized/contextual definitions,
      CoreIr/ControlFlowIr/VC/proof payloads, and broader semantic pass
      coverage remain deferred.
    - Verify: `cargo test -p mizar-test`, `cargo test -p mizar-checker`.
    - Deps: tasks 48, 51, 54, and 55. Broader mode expansion and existential
      evidence remain MC-G014/MC-G020/MC-G026. Refs: Step 5 source-derived
      semantic bridge; mizar-test task 10; spec 03 type expressions; spec 07
      modes; spec 17 base-shape inhabitation evidence.
    - Completed in task 56: `mizar-test` extracts the narrow one-edge
      source-derived local-mode expansion chain, adds active pass coverage for
      `B -> A -> set` and `B -> A -> object`, and keeps attributed dependency
      chains fail-closed on the checker missing mode-expansion diagnostic
      without promoting CoreIr, ControlFlowIr, VC, or proof payloads.

57. **Source-derived local mode structure-RHS evidence-gap bridge.** [x]
    - Extend the task-55 bridge just far enough to produce a real
      `ModeExpansion` payload when a same-module bare local-mode reserve head
      expands to a preceding same-module no-argument local structure head.
      The checker must consume that expansion, then fail closed on missing
      structure base-shape/constructor-witness evidence.
    - Acceptance: checker unit coverage proves `Mode -> LocalStruct` consumes
      the real `ModeExpansion`, does not emit
      `checker.type.external.mode_expansion_payload`, marks the declaration
      partial with `MissingEvidenceQuery`, and exports no verified facts.
      Runner unit coverage proves same-module local structure RHS extraction
      is accepted as a terminal expansion payload. One active
      `type_elaboration` fail fixture covers the real `.miz` source path with
      `type_elaboration.checker.checker.declaration.deferred.evidence_query`.
      Imported, argument-bearing, attributed, ambiguous, cyclic, and
      forward-reference structure RHSs remain outside the slice.
    - Verify: `cargo test -p mizar-test`, `cargo test -p mizar-checker`.
    - Deps: tasks 48, 52, and 55. Structure base-shape evidence and broader
      mode expansion remain MC-G020/MC-G026. Refs: Step 5 source-derived
      semantic bridge; mizar-test task 10; spec 03 type expressions; spec 05
      structures; spec 07 modes; spec 17 base-shape inhabitation evidence.
    - Completed in task 57: `mizar-test` extracts a real AST-derived
      local-mode expansion whose RHS is a same-module local structure head,
      and `mizar-checker` routes the expanded reserve declaration to the
      existing missing evidence-query diagnostic rather than the missing
      expansion-payload diagnostic. Positive structure acceptance,
      base-shape/constructor-witness extraction, imported/argument-bearing/
      attributed structure RHSs, CoreIr, ControlFlowIr, VC, proof payloads,
      and broader semantic pass coverage remain deferred.

58. **Source-derived local mode attributed-builtin RHS evidence-gap bridge.** [x]
    - Extend the task-55 bridge just far enough to produce a real
      `ModeExpansion` payload when a same-module bare local-mode reserve head
      expands to a preceding same-module no-argument local mode whose RHS is an
      attributed bare builtin `set` / `object` type.
    - Acceptance: checker unit coverage proves `Mode -> marked set` consumes
      the real `ModeExpansion`, does not emit
      `checker.type.external.mode_expansion_payload`, preserves the normalized
      attribute, marks the declaration partial with `MissingEvidenceQuery`, and
      exports no verified facts. Runner unit coverage proves direct attributed
      builtin RHS extraction is accepted as a terminal expansion payload while
      chain dependencies ending in attributed RHSs remain withheld. The
      existing active `type_elaboration` attributed-RHS fail fixture is updated
      to cover the real `.miz` source path with
      `type_elaboration.checker.checker.declaration.deferred.evidence_query`.
      Attributed reserve heads, mixed attributed/bare reserve uses, imported or
      argument-bearing attributes/modes, attributed local structure RHSs, chain
      promotion through attributed RHSs, successful attributed-mode
      declarations, and existential evidence remain outside the slice.
    - Verify: `cargo test -p mizar-test`, `cargo test -p mizar-checker`.
    - Deps: tasks 48, 50, 54, and 55. Full attributed-type existential
      evidence and broader mode expansion remain MC-G020/MC-G026. Refs: Step 5
      source-derived semantic bridge; mizar-test task 10; spec 03 type
      expressions; spec 07 modes; spec 17 attributed-type evidence.
    - Completed in task 58: `mizar-test` extracts a real AST-derived
      local-mode expansion whose RHS is an attributed builtin head, and
      `mizar-checker` routes the expanded reserve declaration to the existing
      missing evidence-query diagnostic rather than the missing
      expansion-payload diagnostic. Positive attributed-type acceptance,
      existential evidence extraction, attributed reserve heads,
      attributed-RHS chains, CoreIr, ControlFlowIr, VC, proof payloads, and
      broader semantic pass coverage remain deferred.

59. **Source-derived attributed local mode reserve evidence-gap bridge.** [x]
    - Extend the task-55 bridge just far enough to produce a real
      `ModeExpansion` payload for a same-module attributed local-mode reserve
      head when its unique preceding same-module no-argument mode definition
      has a direct bare builtin `set` / `object` RHS and the same mode is not
      also used as a bare reserve head in the same bridge input.
    - Acceptance: checker unit coverage proves `marked Mode` with a real
      `Mode -> set` expansion no longer emits
      `checker.type.external.mode_expansion_payload`, preserves the normalized
      attribute, marks the declaration partial with `MissingEvidenceQuery`,
      and exports no verified facts. Runner unit coverage proves the single
      attributed local-mode reserve use receives the real direct bare-builtin
      expansion while mixed bare/attributed uses of the same mode still
      withhold expansion. The existing active `type_elaboration` attributed
      local-mode reserve fixture is updated to cover the real `.miz` source
      path with
      `type_elaboration.checker.checker.declaration.deferred.evidence_query`.
      Mixed bare/attributed reserve uses, imported or argument-bearing
      attributes/modes, attributed dependencies, chains, structure RHSs,
      attributed RHSs, successful attributed-mode declarations, and
      existential evidence remain outside the slice.
    - Verify: `cargo test -p mizar-test`, `cargo test -p mizar-checker`.
    - Deps: tasks 48, 50, 54, and 55. Full attributed-type existential
      evidence and broader mode expansion remain MC-G020/MC-G026. Refs: Step 5
      source-derived semantic bridge; mizar-test task 10; spec 03 type
      expressions; spec 07 modes; spec 17 attributed-type evidence.
    - Completed in task 59: `mizar-test` extracts a real AST-derived direct
      bare-builtin local-mode expansion for a same-module attributed reserve
      head when the same mode has no mixed bare reserve use, and
      `mizar-checker` routes the expanded attributed reserve declaration to
      the existing missing evidence-query diagnostic rather than the missing
      expansion-payload diagnostic. Positive attributed-type acceptance,
      existential evidence extraction, mixed attributed/bare uses, attributed
      dependencies or chains, CoreIr, ControlFlowIr, VC, proof payloads, and
      broader semantic pass coverage remain deferred.

60. **Source-derived attributed local mode structure-RHS evidence-gap bridge.** [x]
    - Extend the task-57 structure-RHS bridge just far enough to produce a real
      `ModeExpansion` payload for a same-module attributed local-mode reserve
      head when its mode definition is unique, unrecovered, preceding,
      no-argument, free of definition-local context, and has a direct
      same-module local structure RHS whose unique unrecovered structure
      definition precedes the mode definition. The same mode must not also be
      used as a bare reserve head in the same bridge input.
    - Acceptance: checker unit coverage proves `marked Mode` with a real
      `Mode -> LocalStruct` expansion no longer emits
      `checker.type.external.mode_expansion_payload`, preserves the normalized
      attribute, marks the declaration partial with `MissingEvidenceQuery`,
      and exports no verified facts. Runner unit coverage proves the single
      attributed local-mode reserve use receives the real direct structure-RHS
      expansion while mixed bare/attributed uses of the same mode, attributed
      structure-RHS chains, and cached direct structure-RHS dependencies still
      withhold expansion. A new active `type_elaboration` fail fixture covers
      the real `.miz` source path with
      `type_elaboration.checker.checker.declaration.deferred.evidence_query`;
      additional active fail fixtures cover the mixed structure-RHS and
      attributed structure-RHS chain exclusions with the missing-expansion
      diagnostic.
      Imported or argument-bearing attributes/modes/structures, dependencies,
      chains, attributed structure RHSs, attributed-builtin RHSs, successful
      attributed or structure declarations, and base-shape/existential evidence
      remain outside the slice.
    - Verify: `cargo test -p mizar-test`, `cargo test -p mizar-checker`.
    - Deps: tasks 48, 50, 52, 53, 57, and 59. Structure base-shape evidence,
      full attributed-type existential evidence, and broader mode expansion
      remain MC-G020/MC-G026. Refs: Step 5 source-derived semantic bridge;
      mizar-test task 10; spec 03 type expressions; spec 05 structures; spec
      06 attributes; spec 07 modes; spec 17 attributed-type evidence.
    - Completed in task 60: `mizar-test` extracts a real AST-derived direct
      local-structure RHS expansion for a same-module attributed reserve head
      when the same mode has no mixed bare reserve use, and `mizar-checker`
      routes the expanded attributed reserve declaration to the existing
      missing evidence-query diagnostic rather than the missing expansion-
      payload diagnostic. Positive attributed/structure acceptance, base-
      shape/constructor-witness extraction, existential evidence extraction,
      mixed attributed/bare uses, dependencies or chains, CoreIr, ControlFlowIr,
      VC, proof payloads, and broader semantic pass coverage remain deferred.

61. **Source-derived attributed local mode attributed-builtin-RHS evidence-gap bridge.** [x]
    - Extend the task-58 attributed-builtin RHS bridge just far enough to
      produce a real `ModeExpansion` payload for a same-module attributed
      local-mode reserve head when its mode definition is unique, unrecovered,
      preceding, no-argument, free of definition-local context, and has a
      direct attributed builtin `set` / `object` RHS. The same mode must not
      also be used as a bare reserve head in the same bridge input.
    - Acceptance: checker unit coverage proves `marked Mode` with a real
      `Mode -> marked set` expansion no longer emits
      `checker.type.external.mode_expansion_payload`, preserves normalized
      attributes from the reserve head and the RHS, marks the declaration
      partial with `MissingEvidenceQuery`, and exports no verified facts.
      Runner unit coverage proves the single attributed local-mode reserve use
      receives the real direct attributed-builtin RHS expansion while mixed
      bare/attributed uses of the same mode and attributed dependency chains
      ending in attributed RHSs still withhold expansion. A new active
      `type_elaboration` fail fixture covers the real `.miz` source path with
      `type_elaboration.checker.checker.declaration.deferred.evidence_query`;
      additional active fail fixtures cover the mixed attributed-RHS and
      attributed-RHS chain exclusions with the missing-expansion diagnostic.
      Imported or argument-bearing attributes/modes, dependencies, chains,
      structure RHSs, attributed structure RHSs, successful attributed
      declarations, existential evidence extraction, and CoreIr/ControlFlowIr/
      VC/proof payloads remain outside the slice.
    - Verify: `cargo test -p mizar-test`, `cargo test -p mizar-checker`.
    - Deps: tasks 48, 50, 54, 55, 58, and 59. Full attributed-type
      existential evidence and broader mode expansion remain MC-G020/MC-G026.
      Refs: Step 5 source-derived semantic bridge; mizar-test task 10; spec 03
      type expressions; spec 06 attributes; spec 07 modes; spec 17
      attributed-type evidence.
    - Completed in task 61: `mizar-test` extracts a real AST-derived direct
      attributed-builtin RHS expansion for a same-module attributed reserve
      head when the same mode has no mixed bare reserve use, and
      `mizar-checker` routes the expanded attributed reserve declaration to the
      existing missing evidence-query diagnostic rather than the missing
      expansion-payload diagnostic. Positive attributed acceptance, existential
      evidence extraction, mixed attributed/bare uses, dependencies or chains,
      CoreIr, ControlFlowIr, VC, proof payloads, and broader semantic pass
      coverage remain deferred.

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
