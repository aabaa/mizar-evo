# Implementation Roadmap (Crate Sequencing)

> Canonical language: English. This is the top-level index for crate-level work
> ordering. For new non-trivial tasks, paired task contracts carry the detailed
> task record; per-crate TODOs carry concise status and have Japanese companions
> under each crate's `ja/` directory when that companion exists. Historical
> task bodies remain frozen logs.

This document records the current implementation order across crates. It
complements [README.md](./README.md) (design layout), the pipeline definition in
[architecture/en/00.pipeline_overview.md](./architecture/en/00.pipeline_overview.md),
and the crate ownership map in
[internal/en/07.crate_module_layout.md](./internal/en/07.crate_module_layout.md).

## How To Read This Document

- The [Sequential Execution Plan](#sequential-execution-plan) is the single
  ordering authority. Apply [development-process improvement priority](#development-process-improvement-priority)
  first, then its [M1 Dispatch Policy](#m1-dispatch-policy)
  for dependency-ready source-to-artifact integration slices; otherwise select
  the first unfinished owner task in step/item order whose contract permits
  the slice. Completed, dependency-blocked, and parked slices are skipped,
  not treated as global stops. A blocked slice blocks only its consumers.
  Recheck readiness on each dispatch and retain every open gate.
- Each entry names an owner task. For a new non-trivial task, follow the crate
  TODO's link to its paired `doc/design/task_contracts/{en,ja}/` record for
  scope, acceptance criteria, and verification. Crate TODOs retain only concise
  sequencing status and owner-local checklist deltas. This file never restates
  the detailed contract.
- The [Completion Gates](#completion-gates) table says what "done" means
  end to end. The appendices keep audit/contract traceability; they are
  reference indexes, not a second ordering.

## Status Legend

- [ ] not started
- [~] in progress
- [x] done

## Guiding Principles

1. **Authority flows from spec and tests.** Crate TODOs and source code refine
   `doc/spec/en/`, executable `.miz` tests, expectation metadata, and
   traceability records; they do not introduce language behavior on their own.
   Specification-intent changes update `doc/spec/en/` and `doc/spec/ja/`
   together (see AGENTS.md).
2. **Bottom-up by pipeline phase.** Source identity, lexical analysis, syntax,
   and parser/frontend orchestration are built before resolver, checker, proof,
   artifact, cache, and driver layers.
3. **Leaf-first within a layer.** Implement crates with no internal dependency
   first so each downstream crate can consume a tested, deterministic boundary.
4. **Keep early phase crates query-friendly.** Lexer, syntax, parser, and
   frontend APIs stay immutable and deterministic so later `mizar-driver` /
   `mizar-build` query and cache layers can wrap them.
5. **Separate scaffolds from completed milestones.** Some workspace crates exist
   with an initial slice only; their TODO state, not crate presence alone,
   determines readiness for downstream work.
6. **Fail closed, never fabricate.** A gap is closed only when the owning crate
   exposes the real producer or consumer seam. Do not close a gate by weakening
   tests, matching expectations to current behavior, fabricating semantic
   payloads, or moving trust authority into a convenient downstream crate.
7. **Prioritize a real vertical slice.** Within genuine producer dependencies,
   close the smallest source-to-verified-artifact path before broadening
   unrelated source profiles. Bundle three-to-five identical-shape slices
   when the protocol permits; do not multiply syntax variants into separate
   tasks or conflate distinct semantic/trust owners.

## Completion Gates

This roadmap is sufficient guidance for the verification-focused Mizar-evo
implementation to reach an end-to-end usable state, provided remaining
`external_dependency_gap`, `deferred`, `test_gap`, and `design_drift` records
are promoted into real owner tasks before they are closed. The authority order
in [autonomous_crate_development.md](./autonomous_crate_development.md) must
still agree with the resulting behavior. M1 is a bounded integration
milestone, not a substitute for any completion gate below.

| Gate | Completion condition | Plan steps | Primary owners |
|---|---|---|---|
| Source-to-semantics bridge | Real `.miz` inputs pass from frontend output through resolver and checker-owned payload extraction into `ResolvedTypedAst`, with active semantic corpus coverage instead of extraction-gap sentinels. The audit-1 corpus is active at its sidecar-defined stages; bounded VC generation and Pending coherence do not imply proof acceptance or unrestricted payload extraction. | 2, 4, 5 | `mizar-test`, `mizar-resolve`, `mizar-checker` |
| Core and VC bridge | Checker-derived payloads lower into `CoreIr`, `ControlFlowIr`, and source-derived VC inputs without reconstructing missing source or fabricating registration/proof facts. | 4, 5 | `mizar-checker`, `mizar-core`, `mizar-vc`, `mizar-test` |
| Proof and algorithm verification | Source-derived proof and algorithm obligations flow through VC generation, ATP candidate production, kernel checking, proof policy/status projection, and proof-reuse metadata with active `proof_verification` coverage. | 1, 3, 7 | `mizar-vc`, `mizar-atp`, `mizar-kernel`, `mizar-proof`, `mizar-cache`, `mizar-test` |
| Artifact publication | Verified module, registration, proof-witness, and diagnostic projections are emitted through real `mizar-artifact` store/manifest transactions from producer-owned outputs. | 6 | `mizar-artifact`, `mizar-ir`, `mizar-driver`, producer crates |
| Build orchestration | Clean, incremental, sequential, and parallel driver/build runs agree on externally visible artifacts, proof statuses, cache decisions, and diagnostics for implemented phases. | 6 | `mizar-driver`, `mizar-build`, `mizar-ir`, `mizar-cache`, `mizar-test` |
| User-facing projections | Public diagnostics, LSP features, and documentation rendering consume stable artifacts, diagnostic records, and metadata without owning semantic or proof authority. A VS Code user can install the Marketplace extension and start verifying without other manual setup. | 8 | `mizar-diagnostics`, `mizar-lsp`, `mizar-doc`, producer crates |

When all non-parked items in those gates are complete and the relevant broad
verification commands pass, the roadmap supports claiming a source-to-artifact
verification pipeline. Algorithm *execution* is a separate claim from algorithm
verification: before claiming executable algorithm runtime support, promote the
currently deferred MVM/code-extraction/backend specification work from
`spec.en.20.algorithm_and_verification` coverage into explicit owner tasks with
tests and artifact/build integration.

**Complete processor.** The complete-processor claim covers the whole
specification (chapters 1-24): the source-to-artifact pipeline above plus MVM
execution and `by computation`, code extraction and computational export, the
chapter-23 CLI commands (`explain`, `refine`, `minimize`, `semver-check`), the
MCP and agent interface, imported-module semantic payloads, editor
integration (LSP server, VS Code extension, toolchain installation), and
documentation generation. Items without an owner task must be promoted to
one before the claim. The port of the Mizar Mathematical Library is a
separate program that consumes processor features; its progress is part of
neither claim, and requirements it raises are recorded as processor tasks. Distinguish three
claims: (1) the bounded M1 trusted source-to-artifact demonstration, (2) the
verification-focused pipeline subject to the gates above, and (3) the
complete processor. Neither M1 nor Step 5 alpha completion grants broad
source-proof acceptance or a complete-processor claim.

If a future task discovers that a gate cannot be closed by the existing crate
TODOs, update this roadmap in the same change that records the new gap.

## Crate Status

All crates below are workspace members. "Next work" points into the
[Sequential Execution Plan](#sequential-execution-plan).

| Crate | Role | Status | Next work | TODO |
|---|---|---|---|---|
| mizar-session | Source identity, source maps, source loading, build snapshots, retention | [x] complete | — | [todo](./mizar-session/en/todo.md) |
| mizar-lexer | Raw scan, scope skeletons, lexical environments, token disambiguation | [x] complete plus Step 5A.2/5A.3/5A.5 increments | — | [todo](./mizar-lexer/en/todo.md) |
| mizar-syntax | Rowan-backed `SurfaceAst`, trivia, recovery, typed views | [x] historical milestone plus parser Tasks 48/46 increments, S-026 dense views, and Step 5A.1 complete | S-021 remains the sole deferred syntax task | [todo](./mizar-syntax/en/todo.md) |
| mizar-parser | Grammar, Pratt parsing, syntax recovery, parse-only corpus | [x] Tasks 1-48 plus bounded `PARSER-RECOVERY-B1B1P-P1` and Step 5A.4-5A.8 complete | No inferred Task 49; P-265-47D closed by the Chapter 8 list alignment | [todo](./mizar-parser/en/todo.md) |
| mizar-frontend | Source loading and phase 1-3 orchestration | [x] prior milestone plus Step 5A.2/5A.3/5A.6/5A.8 increments | — | [todo](./mizar-frontend/en/todo.md) |
| mizar-resolve | Module graph, namespaces, symbols, labels, signatures | [~] alpha semantic slices, bounded source-theorem producers and explicitly selected local algorithm call/dependency groups implemented | Step 6 general ModuleResolver/summary publication; external call identities and typed selection remain open | [todo](./mizar-resolve/en/todo.md) |
| mizar-test | Corpus discovery, expectations, staged model, traceability, harness | [~] alpha semantic bridge and source-derived resolver consumer slices implemented | Step 6 integration; pre-audit Task 49 remains gated by 5B.3 | [todo](./mizar-test/en/todo.md) |
| mizar-checker | Type checking, cluster/registration resolution, overload resolution | [~] prior milestone plus [Step 5C.1](./task_contracts/en/STEP5C1-VARIABLE-SEMANTICS.md), [Step 5C.2](./task_contracts/en/STEP5C2-STRUCTURE-SEMANTICS.md), [Step 5C.3](./task_contracts/en/STEP5C3-ATTRIBUTE-SEMANTICS.md), [Step 5C.4](./task_contracts/en/STEP5C4-MODE-SEMANTICS.md), [Step 5C.5](./task_contracts/en/STEP5C5-PREDICATE-FUNCTOR-SEMANTICS.md), [Step 5C.7](./task_contracts/en/STEP5C7-TERM-SEMANTICS.md), [Step 5C.8](./task_contracts/en/STEP5C8-FORMULA-SEMANTICS.md), [Step 5C.9](./task_contracts/en/STEP5C9-PROOF-ORGANIZATION.md), and [Step 5C.10](./task_contracts/en/STEP5C10-THEOREM-SKELETONS.md) semantics | Remaining producer tasks under existing dependencies; Task 277B remains not-ready/zero-credit | [todo](./mizar-checker/en/todo.md) |
| mizar-core | Elaboration, binder-normalized core logic, control-flow preparation | [x] core/control-flow milestone, tasks 27-32, plus [Step 5C.2](./task_contracts/en/STEP5C2-STRUCTURE-SEMANTICS.md) and [Step 5C.7](./task_contracts/en/STEP5C7-TERM-SEMANTICS.md) bounded normalization and [Step 5C.10](./task_contracts/en/STEP5C10-THEOREM-SKELETONS.md) theorem-skeleton lowering | Broader producer Tasks 33-53 beyond the Step 5 alpha corpus, under the Task-32 graph | [todo](./mizar-core/en/todo.md) |
| mizar-vc | VC IR, VC generation, deterministic pre-ATP discharge | [x] contradiction VC milestone through task 31; bounded theorem services and direct-return E1/E4 request/checking prerequisites implemented | Broader Tasks 32-55 and Step 7 task 60; VC 40/53 and S1 dependencies remain explicit | [todo](./mizar-vc/en/todo.md) |
| mizar-kernel | Trusted evidence parsing and checking | [x] SAT-backed milestone plus bounded theorem reflexivity and E1/E4 checking implemented | Step 7 task 36; task 32 remains parked | [todo](./mizar-kernel/en/todo.md) |
| mizar-atp | ATP encoding, backend execution, portfolio candidates | [x] candidate-evidence milestone through task 29; guarded Vampire extraction and explicit sequential portfolio execution implemented | step 7 | [todo](./mizar-atp/en/todo.md) |
| mizar-proof | Proof policy evaluation, status projection, witness selection | [x] policy/status/witness milestone through task 21; bounded theorem witness and native E1-inhabitation/E4 adoption implemented | Step 7 task 23 and accepted-output reuse integration | [todo](./mizar-proof/en/todo.md) |
| mizar-cache | Cache keys, fingerprints, proof reuse, cluster-db storage | [x] internal-cache milestone complete through task 24 | step 7 | [todo](./mizar-cache/en/todo.md) |
| mizar-artifact | Artifact schemas, summaries, store, manifest transactions | [~] schemas/store/manifest through task 24 and bounded theorem/reflexivity FILE/Artifact commit routes implemented | Full Step 6 task 17 emission and broader Step 7 task 25 built-in witnesses remain open | [todo](./mizar-artifact/en/todo.md) |
| mizar-ir | IR storage, snapshot handles, sealed output blobs, projections | [x] storage/projection milestone complete | step 6 | [todo](./mizar-ir/en/todo.md) |
| mizar-build | Phase 0 planning, task graph, scheduler, cache seam, commit boundary | [~] scheduler/native workspace inputs and bounded NoKey/cache-backed edit/clean comparisons implemented | Step 6 task 28, broader registered-service/cache equivalence | [todo](./mizar-build/en/todo.md) |
| mizar-driver | Build requests, phase registry, CLI/watch/LSP entry points | [~] bounded source-theorem services through genuine Kernel status/FILE/Artifact commit and explicit cache routes implemented | Step 6 general services/cache integration and broader real-service equivalence | [todo](./mizar-driver/en/todo.md) |
| mizar-diagnostics | Diagnostic registry, failure records, ordering, rendering | [x] internal milestone and E0220–E0224 import descriptors implemented | Step 8 broader public adoption | [todo](./mizar-diagnostics/en/todo.md) |
| mizar-lsp | Editor protocol and range conversion | [~] range/lifecycle, snapshot-backed syntax and retained-publication completion implemented; broader semantic consumers remain open | Step 8 remaining tasks 6–25 | [todo](./mizar-lsp/en/todo.md) |
| mizar-doc | Documentation rendering and extraction | [~] published reader/index/native inputs, comment attachment and bounded Markdown/link/render prerequisites implemented | Step 8 tasks 4–29 | [todo](./mizar-doc/en/todo.md) |

## Sequential Execution Plan

Revised 2026-07-06 after the July 2026 audit wave (see
[Appendix C](#appendix-c-july-2026-audit-follow-up-inventory)) and the
`mizar-test` task-20 reserve-bridge closeout. Steps 1-3 close audited
soundness holes and settle semantic decisions before further pipeline growth;
steps 4-8 are the implementation waves re-paced around them.

### Development-process improvement priority

Owner standing rule 2026-11-21: development-process improvement tasks take
priority over all other tasks, including the M1 and adopted-wave queues.

- [ ] **CI-RECEIPT-ACTIVATION — enforce signed local-full receipts; full tier.**
  Owner dispatch 2026-11-22, following the orchestrator's 17:08 JST decision:
  activate directly without shadow or transition. Provision production custody,
  pin reviewed identities, enforce exact-head sanity and required-check protection,
  replace per-push/PR full hosted testing with periodic reconciliation, and
  exercise the [activation boundary](./ci_receipt_model.md#activation-boundary).

- [x] **CI-RECEIPT-DESIGN — mizar CI receipt-model adoption design; Lane 1;
  design only.** Authority: owner CI policy recorded in the orchestrator
  ledger 2026-11-21 13:07 JST. Mirror oeis D9 / FCL D1: unit tests run locally;
  the receipt generator stays in orchestrator custody and binds the signed
  local-execution receipt to the sealed local full test. Design signing and
  a CI sanity job that verifies the receipt instead of re-running
  `cargo test` on push; trust-touching exceptions retain full CI. Dependencies:
  none to start design. Exit: a reviewed adoption design and frozen scope for
  a separate implementation unit; design precedes any implementation unit.
  [Adoption design](./ci_receipt_model.md) and [frozen successor scope](./task_contracts/en/CI-RECEIPT-DESIGN.md)
  own the handoff. CI-RECEIPT-IMPLEMENTATION needs a separate dispatch; current
  CI and verification policy remain effective, with no implementation credit.

### M1 Dispatch Policy

**Milestone: trusted source-to-artifact verification (open).** This policy is
part of the single Sequential Execution Plan, not a second task queue,
new task, contract, or change to proof policy. It supersedes numeric Step 5-8
order only for dependency-ready slices on the shortest authentic source
proof-to-artifact path. Existing crate TODOs and paired task contracts own
the exact scope, requirements, implementation gates, and acceptance.

**Target:** one supported real `.miz` theorem, from captured source through
Frontend, Resolver, Checker, Core/VC, Kernel-checked evidence, Proof selection,
and a committed/readable Artifact with witness via a real Driver service.
A direct-premise or reflexivity theorem is only a candidate: verify authentic
source identity, goal, context, polarity, evidence, and provenance before
selecting it. An open VC, Pending coherence, successful backend, opaque
token, unpublished draft, or mock payload is not an accepted proof.

| Rank | Existing owner / task | Dispatch readiness |
|---|---|---|
| M1.1 Source proof inputs | [checker](./mizar-checker/en/todo.md) 258/272, [core](./mizar-core/en/todo.md) 33-37, [VC](./mizar-vc/en/todo.md) 32, and Step 5B.3 only where the chosen real case needs them | Identify the **smallest missing source-derived producer**; existing bounded theorem/reflexivity routes may already meet portions of this edge. Do not finish entire generalization families first or bypass Task 274 / Gate S1 where required. |
| M1.2 Checkable evidence | [Step 7](#step-7--evidence-pipeline-integration--) item 2 (VC 60 built-ins) **or** item 1 (ATP with Kernel checking) | Only dispatch source slices with authentic formula, VC/goal/context, substitution and provenance. VC 60 is deferred where producers are absent; equality requires its Kernel-instance prerequisites. |
| M1.3 Accepted status/witness | [Step 7](#step-7--evidence-pipeline-integration--) item 3 (Proof 23 / Artifact 25) | Requires a genuinely checked, correctly bound result. A flag/hash/token or untrusted backend result never upgrades status or creates a trusted witness. |
| M1.4 Registered real services | [Step 6](#step-6--phase-output-publication-and-orchestration--) item 1 (Driver, IR, Build and phase producers) | Publish only producer-owned, snapshot-fresh phase outputs. Independent service prerequisites may execute while earlier proof edges are blocked. |
| M1.5 Atomic verified emission | [Step 6](#step-6--phase-output-publication-and-orchestration--) item 3 (Artifact 17, with Proof/Artifact staging) | Keep deferred until real VerifiedArtifact inputs, checked witness staging, manifest/store publication, and validated reader reachability exist. |

**Muse selection procedure (repeat after every dispatched unit):**

1. Inspect the latest `main` roadmap, owner TODO, paired EN/JA contract,
   readiness/dependencies, recent commits, and local orchestration handoff.
   Reconcile in-flight and unpushed work before selecting; never redispatch
   an active unit. This document cannot certify the local worktree.
2. Identify the missing edges for the smallest authentic M1 case. Among
   independent *ready* M1 slices, choose the earliest missing producer by
   the table above; an earlier blocked edge does not block later independent
   service prerequisites. Dispatch one owner contract per run (or a permitted
   three-to-five identical-shape bundle) under the
   [dispatch boundary](./autonomous_crate_development.md#dispatch-boundary).
3. A slice is not ready when its authoritative source/test oracle, concrete
   producer, prerequisite, reviewed contract, or required dependency is
   missing. Preserve `blocked`/`deferred`; report its exact owner. No
   fabricated input, speculative task ID, proof/trace promotion, or weakened
   expectation is a workaround. If a real required task has no owner, create
   its paired contract and crate-plan assignment under normal review **before**
   dispatch, not as an implied permission from this priority table.
4. If all M1 slices are blocked, choose a ready, independent Step 6/7
   producer or real-service-equivalence prerequisite; next choose Step 8
   source-input, editor lifecycle, snapshot or native-diagnostic
   transport work using existing outputs; next other dependency-ready
   tasks. Lane 1 follows this queue's written order without skipping
   units of the [Lane 2 set](#lane-2-set) — the MVM designs, their
   implementation owners and the producers the
   [MVM table](#mvm-readiness-decomposition--) names as prerequisites —
   and never duplicates a unit Lane 2 has in flight or claimed. Lane 2
   prioritizes the set under the MVM Muse rule and takes general-queue
   work only when no set unit is startable; neither lane idles while a
   startable unit exists anywhere. When set membership is unclear, Muse
   classifies the unit under that rule before dispatch. Lane 2 is not
   permission to start blocked execution.
   Never advance Artifact 17, accepted-result consumers, semantic LSP
   metadata or MVM execution ahead of actual producers.
5. After M1 acceptance, prioritize [Step 6C equivalence](./task_contracts/en/STEP6C-REAL-SERVICE-EQUIVALENCE.md),
   Step 7 proof reuse/ATP generalization, and broader source/Core/VC proof
   families. Each retains its own original completion gate; M1 does not
   establish cache-hit/miss or sequential/parallel equivalence.
6. If no legitimate contract is ready, report blockers and the owner decision
   required to Muse. Do not manufacture a next task or claim completion.

**M1 exit:** a positive source-derived theorem accepted by the existing
Kernel/Proof policy, with matching checked witness and source/proof status,
atomically published into a VerifiedArtifact and read back through the real
build path. Tests reject absent/forged evidence, goal/context/polarity
mismatches, stale snapshots, and unsupported/rejected proofs without changing
the trust policy, using a mock producer, or weakening tests. This is a
bounded milestone, not full `proof_verification` coverage, MVM execution,
full cache equivalence, or complete-processor status.

### Adopted frontier producer registration — Waves 1–4 [ ]

Authority: `mizar-frontier-producibility-2026-11-21.md` §§4–5, owner adoption
2026-11-21 12:42 JST (option ①), plus NT6/Vampire decisions below. Registration
adds no completion/acceptance/coverage credit; owner gates and contract freezes apply.
M = state maintenance; A = producer/design. `D/` = `doc/design/`, `C/` = `crates/`,
`TC/` = `doc/design/task_contracts/en/`; paired owners keep EN/JA sync. G judgments
remain external blockers for owner decisions at the scene; language-intent changes stop.
Wave order applies among ready units: P22 waits for P2 despite its listed order.
Lane 1 takes eligible either-lane L2-set work without duplicating Lane 2 claims.
Serialize `control_flow.rs`: P5 → P8 → P9 → P10; `type_checker.rs`: P7/P8/P14.
VC53 owns `C/mizar-vc/src/generator.rs`, `C/mizar-kernel/src/checker.rs`,
`C/mizar-proof/src/{status,policy}.rs` and `C/mizar-test/src/runner/proof_verification/term_proof_tests/source_algorithm_return.rs`.
P3/P15 run at VC53 boundaries; P12/P13 wait for E1; internal design follows existing gates.

#### Wave 1 — ready Lane 1 sequence and Lane 2 gaps

Lane 1: NT6 → Vampire-skip → P1 → P4 → P5 → P7. Lane 2: VC53, urgent P11 then P6 in gaps.

- [ ] **NT6 application — Lane 1; docs-only.** Apply the owner-approved
  2026-11-21 10:31 JST NT6 Class 3 draft unchanged: add the shared-fixture
  head-phase clause to repo `AGENTS.md`, Task Workflow And Delegation,
  immediately after the test-matrix bullet. Dependencies: none; serialize
  with P1 if the application also touches this roadmap.
- [ ] **Vampire-skip — Lane 1; test-only.** Owner decision 2026-11-21
  12:39 JST (option ①): in `C/mizar-atp/src/backend/tests.rs`, affected
  `vampire_extraction_refusal_tests` explicitly report the reason for skipping
  when the provisioned Vampire binary is absent. Provisioned dev-linux must
  execute and pass them. Acceptance: GitHub main CI green at the resulting
  main, and local full `cargo test` shows these tests executed, not skipped,
  and passing. Dependencies: none.
- [x] **P1 — Roadmap state sync; M, light; Lane 1.** Reconcile crate-status
  rows, Step 6.1, Step 7.1/3, Step 8.3/5, algorithm/reflected-syntax stale
  rows and remaining ALGDEF-RESOLVE-CALL-EDGES slices against landed commits.
  Scope: this roadmap, `D/mizar-driver/en/registry.md`,
  `D/mizar-build/en/00.crate_plan.md`, JA companions. Dependencies: sealed
  push (satisfied at adoption); serialize with NT6 if files overlap. No extra credit.
- [x] **P4 — LSP 25 close-out and Doc TODO sync; M; Lane 1.** Run the
  outstanding final T2 read-only completion audit and count the task;
  reconcile landed Doc 5–13 work without credit denied by contracts.
  Scope: `D/mizar-lsp/en/todo.md`, `D/mizar-doc/en/todo.md`; roadmap
  completion row through P1 while both are open. Dependencies: none.
- [x] **P5 — Cycle-safe Core CFG use collection; A; L2-set, either lane.**
  Make `collect_formula_uses`/`collect_term_uses` fail closed on cyclic IDs;
  inventory sibling arena walkers and existing guards. Scope: Core 48,
  `C/mizar-core/src/control_flow.rs`, crate tests,
  `D/mizar-core/en/control_flow.md`. No dependencies; file-disjoint from VC53.
  No formula/general cycle-safety claim until this lands.
- [x] **P7 — Automatic selected algorithm call targets; A; L2-set, either lane.**
  First produce checked-source same-module non-overloaded direct-call node →
  selected `SymbolRef`/original synonym maps, feeding
  `source_definiens_dependencies` and `source_algorithm_recursive_groups`.
  Scope: `C/mizar-checker/src/{type_checker.rs,source_overload.rs}`,
  `mizar-test` consumer, `D/mizar-checker/en/type_checker.md`. Dependencies:
  none; file-disjoint from VC53. Freeze Checker ownership against architecture
  03/overload boundaries; unresolved ownership parks at G19.
  [Contract](task_contracts/en/CHECKER-P7-SELECTED-CALLS.md).
- [ ] **P11 — Accepted-definition design/contract freeze; A; urgent Lane 2 gap.**
  Freeze only the bounded direct-return obligation set (E1 typing,
  single-valuedness, inhabitation; E2 postcondition; E4), same-identity
  source/check/Core/snapshot/policy binding, atomic adoption, refusal matrix
  and R3 eligibility input. Scope: new paired acceptance contract and
  `D/mizar-core/en/mvm_correspondence.md`/JA; avoid `status.md`.
  Dependencies: none to start; mandatory T2 for the acceptance boundary.
  Park Deferred termination (G10); it gates P13. No implementation in this unit.
- [ ] **P6 — Kernel soundness-argument revisit; A; Lane 2 gap (either lane).**
  Document premises, caller obligations and rejection corpus for
  `check_sole_return_var`, `check_instantiated_algorithm_returns`,
  `check_algorithm_returns_existential`, `check_object_inhabitation_witness`
  and E1 typing/single-valuedness rules as they land; assess generalization
  limits. Scope: `D/mizar-kernel/en/{soundness_argument,todo}.md`, JA pairs;
  Kernel source read-only. Dependencies: none to start; close before P13.
  Docs-only; code findings become separate units (precedent: Kernel 35).

#### Wave 2 — source/Core roots and Step 7 preparation

Lane 1: P28, P20, P21, P22, P2 (dependencies first); either lane: P8 → P9, P14.

- [ ] **P28 — Checker 280 readiness/definiens cycles; A; Lane 1.** Correct
  stale readiness, then detect direct/mutual `func` definiens cycles.
  Scope: `TC/ALGDEF-CHECKER280-DEFINITION-CYCLES.md`, `C/mizar-checker/src/`,
  real `tests/miz/fail/` sources. Dependencies: none for definiens-only
  (study inference); algorithm edges wait for P7.
- [ ] **P20 — Core 33 real MT10-CIR-TE baseline; A; Lane 1.** Install the
  first genuine source-derived context/items/binders baseline from the nine
  zero-credit associations, opening Core 34–39/VC 32–39/general services.
  Scope: `C/mizar-core/src/elaborator.rs`, `mizar-test` runner,
  `D/mizar-core/en/todo.md`. Dependencies: none; Checker 248 is done.
- [ ] **P21 — Checker umbrella re-inventory; A; Lane 1.** For 257C/258/272/277B,
  freeze a bounded next slice or close with explicit residuals when children
  are done. Scope: `D/mizar-checker/en/{todo,payload_family_decomposition}.md`,
  new paired contracts. Dependencies: none; docs first, implementation follows.
- [ ] **P22 — Driver guarded Vampire BackendRun; A; Lane 1.** Register a
  genuine scheduled `run_vampire_backend` service feeding existing
  Kernel-checked candidates. Scope: `C/mizar-driver/src/registry.rs`,
  `frontend_adapter/`, `driver/scheduler.rs`, tests guarded on
  `.tools/provers/vampire-4.9/vampire`, `D/mizar-driver/en/registry.md`.
  Dependencies: P2; freeze confirms DR-1 covers a Driver-scheduled external process.
- [ ] **P2 — Step 7 owner-document sync; M; Lane 1.** Record implemented
  guarded Vampire and bounded Proof 23 (task 75/composition), retaining
  Proof 23/Artifact 25 residuals. Scope: `TC/ATPK-BACKEND-EXTRACTION.md`
  status and `D/mizar-{atp,proof,artifact}/en/todo.md`, JA pairs. Dependencies: none.
- [ ] **P8 — Core 43 if/path-condition and bare-fail shells; A; L2-set, either lane.**
  Lower source `if`/bare `fail` into Core/CFG without continuation or
  postcondition after fail; add Core 48 internal return-or-fail checking,
  no public E0501. Scope: checker `type_checker.rs`,
  `C/mizar-core/src/{core_ir.rs,elaborator.rs,control_flow.rs}`, test-first
  `.miz`, Core elaborator/control-flow docs. Dependencies: P5; serialize
  with P7 on checker writes. Freeze may split independent if/fail routes.
- [ ] **P9 — Real ControlFlowIr snapshot baseline; A; L2-set, either lane.**
  Add `SnapshotKind::ControlFlowIr` and real `idalgo` MT10-CFG-PV baseline.
  Scope: `C/mizar-test/src/snapshot.rs`, Core `control_flow.rs`, `tests/snapshots/`.
  Dependencies: P5; file order P8 → P9 → P10.
- [x] **P14 — Typed ground invocation; A; Lane 2, either-lane scheduling.**
  Same-module `idalgo(0)` lookup and typed numeral invocation produce
  authenticated target identity, ordered actuals and source/context provenance
  for R4's loader join. Scope: checker `type_checker.rs`, test-first
  `tests/miz/`, checker type-checker docs. Dependencies: P7; serialize P8/P14.
  [P14 contract](task_contracts/en/CHECKER-P14-GROUND-INVOCATION.md); P14-NUMERAL admits initially object-typed numerals to object arguments.

#### Wave 3 — boundary work and independent consumers

Lane 2 boundaries: P3, P15, P16, P17. Lane 1: P23, P24, P26, P27, P25 definition.
Either lane: P10, P19. P16/P17 must be ready before P13 completes; G9 still applies.

- [ ] **P3 — Core/VC/Checker state sync; M; L2-set, Lane 2 boundary.**
  Reconcile Core 42–53/VC 43/54 EN/JA boxes, VC 32/39/47/50 active slices,
  VC53 stale blocked line, checker umbrellas and T1 F1 implemented policy
  consumption deferrals. Scope: relevant TODOs,
  `D/mizar-kernel/en/checker.md`, `D/mizar-vc/en/generator.md`, JA pairs.
  Dependencies: VC53 unit boundary; F1 may join the next VC53 freeze.
- [ ] **P15 — R6-A/B local closure/classification; A; Lane 2 boundary.**
  Build same-package evaluated-call closure and value/procedure, ghost Pick,
  set/sequence loop and resolved-ensures-result classifications. No R6-C goals
  before R3. Scope: VC `generator.rs` or bounded module,
  `TC/ALGDEF-VC58-COMPUTATION-UNIQUENESS.md` refreeze,
  `D/mizar-vc/en/mvm_computation_admission.md`. Dependencies: P7, P10 for loops.
- [ ] **P16 — R3 freeze-preparation dossier; A; Lane 2, either lane.**
  Options/consequences/recommendation for R3-Q1/Q3/N3 and R4/R6/R7;
  collect Q2/N1/N2 facts without answering owner questions. Scope:
  `D/mizar-artifact/en/mvm_runtime_export.md`, JA pair; docs only.
  Dependencies: G9 mechanism permission; owner retains answers.
- [ ] **P17 — R5-Q1 accounting/report dossier; A; Lane 2.** Prepare options,
  consequences and recommendation for step/timeout/nest accounting and report
  framing before R4 pilot. Scope: `D/mizar-vc/en/mvm_executor.md`, JA pair.
  Dependencies: G9; owner retains decisions.
- [ ] **P23 — Task-29 rejecting ATP fixtures; A; Lane 1; tests only.** Drive
  Kernel `fail_certificate_*` cases through genuine ATP candidates for ATP-G-029.
  Scope: `C/mizar-atp/src/translator/tests.rs` or `mizar-test`; no dependencies (study inference).
- [ ] **P24 — Generic/state built-in witness publication; A; Lane 1.** Extend
  bounded theorem-reflexivity acceptance to the profiles retaining publication
  gaps. Scope: `C/mizar-proof/src/witness_store.rs`,
  `C/mizar-artifact/src/proof_witness.rs`, `PROOF-DISCHARGE-ACCEPTANCE` successor; depends on P2.
- [ ] **P26 — LSP navigation definitions; A; Lane 1.** Go-to-definition using
  existing `ModuleSummary`/`PublishedImportView`, with an LSP-NAVIGATION-TOKENS
  style contract. Scope: `C/mizar-lsp/src/{server.rs,metadata.rs}`.
  Dependencies: none; one/two slices, reference index remains later.
- [ ] **P27 — E0225 descriptor; A, light; Lane 1.** Register the spec-allocated
  import-cycle descriptor following AST-DIAGNOSTICS precedent; emission waits
  for complete import graph. Scope: `C/mizar-diagnostics/src/registry/builtin.rs`,
  `D/mizar-diagnostics/en/`. Dependencies: none.
- [ ] **P25 — Native runtime-context definition/staged producer; A; Lane 1.**
  Define toolchain identity, verifier-config hash (`workspace_config_content_hash`),
  default LSP profile and workspace-only dependency inputs in
  `D/mizar-driver/en/` and `D/mizar-build/en/planner.md`. Dependencies: none
  for definition; G14 approval between definition and code. Wave 4 then stages
  the producer (`C/mizar-driver/src/request.rs` or `mizar-build`) and native
  host (`C/mizar-lsp/src/main.rs`, `tests/stdio.rs`), about four units total;
  supports build bootstrap/BUILD-COMPUTATION-POLICY. No early host activation.
- [ ] **P10 — Core 44 range/collection-loop shell; A; L2-set, either lane.**
  Source-derived loop kind with hidden bound/step inputs for R6-B set/sequence
  classification and VC58 loop sites. Scope: P8 files plus real fixture.
  Dependencies: P8; serialize after P9 on `control_flow.rs`.
- [ ] **P19 — MBP-02 Binder resolution remainder; A; L2-set, either lane.**
  First map completed Checker 257B to the plan; then head/body candidates,
  outer declaration/type-source correspondence, checker mode selection before
  head commitment, captures, MB4/6/8 scope controls and E0206 producer.
  Scope: `C/mizar-resolve/src/names.rs`, `C/mizar-checker/src/binding_env.rs`,
  `TC/AST-MULTI-BINDER-PLAN.md`, owner sections. Dependencies: readiness review;
  usefulness limited until G8 (MBP-04 also requires MBP-03).

#### Wave 4 — accepted definition and downstream admission

Lane 2: P12 → P13 after VC53 E1, then P18. Lane 1: existing Core 34+, Core 46 call metadata, Resolver 33 and P25 producer/host under owner freezes/prerequisites.

- [ ] **P12 — E2 same-identity direct-return adoption; A; Lane 2.** Adopt
  regenerated graph's retained postcondition under original identity/current
  policy, following NativeE4NormalReturn; older substituted reflexivity is
  insufficient. Scope: Proof `status.rs`, `source_algorithm_return.rs` tests,
  `D/mizar-proof/en/status.md`. Dependencies: P11; serialize with VC53.
- [ ] **P13 — Accepted-definition aggregate; A; Lane 2.** One atomic
  Proof-owned decision over five adoptions for the identical immutable
  definition, only as R3 eligibility input/VC59's first producer; no status
  promotion, symbol introduction or publication. Scope: Proof `status.rs`,
  tests, Proof status/Core correspondence docs. Dependencies: VC53 E1, P12, P6, G10.
- [ ] **P18 — R6 admission-binding dossier; A; Lane 2.** Prepare options,
  consequences and recommendation for R6-Q1/Q2 closure authentication and
  per-VC/exemption binding to request/contract/closure/policy/version.
  Scope: `D/mizar-vc/en/mvm_computation_admission.md`, JA pair; Proof `policy.md`
  reference only. Dependencies: G9; useful after P15. Owner retains decisions.

### Velocity gate — November 2026 [x]

Selected before any further per-element source-transport slice. Tasks 1-2
change how the remaining Task 257-279 families are produced and tested,
not what they mean; the gate closes when both are done. Tasks 3-4 were split
out of task 2 on 2026-11-04 and follow it under the same ledger discipline.

1. [x] [Large file split](./task_contracts/en/VELOCITY-LARGE-FILE-SPLIT.md)
   — mechanical test-module split of the Rust files over 20k lines; light
   tier.
2. [x] [Generic original-source transport](./task_contracts/en/VELOCITY-GENERIC-SOURCE-TRANSPORT.md)
   — one AST-walking producer and one shape oracle replace per-element
   transport slices and literal-offset profiles; subsumes checker Task 281.
3. [x] [Element-rule transport](./task_contracts/en/VELOCITY-ELEMENT-RULE-TRANSPORT.md)
   — the class `b` element rules of Tasks 257, 269-272 and 279 as one
   family contract over the generic walk (checker Task 285).
4. [x] [Proof-local authenticated intake](./task_contracts/en/CHECKER-PROOF-LOCAL-AUTHENTICATED-INTAKE.md)
   — one authenticated receipt replaces the thirty canonical literal guards
   of the public proof-local build inputs; full tier (checker Task 286).

### Step 1 — Soundness contract closure [x]

Close the kernel-audit F1/F2/F7/F8 findings on the trusted boundary before any
further evidence-pipeline work. No external gaps block this step; it removes
certified-unsound acceptance paths.

1. [x] [mizar-test task 22](./mizar-test/en/todo.md) — certificate-corpus root
   naming reconciliation (F8; docs-only, no deps).
2. [x] [mizar-vc task 27](./mizar-vc/en/todo.md) — explicit goal polarity in
   the kernel-evidence handoff (F1 producer side).
3. [x] [mizar-kernel task 30](./mizar-kernel/en/todo.md) — goal-polarity
   binding in the check service (F1, invariant B4).
4. [x] [mizar-vc task 28](./mizar-vc/en/todo.md) — context-identity payload
   for non-imported source bindings (F2 producer side).
5. [x] [mizar-kernel task 31](./mizar-kernel/en/todo.md) — context-identity
   verification (F2, paired with vc 28).
6. [x] [mizar-test task 21](./mizar-test/en/todo.md) — corrected-path
   soundness vocabulary in the required-case registry (F7).

Exit: audit F1/F2/F7/F8 closed; `soundness_argument.md` invariants B4 and the
P-class rows are marked implemented/resolved; the 23-case certificate corpus
stays rejecting.

### Step 2 — Spec-decision wave [x]

Settle the audited semantic decisions as docs-plus-corpus work before any
checker/core implementation that would preempt them. Per AGENTS.md these are
specification-intent changes: update `doc/spec/en/` and `ja/` together. No
checker code semantics change in this step.

1. [x] [mizar-checker task 35](./mizar-checker/en/todo.md) — constructor
   property arguments vs extensionality (SSA-001, critical).
2. [x] [mizar-checker task 36](./mizar-checker/en/todo.md) — structure member
   identity, upcast paths, acyclicity (SSA-002/011/012).
3. [x] [mizar-checker task 37](./mizar-checker/en/todo.md) — overload
   tie-break and tie ambiguity (SSA-003/010/016/019), coordinated with:
4. [x] [mizar-core task 26](./mizar-core/en/todo.md) — template argument
   inference determinism (template-audit F7).
5. [x] [mizar-checker task 38](./mizar-checker/en/todo.md) — functorial
   cluster `for T` semantics (SSA-004).
6. [x] [mizar-checker task 39](./mizar-checker/en/todo.md) —
   property-implementation coherence (SSA-005).
7. [x] [mizar-checker task 40](./mizar-checker/en/todo.md) — registration
   activation timing contract (SSA-006).
8. [x] [mizar-checker task 41](./mizar-checker/en/todo.md) — closure
   termination, contradiction site, `attr(args)` (SSA-007/008/020).
9. [x] [mizar-checker task 42](./mizar-checker/en/todo.md) — reduction
   determinism signature (SSA-009).
10. [x] [mizar-checker task 43](./mizar-checker/en/todo.md) — sethood for
    dependent modes and built-in inhabitation (SSA-013/014).
11. [x] [mizar-checker task 44](./mizar-checker/en/todo.md) — `reconsider`
    discharge and ambiguous redefinition target (SSA-015/017).

Exit: every SSA decision recorded in bilingual spec text with its rejection
corpus seeds; checker/core implementation tasks in step 4 are unblocked.

### Step 3 — Kernel-contract completion and audit-consumer alignment [x]

Finish the remaining kernel-audit producer/consumer follow-ups so the July
audit inventory is closed except the parked and externally paced rows. Depends
on step 1.

1. [x] [mizar-vc task 29](./mizar-vc/en/todo.md) — imported-statement
   projection, producer side (F6), paired with:
2. [x] [mizar-kernel task 33](./mizar-kernel/en/todo.md) — imported-statement
   projection specification and validation (F6).
3. [x] [mizar-kernel task 34](./mizar-kernel/en/todo.md) — legacy
   tautology-marker semantics (F9, low).
4. [x] [mizar-proof task 21](./mizar-proof/en/todo.md) — policy alignment
   with the corrected kernel rejection taxonomy (F1/F2).
5. [x] [mizar-cache task 24](./mizar-cache/en/todo.md) — proof-reuse identity
   covers the extended kernel-evidence contract (F1/F2; needs proof 21).
6. [x] [mizar-artifact task 24](./mizar-artifact/en/todo.md) — proof-witness
   schema re-check against the audit follow-ups.
7. [x] [mizar-atp task 29](./mizar-atp/en/todo.md) — candidate-evidence
   conformance to the post-audit kernel contract (F1/F2/F6; needs kernel 33).
   The crate-owned F1/F2/F6 consumer regressions are complete; the joint
   source-derived kernel-corpus-through-ATP-candidate execution remains
   classified as an external/deferred extraction gap rather than mocked.

Exit: every July kernel-audit finding is implemented, parked with a recorded
trigger (kernel 32), or resolved by the Step 4 kernel re-audit (kernel 35).

### Step 4 — Checker/core audit implementation [x]

Implement the decisions from step 2 in checker and core. Depends on step 2.

1. [x] [mizar-checker task 45](./mizar-checker/en/todo.md) — overload
   tie-break and redefinition-target implementation (deps: tasks 37, 44).
2. [x] [mizar-checker task 46](./mizar-checker/en/todo.md) — closure
   contradiction and termination rules (deps: tasks 41-42).
3. [x] [mizar-checker task 47](./mizar-checker/en/todo.md) — existential and
   omitted-reconsider gates plus activation contract (deps: tasks 40, 43, 44).
4. [x] [mizar-core task 27](./mizar-core/en/todo.md) — reduct/view lowering
   (template-audit F1/F3; deps: checker 36).
5. [x] [mizar-core task 28](./mizar-core/en/todo.md) — template type-actual
   inhabitation gating (F2; deps: core 27, checker 43).
6. [x] [mizar-core task 29](./mizar-core/en/todo.md) — scheme-actual
   compatibility, guard obligations, functor-actual validation (F4/F6/F8).
7. [x] [mizar-core task 30](./mizar-core/en/todo.md) — sethood plumbing for
   type parameters (F5; deps: core 28, checker 43).
8. [x] [mizar-kernel task 35](./mizar-kernel/en/todo.md) — soundness-argument
   revisit for the reduct-view encoding (deps: core 27).

Exit: audited semantic corrections are implemented with their rejection
corpora; the kernel soundness argument is re-checked against view terms.

### Temporary gate — checker/test design-evidence consolidation [x]

Before selecting another Step 5 semantic task, complete the sequence below in
one continuous autonomous thread. This gate does not reopen or reorder already
completed Step 5 work. Each logical documentation task still receives its own
paired contract, independent reviews, verification, task-only commit, and
clean post-commit inventory. It must pass all nine protocol hard gates and a
valid read-only quality evaluation of at least `90/100` before completion.

This gate is limited to duplicated historical evidence in the derived
`mizar-checker` / `mizar-test` `doc/design` documents and to the paired task
contracts, crate-plan indexes, source inventories, and legacy-compaction ledger
that own those migrations. A separately frozen schema prerequisite may also
update only the generic non-production lint-policy consumer and its same-test
validation; ordinary lifecycle repairs and compaction batches may not. The gate
must not consolidate or edit `doc/spec`, existing `.miz` files, expectations,
trace rows or status, production Rust, diagnostics, active behavior, test
intent, or semantic/coverage credit.

At the checkpoint that established this gate, HEAD was
`7b53784a6f2525ebb35ce8d59230f07d1c9041bf`, the worktree and
`origin/main...HEAD` were clean/`0/0`, protected stash
`f65cf4a13752ec380710814a9ac6392ccb9d75d4` was unchanged, and the schema-2
ledger registered 32 batches, 44 canonical task rows, and 4 `task_ref` rows.
Those values prove only the already registered migrations; they are not a
repository-wide or program-wide consolidation-completion claim.

1. [x] **Audit lifecycle and commit state before deleting more evidence.**
   Read-only reconcile every registered batch's EN/JA contracts, ledger rows,
   links, current lifecycle fields, and reachable prerequisite and migration
   commits located from task-local evidence, ledger/source diffs, and Git
   history. The initial inventory indicates that all 32 registered EN/JA batch
   pairs retain stale top-level status wording; verify that result rather than
   assuming it.
   Classify stale live lifecycle wording as `design_drift`. Report a genuine
   hash or repository-state disagreement as `repo_metadata_conflict` and do
   not repair it automatically.
2. [x] **Repair uniquely attributable lifecycle drift as a separate task.**
   Freeze a paired repair contract and update only fields that own current
   lifecycle state. Preserve historical precommit/postcommit checkpoint prose,
   unique evidence, owner links, and completed migration boundaries. Complete
   reviews, verification, a task-only commit, and clean post-commit proof
   before selecting a compaction batch.
3. [x] **Run the schema-2-safe whole-section compaction wave.** After every
   clean commit, fresh-inventory the remaining duplicates and select exactly
   one dependency-ready coherent family. Freeze its exact old-section-to-owner
   map, source/heading/anchor/count/hash baselines, language-local redirects,
   protected surfaces, and forbidden boundaries in a paired batch contract and
   source inventory. Repeat evidence-equivalence, EN/JA, boundary,
   test-sufficiency, source/documentation, and final-quality reviews to **NO
   FINDINGS**; pass the generic ledger/link/fragment lint, all nine protocol
   hard gates, and a valid quality evaluation of at least `90/100`; then make a
   task-only commit and clean post-commit inventory.
   Continue without stopping while the fresh inventory exposes another safe
   dependency-ready family.
4. [x] **Keep schema-2-inexpressible evidence intact.** Do not force a second
   same-task section from one source file, a mixed owner-local section, or a
   paragraph-only duplicate into the whole-section ledger. Classify every such
   residual by owner and blocking shape. Start a separate schema/ownership
   prerequisite only when its authority, preservation oracle, consumer, and
   validation are unique; otherwise record the exact blocker and leave the
   evidence unchanged.
5. [x] **Close only the schema-2-safe compaction wave.** When fresh inventory
   finds no further safe batch or prerequisite, record the final registered
   batch/task/reference totals, all intentionally retained residual classes,
   lint and hard-gate evidence, clean worktree/origin state, and unchanged
   protected stash. This closeout means that no further currently authorized
   schema-2-safe migration is ready; it must not claim that all historical
   duplication in the repository has been removed.
6. [x] **Resume semantic readiness from the C4C4 postcommit proof.** Re-run the
   authority-order inventory rather than carrying forward a presumed successor
   ID, API, or oracle. Any Task-255/source-set capture or generated-core bridge
   requires its own paired contract and uniquely owned dependencies before
   implementation. Task 277B remains not-ready and zero-credit until its
   recorded dependencies are complete.

Exit: the schema-2-safe wave has the bounded closeout above, every protected
semantic and coverage surface is unchanged, and the clean post-closeout
inventory can select the next Step 5 task without competing documentation
owners.

### Step 5 — Source-derived semantic bridge [x]

Widen real `.miz` source-derived payload extraction, elaboration, and
verification beyond the active reserve/local-mode/formula slices until the
bridge is grammar-complete over `doc/spec/en/` chapters 3-20.

**Alpha completion boundary.** Step 5 is complete when every audit-1
requirement family in the September 2026 oracle corpus map
([semantic_bridge_corpus_map.md](./mizar-test/en/semantic_bridge_corpus_map.md))
reports active coverage — grammar-complete extraction, elaboration, and
verification. Algorithm *execution* (MVM, spec chapter 20 sections
20.9-20.10) stays excluded and parked (see
[Parked and trigger-based work](#parked-and-trigger-based-work)).
This boundary is limited to the mapped corpus and its expected phases; Open
VCs and Pending coherence retain their existing acceptance boundaries.
Pre-audit 5B.3 and broader producer tasks remain with their existing owners
and dependency gates beyond this alpha corpus boundary.

**Binding inputs.** The corpus map above, the frontend gap inventory
([semantic_bridge_frontend_gaps.md](./mizar-test/en/semantic_bridge_frontend_gaps.md)
plus [`tests/coverage/audit1_frontend_gaps.tsv`](../../tests/coverage/audit1_frontend_gaps.tsv)),
the audit-1 requirement rows in
[`tests/coverage/spec_trace.toml`](../../tests/coverage/spec_trace.toml), and
the activation-map ledger
[`tests/coverage/step5_activation_map.tsv`](../../tests/coverage/step5_activation_map.tsv)
binding each of the 120 oracle pairs to its owner task below. Tests
are binding per the AGENTS.md authority order: no oracle case may be
activated, and no gap closed, by matching expectations to current behavior.
The 5A.9 smoke guard remains the owner of the eventual all-source parse claim;
5A.2 closure makes read-only inventory safe without claiming every source is
parse-clean. Step 5A.1's completion evidence is owned by its
[task contract](./task_contracts/en/STEP5A1-G5-QUA-STRUCTURE.md).

**Completed micro-task record.** The 2026 micro-slice narrative for
completed tasks 16-264/269\*/C4C\* that previously filled this section is a
frozen log, moved verbatim to
[archive/step5_microtask_narrative.md](./archive/step5_microtask_narrative.md).
Concise per-crate sequencing status stays in the crate TODOs; the Step 5
task addenda later in this file remain frozen logs pending the audit-2
compaction inventory
([documentation_compaction_rules.md](./documentation_compaction_rules.md)).
This revision reopens or reorders no completed work, changes no active
case, expectation, or trace status, and grants no new coverage credit.

**Decomposition rule (September 2026 audit 2).** The former slice
enumeration multiplied four dimensions — declaration kind x RHS form x
import status x chain depth — into one micro-task per point (~550-750 doc
lines per task). The audit-2 classification, constrained by the audit-1
corpus:

- *Structural dimensions* — each is owned by one AST-bounded structural
  rule (task-74 precedent) with zero new semantic credit: chain depth
  (the task-74 producer rule), builtin terminal duality (`set`/`object`
  radix as a parameter), consumer formula shape over already-normalized
  types (equality, pre-desugaring inequality, right-expected membership,
  normalized-reflexive type assertion, same-symbol asserted head), and
  reserve multiplicity (one or two bindings, shared or distinct written
  ranges). No further per-depth, per-terminal, per-formula-shape, or
  per-multiplicity sibling task may be created; task 5B.1 retires the
  pattern.
- *Semantic dimensions* — stay fine-grained as the bounded owner tasks in
  Step 5C, bound to audit-1 requirement families: import status
  (imported-module AST extraction and provenance), declaration acceptance
  and correctness conditions, attribute/cluster/registration semantics,
  structure evidence, truth/facts/justifications and proof payloads,
  overload resolution, templates, and algorithm verification.

Each 5A/5B/5C entry is an owner task: it receives a paired task contract
and a crate-TODO row when started, per
[autonomous_crate_development.md](./autonomous_crate_development.md). Gate
tier is marked per that protocol's Gate Tiering section; unmarked tasks
are full-gate.

#### Step 5A — Frontend gap closure [x]

Close the audit-1 frontend gaps in the frozen order below. A gap-closure
task grants no semantic oracle credit itself: it turns blocked committed
sources into parse-clean syntax regression material and unblocks the 5C
activation targets recorded per case in the activation-map ledger. The 29
blocked sources are the immediate regression corpus; their expectations
stay binding and must not be weakened.

1. [x] **5A.1 (G5, critical crash)** — `mizar-syntax`: fixed the
   SurfaceAstBuilder panic (`node cannot be shared by multiple non-root
   parents`) on `term qua <structure type>`; `qua` to builtin types
   already parses. Minimal reproducer in the gap inventory. Unblocks
   `fail_type_elaboration_term_qua_invalid_narrowing_001` and (with 5A.5)
   `pass_type_elaboration_synonym_functor_001`. Deps: none. Full gates
   (crash on the syntax trust path). Evidence:
   [STEP5A1-G5-QUA-STRUCTURE](./task_contracts/en/STEP5A1-G5-QUA-STRUCTURE.md).
2. [x] **5A.2 (G1, high)** — `mizar-frontend` with `mizar-lexer`:
   closed the G1 component in the immutable ledger's 20 rows (17 G1-only,
   two G1+G2, one G1+G6). Mixed-row residuals and semantic activation stay
   with their named owners. Evidence:
   [STEP5A2-G1-LOCAL-NOTATION](./task_contracts/en/STEP5A2-G1-LOCAL-NOTATION.md).
3. [x] **5A.3 (G2)** — `mizar-lexer`: closed declaration-site tokenization
   for the immutable ledger's three symbolic user-symbol rows without semantic
   activation. Evidence: [STEP5A3-G2-SYMBOLIC-USER-SYMBOLS](./task_contracts/en/STEP5A3-G2-SYMBOLIC-USER-SYMBOLS.md).
4. [x] **5A.4 (G3)** — `mizar-parser`: accepts omitted-justification
   compact statements beneath `[then] linkable_statement` without semantic
   activation. Evidence: [STEP5A4-G3-THEN-LINKING](./task_contracts/en/STEP5A4-G3-THEN-LINKING.md).
5. [x] **5A.5 (G4)** — `mizar-lexer`, with parser/frontend consumers:
   activates local `synonym`/`antonym` notation spellings after the declaring
   item without semantic activation. Root-cause ownership and evidence:
   [STEP5A5-G4-NOTATION-ALIASES](./task_contracts/en/STEP5A5-G4-NOTATION-ALIASES.md).
   Deps: completed 5A.3 for symbolic-spelling cases.
6. [x] **5A.6 (G6)** — `mizar-parser`/`mizar-frontend`: accept
   argument-bearing local dependent-mode use (`QMode of A`) in
   binder/type positions; the declaration already parses, while task 68 owns
   the same-module reserve extraction boundary and imported provenance remains
   separately owned. Frozen scope:
   [STEP5A6-G6-DEPENDENT-MODE-USE](./task_contracts/en/STEP5A6-G6-DEPENDENT-MODE-USE.md).
   Deps: completed 5A.5 by frozen order.
7. [x] **5A.7 (G9)** — `mizar-parser`: dedicated regression evidence now
   covers already-active notation spellings; production behavior was already
   closed by 5A.2/5A.5. Evidence:
   [STEP5A7-G9-ACTIVE-PATTERN-SPELLINGS](./task_contracts/en/STEP5A7-G9-ACTIVE-PATTERN-SPELLINGS.md).
8. [x] **5A.8 (G7)** — human-approved decision keeps empty justification
   syntax while preserving all proof obligations; the paired specification
   clarification and bounded parser/cache/test scope are frozen in
   [STEP5A8-G7-EMPTY-JUSTIFICATIONS](./task_contracts/en/STEP5A8-G7-EMPTY-JUSTIFICATIONS.md).
9. [x] **5A.9** — `mizar-test`: the committed corpus-wide syntax smoke
   guard checks every non-`parse_only` source and admits only the seven exact
   fail-closed ledger rows. Evidence and deferrals:
   [STEP5A9-CORPUS-SYNTAX-SMOKE](./task_contracts/en/STEP5A9-CORPUS-SYNTAX-SMOKE.md).

#### Step 5B — Consolidation and pending prerequisites [~]

1. [x] **5B.1** — `mizar-checker`/`mizar-test`: the task-74 structural
   product now has one live owner; completed point matrices are regression-only
   guards and create no new task. Light-tier evidence and unchanged protected
   boundaries: [STEP5B1-STRUCTURAL-RULE-CONSOLIDATION](./task_contracts/en/STEP5B1-STRUCTURAL-RULE-CONSOLIDATION.md).
2. [x] **5B.2** — complete item 6 of the temporary checker/test
   design-evidence consolidation gate above (semantic readiness
   re-inventory from the C4C4 postcommit proof). It must finish before
   the first 5C task is selected. The fresh result and next owner are frozen in
   [STEP5B2-C4C4-READINESS-INVENTORY](./task_contracts/en/STEP5B2-C4C4-READINESS-INVENTORY.md).
3. [ ] **5B.3** — the Task-265 execution-authority list below is retained
   unchanged; its item 13 (checker task 49 fixture activation) is the
   sole open pre-audit activation task and is independent of the audit-1
   corpus (it activates the pre-audit 24-fixture set only).

This activation is dependency-gated, not a global stop on M1 or independent
Steps 6-8. Do not infer accepted-status or scheme/theorem roles; resume
Task 49 only when the named prerequisites are genuinely satisfied.

#### Task-265 execution authority (pre-audit vertical slice) [~]


Task 265 replaces the former open-ended promotion rule with the following
initial execution authority and owner-owned decomposition gates. The order
below is authoritative for the shared contradiction-to-VC vertical slice.
Its complete downstream dependency graph is Task 266 -> Task 267 -> Task 268,
Tasks 266 + 268 -> Core Task 31, checker Task 247 -> Core Task 32, and Core
Tasks 31 + 32 -> VC Task 30 -> VC Task 31.
The parser and resolver tasks are independently authorized Task-49
prerequisites; they do not block Task 266. No row authorizes a producer to
reconstruct raw syntax owned by another crate or to fabricate truth, facts,
proof acceptance, terminal goals, Core/VC payloads, or runner success.

1. [x] [mizar-test task 265](./mizar-test/en/todo.md) — docs/traceability-only
   STEP 5 execution-authority decomposition. It assigns the tasks and gates in
   this list without changing source, specification semantics, fixtures,
   expectations, trace status, or coverage credit.
2. [x] [mizar-test task 266](./mizar-test/en/todo.md) — extend the checker-owned
   syntax-free `ResolvedTypedAst` final projection for only the existing Task
   180 standalone `contradiction` theorem. Preserve one resolver theorem owner
   linked to the existing one checked `FormulaKind::Contradiction` result,
   including source ranges and provenance. Do not publish truth/facts, accept
   the theorem, create a proof/terminal goal, or lower Core/VC data.
3. [x] [mizar-test task 267](./mizar-test/en/todo.md) — docs-only checker/core
   contract decision for the omitted-justification theorem form. Choose the
   checker-owned pending-auto-proof status, proof-skeleton and terminal-goal
   payload, and exact core mapping without accepting or discharging the
   theorem.
4. [x] [mizar-test task 268](./mizar-test/en/todo.md) — implement only the
   Task-267 accepted checker-owned proof/terminal-goal producer contract for
   the exact Task-180 source. It must fail closed on missing, duplicate,
   reordered, or mismatched owner/formula/proof identities.
5. [x] [mizar-core task 31](./mizar-core/en/todo.md) — after Tasks 266 and 268,
   lower only their real final checker payload into source-derived `CoreIr`
   and the exact theorem obligation representation selected by Task 267,
   paired with mizar-test Task-10 snapshot consumption. Core must not infer a
   proof state or terminal goal from source text.
6. [x] [mizar-checker task 247](./mizar-checker/en/todo.md) — docs-only
   exhaustive decomposition of the remaining AST-wide declaration,
   attribute, term, formula, proof-skeleton, registration/trace/overload, and
   Task-49 source payload families into bounded producer tasks with prepared
   mizar-test Task-10 consumers. Complete: checker Tasks 248-264/269-279,
   `MT10-FS`/`MT10-AS`, the exact 24-fixture reconciliation mapping (resolver
   Task 31 activates one, Task 49 activates 23 and deduplicates all 24), and
   explicit blocked accepted-status and external scheme/theorem-role Gate S1
   are canonical in the paired payload-family decomposition.
7. [x] [mizar-core task 32](./mizar-core/en/todo.md) — completed docs-only
   exhaustive decomposition of every remaining source-derived
   `CoreIr`/`ControlFlowIr` family into Core Tasks 33-53 and prepared Task-10
   consumers `MT10-CIR-TE`/`FS`/`AS`/`ALG`/`MT10-CFG-PV`. Gates A1/S1,
   artifact/public-code ownership, VC-owned concrete call/result substitution,
   and no-synthetic/no-credit boundaries remain explicit.
8. [x] [mizar-vc task 30](./mizar-vc/en/todo.md) — completed docs-only exact
   contradiction mapping and exhaustive source-derived VC decomposition.
   `source_vc_decomposition.md` assigns VC Tasks 31-55, reserves
   `MT10-VC-T180` solely for VC 31, defines shared `MT10-VC-PV/VC<n>` slices,
   and preserves Core 33-53, VC 40's completed-VC37/39-plus-Core40/A1
   dependency, VC 53's bounded missing canonical evidence-transport authority,
   and Gate S1 for missing roles outside direct VC 41 without adding source,
   fixtures, expectations, trace status/tests, or coverage.
9. [x] [mizar-vc task 31](./mizar-vc/en/todo.md) — implement only the exact
   structural Task-30 mapping for the Task-180 vertical slice and
   `MT10-VC-T180`: validate the direct terminal relation and produce one open,
   unaccepted `TerminalProofGoal` with a full VcIr baseline. Do not inject a
   marker, reclassify the existing type-elaboration case, or imply discharge,
   ATP/kernel execution, proof verification, or acceptance.
   Complete: the exact marker-free adapter, first real proof-verification
   runner/guard, distinct source/sidecar, full VcIr baseline, and one covered
   trace row land together; broader VC families remain unpromoted.
10. [x] [mizar-parser task 47](./mizar-parser/en/todo.md) — aligned omitted,
    explicit-`by`, and proof-block `reconsider` syntax with the canonical
    Chapters 4/8/15 and Appendix-A contract. The real parse-only runner covers
    both formerly deferred exact rows without changing semantic intent.
11. [x] [mizar-parser task 48](./mizar-parser/en/todo.md) — implemented the
    exact top-level Chapter-7 `property_impl` grammar, append-only typed syntax,
    bounded recovery, and active parse-only pass/fail corpus needed by the
    still-inactive Task-39 coherence seed. This grants syntax-only credit.
12. [x] [mizar-resolve task 31](./mizar-resolve/en/todo.md) — expose the
    same-signature/same-return declaration conflict required by the deferred
    Task-37 seed, without performing checker overload selection. Complete: the
    exact internal diagnostic/definition metadata, stable snapshot/detail key,
    mixed-group priority, unit matrix, and active declaration-symbol seed land
    together; the different-return control remains unchanged.
13. [ ] [mizar-checker task 49](./mizar-checker/en/todo.md) — audit-corpus
    activation and task-29 record revision: after `MT10-FS`/`MT10-AS`, checker
    Tasks 248-264 and 269-279, parser Tasks 47-48, resolver Task 31, blocked-reserved
    accepted-status Task 274, and external scheme/theorem-role Gate S1 are
    satisfied, activate 23 members and reconcile/deduplicate the exact
    24-fixture set. Resolver Task 31 solely activates its same-return member
    through `declaration_symbol`. The already active different-return conflict
    is outside the set and is not reactivated or double-counted.

#### Step 5C — Semantic bridge waves (bounded owner tasks) [x]

Each task below is bound to concrete audit-1 requirement ids in
[`tests/coverage/spec_trace.toml`](../../tests/coverage/spec_trace.toml);
its activation targets among the 120 oracle pairs are enumerated
per case, with blocking gaps, in the `owner_task` column of
[`tests/coverage/step5_activation_map.tsv`](../../tests/coverage/step5_activation_map.tsv).
A 5C task is complete when every one of its listed pairs is active at its
sidecar stage through real producer/consumer seams — no expectation
weakening, no fabricated payloads, no trust-authority relocation — its
`spec_trace` rows report active coverage, and the full crate exit gates
pass. All 5C tasks carry semantic credit and are full-gate. Execute waves
top to bottom; a task may start early only when its listed deps are done.
Deps on 5A tasks come from the per-case blocking gaps in the
activation-map ledger; every 5C task also requires 5B.2.

1. [x] **5C.1 — Variables, reservations, and local constants**
   (`mizar-checker`, `mizar-resolve`; spec ch. 4). Requirements:
   `spec.en.04.variables.*` (10 ids). Targets: 12 pairs (the historical G1
   blocker was cleared by 5A.2). Covers `let`/such-that, `set` local constants (duplicates,
   forward references, witness use), `reconsider`
   widening/narrowing-justification, `take` exemplification, inline
   `deffunc`/`defpred`, reserve implicit typing/explicit override, and
   duplicate-generalization rejection. Deps: 5A.2.
2. [x] **5C.2 — Structures** (`mizar-checker`, `mizar-core`; spec
   ch. 5). Requirements: `spec.en.05.structures.*` (10 ids) plus
   `spec.en.13.structure_update.semantic`. Targets: 14
   pairs (none gap-blocked). Covers definition fields/properties,
   inheritance (rename, member coverage, from-`set`, diamond
   consistency), dependent bracket parameters, constructors
   (field access, beta projection), selector resolution, and
   `with`-update. Deps: none beyond 5B.2.
3. [x] **[5C.3 — Attributes and argument widening](./task_contracts/en/STEP5C3-ATTRIBUTE-SEMANTICS.md)** (`mizar-checker`;
   spec ch. 3 and 6). Requirements:
   `spec.en.03.types.widening.argument_position` plus
   `spec.en.06.attributes.*` (7 ids; 8 in total). Targets: 9 pairs; all nine complete with
   bounded attributed widening through its implemented source-proof prerequisite. Covers attribute uniqueness, param-prefix
   declaration/lexicon rejection, struct-qualified disambiguation,
   redefinition coherence, negated attribute chains, non-attribute
   symbol rejection, and argument-position widening. Deps: 5A.2, 5C.2
   (struct-qualified case).
4. [x] **[5C.4 — Modes and property implementations](./task_contracts/en/STEP5C4-MODE-SEMANTICS.md)** (`mizar-checker`;
   spec ch. 7). Requirements: `spec.en.07.modes.*` (7 ids). Targets: 8
   pairs, including the plain-RHS dependent positive. Covers attributed struct radixes, dependent
   modes (`of` parameters, arity mismatch), property implementations
   (`means`/`equals`, unknown property, grammatically mandatory
   correctness), and the unprovable-sethood boundary. Deps: 5A.6, and
   5C.2 for struct radixes; the 5A.8 decision informs but does not block
   (the corpus uses explicit proof blocks).
5. [x] **[5C.5 — Predicates and functors](./task_contracts/en/STEP5C5-PREDICATE-FUNCTOR-SEMANTICS.md)** (`mizar-checker`; spec ch. 9
   and 10). Requirements: `spec.en.09.predicates.*` (7 ids) plus
   `spec.en.10.functors.*` (8 ids). Targets: 17 pairs; phrase/symbolic predicate VC generation complete; means-prefix generation complete;
   equals-infix generation complete with pending coherence; narrower-loci declaration typing complete with Pending coherence. Covers
   symbolic/phrase definitions, `equals`/`means` styles and definitional
   unfolding, dependent return types, bracket application, properties
   declarations and arity checks, duplicate-signature rejection, and
   redefinition coherence. Deps: 5A.2, 5A.3, 5A.6, 5A.7.
6. [x] **[5C.6 — Notation and modules](./task_contracts/en/STEP5C6-MODULE-SEMANTICS.md)** (`mizar-resolve`,
   `mizar-frontend`; spec ch. 11 and 12). Requirements:
   `spec.en.11.symbols.*` (2 ids) plus `spec.en.12.modules.*` (4 ids).
   Targets: 7 pairs; six outcomes complete,
   plus predicate-antonym typing complete. Covers loci checks, branch-form imports, duplicate alias
   and unknown-module rejection, and private visibility. Deeper import
   matrices stay owned by the resolver crate corpus. Deps: 5A.1, 5A.5,
   5A.7.
7. [x] **[5C.7 — Terms](./task_contracts/en/STEP5C7-TERM-SEMANTICS.md)** (`mizar-checker`; spec ch. 13). Requirements:
   `spec.en.13.terms.*` (6 ids). Targets: 8 pairs complete within this bounded profile.
   Covers numeral typing, choice (`the`) inhabitation, `qua`
   widening/invalid narrowing, set enumeration, and guarded set
   comprehension (membership pass, unbound-mapper rejection). Deps:
   5A.1.
8. [x] **[5C.8 — Formulas](./task_contracts/en/STEP5C8-FORMULA-SEMANTICS.md)** (`mizar-checker`; spec ch. 14). Requirements:
   `spec.en.14.formulas.*` (7 ids). Targets: 7 pairs (none gap-blocked).
   Covers connective precedence, non-associative `iff` (parse rejection
   and parenthesized pass), quantifier nesting and multi-witness
   existentials, `is` assertions, and unbound-free-variable rejection.
   Deps: none beyond 5B.2.
9. [x] **[5C.9 — Statements and proof organization](./task_contracts/en/STEP5C9-PROOF-ORGANIZATION.md)** (`mizar-checker`;
   spec ch. 15). Requirements: `spec.en.15.statements.*` (8 ids).
   Targets: 8 pairs. Covers
   `consider`/`given`/`hereby`/`now`, `then`/`hence` linking, iterative
   equality, and `per cases` (suppose pass, completeness obligation
   fail). Deps: 5A.4.
10. [x] **[5C.10 — Theorems and proof skeletons](./task_contracts/en/STEP5C10-THEOREM-SKELETONS.md)** (`mizar-checker`,
    `mizar-vc`, `mizar-proof`; spec ch. 16). Requirements:
    `spec.en.16.theorems.*` (4 ids). Targets: 6 pairs (none
    gap-blocked). Covers thesis tracking (assume-without-antecedent,
    conclusion mismatch, incomplete proof), lemma citation, unknown
    reference labels, and open/assumed theorem status. Deps: 5C.8, 5C.9.
11. [x] **[5C.11 — Clusters and registrations](./task_contracts/en/STEP5C11-CLUSTER-REGISTRATIONS.md)** (`mizar-checker`; spec
    ch. 17). Requirements: `spec.en.17.clusters.*` (7 ids). Targets: 7
    pairs. Covers existential/conditional/functorial
    registrations, reduction registrations, false
    coherence/reducibility rejection, and restricted adjective forms.
    Deps: 5A.2, 5C.3, 5C.10.
12. [x] **[5C.12 — Templates](./task_contracts/en/STEP5C12-UNBOUNDED-TEMPLATES.md)** (`mizar-core`, `mizar-checker`; spec
    ch. 18). Requirements: `spec.en.18.templates.*` (4 ids). Targets: 5
    pairs. Covers type-parameter bounds
    (`extends`, violation rejection), predicate/functor template
    parameters, and instantiation arity. Deps: 5A.2, 5C.5.
13. [x] **[5C.13 — Overload resolution](./task_contracts/en/STEP5C13-DISTINCT-LOCI.md)** (`mizar-checker`; spec ch. 19).
    Requirements: `spec.en.19.overload.*` (2 ids). Targets: 2 pairs
    (distinct-loci and attributed ambiguity active). Covers distinct-loci resolution and ambiguity
    rejection. Deps: 5A.2, 5C.5.
14. [x] **[5C.14 — Algorithm verification](./task_contracts/en/STEP5C14-ALGORITHM-SEMANTICS.md)** (`mizar-checker`,
    `mizar-core`, `mizar-vc`; spec ch. 20, verification constructs
    only). Requirements: `spec.en.20.algorithms.*` (8 ids). Targets: 10
    pairs (all active; bounded while generation and return-Pick failure covered). Covers `var`/`const`/`assert`, contracts
    (`requires`/`ensures`), while-loop invariants, break-outside-loop
    rejection, ghost isolation and snapshots, claim blocks, and
    computation justifications. MVM *execution* stays excluded. Deps:
    5C.10.

Exit: the source-to-semantics and core/VC completion gates hold — active
semantic corpus coverage replaces extraction-gap sentinels for the promoted
families — and the alpha boundary is met: all 103 audit-1 requirements
report active coverage, all 120 oracle pairs are active at their sidecar
stages, gaps G1-G9 are closed (G7 by the recorded 5A.8 spec decision), and
the corpus-wide syntax smoke guard (5A.9) is green. MVM/algorithm
execution remains excluded and parked. This is **alpha-corpus completion**,
not unrestricted Checker/Core/VC producer completion: broader per-crate work
and Step 5B.3 remain independently gated. A per-crate in-progress status
for general implementation does not contradict the completed alpha corpus
and neither state implies ordinary proof acceptance.

### Step 6 — Phase-output publication and orchestration [~]

The real uncached SourceLoad/Frontend edit/rebuild comparison is implemented.
Native editor [storage codecs](./mizar-frontend/en/source.md#canonical-openbuffer-payload) and [captured Driver service handoffs](./mizar-driver/en/request.md#native-source-capture) are implemented.
[Current acquired storage comparison](./task_contracts/en/SOURCE-CURRENT-STORAGE-BINDING.md) is a bounded retained-payload prerequisite. [Native Disk request preparation](./task_contracts/en/DRIVER-NATIVE-DISK-CACHE-REQUEST.md) precedes real lookup/retained scheduling.
[Bound native Disk record encoding](./task_contracts/en/DRIVER-NATIVE-DISK-CACHE-ENCODE.md) is implemented for sealed SourceLoad outputs.
[IR producer expectations](./task_contracts/en/IR-CURRENT-CACHE-PAYLOAD-BINDING.md) guard cached publication before storage decoding.
[Current native Disk lookup](./task_contracts/en/DRIVER-NATIVE-DISK-CACHE-REHYDRATE.md) binds caller-invoked typed reconstruction before retained scheduler integration.
[Explicit cached task association](./task_contracts/en/DRIVER-NATIVE-DISK-CACHE-SCHEDULE.md) owns the bounded genuine SourceLoad scheduler handoff before full runtime integration.
[Explicit retained Frontend scheduling](./task_contracts/en/DRIVER-RETAINED-NATIVE-FRONTEND-CACHE-SCHEDULE.md) binds the same protected capture before full runtime integration.
Their LSP build bridge and current diagnostic publication remain Step 8 prerequisites.
Step 7's [instance-finder design](./task_contracts/en/ATPK-INSTANCE-FINDER.md) follows its existing producer dependencies.
Automatic SourceLoad/Frontend cache routes and bounded cache-hit equivalence are
implemented. Broader active-runtime context, semantic/proof cache producers and
full artifact emission remain dependency-blocked.
Pre-audit 5B.3 and Gate S1 block only consumers requiring their outputs.

**M1 sequencing:** connect authentic, snapshot-fresh services as producers
become available (item 1), while item 2 can verify registered services
independently. Item 3 remains blocked until Step 7 supplies real accepted
witness and VerifiedArtifact inputs. See [M1](#m1-dispatch-policy).

1. [~] Real phase services and producer publication, preserving IV-007
   snapshot freshness. Completed prerequisites
   are indexed in the [driver](./mizar-driver/en/00.crate_plan.md),
   [IR](./mizar-ir/en/00.crate_plan.md), and [build](./mizar-build/en/00.crate_plan.md) plans.
   The supported frontier includes:

   - [x] Scheduled SourceLoad/Frontend publication and restricted import
     diagnostics ([A42](./task_contracts/en/STEP6A42-SCHEDULED-FRONTEND-PREFIX.md)).
   - [x] Workspace leaf/intermediate and cross-package direct lexical handoffs
     ([A47](./task_contracts/en/STEP6A47-CROSS-PACKAGE-WORKSPACE-LEXICAL-HANDOFF.md)),
     plus opt-in pregraph import discovery
     ([A48](./task_contracts/en/STEP6A48-PREGRAPH-IMPORT-DISCOVERY.md)).
   - [x] Source-bound parsed import resolution
     ([A49](./task_contracts/en/STEP6A49-SOURCE-BOUND-IMPORT-RESOLUTION.md)),
     private theorem labels ([A53](./task_contracts/en/STEP6A53-PRIVATE-THEOREM-LABEL-SOURCE.md))
     and distinct variable scopes ([A54](./task_contracts/en/STEP6A54-VARIABLE-SCOPE-IDENTITY.md)).

   [Source public theorem-label candidates](./task_contracts/en/RESOLVE-SOURCE-PUBLIC-THEOREM-LABELS.md) are implemented as raw source-relative rows before complete producer-owned summaries; service registration remains separate.

   [Public theorem formula candidates](./task_contracts/en/RESOLVE-SOURCE-PUBLIC-THEOREM-FORMULAS.md) are implemented as raw same-AST node associations before original body-free statement/signature projection; complete summaries remain separate.

   [Captured original public theorem text](./task_contracts/en/DRIVER-NATIVE-PUBLIC-THEOREM-TEXT.md) is implemented as capture-bound raw formula text; complete signature and publication producers remain separate.

   [Source theorem label-reference candidates](./task_contracts/en/RESOLVE-SOURCE-THEOREM-LABEL-REFERENCES.md) are implemented as internal source-relative batches; complete semantic tables and public diagnostic adoption remain separate.

   [Intrinsic type-reference candidates](./task_contracts/en/RESOLVE-SOURCE-THEOREM-BUILTIN-TYPE-REFERENCES.md) are implemented as bounded type-use batches; complete ordinary-name table assembly remains separate.

   [Closed-profile theorem semantic AST](./task_contracts/en/RESOLVE-SOURCE-THEOREM-SEMANTIC-AST.md) is implemented as a bounded snapshot; complete ModuleResolver/signature publication remains separate.

   [Body-free public theorem statement shells](./task_contracts/en/RESOLVE-SOURCE-PUBLIC-THEOREM-SIGNATURES.md) are implemented as raw surface-token signatures; complete env/summary publication and semantic fingerprints remain separate.

   [Source theorem module metadata](./task_contracts/en/RESOLVE-SOURCE-THEOREM-MODULE-CANDIDATES.md) composes actual AST, signature, label and citation inputs; complete ModuleResolver/summary publication remains separate.

   [Captured native theorem module metadata](./task_contracts/en/DRIVER-NATIVE-THEOREM-MODULE-CANDIDATES.md) binds that actual pair to protected captured source; service registration and current-runtime publication remain separate.

   [Source theorem module storage](./task_contracts/en/RESOLVE-SOURCE-THEOREM-MODULE-STORAGE.md) replays that actual pair from existing Syntax storage with explicit module/source binding; semantic fingerprints and real service publication remain separate.

   [Native Frontend resolver binding](./task_contracts/en/DRIVER-NATIVE-FRONTEND-RESOLVER-BINDING.md) compares a supplied published parent with fresh captured execution before the actual resolver result; runtime and service publication remain separate.

   [Original theorem module inputs](./task_contracts/en/RESOLVE-SOURCE-THEOREM-MODULE-ORIGINAL-INPUTS.md) share the genuine original arena and unaugmented env across existing native/storage consumers; semantic streams and service publication remain separate.
   [Source theorem module semantic payload](./task_contracts/en/RESOLVE-SOURCE-THEOREM-MODULE-SEMANTIC-PAYLOAD.md) supplies the actual pair and its bounded internal phase bytes; coherent service publication and complete outputs remain separate.
   [Source public theorem export candidates](./task_contracts/en/RESOLVE-SOURCE-PUBLIC-THEOREM-EXPORT-ROWS.md) supply unpublished original-owner rows with absent projected proof status; complete outgoing summaries, services and publication remain separate.
   [Source theorem summary candidates](./task_contracts/en/RESOLVE-SOURCE-THEOREM-MODULE-SUMMARY-CANDIDATE.md) compose actual Frontend-bound rows and source-checked empty inventories into an unpublished Artifact shape; native identity binding, services, general summaries and publication remain separate.
   [Captured native theorem summary candidates](./task_contracts/en/DRIVER-NATIVE-THEOREM-MODULE-SUMMARY-CANDIDATE.md) bind protected captured Frontend execution and the existing native identity projection to that raw shape; complete build authentication, services and publication remain separate.
   [Native theorem module publication](./mizar-driver/en/frontend_adapter.md#native-theorem-module-publication) seals the coherent bounded pair/storage/semantic assembly; complete general summaries remain separate.

   [Explicit scheduled theorem resolution](./task_contracts/en/DRIVER-SCHEDULED-THEOREM-MODULE-RESOLVE.md) adds bounded ModuleResolve execution; [explicit signature collection](./task_contracts/en/DRIVER-SCHEDULED-THEOREM-SIGNATURE-COLLECTION.md) adds a leading summary phase. The bounded [theorem services](./mizar-driver/en/frontend_adapter.md#scheduled-source-theorem-type-checking) continue through checker/Core/preparation, VC/discharge, ATP/candidate checking and ArtifactCommit; a separate [genuine reflexivity profile](./mizar-driver/en/frontend_adapter.md#genuine-reflexivity-fileartifact-consumer) supplies Builtin status/FILE/Artifact commit. General services, full source emission and coherent SnapshotPublished remain open.
2. [~] [Real-service equivalence](./task_contracts/en/STEP6C-REAL-SERVICE-EQUIVALENCE.md)
   (build 28 / BUILD-G-017): bounded NoKey reflexivity/discharge profiles and automatic SourceLoad/Frontend cache-backed importer/leaf/intermediate edit–warm–clean comparisons are implemented. Broader registered-service/cache equivalence follows the contract's producer dependencies.
   Build task 24, test task 14 and driver task 16 remain the existing seam-level
   guards; they do not establish this real-service gate.
3. [ ] [Artifact task 17](./mizar-artifact/en/todo.md) — deferred until real
   VerifiedArtifact projections and accepted witness/manifest publication
   outputs exist, including applicable Step 7 producers. See
   [phase-15 prerequisites](./mizar-artifact/en/phase15_emission_reevaluation.md#disposition).
   This dependency does not block item 2 or independent Step 7 production.

Exit: the artifact-publication and build-orchestration completion gates hold
for implemented phases; a prefix comparison alone does not complete Step 6.

### Step 7 — Evidence-pipeline integration [ ]

Wire task-scoped producer, acceptance and consumer seams. Keep proof policy
in Proof, cache validation in Cache, artifact publication in Artifact, and
registry/orchestration in Driver. Designs can start where their contracts
permit; each source-integration slice waits for its actual upstream outputs.

**M1 sequencing:** prefer the smallest genuinely checkable source proof.
The built-in route (item 2) and ATP route (item 1) are alternatives only
when their source inputs and Kernel prerequisites exist; neither alone
grants trusted acceptance. Item 3 supplies acceptance/witness publication,
then Step 6 item 3 performs full artifact emission. Broader portfolio,
ranking, reuse and benchmarks remain separate dependent work.

1. [~] Real ATP backend extraction and portfolio execution in `mizar-atp`
   (IV-004), using the [untrusted instance finder](./task_contracts/en/ATPK-INSTANCE-FINDER.md)
   (atp 30). Equality slices require the [kernel equality instances](./task_contracts/en/ATPK-KERNEL-EQUALITY-INSTANCES.md)
   (kernel 36).
   Bounded source [projections](./task_contracts/en/ATP-SOURCE-REGISTRATION-PROJECTIONS.md), [encoded input bindings](./task_contracts/en/ATP-ENCODED-PROBLEM-PRODUCER.md) and shared [local evidence transport](./task_contracts/en/VC-LOCAL-EVIDENCE-WIRE.md) are implemented; the backend-free finder core is complete. [Guarded Vampire FOF extraction](./mizar-atp/en/evidence.md) connects the provisioned runner to the existing kernel-checked finder; explicit sequential portfolio plans also execute; broader portfolio, backend/format and downstream integrations remain open.
   [Unchecked source candidate inputs](./task_contracts/en/ATP-SOURCE-CANDIDATE-INPUTS.md) supply bound normal wire only; full search/acceptance gates remain open.
   [Observed ground input terms](./task_contracts/en/ATP-OBSERVED-GROUND-TERMS.md) supply the backend-free finder core without unbounded term generation.
   [Explicit policy limits](./task_contracts/en/PROOF-INSTANCE-SEARCH-LIMITS.md) configure finder counting/enforcement unchanged for the guarded backend-assisted route.
   [Source term correspondence](./task_contracts/en/ATP-SOURCE-GROUND-TERM-CORRESPONDENCE.md) is implemented for existing observed choices; general mapping/extraction/search remains open.
   [Bounded source candidate SAT screening](./task_contracts/en/ATP-SOURCE-CANDIDATE-SAT-SCREEN.md) is implemented as an explicit-limit untrusted query; general search and fresh acceptance remain open.
   [Bounded structured source substitutions](./task_contracts/en/ATP-SOURCE-STRUCTURED-SUBSTITUTIONS.md) implement the existing-factory extraction prerequisite; full finder/search and acceptance remain open.
   [Opt-in screened source checking](./task_contracts/en/PROOF-SOURCE-SCREENED-ATTEMPT.md) implements the bounded Proof consumer; full search and acceptance/publication remain open.
   [Uncited theorem local handoffs](./task_contracts/en/VC-SOURCE-THEOREM-LOCAL-HANDOFF.md) supply original guard/goal inputs; equality candidates and fresh acceptance remain open.
   The [genuine theorem reflexivity chain](./mizar-proof/en/status.md#source-theorem-reflexivity-same-result-composition) now composes original source identity, actual checked results, conservative dependencies and trusted status/metadata. Its [FILE/Artifact consumer](./mizar-driver/en/frontend_adapter.md#genuine-reflexivity-fileartifact-consumer) publishes the matching bounded Builtin witness and commits real artifacts. General acceptance, imported-FQN recording, complete footprints and reuse remain open.
2. [ ] [Built-in evidence candidates](./task_contracts/en/PREATP-FORMULA-EVIDENCE.md)
   (VC 60); design frozen, implementation deferred until authentic
   formula/context/substitution/provenance producers exist. [Original-source direct-premise candidates](./task_contracts/en/VC-SOURCE-DIRECT-PREMISE-CANDIDATE.md) supply only bounded untrusted association.
3. [~] [By/built-in acceptance and witness publication](./task_contracts/en/PROOF-DISCHARGE-ACCEPTANCE.md)
   (Proof 23 / Artifact 25). The bounded genuine theorem-reflexivity Builtin route
   above is implemented. General ATP acceptance follows item 1; broader built-in
   acceptance follows item 2. Each witness requires a matching accepted result;
   full emission remains Step 6 item 3.
   [Fresh source candidate checking](./task_contracts/en/PROOF-SOURCE-CANDIDATE-CHECK.md) is a bounded classification prerequisite; general acceptance/publication remain open.
   [Fresh source-bound selection](./task_contracts/en/PROOF-SOURCE-CANDIDATE-SELECTION.md) is implemented for genuine candidate/check outputs; the separate genuine reflexivity route above supplies bounded Builtin witness publication. General acceptance/publication remains open.
   [Direct-premise source checking](./task_contracts/en/PROOF-SOURCE-DISCHARGE-CHECK.md) is a bounded built-in classification prerequisite. [Raw local component hashes](./task_contracts/en/VC-LOCAL-EVIDENCE-COMPONENT-HASHES.md) precede accepted metadata association. [Raw same-call metadata](./task_contracts/en/PROOF-SOURCE-DISCHARGE-METADATA.md) precedes trusted witness production.
   [Original-source raw identity](./task_contracts/en/PROOF-SOURCE-ORIGINAL-IDENTITY.md) is implemented with explicit incomplete VC-local data; same-check witness binding remains open.
4. [ ] Proof cache/witness handoffs: Proof reuse metadata into Cache
   validation (IV-005, fail-closed IV-002). Source-derived trusted reuse follows
   the corresponding accepted producer outputs; internal seams remain separate.
5. [ ] [Deterministic premise ranking](./task_contracts/en/ATPK-PREMISE-RANKING.md)
   (atp 31) follows its own producer dependencies. [Structured local ordering](./task_contracts/en/ATP-STRUCTURED-LOCAL-RANKING.md) precedes complete fact/budget assembly.
6. [ ] [ATP re-proving benchmark](./task_contracts/en/ATPK-BENCHMARK.md)
   (test 25), after items 1 and 5.

Exit: the proof-and-algorithm-verification completion gate holds with active
`proof_verification` coverage.

### Step 8 — User-facing consumer wave [~]

Adopt user-facing surfaces only after the owning producers publish stable
diagnostics, metadata, artifacts, and semantic indexes. Before M1,
independent native source input, editor lifecycle, snapshot and current
diagnostic transport may proceed. Proof-status hover, semantic navigation/
completion and published phase-16 rendering require their actual upstream
outputs; do not fabricate metadata to unblock these consumers.

1. [ ] [mizar-resolve task 30](./mizar-resolve/en/todo.md) — public resolver
   diagnostic adoption beyond the implemented E0220–E0224 import subset
   (SCA-004; see
   [consumer_adoption_decision.md](./mizar-diagnostics/en/consumer_adoption_decision.md)).
   The SSA-018 `of`/`over` scope-sensitivity lint is recorded for this wave.
2. [~] [mizar-lsp tasks 1-25](./mizar-lsp/en/todo.md) — server, snapshots,
   diagnostics, build bridge, metadata, navigation, actions, explanations,
   bounded snapshot-backed completion and the `@show_*`/`@eval` projection
   audit; broader semantic consumers and general SCA-003 coverage remain open.
3. [~] [mizar-doc tasks 1-34](./mizar-doc/en/todo.md) — published-input reading and
   indexing, native inputs and validated published comment attachment are implemented. CommonMark/tag parsing and authenticated link/render prerequisites are available; coherent planning, full HTML/phase-16 rendering and extraction remain open (SCA-002). `mizar-doc` must not re-run semantic analysis.
   [Optional published module summaries](./task_contracts/en/DOC-PUBLISHED-MODULE-SUMMARIES.md) are implemented as preserved label inputs; full index/link planning remains open.
   [Stored label export candidates](./task_contracts/en/DOC-PUBLISHED-LABEL-EXPORT-CANDIDATES.md) are implemented as exact stored-record queries; full label/link planning remains open.
   [Optional native Disk source content](./task_contracts/en/DOC-NATIVE-DISK-SOURCE.md) implements a text/hash prerequisite; full currentness integration remains separate.
   [Optional native lexical source](./task_contracts/en/DOC-NATIVE-LEXICAL-SOURCE.md) is implemented as a raw-trivia prerequisite; it does not replace the published comment-attachment route.
4. [~] [`mizar lsp` stdio entry point](./task_contracts/en/EDITOR-LSP-ENTRY.md)
   has an executable [lifecycle host](./mizar-driver/en/cli.md#stdio-lifecycle-host);
   workspace/settings, build binding and current-build diagnostics remain open. [Raw initialize inputs](./task_contracts/en/LSP-RAW-INITIALIZE-HANDOFF.md) precede their host binding. [Explicit native metadata acquisition](./task_contracts/en/LSP-NATIVE-WORKSPACE-INPUTS.md) precedes runtime context. [Current Disk diagnostic indexes](./task_contracts/en/DRIVER-CURRENT-DISK-DIAGNOSTIC-INDEX.md) and [native OpenBuffer handoffs](./task_contracts/en/DRIVER-CURRENT-NATIVE-DIAGNOSTIC-INDEX.md) precede current LSP transport. [Import-discovered native indexes](./task_contracts/en/DRIVER-IMPORT-DISCOVERED-NATIVE-DIAGNOSTIC-INDEX.md) compose genuine pregraph results before native host binding. [Explicit frontend notifications](./task_contracts/en/LSP-NATIVE-FRONTEND-DIAGNOSTIC-NOTIFICATIONS.md) bind original current OpenBuffer inputs before wire/host integration. [Originating native capture](./task_contracts/en/LSP-NATIVE-OPEN-BUFFER-CAPTURE.md) binds the original open incarnation before full snapshot/host integration.
   [Native filesystem source layout](./task_contracts/en/BUILD-NATIVE-SOURCE-LAYOUT.md) is implemented for ordinary workspace trees; [explicit native OpenBuffer association](./task_contracts/en/BUILD-NATIVE-OPEN-BUFFER-INPUT.md) precedes complete native host binding.
   [Native workspace input acquisition](./task_contracts/en/BUILD-NATIVE-WORKSPACE-INPUTS.md) supplies parser-owned metadata before genuine runtime context and native host binding. [Package replay metadata](./task_contracts/en/BUILD-COMPUTATION-POLICY.md) is a configuration prerequisite; runtime replay and full configuration identity remain pending. [Workspace lock coordinates](./task_contracts/en/BUILD-WORKSPACE-LOCK-PATH-BINDING.md) bind parsed member inputs before native runtime context. [Lock content fingerprints](./task_contracts/en/BUILD-LOCK-CONTENT-FINGERPRINT.md) supply current typed-content identity before complete context and caller binding. [Explicit native lock association](./task_contracts/en/DRIVER-NATIVE-LOCK-CONTENT-BINDING.md) compares that profile at caller request before runtime integration. [Package configuration content](./task_contracts/en/BUILD-CONFIG-CONTENT-FINGERPRINT.md) supplies current typed metadata identity before complete active configuration and caller binding. [Explicit native configuration association](./task_contracts/en/DRIVER-NATIVE-CONFIG-CONTENT-BINDING.md) is a caller-only prerequisite. [Optional package toolchain metadata](./task_contracts/en/BUILD-PACKAGE-TOOLCHAIN-METADATA.md) supplies workspace constraints before complete pinning/runtime identity. [Recorded toolchain metadata](./task_contracts/en/BUILD-LOCKED-TOOLCHAIN-METADATA.md) preserves parsed values; [raw constraint relations](./task_contracts/en/BUILD-RECORDED-TOOLCHAIN-MATCHES.md) precede authenticated pinning and runtime binding.
5. [~] [Code completion](./task_contracts/en/EDITOR-LSP-COMPLETION.md)
   (mizar-lsp task 25): snapshot-backed syntax and retained-publication routes are implemented; broader producer readiness and owner-task closeout remain separate.
6. [ ] [VS Code extension](./task_contracts/en/EDITOR-VSCODE-EXTENSION.md)
   (mizar-lsp task 28), published on the Marketplace from `editors/vscode/`;
   deps items 4 and the toolchain manager below.

Exit: the user-facing-projections completion gate holds.

### Algorithm definitional semantics (specification amendment) [ ]

The September 2026 amendment to specification chapters 10, 11, 16, 20, 22,
and 23 adds definitional algorithms, measured `ensures` self-reference,
algorithm synonyms and citations, the computation uniqueness obligation, and
definition-cycle confinement. These tasks are outside the alpha
source-to-artifact claim and do not block Steps 6-8; each starts when its
listed dependencies are complete.

1. [ ] Core 54 — [defining equations](./task_contracts/en/ALGDEF-CORE54-DEFINING-EQUATIONS.md);
   deps Core 42-43 and 46.
2. [ ] VC 56 — [induction-hypothesis context](./task_contracts/en/ALGDEF-VC56-INDUCTION-HYPOTHESIS.md);
   deps Core 46/52 and VC 43-44, 47, and 51-52.
3. [ ] VC 58 — [computation uniqueness obligation](./task_contracts/en/ALGDEF-VC58-COMPUTATION-UNIQUENESS.md);
   deps Core 42/44/46, VC 44 and R3 validated exports; same-package Open-goal generation also waits for R3.
4. [ ] Checker 280 — [definition dependency cycles](./task_contracts/en/ALGDEF-CHECKER280-DEFINITION-CYCLES.md);
   deps resolver declaration dependency edges and algorithm call targets.
5. [~] Resolver 33 — [algorithm synonym targets and citations](./task_contracts/en/ALGDEF-RESOLVE33-ALGORITHM-REFERENCES.md): definition-label identity projection implemented; automatic synonym targets, citations and promotion adoption remain deferred.
6. [ ] VC 57 — [module-local promotion axioms as premises](./task_contracts/en/ALGDEF-VC57-PROMOTION-FACTS.md);
   deps Core 54, VC 52/53, E1/E2/E4 producers and resolver 33; follows the bounded proof-status
   precedent for module-local acceptance.
7. [ ] VC 59 — [imported promotion axioms](./task_contracts/en/ALGDEF-VC59-IMPORTED-PROMOTION-FACTS.md);
   blocked-reserved on the transferable accepted-status authority shared with
   checker Task 274 and VC 53.

8. [~] [ALGDEF-PARSER-FAIL](./task_contracts/en/ALGDEF-PARSER-FAIL.md) — bare syntax complete; message/report forms remain lexical-spec-gap blocked, reflected reports also await AST-PARSE.
9. [~] [ALGDEF-VC53-NORMAL-RETURN](./task_contracts/en/ALGDEF-VC53-NORMAL-RETURN.md) — bounded direct-return E1/E4 requests/checking, native inhabitation/E4 adoption and typing/single-valuedness schema association implemented; broader call/CFG/termination proof and algorithm acceptance remain open.

[ALGDEF-RESOLVE-CALL-EDGES](./task_contracts/en/ALGDEF-RESOLVE-CALL-EDGES.md) has implemented source inventory/candidates, selected definiens dependencies and complete explicitly selected local recursive groups. Remaining slices concern external imported/builtin identities, typed target selection and consumer adoption; cycle/termination/promotion acceptance remains separate. See the [source-edge owner](./mizar-resolve/en/symbols.md#planned-algorithm-call-dependencies).

Core 48 additionally owns the missing-`return`-or-`fail` diagnostic, and Core 46 the
contract self-reference rejection. Public codes E0330, E0501, and E0502 stay
spec-reserved until a separately authorized diagnostics adoption.

### Reflected-syntax implementation [ ]

Active implementation contracts follow their own dependencies; completed AST-SPEC records remain specification-transfer history.
The [multiple-argument Binder specification](./task_contracts/en/AST-MULTI-BINDER-SPEC.md) fixes the product-domain extension and acceptance requirements for these owners; it adds no implementation activation.
The [Binder task decomposition](./task_contracts/en/AST-MULTI-BINDER-PLAN.md) assigns MBP-01–09 to the existing owners, adds the explicit Resolver prerequisite and separates provisional formation from proof/adoption. MBP-01 syntax prerequisites, MBP-03 pinned-library input inventory and MBP-08 descriptor preparation have landed in bounded scope; certification, formation and source-aware emission remain gated by their authentic producers. Differentiation remains a later specification/library task.

1. [~] [AST-PARSE](./task_contracts/en/AST-PARSE.md) — expr/Binder hosts and explicit computation-input/direct-thesis syntax prerequisites implemented; broader ordinary inputs, ambiguous partitions and semantic/formation/reflection consumers remain open.
2. [~] [AST-DIAGNOSTICS](./task_contracts/en/AST-DIAGNOSTICS.md) — reserved descriptors implemented; emission/adoption waits for source-aware producers.
3. [ ] [AST-TYPE](./task_contracts/en/AST-TYPE.md) — follows syntax and certified library/declaration producers.
4. [ ] [AST-LOWER](./task_contracts/en/AST-LOWER.md) — follows authentic checked formation.
5. [ ] [AST-CONTRACT](./task_contracts/en/AST-CONTRACT.md) — complete guarded obligations; earlier ALGDEF owners retain promotion/uniqueness.
6. [ ] [AST-ARTIFACT](./task_contracts/en/AST-ARTIFACT.md) — runtime export stage follows verified executable/body correspondence.
7. [ ] [ALG-MVM-EVALUATION](./task_contracts/en/ALG-MVM-EVALUATION.md) — runtime producer after verified body/export boundaries.
8. [ ] [AST-COMPUTE](./task_contracts/en/AST-COMPUTE.md) — import/trust/replay after actual normal-return reports.
9. [ ] AST-ARTIFACT computation provenance/publication stage follows AST-COMPUTE; it does not block the earlier runtime export.
10. [ ] [AST-CORPUS](./task_contracts/en/AST-CORPUS.md) — each test stage follows its owner; parser additions can proceed first.

### MVM Readiness Decomposition [ ]

The existing [reflected-syntax implementation](#reflected-syntax-implementation--)
and [algorithm definitional](#algorithm-definitional-semantics-specification-amendment--)
sections remain the **only implementation-contract owners**. This table
decomposes their MVM critical path into narrower dependency/dispatch gates;
it does not redefine the language, add a second global ordering authority,
or make blocked implementations ready. R1 and R2-R8 designs are registered
as **documentation-only** work; R2-R7 implementations and R8 runtime
test activation are not dispatchable from this table. Implementation/test
activation slices require real producers and separately frozen owner
contracts and independently specified test oracles.
All steps remain subject to the real-producer, independent-review and
test-oracle rules.

| Slice | Existing owner and deliverable | Dependency / dispatch rule |
|---|---|---|
| **MVM-R1 [~]** — authentic source readiness inventory | [MVM-READINESS-MAP](./task_contracts/en/MVM-READINESS-MAP.md), owned by `mizar-vc`; [source-backed inventory](./mizar-vc/en/mvm_readiness.md) drafted. The minimal `idalgo` source profile is an **Open VC**, not an executed or accepted algorithm. | **Inventory review closed.** Resolve documented Q1–Q3 under existing authorities before any R2 implementation contract: record a decision the owner already made, link the owner document that answers the question, or keep it open under its owning slice; a review unit makes no new choice. Runtime R2–R8 remain gated. Do not redispatch the completed inventory work; the review runs in Lane 2 under the Muse rule below, which unlocks no implementation. |
| **MVM-R2 [~ design]** — verified body/call correspondence | [Core-owned design contract](./task_contracts/en/MVM-R2-CORRESPONDENCE-DESIGN.md) and [EN/JA design](./mizar-core/en/mvm_correspondence.md) drafted for accepted definition/body identity and separately authenticated typed invocation; existing Core 42-53, AST-TYPE/LOWER, AST-CONTRACT and VC 53 retain implementation ownership. | **Docs-only; design review closed.** R1 review, retained AST-TYPE/AST-LOWER gates (Q1 owner decision) and accepted definition evidence (Q2) gate R3 export; the genuine typed call separately gates R4, not R3. Open VC is not verified. R3/R4 remain blocked. |
| **MVM-R3 [~ design]** — runtime export and validating loader | [Artifact design contract](./task_contracts/en/MVM-R3-ARTIFACT-LOADER-DESIGN.md) and [EN/JA owner design](./mizar-artifact/en/mvm_runtime_export.md) drafted; [AST-ARTIFACT](./task_contracts/en/AST-ARTIFACT.md) retains implementation ownership, Build/Driver are loader consumers. | **Docs-only; design review closed.** R2 authenticated **definition/body** producer (no ground caller prerequisite) and R3 format/loader, same-package staging and correspondence decisions remain required before any export. Runtime export precedes MVM; later R7 computation-record publication must not block it. No unverified code or dependency-source fallback. |
| **MVM-R4 [~ design]** — minimal actual evaluator | [VC-owned design contract](./task_contracts/en/MVM-R4-EXECUTOR-DESIGN.md) and [EN/JA design](./mizar-vc/en/mvm_executor.md) drafted; [ALG-MVM-EVALUATION](./task_contracts/en/ALG-MVM-EVALUATION.md) still owns implementation. Defines the narrow actual ground/direct normal-return path, validation, failure/budget classes and R6/R7 boundary. | **Docs-only; design review closed.** R2 independently accepted definition and typed ground invocation, R3 validated executable/loader, AST-LOWER complete inputs, typed computability facts and VM accounting/report decisions are prerequisites. No R4 implementation, trusted status, E5 credit or new `mizar-mvm` crate is authorized. |
| **MVM-R5 [~ design]** — control flow, failure and resource bounds | [VC-owned design contract](./task_contracts/en/MVM-R5-EXTENDED-EXECUTION-DESIGN.md) and [EN/JA owner design](./mizar-vc/en/mvm_extended_execution.md) drafted. R5-A local state/branch; B loops; C calls/recursion; D set/Pick/match; E failure/ghost; F budgets/cancellation. [ALG-MVM-EVALUATION](./task_contracts/en/ALG-MVM-EVALUATION.md) retains execution ownership. | **Docs-only; design review closed.** Each R5-A–F execution slice waits for supported authentic R4, feature-specific Core/CFG, R3 validated loader and a separately frozen owner/test contract. R6 uniqueness is mandatory for computation **discharge** where applicable, not a substitute for a runtime producer. Failure/exhaustion gives no E5 or non-return proof. |
| **MVM-R6 [~ design]** — full-closure deterministic admission | [VC-owned design contract](./task_contracts/en/MVM-R6-ADMISSION-DESIGN.md) and [EN/JA owner design](./mizar-vc/en/mvm_computation_admission.md) drafted. R6-A complete local/imported call closure; B value/Pick/set triggers and exact exemptions; C full uniqueness VCs; D Proof admission/E0330. [ALGDEF-COMPUTATION-ADMISSION](./task_contracts/en/ALGDEF-COMPUTATION-ADMISSION.md) and [VC 58](./task_contracts/en/ALGDEF-VC58-COMPUTATION-UNIQUENESS.md) retain implementation/acceptance ownership. | **Docs-only; design review closed.** Source-owned local R6-A/B VC58 preparation may precede R4; R6-C requires R3 exports. R6-D final Proof acceptance must bind complete evaluated closure and per-callee genuine proofs/exemptions before computation discharge. Open or missing closure never grants admission; an R4/R5 return or E1/replay status cannot bypass R6. |
| **MVM-R7 [~ design]** — certified computation and replay | [Proof-owned design contract](./task_contracts/en/MVM-R7-CERTIFIED-IMPORT-DESIGN.md) and [EN/JA owner design](./mizar-proof/en/mvm_computation_import.md) drafted. R7-A E5/independent premises; B T1–T8 complete candidate translation; C whole residual inspection; D computed/replayed kernel trust and strict policy; E exact Artifact/Cache provenance. [AST-COMPUTE](./task_contracts/en/AST-COMPUTE.md) retains Proof implementation ownership; [AST-ARTIFACT](./task_contracts/en/AST-ARTIFACT.md) retains later publication. | **Docs-only; design review closed.** Real R4/R5 normal return, R6 accepted closure, AST-TYPE/LOWER/CONTRACT/formation, [BUILD-COMPUTATION-POLICY](./task_contracts/en/BUILD-COMPUTATION-POLICY.md) and genuine replay proof basis are prerequisites. No raw return/partial ensures/unchecked residual, weaker cache hit or strict-policy computed taint may become accepted proof. |
| **MVM-R8 [~ design]** — stage-specific end-to-end corpus | [Test-owned design contract](./task_contracts/en/MVM-R8-CORPUS-DESIGN.md) and [EN/JA test matrix](./mizar-test/en/mvm_corpus_matrix.md) drafted. R8-A source/type, B Core/CFG/VC, C accepted body, D Artifact loader, E MVM execution/failure, F closure admission, G conversion/replay/cache, H true source-to-artifact integration. [AST-CORPUS](./task_contracts/en/AST-CORPUS.md) retains test ownership. | **Docs-only; design review closed.** Existing 7 stage IDs and `expected_phase` list have no MVM-specific runtime phase; its admission requires separate harness review. Future parser-only test-first work requires a frozen owner and real oracle. Every positive R8 case waits for its entire actual producer chain; R8-C/D require R2/R3, and R8-E–H require their applicable R2–R7 producers; no rebaseline, fake positive fixture or coverage credit. |

MVM implementation sequencing follows the
[R1 dependency inventory](./mizar-vc/en/mvm_readiness.md#dependency-dag-and-ordered-owner-freezes):
definition-side export precedes execution; source-owned local R6-A/B
preparation may proceed independently; R6-C requires R3 exports;
runtime and proof stages await their own producers.

**Milestone separation:** (A) R1-R3 readiness/design/export prerequisites;
(B) R4-R5 actual execution of supported verified algorithms; (C) R6-R7
certified `by computation` integration; (D) R8/generality and the remaining
reflected-syntax/Binder, cross-package, extraction and release gates. Completion
of a narrower stage never implies completion of a later stage or the
[complete processor](#completion-gates). The verified ordinary input path is
not an exemption from the currently recorded AST/Proof dependencies.

**Muse rule (two lanes, owner decisions 2026-11-20):** Lane 1 is the
existing dispatch queue with M1 priority; it follows this TODO's written
order without skipping units of the [Lane 2 set](#lane-2-set), never
duplicating a unit Lane 2 has in flight or claimed. Lane 2 prioritizes
every unit of that set — documentation, contract freezes and
implementation, including the producers this table names as
prerequisites — one contract per unit, in its own worktree. It takes a
general-queue unit only when no set unit is startable. It is not a second
global ordering authority: within the set it follows the
[R1 dependency inventory](./mizar-vc/en/mvm_readiness.md#dependency-dag-and-ordered-owner-freezes)
and the milestone order (A) to (D) above, taking the earliest missing
producer of the lowest open milestone, and an independent branch only
while that producer is blocked or parked.
A contract is dispatched by one lane at a time. A unit is in the Lane 2
set when its contract is listed there, when it writes an MVM contract or
`mvm_*` owner design, or when this table or an MVM document names its contract as the
owner or a prerequisite of an R slice; Muse lists the contract before
dispatch. Any other unit is general-queue work. A unit that spans both
sides is split; if it cannot be split it joins the Lane 2 set.
Lane assignment unlocks nothing. Do not dispatch R2-R7
**implementations** or runtime/proof test activation for R8 before the
following hold: first verify authentic producers, the actual stage
runner and oracle, and separately freeze the owning implementation/test
contract. Missing accepted proofs, formation, executable/loader, VM
semantics, complete computation admission, certified import or runtime
stage authority remain blockers. R5-A–F, R6-A–D, R7-A–E and R8-A–H are
design slices only until so frozen.
A finding or stop that would change a gate, prerequisite, dependency
edge, owner or trust/acceptance boundary, or that would answer a
recorded open question of an R design, is parked for an owner decision
and is not decided by the unit; a parked unit in one lane never blocks
the other lane.
Lane 2 commits enter local `main` only by rebase and fast-forward while
no run is active in the Lane 1 worktree, after the checks every changed
task owes pass on the integrated tree and after the authorities and the
producers the unit consumed are unchanged there or re-reviewed. Lane 1
never rebases. The lanes do not edit one file concurrently: the later
unit waits until the earlier one has landed. The changed-task count,
the full `cargo test`, the pre-push audit and the push stay single and
project-wide; a Lane 2 unit counts once, when integrated. Muse pushes
only a sealed revision of `main` on whose exact tree full `cargo test`
passed and which the pre-push audit covered; nothing lands between
seal and push. No active Step 6/7 work is preempted and no second
orchestrator is implied. Muse reports blocked set producers or owner
decisions; neither lane idles while a startable unit exists anywhere.

#### Lane 2 set

Priority set only; each contract keeps its own readiness and gates.

| Class | Contracts | R slices served |
|---|---|---|
| MVM | [MVM-READINESS-MAP](./task_contracts/en/MVM-READINESS-MAP.md) and the R2-R8 design contracts above; [AST-ARTIFACT](./task_contracts/en/AST-ARTIFACT.md); [ALG-MVM-EVALUATION](./task_contracts/en/ALG-MVM-EVALUATION.md); [ALGDEF-COMPUTATION-ADMISSION](./task_contracts/en/ALGDEF-COMPUTATION-ADMISSION.md); [VC 58](./task_contracts/en/ALGDEF-VC58-COMPUTATION-UNIQUENESS.md); [AST-COMPUTE](./task_contracts/en/AST-COMPUTE.md); [BUILD-COMPUTATION-POLICY](./task_contracts/en/BUILD-COMPUTATION-POLICY.md); the R8-A–H families of [AST-CORPUS](./task_contracts/en/AST-CORPUS.md); implementation contracts later frozen from an R design | R1-R8 |
| Registered producers | [ALGDEF-VC53-NORMAL-RETURN](./task_contracts/en/ALGDEF-VC53-NORMAL-RETURN.md); [Core 42-53](./mizar-core/en/todo.md); [AST-TYPE](./task_contracts/en/AST-TYPE.md); [AST-LOWER](./task_contracts/en/AST-LOWER.md); [AST-CONTRACT](./task_contracts/en/AST-CONTRACT.md); [AST-FORMATION-PROOFS](./task_contracts/en/AST-FORMATION-PROOFS.md); [ALGDEF-RESOLVE-CALL-EDGES](./task_contracts/en/ALGDEF-RESOLVE-CALL-EDGES.md) external-identity/typed-selection/consumer slices | R2, R4-R7 |

### Distribution and playground [ ]

Installation and web access for the complete-processor claim. They follow the
LSP entry point and do not block Steps 6-8.

1. [ ] mizar-driver 24 — [toolchain manager](./task_contracts/en/DIST-TOOLCHAIN-MANAGER.md)
   for `mizar` and ATP backends; follows build-owned
   [manifest/lock pinning](./task_contracts/en/BUILD-TOOLCHAIN-PIN.md); release and redistribution prerequisites remain blocked.
2. [ ] mizar-driver 25 — [installers](./task_contracts/en/DIST-INSTALLERS.md)
   for other editors (install script, Windows installer and winget, Homebrew).
3. [ ] mizar-lsp 29 / mizar-driver 26 — [web playground](./task_contracts/en/WEB-PLAYGROUND.md)
   on the Lean 4 web editor model (server sessions; no browser-side WebAssembly).

### Generalization and quality guards [x]

These tasks check that producers generalize beyond the corpus files they were
fitted to. They may run alongside Steps 6-8; the metamorphic guard should run
early because its result qualifies what the active corpus demonstrates.

1. [x] mizar-test 23 — [metamorphic generalization guard](./task_contracts/en/QUALITY-METAMORPHIC-GUARD.md); named bridge defects belong to [mizar-test 26](./task_contracts/en/TEST-METAMORPHIC-BRIDGE-GENERALIZATION.md).
2. [x] mizar-test 24 — [literal source-offset ratchet](./task_contracts/en/QUALITY-LITERAL-OFFSET-RATCHET.md).
3. [x] Checker 281 — [exact-profile retirement](./task_contracts/en/QUALITY-EXACT-PROFILE-RETIREMENT.md);
   completed through Task 284 exit criterion 3.

### Statement review aids (specification amendment) [ ]

The September 2026 amendment to specification chapters 21-23 and Appendix D
adds review aids for translated or AI-generated statements. Each task starts
when its dependencies are complete; none blocks Steps 6-8.

1. [ ] Checker 282 — [elaborated statements](./task_contracts/en/AIREV-ELABORATED-STATEMENTS.md)
   in the artifact and LSP hover; deps checker Task 258 and the artifact writer.
2. [ ] Resolver 34 — [W0004 reserve-typed public statements](./task_contracts/en/AIREV-W0004-RESERVE-TYPED-STATEMENTS.md).
3. [ ] mizar-proof 22 — [W0104 vacuous statements](./task_contracts/en/AIREV-W0104-VACUOUS-STATEMENT.md);
   deps Step 7 accepted evidence.
4. [ ] mizar-atp 32 — [counterexample search on open statements](./task_contracts/en/AIREV-OPEN-STATEMENT-COUNTEREXAMPLE.md);
   deps Step 7 item 1.

### Parked and trigger-based work

Not part of the sequential flow; each row records its re-entry trigger.

| Item | Trigger |
|---|---|
| [mizar-kernel task 32](./mizar-kernel/en/todo.md) — solver step-budget deferral (audit F3) | any pinned `batsat` version change (task-24 audit procedure) |
| [mizar-parser task 46](./mizar-parser/en/todo.md) — concrete operator declarations | [x] completed after fresh audit confirmed frontend Task 20 had already met the trigger |
| [mizar-syntax task 21](./mizar-syntax/en/todo.md) — rustdoc summaries | deferred documentation pass |
| MVM execution (spec 20) | Actual execution follows the existing reflected-syntax contracts and the [MVM readiness decomposition](#mvm-readiness-decomposition--); R1 is independently preparatory, while R2-R8 require their documented producers and reviewed owner contracts. |
| Code extraction / additional runtime backends | Remain parked until separately contracted; neither MVM R4 execution nor `by computation` acceptance by itself authorizes them or the complete-processor claim. |

## Appendix A — Incremental Verification Contract Inventory

The design contract in
[architecture/en/22.incremental_verification_contract.md](./architecture/en/22.incremental_verification_contract.md)
adds cross-crate obligations. This index keeps them bound to owner tasks;
IV-referenced obligations surface in plan steps 3, 6, and 7.

| ID | Classification | Contract delta | Owning TODO task |
|---|---|---|---|
| IV-001 | `source_drift` | VC reuse needs cross-edit `ObligationAnchor`, canonical VC fingerprints, local-context fingerprints, and dependency-slice fingerprints; `VcId`, `SourceRange`, and syntax-node ids are not stable reuse identity. | [mizar-core task 18](./mizar-core/en/todo.md), [mizar-resolve tasks 2, 4, and 17-21](./mizar-resolve/en/todo.md), [mizar-vc task 20](./mizar-vc/en/todo.md), [mizar-cache task 20](./mizar-cache/en/todo.md) |
| IV-002 | `source_drift` | Cache reuse must fail closed: incomplete dependency data, `uncacheable` outputs, schema/toolchain/policy incompatibility, witness mismatch, or deterministic discharge mismatch force a miss. Post-audit: reuse records lacking the extended kernel-evidence handoff identity (goal polarity, context-identity payload) also force a miss. | [mizar-cache tasks 20 and 24](./mizar-cache/en/todo.md), [mizar-ir tasks 9-10](./mizar-ir/en/todo.md), [mizar-build tasks 18 and 24](./mizar-build/en/todo.md) |
| IV-003 | `source_drift` | Clean sequential, clean parallel, incremental sequential, and incremental parallel builds must agree on proof acceptance, published artifacts, interface hashes, dependency-facing summaries, and canonical diagnostics. | [mizar-build task 24](./mizar-build/en/todo.md), [mizar-test task 14](./mizar-test/en/todo.md), [mizar-driver task 16](./mizar-driver/en/todo.md) |
| IV-004 | `source_drift` | ATP portfolio evidence collection may be parallel, but accepted proof identity and early stop are policy-deterministic, not raw-completion-order driven. | [mizar-atp task 25](./mizar-atp/en/todo.md), [mizar-proof tasks 6-7, 9, and 12-13](./mizar-proof/en/todo.md) |
| IV-005 | `source_drift` | Proof-reuse metadata exported to the cache must include compatible verifier policy plus selected proof witness hash or deterministic discharge hash without upgrading evidence classes. Post-audit: selection/status reuse metadata additionally records the accepted evidence's goal polarity, and corrected kernel rejections are never upgraded by policy. | [mizar-proof tasks 17 and 21](./mizar-proof/en/todo.md), [mizar-cache tasks 20 and 24](./mizar-cache/en/todo.md) |
| IV-006 | `design_drift` (closed) | The corrected labeled `redefine pred label: ...` target is documented and implemented by parser task 36 / syntax task 22; syntax task 23 closed the stale roadmap drift. | [mizar-parser task 36](./mizar-parser/en/todo.md), [mizar-syntax tasks 22-23](./mizar-syntax/en/todo.md) |
| IV-007 | `source_drift` | Snapshot-scoped results must respect `BuildSnapshotId` freshness: obsolete or stale results cannot publish as current, obsolete outputs can be reused only as validated cache inputs, and open-buffer results never become package artifacts. | [mizar-ir tasks 7-10 and 13](./mizar-ir/en/todo.md), [mizar-build tasks 14, 18, and 24](./mizar-build/en/todo.md), [mizar-driver tasks 3, 14, and 16](./mizar-driver/en/todo.md), [mizar-diagnostics tasks 8-9](./mizar-diagnostics/en/todo.md), [mizar-lsp tasks 6-9](./mizar-lsp/en/todo.md) |

Already covered by current frontend work: token/AST cache keys separate active
lexical environment fingerprints, parser lexing plan/filter hashes, and
bundle/source-level language edition. Keep this covered by
[mizar-frontend task 19](./mizar-frontend/en/todo.md) and later source/spec
audits; no new architecture-22 task is needed for that slice.

## Appendix B — Specification Coverage Audit Follow-Ups

[spec_coverage_audit.md](./spec_coverage_audit.md) tracks coverage from each
`doc/spec/en/` chapter to implementation-facing design docs. Non-closed
follow-ups, in roadmap order:

| ID | Classification | Coverage gap | Owning TODO task |
|---|---|---|---|
| SCA-001 | `design_drift` | The top-level design index must stay aligned with this roadmap's workspace-crate statuses. | Docs-only sync of [README.md](./README.md); future roadmap sync tasks must re-check it. |
| SCA-002 | `todo` | The [published-input reader](./mizar-doc/en/artifact_reader.md) and initial [raw export index](./mizar-doc/en/doc_build.md) specify input validation/ordering; full build/index, remaining phase-16 specs and generation remain open. | [mizar-doc tasks 4, 6, 9, 11, 13, 16, 18, 21, 23, and 29](./mizar-doc/en/todo.md) (plan step 8) |
| SCA-003 | `todo` | `@show_*` and `@eval` have parser/syntax coverage but need end-to-end display/evaluation projection boundaries. | [mizar-lsp task 24](./mizar-lsp/en/todo.md), plus `mizar-doc` and VC producer tasks as they expose data (plan step 8) |
| SCA-004 | `external_dependency_gap` | Remaining resolver name/label and semantic-import diagnostics need public adoption beyond the implemented E0220–E0224 import subset. | [mizar-resolve task 30](./mizar-resolve/en/todo.md), [mizar-diagnostics consumer adoption](./mizar-diagnostics/en/consumer_adoption_decision.md) (plan step 8) |
| SCA-005 | `source_drift` / `test_gap`; producer dependencies for VC 53 | Algorithm VC/static-safety coverage still lacks source-derived branch/match/range/collection-loop, term-derived/recursive termination, Pick non-emptiness, snapshot/claim, partial-call evidence admission, and ghost-isolation zero-VC integration. | Completed [mizar-vc task 30](./mizar-vc/en/todo.md) is decomposition authority; VC Tasks 42-55 and their `MT10-VC-PV/VC<n>` slices own the bounded families subject to exact Core 42-53 dependencies. Current [ALGDEF-VC53-NORMAL-RETURN](./task_contracts/en/ALGDEF-VC53-NORMAL-RETURN.md) supersedes Task 30's old authority blocker for this slice; authentic source/evidence and proof producers remain prerequisites, and imported evidence remains blocked. Do not invent acceptance or transport inputs. Ghost isolation is a Core-53 static diagnostic/zero-VC boundary, not a proof VC. Task 31 owns only the preceding exact contradiction slice. |
| SCA-006 | `design_drift` (closed) | Phase-16 architecture/internal docs referenced the historical `mizar-extract` split instead of the current `mizar-doc` module names. | Closed by a docs-only sync of architecture 13 and internal 05 EN/JA. |

## Appendix C — July 2026 Audit Follow-Up Inventory

Three pre-implementation audits landed in July 2026. Every finding is bound to
an owner task or a recorded disposition in the owning crate TODO; this is the
roadmap-level index. No finding may be closed by weakening tests or matching
expectations to current behavior.

| Audit | Findings | Owning tasks / dispositions |
|---|---|---|
| [mizar-checker semantic_spec_audit.md](./mizar-checker/en/semantic_spec_audit.md) (2026-07-03, commit `707c95be`) | SSA-001 (critical) through SSA-020; 16-fixture rejection corpus | [mizar-checker tasks 35-49](./mizar-checker/en/todo.md) (step 2 complete; remaining plan steps 4, 5); SSA-018 recorded as a diagnostics-wave lint (step 8), no task; full disposition table in the checker TODO |
| [mizar-kernel soundness_argument.md](./mizar-kernel/en/soundness_argument.md) (2026-07-03, commit `f75af877`) | F1-F9; 23-case reject-first certificate corpus | [mizar-kernel tasks 30-35](./mizar-kernel/en/todo.md); producer side [mizar-vc tasks 28-29](./mizar-vc/en/todo.md), with F1 producer polarity resolved by mizar-vc task 27; consumers [mizar-atp task 29](./mizar-atp/en/todo.md), [mizar-proof task 21](./mizar-proof/en/todo.md), [mizar-cache task 24](./mizar-cache/en/todo.md), [mizar-artifact task 24](./mizar-artifact/en/todo.md); harness [mizar-test task 21](./mizar-test/en/todo.md) for F7, with F8 resolved by mizar-test task 22 (plan steps 1, 3, 4); F4/F5 resolved inside `f75af877` |
| [mizar-core template_encoding_audit.md](./mizar-core/en/template_encoding_audit.md) (2026-07-05, commit `cef7e109`) | F1-F8; original 4-seed encoding corpus plus task 26 F7 inference seeds | spec text for F1-F6/F8 patched inside `cef7e109`; F7 spec decision completed in task 26; task 27 implements explicit-payload reduct/view lowering for F1/F3; task 28 implements explicit-payload type-actual inhabitation gating for F2; task 29 implements explicit-payload scheme-actual compatibility, skipped guard obligation traceability, F6 substitution metadata, and F8 diagnostic-only rejection; task 30 implements explicit-payload sethood plumbing for F5 while source-derived extraction stays external; [mizar-kernel task 35](./mizar-kernel/en/todo.md) re-audits the kernel soundness argument for reduct-view terms with no invariant/corpus-sidecar change; coordination rows in [mizar-checker tasks 36/43](./mizar-checker/en/todo.md) |

`mizar-ir`, `mizar-diagnostics`, `mizar-driver`, and `mizar-doc` reviewed the
audits and recorded a no-crate-owned-task note in their TODOs (they carry no
semantic or proof authority). Completed crates
(session/lexer/syntax/parser/frontend/resolve) are untouched: no audit finding
lands in their scope.

## Appendix D — Resolved And Open Decisions

Resolved decisions (details remain with the linked owners; outstanding
producer/consumer integration is sequenced above):

- **Registration activation gating: resolved.** Checker tasks 19/40 fix the
  accepted-obligation requirement. [Checker's gating boundary](./mizar-checker/en/todo.md#resolved-and-open-decisions)
  retains pending/unverified registrations until real accepted status exists;
  Step 7 integration and Gate S1 remain prerequisites for their consumers.
- **Kernel evidence schema ownership: resolved.** [Kernel tasks 4/23](./mizar-kernel/en/todo.md#resolved-and-open-decisions)
  own legacy certificate and current formula/substitution schemas respectively;
  evidence producers do not become kernel dependencies.
- **Discharge-evidence validation scope: fixed by specification §21.7.1.**
  Trusted built-in status requires kernel acceptance of formula/substitution
  evidence. [Step 7](#step-7--evidence-pipeline-integration--) sequences the
  remaining candidate, acceptance and publication tasks.
- **AST reflection specification transfer: resolved.** Built-in `expr of T`,
  ghost `val` and explicit `by computation` are formalized in §§13.10/20;
  the user-defined tree/`Eval` proposal is withdrawn. Binder notation uses
  shared function abstraction without kernel lambda. See [architecture 15](./architecture/en/15.kernel_certificate_format.md)
  and the [reflected-syntax implementation sequence](#reflected-syntax-implementation--).

- **Lexer span bridging.** `mizar-lexer` stays decoupled; the frontend maps
  lexer byte spans onto `mizar-session::SourceRange` via
  `mizar-frontend::span_bridge`.
- **Parser-assisted lexing contract.** `mizar-frontend` precomputes a
  position-sensitive `ParserLexingPlan`; parser and lexer never interleave.
- **Dot-role surface shape.** The parser resolves dot roles only as far as
  syntax allows (spec [A.2.5](../spec/en/appendix_a.grammar_summary.md));
  scope-dependent finalization is
  [mizar-resolve task 16](./mizar-resolve/en/todo.md), selector validation is
  checker-owned.
- **Resolver module-index seam.** `mizar-resolve` consumes the build-side
  `ModuleIndexProvider` contract; it does not rediscover packages or parse
  dependency-summary artifacts.
- **Syntax tree backend.** Rowan-backed `SurfaceAst` behind the syntax
  builder/event boundary; no raw rowan layout in the parser.
- **Package manifest name spelling.** Lowercase `snake_case` ids, hyphens
  rejected, no normalization; enforced in `mizar-build` planning.
- **Salsa query engine timing.** Target for later query/cache orchestration
  only; owned by [mizar-driver tasks 4-5](./mizar-driver/en/todo.md) and
  [mizar-build task 18](./mizar-build/en/todo.md), not a frontend dependency.
- **`mizar-diagnostics` adoption timing.** E0220–E0224 import descriptors
  and the restricted source-bound continuation are implemented by
  [A40](./task_contracts/en/STEP6A40-IMPORT-DIAGNOSTIC-REGISTRY.md) /
  [A41](./task_contracts/en/STEP6A41-DIAGNOSTIC-IMPORT-CONTINUATION.md) /
  [A49](./task_contracts/en/STEP6A49-SOURCE-BOUND-IMPORT-RESOLUTION.md).
  Broader resolver diagnostics and semantic E0225 publication remain deferred.
- **ModuleSummary reuse timing.** Resolved at resolver task 24 via the
  canonical `mizar-artifact` `ModuleSummary` reader.


## Frozen Task Addenda (Archived)

The frozen per-task addenda that previously followed this appendix moved
verbatim to the archive (compaction batch CPT-01, 2026-09-02):

- [archive/step5_task_addenda.md](./archive/step5_task_addenda.md) — Step 5
  Task 201-218 addenda.
- [archive/checker_task_prerequisite_log.md](./archive/checker_task_prerequisite_log.md)
  — Checker Task 258*/259/260-264/269* prerequisite, implementation, and
  handoff sections plus the mizar-syntax S-026 lower prerequisite.

They are frozen logs, not sequencing authority; concise sequencing status
stays in the crate TODOs and this plan. The Task 269SDC handoff section
below stays in this file: its completion redirect is registered in
[`legacy_compactions.tsv`](./task_contracts/legacy_compactions.tsv)
(batch DOC-269SD-COMPACT) with this file as the declared source.

## Checker Task 269SDC Autonomous Handoff

This completed historical handoff followed the clean Task 269SDP implementation
commit `2ba1ee910aea4939abc26b64a96a113e80c01306`. At that point, origin
divergence `0/1` was a report-only `repo_metadata_conflict`; protected stash
`f65cf4a13752ec380710814a9ac6392ccb9d75d4` was unchanged. Fresh inventory
and independent review selected dependency-ready Task 269SDC only.

SDC freezes a distinct public Given-descendant binding family, exact
`BindingEnv 1/1/0 -> 3/2/0`, inherited child context 2, boxed Typed/Resolved
ownership, seven primary Rust files plus one cfg-test-only predecessor-owner
support file, and four checker/four runner tests. It consumes
the immutable SDP lower and publishes no type, occurrence, Set binding,
capture, fact, proof, obligation, diagnostic, active result, or coverage
credit. Chapter-4/15 `set` disagreement was nonblocking for SDC and remains
blocking for all `z`/`q` work. The subsequent workflow completed the
synchronized 42-file documentation review, gates, and prerequisite commit;
then it fresh-preflighted and implemented SDC alone.

Completion evidence: [central Task-269SDC historical contract](./task_contracts/en/269SDC.md#completion-evidence).
