# mizar-frontend Design

Canonical language: English. Japanese companion: [ja/README.md](./ja/README.md).

This directory contains implementation-facing design notes for the `mizar-frontend` crate.

`mizar-frontend` owns phase 1-3 orchestration: source loading, source maps, preprocessing coordination, active lexical environment construction, lexer invocation, parser invocation, and combined frontend output. It should not own `SurfaceAst` node definitions or parser grammar logic.

## Expected Module Specs

- [en/source.md](./en/source.md) - source loading, UTF-8 validation, source hashes, and line maps
- [en/orchestration.md](./en/orchestration.md) - end-to-end frontend coordination and `FrontendOutput`

