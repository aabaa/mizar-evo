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
| mizar-syntax | Rowan-backed `SurfaceAst`, trivia, recovery, typed views | [x] complete | parked task 21 | [todo](./mizar-syntax/en/todo.md) |
| mizar-parser | Grammar, Pratt parsing, syntax recovery, parse-only corpus | [x] complete through task 45 | parked tasks 46-47 | [todo](./mizar-parser/en/todo.md) |
| mizar-frontend | Source loading and phase 1-3 orchestration | [x] complete | — | [todo](./mizar-frontend/en/todo.md) |
| mizar-resolve | Module graph, namespaces, symbols, labels, signatures | [x] complete through task 29 | step 8 (task 30) | [todo](./mizar-resolve/en/todo.md) |
| mizar-test | Corpus discovery, expectations, staged model, traceability, harness | [~] foundation complete through task 22 plus task 21 soundness vocabulary | step 5 (task 10) | [todo](./mizar-test/en/todo.md) |
| mizar-checker | Type checking, cluster/registration resolution, overload resolution | [x] explicit-payload milestone complete; spec-decision wave complete; step 4 tasks 45-47 complete; step 5 tasks 48 and 50-105 complete | step 5 source-derived bridge slices; task 49 remains dependency-gated | [todo](./mizar-checker/en/todo.md) |
| mizar-core | Elaboration, binder-normalized core logic, control-flow preparation | [x] core/control-flow milestone complete; F7 spec decision complete; step 4 tasks 27-30 complete | step 5 source-derived bridge | [todo](./mizar-core/en/todo.md) |
| mizar-vc | VC IR, VC generation, deterministic pre-ATP discharge | [x] kernel-evidence handoff milestone complete | step 5; tasks 27-29 resolved | [todo](./mizar-vc/en/todo.md) |
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
   payload and the same evidence-query diagnostic, while `non empty object`,
   broader non-`set` heads, imported module AST extraction, attributed-type
   evidence, owner provenance, arguments, and downstream payloads remain on
   extraction/deferred gaps,
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
   task 154 adds the test-first exact three-edge local-mode-chain reserved-
   variable equality contract by combining task 73's real
   `OuterThreeEdgeModeEquality -> MiddleThreeEdgeModeEquality -> InnerThreeEdgeModeEquality -> BaseThreeEdgeModeEquality -> set`
   producer with task 134's equality consumer. The intended slice retains four
   raw outer-mode result/expected inputs, resolves both operands to
   `BindingId(0)` at ordinals 1 and 2, and consumes all four real expansions to
   normalize every role to one terminal-RHS builtin-set identity before two
   inferred variables and one fact/deferred-free checked equality, without
   mode declaration acceptance/inhabitation, equality truth/facts, closure/
   order, or theorem acceptance. The production route, full near-miss/
   corruption matrix, and real frontend/resolver sidecar now guard the active
   105th case,
   task 155 adds the test-first exact three-edge local-object-mode-chain
   reserved-variable equality contract by combining task 73's real
   `OuterThreeEdgeObjectModeEquality -> MiddleThreeEdgeObjectModeEquality -> InnerThreeEdgeObjectModeEquality -> BaseThreeEdgeObjectModeEquality -> object`
   producer with task 135's equality consumer. The intended slice retains four
   raw outer-mode result/expected inputs, resolves both operands to
   `BindingId(0)` at ordinals 1 and 2, and consumes all four real expansions to
   normalize every role to one terminal-RHS builtin-object identity before two
   inferred variables and one fact/deferred-free checked equality, without
   mode declaration acceptance/inhabitation, object/set coercion, equality
   truth/facts, closure/order, or theorem acceptance. The production route,
   full near-miss/corruption matrix, and real frontend/resolver sidecar now
   guard the active 106th case,
   task 156 adds the test-first exact three-edge local-mode-chain reserved-
   variable inequality contract by combining task 73's real
   `OuterThreeEdgeModeInequality -> MiddleThreeEdgeModeInequality -> InnerThreeEdgeModeInequality -> BaseThreeEdgeModeInequality -> set`
   producer with task 136's pre-desugaring inequality consumer. The intended
   slice retains four raw outer-mode result/expected inputs, resolves both
   operands to `BindingId(0)` at ordinals 1 and 2, and consumes all four real
   expansions to normalize every role to one terminal-RHS builtin-set identity
   before two inferred variables and one fact/deferred-free pre-desugaring
   checked inequality, without mode declaration acceptance/inhabitation,
   inequality desugaring, truth/facts, closure/order, or theorem acceptance.
   The production route, full near-miss/corruption matrix, and real frontend/
   resolver sidecar now guard the active 107th case,
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
   formula constant semantic checking beyond task 117's exact constant-kind handoff,
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
   task 120 exact reserved-variable membership, and task 121 exact
   reserved-variable inequality, and task 122 exact reserved-variable type-
   assertion slices,
   recorded facts, imported attribute assertion semantic payloads, imported
   attribute-level non-empty assertion semantic payloads, negated attribute
   admissibility/semantic checking, attribute admissibility/semantic checking,
   set-enumeration result-type/sethood payload extraction beyond task 111,
   `formula_statement`, CoreIr, ControlFlowIr,
   VC, and proof payloads deferred).
This wave has no fixed task list yet: it is promoted slice by slice, opening
new numbered owner tasks as each payload family becomes real. Do not fabricate
semantic payloads; promote corpus rows only with prepared consumers.

1. [ ] AST-wide source-to-checker extraction in `mizar-checker`
   (declarations beyond builtins, attributes, terms, formulas, proof
   skeletons), each slice paired with
   [mizar-test task 10](./mizar-test/en/todo.md) consumer-runner support.
2. [ ] Lower confirmed checker payloads through `mizar-core` into
   source-derived `CoreIr` / `ControlFlowIr` snapshots.
3. [ ] Source-derived VC input families in `mizar-vc` (see
   [source/spec audit](./mizar-vc/en/source_spec_audit.md) for the open
   families: branch/match/range/collection loops, term-only termination,
   Pick non-emptiness, ghost-erasure traces — SCA-005).
4. [ ] [mizar-checker task 49](./mizar-checker/en/todo.md) — audit-corpus
   activation and task-29 record revision (deps: steps 2 and 4, plus runner
   support from this step).

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
| [mizar-parser task 46](./mizar-parser/en/todo.md) — concrete operator declarations | future grammar growth |
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
| SCA-005 | `external_dependency_gap` | Algorithm VC coverage still lacks several source-derived payload families such as branch/match/range/collection loops, term-only termination, Pick non-emptiness, and ghost-erasure traces. | [mizar-vc source/spec audit](./mizar-vc/en/source_spec_audit.md) and step-5 producer integration tasks |
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
