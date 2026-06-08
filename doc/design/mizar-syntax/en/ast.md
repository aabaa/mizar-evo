# mizar-syntax: Surface AST

Status: minimal task-12 `SurfaceAst`, nodes, token nodes, recovery nodes, and syntax diagnostics implemented; full AST coverage planned.

## Purpose

This module defines the source-shaped `SurfaceAst` produced by `mizar-parser`.

## Responsibilities

- define `SurfaceAst`, `SurfaceNode`, syntax node ids, and arena ownership;
- preserve source order, source ranges, and recovery nodes;
- represent modules, items, terms, formulas, statements, proofs, algorithms, and annotations;
- avoid resolved symbol ids, inferred types, overload winners, cluster facts, and proof obligations.
