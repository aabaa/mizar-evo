# mizar-core TODO

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
architecture 06 and 16.

| Module | Spec | Source | Status |
|---|---|---|---|
| core_ir | `core_ir.md` (task 2) | `src/core_ir.rs` | [x] |
| binder_normalization | `binder_normalization.md` (task 4) | `src/binder_normalization.rs` | [x] |
| elaborator | `elaborator.md` (task 7) | `src/elaborator.rs` | [x] |
| control_flow | `control_flow.md` (task 14) | `src/control_flow.rs` | [x] |

`mizar-core` implements pipeline phase 9 (elaboration) and phase 10
(control-flow preparation): `ResolvedTypedAst` in, `CoreIr` and
`ControlFlowIr` out. Elaboration is the last source-shaped boundary — core
representations are normalized for proof, verification, and kernel checking,
with soft types erased only through explicit type predicates. The binder
library it owns (architecture 16) is also the representation later replayed by
`mizar-kernel`'s substitution checker, so its invariants are
soundness-relevant.

Dependency order: `core_ir` data → `binder_normalization` → `elaborator`
(steps 1-6) → `control_flow`. The data and binder tasks (2-6) do not need
checker output and can proceed in parallel with `mizar-checker` waves.

Each task below is deliberately small — one module spec, or one behavior slice
of one module — so that a single task can be implemented, tested, and
committed autonomously without holding the rest of the crate in flight.

## Crate Prerequisites

The crate depends on `mizar-session`, `mizar-resolve` (symbol identities), and
`mizar-checker` (`ResolvedTypedAst`). Elaboration tasks (8-13) are gated on
`mizar-checker` task 28; the data and binder foundations are not.
Architecture: [06.elaboration_and_core_ir.md](../../architecture/en/06.elaboration_and_core_ir.md),
[16.substitution_and_binding.md](../../architecture/en/16.substitution_and_binding.md);
crate ownership: [internal 07](../../internal/en/07.crate_module_layout.md).

## Resolved And Open Decisions

- **Binder representation: resolved by task 4.** `binder_normalization.md`
  chooses a locally nameless representation with de Bruijn indices for bound
  variables and stable `CoreVarId`s for free, schematic, and generated
  variables. The kernel re-checks substitutions independently; the chosen
  representation preserves linear replay with explicit freshness witnesses and
  guard side conditions.
- **ControlFlowIr construction ownership: resolved by internal 07.**
  `mizar-core` owns control-flow preparation (phase 10) including
  `ControlFlowIr` construction; `mizar-vc` consumes `ControlFlowIr` for
  algorithm VC generation. Architecture 07's module list predates this split;
  `control_flow.md` (task 14) records the boundary.
- **Erasure policy: resolved by architecture 06.** Soft type annotations are
  erased only through explicit type predicates and assumptions; `elaborator.md`
  enumerates the erasure rules case by case, and elaboration never runs proof
  search or activates registrations.

## Ordered Task List

Keep `cargo test -p mizar-core` green after each task (see
[Recommended Verification](#recommended-verification)).

### Core IR and binder foundation

1. **Crate scaffold and lint-policy guard.** [x]
   - Add the `mizar-core` workspace member depending on `mizar-session`,
     `mizar-resolve`, and `mizar-checker`; add `tests/lint_policy.rs`
     mirroring the `mizar-frontend` guard.
   - Tests: lint-policy guard passes; workspace builds.
   - Deps: `mizar-checker` task 1. Spec: architecture 06.

2. **Spec: `core_ir.md`.** [x]
   - Write the `CoreIr` data-shape spec (English and Japanese, no code):
     `CoreItem`, core terms/formulas, `CoreDefinitionTable` with stable
     expansion boundaries, `CoreProofTable`, `CoreSourceMap` with
     `GeneratedFrom` markers, and the obligation-seed reference shape,
     including anchor-ready local proof/program paths, labels, normalized
     semantic origins, and source/core provenance consumed by `mizar-vc`.
   - Deps: 1. Spec: architecture 06 "Interface Definitions",
     [01.ir_layers.md](../../architecture/en/01.ir_layers.md).

3. **Implement `core_ir` data shapes.** [x]
   - Implement the core item/term/formula/proof structures and the source
     map per task 2, plus a deterministic debug rendering.
   - Tests: construction round-trips; every node reachable from items maps
     back to source or carries `GeneratedFrom`; rendering stability.
   - Deps: 2. Spec: `core_ir.md`.

4. **Spec: `binder_normalization.md`.** [x]
   - Write the binder spec (English and Japanese, no code): representation
     decision (with rationale and kernel-replay implications),
     alpha-equivalence, capture-avoiding substitution API, free-variable
     conditions, and normalization rules.
   - Deps: 3. Spec:
     [16.substitution_and_binding.md](../../architecture/en/16.substitution_and_binding.md),
     `core_ir.md`.

5. **Binder representation and substitution.** [x]
   - Implement the chosen representation and capture-avoiding substitution
     over core terms/formulas.
   - Tests: substitution fixtures including shadowing and capture cases;
     substitution composition laws. Include the review-audit
     `defpred P(n be Nat) means n < m` shadowing case and malformed
     capture-producing substitution regressions before marking substitution
     coverage complete.
   - Deps: 3, 4. Spec: `binder_normalization.md`.

6. **Alpha-equivalence and normalization utilities.** [x]
   - Implement alpha-equivalence checking and binder normalization with
     deterministic canonical forms.
   - Tests: property tests (equivalence is reflexive/symmetric/transitive;
     normalization idempotent; equal canonical forms iff alpha-equivalent).
   - Deps: 5. Spec: `binder_normalization.md`.

### Elaboration (phase 9)

7. **Spec: `elaborator.md`.** [x]
   - Write the elaboration spec (English and Japanese, no code) with named
     sections the implementation tasks cite: core context preparation,
     type/fact lowering and the case-by-case erasure rules, term/formula
     lowering, definition lowering with expansion boundaries, proof-skeleton
     lowering, and algorithm-shell lowering.
   - Deps: 2, 4. Spec: architecture 06 "Step 1"-"Step 6".

8. **Core context preparation.** [x]
   - Implement Step 1: canonical symbol identities, definition boundary
     registry, and the elaboration context over `ResolvedTypedAst`.
   - Tests: context fixtures; canonical ids never raw spellings.
   - Deps: 3, 7, `mizar-checker` task 28. Spec: `elaborator.md` (context
     section).

9. **Type and fact lowering.** [x]
   - Implement Step 2: lower soft types and type facts into explicit type
     predicates and assumptions per the erasure rules.
   - Tests: each erasure rule has a fixture; no silent erasure (every
     dropped annotation is justified by a rule).
   - Deps: 8. Spec: `elaborator.md` (erasure section).

10. **Term and formula lowering.** [x]
    - Implement Step 3: lower resolved terms and formulas, including
      inserted `qua` views, into binder-normalized core forms.
    - Tests: lowering fixtures per surface form; failed semantic sites stay
      explicit error nodes, never valid core terms. Include stable-choice and
      comprehension review cases: stable `the T` lowers to generated
      `Apply(choice_T(...))` symbols, and Fraenkel comprehensions retain
      required sethood evidence.
    - Deps: 9. Spec: `elaborator.md` (terms/formulas section).

11. **Definition lowering.** [x]
    - Implement Step 4: lower definitions with stable expansion boundaries
      (no eager inlining), including correctness-condition bodies.
    - Tests: expansion boundary fixtures; definition unfolding is explicit,
      never accidental. Record exported definition choices as generated
      dependencies so the later explicit unfolding surface reuses
      definition-owned symbols rather than regenerating them.
    - Deps: 10. Spec: `elaborator.md` (definitions section).

12. **Proof-skeleton lowering.** [x]
    - Implement Step 5: lower proof structures (`proof`/`now`/`per cases`,
      conclusion steps, citations) into core proof trees with thesis
      tracking.
    - Tests: skeleton fixtures per proof form; thesis transitions recorded;
      citation references preserved symbolically; invalid citations, missing
      or wrong-owner proof items, malformed error roots, active path formulas,
      terminal-goal back-references, and external dependency citations covered.
      Include theorem/lemma propositions that own their stable choice symbols.
    - Deps: 11. Spec: `elaborator.md` (proofs section).

13. **Algorithm-shell lowering.** [x]
    - Implement Step 6: lower algorithm bodies to core items (no CFG yet),
      preserving contracts and ghost annotations for phase 10.
    - Tests: shell fixtures; ghost/runtime distinction preserved. Include
      executable algorithm statement `the` sites lowering to `Pick` bindings
      and ghost-only `Pick` sites staying marked for later erasure.
    - Deps: 12. `mizar-parser` tasks 32-34 coverage remains an external
      source-to-checker extraction gap for this task. Spec: `elaborator.md`
      (algorithms section).

### Control-flow preparation (phase 10)

14. **Spec: `control_flow.md`.** [x]
    - Write the `ControlFlowIr` spec (English and Japanese, no code): basic
      blocks, local binding tables, contract sets, ghost-effect tables,
      termination measures, and the core→CFG construction contract;
      record the internal-07 ownership boundary with `mizar-vc`.
    - Deps: 2, 13. Spec: architecture 06 "Step 6", architecture 07 "Step 1",
      [20.algorithm_and_verification.md](../../../spec/en/20.algorithm_and_verification.md).

15. **`ControlFlowIr` construction.** [x]
    - Build control-flow graphs from core algorithm items: blocks, edges,
      local binding information, minimal contexts, statement placement/source
      maps, and structural diagnostics required to form valid flow.
    - Tests: straight-line flow; CFG fixtures per control construct (`while`,
      `if`, `match`, `break`/`continue`); deterministic block ordering and
      debug rendering; local/source-map fidelity; fallthrough, break, loop
      carry, and unreachable-join context regressions.
    - Deps: 13, 14. Spec: `control_flow.md`.

16. **Contracts, ghost effects, and termination measures.** [x]
    - Attach preconditions, postconditions, assertions, invariants,
      ghost-effect tracking, and termination measures to the CFG.
    - Tests: attachment fixtures; ghost state never leaks into runtime
      effect tables.
    - Deps: 15. Spec: `control_flow.md`.

17. **Flow diagnostics.** [x]
    - Implement use-before-assignment and unreachable-code diagnostics over
      the CFG.
    - Tests: pass/fail fixtures per diagnostic; stable diagnostic order.
    - Deps: 15. Spec: `control_flow.md`,
      [22.error_handling_and_diagnostics.md](../../../spec/en/22.error_handling_and_diagnostics.md).

### Hardening and cross-cutting follow-ups

18. **Obligation-seed handoff contract.** [x]
    - Define and implement the obligation-seed output consumed by `mizar-vc`
      (seeds only; concrete `VcId`s are assigned by phase 11), covering
      existing theorem bodies, correctness conditions, checker-initial
      obligations, generated/deferred/error traceability rows, and
      flow-derived algorithm contracts, termination, and ghost-erasure sites.
      Seeds must carry anchor-ready local proof/program paths, labels,
      normalized semantic origins, source/core provenance, and local CFG site
      metadata, while leaving cross-edit reuse identity to `mizar-vc`.
    - Tests: seed coverage fixtures; seeds reference `CoreIr`/`ControlFlowIr`
      nodes, source ranges, local proof/program paths, labels, and provenance.
    - Deps: 12, 16, coordinated with `mizar-vc` tasks 2 and 4. Spec:
      `core_ir.md` (seed section), architecture 06 constraints.

19. **Snapshot dumps and corpus contributions.** [x]
    - Record deferred corpus snapshot baselines for deterministic
      `CoreIr`/`ControlFlowIr` renderings at stages `type_elaboration` and
      `proof_verification` until `mizar-test` exposes those snapshot runners and
      source-derived payload seams.
    - Deps: 12, 15. Spec: [staged_model.md](../../mizar-test/en/staged_model.md),
      [snapshot.md](../../mizar-test/en/snapshot.md).

20. **Determinism suite.** [x]
    - Property-style coverage that identical public-API core fixtures produce
      identical core items, binder numbering, CFGs, obligation-seed handoff, and
      renderings. Full source-derived `ResolvedTypedAst` determinism remains
      deferred until source-to-checker extraction exists.
    - Deps: 18. Spec: [20.test_strategy.md](../../architecture/en/20.test_strategy.md).

21. **Public-enum forward-compatibility policy.** [x]
    - Apply the `mizar-frontend` task-25 procedure to each public enum;
      record decisions in the owning module specs. Current result: every
      public `mizar-core` enum is downstream forward-compatible and must remain
      `#[non_exhaustive]`; no exhaustive exceptions are owned by this crate.
    - Deps: 18. Spec: all module specs.

22. **Source/spec correspondence audit.** [x]
    - Trace every public API and promised behavior in the module specs to
      implementation and tests; record gaps as follow-up tasks. Current result:
      `source_spec_audit.md` records item-level public API groups, no remaining
      `source_undocumented_behavior` for the current public surface, and
      CORE-AUDIT follow-up records for external/deferred seams.
    - Tests: lint-policy audit guard for EN/JA module sections, public-item
      mentions, gap id/class synchronization, and non-empty follow-up details.
    - Deps: 21. Spec: all module specs and this TODO.

23. **Bilingual documentation sync audit.** [x]
    - Compare each English canonical document under
      `doc/design/mizar-core/en/` with its Japanese companion and
      synchronize content. Current result: `bilingual_sync_audit.md` records
      the current paired file set, allowed language-specific differences, and
      no blocking bilingual documentation drift.
    - Tests: docs-only diff checks.
    - Deps: 22. Spec: repository documentation policy.

24. **Module-boundary refactor gate.** [x]
    - Before treating the crate as ready for downstream consumers, audit the
      source layout for oversized files, mixed responsibilities, and private
      helpers that should be split along the module table and spec boundaries.
      Split any review-bottleneck implementation files into private modules
      without changing public APIs, diagnostics, deterministic renderings,
      artifact-facing schemas, or consumer-visible behavior.
      Current result: `module_boundary_audit.md` records large but cohesive
      module-owned source files and no current review-bottleneck requiring a
      source split before closeout.
    - After any split, update this module table/source paths as needed and
      re-run the source/spec and bilingual documentation audit scopes for the
      moved APIs. Do not mix behavior cleanup or API exposure into the move;
      those require their own spec tasks.
    - Deps: 23. Spec: this TODO,
      [internal 07](../../internal/en/07.crate_module_layout.md), all module
      specs.
    - Tests: docs-only diff checks because no Rust source is moved.

25. **Closeout report and quality review.** [x]
    - Add the English/Japanese `crate_exit_report.md` pair, backfill task
      commit hashes in the ledgers, resolve the closeout bilingual audit row,
      run broad verification, and record the final quality review score.
    - Tests: `cargo fmt --check`,
      `cargo clippy --all-targets --all-features -- -D warnings`,
      `cargo test`, `git diff --check`, and staged `git diff --cached --check`.
    - Deps: 24. Spec:
      [autonomous_crate_development.md](../../autonomous_crate_development.md),
      this TODO, and the crate exit criteria.

### Template-encoding audit follow-ups (2026-07-05)

[template_encoding_audit.md](./template_encoding_audit.md) audited the spec
18.10 FOL encoding of templates and recorded findings F1-F8 plus a 4-seed
reject-first encoding corpus under `tests/miz/fail/templates/`
(`spec.en.18.templates.encoding_soundness.semantic`). The spec text for
F1-F6 and F8 was patched in the same change (`cef7e109`: spec 03, 05, 13,
17, 18), and task 26 records the remaining F7 spec decision; what remains here
after task 26 is the elaborator implementation and payload-bearing execution
work. Every finding maps to a task or a recorded disposition:

| Finding | Disposition |
|---|---|
| F1 (structure-view collapse) | spec patched; task 27 implements explicit-payload elaborator reduct-view lowering; kernel-side re-audit is [mizar-kernel task 35](../../mizar-kernel/en/todo.md); member-identity coordination is [mizar-checker task 36](../../mizar-checker/en/todo.md); source-derived runner/extraction remains external |
| F2 (type-actual inhabitation) | spec patched (§17.3.4 gating row); checker task 43 completed the built-in/base-shape inhabitation table; elaborator gating is task 28 |
| F3 (`type extends M` object/schema conflation) | spec patched (§18.10.2); explicit-payload bounded-view lowering is covered together with F1 in task 27 |
| F4 (functor guards, actual signature compatibility) | spec patched (§18.10.4, §18.9); explicit-payload implementation completed in task 29 |
| F5 (type-parameter sethood) | spec patched (§18.10.2 sethood paragraph); explicit-payload plumbing completed in task 30; source-derived extraction remains external |
| F6 (schemes applied inside template bodies) | spec patched (§18.10.3 paragraph); explicit substitution-composition metadata implementation completed in task 29 |
| F7 (inference determinism over widening) | spec patched by task 26; implementation remains deferred to payload-bearing inference/elaboration work |
| F8 (partial-algorithm functor actuals) | spec patched (§18.8.4); explicit diagnostic-only rejection completed in task 29 |
| corpus seeds (6) | inactive `advanced_semantics` seeds: the original 4 encoding seeds plus the task 26 F7 inference-determinism seeds. Activated with the runner via [mizar-checker task 48](../../mizar-checker/en/todo.md) and the mizar-test runner work |

26. **Spec decision: template argument inference determinism (F7).** [x]
    - Decide the §18.2.7 inference algorithm over the widening lattice. The
      audit's recommendation: infer omitted `[T]` at the arguments' declared
      types after mode unfolding; residual multiple candidates are an
      ambiguity error even when the instances would be logically equivalent.
      Determinism only — any well-formed instance is sound post-F1-F5.
      Update spec 18 §18.2.7 (English and Japanese) and coordinate with the
      overload tie-break decision
      ([mizar-checker task 37](../../mizar-checker/en/todo.md)) so template
      inference and overload selection use one comparison story. Checker task
      37 records the Phase B overload tie-break decision; this task records
      the Phase A omitted-template-argument inference rule and must not infer
      missing source payloads.
    - Acceptance: §18.2.7 names the comparison type and the ambiguity
      diagnostic; an ambiguity `.miz` seed with sidecar and
      `spec_trace.toml` entry pins the residual-candidates case.
    - Verify: `cargo test -p mizar-test`.
    - Deps: mizar-checker task 37 records the Phase B tie-break decision; this
      task records the Phase A inference-determinism decision. Refs:
      template_encoding_audit.md F7.
    - Completed by task 26: spec 18 §18.2.7 now infers omitted func/pred
      template type parameters from mode-unfolded declared argument types only.
      It does not search widening ancestors, apply cluster expansion, or infer
      `qua` views; residual distinct declared-type candidates are an ambiguous
      template instantiation even when their closures are equivalent. Added
      inactive seeds `fail_template_inference_declared_type_ambiguity_001` and
      `fail_template_inference_requires_explicit_qua_view_001`, plus trace row
      `spec.en.18.templates.inference_determinism.semantic`. This closes the
      F7 spec decision only; no checker/core source semantics or payload bridge
      behavior changed.

27. **Reduct/view lowering (F1, F3).** [x]
    - Implement the reduct-view encoding in elaboration: emit `view_{D→B}`
      terms for `qua` on renamed or multi-path inherit edges and for
      bounded-type-parameter instantiation; emit attribute atoms and field
      selections against view terms, not the flattened instance; preserve
      explicit exact-instance guard formulas on reduct terms (§5.8.5) while
      leaving source-derived extensionality emission to the checker/runner
      bridge. Cover the
      §18.10.2 object-level story for `type extends M` (view-typed schema
      parameter, `T.binop` lowering). This touches the type/fact and
      term/formula lowering surfaces (tasks 9-10) and the recently landed
      builtin type bridge / typed-AST elaboration seams.
    - Acceptance: the diamond example (Ring → AddGroup/MulMonoid → Magma
      with renaming) lowers without deriving `add(R) = mul(R)`; the
      `fail_template_qua_view_attribute_leak_001` seed's rejection is
      derivable from the lowered forms (attribute evidence on one view does
      not discharge a bound on another); Rust fixtures cover renamed-edge,
      multi-path, and explicit exact-instance guard preservation.
    - Verify: `cargo test -p mizar-core`,
      `cargo clippy -p mizar-core --all-targets -- -D warnings`;
      `cargo test -p mizar-checker` for the shared boundary.
    - Deps: 10, 11; mizar-checker task 36 (member-identity decision) for
      the identity rule; notify mizar-kernel task 35 when landing. Refs:
      spec 05 §5.8.3/§5.8.5, 13 §13.8.7, 18 §18.10.2;
      template_encoding_audit.md F1, F3.
    - Completed by task 27: `ReductViewSeed` / `ReductView` carry the
      checker-owned `QuaPathKey` and ordered explicit reduct functors.
      `CoreTermSeedKind::Qua` now reuses the base term only for no-reduct
      identity/cluster views and lowers explicit reduct payloads to ordered
      `Apply` view terms. Rust fixtures cover renamed diamond views,
      composed/multi-path views, template-bound facts/field selections on the
      final view term, exact-instance guard preservation on reduct terms, and
      empty reduct payload rejection in both type/fact and term/formula
      lowering. No `doc/spec`, existing `.miz`, expectation, source-derived
      runner, or fake checker payload was changed.

28. **Template type-actual inhabitation gating (F2).** [x]
    - Consume checker-owned §17.3.4 inhabitation-evidence gate results for
      template `type_expression` actuals: a schema context may assume
      `∃x. is_T(x)` for each type parameter, and in exchange every
      instantiation site must have a checker result satisfying the
      built-in/base-shape table; attributed actuals require an existential
      registration. Emit the per-parameter inhabitation fact into the schema
      context during lowering, without re-running checker registration
      semantics.
    - Acceptance: `fail_template_type_actual_missing_existential_001`'s
      rejection is derivable (unsatisfiable attribute-chain actual is
      rejected at the instantiation site, and no `ex y st y is hollow set`
      style axiom is ever emitted); pass fixtures show gated actuals with
      evidence lowering cleanly.
    - Verify: `cargo test -p mizar-core`, `cargo test -p mizar-checker`.
    - Deps: 27; consume mizar-checker task 43's built-in/base-shape
      inhabitation table and task 20 gate surface. Refs: spec 07 §7.8,
      17 §17.3.4, 18 §18.10.2; template_encoding_audit.md F2.
    - Completed by task 28: `TemplateTypeParameterInhabitationSeed` lowers
      checker-supplied witness binders to schema-context `∃x. is_T(x)`
      assumptions. `TemplateTypeActualGateSeed` preserves checker
      existential-gate status plus registration, base-evidence, guard-fact,
      and diagnostic backrefs. Non-satisfied gates emit only core diagnostics;
      they do not create actual-side existential axioms or proof obligations.
      Rust fixtures cover accepted registration/base/fact evidence,
      missing-existential rejection for an actual, and invalid gate payloads.
      No `doc/spec`, existing `.miz`, expectation, traceability metadata,
      source-derived runner, or fake checker payload was changed.

29. **Scheme-actual signature compatibility, guard obligations, and functor-actual validation (F4, F6, F8).** [x]
    - Implement the §18.10.4/§18.9 rules for `defpred`/`deffunc` actuals:
      contravariant domain / covariant codomain widening checks; functor
      guards are proof obligations discharged at instantiation, never
      asserted as axioms (no `deffunc shrink(x be Nat) -> Integer` false
      axiom). Implement the §18.10.3 rule for schemes applied inside
      template bodies with the enclosing template's parameters as actuals
      (F6). Reject partial (unpromoted) algorithm actuals for `func(...)`
      parameters — only `deffunc`, template functors, and promoted
      `terminating` algorithms denote FOL function symbols (F8).
    - Acceptance: `fail_template_func_actual_result_widening_001`'s
      rejection is derivable; guard obligations appear as obligation seeds
      (task 18 surface), not asserted axioms; a partial-algorithm actual is
      rejected with a stable diagnostic; nested scheme application uses the
      enclosing parameters soundly per the substitution-lemma
      reconstruction.
    - Completed by task 29: `TemplateSchemeActualSeed` / `TemplateSchemeActual`
      preserve checker-owned scheme-actual rows for type, predicate, and
      functor parameters. Predicate/functor rows keep directional widening
      evidence (schema domain to actual parameter, actual result to schema
      codomain for functors), accepted functor rows emit `Skipped`
      checker-initial guard seeds as traceability instead of axioms or active
      VCs, partial/void/unsupported actuals are diagnostic-only, and enclosing
      template parameters preserve substitution-composition metadata without
      creating fresh symbols or source-derived closure expansions. Source
      extraction and active corpus execution remain external/deferred.
    - Verify: `cargo test -p mizar-core`, `cargo test -p mizar-vc` (seed
      handoff), `cargo test -p mizar-test`.
    - Deps: 27; obligation seeds flow through task 18. Refs: spec 18
      §18.9/§18.10.3/§18.10.4/§18.8.4; template_encoding_audit.md F4, F6,
      F8.

30. **Sethood plumbing for type parameters (F5).** [x]
    - Key Fraenkel-comprehension gating inside template bodies to
      bound-inherited or constraint-supplied sethood per the §18.10.2
      sethood paragraph: a bare type parameter carries no sethood, so a
      comprehension ranging over it is rejected unless the bound or an
      explicit constraint supplies sethood evidence.
    - Acceptance: explicit payload fixtures derive the same rejection as
      `fail_template_fraenkel_over_type_param_001` (no Russell-style
      comprehension over `para[set]`), preserve bound-inherited and
      constraint-supplied sethood in generated Fraenkel origins, keep ordinary
      non-template Fraenkel evidence unchanged, and fail closed for malformed
      or duplicate cross-reference payloads. Source-derived extraction and
      active corpus execution remain external/deferred.
    - Verify: `cargo test -p mizar-core`, `cargo test -p mizar-checker`.
    - Deps: 28; consume mizar-checker task 43's parameterized sethood form
      (SSA-013). Refs: spec 13 §13.4.2, 18 §18.10.2;
      template_encoding_audit.md F5.

## Recommended Verification

Run after each task:

```text
cargo test -p mizar-core
cargo clippy -p mizar-core --all-targets -- -D warnings
```

For tasks that touch the checker boundary or the corpus, also run:

```text
cargo test -p mizar-checker
cargo test -p mizar-test
```

For the obligation-seed handoff and architecture-22 anchor inputs, also run:

```text
cargo test -p mizar-vc
```

Check the task off here once tests pass.

## Notes

- `CoreIr` is backend-neutral: no ATP encoding decisions, no parser trivia,
  no unresolved names, no surface-only details.
- Elaboration never runs proof search and never activates registrations;
  failed semantic sites remain explicit error nodes or skipped items.
- The binder library is soundness-relevant: the kernel independently
  re-checks substitutions, so keep the representation replayable and the
  invariants property-tested.
- Phase 10 lives here per internal 07; `mizar-vc` consumes `ControlFlowIr`
  and never mutates it.
