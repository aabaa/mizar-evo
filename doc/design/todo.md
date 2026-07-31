# Implementation Roadmap (Crate Sequencing)

> Canonical language: English. This is the top-level index for crate-level work
> ordering. Per-crate TODOs carry detailed module checklists and have Japanese
> companions under each crate's `ja/` directory when that companion exists.

This document records the current implementation order across crates. It
complements [README.md](./README.md) (design layout), the pipeline definition in
[architecture/en/00.pipeline_overview.md](./architecture/en/00.pipeline_overview.md),
and the crate ownership map in
[internal/en/07.crate_module_layout.md](./internal/en/07.crate_module_layout.md).

## How To Read This Document

- The [Sequential Execution Plan](#sequential-execution-plan) is the single
  ordering authority: execute steps top to bottom, and tasks inside a step in
  the listed order unless a task's own `Deps:` line says otherwise.
- Each entry names an owner task in a crate TODO; the crate TODO carries the
  full task text, acceptance criteria, and verification commands. This file
  never restates them — follow the link.
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
| mizar-lexer | Raw scan, scope skeletons, lexical environments, token disambiguation | [x] complete | — | [todo](./mizar-lexer/en/todo.md) |
| mizar-syntax | Rowan-backed `SurfaceAst`, trivia, recovery, typed views | [x] historical milestone plus parser Tasks 48/46 increments and S-026 dense views complete | S-021 remains the sole deferred syntax task | [todo](./mizar-syntax/en/todo.md) |
| mizar-parser | Grammar, Pratt parsing, syntax recovery, parse-only corpus | [x] Tasks 1-48 plus bounded `PARSER-RECOVERY-B1B1P-P1` complete; historical post-Task-46 score 99/100 | no inferred Task 49; human-owned P-265-47D remains separate | [todo](./mizar-parser/en/todo.md) |
| mizar-frontend | Source loading and phase 1-3 orchestration | [x] prior milestone plus `PARSER-RECOVERY-B1B1P-P1-FE` regression complete | — | [todo](./mizar-frontend/en/todo.md) |
| mizar-resolve | Module graph, namespaces, symbols, labels, signatures | [x] complete through task 29 | step 8 task 30; independent step-5 task 31 | [todo](./mizar-resolve/en/todo.md) |
| mizar-test | Corpus discovery, expectations, staged model, traceability, harness | [~] foundation complete through task 22; Tasks 265-268, Core-31, and Checker Tasks 248-257C3 consumer increments complete | step 5 task 10, dependency-paced later consumers beginning with Checker Task 258, future `MT10-FS`/`MT10-AS`, and five Core-32 consumer increments | [todo](./mizar-test/en/todo.md) |
| mizar-checker | Type checking, cluster/registration resolution, overload resolution | [x] explicit-payload milestone, bridges through Task 246, Tasks 266-268 final handoff, Task 247 decomposition, and Tasks 248-257C3 source-payload/composition producers complete | Checker Task 258, then Tasks 259-264/269-279 in dependency order; task 49 dependency-gated on blocked external slices | [todo](./mizar-checker/en/todo.md) |
| mizar-core | Elaboration, binder-normalized core logic, control-flow preparation | [x] core/control-flow milestone, tasks 27-32 complete | step-5 Tasks 33-53 under the Task-32 graph | [todo](./mizar-core/en/todo.md) |
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

### Step 5 — Source-derived semantic bridge [ ]

Widen real `.miz` source-derived payload extraction beyond the active
reserve-only builtin declaration slice (`mizar-test` tasks 16-20 plus the
post-task-20 resolver R-G007 and SymbolEnv assertion increments, with
`mizar-checker` task 48 owning the checker-side syntax-free reserve producer
seam, task 50 adding the same-module attributed builtin reserve diagnostic
slice, task 51 adding the same-module local mode reserve diagnostic slice, and
task 52 adding the same-module local structure reserve diagnostic slice, and
task 53 adding the same-module attributed local structure reserve diagnostic
slice, and task 54 adding the same-module attributed local mode reserve
diagnostic slice, and task 55 adding the same-module bare local mode expansion
pass slice, and task 56 adding the one-edge same-module local-mode expansion
chain pass/gap slice, task 57 adding the same-module local mode expansion
to local structure RHS evidence-gap slice, task 58 adding the same-module
local mode attributed-builtin RHS evidence-gap slice, and task 59 adding the
same-module attributed local mode reserve evidence-gap slice when a real direct
bare-builtin mode expansion is available, and task 60 adding the same-module
attributed local mode structure-RHS evidence-gap slice when the real expansion
is a direct local structure RHS, and task 61 adding the same-module attributed
local mode attributed-builtin RHS evidence-gap slice when the real expansion
is a direct attributed builtin RHS, and task 62 adding the one-edge bare local
mode structure-RHS chain evidence-gap slice, and task 63 adding the one-edge
bare local mode attributed-builtin-RHS chain evidence-gap slice, task 64 added
the attributed local mode bare-builtin chain evidence-gap slice, task 65 added
the attributed local mode structure-RHS chain evidence-gap slice, and task 66
added the attributed local mode attributed-builtin-RHS chain
evidence-gap slice, task 67 added the structure-qualified attribute
extraction-gap boundary slice, task 68 added the argument-bearing local mode
reserve extraction-gap boundary slice, and task 69 added the argument-bearing
local structure reserve extraction-gap boundary slice, and task 70 added the
   bracket-form local mode reserve extraction-gap boundary slice, task 71 added
   the bracket-form local structure reserve extraction-gap boundary slice, task
   72 added the two-edge bare local-mode chain expansion pass slice, task 73
   promoted the same source-derived bare local-mode chain seam to three edges,
   task 74 replaced the temporary chain-depth guard with an AST-bounded
   structural bare local-mode chain rule, task 75 added a source-derived
   lower-stage active-range boundary for a reserve head that references a later
   local mode declaration, task 76 added the matching lower-stage boundary for a
   later local structure declaration, task 77 added the corresponding
   lower-stage boundary for a later local attribute declaration, task 78 added
   the imported structure reserve-head extraction-gap boundary, task 79 added
   the imported mode reserve-head extraction-gap boundary, task 80 added the
   imported attribute reserve extraction-gap boundary, and task 81 added the
   argument-bearing local attribute reserve extraction-gap boundary plus the
   resolver suffix-primary projection needed for parameterized local attributes,
   and task 82 promotes the imported mode reserve-head boundary just far enough
   to pass real imported mode symbol provenance/type-head payloads to the
   checker missing mode-expansion diagnostic while keeping imported expansions
   and downstream payloads deferred, and task 83 promotes the imported
   structure `R` reserve-head boundary just far enough to pass real imported
   structure symbol provenance/type-head payloads to the checker evidence-query
   diagnostic while keeping imported module AST extraction, structure evidence,
   and downstream payloads deferred, and task 97 promotes the imported
   structure `TypeCaseStruct` reserve-head boundary to the same real imported
   structure provenance/type-head evidence-query diagnostic while keeping
   imported module AST extraction, structure evidence, broader imported
   structures, and downstream payloads deferred, and task 84 promotes the imported
   attribute `TypeCaseAttr` reserve boundary just far enough to pass real
   imported attribute provenance/`AttributeInput` payloads to the checker
   evidence-query diagnostic while keeping imported module AST extraction,
   attributed-type evidence, generic imported attributes, owner provenance,
   arguments, and downstream payloads deferred, and task 85 promotes the
   imported negative `empty`/builtin-`set` reserve boundary just far enough to
   pass real imported attribute provenance/negative `AttributeInput` payloads
   to the checker evidence-query diagnostic, task 116 promotes the matching
   positive `empty`/builtin-`set` sidecar to a real positive `AttributeInput`
   payload and the same evidence-query diagnostic, and task 171 promotes the
   matching negative `empty`/builtin-`object` sidecar to a real negative
   `AttributeInput` payload and the same evidence-query diagnostic, while
   positive `empty object`, imported attributes on symbol heads, imported
   module AST extraction, attributed-type evidence, owner provenance,
   arguments, and downstream payloads remain on extraction/deferred gaps,
   task 86 records a formula-only theorem source on the active
   source-to-checker boundary, task 115 supersedes that exact
   `FormulaPayloadBoundary: thesis` source by passing a checker recovery
   `FormulaInput`, task 117 supersedes that recovery marker by passing a real
   `FormulaKind::Thesis` payload and also promotes the exact
   connective/quantifier theorem's two `contradiction` constants to real
   `FormulaKind::Contradiction` payloads while keeping formula semantics
   deferred, task 106 supersedes task 87 for the exact
   term-bearing builtin equality theorem source by passing real checker
   term/formula payloads before failing on missing numeric type payloads and
   partial formula checking, task 110 supersedes task 98 for the exact imported
   predicate/functor theorem formula by passing real checker term/formula
   payloads before failing on missing numeric/signature payloads and partial
   formula checking, task 108 supersedes task 100 for
   the exact builtin membership theorem source by passing real checker
   term/formula payloads before failing on missing numeric type payloads and
   partial formula checking, task 107 supersedes task 101 for
   the exact builtin inequality theorem source by passing real checker
   term/formula payloads before failing on missing numeric type payloads and
   partial formula checking, task 118 tightens the shared task 106/107/108
   builtin-binary producer so only exact direct theorem tokens
   `theorem <label> : ;` select those bridges, task 119 adds an exact
   no-diagnostic reserved-variable
   equality slice by resolving both `x` identifier terms through the real
   reserve `BindingEnv`, reusing the written builtin `set` type for term result
   and equality expected-type payloads, and checking formula well-formedness
   without facts or theorem acceptance, task 120 adds the matching exact
   reserved-variable membership well-formedness pass with two source-derived
   result roles and only the right operand's expected-`set` role, also without
   facts or theorem acceptance, task 121 adds the exact reserved-variable
   inequality pre-desugaring well-formedness pass with two linked result/expected
   role pairs and no facts, task 122 repairs the checker reflexive type-assertion
   admissibility gate and adds the exact reserved-variable `x is set` pass from
   independent reserve-result and formula-asserted source inputs without facts,
   task 123 adds the exact distinct-binding equality pass for `x = y` from one
   `reserve x, y for set` segment while preserving both binding identities and
   the shared written type range without facts or theorem acceptance,
   task 124 adds the exact multiple-reserve-declaration equality pass for
   `reserve x for set; reserve y for set; ... x = y`, preserving both distinct
   written type ranges in four pre-normalization inputs while allowing their
   identical builtin `set` semantics to intern to one deterministic normalized
   type, also without facts or theorem acceptance,
   task 125 adds the exact heterogeneous membership pass for
   `reserve x for object; reserve y for set; ... x in y`, retaining distinct
   object/set result identities and only the right expected-`set` role without
   membership facts, coercion evidence, or theorem acceptance,
   task 126 adds the exact direct-local-mode equality pass by combining the
   real task-55 bare-set expansion producer with task 119's reserved-variable
   equality consumer: four raw result/expected inputs retain the written local
   mode while one normalized builtin-`set` identity is anchored at the real
   expansion RHS, without mode-definition acceptance, facts, or theorem
   acceptance,
   task 127 adds the exact one-edge local-mode-chain equality pass by combining
   task 56's real `ChainModeFormula -> BaseModeFormula -> set` producer with
   task 126's equality consumer, retaining four raw outer-mode inputs while
   both real links normalize to one terminal-RHS builtin-`set` identity without
   mode-definition acceptance, facts, or theorem acceptance,
   task 128 adds the exact direct local-object-mode equality pass by combining
   task 55's real `LocalObjectMode -> object` producer with task 126's equality
   consumer, retaining four raw object-mode inputs while the real expansion
   normalizes to one builtin-`object` identity anchored at its RHS without
   mode-definition acceptance, facts, or theorem acceptance,
   task 129 adds the exact one-edge local-object-mode-chain equality pass by
   combining task 56's real object-terminal chain producer with tasks 127/128's
   equality/object consumers, retaining four raw outer-mode inputs while both
   links normalize to one terminal-RHS builtin-object identity without
   declaration acceptance, facts, or theorem acceptance,
   task 130 adds the exact direct local-mode inequality pass by combining task
   55's real `LocalModeInequality -> set` producer with task 121's
   pre-desugaring inequality consumer, retaining four raw mode inputs and one
   real RHS-anchored builtin-set identity without desugaring, facts, or theorem
   acceptance,
   task 131 adds the exact direct local-object-mode inequality pass by combining
   task 55's real `LocalObjectModeInequality -> object` producer with the task
   121/130 pre-desugaring inequality consumer, retaining four raw object-mode
   inputs and one real RHS-anchored builtin-object identity without mode
   declaration acceptance/inhabitation, desugaring, facts, or theorem
   acceptance,
   task 132 adds the exact one-edge local-mode-chain inequality pass by
   combining task 56/127's real
   `ChainModeInequality -> BaseModeInequality -> set` producer with task
   121/130's pre-desugaring inequality consumer, retaining four raw outer-mode
   inputs while both real links normalize to one terminal-RHS builtin-set
   identity without declaration acceptance/inhabitation, desugaring, facts, or
   theorem acceptance,
   task 133 adds the exact one-edge local-object-mode-chain inequality pass by
   combining task 129's real
   `ChainObjectModeInequality -> BaseObjectModeInequality -> object` producer
   with task 131's pre-desugaring inequality consumer, retaining four raw
   outer-mode inputs while both real links normalize to one terminal-RHS
   builtin-object identity without declaration acceptance/inhabitation,
   desugaring, facts, or theorem acceptance,
   task 134 adds the exact two-edge local-mode-chain equality pass by combining
   task 72's real
   `OuterTwoEdgeModeEquality -> MiddleTwoEdgeModeEquality -> BaseTwoEdgeModeEquality -> set`
   producer with task 127's equality consumer, retaining four raw outer-mode
   inputs while all three real links normalize to one terminal-RHS builtin-set
   identity without declaration acceptance/inhabitation, closure/order, facts,
   or theorem acceptance,
   task 135 adds the exact two-edge local-object-mode-chain equality pass by
   combining task 72's real
   `OuterTwoEdgeObjectModeEquality -> MiddleTwoEdgeObjectModeEquality -> BaseTwoEdgeObjectModeEquality -> object`
   producer with task 134's equality consumer and builtin-object terminal
   support, retaining four raw outer-mode inputs while all three real links
   normalize to one terminal-RHS builtin-object identity without declaration
   acceptance/inhabitation, closure/order, facts, or theorem acceptance,
   task 136 adds the exact two-edge local-mode-chain inequality pass by
   combining task 72's real
   `OuterTwoEdgeModeInequality -> MiddleTwoEdgeModeInequality -> BaseTwoEdgeModeInequality -> set`
   producer with task 132's pre-desugaring inequality consumer, retaining four
   raw outer-mode inputs while all three real links normalize to one
   terminal-RHS builtin-set identity without mode declaration
   acceptance/inhabitation, inequality desugaring, closure/order, facts, or
   theorem acceptance,
   task 137 adds the exact two-edge local-object-mode-chain inequality pass by
   combining task 72's real
   `OuterTwoEdgeObjectModeInequality -> MiddleTwoEdgeObjectModeInequality -> BaseTwoEdgeObjectModeInequality -> object`
   producer with task 133's builtin-object pre-desugaring inequality consumer,
   retaining four raw outer-mode inputs while all three real links normalize to
   one terminal-RHS builtin-object identity without declaration
   acceptance/inhabitation, inequality desugaring, closure/order, facts, or
   theorem acceptance,
   task 138 adds the exact direct local-mode reserved-variable normalized-
   reflexive type-assertion pass by combining task 55's real
   `LocalModeTypeAssertion -> set` producer with task 122's type-assertion
   consumer, retaining the raw local-mode subject and independent formula-side
   builtin-set asserted source while the one real expansion normalizes both to
   one terminal-RHS builtin-set identity without mode declaration
   acceptance/inhabitation, general reachability/widening/`qua`, facts, or
   theorem acceptance,
   task 139 adds the exact direct local-mode left reserved-variable membership
   pass by combining task 55's real `LocalModeMembership -> set` producer with
   task 120's right-only expected-set membership consumer and task 125's
   two-binding distinct-source form, retaining the raw local-mode left result
   and independent right result/expected-set provenance while all three roles
   intern to one terminal-RHS builtin-set identity without mode declaration
   acceptance/inhabitation, membership truth/facts, closure/order, or theorem
   acceptance,
   task 140 adds the exact direct local-object-mode left reserved-variable
   membership pass by combining task 55's real
   `LocalObjectModeMembership -> object` producer with task 125's right-only
   expected-set two-binding membership consumer, retaining the raw local
   object-mode left result and independent right result/expected-set
   provenance while the one real expansion produces a terminal-RHS
   builtin-object identity distinct from the explicit-reserve builtin-set
   identity, without mode declaration acceptance/inhabitation, membership
   truth/facts, object/set coercion, closure/order, or theorem acceptance,
   task 141 adds the exact one-edge local-mode-chain left reserved-variable
   membership pass by combining task 56's real
   `ChainModeMembership -> BaseModeMembership -> set` producer with task 139's
   right-only expected-set two-binding membership consumer, retaining the raw
   outer-mode left result and independent right result/expected-set provenance
   while both real expansion links recursively normalize the left, the right
   roles normalize directly, and all three intern to one terminal-RHS builtin-
   set identity without mode declaration acceptance/inhabitation, membership
   truth/facts, closure/order, or theorem acceptance,
   task 142 adds the exact one-edge local-object-mode-chain left reserved-
   variable membership pass by combining task 56's real
   `ChainObjectModeMembership -> BaseObjectModeMembership -> object` producer
   with tasks 140/141's right-only expected-set two-binding membership
   consumer, retaining the raw outer-mode left result and independent right
   result/expected-set provenance while both real expansion links recursively
   normalize the left to a terminal-RHS builtin-object identity and the right
   roles normalize directly to a distinct explicit-reserve builtin-set
   identity, without mode declaration acceptance/inhabitation, membership
   truth/facts, object/set coercion, closure/order, or theorem acceptance,
   task 143 adds the exact two-edge local-mode-chain left reserved-variable
   membership pass by combining task 72's real
   `OuterTwoEdgeModeMembership -> MiddleTwoEdgeModeMembership -> BaseTwoEdgeModeMembership -> set`
   producer with tasks 125/139/141's right-only expected-set two-binding
   membership consumer, retaining the raw outer-mode left result and
   independent right result/expected-set provenance while all three real
   expansions recursively normalize the left and all roles intern to one
   terminal-RHS builtin-set identity, without mode declaration
   acceptance/inhabitation, membership truth/facts, closure/order, or theorem
   acceptance,
   task 144 adds the exact two-edge local-object-mode-chain left reserved-
   variable membership pass by combining task 72's real
   `OuterTwoEdgeObjectModeMembership -> MiddleTwoEdgeObjectModeMembership -> BaseTwoEdgeObjectModeMembership -> object`
   producer with tasks 125/140/142/143's right-only expected-set two-binding
   membership consumer, retaining the raw outer-mode left result and
   independent right result/expected-set provenance while all three real
   expansions recursively normalize the left to a terminal-RHS builtin-object
   identity and the right roles normalize directly to a distinct explicit-
   reserve builtin-set identity, without mode declaration acceptance/
   inhabitation, membership truth/facts, object/set coercion, closure/order, or
   theorem acceptance,
   task 145 adds the exact direct local-object-mode reserved-variable
   normalized-reflexive type assertion pass by combining task 55's real
   `LocalObjectModeTypeAssertion -> object` producer with tasks 122/138's
   identifier-result and independently formula-anchored asserted-type
   consumer, retaining raw subject and asserted-type provenance while one real
   expansion normalizes both inputs to a terminal-RHS builtin-object identity
   before one inferred term and one fact-free checked type assertion, without
   mode declaration acceptance/inhabitation, formula-side local-mode asserted
   heads, general reachability/widening/`qua`, object/set coercion,
   closure/order, or theorem acceptance,
   task 146 adds the exact one-edge local-mode-chain reserved-variable
   normalized-reflexive type assertion pass by combining task 56's real
   `ChainModeTypeAssertion -> BaseModeTypeAssertion -> set` producer with tasks
   122/138's identifier-result and independently formula-anchored asserted-
   type consumer, retaining raw outer-mode subject and asserted-type provenance
   while both real expansions recursively normalize both inputs to a terminal-
   RHS builtin-set identity before one inferred term and one fact-free checked
   type assertion, without mode declaration acceptance/inhabitation, formula-
   side local-mode asserted heads, general reachability/widening/`qua`,
   closure/order, or theorem acceptance,
   task 147 adds the exact one-edge local-object-mode-chain reserved-variable
   normalized-reflexive type assertion pass by combining task 56's real
   `ChainObjectModeTypeAssertion -> BaseObjectModeTypeAssertion -> object`
   producer with tasks 122/145/146's identifier-result and independently
   formula-anchored asserted-type consumer, retaining raw outer-mode subject
   and asserted-type provenance while both real expansions recursively
   normalize both inputs to a terminal-RHS builtin-object identity before one
   inferred term and one fact-free checked type assertion, without mode
   declaration acceptance/inhabitation, formula-side local-mode asserted
   heads, general reachability/widening/`qua`, object/set coercion, closure/
   order, or theorem acceptance,
   task 148 adds the exact two-edge local-mode-chain reserved-variable
   normalized-reflexive type assertion pass by combining task 72's real
   `OuterTwoEdgeModeTypeAssertion -> MiddleTwoEdgeModeTypeAssertion -> BaseTwoEdgeModeTypeAssertion -> set`
   producer with tasks 122/146/147's identifier-result and independently
   formula-anchored asserted-type consumer, retaining raw outer-mode subject
   and asserted-type provenance while all three real expansions recursively
   normalize both inputs to a terminal-RHS builtin-set identity before one
   inferred term and one fact-free checked type assertion, without mode
   declaration acceptance/inhabitation, formula-side local-mode asserted
   heads, general reachability/widening/`qua`, closure/order, or theorem
   acceptance,
   task 149 adds the exact two-edge
   local-object-mode-chain reserved-variable normalized-reflexive type
   assertion pass, combining task 72's real
   `OuterTwoEdgeObjectModeTypeAssertion -> MiddleTwoEdgeObjectModeTypeAssertion -> BaseTwoEdgeObjectModeTypeAssertion -> object`
   producer with tasks 122/145/147/148's identifier-result and independently
   formula-anchored asserted-type consumer. The intended slice retains raw
   outer-mode subject and asserted-type provenance while all three real
   expansions recursively normalize both inputs to a terminal-RHS builtin-
   object identity before one inferred term and one fact-free checked type
   assertion, without mode declaration acceptance/inhabitation, formula-side
   local-mode asserted heads, general reachability/widening/`qua`, object/set
   coercion, closure/order, or theorem acceptance,
   task 150 adds the exact three-edge local-mode-chain reserved-variable
   normalized-reflexive type assertion pass by combining task 73's real
   `OuterThreeEdgeModeTypeAssertion -> MiddleThreeEdgeModeTypeAssertion -> InnerThreeEdgeModeTypeAssertion -> BaseThreeEdgeModeTypeAssertion -> set`
   producer with tasks 122/148/149's identifier-result and independently
   formula-anchored asserted-type consumer. The active slice retains raw
   outer-mode subject and asserted-type provenance while all four real
   expansions recursively normalize both inputs to a terminal-RHS builtin-set
   identity before one inferred term and one fact-free checked type assertion,
   without mode declaration acceptance/inhabitation, formula-side local-mode
   asserted heads, general reachability/widening/`qua`, closure/order, or
   theorem acceptance,
   task 151 adds the exact three-edge local-object-mode-chain reserved-variable
   normalized-reflexive type assertion pass by combining task 73's real
   `OuterThreeEdgeObjectModeTypeAssertion -> MiddleThreeEdgeObjectModeTypeAssertion -> InnerThreeEdgeObjectModeTypeAssertion -> BaseThreeEdgeObjectModeTypeAssertion -> object`
   producer with tasks 122/149/150's identifier-result and independently
   formula-anchored asserted-type consumer. The active slice retains raw
   outer-mode subject and asserted-type provenance while all four real
   expansions recursively normalize both inputs to a terminal-RHS builtin-
   object identity before one inferred term and one fact-free checked type
   assertion, without mode declaration acceptance/inhabitation, formula-side
   local-mode asserted heads, general reachability/widening/`qua`, object/set
   coercion, closure/order, or theorem acceptance,
   task 152 adds the exact four-edge local-mode-chain reserved-variable
   normalized-reflexive type assertion pass by combining task 74's real
   `TooDeepFourEdgeModeTypeAssertion -> OuterFourEdgeModeTypeAssertion -> MiddleFourEdgeModeTypeAssertion -> InnerFourEdgeModeTypeAssertion -> BaseFourEdgeModeTypeAssertion -> set`
   producer with tasks 122/150/151's identifier-result and independently
   formula-anchored asserted-type consumer. The active slice retains raw
   outermost-mode subject and asserted-type provenance while all five real
   expansions recursively normalize both inputs to a terminal-RHS builtin-set
   identity before one inferred term and one fact-free checked type assertion,
   without mode declaration acceptance/inhabitation, formula-side local-mode
   asserted heads, general reachability/widening/`qua`, closure/order, or
   theorem acceptance,
   task 153 adds the exact four-edge local-object-mode-chain reserved-variable
   normalized-reflexive type assertion pass by combining task 74's real
   `TooDeepFourEdgeObjectModeTypeAssertion -> OuterFourEdgeObjectModeTypeAssertion -> MiddleFourEdgeObjectModeTypeAssertion -> InnerFourEdgeObjectModeTypeAssertion -> BaseFourEdgeObjectModeTypeAssertion -> object`
   producer with tasks 122/151/152's identifier-result and independently
   formula-anchored asserted-type consumer. The active slice retains raw
   outermost-mode subject and asserted-type provenance while all five real
   expansions recursively normalize both inputs to a terminal-RHS builtin-
   object identity before one inferred term and one fact-free checked type
   assertion, without mode declaration acceptance/inhabitation, formula-side
   local-mode asserted heads, general reachability/widening/`qua`, object/set
   coercion, closure/order, or theorem acceptance,
   task 154 adds the exact active three-edge local-mode-chain reserved-
   variable equality contract by combining task 73's real
   `OuterThreeEdgeModeEquality -> MiddleThreeEdgeModeEquality -> InnerThreeEdgeModeEquality -> BaseThreeEdgeModeEquality -> set`
   producer with task 134's equality consumer. The active slice retains four
   raw outer-mode result/expected inputs, resolves both operands to
   `BindingId(0)` at ordinals 1 and 2, and consumes all four real expansions to
   normalize every role to one terminal-RHS builtin-set identity before two
   inferred variables and one fact/deferred-free checked equality, without
   mode declaration acceptance/inhabitation, equality truth/facts, closure/
   order, or theorem acceptance. The production route, full near-miss/
   corruption matrix, and real frontend/resolver sidecar now guard the active
   105th case,
   task 155 adds the exact active three-edge local-object-mode-chain
   reserved-variable equality contract by combining task 73's real
   `OuterThreeEdgeObjectModeEquality -> MiddleThreeEdgeObjectModeEquality -> InnerThreeEdgeObjectModeEquality -> BaseThreeEdgeObjectModeEquality -> object`
   producer with task 135's equality consumer. The active slice retains four
   raw outer-mode result/expected inputs, resolves both operands to
   `BindingId(0)` at ordinals 1 and 2, and consumes all four real expansions to
   normalize every role to one terminal-RHS builtin-object identity before two
   inferred variables and one fact/deferred-free checked equality, without
   mode declaration acceptance/inhabitation, object/set coercion, equality
   truth/facts, closure/order, or theorem acceptance. The production route,
   full near-miss/corruption matrix, and real frontend/resolver sidecar now
   guard the active 106th case,
   task 156 adds the exact active three-edge local-mode-chain reserved-
   variable inequality contract by combining task 73's real
   `OuterThreeEdgeModeInequality -> MiddleThreeEdgeModeInequality -> InnerThreeEdgeModeInequality -> BaseThreeEdgeModeInequality -> set`
   producer with task 136's pre-desugaring inequality consumer. The active
   slice retains four raw outer-mode result/expected inputs, resolves both
   operands to `BindingId(0)` at ordinals 1 and 2, and consumes all four real
   expansions to normalize every role to one terminal-RHS builtin-set identity
   before two inferred variables and one fact/deferred-free pre-desugaring
   checked inequality, without mode declaration acceptance/inhabitation,
   inequality desugaring, truth/facts, closure/order, or theorem acceptance.
   The production route, full near-miss/corruption matrix, and real frontend/
   resolver sidecar now guard the active 107th case,
   task 157 adds the exact three-edge local-object-mode-chain
   reserved-variable inequality contract by combining task 73's real
   `OuterThreeEdgeObjectModeInequality -> MiddleThreeEdgeObjectModeInequality -> InnerThreeEdgeObjectModeInequality -> BaseThreeEdgeObjectModeInequality -> object`
   producer with task 137's builtin-object pre-desugaring inequality consumer.
   The active slice retains four raw outer-mode result/expected inputs,
   resolves both operands to `BindingId(0)` at ordinals 1 and 2, and consumes
   all four real expansions to normalize every role to one terminal-RHS
   builtin-object identity before two inferred variables and one fact/deferred-
   free pre-desugaring checked inequality, without mode declaration acceptance/
   inhabitation, object/set coercion, inequality desugaring, truth/facts,
   closure/order, or theorem acceptance. The production route, full near-miss/
   corruption matrix, and real frontend/resolver sidecar now guard the active
   108th case,
   task 158 adds the exact active three-edge local-mode-chain left reserved-
   variable membership contract by combining task 73's real
   `OuterThreeEdgeModeMembership -> MiddleThreeEdgeModeMembership -> InnerThreeEdgeModeMembership -> BaseThreeEdgeModeMembership -> set`
   producer with task 143's two-binding right-only expected-set membership
   consumer. The active slice retains the raw outer-mode left result and
   independent explicit-set right result/sole expected input, has no left
   expected type, resolves `x/y` to `BindingId(0/1)` at ordinals 2/3, and
   consumes all four real expansions to normalize all three roles to one
   terminal-RHS builtin-set identity before two inferred variables and one
   fact/deferred-free checked membership with exactly one right-owned
   constraint. The fixture, expectation, trace row, production route, full near-
   miss/corruption matrix, and real frontend/resolver sidecar now guard the
   active contract as the 109th case,
   task 159 adds an exact active distinct-binding shared-reserve membership
   contract by composing task 123's one-item/two-binding/shared-range reserve
   producer with tasks 120/125's right-only expected-set membership consumer.
   The active slice preserves `BindingId(0/1)` at ordinals 2/3 and one written
   builtin-set range across both bindings plus the left result, right result,
   and sole right expected input, has no left expected input, and requires one
   shared-source-anchored builtin-set identity, two inferred variables, one
   fact/deferred-free checked membership, and exactly one right-owned
   constraint. Production routing, the full near-miss/corruption matrix, and a
   real frontend/resolver sidecar now guard the exact contract as the 110th
   active case, and Chapter 3 receives no new credit,
   task 160 adds the exact active distinct-binding shared-reserve inequality
   contract by composing task 123's one-item/two-binding/shared-range reserve
   producer with task 121's pre-desugaring inequality consumer. It preserves
   `BindingId(0/1)` at ordinals 2/3 and the same written builtin-set range
   across both bindings and all four operand result/expected roles, and
   requires one shared-source-anchored identity, two inferred variables, one
   fact/deferred-free checked inequality, and two ordered operand-owned
   constraints. Production routing, the full near-miss/corruption matrix, and
   a real frontend/resolver sidecar now guard the exact contract as the 111th
   active case, and Chapter 3 receives no new credit,
   task 161 adds the exact active multiple-reserve-declaration inequality
   contract by composing task 124's two-item/two-binding/distinct-written-range
   producer with task 160's pre-desugaring inequality consumer. It retains the
   first reserve range across the left result/expected roles and the second
   range across the right roles, then interns one canonical builtin-set
   identity anchored at the earlier x range before two inferred variables and
   one fact/deferred-free checked inequality with two ordered constraints.
   Production routing, full near-miss/corruption coverage, and a real sidecar
   now guard the exact contract as the 112th active case, and Chapter 3 receives
   no new credit,
   task 162 adds the exact active multiple-reserve-declaration
   membership seam by composing task 124's distinct-written-range producer with
   tasks 120/159's right-only expected-set membership consumer. The active
   contract retains the first range on the left result, the second range on the
   right result and sole right expected input, has no left expected input, and
   interns all three roles to one earlier-x-anchored builtin-set identity before
   two inferred variables and one checked membership with exactly one right-owned
   constraint. Production routing, full near-miss/corruption coverage, and a
   real sidecar now guard the exact contract as the 113th active case, and
   Chapter 3 receives no new credit,
   task 163 adds the exact active three-edge local-object-mode-chain left
   membership seam. It composes tasks 73/151/155/157's real four-expansion
   object-terminal producer with task 144's real object-left/set-right
   membership consumer, and requires raw left plus independent explicit-set
   right provenance, ordinal 2/3 `BindingId(0/1)`, two distinct normalized
   identities, no left expected input, two inferred variables, and one fact-
   free checked membership with exactly one right-owned constraint. Production
   routing, full near-miss/corruption coverage, and the real sidecar guard
   active case 114; no coercion/truth/closure/theorem/proof/Core/ControlFlow/VC
   credit is claimed,
   task 164 adds the exact active four-edge set-terminal local-mode-chain left
   membership seam by composing tasks 74/152's real five-expansion producer
   with task 158's right-only expected-set membership consumer. The test-first
   contract requires raw outermost-mode left and explicit-set right provenance,
   ordinal 2/3 `BindingId(0/1)`, one terminal-set-RHS identity, no left expected
   input, two inferred variables, and one fact-free checked membership with
   exactly one right-owned constraint. Fixture/expectation, six trace backlinks,
   production routing, full near-miss/corruption coverage, and a real sidecar
   now guard active case 115,
   task 165 adds the exact active four-edge object-terminal local-mode-
   chain left membership seam by composing tasks 74/153's real five-expansion
   producer with task 163's object-left/set-right membership consumer. The
   contract requires raw outermost-mode left and explicit-set right provenance,
   ordinal 2/3 `BindingId(0/1)`, distinct terminal-object-RHS and explicit-set
   identities, no left expected input, two inferred variables, and one fact-
   free checked membership with exactly one right-owned constraint. Fixture/
   expectation, six trace backlinks, production routing, full guards, and a
   real sidecar now guard active case 116,
   task 166 adds the exact active four-edge set-terminal local-mode-chain
   reserved-variable equality seam by composing tasks 74/152's real five-
   expansion producer with task 154's equality consumer. The contract requires
   four raw outermost-mode result/expected inputs, ordinal 1/2 `BindingId(0)`,
   all five expansions, one terminal-set-RHS identity, two inferred variables,
   one fact/deferred-free checked equality, and two ordered operand-owned
   expected constraints. Six trace backlinks, exact production routing, full
   corruption/near-miss guards, and a real sidecar now protect active case 117.
   Declaration acceptance/inhabitation,
   truth/facts, closure/order, theorem/proof/Core/ControlFlow/VC, object-terminal
   behavior, other depths, and broader shapes remain deferred,
   task 167 adds the exact active four-edge object-terminal local-mode-chain
   reserved-variable equality seam by composing tasks 74/153's real five-
   expansion producer with task 155's equality consumer. The test-first
   contract requires four raw outermost-mode result/expected inputs, ordinal
   1/2 `BindingId(0)`, all five expansions, one terminal-object-RHS identity,
   two inferred variables, one fact/deferred-free checked equality, and two
   ordered operand-owned expected constraints without object/set coercion.
   Fixture/expectation, six trace backlinks, exact production routing, full
   corruption/near-miss guards, and a real sidecar now protect active case 118.
   Declaration acceptance/
   inhabitation, truth/facts, closure/order, theorem/proof/Core/ControlFlow/VC,
   set-terminal behavior, other depths, and broader shapes remain deferred,
   task 168 specifies the exact four-edge set-terminal local-mode-chain
   reserved-variable inequality seam by composing tasks 74/152's real five-
   expansion producer with task 156's pre-desugaring inequality consumer. The
   test-first contract requires four raw outermost-mode result/expected inputs,
   ordinal 1/2 `BindingId(0)`, all five expansions, one terminal-set-RHS
   identity, two inferred variables, one fact/deferred-free pre-desugaring
   checked inequality, and two ordered operand-owned expected constraints.
   Fixture/expectation, six trace backlinks, exact production routing, full
   corruption/near-miss guards, and a real sidecar now protect active case 119.
   Declaration acceptance/inhabitation, inequality
   desugaring/truth/facts, closure/order, theorem/proof/Core/ControlFlow/VC,
   object-terminal behavior, other depths, and broader shapes remain deferred,
   task 169 specifies the exact four-edge object-terminal local-mode-chain
   reserved-variable inequality seam by composing tasks 74/153's real five-
   expansion producer with task 157's pre-desugaring inequality consumer. The
   test-first contract requires four raw outermost-mode result/expected inputs,
   ordinal 1/2 `BindingId(0)`, all five expansions, one terminal-object-RHS
   identity, two inferred variables, one fact/deferred-free pre-desugaring
   checked inequality, and two ordered operand-owned expected constraints
   without object/set coercion. Fixture/expectation, six trace backlinks, exact
   production routing, full corruption/near-miss guards, and a real sidecar now
   protect active case 120. Declaration acceptance/
   inhabitation, inequality desugaring/truth/facts, closure/order, theorem/
   proof/Core/ControlFlow/VC, set-terminal behavior, other depths, and broader
   shapes remain deferred,
   task 172 specifies the exact local-mode long-chain set-terminal reserved-
   variable equality seam by composing task 74's real seven-expansion producer
   with task 166's equality consumer. The test-first contract requires four raw
   `ChainMode6` result/expected inputs, ordinal 1/2 `BindingId(0)`, all seven
   real AST-derived expansions, one terminal-`BaseMode`-RHS builtin-set
   identity, two inferred variables, one fact/deferred-free checked equality,
   and two ordered operand-owned expected constraints. Exact fixture/
   expectation, six trace backlinks, production routing, full corruption/
   near-miss guards, and a real frontend/resolver sidecar now protect active
   case 121.
   Declaration acceptance/inhabitation, truth/facts, closure/order, theorem/
   proof/Core/ControlFlow/VC, imported/attributed/argument-bearing or other
   chain shapes, and general unbounded semantics remain deferred,
   task 173 specifies the exact long-chain inequality sibling by composing task
   74's seven-expansion producer with task 168's pre-desugaring consumer. It
   preserves four raw `ChainMode6` roles, ordinal 1/2 `BindingId(0)`, one
   terminal-`BaseMode`-RHS identity, two inferred variables, two ordered
   constraints, and one fact/deferred-free checked inequality. Six backlinks,
   full guards, and a real sidecar now protect active case 122; desugaring/truth/facts,
   downstream payloads, other chains, and general semantics remain deferred,
   task 174 specifies the exact long-chain membership sibling by composing task
   74's seven-expansion producer with task 164's right-only expected-set
   consumer. It requires a raw `ChainMode6` left result, independent explicit-
   set right result and sole expected input, ordinal 2/3 `BindingId(0/1)`, one
   terminal-`BaseMode`-RHS identity, no left expected input, two inferred
   variables, one right-owned constraint, and one fact/deferred-free checked
   membership. The fixture, six backlinks, production routing, full guards,
   and a real sidecar now protect active case 123. Truth/facts,
   downstream payloads, other chains, and general semantics remain deferred,
   task 175 specifies the exact long-chain type-assertion sibling by composing
   task 74's seven-expansion producer with task 152's normalized-reflexive
   consumer. It requires a raw `ChainMode6` subject, independent formula-side
   builtin-set asserted input, ordinal 1 `BindingId(0)`, one terminal-
   `BaseMode`-RHS identity, one inferred variable, and one fact/deferred-free
   checked type assertion without general reachability. The fixture, seven
   backlinks, production routing, full guards, and a real sidecar now protect
   active case 124. Widening/`qua`, truth/facts, downstream
   payloads, other chains, and general semantics remain deferred,
   task 176 specifies the exact builtin-object-terminal long-chain equality
   sibling by composing task 74's real AST-bounded chain producer with task
   167's object-normalizing equality consumer. It requires four raw
   `ChainObjectMode6` result/expected inputs, ordinal 1/2 `BindingId(0)`, seven
   real expansions, one terminal-`BaseObjectMode`-RHS identity, two inferred
   terms, two ordered constraints, and one fact/deferred-free checked equality
   without object/set coercion. The fixture, six backlinks, production routing,
   full guards, and a real sidecar now protect active case 125. Truth/facts,
   downstream payloads, other chains, and general
   semantics remain deferred,
   task 177 specifies the matching builtin-object-terminal long-chain inequality
   sibling by composing task 74's real AST-bounded chain producer with task
   169's object-normalizing pre-desugaring inequality consumer. It requires four
   raw `ChainObjectMode6` result/expected inputs, ordinal 1/2 `BindingId(0)`,
   seven real expansions, one terminal-`BaseObjectMode`-RHS identity, two
   inferred terms, two ordered constraints, and one fact/deferred-free pre-
   desugaring checked inequality without object/set coercion. The fixture and
   six backlinks, production routing, full guards, and the real sidecar now
   protect active case 126. Desugaring, truth/facts, downstream
   payloads, other chains, and general semantics remain deferred,
   task 178 specifies the matching builtin-object-terminal long-chain left-
   membership sibling by composing task 74's real AST-bounded chain producer
   with task 165's object-left/set-right membership consumer. It requires the
   raw `ChainObjectMode6` left result, independent explicit-set right result/
   sole expected input, ordinal 2/3 `BindingId(0/1)`, seven real expansions,
   distinct terminal-object-RHS and explicit-set identities, no left expected
   input, two inferred terms, one right-owned constraint, and one fact/deferred-
   free checked membership without object/set coercion. The fixture, six
   backlinks, production routing, full guards, and the real sidecar protect
   active case 127. Truth/facts, downstream payloads, other
   chains, and general semantics remain deferred,
   task 179 specifies the matching builtin-object-terminal long-chain
   normalized-reflexive type-assertion sibling by composing task 74's real AST-
   bounded chain producer with task 153's object-normalizing type-assertion
   consumer and task 175's seven-expansion sibling guard pattern. It requires
   the raw `ChainObjectMode6` subject result, independent formula-side builtin-
   object asserted input, ordinal 1 `BindingId(0)`, seven real expansions, one
   terminal-object-RHS identity, one inferred term, and one fact/deferred-free
   checked type assertion without general reachability or object/set coercion.
   The fixture, six shared backlinks, dedicated row, production routing, full
   guards, and the real sidecar protect active case 128. Truth/facts,
   acceptance, downstream payloads, other chains, and
   general semantics remain deferred,
   task 180 adds the exact standalone
   `SourceDerivedContradictionConstantBoundary: contradiction` formula-leaf
   bridge. A dedicated extractor preserves the real leaf site/range and
   module-root context and passes `FormulaKind::Contradiction` without a
   deferred reason, producing one checked formula with empty term/type/
   constraint/candidate/fact/deferred/diagnostic payload. The fixture,
   dedicated trace row, production route, exact/near-miss/corruption guards,
   and real frontend/resolver sidecar protect active case 129. This credits
   formula type/well-formedness only; falsehood/fact publication, theorem
   acceptance, proof-goal closure, implicit closure/child graph,
   `formula_statement`, proof, CoreIr, ControlFlowIr, and VC remain deferred,
   task 182 adds the exact direct formula-side local-mode asserted-head bridge
   by composing task 55's real AST-derived set-terminal expansion producer with
   tasks 122/138's normalized-reflexive type-assertion consumer. The exact
   source retains independent raw reserve-subject and formula-side asserted
   inputs for the same resolved local-mode symbol, ordinal 1 `BindingId(0)`,
   one real expansion, three known type entries interned to one terminal-
   definition-RHS builtin-set identity, one inferred term, and one fact/
   deferred-free checked type assertion without general reachability. Five
   shared backlinks, the dedicated trace row, production routing, exact/near-
   miss/corruption guards, and a real frontend/resolver sidecar protect active
   case 130. Declaration acceptance/inhabitation, widening/`qua`, truth/facts,
   theorem/proof/CoreIr/ControlFlowIr/VC, other asserted-head families, and
   general semantics remain deferred,
   task 183 adds the exact direct object-terminal formula-side local-mode
   asserted-head bridge by composing task 55's real AST-derived object
   expansion, task 145's normalized object consumer, and task 182's same-symbol
   asserted-head producer. The exact source retains independent raw reserve-
   subject and formula-side asserted inputs for the same resolved mode symbol,
   ordinal 1 `BindingId(0)`, one real expansion, three known type entries
   interned to one terminal-definition-RHS builtin-object identity, one inferred
   term, and one fact/deferred-free checked type assertion without general
   reachability or object/set coercion. Five shared backlinks, the dedicated
   trace row, production routing, exact/near-miss/corruption guards, and a real
   frontend/resolver sidecar protect active case 131. Declaration acceptance/
   inhabitation, truth/facts, theorem/proof/CoreIr/ControlFlowIr/VC, other
   asserted-head families, and general semantics remain deferred,
   task 184 adds the exact one-edge set-terminal same-outer-mode formula-side
   asserted-head bridge by composing task 56's two real AST-derived expansions,
   task 146's normalized set consumer, and task 182's same-symbol formula-side
   asserted-head producer. The exact source retains independent raw reserve-
   subject and formula-side asserted inputs for the same outer mode symbol,
   ordinal 1 `BindingId(0)`, two real expansions, three known type entries
   interned to one terminal-base-definition-RHS builtin-set identity, one
   inferred term, and one fact/deferred-free checked type assertion without
   general reachability. Five shared backlinks, the dedicated trace row,
   production routing, exact/near-miss/corruption guards, and a real frontend/
   resolver sidecar protect active case 132. Declaration acceptance/
   inhabitation, widening/`qua`, truth/facts, closure/order, theorem/proof/
   CoreIr/ControlFlowIr/VC, object-terminal/deeper/other asserted-head chains,
   and general chain semantics remain deferred,
   task 185 adds the exact one-edge object-terminal same-outer-mode formula-side
   asserted-head bridge by composing task 56's two real AST-derived expansions,
   task 147's normalized object consumer, task 183's same-symbol object asserted-
   head producer, and task 184's recursive asserted-head pattern. The exact
   source retains independent raw reserve-subject and formula-side asserted
   inputs for the same outer mode symbol, ordinal 1 `BindingId(0)`, two real
   expansions, three known type entries interned to one terminal-base-definition-
   RHS builtin-object identity, one inferred term, and one fact/deferred-free
   checked type assertion without general reachability, widening/`qua`, or
   object/set coercion. Five shared backlinks, the dedicated trace row,
   production routing, exact/near-miss/corruption guards, and a real frontend/
   resolver sidecar protect active case 133. Declaration/attribute acceptance,
   truth/facts, closure/order, theorem/proof/CoreIr/ControlFlowIr/VC, imported/
   set-terminal/deeper/other asserted-head chains, and general chain semantics
   remain deferred,
   task 186 adds the exact two-edge set-terminal same-outer-mode formula-side
   asserted-head bridge by composing task 72's three real AST-derived
   expansions, task 148's normalized set consumer, and task 184's same-symbol
   asserted-head pattern. The exact source retains independent raw reserve-
   subject and formula-side asserted inputs for the same outer mode symbol,
   ordinal 1 `BindingId(0)`, three real expansions, three known type entries
   interned to one terminal-base-definition-RHS builtin-set identity, one
   inferred term, and one fact/deferred-free checked type assertion without
   reachability, widening, or `qua`. Five shared backlinks, the dedicated trace
   row, production routing, exact/near-miss/corruption guards, and a real
   frontend/resolver sidecar protect active case 134. Declaration/attribute
   acceptance, truth/facts, closure/order, theorem/proof/CoreIr/ControlFlowIr/
   VC, object-terminal/deeper/imported/other asserted-head chains, and general
   chain semantics remain deferred,
   task 187 adds the exact two-edge object-terminal same-outer-mode formula-side
   asserted-head bridge by composing task 72's three real AST-derived
   expansions, task 149's normalized object consumer, and task 185's same-
   symbol asserted-head pattern. The exact source retains distinct raw reserve-
   subject and formula-side asserted sites/ranges for the same local outer mode
   symbol, ordinal 1 `BindingId(0)`, three real expansions, three known type
   entries interned to one terminal-base-definition-RHS builtin-object identity,
   one inferred term, and one fact/deferred-free checked type assertion without
   reachability, widening, `qua`, or object/set coercion. Five shared backlinks,
   the dedicated trace row, production routing, exact/near-miss/corruption
   guards, and a real frontend/resolver sidecar protect active case 135.
   Positive imported semantics, declaration/attribute acceptance, truth/facts,
   closure/order, theorem/proof/CoreIr/ControlFlowIr/VC, set-terminal/deeper/
   other asserted-head chains, and general chain semantics remain deferred,
   task 188 adds the exact builtin-object same-binding equality bridge by
   composing tasks 48/125's real written `object` reserve handoff, task 119's
   exact reserved-variable equality builder, and task 128's real builtin-object
   normalization consumer. The route accepts only `reserve x for object;
   theorem ReservedObjectVariableEqualityPayloadBoundary: x = x;`, resolves
   source-order ordinals 1/2 to `BindingId(0)`, preserves four distinct result/
   expected role sites on the one written type range, interns one canonical
   builtin-object identity, and records two inferred variables, two ordered
   constraints, and one fact/deferred-free checked equality. Five shared
   backlinks, one dedicated row, structural/provenance near misses, corruption
   and immutable-output guards, and a real frontend/resolver sidecar protect
   active case 136. Object/set coercion, general/non-reflexive equality, truth/
   facts, closure/order, declaration/theorem acceptance, proof/CoreIr/
   ControlFlowIr/VC, and broader shapes remain deferred,
   task 189 adds the exact builtin-object reserved-variable normalized-reflexive
   type-assertion bridge by composing tasks 48/125/188's real written `object`
   reserve handoff with task 122's assertion builder and task 145's real
   builtin-object normalization consumer. The route accepts only `reserve x
   for object; theorem ReservedObjectVariableTypeAssertionPayloadBoundary: x
   is object;`, resolves source-order ordinal 1 to `BindingId(0)`, preserves
   distinct reserve-result and formula-side asserted object sites/ranges,
   interns one reserve-anchored canonical builtin-object identity, and records
   one inferred variable, three known type entries, zero expected constraints,
   and one fact/deferred-free checked assertion. Five shared backlinks, one
   dedicated row, structural/provenance near misses, mutable corruption and
   positive immutable-output guards, and a real frontend/resolver sidecar
   protect active case 137. Reachability/widening/`qua`, object/set coercion,
   truth/facts, closure/order, declaration/theorem acceptance, proof/CoreIr/
   ControlFlowIr/VC, and broader shapes remain deferred,
   task 190 adds the exact builtin-object same-binding pre-desugaring inequality
   bridge by composing tasks 48/125/188's real written `object` reserve handoff,
   task 121's exact inequality builder, and task 128's real builtin-object
   normalization consumer. The route accepts only `reserve x for object;
   theorem ReservedObjectVariableInequalityPayloadBoundary: x <> x;`, resolves
   source-order ordinals 1/2 to `BindingId(0)`, preserves four distinct result/
   expected role sites on the one written type range, interns one canonical
   builtin-object identity, and records two inferred variables, six known type
   entries, two ordered constraints, and one fact/candidate/diagnostic/
   deferred-free checked inequality. Five shared backlinks, one dedicated row,
   structural/provenance near misses, corruption and immutable-output guards,
   and a real frontend/resolver sidecar protect active case 138. Inequality
   desugaring/equality truth, object/set coercion, facts, closure/order,
   declaration/theorem acceptance, proof/CoreIr/ControlFlowIr/VC, and broader
   shapes remain deferred,
   task 191 adds the exact distinct-binding shared-builtin-object equality
   bridge by composing task 123's real one-item/two-binding shared-range reserve
   producer with tasks 48/125/188's real builtin-object reserve,
   normalization, and equality consumer. The route accepts only `reserve x, y
   for object; theorem DistinctReservedObjectVariableEqualityPayloadBoundary:
   x = y;`, resolves source-order ordinals 2/3 to `BindingId(0/1)`, preserves
   one written `object` range across both bindings and four distinct result/
   expected role sites, interns one reserve-range-anchored canonical builtin-
   object identity, and records two inferred variables, six known type entries,
   two ordered constraints, and one fact/candidate/diagnostic/deferred-free
   checked equality. Five shared backlinks, one dedicated row, structural/
   provenance near misses, corruption and immutable-output guards, and a real
   frontend/resolver sidecar protect active case 139. Equality truth,
   object/set coercion, facts, closure/order, declaration/theorem acceptance,
   proof/CoreIr/ControlFlowIr/VC, and broader distinct-object shapes remain
   deferred,
   task 192 adds the exact distinct-binding shared-builtin-object inequality
   bridge by composing tasks 123/191's real one-item/two-binding shared-range
   builtin-object producer with tasks 121/160/190's real pre-desugaring
   inequality consumer. This `test_gap`, narrow `source_drift`, and
   `design_drift` slice accepts only `reserve x, y for object; theorem
   DistinctReservedObjectVariableInequalityPayloadBoundary: x <> y;`, resolves
   source-order ordinals 2/3 to `BindingId(0/1)`, preserves one written `object`
   range across both bindings and four distinct result/expected role sites,
   interns one reserve-range-anchored canonical builtin-object identity, and
   records two inferred variables, six known type entries, two ordered
   constraints, and one fact/candidate/diagnostic/deferred-free checked
   inequality. Five shared backlinks, one dedicated row, isolated structural/
   provenance near misses, corruption and immutable-output guards, and a real
   frontend/resolver sidecar protect active case 140 within 355 cases and 319
   requirements. Inequality desugaring/equality truth, object/set coercion,
   facts, closure/order, declaration/theorem acceptance, proof/CoreIr/
   ControlFlowIr/VC, and broader distinct-object shapes remain deferred; Step 5
   remains active, Steps 6/7 remain deferred, and no checker source or module-
   layout change was required,
   task 193 adds the exact multiple-reserve-declaration builtin-object equality
   bridge by composing Task 124's real two-item/two-binding/distinct-written-
   range producer with tasks 188/191's builtin-object equality consumer. This
   `test_gap`, narrow `source_drift`, and `design_drift` slice accepts only
   `reserve x for object; reserve y for object; theorem
   MultipleObjectReserveDeclarationEqualityPayloadBoundary: x = y;`, resolves
   source-order ordinals 2/3 to `BindingId(0/1)`, retains two binding-owned
   written `object` ranges across four distinct result/expected role sites,
   interns one canonical builtin-object identity anchored at the earlier `x`
   range, and records two inferred variables, six known type entries, two
   ordered constraints, and one fact/candidate/diagnostic/deferred-free checked
   equality. Five shared backlinks, one dedicated row, structural/provenance
   near misses, corruption and immutable-output guards, and a real frontend/
   resolver sidecar protect active case 141 within 356 cases and 320
   requirements. Equality truth, object/set coercion, facts, closure/order,
   declaration/theorem acceptance, proof/CoreIr/ControlFlowIr/VC, shared-range
   and broader multiple-reserve object shapes remain deferred; Step 5 remains
   active, Steps 6/7 remain deferred, and no checker source or module-layout
   change was required,
   task 194 adds the exact multiple-reserve-declaration builtin-object
   inequality bridge by composing Task 193's real ordered two-item/two-binding/
   distinct-written-object-range producer with tasks 190/192's pre-desugaring
   builtin-object inequality consumer. This `test_gap`, narrow `source_drift`,
   and `design_drift` slice accepts only `reserve x for object; reserve y for
   object; theorem MultipleObjectReserveDeclarationInequalityPayloadBoundary:
   x <> y;`, resolves source-order ordinals 2/3 to `BindingId(0/1)`, retains two
   ordered binding-owned written `object` ranges across four distinct raw
   result/expected roles, interns one canonical builtin-object identity
   anchored at the earlier `x` range, and records two inferred variables, six
   known type entries, two ordered constraints, and one fact/candidate/
   diagnostic/deferred-free pre-desugaring checked inequality. Five shared
   backlinks, one dedicated row, structural/provenance near misses, corruption
   and immutable-output guards, route isolation, and a real frontend/resolver
   sidecar protect active case 142 within 357 cases and 321 requirements.
   Inequality desugaring/equality truth, object/set coercion, facts, closure/
   order, declaration/theorem acceptance, proof/CoreIr/ControlFlowIr/VC,
   shared-range and broader multiple-reserve object shapes remain deferred;
   Step 5 remains active, Steps 6/7 remain deferred, and no checker source or
   module-layout change was required,
   task 195 adds the exact three-edge set-terminal same-outer-mode asserted-
   head bridge by composing Task 73's real four-expansion producer with Task
   186's formula-side same-symbol asserted-head consumer. This `test_gap`,
   narrow `source_drift`, and `design_drift` slice accepts only four ordered
   local definitions `Outer -> Middle -> Inner -> Base -> set`, `reserve x for
   OuterThreeEdgeModeAssertedHead`, and theorem
   `ThreeEdgeLocalModeAssertedHeadPayloadBoundary: x is
   OuterThreeEdgeModeAssertedHead;`; resolves ordinal 1 to `BindingId(0)`;
   preserves distinct raw subject/asserted sites and ranges; consumes all four
   AST-derived expansions; interns one base-definition-RHS-anchored builtin-set
   identity across three known type entries; and records one inferred variable,
   zero constraints/candidates/facts/diagnostics/deferred reasons, and one
   normalized-reflexive checked type assertion. Five shared backlinks, one
   dedicated row, structural/provenance near misses including unrelated local/
   imported/ambiguous asserted heads, corruption and immutable-output guards,
   route isolation, and a real frontend/resolver sidecar protect active case
   143 within 358 cases and 322 requirements. Object-terminal/deeper/imported/
   attributed/argument-bearing/other asserted heads, reachability/widening/
   `qua`, declaration/theorem acceptance, truth/facts, closure/order, broader
   term/formula/child-graph semantics, proof/CoreIr/ControlFlowIr/VC, general
   chain semantics, and downstream payloads remain deferred; Step 5 remains
   active, Steps 6/7 remain deferred, and no checker source or module-layout
   change was required,
   task 196 adds the exact three-edge object-terminal same-outer-mode asserted-
   head bridge by composing Tasks 73/151's real four-expansion object producer
   with Task 187's formula-side same-symbol asserted-head consumer. This
   `test_gap`, narrow `source_drift`, and `design_drift` slice accepts only four
   ordered local definitions `Outer -> Middle -> Inner -> Base -> object`,
   `reserve x for OuterThreeEdgeObjectModeAssertedHead`, and theorem
   `ThreeEdgeLocalObjectModeAssertedHeadPayloadBoundary: x is
   OuterThreeEdgeObjectModeAssertedHead;`; resolves ordinal 1 to `BindingId(0)`;
   preserves distinct raw subject/asserted sites and ranges; consumes all four
   AST-derived expansions; interns one base-definition-RHS-anchored builtin-
   object identity across three known type entries; and records one inferred
   variable, zero constraints/candidates/facts/diagnostics/deferred reasons,
   and one normalized-reflexive checked type assertion without object/set
   coercion. Five shared backlinks, one dedicated row, structural/provenance
   near misses including unrelated local/imported/ambiguous asserted heads,
   `BuiltinSet`/canonical corruption and immutable-output guards, route
   isolation, and a real frontend/resolver sidecar protect active case 144
   within 359 cases and 323 requirements. Deeper/imported/attributed/argument-
   bearing/other asserted heads, reachability/widening/`qua`, declaration/
   theorem acceptance, truth/facts, closure/order, broader term/formula/child-
   graph semantics, proof/CoreIr/ControlFlowIr/VC, general chain semantics, and
   downstream payloads remain deferred; Step 5 remains active, Steps 6/7 remain
   deferred, and no checker source or module-layout change was required,
   task 197 adds the exact four-edge set-terminal same-outermost-mode asserted-
   head bridge by composing Tasks 74/152's real five-expansion set producer
   with Tasks 186/195's formula-side same-symbol asserted-head consumer. This
   `test_gap`, narrow `source_drift`, and `design_drift` slice accepts only five
   ordered local definitions `TooDeep -> Outer -> Middle -> Inner -> Base ->
   set`, `reserve x for TooDeepFourEdgeModeAssertedHead`, and theorem
   `FourEdgeLocalModeAssertedHeadPayloadBoundary: x is
   TooDeepFourEdgeModeAssertedHead;`; resolves ordinal 1 to `BindingId(0)`;
   preserves distinct raw subject/asserted sites and ranges; consumes all five
   AST-derived expansions; interns one base-definition-RHS-anchored builtin-set
   identity across three known type entries; and records one inferred variable,
   zero constraints/candidates/facts/diagnostics/deferred reasons, and one
   normalized-reflexive checked type assertion. Five shared backlinks, one
   dedicated row, full-reorder/connected-deeper/structural/provenance near
   misses including unrelated local/imported/ambiguous asserted heads,
   `BuiltinObject`/canonical corruption and immutable-output guards, route
   isolation, and a real frontend/resolver sidecar protect active case 145
   within 360 cases and 324 requirements. Object-terminal/other-depth/imported/
   attributed/argument-bearing/other asserted heads, reachability/widening/
   `qua`, declaration/theorem acceptance, truth/facts, closure/order, broader
   term/formula/child-graph semantics, proof/CoreIr/ControlFlowIr/VC, general
   chain semantics, and downstream payloads remain deferred; Step 5 remains
   active, Steps 6/7 remain deferred, and no checker source or module-layout
   change was required,
   task 198 adds the exact four-edge object-terminal same-outermost-mode
   asserted-head bridge by composing Tasks 74/153's real five-expansion object
   producer with Tasks 187/196's formula-side same-symbol asserted-head
   consumer. This `test_gap`, narrow `source_drift`, and `design_drift` slice
   accepts only five ordered local definitions `TooDeep -> Outer -> Middle ->
   Inner -> Base -> object`, `reserve x for
   TooDeepFourEdgeObjectModeAssertedHead`, and theorem
   `FourEdgeLocalObjectModeAssertedHeadPayloadBoundary: x is
   TooDeepFourEdgeObjectModeAssertedHead;`; resolves ordinal 1 to
   `BindingId(0)`; preserves distinct raw subject/asserted sites and ranges;
   consumes all five AST-derived expansions; interns one base-definition-RHS-
   anchored builtin-object identity across three known type entries; and
   records one inferred variable, zero constraints/candidates/facts/
   diagnostics/deferred reasons, and one normalized-reflexive checked type
   assertion without object/set coercion. Five shared backlinks, one dedicated
   row, full-reorder/connected-deeper/structural/provenance near misses
   including unrelated local/imported/ambiguous asserted heads, `BuiltinSet`/
   canonical corruption and immutable-output guards, route isolation, and a
   real frontend/resolver sidecar protect active case 146 within 361 cases and
   325 requirements without changing an existing expectation. Set-terminal/
   other-depth/imported/attributed/argument-bearing/other asserted heads,
   reachability/widening/`qua`, declaration/theorem acceptance, truth/facts,
   closure/order, broader term/formula/child-graph semantics, proof/CoreIr/
   ControlFlowIr/VC, general chain semantics, and downstream payloads remain
   deferred; Step 5 remains active, Steps 6/7 remain deferred, and no checker
   source or module-layout change was required,
   task 199 adds the exact seven-expansion set-terminal same-`ChainMode6`
   asserted-head bridge by composing Tasks 74/175's real set-terminal producer
   with Tasks 186/195/197's formula-side same-symbol asserted-head consumer.
   This `test_gap`, narrow `source_drift`, and `design_drift` slice accepts only
   `BaseMode -> set`, six ordered local links through `ChainMode6 ->
   ChainMode5`, `reserve x for ChainMode6`, and theorem
   `LongLocalModeAssertedHeadPayloadBoundary: x is ChainMode6;`; resolves
   ordinal 1 to `BindingId(0)`; preserves distinct raw subject/asserted sites
   and ranges; consumes all seven AST-derived expansions; interns one
   `BaseModeDef` RHS-anchored builtin-set identity across three known type
   entries; and records one inferred variable, zero constraints/candidates/
   facts/diagnostics/deferred reasons, and one normalized-reflexive checked type
   assertion. Five shared backlinks, one dedicated row, per-link removal/
   reorder, complete-reverse/connected-eighth/structural/provenance near misses
   including unrelated local/imported/ambiguous asserted heads,
   `BuiltinObject`/canonical corruption and immutable-output guards, route
   isolation, and a real frontend/resolver sidecar protect active case 147
   within 362 cases and 326 requirements without changing an existing
   expectation. Object-terminal/other-depth/imported/attributed/argument-
   bearing/other asserted heads, reachability/widening/`qua`, declaration/
   theorem acceptance, truth/facts, closure/order, broader term/formula/child-
   graph semantics, proof/CoreIr/ControlFlowIr/VC, general unbounded chain
   semantics, and downstream payloads remain deferred; Step 5 remains active,
   Steps 6/7 remain deferred, and no checker source or module-layout change was
   required,
   task 200 adds the exact seven-expansion object-terminal same-`ChainObjectMode6`
   asserted-head bridge by composing Tasks 74/179's real object-terminal
   producer with Tasks 187/196/198's formula-side same-symbol asserted-head
   consumer. This `test_gap`, narrow `source_drift`, and `design_drift` slice
   accepts only `BaseObjectMode -> object`, six ordered local links through
   `ChainObjectMode6 -> ChainObjectMode5`, `reserve x for ChainObjectMode6`, and
   theorem `LongLocalObjectModeAssertedHeadPayloadBoundary: x is
   ChainObjectMode6;`; resolves ordinal 1 to `BindingId(0)`; preserves distinct
   raw subject/asserted sites and ranges; consumes all seven AST-derived
   expansions; interns one `BaseObjectModeDef` RHS-anchored builtin-object
   identity across three known type entries; and records one inferred variable,
   zero constraints/candidates/facts/diagnostics/deferred reasons, and one
   normalized-reflexive checked type assertion without object/set coercion.
   Five shared backlinks, one dedicated row, per-link removal/reorder, complete-
   reverse/connected-eighth/structural/provenance near misses including unrelated
   local/imported/ambiguous asserted heads, `BuiltinSet`/canonical corruption and
   immutable-output guards, route isolation, and a real frontend/resolver sidecar
   protect active case 148 within 363 cases and 327 requirements without changing
   an existing expectation. Set-terminal/other-depth/imported/attributed/
   argument-bearing/other asserted heads, reachability/widening/`qua`, declaration/
   theorem acceptance, truth/facts, closure/order, broader term/formula/child-graph
   semantics, proof/CoreIr/ControlFlowIr/VC, general unbounded chain semantics, and
   downstream payloads remain deferred; Step 5 remains active, Steps 6/7 remain
   deferred, and no checker source or module-layout change was required,
   and task 109 supersedes task 102 for
   the exact builtin
   type assertion theorem source by passing real checker term/formula/asserted
   type payloads before failing on missing numeric type payloads and partial
   formula checking, task 113 supersedes task 103 for the exact imported
   attribute assertion theorem formula checker bridge, task 114 supersedes task
   104 for the exact attribute-level non-empty imported attribute assertion
   theorem formula checker bridge, task 111 supersedes task 105 for the exact
   set-enumeration theorem formula checker bridge, task 112 supersedes task 99
   for the exact formula connective/quantifier shell checker bridge, task 88
   records a theorem proof block on the
   same gap, task 89 records statement-level proof justifications on the same
   gap, task 90 records predicate/functor definitions on the same gap, and task
   91 records attribute definitions on the same gap, task 92 records
   mode/structure definitions on the same gap, and task 93 records
   proof-local declarations on the same gap, task 94 records proof-local
   inline definitions on the same gap, task 95 records registration blocks on
   the same gap, and task 96 records redefinition/notation surfaces on the
   same gap while keeping definition
   declaration payloads, formula child/binder payloads beyond task 112,
   definition-local context, definiens formula/term
   payloads, formula-definiens payloads, mode expansion, structure
   base-shape/constructor/selector evidence, proof-local declaration payloads,
   inline definition formal/body payloads, local proof contexts, RHS term
   inference, reconsider coercion/obligation evidence, local abbreviation
   expansion, registration item payloads, accepted activation/evidence status,
   redefinition payloads, notation alias relation payloads, redefinition target
   inference, coherence proof-obligation payloads, theorem acceptance,
   formula constant semantic checking beyond task 180's exact standalone
   contradiction type/well-formedness slice (including truth values and
   `thesis` semantics),
   attributed-type evidence, imported predicate/functor semantic payloads,
   membership operand expected-type construction/checking beyond task 120's
   exact right-operand expected-`set` slice (task 108's numeral bridge still
   lacks it), inequality expected-type construction/checking beyond task 121's
   exact reserved-variable pre-desugaring slice (task 107's numeral bridge
   remains partial without expected types), inequality desugaring/equality
   semantic checking beyond task 121,
   type-assertion reachability/widening/`qua` beyond task 122's exact normalized
   reflexive identity slice, broader asserted-type payload extraction and
   attribute admissibility,
   overload payloads, broader term/formula/proof
   skeleton/statement proof payloads, term inference and formula
   well-formedness checking beyond task 119's exact same-binding equality,
   task 123's exact distinct-binding equality, and task 124's exact
   multiple-reserve-declaration equality, and task 125's exact heterogeneous
   reserve membership, task 126's exact direct-local-mode equality, and task
   127's exact one-edge local-mode-chain equality, task 128's exact direct
   local-object-mode equality, and task 129's exact one-edge
   local-object-mode-chain equality, and task 130's exact direct-local-mode
   inequality, and task 131's exact direct-local-object-mode inequality,
   and task 132's exact one-edge local-mode-chain inequality,
   and task 133's exact one-edge local-object-mode-chain inequality,
   and task 134's exact two-edge local-mode-chain equality,
   and task 135's exact two-edge local-object-mode-chain equality,
   and task 136's exact two-edge local-mode-chain inequality,
   and task 137's exact two-edge local-object-mode-chain inequality,
   and task 138's exact direct local-mode normalized-reflexive type assertion,
   and task 139's exact direct local-mode left membership,
   and task 140's exact direct local-object-mode left membership,
   and task 141's exact one-edge local-mode-chain left membership,
   and task 142's exact one-edge local-object-mode-chain left membership,
   and task 143's exact two-edge local-mode-chain left membership,
   and task 144's exact two-edge local-object-mode-chain left membership,
   and task 145's exact direct local-object-mode normalized-reflexive type
   assertion,
   and task 146's exact one-edge local-mode-chain normalized-reflexive type
   assertion,
   and task 147's exact one-edge local-object-mode-chain normalized-reflexive
   type assertion,
   and task 148's exact two-edge local-mode-chain normalized-reflexive
   type assertion,
   and task 149's exact two-edge local-object-mode-chain normalized-reflexive
   type assertion,
   and task 150's exact three-edge local-mode-chain normalized-
   reflexive type assertion,
   and task 151's exact three-edge local-object-mode-chain normalized-
   reflexive type assertion,
   and task 152's exact four-edge local-mode-chain normalized-
   reflexive type assertion,
   and task 153's exact four-edge local-object-mode-chain normalized-
   reflexive type assertion,
   and task 154's exact three-edge local-mode-chain equality,
   and task 155's exact three-edge local-object-mode-chain equality,
   and task 156's exact three-edge local-mode-chain inequality,
   and task 157's exact three-edge local-object-mode-chain inequality,
   and task 158's exact three-edge local-mode-chain left membership,
   and task 159's exact distinct-binding shared-reserve membership,
   and task 160's exact active distinct-binding shared-reserve inequality over
   task 123's shared-range producer and task 121's pre-desugaring inequality
   consumer,
   and task 161's exact active multiple-reserve-declaration inequality over task
   124's distinct-written-range producer and task 160's pre-desugaring
   inequality consumer,
   and task 162's exact active multiple-reserve-declaration membership over task
   124's distinct-written-range producer and tasks 120/159's right-only expected-
   set membership consumer,
   and task 163's exact active three-edge local-object-mode-chain left
   membership over the real four-expansion object-terminal producer and task
   144's object-left/set-right membership consumer,
   and task 164's exact active four-edge set-terminal local-mode-chain left
   membership over the real five-expansion producer and task 158's set-left/
   set-right membership consumer,
   and task 165's exact active four-edge object-terminal local-mode-chain left
   membership over the real five-expansion producer and task 163's object-left/
   set-right membership consumer,
   and task 166's exact active four-edge set-terminal local-mode-chain equality,
   and task 167's exact active four-edge object-terminal local-mode-chain
   equality,
   and task 168's exact active four-edge set-terminal local-mode-chain
   inequality,
   and task 169's exact active four-edge object-terminal local-mode-chain
   inequality,
   task 120 exact reserved-variable membership, and task 121 exact
   reserved-variable inequality, and task 122 exact reserved-variable type-
   assertion slices,
   recorded facts, imported attribute assertion semantic payloads, imported
   attribute-level non-empty assertion semantic payloads, negated attribute
   admissibility/semantic checking, attribute admissibility/semantic checking,
   set-enumeration result-type payload extraction beyond task 111,
   `formula_statement`, CoreIr, ControlFlowIr,
   VC, and proof payloads deferred).
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

Exit: the source-to-semantics and core/VC completion gates hold — active
semantic corpus coverage replaces extraction-gap sentinels for the promoted
families.

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


## Step 5 Task 201 Addendum

Task 201 closes the exact one-edge set-terminal immediate-radix asserted-head `test_gap`, narrow `source_drift`, and `design_drift` slice without a `spec_gap`. It composes Tasks 56/146's real two-expansion producer with Task 184's formula consumer for `BaseModeRadixAssertedHead -> set`, `OuterModeRadixAssertedHead -> BaseModeRadixAssertedHead`, one outer reserve, and `ChainedLocalModeRadixAssertedHeadPayloadBoundary: x is BaseModeRadixAssertedHead;`. A closed asserted-head relation preserves builtin and same-mode routes and admits only the resolved immediate radix. The active route preserves distinct Outer/Base provenance, resolves ordinal 1 to `BindingId(0)`, normalizes three known entries to one Base-definition-RHS builtin-set identity, and records one inferred variable plus one zero-constraint/fact/candidate/diagnostic/deferred checked assertion. Five shared plus one dedicated backlink and exact/corruption/isolation/real-sidecar guards protect active case 149 within 364 cases and 328 requirements. No existing expectation changed. Step 5 remains active; Steps 6/7 and broader asserted-head/proof/CoreIr/ControlFlowIr/VC semantics remain deferred. No checker source or module-layout change was required.


## Step 5 Task 202 Addendum

Task 202 closes the exact object-terminal immediate-radix asserted-head `test_gap`, narrow `source_drift`, and `design_drift` slice without a `spec_gap`. Tasks 56/147 provide the real two-expansion object producer, Task 185 the object formula consumer, and Task 201 the unchanged closed immediate-radix relation. The active route preserves distinct Outer/Base provenance, resolves ordinal 1 to `BindingId(0)`, normalizes three known entries to one Base-definition-RHS builtin-object identity, and records one inferred variable plus one zero-constraint/fact/candidate/diagnostic/deferred checked assertion without object/set coercion. Five shared plus one dedicated backlink and exact/corruption/real Tasks 147/185/201 isolation/sidecar guards protect active case 150 within 365 cases and 329 requirements. No existing expectation changed. Step 5 remains active; Steps 6/7 and broader semantics remain deferred. No checker source or module-layout change was required.


## Step 5 Task 203 Addendum

Task 203 closes the exact two-edge set-terminal immediate-radix asserted-head `test_gap`, narrow `source_drift`, and `design_drift` slice without a `spec_gap`. Task 72 provides three real expansions, Task 186 the formula consumer, and Tasks 201/202 the unchanged closed immediate-radix relation. The active route preserves distinct Outer/Middle provenance, resolves ordinal 1 to `BindingId(0)`, consumes three expansions, normalizes three known entries to one Base-definition-RHS builtin-set identity, and records one inferred variable plus one zero-constraint/fact/candidate/diagnostic/deferred checked assertion. Five shared plus one dedicated backlink and exact/corruption/order/duplicate/spelling/imported/ambiguous/deeper/real Tasks 122/148/149/186/187/201/202 isolation/sidecar guards protect active case 151 within 366 cases and 330 requirements. No existing expectation changed. Two-hop Base assertion, the object sibling, and broader semantics remain deferred. Step 5 remains active; Steps 6/7 remain deferred. No checker source or module-layout change was required.


## Step 5 Task 204 Addendum

Task 204 closes the exact two-edge object-terminal immediate-radix asserted-head `test_gap`, narrow `source_drift`, and `design_drift` slice without a `spec_gap`. Task 72 provides three real object expansions, Task 187 the formula consumer, and Tasks 202/203 the unchanged closed immediate-radix relation. The active route preserves distinct Outer/Middle provenance, resolves ordinal 1 to `BindingId(0)`, consumes three expansions, normalizes three known entries to one Base-definition-RHS builtin-object identity, and records one inferred variable plus one zero-constraint/fact/candidate/diagnostic/deferred checked assertion without object/set coercion. Five shared plus one dedicated backlink and exact/corruption/order/duplicate/spelling/imported/ambiguous/deeper/real Tasks 189/145/147/149/187/202 and set Tasks 148/186/203 isolation/sidecar guards protect active case 152 within 367 cases and 331 requirements. No existing expectation changed. Two-hop Base assertion and broader semantics remain deferred. Step 5 remains active; Steps 6/7 remain deferred. No checker source or module-layout change was required.

## Step 5 Task 205 Addendum

Task 205 closes the exact three-edge set-terminal immediate-radix asserted-head `test_gap`, narrow `source_drift`, and `design_drift` slice without a `spec_gap`. Task 73 provides four real set-terminal expansions, Task 195 the formula consumer, and Tasks 201/203/204 the unchanged closed immediate-radix relation. The active route preserves distinct Outer/Middle provenance, resolves ordinal 1 to `BindingId(0)`, consumes four expansions, normalizes three known entries to one Base-definition-RHS builtin-set identity, and records one inferred variable plus one zero-constraint/fact/candidate/diagnostic/deferred checked assertion. Five shared plus one dedicated backlink and exact/corruption/all-23-orders/missing/duplicate/label/spelling/radix/imported/ambiguous/deeper/multi-hop/bidirectional-16-route/sidecar guards protect active case 153 within 368 cases and 332 requirements. No existing expectation changed. Multi-hop Inner/Base assertion, the matching object sibling, and broader semantics remain deferred. Step 5 remains active; Steps 6/7 remain deferred. No checker source or module-layout change was required.

## Step 5 Task 206 Addendum

Task 206 closes the exact three-edge object-terminal immediate-radix asserted-head `test_gap`, narrow `source_drift`, and `design_drift` slice without a `spec_gap`. Task 73 provides four real object-terminal expansions, Task 196 the formula consumer, and Tasks 201/204/205 the unchanged closed immediate-radix relation. The active route preserves distinct Outer/Middle provenance, resolves ordinal 1 to `BindingId(0)`, consumes four expansions, normalizes three known entries to one Base-definition-RHS builtin-object identity, and records one inferred variable plus one zero-constraint/fact/candidate/diagnostic/deferred checked assertion without object/set coercion. Five shared plus one dedicated backlink and exact/corruption/all-23-orders/per-definition/imported/ambiguous/deeper/multi-hop/local-other/bidirectional-17-route/sidecar guards protect active case 154 within 369 cases and 333 requirements. No existing expectation changed. Multi-hop Inner/Base assertion and broader semantics remain deferred. Step 5 remains active; Steps 6/7 remain deferred. No checker source or module-layout change was required.

## Step 5 Task 207 Addendum

Task 207 closes the exact four-edge set-terminal immediate-radix asserted-head `test_gap`, narrow `source_drift`, and `design_drift` slice without a `spec_gap`. Task 74 provides five real set-terminal expansions, Task 197 the formula consumer, and Tasks 201/203/205/206 the unchanged closed immediate-radix relation. The active route preserves distinct TooDeep/Outer provenance, resolves ordinal 1 to `BindingId(0)`, consumes five expansions, normalizes three known entries to one Base-definition-RHS builtin-set identity, and records one inferred variable plus one zero-constraint/fact/candidate/diagnostic/deferred checked assertion. Five shared plus one dedicated backlink and exact/corruption/all-119-orders/per-definition/imported/ambiguous/deeper/multi-hop/local-other/bidirectional-20-route/sidecar guards protect active case 155 within 370 cases and 334 requirements. No existing expectation changed. Multi-hop Middle/Inner/Base assertions, the matching object sibling, and broader semantics remain deferred. Step 5 remains active; Steps 6/7 remain deferred. No checker source or module-layout change was required.

## Step 5 Task 208 Addendum

Task 208 closes the exact four-edge object-terminal immediate-radix asserted-head `test_gap`, narrow `source_drift`, and `design_drift` slice without a `spec_gap`. Tasks 74/153 provide five real object-terminal expansions, Task 198 the formula consumer, and Tasks 202/204/206/207 the unchanged closed immediate-radix relation. The active route preserves distinct TooDeep/Outer provenance, resolves ordinal 1 to `BindingId(0)`, consumes five expansions, normalizes three known entries to one Base-definition-RHS builtin-object identity, and records one inferred variable plus one zero-constraint/fact/candidate/diagnostic/deferred checked assertion without object/set coercion. Five shared plus one dedicated backlink and exhaustive all-119-orders/source/provenance/corruption/bidirectional-21-route/sidecar guards protect active case 156 within 371 cases and 335 requirements. No existing expectation changed. Multi-hop Middle/Inner/Base assertions and broader semantics remain deferred. Step 5 remains active; Steps 6/7 remain deferred. No checker source or module-layout change was required.

## Step 5 Task 209 Addendum

Task 209 closes the exact seven-expansion set-terminal immediate-radix asserted-head `test_gap`, narrow `source_drift`, and `design_drift` slice without a `spec_gap`. Task 74 provides seven real expansions, Task 199 the formula consumer, Task 175 the builtin sibling/guards, and the unchanged closed relation the exact `ChainMode6 -> ChainMode5` edge. The active route preserves distinct ChainMode6/ChainMode5 provenance, resolves ordinal 1 to `BindingId(0)`, consumes seven expansions, normalizes three known entries to one BaseModeDef-RHS builtin-set identity, and records one inferred variable plus one zero-constraint/fact/candidate/diagnostic/deferred checked assertion. Five shared plus one dedicated backlink, all 5,039 nonidentity orders, the finite source/provenance/corruption matrix, all 34 pre-existing owner routes, immutable output, and a real sidecar protect active case 157 within 372 cases and 336 requirements. No existing expectation changed. Multi-hop ChainMode4 through BaseMode, the object sibling, and broader semantics remain deferred. Step 5 remains active; Steps 6/7 remain deferred. No checker source or module-layout change was required.

## Step 5 Task 210 Addendum

Task 210 closes the exact seven-expansion object-terminal immediate-radix asserted-head `test_gap`, narrow `source_drift`, and `design_drift` slice without a `spec_gap`. Task 74 provides seven real object-terminal expansions, Task 200 the formula consumer, Task 179 the builtin-object sibling/guards, Task 209 the set-terminal sibling, and the unchanged closed relation the exact `ChainObjectMode6 -> ChainObjectMode5` edge. The active route preserves distinct ChainObjectMode6/ChainObjectMode5 provenance, resolves ordinal 1 to `BindingId(0)`, consumes seven expansions, normalizes three known entries to one BaseObjectModeDef-RHS builtin-object identity, and records one inferred variable plus one zero-constraint/fact/candidate/diagnostic/deferred checked assertion without object/set coercion. Five shared plus one dedicated backlink, all 5,039 nonidentity orders, the finite source/provenance/corruption matrix, all 35 pre-existing owner routes, immutable output, and a real sidecar protect active case 158 within 373 cases and 337 requirements. No existing expectation changed. Multi-hop ChainObjectMode4 through BaseObjectMode, imported-positive expansion, and broader semantics remain deferred. Step 5 remains active; Steps 6/7 remain deferred. No checker source or module-layout change was required.

## Step 5 Task 211 Addendum

Task 211 closes the exact two-edge set-terminal two-hop asserted-head `test_gap`, narrow `source_drift`, and `design_drift` slice without a `spec_gap`. Task 72 provides three real expansions and Tasks 148/186/203 provide the existing formula/checker seam and sibling guards. The new closed relation explicitly validates `OuterTwoHopModeAssertedHead -> MiddleTwoHopModeAssertedHead -> BaseTwoHopModeAssertedHead` plus the Base-to-set terminal; generic terminal traversal alone is not relation evidence. The active route preserves distinct Outer/Base provenance, resolves ordinal 1 to `BindingId(0)`, consumes three expansions, normalizes three known entries to one Base-definition-RHS builtin-set identity, and records one inferred variable plus one zero-constraint/fact/candidate/diagnostic/deferred checked assertion. Five shared plus one dedicated backlink, all five nonidentity orders, the finite structural/provenance/corruption matrix, all 36 prior owner routes, immutable output, and a real sidecar protect active case 159 within 374 cases and 338 requirements. No existing expectation changed. The object sibling, other distances, generic reachability, and broader semantics remain deferred. Step 5 remains active; Steps 6/7 remain deferred. No checker source or module-layout change was required.

## Step 5 Task 212 Addendum

Task 212 closes the exact two-edge object-terminal two-hop asserted-head `test_gap`, narrow `source_drift`, and `design_drift` slice without a `spec_gap`. Task 72 provides three real object expansions and Tasks 149/187/204/211 provide the existing formula/checker seam, object siblings, and closed two-link relation. The relation explicitly validates `OuterTwoHopObjectModeAssertedHead -> MiddleTwoHopObjectModeAssertedHead -> BaseTwoHopObjectModeAssertedHead` plus the Base-to-object terminal; generic terminal traversal alone is not relation evidence. The active route preserves distinct Outer/Base provenance, resolves ordinal 1 to `BindingId(0)`, consumes three expansions, normalizes three known entries to one Base-definition-RHS builtin-object identity, and records one inferred variable plus one zero-constraint/fact/candidate/diagnostic/deferred checked assertion without object/set coercion. Five shared plus one dedicated backlink, all five nonidentity orders, the finite structural/provenance/corruption matrix, all 37 prior owner routes, immutable output, and a real sidecar protect active case 160 within 375 cases and 339 requirements. No existing expectation changed. Other distances, generic reachability, object/set coercion, and broader semantics remain deferred. Step 5 remains active; Steps 6/7 remain deferred. No checker source or module-layout change was required.

## Step 5 Task 213 Addendum

Task 213 closes the exact three-edge set-terminal two-hop asserted-head `test_gap`, narrow `source_drift`, and `design_drift` slice without a `spec_gap`. Task 73 provides four real expansions and Tasks 195/205/211/212 provide the formula/checker seam, immediate-edge sibling, closed two-link relation, and object-terminal guard. The refined relation directly validates the pairwise-distinct `OuterThreeEdgeModeTwoHopAssertedHead -> MiddleThreeEdgeModeTwoHopAssertedHead -> InnerThreeEdgeModeTwoHopAssertedHead` relation; the remaining Inner-to-Base-to-set tail is terminal normalization only and never relation evidence. The active route preserves distinct Outer/Inner provenance, resolves ordinal 1 to `BindingId(0)`, consumes four expansions, normalizes three known entries to one Base-definition-RHS builtin-set identity, and records one inferred variable plus one zero-constraint/fact/candidate/diagnostic/deferred checked assertion. Five shared plus one dedicated backlink, all 23 nonidentity orders, the finite structural/provenance/corruption matrix, focused Task 211/212 regressions, all 38 prior owner routes, immutable output, and a real sidecar protect active case 161 within 376 cases and 340 requirements, with type-elaboration coverage 208/196. No existing expectation changed. The object sibling, full-distance assertion, generic reachability, and broader semantics remain deferred. Step 5 remains active; Steps 6/7 remain deferred. No checker source or module-layout change was required.

## Step 5 Task 214 Addendum

Task 214 closes the exact three-edge object-terminal two-hop asserted-head `test_gap`, narrow `source_drift`, and `design_drift` slice without a `spec_gap`. Task 73 provides four real object expansions and Tasks 196/206/211/212/213 provide the formula/checker seam, immediate-edge sibling, unchanged closed two-link relation, and set-terminal guard. The relation directly validates the pairwise-distinct `OuterThreeEdgeObjectModeTwoHopAssertedHead -> MiddleThreeEdgeObjectModeTwoHopAssertedHead -> InnerThreeEdgeObjectModeTwoHopAssertedHead` relation; the remaining Inner-to-Base-to-object tail is terminal normalization only and never relation evidence. The active route preserves distinct Outer/Inner provenance, resolves ordinal 1 to `BindingId(0)`, consumes four expansions, normalizes three known entries to one Base-definition-RHS builtin-object identity, and records one inferred variable plus one zero-constraint/fact/candidate/diagnostic/deferred checked assertion without object/set coercion. Five shared plus one dedicated backlink, all 23 nonidentity orders, the finite structural/provenance/corruption matrix, focused Task 211/212/213 regressions, all 39 prior owner routes, immutable output, and a real sidecar protect active case 162 within 377 cases and 341 requirements, with type-elaboration coverage 209/197 and pass/fail 193/184. No existing expectation changed. Full-distance assertion, generic reachability, object/set coercion, and broader semantics remain deferred. Step 5 remains active; Steps 6/7 remain deferred. No checker source or module-layout change was required.

## Step 5 Task 215 Addendum

Task 215 closes the exact four-edge set-terminal two-hop asserted-head `test_gap`, narrow `source_drift`, and `design_drift` slice without a `spec_gap`. Task 74 provides five real set expansions, Task 197 provides the formula/checker consumer, Task 207 provides the immediate-edge sibling, and Tasks 211-214 protect the unchanged closed two-link relation. The relation directly validates the pairwise-distinct `TooDeepFourEdgeModeTwoHopAssertedHead -> OuterFourEdgeModeTwoHopAssertedHead -> MiddleFourEdgeModeTwoHopAssertedHead` relation; the remaining Middle-to-Inner-to-Base-to-set tail is terminal normalization only and never relation evidence. The active route preserves distinct TooDeep/Middle provenance, resolves ordinal 1 to `BindingId(0)`, consumes five expansions, normalizes three known entries to one Base-definition-RHS builtin-set identity, and records one inferred variable plus one zero-constraint/fact/candidate/diagnostic/deferred checked assertion without object/set coercion. Five shared plus one dedicated backlink, all 119 nonidentity orders, the finite structural/provenance/corruption matrix, focused Tasks 211-214 regressions, all 40 prior owner routes, immutable output, and a real sidecar protect active case 163 within 378 cases and 342 requirements, with type-elaboration coverage 210/198 and pass/fail 194/184. No existing expectation changed. The object sibling, three-hop/full-distance assertion, generic reachability, object/set coercion, and broader semantics remain deferred. Step 5 remains active; Steps 6/7 remain deferred. No checker source or module-layout change was required.

## Step 5 Task 216 Addendum

Task 216 closes the exact four-edge object-terminal two-hop asserted-head `test_gap`, narrow `source_drift`, and `design_drift` slice without a `spec_gap`. Task 74 provides five real object expansions, Task 198 provides the formula/checker consumer, Task 208 provides the immediate-edge sibling, and Tasks 211-215 protect the byte-for-byte unchanged closed two-link relation. The relation directly validates the pairwise-distinct `TooDeepFourEdgeObjectModeTwoHopAssertedHead -> OuterFourEdgeObjectModeTwoHopAssertedHead -> MiddleFourEdgeObjectModeTwoHopAssertedHead` relation; the remaining Middle-to-Inner-to-Base-to-object tail is terminal normalization only and never relation evidence. The active route preserves distinct TooDeep/Middle provenance, resolves ordinal 1 to `BindingId(0)`, consumes five expansions, normalizes three known entries to one Base-definition-RHS builtin-object identity, and records one inferred variable plus one zero-constraint/fact/candidate/diagnostic/deferred checked assertion without object/set coercion. Five shared plus one dedicated backlink, all 119 nonidentity orders, the finite structural/provenance/corruption matrix, focused Tasks 211-215 regressions, all 41 prior owner routes, immutable output, and a real sidecar protect active case 164 within 379 cases and 343 requirements, with type-elaboration coverage 211/199 and pass/fail 195/184. No existing expectation changed. Three-hop/full-distance assertions, generic reachability, object/set coercion, and broader semantics remain deferred. Step 5 remains active; Steps 6/7 remain deferred. No checker source or module-layout change was required.

## Step 5 Task 217 Addendum

Task 217 closes the exact three-edge set-terminal full-distance three-hop asserted-head `test_gap`, narrow `source_drift`, and `design_drift` slice without a `spec_gap`. Task 73 provides four real set expansions, Task 195 provides the formula/checker consumer, and Tasks 211-216 provide shorter-distance and terminal-sibling guards. The new closed relation directly validates the pairwise-distinct `OuterThreeEdgeModeThreeHopAssertedHead -> MiddleThreeEdgeModeThreeHopAssertedHead -> InnerThreeEdgeModeThreeHopAssertedHead -> BaseThreeEdgeModeThreeHopAssertedHead` links; Base-to-set is terminal normalization only and never relation evidence. The active route preserves distinct Outer/Base provenance, resolves ordinal 1 to `BindingId(0)`, consumes four expansions, normalizes three known entries to one Base-definition-RHS builtin-set identity, and records one inferred variable plus one zero-constraint/fact/candidate/diagnostic/deferred checked assertion. Five shared plus one dedicated backlink, all 23 nonidentity orders, the finite structural/provenance/corruption matrix, focused Tasks 211-216 regressions, all 42 prior owner routes, immutable output, and a real sidecar protect active case 165 within 380 cases and 344 requirements, with type-elaboration coverage 212/200 and pass/fail 196/184. No existing expectation changed. The object sibling, other depths, generic reachability, object/set coercion, and broader semantics remain deferred. Step 5 remains active; Steps 6/7 remain deferred. No checker source or module-layout change was required.

## Step 5 Task 218 Addendum

Task 218 closes the exact three-edge object-terminal full-distance three-hop asserted-head `test_gap`, narrow `source_drift`, and `design_drift` slice without a `spec_gap`. Task 73 provides four real object expansions, Task 196 provides the formula/checker consumer, and Tasks 211-217 provide shorter-distance and terminal-sibling guards plus the byte-for-byte unchanged `BindingThreeHopRadix`. The active route directly validates the pairwise-distinct `OuterThreeEdgeObjectModeThreeHopAssertedHead -> MiddleThreeEdgeObjectModeThreeHopAssertedHead -> InnerThreeEdgeObjectModeThreeHopAssertedHead -> BaseThreeEdgeObjectModeThreeHopAssertedHead` links; Base-to-object is terminal normalization only and never relation evidence. It preserves distinct Outer/Base provenance, resolves ordinal 1 to `BindingId(0)`, consumes four expansions, normalizes three known entries to one Base-definition-RHS builtin-object identity, and records one inferred variable plus one zero-constraint/fact/candidate/diagnostic/deferred checked assertion without object/set coercion. Five shared plus one dedicated backlink, all 23 nonidentity orders, the finite structural/provenance/corruption matrix, focused Tasks 211-217 regressions, all 43 prior owner routes, immutable output, and a real sidecar protect active case 166 within 381 cases and 345 requirements, with type-elaboration coverage 213/201 and pass/fail 197/184. No existing expectation changed. Other depths, generic reachability, object/set coercion, and broader semantics remain deferred. Step 5 remains active; Steps 6/7 remain deferred. No checker source or module-layout change was required.

## Step 5 Task 219 Addendum

Task 219 closes the exact four-edge set-terminal three-hop asserted-head `test_gap`, narrow `source_drift`, and `design_drift` slice without a `spec_gap`. Task 74 provides five real set expansions, Task 197 provides the formula/checker consumer, Task 207 provides the four-edge immediate-edge sibling guard, and Tasks 211-218 provide shorter-distance and terminal-sibling guards plus the byte-for-byte unchanged `BindingThreeHopRadix`. The active route directly validates the pairwise-distinct `TooDeepFourEdgeModeThreeHopAssertedHead -> OuterFourEdgeModeThreeHopAssertedHead -> MiddleFourEdgeModeThreeHopAssertedHead -> InnerFourEdgeModeThreeHopAssertedHead` links; the Inner-to-Base-to-set tail is terminal normalization only and never relation evidence. It preserves distinct TooDeep/Inner provenance, resolves ordinal 1 to `BindingId(0)`, consumes five expansions, normalizes three known entries to one Base-definition-RHS builtin-set identity, and records one inferred variable plus one zero-constraint/fact/candidate/diagnostic/deferred checked assertion. Five shared plus one dedicated backlink, all 119 nonidentity orders, the finite structural/provenance/corruption matrix with independent guards for an unconnected unsupported deeper asserted head and an actual connected sixth-definition/sixth-edge asserted head, focused Tasks 207 and 211-218 regressions, all 44 prior owner routes, immutable output, and a real sidecar protect active case 167 within 382 cases and 346 requirements, with type-elaboration coverage 214/202 and pass/fail 198/184. No existing expectation changed. The object sibling, Base full-distance assertion, generic reachability, object/set coercion, and broader semantics remain deferred. Step 5 remains active; Steps 6/7 remain deferred. No checker source or module-layout change was required.

## Step 5 Task 220 Addendum

Task 220 closes the exact four-edge object-terminal three-hop asserted-head `test_gap`, narrow `source_drift`, and `design_drift` slice without a `spec_gap`. Task 74 provides five real object expansions, Task 198 provides the formula/checker consumer, Task 208 provides the four-edge immediate-edge sibling guard, and Tasks 211-219 provide shorter-distance and terminal-sibling guards plus the byte-for-byte unchanged `BindingThreeHopRadix`. The active route directly validates the pairwise-distinct `TooDeepFourEdgeObjectModeThreeHopAssertedHead -> OuterFourEdgeObjectModeThreeHopAssertedHead -> MiddleFourEdgeObjectModeThreeHopAssertedHead -> InnerFourEdgeObjectModeThreeHopAssertedHead` links; the Inner-to-Base-to-object tail is terminal normalization only and never relation evidence. It preserves distinct TooDeep/Inner provenance, resolves ordinal 1 to `BindingId(0)`, consumes five expansions, normalizes three known entries to one Base-definition-RHS builtin-object identity, and records one inferred variable plus one zero-constraint/fact/candidate/diagnostic/deferred checked assertion without object/set coercion. Five shared plus one dedicated backlink, all 119 nonidentity orders, the finite structural/provenance/corruption matrix with independent guards for an unconnected unsupported deeper asserted head and an actual connected sixth-definition/sixth-edge asserted head, focused Tasks 208 and 211-219 regressions, all 45 prior owner routes, immutable output, and a real sidecar protect active case 168 within 383 cases and 347 requirements, with type-elaboration coverage 215/203 and pass/fail 199/184. No existing expectation changed. The Base full-distance assertion, generic reachability, object/set coercion, and broader semantics remain deferred. Step 5 remains active; Steps 6/7 remain deferred. No checker source or module-layout change was required.

## Step 5 Task 221 Active Addendum

Task 221 closes the exact four-edge set-terminal full-distance four-hop asserted-head `test_gap`, narrow `source_drift`, and `design_drift` slice without a `spec_gap`. Task 74 provides five real set expansions and Task 197 provides the real formula/checker consumer. The active closed `BindingFourHopRadix` directly validates `TooDeepFourEdgeModeFourHopAssertedHead -> OuterFourEdgeModeFourHopAssertedHead -> MiddleFourEdgeModeFourHopAssertedHead -> InnerFourEdgeModeFourHopAssertedHead -> BaseFourEdgeModeFourHopAssertedHead`; Base-to-set is terminal normalization only and never relation evidence. The route preserves distinct TooDeep/Base provenance, ordinal 1 / `BindingId(0)`, five expansions, one Base-definition-RHS builtin-set identity, one inferred variable, and one zero-constraint/fact/candidate/diagnostic/deferred checked assertion. Six backlinks, all 119 nonidentity orders, the exhaustive finite structural/provenance/corruption matrix with independent unconnected-deeper and actual connected fifth-link guards, focused Task 207 and Tasks 211-220 regressions, all 46 prior owner routes, immutable output, and a real sidecar protect active case 169 within 384 cases and 348 requirements, with type-elaboration coverage 216/204 and pass/fail 200/184. Existing expectations remain unchanged. The object sibling, longer chains, imported-positive definitions, attributed/argument-bearing behavior, generic reachability, object/set coercion, broader semantics, proof/CoreIr/ControlFlowIr/VC, and general chains remain deferred. Step 5 stays active; Steps 6/7 stay deferred. Relevant crate verification passed; no checker source or module-layout change was required.

## Step 5 Task 222 Active Addendum

Task 222 closes the exact four-edge object-terminal full-distance four-hop asserted-head `test_gap`, narrow `source_drift`, and `design_drift` slice without a `spec_gap`. Task 74 provides five real object expansions and Task 198 provides the real formula/checker consumer. The byte-for-byte unchanged closed `BindingFourHopRadix` directly validates `TooDeepFourEdgeObjectModeFourHopAssertedHead -> OuterFourEdgeObjectModeFourHopAssertedHead -> MiddleFourEdgeObjectModeFourHopAssertedHead -> InnerFourEdgeObjectModeFourHopAssertedHead -> BaseFourEdgeObjectModeFourHopAssertedHead`; Base-to-object is terminal normalization only and never relation evidence. The route preserves distinct TooDeep/Base provenance, ordinal 1 / `BindingId(0)`, five expansions, one Base-definition-RHS builtin-object identity, one inferred variable, and one zero-constraint/fact/candidate/diagnostic/deferred checked assertion without object/set coercion. Six backlinks, all 119 nonidentity orders, the exhaustive finite structural/provenance/corruption matrix with independent unconnected-deeper and actual connected fifth-link guards, focused Task 208 and Tasks 211-221 regressions, all 47 prior owner routes, immutable output, and a real sidecar protect active case 170 within 385 cases and 349 requirements, with type-elaboration coverage 217/205 and pass/fail 201/184. Existing expectations remain unchanged. Relevant-crate and workspace verification passed. Longer chains, imported-positive definitions, attributed/argument-bearing behavior, generic reachability, object/set coercion, broader semantics, proof/CoreIr/ControlFlowIr/VC, and general chains remain deferred. Step 5 stays active; Steps 6/7 stay deferred. No checker source or module-layout change was required.

## Step 5 Task 223 Active Addendum

Task 223 closes the exact single-left-parenthesized reserved-variable equality `test_gap`, narrow `source_drift`, and `design_drift` slice without a `spec_gap`. The real parser provides `ParenthesizedTerm`; Task 119 provides reserve extraction, `BindingEnv` lookup, builtin-set type projection, and the equality consumer. The active route validates `(x) = x`, preserves independent wrapper/inner/right source metadata, resolves inner/right uses at ordinals 1/2 to `BindingId(0)`, and transparently reuses the inner reference's real reserve-derived type/value without fabricating an independent parenthesis type, axiom, fact, FOL node, or child payload. Five backlinks, the finite direct/right/both/nested/non-identifier/recovery/reserve/formula/provenance/corruption matrix, all 52 prior reserved-variable binary-formula owners bidirectionally, immutable output, and a real sidecar protect active case 171 within 386 cases and 350 requirements, with type-elaboration coverage 218/206 and pass/fail 202/184. Existing expectations remain unchanged. Focused, relevant-crate, and workspace verification passed. Arbitrary nesting/operands/precedence, formula grouping, closure/order materialization, equality truth/facts, theorem acceptance, child graphs, proof/CoreIr/ControlFlowIr/VC, and broader semantics remain deferred. Step 5 stays active; Steps 6/7 stay deferred. No checker source or module-layout change was required.

## Step 5 Task 224 Active Addendum

Task 224 closes the exact seven-expansion set-terminal two-hop asserted-head `test_gap`, narrow `source_drift`, and `design_drift` slice without a `spec_gap`. Task 74 supplies seven real AST expansions, Task 199 supplies the formula/checker consumer, Task 211 supplies the unchanged `BindingTwoHopRadix`, and Task 209 is an immediate-edge sibling regression only. The route directly validates pairwise-distinct `ChainMode6 -> ChainMode5 -> ChainMode4`, uses the remaining tail for terminal normalization only, and preserves distinct provenance, ordinal 1 / `BindingId(0)`, one BaseModeDef-RHS builtin-set identity, one inferred variable, and one zero-constraint/fact/candidate/diagnostic/deferred checked assertion. Six backlinks, all 5,039 nonidentity orders, the finite structural/provenance/corruption matrix, all 48 prior owners, immutable output, and a real sidecar protect active case 172 within 387 cases / 351 requirements, type-elaboration 219/207, and pass/fail 203/184 without changing existing expectations. Focused, relevant-crate, and workspace verification passed. Broader semantics and downstream payloads remain deferred; Step 5 stays active and Steps 6/7 stay deferred. No checker source or module-layout change was required.

## Step 5 Task 225 Active Addendum

Task 225 closes the exact seven-expansion object-terminal two-hop asserted-head `test_gap`, narrow `source_drift`, and `design_drift` slice without a `spec_gap`. Task 74 supplies seven real object AST expansions, Task 200 supplies the formula/checker consumer, Task 211 supplies the unchanged `BindingTwoHopRadix`, Task 210 is the immediate-edge sibling, and Task 224 is the set-terminal two-hop sibling. The route directly validates pairwise-distinct `ChainObjectMode6 -> ChainObjectMode5 -> ChainObjectMode4`, uses the remaining tail for object-terminal normalization only, and preserves distinct provenance, ordinal 1 / `BindingId(0)`, one BaseObjectModeDef-RHS builtin-object identity, one inferred variable, and one zero-constraint/fact/candidate/diagnostic/deferred checked assertion without object/set coercion. Six backlinks, all 5,039 nonidentity orders, the finite structural/provenance/corruption matrix, all 49 prior owners, immutable output, and a real sidecar protect active case 173 within 388 cases / 352 requirements, type-elaboration 220/208, and pass/fail 204/184 without changing existing expectations. Focused, relevant-crate, and workspace verification passed. Broader semantics and downstream payloads remain deferred; Step 5 stays active and Steps 6/7 stay deferred. No checker source or module-layout change was required.

## Step 5 Task 226 Active Addendum

Task 226 closes the exact seven-expansion set-terminal three-hop asserted-head `test_gap`, narrow `source_drift`, and `design_drift` slice without a `spec_gap`. Task 74 supplies seven real set AST expansions, Task 199 supplies the formula/checker consumer, and Task 217 supplies the unchanged `BindingThreeHopRadix`; Task 219 is the set-terminal three-hop longer-tail sibling, while Tasks 209/224 are the immediate/two-hop long-chain siblings. The route directly validates pairwise-distinct `ChainMode6 -> ChainMode5 -> ChainMode4 -> ChainMode3`, uses the remaining tail for set-terminal normalization only, and preserves distinct provenance, ordinal 1 / `BindingId(0)`, one BaseModeDef-RHS builtin-set identity, one inferred variable, and one zero-constraint/fact/candidate/diagnostic/deferred checked assertion. Six backlinks, all 5,039 nonidentity orders, the finite structural/provenance/corruption matrix, all 50 prior owners, immutable output, and a real sidecar protect active case 174 within 389 cases / 353 requirements, type-elaboration 221/209, and pass/fail 205/184 without changing existing expectations. Focused, relevant-crate, and workspace verification passed. The object sibling and downstream payloads remain deferred; Step 5 stays active and Steps 6/7 stay deferred. No checker source or module-layout change was required.

## Step 5 Task 227 Active Addendum

Task 227 closes the exact seven-expansion object-terminal three-hop asserted-head `test_gap`, narrow `source_drift`, and `design_drift` slice without a `spec_gap`. Task 74 supplies seven real object AST expansions, Task 200 supplies the formula/checker consumer, and Task 217 supplies the unchanged `BindingThreeHopRadix`; Task 220 is the object-terminal three-hop longer-tail sibling, Task 226 the depth-matched set sibling, and Tasks 210/225 the immediate/two-hop object long-chain siblings. The route directly validates pairwise-distinct `ChainObjectMode6 -> ChainObjectMode5 -> ChainObjectMode4 -> ChainObjectMode3`, uses the remaining tail for object-terminal normalization only, and preserves distinct provenance, ordinal 1 / `BindingId(0)`, one BaseObjectModeDef-RHS builtin-object identity, one inferred variable, and one zero-constraint/fact/candidate/diagnostic/deferred checked assertion without object/set coercion. Six backlinks, all 5,039 nonidentity orders, the finite structural/provenance/corruption matrix, all 51 prior owners, immutable output, and a real sidecar protect active case 175 within 390 cases / 354 requirements, type-elaboration 222/210, and pass/fail 206/184 without changing existing expectations. Focused, relevant-crate, and workspace verification passed. Broader semantics and downstream payloads remain deferred; Step 5 stays active and Steps 6/7 stay deferred. No checker source or module-layout change was required.

## Step 5 Task 228 Active Addendum

Task 228 closes the exact seven-expansion set-terminal four-hop asserted-head `test_gap`, narrow `source_drift`, and `design_drift` slice without a `spec_gap`. Task 74 supplies seven real set AST expansions, Task 199 supplies the formula/checker consumer, and Task 221 supplies the unchanged `BindingFourHopRadix`; Tasks 224/226 are the shorter-distance long-chain siblings, Task 222 the object-terminal relation sibling, and Task 227 the latest terminal sibling. The route directly validates pairwise-distinct `ChainMode6 -> ChainMode5 -> ChainMode4 -> ChainMode3 -> ChainMode2`, uses the remaining tail for set-terminal normalization only, and preserves distinct provenance, ordinal 1 / `BindingId(0)`, one BaseModeDef-RHS builtin-set identity, one inferred variable, and one zero-constraint/fact/candidate/diagnostic/deferred checked assertion. Six backlinks, all 5,039 nonidentity orders, the finite structural/provenance/corruption matrix, all 52 prior owners, immutable output, and a real sidecar protect active case 176 within 391 cases / 355 requirements, type-elaboration 223/211, and pass/fail 207/184 without changing existing expectations. Focused, relevant-crate, and workspace verification passed. The object sibling, broader semantics, and downstream payloads remain deferred; Step 5 stays active and Steps 6/7 stay deferred. No checker source or module-layout change was required.

## Step 5 Task 229 Active Addendum

Task 229 closes the exact seven-expansion object-terminal four-hop asserted-head `test_gap`, narrow `source_drift`, and `design_drift` slice without a `spec_gap`. Task 74 supplies seven real object AST expansions, Task 200 supplies the formula/checker consumer, and Task 221 supplies the unchanged `BindingFourHopRadix`; Tasks 225/227 are the shorter-distance object siblings, Task 222 the object-terminal relation sibling, and Task 228 the depth-matched set sibling. The route directly validates pairwise-distinct `ChainObjectMode6 -> ChainObjectMode5 -> ChainObjectMode4 -> ChainObjectMode3 -> ChainObjectMode2`, uses the remaining tail for object-terminal normalization only, and preserves distinct provenance, ordinal 1 / `BindingId(0)`, one BaseObjectModeDef-RHS builtin-object identity, one inferred variable, and one zero-constraint/fact/candidate/diagnostic/deferred checked assertion without object/set coercion. Six backlinks, all 5,039 nonidentity orders, the finite structural/provenance/corruption matrix, all 53 prior owners, immutable output, and a real sidecar protect active case 177 within 392 cases / 356 requirements, type-elaboration 224/212, and pass/fail 208/184 without changing existing expectations. Focused, relevant-crate, and workspace verification passed. Imported-positive definitions, broader semantics, and downstream payloads remain deferred; Step 5 stays active and Steps 6/7 stay deferred. No checker source or module-layout change was required.

## Step 5 Task 230 Active Addendum

Task 230 closes the exact seven-expansion set-terminal five-hop asserted-head `test_gap`, narrow `source_drift`, and `design_drift` slice without a `spec_gap`. Task 74 supplies seven real set AST expansions and Task 199 supplies the formula/checker consumer. The new closed `BindingFiveHopRadix` directly validates pairwise-distinct `ChainMode6 -> ChainMode5 -> ChainMode4 -> ChainMode3 -> ChainMode2 -> ChainMode1`; `ChainMode1 -> BaseMode -> set` remains terminal-normalization evidence only. The route preserves distinct provenance, ordinal 1 / `BindingId(0)`, one BaseModeDef-RHS builtin-set identity, one inferred variable, and one zero-constraint/fact/candidate/diagnostic/deferred checked assertion. Six backlinks, all 5,039 nonidentity orders, the finite structural/provenance/corruption matrix, all 54 prior owners, immutable output, and a real sidecar protect active case 178 within 393 cases / 357 requirements, type-elaboration 225/213, and pass/fail 209/184 without changing existing expectations. Focused, relevant-crate, and workspace verification passed. Object-terminal five-hop, imported-positive definitions, broader semantics, and downstream payloads remain deferred; Step 5 stays active and Steps 6/7 stay deferred. No checker source or module-layout change was required.

## Step 5 Task 231 Active Addendum

Task 231 implements the exact seven-expansion object-terminal five-hop asserted-head `test_gap`, narrow `source_drift`, and `design_drift` slice without a `spec_gap`. Task 74 supplies seven real object AST expansions, Task 200 supplies the formula/checker consumer, and Task 230 supplies the byte-for-byte unchanged closed `BindingFiveHopRadix`. The active route directly validates pairwise-distinct `ChainObjectMode6 -> ChainObjectMode5 -> ChainObjectMode4 -> ChainObjectMode3 -> ChainObjectMode2 -> ChainObjectMode1`; `ChainObjectMode1 -> BaseObjectMode -> object` remains terminal-normalization evidence only. It preserves distinct provenance, ordinal 1 / `BindingId(0)`, one BaseObjectModeDef-RHS builtin-object identity, one inferred variable, and one zero-constraint/fact/candidate/diagnostic/deferred checked assertion without object/set coercion. Six backlinks, all 5,039 nonidentity orders, the finite structural/provenance/corruption matrix, all 55 prior owners, immutable output, and a real sidecar protect active case 179 within 394 cases / 358 requirements, type-elaboration 226/214, and pass/fail 210/184 without changing existing expectations. Focused, relevant-crate, and workspace verification passed. Imported-positive definitions, broader semantics, and downstream payloads remain deferred; Step 5 stays active and Steps 6/7 stay deferred. No checker source or module-layout change was required.

## Step 5 Task 233 Active Addendum

Task 233 classifies the exact single-left-parenthesized builtin-object reserved-variable equality seam as `test_gap`, narrow `source_drift`, and `design_drift`, not `spec_gap`. The active source is limited to `reserve x for object; theorem ParenthesizedReservedObjectVariableEqualityPayloadBoundary: (x) = x;`. Task 223 supplies the real unrecovered `ParenthesizedTerm` producer and Task 188 the real object reserve/BindingEnv/equality consumer. The route preserves independent wrapper/inner/right provenance, resolves ordinal 1/2 to `BindingId(0)`, transparently reuses one canonical builtin-object identity for two inferred terms and two ordered expected constraints, and produces one checked equality without object/set coercion or an independent wrapper payload. Six backlinks, a finite exact/near-miss/provenance/corruption matrix, all 53 prior binary-formula owners bidirectionally, immutable output, and a real sidecar protect active runner 180 within 395 cases / 359 requirements, type-elaboration 227/215, and pass/fail 211/184 without changing existing expectations. Arbitrary parentheses/operands, formula grouping, truth/facts, acceptance, proof/IR/VC, child graphs, and broader semantics remain deferred; Step 5 stays active and Steps 6/7 stay deferred. No checker source or module-layout change was required.

## Step 5 Task 234 Active Addendum

Task 234 classifies the exact seven-expansion set-terminal full-distance six-hop asserted-head seam as `test_gap`, narrow `source_drift`, and `design_drift`, not `spec_gap`. Task 74 supplies seven real set AST expansions and Task 199 supplies the real formula/checker consumer. The new closed `BindingSixHopRadix` directly validates pairwise-distinct `ChainMode6 -> ChainMode5 -> ChainMode4 -> ChainMode3 -> ChainMode2 -> ChainMode1 -> BaseMode`; `BaseMode -> set` remains terminal-normalization evidence only. The route preserves distinct provenance, ordinal 1 / `BindingId(0)`, one BaseModeDef-RHS builtin-set identity, one inferred variable, and one zero-constraint/fact/candidate/diagnostic/deferred checked assertion. Six backlinks, all 5,039 nonidentity orders, the finite structural/provenance/corruption matrix, all 56 prior owners, immutable output, and a real sidecar protect active case 181 within 396 cases / 360 requirements, type-elaboration 228/216, and pass/fail 212/184 without changing existing expectations. Object-terminal six-hop, imported-positive definitions, broader semantics, and downstream payloads remain deferred; Step 5 stays active and Steps 6/7 stay deferred. No checker source or module-layout change was required.

## Step 5 Task 236 Active Addendum

Task 236 classifies the exact seven-expansion object-terminal full-distance six-hop asserted-head seam as `test_gap`, narrow `source_drift`, and `design_drift`, not `spec_gap`. Task 74 supplies seven real object AST expansions and Task 200 supplies the real formula/checker consumer. The byte-for-byte unchanged closed `BindingSixHopRadix` directly validates pairwise-distinct `ChainObjectMode6 -> ChainObjectMode5 -> ChainObjectMode4 -> ChainObjectMode3 -> ChainObjectMode2 -> ChainObjectMode1 -> BaseObjectMode`; `BaseObjectMode -> object` remains terminal-normalization evidence only. The route preserves distinct provenance, ordinal 1 / `BindingId(0)`, one BaseObjectModeDef-RHS builtin-object identity, one inferred variable, and one zero-constraint/fact/candidate/diagnostic/deferred checked assertion without object/set coercion. Six backlinks, all 5,039 nonidentity orders, the finite structural/provenance/corruption matrix, all 57 prior owners, immutable output, and a real sidecar protect active case 182 within 397 cases / 361 requirements, type-elaboration 229/217, and pass/fail 213/184 without changing existing expectations. Imported-positive definitions, broader semantics, and downstream payloads remain deferred; Step 5 stays active and Steps 6/7 stay deferred. No checker source or module-layout change was required.

## Step 5 Task 241 Active Addendum

Task 241 closes the exact single-left-parenthesized builtin-set reserved-variable
inequality `test_gap`, narrow `source_drift`, and `design_drift` slice without a
`spec_gap`. The active source is only `reserve x for set; theorem
ParenthesizedReservedVariableInequalityPayloadBoundary: (x) <> x;`. Task 223
supplies the real one-child `ParenthesizedTerm` producer and Task 121 the real
reserve/`BindingEnv`/inequality consumer. The route preserves independent
wrapper/inner/right provenance, resolves ordinal 1/2 to `BindingId(0)`, reuses
one canonical builtin-set identity for two inferred terms and two ordered
expected constraints, and produces one fact/candidate/diagnostic/deferred-free
checked inequality without an independent wrapper payload. Four shared plus one
dedicated backlink, the finite exact/near-miss/provenance/corruption matrix, all
54 prior binary-formula owners bidirectionally, focused equality regressions,
immutable output, and a real sidecar protect active case 183 within 398 cases /
362 requirements, type-elaboration 230/218, and pass/fail 214/184 without
changing existing fixtures or expectations. Parenthesized membership,
imported/other parenthesized variants, desugaring/truth, acceptance, proof/IR/VC,
child graphs, and downstream payloads receive no Task 241 credit. Step 5 stays
active and Steps 6/7 stay deferred. No checker source/API/module-layout change
was required.

## Step 5 Task 242 Active Addendum

Task 242 closes the exact single-left-parenthesized builtin-object reserved-
variable inequality `test_gap`, narrow `source_drift`, and `design_drift` slice
without a `spec_gap`. The active source is only `reserve x for object; theorem
ParenthesizedReservedObjectVariableInequalityPayloadBoundary: (x) <> x;`. Task
233 supplies the real one-child object `ParenthesizedTerm` producer and Task 190
the real builtin-object reserve/`BindingEnv`/inequality consumer. The route
preserves independent wrapper/inner/right provenance, resolves ordinal 1/2 to
`BindingId(0)`, reuses one written-`object`-anchored canonical `BuiltinObject`
for two inferred terms, six type entries, and two ordered expected constraints,
and produces one fact/candidate/diagnostic/deferred-free checked inequality
without object/set coercion or an independent wrapper payload. Five shared plus
one dedicated backlink, the finite exact/near-miss/provenance/corruption matrix,
all 55 prior binary-formula owners bidirectionally, focused Tasks 190/223/233/
241, immutable output, and a real sidecar protect active case 184 within 399
cases / 363 requirements, type-elaboration 231/219, and pass/fail 215/184 without
changing existing fixtures or expectations. Parenthesized membership and active
imported provenance receive no Task 242 credit; missing imported expansion/
evidence/signature payloads and proof/CoreIr/ControlFlowIr/VC remain deferred.
Step 5 stays active and Steps 6/7 stay deferred. No checker source/API/module-
layout change was required.

## Step 5 Task 243 Active Addendum

Task 243 closes the exact single-left-parenthesized builtin-set reserved-variable
membership `test_gap`, narrow `source_drift`, and `design_drift` slice without a
`spec_gap`. The active source is only `reserve x for set; theorem
ParenthesizedReservedVariableMembershipPayloadBoundary: (x) in x;`. Task 223
supplies the real one-child `ParenthesizedTerm` producer and Task 120 the real
reserve/`BindingEnv`/membership consumer, whose unchanged direct-right producer
supplies the sole expected-set input. The route preserves independent wrapper/
inner/right provenance, resolves ordinal 1/2 to `BindingId(0)`, reuses one
written-`set`-anchored canonical `BuiltinSet` for two inferred terms, exactly
five type entries, no left expected input, and one right-owned expected-set
constraint, and produces one fact/candidate/diagnostic/deferred-free checked
membership without an independent wrapper payload. Four shared plus one
dedicated backlink, the finite matrix, all 56 prior binary-formula owners
bidirectionally, focused Tasks 120/223/233/241/242, immutable output, and a real
sidecar protect active case 185 within 400 cases / 364 requirements, type-
elaboration 232/220, and pass/fail 216/184 without changing existing fixtures or
expectations. The former extraction gap is discharged only for this exact
source. Object-left/set-right parenthesized membership and active imported
provenance receive no Task 243 credit; missing imported expansion/evidence/
signature payloads and proof/CoreIr/ControlFlowIr/VC remain deferred. Step 5
stays active and Steps 6/7 stay deferred. No checker source/API/module-layout
change was required.

## Step 5 Task 244 Active Addendum

Task 244 closes the exact two-reserve single-left-parenthesized heterogeneous
membership `test_gap` plus narrow `source_drift` and `design_drift`, not a
`spec_gap`. The test-first source is `reserve x for object; reserve y for set;
theorem ParenthesizedHeterogeneousReserveMembershipPayloadBoundary: (x) in y;`,
derived from Chapters 03/04/13/14/16, Task 233's real object wrapper producer,
and Task 125's existing two-binding membership intent.

The finite config-driven bridge preserves all five prior parenthesized routes
and requires two ordered distinct reserves, ordinals 2/3, `BindingId(0/1)`, two
written-range-anchored object/set identities, two inferred terms, five type
entries, no left expected input, one right expected-set constraint, and a
fact/candidate/diagnostic/deferred/coercion/wrapper-semantic-free checked
membership. Exact/near-miss/provenance/corruption and immutable-output probes,
all 57 prior owners, Tasks 120/125/223/233/241/242/243 regressions, a real
imported-mode-gap diagnostic guard, and a real frontend/resolver sidecar protect
the seam.

The active runner becomes 186. Traceability adds five shared backlinks plus one
dedicated requirement; metadata becomes 401 cases / 365 requirements, type
233/221, and pass/fail 217/184 without rebaselining existing expectations. Only
the exact source discharges the extraction gap. Other parenthesized shapes and
imported-positive provenance receive no Task 244 credit; missing imported
expansion/evidence/signature payloads and proof/CoreIr/ControlFlowIr/VC remain
deferred. Step 5 stays active and Steps 6/7 stay deferred. No checker source/
API/module-layout change is required.

## Step 5 Task 245 Active Addendum

Task 245 closes the exact right-parenthesized builtin-set membership `test_gap`
plus narrow `source_drift` and `design_drift`, not a `spec_gap`. The test-first
source is `reserve x for set; theorem
RightParenthesizedReservedVariableMembershipPayloadBoundary: x in (x);`, derived
from Chapters 04/13/14/16, the real parser wrapper producer, and Task 120's
existing real membership/expected-set consumer.

The route retains an explicit `Right` side and Task-245-only config/key/roles,
resolves direct-left/right-inner ordinals 1/2 to `BindingId(0)`, preserves one
written-set identity, two inferred terms, five type entries, a right-inner-owned
sole expected constraint, and one clean checked membership without wrapper
semantic output. Side/config/range/constraint corruptions, Task-243 cross-route
rejection, all 58 prior owners bidirectionally, all six left routes, immutable/
module boundaries, and a real sidecar guard the seam.

The active runner becomes 187. Traceability adds four shared backlinks plus one
dedicated row; metadata becomes 402/366, type 234/222, pass/fail 218/184 without
rebaselining. Only the exact source receives credit. Other shapes and imported-
positive provenance remain uncredited; missing imported expansion/evidence/
signature and proof/CoreIr/ControlFlowIr/VC remain deferred. Step 5 stays active
and Steps 6/7 stay deferred. No checker source/API/module-layout update was
required.

## Step 5 Task 246 Active Addendum

Task 246 closes the exact parenthesized two-edge set-terminal local-mode equality
`test_gap` plus narrow `source_drift` and `design_drift`, not a `spec_gap`. The
test-first source contains three ordered bare definitions `Base -> set`, `Middle
-> Base`, `Outer -> Middle`, an Outer reserve, and `(z) = z`. A Task-246-only
route conditionally admits mode-definition nodes, preserves three real
expansions and four raw Outer inputs, resolves ordinals 1/2 to `BindingId(0)`,
and emits two inferred terms, six entries, two ordered constraints, and one clean
checked equality sharing a Base-RHS `BuiltinSet`, with no wrapper output.
Finite order/shape/provenance/corruption, Tasks 134/223 cross-rejection,
immutable/module, 59 prior owners, and a real sidecar guard runner 188. Trace is
shared 5 + dedicated 1; metadata is 403/367, type 235/223, pass/fail 219/184.
Step 5 remains active; Steps 6/7 remain deferred. The next handoff must begin
with a fresh inventory rather than assuming a fixed successor seam.

## Step 5 Parser Task 47 Active Addendum

Parser Task 47 closes only the canonical `reconsider_tail` parser slice. The
private producer accepts omission with no justification child/diagnostic,
retains explicit `by`, and reuses `ProofBlock` plus existing `MissingEnd`
recovery. One new active pass fixture and the repaired mixed-recovery sidecar
cover exactly the omitted/proof-block trace rows; no existing `.miz` changed.

The plan is 405/369, parse-only is 97/97 with coverage 43/42, pass/fail is
221/184, and warnings/errors are 23/0. Declaration/type/proof admissions remain
5/188/1. The nonblocking Chapter-8 single-item versus list wording `spec_gap`
is human-owned. At this Task-47 checkpoint, Parser Task 48 was the next
authorized nonempty Step-5 task; Task 46 and Steps 6/7 remained deferred.

## Step 5 Parser Task 48 Active Addendum

Parser Task 48 closes P-265-48 with one dedicated top-level
`PropertyImplementation`, append-only syntax kind 192, exact specialized mode
parameter ownership, means/equals definientia and correctness ordering, and
nested-depth recovery that preserves the real outer terminator and following
declaration. The exact requirement
`spec.en.07.modes.property_implementation.parser` is covered by one new pass
and one new fail sidecar; existing `.miz`/expectations and the inactive
Task-39 semantic seed are unchanged.

The plan is 407/369, parse-only is 99/99 with coverage 43/43, pass/fail is
222/185, and warnings/errors are 23/0. Declaration/type/proof admissions remain
5/188/1. This closes the selected `source_drift`, `test_gap`, paired
`design_drift`, and two internal unit `test_expectation_drift` cases without
granting mode/property resolution, coherence/overlap, proof, checker/Core/CFG/
VC, Step 6, or Step 7 credit. Task 46 remains deferred; successor authority
must be established by fresh inventory rather than inferred from Task 48.

## Parser Crate Closeout Addendum

`PARSER-CRATE-CLOSEOUT` closes only the current `mizar-parser` crate milestone.
Tasks 1-45 and 47-48 are complete; the paired crate exit report records all
nine hard gates passing and an independent 94/100 score. P-043-01/P-046 remains
one trigger-deferred concrete operator-declaration gap, and P-265-47D remains a
separate nonblocking human-owned wording `spec_gap`.

No nonempty successor parser task is authorized. Global Step 5 remains active
for its other crate-owned work, and Task 49 plus Steps 6/7 are not promoted.
This docs-only closeout changes no specification coverage mapping, trace status,
owner, follow-up ownership, or deferred rationale; therefore
`doc/design/spec_coverage_audit.md` remains unchanged.

## Step 5 Parser Task 46 Active Addendum

Fresh inventory superseded the prior parser-closeout conclusion: completed
frontend Task 20 had already satisfied Task 46's named string-required and
local operator-metadata trigger. Task 46 closes aliased P-043-01/P-046 as one
bounded `source_drift` / `test_gap` plus paired `design_drift` slice. The parser
and syntax crates add exact infix/prefix/postfix declaration parsing,
annotated/visible top-level and definition-local placement, append-only
`OperatorDeclaration` raw kind 193, local recovery, one active pass/fail pair,
and an exact covered trace row.

The implementation is syntax-only and does not mutate Pratt metadata or claim
activation, active-functor validation, resolution, overload meaning, or
semantic precedence validation. Existing `.miz` sources and expectations stay
unchanged. The earlier 94/100 closeout is historical and superseded by the
separate post-Task-46 closeout below. Task 46 does not promote Task 49 or Steps
6/7 and does not close global Step 5.

Measured current oracles are plan 409/370, parse coverage 44/44,
parse/declaration/type/proof admission 101/5/188/1, pass/fail 223/186, and
warnings/errors 23/0. Parser production is 12 paths / 38,940 lines and parser
unit tests are 225; `mizar-test` production remains 18 paths / 20,088 lines and
its 276-test raw/normalized list remains unchanged.

## Step 5 Parser Post-Task-46 Closeout Addendum

`PARSER-CRATE-POST-TASK46-CLOSEOUT` closes only the parser milestone through
Tasks 1-48. All nine protocol hard gates pass and the fresh independent
read-only score is 99/100. P-265-47D remains a nonblocking human-owned
`spec_gap`. The independently classified overbroad frontend string-position
heuristic remains an external frontend `source_drift` /
`source_undocumented_behavior` with unit `test_expectation_drift` and receives
no parser credit.

No nonempty successor parser task is authorized. This closeout does not close
global Step 5, infer Task 49, or promote Steps 6/7. It changes no specification,
source, test, expectation, traceability row, coverage mapping, owner, or
deferred rationale; `doc/design/spec_coverage_audit.md` therefore remains
unchanged. Current counts and hashes are recorded in the paired parser
[exit report](./mizar-parser/en/crate_exit_report.md).

## Step 5 Checker Task 252 Frozen-Contract Addendum

Fresh canonical inventory after Checker Task 251 selects one nonempty
documentation prerequisite before Task 252 implementation. The paired
checker/mizar-test authority now freezes the source-only primary-term schema,
the exact three existing Task-10 consumers, aggregate 7/4/2
term/reference/numeric-request oracle, transparent-parenthesis boundary,
synthetic constant/`it` dependency tests, final ownership, corruption and
determinism coverage, trace/count impact, forbidden scope, and exit criteria.

At the prerequisite commit, this task changed no executable artifact or
coverage. The classified disagreements were the repaired `design_drift`,
continuing `source_drift`, and continuing `test_gap`; no blocking `spec_gap`,
`source_undocumented_behavior`, `test_expectation_drift`,
`boundary_violation`, or `repo_metadata_conflict` was found. Task 252 then
remained open for one separate implementation commit. Tasks 260/264/269 retain real
`it`/local-constant owner production, Tasks 253+ are not inferred, and
Steps 6/7 remain deferred.

A post-freeze review then found and corrected one further `design_drift` in
that documentation prerequisite. Task-252 reference ordinals count preceding
completed binding rows rather than merged binding/reference events; exact
duplicate-priority groups share the final dense row's visibility ordinal and
all lookup-priority inputs for `Ambiguous` rejection, and `Resolver` is
structurally unreachable on the payload-free lookup-site path.
At the correction commit, this change affected no executable artifact, coverage mapping, owner,
deferred rationale, count, or hash, so
`doc/design/spec_coverage_audit.md` remains unchanged.

## Step 5 Checker Task 252 Implementation Addendum

The corrected frozen contract is implemented in one logical task by a public
syntax-free checker producer and one private Task-10 source extractor. The
exact three existing routes publish aggregate term/reference/numeric-request
tables 7/4/2 through immutable `TypedAst` and clone-only `ResolvedTypedAst`
ownership. Synthetic constant, `it`, nested-parenthesis, and mixed-family
probes close the bounded producer/corruption coverage without changing a
semantic outcome or adding a fixture.

The new covered `pass_and_fail` trace requirement reaches plan 411/375 and
type 241/229. Case count, pass/fail 224/187, active
parse/declaration/type/proof 101/5/190/1, warnings/errors 23/0, existing
expectation outcomes/details, public diagnostics, and `.miz` bytes remain
unchanged.
The exact Task-252 `source_drift` and `test_gap` are closed; the corrected
`design_drift` remains resolved. No blocking `spec_gap`,
`source_undocumented_behavior`, `test_expectation_drift`,
`boundary_violation`, or `repo_metadata_conflict` was found. Tasks 253+ and
260/264/269 remain with their explicit owners; Steps 6/7 remain deferred.

## Step 5 Checker Task 253 Frozen-Contract Addendum

The approved docs-only prerequisite freezes Checker Task 253 as one future
nonempty implementation task. Its public syntax-free `source_application`
transaction has five immutable tables for applications, cross-family
transparent wrappers/origins, individually authenticated ordinary candidate
references, ordered arguments, and unresolved candidate-signature/
application-result requests. Primary actuals reuse Task-252 IDs; nested
actuals may reference Task-253 application IDs. No primary row is duplicated.

The implementation selector is exactly the existing imported `(1 ++ 2)` case
plus one new same-definition-block later-definiens application whose actual
is the Task-248-authenticated inner `DefinitionParameter`, not its outer
reserve. The future aggregate Task-253 oracle is 2/1/2/3/4 and the Task-252
terms/references/numeric-requests slice is 3/1/2. Inline applications receive
synthetic source-shape coverage only; Task 270 retains callee/formal/capture/
substitution authentication. Template subtrees are excluded, with direct
template transport in Task 277 and ordinary/template candidate/selection
transport in Task 278.

This prerequisite closes the exact Task-253 `design_drift` only. The
implementation `source_drift` and `test_gap` remain open. It changes no
source, fixture, expectation, trace row/status, executable coverage, count,
or hash; the baseline remains plan 411/375, type 241/229, pass/fail 224/187,
active parse/declaration/type/proof 101/5/190/1, and warnings/errors 23/0.
The separate implementation task's projected oracle is plan 412/376, type
242/230, pass/fail 224/188, active type 191, subject to fresh measurement.
No blocking `spec_gap`, `source_undocumented_behavior`,
`test_expectation_drift`, current `boundary_violation`, or
`repo_metadata_conflict` was found. Tasks 254+ and Steps 6/7 remain deferred.

## Step 5 Checker Task 253 Implementation Addendum

The separate implementation task adds the public five-table
`source_application` producer, exact Task-252 fingerprint dependency,
one-shot/final AST ownership, exactly two private real consumers, one local
fail fixture, and one bounded covered trace row. Measured aggregates are
Task-253 2/1/2/3/4 and Task-252 3/1/2; the local actual is
`BindingId(1)` / `BindingContextId(1)` / use ordinal 2.

The increment reaches plan 412/376, type 242/230, pass/fail 224/188, active
parse/declaration/type/proof 101/5/191/1, warnings/errors 23/0, and 303
`mizar-test` library tests. The paired mizar-test audit records the exact five
CLI hashes, raw/normalized test-list hashes, and 24-path/25,607-line
production path/content hashes. Task 253 closes its bounded `source_drift`
and `test_gap`. Tasks 254+, 260, 270, 277-278, Steps 6/7, and global Step-5
completion remain open.

## Step 5 Checker Task 254 Frozen-Contract Addendum

Checker Task 254 now has a reviewed documentation-only frozen contract for
structure constructor, selector-access, and functional-update source
transport. It fixes seven syntax-free tables; constructor-root provenance;
written member chains and non-term `FieldUpdate` ownership; one-way
Task-252/253/254 child composition; reverse Task-253 whole-subtree exclusion;
Task-263 semantic deferrals; and the exact future real oracle
5/0/3/9/2/10/26 plus Task-252 8/0/8.

This prerequisite closes only Task-254 `design_drift`. The implementation
`source_drift` and `test_gap` remain open. It changes no source, fixture,
expectation, sidecar, trace row/status, executable coverage, count, or hash;
the baseline remains plan 412/376, type 242/230, pass/fail 224/188, active
parse/declaration/type/proof 101/5/191/1, warnings/errors 23/0, 303 tests,
and 24 production paths / 25,607 lines. The separate implementation's
projected oracle is plan 413/377, type 243/231, pass/fail 224/189, active
type 192, subject to fresh measurement.

No blocking `spec_gap`, `source_undocumented_behavior`,
`test_expectation_drift`, or current `boundary_violation` was found. The
measured 0/0 origin difference from the supplied one-commit-ahead note is a
report-only `repo_metadata_conflict`; it does not prevent the unique safe
commit. Tasks 255+/263-264, Steps 6/7, and global Step-5 completion remain
deferred.

## Step 5 Checker Task 255 Frozen-Contract Addendum

Checker Task 255 now has a reviewed documentation-only target contract for
set enumeration, condition-free comprehension, choice, and `qua` source
transport. It fixes six syntax-free tables; exact real
terms/wrappers/generators/type-sites/edges/requests 4/0/1/3/4/7 plus
Task-252 4/0/4; one-way Task-252/253/254/255 child composition; bare target
site ownership; binder/formula exclusions; semantic deferrals; and one
future bounded fixture/trace increment.

This prerequisite closes only Task-255 `design_drift`. The implementation
`source_drift` and `test_gap` remain open. It changes no production source,
fixture, expectation, sidecar, trace row/status/count, executable coverage,
count, or hash. The baseline remains 413/377, 243/231, 224/189,
101/5/192/1, 312 tests, and 25 paths / 27,317 lines. The separate
implementation projects 414/378, 244/232, 224/190, and active type 193.

The frozen schema includes exact row fields/cardinalities, wrapper
spelling/nesting, request-to-type-site associations, nearest-owner
maximal-range child partition, optional Task-253/254 overlap/fingerprint
rules, and later-installer revalidation. The future sidecar maps to four
existing rows including Chapter 10 plus the new bounded row; Task-255 request
intents do not silently extend Task 251's evidence origin.

Task 257 retains comprehension binder identity/capture, Tasks 256-257 retain
condition formula edges, and semantic sethood/nonemptiness/widening remains
with the Chapter-7/8/17/21 owners. No blocking `spec_gap`,
`source_undocumented_behavior`, `test_expectation_drift`, or current
`boundary_violation` was found; the initial origin difference remains a
report-only `repo_metadata_conflict`. Steps 6/7 and global Step-5 completion
remain deferred.

## Step 5 Checker Task 255 Implementation Addendum

Checker Task 255 is implemented as the reviewed public six-table
`source_set_term` producer plus one private exact `mizar-test` consumer. The
bounded fixture/sidecar and five reciprocal trace references produce the
exact 4/0/1/3/4/7 plus Task-252 4/0/4 oracle, close the Task-255
`source_drift`/`test_gap`, and reach 414/378, 244/232, 224/190, and active
type 193. A review-found nested-comprehension ordering defect was repaired
within the Task-255 extractor boundary.

Task 256 is next in the dependency-ready checker queue. Task 257 retains
comprehension binding/capture, Tasks 256-257 retain condition formula
ownership, and semantic set/choice/`qua`, accepted facts/proofs, Steps 6/7,
and global Step-5 completion remain deferred.

## Step 5 Checker Task 256 Frozen-Contract Addendum

Checker Task 256 now has a documentation-only frozen contract for atomic
formula source transport. It fixes eight syntax-free tables; eight unchanged
real fail consumers; Task-256 `8/0/1/1/1/2/13/11`; Task-252
`16/0/16`; Task-253 `1/1/1/2/2`; Task-255 `2/0/0/0/4/2`;
no real Task-254 target; and exactly eleven unresolved input requests.

The contract freezes exact predicate/attribute resolver provenance,
formula-owned bare asserted types and simple attributes, direct nearest-family
term ownership, canonical spelling, conditional fingerprints, later-installer
revalidation, subtree exclusions, tests, trace impact, and exit criteria.
Existing semantic routes and all eight sidecar outcome/detail fields remain
unchanged.

This prerequisite closes only `design_drift`; bounded `source_drift` and
`test_gap` remain for implementation. It changes no production source,
fixture, `.miz`, sidecar, expectation, trace row/status/count, executable
coverage, count, or hash. Baseline remains 414/378, 244/232, 224/190,
101/5/193/1, 320 tests, and 26 paths / 29,138 lines. The separate
implementation projects 414/379 and 245/233 with unchanged case count.

Task 257 retains predicate chains, formula negation/connectives/quantifiers,
and conditioned-comprehension composition; Task 270 retains inline closure;
Task 277 templates; Task 278 overload selection. General assertion type
graphs, qualified/argument-bearing attributes, semantic facts/truth, theorem
acceptance, Steps 6/7, and global Step-5 completion remain deferred. The
origin discrepancy remains report-only `repo_metadata_conflict`.

## Step 5 Checker Task 256 Implementation Addendum

Checker Task 256 is implemented as the reviewed public eight-table
`source_atomic_formula` producer plus one private exact `mizar-test` consumer.
The eight unchanged fail sidecars and one bounded covered trace row produce
Task-256 `8/0/1/1/1/2/13/11`, Task-252 `16/0/16`, Task-253
`1/1/1/2/2`, and Task-255 `2/0/0/0/4/2`, with no real Task-254
target and with every prior semantic detail owner preserved.

The implementation closes the bounded `source_drift`/`test_gap` and reaches
414/379 requirements, type 245/233, unchanged pass/fail 224/190, and active
type 193. Task 257 is next in the dependency-ready checker queue. Predicate
chains/operators/binders, conditioned comprehensions, inline/templates,
overload selection, accepted facts/proofs, Steps 6/7, and global Step-5
completion remain deferred.

## Step 5 Checker Task 257A Frozen-Contract Addendum

The Task-257 umbrella is decomposed before implementation. Task 257A freezes
the dependency-ready exact implication/universal/negation/contradiction tree,
one explicit unused binder, seven-table `5/0/1/1/1/4/6` transport, and
`2/1/4` binding environment over the one unchanged real
connective/quantifier fail consumer.

This documentation prerequisite closes only `design_drift`. Public producer,
binding prepass, final handoff, and the real/synthetic/context/corruption/
install/exclusion matrix remain bounded `source_drift` and `test_gap`. It
changes no production source, fixture, `.miz`, sidecar, expectation, trace
row/status/count, executable coverage, count, or hash. Baseline remains
414/379, 245/233, 224/190, 101/5/193/1, checker/mizar-test 287/328 tests,
and 27 private production paths / 30,154 lines. The separate implementation
projects 414/380 and 246/234 with unchanged case count and outcome/detail.

Task 257B retains broader connective/quantifier shapes, implicit binders,
bound-use/capture, and executable wrapper occurrences. Task 257C retains
predicate chains and conditioned comprehensions after separately frozen
Task-256/255 extensions. The parent Task 257 remains incomplete. Formula
semantics, theorem ownership/acceptance, accepted facts/proofs, Steps 6/7,
and global Step-5 completion remain deferred; the origin discrepancy remains
report-only `repo_metadata_conflict`.

## Step 5 Checker Task 257B1 Frozen-Contract Addendum

Task 257B is decomposed in dependency order. Task 257B1 now has a
documentation-only EN/JA frozen contract for one explicit universal, one
Task-256 equality body, two Task-252 binder-selected references, the second
exact `1/0/1/1/1/0/2` Task-257 composite profile, and a `1/2` cross-family
formula-composition handoff. Task 257B2 retains broader connectives and
grouping; Task 257B3 retains existential/restricted/nested and implicit
binder shapes.

This prerequisite closes only `design_drift`. The exact 79-byte pass
consumer, source transactions, final ownership, tests, and trace row remain
bounded `test_gap` and `source_drift` for the separate implementation. It
changes no production source, fixture, sidecar, trace metadata, executable
coverage, count, or hash. Baseline remains plan 414/380, type 246/234,
pass/fail 224/190, active 101/5/193/1, checker/mizar-test 299/333 tests,
and 28 private production paths / 30,654 lines. Implementation projects
415/381, 247/235, 225/190, and active type 194.

## Step 5 Checker Task 257B1 Implementation Addendum

Checker Task 257B1 is implemented as the exact 79-byte pass consumer, second
Task-257 composite profile, Task-252/256 same-arena dependencies, public
`1/2` composition handoff, combined final ownership, tests, and one bounded
covered trace row. It closes the task-local `source_drift` and `test_gap`
without granting formula semantics or theorem acceptance.

Measured coverage is plan `415/381`, type `247/235`, pass/fail `225/190`,
active parse/declaration/type/proof `101/5/194/1`, and warnings/errors
`23/0`. Checker/mizar-test library tests are `306/338`; the mizar-test
production manifest is 29 paths / 31,374 lines with path/content hashes
`ee27e3796008fdd180ad8fdfbedfd5b370cb76a0d0f87356487bc82cc5a8f9f6` /
`8b101e3a0a94fcac1dcfd385d311b31d07f6e9f29cbc47b39f42fb51ac71f0ca`.
Task 257B2 is the next dependency-ready logical task.

## Step 5 Checker Task 257B2 Frozen-Contract Addendum

Checker Task 257B2 now has a paired EN/JA documentation contract for one exact
166-byte explicit-universal source. It freezes fixed and repeated conjunction/
disjunction, `iff`, six executable grouping wrappers, the third exact
Task-257 profile `8/6/1/1/1/7/9`, Task-252 `16/0/16`, Task-256
`8/0/0/0/0/0/16/16`, formula composition `8/0`, installation/final
ownership, tests, exclusions, audit impact, and exit criteria.

The documentation prerequisite changes no production, fixture, sidecar, trace
metadata/status/count, test list, or executable coverage. Baseline remains plan
`415/381`, type `247/235`, pass/fail `225/190`, active
`101/5/194/1`, checker/mizar-test tests `306/338`, and 29 mizar-test
production paths / 31,374 lines. The separate implementation projects
`416/382`, `248/236`, `226/190`, and active type 195.

The contract classifies the missing documentation as `design_drift`, the
unimplemented third profiles/final ownership as bounded `source_drift`, and
the absent exact consumer/matrices as `test_gap`, with no blocking
`spec_gap`. Task 257B3, Task 257C, connective truth, repetition expansion,
theorem ownership/acceptance, facts, proof/IR/VC, Steps 6/7, and global Step-5
completion remain deferred. The origin discrepancy remains report-only
`repo_metadata_conflict`.

## Checker Task 257B2 Implementation Ledger

- [x] Implement and verify the exact connective/grouping source transport,
  synchronized EN/JA design, one pass sidecar/covered trace row, and corpus
  `416/382`.
- [x] Preserve all semantic deferrals and classify the external origin
  difference only as report-only `repo_metadata_conflict`.
- [x] Continue with a separate Task 257B3 frozen-contract prerequisite after
  fresh authority/API inventory.

## Checker Task 257B3 Frozen-Contract Ledger

- [x] Freeze the exact 138-byte source and hash, Task-48 reserve-derived base,
  explicit Task-248 exclusion, nested binding/shadowing, exact lower-family
  and formula profiles, ownership, tests, trace projection, and deferrals in
  synchronized EN/JA documentation.
- [x] Classify the missing contract as `design_drift`, future implementation
  as bounded `source_drift` and `test_gap`, and the external origin difference
  as report-only `repo_metadata_conflict`; no blocking `spec_gap` exists.
- [x] Preserve production, fixtures, sidecars, trace status/count, executable
  coverage, counts, and hashes at the Task-257B2 completion baseline.
- [x] Implement Task 257B3 as the next separate logical task after mandatory
  parser/resolver/API and count/hash preflight.

## Checker Task 257B3 Implementation Ledger

- [x] Implement the exact fourth composite and `3/6` composition profiles,
  Task-48-derived nested binding environment, Task-252/256 dependencies,
  atomic combined installation, and resolved clone ownership.
- [x] Add exactly one pass fixture/sidecar and covered trace row while
  preserving all frozen semantic deferrals and prior A/B1/B2 routes.
- [x] Close independent test, implementation, source/documentation, and final
  quality reviews; verify corpus `417/383`, type `249/237`, active type
  `196`, and checker/mizar-test libraries `319/349`.

## Checker Task 257C1 Frozen-Contract Ledger

- [x] Decompose Task 257C and freeze only the lower Task-256 predicate-chain
  segment extension before predicate-chain composition or the separate
  Task-255 condition-bearing comprehension prerequisite.
- [x] Freeze the exact 107-byte source/hash, parser/resolver provenance,
  Task-252 `3/0/3`, extended Task-256
  `1/0/2/2/2/0/0/3/2`, two segment polarities, and one shared boundary.
- [x] Freeze syntax-free public rows, exact consumer/exclusions, validation,
  debug/final ownership, complete tests, one sidecar/trace projection, and all
  semantic deferrals.
- [x] Classify missing contract as `design_drift`, future public transport as
  bounded `source_drift`, missing consumer/matrices as `test_gap`, and the
  origin difference as report-only `repo_metadata_conflict`; no blocking
  `spec_gap` exists.
- [x] Preserve production, fixtures, sidecars, trace status/count, executable
  coverage, counts, test lists, and hashes at the Task-257B3 baseline.
- [x] Implement Task 257C1 as a separate logical task after mandatory fresh
  parser/resolver/API and count/hash preflight.

## Checker Task 257C1 Implementation Ledger

- [x] Add the syntax-free public segment table/polarity schema and shared
  boundary role while preserving all eleven existing input constructors.
- [x] Add the exact 107-byte private selector and same-arena Task-252/256
  transaction without admitting any near miss or Task-257 composition.
- [x] Add one pass fixture/sidecar and one covered trace row; preserve all
  existing `.miz` sources, expectations, and semantic owners.
- [x] Close exact/corruption/isolation/install/clone tests and measure plan
  `418/384`, type `250/238`, pass/fail `228/190`, active type `197`, and
  libraries `322/353`.
- [x] Keep predicate applicability/selection, implicit conjunction, semantic
  negation, truth/facts, theorem acceptance, proof, downstream IR, and
  conditioned comprehension deferred.
- [x] Continue with the separate Task-255 condition-bearing-comprehension
  frozen-contract prerequisite after this implementation commit.

## Checker Task 255C1 Frozen-Contract Ledger

- [x] Freeze the valid 191-byte zero-locus definition, exact ranges/hash,
  imported `++` provenance, and exact future fail consumer.
- [x] Freeze the seven-table condition API/debug surface, colon and direct
  condition-wrapper arena anchors,
  Task-252 `4/0/4`, Task-253 `1/0/1/2/2`, and Task-255
  `1/0/1/1/1/1/2`.
- [x] Preserve condition operands as immutable Task-252 rows without a
  Task-255 edge, and freeze condition-aware lower-family exclusion for later
  Task-256/257C reuse.
- [x] Freeze the reusable private Task-253 seam, 16 compatibility literals,
  tests, final ownership, trace projection, and every semantic deferral.
- [x] Classify the missing contract as `design_drift`, implementation as
  bounded `source_drift`/`test_gap`, and origin difference as report-only
  `repo_metadata_conflict`; no blocking `spec_gap` remains.
- [x] Preserve production, fixtures, sidecars, trace metadata/status/count,
  executable coverage, counts, and hashes at the Task-257C1 baseline.
- [x] Implement Task 255C1 as a separate logical task after mandatory fresh
  parser/resolver/API and count/hash preflight. The implementation closes the
  bounded `source_drift`/`test_gap`, adds only its one fail
  fixture/sidecar/covered trace row, and measures `419/385`, `251/239`,
  `228/191`, active `101/5/198/1`, and `326/357` library tests.

## Checker Task 257C2 Frozen-Contract Ledger

- [x] Freeze the unchanged 191-byte source/hash, direct Task-255 wrapper to
  Task-256 equality relation, exact lower profiles, and imported/built-in
  provenance boundary.
- [x] Freeze a dedicated immutable one-edge condition-formula composition
  transaction with four dependency fingerprints and no synthetic
  composite-formula handoff.
- [x] Freeze validation, debug, typed/resolved ownership, reusable runner
  seams, exact/corruption/near-miss/isolation tests, and all semantic
  deferrals.
- [x] Freeze reuse of the existing sidecar, one future covered trace row,
  unchanged 419-case/pass-fail/active counts and hashes, projected plan
  `419/386` and type `252/240`, and the required coverage-audit impact.
- [x] Reconcile the stale Task-255C1 umbrella checkbox as `design_drift`;
  classify Task-257C2 implementation as bounded `source_drift`/`test_gap`,
  the Task-256 condition-container rejection as a separate authority-backed
  `source_drift`, and origin divergence as report-only
  `repo_metadata_conflict`.
- [x] Freeze and commit the separate Task-256C1 condition-container
  compatibility documentation prerequisite.
- [x] Implement Task-256C1 in its own verified commit.
- [x] Implement Task 257C2 only after Task-256C1 and fresh
  parser/resolver/API, both-install-order, count, test-list,
  production-manifest, and CLI-hash preflight, in a separate logical task and
  commit.

## Checker Task 256C1 Frozen-Contract Ledger

- [x] Freeze the authority-backed direct condition-container/equality
  compatibility needed before Task 257C2, with exact source/ranges/profiles
  and owner-term/formula context equality, with no broadened formula intent.
- [x] Freeze private-only Task-256 validation, unchanged public schema,
  fingerprints/debug, both install orders, rollback, strict overlap
  rejection, independently valid paired near misses, explicit absent set
  fingerprint/substitution checks, three checker tests, and projected
  libraries `329/357`.
- [x] Record `source_drift`, resolved `design_drift`, `test_gap`, report-only
  `repo_metadata_conflict`, zero trace/coverage impact, and unchanged
  executable baselines.
- [x] Implement, review, verify, and separately commit Task 256C1 after fresh
  preflight.
- [x] Resume Task 257C2 only after Task 256C1 and another fresh inventory.

## Checker Task 257C2 Implementation Ledger

- [x] Complete the dedicated condition/formula transaction and exact runner
  consumer with no fixture or semantic expectation change.
- [x] Add three checker tests, four runner tests, one covered trace row, and
  only the reciprocal existing-sidecar reference/note.
- [x] Measure plan/type `419/386` and `252/240`, libraries `332/361`, active
  `101/5/198/1`, and preserve every frozen semantic deferral.

## Checker Task 257C3 Frozen-Contract Ledger

- [x] Select the unchanged 107-byte Task-257C1 pass consumer and preserve
  exact source/hash/ranges, imported provenance, and lower public APIs.
- [x] Freeze Task-252 `3/0/3`, Task-256
  `1/0/2/2/2/0/0/3/2`, and Task-257C3 `1/1` with no duplicated lower row.
- [x] Freeze the two dense conjunction/negation tables, exact public
  handoff/producer/error/accessors/debug, primary/atomic fingerprints, and
  shared-boundary/negative-token validation.
- [x] Freeze complete-route precedence, unsupported-subtree exclusion,
  typed/resolved one-shot ownership, reciprocal A/B/C2/C3 exclusion,
  corruption/rollback/replay, and final clone tests.
- [x] Freeze reuse of the existing pass sidecar, one future reciprocal
  reference/note, one future covered trace row, projected `419/387` and
  `253/241`, and unchanged cases/active counts.
- [x] Record `design_drift`, bounded `source_drift`/`test_gap`, report-only
  `repo_metadata_conflict`, required coverage-audit impact, and all semantic
  deferrals.
- [x] Preserve documentation-only baseline `419/386`, `252/240`,
  `228/191`, active `101/5/198/1`, libraries `332/361`, runner 29 paths /
  34,064 lines, and all hashes.
- [x] Implement Task 257C3 in a separate logical task and commit only after
  this documentation commit and fresh parser/resolver/lower-API/count/hash
  preflight.

## Checker Task 257C3 Implementation Ledger

- [x] Close the bounded `source_drift` and `test_gap` with the exact checker
  transaction, private runner consumer, and mutation-sensitive matrices.
- [x] Preserve the 107-byte fixture, lower row ownership, empty semantic
  result, and all deferred truth/proof/IR behavior.
- [x] Add exactly three checker/four runner tests, one covered trace row, and
  only the ordered reciprocal reference/note in the existing sidecar.
- [x] Measure plan/type `419/387` / `253/241`, libraries `335/365`, and
  runner production 29 paths / 34,290 lines with frozen CLI outcomes.

## Checker Task 258A Frozen-Contract Ledger

- [x] Select only the exact 81-byte future `MT10-FS` reserved-variable
  equality theorem and freeze its hash, ranges, parser/resolver owner/label
  provenance, and exact subtree exclusions.
- [x] Freeze Task-48 binding, Task-252 `2/2/0`, Task-256
  `1/0/0/0/0/0/2/2`, and source-statement `1/1/1/1/1` ownership without
  extending or fabricating Task 248.
- [x] Freeze the five dense table/API/error/debug contract, typed/resolved
  one-shot ownership, handoff-owned BindingEnv/fingerprint, production plus
  named test-only Task-248 exclusion seams, rollback/replay, real
  frontend/resolver tests, and exact empty-semantic boundary.
- [x] Keep the future fixture/sidecar and deferred trace row with `MT10-FS`;
  preserve the existing active reserved-variable equality case unchanged.
- [x] Classify the missing frozen contract as `design_drift`, future producer
  and dormant route as bounded `source_drift`, tests as `test_gap`, and
  origin divergence as report-only `repo_metadata_conflict`; no blocking
  `spec_gap` exists.
- [x] Preserve documentation-only baselines `419/387`, `253/241`,
  `228/191`, active `101/5/198/1`, libraries `335/365`, runner 29 paths /
  34,290 lines, and every test-list/CLI/production hash.
- [x] Implement Task 258A in a separate logical task/commit only after this
  documentation commit and fresh parser/resolver/lower-API/count/hash
  preflight; keep Task 258B open.
- [x] Close the bounded `source_drift` and `test_gap` with checker/runner
  tests `3/4`, checker/runner libraries `338/369`, and runner production
  30 paths / 34,955 lines; leave fixture/sidecar/trace activation to
  `MT10-FS` and freeze Task 258B separately.

## Checker Task 258B1 Frozen-Contract Ledger

- [x] Decompose Task 258B and select only the exact 139-byte nested equality
  proposition, inner/outer conclusions, proof-step label, and backward local
  citation; defer assumptions, witnesses, composite roots, broader
  visibility, and all proof semantics.
- [x] Freeze the authority, hash/ranges/provenance, Task-48 `3/1/0`,
  Task-252 `8/8/0`, Task-256 `4/0/0/0/0/0/0/8/8`, statement
  `1/4/4/4/4`, and reference `1/1` profiles.
- [x] Freeze the extended source-statement kinds/rows, separate public
  label/citation handoff/API/debug, exact binding-context owner validation/
  debug, retained two-pass 77-node/root-76 resolver AST with sole keyed node
  68 plus resolver projection/reference/result replay, combined typed/final
  installation, Task-248/257/258A exclusion, error precedence, subtree
  mutations, checker tests 4, and runner tests 5.
- [x] Record closed `design_drift`, bounded `source_drift`/`test_gap`,
  current 0/0 upstream relation with no unresolved
  `repo_metadata_conflict`, and no blocking `spec_gap`, expectation drift,
  undocumented behavior, or boundary violation.
- [x] Keep `spec.en.checker.formula_statement.source_payloads` deferred with
  `tests = []`; preserve all fixture/sidecar/expectation/trace
  metadata/status/count and executable artifacts.
- [x] Preserve documentation-only baselines `419/387`, `253/241`,
  `228/191`, active `101/5/198/1`, libraries `338/369`, runner 30 paths /
  34,955 lines, and every Task-258A completion hash.
- [x] Implement Task 258B1 in a separate logical task/commit only after this
  documentation commit and fresh parser/resolver/lower-API/count/hash
  preflight; four checker and five runner tests close the bounded
  `source_drift`/`test_gap`.
- [x] Preserve the deferred trace row with `tests = []`, all corpus artifacts,
  and the `419/387`, `253/241`, `228/191`, `101/5/198/1`, `23/0` metadata
  counts while raising the checker/runner libraries to `342/374`.
- [x] Fresh-inventory and decompose Task 258B2+; freeze only Task 258B2's
  exact single-assumption transport contract, keep B3–B5 and Tasks 269–272
  open, and do not infer proof or justification semantics.

## Checker Task 258B2 frozen-contract prerequisite

- [x] Freeze the exact final-LF 113-byte source and SHA-256
  `c9d77d864ab899865bac77c29c57ff5785d553f8b119ef2274e4e9caf031a125`,
  measured 55-node/root-54 parser tree and ranges, and theorem-only resolver
  provenance with no proof-step reference.
- [x] Freeze Task-48 `2/1/0`, Task-252 `6/6/0`, Task-256
  `3/0/0/0/0/0/0/6/6`, and source-statement `1/3/3/3/3`; add only the
  planned `Assumption` source kind and reuse the base-only typed/final path.
- [x] Freeze exact checker/runner consumers, subtree and ownership
  exclusions, four/five future tests, rollback/clone behavior, and empty
  fact/premise/checked-formula/statement-semantic/proof/goal output.
- [x] Classify the closed contract gap as `design_drift`, future producer
  and route as bounded `source_drift`, and future tests as bounded
  `test_gap`; record no blocking `spec_gap` or other protocol disagreement.
- [x] Keep `spec.en.checker.formula_statement.source_payloads` deferred with
  `tests = []` and preserve all corpus, trace, source, test-list, production,
  count, and hash baselines in this documentation-only prerequisite.
- [x] After this dedicated documentation commit and fresh preflight,
  implemented only Task 258B2 with four checker and five runner tests.
  Bounded `source_drift`/`test_gap` are closed without semantic, corpus, or
  trace activation; fresh-inventory Task 258B3 next.

## Checker Task 258B3 frozen-contract prerequisite

- [x] Freeze the exact final-LF 104-byte source, SHA-256
  `76fb48354fc0dfb17047900a047a5b28b806df60d139a3133e606f0ef12a3f82`,
  49-node/root-48 parser identity/ranges, and theorem-only resolver
  provenance.
- [x] Freeze Task-48 `2/1/0`, Task-252 `5/5/0`, Task-256
  `2/0/0/0/0/0/0/4/4`, base `1/2/2/2/2`, one witness row, term-2 atomic
  exclusion, and the combined ordinal partition `[0,1,2]`.
- [x] Freeze the complete public witness API/error/debug/fingerprint
  contract, pair-only typed/final ownership, all cross-family exclusions,
  rollback/replay, and empty semantics.
- [x] Freeze checker/runner compound tests `4/5`, exact mutation/near-miss
  matrices, real frontend/resolver provenance, all-index parity, active
  isolation, and final clone.
- [x] Classify closed `design_drift`, bounded `source_drift`/`test_gap`, and
  no blocking protocol disagreement; retain B3N/M, B4/B5, and Tasks
  269–272.
- [x] Keep the source semantically dormant because its equality goal does
  not authorize `take`; preserve the deferred trace row with `tests = []`
  and award no coverage credit.
- [x] Preserve documentation-only libraries `346/379`, runner 30 paths /
  36,479 lines, module sizes, all metadata counts, test lists, and hashes.
- [x] After this dedicated documentation commit and fresh preflight,
  implemented only Task 258B3. Libraries are `350/384`; every changed
  baseline is remeasured below. Fresh-inventory Tasks 258B3N/M before
  Task 258B4.

## Checker Task 258B3 implementation result

- [x] Implement the syntax-free one-row witness producer and exact B3 base
  profile without changing lower-stage ownership.
- [x] Install/revalidate only the paired base/witness handoffs in typed and
  final ownership; reject standalone, orphan, stale, reference-hybrid, and
  cross-family orders atomically.
- [x] Add exactly four checker and five runner compound tests.
- [x] Preserve every canonical spec, `.miz`, expectation, sidecar, trace
  row/status/count, active route, and semantic output.
- [x] Measure libraries `350/384`, checker module sizes
  `9812/4644/7195/3156`, and runner production 30 paths / 37,172 lines.
- [x] Fresh-inventory and freeze only Task 258B3N: exact 107-byte named
  primary witness, 51-node/root-50 parser identity, witness/name `1/1`
  syntax-free tables, B3 compatibility, no binding/semantics, and unchanged
  baseline counts/hashes.
- [x] Implement Task 258B3N after its dedicated documentation commit and
  fresh parser/resolver/lower/count/hash preflight. Exact syntax-only name
  transport and four/five compound tests pass; libraries are `354/389`.
- [x] Decompose broad Task 258B3M into exact B3M1 reserved-variable mixed
  multiple-witness transport and B3M2 other witness-term shapes.
- [x] Freeze only B3M1: exact 113-byte/56-node source, lower/base
  provenance, witness/name `2/1`, shared/dense ordinals, no API/semantics,
  four/five tests, unchanged baselines, and updated ownership audit.
- [x] Implement only frozen B3M1 after its documentation commit and fresh
  preflight.
- [x] Decompose B3M2 into exact unnamed-numeral B3M2A and remaining
  other-term B3M2B.
- [x] Freeze only B3M2A: final-LF 107-byte/49-node source, lower/base
  provenance, Task-252 numeric request, witness/name `1/0`, no API or
  semantics, four/five tests, unchanged baselines, and ownership-only
  coverage audit.
- [x] Implement only frozen B3M2A after its documentation commit and fresh
  parser/resolver/lower/count/hash preflight.
- [x] Decompose B3M2B into exact parenthesized B3M2B1 and remaining
  authority-valid B3M2B2.
- [x] Freeze B3M2B1 only: final-LF 113-byte/53-node source, five roots /
  Task-252 `6/5/0`, parent/child and reference ownership, atomic exclusion,
  witness/name `1/0`, no API/semantics, four/five future tests, and
  unchanged baselines/audit credit.
- [x] Implement only B3M2B1 after its documentation commit and fresh
  parser/resolver/lower/count/hash preflight.
- [x] Decompose B3M2B2 into exact nested-parenthesized B3M2B2A and
  remaining authority-valid B3M2B2B.
- [x] Freeze only B3M2B2A: final-LF 121-byte/57-node source, five roots /
  Task-252 `7/5/0`, wrapper chain `2 -> 3 -> 4`, Task-256 subtree
  exclusion, witness/name `1/0`, no API/semantics, four/five future tests,
  and unchanged baselines/audit credit.
- [x] Implement B3M2B2A after its documentation commit and fresh preflight.
- [ ] Freeze/implement B3M2B2B before selecting Task 258B4.

Task 258B3M2B2A implementation completion: the private 57-node nested
parentheses selector/profile, paired base plus `1 witness / 0 names`, exact
checker/runner tests `4/5`, and lower/family/replay/clone fail-close are
implemented. Libraries measure `370/409`; no canonical artifact, active
case, public API, fixture, expectation, sidecar, trace status/count, binding,
or semantic owner changed. B3M2B2B remains next before B4.

## Checker Task 258B3M2B2B1P frozen lower prerequisite

- [x] Decompose B3M2B2B into the private Task-253 proof-context reuse seam
  B1P, exact application-witness B1A, and later Task-253/254/255/compound
  slices.
- [x] Freeze the final-LF 143-byte/hash motivating source, zero
  diagnostics, 63-node/root-62 identity, and projected Task-48 `2/1/0`,
  Task-252 `6/4/2`, Task-253 `1/0/1/2/2` in proof context 1.
- [x] Freeze only a private explicit-context reuse entry point, legacy
  context-0 compatibility, and exactly two runner compound tests.
- [x] Preserve canonical artifacts, active routes, public APIs, fixtures,
  expectations, sidecars, trace status/count, semantic owners, libraries
  `370/409`, all counts/hashes, and coverage `deferred`/`tests = []`.
- [x] After the dedicated docs commit and fresh preflight, implement B1P
  alone with exactly two tests; measure libraries `370/411`, runner
  Task-253 sizes `1782/701/2514/2799`, and 30 paths / 39,857 lines.
- [x] Fresh-inventory and freeze B3M2B2B1A's exact application-witness
  contract in a separate EN/JA documentation commit before implementation.

## Checker Task 258B3M2B2B1A Frozen Application-Witness Ledger

- [x] Freeze Chapters 13/15/16 plus existing parser/resolver authority and
  the exact 143-byte/63-node imported-infix witness source.
- [x] Freeze dependency outputs Task-48 `2/1/0`, Task-252 `6/4/2`,
  Task-253 `1/0/1/2/2`, Task-256 equality-only exclusion, Task-258 base
  `1/2/2/2/2`, and witness `1/0`.
- [x] Freeze the additive application target/fingerprint API, legacy byte
  compatibility, atomic typed/final installation, and sole directed
  witness-to-application ownership edge.
- [x] Freeze checker/runner tests `4/5`, unchanged canonical/fixture/
  expectation/sidecar/trace/active/semantic boundaries, libraries `370/411`,
  production 30 paths / 39,857 lines, and no coverage credit.
- [x] Commit this documentation prerequisite alone, fresh-inventory all
  authorities/counts/hashes, then implement only B3M2B2B1A and measure
  libraries `374/416`.

## Checker Task 258B3M2B2B1A Implementation Completion

- [x] Implement only the exact application-witness slice with the
  application-aware checker producer and atomic three-handoff installation.
- [x] Authenticate all source/resolver/lower dependencies and reject every
  byte/subtree/provenance/dependency/precedence/family/replay/clone near miss
  without partial publication.
- [x] Pass exact checker/runner tests `4/5` and measure checker/runner
  libraries `374/416`.
- [x] Preserve canonical specs and `.miz`, fixtures, expectations,
  sidecars, active routes, trace row/status/count, and semantic/proof/goal
  ownership; retain `deferred`, `tests = []`, without coverage credit.
- [x] Fresh-inventory and complete the next authority-valid Task-258 B1B+
  witness shape as B1B1 before selecting the Task-254-backed family.

## Checker Task 258B3M2B2B1B1P Frozen Lower-Prerequisite

- [x] Select exact `take (1 ++ 2);` before Task-254/255 witness families and
  measure final-LF 158 bytes, SHA-256, zero diagnostics, 67 nodes/root 66.
- [x] Freeze proof-context Task-252 `6/4/2`, wrapped Task-253
  `1/1/1/2/2`, wrapper/application containment, and imported `++`
  provenance.
- [x] Freeze only one runner-private wrapper-aware Task-253 reuse seam and
  exactly two future tests; retain the B1B1 statement consumer separately.
- [x] Preserve canonical artifacts, production/tests, active routes,
  fixtures, expectations, sidecars, trace metadata, public APIs, semantic
  ownership, libraries `374/416`, and all count/hash baselines.
- [x] Commit the B1B1P documentation prerequisite, fresh-inventory, and
  implement B1B1P alone before freezing B1B1.

## `PARSER-RECOVERY-B1B1P-P1` Lower-Stage Prerequisite

- [x] Classify the exact nine imported-postfix builder panics as parser
  `source_drift`, the missing Rust matrix as `test_gap`, and stale closeout
  claims as `design_drift`; exclude the five documented `ast = None` cases.
- [x] Freeze canonical Chapter-22 authority, parser/frontend consumers,
  fallback child ownership, contiguous-claimed-prefix theorem recovery, immutable builder
  invariant, Task-41/Task-28 passthrough/merge/fuzz/cache no-op assessments,
  forbidden scope, unchanged coverage/count/hash baseline, and return to
  Checker B1B1P.
- [x] Commit the paired EN/JA documentation prerequisite alone.
- [x] Fresh-inventory, implement the bounded parser correction and Rust-only
  regressions in a second commit, pass all reviews/gates at 90/100 or higher,
  then resume the isolated Checker B1B1P implementation.

## Checker Task 258B3M2B2B1B1P Implementation Completion

- [x] Implement only the private exact wrapped Task-253 reuse seam and
  authenticate every frozen imported-`++` provenance field.
- [x] Preserve Task-252 `6/4/2`, Task-253 `1/1/1/2/2`, legacy unwrapped
  outputs, dormant public/active routes, and all deferred semantic ownership.
- [x] Pass exactly two compound tests covering every source byte and AST
  field, five same-source substitutions, the exact eight-entry reparsed
  near-miss matrix, atomic rollback/replay, and empty downstream tables.
- [x] Measure checker/runner libraries `374/418`, sizes
  `2652/708/2523/3727`, production 30 paths / 41,173 lines, and the recorded
  production/test-list hashes.
- [x] Add no canonical, `.miz`, fixture, expectation, sidecar, trace
  status/count, public/active route, statement consumer, or coverage credit.
- [x] After the dedicated implementation commit and fresh inventory, freeze
  the B1B1 statement-consumer contract as a separate documentation task.

## Checker Task 258B3M2B2B1B1 Frozen Contract

- [x] Fresh-inventory and freeze the exact 158-byte/67-node parenthesized
  application witness before Task-254/255 and all other application shapes.
- [x] Freeze the complete local theorem and imported `++` provenance, lower
  Task-48/252/253/256 rows, base `1/2/2/2/2`, one unnamed
  `Application(0)` witness/no names, and wrapper containment.
- [x] Freeze reuse of the existing B1A public schema/atomic installer and
  B1B1P private wrapped seam through one explicit private B1B1 profile.
- [x] Freeze four checker and five runner tests for all bytes/nodes,
  resolver substitutions, precedence, B1A/family/active isolation,
  rollback/replay/clone, and semantic deferral.
- [x] Preserve canonical/test/fixture/expectation/sidecar/trace/active/public
  artifacts, executable coverage, libraries `374/418`, and all counts/hashes.
- [x] Commit the documentation prerequisite alone; after fresh inventory,
  implement only B1B1 and measure projected libraries `378/423`.

## Checker Task 258B3M2B2B1B1 Implementation Completion

- [x] Implement one exact private B1B1 profile without broadening B1A or
  changing public/active routes.
- [x] Pass the frozen four checker and five runner tests; libraries are
  `378/423`.
- [x] Close `source_drift`, `test_gap`, and completion `design_drift`;
  test-sufficiency and implementation reviews report no findings.
- [x] Preserve canonical specs, `.miz`, fixtures, expectations, sidecars,
  trace `deferred` / `tests = []`, CLI baselines, and semantic/proof/goal/type
  substitution deferrals.
- [x] Pass every final read-only quality hard gate with a valid `98/100`
  score; make the dedicated implementation commit after cached-diff audit.

## Checker Task 258B3M2B2B2P Frozen Lower-Prerequisite

- [x] Decompose the remaining B3M2B2B dependency-first into the private
  Task-254 proof-context constructor seam B2P, exact constructor witness B2A,
  selector witness B2B, and functional-update witness B2C.
- [x] Freeze the final-LF 172-byte source, SHA-256
  `24e2ee2332ead5c0d46025df6044450eeab3ebb5733ebe83587ceae3ba129eb6`,
  zero diagnostics, all 76 unrecovered nodes/root 75, and exact imported
  `parser.type_fixtures::TypeCaseStruct#5` provenance.
- [x] Freeze Task-48 `2/1/0`, Task-252 `6/4/2`, and Task-254
  `1/0/1/2/0/2/6` in proof context 1. Task 254 owns only constructor node
  59 and member nodes 20/24; qualified root 52 remains unowned resolver
  traversal. Task 252 uses 54/57 only as private extraction roots, publishes
  numeral rows at 53/56, and therefore owns only 53/56 as
  `source.term.numeral` while 54/57 remain arena-unowned.
- [x] Freeze the exact request order, `None` application fingerprint,
  no duplicated lower rows, and a runner-private existing-context/shared-
  Task-252 reuse seam that leaves the current Task-254 route unchanged.
- [x] Freeze exactly two future runner compound tests for the complete
  source/arena/provenance/row corruption matrix, failure precedence,
  rollback/replay, legacy compatibility, and empty upper families.
- [x] Preserve canonical specs, existing `.miz`, fixtures, expectations,
  sidecars, trace row/status/count, active/public routes, semantic owners,
  libraries `378/423`, and all CLI/test-list/production counts and hashes.
- [x] Keep the formula-statement row `deferred`, `tests = []`, without
  backlink or executable credit; the existing Task-254 diagnostic coverage
  is unchanged.
- [x] After the dedicated documentation commit and fresh preflight,
  implement B2P alone before freezing the B2A statement consumer.

## Checker Task 258B3M2B2B2P Implementation Completion

- [x] Implement the exact production-private owned-kind selector and
  existing-context/shared-Task-252 Task-254 reuse seam in the frozen files.
- [x] Pass both compound tests and the 425-test runner library.
- [x] Close `source_drift`, `test_gap`, and completion `design_drift`.
- [x] Preserve canonical, `.miz`, fixture, expectation, sidecar, trace,
  active/public, checker, and semantic boundaries; add no coverage credit.
- [x] Remeasure runner sizes, production manifest, and test-list hashes;
  fresh-inventory B2A next.
- [x] Complete the final read-only quality review with no findings, every
  hard gate passing, and a valid score of `98/100`.
- [x] After commit and fresh inventory, freeze B2A documentation as a
  separate logical task.

## Checker Task 258B3M2B2B2A Frozen Structure-Witness Prerequisite

- [x] Freeze the exact final-LF 172-byte/hash, 76-node/root identity,
  local theorem owner, imported `TypeCaseStruct#5`, and ownership map.
- [x] Freeze Task-48/252/254/256 and Task-258 base/witness syntax-free
  tables, with the sole `Witness(0) -> Structure(0)` cross-family edge.
- [x] Freeze the additive public target/fingerprint/builder/atomic installer
  contract and byte-identical legacy/application compatibility.
- [x] Freeze exactly four checker/five runner tests, validation precedence,
  corruption/rollback/replay/clone coverage, and all semantic deferrals.
- [x] Preserve canonical specs, `.miz`, fixtures, expectations, sidecars,
  trace `deferred` / `tests = []`, active routes, and all counts/hashes.
- [x] Complete no-findings specification review and every documentation hard
  gate with a valid final quality score of `98/100`.
- [x] Make one dedicated docs commit, then fresh-inventory and implement only
  B2A.

## Checker Task 258B3M2B2B2A Implementation Completion

- [x] Implement the exact checker target/fingerprint/builder/atomic
  typed/final APIs and private runner consumer.
- [x] Pass the exact four checker/five runner tests and close bounded B2A
  `source_drift` / `test_gap`.
- [x] Preserve B2B/B2C and semantic/proof/goal deferrals, active routes,
  fixtures, expectations, sidecars, and trace `deferred` / `tests = []`.
- [x] Record tests `382/430`, current module/manifests, and test-list hashes.
- [x] Complete prerequisite spec review and no-findings test-sufficiency,
  implementation, and source/documentation consistency reviews.
- [x] Pass focused checker/runner `4/4` and `5/5`, full format, all-target/
  all-feature Clippy with warnings denied, and `cargo test -q` with libraries
  `382/430` and lint policies `15/14`.
- [x] Pass five CLIs at exit zero with 23 warnings / zero errors and
  unchanged counts/hashes; keep manifests/test lists/forbidden artifacts
  unchanged and `stash@{0}` untouched.
- [x] Complete the final read-only quality review with all nine hard gates
  passing and a valid score of `98/100`.
- [x] Commit this logical task as `7613d50d`, verify post-commit
  metadata/stash invariants, and fresh-inventory the next dependency.

## Checker Task 258B3M2B2B2BP Frozen Private Selector Prerequisite

- [x] Freeze B2BP before B2B as the runner-private Task-254 selector
  proof-context reuse prerequisite for the 171-byte/79-node exact source.
- [x] Freeze Task-48 `2/1/0`, Task-252 `6/4/2`, Task-254
  `2/0/1/3/0/3/9`, exact resolver provenance, ownership, edge/request
  order, malformed selector, and subtree exclusions.
- [x] Freeze only private selector site/owned-kind/existing-context siblings;
  add no checker/public API, Task-256/258, active route, diagnostic, or
  semantic behavior.
- [x] Freeze exactly two runner tests and zero checker tests, including all
  bytes/nodes, corruption/precedence/replay, excluded valid forms,
  constructor compatibility, and empty upper tables.
- [x] Preserve trace `deferred` / `tests = []`, Task-254 credit, canonical
  specs/tests/fixtures/expectations/sidecars, and exact baselines/hashes.
- [x] Complete no-findings specification/source-documentation reviews and
  all verification.
- [x] Record external B2BP docs commit `6f84d4eb` as report-only
  `repo_metadata_conflict`; do not repair repository metadata.
- [x] Freeze docs-only Task `258B3M2B2B2BPC1`, correct the provenance
  boundary, and repeat test/implementation/source-doc reviews to no findings.
- [x] Pass BPC1 final quality with no findings, all nine hard gates, and a
  valid score of `98/100`.
- [x] Make one dedicated correction commit, then fresh-inventory and
  implement only the B2BP seam and two tests in a separate commit.
- [x] After B2BP implementation, fresh-inventory and return to B2B
  frozen-contract documentation; keep B2C/semantics deferred.

## Checker Task 258B3M2B2B2BP Implementation Completion

- [x] Implement the exact private selector reuse seam in the frozen four
  runner files and add exactly the two frozen tests.
- [x] Close bounded `source_drift` / `test_gap`; preserve all public,
  active, canonical, trace, diagnostic, and semantic boundaries.
- [x] Pass focused/full tests, lint policies, format, warnings-denied
  workspace Clippy, five CLIs, and all count/hash gates.
- [x] Complete source/documentation consistency with no findings and pass
  final quality review with all nine hard gates and a valid `98/100`.
- [x] Commit once, then fresh-inventory the B2B frozen-contract
  documentation prerequisite.

## Checker Task 258B3M2B2B2B Frozen-Contract Completion

- [x] Freeze the exact 171-byte source, parser/malformed profiles, resolver
  provenance, lower rows, node ownership, subtree exclusions, Task-258
  base/witness rows, and target edge.
- [x] Freeze B2A/B2B sibling isolation, exact checker/runner consumers and
  file scopes, validation order, rollback/replay, and semantic deferrals.
- [x] Freeze four checker tests and five runner tests while preserving
  existing specs, `.miz`, fixtures, expectations, sidecars, trace metadata,
  active routes, and diagnostic credit.
- [x] Synchronize canonical English and Japanese companion plans, ledgers,
  module/design audits, and this narrative-only coverage audit.
- [x] Record baseline `382/432`, projection `386/437`, current production/
  test-list/CLI counts and hashes, the report-only metadata conflict, and
  the associated formula-node `design_drift`.
- [x] Complete all four reviews with no findings, pass fresh verification,
  and pass all nine final quality hard gates with a valid `98/100`.
- [x] Commit this documentation prerequisite as logical commit `4d2fb2b6`.
- [x] Immediately fresh-inventory Task 258B3M2B2B2B implementation after
  that commit; keep B2C and all semantics deferred.

## Checker Task 258B3M2B2B2B Implementation Completion

- [x] Reconfirm the frozen authority/dependency boundary, then implement only
  the exact eight-file B2B consumer while reusing the completed B2BP seam and
  existing checker APIs.
- [x] Add exactly four checker and five runner tests; complete the independent
  test-sufficiency review with no findings.
- [x] Complete the independent implementation review with no findings while
  preserving B2A/B2B isolation, Task-254 ownership, public/active boundaries,
  and all B2C/semantic deferrals.
- [x] Complete source/documentation consistency review and synchronize the
  paired completion records, metrics, hashes, ledgers, and narrative-only
  coverage audit.
- [x] Pass final read-only quality review and the protocol minimum valid score
  of 90/100; every verification/count/hash gate already passes.
- [x] Commit the B2B implementation as logical commit `8311502c`, verify
  clean worktree/ahead-three origin metadata/untouched stash, and
  fresh-inventory the B2CP prerequisite before B2C.

## Checker Task 258B3M2B2B2CP Frozen Lower-Prerequisite

- [x] Classify direct B2C selection as `design_drift` because an exact
  private Task-254 update proof-context reuse seam is still absent.
- [x] Freeze the final-LF 181-byte/hash, 86-node/root-85 parser source,
  180-byte missing-value recovery, and imported constructor provenance.
- [x] Freeze Task-48 `2/1/0`, Task-252 `7/4/3`, Task-254
  `2/0/1/3/1/4/9`, exact ownership, edges, requests, and subtree exclusion.
- [x] Freeze four runner files, two tests, zero checker tests, no public or
  active route, and all semantic/proof/goal deferrals.
- [x] Preserve `386/437`, all count/hash gates, canonical/corpus artifacts,
  Task-254 credit, and formula trace `deferred` / `tests = []`.
- [x] Complete independent specification/dependency review after recording
  the nonblocking roadmap `design_drift`; §13.3.3 and the complete postfix
  grammar leave no blocking or nonblocking `spec_gap`, and canonical
  specification remains unchanged.
- [x] Report the overlapping earlier-session EN/root draft as a nonblocking
  `repo_metadata_conflict`; keep the exact safe target and do not repair
  repository metadata or touch the stash.
- [x] Complete specification/dependency, test-sufficiency,
  implementation-boundary, and source/documentation reviews with no
  findings; pass documentation verification and all nine hard gates.
- [x] Record concurrent docs commit `817bb92b` as report-only
  `repo_metadata_conflict`; its restored `spec_gap` wording invalidated hard
  gates 1/9 and the recorded `98/100`.
- [x] Complete CPC1 repeated no-findings reviews, pass all nine hard gates,
  and obtain valid final quality `98/100`; explicitly justify live broad
  reruns blocked by the unrelated incomplete source diff.
- [x] Commit docs-only correction Task `258B3M2B2B2CPC1` separately as
  `ee267d9c`.
- [x] Fresh-inventory and implement only the private dormant B2CP seam;
  pass exactly the two frozen tests and close `design_drift`,
  `source_drift`, and `test_gap`.
- [x] Complete final test-sufficiency and implementation re-reviews with no
  findings.
- [x] Pass focused/workspace formatting, Clippy, tests, and all count/hash
  gates.
- [x] Synchronize final metrics and narrative-only audit impact while
  preserving all authority, trace-credit, public/active, and semantic
  boundaries.
- [x] Complete final source/documentation review with no findings.
- [x] Pass independent final quality with no findings, all nine hard gates,
  and a valid `98/100`.
- [x] Pass the staged-diff audit and create the dedicated B2CP
  implementation commit `b146f0f7`.
- [x] Fresh-inventory B2C after the B2CP commit.

## Checker Task 258B3M2B2B2C Frozen-Contract Prerequisite

- [x] Select B2C only after B2CP commit `b146f0f7` and correct the stale
  B2CP-pending completion status without repairing repository metadata.
- [x] Freeze the exact 181-byte/hash, zero-diagnostic 86-node/root-85
  parser profile, 180-byte missing-value recovery, local theorem/label
  provenance, and imported `TypeCaseStruct#5` provenance.
- [x] Freeze Task-48 `2/1/0`, Task-252 `7/4/3`, Task-254
  `2/0/1/3/1/4/9`, equality-only Task-256
  `2/0/0/0/0/0/0/4/4`, Task-258 base `1/2/2/2/2`, and witness/name
  `1/0`.
- [x] Freeze exact arena ownership, subtree exclusions, the
  `SourceStatementWitness(0) -> Structure(0)` edge, and all
  Task-252/254/256 cross-family edges without adding semantic edges.
- [x] Freeze reuse of the existing checker structure-witness APIs and
  private B2CP handoff, the exact eight implementation files, four checker
  tests, five runner tests, validation precedence, rollback/replay/final
  clone, family isolation, and empty semantic outputs.
- [x] Preserve canonical specs, `.miz`, fixtures, expectations, sidecars,
  trace `deferred` / `tests = []`, Task-254 credit, public/active routes,
  and all executable counts/hashes in this docs-only task.
- [x] Complete specification/dependency, test-sufficiency,
  implementation-boundary, and source/documentation reviews with no
  findings; pass all documentation verification and count/hash gates.
- [x] Pass all nine final-quality hard gates with a valid score of
  `98/100`.
- [x] Commit this B2C frozen-contract documentation prerequisite alone as
  `d6076cc757ce675d1b46a720b4f00805923d3c70`, and verify clean
  metadata/stash invariants.
- [x] Fresh-inventory and implement only the frozen B2C contract.

## Checker Task 258B3M2B2B2C Implementation

- [x] Implement exactly the frozen eight-file transaction after clean
  post-`d6076cc7` inventory.
- [x] Consume the unchanged B2CP functional-update seam and existing checker
  structure-witness APIs without adding public or active corpus surfaces.
- [x] Preserve exact Tasks 48/252/254/256/258 ownership and publish only the
  witness-to-update `Structure(0)` edge.
- [x] Add and pass exactly four checker and five runner tests.
- [x] Pass checker library `390`, runner library `444` plus policy suites,
  focused checker `4/4`, and focused runner `5/5`.
- [x] Complete final test-sufficiency and implementation reviews with no
  findings.
- [x] Synchronize the paired checker/test plans, ledgers, module/design
  audits, and narrative-only coverage audit.
- [x] Keep spec, `.miz`, fixtures, expectations, sidecars, trace
  status/tests, coverage credit, active corpus, public API, and semantics
  unchanged.
- [x] Pass broad workspace format, Clippy, and test gates, including focused
  `4/4` and `5/5` and sibling `12/12` and `21/21` suites.
- [x] Complete final source/documentation consistency re-review with
  **NO FINDINGS**.
- [x] Complete independent final read-only quality review with
  **NO FINDINGS**, all nine hard gates PASS, and a valid `98/100`.
- [x] Audit the cached implementation diff and create dedicated B2C
  implementation commit `e8373c683448e524cb98edde83fdf8de83a125cd`.
- [x] Verify clean ahead-eight/behind-zero post-commit state, unchanged stash,
  and fresh-inventory B3P as the next dependency-authorized task.

## Checker Task 258B3M2B2B3P Frozen-Contract Prerequisite

- [x] Freeze the exact 117-byte/hash, zero-diagnostic 57-node/root-56 source,
  significant syntax map, and local-only resolver provenance.
- [x] Freeze lower Task-48 `2/1/0`, Task-252 `6/4/2`, Task-255
  `1/0/0/0/0/2/1`, empty Tasks 253/254/256/258, ownership, and subtree
  exclusions.
- [x] Freeze only the private explicit-context runner seam in four files and
  exactly two compound tests, preserving context-0 bytes.
- [x] Freeze exhaustive two-test sufficiency over all bytes/LF variants,
  nodes, resolver/lower fields, ownership, precedence/replay/rollback/clones,
  empty adjacent/semantic outputs, and literal Task-111 legacy hashes.
- [x] Keep upper B3A checker/public witness ownership and all semantic,
  adjacent-family, active, canonical, and trace changes forbidden.
- [x] Record baseline `390/444`, projection `390/446`, exact source/test/CLI
  counts and hashes, classifications, and narrative-only audit impact.
- [x] Complete specification review with **NO FINDINGS**.
- [x] Complete documentation review/repeat with **NO FINDINGS**.
- [x] Complete test-sufficiency review with **NO FINDINGS**.
- [x] Complete implementation-boundary review with **NO FINDINGS**.
- [x] Complete source/documentation consistency review with
  **NO FINDINGS**.
- [x] Pass source/hash, lint, library, production/test-list, five CLI hash,
  exact-26-doc, diff-check, and trace-no-op verification.
- [x] Complete final quality with **NO FINDINGS**, all nine hard gates PASS,
  and valid `98/100` (`20/20/15/14/10/10/5/4`).
- [x] Audit/stage and commit this documentation prerequisite alone as
  `285a1f11c310bb313c4c6b4feae914eb11f74754`.
- [x] Verify clean post-commit invariants and unchanged stash, then fresh-
  inventory B3P implementation.

## Checker Task 258B3M2B2B3P Implementation Closure

- [x] Preserve prerequisite commit
  `285a1f11c310bb313c4c6b4feae914eb11f74754`.
- [x] Implement exactly four runner files and exactly two B3P tests.
- [x] Add the private explicit-context sibling/context-0 delegate and retain
  literal legacy hashes without checker/public/active changes.
- [x] Cover all bytes/LF, 57 nodes, resolver `63`, binding `39`,
  Task-252/255, fingerprint-only absence, stale precedence, immediate
  replay, clones, and isolation.
- [x] Close B3P `source_drift`/`test_gap`; keep trace credit/status unchanged
  and transfer next ownership to upper B3A.
- [x] Complete test-sufficiency and implementation reviews with
  **NO FINDINGS**.
- [x] Pass focused `2/2`, runner library `446/446`, formatting, package
  Clippy, and diff check; record current counts/hashes.
- [x] Complete repeated source/documentation consistency and
  documentation/boundary reviews with **NO FINDINGS**.
- [x] Pass lint-policy `15/14`, metadata `137`, focused `2/2`, runner
  library `446/446`, formatting, workspace-wide warnings-denied Clippy and
  tests, five CLI/count/hash, current manifest/test-list hash, exact-30-file
  scope, and diff-check gates.
- [x] Complete final read-only quality review with **NO FINDINGS**, all nine
  hard gates PASS, and valid `98/100`
  (`20/20/15/14/10/10/5/4`).
- [x] Audit/stage and create dedicated implementation commit
  `abbfedfc2cdbaa97d8294893859da8cd350ad9a8`.
- [x] Verify clean post-commit HEAD, ahead-10 origin metadata, untouched
  stash `f65cf4a13752ec380710814a9ac6392ccb9d75d4`, and fresh-inventory
  upper B3A.

## Checker Task 258B3M2B2B3A Frozen-Contract Completion

- [x] Close B3P as commit
  `abbfedfc2cdbaa97d8294893859da8cd350ad9a8` and fresh-inventory B3A
  under clean/ahead-10/untouched-stash invariants.
- [x] Freeze Chapters 4/13/15/16 authority and exact source/resolver/lower
  tables for one non-semantic witness-to-`SetTerm(0)` transport edge.
- [x] Freeze one witness/zero names, ownership partition/graph, additive
  checker API, exact seven implementation files, and source-set-term
  producer exclusion.
- [x] Freeze four checker/five runner tests, exhaustive matrices,
  validation precedence, semantic deferrals, and no active/corpus/trace
  credit.
- [x] Record `design_drift`, `source_drift`, `test_gap`, no blocking
  disagreement, fresh/projected counts and hashes, exact `32` docs, and
  deliberate trace no-op.
- [x] Complete specification/documentation, test-sufficiency, and
  implementation/API boundary reviews with **NO FINDINGS**.
- [x] Pass documentation, source/count/hash, lint/library, five CLI,
  exact-scope, diff-check, and trace-no-op verification.
- [x] Complete source/docs consistency and boundary reviews with
  **NO FINDINGS**.
- [x] Complete final quality with **NO FINDINGS**, all nine hard gates PASS,
  and valid `98/100` (`20/20/15/14/10/10/5/4`).
- [x] Create dedicated B3A documentation-only commit
  `f4ff45964d97b31b6c328381120ba8ede080a2b1`.
- [x] Verify clean ahead-11/behind-0 post-commit state, unchanged stash
  `f65cf4a13752ec380710814a9ac6392ccb9d75d4`, and fresh B3A
  implementation inventory.

## Checker Task 258B3M2B2B3A Implementation Completion

- [x] Close prerequisite commit/post-commit/fresh-inventory gates.
- [x] Implement the exact seven-file checker/runner source transport and
  preserve spec, corpus, fixture, expectation, sidecar, trace, and set-term
  producer authority.
- [x] Add the exact additive set-witness API and frozen four checker plus
  five runner tests without semantic or trace credit.
- [x] Complete specification, test-sufficiency, and implementation reviews
  with **NO FINDINGS**.
- [x] Pass focused/package tests, formatting, targeted Clippy, five CLI,
  final count/hash manifests, and diff checks.
- [x] Complete the second source/documentation consistency repeat with
  **NO FINDINGS**.
- [x] Complete the final documentation/boundary reread with
  **NO FINDINGS**.
- [x] Pass parent final verification: focused checker `4` plus runner `5`;
  checker package `394` plus lint-policy `15`; mizar-test package `451`,
  layout `3`, lint-policy `14`, metadata `137`, public-enum `2`, snapshot
  `21`; format; workspace Clippy/tests; five CLI counts/hashes; production
  manifests/test lists; diff check; and exact `39`-file scope.
- [x] Complete independent final read-only quality review with
  **NO FINDINGS**: all nine hard gates PASS with no score cap, valid
  `98/100` (`20/20/15/14/10/10/5/4`), and the stated residual deferrals
  unchanged.
- [x] Create B3A implementation commit
  `a147bad88f1963c504f796051ba0b855eca71d07`.
- [x] Verify clean ahead-12/behind-0 post-commit state and unchanged stash.
- [x] Fresh-inventory and select B3B empty-enumeration documentation.

## Checker Task 258B3M2B2B3B

- [x] Freeze the dependency-minimal `take {};` upper statement profile,
  exact 118-byte/hash and 50-node AST, zero-edge Task-255 contract, one
  SetTerm witness, existing API reuse, and semantic/trace deferrals.
- [x] Keep production source, `.miz`, expectations, sidecars, trace
  status/count, and coverage credit unchanged in the documentation
  prerequisite.
- [x] Finish no-findings reviews and verification with all nine hard gates
  PASS, no score cap, and valid quality `98/100`.
- [x] Create the dedicated docs-only commit
  `080e6824d843655986079f5d5fc41abe06b0fbd6`.
- [x] Verify clean ahead-13/behind-0 state and unchanged stash
  `f65cf4a13752ec380710814a9ac6392ccb9d75d4`, then fresh-inventory and
  select only B3B implementation before the next B3 sibling; B4 remains
  blocked behind the unfinished B3 umbrella.

## Checker Task 258B3M2B2B3B Implementation Completion

- [x] Close prerequisite commit/post-commit/fresh-inventory gates.
- [x] Implement the frozen exact seven-file private checker/runner
  transport and the exact four checker plus five runner tests.
- [x] Preserve public APIs, errors, debug grammar, dependencies, active
  routes, specification, corpus, fixtures, expectations, sidecars, trace
  metadata, and semantic/coverage credit.
- [x] Remediate the initial three test-sufficiency findings within the
  existing nine tests.
- [x] Remediate the additional B3B-specific currently mutable
  Task-48/252/255 mutation/replay finding with exact `32/55/23` matrices.
- [x] Complete the implementation repeat with **NO FINDINGS** before the
  bounded test-only follow-up.
- [x] Add post-auth injection plus stage-prefix/non-generic-guard
  assertions and complete all test-sufficiency repeats with
  **NO FINDINGS**.
- [x] Complete final implementation repeat with **NO FINDINGS**.
- [x] Rerun focused tests, format, diff, and final runner count/hash gates.
- [x] Rerun libraries `398/456`, workspace Clippy/tests, five CLIs, scope,
  and no-op gates.
- [x] Complete source/documentation consistency repeat with
  **NO FINDINGS** after the two `design_drift` wording fixes.
- [x] Complete independent final documentation/boundary review with
  **NO FINDINGS**.
- [x] Complete independent final read-only quality review with
  **NO FINDINGS**, all nine hard gates PASS, no score cap, and valid
  `98/100` (`20/20/15/14/10/10/5/4`).
- [x] Stage only the exact `39` synchronized task files and inspect cached
  diff.
- [x] Create implementation commit
  `dbbf5f6a2b0bd58d8434fb4687f7bfad398ca4bc`.
- [x] Verify clean ahead-14/behind-0 origin/stash invariants.
- [x] Fresh-inventory and select B3C choice witness.

## Checker Task 258B3M2B2B3C Documentation Prerequisite

- [x] Freeze exact `110`-byte/hash, 52-node choice witness, resolver/lower
  profiles, ownership graph, and semantic exclusions.
- [x] Freeze exact checker 4 + runner 5 tests and
  `32/55/39/72/62/21` exhaustive matrices.
- [x] Limit future implementation to seven source consumers; keep specs,
  corpus, expectations, trace metadata, active behavior, semantics, and both
  Task-255 source owners unchanged.
- [x] Resolve initial medium ownership `design_drift` and matrix `test_gap`;
  repeated specification review **NO FINDINGS**.
- [x] Complete synchronized documentation consistency/boundary review with
  **NO FINDINGS**.
- [x] Pass docs-only scope/count/hash/no-op, crate/workspace, and five-CLI
  verification.
- [x] Complete independent final quality with **NO FINDINGS**, all nine
  hard gates PASS, no score cap, and valid `98/100`.
- [x] Create dedicated documentation commit
  `ea48ffc4fa586ac6d0813cd23a6b1d9b571087b2`, verify clean/stash
  invariants, and fresh-inventory B3C implementation.

## Checker Task 258B3M2B2B3C Implementation Completion

- [x] Close the documentation prerequisite at clean ahead-15/behind-0 with
  stash fingerprint `f65cf4a...` unchanged.
- [x] Fresh-inventory authority/API and confirm no lower-stage prerequisite.
- [x] Implement only the exact checker three plus runner four source
  consumers; preserve both Task-255 source owners.
- [x] Implement the frozen checker four plus runner five tests and exact
  `32/55/39/72/62/21` matrices.
- [x] Remediate two medium resolver/upper-prefix `test_gap` findings.
- [x] Remediate the B3A-hard-coded B3C `source_drift`/`test_gap` without
  changing either enumeration sibling.
- [x] Complete repeated test-sufficiency and implementation reviews with
  **NO FINDINGS**.
- [x] Pass focused `4/4 + 5/5`, runner package
  `461+3/14/137/2/21`, and formatting.
- [x] Record final module counts, production/test-list hashes, unchanged
  five CLI hashes/counts, and deliberate trace no-op.
- [x] Complete workspace Clippy/tests and final measurement reruns.
- [x] Complete final source/documentation consistency and independent
  quality reviews.
- [x] Stage only exact 39 synchronized task files and create implementation
  commit `7988a50934656ff90b31e06b883225f86196103b`.
- [x] Verify clean ahead-1/behind-0 post-commit/stash invariants and report
  external origin movement as `repo_metadata_conflict` only.
- [x] Fresh-inventory and select B3D qua witness.

## Checker Task 258B3M2B2B3D Documentation Prerequisite

- [x] Freeze exact 109-byte/hash, 54-node qua witness, resolver/lower
  profiles, ownership graph, and semantic exclusions.
- [x] Freeze exact checker 4 + runner 5 tests and
  `32/70/44/72/62/21` exhaustive matrices.
- [x] Limit future implementation to seven source consumers; keep both
  Task-255 source owners, specs, corpus, expectations, trace metadata,
  active behavior, and semantics unchanged.
- [x] Resolve comprehension-versus-`qua` task-decomposition `design_drift`
  in favor of the strictly smaller qua handoff.
- [x] Complete synchronized reviews with **NO FINDINGS**.
- [x] Pass docs-only scope/count/hash/no-op, crate/workspace, and five-CLI
  verification.
- [x] Complete independent final quality with all hard gates and valid
  score `>=90/100`.
- [x] Create dedicated documentation commit
  `43af562c2cb84e72658cee059abbe7543ee73fe7`.
- [x] Verify clean ahead-2/behind-0 post-commit/stash invariants with
  fingerprint `f65cf4a13752ec...` unchanged and fresh-inventory B3D
  implementation.

## Checker Task 258B3M2B2B3D Implementation Completion Inventory

- [x] Implement only the exact checker three plus runner four source
  consumers; preserve both Task-255 source owners and all authority artifacts.
- [x] Add only the frozen checker four plus runner five tests and exact
  `32/70/44/72/62/21` matrices.
- [x] Complete test-sufficiency review with **NO FINDINGS**.
- [x] Pass focused `4/4 + 5/5`, checker `406+15`, runner
  `466+3/14/137/2/21`, formatting, and full Clippy.
- [x] Record exact final module counts, production/test-list hashes,
  unchanged five CLI hashes/counts, and trace/authority/semantic no-ops.
- [x] Complete repeated independent implementation review with
  **NO FINDINGS**.
- [x] Complete repeated source/documentation consistency, bilingual, and
  boundary review with **NO FINDINGS** after one Medium stale-review and two
  Low 24-order/qua-edge documentation corrections.
- [x] Pass checker package `406+15`, runner package
  `466+3/14/137/2/21`, formatting, full Clippy, full workspace tests, five
  CLIs, and final count/hash reruns.
- [x] Complete independent final read-only quality review with
  **NO FINDINGS**, all nine hard gates PASS, no score cap, and valid
  `100/100` (`20/20/15/15/10/10/5/5`).
- [x] Stage only the synchronized task scope, inspect cached diff, and create
  implementation commit
  `08a7d1e3d8c4b3b439325a16e1e139df4a1c18ed`.
- [x] Verify clean ahead-3/behind-0 post-commit state and unchanged stash
  fingerprint `f65cf4a13752ec...`; retain the inherited origin movement as a
  report-only `repo_metadata_conflict`.
- [x] Fresh-inventory and select B3E condition-free comprehension witness.

## Checker Task 258B3M2B2B3E Documentation Prerequisite

- [x] Select the sole remaining Task-255 set-family statement-witness sibling
  before B4: one condition-free independent comprehension.
- [x] Confirm canonical §§13.4/13.4.2, 4.4.3, 15.4.4, and 16.3.3 authority,
  existing parser/Task-255 fixtures, and no lower-stage prerequisite.
- [x] Freeze the final-LF 139-byte source/hash, 28-token,
  60-node/root-59 Surface profile, resolver provenance, and the exact
  statement/witness ownership partition.
- [x] Freeze Task-48 `2/1/0`, Task-252 `5/4/1`, empty Tasks 253/254,
  Task-255 `1/0/1/1/0/1/2`, Task-256
  `2/0/0/0/0/0/0/4/4`, Task-258 `1/2/2/2/2`, and one unnamed
  SetTerm witness.
- [x] Freeze exact `32/70/53/72/62/21` mutation matrices, all 120
  five-family orders, and the checker four plus runner five future tests.
- [x] Limit future implementation to the exact seven private source
  consumers; forbid both Task-255 source owners, parser/resolver/binding
  source, specifications, corpus artifacts, trace metadata, public API, and
  semantic/coverage credit.
- [x] Complete repeated specification and documentation reviews with
  **NO FINDINGS**.
- [x] Pass documentation-only source/count/hash/scope/no-op verification.
- [x] Pass all nine hard gates and independent final quality with valid
  score `>=90/100`.
- [x] Create dedicated documentation commit
  `8075000bf79be3fdea6b22f366fb6d9e59781fe7`.
- [x] Verify clean post-commit/stash invariants and fresh-inventory B3E
  implementation.

## Checker Task 258B3M2B2B3E Implementation Completion Inventory

- [x] Implement exact checker three plus runner four consumers only.
- [x] Add checker four/runner five tests and exact matrices/orders.
- [x] Use successful coherent same-provenance Task-255 post-auth negatives.
- [x] Complete test-sufficiency and implementation reviews with
  **NO FINDINGS**.
- [x] Pass focused `4/4 + 5/5`, libraries `410/471`, five CLIs, and final
  production/test-list count/hash measurements.
- [x] Preserve specifications, corpus artifacts, trace status/count/
  backlinks, active behavior/coverage, public API, and semantics.
- [x] Complete source/docs, bilingual, and boundary consistency re-review
  with **NO FINDINGS** after three `design_drift` corrections.
- [x] Complete independent final quality with **NO FINDINGS**, all nine
  gates PASS, no cap, and valid `100/100`.
- [x] Pass focused/package, formatting, full Clippy, workspace, five-CLI,
  count/hash/scope/forbidden/stash verification.
- [x] Stage the exact B3E scope and inspect cached diff.
- [x] Create B3E implementation commit
  `e4479691db3b0a8785bb16e94d386bd71a394274`.
- [x] Verify clean ahead-5/behind-0 post-commit state and unchanged stash;
  fresh-inventory Task 258B4A.

## Checker Task 258B4A Documentation Prerequisite

- [x] Decompose the B4 composite-root umbrella and select the
  explicit-universal Task-257B1 consumer first.
- [x] Freeze canonical authority, distinct private 80-byte/double-LF
  selector, parser/resolver provenance, and lower/upper syntax-free tables.
- [x] Resolve the active-case `test_expectation_drift` without changing the
  79-byte `.miz`, expectation, sidecar, trace, or active route.
- [x] Freeze the composite target/fingerprints, dedicated producer, paired
  typed installation, final boundary, eight files (three checker/five
  runner), the sole crate-private Task-257B1 helper visibility seam, nine
  tests, deferrals, baseline, audit narrative-only impact, and exit criteria.
- [x] Complete repeated specification/documentation reviews with no
  findings.
- [x] Pass docs-only verification and all no-op/count/hash/stash gates.
- [x] Pass all hard gates and independent quality score `>=90/100`.
- [x] Stage the exact bilingual documentation scope and create prerequisite
  commit `9da1ac13e811c78359d8d64e740832b2a30dae24`.
- [x] Verify clean ahead-6/behind-0 post-commit state, unchanged stash, and
  fresh-inventory B4A
  implementation.

## Checker Task 258B4A Implementation Completion

- [x] Implement the frozen three checker/five runner files only.
- [x] Add four checker/five runner tests and exact lower/upper/coherent/
  family-order/replay/final matrices.
- [x] Preserve lower root ownership, corpus/trace/active/public-runner/
  semantic boundaries, and all later-task deferrals.
- [x] Complete test-sufficiency and implementation reviews with
  **NO FINDINGS**.
- [x] Pass focused `4/4 + 5/5`; measure libraries `414/476` and production
  `23/139828`, `30/55109`.
- [x] Complete source/docs and bilingual consistency with **NO FINDINGS**
  after three Low `design_drift` corrections.
- [x] Pass full verification and final count/hash/CLI/stash gates.
- [x] Pass independent final quality with **NO FINDINGS**, all nine hard
  gates PASS, no cap, and valid `100/100`.
- [x] Stage/inspect the exact B4A implementation scope.
- [x] Create B4A implementation commit
  `662adbde71e665ab37504ac476e94c935c493535`.
- [x] Verify clean ahead-7/behind-0 post-commit state, unchanged stash, and
  fresh-inventory B4B.

## Checker Task 258B4B Documentation Prerequisite

- [x] Freeze canonical authority and the distinct private
  167-byte/double-LF connective/grouping source, all 124 Surface nodes/root
  123, and local resolver owner provenance.
- [x] Freeze the complete Task-252/256/257/B2/binding transaction, exact
  42/1/81 ownership partition, and upper `1/1/1/0/1` `Composite(0)`
  association.
- [x] Reuse the B4A public API and debug grammar; freeze exactly seven
  future consumers and four checker/five runner tests.
- [x] Preserve the active 166-byte lower-only fixture, every corpus/trace
  artifact and semantic/coverage deferral, with narrative-only audit impact.
- [x] Complete repeated reviews, docs-only verification, and final quality
  with all nine hard gates and score `>=90/100`.
- [x] Stage/inspect and create one dedicated B4B documentation commit
  `b8a7b8257a682f7c88de943ceaa35b67c0585bc4`.
- [x] Verify clean ahead-8/behind-0 post-commit state, unchanged stash
  fingerprint, and fresh-inventory B4B
  implementation.

## Checker Task 258B4B Implementation

- [x] Implement exactly three checker and four runner consumers; preserve
  all lower owners including the 1,853-line formula-composition helper.
- [x] Authenticate private 167-byte source; raw label-free then enriched
  `1/1/1/1/0`; Task-257B2 lower/rootless `42/1/81`; upper
  `1/1/1/0/1` with both `Composite(0)` links.
- [x] Enforce exact B1/A versus B2/B pairing, B4B `0/0/[]`, B4A
  `1/1/[1,1]`, and active 166-byte lower-only exclusion.
- [x] Pass focused checker `4/4` and runner `5/5`; complete separate
  test-sufficiency and implementation reviews with **NO FINDINGS**.
- [x] Record libraries `418/481`, production `23/140821` and `30/56007`,
  exact owner sizes, four test-list hashes, production hashes, and unchanged
  CLI counts/hashes.
- [x] Keep `doc/design/spec_coverage_audit.md` unchanged because B4B changes
  no specification coverage status, owner crate, trace row/backlink, active
  test mapping, or deferred semantic credit.
- [x] Preserve public APIs, semantics, specifications, existing `.miz`,
  expectations, sidecars, corpus, and trace status/count.
- [x] Complete final source/documentation, bilingual, and boundary
  consistency reviews with **NO FINDINGS**.
- [x] Pass broad crate/workspace, fmt, Clippy, CLI, count/hash, scope, and
  stash verification.
- [x] Pass independent final quality with all hard gates and score
  `>=90/100`.
- [x] Stage/inspect and create one dedicated B4B implementation commit
  `752c17ae7d552d5268d1028612b8174e480b6f3e`.
- [x] Verify clean behind-0/ahead-1 post-commit state, unchanged stash, and
  fresh-inventory B4C; report external origin movement only as
  `repo_metadata_conflict`.

## Checker Task 258B4C Documentation Prerequisite

- [x] Freeze Chapter 2/4/14/16 authority and distinguish the active
  138-byte lower-only source from the private 139-byte/double-LF source.
- [x] Freeze all 66 Surface nodes/root 65, raw/enriched resolver provenance,
  contribution anchor, and exact source hashes.
- [x] Freeze the Task-48/252/256/257/B3 lower transaction, nested shadowing,
  `24/1/41` ownership, and `UnassignedStatement` root.
- [x] Freeze upper `1/1/1/0/1`, context visible `[0]`, no input fact, both
  `Composite(0)` links, and private telemetry
  `2/2/[2,2,4,4,4,4]`.
- [x] Classify the exact-source guard as bounded lower-stage `source_drift`
  and require a separate two-file prerequisite commit before B4C upper
  implementation.
- [x] Freeze the same seven eventual upper consumers as B4B, four
  checker/five runner tests, exact B1/A-B2/B-B3/C pairing, deferrals,
  baseline, narrative-only audit impact, and exit criteria.
- [x] Complete repeated specification, test-boundary, bilingual, and
  source/documentation reviews with **NO FINDINGS**.
- [x] Pass docs-only/no-op verification and all frozen scope, count/hash,
  authority/trace/production no-op, and stash gates.
- [x] Pass independent final quality with **NO FINDINGS**, all nine hard
  gates PASS, no cap, and valid `100/100`
  (`20/20/15/15/10/10/5/5`).
- [x] Stage/inspect and create one dedicated B4C documentation commit
  `3c723316ae632a867d29e8f4fc36348be30df202`.
- [x] Verify clean post-commit/stash invariants and fresh-inventory the
  separate Task-257B3 private-selector prerequisite.

## Task 257B3 Private Double-LF Selector Prerequisite

- [x] After the B4C docs commit, fresh-inventory only
  `runner/type_elaboration/source_formula.rs` and the runner
  `source_formula_composition` test owner.
- [x] Admit exactly the active 138-byte and private 139-byte identities;
  reject zero/triple LF and source/AST identity spoofing.
- [x] Prove identical Task-257B3 lower tables/fingerprints and unchanged
  active CLI/trace behavior.
- [x] Review, verify, and create one separate lower-stage prerequisite
  commit `42356f38ed0e679d7b878caf0e647c6aa8148d82` before B4C upper
  implementation.

## Checker Task 258B4C Implementation

- [x] Implement exactly three checker and four runner consumers; preserve
  all lower production owners and all authority/test artifacts.
- [x] Authenticate the private 139-byte source, exact raw-to-enriched
  resolver provenance, Task-257B3 lower transaction, rootless `24/1/41`
  arena, upper `1/1/1/0/1`, and both `Composite(0)` links.
- [x] Enforce exact B1/A versus B2/B versus B3/C pairing, the active
  138-byte lower-only exclusion, and private telemetry
  `2/2/[2,2,4,4,4,4]`.
- [x] Pass focused checker `4/4` and runner `5/5`; complete separate
  test-sufficiency and implementation reviews with **NO FINDINGS**.
- [x] Record checker/runner libraries `422/488`, production
  `23/141952` and `30/56872`, and exact production/test-list hashes.
- [x] Keep `doc/design/spec_coverage_audit.md` unchanged because B4C
  changes no specification coverage status, owner crate, trace
  row/backlink, active test mapping, or deferred semantic credit.
- [x] Preserve public schemas, semantics, specifications, existing `.miz`,
  expectations, sidecars, corpus, and trace status/count.
- [x] Complete final source/documentation, bilingual, and boundary
  consistency reviews with **NO FINDINGS** after correcting one Medium
  `design_drift`.
- [x] Pass broad crate/workspace, fmt, Clippy, CLI, count/hash, scope, and
  stash verification; reproduce every frozen count and hash.
- [x] Pass independent final quality with **NO FINDINGS**, all nine hard
  gates PASS, no cap, and valid `100/100`
  (`20/20/15/15/10/10/5/5`).
- [x] Stage/inspect and create dedicated B4C implementation commit
  `50ab1ebc747e912fff1f0cf111832e3c2c81ba01`.
- [x] Verify clean post-commit state, unchanged protected stash, and
  fresh-inventory the next dependency-ready logical task.

## Checker Task 258B5A Frozen-Contract Documentation Prerequisite

- [x] Freeze the exact private ancestor-label/descendant-citation source,
  authority, resolver/lower/statement/reference provenance, 20/73 ownership,
  exact consumers, tests, baselines, and exit criteria.
- [x] Correct stale B4C ledger state and decompose B5A positive local
  visibility from B5B imported-public and B5C active negative confinement.
- [x] Classify the absent seven-consumer B5A implementation as bounded
  `source_drift` owned by the immediate next implementation task.
- [x] Keep production, specifications, existing `.miz`, expectations,
  sidecars, trace status/count/backlinks, and semantic results unchanged.
- [x] Complete independent specification, test-sufficiency,
  source/documentation boundary, and bilingual reviews with **NO FINDINGS**.
- [x] Reproduce crate/workspace, fmt, Clippy, five-CLI, exact
  scope/count/hash, authority-no-op, repository-state, and stash gates.
- [x] Complete repeated independent final quality with **NO FINDINGS**, all
  nine hard gates PASS, no cap, and valid `100/100`
  (`20/20/15/15/10/10/5/5`).
- [x] Create documentation-only commit
  `59021f764f146d669f84877042f0512882c9c5ff`, verify post-commit
  invariants, and continue into fresh Task-258B5A implementation inventory.

## Checker Task 258B5A Implementation

- [x] Implement exactly three checker and four runner consumers; preserve
  parser/resolver/lower production owners and every public boundary.
- [x] Authenticate the exact source, Surface and resolver identities, lower
  handoffs, Task-258 base/reference profiles, `20/73` ownership, label
  `[0]`, citation `[0,1]`, resolver node 82, and B1/B5A atomic pairing.
- [x] Close bounded B5A `source_drift` while retaining B5B/B5C as separate
  bounded `test_gap` ownership.
- [x] Preserve specifications, existing `.miz`, expectations, sidecars,
  trace status/count/backlinks and credit, public APIs, diagnostics, and all
  semantic outputs.
- [x] Run focused checker `4/4`, runner `5/5`, and preserved B1 runner
  `6/6` tests; record the stable library/test-list inventory.
- [x] Complete separate test-sufficiency and implementation reviews with
  **NO FINDINGS**.
- [x] Complete final source/documentation and bilingual consistency reviews
  with **NO FINDINGS**.
- [x] Pass checker `426/426`, runner `493/493`, full workspace, formatting,
  exact Clippy, five-CLI, count/hash, and diff verification.
- [x] Complete final scope/forbidden-artifact, repository-state, and stash
  verification.
- [x] Pass independent final quality with **NO FINDINGS**, all nine hard
  gates, no cap, and valid `100/100` (`20/20/15/15/10/10/5/5`).
- [x] Stage/inspect and create B5A implementation commit
  `4a79116c1a6f71155e4f366950fee8335b4dc8f1`, verify clean
  post-commit/stash state, and fresh-inventory the next dependency-ready task.

## Checker Task 258B5B Frozen-Contract Documentation Prerequisite

- [x] Record B5A commit
  `4a79116c1a6f71155e4f366950fee8335b4dc8f1` and fresh-inventory B5B.
- [x] Classify the unfrozen contract/API as `design_drift`, mandatory opt-in
  imported-label helper as separate lower `source_drift`, and absent active
  B5B coverage as bounded `test_gap`, with no blocking gap.
- [x] Freeze source/hash, 57-node frontend/resolver, imported `Ref`
  provenance, lower/upper rows, `8/49`, citation API, telemetry, consumers,
  tests, deferrals, baselines, and exit criteria.
- [x] Preserve specification, fixture, expectation, sidecar, trace
  status/count/backlinks/credit, public runner schema, B5C, and semantics.
- [x] Complete specification review with no blocking finding and pass
  crate/workspace, format, Clippy, and five-CLI verification.
- [x] Complete test-contract, source/documentation, and bilingual reviews
  with **NO FINDINGS**.
- [x] Pass final scope/repository/stash gates.
- [x] Pass final quality with **NO FINDINGS**, all nine hard gates PASS, no
  cap, and valid `100/100` (`20/20/15/15/10/10/5/5`).
- [x] Commit only synchronized B5B documentation as `141dc44a` and
  fresh-inventory the mandatory lower-stage prerequisite.

## Checker Task 258B5B Lower-Stage Prerequisite

- [x] Change only `runner/import_fixtures.rs` and the statement test leaf;
  add the opt-in `Ref` helper and two tests in separate commit `46dd9db5`.

## Checker Task 258B5B Upper Implementation

- [x] Implement only the exact seven upper Rust consumers; include required
  synchronized design outputs in the same logical task without expanding
  the code-consumer boundary.
- [x] Authenticate the exact 146-byte source, 57-node/root-56 Surface and
  resolver identities, raw/enriched resolver profiles, all lower/base/
  reference rows, imported theorem provenance, and `8/49` ownership.
- [x] Preserve exact-source-only opt-in, B1/B5A target/debug behavior,
  cross-profile atomicity, immutable clone replay, and empty semantics.
- [x] Pass focused checker `4/4`, runner `7/7` (five upper/two lower), full
  checker `430/430`, runner `500/500`, and current count/hash gates.
- [x] Complete test-sufficiency and repeated implementation reviews with
  **NO FINDINGS** and repair the derived-document `design_drift`.
- [x] Add narrative-only spec-coverage closure while retaining `deferred`,
  `tests = []`, and unchanged trace status/count/backlink/owner/credit.
- [x] Complete final source/documentation consistency with **NO FINDINGS**,
  then pass workspace formatting, exact Clippy, full tests, five CLIs, and
  final count/hash/scope gates.
- [x] Pass independent final quality with **NO FINDINGS**, all nine hard
  gates, no cap, and valid `100/100` (`20/20/15/15/10/10/5/5`).
- [x] Stage/inspect and create B5B upper implementation commit
  `f27d2c9169b08078f00b75c4a57f94e30fa28f59`, verify clean
  post-commit/stash state, and fresh-inventory the next task.

## Checker Task 258B5C Frozen-Contract Documentation Prerequisite

- [x] Freeze the canonical Chapter 15 §15.10 and Chapter 16
  §§16.4.2/16.5.1 proof-label confinement contract and the exact
  inner-to-outer/sibling normal-source transactions.
- [x] Record Medium resolver `source_drift` with potential
  `boundary_violation`, `design_drift`, bounded `test_gap`, and the Low
  deferred nonblocking public-diagnostic `spec_gap`; record no current
  `repo_metadata_conflict`.
- [x] Order four independent commits: synchronized frozen documentation,
  resolver R-032A validated structural Surface-to-resolved arena/map,
  resolver R-032B proof-label source collection, then active B5C
  declaration-symbol fixtures/sidecars/trace/runner assertions.
- [x] Freeze both exact resolver APIs/errors, completion visibility ordinal
  3, general theorem-root scopes, narrow collection boundaries, exact
  provenance paths, and same-block/own-proof/cross-theorem tests.
- [x] Keep this prerequisite documentation-only: no production, `.miz`,
  expectation, sidecar, trace row/status/count/backlink/credit, public schema,
  or semantic change.
- [x] Complete repeated specification, test-contract, source/documentation,
  boundary, and bilingual reviews with **NO FINDINGS**.
- [x] Reproduce unchanged focused/crate/workspace, formatting, Clippy,
  five-CLI, count/hash, forbidden-artifact, repository-state, authority
  no-op, and protected-stash gates.
- [x] Complete independent final quality with **NO FINDINGS**, all nine hard
  gates PASS, no score cap, and valid `100/100`
  (`20/20/15/15/10/10/5/5`).
- [x] Complete task-only staging, the dedicated documentation commit,
  post-commit invariants, and fresh R-032A inventory.
- [x] Freeze the canonical R-032B exact
  `Root -> CompilationUnit -> ItemList -> direct TheoremItem -> direct
  ProofBlock` upper chain, exact-one normal Root/CompilationUnit children,
  direct-normal theorem scanning, positive coverage of every edge, and
  missing/additional/wrong/direct-relocation/`VisibleItem`/mixed-list
  rejection without ordinal, descent, or partial output.
- [x] Freeze full active-runner provenance authentication and independent
  mutation coverage for env/projection/contribution module, namespace, id,
  cardinality, kind, and source id; all corruptions emit only
  `declaration_symbol.label.proof_scope_input`.
- [x] Preserve source-bytes-plus-normal-AST selection, expectation
  non-selection, empty public codes, and the exact 48-file scope.

## mizar-syntax S-026 Dense Node-View Lower Prerequisite

- [x] From clean post-B5C-docs inventory, classify the missing complete
  id-bearing syntax traversal as a High prospective `boundary_violation` and
  the prior existing-API-sufficiency claim as `design_drift`; confirm no
  blocking semantic `spec_gap` or `repo_metadata_conflict`.
- [x] Freeze `SurfaceAst::node_views()` as an immutable dense
  `ExactSizeIterator + DoubleEndedIterator` over every stored node, including
  disconnected, token, recovered, root, and expression-root nodes.
- [x] Forbid public id construction, unsafe/dummy-AST id minting, mutation,
  serialization, semantic identity, raw rowan traversal, parser behavior,
  snapshot, fixture, expectation, trace, specification, or Cargo changes.
- [x] Freeze implementation ownership to syntax `ast.rs` / `ast/tests.rs`,
  exact test obligations, baselines, R-032A as sole current production
  consumer, and no-op `spec_coverage_audit` impact.
- [x] Refine R-032A to store only source/module/arena and validate in exact
  source/module/arena/count/root then per-node
  kind/children/range/recovery/state/key/origin/path order.
- [x] Freeze this documentation task to exactly 45 design files: six paired
  EN/JA families under `mizar-syntax` (plan, AST, bilingual audit, exit,
  source/spec correspondence, TODO), eight paired families under
  `mizar-resolve` (plan, bilingual audit, exit, labels, module boundary,
  resolved AST, source/spec correspondence, TODO), four paired families each
  under `mizar-checker` and `mizar-test` (plan, bilingual audit, module
  boundary, TODO), plus this global TODO. Do not edit
  `doc/design/spec_coverage_audit.md`.
- [x] Complete independent specification, test-contract, and
  source/documentation reviews with **NO FINDINGS**; pass focused and full
  offline verification, exact count/hash/scope/no-op gates, and independent
  final quality with all nine hard gates PASS at valid `100/100`.
- [x] After the dedicated S-026 frozen-contract documentation commit, verify
  clean post-commit/stash invariants, fresh-inventory authority/API, and then
  execute the separate S-026 implementation task.
- [x] Implement only the dense accessor, four-test role/iterator matrix, and
  three public/private-id rustdoc cases; pass syntax `59/8/3`, downstream,
  workspace, CLI, count/hash, and independent specification/test/
  implementation/source-documentation review gates without changing coverage
  credit.
- [x] Freeze the exact implementation scope to 25 files: two Rust files,
  syntax design 12, resolver design 4, checker TODO 2, `mizar-test` design 4,
  and this global TODO.
- [x] Complete independent final quality with **NO FINDINGS**, all nine hard
  gates PASS, no cap, and valid `100/100`
  (`20/20/15/15/10/10/5/5`).
- [x] After the dedicated S-026 implementation commit, fresh-inventory
  resolver R-032A authority, public APIs, baselines, and exact consumers.
- [x] Classify the newly observed mandatory R-026 public-enum owner omission
  as High `design_drift`, with no blocking `spec_gap`, test-intent ambiguity,
  or `repo_metadata_conflict`.
- [x] Freeze a separate synchronized R-032A docs-only scope correction before
  implementation: later ownership is exactly
  `crates/mizar-resolve/src/resolved_ast.rs`,
  `crates/mizar-resolve/src/resolved_ast/tests.rs`, and
  `crates/mizar-resolve/tests/lint_policy.rs`, where the last file may receive
  only the `SurfaceResolvedArenaError` R-026 owning-spec decision entry.
- [x] Freeze this correction to exactly 31 design files: eight paired
  resolver families, four paired checker families, three paired `mizar-test`
  families, and this global TODO. Keep production, test source, fixtures,
  expectations, sidecars, trace rows/status/counts, specifications, Cargo
  metadata, and `doc/design/spec_coverage_audit.md` unchanged.
- [x] Complete docs-only specification/test/source-documentation reviews with
  **NO FINDINGS**, full verification and exact count/hash/no-op gates, and
  independent final quality with all nine hard gates PASS at valid `100/100`
  (`20/20/15/15/10/10/5/5`).
- [x] Complete exact-scope staging, correction commit
  `4184872a4c36b2fedce37d0fb626191270096273`, post-commit repository/stash
  invariants, and fresh inventory; then complete the exact three-Rust-file
  resolver R-032A implementation with synchronized status records.
- [x] Fresh-inventory resolver R-032B and classify the omitted mandatory
  R-026 `ProofLabelSourceCollectionError` owning-spec decision as High
  `design_drift`, with no blocking semantic `spec_gap`, test-intent ambiguity,
  or `repo_metadata_conflict`.
- [x] Freeze a separate synchronized R-032B docs-only scope correction before
  implementation. Later Rust ownership is exactly
  `crates/mizar-resolve/src/labels.rs`,
  `crates/mizar-resolve/src/labels/tests.rs`, and
  `crates/mizar-resolve/tests/lint_policy.rs`; the last file may receive only
  the sole `ProofLabelSourceCollectionError` / `labels.md` R-026 decision.
- [x] Freeze the effective seven-task order through active B5C: S-026 docs,
  S-026 implementation, R-032A lint-policy docs correction, R-032A
  implementation, R-032B lint-policy docs correction, R-032B implementation,
  then active B5C, with fresh inventory between commits.
- [x] Freeze this correction to exactly 31 design files: eight paired
  resolver families, four paired checker families, three paired `mizar-test`
  families, and this global TODO. Keep production/test source, behavior,
  fixtures, expectations, sidecars, trace rows/status/counts/backlinks,
  coverage credit, public diagnostic codes, Cargo/workspace metadata, and
  `doc/design/spec_coverage_audit.md` unchanged.
- [x] Complete repeated specification, test-contract, and
  source/documentation consistency reviews with **NO FINDINGS**.
- [x] Pass docs-only diff/scope, focused and workspace verification, and all
  count/hash preservation gates.
- [x] Complete the final independent read-only quality review with **NO
  FINDINGS**, all nine hard gates PASS, no score cap, and valid `100/100`
  (`20/20/15/15/10/10/5/5`).
- [x] Complete the exact 31-file correction commit
  `f1cf0a5d15f2db51176e9e91a4f5a6447a88ad7a`, verify clean
  post-commit/stash invariants, and fresh-inventory R-032B.
- [x] Complete the exact three-Rust-file resolver R-032B source, fix the
  initial High/Medium and two fresh Medium test gaps plus the Medium
  third-child and unauthorized `Default` / `From` implementation findings,
  complete all final fresh rereviews with **NO FINDINGS**, and pass focused,
  full, CLI, count/hash, and exact 20-file scope gates.
- [x] Complete independent final quality with **NO FINDINGS**, all nine hard
  gates PASS, no score cap, and valid `100/100`
  (`20/20/15/15/10/10/5/5`).
- [x] Complete task-only restaging/cached-diff review, dedicated R-032B commit
  `b3a7e79a6b60db2974e911c69bb56ff5f4609064`, and post-commit
  invariant/fresh inventory.
- [x] Fresh-inventory and continue afterward to active B5C.

## Checker Task 258B5C Active Proof-Label Confinement

- [x] Add the exact two fail fixture/sidecar pairs and two covered trace rows
  derived from Chapters 15 §15.10 and 16 §§16.4.2/16.5.1.
- [x] Consume only unchanged R-032A/R-032B output in the private
  declaration-symbol runner; retain empty public codes and all semantic
  deferrals.
- [x] Close the findings for complete result-field mutation and exact-source/
  mismatched-AST selector coverage.
- [x] Correct exactly four stale metadata assertions from `5` to `7`,
  classified as `test_expectation_drift` and scope `design_drift`.
- [x] Complete repeated no-findings test, implementation, and
  source/documentation reviews plus full verification/count/hash gates.
- [x] Complete independent final quality with **NO FINDINGS**, all nine hard
  gates PASS, no score cap, and valid `100/100`
  (`20/20/15/15/10/10/5/5`).
- [ ] Complete the task-only commit, post-commit invariants, and next-task
  fresh inventory.
