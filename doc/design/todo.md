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
| mizar-syntax | `SurfaceAst`, syntax nodes, trivia, recovery markers | mizar-session | [~] minimal task-12 surface boundary | [README](./mizar-syntax/en/README.md) |
| mizar-parser | Grammar, Pratt parsing, syntax recovery | mizar-session, mizar-syntax | [~] minimal task-12 parser entry/recovery | [README](./mizar-parser/en/README.md) |
| mizar-frontend | Source loading + phase 1-3 orchestration | mizar-session, mizar-lexer, mizar-syntax, mizar-parser | [~] | [todo](./mizar-frontend/en/todo.md) |
| mizar-test | Test corpus + harness | (consumers) | [~] skeleton | — |
| mizar-lsp | Editor integration / range mapping | mizar-session, mizar-lexer | [~] skeleton | — |

## Recommended Order

### Finished: finish **mizar-session**
It is the leaf identity/coordinate layer that every downstream phase references
(`SourceId`, `SourceRange`, `LineMap`, `BuildSnapshotId`). `mizar-lsp` already depends
on it. Module order and remaining work: [mizar-session/en/todo.md](./mizar-session/en/todo.md).

### Now: **mizar-frontend** foundation (pipeline Steps 1-5)
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
`mizar-resolve` → `mizar-checker` → `mizar-core` → `mizar-vc` → `mizar-atp` →
`mizar-kernel` → `mizar-artifact` → `mizar-doc`, plus `mizar-build` for phase 0.
See the affected-modules lists in
[00.pipeline_overview.md](./architecture/en/00.pipeline_overview.md) and
[01.ir_layers.md](./architecture/en/01.ir_layers.md).

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
