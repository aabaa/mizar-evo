# mizar-parser: Grammar

Status: planned.

## Purpose

This module defines parser entry points and the module/item grammar for Mizar Evo.

## Responsibilities

- consume `TokenStream` and produce `mizar-syntax::SurfaceAst`;
- parse modules, imports, definitions, registrations, statements, proofs, algorithms, annotations, terms, and formulas;
- keep parsing semantic-free: no name resolution, type inference, overload selection, or proof-obligation generation.

