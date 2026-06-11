# Implementation Roadmap (Crate Sequencing)

> Canonical language: English. This is the top-level index for crate-level work
> ordering. Per-crate TODOs carry the detailed module checklists and have Japanese
> companions under each crate's `ja/` directory.

This document gives the high-level order in which crates should be implemented.
It complements [README.md](./README.md) (doc layout) and the pipeline definition in
[architecture/en/00.pipeline_overview.md](./architecture/en/00.pipeline_overview.md).

## Status Legend

- [ ] not started
- [~] in progress
- [x] done

## Guiding Principles

1. **Bottom-up by pipeline phase.** Build phases 0–3 (source → tokens → AST) before
   semantic and proof phases. See the phase table in
   [00.pipeline_overview.md](./architecture/en/00.pipeline_overview.md).
2. **Leaf-first within a layer.** Implement crates with no internal dependencies
   before their consumers, so each crate can be tested in isolation.
3. **Keep the lexer decoupled.** `mizar-lexer` does not depend on `mizar-session`;
   span integration is the frontend's job. Preserve the parser-facing API boundary.
4. **Immutable IR snapshots.** Each phase consumes the previous snapshot and produces
   a new one; design crate APIs around `SourceId` / `BuildSnapshotId` identity.

## Crate Status

| Crate | Role | Depends on (internal) | Status | TODO |
|---|---|---|---|---|
| mizar-session | Source identity, source maps, build snapshots, retention (foundation) | — | [x] | [todo](./mizar-session/en/todo.md) |
| mizar-lexer | Raw scan + context-sensitive token disambiguation | — | [x] | [todo](./mizar-lexer/en/todo.md) |
| mizar-syntax | `SurfaceAst`, syntax nodes, trivia, recovery markers | mizar-session | [~] minimal task-12 surface boundary | [todo](./mizar-syntax/en/todo.md) |
| mizar-parser | Grammar, Pratt parsing, syntax recovery | mizar-session, mizar-syntax | [~] minimal task-12 parser entry/recovery | [todo](./mizar-parser/en/todo.md) |
| mizar-frontend | Source loading + phase 1-3 orchestration | mizar-session, mizar-lexer, mizar-syntax, mizar-parser | [x] | [todo](./mizar-frontend/en/todo.md) |
| mizar-test | Test corpus + harness (validation/planning only, no pipeline deps) | — | [~] discovery, expectations, staged model, traceability | [todo](./mizar-test/en/todo.md) |
| mizar-lsp | Editor protocol bridge, snapshots, range mapping | mizar-session, mizar-lexer (+ mizar-diagnostics, mizar-ir, mizar-driver, mizar-artifact later) | [~] range conversion only | [todo](./mizar-lsp/en/todo.md) |
| mizar-resolve | Module graph, namespaces, symbols, labels, signature collection (phases 4-5) | mizar-session, mizar-syntax (+ mizar-artifact summaries later) | [ ] | [todo](./mizar-resolve/en/todo.md) |
| mizar-checker | Type checking, cluster/registration resolution, overload resolution (phases 6-8) | mizar-session, mizar-resolve | [ ] | [todo](./mizar-checker/en/todo.md) |
| mizar-core | Elaboration, binder-normalized core logic, control-flow preparation (phases 9-10) | mizar-session, mizar-resolve, mizar-checker | [ ] | [todo](./mizar-core/en/todo.md) |
| mizar-vc | `VcIr`, VC generation, pre-ATP discharge, dependency slices (phases 11-12) | mizar-session, mizar-core | [ ] | [todo](./mizar-vc/en/todo.md) |
| mizar-kernel | Trusted certificate parsing and checking (phase 14) | mizar-session, mizar-core (deliberately minimal) | [ ] | [todo](./mizar-kernel/en/todo.md) |
| mizar-atp | ATP encoding, backend execution, portfolio candidates (phase 13) | mizar-session, mizar-core, mizar-vc, mizar-kernel | [ ] | [todo](./mizar-atp/en/todo.md) |
| mizar-artifact | Artifact schemas, store, manifest transactions (phase 15 + early summaries) | mizar-session (leaf; producers depend on it) | [ ] | [todo](./mizar-artifact/en/todo.md) |
| mizar-build | Workspace planning, task graph, scheduler, cancellation (phase 0 + scheduling) | mizar-session (+ mizar-artifact commit boundary) | [ ] | [todo](./mizar-build/en/todo.md) |
| mizar-doc | Documentation rendering + extraction (phase 16) | mizar-session, mizar-artifact | [ ] | [todo](./mizar-doc/en/todo.md) |
| mizar-driver | Build requests, phase registry, CLI/watch/LSP entry points | mizar-session, mizar-build, mizar-ir, mizar-diagnostics (+ pipeline crates via service adapters) | [ ] | [todo](./mizar-driver/en/todo.md) |
| mizar-ir | IR storage, snapshot handles, sealed output blobs | mizar-session (+ mizar-artifact for projection) | [ ] | [todo](./mizar-ir/en/todo.md) |
| mizar-proof | Proof policy evaluation, status projection, witness selection | mizar-session, mizar-kernel, mizar-vc, mizar-atp, mizar-artifact | [ ] | [todo](./mizar-proof/en/todo.md) |
| mizar-cache | Cache keys, fingerprints, proof reuse, cluster-db storage | mizar-session, mizar-artifact | [ ] | [todo](./mizar-cache/en/todo.md) |
| mizar-diagnostics | Diagnostic registry, failure records, ordering, rendering | mizar-session | [ ] | [todo](./mizar-diagnostics/en/todo.md) |

Crate ownership boundaries follow
[internal/en/07.crate_module_layout.md](./internal/en/07.crate_module_layout.md).

## Recommended Order

### Finished: finish **mizar-session**
It is the leaf identity/coordinate layer that every downstream phase references
(`SourceId`, `SourceRange`, `LineMap`, `BuildSnapshotId`). `mizar-lsp` already depends
on it. Module order and remaining work: [mizar-session/en/todo.md](./mizar-session/en/todo.md).

### Finished: **mizar-frontend** foundation (pipeline Steps 1-5)
Source loading orchestration — the coordinate bridge, file I/O via `mizar-session`,
preprocessing coordination, active lexical environment construction, and
tokenization, and parser-seam invocation — wiring the existing `mizar-lexer`
helpers to `mizar-session` source identity and calling the minimal
`mizar-parser` entry point when the real seam is configured. Produces
`SourceUnit` / `PreprocessedSource` / `TokenStream` and, through the real seam,
`SurfaceAst`.
Module specs and the implementation roadmap: [mizar-frontend/en/todo.md](./mizar-frontend/en/todo.md).
Architecture: [architecture/en/02.source_and_frontend.md](./architecture/en/02.source_and_frontend.md).

### Next: harden **mizar-syntax (AST)** + **mizar-parser** (phase 3)
The task-12 boundary now provides a minimal `SurfaceAst`, parser entry point,
Pratt operator metadata, and parser recovery passthrough for frontend seam
integration. Next, expand the syntax node model, grammar coverage, and broader
recovery behavior into the full `source → tokens → SurfaceAst` pipeline. Keep syntax data structures in
`mizar-syntax`, grammar and recovery in `mizar-parser`, and phase orchestration
in `mizar-frontend`.

### After that: semantic & proof layers (phases 4–16)

Per-crate roadmaps exist for every crate below; the order here is the
recommended start order, derived from the internal crate dependencies and
[internal/en/07.crate_module_layout.md](./internal/en/07.crate_module_layout.md).
Bottom-up by phase, with three leaf strands pulled forward because they sit
on the critical path of cross-module work.

1. **mizar-resolve** (phases 4-5) — first consumer of `SurfaceAst`; start
   once parser tasks 5-7 (module/import/export items) land and the initial
   parser/syntax public-enum compatibility policy is recorded, then grow with
   parser grammar coverage. [todo](./mizar-resolve/en/todo.md)
2. **Early leaf strands, in parallel with 1:**
   - **mizar-artifact wave A** — canonical serialization plus
     `ModuleSummary`/`RegistrationSummary` schemas; unblocks summary-backed
     cross-module resolution. [todo](./mizar-artifact/en/todo.md)
   - **mizar-build wave A** — manifests, `BuildPlan`, module index; replaces
     the resolver's workspace-stub module-index provider.
     [todo](./mizar-build/en/todo.md)
   - **mizar-diagnostics** — shared diagnostic records, ready before the
     resolver's adoption gate (see open decisions below).
     [todo](./mizar-diagnostics/en/todo.md)
3. **mizar-checker** (phases 6-8) — three waves: type checking →
   cluster/registration resolution with replayable traces → overload
   resolution. [todo](./mizar-checker/en/todo.md)
4. **mizar-core** (phases 9-10) — elaboration and control-flow preparation;
   its `core_ir`/binder foundations are checker-independent and can start
   alongside 3. [todo](./mizar-core/en/todo.md)
5. **mizar-vc** (phases 11-12) — VC generation, the only `VcId` assigner,
   and deterministic pre-ATP discharge. [todo](./mizar-vc/en/todo.md)
6. **mizar-kernel** (phase 14) — deliberately built *before* `mizar-atp` so
   the certificate contract is owned by its checker; certificate schema,
   trace/substitution checking, cluster replay.
   [todo](./mizar-kernel/en/todo.md)
7. **mizar-atp** (phase 13) — translation, protocol encoders, backend
   runner, portfolio candidates against the kernel-owned schema;
   **mizar-proof** (policy, winner selection, witness store) grows
   alongside. [todo](./mizar-atp/en/todo.md),
   [mizar-proof todo](./mizar-proof/en/todo.md)
8. **mizar-artifact wave B + mizar-build wave B** — store, manifest
   transactions, phase-15 emission; task graph, scheduler, cancellation,
   commit boundary. **mizar-ir** (storage/snapshot handles,
   [todo](./mizar-ir/en/todo.md)), **mizar-cache**
   ([todo](./mizar-cache/en/todo.md)), and **mizar-driver** (entry points,
   [todo](./mizar-driver/en/todo.md)) join here.
9. **mizar-doc** (phase 16) — documentation rendering and extraction over
   published artifacts. [todo](./mizar-doc/en/todo.md)

Two crates run as cross-cutting strands rather than steps:
**mizar-test** supports every step from 1 onward (snapshot and
fail/soundness support ahead of their first consumers,
[todo](./mizar-test/en/todo.md)); **mizar-lsp** retrofits its range spec
now, then lands its server/diagnostics wave with `mizar-diagnostics` and
`mizar-driver` and its metadata features with the semantic layers
([todo](./mizar-lsp/en/todo.md)).

See the phase table in
[00.pipeline_overview.md](./architecture/en/00.pipeline_overview.md) and IR
ownership in [01.ir_layers.md](./architecture/en/01.ir_layers.md).

## Resolved And Open Decisions

- **Lexer span bridging: resolved.** `mizar-lexer` stays decoupled and the
  frontend maps lexer byte spans onto `mizar-session::SourceRange` through
  `mizar-frontend::span_bridge`; the lexer does not adopt session types.
- **Parser-assisted lexing contract: resolved.** `mizar-frontend` precomputes a
  position-sensitive `ParserLexingPlan` over lexical byte ranges and passes only
  `ParserLexContext` values to the lexer. The parser and lexer do not interleave,
  and the lexer never receives arbitrary parser state. The plan covers
  grammar-position string literals and parser-driven user-symbol kind filters,
  including Unicode inside annotation string arguments.
- **Dot-role surface shape: open.** The parser resolves dot roles only as far
  as syntax allows (spec
  [§A.2.5](../spec/en/appendix_a.grammar_summary.md)): selector-versus-namespace
  separation depends on variable scope and is finalized by the resolver, so
  `mizar-syntax` must represent unresolved dot chains syntactically. This
  decision spans `mizar-parser`, `mizar-syntax`, and `mizar-resolve`; it is
  owned by [mizar-parser/en/todo.md](./mizar-parser/en/todo.md) task 10
  together with `mizar-syntax` task 8, and closed by
  [mizar-resolve/en/todo.md](./mizar-resolve/en/todo.md) task 16.
- **`mizar-diagnostics` adoption timing: open.** The shared diagnostic crate
  is part of the target layout
  ([internal 07](./internal/en/07.crate_module_layout.md)); decide whether to
  introduce it when resolver diagnostics begin or keep per-crate diagnostics
  one more layer. Owned by
  [mizar-resolve/en/todo.md](./mizar-resolve/en/todo.md) task 13's gate.
- **ModuleSummary reuse timing: open.** The first resolver iteration resolves
  the in-memory dependency closure; summary-backed resolution needs the
  `mizar-artifact` schema wave first. Owned by
  [mizar-resolve/en/todo.md](./mizar-resolve/en/todo.md) task 24 together
  with [mizar-artifact/en/todo.md](./mizar-artifact/en/todo.md) task 5.
- **Registration activation gating: open.** Local registrations must not
  affect inference until their obligations are accepted by verifier policy;
  an interim policy is needed before phases 11-14 exist. Owned by
  [mizar-checker/en/todo.md](./mizar-checker/en/todo.md) task 19; revisited
  when `mizar-vc`/`mizar-proof` land.
- **Certificate schema ownership: open.** Default candidate: `mizar-kernel`
  owns the certificate schema types and `mizar-atp` depends on the kernel to
  construct candidates, so the kernel never depends on evidence producers.
  Owned by [mizar-kernel/en/todo.md](./mizar-kernel/en/todo.md) task 4.
- **Discharge-evidence validation scope: open.** Whether `mizar-vc` pre-ATP
  discharge evidence is kernel-replayed or accepted as policy-level evidence.
  Owned by [mizar-proof/en/todo.md](./mizar-proof/en/todo.md) task 6 together
  with `mizar-kernel`; also tracked in
  [mizar-kernel/en/todo.md](./mizar-kernel/en/todo.md) and
  [mizar-vc/en/todo.md](./mizar-vc/en/todo.md).
