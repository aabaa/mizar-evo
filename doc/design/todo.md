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
  ordering authority: execute steps top to bottom, and tasks inside a step in
  the listed order unless a task's own `Deps:` line says otherwise.
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

## Completion Gates

This roadmap is sufficient guidance for the verification-focused Mizar-evo
implementation to reach an end-to-end usable state, provided remaining
`external_dependency_gap`, `deferred`, `test_gap`, and `design_drift` records
are promoted into real owner tasks before they are closed. The authority order
in [autonomous_crate_development.md](./autonomous_crate_development.md) must
still agree with the resulting behavior.

| Gate | Completion condition | Plan steps | Primary owners |
|---|---|---|---|
| Source-to-semantics bridge | Real `.miz` inputs pass from frontend output through resolver and checker-owned payload extraction into `ResolvedTypedAst`, with active semantic corpus coverage instead of extraction-gap sentinels. The reserve-only builtin declaration slice is active; AST-wide declarations, attributes, terms, formulas, proof, and broader checker payload extraction remain open. | 2, 4, 5 | `mizar-test`, `mizar-resolve`, `mizar-checker` |
| Core and VC bridge | Checker-derived payloads lower into `CoreIr`, `ControlFlowIr`, and source-derived VC inputs without reconstructing missing source or fabricating registration/proof facts. | 4, 5 | `mizar-checker`, `mizar-core`, `mizar-vc`, `mizar-test` |
| Proof and algorithm verification | Source-derived proof and algorithm obligations flow through VC generation, ATP candidate production, kernel checking, proof policy/status projection, and proof-reuse metadata with active `proof_verification` coverage. | 1, 3, 7 | `mizar-vc`, `mizar-atp`, `mizar-kernel`, `mizar-proof`, `mizar-cache`, `mizar-test` |
| Artifact publication | Verified module, registration, proof-witness, and diagnostic projections are emitted through real `mizar-artifact` store/manifest transactions from producer-owned outputs. | 6 | `mizar-artifact`, `mizar-ir`, `mizar-driver`, producer crates |
| Build orchestration | Clean, incremental, sequential, and parallel driver/build runs agree on externally visible artifacts, proof statuses, cache decisions, and diagnostics for implemented phases. | 6 | `mizar-driver`, `mizar-build`, `mizar-ir`, `mizar-cache`, `mizar-test` |
| User-facing projections | Public diagnostics, LSP features, and documentation rendering consume stable artifacts, diagnostic records, and metadata without owning semantic or proof authority. | 8 | `mizar-diagnostics`, `mizar-lsp`, `mizar-doc`, producer crates |

When all non-parked items in those gates are complete and the relevant broad
verification commands pass, the roadmap supports claiming a source-to-artifact
verification pipeline. Algorithm *execution* is a separate claim from algorithm
verification: before claiming executable algorithm runtime support, promote the
currently deferred MVM/code-extraction/backend specification work from
`spec.en.20.algorithm_and_verification` coverage into explicit owner tasks with
tests and artifact/build integration.

If a future task discovers that a gate cannot be closed by the existing crate
TODOs, update this roadmap in the same change that records the new gap.

## Crate Status

All crates below are workspace members except `mizar-doc` (planned; scaffolding
is its task 1). "Next work" points into the
[Sequential Execution Plan](#sequential-execution-plan).

| Crate | Role | Status | Next work | TODO |
|---|---|---|---|---|
| mizar-session | Source identity, source maps, source loading, build snapshots, retention | [x] complete | — | [todo](./mizar-session/en/todo.md) |
| mizar-lexer | Raw scan, scope skeletons, lexical environments, token disambiguation | [x] complete plus Step 5A.2/5A.3/5A.5 increments | — | [todo](./mizar-lexer/en/todo.md) |
| mizar-syntax | Rowan-backed `SurfaceAst`, trivia, recovery, typed views | [x] historical milestone plus parser Tasks 48/46 increments, S-026 dense views, and Step 5A.1 complete | S-021 remains the sole deferred syntax task | [todo](./mizar-syntax/en/todo.md) |
| mizar-parser | Grammar, Pratt parsing, syntax recovery, parse-only corpus | [x] Tasks 1-48 plus bounded `PARSER-RECOVERY-B1B1P-P1` and Step 5A.4-5A.8 complete | No inferred Task 49; human-owned P-265-47D remains separate | [todo](./mizar-parser/en/todo.md) |
| mizar-frontend | Source loading and phase 1-3 orchestration | [x] prior milestone plus Step 5A.2/5A.3/5A.6/5A.8 increments | — | [todo](./mizar-frontend/en/todo.md) |
| mizar-resolve | Module graph, namespaces, symbols, labels, signatures | [~] prior milestone plus [Step 5C.1](./task_contracts/en/STEP5C1-VARIABLE-SEMANTICS.md) variable resolution | Preserve owner boundaries; next Step 5C resolver work follows the activation map | [todo](./mizar-resolve/en/todo.md) |
| mizar-test | Corpus discovery, expectations, staged model, traceability, harness | [~] prior milestone plus [Step 5C.1](./task_contracts/en/STEP5C1-VARIABLE-SEMANTICS.md), [Step 5C.2](./task_contracts/en/STEP5C2-STRUCTURE-SEMANTICS.md), and [Step 5C.3](./task_contracts/en/STEP5C3-ATTRIBUTE-SEMANTICS.md) semantic activations | Start Step 5C.4 in activation-map order | [todo](./mizar-test/en/todo.md) |
| mizar-checker | Type checking, cluster/registration resolution, overload resolution | [~] prior milestone plus [Step 5C.1](./task_contracts/en/STEP5C1-VARIABLE-SEMANTICS.md), [Step 5C.2](./task_contracts/en/STEP5C2-STRUCTURE-SEMANTICS.md), and [Step 5C.3](./task_contracts/en/STEP5C3-ATTRIBUTE-SEMANTICS.md) semantics | Start Step 5C.4; Task 277B remains not-ready/zero-credit | [todo](./mizar-checker/en/todo.md) |
| mizar-core | Elaboration, binder-normalized core logic, control-flow preparation | [x] core/control-flow milestone, tasks 27-32, plus [Step 5C.2](./task_contracts/en/STEP5C2-STRUCTURE-SEMANTICS.md) bounded normalization | step-5 Tasks 33-53 under the Task-32 graph | [todo](./mizar-core/en/todo.md) |
| mizar-vc | VC IR, VC generation, deterministic pre-ATP discharge | [x] exact source-derived contradiction VC integration complete through task 31 | dependency-paced VC Tasks 32-55; VC 40/53 and S1 gates remain explicit | [todo](./mizar-vc/en/todo.md) |
| mizar-kernel | Trusted certificate parsing and checking | [x] SAT-backed kernel milestone complete | step 4 task 35 resolved; task 32 parked; tasks 30-34 resolved | [todo](./mizar-kernel/en/todo.md) |
| mizar-atp | ATP encoding, backend execution, portfolio candidates | [x] candidate-evidence milestone complete through task 29 | step 7 | [todo](./mizar-atp/en/todo.md) |
| mizar-proof | Proof policy evaluation, status projection, witness selection | [x] policy/status/witness milestone complete through task 21 | step 7 | [todo](./mizar-proof/en/todo.md) |
| mizar-cache | Cache keys, fingerprints, proof reuse, cluster-db storage | [x] internal-cache milestone complete through task 24 | step 7 | [todo](./mizar-cache/en/todo.md) |
| mizar-artifact | Artifact schemas, summaries, store, manifest transactions | [~] schemas/store/manifest complete through task 24 | step 6 (task 17) | [todo](./mizar-artifact/en/todo.md) |
| mizar-ir | IR storage, snapshot handles, sealed output blobs, projections | [x] storage/projection milestone complete | step 6 | [todo](./mizar-ir/en/todo.md) |
| mizar-build | Phase 0 planning, task graph, scheduler, cache seam, commit boundary | [x] milestone complete | step 6 | [todo](./mizar-build/en/todo.md) |
| mizar-driver | Build requests, phase registry, CLI/watch/LSP entry points | [x] session/registry/event milestone complete | step 6 | [todo](./mizar-driver/en/todo.md) |
| mizar-diagnostics | Diagnostic registry, failure records, ordering, rendering | [x] internal milestone complete | step 8 | [todo](./mizar-diagnostics/en/todo.md) |
| mizar-lsp | Editor range mapping now; server features later | [~] range-conversion slice only | step 8 (tasks 1-24) | [todo](./mizar-lsp/en/todo.md) |
| mizar-doc | Documentation rendering and extraction | [ ] planned | step 8 (tasks 1-29) | [todo](./mizar-doc/en/todo.md) |

## Sequential Execution Plan

Revised 2026-07-06 after the July 2026 audit wave (see
[Appendix C](#appendix-c-july-2026-audit-follow-up-inventory)) and the
`mizar-test` task-20 reserve-bridge closeout. Steps 1-3 close audited
soundness holes and settle semantic decisions before further pipeline growth;
steps 4-8 are the implementation waves re-paced around them.

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

### Step 5 — Source-derived semantic bridge [~]

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

**Binding inputs.** The corpus map above, the frontend gap inventory
([semantic_bridge_frontend_gaps.md](./mizar-test/en/semantic_bridge_frontend_gaps.md)
plus [`tests/coverage/audit1_frontend_gaps.tsv`](../../tests/coverage/audit1_frontend_gaps.tsv)),
the audit-1 requirement rows in
[`tests/coverage/spec_trace.toml`](../../tests/coverage/spec_trace.toml), and
the activation-map ledger
[`tests/coverage/step5_activation_map.tsv`](../../tests/coverage/step5_activation_map.tsv)
binding each of the 120 inactive oracle pairs to its owner task below. Tests
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

#### Step 5A — Frontend gap closure [ ]

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

#### Step 5B — Consolidation and pending prerequisites [ ]

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

#### Step 5C — Semantic bridge waves (bounded owner tasks) [ ]

Each task below is bound to concrete audit-1 requirement ids in
[`tests/coverage/spec_trace.toml`](../../tests/coverage/spec_trace.toml);
its activation targets among the 120 inactive oracle pairs are enumerated
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
   `spec.en.06.attributes.*` (7 ids; 8 in total). Targets: 9 pairs (2 blocked by
   G1). Covers attribute definition uniqueness, param-prefix
   declaration/lexicon rejection, struct-qualified disambiguation,
   redefinition coherence, negated attribute chains, non-attribute
   symbol rejection, and argument-position widening. Deps: 5A.2, 5C.2
   (struct-qualified case).
4. [ ] **5C.4 — Modes and property implementations** (`mizar-checker`;
   spec ch. 7). Requirements: `spec.en.07.modes.*` (7 ids). Targets: 8
   pairs (1 blocked by G6). Covers attributed struct radixes, dependent
   modes (`of` parameters, arity mismatch), property implementations
   (`means`/`equals`, unknown property, grammatically mandatory
   correctness), and the unprovable-sethood boundary. Deps: 5A.6, and
   5C.2 for struct radixes; the 5A.8 decision informs but does not block
   (the corpus uses explicit proof blocks).
5. [ ] **5C.5 — Predicates and functors** (`mizar-checker`; spec ch. 9
   and 10). Requirements: `spec.en.09.predicates.*` (7 ids) plus
   `spec.en.10.functors.*` (8 ids). Targets: 17 pairs (10 blocked by
   G1/G2/G6/G9 — the largest gap-coupled family). Covers
   symbolic/phrase definitions, `equals`/`means` styles and definitional
   unfolding, dependent return types, bracket application, properties
   declarations and arity checks, duplicate-signature rejection, and
   redefinition coherence. Deps: 5A.2, 5A.3, 5A.6, 5A.7.
6. [ ] **5C.6 — Notation and modules** (`mizar-resolve`,
   `mizar-frontend`; spec ch. 11 and 12). Requirements:
   `spec.en.11.symbols.*` (2 ids) plus `spec.en.12.modules.*` (4 ids).
   Targets: 7 pairs (3 blocked by G4/G5/G9). Covers synonym/antonym
   declarations with loci checks, branch-form imports, duplicate alias
   and unknown-module rejection, and private visibility. Deeper import
   matrices stay owned by the resolver crate corpus. Deps: 5A.1, 5A.5,
   5A.7.
7. [ ] **5C.7 — Terms** (`mizar-checker`; spec ch. 13). Requirements:
   `spec.en.13.terms.*` (6 ids). Targets: 8 pairs (1 blocked by G5).
   Covers numeral typing, choice (`the`) inhabitation, `qua`
   widening/invalid narrowing, set enumeration, and guarded set
   comprehension (membership pass, unbound-mapper rejection). Deps:
   5A.1.
8. [ ] **5C.8 — Formulas** (`mizar-checker`; spec ch. 14). Requirements:
   `spec.en.14.formulas.*` (7 ids). Targets: 7 pairs (none gap-blocked).
   Covers connective precedence, non-associative `iff` (parse rejection
   and parenthesized pass), quantifier nesting and multi-witness
   existentials, `is` assertions, and unbound-free-variable rejection.
   Deps: none beyond 5B.2.
9. [ ] **5C.9 — Statements and proof organization** (`mizar-checker`;
   spec ch. 15). Requirements: `spec.en.15.statements.*` (8 ids).
   Targets: 8 pairs (1 blocked by G3). Covers
   `consider`/`given`/`hereby`/`now`, `then`/`hence` linking, iterative
   equality, and `per cases` (suppose pass, completeness obligation
   fail). Deps: 5A.4.
10. [ ] **5C.10 — Theorems and proof skeletons** (`mizar-checker`,
    `mizar-vc`, `mizar-proof`; spec ch. 16). Requirements:
    `spec.en.16.theorems.*` (4 ids). Targets: 6 pairs (none
    gap-blocked). Covers thesis tracking (assume-without-antecedent,
    conclusion mismatch, incomplete proof), lemma citation, unknown
    reference labels, and open/assumed theorem status. Deps: 5C.8, 5C.9.
11. [ ] **5C.11 — Clusters and registrations** (`mizar-checker`; spec
    ch. 17). Requirements: `spec.en.17.clusters.*` (7 ids). Targets: 7
    pairs (4 blocked by G1). Covers existential/conditional/functorial
    registrations, reduction registrations, false
    coherence/reducibility rejection, and restricted adjective forms.
    Deps: 5A.2, 5C.3, 5C.10.
12. [ ] **5C.12 — Templates** (`mizar-core`, `mizar-checker`; spec
    ch. 18). Requirements: `spec.en.18.templates.*` (4 ids). Targets: 5
    pairs (4 blocked by G1). Covers type-parameter bounds
    (`extends`, violation rejection), predicate/functor template
    parameters, and instantiation arity. Deps: 5A.2, 5C.5.
13. [ ] **5C.13 — Overload resolution** (`mizar-checker`; spec ch. 19).
    Requirements: `spec.en.19.overload.*` (2 ids). Targets: 2 pairs
    (both blocked by G1). Covers distinct-loci resolution and ambiguity
    rejection. Deps: 5A.2, 5C.5.
14. [ ] **5C.14 — Algorithm verification** (`mizar-checker`,
    `mizar-core`, `mizar-vc`; spec ch. 20, verification constructs
    only). Requirements: `spec.en.20.algorithms.*` (8 ids). Targets: 10
    pairs (none gap-blocked). Covers `var`/`const`/`assert`, contracts
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
execution remains excluded and parked.

### Step 6 — Phase-output publication and orchestration [ ]

Wire real phase-service and publication seams once source-derived semantic
outputs exist. Keep absent producer outputs classified rather than adding
placeholder adapters.

1. [ ] Real phase services and producer publication among `mizar-ir`,
   `mizar-driver`, and `mizar-build` (their current
   `external_dependency_gap` records become owner tasks here; IV-007
   snapshot-freshness obligations apply).
2. [ ] [mizar-artifact task 17](./mizar-artifact/en/todo.md) — phase-15
   emission from real producer projections (see
   [phase15_emission_reevaluation.md](./mizar-artifact/en/phase15_emission_reevaluation.md)).
3. [ ] Clean/incremental/sequential/parallel equivalence:
   [mizar-build task 24](./mizar-build/en/todo.md),
   [mizar-test task 14](./mizar-test/en/todo.md) regression metadata,
   [mizar-driver task 16](./mizar-driver/en/todo.md) (IV-002/IV-003).

Exit: the artifact-publication and build-orchestration completion gates hold
for implemented phases.

### Step 7 — Evidence-pipeline integration [ ]

Wire only task-scoped owner seams among `mizar-atp`, `mizar-proof`,
`mizar-cache`, and `mizar-artifact`, with `mizar-build`, `mizar-ir`, and
`mizar-driver` consuming published results. Keep proof policy in `mizar-proof`,
cache validation in `mizar-cache`, artifact publication in `mizar-artifact`,
and registry/orchestration in `mizar-driver`.

1. [ ] Real ATP backend extraction and portfolio execution in `mizar-atp`
   (policy-deterministic acceptance, IV-004).
2. [ ] Proof cache/witness handoffs: `mizar-proof` reuse metadata export into
   `mizar-cache` proof-reuse validation (IV-005, fail-closed IV-002).
3. [ ] Artifact witness publication from producer outputs in `mizar-artifact`.
4. [ ] Settle the open discharge-evidence validation scope decision
   ([mizar-proof task 6](./mizar-proof/en/todo.md) with `mizar-kernel` /
   `mizar-vc`).

Exit: the proof-and-algorithm-verification completion gate holds with active
`proof_verification` coverage.

### Step 8 — User-facing consumer wave [ ]

Adopt user-facing surfaces only after the owning producers publish stable
diagnostics, metadata, artifacts, and semantic indexes.

1. [ ] [mizar-resolve task 30](./mizar-resolve/en/todo.md) — public resolver
   diagnostic adoption, the first narrow `mizar-diagnostics` consumer seam
   (SCA-004; see
   [consumer_adoption_decision.md](./mizar-diagnostics/en/consumer_adoption_decision.md)).
   The SSA-018 `of`/`over` scope-sensitivity lint is recorded for this wave.
2. [ ] [mizar-lsp tasks 1-24](./mizar-lsp/en/todo.md) — server, snapshots,
   diagnostics, build bridge, metadata, navigation, actions, explanations,
   and the `@show_*`/`@eval` projection audit (SCA-003).
3. [ ] [mizar-doc tasks 1-29](./mizar-doc/en/todo.md) — phase-16 rendering
   and extraction over published artifacts, including the focused module
   specs (SCA-002). `mizar-doc` must not re-run semantic analysis.

Exit: the user-facing-projections completion gate holds.

### Parked and trigger-based work

Not part of the sequential flow; each row records its re-entry trigger.

| Item | Trigger |
|---|---|
| [mizar-kernel task 32](./mizar-kernel/en/todo.md) — solver step-budget deferral (audit F3) | any pinned `batsat` version change (task-24 audit procedure) |
| [mizar-parser task 46](./mizar-parser/en/todo.md) — concrete operator declarations | [x] completed after fresh audit confirmed frontend Task 20 had already met the trigger |
| [mizar-syntax task 21](./mizar-syntax/en/todo.md) — rustdoc summaries | deferred documentation pass |
| MVM / code-extraction / backend runtime work (spec 20) | promote to owner tasks before claiming algorithm *execution* support (see [Completion Gates](#completion-gates)) |

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
| SCA-002 | `todo` | Spec 24 documentation generation has only architecture/internal boundaries and `mizar-doc` TODOs; focused module specs are still unwritten. | [mizar-doc tasks 2, 4, 6, 9, 11, 13, 16, 18, 21, 23, and 29](./mizar-doc/en/todo.md) (plan step 8) |
| SCA-003 | `todo` | `@show_*` and `@eval` have parser/syntax coverage but need end-to-end display/evaluation projection boundaries. | [mizar-lsp task 24](./mizar-lsp/en/todo.md), plus `mizar-doc` and VC producer tasks as they expose data (plan step 8) |
| SCA-004 | `external_dependency_gap` | Resolver name/import/label diagnostics remain crate-local/internal until a real public diagnostic adoption task maps them into stable descriptors. | [mizar-resolve task 30](./mizar-resolve/en/todo.md), [mizar-diagnostics consumer adoption](./mizar-diagnostics/en/consumer_adoption_decision.md) (plan step 8) |
| SCA-005 | `external_dependency_gap`; bounded `spec_gap` for VC 53 | Algorithm VC/static-safety coverage still lacks source-derived branch/match/range/collection-loop, term-derived/recursive termination, Pick non-emptiness, snapshot/claim, partial-call evidence admission, and ghost-isolation zero-VC integration. | Completed [mizar-vc task 30](./mizar-vc/en/todo.md) is decomposition authority; VC Tasks 42-55 and their `MT10-VC-PV/VC<n>` slices own the bounded families subject to exact Core 42-53 dependencies. VC 53 remains blocked because canonical authority does not name the authenticated evidence producer/reference identity/schema, authentication contract/rules, or owning tests; do not invent them. Ghost isolation is a Core-53 static diagnostic/zero-VC boundary, not a proof VC. Task 31 owns only the preceding exact contradiction slice. |
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

Open decisions (block or shape upcoming steps):

- **Registration activation gating: open.** Local registrations must not affect
  inference until their obligations are accepted by verifier policy. Owned by
  [mizar-checker task 19](./mizar-checker/en/todo.md); the SSA-006 contract
  language is [mizar-checker task 40](./mizar-checker/en/todo.md) (plan step
  2); revisit when `mizar-vc` / `mizar-proof` integration lands (step 7).
- **Certificate schema ownership: open.** Default candidate: `mizar-kernel`
  owns certificate schema types and `mizar-atp` depends on the kernel to
  construct candidates, so the kernel never depends on evidence producers.
  Owned by [mizar-kernel task 4](./mizar-kernel/en/todo.md).
- **Discharge-evidence validation scope: open.** Decide whether `mizar-vc`
  pre-ATP discharge evidence is kernel-replayed or accepted as policy-level
  evidence. Owned by [mizar-proof task 6](./mizar-proof/en/todo.md) with
  `mizar-kernel`, and tracked in [mizar-kernel](./mizar-kernel/en/todo.md) and
  [mizar-vc](./mizar-vc/en/todo.md) (plan step 7).

Resolved decisions (kept for reference; details live in the linked docs):

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
- **`mizar-diagnostics` adoption timing.** Deferred at resolver task 13;
  resolver failures stay crate-local until diagnostic code ownership is
  specified (plan step 8).
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
