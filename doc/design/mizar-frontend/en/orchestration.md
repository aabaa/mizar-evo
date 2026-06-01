# mizar-frontend: Orchestration

Status: planned.

## Purpose

This module defines the phase 1-3 coordinator that produces `FrontendOutput`.

## Responsibilities

- coordinate source loading, lexer preprocessing helpers, import pre-scan, active lexical environment construction, token disambiguation, and parser invocation;
- call `mizar-parser` to produce `mizar-syntax::SurfaceAst`;
- merge lexical and syntax diagnostics in deterministic order;
- expose `FrontendOutput` without owning AST node definitions or grammar logic.

