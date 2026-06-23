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
      expansion, viability over recorded facts, specificity partial order
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
