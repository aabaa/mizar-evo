# mizar-parser: Pratt Parsing

Status: minimal task-11 explicit infix Pratt parsing implemented and task-1
module split in place as an internal `pratt` module; full term/formula
precedence planned.

## Purpose

This module defines the precedence parser for term and formula expressions.

## Responsibilities

- use active-lexicon operator metadata for term-level prefix, postfix, and infix forms;
- use the fixed formula connective table for formula-level precedence;
- parse syntactic shape without performing overload resolution;
- report non-associative chaining and precedence surprises with source-local diagnostics.

## Public Enum Compatibility

`OperatorAssociativity` remains deliberately exhaustive. It is a closed
parser-facing fixity property with the same three meanings as
`mizar-syntax::SurfaceOperatorAssociativity`: left-associative,
right-associative, and non-associative. A future operator-model change that
needs another associativity category must update this design note, parser
matches, syntax payload mapping, and lint-policy expectations in the same
change.
