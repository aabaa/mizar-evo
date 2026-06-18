# Module Specifications: mizar-frontend

> Canonical language: English. Japanese companion: [../ja/README.md](../ja/README.md).

`mizar-frontend` owns the phase 1-3 orchestration modules (the source_and_frontend pipeline Steps 1-5): source loading, source maps,
preprocessing coordination, active lexical environment construction, lexer
invocation, parser-seam invocation, and the combined frontend output.

It does not own source identity, source hashes, or snapshots (`mizar-session`);
raw scanning, comment stripping, lexical environment assembly, or token
disambiguation rules (`mizar-lexer`); or `SurfaceAst` node definitions
(`mizar-syntax`) and grammar, Pratt precedence, and recovery (`mizar-parser`).
Those crates provide the primitives that the frontend coordinates into a
`FrontendOutput`. `StubParserSeam` remains available for source-to-token
coordinator paths and returns `ast = None`; `MizarParserSeam` calls the current
parser/syntax boundary and passes through recoverable parser output.

## Context

- [doc/design/architecture/en/00.pipeline_overview.md](../../architecture/en/00.pipeline_overview.md) - phase boundaries and build snapshots
- [doc/design/architecture/en/02.source_and_frontend.md](../../architecture/en/02.source_and_frontend.md) - the canonical frontend pipeline, interface definitions, recovery, diagnostics, and incrementality
- [doc/design/architecture/en/01.ir_layers.md](../../architecture/en/01.ir_layers.md) - `SourceUnit`, `PreprocessedSource`, `TokenStream`, `SurfaceAst`
- [doc/design/mizar-session/en/README.md](../../mizar-session/en/README.md) - source identity, source maps, and snapshots consumed here
- [doc/design/mizar-lexer/en/README.md](../../mizar-lexer/en/README.md) - preprocessing helpers, raw scan, import pre-scan, lexical environment, scope skeleton, and disambiguator
- [doc/design/mizar-syntax/en/README.md](../../mizar-syntax/en/README.md) - `SurfaceAst` node definitions consumed here
- [doc/design/mizar-parser/en/README.md](../../mizar-parser/en/README.md) - grammar, Pratt parsing, and recovery invoked here

## Index

| Document | Maps To | Description | Status |
|---|---|---|---|
| [00.crate_plan.md](./00.crate_plan.md) | `doc/design/autonomous_crate_development.md` protocol evidence | Retrospective autonomous crate-development plan, responsibility boundary, gap classification, task decomposition, and exit criteria | Implemented |
| [source.md](./source.md) | `crates/mizar-frontend/src/source.rs` | Step 1: `SourceUnit` loading bridging `mizar-session` source identity, line maps, and loading maps | Implemented |
| [preprocess.md](./preprocess.md) | `crates/mizar-frontend/src/preprocess.rs` | Step 2: `PreprocessedSource`, comment/doc-comment separation, annotation preservation, and shallow import pre-scan coordination | Implemented |
| [lexical_env.md](./lexical_env.md) | `crates/mizar-frontend/src/lexical_env.rs` | Step 3: active lexical environment construction from import stubs and dependency lexical summaries | Implemented through task 6 |
| [lexing.md](./lexing.md) | `crates/mizar-frontend/src/lexing.rs` | Step 4: `TokenStream` via recoverable raw scan, scope skeleton, position-sensitive parser lexing plans, and context-sensitive disambiguation | Implemented through task 22 |
| [parsing.md](./parsing.md) | `crates/mizar-frontend/src/parsing.rs` | Step 5: parser-seam invocation, parser-input assembly, position-sensitive string context planning, and `SurfaceAst` handoff | Implemented through task 28 current parser growth |
| [cache_key.md](./cache_key.md) | `crates/mizar-frontend/src/cache_key.rs` | Layered frontend content cache keys exposed through `FrontendOutput.cache_keys`, including parser lexing-plan content keys | Implemented through task 20 |
| [span_bridge.md](./span_bridge.md) | `crates/mizar-frontend/src/span_bridge.rs` | Lexer byte span → `mizar-session` `SourceRange` coordinate bridge | Implemented for task 1 |
| [orchestration.md](./orchestration.md) | `crates/mizar-frontend/src/orchestration.rs` | End-to-end phase 1-3 coordination (pipeline Steps 1-5), parser lexing-plan wiring, diagnostic merge, and `FrontendOutput` | Implemented through task 28 current parser growth |
| [source_spec_correspondence.md](./source_spec_correspondence.md) | `crates/mizar-frontend` specs, sources, and unit tests | Task 16 public API / error variant / task requirement correspondence audit | Implemented |
| [bilingual_documentation_synchronization.md](./bilingual_documentation_synchronization.md) | `doc/design/mizar-frontend/en/` and `doc/design/mizar-frontend/ja/` | Task 17 bilingual API/status/terminology/link/behavior synchronization audit | Implemented |
| [crate_exit_report.md](./crate_exit_report.md) | `doc/design/autonomous_crate_development.md` protocol evidence | Retrospective hard-gate status, quality score, deferred items, verification results, and next-task handoff | Implemented |
| [todo.md](./todo.md) | `crates/mizar-frontend` | Module implementation order, status, and remaining work | Living |

## Crate Boundary

`mizar-frontend` provides source-to-syntax orchestration:

- single-file source loading projected from `mizar-session` `LoadedSource` into
  `SourceUnit`;
- preprocessing coordination producing `PreprocessedSource` (comments, doc
  comments, annotations-in-lexical-text, shallow import stubs);
- active lexical environment construction from shallow imports and dependency
  lexical summaries;
- context-sensitive tokenization producing a `TokenStream` with session
  `SourceRange` spans;
- parser-seam invocation producing an optional AST (`ast = None` under the stub
  seam or unrecoverable real parser input, `SurfaceAst` under recoverable real
  parser input);
- layered frontend content cache keys for `SourceUnit`, `PreprocessedSource`,
  `ActiveLexicalEnvironment`, `TokenStream`, and `SurfaceAst`;
- the lexer-span → session-`SourceRange` coordinate bridge;
- deterministic diagnostic merging into a single `FrontendOutput`.

It must not:

- own source identity, source hashes, or snapshots;
- own raw scanning, comment stripping, or token disambiguation rules;
- own `SurfaceAst` node definitions or parser grammar/recovery logic;
- own cache storage, cache-hit validation, or scheduler task-key composition;
- perform semantic name resolution, type checking, overload selection, cluster
  registration, or proof-obligation generation.
