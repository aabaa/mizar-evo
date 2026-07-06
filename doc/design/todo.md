# Implementation Roadmap (Crate Sequencing)

> Canonical language: English. This is the top-level index for crate-level work
> ordering. Per-crate TODOs carry detailed module checklists and have Japanese
> companions under each crate's `ja/` directory when that companion exists.

This document records the current implementation order across crates. It
complements [README.md](./README.md) (design layout), the pipeline definition in
[architecture/en/00.pipeline_overview.md](./architecture/en/00.pipeline_overview.md),
and the crate ownership map in
[internal/en/07.crate_module_layout.md](./internal/en/07.crate_module_layout.md).

## Status Legend

- [ ] not started
- [~] in progress
- [x] current milestone done

## Guiding Principles

1. **Authority flows from spec and tests.** Crate TODOs and source code refine
   `doc/spec/en/`, executable `.miz` tests, expectation metadata, and
   traceability records; they do not introduce language behavior on their own.
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

## Completion Compass

This roadmap is intended to be sufficient guidance for the verification-focused
Mizar-evo implementation to reach an end-to-end usable state, provided that
remaining `external_dependency_gap`, `deferred`, `test_gap`, and `design_drift`
records are promoted into real owner tasks before they are closed. A gap is not
complete merely because a downstream crate can construct a placeholder. It is
complete only when the owning crate exposes the real producer or consumer seam,
the relevant corpus runner can exercise it where applicable, and the authority
order in [autonomous_crate_development.md](./autonomous_crate_development.md)
still agrees with the resulting behavior.

The current TODO stream should be read as targeting these completion gates:

| Gate | Completion condition | Primary roadmap owners |
|---|---|---|
| Source-to-semantics bridge | Real `.miz` inputs can pass from frontend output through resolver and checker-owned payload extraction into `ResolvedTypedAst`, with active semantic corpus coverage instead of extraction-gap sentinels. The reserve-only builtin declaration slice is active; AST-wide non-builtin declarations, attributes, terms, formulas, proof, and broader checker payload extraction remains open. | `mizar-test`, `mizar-resolve`, `mizar-checker` |
| Core and VC bridge | Checker-derived payloads lower into `CoreIr`, `ControlFlowIr`, and source-derived VC inputs without reconstructing missing source or fabricating registration/proof facts. | `mizar-checker`, `mizar-core`, `mizar-vc`, `mizar-test` |
| Proof and algorithm verification | Source-derived proof and algorithm obligations can flow through VC generation, ATP candidate production, kernel checking, proof policy/status projection, and proof-reuse metadata with active `proof_verification` coverage. | `mizar-vc`, `mizar-atp`, `mizar-kernel`, `mizar-proof`, `mizar-cache`, `mizar-test` |
| Artifact publication | Verified module, registration, proof-witness, and diagnostic projections are emitted through real `mizar-artifact` store/manifest transactions from producer-owned outputs. | `mizar-artifact`, `mizar-ir`, `mizar-driver`, producer crates |
| Build orchestration | Clean, incremental, sequential, and parallel driver/build runs agree on externally visible artifacts, proof statuses, cache decisions, and diagnostics for implemented phases. | `mizar-driver`, `mizar-build`, `mizar-ir`, `mizar-cache`, `mizar-test` |
| User-facing projections | Public diagnostics, LSP features, and documentation rendering consume stable artifacts, diagnostic records, and metadata without owning semantic or proof authority. | `mizar-diagnostics`, `mizar-lsp`, `mizar-doc`, producer crates |

When all non-parked items in those gates are complete and the relevant broad
verification commands pass, the roadmap supports claiming a source-to-artifact
verification pipeline. Algorithm *execution* is a separate claim from algorithm
verification: before claiming executable algorithm runtime support, promote the
currently deferred MVM/code-extraction/backend specification work from
`spec.en.20.algorithm_and_verification` coverage into explicit owner tasks with
tests and artifact/build integration.

If a future task discovers that one of these gates cannot be closed by the
existing crate TODOs, update this roadmap in the same change that records the
new gap. Do not silently close the gate by weakening tests, changing
expectations to match current behavior, or moving trust authority into a
convenient downstream crate.

## Crate Status

| Crate | Workspace crate | Role | Status | TODO |
|---|---:|---|---|---|
| mizar-session | yes | Source identity, source maps, source loading, build snapshots, retention | [x] current milestone complete; no deferred crate-owned item | [todo](./mizar-session/en/todo.md) |
| mizar-lexer | yes | Raw scan, scope skeletons, lexical environments, context-sensitive token disambiguation | [x] current milestone complete; `.miz` lexer companions and selector semantics are downstream-owned | [todo](./mizar-lexer/en/todo.md) |
| mizar-syntax | yes | Rowan-backed `SurfaceAst`, syntax trivia, recovery, typed views, parser-facing syntax vocabulary | [x] current milestone complete; only deferred rustdoc summaries remain | [todo](./mizar-syntax/en/todo.md) |
| mizar-parser | yes | Grammar, Pratt parsing, syntax recovery, parse-only corpus execution | [x] current milestone complete through task 45; deferred operator-declaration follow-up task 46 remains recorded | [todo](./mizar-parser/en/todo.md) |
| mizar-frontend | yes | Source loading and phase 1-3 orchestration across session, lexer, syntax, and parser | [x] current milestone complete; future parser growth may open bounded follow-ups | [todo](./mizar-frontend/en/todo.md) |
| mizar-test | yes | Corpus discovery, expectation sidecars, staged model, traceability, snapshots, harness behavior | [~] foundation complete through task 20 reserve binder-only core context readiness plus post-task-20 resolver R-G007 active runner and SymbolEnv payload assertion increments; 2026-07 kernel-audit tasks 21-22 (soundness vocabulary, corpus-root naming) are pending; broader consumer-runner support remains paced by source-derived semantic/CoreIr/ControlFlowIr/VC/proof bridges | [todo](./mizar-test/en/todo.md) |
| mizar-build | yes | Phase 0 workspace planning plus task graph, scheduler, resources, cancellation, failure state, cache seam, artifact commit boundary, and scheduler-selected dispatch | [x] current milestone complete; full real clean/incremental/parallel equivalence remains an integration gap | [todo](./mizar-build/en/todo.md) |
| mizar-lsp | yes | Editor range mapping now; future server, snapshots, diagnostics, metadata, navigation, actions, explanations | [~] range conversion slice exists; specs and server features remain planned | [todo](./mizar-lsp/en/todo.md) |
| mizar-resolve | yes | Module graph, namespaces, symbols, labels, signature collection | [x] current resolver milestone complete through R-024 and non-deferred tasks 1-29; R-030 remains future diagnostic consumer adoption | [todo](./mizar-resolve/en/todo.md) |
| mizar-checker | yes | Type checking, cluster/registration resolution, overload resolution | [x] current explicit-payload semantic milestone complete; 2026-07 semantic-audit wave 4 (tasks 35-48: spec decisions SSA-001..020 plus checker alignment) is pending; source-to-checker extraction and artifact/proof reuse remain external gaps | [todo](./mizar-checker/en/todo.md) |
| mizar-core | yes | Elaboration, binder-normalized core logic, control-flow preparation | [x] current core/control-flow milestone complete; 2026-07 template-audit follow-ups (tasks 26-30: reduct-view lowering, inhabitation/sethood gating, scheme-actual rules) are pending; source-derived corpus and downstream consumers remain external gaps | [todo](./mizar-core/en/todo.md) |
| mizar-vc | yes | VC IR, VC generation, deterministic pre-ATP discharge, dependency slices | [x] current VC and kernel-evidence handoff milestone complete; 2026-07 kernel-audit producer tasks 27-29 (goal polarity, context identity, import projection) are pending; ATP/proof/cache/artifact consumers remain external gaps | [todo](./mizar-vc/en/todo.md) |
| mizar-kernel | yes | Trusted certificate parsing and checking | [x] current formula/substitution evidence and SAT-backed kernel milestone complete; 2026-07 soundness-audit tasks 30-35 (polarity binding, context identity, import projection, legacy/reduct follow-ups) are pending; source-derived producer and downstream policy/cache/artifact consumers remain external gaps | [todo](./mizar-kernel/en/todo.md) |
| mizar-atp | yes | ATP encoding, backend execution, portfolio candidates | [x] current candidate-evidence producer milestone complete; 2026-07 kernel-audit conformance task 29 is pending; real backend extraction and proof/cache/artifact integration remain external gaps | [todo](./mizar-atp/en/todo.md) |
| mizar-artifact | yes | Artifact schemas, summaries, store, manifest transactions | [~] schemas, store, manifest, hash inputs, and task-23 witness update complete; phase-15 real emission task 17 remains deferred on producer publication integration | [todo](./mizar-artifact/en/todo.md) |
| mizar-doc | no | Documentation rendering and extraction | [ ] planned | [todo](./mizar-doc/en/todo.md) |
| mizar-driver | yes | Build requests, phase registry, CLI/watch/LSP entry points, query orchestration | [x] current driver/session/registry/event milestone complete; production phase adapters and LSP bridge remain external gaps | [todo](./mizar-driver/en/todo.md) |
| mizar-ir | yes | IR storage, snapshot handles, sealed output blobs, projections | [x] current IR storage/cache-adapter/projection/dispatch-input milestone complete; producer publication and LSP seams remain external gaps | [todo](./mizar-ir/en/todo.md) |
| mizar-proof | yes | Proof policy evaluation, status projection, witness selection | [x] current proof-policy/status/witness/reuse metadata milestone complete; cache adoption, artifact witness publication, and ATP integration remain external gaps | [todo](./mizar-proof/en/todo.md) |
| mizar-cache | yes | Cache keys, fingerprints, proof reuse, cluster-db storage | [x] current internal-cache milestone complete; scheduler/IR/publication integration remains external gap work | [todo](./mizar-cache/en/todo.md) |
| mizar-diagnostics | yes | Diagnostic registry, failure records, ordering, rendering | [x] current internal diagnostics milestone complete; consumer adoption remains integration work | [todo](./mizar-diagnostics/en/todo.md) |

## Incremental Verification Contract Inventory

The design contract in
[architecture/en/22.incremental_verification_contract.md](./architecture/en/22.incremental_verification_contract.md)
adds cross-crate implementation work. The table below keeps those deltas visible
before per-crate task execution starts.

| ID | Classification | Contract delta | Owning TODO task |
|---|---|---|---|
| IV-001 | `source_drift` | VC reuse needs cross-edit `ObligationAnchor`, canonical VC fingerprints, local-context fingerprints, and dependency-slice fingerprints; `VcId`, `SourceRange`, and syntax-node ids are not stable reuse identity. | [mizar-core task 18](./mizar-core/en/todo.md), [mizar-resolve tasks 2, 4, and 17-21](./mizar-resolve/en/todo.md), [mizar-vc task 20](./mizar-vc/en/todo.md), [mizar-cache task 20](./mizar-cache/en/todo.md) |
| IV-002 | `source_drift` | Cache reuse must fail closed: incomplete dependency data, `uncacheable` outputs, schema/toolchain/policy incompatibility, witness mismatch, or deterministic discharge mismatch force a miss. Post-audit: reuse records lacking the extended kernel-evidence handoff identity (goal polarity, context-identity payload) also force a miss. | [mizar-cache tasks 20 and 24](./mizar-cache/en/todo.md), [mizar-ir tasks 9-10](./mizar-ir/en/todo.md), [mizar-build tasks 18 and 24](./mizar-build/en/todo.md) |
| IV-003 | `source_drift` | Clean sequential, clean parallel, incremental sequential, and incremental parallel builds must agree on proof acceptance, published artifacts, interface hashes, dependency-facing summaries, and canonical diagnostics. | [mizar-build task 24](./mizar-build/en/todo.md), [mizar-test task 14](./mizar-test/en/todo.md), [mizar-driver task 16](./mizar-driver/en/todo.md) |
| IV-004 | `source_drift` | ATP portfolio evidence collection may be parallel, but accepted proof identity and early stop are policy-deterministic, not raw-completion-order driven. | [mizar-atp task 25](./mizar-atp/en/todo.md), [mizar-proof tasks 6-7, 9, and 12-13](./mizar-proof/en/todo.md) |
| IV-005 | `source_drift` | Proof-reuse metadata exported to the cache must include compatible verifier policy plus selected proof witness hash or deterministic discharge hash without upgrading evidence classes. Post-audit: witness metadata additionally records the accepted evidence's goal polarity, and corrected kernel rejections are never upgraded by policy. | [mizar-proof tasks 17 and 21](./mizar-proof/en/todo.md), [mizar-cache tasks 20 and 24](./mizar-cache/en/todo.md) |
| IV-006 | `design_drift` (closed) | The corrected labeled `redefine pred label: ...` target is documented and implemented by parser task 36 / syntax task 22. Syntax task 23 closed the stale roadmap/status drift that still described the repair as pending. | [mizar-parser task 36](./mizar-parser/en/todo.md), [mizar-syntax tasks 22-23](./mizar-syntax/en/todo.md) |
| IV-007 | `source_drift` | Snapshot-scoped results must respect `BuildSnapshotId` freshness: obsolete or stale results cannot publish as current, obsolete outputs can be reused only as validated cache inputs, and open-buffer results never become package artifacts. | [mizar-ir tasks 7-10 and 13](./mizar-ir/en/todo.md), [mizar-build tasks 14, 18, and 24](./mizar-build/en/todo.md), [mizar-driver tasks 3, 14, and 16](./mizar-driver/en/todo.md), [mizar-diagnostics tasks 8-9](./mizar-diagnostics/en/todo.md), [mizar-lsp tasks 6-9](./mizar-lsp/en/todo.md) |

Already covered by current frontend work: token/AST cache keys separate active
lexical environment fingerprints, parser lexing plan/filter hashes, and
bundle/source-level language edition. Keep this covered by
[mizar-frontend task 19](./mizar-frontend/en/todo.md) and later source/spec
audits; no new architecture-22 task is needed for that slice.

## Specification Coverage Audit Follow-Ups

[spec_coverage_audit.md](./spec_coverage_audit.md) tracks coverage from each
`doc/spec/en/` chapter to implementation-facing design docs. The table below
keeps the non-closed follow-ups visible in roadmap order.

| ID | Classification | Coverage gap | Owning TODO task |
|---|---|---|---|
| SCA-001 | `design_drift` | The top-level design index must stay aligned with this roadmap's workspace-crate statuses. | This docs-only sync updates [README.md](./README.md). Future roadmap sync tasks must re-check it. |
| SCA-002 | `todo` | Spec 24 documentation generation has only architecture/internal boundaries and `mizar-doc` TODOs; focused module specs are still unwritten. | [mizar-doc tasks 2, 4, 6, 9, 11, 13, 16, 18, 21, 23, and 29](./mizar-doc/en/todo.md) |
| SCA-003 | `todo` | `@show_*` and `@eval` have parser/syntax coverage but need end-to-end display/evaluation projection boundaries. | [mizar-lsp task 24](./mizar-lsp/en/todo.md), plus `mizar-doc` and VC producer tasks as they expose data |
| SCA-004 | `external_dependency_gap` | Resolver name/import/label diagnostics remain crate-local/internal until a real public diagnostic adoption task maps them into stable descriptors. | [mizar-resolve task 30](./mizar-resolve/en/todo.md), [mizar-diagnostics consumer adoption](./mizar-diagnostics/en/consumer_adoption_decision.md) |
| SCA-005 | `external_dependency_gap` | Algorithm VC coverage still lacks several source-derived payload families such as branch/match/range/collection loops, term-only termination, Pick non-emptiness, and ghost-erasure traces. | [mizar-vc source/spec audit](./mizar-vc/en/source_spec_audit.md) and future producer integration tasks |
| SCA-006 | `design_drift` | Phase-16 architecture/internal docs still referenced the historical `mizar-extract` split instead of the current `mizar-doc` module names. | This docs-only sync updates architecture 13 and internal 05 EN/JA. |

## July 2026 Audit Follow-Up Inventory

Three pre-implementation audits landed in July 2026. Every finding is bound
to an owner task or a recorded disposition in the owning crate TODO; the
table below is the roadmap-level index. No finding may be closed by weakening
tests or matching expectations to current behavior.

| Audit | Findings | Owning tasks / dispositions |
|---|---|---|
| [mizar-checker semantic_spec_audit.md](./mizar-checker/en/semantic_spec_audit.md) (2026-07-03, commit `707c95be`) | SSA-001 (critical) through SSA-020; 16-fixture rejection corpus | [mizar-checker tasks 35-48](./mizar-checker/en/todo.md) (spec decisions first, then checker alignment); SSA-018 recorded as a diagnostics-wave lint, no task; full disposition table in the checker TODO |
| [mizar-kernel soundness_argument.md](./mizar-kernel/en/soundness_argument.md) (2026-07-03, commit `f75af877`) | F1-F9; 23-case reject-first certificate corpus | [mizar-kernel tasks 30-35](./mizar-kernel/en/todo.md); producer side [mizar-vc tasks 27-29](./mizar-vc/en/todo.md); consumers [mizar-atp task 29](./mizar-atp/en/todo.md), [mizar-proof task 21](./mizar-proof/en/todo.md), [mizar-cache task 24](./mizar-cache/en/todo.md), [mizar-artifact task 24](./mizar-artifact/en/todo.md); harness [mizar-test tasks 21-22](./mizar-test/en/todo.md); F4/F5 resolved inside `f75af877` |
| [mizar-core template_encoding_audit.md](./mizar-core/en/template_encoding_audit.md) (2026-07-05, commit `cef7e109`) | F1-F8; 4-seed template rejection corpus | spec text for F1-F6/F8 patched inside `cef7e109`; [mizar-core tasks 26-30](./mizar-core/en/todo.md) (F7 spec decision plus elaborator implementation); coordination rows in [mizar-checker tasks 36/43](./mizar-checker/en/todo.md) and [mizar-kernel task 35](./mizar-kernel/en/todo.md) |

`mizar-ir`, `mizar-diagnostics`, `mizar-driver`, and `mizar-doc` reviewed the
audits and recorded a no-crate-owned-task note in their TODOs (they carry no
semantic or proof authority). Completed crates
(session/lexer/syntax/parser/frontend/resolve) are untouched: no audit
finding lands in their scope.

## Recommended Order

### Completed Foundation

The current foundation milestone is complete for:

- **mizar-session** - leaf source identity and coordinate layer.
- **mizar-lexer** - lexer-owned raw scan, local/imported lexical environments,
  source-position-aware operator metadata, and token disambiguation.
- **mizar-syntax** - rowan-backed `SurfaceAst`, syntax diagnostics, trivia,
  recovery vocabulary, parser-facing syntax vocabulary through parser task 35,
  the parser-task-36 predicate-label follow-through, and the private AST source
  split follow-up audit.
- **mizar-frontend** - phase 1-3 orchestration, parser-assisted lexing plan,
  parser handoff, cache keys, diagnostics merge, determinism, fuzz, and current
  parser-growth follow-through.

`mizar-parser` has also completed the main grammar-growth run through task 36,
recovery consolidation task 37, `SurfaceAst` snapshot baselines task 38,
determinism task 39, parser-owned fuzz target task 40, frontend passthrough
audit task 41, module-boundary split task 42, source/spec audit task 43,
bilingual documentation sync task 44, and public enum policy task 45. Deferred
operator-declaration follow-up task 46 is recorded but does not block the
current parser hardening close-out.

### Immediate Next Work

Revised 2026-07-06 after the July audit wave and the reserve-bridge task-20
closeout. Steps 1 and 2 come before further pipeline growth because they close
audited soundness holes and settle semantic decisions that later implementation
must not preempt; steps 3-6 are the previous waves, re-paced around them.

1. **Soundness contract closure** - implement the kernel-audit corrections on
   the trusted boundary before any evidence-pipeline integration:
   goal-polarity binding and context-identity verification
   ([mizar-kernel tasks 30-31](./mizar-kernel/en/todo.md)) with their
   producer payloads ([mizar-vc tasks 27-28](./mizar-vc/en/todo.md)), plus
   the harness soundness vocabulary and corpus-root reconciliation
   ([mizar-test tasks 21-22](./mizar-test/en/todo.md)). These have no
   external gaps blocking them and remove certified-unsound acceptance
   paths (audit F1/F2).
2. **Spec-decision wave** - settle the audited semantic decisions as
   docs-plus-corpus tasks before further checker/core semantics:
   [mizar-checker tasks 35-44](./mizar-checker/en/todo.md) (SSA-001
   constructor/extensionality first, then structure identity, overload
   tie-break, functorial `for`, property coherence, and the clarification
   batch) and [mizar-core task 26](./mizar-core/en/todo.md) (template
   inference determinism, coordinated with the tie-break decision). Per
   AGENTS.md these are specification-intent changes and must update
   `doc/spec/en/` + `ja/` together.
3. **Source-derived semantic bridge** - `mizar-test` tasks 16-20 have landed
   active `.miz` source-to-checker coverage for reserve-only builtin
   declaration payloads through checker-owned `BindingEnv`, `DeclarationInput`,
   `TypedAst`, `ResolvedTypedAst`, and binder-only `CoreContext` preparation.
   The post-task-20 safe fallback also promotes one payload-backed resolver
   R-G007 declaration-symbol seed and exact SymbolEnv-derived declaration-symbol
   pass assertions without promoting checker/core/VC payload families. Continue
   widening real source-derived payload extraction through `mizar-checker`, then
   lowering confirmed checker payloads through `mizar-core` and `mizar-vc`. The
   template-audit implementation tasks
   ([mizar-core tasks 27-30](./mizar-core/en/todo.md)) join this wave once
   their step-2 decisions (checker tasks 36/43) land, followed by the
   checker alignment tasks 45-47. Do not fabricate semantic payloads.
4. **Phase-output publication and orchestration slice** - after
   source-derived semantic outputs exist, wire real phase-service and
   publication seams among `mizar-ir`, `mizar-driver`, `mizar-build`, and
   `mizar-artifact`. Prerequisite for artifact task 17, build
   clean/incremental equivalence, and driver multi-phase dispatch; keep
   absent producer outputs classified rather than adding placeholder
   adapters.
5. **Evidence-pipeline integration wave** - after source-derived VCs and
   publication seams exist, wire only task-scoped owner seams among
   `mizar-atp`, `mizar-proof`, `mizar-cache`, and `mizar-artifact`, with
   `mizar-build`, `mizar-ir`, and `mizar-driver` consuming the published
   results. This wave now includes the audit-alignment consumers
   ([mizar-atp task 29](./mizar-atp/en/todo.md),
   [mizar-proof task 21](./mizar-proof/en/todo.md),
   [mizar-cache task 24](./mizar-cache/en/todo.md),
   [mizar-artifact task 24](./mizar-artifact/en/todo.md)) and the remaining
   kernel follow-ups ([tasks 32-35](./mizar-kernel/en/todo.md), including
   the import-projection pair kernel 33 / vc 29). Keep proof policy in
   `mizar-proof`, cache validation in `mizar-cache`, artifact publication
   in `mizar-artifact`, and registry/orchestration in `mizar-driver`.
6. **User-facing consumer wave** - defer broad `mizar-diagnostics`,
   `mizar-lsp`, and `mizar-doc` adoption until the owning producers publish
   stable diagnostics, metadata, artifacts, and semantic indexes. Resolver
   diagnostic adoption R-030 is the first narrow candidate once a real
   consumer seam is available; the SSA-018 `of`/`over` scope-sensitivity
   lint is recorded for this wave.

### Crate TODO Scan Findings

A follow-up scan on 2026-07-06 re-decomposed the crate TODOs against the July
audit wave: the audit follow-up inventory above replaces the pre-audit
ordering assumption that the semantic bridge was the sole next layer — the
soundness contract closure and spec-decision waves now precede or accompany
it. The earlier findings below remain accurate for the non-audit ordering.

A crate TODO scan on 2026-07-02 found no lower-level task that should displace
the `mizar-test` foundation work after R-024. After `mizar-test` tasks 14-15,
that foundation is complete through the architecture-22 metadata baseline, and
the next missing top-level ordering layer is source-derived semantic payloads
before the existing evidence-pipeline integration wave can be useful. The scan
included the canonical English crate TODOs and their Japanese companions; the
companion documents matched these dependency themes and did not introduce a
separate immediate ordering item.

| Finding | Crate TODO evidence | Roadmap disposition |
| ------- | ------------------- | ------------------- |
| Source-derived semantic payloads remain the next bridge after the reserve-only builtin declaration slice. | `mizar-checker` now has active `.miz` coverage for reserve-only builtin `set`/`object` declarations through checker-owned `BindingEnv`, `DeclarationInput`, `TypedAst`, `ResolvedTypedAst`, summary-only `mizar-core` `ResolvedTypedAstSummary::from_ast` readiness, and binder-only `CoreContext` preparation, but broader semantic `.miz` assertions remain deferred behind AST-wide source-to-checker extraction; `mizar-core` still defers source-derived `CoreIr`/`ControlFlowIr` snapshots; `mizar-vc` defers source-derived `proof_verification` families. | Keep Immediate Next Work step 1 focused on AST-wide checker payloads, then CoreIr/ControlFlowIr/VC/proof handoff. |
| Publication/orchestration is distinct from evidence production. | `mizar-ir`, `mizar-driver`, and `mizar-build` keep real phase services, producer outputs, and clean/incremental publication classified as `external_dependency_gap`; `mizar-artifact` task 17 still waits for real producer projections. | Keep as Immediate Next Work step 2 after the semantic bridge. |
| ATP/proof/cache/artifact integration depends on real VCs and publication seams. | `mizar-atp` concrete backend/evidence routes remain deferred; `mizar-kernel` producer/consumer integration waits for evidence producers; `mizar-proof` cache/witness handoffs remain external; `mizar-cache` proof-reuse consumers remain paced by `mizar-vc` and `mizar-proof`; `mizar-artifact` witness publication remains deferred until producer outputs exist. | Keep the evidence-pipeline wave after steps 1-2. |
| User-facing consumers should not lead producer readiness. | `mizar-diagnostics` adoption is deferred to the first real consumer seam; `mizar-lsp` navigation/metadata work is paced by metadata producers; `mizar-doc` phase 16 consumes published artifacts and must not re-run semantic analysis. | Keep LSP/doc as Immediate Next Work step 4 after producer integration. |
| Parser operator declarations and syntax rustdoc summaries remain parked. | `mizar-parser` task 46 and `mizar-syntax` S-021 are explicitly deferred and do not unblock the semantic/evidence waves. | No new immediate step; keep as future parked work. |

### Semantic And Proof Layers

`mizar-resolve` has completed phases 4-5 through R-024 and all non-deferred
tasks 1-29. Summary-backed module reuse now consumes the canonical
`mizar-artifact` `ModuleSummary` contract without adding resolver-owned
artifact formats.

After the `mizar-test` foundation and task-16 builtin source bridge work,
proceed bottom-up by phase while pulling leaf support crates forward when they
unblock cross-module work:

1. **Soundness and spec-decision waves** - close the audited trusted-boundary
   contracts (kernel 30-31, vc 27-28, test 17-18) and settle the audited
   semantic spec decisions (checker 35-44, core 26) per the July 2026 audit
   inventory above.
   [mizar-kernel todo](./mizar-kernel/en/todo.md),
   [mizar-vc todo](./mizar-vc/en/todo.md),
   [mizar-checker todo](./mizar-checker/en/todo.md),
   [mizar-core todo](./mizar-core/en/todo.md),
   [mizar-test todo](./mizar-test/en/todo.md)
2. **Source-derived semantic bridge** - promote real source-derived semantic
   payloads through `mizar-checker`, `mizar-core`, and `mizar-vc`, with
   `mizar-test` runner/stage support expanding beyond the active builtin
   `type_elaboration` slice only when consumer execution is prepared; the
   template-audit elaborator tasks (core 27-30) and checker alignment tasks
   (45-47) join this wave as their decisions land.
   [mizar-checker todo](./mizar-checker/en/todo.md),
   [mizar-core todo](./mizar-core/en/todo.md),
   [mizar-vc todo](./mizar-vc/en/todo.md),
   [mizar-test todo](./mizar-test/en/todo.md)
3. **Phase-output publication and orchestration** - connect real phase services
   and producer publication among `mizar-ir`, `mizar-driver`, `mizar-build`, and
   `mizar-artifact` before treating artifacts as phase-15 consumable output.
   [mizar-ir todo](./mizar-ir/en/todo.md),
   [mizar-driver todo](./mizar-driver/en/todo.md),
   [mizar-build todo](./mizar-build/en/todo.md),
   [mizar-artifact todo](./mizar-artifact/en/todo.md)
4. **Evidence-pipeline integration** - add narrowly scoped consumer tasks for
   ATP/proof/cache/artifact handoffs, using completed owner crates and
   source-derived VCs rather than placeholder APIs.
   [mizar-atp todo](./mizar-atp/en/todo.md),
   [mizar-proof todo](./mizar-proof/en/todo.md),
   [mizar-cache todo](./mizar-cache/en/todo.md),
   [mizar-artifact todo](./mizar-artifact/en/todo.md)
5. **mizar-diagnostics adoption** - migrate the first real producer/consumer
   seam only after its owning crate can emit stable diagnostic records; R-030 is
   the resolver candidate.
   [mizar-diagnostics todo](./mizar-diagnostics/en/todo.md),
   [mizar-resolve todo](./mizar-resolve/en/todo.md)
6. **mizar-lsp** - server, snapshot, diagnostics, metadata, navigation,
   actions, and explanation features as diagnostics, driver, artifact, and
   semantic surfaces become consumable. [todo](./mizar-lsp/en/todo.md)
7. **mizar-doc** (phase 16) - documentation rendering and extraction over
   published artifacts. [todo](./mizar-doc/en/todo.md)

Two crates run as cross-cutting strands rather than strict steps:

- **mizar-test** supports every stage from parser onward. Its foundation is
  complete through task 15, including active/planned gating, snapshots,
  fail/soundness rules, coverage reporting, and incremental/parallel regression
  metadata. Its remaining runner work should be paired with prepared semantic,
  CoreIr, ControlFlowIr, VC, and later evidence consumers before rows or
  fixtures are promoted to active coverage.
  [todo](./mizar-test/en/todo.md)
- **mizar-lsp** keeps range mapping available now, then lands server,
  snapshot, diagnostics, metadata, navigation, actions, and explanation
  features as diagnostics, driver, artifact, and semantic layers become
  available. [todo](./mizar-lsp/en/todo.md)

## Resolved And Open Decisions

- **Lexer span bridging: resolved.** `mizar-lexer` stays decoupled and the
  frontend maps lexer byte spans onto `mizar-session::SourceRange` through
  `mizar-frontend::span_bridge`; the lexer does not adopt session types.
- **Parser-assisted lexing contract: resolved.** `mizar-frontend` precomputes a
  position-sensitive `ParserLexingPlan` over lexical byte ranges and passes only
  `ParserLexContext` values to the lexer. The parser and lexer do not
  interleave, and the lexer never receives arbitrary parser state.
- **Dot-role surface shape and resolver finalization: resolved.** The parser
  resolves dot roles only as far as syntax allows (spec
  [A.2.5](../spec/en/appendix_a.grammar_summary.md)): dotted qualified-name
  heads remain qualified surfaces, while `.` after an already parsed term is a
  selector/update postfix. Scope-dependent selector-versus-namespace separation
  is finalized by [mizar-resolve task 16](./mizar-resolve/en/todo.md) using
  lexical local-term scope, with selector validation left to checker/type
  phases.
- **Resolver module-index seam: resolved.** `mizar-resolve` consumes the
  build-side `ModuleIndexProvider` contract through
  `mizar_resolve::module_index::ModuleIndexInput`. Its
  `WorkspaceStubModuleIndexProvider` is resolver-local test infrastructure; the
  resolver does not rediscover packages, load sources, construct module indexes,
  or parse dependency-summary artifacts.
- **Syntax tree backend: resolved, rowan-backed.** `mizar-syntax` owns a
  rowan-backed `SurfaceAst` storage boundary. Parser grammar code must go
  through the syntax builder/event boundary rather than relying on arena
  indices or raw rowan layout.
- **Parser grammar status: current hardening milestone complete through task
  45.** Main growth is complete through task 36; recovery consolidation is
  complete through task 37; snapshot baselines, determinism, fuzz target,
  frontend passthrough audit, module split, source/spec audit, bilingual
  synchronization, and public-enum policy are complete through tasks 38-45.
  Deferred operator-declaration task 46 remains future grammar growth.
- **Package manifest name spelling: resolved.** Package ids are lowercase
  `snake_case` (`[a-z][a-z0-9]*(?:_[a-z0-9]+)*`), hyphenated names are rejected,
  and no hyphen-to-underscore normalization is performed. Enforcement belongs
  to `mizar-build` planning, not `mizar-session`.
- **Salsa query engine timing: required later.** `salsa` is the target for
  later query/cache orchestration, but not a dependency of `mizar-lexer`,
  `mizar-syntax`, `mizar-parser`, or `mizar-frontend`. The query boundary is
  owned by [mizar-driver tasks 4-5](./mizar-driver/en/todo.md), with
  scheduler/cache integration owned by
  [mizar-build task 18](./mizar-build/en/todo.md).
- **`mizar-diagnostics` adoption timing: deferred at mizar-resolve task 13.**
  The shared diagnostic crate remains part of the target layout
  ([internal 07](./internal/en/07.crate_module_layout.md)), but R-013 keeps
  resolver failures as crate-local/internal records and R-015 keeps name
  diagnostics crate-local/internal until resolver diagnostic code ownership is
  specified. Revisit before the first later user-facing resolver diagnostic
  integration.
- **ModuleSummary reuse timing: resolved at mizar-resolve task 24.**
  The first resolver iteration uses the in-memory dependency closure. R-024 now
  consumes the canonical `mizar-artifact` `ModuleSummary` reader and projects
  validated public summaries into resolver contribution indexes without
  inventing resolver-local artifact formats.
- **Registration activation gating: open.** Local registrations must not affect
  inference until their obligations are accepted by verifier policy. Owned by
  [mizar-checker task 19](./mizar-checker/en/todo.md) and revisited when
  `mizar-vc` / `mizar-proof` land. The July 2026 semantic audit (SSA-006)
  added [mizar-checker task 40](./mizar-checker/en/todo.md) to state the
  asynchronous-acceptance language contract the interim policy approximates.
- **Certificate schema ownership: open.** Default candidate: `mizar-kernel`
  owns certificate schema types and `mizar-atp` depends on the kernel to
  construct candidates, so the kernel never depends on evidence producers.
  Owned by [mizar-kernel task 4](./mizar-kernel/en/todo.md).
- **Discharge-evidence validation scope: open.** Decide whether `mizar-vc`
  pre-ATP discharge evidence is kernel-replayed or accepted as policy-level
  evidence. Owned by [mizar-proof task 6](./mizar-proof/en/todo.md) with
  `mizar-kernel`, and tracked in [mizar-kernel](./mizar-kernel/en/todo.md) and
  [mizar-vc](./mizar-vc/en/todo.md).
