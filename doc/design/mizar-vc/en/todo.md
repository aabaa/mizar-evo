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
| kernel_evidence_handoff | `kernel_evidence_handoff.md` (task 24) | `src/kernel_evidence_handoff.rs` | [x] |

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
- **Kernel formula/substitution handoff: task-24 spec and task-25 builder
  complete for explicit producer payloads.** `mizar-kernel` tasks 23-29 now provide the checker-side
  formula/substitution evidence path. `VcIr` remains prover-independent, and
  the task-25 builder maps local context, premises, generated formulas,
  substitutions, imported fact requirements, discharge diagnostics, and the
  goal into the kernel evidence schema without adding backend traces, SAT
  clauses, resolution traces, or solver-specific proof methods.
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
     call-precondition, branch, match, range-loop, collection-loop, term-derived
     termination, and Pick non-emptiness payload families, plus historical
     partial-termination/ghost-erasure data rows, visible as
     deferred/no-candidate records rather
     than fabricated VCs. Task 30 later classifies the latter as evidence-admission
     or zero-VC/static boundaries.
   - Tests: candidate fixtures for goal-bearing preconditions,
     postconditions, assertions, invariant entry/preservation, and
     break/continue classifications; no-candidate/deferred fixtures for
     missing flow sites, missing flow data, term-derived termination, partial
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

17. **Public-enum forward-compatibility policy.** [x]
    - Apply the `mizar-frontend` task-25 procedure to each public enum;
      record decisions in the owning module specs.
    - Deps: 14. Spec: all module specs.

18. **Source/spec correspondence audit.** [x]
    - Trace every public API and promised behavior in the module specs to
      implementation and tests; record gaps as follow-up tasks.
    - Deps: 17. Spec: all module specs and this TODO.

19. **Bilingual documentation sync audit.** [x]
    - Compare each English canonical document under `doc/design/mizar-vc/en/`
      with its Japanese companion and synchronize content.
    - Deps: 18. Spec: repository documentation policy.

20. **Obligation anchors and cross-edit reuse identity.** [x]
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
      The proof-witness branch remains a downstream `external_dependency_gap`;
      Task 20 covers the currently available deterministic-discharge branch.
    - Deps: 2, 12, 14, 16. Spec:
      [22.incremental_verification_contract.md](../../architecture/en/22.incremental_verification_contract.md),
      [07.vc_generation.md](../../architecture/en/07.vc_generation.md),
      [18.dependency_fingerprint.md](../../architecture/en/18.dependency_fingerprint.md).

21. **Architecture-22 follow-up audit.** [x]
    - Re-run the source/spec correspondence and bilingual documentation sync
      audits for the task-20 anchor, fingerprint, and proof-reuse identity
      contract; record any remaining architecture-22 gaps as follow-up tasks
      before consumers depend on the contract.
      Task 21 records the focused paired `architecture_22_audit.md` artifact
      and leaves proof-witness hashes, ATP/kernel/proof/cache/artifact
      consumers, source-derived runner support, and incomplete upstream stable
      payload families as classified `external_dependency_gap` / `deferred`
      follow-up.
    - Deps: 20. Spec: all module specs, this TODO, and repository
      documentation policy.

22. **Module-boundary refactor gate.** [x]
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
      Task 22 records the paired `module_boundary_audit.md` artifact and finds
      no required move-only split before closeout. Optional private helper/test
      splits inside large implementation files remain future maintenance work
      and must be separate move-only tasks if pursued.
    - Deps: 21. Spec: this TODO,
      [internal 07](../../internal/en/07.crate_module_layout.md), all module
      specs.

23. **Closeout report and quality review.** [x]
    - Add the English/Japanese `crate_exit_report.md` pair, backfill the Task
      22 commit hash in the ledgers, resolve the closeout bilingual audit row,
      run broad verification, and record the final quality review score.
      Current result: crate development is complete with quality score 94/100,
      all hard gates passing, broad workspace verification passing, remaining
      `external_dependency_gap` / `deferred` items recorded, and the next-phase
      handoff included in the closeout report.
    - Tests: `cargo fmt --check`,
      `cargo clippy --all-targets --all-features -- -D warnings`,
      `cargo test`, `git diff --check`, and staged `git diff --cached --check`.
    - Deps: 22. Spec:
      [autonomous_crate_development.md](../../autonomous_crate_development.md),
      this TODO, and the crate exit criteria.

### Kernel evidence handoff follow-ups

24. **Spec: kernel evidence handoff.** [x]
    - Define how `VcIr`, local context, premise refs, generated formulas,
      discharge records, and goals map into the formula/substitution kernel
      evidence format. Preserve prover independence: no TPTP/SMT-LIB text,
      SAT clauses, backend logs, resolution traces, or solver proof methods
      may enter `VcIr`. Task 24 adds the paired
      `kernel_evidence_handoff.md` spec and records that `mizar-kernel` tasks
      23-29 now provide the checker-side formula/substitution evidence path;
      the VC builder and downstream consumers remain later work.
    - Tests: docs-only verification (`git diff --check`,
      staged `git diff --cached --check`, plus the docs sync review record).
    - Deps: 23, `mizar-kernel` tasks 23-29. Spec:
      [kernel_evidence_handoff.md](./kernel_evidence_handoff.md),
      [15.kernel_certificate_format.md](../../architecture/en/15.kernel_certificate_format.md),
      [08.reasoning_boundary.md](../../architecture/en/08.reasoning_boundary.md).

25. **Kernel evidence handoff builder.** [x]
    - Implemented an immutable handoff builder that packages existing `VcIr`
      data plus explicit producer formula/substitution/provenance payloads
      into kernel evidence inputs. Missing substitution/provenance payloads
      remain fail-closed or classified `external_dependency_gap` / `deferred`,
      not fabricated.
    - Tests: deterministic handoff rendering; local-context, premise,
      imported-fact, substitution, discharge-diagnostic, final-goal, and
      prohibited-backend-material coverage; missing payloads remain
      fail-closed.
    - Deps: 24, `mizar-kernel` task 29. Spec:
      [kernel_evidence_handoff.md](./kernel_evidence_handoff.md).

26. **Dependency-slice and proof-reuse identity update.** [x]
    - Extended dependency slices and architecture-22 proof-reuse identity to
      include the current task-25 canonical kernel evidence handoff hash.
      Legacy reuse without a current handoff now fails closed. Proof-witness
      reuse remains external until `mizar-proof`, `mizar-cache`, and
      `mizar-artifact` define their corresponding schemas.
    - Tests: kernel-evidence hash participation in slice fingerprints and
      reuse keys; missing handoff, stale slice, duplicate handoff, unknown VC,
      and selected-VC mismatch fail closed; unavailable downstream consumers
      remain `external_dependency_gap` / `deferred`.
    - Deps: 25. Spec:
      [22.incremental_verification_contract.md](../../architecture/en/22.incremental_verification_contract.md),
      `dependency_slice.md`.

### Kernel soundness-audit follow-ups (2026-07-03)

The kernel acceptance-boundary audit
([soundness_argument.md](../../mizar-kernel/en/soundness_argument.md),
findings F1, F2, F6) assigns this crate the producer side of three
kernel-facing contracts. The full finding-to-task disposition table lives in
the [mizar-kernel TODO](../../mizar-kernel/en/todo.md); the tasks below are
the paired producer tasks.

27. **Explicit goal polarity in the kernel evidence handoff (kernel F1).** [x]
    - Make the task-25 handoff builder state the goal polarity it emits and
      forbid `AssertTrueForConsistency` for proof obligations: every
      proof-obligation handoff declares refutation polarity explicitly, and
      the builder rejects (fail-closed) any caller request that would pair a
      proof obligation with consistency polarity. Update
      `kernel_evidence_handoff.md` (en+ja) to state the polarity contract
      and its binding to the target obligation's check kind (architecture 15
      "Goal Polarity Is Bound By The Target Obligation").
    - Acceptance: builder output always carries an explicit polarity; a Rust
      regression shows a consistency-polarity request for a proof obligation
      is rejected with a stable diagnostic; the spec names the polarity
      field.
    - Verify: `cargo test -p mizar-vc`,
      `cargo clippy -p mizar-vc --all-targets -- -D warnings`;
      `cargo test -p mizar-kernel` when the kernel-side task 30 exists.
    - Deps: 25; paired: mizar-kernel task 30. Spec: architecture 15,
      `kernel_evidence_handoff.md`; soundness_argument.md F1.
    - Done in task 27: `KernelEvidenceHandoffInput` now carries explicit
      `goal_polarity`; every current `VcKind` maps to
      `AssertFalseForRefutation`; consistency polarity for proof obligations
      fails closed with `GoalPolarityMismatch`. This closes only the
      producer-side F1 handoff contract; mizar-kernel task 30 still owns the
      trusted check-service acceptance binding.

28. **Context-identity payload for non-imported source bindings (kernel F2).** [x]
    - Produce the verification data the kernel needs to check that
      local-hypothesis, cited-premise, and generated-VC-fact bindings really
      belong to the target VC: bind the evidence's local/VC-fact sections to
      the canonical task-25/26 kernel-evidence handoff hash (or a superior
      recorded scheme) so the kernel can verify membership instead of
      trusting producer labels. Update `kernel_evidence_handoff.md` and
      `dependency_slice.md` (en+ja) in the same change as the schema.
    - Acceptance: handoff output carries a context-identity payload covering
      every non-imported source binding; mutating a binding (e.g. labeling
      the goal a local hypothesis) breaks the payload check; the kernel-side
      pass fixture of kernel task 31 can be built from this output.
    - Verify: `cargo test -p mizar-vc`; joint fixture check with
      `cargo test -p mizar-kernel` / `cargo test -p mizar-test` once kernel
      task 31 lands.
    - Deps: 26; paired: mizar-kernel task 31. Spec: architecture 15
      "Context Identity Covers Non-Imported Source Bindings";
      soundness_argument.md F2.
    - Done in task 28: `VcKernelEvidenceHandoff` now carries a stable
      `context_identity` payload with rows for every local-hypothesis,
      cited-premise, and generated-VC-fact formula evidence binding. The
      payload is bound to the target VC and canonical evidence hash, exposes
      `context_identity_hash()`, participates in dependency-slice and
      proof-reuse identity, excludes imported premises, and becomes stale when
      a canonical source binding is mutated. This closes only the producer-side
      F2 payload; mizar-kernel task 31 implements trusted membership
      verification.

29. **Imported-statement projection producer side (kernel F6).** [x]
    - Together with kernel task 33, specify and emit the projection from
      arch-18 imported statement fingerprints to the formula-tree
      fingerprints cited in kernel evidence, so imported facts become citable
      without weakening formula fingerprint validation or imported-fact
      identity checks. Dependency slices must carry whatever projection data
      the kernel validates.
    - Acceptance: a projected imported-fact citation round-trips through the
      handoff and carries the projection payload needed by the paired
      kernel-side pass fixture; stale or mismatched projections fail closed
      producer-side.
    - Verify: `cargo test -p mizar-vc`, `cargo test -p mizar-kernel`,
      `cargo test -p mizar-test`.
    - Deps: 28; paired: mizar-kernel task 33. Spec: architecture 15, 18;
      soundness_argument.md F6.
    - Done in task 29: `KernelImportedFormulaPayload` now carries a
      `KernelImportedStatementProjection` that maps an architecture-18
      imported-statement fingerprint to the kernel formula-tree fingerprint.
      The producer rejects unsupported imported-statement/formula algorithms,
      stale statement projections, mismatched projected formula fingerprints,
      empty or noncanonical projection payloads, and missing context/payload data. Canonical
      evidence rendering/hash input and dependency-slice `kernel_evidence`
      payloads include the projection data. Trusted kernel validation and the
      pass fixture are implemented by paired `mizar-kernel` task 33.

30. **Source-derived VC integration contract and exhaustive task
    decomposition.** [x]
    - Complete: [source_vc_decomposition.md](./source_vc_decomposition.md)
      freezes the exact structural Task-180 mapping, `MT10-VC-T180`, shared
      `MT10-VC-PV`, and the exhaustive VC 32-55 graph with exact available
      canonical/Core dependencies plus the bounded missing authority that
      blocks VC 53. It records entry-`requires`, loop-exit/range-hidden,
      formula/context, kind, anchor, and gate boundaries without adding a
      requirement, source, expectation, trace status/test, VC, or coverage.

31. **Exact source-derived contradiction VC integration.** [x]
    - Implement only Task 30's structural mapping for Core Task 31's real
      theorem obligation and `MT10-VC-T180`: validate the direct terminal
      proof-node backlink, empty CFG, exact ExistingCore handoff/intake, and map
      it to one `TerminalProofGoal`/`Open` VC with an honestly incomplete
      canonical-goal anchor. Add a distinct proof-verification source/sidecar,
      full `VcSet::debug_text()` baseline, exact trace row, and corruption
      matrix; keep the existing type-elaboration Task-180 case unchanged.
    - No marker injection, discharge, ATP/kernel/proof-policy execution,
      theorem acceptance, broader obligations/algorithms, placeholder runner,
      or expectation rebaseline. Deps: Task 30 and Core Task 31.
    - Complete: the exact borrowed adapter validates Core/CFG/handoff/fresh
      intake atomically and returns one marker-free open `TerminalProofGoal`.
      `MT10-VC-T180` adds the distinct proof-verification source/sidecar,
      double-generated full VcIr snapshot, exact covered trace row, public
      runner/CLI report, and admission/corruption/failure tests. The existing
      type-elaboration Task-180 sidecar and all broader deferred rows are
      unchanged.

32. **General theorem and proof VCs.** [ ] — Specs 04.5/14-16; Core
    33-35/37; `MT10-VC-PV/VC32`.
33. **Functor `equals`/`means` definition-correctness VCs.** [ ] — Specs
    10.3-10.6/10.12.2-10.12.6/16.6.1/16.6.4; Core 33-36;
    `MT10-VC-PV/VC33`. Result-type/type-correctness, guarded consistency, and
    missing-`otherwise` coverage apply to both styles; existence/uniqueness
    apply only to `means`.
34. **Predicate/functor algebraic-property VCs.** [ ] — Specs 09.5/10.6/16.6;
    Core 33-36; `MT10-VC-PV/VC34`.
35. **Explicit mode-sethood, structure, and property-implementation correctness
    VCs.** [ ] — Specs 05/07.8.1/16.6/19.2.2; Core 33-36; parser 48 where needed;
    `MT10-VC-PV/VC35`.
36. **Term-derived choice/Fraenkel/non-template `qua` VCs.** [ ] — Specs
    13.4-13.6/13.8.6-13.8.7/14; Core 34-36; `MT10-VC-PV/VC36`.
37. **Registration correctness VCs.** [ ] — Specs 07.8/16.6.3/17.2-17.5;
    Core 39; `MT10-VC-PV/VC37`.
38. **Redefinition compatibility/coherence VCs.** [ ] — Specs
    06.7/09.6-9.7/10.7-10.8/11.1/16.6/19.5; Core 38;
    `MT10-VC-PV/VC38`.
39. **Reduction `reducibility` equality VCs.** [ ] — Specs 17.6/17.9.4;
    Core 39; `MT10-VC-PV/VC39`. Simplification-order rejection is never a VC.
40. **Authenticated registration/cluster/reduction trace-context and fingerprint
    integration.** [blocked] — completed VC 37/39 outputs, Core 40, and Gate
    A1/MC-G004. When all dependencies exist, attach authenticated trace data to
    the real VC-37 registration/cluster correctness VCs and VC-39 reduction
    equality VCs and their snapshots in `MT10-VC-PV/VC40`; never create a
    trace-derived goal or standalone trace-only candidate.
41. **Direct template use-site VCs.** [ ] — Specs
    18.2/18.10.2/18.10.4-18.10.5; Core 34-38; `MT10-VC-PV/VC41`. Missing
    scheme/theorem roles remain outside this slice behind Core 41/Gate S1.
42. **Algorithm narrowing and field-update type VCs.** [ ] — Specs
    05.7-05.8/08.2/13.3/19.3/20.1; Core 42/46/48/52-53;
    `MT10-VC-PV/VC42`.
43. **Algorithm body-contract, return, and assertion VCs.** [ ] — Specs
    20.4-20.5/20.13; Core 42-43/46/48/52-53; `MT10-VC-PV/VC43`.
44. **Call-precondition and concrete substitution VCs.** [ ] — Specs
    20.4.1/20.8/20.13.1; Core 46/48/52-53; `MT10-VC-PV/VC44`.
45. **Conditional path-context VC integration.** [ ] — Specs 20.2.1/20.13.3;
    Core 43/48/52-53; `MT10-VC-PV/VC45`.
46. **Match context and explicit-exhaustiveness VCs.** [ ] — Specs
    20.2.5/20.13.3; Core 45/50/52-53; `MT10-VC-PV/VC46`.
47. **While invariant and jump-context VCs.** [ ] — Specs
    20.2.2/20.5/20.13.3; Core 43/46/48/52-53; `MT10-VC-PV/VC47`.
48. **Range-loop VCs.** [ ] — Specs 20.2.3/20.5/20.13.3; Core
    44/46/49/52-53; `MT10-VC-PV/VC48`.
49. **Collection-loop VCs.** [ ] — Specs 20.2.4/20.5/20.13.3; Core
    44/46/49/52-53; `MT10-VC-PV/VC49`.
50. **Pick non-emptiness VCs.** [ ] — Specs 20.3/20.13.3; Core
    42/46/48/52-53; `MT10-VC-PV/VC50`.
51. **Term-derived loop-measure VCs.** [ ] — Specs 20.5/20.7/20.13.3; Core
    46/48-49/52-53; `MT10-VC-PV/VC51`.
52. **Recursive and mutual-recursion decrease VCs.** [ ] — Specs
    20.7-20.8/20.13.4; Core 46/52-53; `MT10-VC-PV/VC52`.
53. **Partial-call verified-termination evidence admission boundary.** [blocked] — Specs
    20.7-20.8/20.13.1; Core 46/52-53; unexecuted `MT10-VC-PV/VC53`. Current
    canonical authority names no authenticated evidence-reference payload,
    producer, schema, or authentication contract. Future canonical authority must name the
    producer, reference identity/schema, authentication rules, and owning tests
    before this non-VC admission task can execute. It emits no
    `PartialTermination` VC.
54. **Snapshot/claim theorem VCs.** [ ] — Specs 20.6/20.13; Core 47/51-53;
    `MT10-VC-PV/VC54`.
55. **Ghost-isolation zero-VC integration.** [ ] — Specs
    20.1.3/20.3/20.13.5; Core 46/52-53; `MT10-VC-PV/VC55`. It validates
    no-VC accounting and Core-53 rejection, never a `GhostErasureSafety` VC.

Tasks 32-55 use the exact contracts, forbidden boundaries, real-source and
corruption requirements in
[source_vc_decomposition.md](./source_vc_decomposition.md). Each is one
nonempty logical task and one commit; no shared empty infrastructure task is
authorized.

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
- Kernel evidence handoff records may package formula/substitution evidence
  for checking, but `mizar-vc` must not run SAT solving or encode
  backend-specific proof methods.
