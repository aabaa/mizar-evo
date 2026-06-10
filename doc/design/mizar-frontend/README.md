# mizar-frontend Design

Canonical language: English. Japanese companion: [ja/README.md](./ja/README.md).

This directory contains implementation-facing design notes for the `mizar-frontend` crate.

`mizar-frontend` owns the phase 1-3 orchestration modules (the `02.source_and_frontend.md` pipeline Steps 1-5): source loading, source maps, preprocessing coordination, active lexical environment construction, lexer invocation, parser-seam invocation, and the combined frontend output. It should not own `SurfaceAst` node definitions or parser grammar logic. `StubParserSeam` remains available for source-to-token coordinator paths, while `MizarParserSeam` calls the minimal parser/syntax boundary.

The full module index, crate boundary, and implementation roadmap live in [en/README.md](./en/README.md) and [en/todo.md](./en/todo.md) (Japanese companions under [ja/](./ja/)).

## Module Specs

- [en/span_bridge.md](./en/span_bridge.md) - lexer byte span → `mizar-session` `SourceRange` coordinate bridge
- [en/source.md](./en/source.md) - Step 1: `SourceUnit` loading, UTF-8 validation, source hashes, and line maps
- [en/preprocess.md](./en/preprocess.md) - Step 2: `PreprocessedSource`, comments/doc comments, annotations, and shallow import pre-scan
- [en/lexical_env.md](./en/lexical_env.md) - Step 3: active lexical environment construction
- [en/lexing.md](./en/lexing.md) - Step 4: `TokenStream` via raw scan, scope skeleton, and disambiguation
- [en/parsing.md](./en/parsing.md) - Step 5: parser-seam invocation and `SurfaceAst` handoff
- [en/orchestration.md](./en/orchestration.md) - end-to-end frontend coordination, diagnostic merge, and `FrontendOutput`
- [en/source_spec_correspondence.md](./en/source_spec_correspondence.md) - task 16 source/spec/test correspondence audit
- [en/bilingual_documentation_synchronization.md](./en/bilingual_documentation_synchronization.md) - task 17 bilingual documentation synchronization audit
- [en/todo.md](./en/todo.md) - module implementation order, status, and remaining work
