# mizar-vc TODO

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
architecture 07, 16, 18, and 19.

| Module | Spec | Source | Status |
|---|---|---|---|
| vc_ir | `vc_ir.md` (task 2) | `src/vc_ir.rs` | [ ] |
| generator | `generator.md` (task 5) | `src/generator.rs` | [ ] |
| discharge | `discharge.md` (task 10) | `src/discharge.rs` | [ ] |
| dependency_slice | `dependency_slice.md` (task 13) | `src/dependency_slice.rs` | [ ] |

`mizar-vc` implements pipeline phases 11-12: `CoreIr` and `ControlFlowIr` in,
prover-independent `VcIr` out, with deterministic pre-ATP discharge producing
evidence before any external prover runs. It is the boundary between
Mizar-side obligation generation and prover-side translation: this crate is
the only place that assigns `VcId`s, every obligation seed becomes a VC
exactly once, and `mizar-atp` receives only canonical `VcIr` with `NeedsAtp`
status.

Dependency order: `vc_ir` data → seed intake → `generator` (theorem,
definition, algorithm VCs) → normalization/status → `discharge` →
`dependency_slice`.

Each task below is deliberately small — one module spec, or one behavior slice
of one module — so that a single task can be implemented, tested, and
committed autonomously without holding the rest of the crate in flight.

## Crate Prerequisites

The crate depends on `mizar-session` and `mizar-core` (`CoreIr`,
`ControlFlowIr`, binder library, obligation seeds). Generation tasks are
gated on `mizar-core` task 18 (seed handoff). Architecture:
[07.vc_generation.md](../../architecture/en/07.vc_generation.md),
[18.dependency_fingerprint.md](../../architecture/en/18.dependency_fingerprint.md);
crate ownership: [internal 07](../../internal/en/07.crate_module_layout.md).

## Resolved And Open Decisions

- **ControlFlowIr ownership: resolved by internal 07.** `mizar-core` builds
  `ControlFlowIr` (phase 10); this crate consumes it for algorithm VCs and
  never mutates it.
- **`VcId` assignment: resolved by architecture 07.** Phase 11 is the only
  phase that assigns `VcId`s; seeds become VCs exactly once (task 8 enforces
  both).
- **Computation limits for discharge: open, resolved by task 11.** Pre-ATP
  discharge must be deterministic for identical source, dependencies,
  toolchain, policy, and computation limits; decide the limit model
  (step-count budgets, recursion depth, numeric bounds) and its
  configuration surface, and record it in `discharge.md`.
- **Discharge-evidence validation scope: open, owned by `mizar-proof`
  task 6.** Whether the task-12 discharge evidence is kernel-replayed or
  accepted as deterministic built-in evidence per policy; this crate
  guarantees the evidence is replayable either way. Registered at the top
  level.
- **Diagnostics record: follows the `mizar-resolve` decision** on
  `mizar-diagnostics` adoption timing. Registered at the top level.

## Ordered Task List

Keep `cargo test -p mizar-vc` green after each task (see
[Recommended Verification](#recommended-verification)).

### VC IR and seed intake

1. **Crate scaffold and lint-policy guard.** [ ]
   - Add the `mizar-vc` workspace member depending on `mizar-session` and
     `mizar-core`; add `tests/lint_policy.rs` mirroring the `mizar-frontend`
     guard.
   - Tests: lint-policy guard passes; workspace builds.
   - Deps: `mizar-core` task 1. Spec: architecture 07.

2. **Spec: `vc_ir.md`.** [ ]
   - Write the `VcIr` data-shape spec (English and Japanese, no code):
     `VcId`, `VcKind`, `LocalContext`, symbolic `PremiseRef`s, goal formula,
     `ProofHint`, the VC status model (including `NeedsAtp` and policy
     statuses), and the seeds-become-VCs-exactly-once rule.
   - Deps: 1. Spec: architecture 07 "VC IR"/"VC Status",
     [01.ir_layers.md](../../architecture/en/01.ir_layers.md).

3. **Implement `vc_ir` data shapes.** [ ]
   - Implement `VcIr`, status, and context structures per task 2, plus a
     deterministic debug rendering.
   - Tests: construction round-trips; premise refs stay symbolic; rendering
     stability.
   - Deps: 2. Spec: `vc_ir.md`.

4. **Obligation-seed intake.** [ ]
   - Consume the `mizar-core` seed handoff (theorem bodies, correctness
     conditions, checker-initial obligations, algorithm contracts) into a
     deterministic seed table (architecture 07 Step 2).
   - Tests: seed coverage fixtures; duplicate seeds rejected; deterministic
     order.
   - Deps: 3, `mizar-core` task 18. Spec: `vc_ir.md` (seed section).

### Generation (phase 11)

5. **Spec: `generator.md`.** [ ]
   - Write the generation spec (English and Japanese, no code) with named
     sections: local-context construction, theorem/definition VCs (Step 3),
     algorithm VCs over structured control flow (Step 4), controlled
     definition unfolding, and normalization/classification (Step 5).
   - Deps: 2. Spec: architecture 07 "Step 3"-"Step 5",
     [16.theorems_and_proofs.md](../../../spec/en/16.theorems_and_proofs.md),
     [20.algorithm_and_verification.md](../../../spec/en/20.algorithm_and_verification.md).

6. **Theorem and definition VCs.** [ ]
   - Generate VCs for theorem proof steps, citations, and definition
     correctness conditions, preserving explicit local contexts.
   - Tests: VC fixtures per obligation kind; local contexts explicit, never
     implied by global state.
   - Deps: 4, 5. Spec: `generator.md` (theorem/definition section).

7. **Algorithm VCs.** [ ]
   - Generate VCs from `ControlFlowIr` following structured control flow:
     contracts, invariants (entry/preservation), assertions, ghost rules,
     and termination measures.
   - Tests: per-construct VC fixtures (`while`, `if`, `match`); invariant
     entry/preservation pairs; termination VCs reference measures. Include the
     review-audit algorithm fixtures for old-state assignment, field-update
     alias identity, `break` exits that do not gain `not C`,
     `continue`/decreasing checks, `downto` and `step` range loops, and
     ghost-only `Pick` erasure.
   - Deps: 6, `mizar-core` task 16. Spec: `generator.md` (algorithm
     section).

8. **Normalization, classification, and `VcId` assignment.** [ ]
   - Normalize and classify VCs (Step 5), assigning deterministic `VcId`s;
     enforce that every seed becomes exactly one VC and nothing else assigns
     ids.
   - Tests: id determinism across runs; seed↔VC bijection; classification
     fixtures.
   - Deps: 7. Spec: `generator.md` (normalization section), `vc_ir.md`.

9. **Status and policy model.** [ ]
   - Implement the VC status transitions (open, discharged, `NeedsAtp`,
     policy-assigned statuses) so verifier policy is reflected in VCs
     without erasing or weakening ATP-bound obligations.
   - Tests: transition fixtures; policy statuses never drop contexts.
   - Deps: 8. Spec: `vc_ir.md` (status section), architecture 07 "Status
     and Policy Are Reflected in VCs".

### Pre-ATP discharge (phase 12)

10. **Spec: `discharge.md`.** [ ]
    - Write the pre-ATP discharge spec (English and Japanese, no code):
      which obligation forms are discharged Mizar-side (deterministic or
      computation-based), the computation-limit model, explainability
      records, and the rule that ATP-bound VCs are never erased or weakened.
    - Deps: 2. Spec: architecture 07 "Step 6"/"Pre-ATP Discharge Is
      Deterministic and Explainable",
      [08.reasoning_boundary.md](../../architecture/en/08.reasoning_boundary.md).

11. **Deterministic discharge engine.** [ ]
    - Implement discharge for the supported obligation forms with the
      decided computation limits; resolve the computation-limit decision and
      record it.
    - Tests: discharged fixtures reproduce bit-identically; limit-exceeded
      cases produce stable diagnostics, not wrong answers.
    - Deps: 9, 10. Spec: `discharge.md`.

12. **Discharge evidence and explanations.** [ ]
    - Record replayable evidence for each discharged VC (rule applied,
      inputs, computation steps) for diagnostics, artifacts, and later
      kernel-side validation per policy.
    - Tests: evidence round-trips; every discharged VC has evidence.
    - Deps: 11. Spec: `discharge.md` (evidence section).

### Dependency slices and follow-ups

13. **Spec: `dependency_slice.md`.** [ ]
    - Write the dependency-slice spec (English and Japanese, no code): which
      imported facts, registrations, and definitions each VC depends on, and
      how slices feed fingerprints and incremental rebuilds.
    - Deps: 2. Spec:
      [18.dependency_fingerprint.md](../../architecture/en/18.dependency_fingerprint.md).

14. **Dependency-slice computation.** [ ]
    - Compute per-VC dependency slices deterministically from premises,
      local contexts, and trace references.
    - Tests: slice fixtures; unused facts excluded; deterministic ordering.
    - Deps: 8, 13. Spec: `dependency_slice.md`.

15. **Corpus runner at stage `proof_verification`.** [ ]
    - Wire `tests/miz/{pass,fail}/` cases at stage `proof_verification`
      through the harness with `spec_trace.toml` entries; seed cases for
      generation and discharge, including the algorithm VC review-audit cases
      listed in task 7.
    - Deps: 11. Spec: [staged_model.md](../../mizar-test/en/staged_model.md).

16. **Determinism suite.** [ ]
    - Property coverage that identical inputs produce identical VC sets,
      ids, orders, statuses, slices, and discharge evidence.
    - Deps: 14. Spec: [20.test_strategy.md](../../architecture/en/20.test_strategy.md).

17. **Public-enum forward-compatibility policy.** [ ]
    - Apply the `mizar-frontend` task-25 procedure to each public enum;
      record decisions in the owning module specs.
    - Deps: 14. Spec: all module specs.

18. **Source/spec correspondence audit.** [ ]
    - Trace every public API and promised behavior in the module specs to
      implementation and tests; record gaps as follow-up tasks.
    - Deps: 17. Spec: all module specs and this TODO.

19. **Bilingual documentation sync audit.** [ ]
    - Compare each English canonical document under `doc/design/mizar-vc/en/`
      with its Japanese companion and synchronize content.
    - Deps: 18. Spec: repository documentation policy.

## Recommended Verification

Run after each task:

```text
cargo test -p mizar-vc
cargo clippy -p mizar-vc --all-targets -- -D warnings
```

For tasks that touch the core boundary or the corpus, also run:

```text
cargo test -p mizar-core
cargo test -p mizar-test
```

Check the task off here once tests pass.

## Notes

- `VcIr` stays prover-independent: no TPTP/SMT-LIB text, no backend process
  configuration beyond abstract hints, no certificates.
- Phase 12 may discharge VCs or assign policy statuses, but must not erase
  ATP-bound VCs or weaken their contexts; `mizar-atp` receives only
  canonical `VcIr` with `NeedsAtp` status.
- Premise references remain symbolic until ATP translation selects an
  encoding.
- Discharge evidence is untrusted production: trusted acceptance happens in
  `mizar-kernel`/`mizar-proof` per policy.
