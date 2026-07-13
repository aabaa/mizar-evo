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

62. **Add source-derived local mode structure-RHS chain evidence-gap bridge.** [x]
    - Extend the task-56 chain producer only for a bare same-module
      local-mode reserve head `A` whose unique unrecovered no-argument
      preceding mode definition is `A is B`, where `B` is a unique unrecovered
      no-argument same-module local mode whose own preceding definition is
      `B is LocalStruct`. The unique unrecovered same-module local structure
      definition must precede `B`; `B` must precede `A`; `A` must precede the
      reserve use; both mode definitions must be free of definition-local
      context.
    - Acceptance: runner unit coverage proves both real source-derived
      `B -> LocalStruct` and `A -> B` expansion payloads are extracted from the
      same `SurfaceAst`; cached direct structure-RHS payloads may feed this
      one-edge chain, but deeper chains remain withheld. A new active
      `type_elaboration` fail fixture covers the real `.miz` source path and
      reaches `type_elaboration.checker.checker.declaration.deferred.evidence_query`
      rather than `checker.type.external.mode_expansion_payload`. The checker
      emits no verified facts and positive structure acceptance remains
      deferred.
    - Verify: `cargo test -p mizar-test`, `cargo test -p mizar-checker`.
    - Deps: tasks 48, 52, 56, and 57. Structure base-shape/
      constructor-witness evidence and broader mode expansion remain
      MC-G020/MC-G026. Refs: Step 5 source-derived semantic bridge;
      mizar-test task 10; spec 03 type expressions; spec 05 structures;
      spec 07 modes; spec 17 evidence.
    - Completed in task 62: `mizar-test` extracts a real AST-derived
      one-edge bare local-mode chain ending in a same-module local structure
      RHS, and `mizar-checker` routes the expanded reserve declaration to the
      existing missing evidence-query diagnostic. Imported/ambiguous symbols,
      arguments, contextual or parameterized definitions, attributed roots,
      attributed or deeper chains, positive structure acceptance, CoreIr,
      ControlFlowIr, VC, proof payloads, and broader semantic pass coverage
      remain deferred.

63. **Add source-derived local mode attributed-builtin-RHS chain evidence-gap bridge.** [x]
    - Extend the task-56 chain producer only for a bare same-module
      local-mode reserve head `A` whose unique unrecovered no-argument
      preceding mode definition is `A is B`, where `B` is a unique unrecovered
      no-argument same-module local mode whose own preceding definition has a
      direct attributed builtin `set` / `object` RHS. `B` must precede `A`;
      `A` must precede the reserve use; both mode definitions must be free of
      definition-local context; and all RHS attributes must resolve to
      argument-free same-module attribute symbols.
    - Acceptance: runner unit coverage proves both real source-derived
      `B -> marked set` and `A -> B` expansion payloads are extracted from the
      same `SurfaceAst`; cached direct attributed-builtin-RHS payloads may feed
      this one-edge chain, but deeper chains and attributed roots remain
      withheld. A new active `type_elaboration` fail fixture covers the real
      `.miz` source path and reaches
      `type_elaboration.checker.checker.declaration.deferred.evidence_query`
      rather than `checker.type.external.mode_expansion_payload`. The checker
      emits no verified facts and positive attributed-type acceptance remains
      deferred.
    - Verify: `cargo test -p mizar-test`, `cargo test -p mizar-checker`.
    - Deps: tasks 48, 50, 56, 58, and 61. Full attributed-type existential
      evidence and broader mode expansion remain MC-G020/MC-G026. Refs: Step 5
      source-derived semantic bridge; mizar-test task 10; spec 03 type
      expressions; spec 06 attributes; spec 07 modes; spec 17 evidence.
    - Completed in task 63: `mizar-test` extracts a real AST-derived
      one-edge bare local-mode chain ending in an attributed builtin RHS, and
      `mizar-checker` routes the expanded reserve declaration to the existing
      missing evidence-query diagnostic. Imported/ambiguous symbols, attribute
      or mode arguments, contextual or parameterized definitions, attributed
      roots, attributed or deeper chains, positive attributed-type acceptance,
      CoreIr, ControlFlowIr, VC, proof payloads, and broader semantic pass
      coverage remain deferred.

64. **Add source-derived attributed local mode bare-builtin chain evidence-gap bridge.** [x]
    - Extend the task-59 attributed-root producer only for `reserve z for
      marked A` where `A` is a unique unrecovered no-argument same-module mode
      whose preceding definition is `A is B`, `B` is a unique unrecovered
      no-argument same-module mode whose preceding definition has a direct bare
      builtin `set` / `object` RHS, `B` precedes `A`, `A` precedes the reserve
      use, both mode definitions are free of definition-local context, `A` is
      not also used as a bare reserve head in the same bridge input, and `B` is
      not used as an attributed reserve head in the same bridge input.
    - Acceptance: runner unit coverage proves both real source-derived
      `B -> set` and `A -> B` expansion payloads plus the attributed reserve
      head are extracted from the same `SurfaceAst`; cached direct bare-builtin
      dependency payloads may feed this one-edge attributed-root chain, but
      deeper chains, attributed dependencies, mixed bare/attributed use of
      `A`, and attributed roots whose dependency terminates in a local
      structure RHS or attributed builtin RHS remain withheld. A new active
      `type_elaboration` fail fixture covers the real `.miz` source path and
      reaches `type_elaboration.checker.checker.declaration.deferred.evidence_query`
      rather than `checker.type.external.mode_expansion_payload`. The checker
      emits no verified facts and positive attributed-type acceptance remains
      deferred.
    - Verify: `cargo test -p mizar-test`, `cargo test -p mizar-checker`.
    - Deps: tasks 48, 50, 55, 56, and 59. Full attributed-type existential
      evidence and broader mode expansion remain MC-G020/MC-G026. Refs: Step 5
      source-derived semantic bridge; mizar-test task 10; spec 03 type
      expressions; spec 06 attributes; spec 07 modes; spec 17 evidence.

65. **Add source-derived attributed local mode structure-RHS chain evidence-gap bridge.** [x]
    - Extend the task-64 attributed-root chain producer only for `reserve z for
      marked A` where `A is B`, `B is LocalStruct`, `LocalStruct` is a unique
      unrecovered same-module structure definition preceding `B`, both mode
      definitions are unique, unrecovered, same-module, no-argument, and free of
      definition-local context, source order is `LocalStruct -> B -> A -> reserve`,
      `A` is not also used as a bare reserve head, and `B` is not used as an
      attributed reserve head in the same bridge input.
    - Acceptance: runner unit coverage proves both real source-derived
      `B -> LocalStruct` and `A -> B` expansion payloads plus the attributed
      reserve head are extracted from the same `SurfaceAst`; cached direct
      structure-RHS dependency payloads may feed this one-edge attributed-root
      chain, but attributed-builtin terminal dependencies, deeper chains,
      attributed dependencies, mixed bare/attributed `A`, imported/ambiguous
      symbols, arguments, and contextual/parameterized/recovered definitions
      remain withheld. The existing active structure-RHS chain `.miz` fixture
      must move from `checker.type.external.mode_expansion_payload` to
      `type_elaboration.checker.checker.declaration.deferred.evidence_query`.
      The checker emits no verified facts and positive structure/attributed-type
      acceptance remains deferred.
    - Verify: `cargo test -p mizar-test`, `cargo test -p mizar-checker`.
    - Deps: tasks 48, 50, 56, 60, 62, and 64. Structure base-shape /
      constructor-witness evidence, full attributed-type existential evidence,
      and broader mode expansion remain MC-G020/MC-G026. Refs: Step 5
      source-derived semantic bridge; mizar-test task 10; spec 03 type
      expressions; spec 05 structures; spec 06 attributes; spec 07 modes;
      spec 17 evidence.

66. **Add source-derived attributed local mode attributed-builtin-RHS chain evidence-gap bridge.** [x]
    - Extend the task-64/task-65 attributed-root chain producer only for
      `reserve z for marked A` where `A is B`, `B is marked set` or
      `B is marked object`, RHS attributes resolve to argument-free same-module
      attribute symbols, both mode definitions are unique, unrecovered,
      same-module, no-argument, and free of definition-local context, source
      order is `B -> A -> reserve`, `A` is not also used as a bare reserve head,
      and `B` is not used as an attributed reserve head in the same bridge input.
    - Acceptance: runner unit coverage proves both real source-derived
      `B -> marked set/object` and `A -> B` expansion payloads plus the
      attributed reserve head are extracted from the same `SurfaceAst`; mixed
      roots, attributed dependencies, deeper chains, imported/ambiguous symbols,
      attribute or mode arguments, and contextual/parameterized/recovered
      definitions remain withheld. The existing active attributed-RHS chain
      `.miz` fixture must move from `checker.type.external.mode_expansion_payload`
      to `type_elaboration.checker.checker.declaration.deferred.evidence_query`.
      The checker emits no verified facts and positive attributed-type
      acceptance remains deferred.
    - Verify: `cargo test -p mizar-test`, `cargo test -p mizar-checker`.
    - Deps: tasks 48, 50, 56, 61, 63, and 64. Full attributed-type existential
      evidence and broader mode expansion remain MC-G020/MC-G026. Refs: Step 5
      source-derived semantic bridge; mizar-test task 10; spec 03 type
      expressions; spec 06 attributes; spec 07 modes; spec 17 evidence.

67. **Add source-derived structure-qualified attribute gap boundary.** [x]
    - Add an active `type_elaboration` boundary fixture for a same-module
      structure-qualified attribute reference in a reserve type expression,
      for example `LocalStruct.marked LocalStruct`.
    - Acceptance: the active runner proves the real `.miz` source path is
      parser/resolver executable but remains on
      `type_elaboration.external_dependency.ast_payload_extraction` because the
      checker-owned attribute payload does not yet carry structure-qualifier or
      attribute-owner provenance. The bridge must not rewrite the reference to
      an unqualified attribute payload, infer positive attributed-structure
      acceptance, or fabricate existential/evidence, CoreIr, ControlFlowIr, VC,
      or proof payloads.
    - Verify: `cargo test -p mizar-test`, `cargo test -p mizar-checker`.
    - Deps: tasks 48, 50, 52, and 53. Qualified-attribute provenance,
      attribute-owner resolution, full attributed-type existential evidence,
      and broader attribute extraction remain MC-G020/MC-G026. Refs: Step 5
      source-derived semantic bridge; mizar-test task 10; spec 03 type
      expressions; spec 05 structures; spec 06 attributes.

68. **Add source-derived argument-bearing mode reserve gap boundary.** [x]
    - Add an active `type_elaboration` boundary fixture for a reserve type
      expression whose same-module local mode head has `of` type arguments,
      for example `Element of a`.
    - Acceptance: the active runner proves the real `.miz` source path is
      parser/resolver executable but remains on
      `type_elaboration.external_dependency.ast_payload_extraction` because the
      checker-owned reserve source bridge does not yet carry real
      type-argument or term-argument provenance. The boundary must not claim
      mode-argument payload extraction, arity matching, mode expansion,
      positive type elaboration, or CoreIr/ControlFlowIr/VC/proof payloads.
    - Verify: `cargo test -p mizar-test`, `cargo test -p mizar-checker`.
    - Deps: tasks 48, 51, and 55. Type-argument and term-argument provenance,
      argument-bearing mode expansion, arity checking, positive acceptance,
      and broader mode extraction remain MC-G020/MC-G014. Refs: Step 5
      source-derived semantic bridge; mizar-test task 10; spec 03 type
      expressions; spec 07 modes.

69. **Add source-derived argument-bearing structure reserve gap boundary.** [x]
    - Add an active `type_elaboration` boundary fixture for a reserve type
      expression whose same-module local structure declaration has an `of`
      parameter surface and whose reserve head has `of` type arguments, for
      example `LocalStruct of a`.
    - Acceptance: the active runner proves the real `.miz` source path is
      parser/resolver executable but remains on
      `type_elaboration.external_dependency.ast_payload_extraction` because the
      checker-owned reserve source bridge does not yet carry real
      type-argument or term-argument provenance. The boundary must not claim
      structure-argument payload extraction, arity matching, base-shape or
      constructor-witness evidence, positive structure type elaboration, or
      CoreIr/ControlFlowIr/VC/proof payloads.
    - Verify: `cargo test -p mizar-test`, `cargo test -p mizar-checker`.
    - Deps: tasks 48, 52, and 68. Type-argument and term-argument provenance,
      argument-bearing structure payloads, base-shape evidence, arity checking,
      positive acceptance, and broader structure extraction remain MC-G020.
      Refs: Step 5 source-derived semantic bridge; mizar-test task 10; spec 03
      type expressions; spec 05 structures.

70. **Add source-derived bracket-form local mode reserve gap boundary.** [x]
    - Add an active `type_elaboration` boundary fixture for source containing a
      same-module bracket-parameter mode declaration and a bracket-form reserve
      type head, for example `Family[set]`.
    - Acceptance: the active runner proves the real `.miz` source path is
      parser/resolver executable but remains on
      `type_elaboration.external_dependency.ast_payload_extraction` before
      bracket type-argument payload extraction or mode-head resolution because
      the checker-owned reserve source bridge does not yet carry real bracket
      type-argument or `qua`-argument provenance. The boundary must not claim
      bracket payload extraction, arity matching, mode expansion, positive type
      elaboration, or CoreIr/ControlFlowIr/VC/proof payloads.
    - Verify: `cargo test -p mizar-test`, `cargo test -p mizar-checker`.
    - Deps: tasks 48, 51, and 68. Bracket `type_arg_list` provenance,
      `qua`-argument lowering, mode-head resolution with arguments, arity
      checking, positive acceptance, and broader mode extraction remain
      MC-G020/MC-G014. Refs: Step 5 source-derived semantic bridge; mizar-test
      task 10; spec 03 type expressions; spec 07 modes.

71. **Add source-derived bracket-form local structure reserve gap boundary.** [x]
    - Add an active `type_elaboration` boundary fixture for source containing a
      same-module bracket-parameter structure declaration and a bracket-form
      reserve type head, for example `LocalStruct[set]`.
    - Acceptance: the active runner proves the real `.miz` source path is
      parser/resolver executable but remains on
      `type_elaboration.external_dependency.ast_payload_extraction` before
      bracket type-argument payload extraction or structure-head resolution
      because the checker-owned reserve source bridge does not yet carry real
      bracket type-argument or `qua`-argument provenance. The boundary must not
      claim bracket payload extraction, arity matching, base-shape or
      constructor-witness evidence, positive structure type elaboration, or
      CoreIr/ControlFlowIr/VC/proof payloads.
    - Verify: `cargo test -p mizar-test`, `cargo test -p mizar-checker`.
    - Deps: tasks 48, 52, and 69. Bracket `type_arg_list` provenance,
      `qua`-argument lowering, structure-head resolution with arguments, arity
      checking, positive structure acceptance, and broader structure extraction
      remain MC-G020/MC-G014. Refs: Step 5 source-derived semantic bridge;
      mizar-test task 10; spec 03 type expressions; spec 05 structures.

72. **Add source-derived two-edge bare local mode chain bridge.** [x]
    - Extend the task-56 pass producer only for bare same-module no-argument
      local-mode chains `Outer -> Middle -> Base -> set` / `object`.
    - Acceptance: the active runner extracts all three real `ModeExpansion`
      payloads from unique unrecovered same-module mode definitions in source
      order, with no definition-local context, no attributes, and no arguments,
      then the reserve declaration follows the existing `TypedAst`,
      `ResolvedTypedAst`, summary-readiness, and binder-only `CoreContext`
      preparation path. Cold and cached three-edge local-mode chains remained on
      `type_elaboration.checker.checker.type.external.mode_expansion_payload`
      so the two-edge cap could not silently broaden; task 73 later promoted
      the same seam to three edges, and task 74 later replaced the temporary
      depth guard with the AST-bounded structural rule.
    - Verify: `cargo test -p mizar-test`, `cargo test -p mizar-checker`.
    - Deps: tasks 48, 55, and 56. Attributed roots or dependencies,
      structure/attributed-builtin terminals beyond the existing one-edge
      diagnostic slices, imported/argument-bearing/parameterized/contextual/
      ambiguous/cyclic/forward-reference definitions, chains outside task 74's
      structural guards, CoreIr, ControlFlowIr, VC, proof payloads, and broader
      mode extraction remain MC-G020/MC-G014. Refs: Step 5 source-derived
      semantic bridge; mizar-test task 10; spec 03 type expressions; spec 07
      modes.

73. **Add source-derived three-edge bare local mode chain bridge.** [x]
    - Extend the task-72 pass producer only for bare same-module no-argument
      local-mode chains `Outer -> Middle -> Inner -> Base -> set` / `object`.
    - Acceptance: the active runner extracts all four real `ModeExpansion`
      payloads from unique unrecovered same-module mode definitions in source
      order, with no definition-local context, no attributes, and no arguments,
      then the reserve declaration follows the existing `TypedAst`,
      `ResolvedTypedAst`, summary-readiness, and binder-only `CoreContext`
      preparation path. Cold and cached four-edge local-mode chains remained on
      `type_elaboration.checker.checker.type.external.mode_expansion_payload`
      for task 73; task 74 later replaced that temporary depth guard with the
      AST-bounded structural rule.
    - Verify: `cargo test -p mizar-test`, `cargo test -p mizar-checker`.
    - Deps: tasks 48, 55, 56, and 72. Attributed roots or dependencies,
      structure/attributed-builtin terminals beyond the existing one-edge
      diagnostic slices, imported/argument-bearing/parameterized/contextual/
      ambiguous/cyclic/forward-reference definitions, chains outside task 74's
      structural guards, CoreIr, ControlFlowIr, VC, proof payloads, and broader
      mode extraction remain MC-G020/MC-G014. Refs: Step 5 source-derived
      semantic bridge; mizar-test task 10; spec 03 type expressions; spec 07
      modes.

74. **Add source-derived structural bare local mode chain bridge.** [x]
    - Replace the task-73 semantic chain-depth cap with a structural rule for
      bare same-module no-argument local-mode chains ending in builtin `set` /
      `object`.
    - Acceptance: the active runner extracts real `ModeExpansion` payloads for
      every link in an AST-bounded acyclic local-mode chain when each mode
      definition is unique, unrecovered, same-module, no-argument,
      definition-local-context-free, source-preceding, argument-free, and
      attribute-free, and the terminal RHS is exactly builtin `set` / `object`.
      The producer carries an AST-derived traversal budget equal to the number
      of source mode definitions, so resource safety is structural rather than
      a semantic chain-length limit. Four-edge, cached four-edge,
      object-terminal, and long-chain active pass fixtures continue through
      the existing `TypedAst`, `ResolvedTypedAst`, summary-readiness, and
      binder-only `CoreContext` preparation path without promoting CoreIr,
      ControlFlowIr, VC, or proof payloads. Chains that violate the structural
      guards remain fail-closed.
    - Verify: `cargo test -p mizar-test`, `cargo test -p mizar-checker`.
    - Deps: tasks 48, 55, 56, 72, and 73. Attributed roots or dependencies,
      structure/attributed-builtin terminals beyond the existing one-edge
      diagnostic slices, imported/argument-bearing/parameterized/contextual/
      ambiguous/cyclic/forward-reference definitions, structure or attributed
      evidence, CoreIr, ControlFlowIr, VC, proof payloads, and broader mode
      extraction remain MC-G020/MC-G014. Refs: Step 5 source-derived semantic
      bridge; mizar-test task 10; spec 03 type expressions; spec 07 modes;
      spec 17 base-shape inhabitation.

75. **Add source-derived local mode forward-reference active-range boundary.** [x]
    - Add active fail coverage for a reserve head that names a later
      same-module local mode declaration before that declaration item is
      active.
    - Acceptance: the active type-elaboration runner reports
      `type_elaboration.lower_stage.frontend:malformed_type_expression`
      before checker handoff, does not fabricate a `ModeExpansion` from the
      future declaration, and does not promote a successful reserve
      declaration, CoreIr, ControlFlowIr, VC, or proof payload. Forward
      reference acceptance remains forbidden by the Chapter 2/11 active-range
      rules.
    - Verify: `cargo test -p mizar-test`, `cargo test -p mizar-checker`.
    - Deps: tasks 48, 55, and 74. Refs: Step 5 source-derived semantic
      bridge; mizar-test task 10; spec 02 active range; spec 07 modes; spec 11
      symbol management.

76. **Add source-derived local structure forward-reference active-range boundary.** [x]
    - Add active fail coverage for a reserve head that names a later
      same-module local structure declaration before that declaration item is
      active.
    - Acceptance: the active type-elaboration runner reports
      `type_elaboration.lower_stage.frontend:malformed_type_expression`
      before checker handoff, does not fabricate a structure type-head payload
      from the future declaration, and does not promote a successful reserve
      declaration, base-shape/constructor-witness evidence query, CoreIr,
      ControlFlowIr, VC, or proof payload. Forward reference acceptance remains
      forbidden by the Chapter 2/11 active-range rules.
    - Verify: `cargo test -p mizar-test`, `cargo test -p mizar-checker`.
    - Deps: tasks 48, 52, and 75. Refs: Step 5 source-derived semantic
      bridge; mizar-test task 10; spec 02 active range; spec 05 structures;
      spec 11 symbol management.

77. **Add source-derived local attribute forward-reference active-range boundary.** [x]
    - Add active fail coverage for a reserve type that uses a later same-module
      local attribute declaration before that declaration item is active.
    - Acceptance: the active type-elaboration runner reports
      `type_elaboration.lower_stage.frontend:malformed_type_expression`
      before checker handoff, does not fabricate an `AttributeInput` from the
      future declaration, and does not promote a successful reserve
      declaration, attributed-type evidence query, CoreIr, ControlFlowIr, VC,
      or proof payload. Forward reference acceptance remains forbidden by the
      Chapter 2/11 active-range rules.
    - Verify: `cargo test -p mizar-test`, `cargo test -p mizar-checker`.
    - Deps: tasks 48, 50, 75, and 76. Refs: Step 5 source-derived semantic
      bridge; mizar-test task 10; spec 02 active range; spec 06 attributes;
      spec 11 symbol management.

78. **Add source-derived imported structure reserve extraction-gap boundary.** [x]
    - Add active fail coverage for a reserve type whose head is an imported
      structure symbol supplied by the existing `parser.type_fixtures` import
      summary.
    - Acceptance: the active type-elaboration runner reports
      `type_elaboration.external_dependency.ast_payload_extraction`, does not
      fabricate imported structure provenance, structure type-head payloads,
      base-shape/constructor-witness evidence, or positive structure
      elaboration, and does not promote CoreIr, ControlFlowIr, VC, or proof
      payloads. The fixture is diagnostic boundary coverage only.
    - Verify: `cargo test -p mizar-test`, `cargo test -p mizar-checker`.
    - Deps: tasks 48, 52, and 69. Refs: Step 5 source-derived semantic bridge;
      mizar-test task 10; spec 03 type expressions; spec 05 structures; spec
      11 symbol management; spec 12 modules and namespaces.

79. **Add source-derived imported mode reserve extraction-gap boundary.** [x]
    - Add active fail coverage for a reserve type whose head is an imported
      mode symbol supplied by the existing `parser.type_fixtures` import
      summary.
    - Acceptance: the active type-elaboration runner reports
      `type_elaboration.external_dependency.ast_payload_extraction`, does not
      fabricate imported mode provenance, mode type-head payloads,
      `ModeExpansion` payloads, positive mode elaboration, or broader imported
      mode semantics, and does not promote CoreIr, ControlFlowIr, VC, or proof
      payloads. The fixture is diagnostic boundary coverage only and only
      refines traceability for the generic non-builtin imported-mode gap.
      Task 82 supersedes only the documented `TypeCaseMode`
      provenance/type-head slice.
    - Verify: `cargo test -p mizar-test`, `cargo test -p mizar-checker`.
    - Deps: tasks 48, 51, 55, and 78. Refs: Step 5 source-derived semantic
      bridge; mizar-test task 10; spec 03 type expressions; spec 07 modes; spec
      11 symbol management; spec 12 modules and namespaces.

80. **Add source-derived imported attribute reserve extraction-gap boundary.** [x]
    - Add active fail coverage for a reserve type whose attribute is an imported
      attribute symbol supplied by the existing `parser.type_fixtures` import
      summary.
    - Acceptance: before tasks 84, 85, and 116, the active type-elaboration runner
      reported `type_elaboration.external_dependency.ast_payload_extraction`.
      After task 84 supersedes the documented `TypeCaseAttr` portion and task
      85 supersedes the negative `empty`/builtin-`set` portion, and task 116
      supersedes the positive `empty`/builtin-`set` portion, broader imported
      attributes outside those bridges still must not fabricate imported
      attribute provenance, `AttributeInput` payloads, attributed-type evidence,
      positive attributed type elaboration, or broader imported attribute
      semantics, and must not promote CoreIr, ControlFlowIr, VC, or proof
      payloads. The task remains historical diagnostic boundary coverage for
      the generic import-backed attributed reserve gap.
    - Verify: `cargo test -p mizar-test`, `cargo test -p mizar-checker`.
    - Deps: tasks 48, 50, 67, 78, and 79. Refs: Step 5 source-derived semantic
      bridge; mizar-test task 10; spec 03 type expressions; spec 06 attributes;
      spec 11 symbol management; spec 12 modules and namespaces.

81. **Add source-derived argument-bearing local attribute reserve extraction-gap boundary.** [x]
    - Add active fail coverage for a same-module parameterized attribute
      declared with `param_prefix` syntax and used through the Chapter 3/6
      `attribute_name(args)` application form in a reserve type expression.
    - Acceptance: the active type-elaboration runner reports
      `type_elaboration.external_dependency.ast_payload_extraction`, does not
      fabricate term-argument provenance, checker `AttributeInput` argument
      payloads, attributed-type evidence, positive attributed type elaboration,
      or broader parameterized attribute semantics, and does not promote
      CoreIr, ControlFlowIr, VC, or proof payloads. The fixture is diagnostic
      boundary coverage only and only proves that the real source lexer/parser
      producer seam and resolver declaration-symbol suffix projection can carry
      the parameterized local attribute surface to the checker-owned extraction
      boundary.
    - Verify: `cargo test -p mizar-test`, `cargo test -p mizar-checker`,
      `cargo test -p mizar-lexer`, `cargo test -p mizar-frontend`,
      `cargo test -p mizar-parser`.
    - Deps: tasks 48, 50, 67, and 77. Refs: Step 5 source-derived semantic
      bridge; mizar-test task 10; spec 02 lexical structure; spec 03 type
      expressions; spec 06 attributes; spec 11 symbol management;
      mizar-lexer disambiguator design;
      mizar-resolve symbol projection design.

82. **Add source-derived imported mode reserve provenance bridge.** [x]
    - Promote the task-79 imported-mode reserve boundary just far enough for
      the active `type_elaboration` runner to pass the real
      `parser.type_fixtures` import-summary `ImportedSource` mode symbol as a
      checker `TypeHeadInput`.
    - Acceptance: the checker reserve bridge validates that the imported mode
      symbol is visible through `SymbolEnv`, has `SymbolKind::Mode`, and is
      backed by an `ImportedSource` contribution rather than local source.
      The runner no longer reports
      `type_elaboration.external_dependency.ast_payload_extraction` for
      `TypeCaseMode`; it reaches
      `type_elaboration.checker.checker.type.external.mode_expansion_payload`
      because real imported mode-definition/module-summary expansion payloads
      are not available. The task must not synthesize imported module AST
      extraction, `ModeExpansion` payloads, arity checking, positive mode
      elaboration, CoreIr, ControlFlowIr, VC, or proof payloads, and it must
      leave imported structures, imported attributes, arguments, brackets,
      qualified attributes, and imported evidence on their existing gaps.
    - Verify: `cargo test -p mizar-test`, `cargo test -p mizar-checker`.
    - Deps: tasks 48, 51, 55, 78, and 79. Refs: Step 5 source-derived
      semantic bridge; mizar-test task 10; spec 03 type expressions; spec 07
      modes; spec 11 symbol management; spec 12 modules and namespaces.

83. **Add source-derived imported structure reserve provenance bridge.** [x]
    - Promote the task-78 imported-structure reserve boundary just far enough
      for the active `type_elaboration` runner to pass the documented
      `parser.type_fixtures` import-summary `R` structure symbol as a checker
      `TypeHeadInput`.
    - Acceptance: the checker reserve bridge validates that `R` is visible
      through `SymbolEnv`, has `SymbolKind::Structure`, and is backed by an
      `ImportedSource` contribution from `parser.type_fixtures`. The runner no
      longer reports `type_elaboration.external_dependency.ast_payload_extraction`
      for `R`; it reaches
      `type_elaboration.checker.checker.declaration.deferred.evidence_query`
      because imported module AST extraction and base-shape/constructor-witness
      evidence are not available. The task must not synthesize imported module
      AST extraction, base-shape/constructor-witness evidence, positive
      structure elaboration, CoreIr, ControlFlowIr, VC, or proof payloads, and
      it must leave generic imported structures outside the later task-97
      `TypeCaseStruct` slice, imported attributes, arguments, brackets,
      qualified attributes, and imported evidence on their existing gaps.
    - Verify: `cargo test -p mizar-test`, `cargo test -p mizar-checker`.
    - Deps: tasks 48, 52, 76, 78, and 82. Refs: Step 5 source-derived
      semantic bridge; mizar-test task 10; spec 03 type expressions; spec 05
      structures; spec 11 symbol management; spec 12 modules and namespaces.

84. **Add source-derived imported attribute reserve provenance bridge.** [x]
    - Promote the task-80 imported-attribute reserve boundary just far enough
      for the active `type_elaboration` runner to pass the documented
      `parser.type_fixtures` import-summary `TypeCaseAttr` attribute symbol as
      a checker `AttributeInput` on builtin `set`.
    - Acceptance: the checker reserve bridge validates that `TypeCaseAttr` is
      visible through `SymbolEnv`, has `SymbolKind::Attribute`, and is backed by
      an `ImportedSource` contribution from `parser.type_fixtures`. The runner
      no longer reports `type_elaboration.external_dependency.ast_payload_extraction`
      for `TypeCaseAttr set`; it reaches
      `type_elaboration.checker.checker.declaration.deferred.evidence_query`
      because imported module AST extraction and attributed-type
      existential/evidence payloads are not available. The task must not
      synthesize imported module AST extraction, attributed-type evidence,
      positive attributed type elaboration, CoreIr, ControlFlowIr, VC, or proof
      payloads, and it must leave generic imported attributes such as `empty`,
      structure-qualified owner provenance, arguments, brackets, qualified
      attributes, and imported evidence on their existing gaps.
    - Verify: `cargo test -p mizar-test`, `cargo test -p mizar-checker`.
    - Deps: tasks 48, 50, 67, 80, and 83. Refs: Step 5 source-derived
      semantic bridge; mizar-test task 10; spec 03 type expressions; spec 06
      attributes; spec 11 symbol management; spec 12 modules and namespaces.

85. **Add source-derived imported non-empty attribute reserve provenance bridge.** [x]
    - Promote the existing task-80 imported-attribute reserve boundary just far
      enough for the active `type_elaboration` runner to pass the documented
      `parser.type_fixtures` import-summary `empty` attribute symbol as a
      negative checker `AttributeInput` on builtin `set` for `non empty set`.
    - Acceptance: the checker reserve bridge validates that `empty` is visible
      through `SymbolEnv`, has `SymbolKind::Attribute`, is backed by an
      `ImportedSource` contribution from `parser.type_fixtures`, has negative
      polarity, and is attached to builtin `set`. The existing
      `fail_type_elaboration_attributed_reserve_gap_001` fixture no longer
      reports `type_elaboration.external_dependency.ast_payload_extraction`;
      it reaches
      `type_elaboration.checker.checker.declaration.deferred.evidence_query`
      because imported module AST extraction and attributed-type
      existential/evidence payloads are not available. The task must not
      synthesize imported module AST extraction, attributed-type evidence,
      positive `empty set` elaboration, imported `empty` on non-`set` heads,
      CoreIr, ControlFlowIr, VC, or proof payloads. Task 116 supersedes the
      positive `empty set` sidecar; this task leaves `non empty object`,
      attribute arguments, qualified owner provenance, and broader imported
      attributes on their existing gaps.
    - Verify: `cargo test -p mizar-test`, `cargo test -p mizar-checker`.
    - Deps: tasks 48, 50, 80, and 84. Refs: Step 5 source-derived semantic
      bridge; mizar-test task 10; spec 03 type expressions; spec 06
      attributes; spec 11 symbol management; spec 12 modules and namespaces.

86. **Add source-derived theorem formula extraction-gap boundary.** [x]
    - Add a dedicated active `type_elaboration` boundary for a formula-only
      theorem source such as `theorem FormulaPayloadBoundary: thesis;`.
    - Historical acceptance: parser and resolver execute the source, then the
      active runner reports `type_elaboration.external_dependency.ast_payload_extraction`
      because checker-owned theorem/formula payload extraction, local proof
      contexts, recorded facts, theorem acceptance, CoreIr, ControlFlowIr, VC,
      proof payloads, and the `formula_statement` runner are not available.
      Task 115 supersedes only this exact formula-only theorem source by
      passing the source-derived `thesis` formula constant site/range to the
      checker as a recovery `FormulaInput`. The historical task must not be
      read as broader formula payload extraction, theorem acceptance, facts,
      proof skeletons, or downstream semantic payloads.
    - Verify: `cargo test -p mizar-test`.
    - Deps: task 48. Refs: Step 5 source-derived semantic bridge; mizar-test
      task 10; spec 14 formulas; spec 16 theorems and proofs.

115. **Add exact source-derived formula statement recovery checker bridge.** [x]
    - Supersede task 86 only for the exact unrecovered source
      `theorem FormulaPayloadBoundary: thesis;`.
    - Acceptance: parser and resolver execute the source; the active runner
      validates that the module contains exactly one theorem item with one
      `FormulaExpression` child containing direct `FormulaConstant(Thesis)`
      token text `thesis`, then passes that source site/range as a checker
      recovery `FormulaInput` with the source-derived formula-constant site.
      Task 117 supersedes this recovery marker by giving the same source a real
      `FormulaKind::Thesis` payload while preserving the missing-formula
      fail-closed diagnostic.
    - The task must not fabricate formula constant semantics, child-formula
      graph payloads, theorem acceptance, facts, proof skeleton/context/
      statement payloads, `formula_statement`, CoreIr, ControlFlowIr, VC, or
      proof payloads. Non-exact shapes, including proof blocks and additional
      items, remain on `type_elaboration.external_dependency.ast_payload_extraction`.
    - Verify: `cargo test -p mizar-test`.
    - Deps: tasks 86 and 112. Refs: Step 5 source-derived semantic bridge;
      mizar-test task 10; spec 14 formulas; spec 16 theorems and proofs.

116. **Add source-derived imported positive empty attribute reserve provenance bridge.** [x]
    - Promote the existing task-80 positive imported-attribute reserve boundary
      just far enough for the active `type_elaboration` runner to pass the
      documented `parser.type_fixtures` import-summary `empty` attribute symbol
      as a positive checker `AttributeInput` on builtin `set` for `empty set`.
    - Acceptance: the checker reserve bridge validates that `empty` is visible
      through `SymbolEnv`, has `SymbolKind::Attribute`, is backed by an
      `ImportedSource` contribution from `parser.type_fixtures`, has positive
      polarity, and is attached to builtin `set`. The existing
      `fail_type_elaboration_imported_empty_positive_gap_001` fixture no
      longer reports `type_elaboration.external_dependency.ast_payload_extraction`;
      it reaches
      `type_elaboration.checker.checker.declaration.deferred.evidence_query`
      because imported module AST extraction and attributed-type
      existential/evidence payloads are not available. The task must not
      synthesize imported module AST extraction, attributed-type evidence,
      positive attributed-type acceptance, imported `empty` on non-`set` heads,
      CoreIr, ControlFlowIr, VC, or proof payloads, and it must leave
      `non empty object`, attribute arguments, qualified owner provenance, and
      broader imported attributes on their existing gaps.
    - Verify: `cargo test -p mizar-test`, `cargo test -p mizar-checker`.
    - Deps: tasks 48, 50, 80, 84, and 85. Refs: Step 5 source-derived semantic
      bridge; mizar-test task 10; spec 03 type expressions; spec 06
      attributes; spec 11 symbol management; spec 12 modules and namespaces.

117. **Add source-derived formula constant kind checker bridge.** [x]
    - Supersede task 115 for the exact unrecovered
      `theorem FormulaPayloadBoundary: thesis;` source by passing the
      source-derived formula constant as `FormulaKind::Thesis`, not the generic
      unsupported recovery kind.
    - Extend task 112 only for the exact unrecovered
      `FormulaConnectiveQuantifierPayloadBoundary: contradiction implies for x
      being set holds not contradiction` theorem source by passing both real
      `contradiction` constant sites/ranges as `FormulaKind::Contradiction`
      payloads alongside the existing implication, quantifier, and negation
      shell payloads.
    - Acceptance: parser and resolver execute the sources; the active runner
      validates the exact supported AST shapes, passes source-derived checker
      formula constant payloads to `TermFormulaChecker`, and still fails closed
      on `type_elaboration.checker.checker.formula.external.formula_payload`
      plus the existing quantifier payload diagnostic for the connective case.
    - The task must not fabricate formula constant semantic truth values,
      child-formula graph links, quantifier binder/context payloads, formula
      checking, facts, theorem acceptance, proof skeleton/context/statement
      payloads, `formula_statement`, CoreIr, ControlFlowIr, VC, or proof
      payloads. Non-exact shapes remain on
      `type_elaboration.external_dependency.ast_payload_extraction`.
    - Verify: `cargo test -p mizar-checker`, `cargo test -p mizar-test`.
    - Deps: tasks 86, 99, 112, and 115. Refs: Step 5 source-derived semantic
      bridge; mizar-test task 10; spec 14 formulas; spec 16 theorems and
      proofs.

118. **Tighten builtin binary theorem exact-token guard.** [x]
    - Repair the source-derived producer guard for the shared task 106/107/108
      builtin-binary numeral theorem bridge. The active runner now selects an
      equality, inequality, or membership config only when the theorem item has
      the exact direct token slice `theorem <label> : ;`, not merely because the
      label appears among additional theorem tokens.
    - Acceptance: the existing exact active `.miz` sidecars and checker
      term/formula handoff payloads remain unchanged; status-prefixed or
      otherwise extra-token builtin-binary theorem shapes stay on
      `type_elaboration.external_dependency.ast_payload_extraction`.
    - This task must not broaden labels, operators, literals, or accepted
      theorem surfaces, and must not fabricate numeric type payloads, formula
      checking, facts, theorem acceptance, `formula_statement`, CoreIr,
      ControlFlowIr, VC, or proof payloads. It adds no new active sidecar or
      spec coverage credit, so `doc/design/spec_coverage_audit.md` is
      unchanged.
    - Verify: `cargo test -p mizar-test`.
    - Deps: tasks 106, 107, and 108. Refs: Step 5 source-derived semantic
      bridge; mizar-test task 10; spec 13 term expressions; spec 14 formulas;
      spec 16 theorems and proofs.

119. **Add exact source-derived reserved-variable equality checker bridge.** [x]
    - Promote only the exact unrecovered source
      `reserve x for set; theorem ReservedVariableEqualityPayloadBoundary: x = x;`.
    - Acceptance: parser and resolver execute the source; the runner reuses the
      real reserve declaration handoff, resolves both identifier terms through
      checker-owned `BindingEnv::lookup`, projects the written builtin `set`
      reserve type into distinct result/expected-type role sites, and passes two
      variable `TermInput`s plus one equality `FormulaInput` to
      `TermFormulaChecker`. Both terms are `Inferred`, the formula is `Checked`,
      and the active type-elaboration pass case has no diagnostics or facts.
    - `Checked` is limited to type/well-formedness. The task must not fabricate
      implicit universal-closure nodes, equality facts/truth, theorem
      acceptance, proof skeletons, `formula_statement`, CoreIr, ControlFlowIr,
      VC, or proof payloads. Non-exact sources remain on the payload-extraction
      gap.
    - Verify: `cargo test -p mizar-test`; final workspace verification.
    - Deps: tasks 20, 48, 106, and 118. Refs: Step 5 source-derived semantic
      bridge; mizar-test task 10; spec 04 reserved variables; spec 13 term
      expressions; spec 14 formulas; spec 16 theorems and proofs.

120. **Add exact source-derived reserved-variable membership checker bridge.** [x]
    - Promote only the exact unrecovered source
      `reserve x for set; theorem ReservedVariableMembershipPayloadBoundary: x in x;`.
    - Acceptance: reuse task 119's real reserve handoff and independent
      source-order `BindingEnv` lookups; pass two known builtin-`set` variable
      result payloads, the right operand's single expected-`set` payload, and a
      membership `FormulaInput` to `TermFormulaChecker`. Require two `Inferred`
      terms, one no-fact `Checked` membership, exact three role owners, empty
      candidate/deferred/diagnostic output, a task-specific invalid-payload key,
      and a real frontend/resolver active-sidecar payload test.
    - `Checked` is type/well-formedness only. Do not fabricate membership
      truth/facts, implicit closure, theorem acceptance, `formula_statement`,
      proof, CoreIr, ControlFlowIr, or VC payloads. Non-exact sources remain on
      the extraction gap.
    - Verify: `cargo test -p mizar-test`; final workspace verification.
    - Deps: tasks 108, 119. Refs: Step 5; mizar-test task 10; spec 04, 13, 14,
      and 16.

121. **Add exact source-derived reserved-variable inequality checker bridge.** [x]
    - Promote only `reserve x for set; theorem ReservedVariableInequalityPayloadBoundary: x <> x;`.
    - Reuse the shared real lookup/type producer for two linked result and two
      expected roles, two `Inferred` terms, and one fact-free pre-desugaring
      `Checked` inequality. Guard it with a task-specific invalid key, near-miss
      matrix, and real frontend/resolver payload test.
    - Do not claim inequality desugaring/truth/facts, implicit closure, theorem
      acceptance, proof, CoreIr, ControlFlowIr, or VC.
    - Deps: tasks 107, 119, 120. Verify mizar-test and full workspace.

122. **Add checker reflexive type-assertion admissibility and its exact reserved-variable source bridge.** [x]
    - Repair `TermFormulaChecker` so type assertions require one ready subject
      and one asserted type, accept normalized identity only as the currently
      supported reflexive reachability case, and defer known non-identical types
      on `checker.formula.external.type_assertion_reachability_payload`.
    - Promote only
      `reserve x for set; theorem ReservedVariableTypeAssertionPayloadBoundary: x is set;`
      by combining task 119's real reserve lookup/result producer with task
      109's formula-side asserted-type AST producer. Preserve both
      pre-normalization inputs and validate their independent source anchors.
    - Require one `Inferred` variable and one fact-free `Checked` type assertion;
      keep general reachability/widening/`qua`, attributes, truth/facts,
      implicit closure, theorem acceptance, `formula_statement`, proof, CoreIr,
      ControlFlowIr, and VC deferred.
    - Deps: tasks 109, 119. Verify mizar-checker, mizar-test, and full workspace.

123. **Add exact source-derived distinct reserved-variable equality checker bridge.** [x]
    - Add a spec-derived active pass fixture for only
      `reserve x, y for set; theorem DistinctReservedVariableEqualityPayloadBoundary: x = y;`.
    - Acceptance: parser and resolver execute the source; the runner reuses the
      real multi-reserve declaration handoff, preserves two distinct checker
      binding identities whose source type ranges both point to the one written
      `set` type, and resolves `x` and `y` independently through source-order
      `BindingEnv::lookup` sites. Operand-specific result and expected-type role
      inputs must reach two `Inferred` variable terms and one no-fact `Checked`
      equality without candidates, deferred reasons, or diagnostics.
    - Add production invariant validation, a near-miss matrix, and a real
      frontend/resolver active-sidecar payload test. Existing expectations must
      not be rebaselined; the new pass expectation is derived from spec 4.3,
      13.1.1, 14.5.2, and the theorem declaration contract.
    - This task credits only exact distinct-binding type/well-formedness. It
      must not fabricate implicit universal-closure or quantifier-order nodes,
      equality truth/facts, theorem acceptance, `formula_statement`, proof,
      CoreIr, ControlFlowIr, or VC payloads. Non-exact multi-binding sources
      remain on the extraction gap.
    - Update the Chapters 4, 13, 14, and 16 rows in
      `doc/design/spec_coverage_audit.md`; no checker source-layout inventory
      update is required unless `crates/mizar-checker/src/` changes.
    - Deps: tasks 20 and 119. Verify mizar-test and the full workspace.

124. **Add exact source-derived multiple-reserve-declaration equality checker bridge.** [x]
    - Add a spec-derived active pass fixture for only
      `reserve x for set; reserve y for set; theorem MultipleReserveDeclarationEqualityPayloadBoundary: x = y;`.
    - Acceptance: parser and resolver execute both reserve declarations and the
      theorem; the runner reuses the real two-binding handoff, preserves
      `BindingId(0)` / `BindingId(1)`, derives use ordinals after both source
      bindings, and retains distinct written type ranges in operand-specific
      pre-normalization result and expected `TypeExpressionInput`s.
    - Semantic builtin `set` may intern to one `NormalizedTypeId` whose source
      is the deterministic canonical representative. Validate both original
      inputs before normalization; do not require or fabricate duplicate
      normalized nodes merely to attach both source ranges.
    - Require two `Inferred` variable terms and one fact-free `Checked`
      equality, production invariant validation, a task-specific invalid key,
      a near-miss matrix, and a real frontend/resolver sidecar test. Existing
      expectations must not be rebaselined.
    - Credit exact multiple-declaration type/well-formedness only. Do not
      materialize implicit closure/order, equality truth/facts, theorem
      acceptance, `formula_statement`, proof, CoreIr, ControlFlowIr, or VC.
    - Update Chapters 4, 13, 14, and 16 in the spec coverage audit. No checker
      source-layout update is required unless `crates/mizar-checker/src/`
      changes.
    - Deps: tasks 20, 119, and 123. Verify mizar-test and the full workspace.

125. **Add exact source-derived heterogeneous-reserve membership checker bridge.** [x]
    - Add a spec-derived active pass fixture for only
      `reserve x for object; reserve y for set; theorem HeterogeneousReserveMembershipPayloadBoundary: x in y;`.
    - Acceptance: parser and resolver execute both reserve declarations and the
      theorem; the runner reuses the real mixed-builtin two-binding handoff,
      preserves `BindingId(0)` / `BindingId(1)` and source-derived lookup
      ordinals, and retains both distinct written type ranges in a left
      `object` result input, right `set` result input, and right expected-`set`
      input.
    - Require exactly two normalized semantic identities: the right result and
      expected inputs must share the `set` identity, while the left `object`
      identity remains distinct. Each normalized source representative must be
      derived from its original written input; do not collapse the two types or
      fabricate duplicate semantic nodes.
    - Require two `Inferred` variable terms, one fact-free `Checked` membership,
      production invariant validation, a task-specific invalid key, an exact
      near-miss matrix, and a real frontend/resolver sidecar test. Existing
      expectations must not be rebaselined.
    - Credit exact heterogeneous membership type/well-formedness only. Do not
      materialize membership truth/facts, object/set coercion evidence,
      implicit closure/order, theorem acceptance, `formula_statement`, proof,
      CoreIr, ControlFlowIr, or VC.
    - Update Chapters 3, 4, 13, 14, and 16 in the spec coverage audit. No
      checker source-layout update is required unless `crates/mizar-checker/src/`
      changes.
    - Deps: tasks 20, 120, and 124. Verify mizar-test and the full workspace.

126. **Add exact direct-local-mode reserved-variable equality checker bridge.** [x]
    - Add a spec-derived active pass fixture containing exactly one unique,
      unrecovered, source-preceding no-argument local mode definition with bare
      builtin `set` RHS, `reserve x` for that mode, and
      `theorem LocalModeReservedVariableEqualityPayloadBoundary: x = x;`.
    - Acceptance: reuse the task-55 real AST-derived `ModeExpansion`, preserve
      the local-mode symbol/range in all four pre-normalization result/expected
      inputs, and construct `TermFormulaChecker` with the extracted expansion.
      Both source-order lookups resolve `BindingId(0)` and the checker normalizes
      every role to one builtin-`set` identity.
    - Require two `Inferred` variables, one fact-free `Checked` equality,
      production invariant validation, a task-specific invalid key, near misses
      for every withheld mode-definition family, and a real frontend/resolver
      sidecar test. Existing expectations must not be rebaselined.
    - Credit only the exact mode-backed identifier equality type/well-formedness
      slice. Do not credit mode definition declaration checking, accepted mode
      status or inhabitation evidence, broader/chained modes, closure/order,
      truth/facts, theorem acceptance, `formula_statement`, proof, CoreIr,
      ControlFlowIr, or VC.
    - Update Chapters 4, 7, 13, 14, and 16 in the spec coverage audit. No checker
      source-layout update is required unless `crates/mizar-checker/src/` changes.
    - Deps: tasks 55 and 119. Verify mizar-test and the full workspace.

127. **Add exact one-edge local-mode-chain reserved-variable equality checker bridge.** [x]
    - Add a spec-derived active pass fixture with exactly two separate unique,
      unrecovered, source-preceding, no-argument mode definitions
      `BaseModeFormula -> set` and `ChainModeFormula -> BaseModeFormula`, one
      `reserve x for ChainModeFormula`, and theorem
      `ChainedLocalModeReservedVariableEqualityPayloadBoundary: x = x;`.
    - Acceptance: reuse both real task-56 AST-derived `ModeExpansion` entries,
      retain the outer `ChainModeFormula` symbol/range in all four raw
      result/expected inputs, and recursively normalize every role to one
      builtin-`set` identity whose canonical source is the terminal `set` RHS.
      Both source-order lookups must resolve `BindingId(0)`.
    - Require two `Inferred` variables, one fact-free `Checked` equality,
      production invariant validation, a task-specific invalid key, an exact
      withheld-family near-miss matrix, and a real frontend/resolver sidecar.
      Existing expectations must not be rebaselined.
    - Credit only this exact one-edge-chain identifier equality
      type/well-formedness slice. Keep mode declaration checking/acceptance,
      inhabitation evidence, object terminals, longer-chain formulas,
      closure/order, truth/facts, theorem acceptance, `formula_statement`, proof,
      CoreIr, ControlFlowIr, and VC deferred.
    - Update Chapters 4, 7, 13, 14, and 16 in the spec coverage audit. No checker
      source-layout update is required unless `crates/mizar-checker/src/` changes.
    - Deps: tasks 56 and 126. Verify mizar-test and the full workspace.

128. **Add exact direct local-object-mode reserved-variable equality checker bridge.** [x]
    - Add a spec-derived active pass fixture with exactly one unique,
      unrecovered, source-preceding, no-argument definition
      `mode LocalObjectModeDef: LocalObjectMode is object;`, one
      `reserve x for LocalObjectMode`, and theorem
      `LocalObjectModeReservedVariableEqualityPayloadBoundary: x = x;`.
    - Acceptance: reuse the real task-55 AST-derived object `ModeExpansion`,
      retain `LocalObjectMode` symbol/range in all four raw result/expected
      inputs, normalize every role to one builtin-`object` identity whose
      canonical source is the real RHS, and resolve both uses to `BindingId(0)`.
    - Require two `Inferred` variables, one fact-free `Checked` equality,
      production validation, task invalid key, withheld-family near misses, and
      a real frontend/resolver sidecar. Do not rebaseline existing expectations.
    - Credit only this exact direct object-mode equality type/well-formedness
      slice. Keep mode declaration checking/acceptance, inhabitation evidence,
      closure/order, truth/facts, theorem acceptance, `formula_statement`, proof,
      CoreIr, ControlFlowIr, and VC deferred.
    - Update Chapters 3, 4, 7, 13, 14, and 16 in the spec coverage audit. No
      checker source-layout update is required unless checker source changes.
    - Deps: tasks 55 and 126. Verify mizar-test and the full workspace.

129. **Add exact one-edge local-object-mode-chain reserved-variable equality checker bridge.** [x]
    - Add a spec-derived active pass fixture that reuses task 56's exact
      `BaseObjectMode -> object` and `ChainObjectMode -> BaseObjectMode`
      definition blocks, one `reserve z for ChainObjectMode`, and theorem
      `ChainedLocalObjectModeReservedVariableEqualityPayloadBoundary: z = z;`.
    - Acceptance: retain `ChainObjectMode` symbol/range in all four raw
      result/expected inputs, consume both real expansions, normalize every role
      to one builtin-`object` identity whose canonical source is the terminal
      RHS, and resolve both uses to `BindingId(0)`.
    - Require two `Inferred` variables, one fact-free `Checked` equality,
      production validation, invalid-link corruption, withheld-family near
      misses, and a real frontend/resolver sidecar. Do not rebaseline existing
      expectations.
    - Credit only this exact one-edge object-terminal equality
      type/well-formedness slice. Keep mode declaration acceptance/inhabitation,
      closure/order, truth/facts, theorem acceptance, `formula_statement`, proof,
      CoreIr, ControlFlowIr, and VC deferred.
    - Update Chapters 3, 4, 7, 13, 14, and 16 in the spec coverage audit. No
      checker source-layout update is required unless checker source changes.
    - Deps: tasks 56, 127, and 128. Verify mizar-test and the full workspace.

130. **Add exact direct-local-mode reserved-variable inequality checker bridge.** [x]
    - Add the spec-derived active pass source with exact bare-set
      `LocalModeInequality`, one reserve, and `x <> x` theorem.
    - Preserve four raw mode-headed result/expected inputs, consume the one real
      expansion, anchor one builtin-set identity at the RHS, resolve both uses
      to `BindingId(0)`, and require a fact-free pre-desugaring `Checked`
      inequality. Exact/near-miss/corruption and real-sidecar guards are required.
    - Keep declaration acceptance/inhabitation, desugaring, closure/order,
      truth/facts, theorem acceptance, proof/Core/ControlFlow/VC deferred.
    - Update Chapters 4, 7, 13, 14, and 16. Deps: tasks 55 and 121.

131. **Add exact direct-local-object-mode reserved-variable inequality checker bridge.** [x]
    - Add the spec-derived active pass source with exact bare-object
      `LocalObjectModeInequality`, one reserve, and `x <> x` theorem.
    - Preserve four raw object-mode-headed result/expected inputs, consume the
      one real expansion, anchor one builtin-object identity at the RHS,
      resolve both uses to `BindingId(0)`, and require a fact-free
      pre-desugaring `Checked` inequality. Exact/near-miss/corruption and
      real-sidecar guards are required.
    - Keep mode declaration acceptance/inhabitation, desugaring, closure/order,
      truth/facts, theorem acceptance, proof/Core/ControlFlow/VC deferred.
    - Update Chapters 3, 4, 7, 13, 14, and 16. Deps: tasks 55, 121, 128, and 130.

132. **Add exact one-edge local-mode-chain reserved-variable inequality checker bridge.** [x]
    - Add the spec-derived active pass source with exact
      `ChainModeInequality -> BaseModeInequality -> set`, one outer reserve, and
      `x <> x` theorem.
    - Preserve four raw outer-mode result/expected inputs, consume both real
      expansions, anchor one builtin-set identity at the terminal RHS, resolve
      both uses to `BindingId(0)`, and require a fact-free pre-desugaring
      `Checked` inequality. Exact/near-miss/link-corruption and real-sidecar
      guards are required.
    - Keep mode declaration acceptance/inhabitation, object/direct/longer
      shapes, desugaring, closure/order, truth/facts, theorem acceptance,
      proof/Core/ControlFlow/VC deferred.
    - Update Chapters 4, 7, 13, 14, and 16. Deps: tasks 56, 121, 127, and 130.

133. **Add exact one-edge local-object-mode-chain reserved-variable inequality checker bridge.** [x]
    - Add only the spec-derived `ChainObjectModeInequality -> BaseObjectModeInequality -> object`, one outer reserve, and `z <> z` theorem source.
    - Preserve four raw outer-mode inputs and `BindingId(0)`, consume both real expansions, anchor one terminal-RHS builtin-object identity, and require two `Inferred` terms plus one fact-free pre-desugaring `Checked` inequality. Exact, link-corruption, withheld-family near-miss, and real-sidecar guards are required.
    - Keep declaration acceptance/inhabitation, desugaring, closure/order, truth/facts, theorem acceptance, proof/Core/ControlFlow/VC deferred.
    - Update Chapters 3, 4, 7, 13, 14, and 16. Deps: tasks 121, 129, and 131.

134. **Add exact two-edge local-mode-chain reserved-variable equality checker bridge.** [x]
    - Add only the spec-derived `OuterTwoEdgeModeEquality -> MiddleTwoEdgeModeEquality -> BaseTwoEdgeModeEquality -> set`, one outer reserve, and `z = z` theorem source.
    - Preserve four raw outer-mode inputs and `BindingId(0)`, consume all three real expansions, anchor one terminal-RHS builtin-set identity, and require two `Inferred` terms plus one fact-free `Checked` equality. Exact, link-corruption, withheld-family near-miss, and real-sidecar guards are required.
    - Keep declaration acceptance/inhabitation, implicit closure/order, truth/facts, theorem acceptance, proof/Core/ControlFlow/VC deferred.
    - Update Chapters 4, 7, 13, 14, and 16. Deps: tasks 72 and 127.

135. **Add exact two-edge local-object-mode-chain reserved-variable equality checker bridge.** [x]
    - Add only the spec-derived `OuterTwoEdgeObjectModeEquality -> MiddleTwoEdgeObjectModeEquality -> BaseTwoEdgeObjectModeEquality -> object`, one outer reserve, and `z = z` theorem source.
    - Preserve four raw outer-mode inputs and `BindingId(0)`, consume all three real expansions, anchor one terminal-RHS builtin-object identity, and require two `Inferred` terms plus one fact-free `Checked` equality. Exact, link-corruption, withheld-family near-miss, and real-sidecar guards are required.
    - Keep declaration acceptance/inhabitation, implicit closure/order, truth/facts, theorem acceptance, proof/Core/ControlFlow/VC deferred.
    - Update Chapters 3, 4, 7, 13, 14, and 16. Deps: tasks 72 and 134.

136. **Add exact two-edge local-mode-chain reserved-variable inequality checker bridge.** [x]
    - Add only the spec-derived `OuterTwoEdgeModeInequality -> MiddleTwoEdgeModeInequality -> BaseTwoEdgeModeInequality -> set`, one outer reserve, and `z <> z` theorem source.
    - Preserve four raw outer-mode inputs and `BindingId(0)`, consume all three real expansions, anchor one terminal-RHS builtin-set identity, and require two `Inferred` terms plus one fact-free pre-desugaring `Checked` inequality. Exact, link-corruption, withheld-family near-miss, and real-sidecar guards are required.
    - Keep mode declaration acceptance/inhabitation, inequality desugaring, implicit closure/order, truth/facts, theorem acceptance, proof/Core/ControlFlow/VC deferred.
    - Update Chapters 4, 7, 13, 14, and 16. Deps: tasks 72 and 132.

137. **Add exact two-edge local-object-mode-chain reserved-variable inequality checker bridge.** [x]
    - Add only the spec-derived `OuterTwoEdgeObjectModeInequality -> MiddleTwoEdgeObjectModeInequality -> BaseTwoEdgeObjectModeInequality -> object`, one outer reserve, and `z <> z` theorem source.
    - Preserve four raw outer-mode inputs and `BindingId(0)`, consume all three real expansions, anchor one terminal-RHS builtin-object identity, and require two `Inferred` terms plus one fact-free pre-desugaring `Checked` inequality. Exact, link-corruption, withheld-family near-miss, and real-sidecar guards are required.
    - Keep declaration acceptance/inhabitation, inequality desugaring, implicit closure/order, truth/facts, theorem acceptance, proof/Core/ControlFlow/VC deferred.
    - Update Chapters 3, 4, 7, 13, 14, and 16. Deps: tasks 72 and 133.

138. **Add exact direct-local-mode reserved-variable normalized-reflexive type assertion checker bridge.** [x]
    - Add only the spec-derived `LocalModeTypeAssertion -> set`, one reserve of that mode, and `x is set` theorem source.
    - Preserve the raw local-mode subject and independent formula-side builtin-set asserted-type inputs plus `BindingId(0)`, consume the one real expansion, anchor one terminal-RHS builtin-set identity, and require one `Inferred` term plus one fact-free `Checked` type assertion. Exact, expansion-corruption, withheld-family near-miss, and real-sidecar guards are required.
    - Keep mode declaration acceptance/inhabitation, formula-side local-mode asserted heads, general reachability/widening/`qua`, truth/facts, theorem acceptance, proof/Core/ControlFlow/VC deferred.
    - Update Chapters 3, 4, 7, 13, 14, and 16. Deps: tasks 55 and 122.

139. **Add exact direct-local-mode left reserved-variable membership checker bridge.** [x]
    - Add only the spec-derived `LocalModeMembership -> set`, ordered reserves `x` for that mode and `y` for explicit `set`, and `x in y` theorem source.
    - Preserve the raw local-mode left result and independent right result/expected-set inputs plus `BindingId(0/1)`, consume the one real expansion, anchor one terminal-RHS builtin-set identity, and require two `Inferred` terms plus one fact-free `Checked` membership with only the right expected constraint. Exact, expansion/right-expected corruption, withheld-family near-miss, and real-sidecar guards are required.
    - Keep mode declaration acceptance/inhabitation, membership truth/facts, implicit closure/order, theorem acceptance, proof/Core/ControlFlow/VC deferred.
    - Update Chapters 4, 7, 13, 14, and 16. Deps: tasks 55, 120, and 125.

87. **Add source-derived term formula extraction-gap boundary.** [x]
    - Add a dedicated active `type_elaboration` boundary for a theorem formula
      containing source terms, such as
      `theorem TermFormulaPayloadBoundary: 1 = 1;`.
    - Acceptance: parser and resolver execute the source, then the active runner
      originally reports `type_elaboration.external_dependency.ast_payload_extraction`.
      Task 106 supersedes this exact builtin equality slice by extracting real
      checker term/formula payloads while still failing closed on missing
      numeric type payloads and partial formula checking.
    - Verify: `cargo test -p mizar-test`.
    - Deps: task 86. Refs: Step 5 source-derived semantic bridge; mizar-test
      task 10; spec 13 term expressions; spec 14 formulas; spec 16 theorems and
      proofs.

88. **Add source-derived proof skeleton extraction-gap boundary.** [x]
    - Add a dedicated active `type_elaboration` boundary for a theorem with a
      proof block and conclusion statement, such as
      `theorem ProofSkeletonPayloadBoundary: thesis proof thus thesis; end;`.
    - Acceptance: parser and resolver execute the source, then the active runner
      reports `type_elaboration.external_dependency.ast_payload_extraction`
      because checker-owned proof skeleton payload extraction, local proof
      context, formula payload extraction, recorded facts, theorem acceptance,
      CoreIr, ControlFlowIr, VC, proof payloads, and the `formula_statement`
      runner are not available. The task must not fabricate proof skeleton
      payloads, formula payloads, local facts, theorem acceptance, or downstream
      semantic payloads.
    - Verify: `cargo test -p mizar-test`.
    - Deps: task 87. Refs: Step 5 source-derived semantic bridge; mizar-test
      task 10; spec 14 formulas; spec 15 statements; spec 16 theorems and
      proofs.

89. **Add source-derived statement proof extraction-gap boundary.** [x]
    - Add a dedicated active `type_elaboration` boundary for a theorem proof
      containing statement-level proof justifications, such as labeled
      `A: thesis proof ... end;` and final `thus thesis proof ... end;`
      proof blocks.
    - Acceptance: parser and resolver execute the source, then the active
      runner reports `type_elaboration.external_dependency.ast_payload_extraction`
      because checker-owned statement proof payload extraction, nested proof
      skeleton payloads, local proof context, formula payload extraction,
      label-reference semantic checking, recorded facts, theorem acceptance,
      CoreIr, ControlFlowIr, VC, proof payloads, and the `formula_statement`
      runner are not available. The task must not fabricate statement proof
      payloads, proof skeleton payloads, formula payloads, local facts, theorem
      acceptance, or downstream semantic payloads.
    - Verify: `cargo test -p mizar-test`.
    - Deps: task 88. Refs: Step 5 source-derived semantic bridge; mizar-test
      task 10; spec 14 formulas; spec 15 statements; spec 16 theorems and
      proofs.

90. **Add source-derived predicate/functor definition extraction-gap boundary.** [x]
    - Add a dedicated active `type_elaboration` boundary for a definition block
      containing a predicate definition and a functor definition.
    - Acceptance: parser and resolver execute the source, then the active
      runner reports `type_elaboration.external_dependency.ast_payload_extraction`
      because checker-owned predicate/functor definition declaration payload
      extraction, definition-local contexts, definiens formula/term payloads,
      overload payloads, recorded facts, CoreIr, ControlFlowIr, VC, proof
      payloads, and the `formula_statement` runner are not available. The task
      must not fabricate definition payloads, formula/term body payloads,
      overload payloads, facts, or downstream semantic payloads.
    - Verify: `cargo test -p mizar-test`.
    - Deps: task 89. Refs: Step 5 source-derived semantic bridge; mizar-test
      task 10; spec 09 predicate definitions; spec 10 functor definitions.

91. **Add source-derived attribute definition extraction-gap boundary.** [x]
    - Add a dedicated active `type_elaboration` boundary for a definition block
      containing an attribute definition.
    - Acceptance: parser and resolver execute the source, then the active
      runner reports `type_elaboration.external_dependency.ast_payload_extraction`
      because checker-owned attribute definition declaration payload extraction,
      definition-local contexts, formula-definiens payloads, attributed-type
      evidence, recorded facts, CoreIr, ControlFlowIr, VC, proof payloads, and
      the `formula_statement` runner are not available. The task must not
      fabricate definition payloads, formula body payloads, evidence, facts, or
      downstream semantic payloads.
    - Verify: `cargo test -p mizar-test`.
    - Deps: task 90. Refs: Step 5 source-derived semantic bridge; mizar-test
      task 10; spec 06 attribute definitions.

92. **Add source-derived mode/structure definition extraction-gap boundary.** [x]
    - Add a dedicated active `type_elaboration` boundary for a definition block
      containing a structure definition and a mode definition.
    - Acceptance: parser and resolver execute the source, then the active
      runner reports `type_elaboration.external_dependency.ast_payload_extraction`
      because checker-owned mode/structure definition declaration payload
      extraction, mode expansion, structure base-shape/constructor/selector
      evidence, definition-local contexts, recorded facts, CoreIr, ControlFlowIr,
      VC, proof payloads, and the `formula_statement` runner are not available.
      The task must not fabricate definition payloads, mode-expansion payloads,
      structure evidence, facts, or downstream semantic payloads.
    - Verify: `cargo test -p mizar-test`.
    - Deps: task 91. Refs: Step 5 source-derived semantic bridge; mizar-test
      task 10; spec 05 structures; spec 07 mode definitions.

93. **Add source-derived proof-local declaration extraction-gap boundary.** [x]
    - Add a dedicated active `type_elaboration` boundary for a theorem proof
      containing `let`, `given`, `consider`, `set`, and `reconsider`
      statements.
    - Acceptance: parser and resolver execute the source, then the active
      runner reports `type_elaboration.external_dependency.ast_payload_extraction`
      because checker-owned proof-local declaration payload extraction, local
      proof context, formula/term payloads, RHS term inference, reconsider
      coercion/obligation evidence, recorded facts, CoreIr, ControlFlowIr, VC,
      proof payloads, and the `formula_statement` runner are not available. The
      task must not fabricate proof-local declaration payloads, formula/term
      payloads, local facts, theorem acceptance, or downstream semantic
      payloads.
    - Verify: `cargo test -p mizar-test`.
    - Deps: task 92. Refs: Step 5 source-derived semantic bridge; mizar-test
      task 10; spec 15 statements; spec 16 theorems and proofs.

94. **Add source-derived proof-local inline definition extraction-gap boundary.** [x]
    - Add a dedicated active `type_elaboration` boundary for a theorem proof
      containing proof-local `deffunc` and `defpred` statements.
    - Acceptance: parser and resolver execute the source, then the active
      runner reports `type_elaboration.external_dependency.ast_payload_extraction`
      because checker-owned inline definition formal/body payload extraction,
      local abbreviation expansion, term/formula body payloads, guard evidence,
      recorded facts, theorem acceptance, CoreIr, ControlFlowIr, VC, proof
      payloads, and the `formula_statement` runner are not available. The task must not fabricate
      inline definition payloads, local abbreviation expansion, term/formula
      body payloads, facts, theorem acceptance, or downstream semantic payloads.
    - Verify: `cargo test -p mizar-test`.
    - Deps: task 93. Refs: Step 5 source-derived semantic bridge; mizar-test
      task 10; spec 15 statements.

95. **Add source-derived registration block extraction-gap boundary.** [x]
    - Add a dedicated active `type_elaboration` boundary for a top-level
      `registration` block containing existential and conditional clusters.
    - Acceptance: parser and resolver execute the source, then the active
      runner reports `type_elaboration.external_dependency.ast_payload_extraction`
      because checker-owned registration-item payload extraction,
      correctness-condition/proof-obligation payloads, accepted
      activation/evidence status, cluster/reduction semantics, recorded facts,
      CoreIr, ControlFlowIr, VC, proof payloads, and the `formula_statement` /
      `advanced_semantics` runners are not available. The task must not
      fabricate registration payloads, activation status, cluster/reduction
      facts, Chapter 17 semantic coverage, or downstream semantic payloads.
    - Verify: `cargo test -p mizar-test`.
    - Deps: task 94. Refs: Step 5 source-derived semantic bridge; mizar-test
      task 10; spec 17 clusters and registrations.

96. **Add source-derived redefinition/notation extraction-gap boundary.** [x]
    - Add a dedicated active `type_elaboration` boundary for top-level and
      definition-local synonym/antonym aliases plus attribute, predicate, and
      functor redefinition declarations.
    - Acceptance: parser and resolver execute the source, then the active
      runner reports `type_elaboration.external_dependency.ast_payload_extraction`
      because checker-owned redefinition payload extraction, notation alias
      relation payloads, target inference, coherence proof-obligation payloads,
      overload candidate payloads, recorded facts, CoreIr, ControlFlowIr, VC,
      proof payloads, and the `formula_statement` / `advanced_semantics`
      runners are not available. The task must not fabricate alias semantics,
      redefinition payloads, overload facts, Chapter 11 alias semantic
      resolution, Chapter 19 overload/redefinition semantic coverage, or
      downstream semantic payloads.
    - Verify: `cargo test -p mizar-test`.
    - Deps: task 95. Refs: Step 5 source-derived semantic bridge; mizar-test
      task 10; spec 11 symbol management; spec 19 overload resolution.

97. **Add source-derived imported TypeCaseStruct reserve provenance bridge.** [x]
    - Promote the task-78 imported-structure reserve boundary just far enough
      for the active `type_elaboration` runner to pass the documented
      `parser.type_fixtures` import-summary `TypeCaseStruct` structure symbol
      as a checker `TypeHeadInput`.
    - Acceptance: the checker reserve bridge validates that `TypeCaseStruct`
      is visible through `SymbolEnv`, has `SymbolKind::Structure`, and is
      backed by an `ImportedSource` contribution from `parser.type_fixtures`.
      The runner no longer reports
      `type_elaboration.external_dependency.ast_payload_extraction` for
      `TypeCaseStruct`; it reaches
      `type_elaboration.checker.checker.declaration.deferred.evidence_query`
      because imported module AST extraction and base-shape/constructor-witness
      evidence are not available. The task must not synthesize imported module
      AST extraction, base-shape/constructor-witness evidence, positive
      structure elaboration, CoreIr, ControlFlowIr, VC, or proof payloads, and
      it must leave other generic imported structures, imported attributes,
      arguments, brackets, qualified attributes, and imported evidence on their
      existing gaps.
    - Verify: `cargo test -p mizar-test`, `cargo test -p mizar-checker`.
    - Deps: tasks 48, 52, 76, 78, and 83. Refs: Step 5 source-derived
      semantic bridge; mizar-test task 10; spec 03 type expressions; spec 05
      structures; spec 11 symbol management; spec 12 modules and namespaces.

98. **Add source-derived imported predicate/functor term-formula extraction-gap boundary.** [x]
    - Historical boundary: add a dedicated active `type_elaboration` boundary
      for a theorem formula that imports `parser.type_fixtures` and uses
      documented imported predicate/functor surfaces such as `divides` and
      `++`.
    - Task 110 supersedes the exact
      `ImportedPredicateFunctorPayloadBoundary: 1 divides (1 ++ 2)` source by
      passing real checker numeral, imported functor-application, and
      predicate-application payloads before failing on missing
      numeric/signature payloads and partial formula checking. Task 98 remains
      historical for the parser/resolver-executable extraction-gap boundary
      and must not be read as imported module AST extraction, semantic
      predicate/functor signatures, term inference, formula checking, recorded
      facts, theorem acceptance, CoreIr, ControlFlowIr, VC, proof payloads, or
      `formula_statement` runner support.
    - Verify: `cargo test -p mizar-test`.
    - Deps: tasks 86 and 87. Refs: Step 5 source-derived semantic bridge;
      mizar-test task 10; spec 11 symbol management; spec 12 modules and
      namespaces; spec 13 term expressions; spec 14 formulas; spec 16 theorems
      and proofs.

99. **Add source-derived formula connective/quantifier extraction-gap boundary.** [x]
    - Add a dedicated active `type_elaboration` boundary for a theorem formula
      using Chapter 14 connective and quantifier surfaces, such as implication,
      universal quantification, and negation.
    - Acceptance: parser and resolver execute the source, then the active
      runner reports `type_elaboration.external_dependency.ast_payload_extraction`
      because checker-owned formula payload extraction, quantifier
      binder/context payloads, formula checking, recorded facts, theorem
      acceptance, CoreIr, ControlFlowIr, VC, proof payloads, and the
      `formula_statement` runner are not available. The task must not fabricate
      formula payloads, quantifier binder/context payloads, facts, theorem
      acceptance, or downstream semantic payloads.
    - Verify: `cargo test -p mizar-test`.
    - Deps: tasks 86, 87, and 98. Refs: Step 5 source-derived semantic bridge;
      mizar-test task 10; spec 14 formulas; spec 16 theorems and proofs.

112. **Add exact source-derived formula connective/quantifier shell checker bridge.** [x]
    - Supersede task 99 only for the exact unrecovered
      `FormulaConnectiveQuantifierPayloadBoundary: contradiction implies for x
      being set holds not contradiction` theorem source.
    - Acceptance: parser and resolver execute the source, the runner extracts
      real source sites/ranges for implication, universal quantification, and
      negation shells, passes those checker `FormulaInput`s to
      `TermFormulaChecker`, and fails closed on missing formula/quantifier
      payloads. Task 117 later supersedes only the two exact contradiction
      constants in this source as real formula constant kind payloads. The
      bridge must not fabricate child-formula links, binder/context payloads,
      formula facts/checking, theorem acceptance, `formula_statement`, CoreIr,
      ControlFlowIr, VC, or proof payloads.
    - Verify: `cargo test -p mizar-test`.
    - Deps: tasks 86, 99, 106, 110, and 111. Refs: Step 5 source-derived
      semantic bridge; mizar-test task 10; spec 14 formulas; spec 16 theorems
      and proofs.

100. **Add source-derived builtin membership formula extraction-gap boundary.** [x]
    - Add a dedicated active `type_elaboration` boundary for a theorem formula
      using the Chapter 14 builtin membership predicate with Chapter 13 numeral
      term operands.
    - Acceptance: parser and resolver execute the source. Task 108 supersedes
      this exact sidecar by passing real checker term/formula payloads and
      reporting missing numeric type payload plus partial formula checking. The
      task must not fabricate numeric type payloads, membership operand
      expected-type construction/checking, facts, theorem acceptance,
      `formula_statement`, CoreIr, ControlFlowIr, VC, proof payloads, or
      downstream semantic payloads.
    - Verify: `cargo test -p mizar-test`.
    - Deps: tasks 86, 87, and 98. Refs: Step 5 source-derived semantic bridge;
      mizar-test task 10; spec 13 term expressions; spec 14 formulas; spec 16
      theorems and proofs.

101. **Add source-derived builtin inequality formula extraction-gap boundary.** [x]
    - Add a dedicated active `type_elaboration` boundary for a theorem formula
      using the Chapter 14 builtin inequality predicate with Chapter 13 numeral
      term operands.
    - Acceptance: parser and resolver execute the source. Task 107 supersedes
      this exact sidecar by passing real checker term/formula payloads and
      reporting missing numeric type payload plus partial formula checking. The
      task must not fabricate numeric type payloads, inequality
      desugaring/equality semantic checking, facts, theorem acceptance,
      `formula_statement`, CoreIr, ControlFlowIr, VC, proof payloads, or
      downstream semantic payloads.
    - Verify: `cargo test -p mizar-test`.
    - Deps: tasks 86, 87, 98, and 100. Refs: Step 5 source-derived semantic
      bridge; mizar-test task 10; spec 13 term expressions; spec 14 formulas;
      spec 16 theorems and proofs.

102. **Add source-derived builtin type assertion formula extraction-gap boundary.** [x]
    - Add a dedicated active `type_elaboration` boundary for a theorem formula
      using the Chapter 14 builtin type-assertion form with a Chapter 13
      numeral term.
    - Task 109 supersedes only the exact builtin `set` theorem source with
      source-derived checker `TermInput`, `FormulaInput`, and asserted
      `TypeExpressionInput` payloads before failing closed on missing numeric
      type payloads and partial formula checking. Broader asserted type payload
      extraction, type-assertion semantic checking, recorded facts, theorem
      acceptance, CoreIr, ControlFlowIr, VC, proof payloads, and the
      `formula_statement` runner remain unavailable. The tasks must not
      fabricate type-assertion facts, theorem acceptance, or downstream
      semantic payloads.
    - Verify: `cargo test -p mizar-test`.
    - Deps: tasks 86, 87, 98, 100, and 101. Refs: Step 5 source-derived
      semantic bridge; mizar-test task 10; spec 13 term expressions; spec 14
      formulas; spec 16 theorems and proofs.

103. **Add source-derived imported attribute assertion formula extraction-gap boundary.** [x]
    - Add a dedicated active `type_elaboration` boundary for a theorem formula
      importing `parser.type_fixtures` and using its documented `empty`
      attribute in the Chapter 14 attribute-assertion form with a Chapter 13
      numeral subject.
    - Acceptance: parser and resolver execute the source, then the active
      runner reports `type_elaboration.external_dependency.ast_payload_extraction`
      because checker-owned term/formula payload extraction, imported attribute
      assertion attribute-chain/provenance payload extraction, term inference,
      attribute admissibility/semantic checking, formula checking, recorded
      facts, theorem acceptance, CoreIr, ControlFlowIr, VC, proof payloads, and
      the `formula_statement` runner are not available. The task must not
      fabricate term/formula payloads, imported attribute assertion payloads,
      imported module AST extraction, theorem acceptance, or downstream semantic
      payloads.
    - Verify: `cargo test -p mizar-test`.
    - Deps: tasks 86, 87, 98, 100, 101, and 102. Refs: Step 5 source-derived
      semantic bridge; mizar-test task 10; spec 06 attributes; spec 11 symbol
      management; spec 12 modules and namespaces; spec 13 term expressions;
      spec 14 formulas; spec 16 theorems and proofs.

104. **Add source-derived attribute-level non-empty imported attribute assertion formula extraction-gap boundary.** [x]
    - Add a dedicated active `type_elaboration` boundary for a theorem formula
      importing `parser.type_fixtures` and using its documented `empty`
      attribute as an attribute-level `non empty` assertion in the Chapter 14
      attribute-assertion form with a Chapter 13 numeral subject.
    - Acceptance: parser and resolver execute the source, then the active
      runner reports `type_elaboration.external_dependency.ast_payload_extraction`
      because checker-owned term/formula payload extraction, imported
      attribute-level non-empty assertion attribute-chain/provenance payload
      extraction, term inference, negated attribute admissibility/semantic
      checking, formula checking, recorded facts, theorem acceptance, CoreIr,
      ControlFlowIr, VC, proof payloads, and the `formula_statement` runner are
      not available. The task must not fabricate term/formula payloads, imported
      attribute-level non-empty assertion payloads, imported module AST
      extraction, theorem acceptance, or downstream semantic payloads.
      Task 114 supersedes only the exact
      `ImportedNonEmptyAttributeAssertionPayloadBoundary: 1 is non empty`
      source with a real checker term/formula handoff.
    - Verify: `cargo test -p mizar-test`.
    - Deps: tasks 86, 87, 98, 100, 101, 102, and 103. Refs: Step 5
      source-derived semantic bridge; mizar-test task 10; spec 06 attributes;
      spec 11 symbol management; spec 12 modules and namespaces; spec 13 term
      expressions; spec 14 formulas; spec 16 theorems and proofs.

114. **Add exact source-derived attribute-level non-empty imported attribute assertion theorem checker bridge.** [x]
    - Supersede task 104 only for the exact active source
      `import parser.type_fixtures; theorem ImportedNonEmptyAttributeAssertionPayloadBoundary: 1 is non empty;`.
    - Acceptance: parser and resolver execute the source; the active runner
      validates the direct `non` surface and imported `empty` provenance,
      extracts one source-derived numeral `TermInput` and one
      attribute-assertion `FormulaInput`, and `TermFormulaChecker` reports
      missing numeric type payload, missing formula/attribute semantic payload,
      and partial formula checking. The task must not fabricate imported module
      AST extraction, negated attribute-chain semantic payloads,
      theorem-formula `AttributeInput` payloads, negated attribute
      admissibility/semantic checking, formula checking, theorem acceptance,
      `formula_statement`, CoreIr, ControlFlowIr, VC, or proof payloads.
    - Verify: `cargo test -p mizar-test`.
    - Deps: tasks 86, 87, 98, 100, 101, 102, 103, and 104. Refs: Step 5
      source-derived semantic bridge; mizar-test task 10; spec 06 attributes;
      spec 11 symbol management; spec 12 modules and namespaces; spec 13 term
      expressions; spec 14 formulas; spec 16 theorems and proofs.

105. **Add source-derived set-enumeration formula extraction-gap boundary.** [x]
    - Add a dedicated active `type_elaboration` boundary for a theorem formula
      using Chapter 13 set-enumeration term operands with Chapter 14 builtin
      equality.
    - Historical acceptance: parser and resolver execute the source, then the active
      runner reports `type_elaboration.external_dependency.ast_payload_extraction`
      because checker-owned set-enumeration term payload extraction,
      term/formula payload extraction, term inference, equality/formula
      checking, recorded facts, theorem acceptance, CoreIr, ControlFlowIr, VC,
      proof payloads, and the `formula_statement` runner are not available.
      The task must not fabricate set-enumeration payloads, term/formula
      payloads, theorem acceptance, or downstream semantic payloads.
      Task 111 supersedes only the exact `{1, 2} = {1, 2}` source with a real
      checker term/formula handoff.
    - Verify: `cargo test -p mizar-test`.
    - Deps: tasks 86, 87, 98, 100, 101, 102, 103, and 104. Refs: Step 5
      source-derived semantic bridge; mizar-test task 10; spec 13 term
      expressions; spec 14 formulas; spec 16 theorems and proofs.

111. **Add exact source-derived set-enumeration theorem checker bridge.** [x]
    - Supersede task 105 only for the exact active source
      `theorem SetEnumerationPayloadBoundary: {1, 2} = {1, 2};`.
    - Acceptance: parser and resolver execute the source; the active runner
      extracts the four source-derived numeral item terms, two
      set-enumeration `TermInput`s, and one builtin equality `FormulaInput`
      from the AST; `TermFormulaChecker` then reports missing numeric type
      payloads, missing set-enumeration result-type/sethood payloads, and
      partial formula checking. The task must not fabricate sethood/result
      types, equality facts/checking, theorem acceptance, `formula_statement`,
      CoreIr, ControlFlowIr, VC, or proof payloads.
    - Verify: `cargo test -p mizar-test`; final workspace verification.
    - Deps: tasks 105, 106, 107, 108, 109, and 110. Refs: Step 5
      source-derived semantic bridge; mizar-test task 10; spec 13 term
      expressions; spec 14 formulas; spec 16 theorems and proofs.

106. **Add source-derived builtin equality theorem term/formula checker bridge.** [x]
    - Promote only the unrecovered
      `TheoremItem -> FormulaExpression -> BuiltinPredicateApplication("=")`
      source shape with exactly two structural Chapter 13 `NumeralTerm`
      operands.
    - Acceptance: the active runner builds a real module-shell checker binding
      context, passes two source-derived `TermInput`s and one equality
      `FormulaInput` to `TermFormulaChecker`, and fails closed on
      `type_elaboration.checker.checker.term.external.numeric_type_payload` plus
      `type_elaboration.checker.checker.formula.term.partial`. The task must not
      fabricate numeric type payloads, equality facts/checking, theorem
      acceptance, `formula_statement` runner support, or downstream semantic
      payloads, and it must not promote membership, inequality, type assertion,
      imported, set-enumeration, connective/quantifier, or proof theorem
      surfaces.
    - Verify: `cargo test -p mizar-test --test metadata`.
    - Deps: tasks 86 and 87. Refs: Step 5 source-derived semantic bridge;
      mizar-test task 10; spec 13 term expressions; spec 14 formulas; spec 16
      theorems and proofs.

108. **Add source-derived builtin membership theorem term/formula checker bridge.** [x]
    - Promote only the unrecovered
      `TheoremItem -> FormulaExpression -> BuiltinPredicateApplication("in")`
      source shape with label `BuiltinMembershipPayloadBoundary` and exactly two
      structural Chapter 13 `NumeralTerm` operands spelling `1` and `1`.
    - Acceptance: the active runner builds a real module-shell checker binding
      context, passes two source-derived `TermInput`s and one membership
      `FormulaInput` to `TermFormulaChecker`, and fails closed on
      `type_elaboration.checker.checker.term.external.numeric_type_payload` plus
      `type_elaboration.checker.checker.formula.term.partial`. The task must not
      fabricate numeric type payloads, membership operand expected types,
      membership facts, theorem acceptance, `formula_statement` runner support,
      or downstream semantic payloads, and it must not promote equality,
      inequality, type assertion, imported, set-enumeration, connective/
      quantifier, or proof theorem surfaces.
    - Verify: `cargo test -p mizar-test --test metadata`.
    - Deps: tasks 86, 87, 98, and 100. Refs: Step 5 source-derived semantic
      bridge; mizar-test task 10; spec 13 term expressions; spec 14 formulas;
      spec 16 theorems and proofs.

110. **Add source-derived imported predicate/functor theorem checker bridge.** [x]
    - Promote only the exact source importing `parser.type_fixtures` and using
      `theorem ImportedPredicateFunctorPayloadBoundary: 1 divides (1 ++ 2);`.
    - Acceptance: the active runner validates imported `divides` and `++`
      resolver provenance, passes source-derived numeral terms, the imported
      functor-application term, and the predicate-application formula to
      `TermFormulaChecker`, and fails closed on missing numeric/signature
      payloads plus partial formula checking. The task must not fabricate
      imported module AST extraction, semantic predicate/functor signatures,
      term inference, formula checking, facts, theorem acceptance,
      `formula_statement`, or downstream semantic payloads.
    - Verify: `cargo test -p mizar-test --test metadata`.
    - Deps: tasks 86, 87, and 98. Refs: Step 5 source-derived semantic bridge;
      mizar-test task 10; spec 11 symbol management; spec 12 modules and
      namespaces; spec 13 term expressions; spec 14 formulas; spec 16 theorems
      and proofs.

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
