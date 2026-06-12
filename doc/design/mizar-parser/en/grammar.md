# mizar-parser: Grammar

Status: minimal task-11 parser entry implemented and task-1 module split in
place as an internal `grammar` module; full module/item grammar planned.

## Purpose

This module defines parser entry points and the module/item grammar for Mizar Evo.

## Responsibilities

- consume parser-facing token transfer objects and produce `mizar-syntax::SurfaceAst`;
- parse modules, imports, definitions, registrations, statements, proofs, algorithms, annotations, terms, and formulas;
- keep parsing semantic-free: no name resolution, type inference, overload selection, or proof-obligation generation.

Current behavior:

- the crate-root public API (`parse`, `ParseRequest`, `ParserToken`,
  `ParseOutput`, and related transfer enums/entries) remains reachable at the
  original `mizar_parser::...` paths;
- `grammar` owns the current parser orchestration and syntax-tree builder
  handoff, while Pratt expression parsing and recovery policy live in sibling
  implementation modules until later tasks promote a fuller parser
  infrastructure.
