# mizar-vc TODO

> Canonical language: English. Japanese companion: [../ja/todo.md](../ja/todo.md).

## Status Legend

- [ ] not started
- [~] in progress
- [x] done

## Module Implementation

Remaining module specs are written by their own spec tasks (English and
Japanese in the same change) before the implementation tasks that cite them.
Module names follow the minimum split of
[internal 07](../../internal/en/07.crate_module_layout.md); the crate refines
architecture 07, 16, 18, and 19.

| Module | Spec | Source | Status |
|---|---|---|---|
| vc_ir | `vc_ir.md` (task 2) | `src/vc_ir.rs` | [x] |
| generator | `generator.md` (task 5) | `src/generator.rs` | [x] |
| discharge | `discharge.md` (task 10) | `src/discharge.rs` | [x] |
| dependency_slice | `dependency_slice.md` (task 13) | `src/dependency_slice.rs` | [x] |

`mizar-vc` implements pipeline phases 11-12: `CoreIr` and `ControlFlowIr` in,
prover-independent `VcIr` out, with deterministic pre-ATP discharge producing
evidence before any external prover runs. It is the boundary between
Mizar-side obligation generation and prover-side translation: this crate is
the only place that assigns `VcId`s, every obligation seed is intake-accounted
exactly once with explicit concrete-VC cardinality, and `mizar-atp` receives
only canonical `VcIr` with `NeedsAtp` status.

Dependency order: `vc_ir` data → seed intake → `generator` (theorem,
definition, registration-style correctness, algorithm VCs) →
normalization/status → `discharge` → `dependency_slice`.

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
  phase that assigns `VcId`s; seeds are intake-accounted exactly once and task
  8 enforces explicit no-VC / one-VC / expanded seed mappings.
- **Computation limits for discharge: resolved by task 11.** Pre-ATP discharge
  uses deterministic `DischargePolicy` data. The engine default policy key is
  `task-11-computation-step-limit` with `max_steps = 64`; callers may override
  it with another deterministic policy.
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

1. **Crate scaffold and lint-policy guard.** [x]
   - Add the `mizar-vc` workspace member depending on `mizar-session` and
     `mizar-core`; add `tests/lint_policy.rs` mirroring the `mizar-frontend`
     guard.
   - Tests: lint-policy guard passes; workspace builds.
   - Deps: `mizar-core` task 1. Spec: architecture 07.

2. **Spec: `vc_ir.md`.** [x]
   - Write the `VcIr` data-shape spec (English and Japanese, no code):
     `VcId`, `VcKind`, `LocalContext`, symbolic `PremiseRef`s, goal formula,
     `ProofHint`, the VC status model (including `NeedsAtp` and policy
     statuses), the seed accounting and concrete cardinality mapping rule, and the
     architecture-22 `ObligationAnchor` contract. The anchor spec must record
     anchor-ready local proof/program paths, label roles, normalized semantic
     origins, and source/core provenance, while keeping `VcId` and source
     ranges snapshot-local. Task 20 owns the later cross-edit reuse
     implementation and regression gate once discharge evidence, dependency
     slices, and determinism coverage exist.
   - Deps: 1. Spec: architecture 07 "VC IR"/"VC Status",
     [01.ir_layers.md](../../architecture/en/01.ir_layers.md).

3. **Implement `vc_ir` data shapes.** [x]
   - Implement `VcIr`, status, and context structures per task 2, plus a
     deterministic debug rendering.
   - Tests: construction round-trips; premise refs stay symbolic; rendering
     stability.
   - Deps: 2. Spec: `vc_ir.md`.

4. **Obligation-seed intake.** [x]
   - Consume the `mizar-core` seed handoff (theorem bodies, correctness
     conditions, checker-initial obligations, algorithm contracts) into a
     deterministic seed table (architecture 07 Step 2).
   - Tests: seed coverage fixtures; duplicate seeds rejected; deterministic
     order.
   - Deps: 3, `mizar-core` task 18. Spec: `vc_ir.md` (seed section).

### Generation (phase 11)

5. **Spec: `generator.md`.** [x]
   - Write the generation spec (English and Japanese, no code) with named
     sections: local-context construction, theorem/definition VCs (Step 3),
     explicit registration/redefinition/reduction correctness seeds when
     available, algorithm VCs over structured control flow (Step 4),
     controlled definition unfolding, and normalization/classification
     (Step 5).
   - Deps: 2. Spec: architecture 07 "Step 3"-"Step 5",
     [17.clusters_and_registrations.md](../../../spec/en/17.clusters_and_registrations.md),
     [16.theorems_and_proofs.md](../../../spec/en/16.theorems_and_proofs.md),
     [20.algorithm_and_verification.md](../../../spec/en/20.algorithm_and_verification.md).

6. **Theorem, definition, generated core, and registration-style correctness VCs.** [x]
   - Generate VCs for theorem proof steps, citations, and definition
     correctness conditions, preserving explicit local contexts. Generate
     explicit core-seed obligations for non-emptiness, sethood, and Fraenkel
     membership axioms. When checker-initial or core correctness seeds explicitly represent
     registration, redefinition, or reduction correctness, preserve them as
     registration-style correctness VCs; when those explicit payloads are not
     available, classify the gap as external/deferred rather than fabricating
     registration activation or proof acceptance.
   - Tests: VC fixtures per obligation kind; generated core seed fixtures;
     local contexts explicit, never implied by global state; unavailable
     explicit registration payloads recorded as deferred.
   - Deps: 4, 5. Spec: `generator.md` (theorem/definition section).

7. **Algorithm VCs.** [x]
   - Generate VCs from explicit flow-derived `ControlFlowIr` handoff rows for
     goal-bearing contracts, assertions, and invariants. Keep unavailable
     call-precondition, branch, match, range-loop, collection-loop, term-only
     termination, partial-termination, Pick non-emptiness, and ghost-erasure
     payload families visible as deferred/no-candidate records rather than
     fabricated VCs.
   - Tests: candidate fixtures for goal-bearing preconditions,
     postconditions, assertions, invariant entry/preservation, and
     break/continue classifications; no-candidate/deferred fixtures for
     missing flow sites, missing flow data, term-only termination, partial
     termination, ghost erasure, and unavailable audit families such as
     old-state assignment, field-update alias identity, branch/match,
     `downto`/`step` range loops, and ghost-only `Pick` erasure.
   - Deps: 6, `mizar-core` task 16. Spec: `generator.md` (algorithm
     section).

8. **Normalization, classification, and `VcId` assignment.** [x]
   - Normalize and classify VCs (Step 5), assigning deterministic `VcId`s;
     enforce that every seed is intake-accounted exactly once, that concrete
     cardinality is represented as no VC / one VC / explicit expansion, and
     that nothing else assigns ids.
   - Tests: id determinism across runs; seed accounting and seed-to-VC mapping
     fixtures; classification fixtures.
   - Deps: 7. Spec: `generator.md` (normalization section), `vc_ir.md`.

9. **Status and policy model.** [x]
   - Implement deterministic status-policy projection (`Preserve`,
     `NeedsAtp`, `PolicyOpen`, `AssumedByPolicy`) so verifier policy is
     reflected in VCs without erasing or weakening ATP-bound obligations.
     Discharged evidence remains deferred to the discharge tasks.
   - Tests: transition fixtures; policy statuses never drop contexts; no
     discharge evidence is created by status projection.
   - Deps: 8. Spec: `vc_ir.md` (status section), architecture 07 "Status
     and Policy Are Reflected in VCs".

### Pre-ATP discharge (phase 12)

10. **Spec: `discharge.md`.** [x]
    - Write the pre-ATP discharge spec (English and Japanese, no code):
      which obligation forms are discharged Mizar-side (deterministic or
      computation-based), the computation-limit model, explainability
      records, and the rule that ATP-bound VCs are never erased or weakened.
    - Deps: 2. Spec: architecture 07 "Step 6"/"Pre-ATP Discharge Is
      Deterministic and Explainable",
      [08.reasoning_boundary.md](../../architecture/en/08.reasoning_boundary.md).

11. **Deterministic discharge engine.** [x]
    - Add `src/discharge.rs`, expose `pub mod discharge`, update the lint
      guard, and implement the task-11 discharge API for explicit classes
      already represented in `VcIr`.
    - Record the engine default computation limit. Use minimal stable
      `DischargeEvidenceRef` values for discharged VCs; detailed evidence
      serialization remains task 12.
    - Tests: discharge tautology/contradiction, explicit local facts, explicit
      trace refs, policy-gated definitional reductions, and bounded computation;
      limit-exceeded or unsupported cases preserve full `NeedsAtp` context and
      stable explanations.
    - Gap classification: resolve the task-11 `source_drift`/`test_gap` for
      engine, module declaration, lint guard, and focused tests. Keep dependency
      slices, ATP/kernel/proof/cache/corpus integration, `.miz` fixtures,
      expectations, `doc/spec`, and traceability metadata deferred.
    - Deps: 9, 10. Spec: `discharge.md`.

12. **Discharge evidence and explanations.** [x]
    - Record in-memory replayable evidence for each newly discharged VC
      (rule applied, inputs, explicit trace refs, policy inputs, computation
      hints, and limit tuple when present) and preserved-evidence records for
      pre-existing `Discharged` inputs.
    - Tests: deterministic evidence render/clone/accessor coverage; every
      discharged output VC has matching evidence; policy/deferred/error VCs
      have explanations but no discharged evidence.
    - Deps: 11. Spec: `discharge.md` (evidence section).

### Dependency slices and follow-ups

13. **Spec: `dependency_slice.md`.** [x]
    - Write the dependency-slice spec (English and Japanese, no code): which
      imported facts, registrations, and definitions each VC depends on, and
      how slices feed canonical dependency-slice fingerprints, proof reuse,
      and incremental rebuilds. Specify that incomplete or unknown dependency
      coverage is represented conservatively so consumers can force cache
      misses.
    - Deps: 2. Spec:
      [18.dependency_fingerprint.md](../../architecture/en/18.dependency_fingerprint.md).

14. **Dependency-slice computation.** [x]
    - Compute per-VC dependency slices deterministically from local contexts,
      generated formulas, premises, proof hints, anchors, statuses, seed
      accounting, policy inputs, trace references, and task-12 discharge
      evidence/explanations.
    - Tests: slice fixtures; local context/generated formula/core-goal
      formula/premise/proof hint/policy/anchor/seed/discharge-evidence
      dependencies; definition/unfold dependencies; trace refs; unused facts
      excluded; conservative unknown coverage; deterministic ordering and
      fingerprint/debug rendering; `VcId` excluded from reusable fingerprints;
      unknown markers included in fingerprints; completeness/cache-miss intent
      is consumer-visible; mismatched `DischargeOutput`/`VcSet` is rejected;
      status-boundary preservation.
    - Deps: 8, 12, 13. Spec: `dependency_slice.md`.

15. **Corpus runner at stage `proof_verification`.** [x]
    - Reassess `mizar-test` support before editing. If an active
      `proof_verification` runner and source-to-core extraction seams exist,
      wire `tests/miz/{pass,fail}/` cases through the harness with
      `spec_trace.toml` entries; seed cases for generation and discharge,
      including the algorithm VC review-audit cases listed in task 7.
      If the runner or extraction seams are still missing, record the corpus
      obligations as deferred with concrete external-dependency reasons instead
      of fabricating active fixtures.
    - Deps: 11. Spec: [staged_model.md](../../mizar-test/en/staged_model.md).

16. **Determinism suite.** [x]
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

20. **Obligation anchors and cross-edit reuse identity.** [ ]
    - Complete the cross-edit reuse implementation for the task-2 `VcIr` /
      seed contract by wiring `ObligationAnchor`, canonical VC fingerprints,
      local-context fingerprints, and dependency-slice fingerprints through the
      generated obligations. `VcId`, `SourceRange`, `SurfaceNodeId`, and
      task-local ids must remain snapshot-local evidence only, never
      cross-edit proof-reuse identity.
    - Tests: inserting a proof step before an existing obligation changes
      `VcId` ordering but preserves reuse eligibility only when the anchor,
      canonical VC fingerprint, local context fingerprint, dependency slice
      fingerprint, compatible verifier policy, and selected proof witness hash
      or deterministic discharge hash match.
    - Deps: 2, 12, 14, 16. Spec:
      [22.incremental_verification_contract.md](../../architecture/en/22.incremental_verification_contract.md),
      [07.vc_generation.md](../../architecture/en/07.vc_generation.md),
      [18.dependency_fingerprint.md](../../architecture/en/18.dependency_fingerprint.md).

21. **Architecture-22 follow-up audit.** [ ]
    - Re-run the source/spec correspondence and bilingual documentation sync
      audits for the task-20 anchor, fingerprint, and proof-reuse identity
      contract; record any remaining architecture-22 gaps as follow-up tasks
      before consumers depend on the contract.
    - Deps: 20. Spec: all module specs, this TODO, and repository
      documentation policy.

22. **Module-boundary refactor gate.** [ ]
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
    - Deps: 21. Spec: this TODO,
      [internal 07](../../internal/en/07.crate_module_layout.md), all module
      specs.

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

For the architecture-22 reuse-identity contract, also run the consumers of
the anchor and proof metadata when those crates exist in the workspace and the
task actually touches the integration boundary:

```text
cargo test -p mizar-cache
cargo test -p mizar-proof
```

If either crate is not yet available, classify the missing command as an
`external_dependency_gap` / `deferred` verification item for that task rather
than adding placeholder crates.

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
