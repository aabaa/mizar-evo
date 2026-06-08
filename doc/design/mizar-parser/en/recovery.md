# mizar-parser: Recovery

Status: minimal task-12 recovery implemented; full grammar recovery planned.

## Purpose

This module defines parser synchronization and recovery policy.

## Responsibilities

- synchronize at stable boundaries such as `;`, `end`, top-level item keywords, and EOF;
- emit syntax diagnostics while preserving recoverable syntax structure;
- create `mizar-syntax` recovery nodes instead of inventing semantic facts.

Current minimal behavior:

- missing `end` for block-like keywords is diagnosed at EOF when no `end` token
  is present and represented with an explicit recovered `MissingEnd` node;
- missing string literals in synthetic string-required parser contexts are
  diagnosed and represented with an explicit recovered `MissingStringLiteral`
  node;
- the one-token stray `end` stream returns syntax diagnostics with `ast = None`.
