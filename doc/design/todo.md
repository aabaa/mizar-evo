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
| mizar-session | Source identity, source maps, build snapshots, retention (foundation) | — | [~] | [todo](./mizar-session/en/todo.md) |
| mizar-lexer | Raw scan + context-sensitive token disambiguation | — | [x] | [todo](./mizar-lexer/en/todo.md) |
| mizar-syntax | `SurfaceAst`, syntax nodes, trivia, recovery markers | mizar-session | [ ] | [README](./mizar-syntax/en/README.md) |
| mizar-parser | Grammar, Pratt parsing, syntax recovery | mizar-lexer, mizar-syntax | [ ] | [README](./mizar-parser/en/README.md) |
| mizar-frontend | Source loading + phase 1-3 orchestration | mizar-session, mizar-lexer, mizar-syntax, mizar-parser | [ ] | [README](./mizar-frontend/en/README.md) |
| mizar-test | Test corpus + harness | (consumers) | [~] skeleton | — |
| mizar-lsp | Editor integration / range mapping | mizar-session, mizar-lexer | [~] skeleton | — |

## Recommended Order

### Now: finish **mizar-session** ← next focus
It is the leaf identity/coordinate layer that every downstream phase references
(`SourceId`, `SourceRange`, `LineMap`, `BuildSnapshotId`). `mizar-lsp` already depends
on it. Module order and remaining work: [mizar-session/en/todo.md](./mizar-session/en/todo.md).

### Next: **mizar-frontend** foundation (phases 1-2)
Source loading orchestration — file I/O, UTF-8 validation, line maps — wiring the
existing `mizar-lexer` helpers to `mizar-session` source identity. Produces
`SourceUnit` / `PreprocessedSource`. Requires `mizar-session` first.
Spec: [architecture/en/02.source_and_frontend.md](./architecture/en/02.source_and_frontend.md).

### Then: **mizar-syntax (AST)** + **mizar-parser** (phase 3)
`SurfaceAst` node definitions, then the parser consuming `TokenStream` to reach an
end-to-end `source → tokens → SurfaceAst` pipeline. First user-visible milestone.
Keep syntax data structures in `mizar-syntax`, grammar and recovery in `mizar-parser`,
and the phase orchestration in `mizar-frontend`.

### After that: semantic & proof layers (phases 4–16)
`mizar-resolve` → `mizar-checker` → `mizar-core` → `mizar-vc` → `mizar-atp` →
`mizar-kernel` → `mizar-artifact` → `mizar-doc`, plus `mizar-build` for phase 0.
See the affected-modules lists in
[00.pipeline_overview.md](./architecture/en/00.pipeline_overview.md) and
[01.ir_layers.md](./architecture/en/01.ir_layers.md).

## Open Decisions

- **Lexer span bridging.** Confirm that `mizar-lexer` stays decoupled and the frontend
  maps lexer spans onto `mizar-session::SourceRange`, vs. the lexer adopting session
  types directly. This gates `SourceId` integration in `mizar-session/source_map`.
- **Parser-assisted lexing contract.** Specify the narrow request object by which
  `mizar-parser` communicates grammar-required string literal positions and symbol
  kind filters to `mizar-lexer` without giving the lexer arbitrary parser state.
