# mizar-parser: Recovery

Status: minimal task-12 recovery plus task-28 nested block-end recovery
implemented, with task-1 module split in place as an internal `recovery`
module; full grammar recovery planned.

## Purpose

This module defines parser synchronization and recovery policy.

## Responsibilities

- synchronize at stable boundaries such as `;`, `end`, top-level item keywords, and EOF;
- emit syntax diagnostics while preserving recoverable syntax structure;
- create `mizar-syntax` recovery nodes instead of inventing semantic facts.

Current behavior:

- missing `end` for block-like keywords is diagnosed at EOF when the parser's
  block stack remains open after matching available `end` tokens, and each
  missing close is represented with an explicit recovered `MissingEnd` node.
  The current stack includes top-level blocks plus algorithm control blocks
  with their own `end`. `for` is opened only for loop-like
  `for <identifier> = ...` / `for <identifier> in ...` token shapes so formula
  quantifiers do not consume block ends, and `else if` is treated as one
  conditional chain rather than a nested block opener;
- missing string literals in synthetic string-required parser contexts are
  diagnosed and represented with an explicit recovered `MissingStringLiteral`
  node;
- a stray `end` that has no matching block opener returns syntax diagnostics
  with `ast = None`.
