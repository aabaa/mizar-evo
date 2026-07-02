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

## Crate Status

| Crate | Workspace crate | Role | Status | TODO |
|---|---:|---|---|---|
| mizar-session | yes | Source identity, source maps, source loading, build snapshots, retention | [x] current milestone complete; no deferred crate-owned item | [todo](./mizar-session/en/todo.md) |
| mizar-lexer | yes | Raw scan, scope skeletons, lexical environments, context-sensitive token disambiguation | [x] current milestone complete; `.miz` lexer companions and selector semantics are downstream-owned | [todo](./mizar-lexer/en/todo.md) |
| mizar-syntax | yes | Rowan-backed `SurfaceAst`, syntax trivia, recovery, typed views, parser-facing syntax vocabulary | [x] current milestone complete; only deferred rustdoc summaries remain | [todo](./mizar-syntax/en/todo.md) |
| mizar-parser | yes | Grammar, Pratt parsing, syntax recovery, parse-only corpus execution | [x] current milestone complete through task 45; deferred operator-declaration follow-up task 46 remains recorded | [todo](./mizar-parser/en/todo.md) |
| mizar-frontend | yes | Source loading and phase 1-3 orchestration across session, lexer, syntax, and parser | [x] current milestone complete; future parser growth may open bounded follow-ups | [todo](./mizar-frontend/en/todo.md) |
| mizar-test | yes | Corpus discovery, expectation sidecars, staged model, traceability, snapshots, harness behavior | [~] implementation exists; formal lint/gap audit, runner validation, snapshots, and reporting remain | [todo](./mizar-test/en/todo.md) |
| mizar-build | yes | Phase 0 workspace planning plus task graph, scheduler, resources, cancellation, failure state, cache seam, artifact commit boundary, and scheduler-selected dispatch | [x] current milestone complete; full real clean/incremental/parallel equivalence remains an integration gap | [todo](./mizar-build/en/todo.md) |
| mizar-lsp | yes | Editor range mapping now; future server, snapshots, diagnostics, metadata, navigation, actions, explanations | [~] range conversion slice exists; specs and server features remain planned | [todo](./mizar-lsp/en/todo.md) |
| mizar-resolve | yes | Module graph, namespaces, symbols, labels, signature collection | [~] non-deferred tasks 1-23 and 25-29 complete; R-024 is ready to resume now that `mizar-artifact` task 5 exists | [todo](./mizar-resolve/en/todo.md) |
| mizar-checker | yes | Type checking, cluster/registration resolution, overload resolution | [x] current explicit-payload semantic milestone complete; source-to-checker extraction and artifact/proof reuse remain external gaps | [todo](./mizar-checker/en/todo.md) |
| mizar-core | yes | Elaboration, binder-normalized core logic, control-flow preparation | [x] current core/control-flow milestone complete; source-derived corpus and downstream consumers remain external gaps | [todo](./mizar-core/en/todo.md) |
| mizar-vc | yes | VC IR, VC generation, deterministic pre-ATP discharge, dependency slices | [x] current VC and kernel-evidence handoff milestone complete; ATP/proof/cache/artifact consumers remain external gaps | [todo](./mizar-vc/en/todo.md) |
| mizar-kernel | yes | Trusted certificate parsing and checking | [x] current formula/substitution evidence and SAT-backed kernel milestone complete; source-derived producer and downstream policy/cache/artifact consumers remain external gaps | [todo](./mizar-kernel/en/todo.md) |
| mizar-atp | yes | ATP encoding, backend execution, portfolio candidates | [x] current candidate-evidence producer milestone complete; real backend extraction and proof/cache/artifact integration remain external gaps | [todo](./mizar-atp/en/todo.md) |
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
| IV-002 | `source_drift` | Cache reuse must fail closed: incomplete dependency data, `uncacheable` outputs, schema/toolchain/policy incompatibility, witness mismatch, or deterministic discharge mismatch force a miss. | [mizar-cache task 20](./mizar-cache/en/todo.md), [mizar-ir tasks 9-10](./mizar-ir/en/todo.md), [mizar-build tasks 18 and 24](./mizar-build/en/todo.md) |
| IV-003 | `source_drift` | Clean sequential, clean parallel, incremental sequential, and incremental parallel builds must agree on proof acceptance, published artifacts, interface hashes, dependency-facing summaries, and canonical diagnostics. | [mizar-build task 24](./mizar-build/en/todo.md), [mizar-test task 14](./mizar-test/en/todo.md), [mizar-driver task 16](./mizar-driver/en/todo.md) |
| IV-004 | `source_drift` | ATP portfolio evidence collection may be parallel, but accepted proof identity and early stop are policy-deterministic, not raw-completion-order driven. | [mizar-atp task 25](./mizar-atp/en/todo.md), [mizar-proof tasks 6-7, 9, and 12-13](./mizar-proof/en/todo.md) |
| IV-005 | `source_drift` | Proof-reuse metadata exported to the cache must include compatible verifier policy plus selected proof witness hash or deterministic discharge hash without upgrading evidence classes. | [mizar-proof task 17](./mizar-proof/en/todo.md), [mizar-cache task 20](./mizar-cache/en/todo.md) |
| IV-006 | `design_drift` (closed) | The corrected labeled `redefine pred label: ...` target is documented and implemented by parser task 36 / syntax task 22. Syntax task 23 closed the stale roadmap/status drift that still described the repair as pending. | [mizar-parser task 36](./mizar-parser/en/todo.md), [mizar-syntax tasks 22-23](./mizar-syntax/en/todo.md) |
| IV-007 | `source_drift` | Snapshot-scoped results must respect `BuildSnapshotId` freshness: obsolete or stale results cannot publish as current, obsolete outputs can be reused only as validated cache inputs, and open-buffer results never become package artifacts. | [mizar-ir tasks 7-10 and 13](./mizar-ir/en/todo.md), [mizar-build tasks 14, 18, and 24](./mizar-build/en/todo.md), [mizar-driver tasks 3, 14, and 16](./mizar-driver/en/todo.md), [mizar-diagnostics tasks 8-9](./mizar-diagnostics/en/todo.md), [mizar-lsp tasks 6-9](./mizar-lsp/en/todo.md) |

Already covered by current frontend work: token/AST cache keys separate active
lexical environment fingerprints, parser lexing plan/filter hashes, and
bundle/source-level language edition. Keep this covered by
[mizar-frontend task 19](./mizar-frontend/en/todo.md) and later source/spec
audits; no new architecture-22 task is needed for that slice.

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

1. **mizar-resolve R-024 resume** - consume the canonical `ModuleSummary`
   schema, writer, validating reader, and compatibility policy now provided by
   `mizar-artifact` task 5. Compare summary-backed and source-backed
   resolution without creating resolver-local artifact formats.
2. **mizar-test foundation cleanup** - run the lint-policy guard and
   source/spec gap audit (tasks 1-2), then harden validation/reporting,
   snapshots, and coverage reporting. The source crate already exists; the TODO
   is the formal gap-closing plan.
3. **Evidence-pipeline integration wave** - after R-024, wire only
   task-scoped owner seams among `mizar-atp`, `mizar-proof`, `mizar-cache`,
   `mizar-artifact`, `mizar-ir`, `mizar-build`, and `mizar-driver`. Keep
   proof policy in `mizar-proof`, cache validation in `mizar-cache`, artifact
   publication in `mizar-artifact`, and registry/orchestration in
   `mizar-driver`.

### Semantic And Proof Layers

`mizar-resolve` has completed phases 4-5 for all non-deferred tasks.
`mizar-artifact` task 5 has landed, so the previously deferred
summary-backed reuse task is now the next resolver implementation task:
[mizar-resolve task 24](./mizar-resolve/en/todo.md).

After the immediate foundation work, proceed bottom-up by phase while pulling
leaf support crates forward when they unblock cross-module work:

1. **mizar-resolve R-024** - summary-backed module reuse against canonical
   `mizar-artifact` schemas. [todo](./mizar-resolve/en/todo.md)
2. **Evidence-pipeline integration** - add narrowly scoped consumer tasks for
   ATP/proof/cache/artifact/build/IR/driver handoffs, using completed owner
   crates rather than placeholder APIs. [mizar-atp todo](./mizar-atp/en/todo.md),
   [mizar-proof todo](./mizar-proof/en/todo.md),
   [mizar-cache todo](./mizar-cache/en/todo.md),
   [mizar-artifact todo](./mizar-artifact/en/todo.md),
   [mizar-ir todo](./mizar-ir/en/todo.md),
   [mizar-build todo](./mizar-build/en/todo.md),
   [mizar-driver todo](./mizar-driver/en/todo.md)
3. **mizar-lsp** - server, snapshot, diagnostics, metadata, navigation,
   actions, and explanation features as diagnostics, driver, artifact, and
   semantic surfaces become consumable. [todo](./mizar-lsp/en/todo.md)
4. **mizar-doc** (phase 16) - documentation rendering and extraction over
   published artifacts. [todo](./mizar-doc/en/todo.md)

Two crates run as cross-cutting strands rather than strict steps:

- **mizar-test** supports every stage from parser onward and should grow
  active/planned gating, snapshots, fail/soundness rules, coverage reporting,
  and incremental/parallel regression metadata with each consumer.
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
- **ModuleSummary reuse timing: ready to resume at mizar-resolve task 24.**
  The first resolver iteration uses the in-memory dependency closure. The
  previous R-024 gate on [mizar-artifact task 5](./mizar-artifact/en/todo.md)
  is now satisfied; summary-backed resolution should consume the canonical
  `ModuleSummary` schema, writer, reader, and version compatibility policy
  without inventing resolver-local artifact formats.
- **Registration activation gating: open.** Local registrations must not affect
  inference until their obligations are accepted by verifier policy. Owned by
  [mizar-checker task 19](./mizar-checker/en/todo.md) and revisited when
  `mizar-vc` / `mizar-proof` land.
- **Certificate schema ownership: open.** Default candidate: `mizar-kernel`
  owns certificate schema types and `mizar-atp` depends on the kernel to
  construct candidates, so the kernel never depends on evidence producers.
  Owned by [mizar-kernel task 4](./mizar-kernel/en/todo.md).
- **Discharge-evidence validation scope: open.** Decide whether `mizar-vc`
  pre-ATP discharge evidence is kernel-replayed or accepted as policy-level
  evidence. Owned by [mizar-proof task 6](./mizar-proof/en/todo.md) with
  `mizar-kernel`, and tracked in [mizar-kernel](./mizar-kernel/en/todo.md) and
  [mizar-vc](./mizar-vc/en/todo.md).
