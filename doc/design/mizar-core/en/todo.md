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
| elaborator | `elaborator.md` (task 7) | `src/elaborator.rs` | [~] |
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

19. **Snapshot dumps and corpus contributions.** [ ]
    - Wire deterministic `CoreIr`/`ControlFlowIr` renderings into corpus
      snapshot baselines at stages `type_elaboration` and
      `proof_verification`.
    - Deps: 12, 15. Spec: [staged_model.md](../../mizar-test/en/staged_model.md),
      [snapshot.md](../../mizar-test/en/snapshot.md).

20. **Determinism suite.** [ ]
    - Property coverage that identical `ResolvedTypedAst` inputs produce
      identical core items, binder numbering, CFGs, and renderings.
    - Deps: 18. Spec: [20.test_strategy.md](../../architecture/en/20.test_strategy.md).

21. **Public-enum forward-compatibility policy.** [ ]
    - Apply the `mizar-frontend` task-25 procedure to each public enum;
      record decisions in the owning module specs.
    - Deps: 18. Spec: all module specs.

22. **Source/spec correspondence audit.** [ ]
    - Trace every public API and promised behavior in the module specs to
      implementation and tests; record gaps as follow-up tasks.
    - Deps: 21. Spec: all module specs and this TODO.

23. **Bilingual documentation sync audit.** [ ]
    - Compare each English canonical document under
      `doc/design/mizar-core/en/` with its Japanese companion and
      synchronize content.
    - Deps: 22. Spec: repository documentation policy.

24. **Module-boundary refactor gate.** [ ]
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
    - Deps: 23. Spec: this TODO,
      [internal 07](../../internal/en/07.crate_module_layout.md), all module
      specs.

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
